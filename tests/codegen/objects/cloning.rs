//! Purpose:
//! Integration or regression tests for PHP object cloning codegen.
//! Covers shallow object copies, declared property slots, stdClass dynamic properties, and `__clone` hooks.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures compile to native binaries and compare stdout against PHP clone semantics.

use super::*;

/// Verifies cloning declared scalar/string properties creates an independent object slot copy.
#[test]
fn test_clone_copies_declared_properties_independently() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public int $n = 1;
    public string $label = "one";
}
$a = new Item();
$b = clone $a;
$b->n = 2;
$b->label = "two";
echo $a->n . ":" . $a->label . "|" . $b->n . ":" . $b->label;
"#,
    );
    assert_eq!(out, "1:one|2:two");
}

/// Verifies `__clone()` is invoked after the shallow copy and mutates the clone, not the source.
#[test]
fn test_clone_invokes_magic_clone_on_the_copy() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public int $n = 1;
    public function __clone(): void {
        echo "hook;";
        $this->n = $this->n + 10;
    }
}
$a = new Counter();
$b = clone $a;
echo $a->n . "|" . $b->n;
"#,
    );
    assert_eq!(out, "hook;1|11");
}

/// Verifies `__clone()` can replace a string property without corrupting the source object.
#[test]
fn test_clone_persists_string_property_before_magic_clone_mutation() {
    let out = compile_and_run(
        r#"<?php
class LabelBox {
    public string $label = "A";
    public function __clone(): void {
        $this->label = $this->label . ":copy";
    }
}
$a = new LabelBox();
$b = clone $a;
echo $a->label . "|" . $b->label;
"#,
    );
    assert_eq!(out, "A|A:copy");
}

/// Verifies object-valued properties are shallow-copied, so nested object mutations remain shared.
#[test]
fn test_clone_keeps_nested_objects_shared() {
    let out = compile_and_run(
        r#"<?php
class Child {
    public int $x = 1;
}
class Boxed {
    public Child $child;
    public function __construct() {
        $this->child = new Child();
    }
}
$a = new Boxed();
$b = clone $a;
$b->child->x = 7;
echo $a->child->x . "|" . $b->child->x;
"#,
    );
    assert_eq!(out, "7|7");
}

/// Verifies cloning preserves a declared object union for a following fluent method call.
#[test]
fn test_clone_preserves_object_union_for_fluent_dispatch() {
    let out = compile_and_run(
        r#"<?php
final class CloneRoute {
    public string $path = '';

    public function setPath(string $path): self {
        $this->path = $path;
        return $this;
    }
}

final class CloneCollection {}

final class CloneHolder {
    private CloneRoute|CloneCollection $route;

    public function __construct() {
        $this->route = new CloneRoute();
    }

    public function route(string $path): CloneRoute {
        return (clone $this->route)->setPath($path);
    }
}

$route = (new CloneHolder())->route('/ready');
echo $route->path;
"#,
    );
    assert_eq!(out, "/ready");
}

/// Verifies stdClass dynamic properties are copied into a separate hash table during cloning.
#[test]
fn test_clone_copies_stdclass_dynamic_properties_independently() {
    let out = compile_and_run(
        r#"<?php
$a = new stdClass();
$a->name = "source";
$b = clone $a;
$b->name = "copy";
$b->extra = "new";
echo $a->name . "|" . $b->name . "|" . (isset($a->extra) ? "Y" : "N");
"#,
    );
    assert_eq!(out, "source|copy|N");
}

/// Verifies cloning through an interface uses runtime class metadata rather than treating the
/// interface name as a concrete object layout.
#[test]
fn test_clone_interface_typed_receiver_dispatches_runtime_class() {
    let out = compile_and_run(
        r#"<?php
interface CloneView {
    public function value(): string;
}

class CloneViewImpl implements CloneView {
    public string $name = 'source';

    public function value(): string {
        return $this->name;
    }

    public function __clone(): void {
        $this->name = 'clone';
    }
}

function duplicate(CloneView $value): CloneView {
    return clone $value;
}

$source = new CloneViewImpl();
$clone = duplicate($source);
echo $source->value() . '|' . $clone->value();
"#,
    );
    assert_eq!(out, "source|clone");
}

/// Verifies cloning an untyped parameter checks and dispatches on its runtime object class.
#[test]
fn test_clone_mixed_receiver_dispatches_runtime_class() {
    let out = compile_and_run(
        r#"<?php
class DynamicCloneValue {
    public string $name = 'source';

    public function __clone(): void {
        $this->name = 'clone';
    }
}

function duplicate_dynamic($value) {
    return clone $value;
}

$source = new DynamicCloneValue();
$clone = duplicate_dynamic($source);
echo $source->name . '|' . $clone->name;
"#,
    );
    assert_eq!(out, "source|clone");
}

/// Verifies a clone whose source is untyped remains gradual when passed to a concrete object parameter.
#[test]
fn test_clone_mixed_result_can_flow_to_concrete_object_parameter() {
    let out = compile_and_run(
        r#"<?php
class GradualCloneValue {
    public string $name = 'ready';
}

function duplicate_gradual($value) {
    return clone $value;
}

function read_gradual_clone(GradualCloneValue $value): string {
    return $value->name;
}

echo read_gradual_clone(duplicate_gradual(new GradualCloneValue()));
"#,
    );
    assert_eq!(out, "ready");
}
