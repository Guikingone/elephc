//! Purpose:
//! End-to-end tests for foreach lvalue key/value binding targets
//! (`foreach ($defs as $this->id => $d)`, `foreach ($rows as $out["k"])`,
//! `foreach ($m as R::$k => $v)`), which the parser desugars onto a hidden
//! loop variable plus a prepended plain assignment statement.
//!
//! Called from:
//! - `cargo test` through the `codegen_tests` harness via `crate::support`.
//!
//! Key details:
//! - Every expected output is cross-checked against `php` (PHP 8 semantics).
//! - PHP assigns the lvalue each iteration BEFORE the body runs, leaves the
//!   last iteration's binding after the loop, and never assigns for an empty
//!   iterable; `continue`/`break` interact with the store as the first body
//!   statement.

use crate::support::*;

/// Verifies a property KEY target (`foreach ($defs as $this->currentId => $d)`)
/// assigns the property each iteration and leaves the last key after the loop
/// (the ResolveInvalidReferencesPass.php:45 shape).
#[test]
fn test_foreach_property_key_target() {
    let out = compile_and_run(
        r#"<?php
class P {
    public string $currentId = "";
    public function run(array $defs): void {
        foreach ($defs as $this->currentId => $d) {
            echo $this->currentId, "=", $d, ";";
        }
        echo "\n", "last:", $this->currentId, "\n";
    }
}
(new P())->run(["a" => 1, "b" => 2]);
"#,
    );
    assert_eq!(out, "a=1;b=2;\nlast:b\n");
}

/// Verifies a property VALUE target (`foreach ($arr as $q->v)`) stores each
/// element into the property and keeps the last element after the loop.
#[test]
fn test_foreach_property_value_target() {
    let out = compile_and_run(
        r#"<?php
class Q { public int $v = 0; }
$q = new Q();
$arr = [10, 20, 30];
foreach ($arr as $q->v) { echo $q->v, ";"; }
echo "\n", $q->v, "\n";
"#,
    );
    assert_eq!(out, "10;20;30;\n30\n");
}

/// Verifies an array-element VALUE target (`foreach ([1,2,3] as $out["k"])`)
/// writes each element into the array slot through the existing
/// `ArrayAssign` machinery.
#[test]
fn test_foreach_array_element_value_target() {
    let out = compile_and_run(
        r#"<?php
$out = [];
foreach ([1, 2, 3] as $out["k"]) { echo $out["k"], ";"; }
echo "\n";
"#,
    );
    assert_eq!(out, "1;2;3;\n");
}

/// Verifies a static-property KEY target (`foreach (["x" => 5] as R::$k => $v)`)
/// stores the (runtime-Mixed) foreach key into the declared string static
/// property each iteration.
#[test]
fn test_foreach_static_property_key_target() {
    let out = compile_and_run(
        r#"<?php
class R { public static string $k = ""; }
foreach (["x" => 5] as R::$k => $v) { echo R::$k, "=", $v, "\n"; }
"#,
    );
    assert_eq!(out, "x=5\n");
}

/// Verifies BOTH positions desugared in the same loop
/// (`foreach ($a as $t->k => $t->v)`): both properties are bound before the
/// body runs and both hold the last iteration's binding after the loop.
#[test]
fn test_foreach_both_positions_property_targets() {
    let out = compile_and_run(
        r#"<?php
class T { public string $k = "init"; public int $v = -1; }
$t = new T();
foreach (["a" => 1, "b" => 2] as $t->k => $t->v) { echo $t->k, ":", $t->v, ";"; }
echo "\n", $t->k, "/", $t->v, "\n";
"#,
    );
    assert_eq!(out, "a:1;b:2;\nb/2\n");
}

/// Verifies an empty iterable never assigns the lvalue target: the property
/// keeps its pre-loop value.
#[test]
fn test_foreach_lvalue_target_empty_iterable_untouched() {
    let out = compile_and_run(
        r#"<?php
class Q { public int $v = 42; }
$q = new Q();
$empty = [];
foreach ($empty as $q->v) { echo "never"; }
echo $q->v, "\n";
"#,
    );
    assert_eq!(out, "42\n");
}

/// Verifies `continue` interacts correctly with the desugared store: the
/// per-iteration assignment is the first body statement, so a skipped
/// iteration still updated the property.
#[test]
fn test_foreach_lvalue_target_with_continue() {
    let out = compile_and_run(
        r#"<?php
class Q { public int $v = 0; }
$q = new Q();
foreach ([1, 2, 3, 4] as $q->v) {
    if ($q->v % 2 == 0) { continue; }
    echo $q->v, ";";
}
echo "\n", $q->v, "\n";
"#,
    );
    assert_eq!(out, "1;3;\n4\n");
}

/// Verifies `break` leaves the lvalue holding the binding of the iteration
/// that broke (the store runs before the body).
#[test]
fn test_foreach_lvalue_target_with_break() {
    let out = compile_and_run(
        r#"<?php
class B { public int $v = 0; }
$b = new B();
foreach ([1, 2, 3] as $b->v) {
    if ($b->v == 2) { break; }
    echo $b->v, ";";
}
echo "\n", $b->v, "\n";
"#,
    );
    assert_eq!(out, "1;\n2\n");
}

/// Verifies two lvalue-target desugars in nested loops in the same scope: the
/// hidden loop variables (named by line:col) stay distinct and both loops bind
/// their targets independently.
#[test]
fn test_foreach_lvalue_targets_nested_loops() {
    let out = compile_and_run(
        r#"<?php
class P { public string $k = ""; public int $v = 0; }
$p = new P();
$out = [];
foreach (["x" => 1, "y" => 2] as $p->k => $d) {
    foreach ([10, 20] as $out["cell"]) {
        echo $p->k, "-", $d, "-", $out["cell"], ";";
    }
}
echo "\n", $p->k, "/", $out["cell"], "\n";
"#,
    );
    assert_eq!(out, "x-1-10;x-1-20;y-2-10;y-2-20;\ny/20\n");
}

/// Verifies a dynamic-property VALUE target (`foreach ($a as $d->{$name})`)
/// routes through the expression-position assignment shape used for
/// `$obj->{$name} = $v;`.
#[test]
fn test_foreach_dynamic_property_value_target() {
    let out = compile_and_run(
        r#"<?php
class D { public int $a = 0; }
$d = new D();
$name = "a";
foreach ([7, 8] as $d->{$name}) { echo $d->a, ";"; }
echo "\n", $d->a, "\n";
"#,
    );
    assert_eq!(out, "7;8;\n8\n");
}
