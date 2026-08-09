//! Purpose:
//! Integration regression tests asserting that constant folding of literal operands produces
//! the same program output PHP 8.4 produces: integer-exact comparisons, `switch` loose
//! equality, overflowing integer arithmetic, associative array-key normalization, signed-zero
//! propagation, and string casts.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected string was captured from `php <fixture>` on PHP 8.4.20 before the test was
//!   written; a fold that disagrees with these strings is a silent miscompilation.
//! - Operands are literals on purpose — the fold is the thing under test. Where a construct has
//!   to survive folding, `$argc` supplies a runtime-unknown value.

use super::*;

/// Verifies integer comparisons above 2^53 stay exact instead of routing through `f64`.
///
/// php -r 'var_dump(9223372036854775806 < 9223372036854775807, 9223372036854775806 <=> 9223372036854775807, 9223372036854775806 == 9223372036854775807);'
#[test]
fn test_fold_large_integer_comparisons_match_php() {
    let out = compile_and_run(
        r#"<?php
var_dump(9223372036854775806 < 9223372036854775807);
var_dump(9223372036854775806 <=> 9223372036854775807);
var_dump(9223372036854775806 == 9223372036854775807);
var_dump(9223372036854775806 > 9223372036854775807);
"#,
    );
    assert_eq!(out, "bool(true)\nint(-1)\nbool(false)\nbool(false)\n");
}

/// Verifies an integer compared against a numeric string uses PHP 8's exact integer rule.
///
/// php -r 'var_dump(9223372036854775806 == "9223372036854775807", 9223372036854775807 == "9223372036854775807", "9223372036854775806" == "9223372036854775807");'
#[test]
fn test_fold_integer_versus_numeric_string_matches_php() {
    let out = compile_and_run(
        r#"<?php
var_dump(9223372036854775806 == "9223372036854775807");
var_dump(9223372036854775807 == "9223372036854775807");
var_dump("9223372036854775806" == "9223372036854775807");
var_dump("0e1" == "0e2");
var_dump("1e3" == "1000");
var_dump("abc" == 0);
var_dump("1abc" == 1);
var_dump(" 1" == 1);
var_dump("1 " == 1);
var_dump("" == 0);
var_dump(null == "");
var_dump(null == "0");
var_dump("0x1A" == 26);
var_dump("1_000" == 1000);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(true)\n",
            "bool(false)\n",
            "bool(false)\n",
            "bool(false)\n",
        )
    );
}

/// Verifies literal string comparisons fold to PHP's byte/numeric-string result.
///
/// php -r 'var_dump("a" <=> "b", "a" < "b", "B" < "a", "10" > "9", "abc" == "abc");'
///
/// These compile only because the fold answers them before type checking: the checker still
/// rejects `<`, `<=`, `>`, `>=` and `<=>` on non-constant string operands (issue #507, pinned
/// by `error_tests::misc::syntax_misc::test_error_spaceship_string`). The fold is the
/// PHP-correct half of that gap, so it must not regress while the checker catches up.
#[test]
fn test_fold_literal_string_comparisons_match_php() {
    let out = compile_and_run(
        r#"<?php
var_dump("a" <=> "b");
var_dump("a" < "b");
var_dump("B" < "a");
var_dump("10" > "9");
var_dump("abc" == "abc");
"#,
    );
    assert_eq!(
        out,
        "int(-1)\nbool(true)\nbool(true)\nbool(true)\nbool(true)\n"
    );
}

/// Verifies `switch` case selection over a constant subject uses PHP's `==`.
///
/// `switch (2) { case true: }` selects the case because `2 == true` is `(bool) 2`; the fold
/// used to coerce `true` to the integer `1` and fall through to `default`.
#[test]
fn test_fold_switch_case_loose_comparison_matches_php() {
    let out = compile_and_run(
        r#"<?php
switch (2) { case true: echo "A"; break; default: echo "B"; }
switch (-1) { case true: echo "A"; break; default: echo "B"; }
switch (0) { case false: echo "A"; break; default: echo "B"; }
switch (0.0) { case false: echo "A"; break; default: echo "B"; }
switch (0) { case null: echo "A"; break; default: echo "B"; }
switch (1) { case "abc": echo "A"; break; default: echo "B"; }
switch ("foo") { case 0: echo "A"; break; default: echo "B"; }
switch (0) { case "foo": echo "A"; break; default: echo "B"; }
switch ("1") { case 1: echo "A"; break; default: echo "B"; }
switch (1.5) { case "1.5": echo "A"; break; default: echo "B"; }
switch ("abc") { case "ABC": echo "A"; break; default: echo "B"; }
"#,
    );
    assert_eq!(out, "AAAAABBBAAB");
}

/// Verifies integer arithmetic folds keep PHP's types and overflow behavior.
///
/// `PHP_INT_MIN % -1` used to panic the compiler outright; `6 / 3` and `2 ** 3` used to fold
/// to floats, and `1 << 64` / `-PHP_INT_MIN` reached a wrapping runtime instead of folding.
#[test]
fn test_fold_integer_arithmetic_overflow_matches_php() {
    let out = compile_and_run(
        r#"<?php
var_dump(PHP_INT_MIN % -1);
var_dump(6 / 3);
var_dump(7 / 2);
var_dump(2 ** 3);
var_dump(2 ** -1);
var_dump(1 << 63);
var_dump(1 << 64);
var_dump(-1 >> 64);
var_dump(-1 >> 63);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "int(0)\n",
            "int(2)\n",
            "float(3.5)\n",
            "int(8)\n",
            "float(0.5)\n",
            "int(-9223372036854775808)\n",
            "int(0)\n",
            "int(-1)\n",
            "int(-1)\n",
        )
    );
}

/// Verifies `intdiv(PHP_INT_MIN, -1)` still raises instead of being folded.
///
/// php -r 'var_dump(intdiv(PHP_INT_MIN, -1));' throws `ArithmeticError`, so the compiled
/// program must fail rather than print a wrapped integer.
#[test]
fn test_intdiv_int_min_by_minus_one_still_raises() {
    let output = compile_and_run_expect_failure(
        "<?php var_dump(intdiv(PHP_INT_MIN, -1)); echo \"unreachable\";",
    );
    assert!(
        !output.contains("unreachable"),
        "intdiv overflow must abort before the following statement:\n{output}"
    );
}

/// Verifies associative array-literal access normalizes keys like PHP's hash table.
///
/// php -r 'var_dump([0 => "a", false => "b"][0], ["1" => "a", 1 => "b"]["1"], [null => "a", "" => "b"][null]);'
#[test]
fn test_fold_assoc_array_key_normalization_matches_php() {
    let out = compile_and_run(
        r#"<?php
var_dump([0 => "a", false => "b"][0]);
var_dump(["1" => "a", 1 => "b"]["1"]);
var_dump([null => "a", "" => "b"][null]);
var_dump([true => "a", 1 => "b"][true]);
var_dump(["01" => "a", 1 => "b"]["01"]);
var_dump([" 1" => "a", 1 => "b"][" 1"]);
var_dump([false => "a", true => "b", null => "c"][0]);
var_dump([false => "a", true => "b", null => "c"][1]);
var_dump([false => "a", true => "b", null => "c"][""]);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "string(1) \"b\"\n",
            "string(1) \"b\"\n",
            "string(1) \"b\"\n",
            "string(1) \"b\"\n",
            "string(1) \"a\"\n",
            "string(1) \"a\"\n",
            "string(1) \"a\"\n",
            "string(1) \"b\"\n",
            "string(1) \"c\"\n",
        )
    );
}

/// Verifies constant propagation keeps `-0.0` distinct from `0.0`.
///
/// php -r '$c = $argc > 1000; $x = $c ? 0.0 : -0.0; echo $x;' prints `-0`; merging the ternary
/// arms into one constant printed `0`.
#[test]
fn test_signed_zero_survives_constant_propagation() {
    let out = compile_and_run(
        r#"<?php
$c = $argc > 1000;
$x = $c ? 0.0 : -0.0;
echo $x, "\n";
var_dump($x);
"#,
    );
    assert_eq!(out, "-0\nfloat(-0)\n");
}

/// Verifies DCE guard state treats `0.0` and `-0.0` as the same value under `===`.
///
/// php -r 'function probe(float $x): void { if ($x === 0.0) { if (-0.0 === $x) { echo "A"; } else { echo "B"; } } } probe($argc > 1000 ? 1.0 : -0.0);' prints `A`.
#[test]
fn test_signed_zero_guard_does_not_prune_live_branch() {
    let out = compile_and_run(
        r#"<?php
function probe(float $x): void {
    if ($x === 0.0) {
        if (-0.0 === $x) { echo "A"; } else { echo "B"; }
    }
}
probe($argc > 1000 ? 1.0 : -0.0);
"#,
    );
    assert_eq!(out, "A");
}

/// Verifies `(float)` and `(int)` casts of literal strings use PHP's numeric grammar.
///
/// PHP has no `INF`/`NAN`/hexadecimal numeric-string forms, so `(float) "INF"` is `0`; the fold
/// used Rust's `str::parse::<f64>()`, which accepted them.
#[test]
fn test_fold_string_casts_match_php() {
    let out = compile_and_run(
        r#"<?php
var_dump((float)"INF", (float)"nan", (float)"infinity", (float)"-INF");
var_dump((float)"1e3", (float)".5", (float)"5.", (float)"+.5e-2");
var_dump((int)"1e3", (int)".5", (int)"5.", (int)"12abc");
var_dump((int)"9223372036854775808", (int)"1e400");
"#,
    );
    assert_eq!(
        out,
        concat!(
            "float(0)\nfloat(0)\nfloat(0)\nfloat(0)\n",
            "float(1000)\nfloat(0.5)\nfloat(5)\nfloat(0.005)\n",
            "int(1000)\nint(0)\nint(5)\nint(12)\n",
            "int(9223372036854775807)\nint(0)\n",
        )
    );
}
