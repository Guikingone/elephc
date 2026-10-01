//! Purpose:
//! Verifies heap balance and reference aliasing when function-local hashes widen their values.
//!
//! Called from:
//! - The runtime GC codegen regression suite.
//!
//! Key details:
//! - Preserves the original issue 1508 fixtures and their heap-clean assertions.
//! - Shared copies must retain their entries when a reference-bound receiver widens.

use crate::support::*;

/// The widened hash writes release every value they replace or box. Regression for #1508.
#[test]
fn test_mismatched_value_write_widening_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function p(int $n): array {
    $m = ["a" => "s"];
    $m[] = $n;
    $m["b"] = 2.5;
    $m[] = null;
    $m[] = [1, "x" . $n];
    $m["c"] = "str" . $n;
    return $m;
}
$n = 100 + ($argc > 5 ? 1 : 0);
$t = 0;
for ($i = 0; $i < $n; $i++) { $t += count(p($i)); }
echo $t, "\n";
"#,
    );
    assert!(out.success, "program exited non-zero: {}", out.stderr);
    assert_eq!(out.stdout, "600\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Widening a hash through a reference alias (`$b = &$a; $b[] = 5;`) leaves another variable that
/// shares the original hash intact: the conversion takes its own owner of the borrowed cell
/// payload, so the store-back retires the old owner once. Regression for #1508.
#[test]
fn test_mismatched_value_write_through_reference_alias_keeps_shared_copy() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() {
    $a = ["x" => "s"];
    $g = $a;
    $b = &$a;
    $b[] = 5;
    return $g;
}
var_dump(f());
"#,
    );
    assert!(out.success, "program exited non-zero: {}", out.stderr);
    assert_eq!(out.stdout, "array(1) {\n  [\"x\"]=>\n  string(1) \"s\"\n}\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A mismatched write through a by-reference parameter or a `use (&$c)` capture, while another
/// variable still shares the hash, splits it once: the sharer keeps the original entries, the
/// written name sees the new ones, and nothing is released twice. Regression for #1508.
#[test]
fn test_mismatched_value_write_through_by_ref_param_and_capture_keeps_sharer() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function p(&$m) {
    $m = ["a" => "s"];
    $f = $m;
    $m[] = 5;
    $m["k"] = 1.5;
    return $f;
}
$m = ["z" => 1];
var_dump(p($m));
var_dump($m);
$c = ["x" => "s"];
$g = $c;
$fn = function () use (&$c) { $c[] = 5; };
$fn();
var_dump($g, $c);
"#,
    );
    assert!(out.success, "program exited non-zero: {}", out.stderr);
    assert_eq!(
        out.stdout,
        concat!(
            "array(1) {\n  [\"a\"]=>\n  string(1) \"s\"\n}\n",
            "array(3) {\n  [\"a\"]=>\n  string(1) \"s\"\n  [0]=>\n  int(5)\n  [\"k\"]=>\n  float(1.5)\n}\n",
            "array(1) {\n  [\"x\"]=>\n  string(1) \"s\"\n}\n",
            "array(2) {\n  [\"x\"]=>\n  string(1) \"s\"\n  [0]=>\n  int(5)\n}\n",
        )
    );
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

