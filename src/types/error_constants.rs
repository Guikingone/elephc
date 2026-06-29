//! Purpose:
//! Defines PHP error-level and debug-backtrace integer constants exposed by elephc.
//! Keeps error-reporting constants in one source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values must match PHP's error-level bitmask constants exactly (verified against PHP 8.5).
//! - `E_ALL` = 30719 (PHP 8.4+: all levels except the deprecated `E_STRICT`).
//! - `E_STRICT` = 2048 is deprecated since PHP 8.4 and is not included in `E_ALL`.

/// Tuple of `(name, value)` pairs for PHP error-level and backtrace integer constants.
///
/// Used by `error_reporting()`, `trigger_error()`, and `debug_backtrace()`.
pub(crate) const ERROR_INT_CONSTANTS: &[(&str, i64)] = &[
    ("E_ERROR", 1),
    ("E_WARNING", 2),
    ("E_PARSE", 4),
    ("E_NOTICE", 8),
    ("E_CORE_ERROR", 16),
    ("E_CORE_WARNING", 32),
    ("E_COMPILE_ERROR", 64),
    ("E_COMPILE_WARNING", 128),
    ("E_USER_ERROR", 256),
    ("E_USER_WARNING", 512),
    ("E_USER_NOTICE", 1024),
    ("E_STRICT", 2048),
    ("E_RECOVERABLE_ERROR", 4096),
    ("E_DEPRECATED", 8192),
    ("E_USER_DEPRECATED", 16384),
    ("E_ALL", 30719),
    ("DEBUG_BACKTRACE_IGNORE_ARGS", 2),
    ("DEBUG_BACKTRACE_PROVIDE_OBJECT", 1),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `E_USER_DEPRECATED` has the PHP-standard value 16384.
    #[test]
    fn e_user_deprecated_is_16384() {
        let entry = ERROR_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "E_USER_DEPRECATED")
            .expect("E_USER_DEPRECATED defined");
        assert_eq!(entry.1, 16384);
    }

    /// Verifies that `E_ALL` equals 30719 (PHP 8.4+, excluding the removed E_STRICT).
    #[test]
    fn e_all_is_30719() {
        let entry = ERROR_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "E_ALL")
            .expect("E_ALL defined");
        assert_eq!(entry.1, 30719);
    }

    /// Verifies that `DEBUG_BACKTRACE_IGNORE_ARGS` has the PHP-standard value 2.
    #[test]
    fn debug_backtrace_ignore_args_is_2() {
        let entry = ERROR_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "DEBUG_BACKTRACE_IGNORE_ARGS")
            .expect("DEBUG_BACKTRACE_IGNORE_ARGS defined");
        assert_eq!(entry.1, 2);
    }

    /// Asserts no duplicate names exist in `ERROR_INT_CONSTANTS`.
    #[test]
    fn no_duplicate_constant_names() {
        let mut names: Vec<&str> = ERROR_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate error constant name");
    }
}
