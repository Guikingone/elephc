//! Purpose:
//! Checks WeakMap value storage across cloning and entry references.
//!
//! Called from:
//! - The Core class codegen integration tests.
//!
//! Key details:
//! - Copies preserve object identity and detach array values through PHP COW.
//! - Managed references survive removal of their weak key and the map itself.

use crate::support::*;

/// Opaque eval clones the native map while an AOT mutation keeps the clone's array independent.
#[test]
fn test_core_weak_map_clone_cow_across_opaque_eval() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function changeNativeWeakMapValue(WeakMap $map, object $key): void { $map[$key]['n'] = 1; }
$source = '$key = new stdClass();
$map = new WeakMap();
$map[$key] = ["n" => 0];
$copy = clone $map;
changeNativeWeakMapValue($map, $key);
echo $map[$key]["n"], ":", $copy[$key]["n"];
unset($copy, $map, $key);';
eval($source . ' // ' . $argc);
"#);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "1:0", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Map cloning separates nested array mutations even across repeated clones.
#[test]
fn test_core_weak_map_clone_separates_nested_values() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$key = new stdClass();
$map = new WeakMap();
$map[$key] = ['n' => 0];
$value = $map[$key];
$copy = clone $map;
$map[$key]['n'] = 1;
$copy[$key]['n'] = 2;
echo $map[$key]['n'], ':', $value['n'], ':', $copy[$key]['n'], ';';
$again = clone $copy;
$again[$key]['n'] = 3;
echo $copy[$key]['n'], ':', $again[$key]['n'];
unset($value, $again, $copy, $map, $key);
"#);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "1:0:2;2:3", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Replacing an entry detaches its old reference, which outlives the key and map.
#[test]
fn test_core_weak_map_reference_lifetime_and_replacement() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$key = new stdClass();
$map = new WeakMap();
$map[$key] = ['n' => 0];
$reference = &$map[$key];
$reference['n'] = 2;
echo $map[$key]['n'], ':';
$map[$key] = ['n' => 3];
echo $reference['n'], ':', $map[$key]['n'], ':';
unset($key);
echo count($map), ':', $reference['n'], ':';
unset($map);
$reference['n'] = 4;
echo $reference['n'];
unset($reference);
"#);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "2:2:3:0:2:4", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A clone preserves live reference sets and separates entries whose reference is orphaned.
#[test]
fn test_core_weak_map_clone_reference_sets() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$key = new stdClass();
$map = new WeakMap();
$map[$key] = 0;
$reference = &$map[$key];
$copy = clone $map;
$reference = 2;
echo $map[$key], ':', $copy[$key], ';';
unset($reference, $copy);
$orphanCopy = clone $map;
$orphanCopy[$key] = 3;
echo $map[$key], ':', $orphanCopy[$key];
unset($orphanCopy, $map, $key);
"#);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "2:2;2:3", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
