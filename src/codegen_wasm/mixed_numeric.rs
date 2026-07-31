//! Purpose:
//! Emits `__rt_mixed_numeric_add/sub/mul`, PHP arithmetic over boxed Mixed operands.
//! Carries PHP's integer-overflow promotion and its numeric-string classification.
//!
//! Called from:
//! - `crate::codegen_wasm::runtime::emit_command_runtime()`, after the mixed and float
//!   runtimes whose `__rt_mixed_unbox`, `__rt_mixed_from_value`, `__rt_str_to_int`, and
//!   `__rt_str_to_f64` helpers it calls.
//!
//! Key details:
//! - Operand classification follows php-src, not the operand's static type: `"7" + 5` is
//!   an integer while `"7.0" + 5` is a double.
//! - A non-numeric operand is a PHP `TypeError`. WebAssembly has no exception machinery
//!   yet, so it is reported as an uncaught fatal and exits 255; catching it needs W2.

use super::wat::WatModule;

/// Byte offsets into the numeric-string classifier's shared scratch region.
///
/// The classifier reports a class plus the parsed value; the value lands here rather
/// than in extra multi-value results so the arithmetic helpers can read whichever of
/// the two representations the class selects.
const CLASS_VALUE_OFFSET: i32 = 10496;

/// Adds the boxed-Mixed arithmetic runtime to `wm`.
pub(super) fn emit_mixed_numeric_runtime(wm: &mut WatModule) {
    wm.add_raw_func(&rt_str_numeric_class());
    wm.add_raw_func(&rt_mixed_numeric_operand());
    wm.add_raw_func(RT_MIXED_NUMERIC_COMMON);
    wm.add_raw_func(RT_MIXED_NUMERIC_ADD);
    wm.add_raw_func(RT_MIXED_NUMERIC_SUB);
    wm.add_raw_func(RT_MIXED_NUMERIC_MUL);
}

/// Classifies a PHP string for arithmetic, following php-src's `is_numeric_string_ex`.
///
/// PHP accepts `WS* [+-]? (DIGITS (. DIGITS?)? | . DIGITS) ([eE][+-]? DIGITS)? WS*`. A
/// string matching it entirely is numeric; one matching only a prefix is "leading
/// numeric", which arithmetic accepts while emitting a warning; anything else is not a
/// number at all and arithmetic rejects it. Integer versus float is decided by form, not
/// by magnitude: a string carrying a decimal point or an exponent is always a float, and
/// so is an integral string too large for `i64` (`"9223372036854775808"`).
///
/// Returns 0 for non-numeric, 1 for an integer, 2 for a float, 3 for a leading-numeric
/// integer, and 4 for a leading-numeric float. Classes 1 and 3 leave the parsed `i64` at
/// the scratch offset; classes 2 and 4 leave raw `f64` bits there.
fn rt_str_numeric_class() -> String {
    format!(
        r#"(func $__rt_str_numeric_class (param $ptr i32) (param $len i32) (result i32)
  (local $i i32) (local $c i32) (local $digits i32) (local $isfloat i32)
  (local $end i32) (local $numend i32) (local $vlen i32) (local $iv i64)
  (local.set $i (i32.const 0))                                    ;; cursor
  (block $ws                                                      ;; skip PHP's leading whitespace
    (loop $wl
      (br_if $ws (i32.ge_u (local.get $i) (local.get $len)))      ;; end of string
      (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; current byte
      (br_if $ws (i32.eqz (i32.or (i32.or (i32.or
        (i32.eq (local.get $c) (i32.const 32))                    ;; space
        (i32.eq (local.get $c) (i32.const 9)))                    ;; tab
        (i32.or (i32.eq (local.get $c) (i32.const 10))            ;; newline
                (i32.eq (local.get $c) (i32.const 13))))          ;; carriage return
        (i32.or (i32.eq (local.get $c) (i32.const 11))            ;; vertical tab
                (i32.eq (local.get $c) (i32.const 12))))))        ;; form feed
      (local.set $i (i32.add (local.get $i) (i32.const 1)))       ;; consume whitespace
      (br $wl)))
  (if (i32.lt_u (local.get $i) (local.get $len))                  ;; optional sign
    (then
      (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; sign candidate
      (if (i32.or (i32.eq (local.get $c) (i32.const 43))          ;; '+'
                  (i32.eq (local.get $c) (i32.const 45)))         ;; '-'
        (then (local.set $i (i32.add (local.get $i) (i32.const 1)))))))       ;; consume sign
  (local.set $digits (i32.const 0))                               ;; integral digit count
  (block $id                                                      ;; integral digits
    (loop $il
      (br_if $id (i32.ge_u (local.get $i) (local.get $len)))      ;; end of string
      (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; current byte
      (br_if $id (i32.or (i32.lt_u (local.get $c) (i32.const 48)) ;; not a digit
                         (i32.gt_u (local.get $c) (i32.const 57))))
      (local.set $digits (i32.add (local.get $digits) (i32.const 1)))         ;; count it
      (local.set $i (i32.add (local.get $i) (i32.const 1)))       ;; consume digit
      (br $il)))
  (local.set $isfloat (i32.const 0))                              ;; integer until proven otherwise
  (if (i32.lt_u (local.get $i) (local.get $len))                  ;; optional fraction
    (then
      (if (i32.eq (i32.load8_u (i32.add (local.get $ptr) (local.get $i))) (i32.const 46))  ;; '.'
        (then
          (local.set $isfloat (i32.const 1))                      ;; a decimal point forces float
          (local.set $i (i32.add (local.get $i) (i32.const 1)))   ;; consume '.'
          (block $fd                                              ;; fractional digits
            (loop $fl
              (br_if $fd (i32.ge_u (local.get $i) (local.get $len)))          ;; end of string
              (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; current byte
              (br_if $fd (i32.or (i32.lt_u (local.get $c) (i32.const 48))     ;; not a digit
                                 (i32.gt_u (local.get $c) (i32.const 57))))
              (local.set $digits (i32.add (local.get $digits) (i32.const 1))) ;; count it
              (local.set $i (i32.add (local.get $i) (i32.const 1)))           ;; consume digit
              (br $fl)))))))
  (if (i32.eqz (local.get $digits))                               ;; no mantissa digit at all
    (then (return (i32.const 0))))                                ;; not a number
  (local.set $numend (local.get $i))                              ;; end of the mantissa
  (if (i32.lt_u (local.get $i) (local.get $len))                  ;; optional exponent
    (then
      (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; 'e' candidate
      (if (i32.or (i32.eq (local.get $c) (i32.const 101))         ;; 'e'
                  (i32.eq (local.get $c) (i32.const 69)))         ;; 'E'
        (then
          (local.set $end (i32.add (local.get $i) (i32.const 1))) ;; provisional cursor past 'e'
          (if (i32.lt_u (local.get $end) (local.get $len))        ;; optional exponent sign
            (then
              (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $end))))  ;; sign candidate
              (if (i32.or (i32.eq (local.get $c) (i32.const 43))  ;; '+'
                          (i32.eq (local.get $c) (i32.const 45))) ;; '-'
                (then (local.set $end (i32.add (local.get $end) (i32.const 1)))))))     ;; consume sign
          (local.set $c (i32.const 0))                            ;; exponent digit count
          (block $ed                                              ;; exponent digits
            (loop $el
              (br_if $ed (i32.ge_u (local.get $end) (local.get $len)))        ;; end of string
              (br_if $ed (i32.or
                (i32.lt_u (i32.load8_u (i32.add (local.get $ptr) (local.get $end))) (i32.const 48))
                (i32.gt_u (i32.load8_u (i32.add (local.get $ptr) (local.get $end))) (i32.const 57))))  ;; not a digit
              (local.set $c (i32.add (local.get $c) (i32.const 1)))           ;; count it
              (local.set $end (i32.add (local.get $end) (i32.const 1)))       ;; consume digit
              (br $el)))
          ;; an exponent needs at least one digit, otherwise 'e' is trailing garbage
          (if (local.get $c)
            (then
              (local.set $isfloat (i32.const 1))                  ;; an exponent forces float
              (local.set $numend (local.get $end))                ;; mantissa+exponent consumed
              (local.set $i (local.get $end))))))))               ;; advance past the exponent
  (local.set $vlen (local.get $numend))                           ;; bytes the number occupies
  (block $tw                                                      ;; skip PHP's trailing whitespace
    (loop $tl
      (br_if $tw (i32.ge_u (local.get $i) (local.get $len)))      ;; end of string
      (local.set $c (i32.load8_u (i32.add (local.get $ptr) (local.get $i))))  ;; current byte
      (br_if $tw (i32.eqz (i32.or (i32.or (i32.or
        (i32.eq (local.get $c) (i32.const 32))                    ;; space
        (i32.eq (local.get $c) (i32.const 9)))                    ;; tab
        (i32.or (i32.eq (local.get $c) (i32.const 10))            ;; newline
                (i32.eq (local.get $c) (i32.const 13))))          ;; carriage return
        (i32.or (i32.eq (local.get $c) (i32.const 11))            ;; vertical tab
                (i32.eq (local.get $c) (i32.const 12))))))        ;; form feed
      (local.set $i (i32.add (local.get $i) (i32.const 1)))       ;; consume whitespace
      (br $tl)))
  (if (i32.eqz (local.get $isfloat))                              ;; integral form: does it fit i64?
    (then
      (local.set $iv (call $__rt_str_to_int (local.get $ptr) (local.get $vlen) (global.get $__float_scratch)))  ;; parse the integral text
      (i64.store (i32.add (global.get $__float_scratch) (i32.const {value_offset})) (local.get $iv))            ;; publish the i64
      (call $__rt_str_to_f64 (local.get $ptr) (local.get $vlen) (i32.add (global.get $__float_scratch) (i32.const 10240)) (global.get $__float_scratch))  ;; same text as f64
      (if (f64.ne
            (f64.convert_i64_s (local.get $iv))
            (f64.reinterpret_i64 (i64.load (i32.add (global.get $__float_scratch) (i32.const 10240)))))
        (then (local.set $isfloat (i32.const 1))))))              ;; magnitude exceeds i64: PHP calls it a float
  (if (local.get $isfloat)                                        ;; float form: publish raw f64 bits
    (then
      (call $__rt_str_to_f64 (local.get $ptr) (local.get $vlen) (i32.add (global.get $__float_scratch) (i32.const {value_offset})) (global.get $__float_scratch))))  ;; parse into the value slot
  (if (i32.eq (local.get $i) (local.get $len))                    ;; the whole string was consumed
    (then (return (i32.add (i32.const 1) (local.get $isfloat))))) ;; 1 = int, 2 = float
  (i32.add (i32.const 3) (local.get $isfloat)))                   ;; 3 = leading int, 4 = leading float
"#,
        value_offset = CLASS_VALUE_OFFSET
    )
}

/// Builds `__rt_mixed_numeric_operand`, which reduces one boxed operand to a number.
///
/// Returns `(is_float, value)` where `value` carries either the `i64` or raw `f64` bits.
/// Integers and booleans are integers, floats are floats, and null is integer zero —
/// matching PHP, where `null + 1` is `1`. A string is classified: a leading-numeric one
/// contributes its numeric prefix after warning, and a wholly non-numeric one is a
/// `TypeError` that exits 255 because catching it needs exception support this target
/// does not have. Arrays, objects, and resources never reach here: the capability audit
/// rejects them before emission.
fn rt_mixed_numeric_operand() -> String {
    format!(
        r#"(func $__rt_mixed_numeric_operand (param $ptr i32) (result i32) (result i64)
  (local $tag i64) (local $lo i64) (local $hi i64) (local $class i32)
  (call $__rt_mixed_unbox (local.get $ptr))                       ;; unbox -> stack: tag, lo, hi
  (local.set $hi)                                                 ;; pop value high word
  (local.set $lo)                                                 ;; pop value low word
  (local.set $tag)                                                ;; pop runtime tag
  (if (i64.eqz (local.get $tag))                                  ;; tag 0 = int
    (then (return (i32.const 0) (local.get $lo))))                ;; integer operand
  (if (i64.eq (local.get $tag) (i64.const 3))                     ;; tag 3 = bool
    (then (return (i32.const 0) (local.get $lo))))                ;; PHP treats a bool as 0/1
  (if (i64.eq (local.get $tag) (i64.const 8))                     ;; tag 8 = null
    (then (return (i32.const 0) (i64.const 0))))                  ;; PHP treats null as 0
  (if (i64.eq (local.get $tag) (i64.const 2))                     ;; tag 2 = float
    (then (return (i32.const 1) (local.get $lo))))                ;; forward the stored f64 bits
  (if (i64.eq (local.get $tag) (i64.const 1))                     ;; tag 1 = string
    (then
      (local.set $class (call $__rt_str_numeric_class (i32.wrap_i64 (local.get $lo)) (i32.wrap_i64 (local.get $hi))))  ;; classify the text
      (if (i32.eqz (local.get $class))                            ;; no numeric prefix at all
        (then (call $__rt_fatal_unsupported_operand)))            ;; PHP TypeError, uncatchable here
      (if (i32.gt_u (local.get $class) (i32.const 2))             ;; leading numeric: value plus a warning
        (then (call $__rt_warn_non_numeric_value)))               ;; "A non-numeric value encountered"
      (return
        (i32.eqz (i32.and (local.get $class) (i32.const 1)))      ;; even classes (2 and 4) are the floats
        (i64.load (i32.add (global.get $__float_scratch) (i32.const {value_offset}))))))  ;; parsed value
  (call $__rt_fatal_unsupported_operand)                          ;; any other tag is not a number
  (i32.const 0) (i64.const 0))                                    ;; unreachable, keeps the signature
"#,
        value_offset = CLASS_VALUE_OFFSET
    )
}

/// Shared body of the three arithmetic helpers, selected by `$op` (0 add, 1 sub, 2 mul).
///
/// Both operands are reduced first, so a `TypeError` or warning fires in PHP's own
/// left-to-right order. When both are integers the operation runs in `i64` and its
/// overflow is detected exactly, promoting to a double the way php-src does; otherwise
/// both widen to `f64`. The result is boxed so the caller observes either tag.
const RT_MIXED_NUMERIC_COMMON: &str =
    r#"(func $__rt_mixed_numeric_common (param $l i32) (param $r i32) (param $op i32) (result i32)
  (local $lf i32) (local $lv i64) (local $rf i32) (local $rv i64)
  (local $res i64) (local $ovf i32) (local $x f64) (local $y f64)
  (call $__rt_mixed_numeric_operand (local.get $l))               ;; reduce the left operand first
  (local.set $lv)                                                 ;; pop its value
  (local.set $lf)                                                 ;; pop its float flag
  (call $__rt_mixed_numeric_operand (local.get $r))               ;; then the right operand
  (local.set $rv)                                                 ;; pop its value
  (local.set $rf)                                                 ;; pop its float flag
  (if (i32.eqz (i32.or (local.get $lf) (local.get $rf)))          ;; both integers: try i64 arithmetic
    (then
      (if (i32.eqz (local.get $op))                               ;; add
        (then
          (local.set $res (i64.add (local.get $lv) (local.get $rv)))                    ;; wrapped sum
          (local.set $ovf (i64.lt_s
            (i64.and (i64.xor (local.get $lv) (local.get $res))
                     (i64.xor (local.get $rv) (local.get $res)))
            (i64.const 0)))))                                     ;; signed-add overflow
      (if (i32.eq (local.get $op) (i32.const 1))                  ;; sub
        (then
          (local.set $res (i64.sub (local.get $lv) (local.get $rv)))                    ;; wrapped difference
          (local.set $ovf (i64.lt_s
            (i64.and (i64.xor (local.get $lv) (local.get $rv))
                     (i64.xor (local.get $lv) (local.get $res)))
            (i64.const 0)))))                                     ;; signed-sub overflow
      (if (i32.eq (local.get $op) (i32.const 2))                  ;; mul
        (then
          (local.set $res (i64.mul (local.get $lv) (local.get $rv)))                    ;; wrapped product
          (local.set $ovf (i32.const 0))                          ;; assume it fits
          (if (i64.ne (local.get $lv) (i64.const 0))              ;; a zero factor never overflows
            (then
              (if (i32.or
                    (i64.ne (i64.div_s (local.get $res) (local.get $lv)) (local.get $rv))  ;; division disagrees
                    (i32.and (i64.eq (local.get $lv) (i64.const -1))
                             (i64.eq (local.get $rv) (i64.const -9223372036854775808))))   ;; -1 * INT_MIN
                (then (local.set $ovf (i32.const 1))))))))        ;; product does not fit i64
      (if (i32.eqz (local.get $ovf))                              ;; it fits: an integer result
        (then (return (call $__rt_mixed_from_value (i64.const 0) (local.get $res) (i64.const 0)))))))
  (local.set $x (if (result f64) (local.get $lf)                  ;; widen the left operand
    (then (f64.reinterpret_i64 (local.get $lv)))
    (else (f64.convert_i64_s (local.get $lv)))))
  (local.set $y (if (result f64) (local.get $rf)                  ;; widen the right operand
    (then (f64.reinterpret_i64 (local.get $rv)))
    (else (f64.convert_i64_s (local.get $rv)))))
  (if (i32.eqz (local.get $op))                                   ;; add
    (then (local.set $x (f64.add (local.get $x) (local.get $y)))))
  (if (i32.eq (local.get $op) (i32.const 1))                      ;; sub
    (then (local.set $x (f64.sub (local.get $x) (local.get $y)))))
  (if (i32.eq (local.get $op) (i32.const 2))                      ;; mul
    (then (local.set $x (f64.mul (local.get $x) (local.get $y)))))
  (call $__rt_mixed_from_value (i64.const 2) (i64.reinterpret_f64 (local.get $x)) (i64.const 0)))
"#;

/// PHP `+` over boxed Mixed operands.
const RT_MIXED_NUMERIC_ADD: &str =
    r#"(func $__rt_mixed_numeric_add (param $l i32) (param $r i32) (result i32)
  (call $__rt_mixed_numeric_common (local.get $l) (local.get $r) (i32.const 0)))  ;; op 0 = add
"#;

/// PHP `-` over boxed Mixed operands.
const RT_MIXED_NUMERIC_SUB: &str =
    r#"(func $__rt_mixed_numeric_sub (param $l i32) (param $r i32) (result i32)
  (call $__rt_mixed_numeric_common (local.get $l) (local.get $r) (i32.const 1)))  ;; op 1 = sub
"#;

/// PHP `*` over boxed Mixed operands.
const RT_MIXED_NUMERIC_MUL: &str =
    r#"(func $__rt_mixed_numeric_mul (param $l i32) (param $r i32) (result i32)
  (call $__rt_mixed_numeric_common (local.get $l) (local.get $r) (i32.const 2)))  ;; op 2 = mul
"#;
