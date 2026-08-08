//! Purpose:
//! Emits the `__rt_stripslashes`, `__rt_stripslashes_loop` runtime helper assembly for stripslashes.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - Unescaping never grows the payload, so the source length is reserved through
//!   `__rt_concat_reserve` before the first store; inputs beyond the 64 KiB concat scratch
//!   buffer fall back to heap storage instead of running off the end of it.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_stripslashes` runtime helper for the current target.
///
/// Removes escape backslashes from a PHP byte-string by copying bytes from source
/// to reserved destination storage, skipping backslashes and their following escaped characters.
/// Trailing backslashes (no character to escape) are preserved as literal backslashes.
///
/// # ABI
/// - ARM64: input string in x1/x2 (pointer/length), result returned in x1/x2
/// - x86_64 Linux: input string in rax/rdx (pointer/length), result returned in rax/rdx
/// - Reserves the (never-exceeded) source length through `__rt_concat_reserve` and publishes the
///   written length through `__rt_concat_publish`, so only scratch-backed results move `_concat_off`.
/// - Clobbers every caller-saved register, because the reservation can reach `__rt_heap_alloc`.
pub fn emit_stripslashes(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_stripslashes_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: stripslashes ---");
    emitter.label_global("__rt_stripslashes");

    // -- reserve the worst-case (unchanged-length) unescaped result before writing anything --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed source string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the stripslashes helper frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the source pointer and length across the reservation call
    emitter.instruction("mov x0, x2");                                          // unescaping never grows the payload, so the source length bounds the result
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the unescaped result
    emitter.instruction("mov x9, x0");                                          // destination pointer
    emitter.instruction("mov x10, x0");                                         // save result start
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed source pointer and length
    emitter.instruction("mov x11, x2");                                         // remaining byte count

    emitter.label("__rt_stripslashes_loop");
    emitter.instruction("cbz x11, __rt_stripslashes_done");                     // done if no bytes left
    emitter.instruction("ldrb w12, [x1], #1");                                  // load byte, advance source
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining
    emitter.instruction("cmp w12, #92");                                        // is it a backslash?
    emitter.instruction("b.ne __rt_stripslashes_store");                        // no → store as-is
    // -- backslash: skip it and store the next char --
    emitter.instruction("cbz x11, __rt_stripslashes_store");                    // trailing backslash → store it
    emitter.instruction("ldrb w12, [x1], #1");                                  // load escaped char, advance
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining
    emitter.label("__rt_stripslashes_store");
    emitter.instruction("strb w12, [x9], #1");                                  // store byte to output
    emitter.instruction("b __rt_stripslashes_loop");                            // next byte

    emitter.label("__rt_stripslashes_done");
    emitter.instruction("mov x1, x10");                                         // result pointer
    emitter.instruction("sub x2, x9, x10");                                     // result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the stripslashes helper frame
    emitter.instruction("ret");                                                 // return
}

/// Emits `__rt_stripslashes` for the x86_64 Linux ABI.
/// x86_64 calling convention: input string in rax/rdx, result in rax/rdx.
/// Reserves the (never-exceeded) source length through `__rt_concat_reserve` and publishes the
/// written length through `__rt_concat_publish`.
fn emit_stripslashes_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: stripslashes ---");
    emitter.label_global("__rt_stripslashes");

    // -- reserve the worst-case (unchanged-length) unescaped result before writing anything --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed source string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the source pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the borrowed source pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the borrowed source length across the reservation call
    emitter.instruction("mov rax, rdx");                                        // unescaping never grows the payload, so the source length bounds the result
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the unescaped result
    emitter.instruction("mov r9, rax");                                         // compute the destination write pointer where the unescaped string begins
    emitter.instruction("mov r10, r9");                                         // preserve the unescaped-string start pointer for the final result slice
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // track how many source bytes remain to be scanned for escape prefixes
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // reload the borrowed source cursor the unescape loop advances through

    emitter.label("__rt_stripslashes_loop");
    emitter.instruction("test rcx, rcx");                                       // have we consumed every byte of the escaped source string?
    emitter.instruction("je __rt_stripslashes_done");                           // finish once no source bytes remain
    emitter.instruction("movzx r11d, BYTE PTR [rax]");                          // load the next source byte and widen it for unsigned backslash comparisons
    emitter.instruction("add rax, 1");                                          // advance the source pointer after consuming the current byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining source-byte count after the load
    emitter.instruction("cmp r11b, 92");                                        // does the current source byte start an escape sequence?
    emitter.instruction("jne __rt_stripslashes_store");                         // ordinary bytes copy through unchanged when no backslash prefix is present
    emitter.instruction("test rcx, rcx");                                       // is the backslash the final byte of the source string?
    emitter.instruction("je __rt_stripslashes_store");                          // trailing backslashes stay literal because there is no escaped byte to consume
    emitter.instruction("movzx r11d, BYTE PTR [rax]");                          // load the escaped byte that follows the backslash prefix
    emitter.instruction("add rax, 1");                                          // advance past the escaped byte after discarding the prefix backslash
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining count for the escaped byte we just consumed

    emitter.label("__rt_stripslashes_store");
    emitter.instruction("mov BYTE PTR [r9], r11b");                             // copy the current logical output byte into the concat buffer
    emitter.instruction("add r9, 1");                                           // advance the concat-buffer write pointer past the copied output byte
    emitter.instruction("jmp __rt_stripslashes_loop");                          // continue processing the remaining source bytes

    emitter.label("__rt_stripslashes_done");
    emitter.instruction("mov rax, r10");                                        // return the unescaped-string start pointer in the x86_64 string result pointer register
    emitter.instruction("mov rdx, r9");                                         // snapshot the final destination write pointer before computing the unescaped result length
    emitter.instruction("sub rdx, r10");                                        // compute the unescaped result length from the write pointer minus the start pointer
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the stripslashes spill slots before returning the unescaped string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the unescaped string
    emitter.instruction("ret");                                                 // return to the caller with the unescaped string slice in rax/rdx
}
