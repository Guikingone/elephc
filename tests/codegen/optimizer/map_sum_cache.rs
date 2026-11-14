//! Purpose:
//! End-to-end coverage for caching a loop-invariant `array_sum(array_map(...))` before its loop.
//!
//! Called from:
//! - The codegen optimizer suite through `optimizer`.
//!
//! Key details:
//! - The cached form executes the map and sum once, then only accumulates the hidden Mixed local;
//!   the normal local epilogue must release it (heap debug stays clean).
//! - A mapped local observed after the loop, a source written inside the loop, and a bound that
//!   may not run a single iteration must keep the per-iteration map.

use crate::support::*;

/// Emits the optimized EIR text for one fixture.
fn optimized_ir(source: &str) -> String {
    let dir = make_cli_test_dir("elephc_map_sum_cache_ir");
    let path = dir.join("main.php");
    fs::write(&path, source).expect("write map sum cache fixture");
    let output = elephc_cli_command(&dir)
        .arg("--emit-ir")
        .arg(&path)
        .output()
        .expect("run elephc --emit-ir");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(dir);
    String::from_utf8(output.stdout).expect("EIR is UTF-8")
}

/// Returns the first `for` body slice of an EIR module dump.
fn first_loop_body(ir: &str) -> &str {
    ir.split("for.body")
        .nth(1)
        .expect("fixture has a for body")
        .split("for.update")
        .next()
        .expect("fixture has a for update")
}

/// A loop-invariant map+sum runs once and then only accumulates its cached Mixed value.
#[test]
fn test_map_sum_cache_hoists_invariant_pair() {
    let source = r#"<?php
$values = [1, 2, 3];
$sum = 0;
for ($i = 0; $i < 3 + $argc; $i++) {
    $mapped = array_map(fn($v) => $v * 2, $values);
    $sum += array_sum($mapped);
}
echo $sum, "\n";
"#;
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "48\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
    let ir = optimized_ir(source);
    let body = first_loop_body(&ir);
    assert!(!body.contains("runtime.array_map"), "{body}");
    assert!(!body.contains("runtime.array_sum"), "{body}");
}

/// A mapped local read after the loop keeps the per-iteration map.
#[test]
fn test_map_sum_cache_keeps_map_observed_after_loop() {
    let source = r#"<?php
$values = [1, 2, 3];
$sum = 0;
for ($i = 0; $i < 3 + $argc; $i++) {
    $mapped = array_map(fn($v) => $v * 2, $values);
    $sum += array_sum($mapped);
}
echo $sum, "|", implode(",", $mapped), "\n";
"#;
    assert_eq!(compile_and_run(source), "48|2,4,6\n");
    let ir = optimized_ir(source);
    assert!(first_loop_body(&ir).contains("runtime.array_map"), "{ir}");
}

/// A source written inside the loop keeps the per-iteration map.
#[test]
fn test_map_sum_cache_keeps_mutated_source() {
    let source = r#"<?php
$sum = 0;
for ($i = 0; $i < 3 + $argc; $i++) {
    $values = [1, 2, 3];
    $mapped = array_map(fn($v) => $v * 2, $values);
    $sum += array_sum($mapped);
}
echo $sum, "\n";
"#;
    assert_eq!(compile_and_run(source), "48\n");
    let ir = optimized_ir(source);
    assert!(first_loop_body(&ir).contains("runtime.array_map"), "{ir}");
}

/// A bound that may run zero times keeps the map inside the loop.
#[test]
fn test_map_sum_cache_keeps_nonpositive_bound() {
    let source = r#"<?php
$values = [1, 2, 3];
$sum = 0;
for ($i = 0; $i < $argc - 10; $i++) {
    $mapped = array_map(fn($v) => $v * 2, $values);
    $sum += array_sum($mapped);
}
echo $sum, "\n";
"#;
    assert_eq!(compile_and_run(source), "0\n");
    let ir = optimized_ir(source);
    assert!(first_loop_body(&ir).contains("runtime.array_map"), "{ir}");
}
