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
