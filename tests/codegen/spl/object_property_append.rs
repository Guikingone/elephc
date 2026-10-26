//! Purpose:
//! Regressions for SPL numeric appends into object property tables.
//!
//! Called from:
//! - The SPL codegen integration suite.
//!
//! Key details:
//! - Integer property buckets stay distinct from string property names such as "0".
//! - Projection, visibility and unset behavior follow the pinned PHP oracle.

use crate::support::compile_and_run_with_heap_debug;

#[test]
fn test_spl_object_property_append_numeric_buckets() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$object = new stdClass();
$wrapper = new ArrayObject($object);
$wrapper[] = 'first';
$wrapper[] = 'second';
var_dump((array) $object, $wrapper->getArrayCopy());
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, r#"array(2) {
  [0]=>
  string(5) "first"
  [1]=>
  string(6) "second"
}
array(2) {
  [0]=>
  string(5) "first"
  [1]=>
  string(6) "second"
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_property_append_numeric_unset_visibility() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$object = new stdClass();
$wrapper = new ArrayObject($object);
$wrapper[] = 'first';
unset($wrapper[0]);
$wrapper[] = 'second';
$wrapper->offsetSet(null, 'third');
var_dump((array) $object, $wrapper->getArrayCopy());
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, r#"array(3) {
  [0]=>
  string(5) "first"
  [1]=>
  string(6) "second"
  [2]=>
  string(5) "third"
}
array(3) {
  [0]=>
  string(5) "first"
  [1]=>
  string(6) "second"
  [2]=>
  string(5) "third"
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_property_append_integer_and_string_key_distinction() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$object = new stdClass();
$wrapper = new ArrayObject($object);
$wrapper[] = 'first';
var_dump($wrapper[0], isset($wrapper[0]), $object->{'0'});
$wrapper[0] = 'changed';
var_dump((array) $object, $wrapper->getArrayCopy());
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, r#"NULL
bool(false)
NULL
array(1) {
  [0]=>
  string(7) "changed"
}
array(2) {
  [0]=>
  string(5) "first"
  ["0"]=>
  string(7) "changed"
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// SPL copies include visibility-mangled declared properties without changing public projections.
#[test]
fn test_spl_object_property_append_raw_copy_preserves_declared_visibility() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class C { public $a = 1; protected $b = 2; private $c = 3; }
$object = new C();
$wrapper = new ArrayObject($object);
$copy = $wrapper->getArrayCopy();
echo count($copy), ':', $copy['a'], ':', $copy["\0*\0b"], ':', $copy["\0C\0c"], "\n";
echo count(get_object_vars($object)), ':', count((array) $object), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3:1:2:3\n1:3\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
/// Checks the oracle-defined behavior for another backing-store shape.
#[test]
fn test_spl_object_property_append_nested_wrapper_forwarding() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$object = new stdClass();
$inner = new ArrayIterator($object);
$outer = new ArrayObject($inner);
$outer[] = 42;
var_dump((array) $object, $inner->getArrayCopy(), $outer->getArrayCopy());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, r#"array(1) {
  [0]=>
  int(42)
}
array(1) {
  [0]=>
  int(42)
}
array(1) {
  [0]=>
  int(42)
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Checks the oracle-defined behavior for another backing-store shape.
#[test]
fn test_spl_object_property_append_named_class_tail() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class Backing { public $existing = 'original'; }
$object = new Backing();
$wrapper = new ArrayObject($object);
$wrapper[] = 42;
var_dump($wrapper['existing'], $wrapper[0], (array) $object, $wrapper->getArrayCopy());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, r#"string(8) "original"
NULL
array(2) {
  ["existing"]=>
  string(8) "original"
  [0]=>
  int(42)
}
array(2) {
  ["existing"]=>
  string(8) "original"
  [0]=>
  int(42)
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Checks the oracle-defined behavior for another backing-store shape.
#[test]
fn test_spl_object_property_append_self_backed_visibility() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$wrapper = new ArrayObject([]);
$wrapper->exchangeArray($wrapper);
$wrapper[] = 'integer';
var_dump($wrapper[0], isset($wrapper[0]));
$wrapper[0] = 'named';
var_dump($wrapper->getArrayCopy());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, r#"NULL
bool(false)
array(2) {
  [0]=>
  string(7) "integer"
  ["0"]=>
  string(5) "named"
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves backing-table ownership across retained projections and independent objects.
#[test]
fn test_spl_object_property_append_snapshot_value_owners() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class ValueOwner { public function __destruct() { echo 'drop|'; } }
$object = new stdClass();
$wrapper = new ArrayObject($object);
$wrapper[] = ['child' => new ValueOwner()];
$snapshot = $wrapper->getArrayCopy();
unset($wrapper, $object);
gc_collect_cycles();
echo 'held|';
unset($snapshot);
gc_collect_cycles();
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, r#"held|drop|done"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves backing-table ownership across retained projections and independent objects.
#[test]
fn test_spl_object_property_append_cloned_object_table_cow() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$original = new stdClass();
$first = new ArrayObject($original);
$first[] = 'first';
$copy = clone $original;
$second = new ArrayObject($copy);
$second[] = 'second';
echo count($first->getArrayCopy()), ':', count($second->getArrayCopy()), "\n";
var_dump($first->getArrayCopy(), $second->getArrayCopy());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, r#"1:2
array(1) {
  [0]=>
  string(5) "first"
}
array(2) {
  [0]=>
  string(5) "first"
  [1]=>
  string(6) "second"
}
"#, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Iterator/count expose public properties while raw copies retain all declared properties.
#[test]
fn test_spl_object_property_append_iteration_uses_public_raw_view() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class PublicView { public $a = 1; protected $b = 2; private $c = 3; }
$object = new PublicView();
foreach ([new ArrayObject($object), new ArrayIterator($object)] as $wrapper) {
    echo count($wrapper), ':', count($wrapper->getArrayCopy()), '|';
    foreach ($wrapper as $key => $value) { echo $key, '=', $value, '|'; }
}
$object = new stdClass();
$wrapper = new ArrayObject($object);
$wrapper[] = 'integer';
$wrapper[0] = 'named';
echo count($wrapper), '|';
foreach ($wrapper as $key => $value) { var_dump($key, $value); }
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:3|a=1|1:3|a=1|2|int(0)\nstring(7) \"integer\"\nstring(1) \"0\"\nstring(5) \"named\"\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
