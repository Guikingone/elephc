//! Purpose:
//! Emits `__rt_exit_destructors`, which runs the `__destruct()` of every object still alive when
//! the program calls `exit()`, as php-src's shutdown does before it flushes output.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via
//!   `crate::codegen_support::runtime::objects`.
//! - `exit()`'s lowering, before output buffers are flushed and the process ends.
//!
//! Key details:
//! - LIVE OBJECTS COME FROM THE HANDLE SIDE TABLE. `_obj_handle_index` holds a handle for the
//!   16-byte heap granule of every live object's payload and is cleared when the block is freed,
//!   so a non-zero slot names a live object. Payloads are 8-byte aligned, so the granule's payload
//!   base is the one of its two 8-byte halves `_heap_live_bases` marks.
//! - ORDER. The walk runs from the highest address down, which is newest object first for the
//!   bump allocator: php-src destroys the active frames' locals, then the globals in reverse,
//!   which is the same order for the objects those hold.
//! - `__rt_call_object_destructor` runs each destructor at most once and skips a class without
//!   one. A destructor that frees other objects clears their slots before the walk reaches them.
//! - `_exit_destructors_ran` makes the walk happen once, even when a destructor calls `exit()`.

use crate::codegen_support::abi;
use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits `__rt_shutdown_destructors`, the program end's destructor pass: php-src's
/// `zend_objects_store_call_destructors`, which runs the `__destruct()` of every object still
/// alive in object-handle order (creation order, freed handles reused), once, without freeing it.
/// The objects are released afterwards, by their owners, after RSHUTDOWN.
///
/// The walk is by HANDLE, not by address: a temporary `calloc` buffer maps each handle below
/// `_obj_handle_next` to its granule, filled from one sweep of `_obj_handle_index`, then the
/// handles are visited in ascending order. A slot that no longer holds the handle was freed by an
/// earlier destructor and is skipped. `__rt_call_object_destructor` leaves bit 31 of a destructed
/// object's refcount set (the free path that normally follows never reads it again); the walk
/// clears it, since these objects are released later through their ordinary refcount.
/// Input: none. Output: none; clobbers the caller-saved registers.
pub(crate) fn emit_shutdown_destructors(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: shutdown_destructors ---");
    emitter.label_global("__rt_shutdown_destructors");
    match emitter.target.arch {
        Arch::AArch64 => emit_shutdown_aarch64(emitter),
        Arch::X86_64 => emit_shutdown_x86_64(emitter),
    }
}

/// AArch64 body of `__rt_shutdown_destructors`. Frame: [sp+16] handle count, [sp+24] buffer,
/// [sp+32] cursor, [sp+40] granule, [sp+48] payload.
fn emit_shutdown_aarch64(emitter: &mut Emitter) {
    emitter.instruction("stp x29, x30, [sp, #-64]!");                           // open the walk frame and save the linkage
    emitter.instruction("mov x29, sp");                                         // establish the walk frame
    abi::emit_symbol_address(emitter, "x9", "_exit_destructors_ran");
    emitter.instruction("mov x10, #1");                                         // an exit() from a destructor must not walk again
    emitter.instruction("str x10, [x9]");                                       // publish the mark before any destructor runs
    abi::emit_symbol_address(emitter, "x9", "_obj_handle_next");
    emitter.instruction("ldr x0, [x9]");                                        // handles below this were ever handed out
    emitter.instruction("cmp x0, #1");                                          // was any object ever created?
    emitter.instruction("b.ls __rt_shutdown_destructors_ret");                  // no: nothing to destruct
    emitter.instruction("str x0, [sp, #16]");                                   // keep the handle count across the calls
    emitter.instruction("mov x1, #4");                                          // one 32-bit granule number per handle
    abi::emit_call_label(emitter, &emitter.target.extern_symbol("calloc"));
    emitter.instruction("cbz x0, __rt_shutdown_destructors_ret");               // no memory: leave destruction to the owners' releases
    emitter.instruction("str x0, [sp, #24]");                                   // the handle-to-granule buffer
    // -- map every live handle to its granule (+1, so 0 means none) --
    abi::emit_symbol_address(emitter, "x9", "_heap_off");
    emitter.instruction("ldr x10, [x9]");                                       // bytes of heap handed out so far
    emitter.instruction("lsr x10, x10, #4");                                    // the number of 16-byte granules they span
    abi::emit_symbol_address(emitter, "x11", "_obj_handle_index");
    emitter.instruction("ldr x12, [sp, #16]");                                  // the handle count
    emitter.instruction("mov x13, #0");                                         // the granule cursor
    emitter.label("__rt_shutdown_destructors_map");
    emitter.instruction("cmp x13, x10");                                        // every granule visited?
    emitter.instruction("b.hs __rt_shutdown_destructors_walk");                 // then walk the handles
    emitter.instruction("ldr w14, [x11, x13, lsl #2]");                         // the handle of a live object here, or 0
    emitter.instruction("cbz w14, __rt_shutdown_destructors_map_next");         // no live object in this granule
    emitter.instruction("cmp x14, x12");                                        // a handle the buffer covers?
    emitter.instruction("b.hs __rt_shutdown_destructors_map_next");             // a later one cannot be: skip it defensively
    emitter.instruction("add w15, w13, #1");                                    // the granule number, biased so 0 stays empty
    emitter.instruction("str w15, [x0, x14, lsl #2]");                          // record where this handle lives
    emitter.label("__rt_shutdown_destructors_map_next");
    emitter.instruction("add x13, x13, #1");                                    // the next granule up
    emitter.instruction("b __rt_shutdown_destructors_map");                     // keep mapping
    // -- run the destructors in ascending handle order --
    emitter.label("__rt_shutdown_destructors_walk");
    emitter.instruction("mov x13, #1");                                         // handles start at 1
    emitter.instruction("str x13, [sp, #32]");                                  // the handle cursor
    emitter.label("__rt_shutdown_destructors_loop");
    emitter.instruction("ldr x13, [sp, #32]");                                  // reload the handle cursor
    emitter.instruction("ldr x12, [sp, #16]");                                  // and the handle count
    emitter.instruction("cmp x13, x12");                                        // every handle visited?
    emitter.instruction("b.hs __rt_shutdown_destructors_free");                 // then release the buffer
    emitter.instruction("add x14, x13, #1");                                    // advance the cursor now: the destructor clobbers registers
    emitter.instruction("str x14, [sp, #32]");                                  // publish the advanced cursor
    emitter.instruction("ldr x0, [sp, #24]");                                   // the buffer
    emitter.instruction("ldr w10, [x0, x13, lsl #2]");                          // where this handle lived, +1, or 0
    emitter.instruction("cbz w10, __rt_shutdown_destructors_loop");             // never mapped: no live object holds it
    emitter.instruction("sub x10, x10, #1");                                    // the granule
    abi::emit_symbol_address(emitter, "x11", "_obj_handle_index");
    emitter.instruction("ldr w12, [x11, x10, lsl #2]");                         // the handle that granule holds now
    emitter.instruction("cmp x12, x13");                                        // still this object?
    emitter.instruction("b.ne __rt_shutdown_destructors_loop");                 // freed by an earlier destructor
    emitter.instruction("str x10, [sp, #40]");                                  // keep the granule across the call
    emitter.instruction("lsl x13, x10, #4");                                    // the granule's first payload offset
    abi::emit_symbol_address(emitter, "x15", "_heap_live_bases");
    emitter.instruction("lsr x14, x13, #3");                                    // its 8-byte granule bit
    emitter.instruction("lsr x16, x14, #3");                                    // the byte holding that bit
    emitter.instruction("ldrb w17, [x15, x16]");                                // load the live-base byte
    emitter.instruction("and x14, x14, #7");                                    // the bit within the byte
    emitter.instruction("lsr w17, w17, w14");                                   // move it to bit 0
    emitter.instruction("tbnz w17, #0, __rt_shutdown_destructors_found");       // the payload starts at the granule
    emitter.instruction("add x13, x13, #8");                                    // else it starts 8 bytes in
    emitter.instruction("lsr x14, x13, #3");                                    // its 8-byte granule bit
    emitter.instruction("lsr x16, x14, #3");                                    // the byte holding that bit
    emitter.instruction("ldrb w17, [x15, x16]");                                // load the live-base byte
    emitter.instruction("and x14, x14, #7");                                    // the bit within the byte
    emitter.instruction("lsr w17, w17, w14");                                   // move it to bit 0
    emitter.instruction("tbz w17, #0, __rt_shutdown_destructors_loop");         // no payload base here after all
    emitter.label("__rt_shutdown_destructors_found");
    abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("add x0, x9, x13");                                     // the live object's payload
    emitter.instruction("str x0, [sp, #48]");                                   // keep it across the call
    emitter.instruction("bl __rt_call_object_destructor");                      // run its __destruct() once, if its class has one
    emitter.instruction("ldr x10, [sp, #40]");                                  // the granule
    abi::emit_symbol_address(emitter, "x11", "_obj_handle_index");
    emitter.instruction("ldr w12, [x11, x10, lsl #2]");                         // the handle it holds after the destructor
    emitter.instruction("ldr x13, [sp, #32]");                                  // the advanced cursor
    emitter.instruction("sub x13, x13, #1");                                    // this object's handle
    emitter.instruction("cmp x12, x13");                                        // still alive?
    emitter.instruction("b.ne __rt_shutdown_destructors_loop");                 // the destructor released it
    emitter.instruction("ldr x0, [sp, #48]");                                   // the payload
    emitter.instruction("ldr w9, [x0, #-12]");                                  // its refcount, with the in-destructor bit
    emitter.instruction("and w9, w9, #0x7fffffff");                             // clear the bit: its owners release it later
    emitter.instruction("str w9, [x0, #-12]");                                  // publish the plain refcount
    emitter.instruction("b __rt_shutdown_destructors_loop");                    // the next handle
    emitter.label("__rt_shutdown_destructors_free");
    emitter.instruction("ldr x0, [sp, #24]");                                   // the buffer
    abi::emit_call_label(emitter, &emitter.target.extern_symbol("free"));
    emitter.label("__rt_shutdown_destructors_ret");
    emitter.instruction("ldp x29, x30, [sp], #64");                             // close the walk frame
    emitter.instruction("ret");                                                 // back to the program epilogue
}

/// x86_64 body of `__rt_shutdown_destructors`. Frame: [rbp-8] handle count, [rbp-16] buffer,
/// [rbp-24] cursor, [rbp-32] granule, [rbp-40] payload.
fn emit_shutdown_x86_64(emitter: &mut Emitter) {
    emitter.instruction("push rbp");                                            // save the caller frame
    emitter.instruction("mov rbp, rsp");                                        // establish the walk frame
    emitter.instruction("sub rsp, 48");                                         // five slots, keeping 16-byte call alignment
    abi::emit_symbol_address(emitter, "r10", "_exit_destructors_ran");
    emitter.instruction("mov QWORD PTR [r10], 1");                              // an exit() from a destructor must not walk again
    abi::emit_symbol_address(emitter, "r10", "_obj_handle_next");
    emitter.instruction("mov rdi, QWORD PTR [r10]");                            // handles below this were ever handed out
    emitter.instruction("cmp rdi, 1");                                          // was any object ever created?
    emitter.instruction("jbe __rt_shutdown_destructors_ret");                   // no: nothing to destruct
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // keep the handle count across the calls
    emitter.instruction("mov esi, 4");                                          // one 32-bit granule number per handle
    abi::emit_call_label(emitter, &emitter.target.extern_symbol("calloc"));
    emitter.instruction("test rax, rax");                                       // no memory?
    emitter.instruction("jz __rt_shutdown_destructors_ret");                    // leave destruction to the owners' releases
    emitter.instruction("mov QWORD PTR [rbp - 16], rax");                       // the handle-to-granule buffer
    // -- map every live handle to its granule (+1, so 0 means none) --
    abi::emit_symbol_address(emitter, "r10", "_heap_off");
    emitter.instruction("mov rcx, QWORD PTR [r10]");                            // bytes of heap handed out so far
    emitter.instruction("shr rcx, 4");                                          // the number of 16-byte granules they span
    abi::emit_symbol_address(emitter, "r11", "_obj_handle_index");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // the handle count
    emitter.instruction("xor edx, edx");                                        // the granule cursor
    emitter.label("__rt_shutdown_destructors_map");
    emitter.instruction("cmp rdx, rcx");                                        // every granule visited?
    emitter.instruction("jae __rt_shutdown_destructors_walk");                  // then walk the handles
    emitter.instruction("mov r9d, DWORD PTR [r11 + rdx * 4]");                  // the handle of a live object here, or 0
    emitter.instruction("test r9d, r9d");                                       // a live object in this granule?
    emitter.instruction("jz __rt_shutdown_destructors_map_next");               // no
    emitter.instruction("cmp r9, r8");                                          // a handle the buffer covers?
    emitter.instruction("jae __rt_shutdown_destructors_map_next");              // a later one cannot be: skip it defensively
    emitter.instruction("lea r10d, [edx + 1]");                                 // the granule number, biased so 0 stays empty
    emitter.instruction("mov DWORD PTR [rax + r9 * 4], r10d");                  // record where this handle lives
    emitter.label("__rt_shutdown_destructors_map_next");
    emitter.instruction("add rdx, 1");                                          // the next granule up
    emitter.instruction("jmp __rt_shutdown_destructors_map");                   // keep mapping
    // -- run the destructors in ascending handle order --
    emitter.label("__rt_shutdown_destructors_walk");
    emitter.instruction("mov QWORD PTR [rbp - 24], 1");                         // handles start at 1
    emitter.label("__rt_shutdown_destructors_loop");
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // reload the handle cursor
    emitter.instruction("cmp rdx, QWORD PTR [rbp - 8]");                        // every handle visited?
    emitter.instruction("jae __rt_shutdown_destructors_free");                  // then release the buffer
    emitter.instruction("lea r8, [rdx + 1]");                                   // advance the cursor now: the destructor clobbers registers
    emitter.instruction("mov QWORD PTR [rbp - 24], r8");                        // publish the advanced cursor
    emitter.instruction("mov rax, QWORD PTR [rbp - 16]");                       // the buffer
    emitter.instruction("mov ecx, DWORD PTR [rax + rdx * 4]");                  // where this handle lived, +1, or 0
    emitter.instruction("test ecx, ecx");                                       // ever mapped?
    emitter.instruction("jz __rt_shutdown_destructors_loop");                   // no live object holds it
    emitter.instruction("sub rcx, 1");                                          // the granule
    abi::emit_symbol_address(emitter, "r10", "_obj_handle_index");
    emitter.instruction("mov r9d, DWORD PTR [r10 + rcx * 4]");                  // the handle that granule holds now
    emitter.instruction("cmp r9, rdx");                                         // still this object?
    emitter.instruction("jne __rt_shutdown_destructors_loop");                  // freed by an earlier destructor
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // keep the granule across the call
    emitter.instruction("shl rcx, 4");                                          // the granule's first payload offset
    abi::emit_symbol_address(emitter, "r10", "_heap_live_bases");
    emitter.instruction("mov rdx, rcx");                                        // the offset
    emitter.instruction("shr rdx, 3");                                          // its 8-byte granule bit
    emitter.instruction("mov r8, rdx");                                         // the bit index
    emitter.instruction("shr r8, 3");                                           // the byte holding that bit
    emitter.instruction("movzx r9d, BYTE PTR [r10 + r8]");                      // load the live-base byte
    emitter.instruction("and edx, 7");                                          // the bit within the byte
    emitter.instruction("bt r9d, edx");                                         // is it set?
    emitter.instruction("jc __rt_shutdown_destructors_found");                  // the payload starts at the granule
    emitter.instruction("add rcx, 8");                                          // else it starts 8 bytes in
    emitter.instruction("mov rdx, rcx");                                        // the offset
    emitter.instruction("shr rdx, 3");                                          // its 8-byte granule bit
    emitter.instruction("mov r8, rdx");                                         // the bit index
    emitter.instruction("shr r8, 3");                                           // the byte holding that bit
    emitter.instruction("movzx r9d, BYTE PTR [r10 + r8]");                      // load the live-base byte
    emitter.instruction("and edx, 7");                                          // the bit within the byte
    emitter.instruction("bt r9d, edx");                                         // is it set?
    emitter.instruction("jnc __rt_shutdown_destructors_loop");                  // no payload base here after all
    emitter.label("__rt_shutdown_destructors_found");
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea rdi, [r10 + rcx]");                                // the live object's payload
    emitter.instruction("mov QWORD PTR [rbp - 40], rdi");                       // keep it across the call
    emitter.instruction("call __rt_call_object_destructor");                    // run its __destruct() once, if its class has one
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // the granule
    abi::emit_symbol_address(emitter, "r10", "_obj_handle_index");
    emitter.instruction("mov r9d, DWORD PTR [r10 + rcx * 4]");                  // the handle it holds after the destructor
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // the advanced cursor
    emitter.instruction("sub rdx, 1");                                          // this object's handle
    emitter.instruction("cmp r9, rdx");                                         // still alive?
    emitter.instruction("jne __rt_shutdown_destructors_loop");                  // the destructor released it
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // the payload
    emitter.instruction("and DWORD PTR [rdi - 12], 0x7fffffff");                // clear the in-destructor bit: its owners release it later
    emitter.instruction("jmp __rt_shutdown_destructors_loop");                  // the next handle
    emitter.label("__rt_shutdown_destructors_free");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 16]");                       // the buffer
    abi::emit_call_label(emitter, &emitter.target.extern_symbol("free"));
    emitter.label("__rt_shutdown_destructors_ret");
    emitter.instruction("mov rsp, rbp");                                        // drop the frame
    emitter.instruction("pop rbp");                                             // restore the caller frame
    emitter.instruction("ret");                                                 // back to the program epilogue
}

/// Emits `__rt_exit_destructors` for the active target. Input: none. Output: none; clobbers the
/// caller-saved registers.
pub(crate) fn emit_exit_destructors(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: exit_destructors ---");
    emitter.label_global("__rt_exit_destructors");
    match emitter.target.arch {
        Arch::AArch64 => emit_aarch64(emitter),
        Arch::X86_64 => emit_x86_64(emitter),
    }
}

/// AArch64 body of `__rt_exit_destructors`.
fn emit_aarch64(emitter: &mut Emitter) {
    emitter.instruction("stp x29, x30, [sp, #-32]!");                           // open the walk frame and save the linkage
    emitter.instruction("mov x29, sp");                                         // establish the walk frame
    abi::emit_symbol_address(emitter, "x9", "_exit_destructors_ran");
    emitter.instruction("ldr x10, [x9]");                                       // did an earlier exit() already run them?
    emitter.instruction("cbnz x10, __rt_exit_destructors_done");                // run them once
    emitter.instruction("mov x10, #1");                                         // mark the walk as started
    emitter.instruction("str x10, [x9]");                                       // publish the mark before any destructor runs
    abi::emit_symbol_address(emitter, "x9", "_heap_off");
    emitter.instruction("ldr x10, [x9]");                                       // bytes of heap handed out so far
    emitter.instruction("lsr x10, x10, #4");                                    // the number of 16-byte granules they span
    emitter.instruction("str x10, [sp, #16]");                                  // the walk's cursor, from the top down
    emitter.label("__rt_exit_destructors_loop");
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the cursor
    emitter.instruction("cbz x10, __rt_exit_destructors_done");                 // every granule visited
    emitter.instruction("sub x10, x10, #1");                                    // the next granule down
    emitter.instruction("str x10, [sp, #16]");                                  // save the cursor across the destructor call
    abi::emit_symbol_address(emitter, "x11", "_obj_handle_index");
    emitter.instruction("ldr w12, [x11, x10, lsl #2]");                         // the handle of a live object here, or 0
    emitter.instruction("cbz w12, __rt_exit_destructors_loop");                 // no live object in this granule
    emitter.instruction("lsl x13, x10, #4");                                    // the granule's first payload offset
    abi::emit_symbol_address(emitter, "x15", "_heap_live_bases");
    emitter.instruction("lsr x14, x13, #3");                                    // its 8-byte granule bit
    emitter.instruction("lsr x16, x14, #3");                                    // the byte holding that bit
    emitter.instruction("ldrb w17, [x15, x16]");                                // load the live-base byte
    emitter.instruction("and x14, x14, #7");                                    // the bit within the byte
    emitter.instruction("lsr w17, w17, w14");                                   // move it to bit 0
    emitter.instruction("tbnz w17, #0, __rt_exit_destructors_found");           // the payload starts at the granule
    emitter.instruction("add x13, x13, #8");                                    // else it starts 8 bytes in
    emitter.instruction("lsr x14, x13, #3");                                    // its 8-byte granule bit
    emitter.instruction("lsr x16, x14, #3");                                    // the byte holding that bit
    emitter.instruction("ldrb w17, [x15, x16]");                                // load the live-base byte
    emitter.instruction("and x14, x14, #7");                                    // the bit within the byte
    emitter.instruction("lsr w17, w17, w14");                                   // move it to bit 0
    emitter.instruction("tbz w17, #0, __rt_exit_destructors_loop");             // no payload base here after all
    emitter.label("__rt_exit_destructors_found");
    abi::emit_symbol_address(emitter, "x9", "_heap_buf");
    emitter.instruction("add x0, x9, x13");                                     // the live object's payload
    emitter.instruction("bl __rt_call_object_destructor");                      // run its __destruct() once, if its class has one
    emitter.instruction("b __rt_exit_destructors_loop");                        // continue down the heap
    emitter.label("__rt_exit_destructors_done");
    emitter.instruction("ldp x29, x30, [sp], #32");                             // close the walk frame
    emitter.instruction("ret");                                                 // back to exit()
}

/// x86_64 body of `__rt_exit_destructors`.
fn emit_x86_64(emitter: &mut Emitter) {
    emitter.instruction("push rbp");                                            // save the caller frame
    emitter.instruction("mov rbp, rsp");                                        // establish the walk frame
    emitter.instruction("sub rsp, 16");                                         // the cursor slot, keeping 16-byte alignment
    abi::emit_symbol_address(emitter, "r10", "_exit_destructors_ran");
    emitter.instruction("cmp QWORD PTR [r10], 0");                              // did an earlier exit() already run them?
    emitter.instruction("jne __rt_exit_destructors_done");                      // run them once
    emitter.instruction("mov QWORD PTR [r10], 1");                              // mark the walk as started
    abi::emit_symbol_address(emitter, "r10", "_heap_off");
    emitter.instruction("mov rax, QWORD PTR [r10]");                            // bytes of heap handed out so far
    emitter.instruction("shr rax, 4");                                          // the number of 16-byte granules they span
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // the walk's cursor, from the top down
    emitter.label("__rt_exit_destructors_loop");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the cursor
    emitter.instruction("test rax, rax");                                       // every granule visited?
    emitter.instruction("jz __rt_exit_destructors_done");                       // done
    emitter.instruction("sub rax, 1");                                          // the next granule down
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the cursor across the destructor call
    abi::emit_symbol_address(emitter, "r10", "_obj_handle_index");
    emitter.instruction("mov r11d, DWORD PTR [r10 + rax * 4]");                 // the handle of a live object here, or 0
    emitter.instruction("test r11d, r11d");                                     // a live object in this granule?
    emitter.instruction("jz __rt_exit_destructors_loop");                       // no
    emitter.instruction("mov rcx, rax");                                        // the granule index
    emitter.instruction("shl rcx, 4");                                          // the granule's first payload offset
    abi::emit_symbol_address(emitter, "r10", "_heap_live_bases");
    emitter.instruction("mov rdx, rcx");                                        // the offset
    emitter.instruction("shr rdx, 3");                                          // its 8-byte granule bit
    emitter.instruction("mov r8, rdx");                                         // the bit index
    emitter.instruction("shr r8, 3");                                           // the byte holding that bit
    emitter.instruction("movzx r9d, BYTE PTR [r10 + r8]");                      // load the live-base byte
    emitter.instruction("and edx, 7");                                          // the bit within the byte
    emitter.instruction("bt r9d, edx");                                         // is it set?
    emitter.instruction("jc __rt_exit_destructors_found");                      // the payload starts at the granule
    emitter.instruction("add rcx, 8");                                          // else it starts 8 bytes in
    emitter.instruction("mov rdx, rcx");                                        // the offset
    emitter.instruction("shr rdx, 3");                                          // its 8-byte granule bit
    emitter.instruction("mov r8, rdx");                                         // the bit index
    emitter.instruction("shr r8, 3");                                           // the byte holding that bit
    emitter.instruction("movzx r9d, BYTE PTR [r10 + r8]");                      // load the live-base byte
    emitter.instruction("and edx, 7");                                          // the bit within the byte
    emitter.instruction("bt r9d, edx");                                         // is it set?
    emitter.instruction("jnc __rt_exit_destructors_loop");                      // no payload base here after all
    emitter.label("__rt_exit_destructors_found");
    abi::emit_symbol_address(emitter, "r10", "_heap_buf");
    emitter.instruction("lea rdi, [r10 + rcx]");                                // the live object's payload
    emitter.instruction("call __rt_call_object_destructor");                    // run its __destruct() once, if its class has one
    emitter.instruction("jmp __rt_exit_destructors_loop");                      // continue down the heap
    emitter.label("__rt_exit_destructors_done");
    emitter.instruction("mov rsp, rbp");                                        // drop the cursor slot
    emitter.instruction("pop rbp");                                             // restore the caller frame
    emitter.instruction("ret");                                                 // back to exit()
}
