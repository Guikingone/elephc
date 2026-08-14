//! Purpose:
//! Owns BCMath's sign/base-10-digit/scale representation and unsigned digit arithmetic.
//!
//! Called from:
//! - `crate::parse` to construct normalized decimal values.
//! - Arithmetic, power, square-root, and rounding modules for exact decimal operations.
//!
//! Key details:
//! - Digits are most-significant first and normalized to one zero digit for zero.
//! - Helpers never choose PHP output scale or formatting policy.

use std::cmp::Ordering;

/// An exact signed base-10 number whose coefficient is `digits * 10^-scale`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BcNum {
    /// Whether the value is negative; normalized zero is never negative.
    pub negative: bool,
    /// Base-10 coefficient digits, most significant first.
    pub digits: Vec<u8>,
    /// Number of coefficient digits after the decimal point.
    pub scale: i32,
}
impl BcNum {
    /// Constructs a normalized decimal number from a sign, coefficient, and scale.
    pub(crate) fn new(negative: bool, digits: Vec<u8>, scale: i32) -> Self {
        let digits = normalize_digits(digits);
        let negative = negative && !is_zero_digits(&digits);
        Self {
            negative,
            digits,
            scale,
        }
    }

    /// Returns true when the numeric value is zero.
    pub fn is_zero(&self) -> bool {
        is_zero_digits(&self.digits)
    }
}

/// Removes leading coefficient zeros while retaining one digit for zero.
pub(crate) fn normalize_digits(mut digits: Vec<u8>) -> Vec<u8> {
    let first_nonzero = digits.iter().position(|digit| *digit != 0);
    match first_nonzero {
        Some(0) => digits,
        Some(index) => {
            digits.drain(..index);
            digits
        }
        None => vec![0],
    }
}

/// Returns true when a normalized or unnormalized coefficient is zero.
pub(crate) fn is_zero_digits(digits: &[u8]) -> bool {
    digits.iter().all(|digit| *digit == 0)
}

/// Compares two unsigned base-10 coefficients.
pub(crate) fn cmp_digits(left: &[u8], right: &[u8]) -> Ordering {
    let left = trim_leading(left);
    let right = trim_leading(right);
    left.len()
        .cmp(&right.len())
        .then_with(|| left.cmp(right))
}

/// Adds two unsigned base-10 coefficients.
pub(crate) fn add_digits(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut li = left.len();
    let mut ri = right.len();
    let mut carry = 0u8;
    while li > 0 || ri > 0 || carry != 0 {
        let l = if li > 0 {
            li -= 1;
            left[li]
        } else {
            0
        };
        let r = if ri > 0 {
            ri -= 1;
            right[ri]
        } else {
            0
        };
        let sum = l + r + carry;
        out.push(sum % 10);
        carry = sum / 10;
    }
    out.reverse();
    normalize_digits(out)
}

/// Subtracts `right` from `left`, requiring `left >= right`.
pub(crate) fn sub_digits(left: &[u8], right: &[u8]) -> Vec<u8> {
    debug_assert!(cmp_digits(left, right) != Ordering::Less);
    let mut out = Vec::with_capacity(left.len());
    let mut li = left.len();
    let mut ri = right.len();
    let mut borrow = 0i8;
    while li > 0 {
        li -= 1;
        let mut value = left[li] as i8 - borrow;
        let r = if ri > 0 {
            ri -= 1;
            right[ri] as i8
        } else {
            0
        };
        if value < r {
            value += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out.push((value - r) as u8);
    }
    debug_assert_eq!(borrow, 0);
    out.reverse();
    normalize_digits(out)
}

/// Multiplies two unsigned base-10 coefficients.
pub(crate) fn mul_digits(left: &[u8], right: &[u8]) -> Vec<u8> {
    if is_zero_digits(left) || is_zero_digits(right) {
        return vec![0];
    }
    let mut accum = vec![0u32; left.len() + right.len()];
    for (li, l) in left.iter().rev().enumerate() {
        for (ri, r) in right.iter().rev().enumerate() {
            let index = accum.len() - 1 - (li + ri);
            accum[index] += u32::from(*l) * u32::from(*r);
        }
    }
    for index in (1..accum.len()).rev() {
        let carry = accum[index] / 10;
        accum[index] %= 10;
        accum[index - 1] += carry;
    }
    while accum[0] >= 10 {
        let carry = accum[0] / 10;
        accum[0] %= 10;
        accum.insert(0, carry);
    }
    normalize_digits(accum.into_iter().map(|digit| digit as u8).collect())
}

/// Multiplies an unsigned coefficient by one decimal digit-sized value.
pub(crate) fn mul_small(digits: &[u8], factor: u8) -> Vec<u8> {
    if factor == 0 || is_zero_digits(digits) {
        return vec![0];
    }
    let mut out = Vec::with_capacity(digits.len() + 1);
    let mut carry = 0u16;
    for digit in digits.iter().rev() {
        let value = u16::from(*digit) * u16::from(factor) + carry;
        out.push((value % 10) as u8);
        carry = value / 10;
    }
    while carry != 0 {
        out.push((carry % 10) as u8);
        carry /= 10;
    }
    out.reverse();
    normalize_digits(out)
}

/// Adds a value smaller than ten to an unsigned coefficient.
pub(crate) fn add_small(digits: &[u8], value: u8) -> Vec<u8> {
    add_digits(digits, &[value])
}

/// Appends decimal zero digits, equivalent to multiplying by a power of ten.
pub(crate) fn append_zeros(mut digits: Vec<u8>, count: usize) -> Vec<u8> {
    if !is_zero_digits(&digits) {
        digits.resize(digits.len().saturating_add(count), 0);
    }
    digits
}

/// Divides two unsigned coefficients and returns quotient and remainder.
pub(crate) fn div_rem_digits(
    numerator: &[u8],
    denominator: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    debug_assert!(!is_zero_digits(denominator));
    if cmp_digits(numerator, denominator) == Ordering::Less {
        return (vec![0], normalize_digits(numerator.to_vec()));
    }
    let mut quotient = Vec::with_capacity(numerator.len());
    let mut remainder = vec![0];
    for digit in numerator {
        if is_zero_digits(&remainder) {
            remainder[0] = *digit;
        } else {
            remainder.push(*digit);
        }
        remainder = normalize_digits(remainder);
        let mut qdigit = 0u8;
        while cmp_digits(&remainder, denominator) != Ordering::Less {
            remainder = sub_digits(&remainder, denominator);
            qdigit += 1;
        }
        quotient.push(qdigit);
    }
    (normalize_digits(quotient), normalize_digits(remainder))
}

/// Returns the unsigned coefficient modulo `modulus`.
pub(crate) fn mod_digits(value: &[u8], modulus: &[u8]) -> Vec<u8> {
    div_rem_digits(value, modulus).1
}

/// Divides an unsigned coefficient by two, returning quotient and remainder bit.
pub(crate) fn div_two(digits: &[u8]) -> (Vec<u8>, u8) {
    let mut quotient = Vec::with_capacity(digits.len());
    let mut carry = 0u8;
    for digit in digits {
        let value = carry * 10 + *digit;
        quotient.push(value / 2);
        carry = value % 2;
    }
    (normalize_digits(quotient), carry)
}

/// Returns a coefficient slice without insignificant leading zeros.
fn trim_leading(digits: &[u8]) -> &[u8] {
    match digits.iter().position(|digit| *digit != 0) {
        Some(index) => &digits[index..],
        None => &digits[digits.len().saturating_sub(1)..],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies long division retains both quotient and remainder exactly.
    #[test]
    fn decimal_long_division_round_trips() {
        let (quotient, remainder) = div_rem_digits(&[1, 2, 3, 4, 5], &[6, 7]);
        assert_eq!(quotient, vec![1, 8, 4]);
        assert_eq!(remainder, vec![1, 7]);
    }

    /// Verifies multiplication carries across several coefficient positions.
    #[test]
    fn decimal_multiplication_carries() {
        assert_eq!(mul_digits(&[9, 9, 9], &[9, 9]), vec![9, 8, 9, 0, 1]);
    }
}
