//! Purpose:
//! Regression coverage for the expression, call, and overwrite leaks grouped in #551.
//!
//! Called from:
//! - The codegen integration harness through `runtime_gc`.
//!
//! Key details:
//! - Compare short and long loops when enum singletons leave a fixed live baseline.
//! - Heap-debug fixtures check that direct Mixed arguments and repeated stores retire owners.

use crate::support::{
    compile_and_run_with_gc_stats, compile_and_run_with_heap_debug, parse_gc_stats, ProgramOutput,
};

/// Confirms a fixture exits successfully, prints the expected result, and has no leaked blocks.
fn assert_heap_clean(output: ProgramOutput, expected: &str) {
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, expected, "{}", output.stderr);
    assert!(
        output.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        output.stderr
    );
}

/// Confirms increasing the loop count does not increase the live allocation count.
fn assert_bounded_live(source: &str, expected_per_iteration: usize) {
    let live = |iterations: usize| {
        let source = source.replace("ITERATIONS", &iterations.to_string());
        let output = compile_and_run_with_gc_stats(&source);
        assert!(output.success, "{}", output.stderr);
        assert_eq!(
            output.stdout,
            (expected_per_iteration * iterations).to_string(),
            "{}",
            output.stderr
        );
        let (allocs, frees) = parse_gc_stats(&output.stderr);
        allocs - frees
    };
    assert_eq!(live(40), live(5), "live blocks grew with loop iterations");
}

/// #483: direct enum member reads and both nullsafe exits retire their call receivers.
#[test]
fn test_issue_551_enum_expression_receivers_do_not_grow_live_heap() {
    assert_bounded_live(
        r#"<?php
enum L: int { case A = 1; case B = 2; }
$acc = 0;
for ($n = 0; $n < ITERATIONS; $n++) {
    $acc += L::tryFrom(2)->value;
    $acc += L::tryFrom(2)?->value ?? 0;
    $acc += L::tryFrom(99)?->value ?? 0;
    $c = L::tryFrom(2);
    $acc += $c === null ? 0 : $c->value;
    $missing = L::tryFrom(99);
    $acc += $missing === null ? 0 : $missing->value;
}
echo $acc;
"#,
        6,
    );
}

/// #483: a non-nullable object returned by a call is released after its property read.
#[test]
fn test_issue_551_object_receiver_does_not_grow_live_heap() {
    assert_bounded_live(
        r#"<?php
class P { public int $v = 3; }
function makeP(): P { return new P(); }
$acc = 0;
for ($n = 0; $n < ITERATIONS; $n++) {
    $acc += makeP()->v;
}
echo $acc;
"#,
        3,
    );
}

/// #486: direct object and array arguments to a Mixed parameter release producer owners.
#[test]
fn test_issue_551_direct_mixed_arguments_leave_heap_clean() {
    let output = compile_and_run_with_heap_debug(
        r#"<?php
class P { public int $v = 3; }
function probe(mixed $m): int { return is_object($m) ? 1 : 2; }
$acc = 0;
for ($n = 0; $n < 50; $n++) {
    $acc += probe(new P());
    $acc += probe([$n, $n]);
}
echo $acc;
"#,
    );
    assert_heap_clean(output, "150");
}

/// #498: overwriting one untyped property with promoted arithmetic releases old boxes.
#[test]
fn test_issue_551_untyped_property_overwrite_leaves_heap_clean() {
    let output = compile_and_run_with_heap_debug(
        r#"<?php
class C { public $v = 0; }
$c = new C();
$i = $argc;
while ($i < 50) {
    $c->v = $i * 3 - 7;
    $i = $i + 1;
}
echo $c->v;
"#,
    );
    assert_heap_clean(output, "140");
}

/// #498: overwriting one PHP array element releases its previous Mixed box.
#[test]
fn test_issue_551_array_element_overwrite_leaves_heap_clean() {
    let output = compile_and_run_with_heap_debug(
        r#"<?php
$a = [0];
$i = $argc;
while ($i < 50) {
    $a[0] = $i * 3 - 7;
    $i = $i + 1;
}
echo $a[0];
"#,
    );
    assert_heap_clean(output, "140");
}
