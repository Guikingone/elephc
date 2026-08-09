//! Purpose:
//! Emits the `__rt_nl2br`, `__rt_nl2br_loop` runtime helper assembly for nl2br.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.
//! - The worst-case `7 * len` expansion (an all-newline input becomes `<br />\n` per byte) is
//!   reserved through `__rt_concat_reserve` before the first store, so long inputs fall back to
//!   heap storage instead of running off the end of the 64 KiB concat scratch buffer.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_nl2br` runtime helper for the nl2br PHP builtin.
///
/// Dispatches to the platform-specific implementation (x86_64 Linux or ARM64).
/// On ARM64 the helper reads the input string from x1 (ptr) and x2 (len), scans each
/// byte, and inserts the literal `<br />` before every `0x0A` newline. The result
/// pointer/length are returned in x1/x2.
///
/// Reserves the worst-case `7 * len` expansion through `__rt_concat_reserve` (concat scratch
/// while it fits, owned heap storage otherwise) and finishes through `__rt_concat_publish`,
/// which advances `_concat_off` only for scratch-backed results.
///
/// Clobbers every caller-saved register, because the reservation can reach `__rt_heap_alloc`.
/// A wrapped `7 * len` product reports PHP's allocation-overflow fatal through
/// `__rt_alloc_overflow` instead of reserving a too-small destination.
pub fn emit_nl2br(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_nl2br_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: nl2br ---");
    emitter.label_global("__rt_nl2br");

    // -- reserve the worst-case seven-bytes-per-input-byte expansion before writing anything --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed source string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the nl2br helper frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the source pointer and length across the reservation call
    emitter.instruction("mov x9, #7");                                          // worst-case expansion factor for an all-newline input (`<br />` plus the newline)
    emitter.instruction("umulh x10, x2, x9");                                   // capture the high half of the 7 * length product
    emitter.instruction("cbnz x10, __rt_nl2br_size_overflow");                  // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("mul x0, x2, x9");                                      // compute the worst-case expanded result size
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the expanded result
    emitter.instruction("mov x9, x0");                                          // destination pointer
    emitter.instruction("mov x10, x0");                                         // save result start
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed source pointer and length
    emitter.instruction("mov x11, x2");                                         // remaining count

    emitter.label("__rt_nl2br_loop");
    emitter.instruction("cbz x11, __rt_nl2br_done");                            // no bytes left → done
    emitter.instruction("ldrb w12, [x1], #1");                                  // load byte, advance source
    emitter.instruction("sub x11, x11, #1");                                    // decrement remaining
    emitter.instruction("cmp w12, #10");                                        // is it '\n'?
    emitter.instruction("b.ne __rt_nl2br_store");                               // no → store as-is
    // -- insert "<br />" before the newline --
    emitter.instruction("mov w13, #60");                                        // '<'
    emitter.instruction("strb w13, [x9], #1");                                  // write '<'
    emitter.instruction("mov w13, #98");                                        // 'b'
    emitter.instruction("strb w13, [x9], #1");                                  // write 'b'
    emitter.instruction("mov w13, #114");                                       // 'r'
    emitter.instruction("strb w13, [x9], #1");                                  // write 'r'
    emitter.instruction("mov w13, #32");                                        // ' '
    emitter.instruction("strb w13, [x9], #1");                                  // write ' '
    emitter.instruction("mov w13, #47");                                        // '/'
    emitter.instruction("strb w13, [x9], #1");                                  // write '/'
    emitter.instruction("mov w13, #62");                                        // '>'
    emitter.instruction("strb w13, [x9], #1");                                  // write '>'
    emitter.label("__rt_nl2br_store");
    emitter.instruction("strb w12, [x9], #1");                                  // write original byte (including '\n')
    emitter.instruction("b __rt_nl2br_loop");                                   // next byte

    emitter.label("__rt_nl2br_done");
    emitter.instruction("mov x1, x10");                                         // result pointer
    emitter.instruction("sub x2, x9, x10");                                     // result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the nl2br helper frame
    emitter.instruction("ret");                                                 // return

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_nl2br_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits the x86_64 Linux variant of the `__rt_nl2br` runtime helper.
///
/// Reads the input string from rax (ptr) and rdx (len). Scans each byte and inserts
/// the literal `<br />` before every `0x0A` newline. The result pointer/length are
/// returned in rax/rdx.
///
/// Reserves the worst-case `7 * len` expansion through `__rt_concat_reserve` and publishes the
/// written length through `__rt_concat_publish`, so long inputs use owned heap storage instead
/// of running off the end of the 64 KiB concat scratch buffer.
fn emit_nl2br_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: nl2br ---");
    emitter.label_global("__rt_nl2br");

    // -- reserve the worst-case seven-bytes-per-input-byte expansion before writing anything --
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed source string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the source pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the borrowed source pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the borrowed source length across the reservation call
    emitter.instruction("imul rax, rdx, 7");                                    // compute the worst-case expanded result size as 7 * source length
    emitter.instruction("jo __rt_nl2br_size_overflow_linux_x86_64");            // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the expanded result
    emitter.instruction("mov r11, rax");                                        // compute the destination pointer where the nl2br() result begins
    emitter.instruction("mov r8, r11");                                         // preserve the result start pointer for the returned string value after the loop mutates the destination cursor
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // seed the remaining source length counter from the borrowed input string length
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // preserve the borrowed source string cursor in a dedicated register before the loop mutates caller-saved registers

    emitter.label("__rt_nl2br_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every source byte has been classified and copied into concat storage
    emitter.instruction("jz __rt_nl2br_done_linux_x86_64");                     // finish once the borrowed source string has been fully consumed
    emitter.instruction("mov dl, BYTE PTR [rsi]");                              // load one source byte before deciding whether nl2br() must inject a break tag
    emitter.instruction("add rsi, 1");                                          // advance the borrowed source string cursor after consuming one byte
    emitter.instruction("sub rcx, 1");                                          // decrement the remaining source length after consuming one byte
    emitter.instruction("cmp dl, 10");                                          // is the current byte a newline that should gain a preceding HTML break tag?
    emitter.instruction("jne __rt_nl2br_store_linux_x86_64");                   // copy non-newline bytes straight through without injecting extra HTML markup
    emitter.instruction("mov BYTE PTR [r11], 60");                              // write '<' as the first byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the '<' of the break tag
    emitter.instruction("mov BYTE PTR [r11], 98");                              // write 'b' as the second byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the 'b' of the break tag
    emitter.instruction("mov BYTE PTR [r11], 114");                             // write 'r' as the third byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the 'r' of the break tag
    emitter.instruction("mov BYTE PTR [r11], 32");                              // write the space byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the space of the break tag
    emitter.instruction("mov BYTE PTR [r11], 47");                              // write '/' as the fifth byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the slash of the break tag
    emitter.instruction("mov BYTE PTR [r11], 62");                              // write '>' as the final byte of the injected `<br />` break tag
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after emitting the closing angle bracket of the break tag

    emitter.label("__rt_nl2br_store_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r11], dl");                              // store the original source byte, including newline bytes after any injected break tag prefix
    emitter.instruction("add r11, 1");                                          // advance the concat-buffer destination cursor after storing the original source byte
    emitter.instruction("jmp __rt_nl2br_loop_linux_x86_64");                    // continue scanning the remaining source bytes until the input string is exhausted

    emitter.label("__rt_nl2br_done_linux_x86_64");
    emitter.instruction("mov rax, r8");                                         // return the reserved result start pointer after nl2br() finishes expanding the input string
    emitter.instruction("mov rdx, r11");                                        // copy the final destination cursor before computing the produced string length
    emitter.instruction("sub rdx, r8");                                         // compute the produced string length as dest_end - dest_start for the returned x86_64 string value
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the nl2br spill slots before returning the expanded string
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning the expanded string
    emitter.instruction("ret");                                                 // return the nl2br() result in the standard x86_64 string result registers

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_nl2br_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
