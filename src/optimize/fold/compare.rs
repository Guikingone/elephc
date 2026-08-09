//! Purpose:
//! Reimplements PHP 8's `zend_compare()` over compile-time scalar literals.
//! Owns numeric-string recognition, integer-exact ordering, and the loose (`==`) result
//! used by binary-operator folding, `switch` case selection, and array-key coercion.
//!
//! Called from:
//! - `crate::optimize::fold::scalar`
//! - `crate::optimize::fold::ops`
//! - `crate::optimize::fold::casts`
//! - `crate::optimize::control::switch`
//!
//! Key details:
//! - Integers are compared as `i64`, never through `f64`: `PHP_INT_MAX - 1 < PHP_INT_MAX`
//!   is observable and a `f64` round-trip loses it.
//! - PHP 8 compares an int/float against a *non-numeric* string by stringifying the number,
//!   so `0 == "abc"` is `false`. Float stringification is precision-dependent, so that one
//!   pair declines the fold instead of guessing.
//! - `ZEND_THREEWAY_COMPARE` answers `1` for any NAN pair; `<`/`<=`/`>`/`>=` are therefore
//!   spelled through `zend_compare` argument order (PHP implements `a > b` as `b < a`).

use std::cmp::Ordering;

use super::scalar::ScalarValue;

/// PHP's `is_numeric_string()` classification of a numeric string.
///
/// `Int` mirrors `IS_LONG`; `Float` mirrors `IS_DOUBLE`, carrying the `oflow` flag PHP
/// sets when the text was a plain integer too large (`1`) or too small (`-1`) for `i64`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::optimize) enum PhpNumeric {
    Int(i64),
    Float { value: f64, integer_overflow: i8 },
}

impl PhpNumeric {
    /// Returns the value as an `f64`, matching PHP's `dval` slot for either classification.
    fn as_f64(self) -> f64 {
        match self {
            PhpNumeric::Int(value) => value as f64,
            PhpNumeric::Float { value, .. } => value,
        }
    }

    /// Returns PHP's `oflow` marker: non-zero only for integer text that exceeded `i64`.
    fn integer_overflow(self) -> i8 {
        match self {
            PhpNumeric::Int(_) => 0,
            PhpNumeric::Float {
                integer_overflow, ..
            } => integer_overflow,
        }
    }
}

/// Returns whether a byte is one of the six characters PHP's `ZEND_IS_WHITESPACE` accepts.
fn is_php_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// The longest leading numeric run of a string, as scanned by PHP's `_is_numeric_string_ex`.
struct NumericScan<'a> {
    /// The matched numeric text with leading whitespace and trailing garbage removed.
    text: &'a str,
    /// Whether the run contained a decimal point or a consumed exponent (`IS_DOUBLE` syntax).
    is_float: bool,
    /// Everything after the matched run, still un-trimmed.
    trailing: &'a str,
}

/// Scans the longest leading numeric run of `value` using PHP's numeric-string grammar.
///
/// Accepts optional leading whitespace, an optional sign, a mantissa with at least one
/// digit (`12`, `.5`, `5.`), and an exponent only when at least one digit follows it —
/// `"1e"` therefore scans as `1` with `"e"` left over, exactly like PHP. Hex, underscore
/// separators, `INF` and `NAN` are not part of the grammar. Returns `None` when no digit
/// is present at all.
fn scan_numeric_prefix(value: &str) -> Option<NumericScan<'_>> {
    let bytes = value.as_bytes();
    let mut idx = 0;
    while idx < bytes.len() && is_php_whitespace(bytes[idx]) {
        idx += 1;
    }

    let start = idx;
    if idx < bytes.len() && matches!(bytes[idx], b'+' | b'-') {
        idx += 1;
    }

    let mut digits = 0;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        idx += 1;
        digits += 1;
    }

    let mut is_float = false;
    if idx < bytes.len() && bytes[idx] == b'.' {
        let mut probe = idx + 1;
        while probe < bytes.len() && bytes[probe].is_ascii_digit() {
            probe += 1;
            digits += 1;
        }
        if digits > 0 {
            idx = probe;
            is_float = true;
        }
    }
    if digits == 0 {
        return None;
    }

    if idx < bytes.len() && matches!(bytes[idx], b'e' | b'E') {
        let mut probe = idx + 1;
        if probe < bytes.len() && matches!(bytes[probe], b'+' | b'-') {
            probe += 1;
        }
        let exponent_start = probe;
        while probe < bytes.len() && bytes[probe].is_ascii_digit() {
            probe += 1;
        }
        if probe > exponent_start {
            idx = probe;
            is_float = true;
        }
    }

    Some(NumericScan {
        text: &value[start..idx],
        is_float,
        trailing: &value[idx..],
    })
}

/// Classifies a scanned numeric run into PHP's `IS_LONG` / `IS_DOUBLE` result.
///
/// Integer text that does not fit `i64` becomes `Float` with the `oflow` sign PHP records,
/// which `zendi_smart_strcmp` needs to fall back to a byte comparison.
fn classify_numeric_scan(scan: &NumericScan<'_>) -> Option<PhpNumeric> {
    if scan.is_float {
        // Rust's `f64` parser accepts the exact grammar scanned above and, like
        // `zend_strtod`, saturates to infinity on exponent overflow.
        return scan.text.parse::<f64>().ok().map(|value| PhpNumeric::Float {
            value,
            integer_overflow: 0,
        });
    }
    match scan.text.parse::<i64>() {
        Ok(value) => Some(PhpNumeric::Int(value)),
        Err(_) => scan.text.parse::<f64>().ok().map(|value| PhpNumeric::Float {
            value,
            integer_overflow: if scan.text.starts_with('-') { -1 } else { 1 },
        }),
    }
}

/// Recognizes a *fully* numeric string, PHP's `is_numeric_string(..., allow_errors = 0)`.
///
/// Leading and trailing whitespace are permitted (PHP 8 accepts `"1 "`), any other trailing
/// byte makes the string non-numeric. Returns `None` for `""`, `"abc"`, `"1abc"`, `"0x1A"`,
/// `"1_000"`, `"INF"`, and `"NAN"`.
pub(in crate::optimize) fn php_numeric_string(value: &str) -> Option<PhpNumeric> {
    let scan = scan_numeric_prefix(value)?;
    if !scan.trailing.bytes().all(is_php_whitespace) {
        return None;
    }
    classify_numeric_scan(&scan)
}

/// Recognizes a *leading* numeric run, PHP's `is_numeric_string(..., allow_errors = 1)`.
///
/// This is the form the `(int)` / `(float)` string casts use, so `"12abc"` yields `12` and
/// `"1.2.3"` yields `1.2`. Returns `None` when the string has no numeric prefix, which the
/// casts turn into `0` / `0.0`.
pub(in crate::optimize) fn php_numeric_prefix(value: &str) -> Option<PhpNumeric> {
    let scan = scan_numeric_prefix(value)?;
    classify_numeric_scan(&scan)
}

/// PHP's `ZEND_THREEWAY_COMPARE`: equal, less, otherwise greater.
///
/// Any NAN operand falls into the `Greater` arm, which is what makes `NAN < 1`, `NAN > 1`,
/// `NAN <= 1` and `NAN >= 1` all evaluate to `false` once the caller spells the operator
/// through `zend_compare` argument order.
fn three_way(left: f64, right: f64) -> Ordering {
    if left == right {
        Ordering::Equal
    } else if left < right {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// PHP's `zend_binary_strcmp`: byte-wise `memcmp` with the shorter string ordered first on a
/// common prefix.
fn compare_bytes(left: &str, right: &str) -> Ordering {
    left.as_bytes().cmp(right.as_bytes())
}

/// PHP's `zendi_smart_strcmp`: two numeric strings compare numerically, anything else
/// compares byte-wise.
///
/// Two integer strings compare exactly as `i64`, so `"9223372036854775806"` is genuinely
/// smaller than `"9223372036854775807"`. Values that both overflowed `i64` in the same
/// direction, or that are both infinite, fall back to the byte comparison exactly as PHP
/// does to avoid the accuracy loss of the `f64` round-trip.
fn compare_strings(left: &str, right: &str) -> Ordering {
    let (Some(left_num), Some(right_num)) = (php_numeric_string(left), php_numeric_string(right))
    else {
        return compare_bytes(left, right);
    };

    let (left_overflow, right_overflow) = (left_num.integer_overflow(), right_num.integer_overflow());
    if left_overflow != 0
        && left_overflow == right_overflow
        && left_num.as_f64() - right_num.as_f64() == 0.0
    {
        return compare_bytes(left, right);
    }

    match (left_num, right_num) {
        (PhpNumeric::Int(left_value), PhpNumeric::Int(right_value)) => left_value.cmp(&right_value),
        (PhpNumeric::Int(_), PhpNumeric::Float { .. }) if right_overflow != 0 => {
            // The right operand is an integer beyond `i64`; its sign decides the order.
            if right_overflow > 0 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (PhpNumeric::Float { .. }, PhpNumeric::Int(_)) if left_overflow != 0 => {
            if left_overflow > 0 {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        _ => {
            let (left_value, right_value) = (left_num.as_f64(), right_num.as_f64());
            if left_value == right_value && !left_value.is_finite() {
                return compare_bytes(left, right);
            }
            three_way(left_value - right_value, 0.0)
        }
    }
}

/// PHP's `compare_long_to_string`.
///
/// A numeric string compares against the integer as a number (exactly when the string is
/// itself an integer); a non-numeric string makes PHP 8 stringify the integer and compare
/// bytes, which is why `0 == "abc"` is `false`.
fn compare_int_to_string(left: i64, right: &str) -> Ordering {
    match php_numeric_string(right) {
        Some(PhpNumeric::Int(right)) => left.cmp(&right),
        Some(PhpNumeric::Float { value, .. }) => three_way(left as f64, value),
        None => compare_bytes(&left.to_string(), right),
    }
}

/// PHP's `compare_double_to_string`.
///
/// Returns `None` for a non-numeric string: PHP stringifies the float through
/// `zend_double_to_str`, whose output (`"INF"`, `"1.0E+25"`, …) depends on formatting rules
/// this fold deliberately does not reimplement, so the comparison stays on the runtime path.
fn compare_float_to_string(left: f64, right: &str) -> Option<Ordering> {
    match php_numeric_string(right) {
        Some(PhpNumeric::Int(right)) => Some(three_way(left - right as f64, 0.0)),
        Some(PhpNumeric::Float { value, .. }) => Some(three_way(left, value)),
        None => None,
    }
}

/// PHP 8's `zend_compare()` over two scalar literals.
///
/// Returns `None` only when the result cannot be established at compile time (a float
/// against a non-numeric string). Callers must spell the relational operators the way the
/// engine does — `a > b` is `compare(b, a) == Less` — so NAN keeps PHP's behavior.
pub(in crate::optimize) fn compare_scalars(
    left: &ScalarValue,
    right: &ScalarValue,
) -> Option<Ordering> {
    match (left, right) {
        (ScalarValue::Int(left), ScalarValue::Int(right)) => Some(left.cmp(right)),
        (ScalarValue::Int(left), ScalarValue::Float(right)) => Some(three_way(*left as f64, *right)),
        (ScalarValue::Float(left), ScalarValue::Int(right)) => Some(three_way(*left, *right as f64)),
        (ScalarValue::Float(left), ScalarValue::Float(right)) => Some(three_way(*left, *right)),
        (ScalarValue::String(left), ScalarValue::String(right)) => Some(compare_strings(left, right)),
        // PHP special-cases null against a string: only the empty string compares equal, so
        // `null == "0"` is false even though `"0"` is falsy.
        (ScalarValue::Null, ScalarValue::String(right)) => Some(if right.is_empty() {
            Ordering::Equal
        } else {
            Ordering::Less
        }),
        (ScalarValue::String(left), ScalarValue::Null) => Some(if left.is_empty() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }),
        (ScalarValue::Int(left), ScalarValue::String(right)) => {
            Some(compare_int_to_string(*left, right))
        }
        (ScalarValue::String(left), ScalarValue::Int(right)) => {
            Some(compare_int_to_string(*right, left).reverse())
        }
        (ScalarValue::Float(left), ScalarValue::String(right)) => {
            compare_float_to_string(*left, right)
        }
        (ScalarValue::String(left), ScalarValue::Float(right)) => {
            compare_float_to_string(*right, left).map(Ordering::reverse)
        }
        // Everything else goes through PHP's boolean fallback in `zend_compare`'s default arm.
        (ScalarValue::Null | ScalarValue::Bool(false), right) => Some(if right.truthy() {
            Ordering::Less
        } else {
            Ordering::Equal
        }),
        (ScalarValue::Bool(true), right) => Some(if right.truthy() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }),
        (left, ScalarValue::Null | ScalarValue::Bool(false)) => Some(if left.truthy() {
            Ordering::Greater
        } else {
            Ordering::Equal
        }),
        (left, ScalarValue::Bool(true)) => Some(if left.truthy() {
            Ordering::Equal
        } else {
            Ordering::Less
        }),
    }
}

/// PHP's `==` over two scalar literals, i.e. `zend_compare(...) == 0`.
pub(in crate::optimize) fn loose_eq_values(
    left: &ScalarValue,
    right: &ScalarValue,
) -> Option<bool> {
    compare_scalars(left, right).map(|ordering| ordering == Ordering::Equal)
}
