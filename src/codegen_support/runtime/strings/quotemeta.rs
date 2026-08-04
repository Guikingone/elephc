//! Purpose:
//! Emits the `__rt_quotemeta` runtime helper assembly for PHP's `quotemeta`: prefixes every
//! regular-expression metacharacter in a byte string with a single backslash.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The escaped set is php-src's `quotemeta` switch verbatim: `.` `\` `+` `*` `?` `[` `^`
//!   `]` `$` `(` `)`. Every other byte, including NUL and high-bit bytes, is copied through.
//! - Membership is a single 64-bit bitmap test instead of an eleven-way compare chain: all
//!   escaped characters live in the contiguous ASCII window `36..=94`, so `c - 36` indexes
//!   `QUOTEMETA_ESCAPE_MASK` directly and any byte outside the window skips the test.
//! - The worst case is two output bytes per input byte, so `2 * len` is reserved through
//!   `__rt_concat_reserve` before the first store and the ACTUAL written length is handed to
//!   `__rt_concat_publish`. Over-reserving is safe; writing past the reservation is not.

use crate::codegen_support::abi;
use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Bitmap of the bytes `quotemeta` escapes, indexed by `byte - 36`.
///
/// Bit `n` is set when the character `36 + n` must be prefixed with a backslash, covering
/// `$`(36) `(`(40) `)`(41) `*`(42) `+`(43) `.`(46) `?`(63) `[`(91) `\`(92) `]`(93) `^`(94).
/// The window is 59 characters wide, so the whole set fits one 64-bit register.
const QUOTEMETA_ESCAPE_MASK: i64 = 0x0780_0000_0800_04F1;

/// First byte covered by `QUOTEMETA_ESCAPE_MASK`; bytes below it are never escaped.
const QUOTEMETA_WINDOW_START: u32 = 36;

/// Width of the escape window in characters; `byte - 36` must stay below it to be tested.
const QUOTEMETA_WINDOW_LEN: u32 = 59;

/// Emits the `__rt_quotemeta` runtime helper.
///
/// ABI (AArch64):
///   Input:  `x1` = source pointer, `x2` = source length.
///   Output: `x1` = result pointer, `x2` = result length.
///
/// ABI (x86_64 System V):
///   Input:  `rax` = source pointer, `rdx` = source length.
///   Output: `rax` = result pointer, `rdx` = result length.
///
/// An empty input reserves and publishes zero bytes, which matches PHP's empty-string
/// result. The result is published through `__rt_concat_publish`, so it lives in the shared
/// concat scratch while it fits and in an owned heap block otherwise.
pub fn emit_quotemeta(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_quotemeta_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: quotemeta ---");
    emitter.label_global("__rt_quotemeta");

    // -- reserve the worst-case two-bytes-per-input-byte result before writing anything --
    emitter.instruction("sub sp, sp, #32");                                     // allocate spill space for the borrowed source string
    emitter.instruction("stp x29, x30, [sp, #16]");                             // save the frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #16");                                    // establish the quotemeta helper frame pointer
    emitter.instruction("stp x1, x2, [sp]");                                    // save the source pointer and length across the reservation call
    emitter.instruction("adds x0, x2, x2");                                     // compute the worst-case result size as 2 * source length
    emitter.instruction("b.cs __rt_quotemeta_size_overflow");                   // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the escaped result
    emitter.instruction("mov x9, x0");                                          // destination cursor
    emitter.instruction("mov x10, x0");                                         // save the result start for the published pointer
    emitter.instruction("ldp x1, x2, [sp]");                                    // reload the borrowed source pointer and length
    emitter.instruction("mov x11, x2");                                         // remaining source byte count
    abi::emit_load_int_immediate(emitter, "x15", QUOTEMETA_ESCAPE_MASK);

    emitter.label("__rt_quotemeta_loop");
    emitter.instruction("cbz x11, __rt_quotemeta_done");                        // finish once every source byte has been consumed
    emitter.instruction("ldrb w12, [x1], #1");                                  // load the next source byte and advance the source cursor
    emitter.instruction("sub x11, x11, #1");                                    // record that one source byte has been consumed
    emitter.instruction(&format!("sub w13, w12, #{QUOTEMETA_WINDOW_START}"));   // index the escape bitmap by shifting the byte into window space
    emitter.instruction(&format!("cmp w13, #{QUOTEMETA_WINDOW_LEN}"));          // is the byte outside the escapable window (unsigned, so low bytes wrap high)?
    emitter.instruction("b.hs __rt_quotemeta_store");                           // bytes outside the window are copied through untouched
    emitter.instruction("lsr x14, x15, x13");                                   // move this character's escape bit into position 0
    emitter.instruction("tbz x14, #0, __rt_quotemeta_store");                   // characters without an escape bit are copied through untouched
    emitter.instruction("mov w13, #92");                                        // ASCII backslash is the escape prefix
    emitter.instruction("strb w13, [x9], #1");                                  // write the escape prefix ahead of the metacharacter

    emitter.label("__rt_quotemeta_store");
    emitter.instruction("strb w12, [x9], #1");                                  // copy the source byte itself into the result
    emitter.instruction("b __rt_quotemeta_loop");                               // continue with the next source byte

    emitter.label("__rt_quotemeta_done");
    emitter.instruction("mov x1, x10");                                         // return the escaped string start pointer
    emitter.instruction("sub x2, x9, x10");                                     // the written byte count is the result length
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #16]");                             // restore the frame pointer and return address
    emitter.instruction("add sp, sp, #32");                                     // release the quotemeta helper frame
    emitter.instruction("ret");                                                 // return the escaped string as a PHP string pair

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_quotemeta_size_overflow");
    emitter.instruction("b __rt_alloc_overflow");                               // unconditional branch keeps the fatal trampoline cross-atom safe
}

/// Emits `__rt_quotemeta` for x86_64 Linux using the System V ABI.
///
/// Uses `bt` against the escape bitmap so the membership test needs no `cl` shift count,
/// which keeps `rcx` free as the source countdown register.
fn emit_quotemeta_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: quotemeta ---");
    emitter.label_global("__rt_quotemeta");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base for the borrowed source string
    emitter.instruction("sub rsp, 32");                                         // reserve aligned spill slots for the source pointer and length
    emitter.instruction("mov QWORD PTR [rbp - 8], rax");                        // save the source pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 16], rdx");                       // save the source byte count across the reservation call
    emitter.instruction("mov rax, rdx");                                        // seed the result size from the source byte count
    emitter.instruction("add rax, rax");                                        // compute the worst-case result size as 2 * source length
    emitter.instruction("jc __rt_quotemeta_size_overflow_linux_x86_64");        // reject a wrapped size instead of reserving a too-small destination
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the escaped result
    emitter.instruction("mov r9, rax");                                         // destination cursor
    emitter.instruction("mov r10, rax");                                        // save the result start for the published pointer
    emitter.instruction("mov rsi, QWORD PTR [rbp - 8]");                        // reload the borrowed source pointer as a read cursor
    emitter.instruction("mov rcx, QWORD PTR [rbp - 16]");                       // reload the source byte count as a decrementing counter
    emitter.instruction(&format!("mov r11, 0x{QUOTEMETA_ESCAPE_MASK:x}"));      // materialize the escape bitmap for the membership test

    emitter.label("__rt_quotemeta_loop_linux_x86_64");
    emitter.instruction("test rcx, rcx");                                       // stop once every source byte has been consumed
    emitter.instruction("je __rt_quotemeta_done_linux_x86_64");                 // finish when the source string has been fully escaped
    emitter.instruction("movzx eax, BYTE PTR [rsi]");                           // load the next source byte and widen it for the window test
    emitter.instruction("add rsi, 1");                                          // advance the source cursor after consuming one byte
    emitter.instruction("sub rcx, 1");                                          // record that one source byte has been consumed
    emitter.instruction("mov edx, eax");                                        // copy the source byte before shifting it into window space
    emitter.instruction(&format!("sub edx, {QUOTEMETA_WINDOW_START}"));         // index the escape bitmap by shifting the byte into window space
    emitter.instruction(&format!("cmp edx, {QUOTEMETA_WINDOW_LEN}"));           // is the byte outside the escapable window (unsigned, so low bytes wrap high)?
    emitter.instruction("jae __rt_quotemeta_store_linux_x86_64");               // bytes outside the window are copied through untouched
    emitter.instruction("bt r11, rdx");                                         // test this character's escape bit inside the bitmap
    emitter.instruction("jnc __rt_quotemeta_store_linux_x86_64");               // characters without an escape bit are copied through untouched
    emitter.instruction("mov BYTE PTR [r9], 92");                               // write the ASCII backslash escape prefix
    emitter.instruction("add r9, 1");                                           // advance the destination cursor past the escape prefix

    emitter.label("__rt_quotemeta_store_linux_x86_64");
    emitter.instruction("mov BYTE PTR [r9], al");                               // copy the source byte itself into the result
    emitter.instruction("add r9, 1");                                           // advance the destination cursor past the copied byte
    emitter.instruction("jmp __rt_quotemeta_loop_linux_x86_64");                // continue with the next source byte

    emitter.label("__rt_quotemeta_done_linux_x86_64");
    emitter.instruction("mov rax, r10");                                        // return the escaped string start pointer
    emitter.instruction("mov rdx, r9");                                         // copy the destination cursor into the length scratch register
    emitter.instruction("sub rdx, r10");                                        // the written byte count is the result length
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 32");                                         // release the quotemeta spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the escaped string as a PHP string pair

    // -- impossible result size: report the shared allocation-overflow fatal error --
    emitter.label("__rt_quotemeta_size_overflow_linux_x86_64");
    emitter.instruction("jmp __rt_alloc_overflow");                             // unconditional branch keeps the fatal trampoline reachable from every caller
}
