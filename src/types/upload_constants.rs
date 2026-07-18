//! Purpose:
//! Defines PHP file-upload error integer constants (`UPLOAD_ERR_*`) exposed by
//! elephc. Keeps the `$_FILES[...]['error']` code table in one source of truth
//! for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values are stable across PHP versions (unchanged since PHP 4.3) and
//!   php-verified with `php -n -r 'var_dump(UPLOAD_ERR_OK, ...);'` (PHP 8.5.6 local).
//! - There is no code `5`; PHP jumps from `UPLOAD_ERR_PARTIAL` (3) to
//!   `UPLOAD_ERR_NO_FILE` (4) to `UPLOAD_ERR_NO_TMP_DIR` (6).

/// Tuple of `(name, value)` pairs for PHP `UPLOAD_ERR_*` integer constants.
///
/// Used to interpret the `error` field of a `$_FILES` upload-metadata entry.
pub(crate) const UPLOAD_ERR_INT_CONSTANTS: &[(&str, i64)] = &[
    ("UPLOAD_ERR_OK", 0),
    ("UPLOAD_ERR_INI_SIZE", 1),
    ("UPLOAD_ERR_FORM_SIZE", 2),
    ("UPLOAD_ERR_PARTIAL", 3),
    ("UPLOAD_ERR_NO_FILE", 4),
    ("UPLOAD_ERR_NO_TMP_DIR", 6),
    ("UPLOAD_ERR_CANT_WRITE", 7),
    ("UPLOAD_ERR_EXTENSION", 8),
];

#[cfg(test)]
mod tests {
    use super::UPLOAD_ERR_INT_CONSTANTS;

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        UPLOAD_ERR_INT_CONSTANTS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Verifies every `UPLOAD_ERR_*` value matches PHP, including the gap at code 5.
    #[test]
    fn test_upload_err_values_match_php() {
        assert_eq!(value_of("UPLOAD_ERR_OK"), 0);
        assert_eq!(value_of("UPLOAD_ERR_INI_SIZE"), 1);
        assert_eq!(value_of("UPLOAD_ERR_FORM_SIZE"), 2);
        assert_eq!(value_of("UPLOAD_ERR_PARTIAL"), 3);
        assert_eq!(value_of("UPLOAD_ERR_NO_FILE"), 4);
        assert_eq!(value_of("UPLOAD_ERR_NO_TMP_DIR"), 6);
        assert_eq!(value_of("UPLOAD_ERR_CANT_WRITE"), 7);
        assert_eq!(value_of("UPLOAD_ERR_EXTENSION"), 8);
    }

    /// Asserts no duplicate names exist in `UPLOAD_ERR_INT_CONSTANTS`.
    #[test]
    fn test_upload_err_constants_have_unique_names() {
        let mut names: Vec<&str> = UPLOAD_ERR_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate upload_err constant name");
    }
}
