//! Purpose:
//! Verifies ownership of fresh objects across successful and failed constructors.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Failed constructors retire their receiver without invoking its PHP destructor.
//! - Static allocation and construction with nested arguments must retire their owners.

use crate::support::compile_and_run_with_heap_debug;

/// Fused construction releases an incomplete receiver after each constructor exception.
#[test]
fn test_gc_constructor_throw_releases_fused_receiver() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Broken {
    public function __construct() { throw new Exception('stop'); }
    public function __destruct() { echo 'invalid|'; }
}
for ($i = 0; $i < 3; $i++) {
    try { new Broken(); }
    catch (Exception $error) { echo $error->getMessage(), '|'; unset($error); }
}
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "stop|stop|stop|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Failed construction retires an owner stored in the incomplete receiver's property.
#[test]
fn test_gc_constructor_throw_releases_receiver_property_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Child {
    public function __destruct() { echo 'child|'; }
}
class Broken {
    public Child $child;
    public function __construct() { $this->child = new Child(); throw new Exception('stop'); }
    public function __destruct() { echo 'invalid|'; }
}
try { new Broken(); }
catch (Exception $error) { echo $error->getMessage(), '|'; unset($error); }
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "child|stop|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Removing a construction guard on success leaves the receiver's normal owner and destructor.
#[test]
fn test_gc_constructor_success_preserves_receiver_and_nested_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Child {
    public function __destruct() { echo 'child|'; }
}
class Good {
    public Child $child;
    public function __construct(Child $child) { $this->child = $child; echo 'made|'; }
    public function __destruct() { echo 'parent|'; }
}
$parent = new Good(new Child());
unset($parent);
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "made|parent|child|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
