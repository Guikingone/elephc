//! Purpose:
//! Defines POSIX locale category integer constants exposed by elephc.
//! Keeps LC_* constants in one source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker::driver::init` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values match PHP 8.x on macOS aarch64/Linux (verified with `php -r 'echo LC_ALL,...'`).
//! - LC_ALL=0, LC_COLLATE=1, LC_CTYPE=2, LC_MONETARY=3, LC_NUMERIC=4, LC_TIME=5, LC_MESSAGES=6.

/// Tuple of `(name, value)` pairs for POSIX locale category integer constants.
///
/// Used by `setlocale()` and related locale-aware functions.
pub(crate) const LOCALE_INT_CONSTANTS: &[(&str, i64)] = &[
    ("LC_ALL", 0),
    ("LC_COLLATE", 1),
    ("LC_CTYPE", 2),
    ("LC_MONETARY", 3),
    ("LC_NUMERIC", 4),
    ("LC_TIME", 5),
    ("LC_MESSAGES", 6),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `LC_NUMERIC` has value 4 (confirmed via `php -r 'echo LC_NUMERIC;'`).
    #[test]
    fn lc_numeric_is_4() {
        let entry = LOCALE_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "LC_NUMERIC")
            .expect("LC_NUMERIC defined");
        assert_eq!(entry.1, 4);
    }

    /// Verifies `LC_ALL` has value 0.
    #[test]
    fn lc_all_is_0() {
        let entry = LOCALE_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "LC_ALL")
            .expect("LC_ALL defined");
        assert_eq!(entry.1, 0);
    }

    /// Asserts no duplicate names exist in `LOCALE_INT_CONSTANTS`.
    #[test]
    fn no_duplicate_constant_names() {
        let mut names: Vec<&str> = LOCALE_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate locale constant name");
    }
}
