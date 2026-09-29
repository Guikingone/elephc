//! Purpose:
//! Regression tests for PHP references stored in array elements: write-through, copy sharing and
//! unwrapping, traversal, rebinding, elements bound as reference sources, literal references, and
//! the by-reference `foreach` that shares its elements.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output is `php -n` 8.5's own output for the same fixture.
//! - An element that is a reference holds a marker box `{13, cell, 7}` over a refcounted cell; the
//!   heap is asserted clean wherever the fixture leaves nothing behind.

use crate::support::*;

/// A plain write to an element that is a reference changes the referenced variable, and the 
/// variable's writes show in the element. The array used to be cast back to `int` elements, so 
/// the reference was never stored.
#[test]
fn test_an_indexed_element_bound_to_a_local_is_written_through() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $a = [0, 0]; $a[1] = &$x; $a[1] = 2; echo $x, "\n"; $x = 3; echo $a[1], "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"2
3
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same through a hash of strings: the write used to replace the reference with the plain 
/// value.
#[test]
fn test_a_hash_element_bound_to_a_string_is_written_through() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = "a"; $a = ["k" => "z"]; $a["k"] = &$x; $a["k"] = "b"; echo $x, "\n"; $x = "c"; echo $a["k"], "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"b
c
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A write under a key only known at run time reaches the referenced variable too.
#[test]
fn test_a_runtime_keyed_write_goes_through_a_reference_element() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $a = ["p" => 0]; $a["p"] = &$x; foreach (["p"] as $k) { $a[$k] = 5; } echo $x, "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"5
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `$a["k"][] = 2` appends to the array the element references.
#[test]
fn test_a_nested_write_goes_through_a_reference_element() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = [1]; $a = []; $a["k"] = &$x; $a["k"][] = 2; echo count($x), "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"2
"#
    );
}

/// Copying an array keeps a reference element shared between both copies, as PHP does.
#[test]
fn test_a_copy_shares_the_reference() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $a = [0]; $a[0] = &$x; $b = $a; $x = 9; echo $b[0], "\n"; $b[0] = 8; echo $x, "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"9
8
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// DeepClone detects a reference by writing through a copy and reading the original back.
#[test]
fn test_the_copy_detection_trick_sees_a_shared_reference() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $v = ["a" => 5]; $v["r"] = &$x; $refs = $v; $s = new stdClass; foreach ($v as $k => $_) { $refs[$k] = $s; echo $k, ":", $v[$k] === $s ? "ref" : "val", "\n"; } } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"a:val
r:ref
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A reference whose only holder is the array is copied by VALUE (`zend_array_dup`), so a write 
/// to the copy leaves the original alone.
#[test]
fn test_a_copy_unwraps_a_reference_nobody_else_holds() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $a = [1]; $x = &$a[0]; unset($x); $b = $a; $b[0] = 2; echo $a[0], "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"1
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// implode, json_encode, print_r, var_dump, foreach, in_array and array_values all see the value, 
/// never the internal marker (they printed empty strings and NULL).
#[test]
fn test_traversals_read_the_referenced_value_of_a_list() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = "r"; $a = ["p"]; $a[1] = &$x; echo implode(",", $a), " ", json_encode($a), "\n"; print_r($a); var_dump($a); foreach ($a as $v) { echo $v; } echo "\n", in_array("r", $a) ? "y" : "n", " ", count($a), " ", array_values($a)[1], "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"p,r ["p","r"]
Array
(
    [0] => p
    [1] => r
)
array(2) {
  [0]=>
  string(1) "p"
  [1]=>
  &string(1) "r"
}
pr
y 2 r
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The same for a hash.
#[test]
fn test_traversals_read_the_referenced_value_of_a_hash() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = "r"; $a = ["p" => "q"]; $a["s"] = &$x; echo implode(",", $a), " ", json_encode($a), "\n"; print_r($a); foreach ($a as $k => $v) { echo $k, $v; } echo "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"q,r {"p":"q","s":"r"}
Array
(
    [p] => q
    [s] => r
)
pqsr
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `$a[0] = &$y` after `$a[0] = &$x` rebinds the element; the next write reaches `$y` only.
#[test]
fn test_rebinding_an_element_moves_it_to_the_new_variable() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $y = 2; $a = [0]; $a[0] = &$x; $a[0] = &$y; $a[0] = 7; echo $x, $y, "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"17
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `unset($a[0])` drops the element's share only; a later write to the key creates a new element.
#[test]
fn test_unsetting_a_reference_element_frees_the_variable() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $x = 1; $a = [0]; $a[0] = &$x; unset($a[0]); $a[0] = 5; echo $x, "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"1
"#
    );
}

/// `$r = &$a[0]` makes the element itself the reference: growing the array no longer strands `$r` 
/// on the old storage, and a copy of the array shares it.
#[test]
fn test_a_local_bound_to_an_element_survives_growth_and_copies() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $a = [1, 2]; $r = &$a[0]; $r = 10; for ($i = 0; $i < 40; $i++) { $a[] = $i; } $r = 11; $b = $a; $r = 12; echo $a[0], " ", $b[0], " ", count($a), "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"12 12 42
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The bound local owns a share of the reference, so it keeps the value after the hash is gone.
#[test]
fn test_a_local_bound_to_a_hash_element_outlives_the_hash() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $h = ["k" => "v"]; $x = &$h["k"]; unset($h); echo $x, "\n"; $x = "w"; echo $x, "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"v
w
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `$v["a"] = &$refs[1]` makes both elements one reference (it did not compile).
#[test]
fn test_an_element_can_be_bound_to_another_element() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $refs = [1 => "one", 2 => "two"]; $v = ["a" => 0, "b" => 0]; $v["a"] = &$refs[1]; $refs[1] = "ONE"; $v["b"] = &$refs[2]; $v["b"] = "TWO"; var_dump($v, $refs); } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"array(2) {
  ["a"]=>
  &string(3) "ONE"
  ["b"]=>
  &string(3) "TWO"
}
array(2) {
  [1]=>
  &string(3) "ONE"
  [2]=>
  &string(3) "TWO"
}
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `[&$refs["a"], $v, &$v]` stores two references and one value (only superglobals compiled).
#[test]
fn test_array_literals_hold_references() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $refs = ["a" => 1, "b" => 2]; $v = 10; $pool = [&$refs["a"], $v, &$v]; $pool[0] = 99; $pool[2] = 77; var_dump($refs, $v, $pool); } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"array(2) {
  ["a"]=>
  &int(99)
  ["b"]=>
  int(2)
}
int(77)
array(3) {
  [0]=>
  &int(99)
  [1]=>
  int(10)
  [2]=>
  &int(77)
}
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The DeepClone `prepare()` shape: a literal of references appended inside a loop.
#[test]
fn test_literal_references_built_in_a_loop() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $refs = [1, 2]; $pool = []; foreach ([0, 1] as $k) { $value = $k * 10; $pool[] = [&$refs[$k], $value, &$value]; } $pool[0][0] = "x"; $pool[1][2] = "y"; var_dump($refs, $pool); } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"array(2) {
  [0]=>
  &string(1) "x"
  [1]=>
  &int(2)
}
array(2) {
  [0]=>
  array(3) {
    [0]=>
    &string(1) "x"
    [1]=>
    int(0)
    [2]=>
    &string(1) "y"
  }
  [1]=>
  array(3) {
    [0]=>
    &int(2)
    [1]=>
    int(10)
    [2]=>
    &string(1) "y"
  }
}
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A literal reference to a local owns its share of the cell, so it outlives the frame.
#[test]
fn test_a_literal_reference_escapes_its_function() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function make() { $x = 5; return [&$x]; } function f() { $a = make(); $b = $a; $b[0] = 6; echo $a[0], "\n"; } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"5
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// DeepClone's `$values[$k] = &$value; unset($value);` leaves each element holding its own value.
#[test]
fn test_the_break_hard_reference_loop() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $values = [1, 2, 3]; foreach ($values as $k => $value) { $values[$k] = &$value; unset($value); } $values[0] = "z"; var_dump($values); } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"array(3) {
  [0]=>
  string(1) "z"
  [1]=>
  int(2)
  [2]=>
  int(3)
}
"#
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// DeepClone's scope builder: `foreach ($vars as $name => &$value) { $scoped[..][$name] = 
/// &$value; }` makes each element of `$vars` a reference shared with `$scoped`.
#[test]
fn test_a_by_reference_foreach_shares_its_elements() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f() { $scoped = []; $vars = ["x" => 1, "y" => 2]; foreach ($vars as $name => &$value) { $scoped["S"][$name] = &$value; } unset($value); $scoped["S"]["x"] = 100; var_dump($vars, $scoped); } f();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"array(2) {
  ["x"]=>
  &int(100)
  ["y"]=>
  &int(2)
}
array(1) {
  ["S"]=>
  array(2) {
    ["x"]=>
    &int(100)
    ["y"]=>
    &int(2)
  }
}
"#
    );
}
