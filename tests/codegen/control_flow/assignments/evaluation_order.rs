//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of control flow, assignments evaluation order, including assignment expression effectful index evaluates once, assignment expression uses rhs mutated variable index, and compound assignment expression uses rhs mutated variable index.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies that in `$items[idx()] = val()`, idx() is called exactly once (not
/// twice: once to read the old value and once to write the new value). Echo
/// output "iv7:7" confirms idx() runs once, val() runs once, and items[1]=7.
#[test]
fn test_array_assignment_expression_effectful_index_evaluates_once() {
    let out = compile_and_run(
        r#"<?php
function idx(): int {
    echo "i";
    return 1;
}
function val(): int {
    echo "v";
    return 7;
}
$items = [0, 0];
echo ($items[idx()] = val());
echo ":" . $items[1];
"#,
    );
    assert_eq!(out, "iv7:7");
}

/// Verifies that in `$items[$i] = ($i = 1)`, the index `$i` is captured before the
/// RHS mutates it. Output "1:10:1:1" confirms items[0] receives RHS value 1, items[1]
/// stays 20, and $i becomes 1. The RHS assignment to $i does not retroactively
/// change which slot was selected for the initial array access.
#[test]
fn test_array_assignment_expression_uses_rhs_mutated_variable_index() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 0;
echo ($items[$i] = ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "1:10:1:1");
}

/// Verifies that in `$items[$i] += ($i = 1)`, the compound assignment uses the
/// index captured before RHS mutation. Output "21:10:21:1" confirms items[0] (at $i=0)
/// receives 10+1=21, items[1] stays 10, $i becomes 1.
#[test]
fn test_array_compound_assignment_expression_uses_rhs_mutated_variable_index() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 0;
echo ($items[$i] += ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "21:10:21:1");
}

/// Verifies that in `$items[$i + 0] = ($i = 1)`, the computed index expression
/// stabilizes to index 0 before the RHS mutates $i. Output "1:1:20:1" confirms
/// items[0] receives the RHS value 1, items[1] stays 20, and $i becomes 1.
#[test]
fn test_array_assignment_expression_stabilizes_computed_index_before_rhs() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 0;
echo ($items[$i + 0] = ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "1:1:20:1");
}

/// Verifies that in `make_box()->value += inc()`, the receiver expression make_box()
/// is called exactly once. Echo output "mr5" confirms make_box() runs once ("m"),
/// inc() runs once ("r"), and the result is 5. This is a property rather than an
/// array index case.
#[test]
fn test_property_assignment_expression_effectful_receiver_evaluates_once() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public $value = 1;
}
function make_box(): Box {
    echo "m";
    return new Box();
}
function inc(): int {
    echo "r";
    return 4;
}
echo (make_box()->value += inc());
"#,
    );
    assert_eq!(out, "mr5");
}

/// Verifies that in `Registry::$items[idx()] += 2`, idx() is called exactly once.
/// Echo output "i5:5" confirms idx() runs once ("i"), items[0] becomes 5, and the
/// result is 5. This is a static property with a function call index.
#[test]
fn test_static_property_array_assignment_expression_effectful_index_evaluates_once() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static $items = [3, 4];
}
function idx(): int {
    echo "i";
    return 0;
}
echo (Registry::$items[idx()] += 2);
echo ":" . Registry::$items[0];
"#,
    );
    assert_eq!(out, "i5:5");
}

/// Verifies that in `$items[idx()] ??= fallback()`, idx() is called exactly once
/// and short-circuit works when the index exists. Echo output "i5:5" confirms idx()
/// runs once ("i"), fallback() is not called (no "f"), and items[0] stays 5.
#[test]
fn test_null_coalesce_assignment_expression_effectful_index_short_circuits_once() {
    let out = compile_and_run(
        r#"<?php
function idx(): int {
    echo "i";
    return 0;
}
function fallback(): int {
    echo "f";
    return 9;
}
$items = [5, 2];
echo ($items[idx()] ??= fallback());
echo ":" . $items[0];
"#,
    );
    assert_eq!(out, "i5:5");
}

/// Verifies that in `$items[$i] ??= ($i = 1)` with $i=2, the index $i is captured
/// before the RHS mutates it, and the null-coalesce assigns because items[2] is null.
/// Output "1:10:1:1" confirms items[2] receives RHS value 1, items[0]=10, items[1]=20,
/// and $i becomes 1.
#[test]
fn test_null_coalesce_assignment_expression_uses_rhs_mutated_variable_index() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 2;
echo ($items[$i] ??= ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "1:10:1:1");
}

/// Verifies that in `$items[$i] ??= ($i = 1)` with $i=0 where items[0] is 10 (not null),
/// the RHS is not evaluated and $i stays 0. Output "10:10:20:0" confirms short-circuit:
/// items[0] remains 10, items[1] remains 20, and $i is not mutated by the RHS.
#[test]
fn test_null_coalesce_assignment_expression_short_circuits_rhs_mutated_index() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 0;
echo ($items[$i] ??= ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "10:10:20:0");
}

/// Verifies that in `$items[$i + 0] ??= ($i = 1)` with $i=0 where items[0]=10 (not null),
/// the computed index stabilizes to 0, the short-circuit prevents RHS evaluation, and
/// $i stays 0. Output "10:10:20:0" confirms neither the index expression nor the
/// RHS mutation affects items or $i.
#[test]
fn test_null_coalesce_assignment_expression_stabilizes_computed_index_before_rhs() {
    let out = compile_and_run(
        r#"<?php
$items = [10, 20];
$i = 0;
echo ($items[$i + 0] ??= ($i = 1));
echo ":" . $items[0] . ":" . $items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "10:10:20:0");
}

/// Verifies that in `$items[idx()] ??= ($i = 1)` with idx() returning 2, the index
/// function is called exactly once ("i"), the null-coalesce assigns because items[2]
/// is null, and $i is mutated to 1 by the RHS. Output "i1:1:1" confirms idx() runs
/// once, fallback is not called, items[2]=1, and $i=1.
#[test]
fn test_null_coalesce_assignment_expression_effectful_index_mutating_rhs_runs_once() {
    let out = compile_and_run(
        r#"<?php
function idx(): int {
    echo "i";
    return 2;
}
$items = [10, 20];
$i = 0;
echo ($items[idx()] ??= ($i = 1));
echo ":" . $items[2] . ":" . $i;
"#,
    );
    assert_eq!(out, "i1:1:1");
}

/// Verifies that in `Registry::$items[$i] ??= ($i = 1)` with $i=2, the static property
/// index $i is captured before the RHS mutates it. Since items[2] is null, the
/// null-coalesce assigns and $i becomes 1. Output "1:1:1" confirms items[1]=1 and $i=1.
#[test]
fn test_static_property_null_coalesce_assignment_expression_rhs_mutated_index() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public static $items = [10, 20];
}
$i = 2;
echo (Registry::$items[$i] ??= ($i = 1));
echo ":" . Registry::$items[1] . ":" . $i;
"#,
    );
    assert_eq!(out, "1:1:1");
}

/// Verifies that a variable assigned in one `match` arm condition is visible (with the
/// correct value) to the conditions of later arms. PHP evaluates match-arm conditions
/// top-to-bottom in a single scope, so `$len`, assigned in the first arm's condition, must
/// resolve in the `$len < 0` and `$len < 10` conditions of the following arms. This is the
/// symfony/yaml `Inline::dump()` pattern (`!$length = \strlen(...) => 'c', $length < 4 => ...`).
#[test]
fn test_match_arm_condition_assignment_visible_in_later_arms() {
    let out = compile_and_run(
        r#"<?php
function classify(int $n): string {
    return match (true) {
        ($len = $n) > 100 => 'huge',
        $len < 0 => 'neg',
        $len < 10 => 'small',
        default => 'mid',
    };
}
echo classify(5), "|", classify(-3), "|", classify(200), "|", classify(50);
"#,
    );
    assert_eq!(out, "small|neg|huge|mid");
}

/// Verifies that a variable assigned in an earlier operand of a short-circuit `&&` chain is
/// visible (with the correct value) to later operands of the same chain. PHP evaluates `&&`
/// left-to-right, so `$sum`, assigned in the second operand, must resolve in the third operand
/// `$sum < 100`. This is the symfony/yaml `Parser` pattern
/// (`... && ($whitespaces = strspn(...)) < n && '#' !== $line[$whitespaces]`).
#[test]
fn test_and_chain_operand_assignment_visible_in_later_operand() {
    let out = compile_and_run(
        r#"<?php
function check(int $a, int $b): string {
    if ($a > 0 && ($sum = $a + $b) > 5 && $sum < 100) {
        return "in-range";
    }
    return "out";
}
echo check(3, 4), "|", check(3, 1), "|", check(50, 60);
"#,
    );
    assert_eq!(out, "in-range|out|out");
}

/// Verifies the same left-to-right visibility for a `||` chain: in a pure `||` chain each later
/// operand runs only after the earlier operands have been evaluated, so `$d`, assigned in the
/// second operand, must resolve (with the right value) in the third operand `$d === 30`.
#[test]
fn test_or_chain_operand_assignment_visible_in_later_operand() {
    let out = compile_and_run(
        r#"<?php
function pick(int $a): string {
    if ($a < 0 || ($d = $a * 10) > 50 || $d === 30) {
        return "hit";
    }
    return "miss";
}
echo pick(-1), "|", pick(6), "|", pick(3), "|", pick(1);
"#,
    );
    assert_eq!(out, "hit|hit|hit|miss");
}

/// Verifies that an ordinary assignment in the RIGHT operand of a short-circuit `&&` chain
/// surfaces the assigned variable to the outer scope, so a later read is not a false "Undefined
/// variable". PHP prints `5` for `$a = 1; if ($a > 0 && ($u = 5) > 0) { echo $u; }`.
#[test]
fn test_and_chain_rhs_assignment_surfaces_to_outer_scope() {
    let out = compile_and_run(
        r#"<?php
$a = 1;
if ($a > 0 && ($u = 5) > 0) { echo $u; }
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies the same outer-scope surfacing for a `||` chain: an assignment in the right operand
/// (`($u = 5) > 0`) defines `$u` for the following body read. PHP prints `5` for
/// `$a = 0; if ($a > 0 || ($u = 5) > 0) { echo $u; }`.
#[test]
fn test_or_chain_rhs_assignment_surfaces_to_outer_scope() {
    let out = compile_and_run(
        r#"<?php
$a = 0;
if ($a > 0 || ($u = 5) > 0) { echo $u; }
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies the symfony `AbstractUnicodeString::wcswidth` shape: an assignment nested inside an
/// array-index subscript in the RIGHT operand of an `&&` condition
/// (`$tbl[$ubound = \count($tbl) - 1]`) surfaces `$ubound` so the following `while` loop can read
/// it. A local array is used (not a static nested-array property) so the fixture does not depend on
/// the separate EIR static-property-default codegen surface. PHP prints `1|2|3|0`.
#[test]
fn test_and_chain_nested_index_assignment_surfaces_to_outer_scope() {
    let out = compile_and_run(
        r#"<?php
function widthOf(int $cp): int {
    $tbl = [10, 30, 50];
    $lbound = 0;
    if ($cp >= 0 && $cp <= $tbl[$ubound = \count($tbl) - 1]) {
        while ($ubound >= $lbound) {
            $mid = intdiv($lbound + $ubound, 2);
            if ($cp > $tbl[$mid]) {
                $lbound = $mid + 1;
            } else {
                if ($mid === 0 || $cp > $tbl[$mid - 1]) {
                    return $mid + 1;
                }
                $ubound = $mid - 1;
            }
        }
    }
    return 0;
}
echo widthOf(5), "|", widthOf(25), "|", widthOf(45), "|", widthOf(99);
"#,
    );
    assert_eq!(out, "1|2|3|0");
}

/// Verifies that an assignment in the LAST operand of a three-operand `&&` chain surfaces to the
/// outer scope. With `$a = $b = true`, the third operand `($u = 7)` runs and `$u` must be readable
/// after the chain. PHP prints `7`.
#[test]
fn test_three_operand_and_chain_last_operand_assignment_surfaces() {
    let out = compile_and_run(
        r#"<?php
$a = true;
$b = true;
if ($a && $b && ($u = 7)) { echo $u; }
"#,
    );
    assert_eq!(out, "7");
}

/// Regression: a variable already defined outside the chain is not corrupted when it is reassigned
/// inside a later operand. The `or_insert` surfacing must not overwrite the outer binding, and the
/// runtime value after the chain is the reassigned one. PHP prints `9` for
/// `$x = 3; if (true && ($x = 9)) {} echo $x;`.
#[test]
fn test_outer_variable_not_corrupted_by_chain_reassignment() {
    let out = compile_and_run(
        r#"<?php
$x = 3;
if (true && ($x = 9)) {}
echo $x;
"#,
    );
    assert_eq!(out, "9");
}

/// Regression: an assignment in the FIRST (left) operand still surfaces (the already-working path
/// is unchanged). `($u = 5) > 0` in the left operand then `$u < 10` in the right, then a body read.
/// PHP prints `5`.
#[test]
fn test_left_operand_assignment_still_surfaces() {
    let out = compile_and_run(
        r#"<?php
if (($u = 5) > 0 && $u < 10) { echo $u; }
"#,
    );
    assert_eq!(out, "5");
}

/// Regression: a short-circuit chain with NO assignment surfaces nothing — the merge is a no-op and
/// the program behaves normally. PHP prints `ok` for
/// `$a = 1; $b = 2; if ($a && $b) { echo "ok"; }`.
#[test]
fn test_short_circuit_chain_without_assignment_is_noop() {
    let out = compile_and_run(
        r#"<?php
$a = 1;
$b = 2;
if ($a && $b) { echo "ok"; }
"#,
    );
    assert_eq!(out, "ok");
}

/// Regression (sibling of the short-circuit-chain assignment surfacing): a variable assigned in the
/// RHS of a null-coalescing assignment to a non-local target (`$cache[$k] ??= ($p = 5) > 0 ? …`) is
/// visible to LATER sub-expressions of that same RHS. PHP evaluates the `??=` default left-to-right,
/// so `$p` is defined before the ternary branch `$p * 2` reads it. Before the fix the non-local
/// `??=` value was re-inferred through a plain (non-effect) pass that reported a spurious
/// "Undefined variable: $p" at `$p * 2`. This is the symfony/var-dumper `Cloner\Stub` shape. PHP
/// prints `10`.
#[test]
fn test_coalesce_assign_rhs_assignment_visible_within_same_rhs() {
    let out = compile_and_run(
        r#"<?php
$cache = [];
$k = 'a';
$d = $cache[$k] ??= ($p = 5) > 0 ? $p * 2 : 0;
echo $d;
"#,
    );
    assert_eq!(out, "10");
}

/// Regression (nested-array `??=` target, both ternary branches read the RHS-assigned variable):
/// `$m[$c][$k] ??= ($p = 4) > 1 ? $p + 100 : $p` compiles and both branches see `$p`. PHP prints
/// `104`.
#[test]
fn test_coalesce_assign_nested_target_rhs_assignment_visible_in_both_branches() {
    let out = compile_and_run(
        r#"<?php
$m = [];
$c = 'x';
$k = 'y';
$r = $m[$c][$k] ??= ($p = 4) > 1 ? $p + 100 : $p;
echo $r;
"#,
    );
    assert_eq!(out, "104");
}

/// Soundness control for the fix above: a variable defined ONLY inside a `??=` RHS must NOT leak as
/// definitely-defined to code AFTER the statement. When the target is already non-null the RHS never
/// runs, so PHP leaves `$p` undefined afterwards (a runtime warning). The checker must still reject
/// the post-statement read; the within-RHS visibility fix is scoped to a cloned env and does not
/// over-leak. Mirrors PHP's `Undefined variable $p` for
/// `$c = ['a' => 1]; $k = 'a'; $c[$k] ??= ($p = 5); echo $p;`.
#[test]
fn test_coalesce_assign_rhs_only_variable_not_defined_after_statement() {
    let out = compile_expect_check_error(
        r#"<?php
$c = ['a' => 1];
$k = 'a';
$c[$k] ??= ($p = 5);
echo $p;
"#,
    );
    assert!(
        out.contains("Undefined variable: $p"),
        "expected post-`??=` read of an RHS-only variable to be rejected, got: {out}"
    );
}

