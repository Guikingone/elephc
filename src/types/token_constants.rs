//! Purpose:
//! Defines version-sensitive integer constants exposed by PHP's tokenizer surface.
//! Keeps token identifiers aligned with the selected compatibility profile.
//!
//! Called from:
//! - `crate::types::checker::driver::init` when registering predefined constant types.
//! - `crate::codegen_support::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Token identifiers may change between PHP minor versions and must not be treated as stable.
//! - Released profiles use their generated parser tables; preview profiles follow their grammar.

use crate::php_version::PhpVersion;

/// Version-sensitive tokenizer constants and their values from PHP 8.0 through 8.6.
pub(crate) const TOKEN_INT_CONSTANTS: &[(&str, [i64; 7])] = &[
    ("T_COMMENT", [388, 387, 387, 392, 391, 392, 398]),
    ("T_DOC_COMMENT", [389, 388, 388, 393, 392, 393, 399]),
    ("T_OPEN_TAG", [390, 389, 389, 394, 393, 394, 400]),
    ("T_OPEN_TAG_WITH_ECHO", [391, 390, 390, 395, 394, 395, 401]),
    ("T_CLOSE_TAG", [392, 391, 391, 396, 395, 396, 402]),
    ("T_WHITESPACE", [393, 392, 392, 397, 396, 397, 403]),
    ("T_START_HEREDOC", [394, 393, 393, 398, 397, 398, 404]),
    ("T_END_HEREDOC", [395, 394, 394, 399, 398, 399, 405]),
    ("T_DOLLAR_OPEN_CURLY_BRACES", [396, 395, 395, 400, 399, 400, 406]),
    ("T_CURLY_OPEN", [397, 396, 396, 401, 400, 401, 407]),
    ("T_PAAMAYIM_NEKUDOTAYIM", [398, 397, 397, 402, 401, 402, 408]),
    ("T_DOUBLE_COLON", [398, 397, 397, 402, 401, 402, 408]),
];

/// Returns the selected profile's value for a predefined tokenizer constant.
pub(crate) fn token_int_constant_value(name: &str, php_version: PhpVersion) -> Option<i64> {
    let profile_index = match php_version {
        PhpVersion::Php80 => 0,
        PhpVersion::Php81 => 1,
        PhpVersion::Php82 => 2,
        PhpVersion::Php83 => 3,
        PhpVersion::Php84 => 4,
        PhpVersion::Php85 => 5,
        PhpVersion::Php86 => 6,
    };
    TOKEN_INT_CONSTANTS
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, values)| values[profile_index])
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for version-sensitive tokenizer constant values and uniqueness.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Each expected value is sourced from the matching generated parser table or grammar.

    use super::*;

    /// Verifies `T_START_HEREDOC` follows each supported compatibility profile.
    #[test]
    fn heredoc_token_values_follow_php_profiles() {
        let expected = [394, 393, 393, 398, 397, 398, 404];
        for (profile, value) in PhpVersion::ALL.into_iter().zip(expected) {
            assert_eq!(token_int_constant_value("T_START_HEREDOC", profile), Some(value));
        }
    }

    /// Verifies `T_WHITESPACE` follows each supported compatibility profile.
    #[test]
    fn whitespace_token_values_follow_php_profiles() {
        let expected = [393, 392, 392, 397, 396, 397, 403];
        for (profile, value) in PhpVersion::ALL.into_iter().zip(expected) {
            assert_eq!(token_int_constant_value("T_WHITESPACE", profile), Some(value));
        }
    }

    /// Verifies the neighboring lexical-token constants retain their parser-table ordering.
    #[test]
    fn neighboring_lexical_tokens_follow_php_profiles() {
        for profile in PhpVersion::ALL {
            let whitespace = token_int_constant_value("T_WHITESPACE", profile).unwrap();
            assert_eq!(token_int_constant_value("T_COMMENT", profile), Some(whitespace - 5));
            assert_eq!(token_int_constant_value("T_CLOSE_TAG", profile), Some(whitespace - 1));
            assert_eq!(token_int_constant_value("T_END_HEREDOC", profile), Some(whitespace + 2));
            assert_eq!(token_int_constant_value("T_CURLY_OPEN", profile), Some(whitespace + 4));
            assert_eq!(
                token_int_constant_value("T_DOUBLE_COLON", profile),
                token_int_constant_value("T_PAAMAYIM_NEKUDOTAYIM", profile)
            );
        }
    }

    /// Verifies the tokenizer constant registry contains no duplicate names.
    #[test]
    fn token_constant_names_are_unique() {
        let mut names: Vec<&str> = TOKEN_INT_CONSTANTS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate tokenizer constant name");
    }
}
