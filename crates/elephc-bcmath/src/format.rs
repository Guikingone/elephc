//! Purpose:
//! Formats exact BCMath numbers with PHP's requested result scale.
//!
//! Called from:
//! - Arithmetic, power, square-root, and rounding operations after exact computation.
//!
//! Key details:
//! - Excess fractional digits are truncated and missing digits are zero-padded.
//! - Zero never retains a negative sign.

use crate::error::BcError;
use crate::num::{is_zero_digits, normalize_digits, BcNum};

/// Formats a decimal at `result_scale`, truncating rather than rounding.
pub fn format_bcmath_number(number: &BcNum, result_scale: i32) -> Result<String, BcError> {
    if result_scale < 0 {
        return Err(BcError::ScaleRange {
            func: "bcmath",
            arg_pos: 1,
        });
    }
    let target_scale = result_scale as usize;
    let source_scale = number.scale as usize;
    let mut digits = number.digits.clone();
    if source_scale > target_scale {
        let drop_count = source_scale - target_scale;
        if drop_count >= digits.len() {
            digits = vec![0];
        } else {
            digits.truncate(digits.len() - drop_count);
            digits = normalize_digits(digits);
        }
    } else if source_scale < target_scale && !is_zero_digits(&digits) {
        digits.resize(digits.len() + (target_scale - source_scale), 0);
    }

    let is_zero = is_zero_digits(&digits);
    let integer_len = digits.len().saturating_sub(target_scale);
    let mut output = String::with_capacity(
        digits.len() + usize::from(target_scale > 0) + usize::from(number.negative),
    );
    if number.negative && !is_zero {
        output.push('-');
    }
    if integer_len == 0 {
        output.push('0');
    } else {
        push_digits(&mut output, &digits[..integer_len]);
    }
    if target_scale > 0 {
        output.push('.');
        let leading_fraction_zeros = target_scale.saturating_sub(digits.len());
        for _ in 0..leading_fraction_zeros {
            output.push('0');
        }
        let fraction_start = digits.len().saturating_sub(target_scale);
        push_digits(&mut output, &digits[fraction_start..]);
        let written_fraction = leading_fraction_zeros + digits.len() - fraction_start;
        for _ in written_fraction..target_scale {
            output.push('0');
        }
    }
    Ok(output)
}
/// Appends decimal digits to one output string.
fn push_digits(output: &mut String, digits: &[u8]) {
    output.extend(digits.iter().map(|digit| char::from(b'0' + *digit)));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies truncation, padding, and negative-zero normalization.
    #[test]
    fn format_truncates_pads_and_normalizes_zero() {
        assert_eq!(
            format_bcmath_number(&BcNum::new(false, vec![1, 2, 3, 4], 3), 2)
                .expect("format"),
            "1.23"
        );
        assert_eq!(
            format_bcmath_number(&BcNum::new(false, vec![1, 2], 1), 3).expect("format"),
            "1.200"
        );
        assert_eq!(
            format_bcmath_number(&BcNum::new(true, vec![1], 3), 2).expect("format"),
            "0.00"
        );
    }
}
