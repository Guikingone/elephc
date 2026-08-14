//! Purpose:
//! Defines PHP array-related integer constants exposed by elephc.
//! Keeps callback-mode constants in one source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values must match PHP's array extension constants exactly for callback, extraction, and
//!   sorting parity.

/// Tuple of `(name, value)` pairs for PHP array integer constants.
///
/// `array_filter()` uses the `ARRAY_FILTER_*` constants to select which callback arguments
/// are passed; `count()` uses the `COUNT_*` constants to select flat or recursive counting.
pub(crate) const ARRAY_INT_CONSTANTS: &[(&str, i64)] = &[
    ("ARRAY_FILTER_USE_VALUE", 0),
    ("ARRAY_FILTER_USE_BOTH", 1),
    ("ARRAY_FILTER_USE_KEY", 2),
    ("COUNT_NORMAL", 0),
    ("COUNT_RECURSIVE", 1),
    ("EXTR_OVERWRITE", 0),
    ("EXTR_SKIP", 1),
    ("EXTR_PREFIX_SAME", 2),
    ("EXTR_PREFIX_ALL", 3),
    ("EXTR_PREFIX_INVALID", 4),
    ("EXTR_PREFIX_IF_EXISTS", 5),
    ("EXTR_IF_EXISTS", 6),
    ("EXTR_REFS", 256),
    ("SORT_REGULAR", 0),
    ("SORT_NUMERIC", 1),
    ("SORT_STRING", 2),
    ("SORT_LOCALE_STRING", 5),
    ("SORT_NATURAL", 6),
    ("SORT_FLAG_CASE", 8),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies PHP 8.6's value-mode constant is the default mode value.
    #[test]
    fn array_filter_use_value_is_zero() {
        let entry = ARRAY_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "ARRAY_FILTER_USE_VALUE")
            .expect("ARRAY_FILTER_USE_VALUE defined");
        assert_eq!(entry.1, 0);
    }

    /// Verifies `count()`'s mode constants carry php-src's exact values.
    ///
    /// `count()`'s omitted-`$mode` default and its `ValueError` range check both assume
    /// `COUNT_NORMAL == 0` and `COUNT_RECURSIVE == 1`.
    #[test]
    fn count_modes_match_php() {
        let normal = ARRAY_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "COUNT_NORMAL")
            .expect("COUNT_NORMAL defined");
        let recursive = ARRAY_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "COUNT_RECURSIVE")
            .expect("COUNT_RECURSIVE defined");
        assert_eq!((normal.1, recursive.1), (0, 1));
    }

    /// Asserts no duplicate names exist in `ARRAY_INT_CONSTANTS`.
    #[test]
    fn no_duplicate_constant_names() {
        let mut names: Vec<&str> = ARRAY_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate array constant name");
    }
}
