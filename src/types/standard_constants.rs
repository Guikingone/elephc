//! Purpose:
//! Defines predefined PHP integer constants shared by core and standard extensions.
//! Keeps target-independent values in one source of truth for checking and code generation.
//!
//! Called from:
//! - `crate::types::checker::driver::init` when registering predefined constant types.
//! - `crate::codegen_support::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Target-dependent constants such as `LC_NUMERIC` and `PHP_MAXPATHLEN` stay on `Platform`.
//! - Values mirror PHP's public constants rather than any particular application or tool.

/// Target-independent predefined integer constants exposed by core and standard extensions.
pub(crate) const STANDARD_INT_CONSTANTS: &[(&str, i64)] = &[
    ("DEBUG_BACKTRACE_PROVIDE_OBJECT", 1),
    ("DEBUG_BACKTRACE_IGNORE_ARGS", 2),
    ("PHP_OUTPUT_HANDLER_WRITE", 0),
    ("PHP_OUTPUT_HANDLER_START", 1),
    ("PHP_OUTPUT_HANDLER_CLEAN", 2),
    ("PHP_OUTPUT_HANDLER_FLUSH", 4),
    ("PHP_OUTPUT_HANDLER_FINAL", 8),
    ("PHP_OUTPUT_HANDLER_CONT", 0),
    ("PHP_OUTPUT_HANDLER_END", 8),
    ("PHP_OUTPUT_HANDLER_CLEANABLE", 16),
    ("PHP_OUTPUT_HANDLER_FLUSHABLE", 32),
    ("PHP_OUTPUT_HANDLER_REMOVABLE", 64),
    ("PHP_OUTPUT_HANDLER_STDFLAGS", 112),
    ("PHP_OUTPUT_HANDLER_STARTED", 4096),
    ("PHP_OUTPUT_HANDLER_DISABLED", 8192),
    ("PHP_OUTPUT_HANDLER_PROCESSED", 16384),
    ("UPLOAD_ERR_OK", 0),
    ("UPLOAD_ERR_INI_SIZE", 1),
    ("UPLOAD_ERR_FORM_SIZE", 2),
    ("UPLOAD_ERR_PARTIAL", 3),
    ("UPLOAD_ERR_NO_FILE", 4),
    ("UPLOAD_ERR_NO_TMP_DIR", 6),
    ("UPLOAD_ERR_CANT_WRITE", 7),
    ("UPLOAD_ERR_EXTENSION", 8),
    ("CASE_LOWER", 0),
    ("CASE_UPPER", 1),
    ("INI_SCANNER_NORMAL", 0),
    ("INI_SCANNER_RAW", 1),
    ("INI_SCANNER_TYPED", 2),
    ("PHP_QUERY_RFC1738", 1),
    ("PHP_QUERY_RFC3986", 2),
    ("SIG_DFL", 0),
    ("SIGINT", 2),
    ("SIGQUIT", 3),
    ("SIGALRM", 14),
    ("SIGTERM", 15),
    ("GRAPHEME_EXTR_MAXBYTES", 1),
    ("FILEINFO_MIME_TYPE", 16),
    ("PCRE_VERSION_MAJOR", 10),
    ("PCRE_VERSION_MINOR", 47),
    ("LIBXML_COMPACT", 65_536),
    ("LIBXML_NONET", 2048),
    ("LIBXML_ERR_NONE", 0),
    ("LIBXML_ERR_WARNING", 1),
    ("LIBXML_ERR_ERROR", 2),
    ("LIBXML_ERR_FATAL", 3),
];

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for predefined target-independent integer constant values and uniqueness.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.

    use super::*;

    /// Verifies representative constants from each predefined group match PHP values.
    #[test]
    fn representative_values_match_php() {
        let value = |name: &str| {
            STANDARD_INT_CONSTANTS
                .iter()
                .find(|(candidate, _)| *candidate == name)
                .map(|(_, value)| *value)
                .expect("predefined constant")
        };
        assert_eq!(value("UPLOAD_ERR_NO_FILE"), 4);
        assert_eq!(value("PHP_QUERY_RFC3986"), 2);
        assert_eq!(value("PHP_OUTPUT_HANDLER_REMOVABLE"), 64);
        assert_eq!(value("SIGINT"), 2);
        assert_eq!(value("SIGQUIT"), 3);
        assert_eq!(value("SIGTERM"), 15);
        assert_eq!(value("GRAPHEME_EXTR_MAXBYTES"), 1);
        assert_eq!(value("FILEINFO_MIME_TYPE"), 16);
        assert_eq!(value("PCRE_VERSION_MAJOR"), 10);
        assert_eq!(value("PCRE_VERSION_MINOR"), 47);
        assert_eq!(value("LIBXML_COMPACT"), 65_536);
        assert_eq!(value("LIBXML_NONET"), 2048);
    }

    /// Verifies the predefined constant registry contains no duplicate names.
    #[test]
    fn constant_names_are_unique() {
        let mut names: Vec<&str> = STANDARD_INT_CONSTANTS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate predefined constant name");
    }
}
