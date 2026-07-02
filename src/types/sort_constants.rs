//! Purpose:
//! Defines PHP sort-flag integer constants exposed by elephc.
//! Keeps the `sort()`/`asort()`/`usort()` comparison-mode flags in one source of
//! truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values match ext/standard's sort comparison-mode constants exactly.

/// Tuple of `(name, value)` pairs for PHP sort-flag integer constants.
///
/// These select the comparison mode passed to array sorting builtins
/// (`sort()`, `rsort()`, `asort()`, `ksort()`, and friends). `SORT_FLAG_CASE`
/// is a modifier bit combined (bitwise-or) with `SORT_STRING`/`SORT_NATURAL`
/// to request case-insensitive comparison.
pub(crate) const SORT_INT_CONSTANTS: &[(&str, i64)] = &[
    ("SORT_STRING", 2),
    ("SORT_NATURAL", 6),
    ("SORT_FLAG_CASE", 8),
];

#[cfg(test)]
mod tests {
    use super::SORT_INT_CONSTANTS;

    /// Verifies the sort comparison-mode constants match PHP's ext/standard values.
    #[test]
    fn test_sort_flag_values() {
        let string_mode = SORT_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "SORT_STRING")
            .expect("SORT_STRING defined")
            .1;
        assert_eq!(string_mode, 2);
        let natural_mode = SORT_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "SORT_NATURAL")
            .expect("SORT_NATURAL defined")
            .1;
        assert_eq!(natural_mode, 6);
        let case_flag = SORT_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "SORT_FLAG_CASE")
            .expect("SORT_FLAG_CASE defined")
            .1;
        assert_eq!(case_flag, 8);
    }

    /// Asserts no duplicate names exist in `SORT_INT_CONSTANTS`.
    #[test]
    fn test_sort_constants_have_unique_names() {
        let mut names: Vec<&str> = SORT_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate sort constant name");
    }
}
