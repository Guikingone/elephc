//! Purpose:
//! Checks release of concrete scalar-array leaves when cyclic owners are reclaimed.
//! Distinguishes concrete untyped storage from boxed PHP array properties.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Explicit collection makes the ownership checkpoint independent of shutdown policy.

use crate::support::compile_and_run_with_heap_debug;

fn check_leaf(property: &str) {
    let source = format!(r#"<?php
 gc_disable();
 class Node {{
     public mixed $peer = null;
     {property}
     public string $text;
     public function __construct() {{ $this->text = str_repeat('x', 32); }}
 }}
 $node = new Node();
 $node->peer = $node;
 unset($node);
 gc_collect_cycles();
 echo 'done';
"#);
    let out = compile_and_run_with_heap_debug(&source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_cycle_scalar_leaves_untyped_integer_array() { check_leaf("public $leaf = [1, 2];"); }

#[test]
fn test_cycle_scalar_leaves_typed_array_guard() { check_leaf("public array $leaf = [1, 2];"); }

#[test]
fn test_cycle_scalar_leaves_strings() { check_leaf("public $leaf = ['a', 'b'];"); }

#[test]
fn test_cycle_scalar_leaves_bools() { check_leaf("public $leaf = [true, false];"); }

#[test]
fn test_cycle_scalar_leaves_floats() { check_leaf("public $leaf = [1.5, 2.5];"); }

#[test]
fn test_cycle_scalar_leaves_empty() { check_leaf("public $leaf = [];"); }

#[test]
fn test_cycle_scalar_leaves_shared() {
    let out = compile_and_run_with_heap_debug(r#"<?php gc_disable();class Node {public mixed $peer=null;public $leaf=[1,2];}$n=new Node();$n->peer=$n;$held=$n->leaf;unset($n);gc_collect_cycles();echo implode(",",$held);unset($held);echo "|done";"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1,2|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_cycle_scalar_leaves_object_array_guard() {
    let out = compile_and_run_with_heap_debug(r#"<?php gc_disable();class Payload {public function __destruct(){echo "payload|";}}class Node {public mixed $peer=null;public $leaf;public function __construct(){$this->leaf=[new Payload()];}}$n=new Node();$n->peer=$n;unset($n);gc_collect_cycles();echo "done";"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "payload|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
