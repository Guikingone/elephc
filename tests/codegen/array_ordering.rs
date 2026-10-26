//! Purpose:
//! Regressions for PHP's ordering comparison of arrays (`<`, `>`, `<=`, `>=`, `<=>`).
//!
//! Called from:
//! - `cargo test --test codegen_tests array_ordering`.
//!
//! Key details:
//! - PHP's `zend_compare_arrays`: the smaller array sorts first, then each left key in
//!   insertion order must exist on the right (an absent key reports `1`), and the first
//!   unequal element pair decides; an array ranks above other non-bool, non-null operands.
//! - The same runtime walk serves typed arrays, Mixed cells and SPL heap comparators.

use crate::support::*;

/// Typed and Mixed arrays order by count, keys and elements exactly as PHP does.
#[test]
fn test_array_ordering_follows_zend_compare_arrays() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$a = ['type' => 'array1', 'value' => 10];
$b = ['type' => 'array3', 'value' => 5];
echo $a <=> $b, ",", $b <=> $a, ",", $a <=> $a, "\n";
echo var_export($a < $b, true), ",", var_export($a > $b, true), ",", var_export($a <= $a, true), "\n";
echo [1, 2] <=> [1, 2, 3], ",", [1, 2, 3] <=> [9, 9], ",", [] <=> [], "\n";
echo ['x' => 1] <=> ['y' => 1], ",", ['y' => 1] <=> ['x' => 1], "\n";
echo [1, [2, 3]] <=> [1, [2, 4]], ",", [1, "10"] <=> [1, "9"], "\n";
echo [1] <=> 5, ",", 5 <=> [1], ",", [] <=> null, ",", [1] <=> true, "\n";
$m = [$a, $b];
echo $m[0] <=> $m[1], "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "-1,1,0\ntrue,false,true\n-1,1,0\n1,1\n-1,1\n1,-1,0,0\n-1\n");
    assert!(out.stderr.contains("leak summary: clean"), "array ordering leaked: {}", out.stderr);
}

/// SPL heaps order array elements through the same rule.
#[test]
fn test_array_ordering_drives_spl_heaps() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$h = new SplMaxHeap();
$h->insert(['type' => 'array1', 'value' => 10]);
$h->insert(['type' => 'array2', 'value' => 20]);
$h->insert(['type' => 'array3', 'value' => 5]);
foreach ($h as $e) { echo $e['type'], " "; }
echo "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "array3 array2 array1 \n");
    assert!(out.stderr.contains("leak summary: clean"), "heap ordering leaked: {}", out.stderr);
}
