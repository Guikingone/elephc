//! Purpose:
//! Emits the `__rt_strnatcmp` and `__rt_strnatcasecmp` runtime helpers implementing PHP's
//! natural-order string comparison (`php_strnatcmp_ex`) for byte strings.
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//! - Invoked by the EIR `strnatcmp`/`strnatcasecmp` lowering through
//!   `strings::lower_binary_string_runtime`.
//!
//! Key details:
//! - Faithful port of PHP's `php_strnatcmp_ex`: leading zeros are skipped once, whitespace is
//!   skipped at each loop head, a run where both sides are digits is compared numerically
//!   (`compare_left` for a leading-zero/fractional run, `compare_right` otherwise), and after a
//!   digit run the current characters are reloaded and compared as bytes WITHOUT a further
//!   whitespace skip. Reading past the end yields a NUL sentinel (matching PHP's reliance on the
//!   trailing `\0`), so a truncated whitespace run stops on a non-space `\0`.
//! - The result is exactly `-1`, `0`, or `+1`. `__rt_strnatcasecmp` upper-folds ASCII letters
//!   in the single-character comparison; both entry points share one core with a fold flag.
//! - Both helpers are leaf routines (no calls) and use only caller-saved registers, so no
//!   stack frame is needed on either target.

use crate::codegen_support::emit::Emitter;
use crate::codegen::platform::Arch;

/// Emits `__rt_strnatcmp` and `__rt_strnatcasecmp` for the host target.
///
/// Register contract (ARM64): x1 = a ptr, x2 = a len, x3 = b ptr, x4 = b len; result in x0.
/// Register contract (x86_64 System V): rdi = a ptr, rsi = a len, rdx = b ptr, rcx = b len;
/// result in rax. Matches the binary-string runtime ABI shared with `strcmp`/`strcasecmp`.
pub fn emit_strnatcmp(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_strnatcmp_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: strnatcmp / strnatcasecmp ---");
    emitter.label_global("__rt_strnatcasecmp");
    emitter.instruction("mov x9, #1");                                          // fold flag = 1 (case-insensitive comparison)
    emitter.instruction("b __rt_natcmp_core");                                  // share the natural-compare core with the folded flag
    emitter.label_global("__rt_strnatcmp");
    emitter.instruction("mov x9, #0");                                          // fold flag = 0 (case-sensitive comparison)

    emitter.label("__rt_natcmp_core");
    // -- empty-string fast paths (PHP compares lengths when either side is empty) --
    emitter.instruction("cbnz x2, __rt_natcmp_a_nonempty");                     // does string a have any bytes?
    emitter.instruction("cbz x4, __rt_natcmp_both_empty");                      // a empty: is b also empty?
    emitter.instruction("mov x0, #-1");                                         // a empty, b non-empty -> a sorts first
    emitter.instruction("ret");                                                 // return -1
    emitter.label("__rt_natcmp_both_empty");
    emitter.instruction("mov x0, #0");                                          // both empty -> equal
    emitter.instruction("ret");                                                 // return 0
    emitter.label("__rt_natcmp_a_nonempty");
    emitter.instruction("cbnz x4, __rt_natcmp_both_nonempty");                  // is string b also non-empty?
    emitter.instruction("mov x0, #1");                                          // a non-empty, b empty -> a sorts last
    emitter.instruction("ret");                                                 // return +1

    emitter.label("__rt_natcmp_both_nonempty");
    emitter.instruction("add x7, x1, x2");                                      // aend = a ptr + a len
    emitter.instruction("add x8, x3, x4");                                      // bend = b ptr + b len
    emitter.instruction("mov x5, x1");                                          // ap = start of a
    emitter.instruction("mov x6, x3");                                          // bp = start of b
    emitter.instruction("ldrb w10, [x5]");                                      // ca = *ap
    emitter.instruction("ldrb w11, [x6]");                                      // cb = *bp

    // -- skip leading zeros once (only when followed by another digit) --
    emitter.label("__rt_natcmp_lz_a");
    emitter.instruction("cmp w10, #48");                                        // is ca the digit '0'?
    emitter.instruction("b.ne __rt_natcmp_lz_b");                               // stop skipping a once ca is not '0'
    emitter.instruction("add x12, x5, #1");                                     // address of the byte after ap
    emitter.instruction("cmp x12, x7");                                         // is that byte still within a?
    emitter.instruction("b.hs __rt_natcmp_lz_b");                               // a trailing '0' at end is not a leading zero
    emitter.instruction("ldrb w13, [x12]");                                     // load the byte after ap
    emitter.instruction("sub w14, w13, #48");                                   // normalize to 0..9 for the digit test
    emitter.instruction("cmp w14, #9");                                         // is the following byte a digit?
    emitter.instruction("b.hi __rt_natcmp_lz_b");                               // '0' before a non-digit is significant
    emitter.instruction("mov x5, x12");                                         // advance ap past the leading zero
    emitter.instruction("mov w10, w13");                                        // ca = new current byte
    emitter.instruction("b __rt_natcmp_lz_a");                                  // keep collapsing leading zeros in a
    emitter.label("__rt_natcmp_lz_b");
    emitter.instruction("cmp w11, #48");                                        // is cb the digit '0'?
    emitter.instruction("b.ne __rt_natcmp_main");                               // stop skipping b once cb is not '0'
    emitter.instruction("add x12, x6, #1");                                     // address of the byte after bp
    emitter.instruction("cmp x12, x8");                                         // is that byte still within b?
    emitter.instruction("b.hs __rt_natcmp_main");                               // a trailing '0' at end is not a leading zero
    emitter.instruction("ldrb w13, [x12]");                                     // load the byte after bp
    emitter.instruction("sub w14, w13, #48");                                   // normalize to 0..9 for the digit test
    emitter.instruction("cmp w14, #9");                                         // is the following byte a digit?
    emitter.instruction("b.hi __rt_natcmp_main");                               // '0' before a non-digit is significant
    emitter.instruction("mov x6, x12");                                         // advance bp past the leading zero
    emitter.instruction("mov w11, w13");                                        // cb = new current byte
    emitter.instruction("b __rt_natcmp_lz_b");                                  // keep collapsing leading zeros in b

    // -- main loop head: skip consecutive whitespace on each side --
    emitter.label("__rt_natcmp_main");
    emitter.label("__rt_natcmp_ws_a");
    emitter.instruction("cmp w10, #32");                                        // is ca a space?
    emitter.instruction("b.eq __rt_natcmp_ws_a_skip");                          // spaces are skipped
    emitter.instruction("sub w14, w10, #9");                                    // map tab..carriage-return (9..13) to 0..4
    emitter.instruction("cmp w14, #4");                                         // is ca a control whitespace byte?
    emitter.instruction("b.hi __rt_natcmp_ws_b");                               // ca is not whitespace -> move to b
    emitter.label("__rt_natcmp_ws_a_skip");
    emitter.instruction("add x5, x5, #1");                                      // advance ap past the whitespace byte
    emitter.instruction("cmp x5, x7");                                          // did ap reach the end of a?
    emitter.instruction("b.hs __rt_natcmp_ws_a_nul");                           // past the end reads the NUL sentinel
    emitter.instruction("ldrb w10, [x5]");                                      // ca = next byte
    emitter.instruction("b __rt_natcmp_ws_a");                                  // keep skipping whitespace in a
    emitter.label("__rt_natcmp_ws_a_nul");
    emitter.instruction("mov w10, #0");                                         // ca = NUL (stops the whitespace run)
    emitter.label("__rt_natcmp_ws_b");
    emitter.instruction("cmp w11, #32");                                        // is cb a space?
    emitter.instruction("b.eq __rt_natcmp_ws_b_skip");                          // spaces are skipped
    emitter.instruction("sub w14, w11, #9");                                    // map tab..carriage-return (9..13) to 0..4
    emitter.instruction("cmp w14, #4");                                         // is cb a control whitespace byte?
    emitter.instruction("b.hi __rt_natcmp_digit_check");                        // cb is not whitespace -> classify the pair
    emitter.label("__rt_natcmp_ws_b_skip");
    emitter.instruction("add x6, x6, #1");                                      // advance bp past the whitespace byte
    emitter.instruction("cmp x6, x8");                                          // did bp reach the end of b?
    emitter.instruction("b.hs __rt_natcmp_ws_b_nul");                           // past the end reads the NUL sentinel
    emitter.instruction("ldrb w11, [x6]");                                      // cb = next byte
    emitter.instruction("b __rt_natcmp_ws_b");                                  // keep skipping whitespace in b
    emitter.label("__rt_natcmp_ws_b_nul");
    emitter.instruction("mov w11, #0");                                         // cb = NUL (stops the whitespace run)

    // -- classify: only a digit/digit pair enters the numeric run --
    emitter.label("__rt_natcmp_digit_check");
    emitter.instruction("sub w14, w10, #48");                                   // normalize ca to 0..9 for the digit test
    emitter.instruction("cmp w14, #9");                                         // is ca a digit?
    emitter.instruction("b.hi __rt_natcmp_char");                               // a non-digit ca falls to the byte comparison
    emitter.instruction("sub w14, w11, #48");                                   // normalize cb to 0..9 for the digit test
    emitter.instruction("cmp w14, #9");                                         // is cb a digit?
    emitter.instruction("b.hi __rt_natcmp_char");                               // a non-digit cb falls to the byte comparison
    emitter.instruction("cmp w10, #48");                                        // does the a-run start with '0'?
    emitter.instruction("b.eq __rt_natcmp_cmp_left");                           // a leading-zero run compares left-aligned
    emitter.instruction("cmp w11, #48");                                        // does the b-run start with '0'?
    emitter.instruction("b.eq __rt_natcmp_cmp_left");                           // a leading-zero run compares left-aligned
    // fall through to compare_right for a non-fractional numeric run

    // -- compare_right: longest digit run wins, else first difference via bias --
    emitter.label("__rt_natcmp_cmp_right");
    emitter.instruction("mov w15, #0");                                         // bias = 0 (no difference seen yet)
    emitter.label("__rt_natcmp_cr_loop");
    emitter.instruction("cmp x5, x7");                                          // is ap at the end of a?
    emitter.instruction("b.hs __rt_natcmp_cr_a_end");                           // a exhausted -> a has no digit here
    emitter.instruction("ldrb w12, [x5]");                                      // load *ap
    emitter.instruction("sub w14, w12, #48");                                   // normalize *ap to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *ap a digit?
    emitter.instruction("b.hi __rt_natcmp_cr_a_end");                           // *ap non-digit -> a-run ended
    emitter.instruction("cmp x6, x8");                                          // is bp at the end of b?
    emitter.instruction("b.hs __rt_natcmp_cr_ret_pos");                         // a digit but b exhausted -> a is greater
    emitter.instruction("ldrb w13, [x6]");                                      // load *bp
    emitter.instruction("sub w14, w13, #48");                                   // normalize *bp to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *bp a digit?
    emitter.instruction("b.hi __rt_natcmp_cr_ret_pos");                         // a digit but b non-digit -> a is greater
    emitter.instruction("cbnz w15, __rt_natcmp_cr_adv");                        // once a bias is set the magnitude is fixed
    emitter.instruction("cmp w12, w13");                                        // compare the current digits
    emitter.instruction("b.lo __rt_natcmp_cr_set_neg");                         // *ap < *bp -> tentative -1
    emitter.instruction("b.hi __rt_natcmp_cr_set_pos");                         // *ap > *bp -> tentative +1
    emitter.instruction("b __rt_natcmp_cr_adv");                                // equal digits keep the current bias
    emitter.label("__rt_natcmp_cr_set_neg");
    emitter.instruction("mov w15, #-1");                                        // record a smaller-magnitude bias
    emitter.instruction("b __rt_natcmp_cr_adv");                                // continue scanning the digit run
    emitter.label("__rt_natcmp_cr_set_pos");
    emitter.instruction("mov w15, #1");                                         // record a larger-magnitude bias
    emitter.label("__rt_natcmp_cr_adv");
    emitter.instruction("add x5, x5, #1");                                      // advance ap within the digit run
    emitter.instruction("add x6, x6, #1");                                      // advance bp within the digit run
    emitter.instruction("b __rt_natcmp_cr_loop");                               // keep comparing digits
    emitter.label("__rt_natcmp_cr_a_end");
    emitter.instruction("cmp x6, x8");                                          // is bp also at the end of b?
    emitter.instruction("b.hs __rt_natcmp_cr_ret_bias");                        // both runs ended -> the bias decides
    emitter.instruction("ldrb w13, [x6]");                                      // load *bp
    emitter.instruction("sub w14, w13, #48");                                   // normalize *bp to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *bp a digit?
    emitter.instruction("b.hi __rt_natcmp_cr_ret_bias");                        // both non-digit -> the bias decides
    emitter.instruction("mov x0, #-1");                                         // a shorter digit run -> a is smaller
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cr_ret_pos");
    emitter.instruction("mov x0, #1");                                          // a longer digit run -> a is greater
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cr_ret_bias");
    emitter.instruction("sxtw x0, w15");                                        // result = bias (sign-extended to 64 bits)
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue

    // -- compare_left: first differing digit wins (leading-zero / fractional run) --
    emitter.label("__rt_natcmp_cmp_left");
    emitter.label("__rt_natcmp_cl_loop");
    emitter.instruction("cmp x5, x7");                                          // is ap at the end of a?
    emitter.instruction("b.hs __rt_natcmp_cl_a_end");                           // a exhausted -> a has no digit here
    emitter.instruction("ldrb w12, [x5]");                                      // load *ap
    emitter.instruction("sub w14, w12, #48");                                   // normalize *ap to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *ap a digit?
    emitter.instruction("b.hi __rt_natcmp_cl_a_end");                           // *ap non-digit -> a-run ended
    emitter.instruction("cmp x6, x8");                                          // is bp at the end of b?
    emitter.instruction("b.hs __rt_natcmp_cl_ret_pos");                         // a digit but b exhausted -> a is greater
    emitter.instruction("ldrb w13, [x6]");                                      // load *bp
    emitter.instruction("sub w14, w13, #48");                                   // normalize *bp to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *bp a digit?
    emitter.instruction("b.hi __rt_natcmp_cl_ret_pos");                         // a digit but b non-digit -> a is greater
    emitter.instruction("cmp w12, w13");                                        // compare the current digits
    emitter.instruction("b.lo __rt_natcmp_cl_ret_neg");                         // *ap < *bp -> a is smaller
    emitter.instruction("b.hi __rt_natcmp_cl_ret_pos");                         // *ap > *bp -> a is greater
    emitter.instruction("add x5, x5, #1");                                      // advance ap within the digit run
    emitter.instruction("add x6, x6, #1");                                      // advance bp within the digit run
    emitter.instruction("b __rt_natcmp_cl_loop");                               // keep comparing digits
    emitter.label("__rt_natcmp_cl_a_end");
    emitter.instruction("cmp x6, x8");                                          // is bp also at the end of b?
    emitter.instruction("b.hs __rt_natcmp_cl_ret_zero");                        // both runs ended equal -> 0
    emitter.instruction("ldrb w13, [x6]");                                      // load *bp
    emitter.instruction("sub w14, w13, #48");                                   // normalize *bp to 0..9
    emitter.instruction("cmp w14, #9");                                         // is *bp a digit?
    emitter.instruction("b.hi __rt_natcmp_cl_ret_zero");                        // both non-digit -> 0
    emitter.instruction("mov x0, #-1");                                         // a ended first -> a is smaller
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_pos");
    emitter.instruction("mov x0, #1");                                          // a greater digit -> a is greater
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_neg");
    emitter.instruction("mov x0, #-1");                                         // a smaller digit -> a is smaller
    emitter.instruction("b __rt_natcmp_after_cmp");                             // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_zero");
    emitter.instruction("mov x0, #0");                                          // equal-length equal-value runs -> 0

    // -- digit-run epilogue: nonzero result returns; else resolve by run ends --
    emitter.label("__rt_natcmp_after_cmp");
    emitter.instruction("cbnz x0, __rt_natcmp_ret");                            // a decided numeric run returns immediately
    emitter.instruction("cmp x5, x7");                                          // did the a-run consume all of a?
    emitter.instruction("b.ne __rt_natcmp_ac_a_in");                            // a still has bytes -> inspect b
    emitter.instruction("cmp x6, x8");                                          // did the b-run consume all of b?
    emitter.instruction("b.ne __rt_natcmp_neg1");                               // a ended, b remains -> a is smaller
    emitter.instruction("mov x0, #0");                                          // both strings fully consumed -> equal
    emitter.instruction("b __rt_natcmp_ret");                                   // return 0
    emitter.label("__rt_natcmp_ac_a_in");
    emitter.instruction("cmp x6, x8");                                          // did the b-run consume all of b?
    emitter.instruction("b.eq __rt_natcmp_pos1");                               // b ended, a remains -> a is greater
    emitter.instruction("ldrb w10, [x5]");                                      // reload ca at the post-run position
    emitter.instruction("ldrb w11, [x6]");                                      // reload cb at the post-run position
    // fall through to the byte comparison WITHOUT re-skipping whitespace

    // -- single-byte comparison (with optional ASCII upper-fold) plus advance --
    emitter.label("__rt_natcmp_char");
    emitter.instruction("cbz x9, __rt_natcmp_char_cmp");                        // skip case folding when comparing case-sensitively
    emitter.instruction("sub w14, w10, #97");                                   // ca - 'a' for the lowercase-range test
    emitter.instruction("cmp w14, #25");                                        // is ca an ASCII lowercase letter?
    emitter.instruction("b.hi __rt_natcmp_fold_b");                             // leave non-letters unchanged
    emitter.instruction("sub w10, w10, #32");                                   // upper-fold ca
    emitter.label("__rt_natcmp_fold_b");
    emitter.instruction("sub w14, w11, #97");                                   // cb - 'a' for the lowercase-range test
    emitter.instruction("cmp w14, #25");                                        // is cb an ASCII lowercase letter?
    emitter.instruction("b.hi __rt_natcmp_char_cmp");                           // leave non-letters unchanged
    emitter.instruction("sub w11, w11, #32");                                   // upper-fold cb
    emitter.label("__rt_natcmp_char_cmp");
    emitter.instruction("cmp w10, w11");                                        // compare the two current bytes (unsigned)
    emitter.instruction("b.lo __rt_natcmp_neg1");                               // ca < cb -> a is smaller
    emitter.instruction("b.hi __rt_natcmp_pos1");                               // ca > cb -> a is greater
    emitter.instruction("add x5, x5, #1");                                      // bytes equal -> advance ap
    emitter.instruction("add x6, x6, #1");                                      // bytes equal -> advance bp
    emitter.instruction("cmp x5, x7");                                          // did ap reach the end of a?
    emitter.instruction("b.lo __rt_natcmp_char_a_in");                          // a still has bytes -> inspect b
    emitter.instruction("cmp x6, x8");                                          // did bp reach the end of b?
    emitter.instruction("b.lo __rt_natcmp_neg1");                               // a ended, b remains -> a is smaller
    emitter.instruction("mov x0, #0");                                          // both ended together -> equal
    emitter.instruction("b __rt_natcmp_ret");                                   // return 0
    emitter.label("__rt_natcmp_char_a_in");
    emitter.instruction("cmp x6, x8");                                          // did bp reach the end of b?
    emitter.instruction("b.hs __rt_natcmp_pos1");                               // b ended, a remains -> a is greater
    emitter.instruction("ldrb w10, [x5]");                                      // reload ca for the next iteration
    emitter.instruction("ldrb w11, [x6]");                                      // reload cb for the next iteration
    emitter.instruction("b __rt_natcmp_main");                                  // loop back through the whitespace skip

    emitter.label("__rt_natcmp_neg1");
    emitter.instruction("mov x0, #-1");                                         // final result: a sorts before b
    emitter.instruction("b __rt_natcmp_ret");                                   // return -1
    emitter.label("__rt_natcmp_pos1");
    emitter.instruction("mov x0, #1");                                          // final result: a sorts after b
    emitter.label("__rt_natcmp_ret");
    emitter.instruction("ret");                                                 // return the natural-order comparison result
}

/// Emits the x86_64 System V implementation of `__rt_strnatcmp` / `__rt_strnatcasecmp`.
///
/// Leaf routine using only caller-saved registers. After setup: rdi = ap, rdx = bp,
/// r10 = aend, r11 = bend, r8 = fold flag, r9b = ca, sil = cb; rax and rcx are scratch (eax
/// carries the bias during a `compare_right` run). Result -1 uses a full-width `mov rax, -1`
/// so the sign survives in the 64-bit return register.
fn emit_strnatcmp_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: strnatcmp / strnatcasecmp ---");
    emitter.label_global("__rt_strnatcasecmp");
    emitter.instruction("mov r8d, 1");                                          // fold flag = 1 (case-insensitive comparison)
    emitter.instruction("jmp __rt_natcmp_core");                                // share the natural-compare core with the folded flag
    emitter.label_global("__rt_strnatcmp");
    emitter.instruction("xor r8d, r8d");                                        // fold flag = 0 (case-sensitive comparison)

    // -- empty-string fast paths (rsi = a len, rcx = b len on entry) --
    emitter.label("__rt_natcmp_core");
    emitter.instruction("test rsi, rsi");                                       // does string a have any bytes?
    emitter.instruction("jne __rt_natcmp_a_nonempty");                          // branch when a is non-empty
    emitter.instruction("test rcx, rcx");                                       // a empty: is b also empty?
    emitter.instruction("je __rt_natcmp_both_empty");                           // both empty -> equal
    emitter.instruction("mov rax, -1");                                         // a empty, b non-empty -> a sorts first
    emitter.instruction("ret");                                                 // return -1
    emitter.label("__rt_natcmp_both_empty");
    emitter.instruction("xor eax, eax");                                        // both empty -> equal
    emitter.instruction("ret");                                                 // return 0
    emitter.label("__rt_natcmp_a_nonempty");
    emitter.instruction("test rcx, rcx");                                       // is string b also non-empty?
    emitter.instruction("jne __rt_natcmp_both_nonempty");                       // branch when both are non-empty
    emitter.instruction("mov eax, 1");                                          // a non-empty, b empty -> a sorts last
    emitter.instruction("ret");                                                 // return +1

    emitter.label("__rt_natcmp_both_nonempty");
    emitter.instruction("lea r10, [rdi + rsi]");                                // aend = a ptr + a len
    emitter.instruction("lea r11, [rdx + rcx]");                                // bend = b ptr + b len
    emitter.instruction("movzx r9d, BYTE PTR [rdi]");                           // ca = *ap (ap stays in rdi)
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // cb = *bp (bp stays in rdx)

    // -- skip leading zeros once (only when followed by another digit) --
    emitter.label("__rt_natcmp_lz_a");
    emitter.instruction("cmp r9b, 48");                                         // is ca the digit '0'?
    emitter.instruction("jne __rt_natcmp_lz_b");                                // stop skipping a once ca is not '0'
    emitter.instruction("lea rax, [rdi + 1]");                                  // address of the byte after ap
    emitter.instruction("cmp rax, r10");                                        // is that byte still within a?
    emitter.instruction("jae __rt_natcmp_lz_b");                                // a trailing '0' at end is not a leading zero
    emitter.instruction("movzx ecx, BYTE PTR [rax]");                           // load the byte after ap
    emitter.instruction("lea r9d, [rcx - 48]");                                 // normalize to 0..9 for the digit test
    emitter.instruction("cmp r9d, 9");                                          // is the following byte a digit?
    emitter.instruction("ja __rt_natcmp_lz_a_stop");                            // '0' before a non-digit is significant
    emitter.instruction("mov rdi, rax");                                        // advance ap past the leading zero
    emitter.instruction("mov r9d, ecx");                                        // ca = new current byte
    emitter.instruction("jmp __rt_natcmp_lz_a");                                // keep collapsing leading zeros in a
    emitter.label("__rt_natcmp_lz_a_stop");
    emitter.instruction("movzx r9d, BYTE PTR [rdi]");                           // restore ca to the unadvanced '0' byte
    emitter.label("__rt_natcmp_lz_b");
    emitter.instruction("cmp sil, 48");                                         // is cb the digit '0'?
    emitter.instruction("jne __rt_natcmp_main");                                // stop skipping b once cb is not '0'
    emitter.instruction("lea rax, [rdx + 1]");                                  // address of the byte after bp
    emitter.instruction("cmp rax, r11");                                        // is that byte still within b?
    emitter.instruction("jae __rt_natcmp_main");                                // a trailing '0' at end is not a leading zero
    emitter.instruction("movzx ecx, BYTE PTR [rax]");                           // load the byte after bp
    emitter.instruction("lea esi, [rcx - 48]");                                 // normalize to 0..9 for the digit test
    emitter.instruction("cmp esi, 9");                                          // is the following byte a digit?
    emitter.instruction("ja __rt_natcmp_lz_b_stop");                            // '0' before a non-digit is significant
    emitter.instruction("mov rdx, rax");                                        // advance bp past the leading zero
    emitter.instruction("mov esi, ecx");                                        // cb = new current byte
    emitter.instruction("jmp __rt_natcmp_lz_b");                                // keep collapsing leading zeros in b
    emitter.label("__rt_natcmp_lz_b_stop");
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // restore cb to the unadvanced '0' byte

    // -- main loop head: skip consecutive whitespace on each side --
    emitter.label("__rt_natcmp_main");
    emitter.label("__rt_natcmp_ws_a");
    emitter.instruction("cmp r9b, 32");                                         // is ca a space?
    emitter.instruction("je __rt_natcmp_ws_a_skip");                            // spaces are skipped
    emitter.instruction("lea eax, [r9 - 9]");                                   // map tab..carriage-return (9..13) to 0..4
    emitter.instruction("cmp eax, 4");                                          // is ca a control whitespace byte?
    emitter.instruction("ja __rt_natcmp_ws_b");                                 // ca is not whitespace -> move to b
    emitter.label("__rt_natcmp_ws_a_skip");
    emitter.instruction("add rdi, 1");                                          // advance ap past the whitespace byte
    emitter.instruction("cmp rdi, r10");                                        // did ap reach the end of a?
    emitter.instruction("jae __rt_natcmp_ws_a_nul");                            // past the end reads the NUL sentinel
    emitter.instruction("movzx r9d, BYTE PTR [rdi]");                           // ca = next byte
    emitter.instruction("jmp __rt_natcmp_ws_a");                                // keep skipping whitespace in a
    emitter.label("__rt_natcmp_ws_a_nul");
    emitter.instruction("xor r9d, r9d");                                        // ca = NUL (stops the whitespace run)
    emitter.label("__rt_natcmp_ws_b");
    emitter.instruction("cmp sil, 32");                                         // is cb a space?
    emitter.instruction("je __rt_natcmp_ws_b_skip");                            // spaces are skipped
    emitter.instruction("lea eax, [rsi - 9]");                                  // map tab..carriage-return (9..13) to 0..4
    emitter.instruction("cmp eax, 4");                                          // is cb a control whitespace byte?
    emitter.instruction("ja __rt_natcmp_digit_check");                          // cb is not whitespace -> classify the pair
    emitter.label("__rt_natcmp_ws_b_skip");
    emitter.instruction("add rdx, 1");                                          // advance bp past the whitespace byte
    emitter.instruction("cmp rdx, r11");                                        // did bp reach the end of b?
    emitter.instruction("jae __rt_natcmp_ws_b_nul");                            // past the end reads the NUL sentinel
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // cb = next byte
    emitter.instruction("jmp __rt_natcmp_ws_b");                                // keep skipping whitespace in b
    emitter.label("__rt_natcmp_ws_b_nul");
    emitter.instruction("xor esi, esi");                                        // cb = NUL (stops the whitespace run)

    // -- classify: only a digit/digit pair enters the numeric run --
    emitter.label("__rt_natcmp_digit_check");
    emitter.instruction("lea eax, [r9 - 48]");                                  // normalize ca to 0..9 for the digit test
    emitter.instruction("cmp eax, 9");                                          // is ca a digit?
    emitter.instruction("ja __rt_natcmp_char");                                 // a non-digit ca falls to the byte comparison
    emitter.instruction("lea eax, [rsi - 48]");                                 // normalize cb to 0..9 for the digit test
    emitter.instruction("cmp eax, 9");                                          // is cb a digit?
    emitter.instruction("ja __rt_natcmp_char");                                 // a non-digit cb falls to the byte comparison
    emitter.instruction("cmp r9b, 48");                                         // does the a-run start with '0'?
    emitter.instruction("je __rt_natcmp_cmp_left");                             // a leading-zero run compares left-aligned
    emitter.instruction("cmp sil, 48");                                         // does the b-run start with '0'?
    emitter.instruction("je __rt_natcmp_cmp_left");                             // a leading-zero run compares left-aligned
    // fall through to compare_right for a non-fractional numeric run

    // -- compare_right: longest digit run wins, else first difference via bias --
    emitter.label("__rt_natcmp_cmp_right");
    emitter.instruction("xor eax, eax");                                        // bias = 0 (no difference seen yet)
    emitter.label("__rt_natcmp_cr_loop");
    emitter.instruction("cmp rdi, r10");                                        // is ap at the end of a?
    emitter.instruction("jae __rt_natcmp_cr_a_end");                            // a exhausted -> a has no digit here
    emitter.instruction("movzx ecx, BYTE PTR [rdi]");                           // load *ap
    emitter.instruction("lea r9d, [rcx - 48]");                                 // normalize *ap to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *ap a digit?
    emitter.instruction("ja __rt_natcmp_cr_a_end");                             // *ap non-digit -> a-run ended
    emitter.instruction("cmp rdx, r11");                                        // is bp at the end of b?
    emitter.instruction("jae __rt_natcmp_cr_ret_pos");                          // a digit but b exhausted -> a is greater
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // load *bp
    emitter.instruction("lea r9d, [rsi - 48]");                                 // normalize *bp to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *bp a digit?
    emitter.instruction("ja __rt_natcmp_cr_ret_pos");                           // a digit but b non-digit -> a is greater
    emitter.instruction("test eax, eax");                                       // is a bias already fixed?
    emitter.instruction("jne __rt_natcmp_cr_adv");                              // once biased the magnitude is decided
    emitter.instruction("cmp cl, sil");                                         // compare the current digits (*ap vs *bp)
    emitter.instruction("jb __rt_natcmp_cr_set_neg");                           // *ap < *bp -> tentative -1
    emitter.instruction("ja __rt_natcmp_cr_set_pos");                           // *ap > *bp -> tentative +1
    emitter.instruction("jmp __rt_natcmp_cr_adv");                              // equal digits keep the current bias
    emitter.label("__rt_natcmp_cr_set_neg");
    emitter.instruction("mov eax, -1");                                         // record a smaller-magnitude bias
    emitter.instruction("jmp __rt_natcmp_cr_adv");                              // continue scanning the digit run
    emitter.label("__rt_natcmp_cr_set_pos");
    emitter.instruction("mov eax, 1");                                          // record a larger-magnitude bias
    emitter.label("__rt_natcmp_cr_adv");
    emitter.instruction("add rdi, 1");                                          // advance ap within the digit run
    emitter.instruction("add rdx, 1");                                          // advance bp within the digit run
    emitter.instruction("jmp __rt_natcmp_cr_loop");                             // keep comparing digits
    emitter.label("__rt_natcmp_cr_a_end");
    emitter.instruction("cmp rdx, r11");                                        // is bp also at the end of b?
    emitter.instruction("jae __rt_natcmp_cr_ret_bias");                         // both runs ended -> the bias decides
    emitter.instruction("movzx ecx, BYTE PTR [rdx]");                           // load *bp
    emitter.instruction("lea r9d, [rcx - 48]");                                 // normalize *bp to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *bp a digit?
    emitter.instruction("ja __rt_natcmp_cr_ret_bias");                          // both non-digit -> the bias decides
    emitter.instruction("mov rax, -1");                                         // a shorter digit run -> a is smaller
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cr_ret_pos");
    emitter.instruction("mov eax, 1");                                          // a longer digit run -> a is greater
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cr_ret_bias");
    emitter.instruction("movsxd rax, eax");                                     // result = bias (sign-extended to 64 bits)
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue

    // -- compare_left: first differing digit wins (leading-zero / fractional run) --
    emitter.label("__rt_natcmp_cmp_left");
    emitter.label("__rt_natcmp_cl_loop");
    emitter.instruction("cmp rdi, r10");                                        // is ap at the end of a?
    emitter.instruction("jae __rt_natcmp_cl_a_end");                            // a exhausted -> a has no digit here
    emitter.instruction("movzx ecx, BYTE PTR [rdi]");                           // load *ap
    emitter.instruction("lea r9d, [rcx - 48]");                                 // normalize *ap to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *ap a digit?
    emitter.instruction("ja __rt_natcmp_cl_a_end");                             // *ap non-digit -> a-run ended
    emitter.instruction("cmp rdx, r11");                                        // is bp at the end of b?
    emitter.instruction("jae __rt_natcmp_cl_ret_pos");                          // a digit but b exhausted -> a is greater
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // load *bp
    emitter.instruction("lea r9d, [rsi - 48]");                                 // normalize *bp to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *bp a digit?
    emitter.instruction("ja __rt_natcmp_cl_ret_pos");                           // a digit but b non-digit -> a is greater
    emitter.instruction("cmp cl, sil");                                         // compare the current digits (*ap vs *bp)
    emitter.instruction("jb __rt_natcmp_cl_ret_neg");                           // *ap < *bp -> a is smaller
    emitter.instruction("ja __rt_natcmp_cl_ret_pos");                           // *ap > *bp -> a is greater
    emitter.instruction("add rdi, 1");                                          // advance ap within the digit run
    emitter.instruction("add rdx, 1");                                          // advance bp within the digit run
    emitter.instruction("jmp __rt_natcmp_cl_loop");                             // keep comparing digits
    emitter.label("__rt_natcmp_cl_a_end");
    emitter.instruction("cmp rdx, r11");                                        // is bp also at the end of b?
    emitter.instruction("jae __rt_natcmp_cl_ret_zero");                         // both runs ended equal -> 0
    emitter.instruction("movzx ecx, BYTE PTR [rdx]");                           // load *bp
    emitter.instruction("lea r9d, [rcx - 48]");                                 // normalize *bp to 0..9
    emitter.instruction("cmp r9d, 9");                                          // is *bp a digit?
    emitter.instruction("ja __rt_natcmp_cl_ret_zero");                          // both non-digit -> 0
    emitter.instruction("mov rax, -1");                                         // a ended first -> a is smaller
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_pos");
    emitter.instruction("mov eax, 1");                                          // a greater digit -> a is greater
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_neg");
    emitter.instruction("mov rax, -1");                                         // a smaller digit -> a is smaller
    emitter.instruction("jmp __rt_natcmp_after_cmp");                           // hand the result to the digit-run epilogue
    emitter.label("__rt_natcmp_cl_ret_zero");
    emitter.instruction("xor eax, eax");                                        // equal-length equal-value runs -> 0

    // -- digit-run epilogue: nonzero result returns; else resolve by run ends --
    emitter.label("__rt_natcmp_after_cmp");
    emitter.instruction("test rax, rax");                                       // did the numeric run decide the order?
    emitter.instruction("jne __rt_natcmp_ret");                                 // a decided numeric run returns immediately
    emitter.instruction("cmp rdi, r10");                                        // did the a-run consume all of a?
    emitter.instruction("jne __rt_natcmp_ac_a_in");                             // a still has bytes -> inspect b
    emitter.instruction("cmp rdx, r11");                                        // did the b-run consume all of b?
    emitter.instruction("jne __rt_natcmp_neg1");                                // a ended, b remains -> a is smaller
    emitter.instruction("xor eax, eax");                                        // both strings fully consumed -> equal
    emitter.instruction("jmp __rt_natcmp_ret");                                 // return 0
    emitter.label("__rt_natcmp_ac_a_in");
    emitter.instruction("cmp rdx, r11");                                        // did the b-run consume all of b?
    emitter.instruction("je __rt_natcmp_pos1");                                 // b ended, a remains -> a is greater
    emitter.instruction("movzx r9d, BYTE PTR [rdi]");                           // reload ca at the post-run position
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // reload cb at the post-run position
    // fall through to the byte comparison WITHOUT re-skipping whitespace

    // -- single-byte comparison (with optional ASCII upper-fold) plus advance --
    emitter.label("__rt_natcmp_char");
    emitter.instruction("test r8, r8");                                         // is case folding disabled?
    emitter.instruction("je __rt_natcmp_char_cmp");                             // skip folding for case-sensitive compares
    emitter.instruction("lea eax, [r9 - 97]");                                  // ca - 'a' for the lowercase-range test
    emitter.instruction("cmp eax, 25");                                         // is ca an ASCII lowercase letter?
    emitter.instruction("ja __rt_natcmp_fold_b");                               // leave non-letters unchanged
    emitter.instruction("sub r9d, 32");                                         // upper-fold ca
    emitter.label("__rt_natcmp_fold_b");
    emitter.instruction("lea eax, [rsi - 97]");                                 // cb - 'a' for the lowercase-range test
    emitter.instruction("cmp eax, 25");                                         // is cb an ASCII lowercase letter?
    emitter.instruction("ja __rt_natcmp_char_cmp");                             // leave non-letters unchanged
    emitter.instruction("sub esi, 32");                                         // upper-fold cb
    emitter.label("__rt_natcmp_char_cmp");
    emitter.instruction("cmp r9b, sil");                                        // compare the two current bytes (unsigned)
    emitter.instruction("jb __rt_natcmp_neg1");                                 // ca < cb -> a is smaller
    emitter.instruction("ja __rt_natcmp_pos1");                                 // ca > cb -> a is greater
    emitter.instruction("add rdi, 1");                                          // bytes equal -> advance ap
    emitter.instruction("add rdx, 1");                                          // bytes equal -> advance bp
    emitter.instruction("cmp rdi, r10");                                        // did ap reach the end of a?
    emitter.instruction("jb __rt_natcmp_char_a_in");                            // a still has bytes -> inspect b
    emitter.instruction("cmp rdx, r11");                                        // did bp reach the end of b?
    emitter.instruction("jb __rt_natcmp_neg1");                                 // a ended, b remains -> a is smaller
    emitter.instruction("xor eax, eax");                                        // both ended together -> equal
    emitter.instruction("jmp __rt_natcmp_ret");                                 // return 0
    emitter.label("__rt_natcmp_char_a_in");
    emitter.instruction("cmp rdx, r11");                                        // did bp reach the end of b?
    emitter.instruction("jae __rt_natcmp_pos1");                                // b ended, a remains -> a is greater
    emitter.instruction("movzx r9d, BYTE PTR [rdi]");                           // reload ca for the next iteration
    emitter.instruction("movzx esi, BYTE PTR [rdx]");                           // reload cb for the next iteration
    emitter.instruction("jmp __rt_natcmp_main");                                // loop back through the whitespace skip

    emitter.label("__rt_natcmp_neg1");
    emitter.instruction("mov rax, -1");                                         // final result: a sorts before b
    emitter.instruction("jmp __rt_natcmp_ret");                                 // return -1
    emitter.label("__rt_natcmp_pos1");
    emitter.instruction("mov eax, 1");                                          // final result: a sorts after b
    emitter.label("__rt_natcmp_ret");
    emitter.instruction("ret");                                                 // return the natural-order comparison result
}
