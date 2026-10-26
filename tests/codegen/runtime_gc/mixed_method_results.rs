//! Purpose:
//! Checks ownership transfer when dynamic method dispatch boxes concrete return values.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Each callee returns an owned value; the result box must adopt that reference once.
//! - Shared property returns must remain alive until their final owner is released.

use crate::support::compile_and_run_with_heap_debug;

#[test]
fn test_mixed_method_results_adopt_array_and_hash_returns() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class A { public function values() { return ['a' => 1]; } }
class B { public function values() { return [1, 2]; } }
foreach ([new A(), new B()] as $receiver) {
    echo count($receiver->values()), '|';
}
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1|2|", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_mixed_method_results_retire_object_before_following_statement() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Value { public function __destruct() { echo 'drop|'; } }
class A { public function values() { return new Value(); } }
class B { public function values() { return [1, 2]; } }
foreach ([new A(), new B()] as $receiver) {
    $result = $receiver->values();
    unset($result);
    echo 'next|';
}
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "drop|next|next|", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_mixed_method_results_preserve_shared_property_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Value { public function __destruct() { echo 'drop|'; } }
class A {
    public array $data;
    public function __construct() { $this->data = ['child' => new Value()]; }
    public function values() { return $this->data; }
}
class B { public function values() { return [1, 2]; } }
$first = new A();
$receivers = [$first, new B()];
foreach ($receivers as $receiver) {
    $result = $receiver->values();
    echo count($result), '|';
    unset($result);
}
echo count($first->data), '|';
unset($receiver, $receivers, $first);
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1|2|1|drop|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
