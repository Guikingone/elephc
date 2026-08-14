//! Purpose:
//! Parses the ASCII decimal grammar accepted by PHP BCMath functions.
//!
//! Called from:
//! - Every arithmetic operation before exact decimal processing.
//! - Crate tests that pin accepted and rejected numeric strings.
//!
//! Key details:
//! - Input is scanned verbatim: whitespace, scientific notation, and other junk are rejected.
//! - PHP normalizes the otherwise-valid digitless forms (`""`, signs, and a point) to zero.

use crate::error::BcError;
use crate::num::BcNum;

/// Parses a BCMath numeric string without operation-specific error context.
pub fn parse_bcmath_number(input: &str) -> Result<BcNum, BcError> {
    parse_bcmath_number_for(input, "bcmath", 1, "num")
}

/// Parses a numeric operand and records its PHP function/argument context on failure.
pub(crate) fn parse_bcmath_number_for(
    input: &str,
    func: &'static str,
    arg_pos: u32,
    arg_name: &'static str,
) -> Result<BcNum, BcError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    let mut negative = false;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        negative = bytes[0] == b'-';
        index += 1;
    }

    let mut digits = Vec::with_capacity(bytes.len());
    let mut saw_dot = false;
    let mut scale = 0i32;
    while index < bytes.len() {
        match bytes[index] {
            b'0'..=b'9' => {
                digits.push(bytes[index] - b'0');
                if saw_dot {
                    scale = scale.checked_add(1).ok_or(BcError::Malformed {
                        func,
                        arg_pos,
                        arg_name,
                    })?;
                }
            }
            b'.' if !saw_dot => saw_dot = true,
            _ => {
                return Err(BcError::Malformed {
                    func,
                    arg_pos,
                    arg_name,
                })
            }
        }
        index += 1;
    }
    Ok(BcNum::new(negative, digits, scale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::format_bcmath_number;

    /// Verifies signed decimals without surrounding whitespace are accepted and normalized.
    #[test]
    fn parse_accepts_signed_decimal() {
        let number = parse_bcmath_number("-003.50").expect("valid decimal");
        assert_eq!(format_bcmath_number(&number, 2).expect("format"), "-3.50");
    }

    /// Verifies PHP's syntactically valid digitless forms normalize to positive zero.
    #[test]
    fn parse_accepts_digitless_zero_forms() {
        for zero in ["", "+", "-", ".", "+.", "-."] {
            let number = parse_bcmath_number(zero).expect("digitless zero");
            assert_eq!(format_bcmath_number(&number, 2).expect("format"), "0.00");
        }
    }

    /// Verifies whitespace, scientific notation, and malformed punctuation are rejected.
    #[test]
    fn parse_rejects_whitespace_scientific_and_junk() {
        for invalid in [" 0", "0 ", "\t0", "1e2", "1.2.3", "..", "+-"] {
            assert!(parse_bcmath_number(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    /// Verifies PHP's digit-optional forms around the decimal point remain valid.
    #[test]
    fn parse_accepts_fractional_only_and_trailing_dot() {
        assert_eq!(
            format_bcmath_number(&parse_bcmath_number(".5").expect("parse"), 1)
                .expect("format"),
            "0.5"
        );
        assert_eq!(
            format_bcmath_number(&parse_bcmath_number("5.").expect("parse"), 0)
                .expect("format"),
            "5"
        );
    }
}
