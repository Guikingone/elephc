//! Purpose:
//! Regression tests for two general EIR lowering bugs:
//! 1. A `switch` over a string subject collapsed every case to `0 == 0`, so it
//!    always took the first case (the subject and each case were `coerce_to_int`'d,
//!    turning non-numeric strings into 0).
//! 2. An `int`/`bool` argument passed to a `float` parameter was deposited in an
//!    integer register and read back as garbage from a floating-point slot, because
//!    no int→float widening happened at the call boundary.
//! 3. Loose equality between a float and an int (`1.5 == 1`, and `switch (1.5)`)
//!    either failed to compile (`loose_eq for PHP types Float and Int` was an
//!    unsupported backend feature) or truncated the float subject to int in the
//!    dynamic switch dispatch, so `switch (1.5) { case 1.5; }` wrongly matched
//!    `case 1`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Bug 1 fix: the dynamic switch dispatch compares with PHP loose equality
//!   (`Op::LooseEq`) for string subjects/cases; the integer jump table is reserved
//!   for genuinely integer-typed subjects.
//! - Bug 2 fix: `coerce_operands_to_params` widens int/bool operands bound to pure
//!   `float` parameters before the call is emitted.
//! - Bug 3 fix: `lower_loose_eq` promotes a float-vs-int pair to float and compares
//!   numerically, and the dynamic switch routes float/numeric pairs through
//!   `Op::LooseEq` instead of the int jump path. These tests cover statically-typed
//!   float operands; an untyped (`Mixed`) float subject is a separate, broader
//!   loose-equality limitation (issue #397) and is intentionally not asserted here.

use crate::support::*;

/// A `switch` over a string matches the correct case rather than always the first.
#[test]
fn test_string_switch_matches_correct_case() {
    let out = compile_and_run(
        r#"<?php
function classify(string $s): int {
    switch ($s) {
        case "black": return 10;
        case "white": return 20;
        case "red":   return 30;
        default:      return 99;
    }
}
echo classify("black"), classify("white"), classify("red"), classify("green");
"#,
    );
    assert_eq!(out, "10203099");
}

/// String switches honor comma-separated case labels and PHP fallthrough.
#[test]
fn test_string_switch_fallthrough_and_multilabel() {
    let out = compile_and_run(
        r#"<?php
function grp(string $s): string {
    switch ($s) {
        case "a":
        case "b": return "ab";
        case "c": return "c";
        default:  return "?";
    }
}
echo grp("a"), grp("b"), grp("c"), grp("z");
"#,
    );
    assert_eq!(out, "ababc?");
}

/// A string switch with a non-default first case still falls through to default
/// when nothing matches (proves the subject is not silently coerced to the first).
#[test]
fn test_string_switch_default_when_no_match() {
    let out = compile_and_run(
        r#"<?php
$s = "zzz";
switch ($s) {
    case "one": echo "1"; break;
    case "two": echo "2"; break;
    default:    echo "D"; break;
}
"#,
    );
    assert_eq!(out, "D");
}

/// The integer jump-table path is preserved for integer-typed subjects.
#[test]
fn test_int_switch_jump_table_still_works() {
    let out = compile_and_run(
        r#"<?php
function isw(int $n): string {
    switch ($n) {
        case 1: return "one";
        case 2: return "two";
        default: return "many";
    }
}
echo isw(1), isw(2), isw(9);
"#,
    );
    assert_eq!(out, "onetwomany");
}

/// An int argument passed to a `float` parameter is widened, not read as garbage.
#[test]
fn test_int_arg_to_float_param_widens() {
    let out = compile_and_run(
        r#"<?php
function takesFloat(float $x): float { return $x + 0.5; }
echo takesFloat(3), " ", takesFloat(10), " ", takesFloat(3.0);
"#,
    );
    assert_eq!(out, "3.5 10.5 3.5");
}

/// Int→float widening applies to instance-method parameters too.
#[test]
fn test_int_arg_to_float_method_param_widens() {
    let out = compile_and_run(
        r#"<?php
class Calc {
    public function scale(float $f): float { return $f * 2.0; }
}
$c = new Calc();
echo $c->scale(5);
"#,
    );
    assert_eq!(out, "10");
}

/// Int defaults and named int arguments bound to float parameters widen correctly.
#[test]
fn test_int_default_and_named_float_params_widen() {
    let out = compile_and_run(
        r#"<?php
function withDefault(float $x = 7): float { return $x; }
function named(float $a, float $b): float { return $a - $b; }
echo withDefault(), " ", named(b: 2, a: 10);
"#,
    );
    assert_eq!(out, "7 8");
}

/// A bool argument bound to a float parameter widens to 1.0 / 0.0 per PHP.
#[test]
fn test_bool_arg_to_float_param_widens() {
    let out = compile_and_run(
        r#"<?php
function f(float $x): float { return $x + 0.25; }
echo f(true), " ", f(false);
"#,
    );
    assert_eq!(out, "1.25 0.25");
}

/// Loose equality between a float and an int compiles and compares numerically
/// (`1.5 == 1` is false, `1.0 == 1` is true) — previously an unsupported backend
/// feature. Operands come from runtime-typed locals so the compare survives folding.
#[test]
fn test_float_int_loose_equality() {
    let out = compile_and_run(
        r#"<?php
function eq(float $a, int $b): string { return ($a == $b) ? "1" : "0"; }
echo eq(1.5, 1), eq(1.0, 1), eq(2.0, 2), eq(2.5, 2);
"#,
    );
    assert_eq!(out, "0110");
}

/// A `switch` over a typed `float` subject matches by PHP loose equality, so a
/// fractional subject does not truncate into an integer case label.
#[test]
fn test_float_switch_matches_numeric_case() {
    let out = compile_and_run(
        r#"<?php
function classify(float $x): string {
    switch ($x) {
        case 1:   return "int-one";
        case 1.5: return "onefive";
        case 2.0: return "two";
        default:  return "other";
    }
}
echo classify(1.5), "|", classify(1.0), "|", classify(2.0), "|", classify(3.7);
"#,
    );
    assert_eq!(out, "onefive|int-one|two|other");
}

/// A `switch` over an integer subject still matches a fractional case only when
/// numerically equal (`2` matches `case 2.0` but not `case 2.5`).
#[test]
fn test_int_switch_with_float_case_labels() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n): string {
    switch ($n) {
        case 2.5: return "twofive";
        case 2.0: return "two";
        default:  return "none";
    }
}
echo pick(2), "|", pick(3);
"#,
    );
    assert_eq!(out, "two|none");
}

/// A local initialized from a literal keeps what earlier code in the switch wrote: the body a
/// case falls into, a later label after an earlier label assigned it, a `default` written
/// between cases (falling into the next one) or first, a nested switch, and the read after the
/// switch. Constant propagation started every body from the value before the switch, so each of
/// these folded to the stale literal.
#[test]
fn test_switch_constant_propagation_follows_fallthrough_and_labels() {
    let out = compile_and_run(
        r#"<?php
function labels(int $n): string {
    $s = "a";
    switch ($n) {
        case ($s = "b") === "x": return "never";
        case ($s = "zz") ? 2 : 2: return $s;
        default: return "d" . $s;
    }
}
function mid_default(int $n): string {
    $k = 0;
    switch ($n) {
        case 1: $k = 5; break;
        default: $k = 7;
        case 2: $k += 1; break;
        case 3: $k = 100;
    }
    return (string) $k;
}
function default_first(int $n): string {
    $t = "t";
    switch ($n) {
        default: $t = "d";
        case 1: $t .= "1";
    }
    return $t;
}
function nested(int $n): int {
    $a = 1;
    switch ($n) {
        case 1:
            $a = 2;
            switch ($n + 1) {
                case 2: $a *= 10;
                case 3: $a += 1;
            }
        case 5:
            $a += 100;
            break;
    }
    return $a;
}
function after(int $n): int {
    $v = 3;
    switch ($n) {
        case 1: $v = 4;
        case 2: $v *= 2; break;
    }
    return $v;
}
function known(): int {
    $w = 1;
    switch (2) {
        case 1: $w = 10;
        case 2: $w += 5;
        case 3: $w += 7; break;
        case 4: $w = 99;
    }
    return $w;
}
echo labels(2), " ", labels(9), "|", mid_default(1), " ", mid_default(2), " ", mid_default(9), " ", mid_default(3), "|";
echo default_first(1), " ", default_first(9), "|", nested(1), " ", nested(5), "|", after(1), " ", after(2), " ", after(3), "|", known(), "\n";
"#,
    );
    assert_eq!(out, "zz dzz|5 1 8 100|t1 d1|121 101|8 6 3|13\n");
}

/// The switch exit models follow execution order and every way out of a body: a `default` written
/// before a case that falls off the end (nested, in a loop, and before code after the switch)
/// does not make the switch "always exit"; `continue` in a body leaves the switch like `break`;
/// and a `break` nested under an `if` keeps its target and the writes before it. Each of these
/// read an uninitialized value, hung, crashed, or folded a stale constant. Review follow-up for
/// #1631.
#[test]
fn test_switch_exit_models_follow_execution_order_and_nested_breaks() {
    let out = compile_and_run(
        r#"<?php
function nested_mid_default(int $n): int {
    $x = 0;
    switch ($n) {
        case 1:
            switch ($n + 1) {
                default: return 9;
                case 2: $x = 5;
            }
            break;
    }
    return $x;
}
function loop_mid_default(int $n): int {
    $x = 0;
    while (true) {
        if ($n === 0) { $x = 7; break; }
        switch ($n) {
            case 1: $x = 1;
            default: return 9;
            case 2: $x = 5;
        }
        break;
    }
    return $x;
}
function continue_in_switch(int $n): int {
    $x = 1;
    switch ($n) {
        case 0: $x = 2; continue 1;
        default: break;
    }
    return $x;
}
function nested_break(int $n, bool $c): int {
    $x = 1;
    switch ($n) {
        case 0:
            if ($c) { $x = 2; break; }
            return 5;
        default: break;
    }
    return $x;
}
function after_mid_default(int $n): int {
    $x = 1;
    switch ($n) {
        case 0: return 10;
        default: return 20;
        case 9: $y = $x;
    }
    $x = 2;
    return $x;
}
echo nested_mid_default(1), nested_mid_default(2), " ", loop_mid_default(0), loop_mid_default(2), " ";
echo continue_in_switch(0), " ", nested_break(0, true), nested_break(0, false), " ";
echo after_mid_default(9), after_mid_default(0), after_mid_default(5), "\n";
"#,
    );
    assert_eq!(out, "50 75 2 25 21020\n");
}
