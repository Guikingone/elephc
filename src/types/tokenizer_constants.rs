//! Purpose:
//! Defines `ext/tokenizer` token-kind integer constants (`T_*`) exposed by
//! elephc. Only the subset demanded by the bundled Symfony source is registered; the
//! full tokenizer token space is intentionally out of scope.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - `T_*` token-kind numbering is PHP-version-sensitive. Values here are pinned to
//!   PHP 8.5.6 local and cover exactly the tokens inspected by bundled Symfony code.

/// Tuple of `(name, value)` pairs for the demanded `ext/tokenizer` `T_*` constants.
pub(crate) const TOKENIZER_INT_CONSTANTS: &[(&str, i64)] = &[
    ("T_STRING", 262),
    ("T_NAME_QUALIFIED", 265),
    ("T_INLINE_HTML", 267),
    ("T_NEW", 284),
    ("T_FUNCTION", 310),
    ("T_FN", 311),
    ("T_CLASS", 336),
    ("T_NAMESPACE", 342),
    ("T_COMMENT", 392),
    ("T_DOC_COMMENT", 393),
    ("T_OPEN_TAG", 394),
    ("T_WHITESPACE", 397),
    ("T_START_HEREDOC", 398),
    ("T_END_HEREDOC", 399),
    ("T_DOUBLE_COLON", 402),
    ("T_NS_SEPARATOR", 403),
];

#[cfg(test)]
mod tests {
    use super::TOKENIZER_INT_CONSTANTS;

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        TOKENIZER_INT_CONSTANTS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Verifies Symfony-demanded tokenizer ids match PHP 8.5.6 local.
    #[test]
    fn test_heredoc_token_ids_match_php() {
        assert_eq!(value_of("T_STRING"), 262);
        assert_eq!(value_of("T_NAME_QUALIFIED"), 265);
        assert_eq!(value_of("T_INLINE_HTML"), 267);
        assert_eq!(value_of("T_NEW"), 284);
        assert_eq!(value_of("T_FUNCTION"), 310);
        assert_eq!(value_of("T_FN"), 311);
        assert_eq!(value_of("T_CLASS"), 336);
        assert_eq!(value_of("T_NAMESPACE"), 342);
        assert_eq!(value_of("T_COMMENT"), 392);
        assert_eq!(value_of("T_DOC_COMMENT"), 393);
        assert_eq!(value_of("T_OPEN_TAG"), 394);
        assert_eq!(value_of("T_WHITESPACE"), 397);
        assert_eq!(value_of("T_START_HEREDOC"), 398);
        assert_eq!(value_of("T_END_HEREDOC"), 399);
        assert_eq!(value_of("T_DOUBLE_COLON"), 402);
        assert_eq!(value_of("T_NS_SEPARATOR"), 403);
    }

    /// Asserts no duplicate names exist in `TOKENIZER_INT_CONSTANTS`.
    #[test]
    fn test_tokenizer_constants_have_unique_names() {
        let mut names: Vec<&str> = TOKENIZER_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate tokenizer constant name");
    }
}
