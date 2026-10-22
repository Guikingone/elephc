//! Purpose:
//! Regression coverage for append ownership on untyped locals used as reference arguments.
//!
//! Called from:
//! - The codegen integration suite through `runtime_gc`.
//!
//! Key details:
//! - Runtime promotion to a reference cell must not leak unboxed array owners (issue #1907).

use crate::support::compile_and_run_with_heap_debug;

/// Method-local appends remain balanced before and after reference-cell promotion.
#[test]
fn test_ref_local_append_method_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AppendOwner {
    public function render(): void {
        $clip = [];
        $sum = 0;
        for ($i = 0; $i < 100; $i++) { $clip[] = $i; $sum += $clip[$i]; }
        $this->seg($clip, 5);
        for ($i = 100; $i < 200; $i++) { $clip[] = $i; }
        echo $sum, "|", count($clip), "|", $clip[5], "|", $clip[199], "\n";
    }
    public function seg(&$clip, int $i): void { $clip[$i] = 42; }
}
(new AppendOwner())->render();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "4950|200|42|199\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Merely appearing in a reference call must not make earlier appends leak.
#[test]
fn test_ref_local_append_uncalled_reference_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function append_touch(&$items): void { $items[] = 99; }
function build(int $run): void {
    $items = [];
    for ($i = 0; $i < 100; $i++) { $items[] = 0; }
    if ($run < 0) { append_touch($items); }
    echo count($items), "\n";
}
build($argc);
build(-$argc);
$items = [];
for ($i = 0; $i < 100; $i++) { $items[] = 0; }
append_touch($items);
echo count($items), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "100\n101\n101\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Closure-local appends use their final boxed storage when a later branch passes by reference.
#[test]
fn test_ref_local_append_closure_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function keepClosureAppendItems(&$items): void {}
function run(): void {
    $build = function (int $run): void {
        $items = [];
        for ($i = 0; $i < 3000; $i++) { $items[] = $i; }
        if ($run < 0) { keepClosureAppendItems($items); }
        echo count($items), "|", $items[2999], "\n";
    };
    $build($argc);
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3000|2999\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A released function-return string is copied into Mixed append storage before cleanup.
#[test]
fn test_ref_local_append_call_string_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function makeAppendString(int $length): string { return str_repeat("x", $length); }
function keepCallStringItems(&$items): void {}
function run(): void {
    $items = ["seed"];
    $items[] = makeAppendString(32);
    if ($argc < 0) { keepCallStringItems($items); }
    echo strlen($items[1]), "|", $items[1], "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, format!("32|{}\n", "x".repeat(32)), "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Reference appends preserve value snapshots, real aliases, and geometric growth.
#[test]
fn test_ref_local_append_aliases_and_growth() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function append_promote(&$items): void { $items[] = 7; }
function build(): void {
    $items = [];
    for ($i = 0; $i < 3000; $i++) { $items[] = $i; }
    $snapshot = $items;
    append_promote($items);
    $alias = &$items;
    $items[] = 8;
    echo count($snapshot), "|", count($alias), "|", $items[2999], "|", $alias[3001], "\n";
}

build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3000|3002|2999|8\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}


/// Final storage normalization also applies when optional EIR optimization is disabled.
#[test]
fn test_ref_local_append_aliases_and_growth_without_ir_opt() {
    crate::support::without_ir_opt(test_ref_local_append_aliases_and_growth);
}

/// RHS replacement is visible to append, including a value snapshot of the original array.
#[test]
fn test_ref_local_append_rhs_replacement() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function replaceAppendTarget(array &$items): int { $items = [7, 8]; return 9; }
function build(): void {
    $items = [1];
    $snapshot = $items;
    $items[] = replaceAppendTarget($items);
    echo implode(",", $items), "|", implode(",", $snapshot), "\n";
}
build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "7,8,9|1\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Widened string and object element layouts preserve values and balance their payload owners.
#[test]
fn test_ref_local_append_heap_elements() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AppendElement { public int $id; public function __construct(int $id) { $this->id = $id; } }
function keepAppendItems(&$items): void {}
function build(): void {
    $strings = ["seed"];
    for ($i = 0; $i < 4; $i++) { $strings[] = "v" . $i; }
    echo $strings[1], "|", $strings[4], "|", count($strings), "\n";
    keepAppendItems($strings);
    $objects = [new AppendElement(7)];
    $objects[] = new AppendElement(8);
    echo $objects[0]->id, "|", $objects[1]->id, "\n";
    keepAppendItems($objects);
}
build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "v0|v3|5\n7|8\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Associative arrays keep automatic integer keys and copy-on-write snapshots across append.
#[test]
fn test_ref_local_append_associative_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function keepAssocAppendItems(&$items): void {}
function build(): void {
    $items = ["keep" => 1];
    $snapshot = $items;
    for ($i = 0; $i < 100; $i++) { $items[] = $i; }
    echo count($items), "|", $items["keep"], "|", $items[99], "|", count($snapshot), "\n";
    keepAssocAppendItems($items);
}
build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "101|1|99|1\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Throwing RHS evaluation leaves the original array reachable without an unboxed-owner leak.
#[test]
fn test_ref_local_append_throwing_rhs() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function failAppendValue(int $i): int {
    if ($i >= 0) { throw new Exception("stop"); }
    return 1;
}
function keepThrowAppendItems(&$items): void {}
function build(): void {
    $items = [];
    for ($i = 0; $i < 10; $i++) {
        try { $items[] = failAppendValue($i); } catch (Exception $e) {}
    }
    keepThrowAppendItems($items);
    echo count($items), "\n";
}
build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Heterogeneous append conversion remains balanced when a loop revisits a promoted local.
#[test]
fn test_ref_local_append_heterogeneous_loop() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function keepHeterogeneousAppend(&$items): void {}
function build(): void {
    for ($round = 0; $round < 3; $round++) {
        $items = [];
        $items[] = 1;
        $items[] = "x";
        keepHeterogeneousAppend($items);
        echo json_encode($items), "\n";
    }
}
build();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "[1,\"x\"]\n[1,\"x\"]\n[1,\"x\"]\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Typed reference parameters borrow their array while promoted caller locals retain correct reads.
#[test]
fn test_ref_local_append_typed_reference_reads() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function countTypedAppendRef(array &$items): int { return count($items); }
function keepConditionalAppend(&$items): void {}
function build(int $run): void {
    $items = [1, 2];
    $sum = 0;
    for ($i = 0; $i < 10; $i++) { $sum += countTypedAppendRef($items); }
    echo $sum, "|", count($items), "\n";
    $other = [];
    if ($run < 0) { keepConditionalAppend($other); }
    for ($j = 0; $j < 10; $j++) { $other[] = $j; }
    echo count($other), "|", $other[9], "\n";
}
build($argc);
build(-$argc);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "20|2\n10|9\n20|2\n10|9\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// RHS changes to scalar or string retain catchable errors and release the pending value.
#[test]
fn test_ref_local_append_rhs_parent_guards() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function replaceAppendScalar(&$items, string $pending): string { $items = 7; return $pending; }
function replaceAppendString(&$items): int { $items = "kept"; return 9; }
function replaceAppendNull(&$items): int { $items = null; return 9; }
function buildAppendGuards(): void {
    for ($round = 0; $round < 3; $round++) {
        $items = [1];
        $snapshot = $items;
        try { $items[] = replaceAppendScalar($items, str_repeat("x", 400000)); echo "missed|"; }
        catch (Error $e) { echo "caught|"; }
        echo $items, "|", $snapshot[0], "\n";
        $strings = ["keep" => 1];
        try { $strings[] = replaceAppendString($strings); echo "missed|"; }
        catch (Error $e) { echo "caught|"; }
        echo $strings, "\n";
        $nullable = [1];
        $nullable[] = replaceAppendNull($nullable);
        echo json_encode($nullable), "\n";
    }
}
buildAppendGuards();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "caught|7|1\ncaught|kept\n[9]\n".repeat(3), "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// RHS parent checks also run with mandatory storage normalization and no EIR optimization.
#[test]
fn test_ref_local_append_rhs_parent_guards_without_ir_opt() {
    crate::support::without_ir_opt(test_ref_local_append_rhs_parent_guards);
}

/// Eval-boxed locals preserve native string layouts for sorting without leaking append owners.
#[test]
fn test_ref_local_append_native_layout_heap_balance() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$items = [];
foreach (["THREE", "ONE", "TWO"] as $name) { $items[] = strtolower($name); }
sort($items);
echo implode(",", $items), "\n";
eval('echo "done\n";');
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "one,three,two\ndone\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
