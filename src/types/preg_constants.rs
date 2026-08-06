//! Purpose:
//! Defines preg/PCRE flag constants exposed as PHP integer constants.
//! Keeps regex-related global constants in one shared table for checker and codegen seeding.
//!
//! Called from:
//! - `crate::types::checker::driver::init`
//! - `crate::codegen::prescan`
//!
//! Key details:
//! - Values match PHP's ext/pcre constants for RegexIterator and preg_split flag parity.
//! - `PCRE_VERSION_MAJOR`/`PCRE_VERSION_MINOR` and the `PCRE_VERSION` string mirror
//!   the PCRE2 release bundled with PHP 8.4 (10.42).

/// PHP preg integer constants used by regex builtins and SPL regex iterators.
///
/// Also exposes the `PCRE_VERSION_MAJOR`/`PCRE_VERSION_MINOR` components of the
/// bundled PCRE2 library version.
pub(crate) const PREG_INT_CONSTANTS: &[(&str, i64)] = &[
    ("PREG_PATTERN_ORDER", 1),
    ("PREG_SET_ORDER", 2),
    ("PREG_OFFSET_CAPTURE", 256),
    ("PREG_UNMATCHED_AS_NULL", 512),
    ("PREG_SPLIT_NO_EMPTY", 1),
    ("PREG_SPLIT_DELIM_CAPTURE", 2),
    ("PREG_SPLIT_OFFSET_CAPTURE", 4),
    ("PREG_GREP_INVERT", 1),
    // `preg_last_error()`'s result codes. They were absent entirely, so
    // `symfony/string`'s `ByteString::replaceMatches()` — which scans
    // `get_defined_constants(true)['pcre']` for the `*_ERROR` name matching
    // `preg_last_error()` — could never name the failure.
    ("PREG_NO_ERROR", 0),
    ("PREG_INTERNAL_ERROR", 1),
    ("PREG_BACKTRACK_LIMIT_ERROR", 2),
    ("PREG_RECURSION_LIMIT_ERROR", 3),
    ("PREG_BAD_UTF8_ERROR", 4),
    ("PREG_BAD_UTF8_OFFSET_ERROR", 5),
    ("PREG_JIT_STACKLIMIT_ERROR", 6),
    ("PCRE_VERSION_MAJOR", 10),
    ("PCRE_VERSION_MINOR", 42),
];

/// The PCRE2 library version string reported by the `PCRE_VERSION` constant.
///
/// Matches the PCRE2 release bundled with the emulated PHP 8.4 runtime.
// Retained for the checker unit tests and pending re-integration after the
// origin/main merge; not reachable from the production checker paths yet.
#[allow(dead_code)]
pub(crate) const PCRE_VERSION_STR: &str = "10.42 2022-12-11";

#[cfg(test)]
mod tests {
    use super::{PCRE_VERSION_STR, PREG_INT_CONSTANTS};

    /// Verifies that PHP's offset-capture bit keeps its ext/pcre value.
    #[test]
    fn test_preg_offset_capture_value() {
        let entry = PREG_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PREG_OFFSET_CAPTURE")
            .expect("PREG_OFFSET_CAPTURE defined");
        assert_eq!(entry.1, 256);
    }

    /// Verifies the PCRE version components match the bundled PCRE2 10.42 release.
    #[test]
    fn test_pcre_version_components() {
        let major = PREG_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PCRE_VERSION_MAJOR")
            .expect("PCRE_VERSION_MAJOR defined")
            .1;
        let minor = PREG_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "PCRE_VERSION_MINOR")
            .expect("PCRE_VERSION_MINOR defined")
            .1;
        assert_eq!(major, 10);
        assert_eq!(minor, 42);
        assert!(PCRE_VERSION_STR.starts_with(&format!("{major}.{minor}")));
    }

    /// Verifies that no preg constant name is declared twice.
    #[test]
    fn test_preg_constants_have_unique_names() {
        let mut names: Vec<&str> = PREG_INT_CONSTANTS.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), PREG_INT_CONSTANTS.len());
    }
}
