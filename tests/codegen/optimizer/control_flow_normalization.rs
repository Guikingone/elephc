//! Purpose:
//! End-to-end coverage for the v2 control-flow normalization shell rewrites: `for` loops
//! without an update clause, leading and trailing break guards, trailing `continue` and
//! `return;` removal, final `switch` body `break` removal, `do ... while (true)`, and
//! negated two-way `if` branches.
//!
//! Called from:
//! - `cargo test --test codegen_tests optimizer::control_flow_normalization`.
//!
//! Key details:
//! - Every fixture keeps its decisions runtime-unknown through `$argc` (1 when run without
//!   arguments) so the shapes survive constant folding and reach the normalizer.
//! - Expected outputs were cross-checked with the PHP interpreter.

use super::*;

/// A `for` loop without an update clause whose leading break guard carries an `else` block:
/// the guard folds into the loop test and the `else` body leads the remaining body.
#[test]
fn test_normalization_folds_leading_break_guard_with_else_of_for_without_update() {
    let out = compile_and_run(
        r#"<?php
$i = 0;
for ($i = 0; $i < 5;) {
    if ($i == 3) { break; } else { echo "e"; }
    echo $i;
    $i++;
}
echo "|", $i;
"#,
    );

    assert_eq!(out, "e0e1e2|3");
}

/// `while (true)` ending in a break guard rotates into `do ... while`; the body still runs
/// before the first test and the exit test still sees the incremented counter.
#[test]
fn test_normalization_rotates_endless_loop_with_trailing_break_guard() {
    let out = compile_and_run(
        r#"<?php
$i = $argc;
while (true) {
    echo $i;
    $i++;
    if ($i > 3) { break; }
}
echo "|", $i;
"#,
    );

    assert_eq!(out, "123|4");
}

/// A `continue` that targets the loop skips the trailing guard, so rotation must be refused:
/// `$i == 3` continues past `$i >= 3` and the loop only exits at 4.
#[test]
fn test_normalization_keeps_endless_loop_whose_continue_skips_trailing_guard() {
    let out = compile_and_run(
        r#"<?php
$i = 0;
while (true) {
    $i++;
    if ($i % 2 == $argc) { continue; }
    echo $i;
    if ($i >= 3) { break; }
}
echo "|", $i;
"#,
    );

    assert_eq!(out, "24|4");
}

/// A trailing `continue` inside a `for` loop with an update clause is dropped and the update
/// still runs for that iteration.
#[test]
fn test_normalization_drops_trailing_continue_and_keeps_for_update() {
    let out = compile_and_run(
        r#"<?php
for ($i = 0; $i < 3; $i++) {
    echo $i;
    if ($i == $argc) { echo "c"; continue; }
}
echo "|", $i;
"#,
    );

    assert_eq!(out, "01c2|3");
}

/// `do ... while (true)` becomes `while (true)`; the break guard in the middle of the body
/// keeps working and the counter is observed after the loop.
#[test]
fn test_normalization_rewrites_do_while_true() {
    let out = compile_and_run(
        r#"<?php
$i = 0;
do {
    $i++;
    if ($i > 2 + $argc) { break; }
    echo $i;
} while (true);
echo "|", $i;
"#,
    );

    assert_eq!(out, "123|4");
}

/// The trailing `break` of the last `switch` body is dropped, with and without a `default`,
/// and every case still leaves the switch instead of falling through.
#[test]
fn test_normalization_drops_final_switch_break() {
    let out = compile_and_run(
        r#"<?php
function pick($x) {
    switch ($x) {
        case 1: echo "one"; break;
        case 2: echo "two"; break;
        default: echo "many"; break;
    }
    echo ";";
}
function tag($x) {
    switch ($x) {
        case 1: echo "a"; break;
        case 2: echo "b"; break;
    }
    echo "|";
}
pick($argc); pick($argc + 1); pick($argc + 5);
tag($argc); tag($argc + 1); tag($argc + 2);
"#,
    );

    assert_eq!(out, "one;two;many;a|b||");
}

/// `if (!c) { A } else { B }` swaps into `if (c) { B } else { A }`; a side-effecting condition
/// is still evaluated exactly once and the right branch runs for both outcomes.
#[test]
fn test_normalization_swaps_negated_two_way_branches() {
    let out = compile_and_run(
        r#"<?php
function t($v) { echo "t"; return $v; }
if (!($argc > 5)) { echo "small"; } else { echo "big"; }
if (!t($argc > 5)) { echo "A"; } else { echo "B"; }
if (!t($argc == 1)) { echo "C"; } else { echo "D"; }
"#,
    );

    assert_eq!(out, "smalltAtD");
}

/// Trailing bare `return;` statements are dropped from function and method bodies, through
/// `if` branches and a `try` body whose `finally` still runs on the way out.
#[test]
fn test_normalization_drops_trailing_bare_returns() {
    let out = compile_and_run(
        r#"<?php
function greet($n) {
    if ($n > 1) { echo "many"; return; }
    echo "one";
    return;
}
function safe($x) {
    try {
        echo intdiv(4, $x);
        return;
    } finally {
        echo "f";
    }
}
class Runner {
    public function run(int $n): void {
        if ($n > 1) { echo "R"; return; } else { echo "r"; return; }
    }
}
greet($argc); echo "|"; greet($argc + 1); echo "|";
safe($argc); echo "|";
(new Runner())->run($argc); (new Runner())->run($argc + 1);
"#,
    );

    assert_eq!(out, "one|many|4f|rR");
}

/// Mixed shapes in one program: a `while` with a leading guard and an inner
/// `continue`-carrying guard, and an endless `for` wrapping an update-less inner `for`.
#[test]
fn test_normalization_keeps_nested_loop_semantics() {
    let out = compile_and_run(
        r#"<?php
$total = 0;
$i = $argc;
while ($i < 10) {
    if ($i == 6) { break; }
    if ($i % 2 == 0) { $i++; continue; }
    $total += $i;
    $i++;
}
echo $total, "|", $i, "|";
$rows = [];
for ($r = 0; ; $r++) {
    if ($r >= 3) { break; }
    for ($c = 0; $c < 3;) {
        if ($c == $r) { $c++; continue; }
        $rows[] = "$r$c";
        $c++;
    }
}
echo implode(",", $rows);
"#,
    );

    assert_eq!(out, "9|6|01,02,10,12,20,21");
}

/// A `default` written between two cases keeps its trailing `break`: EIR lowering places the
/// default at its source position, so dropping the `break` would fall through into `case 2`.
#[test]
fn test_normalization_keeps_break_of_default_written_between_cases() {
    let out = compile_and_run(
        r#"<?php
function f($x) {
    switch ($x) {
        case 1: echo "a"; break;
        default: echo "d"; break;
        case 2: echo "b"; break;
    }
    echo "|";
}
f($argc); f($argc + 1); f($argc + 2);
"#,
    );

    assert_eq!(out, "a|b|d|");
}
