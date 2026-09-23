//! Purpose:
//! Emits the `__rt_itoa`, `__rt_itoa_positive` runtime helper assembly for integer-to-string conversion.
//! Keeps PHP byte-string pointer/length behavior and target-specific ABI variants in one focused emitter.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//!
//! Key details:
//! - String helpers use PHP pointer/length pairs and target ABI return registers; heap-backed results must remain refcount-compatible.

use crate::codegen_support::{emit::Emitter, platform::Arch};

/// Emits the `__rt_itoa` runtime helper: converts a signed 64-bit integer to a decimal string.
///
/// Digits are rendered right-to-left into a frame-local buffer and only then copied into a
/// reservation of the EXACT result size taken from `__rt_concat_reserve`, which is the same
/// shape `__rt_dec_to_base` uses and for the same two reasons: the reservation is bounded, and
/// writing right-to-left straight into it would hand back an interior pointer of a heap-backed
/// block that the release path could not free.
///
/// # ABI
/// - ARM64: input in `x0`, returns pointer in `x1`, length in `x2`.
/// - x86_64 Linux: input in `rax`, returns pointer in `rax`, length in `rdx`.
/// - x86_64 macOS uses `emit_itoa_linux_x86_64` with the same convention.
///
/// # Side effects
/// - Reserves the result through `__rt_concat_reserve` and publishes it through
///   `__rt_concat_publish`, so the result lives in the shared concat scratch while it fits and
///   in an owned heap block otherwise.
/// - Clobbers every caller-saved register, like every other producer that reserves.
///
/// # Why it is not a bare scratch write any more
/// It used to write at `_concat_buf + _concat_off + 20` and advance `_concat_off` by 21 with NO
/// capacity check, so a program whose `_concat_off` had passed 65536 wrote outside the buffer.
/// MEASURED on Symfony's `bin/console about`: `_concat_off` runs past the 64 KiB scratch while
/// `MergeExtensionConfigurationParameterBag::freezeAfterProcessing` serializes the parameter bag,
/// and `__rt_itoa` stored the digit `'6'` 8344 bytes past the buffer's end -- into
/// `_elephc_eval_serialize_object_fn`, the BSS slot the interpreter's serialize callback lives
/// in. `__rt_serialize_object` then branched to `0x645f746c7561` ("ault_d"), the string bytes its
/// own caller had written over the neighbouring slots.
pub fn emit_itoa(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_itoa_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: itoa ---");
    emitter.label_global("__rt_itoa");

    // -- set up a frame whose local buffer holds the widest possible rendering --
    emitter.instruction("sub sp, sp, #96");                                     // reserve a 64-byte render buffer plus spill and frame slots
    emitter.instruction("stp x29, x30, [sp, #80]");                             // save the frame pointer and return address across the reservation call
    emitter.instruction("add x29, sp, #80");                                    // establish the itoa helper frame pointer
    emitter.instruction("add x9, sp, #64");                                     // point the digit cursor just past the end of the render buffer

    // -- initialize counters --
    emitter.instruction("mov x10, #0");                                         // digit count = 0
    emitter.instruction("mov x15, #0");                                         // negative flag = 0 (not negative)

    // -- handle sign --
    emitter.instruction("cmp x0, #0");                                          // check if input is negative
    emitter.instruction("b.ge __rt_itoa_positive");                             // skip negation if >= 0
    emitter.instruction("mov x15, #1");                                         // set negative flag
    emitter.instruction("neg x0, x0");                                          // negate to make value positive

    // -- handle zero special case --
    emitter.label("__rt_itoa_positive");
    emitter.instruction("cbnz x0, __rt_itoa_loop");                             // if value != 0, start digit extraction loop
    emitter.instruction("mov w12, #48");                                        // ASCII '0'
    emitter.instruction("strb w12, [x9, #-1]!");                                // write the single zero digit and step the cursor onto it
    emitter.instruction("mov x10, #1");                                         // digit count = 1
    emitter.instruction("b __rt_itoa_sign");                                    // zero is never negative, so the sign check just falls through

    // -- extract digits right-to-left via repeated division by 10 --
    emitter.label("__rt_itoa_loop");
    emitter.instruction("mov x12, #10");                                        // divisor = 10
    emitter.instruction("udiv x13, x0, x12");                                   // quotient = value / 10
    emitter.instruction("msub x14, x13, x12, x0");                              // remainder = value - (quotient * 10)
    emitter.instruction("add x14, x14, #48");                                   // convert remainder to ASCII digit
    emitter.instruction("strb w14, [x9, #-1]!");                                // store the digit and step the cursor left (right-to-left)
    emitter.instruction("add x10, x10, #1");                                    // increment digit count
    emitter.instruction("mov x0, x13");                                         // value = quotient for next iteration
    emitter.instruction("cbnz x0, __rt_itoa_loop");                             // continue while the quotient is non-zero

    // -- prepend minus sign if negative --
    emitter.label("__rt_itoa_sign");
    emitter.instruction("cbz x15, __rt_itoa_emit");                             // skip if not negative
    emitter.instruction("mov w12, #45");                                        // ASCII '-'
    emitter.instruction("strb w12, [x9, #-1]!");                                // store the minus sign ahead of the first digit
    emitter.instruction("add x10, x10, #1");                                    // count the sign in total length

    // -- reserve the exact rendered length and copy the rendering into it --
    emitter.label("__rt_itoa_emit");
    emitter.instruction("stp x9, x10, [sp, #64]");                              // save the first-character pointer and the length across the reservation
    emitter.instruction("mov x0, x10");                                         // reserve exactly as many bytes as the rendering needs
    emitter.instruction("bl __rt_concat_reserve");                              // reserve scratch or heap storage, bounded by the scratch capacity
    emitter.instruction("ldp x9, x10, [sp, #64]");                              // reload the first-character pointer and length after the reservation
    emitter.instruction("mov x1, x0");                                          // the reservation start is the published result pointer
    emitter.instruction("mov x2, x10");                                         // the rendered length is the published result length
    emitter.instruction("mov x13, #0");                                         // start copying at the first rendered character

    emitter.label("__rt_itoa_copy");
    emitter.instruction("cmp x13, x10");                                        // has the whole rendering been copied out?
    emitter.instruction("b.hs __rt_itoa_copied");                               // finish once every character has been copied
    emitter.instruction("ldrb w14, [x9, x13]");                                 // load the next rendered character from the frame buffer
    emitter.instruction("strb w14, [x0, x13]");                                 // store it into the reserved result storage
    emitter.instruction("add x13, x13, #1");                                    // advance the copy index
    emitter.instruction("b __rt_itoa_copy");                                    // copy the next character

    emitter.label("__rt_itoa_copied");
    emitter.instruction("bl __rt_concat_publish");                              // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("ldp x29, x30, [sp, #80]");                             // restore frame pointer and return address
    emitter.instruction("add sp, sp, #96");                                     // release the itoa helper frame
    emitter.instruction("ret");                                                 // return with x1 = pointer and x2 = length
}

/// Emits the `__rt_itoa` runtime helper for x86_64 Linux.
///
/// # ABI
/// Input is expected in `rax`; the function returns the string pointer in `rax` and the length
/// in `rdx`. This is the System V AMD64 ABI convention used on Linux.
///
/// # Implementation notes
/// Mirrors the ARM64 logic: renders digits right-to-left into a frame-local buffer, handles the
/// zero special case, prepends a minus sign for negative values, then copies the rendering into
/// an exact-size `__rt_concat_reserve` reservation and publishes it. The frame keeps its render
/// buffer at `[rbp-128, rbp-64)` and its two spill slots at `[rbp-64]`/`[rbp-56]`, deliberately
/// clear of `[rbp-8]`..`[rbp-24]` so the layout cannot collide with the saved-register slots
/// other runtime emitters expect there.
fn emit_itoa_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: itoa ---");
    emitter.label_global("__rt_itoa");

    // -- set up stack frame --
    emitter.instruction("push rbp");                                            // save the caller frame pointer before using rbp locally
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame pointer for the routine
    emitter.instruction("sub rsp, 128");                                        // reserve the render buffer and spill slots, keeping the stack 16-byte aligned
    emitter.instruction("lea r10, [rbp - 64]");                                 // point the digit cursor just past the end of the render buffer

    // -- initialize counters --
    emitter.instruction("xor ecx, ecx");                                        // digit count = 0
    emitter.instruction("xor r11d, r11d");                                      // negative flag = 0

    // -- handle sign --
    emitter.instruction("test rax, rax");                                       // check whether the input integer is negative
    emitter.instruction("jns __rt_itoa_positive");                              // skip negation when the input is already non-negative
    emitter.instruction("mov r11d, 1");                                         // remember that we need to prepend a minus sign later
    emitter.instruction("neg rax");                                             // negate the value so the digit loop can use unsigned division

    // -- handle zero special case --
    emitter.label("__rt_itoa_positive");
    emitter.instruction("test rax, rax");                                       // check whether the absolute value is zero
    emitter.instruction("jne __rt_itoa_loop");                                  // start the digit extraction loop when the value is non-zero
    emitter.instruction("dec r10");                                             // step the cursor onto the single digit position
    emitter.instruction("mov BYTE PTR [r10], 48");                              // store ASCII '0' into the render buffer
    emitter.instruction("mov ecx, 1");                                          // digit count = 1 for the zero special case
    emitter.instruction("jmp __rt_itoa_sign");                                  // zero is never negative, so the sign check just falls through

    // -- extract digits right-to-left via repeated division by 10 --
    emitter.label("__rt_itoa_loop");
    emitter.instruction("mov esi, 10");                                         // divisor = 10 for decimal digit extraction
    emitter.instruction("xor edx, edx");                                        // clear the high dividend half before unsigned division
    emitter.instruction("div rsi");                                             // quotient -> rax, remainder -> rdx
    emitter.instruction("add dl, 48");                                          // convert the decimal remainder to its ASCII digit
    emitter.instruction("dec r10");                                             // step the cursor left onto the next digit position
    emitter.instruction("mov BYTE PTR [r10], dl");                              // store the digit into the render buffer
    emitter.instruction("inc ecx");                                             // increment the output length after storing one digit
    emitter.instruction("test rax, rax");                                       // check whether more quotient digits remain
    emitter.instruction("jne __rt_itoa_loop");                                  // continue until the quotient reaches zero

    // -- prepend minus sign if negative --
    emitter.label("__rt_itoa_sign");
    emitter.instruction("test r11, r11");                                       // check whether the original value was negative
    emitter.instruction("jz __rt_itoa_emit");                                   // skip sign emission for non-negative values
    emitter.instruction("dec r10");                                             // step the cursor left onto the sign position
    emitter.instruction("mov BYTE PTR [r10], 45");                              // store ASCII '-' before the first digit
    emitter.instruction("inc ecx");                                             // count the sign in the returned string length

    // -- reserve the exact rendered length and copy the rendering into it --
    emitter.label("__rt_itoa_emit");
    emitter.instruction("mov QWORD PTR [rbp - 64], r10");                       // save the first-character pointer across the reservation call
    emitter.instruction("mov QWORD PTR [rbp - 56], rcx");                       // save the rendered length across the reservation call
    emitter.instruction("mov rax, rcx");                                        // reserve exactly as many bytes as the rendering needs
    emitter.instruction("call __rt_concat_reserve");                            // reserve scratch or heap storage, bounded by the scratch capacity
    emitter.instruction("mov r10, QWORD PTR [rbp - 64]");                       // reload the first-character pointer after the reservation
    emitter.instruction("mov rcx, QWORD PTR [rbp - 56]");                       // reload the rendered length after the reservation
    emitter.instruction("mov rdx, rcx");                                        // the rendered length is the published result length
    emitter.instruction("xor r9d, r9d");                                        // start copying at the first rendered character

    emitter.label("__rt_itoa_copy");
    emitter.instruction("cmp r9, rcx");                                         // has the whole rendering been copied out?
    emitter.instruction("jae __rt_itoa_copied");                                // finish once every character has been copied
    emitter.instruction("mov r11b, BYTE PTR [r10 + r9]");                       // load the next rendered character from the frame buffer
    emitter.instruction("mov BYTE PTR [rax + r9], r11b");                       // store it into the reserved result storage
    emitter.instruction("inc r9");                                              // advance the copy index
    emitter.instruction("jmp __rt_itoa_copy");                                  // copy the next character

    emitter.label("__rt_itoa_copied");
    emitter.instruction("call __rt_concat_publish");                            // advance the concat scratch offset only for scratch-backed results
    emitter.instruction("mov rsp, rbp");                                        // release the itoa helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer before returning
    emitter.instruction("ret");                                                 // return to the caller with rax=ptr and rdx=len
}
