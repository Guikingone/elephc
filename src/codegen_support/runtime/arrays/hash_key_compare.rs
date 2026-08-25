//! Purpose:
//! Emits PHP SORT_REGULAR comparison for normalized associative-array keys.
//!
//! Called from:
//! - `crate::codegen_support::runtime::arrays::hash_sort`.
//!
//! Key details:
//! - Preserves exact decimal-integer ordering beyond binary64 precision.
//! - Accepts only PHP numeric-string spellings and keeps bounded binary keys lexical.
//! - Supports every target through matching AArch64 and x86_64 helpers.

use crate::codegen_support::emit::Emitter;
use crate::codegen_support::platform::Arch;

/// Emits the exact regular key comparator for the active target.
pub(super) fn emit_hash_key_compare(emitter: &mut Emitter) {
    match emitter.target.arch {
        Arch::AArch64 => {
            emit_regular_key_compare_aarch64(emitter);
            emit_exact_decimal_key_compare_aarch64(emitter);
        }
        Arch::X86_64 => {
            emit_regular_key_compare_x86_64(emitter);
            emit_exact_decimal_key_compare_x86_64(emitter);
        }
    }
}

/// Emits PHP `SORT_REGULAR` comparison for normalized hash keys on AArch64.
///
/// Integer/integer pairs use signed ordering. String/string and mixed pairs use numeric ordering
/// when every participating string is numeric, otherwise bounded byte ordering. Embedded NULs
/// are rejected before the C-string numeric parser so binary keys stay lexically comparable.
fn emit_regular_key_compare_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: regular hash-key comparison ---");
    emitter.label_global("__rt_key_compare_regular");
    emitter.instruction("sub sp, sp, #128");                                    // reserve key words, scratch space, decimal bytes, and the saved frame
    emitter.instruction("stp x29, x30, [sp, #112]");                            // preserve frame pointer and return address across runtime helpers
    emitter.instruction("add x29, sp, #112");                                   // establish a stable helper frame pointer
    emitter.instruction("str x0, [sp, #0]");                                    // save the left key low word
    emitter.instruction("str x1, [sp, #8]");                                    // save the left key high word or integer sentinel
    emitter.instruction("str x2, [sp, #16]");                                   // save the right key low word
    emitter.instruction("str x3, [sp, #24]");                                   // save the right key high word or integer sentinel
    emitter.instruction("ldr x9, [sp, #8]");                                    // inspect the left normalized key tag
    emitter.instruction("cmn x9, #1");                                          // integer keys use the all-ones high-word sentinel
    emitter.instruction("b.eq __rt_key_compare_regular_left_int");              // route an integer left key to the remaining tag checks
    emitter.instruction("ldr x10, [sp, #24]");                                  // inspect the right normalized key tag
    emitter.instruction("cmn x10, #1");                                         // does the right key carry an integer sentinel?
    emitter.instruction("b.eq __rt_key_compare_regular_mixed_right_int");       // normalize a string-left/integer-right mixed comparison
    emitter.instruction("b __rt_key_compare_regular_strings");                  // both string keys compare as bounded byte sequences

    emitter.label("__rt_key_compare_regular_left_int");
    emitter.instruction("ldr x10, [sp, #24]");                                  // inspect the right normalized key tag
    emitter.instruction("cmn x10, #1");                                         // is the right key also an integer?
    emitter.instruction("b.ne __rt_key_compare_regular_mixed_left_int");        // normalize an integer-left/string-right mixed comparison
    emitter.instruction("ldr x9, [sp, #0]");                                    // reload the left signed integer key
    emitter.instruction("ldr x10, [sp, #16]");                                  // reload the right signed integer key
    emitter.instruction("cmp x9, x10");                                         // compare integer keys with PHP signed ordering
    emitter.instruction("cset x9, lt");                                         // materialize whether the left key is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left key is greater
    emitter.instruction("sub x0, x10, x9");                                     // return -1, 0, or 1 for left-versus-right ordering
    emitter.instruction("b __rt_key_compare_regular_done");                     // skip the string and mixed comparison paths

    emitter.label("__rt_key_compare_regular_strings");
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass the left bounded key to exact decimal-integer comparison
    emitter.instruction("ldr x2, [sp, #8]");                                    // pass the left bounded key length to exact decimal-integer comparison
    emitter.instruction("ldr x3, [sp, #16]");                                   // pass the right bounded key to exact decimal-integer comparison
    emitter.instruction("ldr x4, [sp, #24]");                                   // pass the right bounded key length to exact decimal-integer comparison
    emitter.instruction("bl __rt_key_compare_exact_decimal_integers");          // preserve integer-string precision before the floating fallback
    emitter.instruction("cmp x0, #3");                                          // did two overflowing integer spellings require PHP's equal-double tiebreak?
    emitter.instruction("b.eq __rt_key_compare_regular_strings_overflow_pair"); // remember to compare equal overflowing spellings lexically
    emitter.instruction("cmp x0, #2");                                          // does either key require non-integer numeric or lexical comparison?
    emitter.instruction("b.ne __rt_key_compare_regular_done");                  // return an exact ordering when at least one integer fits signed 64-bit
    emitter.instruction("str xzr, [sp, #40]");                                  // keep equal finite fractional and exponent spellings stable
    emitter.instruction("b __rt_key_compare_regular_strings_fallback");         // continue through the shared bounded numeric parser
    emitter.label("__rt_key_compare_regular_strings_overflow_pair");
    emitter.instruction("mov x9, #1");                                          // mark the overflow pair for PHP's raw-string equal-double tiebreak
    emitter.instruction("str x9, [sp, #40]");                                   // preserve the tiebreak policy across both numeric parses
    emitter.label("__rt_key_compare_regular_strings_fallback");
    emitter.instruction("ldr x9, [sp, #0]");                                    // load the left string pointer for the bounded embedded-NUL scan
    emitter.instruction("ldr x10, [sp, #8]");                                   // load the left string length for the bounded embedded-NUL scan
    emitter.instruction("mov x11, #0");                                         // begin scanning the left string at byte zero
    emitter.label("__rt_key_compare_regular_strings_left_nul_loop");
    emitter.instruction("cmp x11, x10");                                        // did the left bounded scan consume every byte?
    emitter.instruction("b.hs __rt_key_compare_regular_strings_left_parse");    // only NUL-free keys may use the numeric parser
    emitter.instruction("ldrb w12, [x9, x11]");                                 // inspect the next left string byte without sign extension
    emitter.instruction("cbz w12, __rt_key_compare_regular_strings_lexical");   // binary keys with NUL bytes must use bytewise ordering
    emitter.instruction("add x11, x11, #1");                                    // advance after a non-NUL left string byte
    emitter.instruction("b __rt_key_compare_regular_strings_left_nul_loop");    // continue scanning the bounded left key

    emitter.label("__rt_key_compare_regular_strings_left_parse");
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass the left string pointer to PHP numeric-string parsing
    emitter.instruction("ldr x2, [sp, #8]");                                    // pass the left string length to PHP numeric-string parsing
    emitter.instruction("bl __rt_str_looks_like_int_for_coercion");             // classify and parse the left string key with PHP-only numeric spellings
    emitter.instruction("cbz x0, __rt_key_compare_regular_strings_lexical");    // a non-numeric left key selects bounded byte ordering
    emitter.instruction("str d0, [sp, #32]");                                   // preserve the parsed left number across the right parse
    emitter.instruction("ldr x9, [sp, #16]");                                   // load the right string pointer for the bounded embedded-NUL scan
    emitter.instruction("ldr x10, [sp, #24]");                                  // load the right string length for the bounded embedded-NUL scan
    emitter.instruction("mov x11, #0");                                         // begin scanning the right string at byte zero
    emitter.label("__rt_key_compare_regular_strings_right_nul_loop");
    emitter.instruction("cmp x11, x10");                                        // did the right bounded scan consume every byte?
    emitter.instruction("b.hs __rt_key_compare_regular_strings_right_parse");   // only NUL-free keys may use the numeric parser
    emitter.instruction("ldrb w12, [x9, x11]");                                 // inspect the next right string byte without sign extension
    emitter.instruction("cbz w12, __rt_key_compare_regular_strings_lexical");   // binary keys with NUL bytes must use bytewise ordering
    emitter.instruction("add x11, x11, #1");                                    // advance after a non-NUL right string byte
    emitter.instruction("b __rt_key_compare_regular_strings_right_nul_loop");   // continue scanning the bounded right key

    emitter.label("__rt_key_compare_regular_strings_right_parse");
    emitter.instruction("ldr x1, [sp, #16]");                                   // pass the right string pointer to PHP numeric-string parsing
    emitter.instruction("ldr x2, [sp, #24]");                                   // pass the right string length to PHP numeric-string parsing
    emitter.instruction("bl __rt_str_looks_like_int_for_coercion");             // classify and parse the right string key with PHP-only numeric spellings
    emitter.instruction("cbz x0, __rt_key_compare_regular_strings_lexical");    // a non-numeric right key selects bounded byte ordering
    emitter.instruction("ldr d1, [sp, #32]");                                   // reload the parsed left numeric-string value
    emitter.instruction("fcmp d1, d0");                                         // compare the two PHP numeric-string values
    emitter.instruction("b.vs __rt_key_compare_regular_strings_lexical");       // unordered libc spellings fall back to byte ordering
    emitter.instruction("b.ne __rt_key_compare_regular_strings_ordered");       // unequal finite or infinite values retain numeric ordering
    emitter.instruction("fmov x9, d0");                                         // inspect one equal numeric result without changing its binary64 value
    emitter.instruction("ubfx x9, x9, #52, #11");                               // isolate the binary64 exponent field to recognize infinity
    emitter.instruction("cmp x9, #0x7ff");                                      // did both numeric strings round to the same signed infinity?
    emitter.instruction("b.eq __rt_key_compare_regular_strings_lexical");       // PHP breaks equal-infinity ties with bounded string ordering
    emitter.instruction("ldr x9, [sp, #40]");                                   // recover whether both integer spellings overflowed signed 64-bit
    emitter.instruction("cbnz x9, __rt_key_compare_regular_strings_lexical");   // PHP byte-compares equal finite doubles from two overflow integers
    emitter.instruction("mov x0, #0");                                          // equal finite binary64 values preserve the stable insertion order
    emitter.instruction("b __rt_key_compare_regular_done");                     // return equality without applying a textual tiebreaker
    emitter.label("__rt_key_compare_regular_strings_ordered");
    emitter.instruction("cset x9, lt");                                         // materialize whether the left numeric key is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left numeric key is greater
    emitter.instruction("sub x0, x10, x9");                                     // return the normalized numeric string ordering
    emitter.instruction("b __rt_key_compare_regular_done");                     // skip bytewise ordering after numeric comparison

    emitter.label("__rt_key_compare_regular_strings_lexical");
    emitter.instruction("ldr x1, [sp, #0]");                                    // pass the left string pointer to the bounded comparator
    emitter.instruction("ldr x2, [sp, #8]");                                    // pass the left string length to the bounded comparator
    emitter.instruction("ldr x3, [sp, #16]");                                   // pass the right string pointer to the bounded comparator
    emitter.instruction("ldr x4, [sp, #24]");                                   // pass the right string length to the bounded comparator
    emitter.instruction("bl __rt_strcmp");                                      // compare non-numeric byte sequences without C-string truncation
    emitter.instruction("cmp x0, #0");                                          // normalize the bounded comparator's signed result
    emitter.instruction("cset x9, lt");                                         // materialize whether the left string is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left string is greater
    emitter.instruction("sub x0, x10, x9");                                     // return the normalized signed string ordering
    emitter.instruction("b __rt_key_compare_regular_done");                     // skip the mixed comparison path

    emitter.label("__rt_key_compare_regular_mixed_left_int");
    emitter.instruction("ldr x9, [sp, #16]");                                   // select the right string pointer for mixed parsing
    emitter.instruction("ldr x10, [sp, #24]");                                  // select the right string length for mixed parsing
    emitter.instruction("ldr x11, [sp, #0]");                                   // select the left integer to stringify or convert
    emitter.instruction("mov x12, #1");                                         // remember that the integer is the left comparison operand
    emitter.instruction("b __rt_key_compare_regular_mixed_ready");              // share the numeric-string and lexical fallback logic

    emitter.label("__rt_key_compare_regular_mixed_right_int");
    emitter.instruction("ldr x9, [sp, #0]");                                    // select the left string pointer for mixed parsing
    emitter.instruction("ldr x10, [sp, #8]");                                   // select the left string length for mixed parsing
    emitter.instruction("ldr x11, [sp, #16]");                                  // select the right integer to stringify or convert
    emitter.instruction("mov x12, #0");                                         // remember that the string is the left comparison operand

    emitter.label("__rt_key_compare_regular_mixed_ready");
    emitter.instruction("str x9, [sp, #32]");                                   // save the mixed string pointer across numeric parsing
    emitter.instruction("str x10, [sp, #40]");                                  // save the mixed string byte length across numeric parsing
    emitter.instruction("str x11, [sp, #48]");                                  // save the mixed integer across runtime calls
    emitter.instruction("str x12, [sp, #56]");                                  // save the mixed operand orientation flag
    emitter.instruction("ldr x1, [sp, #32]");                                   // classify the mixed string before applying exact integer ordering
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass its bounded byte length to the integer-spelling classifier
    emitter.instruction("bl __rt_key_parse_i64_decimal");                       // distinguish fitting decimal strings from signed-64 overflow spellings
    emitter.instruction("cmp x0, #2");                                          // does the mixed string exceed signed 64-bit while remaining an integer spelling?
    emitter.instruction("b.eq __rt_key_compare_regular_mixed_parse");           // compare overflow spellings through PHP's shared binary64 numeric path
    emitter.instruction("b __rt_key_compare_regular_mixed_exact_render");       // render the integer once for exact decimal-string comparison
    emitter.label("__rt_key_compare_regular_mixed_exact_render");
    emitter.instruction("ldr x0, [sp, #48]");                                   // exact decimal helper instruction
    emitter.instruction("add x1, sp, #104");                                    // exact decimal helper instruction
    emitter.instruction("mov x2, #0");                                          // exact decimal helper instruction
    emitter.instruction("mov x3, #0");                                          // exact decimal helper instruction
    emitter.instruction("cmp x0, #0");                                          // exact decimal helper instruction
    emitter.instruction("b.ge __rt_key_compare_regular_mixed_exact_digits");    // exact decimal helper instruction
    emitter.instruction("mov x3, #1");                                          // exact decimal helper instruction
    emitter.instruction("neg x0, x0");                                          // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_mixed_exact_digits");
    emitter.instruction("mov x4, #10");                                         // exact decimal helper instruction
    emitter.instruction("udiv x5, x0, x4");                                     // exact decimal helper instruction
    emitter.instruction("msub x6, x5, x4, x0");                                 // exact decimal helper instruction
    emitter.instruction("sub x1, x1, #1");                                      // exact decimal helper instruction
    emitter.instruction("add x6, x6, #48");                                     // exact decimal helper instruction
    emitter.instruction("strb w6, [x1]");                                       // exact decimal helper instruction
    emitter.instruction("add x2, x2, #1");                                      // exact decimal helper instruction
    emitter.instruction("mov x0, x5");                                          // exact decimal helper instruction
    emitter.instruction("cbnz x0, __rt_key_compare_regular_mixed_exact_digits");// exact decimal helper instruction
    emitter.instruction("cbz x3, __rt_key_compare_regular_mixed_exact_ready");  // exact decimal helper instruction
    emitter.instruction("sub x1, x1, #1");                                      // exact decimal helper instruction
    emitter.instruction("mov w4, #45");                                         // exact decimal helper instruction
    emitter.instruction("strb w4, [x1]");                                       // exact decimal helper instruction
    emitter.instruction("add x2, x2, #1");                                      // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_mixed_exact_ready");
    emitter.instruction("str x1, [sp, #64]");                                   // exact decimal helper instruction
    emitter.instruction("str x2, [sp, #72]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x12, [sp, #56]");                                  // exact decimal helper instruction
    emitter.instruction("cbz x12, __rt_key_compare_regular_mixed_exact_str_left");// exact decimal helper instruction
    emitter.instruction("ldr x1, [sp, #64]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x2, [sp, #72]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x3, [sp, #32]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x4, [sp, #40]");                                   // exact decimal helper instruction
    emitter.instruction("b __rt_key_compare_regular_mixed_exact_call");         // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_mixed_exact_str_left");
    emitter.instruction("ldr x1, [sp, #32]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x2, [sp, #40]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x3, [sp, #64]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x4, [sp, #72]");                                   // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_mixed_exact_call");
    emitter.instruction("bl __rt_key_compare_exact_decimal_integers");          // exact decimal helper instruction
    emitter.instruction("cmp x0, #2");                                          // exact decimal helper instruction
    emitter.instruction("b.ne __rt_key_compare_regular_done");                  // exact decimal helper instruction
    emitter.instruction("ldr x9, [sp, #32]");                                   // exact decimal helper instruction
    emitter.instruction("ldr x10, [sp, #40]");                                  // exact decimal helper instruction
    emitter.instruction("mov x13, #0");                                         // begin an explicit bounded embedded-NUL scan
    emitter.label("__rt_key_compare_regular_mixed_nul_loop");
    emitter.instruction("cmp x13, x10");                                        // did the scan consume every string byte?
    emitter.instruction("b.hs __rt_key_compare_regular_mixed_parse");           // only NUL-free strings may use the C-string numeric helper
    emitter.instruction("ldrb w14, [x9, x13]");                                 // inspect the next bounded string byte
    emitter.instruction("cbz w14, __rt_key_compare_regular_mixed_lexical");     // embedded NUL makes the PHP string non-numeric here
    emitter.instruction("add x13, x13, #1");                                    // advance after a non-NUL byte
    emitter.instruction("b __rt_key_compare_regular_mixed_nul_loop");           // continue scanning the bounded byte string

    emitter.label("__rt_key_compare_regular_mixed_parse");
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the string pointer to PHP numeric-string parsing
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass the string length to PHP numeric-string parsing
    emitter.instruction("bl __rt_str_looks_like_int_for_coercion");             // parse numeric strings through the strict PHP numeric helper
    emitter.instruction("cbz x0, __rt_key_compare_regular_mixed_lexical");      // non-numeric strings use string-versus-decimal-int ordering
    emitter.instruction("ldr x11, [sp, #48]");                                  // reload the signed integer for numeric comparison
    emitter.instruction("scvtf d1, x11");                                       // convert the integer to the shared floating numeric representation
    emitter.instruction("ldr x12, [sp, #56]");                                  // reload whether the integer was the left operand
    emitter.instruction("cbz x12, __rt_key_compare_regular_numeric_str_left");  // reverse float operands when the string is on the left
    emitter.instruction("fcmp d1, d0");                                         // compare left integer with right parsed numeric string
    emitter.instruction("b.vs __rt_key_compare_regular_mixed_lexical");         // unordered libc spellings fall back to byte ordering
    emitter.instruction("b __rt_key_compare_regular_numeric_normalize");        // normalize the numeric condition flags
    emitter.label("__rt_key_compare_regular_numeric_str_left");
    emitter.instruction("fcmp d0, d1");                                         // compare left parsed numeric string with right integer
    emitter.instruction("b.vs __rt_key_compare_regular_mixed_lexical");         // unordered libc spellings fall back to byte ordering
    emitter.label("__rt_key_compare_regular_numeric_normalize");
    emitter.instruction("cset x9, lt");                                         // materialize whether the left numeric value is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left numeric value is greater
    emitter.instruction("sub x0, x10, x9");                                     // return the normalized numeric comparison result
    emitter.instruction("b __rt_key_compare_regular_done");                     // skip decimal rendering after numeric comparison

    emitter.label("__rt_key_compare_regular_mixed_lexical");
    emitter.instruction("ldr x0, [sp, #48]");                                   // load the integer into the frame-local decimal renderer
    emitter.instruction("add x1, sp, #104");                                    // begin at the end of the private 24-byte decimal buffer
    emitter.instruction("mov x2, #0");                                          // start the rendered decimal length at zero
    emitter.instruction("mov x3, #0");                                          // clear the negative-sign flag before magnitude conversion
    emitter.instruction("cmp x0, #0");                                          // does the signed integer need a leading minus sign?
    emitter.instruction("b.ge __rt_key_compare_regular_decimal_digits");        // non-negative values already have their unsigned magnitude
    emitter.instruction("mov x3, #1");                                          // remember to prefix the rendered magnitude with a minus sign
    emitter.instruction("neg x0, x0");                                          // form the unsigned magnitude, including INT64_MIN modulo two to the 63rd
    emitter.label("__rt_key_compare_regular_decimal_digits");
    emitter.instruction("mov x4, #10");                                         // divide the remaining unsigned magnitude by decimal radix ten
    emitter.instruction("udiv x5, x0, x4");                                     // compute the next unsigned quotient without signed overflow
    emitter.instruction("msub x6, x5, x4, x0");                                 // derive the current decimal digit remainder
    emitter.instruction("sub x1, x1, #1");                                      // reserve one byte before the already-rendered suffix
    emitter.instruction("add x6, x6, #48");                                     // encode the unsigned remainder as one ASCII decimal digit
    emitter.instruction("strb w6, [x1]");                                       // write the digit inside the private bounded frame buffer
    emitter.instruction("add x2, x2, #1");                                      // include the digit in the bounded decimal string length
    emitter.instruction("mov x0, x5");                                          // continue with the unsigned quotient
    emitter.instruction("cbnz x0, __rt_key_compare_regular_decimal_digits");    // render every magnitude digit, including the INT64_MIN magnitude
    emitter.instruction("cbz x3, __rt_key_compare_regular_decimal_ready");      // omit the sign for non-negative values
    emitter.instruction("sub x1, x1, #1");                                      // reserve the leading minus-sign byte in the private buffer
    emitter.instruction("mov w4, #45");                                         // materialize the ASCII minus sign
    emitter.instruction("strb w4, [x1]");                                       // prefix the decimal magnitude with its sign
    emitter.instruction("add x2, x2, #1");                                      // include the leading sign in the string length
    emitter.label("__rt_key_compare_regular_decimal_ready");
    emitter.instruction("ldr x12, [sp, #56]");                                  // reload whether the integer is the left comparison operand
    emitter.instruction("cbz x12, __rt_key_compare_regular_lexical_str_left");  // arrange bounded comparator arguments for a string-left pair
    emitter.instruction("mov x3, x1");                                          // retain the private decimal pointer while loading the opposite operand
    emitter.instruction("mov x4, x2");                                          // retain the private decimal length while loading the opposite operand
    emitter.instruction("mov x1, x3");                                          // pass the decimal integer pointer as the left string
    emitter.instruction("mov x2, x4");                                          // pass the decimal integer length as the left string
    emitter.instruction("ldr x3, [sp, #32]");                                   // pass the original non-numeric string pointer as the right string
    emitter.instruction("ldr x4, [sp, #40]");                                   // pass the original non-numeric string length as the right string
    emitter.instruction("b __rt_key_compare_regular_lexical_compare");          // compare the arranged bounded strings
    emitter.label("__rt_key_compare_regular_lexical_str_left");
    emitter.instruction("mov x5, x1");                                          // retain the private decimal pointer while loading the left string
    emitter.instruction("mov x6, x2");                                          // retain the private decimal length while loading the left string
    emitter.instruction("ldr x1, [sp, #32]");                                   // pass the original non-numeric string pointer as the left string
    emitter.instruction("ldr x2, [sp, #40]");                                   // pass the original non-numeric string length as the left string
    emitter.instruction("mov x3, x5");                                          // pass the private decimal integer pointer as the right string
    emitter.instruction("mov x4, x6");                                          // pass the private decimal integer length as the right string
    emitter.label("__rt_key_compare_regular_lexical_compare");
    emitter.instruction("bl __rt_strcmp");                                      // compare non-numeric mixed operands as bounded strings
    emitter.instruction("cmp x0, #0");                                          // normalize the lexical mixed-comparison result
    emitter.instruction("cset x9, lt");                                         // materialize whether the left lexical value is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left lexical value is greater
    emitter.instruction("sub x0, x10, x9");                                     // return -1, 0, or 1 for the lexical mixed comparison

    emitter.label("__rt_key_compare_regular_done");
    emitter.instruction("ldp x29, x30, [sp, #112]");                            // restore frame pointer and return address
    emitter.instruction("add sp, sp, #128");                                    // release the regular-comparison helper frame
    emitter.instruction("ret");                                                 // return the normalized PHP SORT_REGULAR ordering
}

/// Emits PHP `SORT_REGULAR` comparison for normalized hash keys on x86_64.
fn emit_regular_key_compare_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: regular hash-key comparison ---");
    emitter.label_global("__rt_key_compare_regular");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer
    emitter.instruction("mov rbp, rsp");                                        // establish a stable helper frame pointer
    emitter.instruction("sub rsp, 128");                                        // reserve key words, scratch slots, and a private decimal buffer
    emitter.instruction("mov QWORD PTR [rbp - 8], rdi");                        // save the left key low word
    emitter.instruction("mov QWORD PTR [rbp - 16], rsi");                       // save the left key high word or integer sentinel
    emitter.instruction("mov QWORD PTR [rbp - 24], rdx");                       // save the right key low word
    emitter.instruction("mov QWORD PTR [rbp - 32], rcx");                       // save the right key high word or integer sentinel
    emitter.instruction("cmp rsi, -1");                                         // does the left key use the integer sentinel?
    emitter.instruction("je __rt_key_compare_regular_x_left_int");              // route an integer left key to the remaining tag checks
    emitter.instruction("cmp rcx, -1");                                         // does the right key use the integer sentinel?
    emitter.instruction("je __rt_key_compare_regular_x_mixed_right_int");       // normalize a string-left/integer-right mixed comparison
    emitter.instruction("jmp __rt_key_compare_regular_x_strings");              // both string keys compare as bounded byte sequences

    emitter.label("__rt_key_compare_regular_x_left_int");
    emitter.instruction("cmp QWORD PTR [rbp - 32], -1");                        // is the right key also an integer?
    emitter.instruction("jne __rt_key_compare_regular_x_mixed_left_int");       // normalize an integer-left/string-right mixed comparison
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // reload the left signed integer key
    emitter.instruction("cmp r8, QWORD PTR [rbp - 24]");                        // compare integer keys with PHP signed ordering
    emitter.instruction("setl al");                                             // materialize whether the left key is smaller
    emitter.instruction("setg dl");                                             // materialize whether the left key is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx edx, dl");                                       // widen the greater-than flag
    emitter.instruction("sub rdx, rax");                                        // form -1, 0, or 1 from the two flags
    emitter.instruction("mov rax, rdx");                                        // return the normalized signed integer ordering
    emitter.instruction("jmp __rt_key_compare_regular_x_done");                 // skip the string and mixed comparison paths

    emitter.label("__rt_key_compare_regular_x_strings");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the left bounded key to exact decimal-integer comparison
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // pass the left bounded key length to exact decimal-integer comparison
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // pass the right bounded key to exact decimal-integer comparison
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // pass the right bounded key length to exact decimal-integer comparison
    emitter.instruction("call __rt_key_compare_exact_decimal_integers");        // preserve integer-string precision before the floating fallback
    emitter.instruction("cmp rax, 3");                                          // did two overflowing integer spellings require PHP's equal-double tiebreak?
    emitter.instruction("je __rt_key_compare_regular_x_strings_overflow_pair"); // remember to compare equal overflowing spellings lexically
    emitter.instruction("cmp rax, 2");                                          // does either key require non-integer numeric or lexical comparison?
    emitter.instruction("jne __rt_key_compare_regular_x_done");                 // return an exact ordering when at least one integer fits signed 64-bit
    emitter.instruction("mov QWORD PTR [rbp - 48], 0");                         // keep equal finite fractional and exponent spellings stable
    emitter.instruction("jmp __rt_key_compare_regular_x_strings_fallback");     // continue through the shared bounded numeric parser
    emitter.label("__rt_key_compare_regular_x_strings_overflow_pair");
    emitter.instruction("mov QWORD PTR [rbp - 48], 1");                         // preserve PHP's overflow-pair tiebreak policy across both parses
    emitter.label("__rt_key_compare_regular_x_strings_fallback");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // load the left string pointer for the bounded embedded-NUL scan
    emitter.instruction("mov r9, QWORD PTR [rbp - 16]");                        // load the left string length for the bounded embedded-NUL scan
    emitter.instruction("xor r10d, r10d");                                      // begin scanning the left string at byte zero
    emitter.label("__rt_key_compare_regular_x_strings_left_nul_loop");
    emitter.instruction("cmp r10, r9");                                         // did the left bounded scan consume every byte?
    emitter.instruction("jae __rt_key_compare_regular_x_strings_left_parse");   // only NUL-free keys may use the numeric parser
    emitter.instruction("movzx r11d, BYTE PTR [r8 + r10]");                     // inspect the next left string byte without sign extension
    emitter.instruction("test r11d, r11d");                                     // is the current bounded byte a NUL?
    emitter.instruction("je __rt_key_compare_regular_x_strings_lexical");       // binary keys with NUL bytes must use bytewise ordering
    emitter.instruction("add r10, 1");                                          // advance after a non-NUL left string byte
    emitter.instruction("jmp __rt_key_compare_regular_x_strings_left_nul_loop"); // continue scanning the bounded left key

    emitter.label("__rt_key_compare_regular_x_strings_left_parse");
    emitter.instruction("mov rax, QWORD PTR [rbp - 8]");                        // pass the left string pointer to PHP numeric-string parsing
    emitter.instruction("mov rdx, QWORD PTR [rbp - 16]");                       // pass the left string length to PHP numeric-string parsing
    emitter.instruction("call __rt_str_looks_like_int_for_coercion");           // classify and parse the left string key with PHP-only numeric spellings
    emitter.instruction("test rax, rax");                                       // did strict PHP numeric-string parsing succeed?
    emitter.instruction("je __rt_key_compare_regular_x_strings_lexical");       // a non-numeric left key selects bounded byte ordering
    emitter.instruction("movsd QWORD PTR [rbp - 40], xmm0");                    // preserve the parsed left number across the right parse
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // load the right string pointer for the bounded embedded-NUL scan
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // load the right string length for the bounded embedded-NUL scan
    emitter.instruction("xor r10d, r10d");                                      // begin scanning the right string at byte zero
    emitter.label("__rt_key_compare_regular_x_strings_right_nul_loop");
    emitter.instruction("cmp r10, r9");                                         // did the right bounded scan consume every byte?
    emitter.instruction("jae __rt_key_compare_regular_x_strings_right_parse");  // only NUL-free keys may use the numeric parser
    emitter.instruction("movzx r11d, BYTE PTR [r8 + r10]");                     // inspect the next right string byte without sign extension
    emitter.instruction("test r11d, r11d");                                     // is the current bounded byte a NUL?
    emitter.instruction("je __rt_key_compare_regular_x_strings_lexical");       // binary keys with NUL bytes must use bytewise ordering
    emitter.instruction("add r10, 1");                                          // advance after a non-NUL right string byte
    emitter.instruction("jmp __rt_key_compare_regular_x_strings_right_nul_loop"); // continue scanning the bounded right key

    emitter.label("__rt_key_compare_regular_x_strings_right_parse");
    emitter.instruction("mov rax, QWORD PTR [rbp - 24]");                       // pass the right string pointer to PHP numeric-string parsing
    emitter.instruction("mov rdx, QWORD PTR [rbp - 32]");                       // pass the right string length to PHP numeric-string parsing
    emitter.instruction("call __rt_str_looks_like_int_for_coercion");           // classify and parse the right string key with PHP-only numeric spellings
    emitter.instruction("test rax, rax");                                       // did strict PHP numeric-string parsing succeed?
    emitter.instruction("je __rt_key_compare_regular_x_strings_lexical");       // a non-numeric right key selects bounded byte ordering
    emitter.instruction("movsd xmm1, QWORD PTR [rbp - 40]");                    // reload the parsed left numeric-string value
    emitter.instruction("ucomisd xmm1, xmm0");                                  // compare the two PHP numeric-string values
    emitter.instruction("jp __rt_key_compare_regular_x_strings_lexical");       // unordered libc spellings fall back to byte ordering
    emitter.instruction("jne __rt_key_compare_regular_x_strings_ordered");      // unequal finite or infinite values retain numeric ordering
    emitter.instruction("movq rax, xmm0");                                      // inspect one equal numeric result without changing its binary64 value
    emitter.instruction("shr rax, 52");                                         // move the binary64 sign and exponent field into the low bits
    emitter.instruction("and eax, 2047");                                       // discard the sign and retain the complete exponent field
    emitter.instruction("cmp eax, 2047");                                       // did both numeric strings round to the same signed infinity?
    emitter.instruction("je __rt_key_compare_regular_x_strings_lexical");       // PHP breaks equal-infinity ties with bounded string ordering
    emitter.instruction("cmp QWORD PTR [rbp - 48], 0");                         // did both integer spellings overflow signed 64-bit before parsing?
    emitter.instruction("jne __rt_key_compare_regular_x_strings_lexical");      // PHP byte-compares their equal finite binary64 representations
    emitter.instruction("xor eax, eax");                                        // equal finite binary64 values preserve the stable insertion order
    emitter.instruction("jmp __rt_key_compare_regular_x_done");                 // return equality without applying a textual tiebreaker
    emitter.label("__rt_key_compare_regular_x_strings_ordered");
    emitter.instruction("setb al");                                             // materialize whether the left numeric key is smaller
    emitter.instruction("seta dl");                                             // materialize whether the left numeric key is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx edx, dl");                                       // widen the greater-than flag
    emitter.instruction("sub rdx, rax");                                        // form -1, 0, or 1 from the two flags
    emitter.instruction("mov rax, rdx");                                        // return the normalized numeric string ordering
    emitter.instruction("jmp __rt_key_compare_regular_x_done");                 // skip bytewise ordering after numeric comparison

    emitter.label("__rt_key_compare_regular_x_strings_lexical");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 8]");                        // pass the left string pointer to the bounded comparator
    emitter.instruction("mov rsi, QWORD PTR [rbp - 16]");                       // pass the left string length to the bounded comparator
    emitter.instruction("mov rdx, QWORD PTR [rbp - 24]");                       // pass the right string pointer to the bounded comparator
    emitter.instruction("mov rcx, QWORD PTR [rbp - 32]");                       // pass the right string length to the bounded comparator
    emitter.instruction("call __rt_strcmp");                                    // compare byte sequences without C-string truncation
    emitter.instruction("test rax, rax");                                       // normalize the bounded comparator's signed result
    emitter.instruction("setl al");                                             // materialize whether the left string is smaller
    emitter.instruction("setg dl");                                             // materialize whether the left string is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx edx, dl");                                       // widen the greater-than flag
    emitter.instruction("sub rdx, rax");                                        // form -1, 0, or 1 from the two flags
    emitter.instruction("mov rax, rdx");                                        // return the normalized string ordering
    emitter.instruction("jmp __rt_key_compare_regular_x_done");                 // skip the mixed comparison path

    emitter.label("__rt_key_compare_regular_x_mixed_left_int");
    emitter.instruction("mov r8, QWORD PTR [rbp - 24]");                        // select the right string pointer for mixed parsing
    emitter.instruction("mov r9, QWORD PTR [rbp - 32]");                        // select the right string length for mixed parsing
    emitter.instruction("mov r10, QWORD PTR [rbp - 8]");                        // select the left integer to stringify or convert
    emitter.instruction("mov r11, 1");                                          // remember that the integer is the left comparison operand
    emitter.instruction("jmp __rt_key_compare_regular_x_mixed_ready");          // share the numeric-string and lexical fallback logic

    emitter.label("__rt_key_compare_regular_x_mixed_right_int");
    emitter.instruction("mov r8, QWORD PTR [rbp - 8]");                         // select the left string pointer for mixed parsing
    emitter.instruction("mov r9, QWORD PTR [rbp - 16]");                        // select the left string length for mixed parsing
    emitter.instruction("mov r10, QWORD PTR [rbp - 24]");                       // select the right integer to stringify or convert
    emitter.instruction("xor r11d, r11d");                                      // remember that the string is the left comparison operand

    emitter.label("__rt_key_compare_regular_x_mixed_ready");
    emitter.instruction("mov QWORD PTR [rbp - 40], r8");                        // save the mixed string pointer across numeric parsing
    emitter.instruction("mov QWORD PTR [rbp - 48], r9");                        // save the mixed string byte length across numeric parsing
    emitter.instruction("mov QWORD PTR [rbp - 56], r10");                       // save the mixed integer across runtime calls
    emitter.instruction("mov QWORD PTR [rbp - 64], r11");                       // save the mixed operand orientation flag
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // classify the mixed string before applying exact integer ordering
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // pass its bounded byte length to the integer-spelling classifier
    emitter.instruction("call __rt_key_parse_i64_decimal");                     // distinguish fitting decimal strings from signed-64 overflow spellings
    emitter.instruction("cmp rax, 2");                                          // does the mixed string exceed signed 64-bit while remaining an integer spelling?
    emitter.instruction("je __rt_key_compare_regular_x_mixed_parse");           // compare overflow spellings through PHP's shared binary64 numeric path
    emitter.instruction("jmp __rt_key_compare_regular_x_mixed_exact_render");   // render the integer once for exact decimal-string comparison
    emitter.label("__rt_key_compare_regular_x_mixed_exact_render");
    emitter.instruction("lea rcx, [rbp - 88]");                                 // exact decimal helper instruction
    emitter.instruction("xor r11d, r11d");                                      // exact decimal helper instruction
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // exact decimal helper instruction
    emitter.instruction("xor r9d, r9d");                                        // exact decimal helper instruction
    emitter.instruction("test r8, r8");                                         // exact decimal helper instruction
    emitter.instruction("jns __rt_key_compare_regular_x_mixed_exact_digits");   // exact decimal helper instruction
    emitter.instruction("mov r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("neg r8");                                              // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_x_mixed_exact_digits");
    emitter.instruction("mov r10, 10");                                         // exact decimal helper instruction
    emitter.instruction("mov rax, r8");                                         // exact decimal helper instruction
    emitter.instruction("xor edx, edx");                                        // exact decimal helper instruction
    emitter.instruction("div r10");                                             // exact decimal helper instruction
    emitter.instruction("sub rcx, 1");                                          // exact decimal helper instruction
    emitter.instruction("add dl, 48");                                          // exact decimal helper instruction
    emitter.instruction("mov BYTE PTR [rcx], dl");                              // exact decimal helper instruction
    emitter.instruction("add r11, 1");                                          // exact decimal helper instruction
    emitter.instruction("mov r8, rax");                                         // exact decimal helper instruction
    emitter.instruction("test r8, r8");                                         // exact decimal helper instruction
    emitter.instruction("jne __rt_key_compare_regular_x_mixed_exact_digits");   // exact decimal helper instruction
    emitter.instruction("test r9, r9");                                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_compare_regular_x_mixed_exact_ready");     // exact decimal helper instruction
    emitter.instruction("sub rcx, 1");                                          // exact decimal helper instruction
    emitter.instruction("mov BYTE PTR [rcx], 45");                              // exact decimal helper instruction
    emitter.instruction("add r11, 1");                                          // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_x_mixed_exact_ready");
    emitter.instruction("mov QWORD PTR [rbp - 72], rcx");                       // exact decimal helper instruction
    emitter.instruction("mov QWORD PTR [rbp - 80], r11");                       // exact decimal helper instruction
    emitter.instruction("cmp QWORD PTR [rbp - 64], 0");                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_compare_regular_x_mixed_exact_str_left");  // exact decimal helper instruction
    emitter.instruction("mov rdi, QWORD PTR [rbp - 72]");                       // exact decimal helper instruction
    emitter.instruction("mov rsi, QWORD PTR [rbp - 80]");                       // exact decimal helper instruction
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // exact decimal helper instruction
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_compare_regular_x_mixed_exact_call");     // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_x_mixed_exact_str_left");
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // exact decimal helper instruction
    emitter.instruction("mov rsi, QWORD PTR [rbp - 48]");                       // exact decimal helper instruction
    emitter.instruction("mov rdx, QWORD PTR [rbp - 72]");                       // exact decimal helper instruction
    emitter.instruction("mov rcx, QWORD PTR [rbp - 80]");                       // exact decimal helper instruction
    emitter.label("__rt_key_compare_regular_x_mixed_exact_call");
    emitter.instruction("call __rt_key_compare_exact_decimal_integers");        // exact decimal helper instruction
    emitter.instruction("cmp rax, 2");                                          // exact decimal helper instruction
    emitter.instruction("jne __rt_key_compare_regular_x_done");                 // exact decimal helper instruction
    emitter.instruction("mov r8, QWORD PTR [rbp - 40]");                        // exact decimal helper instruction
    emitter.instruction("mov r9, QWORD PTR [rbp - 48]");                        // exact decimal helper instruction
    emitter.instruction("xor r10d, r10d");                                      // begin an explicit bounded embedded-NUL scan
    emitter.label("__rt_key_compare_regular_x_mixed_nul_loop");
    emitter.instruction("cmp r10, r9");                                         // did the scan consume every string byte?
    emitter.instruction("jae __rt_key_compare_regular_x_mixed_parse");          // only NUL-free strings may use the C-string numeric helper
    emitter.instruction("movzx r11d, BYTE PTR [r8 + r10]");                     // inspect the next bounded string byte
    emitter.instruction("test r11d, r11d");                                     // is the current bounded byte a NUL?
    emitter.instruction("je __rt_key_compare_regular_x_mixed_lexical");         // embedded NUL makes the PHP string non-numeric here
    emitter.instruction("add r10, 1");                                          // advance after a non-NUL byte
    emitter.instruction("jmp __rt_key_compare_regular_x_mixed_nul_loop");       // continue scanning the bounded byte string

    emitter.label("__rt_key_compare_regular_x_mixed_parse");
    emitter.instruction("mov rax, QWORD PTR [rbp - 40]");                       // pass the string pointer to PHP numeric-string parsing
    emitter.instruction("mov rdx, QWORD PTR [rbp - 48]");                       // pass the string length to PHP numeric-string parsing
    emitter.instruction("call __rt_str_looks_like_int_for_coercion");           // parse numeric strings through the strict PHP numeric helper
    emitter.instruction("test rax, rax");                                       // did numeric-string parsing succeed?
    emitter.instruction("je __rt_key_compare_regular_x_mixed_lexical");         // non-numeric strings use string-versus-decimal-int ordering
    emitter.instruction("cvtsi2sd xmm1, QWORD PTR [rbp - 56]");                 // convert the signed integer to the shared floating numeric representation
    emitter.instruction("cmp QWORD PTR [rbp - 64], 0");                         // was the integer the left comparison operand?
    emitter.instruction("je __rt_key_compare_regular_x_numeric_str_left");      // reverse float operands when the string is on the left
    emitter.instruction("ucomisd xmm1, xmm0");                                  // compare left integer with right parsed numeric string
    emitter.instruction("jp __rt_key_compare_regular_x_mixed_lexical");         // unordered libc spellings fall back to byte ordering
    emitter.instruction("jmp __rt_key_compare_regular_x_numeric_normalize");    // normalize the numeric condition flags
    emitter.label("__rt_key_compare_regular_x_numeric_str_left");
    emitter.instruction("ucomisd xmm0, xmm1");                                  // compare left parsed numeric string with right integer
    emitter.instruction("jp __rt_key_compare_regular_x_mixed_lexical");         // unordered libc spellings fall back to byte ordering
    emitter.label("__rt_key_compare_regular_x_numeric_normalize");
    emitter.instruction("setb al");                                             // materialize whether the left numeric value is smaller
    emitter.instruction("seta dl");                                             // materialize whether the left numeric value is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx edx, dl");                                       // widen the greater-than flag
    emitter.instruction("sub rdx, rax");                                        // form -1, 0, or 1 from the two flags
    emitter.instruction("mov rax, rdx");                                        // return the normalized numeric ordering
    emitter.instruction("jmp __rt_key_compare_regular_x_done");                 // skip decimal rendering after numeric comparison

    emitter.label("__rt_key_compare_regular_x_mixed_lexical");
    emitter.instruction("lea rcx, [rbp - 88]");                                 // begin at the end of the private 24-byte decimal buffer
    emitter.instruction("xor r11d, r11d");                                      // start the rendered decimal length at zero
    emitter.instruction("mov r8, QWORD PTR [rbp - 56]");                        // load the integer into the frame-local decimal renderer
    emitter.instruction("xor r9d, r9d");                                        // clear the negative-sign flag before magnitude conversion
    emitter.instruction("test r8, r8");                                         // does the signed integer need a leading minus sign?
    emitter.instruction("jns __rt_key_compare_regular_x_decimal_digits");       // non-negative values already have their unsigned magnitude
    emitter.instruction("mov r9, 1");                                           // remember to prefix the rendered magnitude with a minus sign
    emitter.instruction("neg r8");                                              // form the unsigned magnitude, including INT64_MIN modulo two to the 63rd
    emitter.label("__rt_key_compare_regular_x_decimal_digits");
    emitter.instruction("mov r10, 10");                                         // divide the remaining unsigned magnitude by decimal radix ten
    emitter.instruction("mov rax, r8");                                         // load the magnitude dividend for unsigned division
    emitter.instruction("xor edx, edx");                                        // clear the unsigned division high word
    emitter.instruction("div r10");                                             // compute the next unsigned quotient and decimal remainder
    emitter.instruction("sub rcx, 1");                                          // reserve one byte before the already-rendered suffix
    emitter.instruction("add dl, 48");                                          // encode the unsigned remainder as one ASCII decimal digit
    emitter.instruction("mov BYTE PTR [rcx], dl");                              // write the digit inside the private bounded frame buffer
    emitter.instruction("add r11, 1");                                          // include the digit in the bounded decimal string length
    emitter.instruction("mov r8, rax");                                         // continue with the unsigned quotient
    emitter.instruction("test r8, r8");                                         // did the quotient retain more decimal digits?
    emitter.instruction("jne __rt_key_compare_regular_x_decimal_digits");       // render every magnitude digit, including the INT64_MIN magnitude
    emitter.instruction("test r9, r9");                                         // does the rendered magnitude need its leading sign?
    emitter.instruction("je __rt_key_compare_regular_x_decimal_ready");         // omit the sign for non-negative values
    emitter.instruction("sub rcx, 1");                                          // reserve the leading minus-sign byte in the private buffer
    emitter.instruction("mov BYTE PTR [rcx], 45");                              // prefix the decimal magnitude with the ASCII minus sign
    emitter.instruction("add r11, 1");                                          // include the leading sign in the string length
    emitter.label("__rt_key_compare_regular_x_decimal_ready");
    emitter.instruction("cmp QWORD PTR [rbp - 64], 0");                         // is the integer the left comparison operand?
    emitter.instruction("je __rt_key_compare_regular_x_lexical_str_left");      // arrange bounded comparator arguments for a string-left pair
    emitter.instruction("mov rdi, rcx");                                        // pass the decimal integer pointer as the left string
    emitter.instruction("mov rsi, r11");                                        // pass the decimal integer length as the left string
    emitter.instruction("mov rdx, QWORD PTR [rbp - 40]");                       // pass the original non-numeric string pointer as the right string
    emitter.instruction("mov rcx, QWORD PTR [rbp - 48]");                       // pass the original non-numeric string length as the right string
    emitter.instruction("jmp __rt_key_compare_regular_x_lexical_compare");      // compare the arranged bounded strings
    emitter.label("__rt_key_compare_regular_x_lexical_str_left");
    emitter.instruction("mov r8, rcx");                                         // retain the private decimal pointer while loading the left string
    emitter.instruction("mov r9, r11");                                         // retain the private decimal length while loading the left string
    emitter.instruction("mov rdi, QWORD PTR [rbp - 40]");                       // pass the original non-numeric string pointer as the left string
    emitter.instruction("mov rsi, QWORD PTR [rbp - 48]");                       // pass the original non-numeric string length as the left string
    emitter.instruction("mov rdx, r8");                                         // pass the private decimal integer pointer as the right string
    emitter.instruction("mov rcx, r9");                                         // pass the private decimal integer length as the right string
    emitter.label("__rt_key_compare_regular_x_lexical_compare");
    emitter.instruction("call __rt_strcmp");                                    // compare non-numeric mixed operands as bounded strings
    emitter.instruction("test rax, rax");                                       // normalize the lexical mixed-comparison result
    emitter.instruction("setl al");                                             // materialize whether the left lexical value is smaller
    emitter.instruction("setg dl");                                             // materialize whether the left lexical value is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx edx, dl");                                       // widen the greater-than flag
    emitter.instruction("sub rdx, rax");                                        // form -1, 0, or 1 from the two flags
    emitter.instruction("mov rax, rdx");                                        // return the normalized lexical ordering

    emitter.label("__rt_key_compare_regular_x_done");
    emitter.instruction("add rsp, 128");                                        // release the regular-comparison helper frame
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");                                                 // return the normalized PHP SORT_REGULAR ordering
}

/// Classifies integer spellings and compares every pair with at least one signed-64 value exactly.
///
/// Two overflowing spellings return `3` so the caller can reproduce PHP's binary64 comparison and
/// bounded lexical tiebreak. Non-integer spellings return `2` for the ordinary numeric fallback.
fn emit_exact_decimal_key_compare_aarch64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_key_compare_exact_decimal_integers");
    emitter.instruction("sub sp, sp, #64");                                     // reserve parse metadata and the saved return state
    emitter.instruction("stp x29, x30, [sp, #48]");                             // preserve the caller frame and return address
    emitter.instruction("str x3, [sp, #24]");                                   // preserve the right spelling pointer across the left parse
    emitter.instruction("str x4, [sp, #32]");                                   // preserve the right spelling length across the left parse
    emitter.instruction("bl __rt_key_parse_i64_decimal");                       // classify the left bounded decimal integer spelling
    emitter.instruction("cbz x0, __rt_key_compare_exact_decimal_noninteger");   // non-integer spellings use the general numeric fallback
    emitter.instruction("str x0, [sp, #0]");                                    // preserve whether the left value overflowed signed 64-bit
    emitter.instruction("str x1, [sp, #8]");                                    // preserve the left signed value when it fits
    emitter.instruction("str x2, [sp, #16]");                                   // preserve the left sign for overflow ordering
    emitter.instruction("ldr x1, [sp, #24]");                                   // load the right bounded spelling pointer
    emitter.instruction("ldr x2, [sp, #32]");                                   // load the right bounded spelling length
    emitter.instruction("bl __rt_key_parse_i64_decimal");                       // classify the right bounded decimal integer spelling
    emitter.instruction("cbz x0, __rt_key_compare_exact_decimal_noninteger");   // non-integer spellings use the general numeric fallback
    emitter.instruction("ldr x9, [sp, #0]");                                    // recover the left fit-or-overflow classification
    emitter.instruction("cmp x9, #2");                                          // did the left integer overflow signed 64-bit?
    emitter.instruction("b.ne __rt_key_compare_exact_decimal_left_fits");       // compare through signed values unless only the right overflowed
    emitter.instruction("cmp x0, #2");                                          // did the right integer overflow signed 64-bit too?
    emitter.instruction("b.eq __rt_key_compare_exact_decimal_both_overflow");   // defer two overflow values to binary64 plus lexical tiebreak
    emitter.instruction("ldr x9, [sp, #16]");                                   // recover the sign of the overflowing left value
    emitter.instruction("cbnz x9, __rt_key_compare_exact_decimal_left_less");   // negative overflow is below every signed 64-bit value
    emitter.instruction("mov x0, #1");                                          // positive overflow is above every signed 64-bit value
    emitter.instruction("b __rt_key_compare_exact_decimal_done");               // return the exact mixed-range ordering
    emitter.label("__rt_key_compare_exact_decimal_left_fits");
    emitter.instruction("cmp x0, #2");                                          // did only the right integer overflow signed 64-bit?
    emitter.instruction("b.ne __rt_key_compare_exact_decimal_both_fit");        // signed values can be compared directly when both fit
    emitter.instruction("cbnz x2, __rt_key_compare_exact_decimal_left_greater");// negative right overflow is below every signed value
    emitter.label("__rt_key_compare_exact_decimal_left_less");
    emitter.instruction("mov x0, #-1");                                         // return that the left mathematical integer is smaller
    emitter.instruction("b __rt_key_compare_exact_decimal_done");               // skip the remaining classification cases
    emitter.label("__rt_key_compare_exact_decimal_left_greater");
    emitter.instruction("mov x0, #1");                                          // return that the left mathematical integer is greater
    emitter.instruction("b __rt_key_compare_exact_decimal_done");               // skip the signed-value comparison
    emitter.label("__rt_key_compare_exact_decimal_both_fit");
    emitter.instruction("ldr x9, [sp, #8]");                                    // recover the left signed 64-bit value
    emitter.instruction("cmp x9, x1");                                          // compare two exactly represented signed integers
    emitter.instruction("cset x9, lt");                                         // materialize whether the left value is smaller
    emitter.instruction("cset x10, gt");                                        // materialize whether the left value is greater
    emitter.instruction("sub x0, x10, x9");                                     // normalize the signed comparison to -1, 0, or 1
    emitter.instruction("b __rt_key_compare_exact_decimal_done");               // return the exact signed ordering
    emitter.label("__rt_key_compare_exact_decimal_both_overflow");
    emitter.instruction("mov x0, #3");                                          // request PHP's numeric comparison and equal-double lexical tiebreak
    emitter.instruction("b __rt_key_compare_exact_decimal_done");               // return the dedicated overflow-pair classification
    emitter.label("__rt_key_compare_exact_decimal_noninteger");
    emitter.instruction("mov x0, #2");                                          // request the ordinary numeric-or-lexical fallback
    emitter.label("__rt_key_compare_exact_decimal_done");
    emitter.instruction("ldp x29, x30, [sp, #48]");                             // restore the caller frame and return address
    emitter.instruction("add sp, sp, #64");                                     // release the aligned metadata frame
    emitter.instruction("ret");                                                 // return the normalized comparison or fallback classification

    emitter.label_global("__rt_key_parse_i64_decimal");
    emitter.instruction("mov x9, x1");                                          // exact decimal helper instruction
    emitter.instruction("mov x10, x2");                                         // exact decimal helper instruction
    emitter.instruction("mov x11, #0");                                         // exact decimal helper instruction
    emitter.instruction("mov x12, #0");                                         // exact decimal helper instruction
    emitter.instruction("mov x15, #10");                                        // exact decimal helper instruction
    emitter.instruction("mov x16, #0");                                         // track magnitude overflow while still validating every bounded byte
    emitter.label("__rt_key_parse_i64_decimal_ws");
    emitter.instruction("cbz x10, __rt_key_parse_i64_decimal_fail");            // exact decimal helper instruction
    emitter.instruction("ldrb w13, [x9]");                                      // exact decimal helper instruction
    emitter.instruction("cmp w13, #32");                                        // exact decimal helper instruction
    emitter.instruction("b.eq __rt_key_parse_i64_decimal_ws_next");             // exact decimal helper instruction
    emitter.instruction("sub w14, w13, #9");                                    // exact decimal helper instruction
    emitter.instruction("cmp w14, #4");                                         // exact decimal helper instruction
    emitter.instruction("b.ls __rt_key_parse_i64_decimal_ws_next");             // exact decimal helper instruction
    emitter.instruction("b __rt_key_parse_i64_decimal_sign");                   // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_ws_next");
    emitter.instruction("add x9, x9, #1");                                      // exact decimal helper instruction
    emitter.instruction("sub x10, x10, #1");                                    // exact decimal helper instruction
    emitter.instruction("b __rt_key_parse_i64_decimal_ws");                     // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_sign");
    emitter.instruction("cmp w13, #45");                                        // exact decimal helper instruction
    emitter.instruction("b.ne __rt_key_parse_i64_decimal_plus");                // exact decimal helper instruction
    emitter.instruction("mov x11, #1");                                         // exact decimal helper instruction
    emitter.instruction("add x9, x9, #1");                                      // exact decimal helper instruction
    emitter.instruction("sub x10, x10, #1");                                    // exact decimal helper instruction
    emitter.instruction("b __rt_key_parse_i64_decimal_digits");                 // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_plus");
    emitter.instruction("cmp w13, #43");                                        // exact decimal helper instruction
    emitter.instruction("b.ne __rt_key_parse_i64_decimal_digits");              // exact decimal helper instruction
    emitter.instruction("add x9, x9, #1");                                      // exact decimal helper instruction
    emitter.instruction("sub x10, x10, #1");                                    // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_digits");
    emitter.instruction("cbz x10, __rt_key_parse_i64_decimal_fail");            // exact decimal helper instruction
    emitter.instruction("mov x14, #0");                                         // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_digit_loop");
    emitter.instruction("cbz x10, __rt_key_parse_i64_decimal_finish");          // exact decimal helper instruction
    emitter.instruction("ldrb w13, [x9]");                                      // exact decimal helper instruction
    emitter.instruction("sub w17, w13, #48");                                   // exact decimal helper instruction
    emitter.instruction("cmp w17, #9");                                         // exact decimal helper instruction
    emitter.instruction("b.hi __rt_key_parse_i64_decimal_tail");                // exact decimal helper instruction
    emitter.instruction("cbnz x16, __rt_key_parse_i64_decimal_digit_next");     // an overflowed magnitude only needs the remaining grammar scan
    emitter.instruction("umulh x13, x12, x15");                                 // exact decimal helper instruction
    emitter.instruction("cbnz x13, __rt_key_parse_i64_decimal_overflow_digit"); // retain the overflow class instead of rejecting an integer spelling
    emitter.instruction("mul x12, x12, x15");                                   // exact decimal helper instruction
    emitter.instruction("adds x12, x12, x17");                                  // exact decimal helper instruction
    emitter.instruction("b.cc __rt_key_parse_i64_decimal_digit_next");          // retain the accumulated magnitude while it remains unsigned-64 safe
    emitter.label("__rt_key_parse_i64_decimal_overflow_digit");
    emitter.instruction("mov x16, #1");                                         // classify the spelling as an arbitrary-size integer
    emitter.label("__rt_key_parse_i64_decimal_digit_next");
    emitter.instruction("add x9, x9, #1");                                      // exact decimal helper instruction
    emitter.instruction("sub x10, x10, #1");                                    // exact decimal helper instruction
    emitter.instruction("mov x14, #1");                                         // exact decimal helper instruction
    emitter.instruction("b __rt_key_parse_i64_decimal_digit_loop");             // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_tail");
    emitter.instruction("cbz x14, __rt_key_parse_i64_decimal_fail");            // exact decimal helper instruction
    emitter.instruction("cmp w13, #32");                                        // exact decimal helper instruction
    emitter.instruction("b.eq __rt_key_parse_i64_decimal_tail_next");           // exact decimal helper instruction
    emitter.instruction("sub w17, w13, #9");                                    // exact decimal helper instruction
    emitter.instruction("cmp w17, #4");                                         // exact decimal helper instruction
    emitter.instruction("b.hi __rt_key_parse_i64_decimal_fail");                // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_tail_next");
    emitter.instruction("add x9, x9, #1");                                      // exact decimal helper instruction
    emitter.instruction("sub x10, x10, #1");                                    // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_tail_loop");
    emitter.instruction("cbz x10, __rt_key_parse_i64_decimal_finish");          // exact decimal helper instruction
    emitter.instruction("ldrb w13, [x9]");                                      // exact decimal helper instruction
    emitter.instruction("cmp w13, #32");                                        // exact decimal helper instruction
    emitter.instruction("b.eq __rt_key_parse_i64_decimal_tail_next");           // exact decimal helper instruction
    emitter.instruction("sub w17, w13, #9");                                    // exact decimal helper instruction
    emitter.instruction("cmp w17, #4");                                         // exact decimal helper instruction
    emitter.instruction("b.ls __rt_key_parse_i64_decimal_tail_next");           // exact decimal helper instruction
    emitter.instruction("b __rt_key_parse_i64_decimal_fail");                   // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_finish");
    emitter.instruction("cbnz x16, __rt_key_parse_i64_decimal_overflow");       // report a fully validated magnitude beyond unsigned 64-bit
    emitter.instruction("mov x13, #0xffff");                                    // exact decimal helper instruction
    emitter.instruction("movk x13, #0xffff, lsl #16");                          // exact decimal helper instruction
    emitter.instruction("movk x13, #0xffff, lsl #32");                          // exact decimal helper instruction
    emitter.instruction("movk x13, #0x7fff, lsl #48");                          // exact decimal helper instruction
    emitter.instruction("cbz x11, __rt_key_parse_i64_decimal_positive");        // exact decimal helper instruction
    emitter.instruction("add x13, x13, #1");                                    // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_positive");
    emitter.instruction("cmp x12, x13");                                        // exact decimal helper instruction
    emitter.instruction("b.hi __rt_key_parse_i64_decimal_overflow");            // signed-range overflow remains a valid integer spelling
    emitter.instruction("cbz x11, __rt_key_parse_i64_decimal_result");          // exact decimal helper instruction
    emitter.instruction("neg x12, x12");                                        // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_result");
    emitter.instruction("mov x0, #1");                                          // exact decimal helper instruction
    emitter.instruction("mov x1, x12");                                         // exact decimal helper instruction
    emitter.instruction("mov x2, x11");                                         // expose the normalized sign alongside the exact signed value
    emitter.instruction("ret");                                                 // return a signed-64 integer classification
    emitter.label("__rt_key_parse_i64_decimal_overflow");
    emitter.instruction("mov x0, #2");                                          // classify a valid integer spelling outside signed 64-bit
    emitter.instruction("mov x2, x11");                                         // expose its sign for exact mixed-range ordering
    emitter.instruction("ret");                                                 // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_fail");
    emitter.instruction("mov x0, #0");                                          // exact decimal helper instruction
    emitter.instruction("ret");                                                 // exact decimal helper instruction
}

/// Emits the x86_64 integer-spelling classifier and exact mixed-range comparator.
fn emit_exact_decimal_key_compare_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.label_global("__rt_key_compare_exact_decimal_integers");
    emitter.instruction("sub rsp, 56");                                         // reserve parse metadata while preserving SysV call alignment
    emitter.instruction("mov QWORD PTR [rsp + 24], rdx");                       // preserve the right spelling pointer across the left parse
    emitter.instruction("mov QWORD PTR [rsp + 32], rcx");                       // preserve the right spelling length across the left parse
    emitter.instruction("mov rax, rdi");                                        // pass the left bounded spelling pointer to the classifier
    emitter.instruction("mov rdx, rsi");                                        // pass the left bounded spelling length to the classifier
    emitter.instruction("call __rt_key_parse_i64_decimal");                     // classify the left bounded decimal integer spelling
    emitter.instruction("test rax, rax");                                       // was the left spelling an integer without fraction or exponent?
    emitter.instruction("je __rt_key_compare_exact_decimal_x_noninteger");      // non-integer spellings use the general numeric fallback
    emitter.instruction("mov QWORD PTR [rsp], rax");                            // preserve whether the left value overflowed signed 64-bit
    emitter.instruction("mov QWORD PTR [rsp + 8], rdx");                        // preserve the left signed value when it fits
    emitter.instruction("mov QWORD PTR [rsp + 16], r8");                        // preserve the left sign for overflow ordering
    emitter.instruction("mov rax, QWORD PTR [rsp + 24]");                       // load the right bounded spelling pointer
    emitter.instruction("mov rdx, QWORD PTR [rsp + 32]");                       // load the right bounded spelling length
    emitter.instruction("call __rt_key_parse_i64_decimal");                     // classify the right bounded decimal integer spelling
    emitter.instruction("test rax, rax");                                       // was the right spelling an integer without fraction or exponent?
    emitter.instruction("je __rt_key_compare_exact_decimal_x_noninteger");      // non-integer spellings use the general numeric fallback
    emitter.instruction("cmp QWORD PTR [rsp], 2");                              // did the left integer overflow signed 64-bit?
    emitter.instruction("jne __rt_key_compare_exact_decimal_x_left_fits");      // compare through signed values unless only the right overflowed
    emitter.instruction("cmp rax, 2");                                          // did the right integer overflow signed 64-bit too?
    emitter.instruction("je __rt_key_compare_exact_decimal_x_both_overflow");   // defer two overflow values to binary64 plus lexical tiebreak
    emitter.instruction("cmp QWORD PTR [rsp + 16], 0");                         // was the overflowing left integer negative?
    emitter.instruction("jne __rt_key_compare_exact_decimal_x_left_less");      // negative overflow is below every signed 64-bit value
    emitter.instruction("mov rax, 1");                                          // positive overflow is above every signed 64-bit value
    emitter.instruction("jmp __rt_key_compare_exact_decimal_x_done");           // return the exact mixed-range ordering
    emitter.label("__rt_key_compare_exact_decimal_x_left_fits");
    emitter.instruction("cmp rax, 2");                                          // did only the right integer overflow signed 64-bit?
    emitter.instruction("jne __rt_key_compare_exact_decimal_x_both_fit");       // signed values can be compared directly when both fit
    emitter.instruction("test r8, r8");                                         // was the overflowing right integer negative?
    emitter.instruction("jne __rt_key_compare_exact_decimal_x_left_greater");   // negative right overflow is below every signed value
    emitter.label("__rt_key_compare_exact_decimal_x_left_less");
    emitter.instruction("mov rax, -1");                                         // return that the left mathematical integer is smaller
    emitter.instruction("jmp __rt_key_compare_exact_decimal_x_done");           // skip the remaining classification cases
    emitter.label("__rt_key_compare_exact_decimal_x_left_greater");
    emitter.instruction("mov rax, 1");                                          // return that the left mathematical integer is greater
    emitter.instruction("jmp __rt_key_compare_exact_decimal_x_done");           // skip the signed-value comparison
    emitter.label("__rt_key_compare_exact_decimal_x_both_fit");
    emitter.instruction("mov r9, QWORD PTR [rsp + 8]");                         // recover the left signed 64-bit value
    emitter.instruction("cmp r9, rdx");                                         // compare two exactly represented signed integers
    emitter.instruction("setl al");                                             // materialize whether the left value is smaller
    emitter.instruction("setg cl");                                             // materialize whether the left value is greater
    emitter.instruction("movzx eax, al");                                       // widen the smaller-than flag
    emitter.instruction("movzx ecx, cl");                                       // widen the greater-than flag
    emitter.instruction("sub rcx, rax");                                        // form -1, 0, or 1 from the relation flags
    emitter.instruction("mov rax, rcx");                                        // return the normalized signed ordering
    emitter.instruction("jmp __rt_key_compare_exact_decimal_x_done");           // release the helper frame
    emitter.label("__rt_key_compare_exact_decimal_x_both_overflow");
    emitter.instruction("mov rax, 3");                                          // request PHP's numeric comparison and equal-double lexical tiebreak
    emitter.instruction("jmp __rt_key_compare_exact_decimal_x_done");           // return the dedicated overflow-pair classification
    emitter.label("__rt_key_compare_exact_decimal_x_noninteger");
    emitter.instruction("mov rax, 2");                                          // request the ordinary numeric-or-lexical fallback
    emitter.label("__rt_key_compare_exact_decimal_x_done");
    emitter.instruction("add rsp, 56");                                         // release the aligned parse-metadata frame
    emitter.instruction("ret");                                                 // return the normalized comparison or fallback classification

    emitter.label_global("__rt_key_parse_i64_decimal");
    emitter.instruction("mov r9, rax");                                         // exact decimal helper instruction
    emitter.instruction("mov r10, rdx");                                        // exact decimal helper instruction
    emitter.instruction("xor r8d, r8d");                                        // exact decimal helper instruction
    emitter.instruction("xor edi, edi");                                        // exact decimal helper instruction
    emitter.instruction("mov rsi, 10");                                         // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_ws");
    emitter.instruction("test r10, r10");                                       // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_fail");                // exact decimal helper instruction
    emitter.instruction("movzx ecx, BYTE PTR [r9]");                            // exact decimal helper instruction
    emitter.instruction("cmp ecx, 32");                                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_ws_next");             // exact decimal helper instruction
    emitter.instruction("mov edx, ecx");                                        // preserve the original byte for optional sign matching
    emitter.instruction("sub edx, 9");                                          // normalize the control-whitespace range in a separate scratch register
    emitter.instruction("cmp edx, 4");                                          // accept only tab through carriage return as leading whitespace
    emitter.instruction("jbe __rt_key_parse_i64_decimal_x_ws_next");            // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_sign");               // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_ws_next");
    emitter.instruction("add r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("sub r10, 1");                                          // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_ws");                 // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_sign");
    emitter.instruction("cmp ecx, 45");                                         // exact decimal helper instruction
    emitter.instruction("jne __rt_key_parse_i64_decimal_x_plus");               // exact decimal helper instruction
    emitter.instruction("mov r8, 1");                                           // exact decimal helper instruction
    emitter.instruction("add r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("sub r10, 1");                                          // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_digits");             // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_plus");
    emitter.instruction("cmp ecx, 43");                                         // exact decimal helper instruction
    emitter.instruction("jne __rt_key_parse_i64_decimal_x_digits");             // exact decimal helper instruction
    emitter.instruction("add r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("sub r10, 1");                                          // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_digits");
    emitter.instruction("test r10, r10");                                       // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_fail");                // exact decimal helper instruction
    emitter.instruction("xor r11d, r11d");                                      // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_digit_loop");
    emitter.instruction("test r10, r10");                                       // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_finish");              // exact decimal helper instruction
    emitter.instruction("movzx ecx, BYTE PTR [r9]");                            // exact decimal helper instruction
    emitter.instruction("sub ecx, 48");                                         // exact decimal helper instruction
    emitter.instruction("cmp ecx, 9");                                          // exact decimal helper instruction
    emitter.instruction("ja __rt_key_parse_i64_decimal_x_tail");                // exact decimal helper instruction
    emitter.instruction("cmp r11, 2");                                          // has an earlier digit already overflowed unsigned 64-bit?
    emitter.instruction("je __rt_key_parse_i64_decimal_x_digit_next");          // continue the bounded grammar scan without further arithmetic
    emitter.instruction("mov rax, rdi");                                        // exact decimal helper instruction
    emitter.instruction("mul rsi");                                             // exact decimal helper instruction
    emitter.instruction("test rdx, rdx");                                       // exact decimal helper instruction
    emitter.instruction("jne __rt_key_parse_i64_decimal_x_overflow_digit");     // retain the overflow class instead of rejecting an integer spelling
    emitter.instruction("add rax, rcx");                                        // exact decimal helper instruction
    emitter.instruction("jc __rt_key_parse_i64_decimal_x_overflow_digit");      // retain carry-out as arbitrary-size integer classification
    emitter.instruction("mov rdi, rax");                                        // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_digit_next");         // consume the digit after a safe accumulation
    emitter.label("__rt_key_parse_i64_decimal_x_overflow_digit");
    emitter.instruction("mov r11, 2");                                          // mark magnitude overflow while validating all remaining bytes
    emitter.label("__rt_key_parse_i64_decimal_x_digit_next");
    emitter.instruction("add r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("sub r10, 1");                                          // exact decimal helper instruction
    emitter.instruction("cmp r11, 2");                                          // preserve the overflow classification across later digits
    emitter.instruction("je __rt_key_parse_i64_decimal_x_digit_loop");          // continue scanning an already-overflowed spelling
    emitter.instruction("mov r11, 1");                                          // remember that at least one in-range digit was consumed
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_digit_loop");         // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_tail");
    emitter.instruction("test r11, r11");                                       // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_fail");                // exact decimal helper instruction
    emitter.instruction("cmp ecx, -16");                                        // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_tail_next");           // exact decimal helper instruction
    emitter.instruction("sub ecx, -39");                                        // exact decimal helper instruction
    emitter.instruction("cmp ecx, 4");                                          // exact decimal helper instruction
    emitter.instruction("ja __rt_key_parse_i64_decimal_x_fail");                // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_tail_next");
    emitter.instruction("add r9, 1");                                           // exact decimal helper instruction
    emitter.instruction("sub r10, 1");                                          // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_tail_loop");
    emitter.instruction("test r10, r10");                                       // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_finish");              // exact decimal helper instruction
    emitter.instruction("movzx ecx, BYTE PTR [r9]");                            // exact decimal helper instruction
    emitter.instruction("cmp ecx, 32");                                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_tail_next");           // exact decimal helper instruction
    emitter.instruction("sub ecx, 9");                                          // exact decimal helper instruction
    emitter.instruction("cmp ecx, 4");                                          // exact decimal helper instruction
    emitter.instruction("jbe __rt_key_parse_i64_decimal_x_tail_next");          // exact decimal helper instruction
    emitter.instruction("jmp __rt_key_parse_i64_decimal_x_fail");               // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_finish");
    emitter.instruction("cmp r11, 2");                                          // did the validated magnitude overflow unsigned 64-bit?
    emitter.instruction("je __rt_key_parse_i64_decimal_x_overflow");            // return an overflow-integer class without allocation or copying
    emitter.instruction("mov r11, 0x7fffffffffffffff");                         // exact decimal helper instruction
    emitter.instruction("test r8, r8");                                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_positive");            // exact decimal helper instruction
    emitter.instruction("add r11, 1");                                          // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_positive");
    emitter.instruction("cmp rdi, r11");                                        // exact decimal helper instruction
    emitter.instruction("ja __rt_key_parse_i64_decimal_x_overflow");            // signed-range overflow remains a valid integer spelling
    emitter.instruction("mov rdx, rdi");                                        // exact decimal helper instruction
    emitter.instruction("test r8, r8");                                         // exact decimal helper instruction
    emitter.instruction("je __rt_key_parse_i64_decimal_x_result");              // exact decimal helper instruction
    emitter.instruction("neg rdx");                                             // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_result");
    emitter.instruction("mov rax, 1");                                          // exact decimal helper instruction
    emitter.instruction("ret");                                                 // return a signed-64 integer classification
    emitter.label("__rt_key_parse_i64_decimal_x_overflow");
    emitter.instruction("mov rax, 2");                                          // classify a valid integer spelling outside signed 64-bit
    emitter.instruction("ret");                                                 // exact decimal helper instruction
    emitter.label("__rt_key_parse_i64_decimal_x_fail");
    emitter.instruction("xor rax, rax");                                        // exact decimal helper instruction
    emitter.instruction("ret");                                                 // exact decimal helper instruction
}
