//! Purpose:
//! Integration tests for expression-position list/bracket destructuring with skipped,
//! keyed, nested, and non-variable-target patterns (`if ([, , , $access] = $scopes[$name]
//! ?? null)` — the symfony/var-exporter LazyDecoratorTrait.php:94 gate), desugared at parse
//! time onto the statement-form destructuring through the `ExprKind::Assignment` prelude.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output is cross-checked against `php` (PHP 8.5): a destructuring
//!   assignment expression yields its full right-hand side, destructuring a runtime null
//!   assigns null to every target without a crash, and conditional contexts (ternary
//!   branches, `&&` right operands) confine the destructure to the taken path.
//! - Fixtures thread `$argc` into the arrays/conditions so the constructs survive AST-level
//!   constant folding and actually reach EIR lowering.

use super::*;

/// Verifies THE `--web` gate shape (LazyDecoratorTrait.php:94): a skipped-slot pattern
/// destructuring `$scopes[$name] ?? null` in an `if` condition takes the truthy branch and
/// binds the fourth element when the key exists.
#[test]
fn test_expr_destructure_gate_shape_hit() {
    let out = compile_and_run(
        "<?php\n\
         $scopes = [\"k\" => [10, 20, 30, 40]];\n\
         $name = $argc > 99 ? \"z\" : \"k\";\n\
         if ([, , , $access] = $scopes[$name] ?? null) { echo $access; } else { echo \"none\"; }",
    );
    assert_eq!(out, "40");
}

/// Verifies the gate shape's falsy path: a missing key makes `?? null` yield null, the
/// destructuring assignment evaluates to that null, and the `else` branch runs (PHP: `none`).
#[test]
fn test_expr_destructure_gate_shape_missing_key() {
    let out = compile_and_run(
        "<?php\n\
         $scopes = [\"k\" => [10, 20, 30, 40]];\n\
         $name = $argc > 99 ? \"k\" : \"z\";\n\
         if ([, , , $access] = $scopes[$name] ?? null) { echo $access; } else { echo \"none\"; }",
    );
    assert_eq!(out, "none");
}

/// Verifies a skipped-slot pattern in an `if` condition binds the kept element and the
/// condition tests the full right-hand side array (truthy).
#[test]
fn test_expr_destructure_holes_in_if() {
    let out =
        compile_and_run("<?php $pair = [$argc, 7]; if ([, $b] = $pair) { echo $b; }");
    assert_eq!(out, "7");
}

/// Verifies a keyed pattern (`[\"b\" => $x]`) works in expression position, binding from the
/// string key like the statement form.
#[test]
fn test_expr_destructure_keyed_in_expression() {
    let out = compile_and_run(
        "<?php $arr = [\"a\" => 1, \"b\" => $argc + 1]; if ([\"b\" => $x] = $arr) { echo $x; }",
    );
    assert_eq!(out, "2");
}

/// Verifies a nested pattern (`[[$a, $b], [$c, $d]]`) works in expression position,
/// destructuring both inner arrays.
#[test]
fn test_expr_destructure_nested_in_expression() {
    let out = compile_and_run(
        "<?php $p = [[$argc, 2], [3, 4]]; if ([[$a, $b], [$c, $d]] = $p) { echo $a, $b, $c, $d; }",
    );
    assert_eq!(out, "1234");
}

/// Verifies the expression yields the FULL right-hand side (PHP: the value of a
/// destructuring assignment is the RHS array): `$ok = [, $b] = [1, 2]` leaves `$ok` an
/// array of BOTH elements, not just the bound one.
#[test]
fn test_expr_destructure_yields_full_rhs_array() {
    let out = compile_and_run(
        "<?php $ok = [, $b] = [1, $argc + 1]; echo $b, \" \", count($ok), \" \", $ok[0], $ok[1];",
    );
    assert_eq!(out, "2 2 12");
}

/// Verifies a destructuring assignment as a `while` condition re-evaluates per iteration:
/// each pass rebinds `$tag` from the next queue row until `?? null` turns the RHS falsy.
#[test]
fn test_expr_destructure_while_condition() {
    let out = compile_and_run(
        "<?php\n\
         $queue = [[1, \"a\"], [2, \"b\"]];\n\
         $i = $argc - 1;\n\
         while ([, $tag] = $queue[$i] ?? null) { echo $tag; $i = $i + 1; }\n\
         echo \"end\";",
    );
    assert_eq!(out, "abend");
}

/// Verifies ternary-branch confinement (untaken branch): with the condition false, the
/// destructure in the true branch must NOT run — `$y` keeps its prior value and the
/// ternary result is the false arm.
#[test]
fn test_expr_destructure_ternary_branch_confinement_untaken() {
    let out = compile_and_run(
        "<?php $cond = $argc > 1; $src = [7, 8]; $y = 0; $r = $cond ? ([, $y] = $src) : 0; \
         echo $y, \" \", is_array($r) ? count($r) : $r;",
    );
    assert_eq!(out, "0 0");
}

/// Verifies ternary-branch confinement (taken branch): with the condition true, the
/// destructure runs, rebinding `$y`, and the ternary result is the full RHS array.
#[test]
fn test_expr_destructure_ternary_branch_confinement_taken() {
    let out = compile_and_run(
        "<?php $cond = $argc > 0; $src = [7, 8]; $y = 0; $r = $cond ? ([, $y] = $src) : 0; \
         echo $y, \" \", is_array($r) ? count($r) : $r;",
    );
    assert_eq!(out, "8 2");
}

/// Verifies `&&` right-operand confinement (short-circuit): with a false left operand the
/// destructure never runs and the guarded read is skipped (PHP: `skipped`).
#[test]
fn test_expr_destructure_and_rhs_short_circuits() {
    let out = compile_and_run(
        "<?php $flag = $argc > 1; $took = $flag && ([, $z] = [3, 4]); \
         echo $took ? \"z=$z\" : \"skipped\";",
    );
    assert_eq!(out, "skipped");
}

/// Verifies `&&` right-operand execution: with a true left operand the destructure runs
/// and binds `$z` from the second element.
#[test]
fn test_expr_destructure_and_rhs_runs_when_left_true() {
    let out = compile_and_run(
        "<?php $flag = $argc > 0; $took = $flag && ([, $z] = [3, 4]); \
         echo $took ? \"z=$z\" : \"skipped\";",
    );
    assert_eq!(out, "z=4");
}

/// Verifies destructuring a runtime null right-hand side matches PHP's observable
/// behavior: the expression is falsy (else branch) and the target variable holds null —
/// no warning divergence asserted, no crash.
#[test]
fn test_expr_destructure_runtime_null_rhs_falsy_and_binds_null() {
    let out = compile_and_run(
        "<?php $v = ($argc > 99) ? [1, 2] : null; \
         if ([, $b] = $v) { echo \"truthy\"; } else { echo \"falsy\"; } \
         echo \" \", is_null($b) ? \"null\" : \"set\";",
    );
    assert_eq!(out, "falsy null");
}

/// Verifies the `list(...)` construct with a skipped slot works in expression position
/// (`if (list(, $b) = ...)`), which PHP permits like the bracket form.
#[test]
fn test_expr_list_construct_with_hole_in_if() {
    let out = compile_and_run("<?php if (list(, $b) = [5, $argc + 5]) { echo $b; }");
    assert_eq!(out, "6");
}

/// Verifies the all-simple `list($x, $y)` construct in expression position maps onto the
/// same lowering as the bracket `[$x, $y]` form and binds both elements.
#[test]
fn test_expr_list_construct_all_simple_in_if() {
    let out = compile_and_run("<?php if (list($x, $y) = [$argc, 2]) { echo $x, $y; }");
    assert_eq!(out, "12");
}

/// Verifies a keyed destructure used as a ternary CONDITION: the destructure always runs,
/// binds `$lvl`, and its truthy RHS picks the interpolating arm.
#[test]
fn test_expr_destructure_as_ternary_condition() {
    let out = compile_and_run(
        "<?php $parts = [\"date\" => \"2026-07-16\", \"level\" => $argc + 2]; \
         $msg = ([\"level\" => $lvl] = $parts) ? \"lvl=$lvl\" : \"none\"; echo $msg;",
    );
    assert_eq!(out, "lvl=3");
}

/// Verifies a non-variable pattern target (an object property) in expression position:
/// `[$o->v, $b] = [...]` writes through to the property like the statement form.
#[test]
fn test_expr_destructure_property_target() {
    let out = compile_and_run(
        "<?php class Box { public int $v = 0; } $o = new Box(); \
         if ([$o->v, $b] = [5, $argc + 5]) { echo $o->v, $b; }",
    );
    assert_eq!(out, "56");
}
