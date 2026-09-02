//! Purpose:
//! Emits the `__rt_cstr`, `__rt_cstr_loop` runtime helper assembly for C-string conversion scratch storage.
//! Keeps PHP filesystem/resource behavior, libc calls, and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::io`.
//!
//! Key details:
//! - I/O helpers bridge PHP strings, resources, descriptors, and libc calls while returning runtime arrays or pointer/length strings.

use crate::codegen_support::{abi, emit::Emitter, platform::Arch};

/// cstr: convert an elephc string (x1=ptr, x2=len) to a null-terminated C string.
/// Uses _cstr_buf (4096 bytes) as a fast scratch path and a runtime-heap replacement for
/// larger strings.
/// Input:  x1=ptr, x2=len
/// Output: x0=pointer to null-terminated string in _cstr_buf
///
/// cstr2: same but uses _cstr_buf2 for an independent second C-string path.
/// Input:  x1=ptr, x2=len
/// Output: x0=pointer to null-terminated string in _cstr_buf2
pub fn emit_cstr(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_cstr_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: cstr ---");
    emitter.label_global("__rt_cstr");

    // The fixed scratch buffer is only a fast path. PHP strings are not bounded by PATH_MAX:
    // regex subjects and arbitrary values supplied to libc-facing helpers may be much larger.
    emitter.instruction("cmp x2, #4095");                                     // reserve one byte for the C terminator
    emitter.instruction("b.hi __rt_cstr_dynamic");                            // grow beyond the fixed 4 KiB scratch buffer

    // -- load destination buffer address --
    abi::emit_symbol_address(emitter, "x9", "_cstr_buf");                       // load page address of cstr scratch buffer

    // -- copy bytes from source to buffer --
    emitter.instruction("mov x10, x9");                                         // save buffer start for return value
    emitter.instruction("mov x11, x2");                                         // copy length as loop counter
    emitter.label("__rt_cstr_loop");
    emitter.instruction("cbz x11, __rt_cstr_null");                             // if no bytes remain, append null terminator
    emitter.instruction("ldrb w12, [x1], #1");                                  // load byte from source, advance source ptr
    emitter.instruction("strb w12, [x9], #1");                                  // store byte to buffer, advance buffer ptr
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining byte count
    emitter.instruction("b __rt_cstr_loop");                                    // continue copying

    // -- append null terminator and return --
    emitter.label("__rt_cstr_null");
    emitter.instruction("strb wzr, [x9]");                                      // write null terminator after last byte
    emitter.instruction("mov x0, x10");                                         // return pointer to null-terminated string
    emitter.instruction("ret");                                                 // return to caller

    // -- dynamically sized C string --
    emitter.label("__rt_cstr_dynamic");
    emitter.instruction("sub sp, sp, #48");                                    // reserve saved input, replacement pointer, and aligned frame storage
    emitter.instruction("add x9, sp, #32");                                    // locate the saved frame pair beyond the temporary slots
    emitter.instruction("stp x29, x30, [x9]");                                 // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #32");                                   // establish the helper frame
    emitter.instruction("stp x1, x2, [sp, #0]");                               // retain source bytes and length across allocation and release calls
    emitter.instruction("adds x0, x2, #1");                                    // request source bytes plus a trailing C terminator
    emitter.instruction("b.cs __rt_heap_exhausted");                           // an impossible length must fail before size arithmetic wraps
    emitter.instruction("bl __rt_heap_alloc");                                 // allocate a runtime-owned replacement buffer
    emitter.instruction("str x0, [sp, #16]");                                  // save the new buffer while replacing the retained old one
    abi::emit_symbol_address(emitter, "x9", "_cstr_buf_dynamic");              // locate the persistent dynamic scratch-pointer slot
    emitter.instruction("ldr x10, [x9]");                                      // load the previous dynamic scratch allocation
    emitter.instruction("str x0, [x9]");                                       // publish the replacement before releasing stale storage
    emitter.instruction("cbz x10, __rt_cstr_dynamic_copy");                    // first long input has no old allocation to reclaim
    emitter.instruction("mov x0, x10");                                        // pass the prior raw runtime-heap buffer to the safe releaser
    emitter.instruction("bl __rt_heap_free_safe");                             // return the superseded scratch storage without touching static pointers
    emitter.label("__rt_cstr_dynamic_copy");
    emitter.instruction("ldr x9, [sp, #16]");                                  // destination cursor = newly allocated replacement buffer
    emitter.instruction("mov x10, x9");                                        // preserve replacement base for the return value
    emitter.instruction("ldp x1, x11, [sp, #0]");                              // restore source cursor and remaining byte count
    emitter.label("__rt_cstr_dynamic_copy_loop");
    emitter.instruction("cbz x11, __rt_cstr_dynamic_done");                    // append the terminator after every source byte is copied
    emitter.instruction("ldrb w12, [x1], #1");                                 // load one source byte and advance the PHP string cursor
    emitter.instruction("strb w12, [x9], #1");                                 // write one byte into the runtime-owned C buffer
    emitter.instruction("sub x11, x11, #1");                                   // consume one input byte
    emitter.instruction("b __rt_cstr_dynamic_copy_loop");                      // continue until the full PHP string is represented
    emitter.label("__rt_cstr_dynamic_done");
    emitter.instruction("strb wzr, [x9]");                                     // terminate the dynamically allocated C string
    emitter.instruction("mov x0, x10");                                        // return the stable replacement-buffer base
    emitter.instruction("ldp x29, x30, [sp, #32]");                            // restore the caller frame and return address
    emitter.instruction("add sp, sp, #48");                                    // discard dynamic-helper temporary slots
    emitter.instruction("ret");                                                 // return the C string to the libc-facing caller

    emitter.blank();
    emitter.comment("--- runtime: cstr2 ---");
    emitter.label_global("__rt_cstr2");

    emitter.instruction("cmp x2, #4095");                                     // reserve one byte for the C terminator
    emitter.instruction("b.hi __rt_cstr2_dynamic");                           // grow beyond the secondary fixed scratch buffer

    // -- load second buffer address --
    abi::emit_symbol_address(emitter, "x9", "_cstr_buf2");                      // load page address of second cstr buffer

    // -- copy bytes from source to buffer --
    emitter.instruction("mov x10, x9");                                         // save buffer start for return value
    emitter.instruction("mov x11, x2");                                         // copy length as loop counter
    emitter.label("__rt_cstr2_loop");
    emitter.instruction("cbz x11, __rt_cstr2_null");                            // if no bytes remain, append null terminator
    emitter.instruction("ldrb w12, [x1], #1");                                  // load byte from source, advance source ptr
    emitter.instruction("strb w12, [x9], #1");                                  // store byte to buffer, advance buffer ptr
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining byte count
    emitter.instruction("b __rt_cstr2_loop");                                   // continue copying

    // -- append null terminator and return --
    emitter.label("__rt_cstr2_null");
    emitter.instruction("strb wzr, [x9]");                                      // write null terminator after last byte
    emitter.instruction("mov x0, x10");                                         // return pointer to null-terminated string
    emitter.instruction("ret");                                                 // return to caller

    // -- dynamically sized secondary C string --
    emitter.label("__rt_cstr2_dynamic");
    emitter.instruction("sub sp, sp, #48");                                    // reserve saved input, replacement pointer, and aligned frame storage
    emitter.instruction("add x9, sp, #32");                                    // locate the saved frame pair beyond the temporary slots
    emitter.instruction("stp x29, x30, [x9]");                                 // preserve the caller frame and return address
    emitter.instruction("add x29, sp, #32");                                   // establish the helper frame
    emitter.instruction("stp x1, x2, [sp, #0]");                               // retain source bytes and length across allocation and release calls
    emitter.instruction("adds x0, x2, #1");                                    // request source bytes plus a trailing C terminator
    emitter.instruction("b.cs __rt_heap_exhausted");                           // an impossible length must fail before size arithmetic wraps
    emitter.instruction("bl __rt_heap_alloc");                                 // allocate a runtime-owned replacement buffer
    emitter.instruction("str x0, [sp, #16]");                                  // save the new buffer while replacing the retained old one
    abi::emit_symbol_address(emitter, "x9", "_cstr_buf2_dynamic");             // locate the persistent secondary dynamic-pointer slot
    emitter.instruction("ldr x10, [x9]");                                      // load the previous secondary dynamic scratch allocation
    emitter.instruction("str x0, [x9]");                                       // publish the replacement before releasing stale storage
    emitter.instruction("cbz x10, __rt_cstr2_dynamic_copy");                   // first long input has no old allocation to reclaim
    emitter.instruction("mov x0, x10");                                        // pass the prior raw runtime-heap buffer to the safe releaser
    emitter.instruction("bl __rt_heap_free_safe");                             // return the superseded scratch storage without touching static pointers
    emitter.label("__rt_cstr2_dynamic_copy");
    emitter.instruction("ldr x9, [sp, #16]");                                  // destination cursor = newly allocated replacement buffer
    emitter.instruction("mov x10, x9");                                        // preserve replacement base for the return value
    emitter.instruction("ldp x1, x11, [sp, #0]");                              // restore source cursor and remaining byte count
    emitter.label("__rt_cstr2_dynamic_copy_loop");
    emitter.instruction("cbz x11, __rt_cstr2_dynamic_done");                   // append the terminator after every source byte is copied
    emitter.instruction("ldrb w12, [x1], #1");                                 // load one source byte and advance the PHP string cursor
    emitter.instruction("strb w12, [x9], #1");                                 // write one byte into the runtime-owned C buffer
    emitter.instruction("sub x11, x11, #1");                                   // consume one input byte
    emitter.instruction("b __rt_cstr2_dynamic_copy_loop");                     // continue until the full PHP string is represented
    emitter.label("__rt_cstr2_dynamic_done");
    emitter.instruction("strb wzr, [x9]");                                     // terminate the dynamically allocated C string
    emitter.instruction("mov x0, x10");                                        // return the stable replacement-buffer base
    emitter.instruction("ldp x29, x30, [sp, #32]");                            // restore the caller frame and return address
    emitter.instruction("add sp, sp, #48");                                    // discard dynamic-helper temporary slots
    emitter.instruction("ret");                                                 // return the C string to the libc-facing caller
}

/// Emits `__rt_cstr` and `__rt_cstr2` runtime helpers for the x86_64 Linux target.
/// Uses `_cstr_buf` (4096 bytes) as the primary fast scratch path and `_cstr_buf2` (4096
/// bytes) for the independent secondary path; each helper switches to a reusable runtime-heap
/// buffer when its input exceeds that fixed capacity.
/// Input:  rdi=ignored, rsi=ignored, rdx=source length (caller passes in rdx),
///         rax=source pointer (caller passes in rax)
/// Output: rax=pointer to null-terminated string in _cstr_buf (or _cstr_buf2 for cstr2)
/// ABI:    System V AMD64 (caller-saved registers r8-r11 used as scratch).
fn emit_cstr_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: cstr ---");
    emitter.label_global("__rt_cstr");

    emitter.instruction("cmp rdx, 4095");                                     // reserve one byte for the C terminator
    emitter.instruction("ja __rt_cstr_dynamic_linux_x86_64");                  // grow beyond the fixed 4 KiB scratch buffer

    abi::emit_symbol_address(emitter, "r8", "_cstr_buf");
    emitter.instruction("mov r9, r8");                                          // preserve the start of the primary C-string scratch buffer for the return value
    emitter.instruction("mov r10, rax");                                        // copy the elephc source pointer into a dedicated source cursor
    emitter.instruction("mov rcx, rdx");                                        // copy the elephc source length into the loop counter
    emitter.label("__rt_cstr_loop");
    emitter.instruction("test rcx, rcx");                                       // stop copying once the full elephc string length has been consumed
    emitter.instruction("je __rt_cstr_null");                                   // append the null terminator once no bytes remain
    emitter.instruction("mov r11b, BYTE PTR [r10]");                            // load one byte from the elephc string payload
    emitter.instruction("mov BYTE PTR [r8], r11b");                             // store the byte into the primary C-string scratch buffer
    emitter.instruction("add r10, 1");                                          // advance the source cursor to the next elephc byte
    emitter.instruction("add r8, 1");                                           // advance the destination cursor to the next scratch byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining-byte counter
    emitter.instruction("jmp __rt_cstr_loop");                                  // continue copying until every byte has been moved

    emitter.label("__rt_cstr_null");
    emitter.instruction("mov BYTE PTR [r8], 0");                                // append the trailing C null terminator after the copied bytes
    emitter.instruction("mov rax, r9");                                         // return the start of the primary C-string scratch buffer
    emitter.instruction("ret");                                                 // return to the caller with a null-terminated path pointer

    // -- dynamically sized C string --
    emitter.label("__rt_cstr_dynamic_linux_x86_64");
    emitter.instruction("push rbp");                                           // preserve the caller frame before reserving aligned helper storage
    emitter.instruction("mov rbp, rsp");                                       // establish a stable helper frame
    emitter.instruction("sub rsp, 32");                                        // save source bytes, length, and the replacement pointer across helper calls
    emitter.instruction("mov QWORD PTR [rsp], rax");                           // retain source bytes across allocation and release calls
    emitter.instruction("mov QWORD PTR [rsp + 8], rdx");                       // retain source length across allocation and release calls
    emitter.instruction("mov rax, rdx");                                       // prepare source length for allocation-size arithmetic
    emitter.instruction("add rax, 1");                                         // request source bytes plus a trailing C terminator
    emitter.instruction("jc __rt_heap_exhausted");                             // an impossible length must fail before size arithmetic wraps
    emitter.instruction("call __rt_heap_alloc");                               // allocate a runtime-owned replacement buffer
    emitter.instruction("mov QWORD PTR [rsp + 16], rax");                      // save the new buffer while replacing the retained old one
    abi::emit_symbol_address(emitter, "r8", "_cstr_buf_dynamic");              // locate the persistent dynamic scratch-pointer slot
    emitter.instruction("mov r9, QWORD PTR [r8]");                             // load the previous dynamic scratch allocation
    emitter.instruction("mov QWORD PTR [r8], rax");                            // publish the replacement before releasing stale storage
    emitter.instruction("test r9, r9");                                        // does a previous long-input buffer exist?
    emitter.instruction("jz __rt_cstr_dynamic_copy_linux_x86_64");             // first long input has no old allocation to reclaim
    emitter.instruction("mov rax, r9");                                        // pass the prior raw runtime-heap buffer to the safe releaser
    emitter.instruction("call __rt_heap_free_safe");                           // return the superseded scratch storage without touching static pointers
    emitter.label("__rt_cstr_dynamic_copy_linux_x86_64");
    emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                       // destination cursor = newly allocated replacement buffer
    emitter.instruction("mov r9, r8");                                         // preserve replacement base for the return value
    emitter.instruction("mov r10, QWORD PTR [rsp]");                           // restore source cursor
    emitter.instruction("mov rcx, QWORD PTR [rsp + 8]");                       // restore remaining byte count
    emitter.label("__rt_cstr_dynamic_copy_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                      // have all source bytes been copied?
    emitter.instruction("jz __rt_cstr_dynamic_done_linux_x86_64");             // append the terminator once copying is complete
    emitter.instruction("mov r11b, BYTE PTR [r10]");                           // load one source byte
    emitter.instruction("mov BYTE PTR [r8], r11b");                            // write one byte into the runtime-owned C buffer
    emitter.instruction("inc r10");                                            // advance the PHP string cursor
    emitter.instruction("inc r8");                                             // advance the destination cursor
    emitter.instruction("dec rcx");                                            // consume one input byte
    emitter.instruction("jmp __rt_cstr_dynamic_copy_loop_linux_x86_64");       // continue until the full PHP string is represented
    emitter.label("__rt_cstr_dynamic_done_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r8], 0");                               // terminate the dynamically allocated C string
    emitter.instruction("mov rax, r9");                                        // return the stable replacement-buffer base
    emitter.instruction("add rsp, 32");                                        // discard helper temporary slots
    emitter.instruction("pop rbp");                                            // restore the caller frame
    emitter.instruction("ret");                                                 // return the C string to the libc-facing caller

    emitter.blank();
    emitter.comment("--- runtime: cstr2 ---");
    emitter.label_global("__rt_cstr2");

    emitter.instruction("cmp rdx, 4095");                                     // reserve one byte for the C terminator
    emitter.instruction("ja __rt_cstr2_dynamic_linux_x86_64");                 // grow beyond the secondary fixed scratch buffer

    abi::emit_symbol_address(emitter, "r8", "_cstr_buf2");
    emitter.instruction("mov r9, r8");                                          // preserve the start of the secondary C-string scratch buffer for the return value
    emitter.instruction("mov r10, rax");                                        // copy the elephc source pointer into a dedicated source cursor
    emitter.instruction("mov rcx, rdx");                                        // copy the elephc source length into the loop counter
    emitter.label("__rt_cstr2_loop");
    emitter.instruction("test rcx, rcx");                                       // stop copying once the full elephc string length has been consumed
    emitter.instruction("je __rt_cstr2_null");                                  // append the null terminator once no bytes remain
    emitter.instruction("mov r11b, BYTE PTR [r10]");                            // load one byte from the elephc string payload
    emitter.instruction("mov BYTE PTR [r8], r11b");                             // store the byte into the secondary C-string scratch buffer
    emitter.instruction("add r10, 1");                                          // advance the source cursor to the next elephc byte
    emitter.instruction("add r8, 1");                                           // advance the destination cursor to the next scratch byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining-byte counter
    emitter.instruction("jmp __rt_cstr2_loop");                                 // continue copying until every byte has been moved

    emitter.label("__rt_cstr2_null");
    emitter.instruction("mov BYTE PTR [r8], 0");                                // append the trailing C null terminator after the copied bytes
    emitter.instruction("mov rax, r9");                                         // return the start of the secondary C-string scratch buffer
    emitter.instruction("ret");                                                 // return to the caller with a null-terminated path pointer

    // -- dynamically sized secondary C string --
    emitter.label("__rt_cstr2_dynamic_linux_x86_64");
    emitter.instruction("push rbp");                                           // preserve the caller frame before reserving aligned helper storage
    emitter.instruction("mov rbp, rsp");                                       // establish a stable helper frame
    emitter.instruction("sub rsp, 32");                                        // save source bytes, length, and the replacement pointer across helper calls
    emitter.instruction("mov QWORD PTR [rsp], rax");                           // retain source bytes across allocation and release calls
    emitter.instruction("mov QWORD PTR [rsp + 8], rdx");                       // retain source length across allocation and release calls
    emitter.instruction("mov rax, rdx");                                       // prepare source length for allocation-size arithmetic
    emitter.instruction("add rax, 1");                                         // request source bytes plus a trailing C terminator
    emitter.instruction("jc __rt_heap_exhausted");                             // an impossible length must fail before size arithmetic wraps
    emitter.instruction("call __rt_heap_alloc");                               // allocate a runtime-owned replacement buffer
    emitter.instruction("mov QWORD PTR [rsp + 16], rax");                      // save the new buffer while replacing the retained old one
    abi::emit_symbol_address(emitter, "r8", "_cstr_buf2_dynamic");             // locate the persistent secondary dynamic-pointer slot
    emitter.instruction("mov r9, QWORD PTR [r8]");                             // load the previous secondary dynamic scratch allocation
    emitter.instruction("mov QWORD PTR [r8], rax");                            // publish the replacement before releasing stale storage
    emitter.instruction("test r9, r9");                                        // does a previous long-input buffer exist?
    emitter.instruction("jz __rt_cstr2_dynamic_copy_linux_x86_64");            // first long input has no old allocation to reclaim
    emitter.instruction("mov rax, r9");                                        // pass the prior raw runtime-heap buffer to the safe releaser
    emitter.instruction("call __rt_heap_free_safe");                           // return the superseded scratch storage without touching static pointers
    emitter.label("__rt_cstr2_dynamic_copy_linux_x86_64");
    emitter.instruction("mov r8, QWORD PTR [rsp + 16]");                       // destination cursor = newly allocated replacement buffer
    emitter.instruction("mov r9, r8");                                         // preserve replacement base for the return value
    emitter.instruction("mov r10, QWORD PTR [rsp]");                           // restore source cursor
    emitter.instruction("mov rcx, QWORD PTR [rsp + 8]");                       // restore remaining byte count
    emitter.label("__rt_cstr2_dynamic_copy_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                      // have all source bytes been copied?
    emitter.instruction("jz __rt_cstr2_dynamic_done_linux_x86_64");            // append the terminator once copying is complete
    emitter.instruction("mov r11b, BYTE PTR [r10]");                           // load one source byte
    emitter.instruction("mov BYTE PTR [r8], r11b");                            // write one byte into the runtime-owned C buffer
    emitter.instruction("inc r10");                                            // advance the PHP string cursor
    emitter.instruction("inc r8");                                             // advance the destination cursor
    emitter.instruction("dec rcx");                                            // consume one input byte
    emitter.instruction("jmp __rt_cstr2_dynamic_copy_loop_linux_x86_64");      // continue until the full PHP string is represented
    emitter.label("__rt_cstr2_dynamic_done_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r8], 0");                               // terminate the dynamically allocated C string
    emitter.instruction("mov rax, r9");                                        // return the stable replacement-buffer base
    emitter.instruction("add rsp, 32");                                        // discard helper temporary slots
    emitter.instruction("pop rbp");                                            // restore the caller frame
    emitter.instruction("ret");                                                 // return the C string to the libc-facing caller
}
