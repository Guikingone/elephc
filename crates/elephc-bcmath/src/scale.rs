//! Purpose:
//! Stores and validates BCMath's process-wide default scale.
//!
//! Called from:
//! - Arithmetic operations when their explicit scale is omitted or null.
//! - AOT and Magician `bcscale()` wrappers through the shared crate instance.
//!
//! Key details:
//! - The default is zero and accepted values match PHP's signed 32-bit non-negative range.

use std::sync::atomic::{AtomicI32, Ordering};

use crate::error::BcError;

static GLOBAL_SCALE: AtomicI32 = AtomicI32::new(0);

/// Returns the current process-wide BCMath scale.
pub fn get_scale() -> i32 {
    GLOBAL_SCALE.load(Ordering::SeqCst)
}

/// Sets the process-wide BCMath scale and returns its previous value.
pub fn set_scale(scale: i64) -> Result<i32, BcError> {
    let scale = validate_scale(scale, "bcscale", 1)?;
    Ok(GLOBAL_SCALE.swap(scale, Ordering::SeqCst))
}

/// Resolves an optional operation scale, using the global scale for `None`.
pub(crate) fn resolve_scale(
    scale: Option<i64>,
    func: &'static str,
    arg_pos: u32,
) -> Result<i32, BcError> {
    match scale {
        Some(scale) => validate_scale(scale, func, arg_pos),
        None => Ok(get_scale()),
    }
}

/// Validates one scale against PHP's `0..=2147483647` range.
fn validate_scale(scale: i64, func: &'static str, arg_pos: u32) -> Result<i32, BcError> {
    i32::try_from(scale)
        .ok()
        .filter(|scale| *scale >= 0)
        .ok_or(BcError::ScaleRange { func, arg_pos })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies global scale get/set behavior and rejected negative scales.
    #[test]
    fn scale_get_set_round_trips() {
        let saved = get_scale();
        let previous = set_scale(4).expect("set scale");
        assert_eq!(previous, saved);
        assert_eq!(get_scale(), 4);
        assert!(set_scale(-1).is_err());
        set_scale(i64::from(saved)).expect("restore scale");
    }
}
