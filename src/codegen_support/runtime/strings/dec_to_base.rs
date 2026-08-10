//! Purpose:
//! Emits the `__rt_dec_to_base` runtime helper assembly shared by PHP's `dechex`, `decbin`,
//! and `decoct` builtins: renders a 64-bit value as an unsigned string in a given base.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - The value is interpreted as UNSIGNED, which is what makes `dechex(-1)` render
//!   `"ffffffffffffffff"` exactly like reference PHP instead of a signed `-1`.
//! - Digits above 9 use lowercase `a`-`z`, matching php-src's `_php_math_longtobase`.
//! - Digits are produced least-significant-first into a frame-local 64-byte buffer (the widest
//!   possible result is base 2 of `PHP_INT_MAX`-sized input, i.e. 64 characters) and only then
//!   copied into a reservation of the EXACT result size taken from `__rt_concat_reserve`.
//!   Writing right-to-left straight into the reservation would hand back an interior pointer
//!   of a heap-backed block, which the release path could not free.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the `__rt_dec_to_base` runtime helper.
///
/// ABI (AArch64):
///   Input:  `x0` = value (read as unsigned 64-bit), `x3` = base (2..36).
///   Output: `x1` = result pointer, `x2` = result length.
///
/// ABI (x86_64 System V):
///   Input:  `rax` = value (read as unsigned 64-bit), `rdi` = base (2..36).
///   Output: `rax` = result pointer, `rdx` = result length.
///
/// A zero value renders the single character `"0"`; every other value renders without
/// leading zeros. The result is published through `__rt_concat_publish`, so it lives in the
/// shared concat scratch while it fits and in an owned heap block otherwise.
pub fn emit_dec_to_base(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_dec_to_base_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: dec_to_base ---");
    emitter.label_global("__rt_dec_to_base");

    emitter.instruction("sub sp, sp, #96");                                     // reserve a 64-byte digit buffer plus spill and frame slots
    emitter.instruction("stp x29, x30, [sp, #80]");                             // save the frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #80");                                    // establish the dec_to_base helper frame pointer
    emitter.instruction("add x9, sp, #64");                                     // point the digit cursor just past the end of the digit buffer
    emitter.instruction("mov x10, #0");                                         // start with no digits emitted
    emitter.instruction("mov x11, x0");                                         // copy the value into the shrinking conversion accumulator
    emitter.instruction("mov x12, x3");                                         // keep the requested base in a stable register
    emitter.instruction("cbnz x11, __rt_dec_to_base_loop");                     // a non-zero value produces its digits in the loop below
    emitter.instruction("mov w13, #48");                                        // ASCII '0' is the whole result for a zero value
    emitter.instruction("strb w13, [x9, #-1]!");                                // write the single zero digit and step the cursor back onto it
    emitter.instruction("mov x10, #1");                                         // the zero result is exactly one character long
    emitter.instruction("b __rt_dec_to_base_emit");                             // publish the single-character result

    emitter.label("__rt_dec_to_base_loop");
    emitter.instruction("udiv x14, x11, x12");                                  // divide the accumulator by the base, unsigned
    emitter.instruction("msub x15, x14, x12, x11");                             // recover the remainder as value - quotient * base
    emitter.instruction("mov x11, x14");                                        // the quotient becomes the next accumulator
    emitter.instruction("cmp x15, #10");                                        // does this digit need a letter rather than a numeral?
    emitter.instruction("b.lo __rt_dec_to_base_numeral");                       // digits 0-9 use the ASCII numerals
    emitter.instruction("add w15, w15, #87");                                   // map digits 10-35 to lowercase 'a'-'z'
    emitter.instruction("b __rt_dec_to_base_store");                            // the digit character is ready to store
    emitter.label("__rt_dec_to_base_numeral");
    emitter.instruction("add w15, w15, #48");                                   // map digits 0-9 to ASCII '0'-'9'
    emitter.label("__rt_dec_to_base_store");
    emitter.instruction("strb w15, [x9, #-1]!");                                // store the digit least-significant-first, walking backwards
    emitter.instruction("add x10, x10, #1");                                    // count the digit just emitted
    emitter.instruction("cbnz x11, __rt_dec_to_base_loop");                     // keep converting while the accumulator is non-zero

    emitter.label("__rt_dec_to_base_emit");
    emitter.instruction("stp x9, x10, [sp, #64]");                              // save the first-digit pointer and the digit count across the reservation
    emitter.instruction("mov x0, x10");                                         // reserve exactly as many bytes as the rendered result needs
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage for the rendered digits
    emitter.instruction("ldp x9, x10, [sp, #64]");                              // reload the first-digit pointer and digit count after the reservation
    emitter.instruction("mov x1, x0");                                          // the reservation start is the published result pointer
    emitter.instruction("mov x2, x10");                                         // the digit count is the published result length
    emitter.instruction("mov x13, #0");                                         // start copying at the first rendered digit

    emitter.label("__rt_dec_to_base_copy");
    emitter.instruction("cmp x13, x10");                                        // have all rendered digits been copied out?
    emitter.instruction("b.hs __rt_dec_to_base_copied");                        // finish once the whole result has been copied
    emitter.instruction("ldrb w14, [x9, x13]");                                 // load the next rendered digit from the frame buffer
    emitter.instruction("strb w14, [x0, x13]");                                 // store it into the reserved result storage
    emitter.instruction("add x13, x13, #1");                                    // advance the copy index
    emitter.instruction("b __rt_dec_to_base_copy");                             // copy the next rendered digit

    emitter.label("__rt_dec_to_base_copied");
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore the frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the dec_to_base helper frame
    emitter.instruction("ret");                                                 // return the rendered digits as a PHP string pair
}

/// Emits `__rt_dec_to_base` for x86_64 Linux using the System V ABI.
///
/// The frame keeps its 64-byte digit buffer at `[rbp-128, rbp-64)` and its two spill slots at
/// `[rbp-64]`/`[rbp-56]`, deliberately clear of `[rbp-8]`..`[rbp-24]` so the layout cannot
/// collide with the saved-register slots other runtime emitters expect there.
fn emit_dec_to_base_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: dec_to_base ---");
    emitter.label_global("__rt_dec_to_base");

    emitter.instruction("push rbp");                                            // preserve the caller frame pointer across the reservation and publish calls
    emitter.instruction("mov rbp, rsp");                                        // establish the dec_to_base helper frame pointer
    emitter.instruction("sub rsp, 128");                                        // reserve the digit buffer and spill slots, keeping the stack 16-byte aligned
    emitter.instruction("lea r9, [rbp - 64]");                                  // point the digit cursor just past the end of the digit buffer
    emitter.instruction("xor r10d, r10d");                                      // start with no digits emitted
    emitter.instruction("mov r11, rdi");                                        // keep the requested base in a stable register
    emitter.instruction("test rax, rax");                                       // is the value zero?
    emitter.instruction("jnz __rt_dec_to_base_loop_linux_x86_64");              // a non-zero value produces its digits in the loop below
    emitter.instruction("sub r9, 1");                                           // step the cursor back onto the single zero digit
    emitter.instruction("mov BYTE PTR [r9], 48");                               // ASCII '0' is the whole result for a zero value
    emitter.instruction("mov r10, 1");                                          // the zero result is exactly one character long
    emitter.instruction("jmp __rt_dec_to_base_emit_linux_x86_64");              // publish the single-character result

    emitter.label("__rt_dec_to_base_loop_linux_x86_64");
    emitter.instruction("xor edx, edx");                                        // clear the high dividend half before the unsigned division
    emitter.instruction("div r11");                                             // divide the accumulator by the base, leaving the remainder in rdx
    emitter.instruction("cmp rdx, 10");                                         // does this digit need a letter rather than a numeral?
    emitter.instruction("jb __rt_dec_to_base_numeral_linux_x86_64");            // digits 0-9 use the ASCII numerals
    emitter.instruction("add rdx, 87");                                         // map digits 10-35 to lowercase 'a'-'z'
    emitter.instruction("jmp __rt_dec_to_base_store_linux_x86_64");             // the digit character is ready to store
    emitter.label("__rt_dec_to_base_numeral_linux_x86_64");
    emitter.instruction("add rdx, 48");                                         // map digits 0-9 to ASCII '0'-'9'
    emitter.label("__rt_dec_to_base_store_linux_x86_64");
    emitter.instruction("sub r9, 1");                                           // walk the digit cursor backwards by one character
    emitter.instruction("mov BYTE PTR [r9], dl");                               // store the digit least-significant-first
    emitter.instruction("add r10, 1");                                          // count the digit just emitted
    emitter.instruction("test rax, rax");                                       // is the remaining quotient non-zero?
    emitter.instruction("jnz __rt_dec_to_base_loop_linux_x86_64");              // keep converting while the accumulator is non-zero

    emitter.label("__rt_dec_to_base_emit_linux_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 64], r9");                        // save the first-digit pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 56], r10");                       // save the digit count across the reservation call
    emitter.instruction("mov rax, r10");                                        // reserve exactly as many bytes as the rendered result needs
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage for the rendered digits
    emitter.instruction("mov r9, QWORD PTR [rbp - 64]");                        // reload the first-digit pointer after the reservation
    emitter.instruction("mov r10, QWORD PTR [rbp - 56]");                       // reload the digit count after the reservation
    emitter.instruction("xor ecx, ecx");                                        // start copying at the first rendered digit

    emitter.label("__rt_dec_to_base_copy_linux_x86_64");
    emitter.instruction("cmp rcx, r10");                                        // have all rendered digits been copied out?
    emitter.instruction("jae __rt_dec_to_base_copied_linux_x86_64");            // finish once the whole result has been copied
    emitter.instruction("mov r8b, BYTE PTR [r9 + rcx]");                        // load the next rendered digit from the frame buffer
    emitter.instruction("mov BYTE PTR [rax + rcx], r8b");                       // store it into the reserved result storage
    emitter.instruction("add rcx, 1");                                          // advance the copy index
    emitter.instruction("jmp __rt_dec_to_base_copy_linux_x86_64");              // copy the next rendered digit

    emitter.label("__rt_dec_to_base_copied_linux_x86_64");
    emitter.instruction("mov rdx, r10");                                        // the digit count is the published result length
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("add rsp, 128");                                        // release the dec_to_base helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the rendered digits as a PHP string pair
}
