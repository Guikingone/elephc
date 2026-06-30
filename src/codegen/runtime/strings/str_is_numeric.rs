//! Purpose:
//! Emits the `__rt_str_is_numeric` runtime helper for PHP numeric-string detection.
//! Used by backed-enum `from()`/`tryFrom()` to decide whether a string argument can be
//! coerced to the integer backing type at runtime.
//!
//! Called from:
//! - `crate::codegen::runtime::emitters::emit_runtime()` via `crate::codegen::runtime::strings`.
//!
//! Key details:
//! - Input follows the active string-result convention: AArch64 uses `x1`/`x2`,
//!   x86_64 uses `rax`/`rdx`.
//! - Returns 1 in the integer result register (`x0`/`rax`) when the entire string is a
//!   valid PHP numeric string, 0 otherwise.
//! - PHP `is_numeric()` allows leading whitespace but rejects trailing whitespace,
//!   an optional leading sign, at least one digit, an optional fractional part, and an
//!   optional exponent. The whole input must be consumed.

use crate::codegen::{emit::Emitter, platform::Arch};

/// Emits `__rt_str_is_numeric`: reports whether a PHP string is fully numeric.
///
/// Reads the string from the active string-result registers (`x1`/`x2` on AArch64,
/// `rax`/`rdx` on x86_64) and returns 1 in the integer result register when the string
/// matches PHP's numeric-string grammar (leading whitespace, optional sign, digits,
/// optional fraction, optional exponent, end-of-string), 0 otherwise.
pub fn emit_str_is_numeric(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_str_is_numeric_linux_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: str_is_numeric ---");
    emitter.label_global("__rt_str_is_numeric");

    // -- default to non-numeric and reject empty strings --
    emitter.instruction("mov x0, #0");                                          // default result is non-numeric
    emitter.instruction("cbz x2, __rt_str_is_numeric_done");                    // empty strings are not numeric

    // -- skip leading whitespace --
    emitter.instruction("mov x9, #0");                                          // initialize the source scan index
    emitter.label("__rt_str_is_numeric_ws_loop");
    emitter.instruction("cmp x9, x2");                                          // check whether the scan reached the string length
    emitter.instruction("b.ge __rt_str_is_numeric_done");                       // a whitespace-only string is not numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // load the current leading byte
    emitter.instruction("cmp w10, #32");                                        // ASCII space
    emitter.instruction("b.eq __rt_str_is_numeric_ws_advance");                 // consume one leading space
    emitter.instruction("sub w11, w10, #9");                                    // normalize ASCII tab/newline/form-feed/carriage-return range
    emitter.instruction("cmp w11, #4");                                         // values 9 through 13 are accepted leading whitespace
    emitter.instruction("b.ls __rt_str_is_numeric_ws_advance");                 // consume one accepted control-whitespace byte
    emitter.instruction("b __rt_str_is_numeric_sign");                          // start sign handling after the leading whitespace
    emitter.label("__rt_str_is_numeric_ws_advance");
    emitter.instruction("add x9, x9, #1");                                      // advance past the accepted whitespace byte
    emitter.instruction("b __rt_str_is_numeric_ws_loop");                       // continue scanning leading whitespace

    // -- optional sign --
    emitter.label("__rt_str_is_numeric_sign");
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the first non-whitespace byte
    emitter.instruction("cmp w10, #45");                                        // is it '-'?
    emitter.instruction("b.eq __rt_str_is_numeric_sign_consume");               // consume the minus sign
    emitter.instruction("cmp w10, #43");                                        // is it '+'?
    emitter.instruction("b.eq __rt_str_is_numeric_sign_consume");               // consume the plus sign
    emitter.instruction("b __rt_str_is_numeric_int_start");                     // no sign → start integer digits
    emitter.label("__rt_str_is_numeric_sign_consume");
    emitter.instruction("add x9, x9, #1");                                      // consume the sign byte
    emitter.instruction("cmp x9, x2");                                          // is there anything after the sign?
    emitter.instruction("b.ge __rt_str_is_numeric_done");                       // a bare sign is not numeric

    // -- integer part: at least one digit is mandatory --
    emitter.label("__rt_str_is_numeric_int_start");
    emitter.instruction("ldrb w10, [x1, x9]");                                  // load the first integer-part byte
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_done");                       // the first integer byte must be a digit
    emitter.instruction("add x9, x9, #1");                                      // consume the first integer digit

    // -- remaining integer digits --
    emitter.label("__rt_str_is_numeric_int_loop");
    emitter.instruction("cmp x9, x2");                                          // end of input?
    emitter.instruction("b.ge __rt_str_is_numeric_ok");                         // a pure integer is numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the next integer byte
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_after_int");                  // non-digit → check fraction or exponent
    emitter.instruction("add x9, x9, #1");                                      // consume the integer digit
    emitter.instruction("b __rt_str_is_numeric_int_loop");                      // continue scanning integer digits

    // -- after integer part: fraction or exponent or end --
    emitter.label("__rt_str_is_numeric_after_int");
    emitter.instruction("cmp w10, #46");                                        // is it '.'?
    emitter.instruction("b.eq __rt_str_is_numeric_frac_start");                 // start the fractional part
    emitter.instruction("cmp w10, #101");                                       // is it 'e'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_sign");                   // start the exponent
    emitter.instruction("cmp w10, #69");                                        // is it 'E'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_sign");                   // start the exponent
    emitter.instruction("b __rt_str_is_numeric_done");                          // any other trailing byte → not numeric

    // -- fractional part: at least one digit is required after the dot --
    emitter.label("__rt_str_is_numeric_frac_start");
    emitter.instruction("add x9, x9, #1");                                      // consume the '.'
    emitter.instruction("cmp x9, x2");                                          // is there a digit after the dot?
    emitter.instruction("b.ge __rt_str_is_numeric_done");                       // a bare 'X.' is not numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the first fractional byte
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_done");                       // need at least one fractional digit
    emitter.instruction("add x9, x9, #1");                                      // consume the first fractional digit

    emitter.label("__rt_str_is_numeric_frac_loop");
    emitter.instruction("cmp x9, x2");                                          // end of input?
    emitter.instruction("b.ge __rt_str_is_numeric_ok");                         // a fraction-only number is numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the next fractional byte
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_after_frac");                 // non-digit → check exponent
    emitter.instruction("add x9, x9, #1");                                      // consume the fractional digit
    emitter.instruction("b __rt_str_is_numeric_frac_loop");                     // continue scanning fractional digits

    // -- after fraction: exponent or end --
    emitter.label("__rt_str_is_numeric_after_frac");
    emitter.instruction("cmp w10, #101");                                       // is it 'e'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_sign");                   // start the exponent
    emitter.instruction("cmp w10, #69");                                        // is it 'E'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_sign");                   // start the exponent
    emitter.instruction("b __rt_str_is_numeric_done");                          // any other trailing byte → not numeric

    // -- exponent: optional sign then at least one digit --
    emitter.label("__rt_str_is_numeric_exp_sign");
    emitter.instruction("add x9, x9, #1");                                      // consume the 'e' or 'E'
    emitter.instruction("cmp x9, x2");                                          // is there anything after the exponent marker?
    emitter.instruction("b.ge __rt_str_is_numeric_done");                       // a bare 'Xe' is not numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the byte after the exponent marker
    emitter.instruction("cmp w10, #43");                                        // optional '+'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_consume_sign");           // consume the exponent plus sign
    emitter.instruction("cmp w10, #45");                                        // optional '-'?
    emitter.instruction("b.eq __rt_str_is_numeric_exp_consume_sign");           // consume the exponent minus sign
    emitter.instruction("b __rt_str_is_numeric_exp_first_digit");               // continue to the first exponent digit
    emitter.label("__rt_str_is_numeric_exp_consume_sign");
    emitter.instruction("add x9, x9, #1");                                      // consume the exponent sign
    emitter.instruction("cmp x9, x2");                                          // is there a digit after the sign?
    emitter.instruction("b.ge __rt_str_is_numeric_done");                       // a bare 'e+'/'e-' is not numeric

    emitter.label("__rt_str_is_numeric_exp_first_digit");
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the first exponent digit
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_done");                       // need at least one exponent digit
    emitter.instruction("add x9, x9, #1");                                      // consume the first exponent digit

    emitter.label("__rt_str_is_numeric_exp_loop");
    emitter.instruction("cmp x9, x2");                                          // end of input?
    emitter.instruction("b.ge __rt_str_is_numeric_ok");                         // a valid exponent reached end-of-string → numeric
    emitter.instruction("ldrb w10, [x1, x9]");                                  // peek the next exponent byte
    emitter.instruction("sub w11, w10, #48");                                   // normalize to a candidate digit
    emitter.instruction("cmp w11, #9");                                         // 0..9 range
    emitter.instruction("b.hi __rt_str_is_numeric_done");                       // any non-digit after exponent digits → not numeric
    emitter.instruction("add x9, x9, #1");                                      // consume the exponent digit
    emitter.instruction("b __rt_str_is_numeric_exp_loop");                      // continue scanning exponent digits

    emitter.label("__rt_str_is_numeric_ok");
    emitter.instruction("mov x0, #1");                                          // signal that the string is numeric
    emitter.label("__rt_str_is_numeric_done");
    emitter.instruction("ret");                                                 // return the numeric flag in x0
}

/// Emits the Linux x86_64 `__rt_str_is_numeric` runtime helper.
///
/// Reads the string from the active string-result registers (`rax`/`rdx`) and returns 1
/// in `rax` when the string matches PHP's numeric-string grammar, 0 otherwise. The grammar
/// allows leading whitespace, an optional sign, at least one digit, an optional fraction,
/// and an optional exponent, with the whole input consumed.
fn emit_str_is_numeric_linux_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: str_is_numeric ---");
    emitter.label_global("__rt_str_is_numeric");

    // -- default to non-numeric and reject empty strings --
    emitter.instruction("mov rax, 0");                                          // default result is non-numeric
    emitter.instruction("test rdx, rdx");                                       // is the string length zero?
    emitter.instruction("je __rt_str_is_numeric_done_linux_x86_64");            // empty strings are not numeric

    // -- skip leading whitespace --
    emitter.instruction("mov rcx, 0");                                          // initialize the source scan index
    emitter.label("__rt_str_is_numeric_ws_loop_linux_x86_64");
    emitter.instruction("cmp rcx, rdx");                                        // check whether the scan reached the string length
    emitter.instruction("jae __rt_str_is_numeric_done_linux_x86_64");           // a whitespace-only string is not numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // load the current leading byte
    emitter.instruction("cmp r8d, 32");                                         // ASCII space
    emitter.instruction("je __rt_str_is_numeric_ws_advance_linux_x86_64");      // consume one leading space
    emitter.instruction("sub r8d, 9");                                          // normalize ASCII tab/newline/form-feed/carriage-return range
    emitter.instruction("cmp r8d, 4");                                          // values 9 through 13 are accepted leading whitespace
    emitter.instruction("jbe __rt_str_is_numeric_ws_advance_linux_x86_64");     // consume one accepted control-whitespace byte
    emitter.instruction("jmp __rt_str_is_numeric_sign_linux_x86_64");           // start sign handling after the leading whitespace
    emitter.label("__rt_str_is_numeric_ws_advance_linux_x86_64");
    emitter.instruction("add rcx, 1");                                          // advance past the accepted whitespace byte
    emitter.instruction("jmp __rt_str_is_numeric_ws_loop_linux_x86_64");        // continue scanning leading whitespace

    // -- optional sign --
    emitter.label("__rt_str_is_numeric_sign_linux_x86_64");
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the first non-whitespace byte
    emitter.instruction("cmp r8d, 45");                                         // is it '-'?
    emitter.instruction("je __rt_str_is_numeric_sign_consume_linux_x86_64");    // consume the minus sign
    emitter.instruction("cmp r8d, 43");                                         // is it '+'?
    emitter.instruction("je __rt_str_is_numeric_sign_consume_linux_x86_64");    // consume the plus sign
    emitter.instruction("jmp __rt_str_is_numeric_int_start_linux_x86_64");      // no sign → start integer digits
    emitter.label("__rt_str_is_numeric_sign_consume_linux_x86_64");
    emitter.instruction("add rcx, 1");                                          // consume the sign byte
    emitter.instruction("cmp rcx, rdx");                                        // is there anything after the sign?
    emitter.instruction("jae __rt_str_is_numeric_done_linux_x86_64");           // a bare sign is not numeric

    // -- integer part: at least one digit is mandatory --
    emitter.label("__rt_str_is_numeric_int_start_linux_x86_64");
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // load the first integer-part byte
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_done_linux_x86_64");            // the first integer byte must be a digit
    emitter.instruction("add rcx, 1");                                          // consume the first integer digit

    // -- remaining integer digits --
    emitter.label("__rt_str_is_numeric_int_loop_linux_x86_64");
    emitter.instruction("cmp rcx, rdx");                                        // end of input?
    emitter.instruction("jae __rt_str_is_numeric_ok_linux_x86_64");             // a pure integer is numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the next integer byte
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_after_int_linux_x86_64");       // non-digit → check fraction or exponent
    emitter.instruction("add rcx, 1");                                          // consume the integer digit
    emitter.instruction("jmp __rt_str_is_numeric_int_loop_linux_x86_64");       // continue scanning integer digits

    // -- after integer part: fraction or exponent or end --
    emitter.label("__rt_str_is_numeric_after_int_linux_x86_64");
    emitter.instruction("cmp r8d, 46");                                         // is it '.'?
    emitter.instruction("je __rt_str_is_numeric_frac_start_linux_x86_64");      // start the fractional part
    emitter.instruction("cmp r8d, 101");                                        // is it 'e'?
    emitter.instruction("je __rt_str_is_numeric_exp_sign_linux_x86_64");        // start the exponent
    emitter.instruction("cmp r8d, 69");                                         // is it 'E'?
    emitter.instruction("je __rt_str_is_numeric_exp_sign_linux_x86_64");        // start the exponent
    emitter.instruction("jmp __rt_str_is_numeric_done_linux_x86_64");           // any other trailing byte → not numeric

    // -- fractional part: at least one digit is required after the dot --
    emitter.label("__rt_str_is_numeric_frac_start_linux_x86_64");
    emitter.instruction("add rcx, 1");                                          // consume the '.'
    emitter.instruction("cmp rcx, rdx");                                        // is there a digit after the dot?
    emitter.instruction("jae __rt_str_is_numeric_done_linux_x86_64");           // a bare 'X.' is not numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the first fractional byte
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_done_linux_x86_64");            // need at least one fractional digit
    emitter.instruction("add rcx, 1");                                          // consume the first fractional digit

    emitter.label("__rt_str_is_numeric_frac_loop_linux_x86_64");
    emitter.instruction("cmp rcx, rdx");                                        // end of input?
    emitter.instruction("jae __rt_str_is_numeric_ok_linux_x86_64");             // a fraction-only number is numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the next fractional byte
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_after_frac_linux_x86_64");      // non-digit → check exponent
    emitter.instruction("add rcx, 1");                                          // consume the fractional digit
    emitter.instruction("jmp __rt_str_is_numeric_frac_loop_linux_x86_64");      // continue scanning fractional digits

    // -- after fraction: exponent or end --
    emitter.label("__rt_str_is_numeric_after_frac_linux_x86_64");
    emitter.instruction("cmp r8d, 101");                                        // is it 'e'?
    emitter.instruction("je __rt_str_is_numeric_exp_sign_linux_x86_64");        // start the exponent
    emitter.instruction("cmp r8d, 69");                                         // is it 'E'?
    emitter.instruction("je __rt_str_is_numeric_exp_sign_linux_x86_64");        // start the exponent
    emitter.instruction("jmp __rt_str_is_numeric_done_linux_x86_64");           // any other trailing byte → not numeric

    // -- exponent: optional sign then at least one digit --
    emitter.label("__rt_str_is_numeric_exp_sign_linux_x86_64");
    emitter.instruction("add rcx, 1");                                          // consume the 'e' or 'E'
    emitter.instruction("cmp rcx, rdx");                                        // is there anything after the exponent marker?
    emitter.instruction("jae __rt_str_is_numeric_done_linux_x86_64");           // a bare 'Xe' is not numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the byte after the exponent marker
    emitter.instruction("cmp r8d, 43");                                         // optional '+'?
    emitter.instruction("je __rt_str_is_numeric_exp_consume_sign_linux_x86_64"); // consume the exponent plus sign
    emitter.instruction("cmp r8d, 45");                                         // optional '-'?
    emitter.instruction("je __rt_str_is_numeric_exp_consume_sign_linux_x86_64"); // consume the exponent minus sign
    emitter.instruction("jmp __rt_str_is_numeric_exp_first_digit_linux_x86_64"); // continue to the first exponent digit
    emitter.label("__rt_str_is_numeric_exp_consume_sign_linux_x86_64");
    emitter.instruction("add rcx, 1");                                          // consume the exponent sign
    emitter.instruction("cmp rcx, rdx");                                        // is there a digit after the sign?
    emitter.instruction("jae __rt_str_is_numeric_done_linux_x86_64");           // a bare 'e+'/'e-' is not numeric

    emitter.label("__rt_str_is_numeric_exp_first_digit_linux_x86_64");
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the first exponent digit
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_done_linux_x86_64");            // need at least one exponent digit
    emitter.instruction("add rcx, 1");                                          // consume the first exponent digit

    emitter.label("__rt_str_is_numeric_exp_loop_linux_x86_64");
    emitter.instruction("cmp rcx, rdx");                                        // end of input?
    emitter.instruction("jae __rt_str_is_numeric_ok_linux_x86_64");             // a valid exponent reached end-of-string → numeric
    emitter.instruction("movzx r8d, BYTE PTR [rax + rcx]");                     // peek the next exponent byte
    emitter.instruction("sub r8d, 48");                                         // normalize to a candidate digit
    emitter.instruction("cmp r8d, 9");                                          // 0..9 range
    emitter.instruction("ja __rt_str_is_numeric_done_linux_x86_64");            // any non-digit after exponent digits → not numeric
    emitter.instruction("add rcx, 1");                                          // consume the exponent digit
    emitter.instruction("jmp __rt_str_is_numeric_exp_loop_linux_x86_64");       // continue scanning exponent digits

    emitter.label("__rt_str_is_numeric_ok_linux_x86_64");
    emitter.instruction("mov rax, 1");                                          // signal that the string is numeric
    emitter.label("__rt_str_is_numeric_done_linux_x86_64");
    emitter.instruction("ret");                                                 // return the numeric flag in rax
}