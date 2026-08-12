//! Purpose:
//! Implements exact BCMath addition, subtraction, multiplication, division, modulo, comparison, and divmod.
//!
//! Called from:
//! - Public Rust operations used by Magician.
//! - C ABI wrappers used by AOT-generated programs.
//!
//! Key details:
//! - Arithmetic is performed on base-10 coefficients, so requested scale always truncates.
//! - Division truncates toward zero and remainders retain the dividend's sign.

use std::cmp::Ordering;

use crate::error::BcError;
use crate::format::format_bcmath_number;
use crate::num::{
    add_digits, append_zeros, cmp_digits, div_rem_digits, is_zero_digits, mul_digits,
    normalize_digits, sub_digits, BcNum,
};
use crate::parse::parse_bcmath_number_for;
use crate::scale::resolve_scale;

/// Adds two BCMath numeric strings at an explicit or process-default scale.
pub fn bc_add(left: &str, right: &str, scale: Option<i64>) -> Result<String, BcError> {
    binary_add_sub("bcadd", left, right, scale, false)
}

/// Subtracts two BCMath numeric strings at an explicit or process-default scale.
pub fn bc_sub(left: &str, right: &str, scale: Option<i64>) -> Result<String, BcError> {
    binary_add_sub("bcsub", left, right, scale, true)
}

/// Multiplies two BCMath numeric strings at an explicit or process-default scale.
pub fn bc_mul(left: &str, right: &str, scale: Option<i64>) -> Result<String, BcError> {
    let result_scale = resolve_scale(scale, "bcmul", 3)?;
    let (left, right) = parse_binary("bcmul", left, right)?;
    let product_scale = left
        .scale
        .checked_add(right.scale)
        .ok_or(BcError::ScaleRange {
            func: "bcmul",
            arg_pos: 3,
        })?;
    let product = BcNum::new(
        left.negative ^ right.negative,
        mul_digits(&left.digits, &right.digits),
        product_scale,
    );
    format_bcmath_number(&product, result_scale)
}

/// Divides two BCMath numeric strings, truncating toward zero at the requested scale.
pub fn bc_div(left: &str, right: &str, scale: Option<i64>) -> Result<String, BcError> {
    let result_scale = resolve_scale(scale, "bcdiv", 3)?;
    let (left, right) = parse_binary("bcdiv", left, right)?;
    if right.is_zero() {
        return Err(BcError::DivisionByZero { func: "bcdiv" });
    }
    format_bcmath_number(&divide_numbers(&left, &right, result_scale), result_scale)
}

/// Computes a truncated BCMath remainder with the dividend's sign.
pub fn bc_mod(left: &str, right: &str, scale: Option<i64>) -> Result<String, BcError> {
    let result_scale = resolve_scale(scale, "bcmod", 3)?;
    let (left, right) = parse_binary("bcmod", left, right)?;
    let (_, remainder) = divmod_numbers(&left, &right, "bcmod")?;
    format_bcmath_number(&remainder, result_scale)
}

/// Computes BCMath's integer quotient and scaled remainder in one operation.
pub fn bc_divmod(
    left: &str,
    right: &str,
    scale: Option<i64>,
) -> Result<(String, String), BcError> {
    let result_scale = resolve_scale(scale, "bcdivmod", 3)?;
    let (left, right) = parse_binary("bcdivmod", left, right)?;
    let (quotient, remainder) = divmod_numbers(&left, &right, "bcdivmod")?;
    Ok((
        format_bcmath_number(&quotient, 0)?,
        format_bcmath_number(&remainder, result_scale)?,
    ))
}

/// Compares two BCMath numeric strings after truncating both to the requested scale.
pub fn bc_comp(left: &str, right: &str, scale: Option<i64>) -> Result<i32, BcError> {
    let result_scale = resolve_scale(scale, "bccomp", 3)?;
    let (left, right) = parse_binary("bccomp", left, right)?;
    let left_digits = coefficient_at_scale(&left, result_scale);
    let right_digits = coefficient_at_scale(&right, result_scale);
    let left_negative = left.negative && !is_zero_digits(&left_digits);
    let right_negative = right.negative && !is_zero_digits(&right_digits);
    let ordering = match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => cmp_digits(&left_digits, &right_digits),
        (true, true) => cmp_digits(&right_digits, &left_digits),
    };
    Ok(match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

/// Divides two already-parsed values at a fixed non-negative result scale.
pub(crate) fn divide_numbers(left: &BcNum, right: &BcNum, result_scale: i32) -> BcNum {
    debug_assert!(!right.is_zero());
    let power = i64::from(right.scale) - i64::from(left.scale) + i64::from(result_scale);
    let (numerator, denominator) = if power >= 0 {
        (
            append_zeros(left.digits.clone(), power as usize),
            right.digits.clone(),
        )
    } else {
        (
            left.digits.clone(),
            append_zeros(right.digits.clone(), (-power) as usize),
        )
    };
    let (quotient, _) = div_rem_digits(&numerator, &denominator);
    BcNum::new(left.negative ^ right.negative, quotient, result_scale)
}

/// Returns the absolute integer quotient and exact signed remainder of two parsed values.
pub(crate) fn divmod_numbers(
    left: &BcNum,
    right: &BcNum,
    func: &'static str,
) -> Result<(BcNum, BcNum), BcError> {
    if right.is_zero() {
        return Err(BcError::DivisionByZero { func });
    }
    let quotient_numerator = append_zeros(left.digits.clone(), right.scale as usize);
    let quotient_denominator = append_zeros(right.digits.clone(), left.scale as usize);
    let (quotient_digits, _) = div_rem_digits(&quotient_numerator, &quotient_denominator);

    let common_scale = left.scale.max(right.scale);
    let left_at_common = coefficient_at_scale(left, common_scale);
    let right_at_common = coefficient_at_scale(right, common_scale);
    let consumed = mul_digits(&quotient_digits, &right_at_common);
    debug_assert!(cmp_digits(&left_at_common, &consumed) != Ordering::Less);
    let remainder_digits = sub_digits(&left_at_common, &consumed);
    Ok((
        BcNum::new(
            left.negative ^ right.negative,
            quotient_digits,
            0,
        ),
        BcNum::new(left.negative, remainder_digits, common_scale),
    ))
}

/// Produces a coefficient whose integer value represents `number` at `target_scale`.
pub(crate) fn coefficient_at_scale(number: &BcNum, target_scale: i32) -> Vec<u8> {
    if number.scale < target_scale {
        append_zeros(
            number.digits.clone(),
            (target_scale - number.scale) as usize,
        )
    } else if number.scale > target_scale {
        let drop_count = (number.scale - target_scale) as usize;
        if drop_count >= number.digits.len() {
            vec![0]
        } else {
            normalize_digits(number.digits[..number.digits.len() - drop_count].to_vec())
        }
    } else {
        number.digits.clone()
    }
}

/// Parses both operands with PHP's standard binary BCMath argument names.
fn parse_binary(
    func: &'static str,
    left: &str,
    right: &str,
) -> Result<(BcNum, BcNum), BcError> {
    Ok((
        parse_bcmath_number_for(left, func, 1, "num1")?,
        parse_bcmath_number_for(right, func, 2, "num2")?,
    ))
}

/// Implements signed addition and subtraction over one shared aligned scale.
fn binary_add_sub(
    func: &'static str,
    left: &str,
    right: &str,
    scale: Option<i64>,
    subtract: bool,
) -> Result<String, BcError> {
    let result_scale = resolve_scale(scale, func, 3)?;
    let (left, mut right) = parse_binary(func, left, right)?;
    if subtract && !right.is_zero() {
        right.negative = !right.negative;
    }
    let common_scale = left.scale.max(right.scale);
    let left_digits = coefficient_at_scale(&left, common_scale);
    let right_digits = coefficient_at_scale(&right, common_scale);
    let (negative, digits) = if left.negative == right.negative {
        (left.negative, add_digits(&left_digits, &right_digits))
    } else {
        match cmp_digits(&left_digits, &right_digits) {
            Ordering::Less => (right.negative, sub_digits(&right_digits, &left_digits)),
            Ordering::Equal => (false, vec![0]),
            Ordering::Greater => (left.negative, sub_digits(&left_digits, &right_digits)),
        }
    };
    format_bcmath_number(&BcNum::new(negative, digits, common_scale), result_scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{get_scale, set_scale};

    /// Verifies addition truncates or pads exactly to the selected scale.
    #[test]
    fn add_truncates_to_scale_and_pads() {
        assert_eq!(bc_add("1.234", "5", Some(4)).expect("add"), "6.2340");
        assert_eq!(bc_add("1.234", "5", Some(0)).expect("add"), "6");
    }

    /// Verifies division truncates rather than rounding the next digit.
    #[test]
    fn div_truncates_does_not_round() {
        assert_eq!(bc_div("10", "3", Some(2)).expect("divide"), "3.33");
    }

    /// Verifies a zero divisor returns the dedicated typed failure.
    #[test]
    fn div_by_zero_is_div_zero_error() {
        assert!(matches!(
            bc_div("1", "0", Some(0)),
            Err(BcError::DivisionByZero { .. })
        ));
    }

    /// Verifies global scale is consumed only when an explicit scale is absent.
    #[test]
    fn omitted_scale_reads_process_state() {
        let saved = get_scale();
        set_scale(4).expect("set scale");
        assert_eq!(bc_add("1", "1", None).expect("add"), "2.0000");
        assert_eq!(bc_add("1", "1", Some(0)).expect("add"), "2");
        set_scale(i64::from(saved)).expect("restore scale");
    }

    /// Verifies modulo and divmod retain PHP's quotient and dividend-sign rules.
    #[test]
    fn divmod_signs_match_php() {
        for (left, right, expected) in [
            ("5", "3", ("1", "2")),
            ("5", "-3", ("-1", "2")),
            ("-5", "3", ("-1", "-2")),
            ("-5", "-3", ("1", "-2")),
        ] {
            let actual = bc_divmod(left, right, Some(0)).expect("divmod");
            assert_eq!(actual, (expected.0.to_string(), expected.1.to_string()));
        }
        assert_eq!(bc_mod("5.7", "1.3", Some(1)).expect("mod"), "0.5");
    }
}

