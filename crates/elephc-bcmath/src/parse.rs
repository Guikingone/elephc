//! Purpose:
//! Parses the ASCII decimal grammar accepted by PHP BCMath functions.
//!
//! Called from:
//! - Every arithmetic operation before exact decimal processing.
//! - Crate tests that pin accepted and rejected numeric strings.
//!
//! Key details:
//! - ASCII edge whitespace is trimmed, while scientific notation is rejected.
//! - At least one digit is required across the integer and fractional portions.

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
    let bytes = trim_ascii(input.as_bytes());
    let mut index = 0usize;
    let mut negative = false;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        negative = bytes[0] == b'-';
        index += 1;
    }

    let mut digits = Vec::with_capacity(bytes.len());
    let mut saw_digit = false;
    let mut saw_dot = false;
    let mut scale = 0i32;
    while index < bytes.len() {
        match bytes[index] {
            b'0'..=b'9' => {
                saw_digit = true;
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
    if !saw_digit {
        return Err(BcError::Malformed {
            func,
            arg_pos,
            arg_name,
        });
    }
    Ok(BcNum::new(negative, digits, scale))
}

/// Trims only the ASCII whitespace accepted around BCMath numeric strings.
fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while bytes.first().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[1..];
    }
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes = &bytes[..bytes.len() - 1];
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::format_bcmath_number;

    /// Verifies signed decimals with surrounding ASCII whitespace are accepted and normalized.
    #[test]
    fn parse_accepts_trimmed_signed_decimal() {
        let number = parse_bcmath_number("  -003.50  ").expect("valid decimal");
        assert_eq!(format_bcmath_number(&number, 2).expect("format"), "-3.50");
    }

    /// Verifies empty, sign-only, dot-only, and scientific forms are rejected.
    #[test]
    fn parse_rejects_scientific_and_empty() {
        for invalid in ["1e2", "", ".", "+", "-"] {
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

