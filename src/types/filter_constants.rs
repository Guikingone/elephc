//! Purpose:
//! Defines PHP `ext/filter` integer constants exposed by elephc.
//! Keeps the validation-filter identifiers, flags, and sanitize/default
//! filter ids in one source of truth for type checking and codegen.
//!
//! Called from:
//! - `crate::types::checker` when registering predefined constants.
//! - `crate::codegen::prescan` when materializing constant literal values.
//! - `crate::types::checker::builtins::system` and `crate::ir_lower::expr::filter`
//!   when statically resolving a literal `filter_var()` filter id or flags value.
//!
//! Key details:
//! - `FILTER_VALIDATE_BOOL` is PHP 8's alias of `FILTER_VALIDATE_BOOLEAN`; both
//!   resolve to the same ext/filter value (258).
//! - `FILTER_DEFAULT` = 516, matching `FILTER_UNSAFE_RAW` (both select the
//!   passthrough filter). Verified with `php -n -r 'var_dump(FILTER_DEFAULT);'`
//!   (PHP 8.5.6 local): the constant is 516, NOT 518 — the previous value here
//!   was a bug (regression-tested by `filter_default_is_516`).
//! - `FILTER_CALLBACK`, `FILTER_REQUIRE_SCALAR`, `FILTER_REQUIRE_ARRAY`,
//!   `FILTER_FORCE_ARRAY`, and every `FILTER_VALIDATE_*` id beyond
//!   INT/FLOAT/BOOL(EAN)/IP are registered for name resolution only; elephc's
//!   `filter_var()` lowering rejects them with a loud diagnostic (see
//!   `crate::types::checker::builtins::system`).
//! - `FILTER_FLAG_IPV4`/`FILTER_FLAG_IPV6` restrict `FILTER_VALIDATE_IP` to a
//!   single address family (both, or neither, accept either family — see
//!   `crate::ir_lower::expr::filter`). `FILTER_FLAG_NO_PRIV_RANGE`/
//!   `FILTER_FLAG_NO_RES_RANGE` are deliberately NOT registered: PHP's
//!   private/reserved-range matrix diverges from the naive RFC 1918/4193
//!   ranges (php-verified: `fe80::1` — a link-local IPv6 address — passes
//!   `FILTER_FLAG_NO_PRIV_RANGE`), so a partial implementation would silently
//!   mis-validate; `filter_var()` keeps these flags loud instead.

/// Tuple of `(name, value)` pairs for PHP `ext/filter` integer constants.
///
/// Both the modern `FILTER_VALIDATE_BOOL` name and the legacy
/// `FILTER_VALIDATE_BOOLEAN` alias are registered with the same value.
/// `FILTER_DEFAULT` is the default filter applied by `filter_var`.
pub(crate) const FILTER_INT_CONSTANTS: &[(&str, i64)] = &[
    ("FILTER_DEFAULT", 516),
    ("FILTER_VALIDATE_BOOL", 258),
    ("FILTER_VALIDATE_BOOLEAN", 258),
    ("FILTER_VALIDATE_INT", 257),
    ("FILTER_VALIDATE_FLOAT", 259),
    ("FILTER_VALIDATE_REGEXP", 272),
    ("FILTER_VALIDATE_URL", 273),
    ("FILTER_VALIDATE_EMAIL", 274),
    ("FILTER_VALIDATE_IP", 275),
    ("FILTER_VALIDATE_MAC", 276),
    ("FILTER_VALIDATE_DOMAIN", 277),
    ("FILTER_UNSAFE_RAW", 516),
    ("FILTER_CALLBACK", 1024),
    ("FILTER_NULL_ON_FAILURE", 134_217_728),
    ("FILTER_REQUIRE_SCALAR", 33_554_432),
    ("FILTER_REQUIRE_ARRAY", 16_777_216),
    ("FILTER_FORCE_ARRAY", 67_108_864),
    ("FILTER_FLAG_IPV4", 1_048_576),
    ("FILTER_FLAG_IPV6", 2_097_152),
];

#[cfg(test)]
mod tests {
    use super::FILTER_INT_CONSTANTS;

    /// Verifies the boolean-validation alias pair shares PHP's ext/filter value.
    #[test]
    fn test_filter_validate_bool_alias_shares_value() {
        let bool_value = FILTER_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "FILTER_VALIDATE_BOOL")
            .expect("FILTER_VALIDATE_BOOL defined")
            .1;
        let boolean_value = FILTER_INT_CONSTANTS
            .iter()
            .find(|(name, _)| *name == "FILTER_VALIDATE_BOOLEAN")
            .expect("FILTER_VALIDATE_BOOLEAN defined")
            .1;
        assert_eq!(bool_value, 258);
        assert_eq!(boolean_value, 258);
    }

    /// Asserts no duplicate names exist in `FILTER_INT_CONSTANTS`.
    #[test]
    fn test_filter_constants_have_unique_names() {
        let mut names: Vec<&str> = FILTER_INT_CONSTANTS.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        let len_before = names.len();
        names.dedup();
        assert_eq!(names.len(), len_before, "duplicate filter constant name");
    }

    /// Looks up a constant value by name, panicking if it is absent.
    fn value_of(name: &str) -> i64 {
        FILTER_INT_CONSTANTS
            .iter()
            .find(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("{name} defined"))
            .1
    }

    /// Regression test for the `FILTER_DEFAULT` bug fix: PHP reports 516 (the same
    /// value as `FILTER_UNSAFE_RAW`), not the previously-hardcoded 518. Verified with
    /// `php -n -r 'var_dump(FILTER_DEFAULT);'` (PHP 8.5.6 local).
    #[test]
    fn filter_default_is_516() {
        assert_eq!(value_of("FILTER_DEFAULT"), 516);
        assert_eq!(
            value_of("FILTER_DEFAULT"),
            value_of("FILTER_UNSAFE_RAW"),
            "FILTER_DEFAULT and FILTER_UNSAFE_RAW share the same ext/filter value"
        );
    }

    /// Verifies the validation-filter ids used by `filter_var()` core lowering,
    /// php-verified with `php -n -r 'var_dump(FILTER_VALIDATE_INT, ...);'`.
    #[test]
    fn test_validate_filter_ids_match_php() {
        assert_eq!(value_of("FILTER_VALIDATE_INT"), 257);
        assert_eq!(value_of("FILTER_VALIDATE_FLOAT"), 259);
        assert_eq!(value_of("FILTER_VALIDATE_REGEXP"), 272);
        assert_eq!(value_of("FILTER_VALIDATE_URL"), 273);
        assert_eq!(value_of("FILTER_VALIDATE_EMAIL"), 274);
        assert_eq!(value_of("FILTER_VALIDATE_IP"), 275);
        assert_eq!(value_of("FILTER_VALIDATE_MAC"), 276);
        assert_eq!(value_of("FILTER_VALIDATE_DOMAIN"), 277);
    }

    /// Verifies the filter flag bitmasks, php-verified with
    /// `php -n -r 'var_dump(FILTER_NULL_ON_FAILURE, ...);'`.
    #[test]
    fn test_filter_flags_match_php() {
        assert_eq!(value_of("FILTER_NULL_ON_FAILURE"), 134_217_728);
        assert_eq!(value_of("FILTER_REQUIRE_SCALAR"), 33_554_432);
        assert_eq!(value_of("FILTER_REQUIRE_ARRAY"), 16_777_216);
        assert_eq!(value_of("FILTER_FORCE_ARRAY"), 67_108_864);
        assert_eq!(value_of("FILTER_CALLBACK"), 1024);
    }

    /// Verifies the IP-family restriction flags, php-verified with
    /// `php -n -r 'var_dump(FILTER_FLAG_IPV4, FILTER_FLAG_IPV6);'`.
    #[test]
    fn test_filter_flag_ip_family_matches_php() {
        assert_eq!(value_of("FILTER_FLAG_IPV4"), 1_048_576);
        assert_eq!(value_of("FILTER_FLAG_IPV6"), 2_097_152);
    }
}
