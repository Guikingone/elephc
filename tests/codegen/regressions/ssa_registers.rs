//! Purpose:
//! Regression tests for scalar locals promoted to SSA values and kept in registers: loop-carried
//! parallel moves, fused float compares, call arguments, immediates, and by-reference calls on
//! locals whose storage was widened to `mixed`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use super::*;

/// Rotating several loop-carried ints and swapping two floats each iteration exercises cyclic block-argument copies.
#[test]
fn test_loop_carried_rotation_is_a_parallel_move() {
    let out = compile_and_run(r#"<?php
function swaps(int $n): string {
    $a = 1; $b = 2; $c = 3; $x = 1.5; $y = -2.5;
    for ($i = 0; $i < $n; $i++) { $t = $a; $a = $b; $b = $c; $c = $t; $f = $x; $x = $y; $y = $f; }
    return "$a $b $c $x $y";
}
echo swaps(4), "|", swaps(5), "\n";
"#);
    assert_eq!(out, "2 3 1 1.5 -2.5|3 1 2 -2.5 1.5\n");
}

/// Float compares that feed a branch directly keep PHP's NaN semantics (every ordering is false, != is true).
#[test]
fn test_fused_float_compares_are_false_for_nan() {
    let out = compile_and_run(r#"<?php
function cmp(float $a, float $b): string {
    $s = "";
    if ($a < $b) { $s .= "lt "; } if ($a <= $b) { $s .= "le "; }
    if ($a > $b) { $s .= "gt "; } if ($a >= $b) { $s .= "ge "; }
    if ($a == $b) { $s .= "eq "; } if ($a != $b) { $s .= "ne "; }
    return $s;
}
echo cmp(1.0, 2.0), "|", cmp(2.0, 1.0), "|", cmp(1.0, 1.0), "|", cmp(NAN, 1.0), "|", cmp(1.0, NAN), "\n";
"#);
    assert_eq!(out, "lt le ne |gt ge ne |le ge eq |ne |ne \n");
}

/// Promoted scalar locals still reach by-value parameters directly while a by-reference argument keeps updating the caller.
#[test]
fn test_scalar_by_value_and_by_ref_args_in_promoted_loop() {
    let out = compile_and_run(r#"<?php
function bump(int &$x): void { $x = $x + 10; }
function twice(int $x, float $y): float { return $x * $y; }
function run(): string {
    $i = 0; $acc = 0.0; $steps = 0;
    while ($i < 50) { bump($i); $acc += twice($i, 0.5); $i = $i + 1; $steps = $steps + 1; }
    return "$i $acc $steps";
}
echo run(), "\n";
"#);
    assert_eq!(out, "55 80 5\n");
}

/// Integer constants outside the 32-bit immediate range are loaded in full for compares and arithmetic.
#[test]
fn test_wide_integer_constants_are_not_truncated_immediates() {
    let out = compile_and_run(r#"<?php
function big(int $n): string {
    $hits = 0; $k = 5000000000;
    for ($i = 4999999990; $i < $k; $i++) { if ($i * 3 > 14999999980) { $hits = $hits + 1; } }
    $m = $n * 6000000000;
    return "$hits $m " . ($n - 3000000000) . " " . ($n & 0xFFFFFFFFF);
}
echo big(2), "\n";
"#);
    assert_eq!(out, "6 12000000000 -2999999998 2\n");
}

/// Float operations whose result overwrites their right operand's register keep the right operand intact.
#[test]
fn test_float_ops_whose_destination_is_the_right_operand() {
    let out = compile_and_run(r#"<?php
function mix(float $x, float $y, int $n): string {
    for ($i = 0; $i < $n; $i++) { $x = $y - $x; $y = $x / $y + $y * $x; $x = $x * $x - $y; }
    return sprintf("%.6f %.6f", $x, $y);
}
echo mix(0.25, 0.75, 3), "\n";
"#);
    assert_eq!(out, "0.143661 15.683475\n");
}

/// A local widened to mixed (its ++ may overflow) passed to an `int &` parameter is updated by the callee, in main and function scope.
#[test]
fn test_mixed_widened_local_passed_to_int_reference_param() {
    let out = compile_and_run(r#"<?php
function inc(int &$x): void { $x++; }
$i = 0;
inc($i); echo $i, ",";
$i++; echo $i, ",";
inc($i); echo $i, "\n";
function f() { $j = 0; inc($j); echo $j, ","; $j++; echo $j, ","; while ($j < 5) { inc($j); $j++; } echo $j, "\n"; }
f();
"#);
    assert_eq!(out, "1,2,3\n1,2,6\n");
}
