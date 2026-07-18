//! Purpose:
//! Defines `parse_url()` component selector constants (`PHP_URL_*`) and
//! `http_build_query()` encoding-mode constants (`PHP_QUERY_*`) exposed by elephc.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//!
//! Key details:
//! - Values are php-verified with `php -n -r 'var_dump(PHP_URL_SCHEME, ...);'`
//!   (PHP 8.5.6 local) and are stable across PHP versions.

/// Tuple of `(name, value)` pairs for `PHP_URL_*` (`parse_url()` component selector)
/// and `PHP_QUERY_*` (`http_build_query()` encoding mode) integer constants.
pub(crate) const URL_INT_CONSTANTS: &[(&str, i64)] = &[
    ("PHP_URL_SCHEME", 0),
    ("PHP_URL_HOST", 1),
    ("PHP_URL_PORT", 2),
    ("PHP_URL_USER", 3),
    ("PHP_URL_PASS", 4),
    ("PHP_URL_PATH", 5),
    ("PHP_URL_QUERY", 6),
    ("PHP_URL_FRAGMENT", 7),
    ("PHP_QUERY_RFC1738", 1),
    ("PHP_QUERY_RFC3986", 2),
];

#[cfg(test)]
mod tests {
    use super::URL_INT_CONSTANTS;

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        URL_INT_CONSTANTS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Verifies every `PHP_URL_*` component selector matches PHP's `parse_url()` ids.
    #[test]
    fn test_php_url_component_values_match_php() {
        assert_eq!(value_of("PHP_URL_SCHEME"), 0);
        assert_eq!(value_of("PHP_URL_HOST"), 1);
        assert_eq!(value_of("PHP_URL_PORT"), 2);
        assert_eq!(value_of("PHP_URL_USER"), 3);
        assert_eq!(value_of("PHP_URL_PASS"), 4);
        assert_eq!(value_of("PHP_URL_PATH"), 5);
        assert_eq!(value_of("PHP_URL_QUERY"), 6);
        assert_eq!(value_of("PHP_URL_FRAGMENT"), 7);
    }

    /// Verifies the `http_build_query()` encoding-mode constants match PHP.
    #[test]
    fn test_php_query_encoding_values_match_php() {
        assert_eq!(value_of("PHP_QUERY_RFC1738"), 1);
        assert_eq!(value_of("PHP_QUERY_RFC3986"), 2);
    }

    /// Asserts no duplicate names exist in `URL_INT_CONSTANTS`.
    #[test]
    fn test_url_constants_have_unique_names() {
        let mut names: Vec<&str> = URL_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate url constant name");
    }
}
