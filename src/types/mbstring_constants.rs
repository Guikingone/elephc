//! Purpose:
//! Defines PHP mbstring case-mode integer constants exposed by elephc.
//! Keeps `mb_convert_case()` mode flags in one source of truth for type checking
//! and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values match ext/mbstring's `MB_CASE_*` constants exactly.

/// Tuple of `(name, value)` pairs for PHP mbstring case-mode integer constants.
///
/// Selects the case-conversion mode passed to `mb_convert_case()`.
pub(crate) const MBSTRING_INT_CONSTANTS: &[(&str, i64)] = &[
    ("MB_CASE_FOLD_SIMPLE", 7),
];

#[cfg(test)]
mod tests {
    use super::MBSTRING_INT_CONSTANTS;

    /// Verifies `MB_CASE_FOLD_SIMPLE` keeps its ext/mbstring value.
    #[test]
    fn test_mb_case_fold_simple_value() {
        let entry = MBSTRING_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "MB_CASE_FOLD_SIMPLE")
            .expect("MB_CASE_FOLD_SIMPLE defined");
        assert_eq!(entry.1, 7);
    }

    /// Asserts no duplicate names exist in `MBSTRING_INT_CONSTANTS`.
    #[test]
    fn test_mbstring_constants_have_unique_names() {
        let mut names: Vec<&str> = MBSTRING_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate mbstring constant name");
    }
}
