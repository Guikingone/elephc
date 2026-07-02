//! Purpose:
//! Defines PHP `ext/filter` integer constants exposed by elephc.
//! Keeps the validation-filter identifiers in one source of truth for type
//! checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - `FILTER_VALIDATE_BOOL` is PHP 8's alias of `FILTER_VALIDATE_BOOLEAN`; both
//!   resolve to the same ext/filter value (258).

/// Tuple of `(name, value)` pairs for PHP `ext/filter` integer constants.
///
/// Both the modern `FILTER_VALIDATE_BOOL` name and the legacy
/// `FILTER_VALIDATE_BOOLEAN` alias are registered with the same value.
pub(crate) const FILTER_INT_CONSTANTS: &[(&str, i64)] = &[
    ("FILTER_VALIDATE_BOOL", 258),
    ("FILTER_VALIDATE_BOOLEAN", 258),
];

#[cfg(test)]
mod tests {
    use super::FILTER_INT_CONSTANTS;

    /// Verifies the boolean-validation alias pair shares PHP's ext/filter value.
    #[test]
    fn test_filter_validate_bool_alias_shares_value() {
        let bool_value = FILTER_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "FILTER_VALIDATE_BOOL")
            .expect("FILTER_VALIDATE_BOOL defined")
            .1;
        let boolean_value = FILTER_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "FILTER_VALIDATE_BOOLEAN")
            .expect("FILTER_VALIDATE_BOOLEAN defined")
            .1;
        assert_eq!(bool_value, 258);
        assert_eq!(boolean_value, 258);
    }

    /// Asserts no duplicate names exist in `FILTER_INT_CONSTANTS`.
    #[test]
    fn test_filter_constants_have_unique_names() {
        let mut names: Vec<&str> = FILTER_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate filter constant name");
    }
}
