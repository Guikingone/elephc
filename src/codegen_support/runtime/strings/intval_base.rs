//! Purpose:
//! Emits the `__rt_intval_base` runtime helper: a DEDICATED base-aware integer
//! parser backing `intval($value, $base)` for string input when `$base != 10`.
//! Never reuses libc `strtol`/`strtoll` (which does not understand PHP's `"0b"`
//! binary prefix under base-0 auto-detect) and never reuses the stricter
//! `__rt_filter_validate_int` grammar (which rejects prefixes/leading zeros
//! that `intval()` accepts and stops at the first invalid digit instead of
//! rejecting the whole string).
//!
//! Called from:
//! - `crate::codegen_support::runtime::emitters::emit_runtime()` via `crate::codegen_support::runtime::strings`.
//! - `crate::codegen::lower_inst::builtins::lower_intval()` (the `intval()` EIR lowering)
//!   for the `Str` operand case when `$base` is not the literal `10`.
//!
//! Key details:
//! - Grammar (php-verified with `php -n -r 'var_dump(intval(..., $base));'`, PHP 8.5.6
//!   local): `$base == 10` behaves exactly like the default (float-aware leading-numeric-
//!   string) `intval()` conversion, so this helper tail-jumps straight into
//!   `__rt_str_to_int` for that case instead of re-implementing it.
//! - For every other base: optional leading whitespace, optional `+`/`-` sign, then an
//!   OPTIONAL base prefix (`"0x"/"0X"` for base 16, `"0b"/"0B"` for base 2, a single
//!   leading `"0"` for base 8; base `0` auto-detects the base from whichever prefix is
//!   present, defaulting to octal on a bare leading `"0"` and decimal otherwise), then
//!   zero or more digits valid for the effective base — parsing STOPS at the first byte
//!   that is not a valid digit (a prefix parse, not a validator: `"12abc"` parses to `12`).
//! - A base outside `{0} ∪ [2, 36]` always yields `0` (php-verified: `intval("42", 1)`
//!   and `intval("42", 37)` are both `0`).
//! - Overflow saturates to `PHP_INT_MAX`/`PHP_INT_MIN` using the sign parsed BEFORE the
//!   digit run, matching PHP exactly for both directions (`intval(str_repeat("f", 20),
//!   16)` == `PHP_INT_MAX`; the negated form saturates to the exact `PHP_INT_MIN`, not
//!   `-PHP_INT_MAX`, because the accumulator's saturation ceiling is sign-dependent:
//!   `0x7FFF...FFFF` for positive, `0x8000...0000` for negative).

use crate::codegen::{abi, emit::Emitter, platform::Arch};

/// Emits `__rt_intval_base` for the host target.
///
/// AArch64: input x1=ptr, x2=len, x3=base. Output: x0=parsed `i64`.
/// x86_64: input rax=ptr, rdx=len, rdi=base. Output: rax=parsed `i64`.
///
/// Both variants tail-jump into `__rt_str_to_int` (no frame set up yet, so the
/// caller's return address is still valid) whenever `base == 10`.
pub fn emit_intval_base(emitter: &mut Emitter) {
    if emitter.target.arch == Arch::X86_64 {
        emit_intval_base_x86_64(emitter);
        return;
    }

    emitter.blank();
    emitter.comment("--- runtime: intval_base ---");
    emitter.label_global("__rt_intval_base");

    // -- base==10 behaves exactly like default intval(): reuse the float-aware parser --
    // `.subsections_via_symbols` forbids a conditional branch to another global
    // symbol, so hop through a local label and use an unconditional tail-jump.
    emitter.instruction("cmp x3, #10");                                         // is this the PHP-equivalent-to-default base?
    emitter.instruction("b.ne __rt_intval_base_not_ten");                       // non-default bases parse below
    emitter.instruction("b __rt_str_to_int");                                   // tail-jump: no frame set up yet, x30 still valid
    emitter.label("__rt_intval_base_not_ten");

    // -- reject any base outside {0} ∪ [2, 36] --
    emitter.instruction("cbz x3, __rt_intval_base_base_ok");                    // base 0 = auto-detect, always valid
    emitter.instruction("cmp x3, #2");                                          // base must be at least 2
    emitter.instruction("b.lt __rt_intval_base_invalid_base");                  // reject bases below 2
    emitter.instruction("cmp x3, #36");                                         // base must be at most 36 (10 digits + 26 letters)
    emitter.instruction("b.gt __rt_intval_base_invalid_base");                  // reject bases above 36
    emitter.instruction("b __rt_intval_base_base_ok");                          // base is in range
    emitter.label("__rt_intval_base_invalid_base");
    emitter.instruction("mov x0, #0");                                          // an out-of-range base always yields 0
    emitter.instruction("ret");                                                 // return immediately

    emitter.label("__rt_intval_base_base_ok");
    emitter.instruction("mov x4, #0");                                          // cursor index into the string

    // -- skip leading PHP whitespace: space, \t, \n, \r, \v, \f --
    emitter.label("__rt_intval_base_ws_loop");
    emitter.instruction("cmp x4, x2");                                          // reached the end while skipping whitespace?
    emitter.instruction("b.ge __rt_intval_base_no_digits");                     // whitespace-only input parses to 0
    emitter.instruction("ldrb w10, [x1, x4]");                                  // load the current byte
    emitter.instruction("cmp w10, #0x20");                                      // space?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("cmp w10, #0x09");                                      // tab?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("cmp w10, #0x0A");                                      // newline?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("cmp w10, #0x0D");                                      // carriage return?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("cmp w10, #0x0B");                                      // vertical tab?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("cmp w10, #0x0C");                                      // form feed?
    emitter.instruction("b.eq __rt_intval_base_ws_next");
    emitter.instruction("b __rt_intval_base_ws_done");                          // first non-whitespace byte found
    emitter.label("__rt_intval_base_ws_next");
    emitter.instruction("add x4, x4, #1");                                      // consume the whitespace byte
    emitter.instruction("b __rt_intval_base_ws_loop");
    emitter.label("__rt_intval_base_ws_done");

    // -- optional sign --
    emitter.instruction("mov x5, #0");                                          // sign flag: 0 = positive, 1 = negative
    emitter.instruction("cmp x4, x2");                                          // any bytes left for a sign/digit?
    emitter.instruction("b.ge __rt_intval_base_no_digits");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // load the candidate sign byte
    emitter.instruction("cmp w10, #0x2D");                                      // '-'?
    emitter.instruction("b.eq __rt_intval_base_sign_neg");
    emitter.instruction("cmp w10, #0x2B");                                      // '+'?
    emitter.instruction("b.eq __rt_intval_base_sign_pos");
    emitter.instruction("b __rt_intval_base_sign_done");                        // no sign present
    emitter.label("__rt_intval_base_sign_neg");
    emitter.instruction("mov x5, #1");                                          // remember the negative sign
    emitter.instruction("add x4, x4, #1");                                      // consume the sign byte
    emitter.instruction("b __rt_intval_base_sign_done");
    emitter.label("__rt_intval_base_sign_pos");
    emitter.instruction("add x4, x4, #1");                                      // consume the sign byte
    emitter.label("__rt_intval_base_sign_done");

    // -- determine the effective base and consume any matching prefix --
    emitter.instruction("cbz x3, __rt_intval_base_auto");                       // base 0: auto-detect from the prefix
    emitter.instruction("cmp x3, #16");
    emitter.instruction("b.eq __rt_intval_base_maybe_hex");
    emitter.instruction("cmp x3, #2");
    emitter.instruction("b.eq __rt_intval_base_maybe_bin");
    emitter.instruction("cmp x3, #8");
    emitter.instruction("b.eq __rt_intval_base_maybe_oct");
    emitter.instruction("mov x13, x3");                                         // any other explicit base: no prefix to strip
    emitter.instruction("b __rt_intval_base_digits_start");

    emitter.label("__rt_intval_base_maybe_hex");
    emitter.instruction("mov x13, #16");                                        // effective base = 16
    // -- inline "0x"/"0X" prefix strip (no `bl`: this function is a leaf and must not clobber x30) --
    emitter.instruction("cmp x4, x2");                                          // any byte left to inspect?
    emitter.instruction("b.ge __rt_intval_base_digits_start");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // peek the next byte
    emitter.instruction("cmp w10, #0x30");                                      // leading '0'?
    emitter.instruction("b.ne __rt_intval_base_digits_start");
    emitter.instruction("add x9, x4, #1");                                      // index of the byte after the leading zero
    emitter.instruction("cmp x9, x2");                                          // any byte left after the leading zero?
    emitter.instruction("b.ge __rt_intval_base_digits_start");
    emitter.instruction("ldrb w11, [x1, x9]");                                  // peek the marker byte
    emitter.instruction("cmp w11, #0x78");                                      // 'x'?
    emitter.instruction("b.eq __rt_intval_base_strip2_hex");
    emitter.instruction("cmp w11, #0x58");                                      // 'X'?
    emitter.instruction("b.ne __rt_intval_base_digits_start");
    emitter.label("__rt_intval_base_strip2_hex");
    emitter.instruction("add x4, x4, #2");                                      // strip the two-byte prefix
    emitter.instruction("b __rt_intval_base_digits_start");

    emitter.label("__rt_intval_base_maybe_bin");
    emitter.instruction("mov x13, #2");                                         // effective base = 2
    // -- inline "0b"/"0B" prefix strip (no `bl`: this function is a leaf and must not clobber x30) --
    emitter.instruction("cmp x4, x2");                                          // any byte left to inspect?
    emitter.instruction("b.ge __rt_intval_base_digits_start");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // peek the next byte
    emitter.instruction("cmp w10, #0x30");                                      // leading '0'?
    emitter.instruction("b.ne __rt_intval_base_digits_start");
    emitter.instruction("add x9, x4, #1");                                      // index of the byte after the leading zero
    emitter.instruction("cmp x9, x2");                                          // any byte left after the leading zero?
    emitter.instruction("b.ge __rt_intval_base_digits_start");
    emitter.instruction("ldrb w11, [x1, x9]");                                  // peek the marker byte
    emitter.instruction("cmp w11, #0x62");                                      // 'b'?
    emitter.instruction("b.eq __rt_intval_base_strip2_bin");
    emitter.instruction("cmp w11, #0x42");                                      // 'B'?
    emitter.instruction("b.ne __rt_intval_base_digits_start");
    emitter.label("__rt_intval_base_strip2_bin");
    emitter.instruction("add x4, x4, #2");                                      // strip the two-byte prefix
    emitter.instruction("b __rt_intval_base_digits_start");

    emitter.label("__rt_intval_base_maybe_oct");
    emitter.instruction("mov x13, #8");                                         // effective base = 8
    emitter.instruction("cmp x4, x2");                                          // any byte left to inspect?
    emitter.instruction("b.ge __rt_intval_base_digits_start");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // peek the next byte
    emitter.instruction("cmp w10, #0x30");                                      // is it a leading '0'?
    emitter.instruction("b.ne __rt_intval_base_digits_start");                  // no prefix to strip
    emitter.instruction("add x4, x4, #1");                                      // strip the single leading zero
    emitter.instruction("b __rt_intval_base_digits_start");

    emitter.label("__rt_intval_base_auto");
    emitter.instruction("cmp x4, x2");                                          // any byte left to inspect?
    emitter.instruction("b.ge __rt_intval_base_auto_decimal");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // peek the next byte
    emitter.instruction("cmp w10, #0x30");                                      // is it a leading '0'?
    emitter.instruction("b.ne __rt_intval_base_auto_decimal");                  // no leading zero: base 10
    emitter.instruction("add x9, x4, #1");                                      // index of the byte after the leading zero
    emitter.instruction("cmp x9, x2");                                          // any byte left after the leading zero?
    emitter.instruction("b.ge __rt_intval_base_auto_octal");                    // "0" alone: octal with no digits left
    emitter.instruction("ldrb w11, [x1, x9]");                                  // peek the byte after the leading zero
    emitter.instruction("cmp w11, #0x78");                                      // 'x'?
    emitter.instruction("b.eq __rt_intval_base_auto_hex");
    emitter.instruction("cmp w11, #0x58");                                      // 'X'?
    emitter.instruction("b.eq __rt_intval_base_auto_hex");
    emitter.instruction("cmp w11, #0x62");                                      // 'b'?
    emitter.instruction("b.eq __rt_intval_base_auto_bin");
    emitter.instruction("cmp w11, #0x42");                                      // 'B'?
    emitter.instruction("b.eq __rt_intval_base_auto_bin");
    emitter.instruction("b __rt_intval_base_auto_octal");                       // leading zero with no x/b marker: octal
    emitter.label("__rt_intval_base_auto_hex");
    emitter.instruction("mov x13, #16");                                        // auto-detected hexadecimal
    emitter.instruction("add x4, x4, #2");                                      // strip the "0x"/"0X" prefix
    emitter.instruction("b __rt_intval_base_digits_start");
    emitter.label("__rt_intval_base_auto_bin");
    emitter.instruction("mov x13, #2");                                         // auto-detected binary
    emitter.instruction("add x4, x4, #2");                                      // strip the "0b"/"0B" prefix
    emitter.instruction("b __rt_intval_base_digits_start");
    emitter.label("__rt_intval_base_auto_octal");
    emitter.instruction("mov x13, #8");                                         // auto-detected octal
    emitter.instruction("add x4, x4, #1");                                      // strip the single leading zero
    emitter.instruction("b __rt_intval_base_digits_start");
    emitter.label("__rt_intval_base_auto_decimal");
    emitter.instruction("mov x13, #10");                                        // no leading zero: auto-detected decimal
    emitter.instruction("b __rt_intval_base_digits_start");

    // -- digit scan: accumulate with sign-dependent saturating overflow, stop at the first invalid digit --
    emitter.label("__rt_intval_base_digits_start");
    emitter.instruction("cbz x5, __rt_intval_base_mag_pos");                    // branch on the sign parsed earlier
    abi::emit_load_int_immediate(emitter, "x9", i64::MIN);                     // negative ceiling: |PHP_INT_MIN|, bit pattern 0x8000000000000000
    emitter.instruction("b __rt_intval_base_mag_set");
    emitter.label("__rt_intval_base_mag_pos");
    abi::emit_load_int_immediate(emitter, "x9", i64::MAX);                     // positive ceiling: PHP_INT_MAX
    emitter.label("__rt_intval_base_mag_set");
    emitter.instruction("mov x7, #0");                                          // acc: unsigned magnitude accumulator
    emitter.instruction("mov x6, #0");                                          // digit-seen flag

    emitter.label("__rt_intval_base_digit_loop");
    emitter.instruction("cmp x4, x2");                                          // reached the end of the string?
    emitter.instruction("b.ge __rt_intval_base_digits_done");
    emitter.instruction("ldrb w10, [x1, x4]");                                  // load the current byte
    emitter.instruction("cmp w10, #0x30");                                      // below '0'?
    emitter.instruction("b.lt __rt_intval_base_digits_done");                   // not a digit: stop the prefix parse
    emitter.instruction("cmp w10, #0x39");                                      // '0'-'9' range?
    emitter.instruction("b.gt __rt_intval_base_digit_alpha");
    emitter.instruction("sub w10, w10, #0x30");                                 // digit value 0-9
    emitter.instruction("b __rt_intval_base_digit_have");
    emitter.label("__rt_intval_base_digit_alpha");
    emitter.instruction("cmp w10, #0x61");                                      // 'a'?
    emitter.instruction("b.lt __rt_intval_base_digit_alpha_upper");
    emitter.instruction("cmp w10, #0x7A");                                      // 'z'?
    emitter.instruction("b.gt __rt_intval_base_digits_done");                   // not a digit at all: stop
    emitter.instruction("sub w10, w10, #0x61");                                 // normalize 'a'-'z'
    emitter.instruction("add w10, w10, #10");                                   // digit value 10-35
    emitter.instruction("b __rt_intval_base_digit_have");
    emitter.label("__rt_intval_base_digit_alpha_upper");
    emitter.instruction("cmp w10, #0x41");                                      // 'A'?
    emitter.instruction("b.lt __rt_intval_base_digits_done");                   // not a digit at all: stop
    emitter.instruction("cmp w10, #0x5A");                                      // 'Z'?
    emitter.instruction("b.gt __rt_intval_base_digits_done");                   // not a digit at all: stop
    emitter.instruction("sub w10, w10, #0x41");                                 // normalize 'A'-'Z'
    emitter.instruction("add w10, w10, #10");                                   // digit value 10-35
    emitter.label("__rt_intval_base_digit_have");
    emitter.instruction("cmp x10, x13");                                        // is the digit valid for the effective base?
    emitter.instruction("b.hs __rt_intval_base_digits_done");                   // digit >= base: not valid here, stop

    emitter.instruction("umulh x11, x7, x13");                                  // high 64 bits of acc*base
    emitter.instruction("cbnz x11, __rt_intval_base_overflow");                 // the multiply alone overflowed 64 bits
    emitter.instruction("mul x12, x7, x13");                                    // low 64 bits: acc*base (exact since high==0)
    emitter.instruction("subs x8, x9, x12");                                    // ceiling - acc*base (unsigned)
    emitter.instruction("b.lo __rt_intval_base_overflow");                      // acc*base already exceeds the ceiling
    emitter.instruction("cmp x10, x8");                                         // does this digit push past the remaining headroom?
    emitter.instruction("b.hi __rt_intval_base_overflow");
    emitter.instruction("add x7, x12, x10");                                    // acc = acc*base + digit
    emitter.instruction("mov x6, #1");                                          // record that a digit was consumed
    emitter.instruction("add x4, x4, #1");                                      // advance the cursor
    emitter.instruction("b __rt_intval_base_digit_loop");

    emitter.label("__rt_intval_base_overflow");
    emitter.instruction("mov x7, x9");                                          // saturate the accumulator to the sign-dependent ceiling
    emitter.instruction("mov x6, #1");                                          // an overflowing run still counts as digits consumed
    emitter.instruction("b __rt_intval_base_digits_done");

    emitter.label("__rt_intval_base_digits_done");
    emitter.instruction("cbz x6, __rt_intval_base_no_digits");                  // no valid digits after the prefix: result 0
    emitter.instruction("cbz x5, __rt_intval_base_result_pos");
    emitter.instruction("neg x0, x7");                                          // negative result (wraps 0x8000...0000 to exactly PHP_INT_MIN)
    emitter.instruction("ret");
    emitter.label("__rt_intval_base_result_pos");
    emitter.instruction("mov x0, x7");                                          // positive result is the accumulated magnitude
    emitter.instruction("ret");

    emitter.label("__rt_intval_base_no_digits");
    emitter.instruction("mov x0, #0");                                          // no digits parsed at all: PHP intval() returns 0
    emitter.instruction("ret");
}

/// Emits the x86_64 System V variant of `__rt_intval_base`.
///
/// Register plan (leaf function; only caller-saved scratch registers are used
/// per this repo's `__rt_*` runtime adapter convention): r8=ptr, r9=len,
/// rcx=cursor, r10=effective base, r11=acc, rsi=ceiling, rdi=digit value
/// (transient), rax/rdx are used transiently by `mul`. The sign flag is
/// spilled to the stack once computed, freeing a register for the digit loop.
fn emit_intval_base_x86_64(emitter: &mut Emitter) {
    emitter.blank();
    emitter.comment("--- runtime: intval_base ---");
    emitter.label_global("__rt_intval_base");

    // -- base==10 behaves exactly like default intval(): reuse the float-aware parser --
    emitter.instruction("cmp rdi, 10");                                         // is this the PHP-equivalent-to-default base?
    emitter.instruction("je __rt_str_to_int");                                  // tail-jump: no frame set up yet, the return address is still valid

    // -- reject any base outside {0} ∪ [2, 36] --
    emitter.instruction("test rdi, rdi");                                       // base 0 = auto-detect, always valid
    emitter.instruction("jz __rt_intval_base_base_ok_x86_64");
    emitter.instruction("cmp rdi, 2");                                          // base must be at least 2
    emitter.instruction("jl __rt_intval_base_invalid_base_x86_64");
    emitter.instruction("cmp rdi, 36");                                         // base must be at most 36
    emitter.instruction("jg __rt_intval_base_invalid_base_x86_64");
    emitter.instruction("jmp __rt_intval_base_base_ok_x86_64");
    emitter.label("__rt_intval_base_invalid_base_x86_64");
    emitter.instruction("xor eax, eax");                                        // an out-of-range base always yields 0
    emitter.instruction("ret");

    emitter.label("__rt_intval_base_base_ok_x86_64");
    emitter.instruction("push rbp");                                            // preserve the caller frame pointer for the sign spill slot
    emitter.instruction("mov rbp, rsp");                                        // establish a stable frame base
    emitter.instruction("sub rsp, 16");                                         // reserve an aligned spill slot for the sign flag
    emitter.instruction("mov r8, rax");                                         // r8 = ptr (freed rax for `mul` use later)
    emitter.instruction("mov r9, rdx");                                         // r9 = len
    emitter.instruction("mov r10, rdi");                                        // r10 = requested base (repurposed into the effective base below)
    emitter.instruction("xor rcx, rcx");                                        // cursor index into the string

    // -- skip leading PHP whitespace: space, \t, \n, \r, \v, \f --
    emitter.label("__rt_intval_base_ws_loop_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // reached the end while skipping whitespace?
    emitter.instruction("jge __rt_intval_base_no_digits_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r8 + rcx]");                      // load the current byte
    emitter.instruction("cmp al, 0x20");                                        // space?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("cmp al, 0x09");                                        // tab?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("cmp al, 0x0A");                                        // newline?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("cmp al, 0x0D");                                        // carriage return?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("cmp al, 0x0B");                                        // vertical tab?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("cmp al, 0x0C");                                        // form feed?
    emitter.instruction("je __rt_intval_base_ws_next_x86_64");
    emitter.instruction("jmp __rt_intval_base_ws_done_x86_64");                 // first non-whitespace byte found
    emitter.label("__rt_intval_base_ws_next_x86_64");
    emitter.instruction("inc rcx");                                             // consume the whitespace byte
    emitter.instruction("jmp __rt_intval_base_ws_loop_x86_64");
    emitter.label("__rt_intval_base_ws_done_x86_64");

    // -- optional sign --
    emitter.instruction("mov QWORD PTR [rbp - 8], 0");                          // sign flag spill: 0 = positive, 1 = negative
    emitter.instruction("cmp rcx, r9");                                         // any bytes left for a sign/digit?
    emitter.instruction("jge __rt_intval_base_no_digits_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r8 + rcx]");                      // load the candidate sign byte
    emitter.instruction("cmp al, 0x2D");                                        // '-'?
    emitter.instruction("je __rt_intval_base_sign_neg_x86_64");
    emitter.instruction("cmp al, 0x2B");                                        // '+'?
    emitter.instruction("je __rt_intval_base_sign_pos_x86_64");
    emitter.instruction("jmp __rt_intval_base_sign_done_x86_64");               // no sign present
    emitter.label("__rt_intval_base_sign_neg_x86_64");
    emitter.instruction("mov QWORD PTR [rbp - 8], 1");                          // remember the negative sign
    emitter.instruction("inc rcx");                                             // consume the sign byte
    emitter.instruction("jmp __rt_intval_base_sign_done_x86_64");
    emitter.label("__rt_intval_base_sign_pos_x86_64");
    emitter.instruction("inc rcx");                                             // consume the sign byte
    emitter.label("__rt_intval_base_sign_done_x86_64");

    // -- determine the effective base and consume any matching prefix --
    emitter.instruction("test r10, r10");                                       // base 0: auto-detect from the prefix
    emitter.instruction("jz __rt_intval_base_auto_x86_64");
    emitter.instruction("cmp r10, 16");
    emitter.instruction("je __rt_intval_base_maybe_hex_x86_64");
    emitter.instruction("cmp r10, 2");
    emitter.instruction("je __rt_intval_base_maybe_bin_x86_64");
    emitter.instruction("cmp r10, 8");
    emitter.instruction("je __rt_intval_base_maybe_oct_x86_64");
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");            // any other explicit base: no prefix to strip (r10 already holds it)

    emitter.label("__rt_intval_base_maybe_hex_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // any byte left to inspect?
    emitter.instruction("jge __rt_intval_base_digits_start_x86_64");
    emitter.instruction("cmp BYTE PTR [r8 + rcx], 0x30");                       // leading '0'?
    emitter.instruction("jne __rt_intval_base_digits_start_x86_64");
    emitter.instruction("lea rax, [rcx + 1]");                                  // index of the byte after the leading zero
    emitter.instruction("cmp rax, r9");                                         // any byte left after the leading zero?
    emitter.instruction("jge __rt_intval_base_digits_start_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r8 + rax]");                      // peek the marker byte
    emitter.instruction("cmp al, 0x78");                                        // 'x'?
    emitter.instruction("je __rt_intval_base_strip2_hex_x86_64");
    emitter.instruction("cmp al, 0x58");                                        // 'X'?
    emitter.instruction("jne __rt_intval_base_digits_start_x86_64");
    emitter.label("__rt_intval_base_strip2_hex_x86_64");
    emitter.instruction("add rcx, 2");                                          // strip the "0x"/"0X" prefix
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");

    emitter.label("__rt_intval_base_maybe_bin_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // any byte left to inspect?
    emitter.instruction("jge __rt_intval_base_digits_start_x86_64");
    emitter.instruction("cmp BYTE PTR [r8 + rcx], 0x30");                       // leading '0'?
    emitter.instruction("jne __rt_intval_base_digits_start_x86_64");
    emitter.instruction("lea rax, [rcx + 1]");                                  // index of the byte after the leading zero
    emitter.instruction("cmp rax, r9");                                         // any byte left after the leading zero?
    emitter.instruction("jge __rt_intval_base_digits_start_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r8 + rax]");                      // peek the marker byte
    emitter.instruction("cmp al, 0x62");                                        // 'b'?
    emitter.instruction("je __rt_intval_base_strip2_bin_x86_64");
    emitter.instruction("cmp al, 0x42");                                        // 'B'?
    emitter.instruction("jne __rt_intval_base_digits_start_x86_64");
    emitter.label("__rt_intval_base_strip2_bin_x86_64");
    emitter.instruction("add rcx, 2");                                          // strip the "0b"/"0B" prefix
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");

    emitter.label("__rt_intval_base_maybe_oct_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // any byte left to inspect?
    emitter.instruction("jge __rt_intval_base_digits_start_x86_64");
    emitter.instruction("cmp BYTE PTR [r8 + rcx], 0x30");                       // leading '0'?
    emitter.instruction("jne __rt_intval_base_digits_start_x86_64");
    emitter.instruction("inc rcx");                                             // strip the single leading zero
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");

    emitter.label("__rt_intval_base_auto_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // any byte left to inspect?
    emitter.instruction("jge __rt_intval_base_auto_decimal_x86_64");
    emitter.instruction("cmp BYTE PTR [r8 + rcx], 0x30");                       // leading '0'?
    emitter.instruction("jne __rt_intval_base_auto_decimal_x86_64");
    emitter.instruction("lea rax, [rcx + 1]");                                  // index of the byte after the leading zero
    emitter.instruction("cmp rax, r9");                                         // any byte left after the leading zero?
    emitter.instruction("jge __rt_intval_base_auto_octal_x86_64");              // "0" alone: octal with no digits left
    emitter.instruction("movzx eax, BYTE PTR [r8 + rax]");                      // peek the byte after the leading zero
    emitter.instruction("cmp al, 0x78");                                        // 'x'?
    emitter.instruction("je __rt_intval_base_auto_hex_x86_64");
    emitter.instruction("cmp al, 0x58");                                        // 'X'?
    emitter.instruction("je __rt_intval_base_auto_hex_x86_64");
    emitter.instruction("cmp al, 0x62");                                        // 'b'?
    emitter.instruction("je __rt_intval_base_auto_bin_x86_64");
    emitter.instruction("cmp al, 0x42");                                        // 'B'?
    emitter.instruction("je __rt_intval_base_auto_bin_x86_64");
    emitter.instruction("jmp __rt_intval_base_auto_octal_x86_64");              // leading zero with no x/b marker: octal
    emitter.label("__rt_intval_base_auto_hex_x86_64");
    emitter.instruction("mov r10, 16");                                         // auto-detected hexadecimal
    emitter.instruction("add rcx, 2");                                          // strip the "0x"/"0X" prefix
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");
    emitter.label("__rt_intval_base_auto_bin_x86_64");
    emitter.instruction("mov r10, 2");                                          // auto-detected binary
    emitter.instruction("add rcx, 2");                                          // strip the "0b"/"0B" prefix
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");
    emitter.label("__rt_intval_base_auto_octal_x86_64");
    emitter.instruction("mov r10, 8");                                          // auto-detected octal
    emitter.instruction("inc rcx");                                             // strip the single leading zero
    emitter.instruction("jmp __rt_intval_base_digits_start_x86_64");
    emitter.label("__rt_intval_base_auto_decimal_x86_64");
    emitter.instruction("mov r10, 10");                                         // no leading zero: auto-detected decimal

    // -- digit scan: accumulate with sign-dependent saturating overflow, stop at the first invalid digit --
    emitter.label("__rt_intval_base_digits_start_x86_64");
    emitter.instruction("cmp QWORD PTR [rbp - 8], 0");                          // branch on the sign parsed earlier
    emitter.instruction("jne __rt_intval_base_mag_neg_x86_64");
    abi::emit_load_int_immediate(emitter, "rsi", i64::MAX);                    // positive ceiling: PHP_INT_MAX
    emitter.instruction("jmp __rt_intval_base_mag_set_x86_64");
    emitter.label("__rt_intval_base_mag_neg_x86_64");
    abi::emit_load_int_immediate(emitter, "rsi", i64::MIN);                    // negative ceiling: |PHP_INT_MIN|, bit pattern 0x8000000000000000
    emitter.label("__rt_intval_base_mag_set_x86_64");
    emitter.instruction("xor r11, r11");                                        // acc: unsigned magnitude accumulator
    emitter.instruction("mov QWORD PTR [rbp - 16], 0");                         // digit-seen flag spill

    emitter.label("__rt_intval_base_digit_loop_x86_64");
    emitter.instruction("cmp rcx, r9");                                         // reached the end of the string?
    emitter.instruction("jge __rt_intval_base_digits_done_x86_64");
    emitter.instruction("movzx eax, BYTE PTR [r8 + rcx]");                      // load the current byte
    emitter.instruction("cmp al, 0x30");                                        // below '0'?
    emitter.instruction("jl __rt_intval_base_digits_done_x86_64");              // not a digit: stop the prefix parse
    emitter.instruction("cmp al, 0x39");                                        // '0'-'9' range?
    emitter.instruction("jg __rt_intval_base_digit_alpha_x86_64");
    emitter.instruction("sub al, 0x30");                                        // digit value 0-9
    emitter.instruction("movzx edi, al");                                       // digit value → rdi
    emitter.instruction("jmp __rt_intval_base_digit_have_x86_64");
    emitter.label("__rt_intval_base_digit_alpha_x86_64");
    emitter.instruction("cmp al, 0x61");                                        // 'a'?
    emitter.instruction("jl __rt_intval_base_digit_alpha_upper_x86_64");
    emitter.instruction("cmp al, 0x7A");                                        // 'z'?
    emitter.instruction("jg __rt_intval_base_digits_done_x86_64");              // not a digit at all: stop
    emitter.instruction("sub al, 0x61");                                        // normalize 'a'-'z'
    emitter.instruction("add al, 10");                                          // digit value 10-35
    emitter.instruction("movzx edi, al");                                       // digit value → rdi
    emitter.instruction("jmp __rt_intval_base_digit_have_x86_64");
    emitter.label("__rt_intval_base_digit_alpha_upper_x86_64");
    emitter.instruction("cmp al, 0x41");                                        // 'A'?
    emitter.instruction("jl __rt_intval_base_digits_done_x86_64");              // not a digit at all: stop
    emitter.instruction("cmp al, 0x5A");                                        // 'Z'?
    emitter.instruction("jg __rt_intval_base_digits_done_x86_64");              // not a digit at all: stop
    emitter.instruction("sub al, 0x41");                                        // normalize 'A'-'Z'
    emitter.instruction("add al, 10");                                          // digit value 10-35
    emitter.instruction("movzx edi, al");                                       // digit value → rdi
    emitter.label("__rt_intval_base_digit_have_x86_64");
    emitter.instruction("cmp rdi, r10");                                        // is the digit valid for the effective base?
    emitter.instruction("jae __rt_intval_base_digits_done_x86_64");             // digit >= base: not valid here, stop

    emitter.instruction("mov rax, r11");                                        // acc → multiplicand
    emitter.instruction("mul r10");                                             // rdx:rax = acc * effective base
    emitter.instruction("test rdx, rdx");                                       // did the multiply alone overflow 64 bits?
    emitter.instruction("jnz __rt_intval_base_overflow_x86_64");
    emitter.instruction("mov rdx, rsi");                                        // ceiling → rdx (safe: the multiply-overflow test already consumed the old rdx)
    emitter.instruction("sub rdx, rax");                                        // ceiling - acc*base (unsigned)
    emitter.instruction("jb __rt_intval_base_overflow_x86_64");                 // acc*base already exceeds the ceiling
    emitter.instruction("cmp rdi, rdx");                                        // does this digit push past the remaining headroom?
    emitter.instruction("ja __rt_intval_base_overflow_x86_64");
    emitter.instruction("add rax, rdi");                                        // acc*base + digit
    emitter.instruction("mov r11, rax");                                        // acc = acc*base + digit
    emitter.instruction("mov QWORD PTR [rbp - 16], 1");                         // record that a digit was consumed
    emitter.instruction("inc rcx");                                             // advance the cursor
    emitter.instruction("jmp __rt_intval_base_digit_loop_x86_64");

    emitter.label("__rt_intval_base_overflow_x86_64");
    emitter.instruction("mov r11, rsi");                                        // saturate the accumulator to the sign-dependent ceiling
    emitter.instruction("mov QWORD PTR [rbp - 16], 1");                         // an overflowing run still counts as digits consumed
    emitter.instruction("jmp __rt_intval_base_digits_done_x86_64");

    emitter.label("__rt_intval_base_digits_done_x86_64");
    emitter.instruction("cmp QWORD PTR [rbp - 16], 0");                         // any valid digits consumed after the prefix?
    emitter.instruction("je __rt_intval_base_no_digits_x86_64");
    emitter.instruction("cmp QWORD PTR [rbp - 8], 0");                          // branch on the recorded sign
    emitter.instruction("je __rt_intval_base_result_pos_x86_64");
    emitter.instruction("mov rax, r11");                                        // move the magnitude into the result register
    emitter.instruction("neg rax");                                             // negate (wraps 0x8000...0000 to exactly PHP_INT_MIN)
    emitter.instruction("add rsp, 16");                                         // release the sign/digit-seen spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");
    emitter.label("__rt_intval_base_result_pos_x86_64");
    emitter.instruction("mov rax, r11");                                        // the positive magnitude is the final value
    emitter.instruction("add rsp, 16");                                         // release the sign/digit-seen spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");

    emitter.label("__rt_intval_base_no_digits_x86_64");
    emitter.instruction("xor eax, eax");                                        // no digits parsed at all: PHP intval() returns 0
    emitter.instruction("add rsp, 16");                                         // release the sign/digit-seen spill slots
    emitter.instruction("pop rbp");                                             // restore the caller frame pointer
    emitter.instruction("ret");
}
