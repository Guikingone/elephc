//! Purpose:
//! Defines PHP string-function integer constants exposed by elephc.
//! Keeps `str_pad()` padding modes, case-conversion modes, and html-entity flags in one
//! source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - `STR_PAD_*` values match ext/standard's `str_pad()` mode constants.
//! - `ENT_*` values match ext/standard's `htmlspecialchars()`/`htmlentities()` flags.

/// Tuple of `(name, value)` pairs for PHP string-function integer constants.
///
/// Covers `str_pad()` padding modes (`STR_PAD_*`), case-conversion modes (`CASE_*`), and
/// the html-entity quote/document flags (`ENT_*`) used by `htmlspecialchars()` and friends.
pub(crate) const STRING_INT_CONSTANTS: &[(&str, i64)] = &[
    ("STR_PAD_RIGHT", 1),
    ("STR_PAD_LEFT", 0),
    ("STR_PAD_BOTH", 2),
    ("CASE_LOWER", 0),
    ("CASE_UPPER", 1),
    ("ENT_QUOTES", 3),
    ("ENT_COMPAT", 2),
    ("ENT_HTML401", 0),
];

#[cfg(test)]
mod tests {
    use super::STRING_INT_CONSTANTS;

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        STRING_INT_CONSTANTS
            .iter()
            .find(|(constant_name, _)| *constant_name == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Verifies `str_pad()` mode constants match PHP's ext/standard values.
    #[test]
    fn test_str_pad_mode_values() {
        assert_eq!(value_of("STR_PAD_RIGHT"), 1);
        assert_eq!(value_of("STR_PAD_LEFT"), 0);
        assert_eq!(value_of("STR_PAD_BOTH"), 2);
    }

    /// Verifies PHP's lower/upper case-conversion mode values.
    #[test]
    fn test_case_conversion_mode_values() {
        assert_eq!(value_of("CASE_LOWER"), 0);
        assert_eq!(value_of("CASE_UPPER"), 1);
    }

    /// Verifies html-entity flag constants match PHP's ext/standard values.
    #[test]
    fn test_html_entity_flag_values() {
        assert_eq!(value_of("ENT_QUOTES"), 3);
        assert_eq!(value_of("ENT_COMPAT"), 2);
        assert_eq!(value_of("ENT_HTML401"), 0);
    }

    /// Asserts no duplicate names exist in `STRING_INT_CONSTANTS`.
    #[test]
    fn test_string_constants_have_unique_names() {
        let mut names: Vec<&str> = STRING_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate string constant name");
    }
}
