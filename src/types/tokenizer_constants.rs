//! Purpose:
//! Defines `ext/tokenizer` token-kind integer constants (`T_*`) exposed by
//! elephc. Only the subset demanded by real-world PHP source (currently the
//! heredoc/nowdoc boundary tokens) is registered; the full tokenizer token
//! space is intentionally out of scope.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - `T_*` token-kind numbering is PHP-version-sensitive (new tokens are
//!   appended as the grammar grows). Values here are pinned to PHP 8.5.6
//!   local (`php -n -r 'var_dump(T_START_HEREDOC, T_END_HEREDOC);'`) and match
//!   the stable values PHP has used since heredoc/nowdoc tokens were split out.

/// Tuple of `(name, value)` pairs for the demanded `ext/tokenizer` `T_*` constants.
pub(crate) const TOKENIZER_INT_CONSTANTS: &[(&str, i64)] = &[
    ("T_START_HEREDOC", 398),
    ("T_END_HEREDOC", 399),
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

    /// Verifies the heredoc/nowdoc boundary token ids match PHP 8.5.6 local.
    #[test]
    fn test_heredoc_token_ids_match_php() {
        assert_eq!(value_of("T_START_HEREDOC"), 398);
        assert_eq!(value_of("T_END_HEREDOC"), 399);
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
