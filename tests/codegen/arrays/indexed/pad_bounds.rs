//! Purpose:
//! Regression tests for PHP's `array_pad()` `$length` bounds: the sign/magnitude matrix that
//! decides between padding left, padding right, and copying, and the oversized magnitudes that
//! reference PHP rejects with a catchable `ValueError` instead of building an array.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected string in this file is verbatim `LC_ALL=C php` 8.4 output for the same fixture.
//! - The matrices assert `count()` on every result, so a runtime helper that publishes a negative
//!   logical length in the array header can never pass again.
//! - `PHP_INT_MIN` is the sharpest case: its magnitude is not representable, so the pre-fix helpers
//!   negated it straight back to a negative "length" instead of reporting an error.
//! - The scalar fixtures exercise `__rt_array_pad`; the `[[1], [2]]` fixtures exercise
//!   `__rt_array_pad_refcounted`. Both take `$length` in the second ABI argument register, which is
//!   where the shared lowering guard inspects it.
//! - Lengths stay literal or plainly int-typed: an arithmetic expression over `$argc` produces a
//!   boxed `Mixed` local, and passing one of those to any runtime builtin is a separate,
//!   pre-existing lowering gap that would mask the behavior under test.
//! - The maximum accepted magnitude (`1073741824`) is deliberately never executed: reference PHP
//!   accepts it and then fails on memory, and so does elephc, but the attempt would ask this
//!   process for an 8 GiB payload.

use super::*;

/// php-src's verbatim `ValueError` message for an `array_pad()` `$length` past the maximum
/// allowed array size, repeated once per rejected fixture length.
const PAD_LENGTH_VALUE_ERROR: &str =
    "ValueError: array_pad(): Argument #2 ($length) must not exceed the maximum allowed array size";

/// Verifies the full sign/magnitude matrix for a scalar source array: a magnitude larger than the
/// source pads (right for a positive `$length`, left for a negative one), a magnitude at or below
/// the source length copies, and the source array itself is never mutated.
#[test]
fn test_array_pad_length_matrix_matches_php() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2, 3];
foreach ([7, 4, 3, 2, 0, -2, -3, -4, -7] as $n) {
    $r = array_pad($a, $n, 9);
    echo $n, " ", count($r), " ", implode(",", $r), "\n";
}
echo count($a), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"7 7 1,2,3,9,9,9,9
4 4 1,2,3,9
3 3 1,2,3
2 3 1,2,3
0 3 1,2,3
-2 3 1,2,3
-3 3 1,2,3
-4 4 9,1,2,3
-7 7 9,9,9,9,1,2,3
3
"#
    );
}

/// Verifies the same matrix on the refcounted helper, whose destination length is published by the
/// repeated append helper rather than by a header store, so a negative pad count would loop or
/// under-fill instead of over-reporting `count()`.
#[test]
fn test_array_pad_refcounted_length_matrix_matches_php() {
    let out = compile_and_run(
        r#"<?php
$a = [[1], [2]];
$right = array_pad($a, 5, [9]);
echo count($right), " ", $right[0][0], $right[1][0], $right[2][0], $right[3][0], $right[4][0], "\n";
$left = array_pad($a, -5, [9]);
echo count($left), " ", $left[0][0], $left[1][0], $left[2][0], $left[3][0], $left[4][0], "\n";
$same = array_pad($a, 2, [9]);
echo count($same), " ", $same[0][0], $same[1][0], "\n";
$short = array_pad($a, -1, [9]);
echo count($short), " ", $short[0][0], $short[1][0], "\n";
$zero = array_pad($a, 0, [9]);
echo count($zero), " ", $zero[0][0], $zero[1][0], "\n";
echo count($a), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"5 12999
5 99912
2 12
2 12
2 12
2
"#
    );
}

/// Regression: a `$length` whose magnitude exceeds the maximum allowed array size must raise
/// reference PHP's catchable `ValueError`, not build an array.
///
/// `array_pad([1, 2], -2000000000, 0)` used to reach `__rt_array_pad` with an unbounded
/// `abs($length)`, and `array_pad([1, 2], PHP_INT_MIN, 0)` used to negate a magnitude that does not
/// exist and silently answer with a copy of the source array. Both are `ValueError` in reference
/// PHP, and both sides of the boundary (`±1073741825`) are asserted here.
#[test]
fn test_array_pad_oversized_length_throws_catchable_value_error() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2];
foreach ([1073741825, -1073741825, 2000000000, -2000000000, PHP_INT_MAX, PHP_INT_MIN] as $n) {
    try {
        $r = array_pad($a, $n, 0);
        echo "no throw ", count($r), "\n";
    } catch (ValueError $e) {
        echo get_class($e), ": ", $e->getMessage(), "\n";
    }
}
echo count($a), "\n";
"#,
    );
    assert_eq!(
        out,
        format!("{PAD_LENGTH_VALUE_ERROR}\n").repeat(6) + "2\n"
    );
}

/// Regression: the refcounted pad helper is guarded by the same bound, so an oversized `$length`
/// over a nested-array source throws instead of asking the allocator for an impossible payload.
#[test]
fn test_array_pad_refcounted_oversized_length_throws_catchable_value_error() {
    let out = compile_and_run(
        r#"<?php
$b = [[1], [2]];
try {
    $r = array_pad($b, PHP_INT_MIN, [0]);
    echo "no throw ", count($r), "\n";
} catch (ValueError $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
try {
    $r = array_pad($b, -2000000000, [0]);
    echo "no throw ", count($r), "\n";
} catch (ValueError $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
echo count($b), "\n";
"#,
    );
    assert_eq!(
        out,
        format!("{PAD_LENGTH_VALUE_ERROR}\n").repeat(2) + "2\n"
    );
}

/// Verifies the guard survives dead-code elimination and the try-prefix hoist: a discarded
/// `array_pad()` result still throws, and the throw still lands inside the enclosing `try`.
///
/// `RuntimeFnId::ArrayPad` used to report no effects at all, so the optimizer treated the call as a
/// removable pure expression and hoisted it out of the `try` body it belongs to.
#[test]
fn test_array_pad_oversized_length_throws_from_discarded_result_inside_try() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 2];
try {
    array_pad($a, PHP_INT_MIN, 0);
    echo "no throw\n";
} catch (Throwable $e) {
    echo get_class($e), ": ", $e->getMessage(), "\n";
}
echo "after\n";
"#,
    );
    assert_eq!(out, format!("{PAD_LENGTH_VALUE_ERROR}\nafter\n"));
}

/// Verifies an uncaught oversized `$length` terminates with the PHP-shaped uncaught diagnostic
/// rather than an allocator fatal or a silent out-of-bounds array.
#[test]
fn test_array_pad_oversized_length_uncaught_is_fatal() {
    let err = compile_and_run_expect_failure(
        "<?php $a = array_pad([1, 2], PHP_INT_MIN, 0); echo count($a);",
    );
    assert!(
        err.contains(&format!("Uncaught {PAD_LENGTH_VALUE_ERROR}")),
        "{}",
        err
    );
}

/// Verifies the linux-x86_64 runtime carries the same unrepresentable-magnitude clamp as the ARM64
/// runtime. The x86_64 code cannot be executed from an aarch64 host, so this asserts on the emitted
/// assembly text: both pad helpers must negate the requested size and then force a still-negative
/// result to zero before it can become a capacity, a pad count, or a header length.
#[test]
fn test_x86_64_runtime_array_pad_clamps_unrepresentable_magnitude() {
    let target = Target::parse("linux-x86_64").expect("linux-x86_64 is a supported target");
    let runtime_asm = elephc::codegen::generate_runtime(8_388_608, target);
    for label in ["__rt_array_pad", "__rt_array_pad_refcounted"] {
        let marker = format!("{label}:");
        let start = runtime_asm
            .find(&marker)
            .unwrap_or_else(|| panic!("missing assembly label {label}"));
        let rest = &runtime_asm[start..];
        let end = rest.find("\n\n").unwrap_or(rest.len());
        let body = &rest[..end];
        for expected in ["neg r11", "test r11, r11", "cmovs r11, rax"] {
            assert!(
                body.contains(expected),
                "x86_64 {label} missing {expected}: {body}"
            );
        }
    }
}
