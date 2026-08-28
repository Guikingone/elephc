//! Purpose:
//! Integration tests for PHP 8.4's `round($num, $precision, $mode)` third argument and the
//! `PHP_ROUND_HALF_*` predefined constants.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected string is real `LC_ALL=C php` 8.4.20 output for the same fixture.
//! - `$argc`-derived values keep the call runtime-unknown so the AST/EIR folders cannot
//!   replace the call with a literal and hide the `__rt_round_mode` lowering.

use crate::support::*;

/// Verifies the four `PHP_ROUND_HALF_*` constants resolve to php-src's integer values.
#[test]
fn test_round_half_constants_values() {
    let out = compile_and_run(
        r#"<?php echo PHP_ROUND_HALF_UP, " ", PHP_ROUND_HALF_DOWN, " ", PHP_ROUND_HALF_EVEN, " ", PHP_ROUND_HALF_ODD;"#,
    );
    assert_eq!(out, "1 2 3 4");
}

/// Verifies each rounding mode breaks a positive `.5` tie the way php-src does.
#[test]
fn test_round_modes_positive_tie() {
    let out = compile_and_run(
        r#"<?php
echo round(2.5, 0, PHP_ROUND_HALF_UP), "|", round(2.5, 0, PHP_ROUND_HALF_DOWN), "|", round(2.5, 0, PHP_ROUND_HALF_EVEN), "|", round(2.5, 0, PHP_ROUND_HALF_ODD);
"#,
    );
    assert_eq!(out, "3|2|2|3");
}

/// Verifies each rounding mode breaks a negative `.5` tie symmetrically around zero.
#[test]
fn test_round_modes_negative_tie() {
    let out = compile_and_run(
        r#"<?php
echo round(-2.5, 0, PHP_ROUND_HALF_UP), "|", round(-2.5, 0, PHP_ROUND_HALF_DOWN), "|", round(-2.5, 0, PHP_ROUND_HALF_EVEN), "|", round(-2.5, 0, PHP_ROUND_HALF_ODD);
"#,
    );
    assert_eq!(out, "-3|-2|-2|-3");
}

/// Verifies the precision path reproduces php-src's integral-part correction.
///
/// `1.005` and `0.285` are the classic cases where the binary double sits just below the
/// decimal tie; php-src recovers the tie before applying the mode, so `HALF_UP` rounds up.
#[test]
fn test_round_precision_matches_php_correction() {
    let out = compile_and_run(
        r#"<?php
echo round(1.005, 2), "|", round(0.285, 2), "|", round(3.555, 2), "|", round(1.55, 1, PHP_ROUND_HALF_EVEN), "|", round(1.45, 1, PHP_ROUND_HALF_ODD);
"#,
    );
    assert_eq!(out, "1.01|0.29|3.56|1.6|1.5");
}

/// Verifies `round()` with a mode resolves case-insensitively, namespaced, and by named argument.
#[test]
fn test_round_mode_case_insensitive_namespaced_and_named_args() {
    let out = compile_and_run(
        r#"<?php
$v = 2.5 + ($argc - 1);
echo ROUND($v, 0, PHP_ROUND_HALF_EVEN), "|", \round($v, mode: PHP_ROUND_HALF_ODD), "|", round(num: $v, precision: 0, mode: PHP_ROUND_HALF_DOWN);
"#,
    );
    assert_eq!(out, "2|3|2");
}

/// Verifies negative precision, signed zero, and a negative even tie stay php-identical.
#[test]
fn test_round_mode_negative_precision_and_signed_zero() {
    let out = compile_and_run(
        r#"<?php
echo round(-0.4, 0, PHP_ROUND_HALF_UP), "|", round(1234.5678, -2, PHP_ROUND_HALF_UP), "|", round(0, 0, PHP_ROUND_HALF_EVEN), "|", round(-1.5, 0, PHP_ROUND_HALF_EVEN);
"#,
    );
    assert_eq!(out, "-0|1200|0|-2");
}

/// Verifies a runtime `$mode` outside php-src's enumeration raises the catchable `ValueError`.
#[test]
fn test_round_invalid_mode_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
$m = 9 + ($argc - 1);
try { echo round(1.5, 0, $m); } catch (\ValueError $e) { echo get_class($e), "|", $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "ValueError|round(): Argument #3 ($mode) must be a valid rounding mode (RoundingMode::*)"
    );
}

/// Verifies `count()`'s `$mode` argument, including `COUNT_RECURSIVE` over a flat array.
///
/// A flat array's recursive count equals its normal count, so all four spellings agree.
#[test]
fn test_count_mode_flat_array() {
    let out = compile_and_run(
        r#"<?php
$a = ["x", "y", "z"];
echo count($a, COUNT_RECURSIVE), "|", count($a, COUNT_NORMAL), "|", COUNT($a, mode: COUNT_RECURSIVE), "|", \count($a, 1);
"#,
    );
    assert_eq!(out, "3|3|3|3");
}

/// Verifies a runtime `$mode` outside `COUNT_NORMAL`/`COUNT_RECURSIVE` raises `ValueError`.
#[test]
fn test_count_invalid_mode_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
$m = 5 + ($argc - 1);
try { echo count([1, 2], $m); } catch (\ValueError $e) { echo get_class($e), "|", $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "ValueError|count(): Argument #2 ($mode) must be either COUNT_NORMAL or COUNT_RECURSIVE"
    );
}

/// Verifies `COUNT_NORMAL` and `COUNT_RECURSIVE` carry php-src's integer values.
#[test]
fn test_count_mode_constants_values() {
    let out = compile_and_run(r#"<?php echo COUNT_NORMAL, " ", COUNT_RECURSIVE;"#);
    assert_eq!(out, "0 1");
}
