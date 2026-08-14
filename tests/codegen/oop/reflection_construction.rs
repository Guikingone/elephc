//! Purpose:
//! End-to-end codegen tests for ReflectionClass and ReflectionObject
//! construction helpers over reflected class metadata.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers inherited ReflectionObject construction helpers that must route
//!   through the same dynamic class-name lowering as ReflectionClass.

use super::*;

/// Verifies member reflectors accept an object and derive its declaring class at runtime.
#[test]
fn test_reflection_member_constructors_accept_object() {
    let out = compile_and_run_capture(
        r#"<?php
class RuntimeReflectionOwner {
    public const LABEL = "constant";
    public string $value = "property";
    public function run(): void {}
}

$object = new RuntimeReflectionOwner();
echo (new ReflectionMethod($object, "run"))->getName() . ":";
echo (new ReflectionProperty($object, "value"))->getName() . ":";
echo (new ReflectionClassConstant($object, "LABEL"))->getValue();
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "run:value:constant");
}

/// Verifies ReflectionClass resolves a case-insensitive class name held in a runtime string.
#[test]
fn test_reflection_class_accepts_runtime_class_string() {
    let out = compile_and_run_capture(
        r#"<?php
class DynamicReflectionTarget {}
$name = strtolower("DynamicReflectionTarget");
$reflection = new ReflectionClass($name);
echo $reflection->getName();
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "DynamicReflectionTarget");
}

/// Verifies an unknown runtime class name raises a catchable ReflectionException.
#[test]
fn test_reflection_class_runtime_unknown_name_throws() {
    let out = compile_and_run_capture(
        r#"<?php
$name = strtolower("DefinitelyMissing");
try {
    new ReflectionClass($name);
} catch (ReflectionException $error) {
    echo $error->getMessage();
}
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "Class \"definitelymissing\" does not exist");
}

/// Verifies `ReflectionObject` inherits working construction helpers from `ReflectionClass`.
#[test]
fn test_reflection_object_construction_helpers_use_runtime_class() {
    let out = compile_and_run_capture(
        r#"<?php
class ReflectObjectConstructBase {}
class ReflectObjectConstructChild extends ReflectObjectConstructBase {
    public function __construct(string $left = "L", string $right = "R") {
        echo $left . $right . "|";
    }
}

$object = new ReflectObjectConstructChild("I", "N");
$ref = new ReflectionObject($object);
$first = $ref->newInstance("A", "B");
echo get_class($first) . ":";
$second = $ref->newInstanceArgs(["right" => "Y", "left" => "X"]);
echo get_class($second) . ":";
$third = $ref->newInstance(right: "N", left: "M");
echo get_class($third) . ":";
$fourth = $ref->newInstanceWithoutConstructor();
echo get_class($fourth);
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(
        out.stdout,
        "IN|AB|ReflectObjectConstructChild:XY|ReflectObjectConstructChild:MN|ReflectObjectConstructChild:ReflectObjectConstructChild"
    );
}

/// Verifies `ReflectionClass` construction helpers support inferred constructor signatures.
#[test]
fn test_reflection_class_construction_helpers_call_inferred_constructor_signature() {
    let out = compile_and_run_capture(
        r#"<?php
class ReflectConstructInferredTarget {
    public function __construct($left, $right = "B") {
        echo $left . $right . "|";
    }
}

$ref = new ReflectionClass(ReflectConstructInferredTarget::class);
$ref->newInstance("A", "C");
$ref->newInstanceArgs(["right" => "Y", "left" => "X"]);
$ref->newInstance(right: "N", left: "M");
"#,
    );
    assert!(
        out.success,
        "program failed: stdout={:?} stderr={}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "AC|XY|MN|");
}
