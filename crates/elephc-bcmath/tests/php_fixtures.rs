//! Purpose:
//! Table-driven parity fixtures for PHP 8.4 BCMath procedural functions.
//!
//! Called from:
//! - `cargo test --manifest-path crates/elephc-bcmath/Cargo.toml`.
//!
//! Key details:
//! - Expected strings were captured from PHP 8.4.19 with the bcmath extension.
//! - Tests cover truncation, padding, signs, global scale, rounding, and typed failures.

use elephc_bcmath::{
    bc_add, bc_ceil, bc_comp, bc_div, bc_divmod, bc_floor, bc_mod, bc_mul, bc_pow,
    bc_powmod, bc_round, bc_sqrt, bc_sub, get_scale, set_scale, BcError,
};

/// Verifies core arithmetic output strings against PHP 8.4 fixtures.
#[test]
fn arithmetic_fixtures_match_php() {
    assert_eq!(bc_add("1.234", "5", Some(0)).expect("add"), "6");
    assert_eq!(bc_add("1.234", "5", Some(4)).expect("add"), "6.2340");
    assert_eq!(bc_sub("1", "2.5", Some(2)).expect("sub"), "-1.50");
    assert_eq!(bc_mul("1.20", "1.20", Some(2)).expect("mul"), "1.44");
    assert_eq!(bc_div("105", "6.55957", Some(3)).expect("div"), "16.007");
    assert_eq!(bc_mod("5.7", "1.3", Some(1)).expect("mod"), "0.5");
    assert_eq!(bc_comp("1", "2", Some(0)).expect("comp"), -1);
}

/// Verifies powers, square roots, and integer-boundary functions against PHP 8.4.
#[test]
fn advanced_arithmetic_fixtures_match_php() {
    assert_eq!(bc_sqrt("2", Some(3)).expect("sqrt"), "1.414");
    assert_eq!(bc_pow("4.2", "3", Some(2)).expect("pow"), "74.08");
    assert_eq!(bc_pow("5", "2", Some(2)).expect("pow"), "25.00");
    assert_eq!(bc_powmod("5", "2", "7", Some(3)).expect("powmod"), "4.000");
    assert_eq!(bc_ceil("1.1").expect("ceil"), "2");
    assert_eq!(bc_floor("1.9").expect("floor"), "1");
    assert_eq!(bc_floor("-1.1").expect("floor"), "-2");
}

/// Verifies signed quotient/remainder pairs against the PHP manual sign table.
#[test]
fn divmod_fixtures_match_php() {
    let cases = [
        ("5", "3", ("1", "2")),
        ("5", "-3", ("-1", "2")),
        ("-5", "3", ("-1", "-2")),
        ("-5", "-3", ("1", "-2")),
    ];
    for (left, right, expected) in cases {
        let actual = bc_divmod(left, right, Some(0)).expect("divmod");
        assert_eq!(actual, (expected.0.to_string(), expected.1.to_string()));
    }
    assert_eq!(
        bc_divmod("5.7", "1.3", Some(1)).expect("divmod"),
        ("4".to_string(), "0.5".to_string())
    );
}

/// Verifies representative rounding fixtures including negative precision.
#[test]
fn rounding_fixtures_match_php() {
    assert_eq!(bc_round("3.5", 0, 1).expect("round"), "4");
    assert_eq!(bc_round("5.045", 2, 1).expect("round"), "5.05");
    assert_eq!(bc_round("345", -2, 1).expect("round"), "300");
}

/// Verifies global scale affects omitted-scale calls and explicit zero overrides it.
#[test]
fn process_scale_fixture_matches_php() {
    let saved = get_scale();
    set_scale(4).expect("set scale");
    assert_eq!(bc_mul("1", "1", None).expect("mul"), "1.0000");
    assert_eq!(bc_mul("1", "1", Some(0)).expect("mul"), "1");
    set_scale(i64::from(saved)).expect("restore scale");
}

/// Verifies malformed values, invalid scales, and division failures retain typed errors.
#[test]
fn failure_fixtures_match_php_categories() {
    assert!(matches!(
        bc_add("1e2", "1", Some(0)),
        Err(BcError::Malformed { .. })
    ));
    assert!(matches!(
        bc_add("1", "2", Some(-1)),
        Err(BcError::ScaleRange { .. })
    ));
    assert!(matches!(
        bc_div("1", "0", Some(0)),
        Err(BcError::DivisionByZero { .. })
    ));
    assert!(matches!(
        bc_pow("0", "-1", Some(0)),
        Err(BcError::DivisionByZero { .. })
    ));
    assert!(matches!(
        bc_powmod("2", "-1", "3", Some(0)),
        Err(BcError::PowModNegativeExponent)
    ));
    assert!(matches!(bc_round("1", 0, 9), Err(BcError::RoundMode)));
    assert_eq!(
        BcError::RoundMode.php_message(),
        "bcround(): Argument #3 ($mode) must be a valid rounding mode (RoundingMode::*)"
    );
}
