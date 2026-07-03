//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of indexed array aggregates, including reverse, sum, and product.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Compiles `array_reverse($a)` and verifies the returned array has elements in reverse order.
/// Input: `[3, 1, 2]` → reversed to `[2, 1, 3]` → access via indices 0,1,2 yields `"213"`.
#[test]
fn test_array_reverse() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
$b = array_reverse($a);
echo $b[0] . $b[1] . $b[2];
"#,
    );
    assert_eq!(out, "213");
}

/// Compiles `array_reverse($a, true)` — the 2-argument form preserves keys. Input
/// `[1, 2, 3]` with `preserve_keys=true` keeps keys 0,1,2 in reverse value order, so
/// accessing by key 0,1,2 yields the reversed values `"321"`.
#[test]
fn test_array_reverse_preserve_keys() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
$b = array_reverse($a, true);
echo $b[0] . $b[1] . $b[2];
"#,
    );
    assert_eq!(out, "321");
}

/// Compiles `array_sum($a)` and verifies integer summation of all elements.
/// Input: `[10, 20, 30]` → sum = 60.
#[test]
fn test_array_sum() {
    let out = compile_and_run(
        r#"<?php
$a = [10, 20, 30];
echo array_sum($a);
"#,
    );
    assert_eq!(out, "60");
}

/// Compiles `array_product($a)` and verifies integer multiplication of all elements.
/// Input: `[2, 3, 4]` → product = 24.
#[test]
fn test_array_product() {
    let out = compile_and_run(
        r#"<?php
$a = [2, 3, 4];
echo array_product($a);
"#,
    );
    assert_eq!(out, "24");
}

/// Verifies `array_reverse()` accepts a gradual `Mixed` operand (a `mixed` parameter holding an
/// array) and reverses the underlying array through the assert-array boundary path.
/// Input: `[1, 2, 3]` boxed as `Mixed` → reversed → `"3,2,1"`.
#[test]
fn test_array_reverse_gradual_mixed_operand() {
    let out = compile_and_run(
        r#"<?php
function rev(mixed $u) { return implode(",", array_reverse($u)); }
echo rev([1, 2, 3]);
"#,
    );
    assert_eq!(out, "3,2,1");
}

/// Verifies `array_reverse()` on a runtime `false` gradual operand fatals with a `TypeError`,
/// matching PHP 8 (`array_reverse(false)` is a `TypeError`) instead of silently yielding `[]`.
#[test]
fn test_array_reverse_gradual_false_operand_fatals() {
    let err = compile_and_run_expect_failure(
        r#"<?php
function rev(mixed $u) { return array_reverse($u); }
$r = rev(false);
echo "unreached";
"#,
    );
    assert!(
        err.contains("array builtin argument must be of type array"),
        "unexpected stderr: {err}"
    );
}
