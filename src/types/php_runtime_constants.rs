//! Purpose:
//! Defines PHP runtime and version integer constants exposed by elephc.
//! Keeps version-identification and sizing constants in one source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - `PHP_INT_SIZE` = 8 on all supported targets (64-bit LP64 only).
//! - Version constants (`PHP_VERSION_ID`, etc.) claim PHP 8.4 compatibility.
//! - String constants (`PHP_SAPI`, `PHP_VERSION`, `PHP_OS_FAMILY`) are handled
//!   directly in `crate::codegen::prescan` to allow target-aware values.

/// Tuple of `(name, value)` pairs for PHP runtime integer constants.
///
/// Used by code that inspects the PHP version or platform word size at compile time.
pub(crate) const PHP_RUNTIME_INT_CONSTANTS: &[(&str, i64)] = &[
    ("PHP_MAJOR_VERSION", 8),
    ("PHP_MINOR_VERSION", 4),
    ("PHP_RELEASE_VERSION", 0),
    ("PHP_VERSION_ID", 80400),
    ("PHP_INT_SIZE", 8),
];

/// The PHP version string reported by `PHP_VERSION`.
pub(crate) const PHP_VERSION_STR: &str = "8.4.0";

/// The SAPI name reported by `PHP_SAPI` (always CLI for elephc-compiled programs).
pub(crate) const PHP_SAPI_STR: &str = "cli";

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies `PHP_INT_SIZE` is 8 (64-bit LP64 target).
    #[test]
    fn php_int_size_is_8() {
        let entry = PHP_RUNTIME_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PHP_INT_SIZE")
            .expect("PHP_INT_SIZE defined");
        assert_eq!(entry.1, 8);
    }

    /// Verifies `PHP_VERSION_ID` encodes major*10000 + minor*100 + release = 80400.
    #[test]
    fn php_version_id_matches_components() {
        let id = PHP_RUNTIME_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PHP_VERSION_ID")
            .expect("PHP_VERSION_ID defined")
            .1;
        let major = PHP_RUNTIME_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PHP_MAJOR_VERSION")
            .expect("PHP_MAJOR_VERSION defined")
            .1;
        let minor = PHP_RUNTIME_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PHP_MINOR_VERSION")
            .expect("PHP_MINOR_VERSION defined")
            .1;
        let release = PHP_RUNTIME_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PHP_RELEASE_VERSION")
            .expect("PHP_RELEASE_VERSION defined")
            .1;
        assert_eq!(id, major * 10000 + minor * 100 + release);
    }

    /// Asserts no duplicate names exist in `PHP_RUNTIME_INT_CONSTANTS`.
    #[test]
    fn no_duplicate_constant_names() {
        let mut names: Vec<&str> = PHP_RUNTIME_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate PHP runtime constant name");
    }
}
