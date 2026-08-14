//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP misc, including inherited constructor specializes base string property type, literal allows sibling objects with common parent, and match without default is fatal.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses checked-in example PHP fixtures through include_str! in addition to inline native-output assertions.

use super::*;

/// Verifies PHP's generic `object` parameter type accepts concrete objects and
/// preserves object-shaped ABI lowering.
#[test]
fn test_generic_object_parameter_type_accepts_concrete_object() {
    let out = compile_and_run(
        r#"<?php
class GenericObjectParam {}
function accepts_object(object $value): string {
    return is_object($value) ? "object" : "bad";
}
echo accepts_object(new GenericObjectParam());
"#,
    );
    assert_eq!(out, "object");
}

/// Verifies PHP's incomplete-object placeholder resolves as an internal nominal class.
#[test]
fn test_php_incomplete_class_type_and_instanceof_compile() {
    let out = compile_and_run(
        r#"<?php
function is_incomplete(object $value): bool {
    return $value instanceof __PHP_Incomplete_Class;
}
var_dump(class_exists("__PHP_Incomplete_Class"));
var_dump(is_incomplete(new stdClass()));
"#,
    );
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// Verifies lowercase and mixed-case `object` hints remain generic object types inside a
/// namespace instead of being rewritten to namespace-local class names.
#[test]
fn test_namespaced_generic_object_parameter_type_is_not_prefixed() {
    let out = compile_and_run(
        r#"<?php
namespace App;

class NamespacedObjectValue {}

function accepts_lower_object(object $value): string {
    return is_object($value) ? "lower" : "bad";
}

function accepts_mixed_case_object(Object $value): string {
    return is_object($value) ? "upper" : "bad";
}

$value = new NamespacedObjectValue();
echo accepts_lower_object($value) . "|" . accepts_mixed_case_object($value);
"#,
    );
    assert_eq!(out, "lower|upper");
}

/// Verifies a namespaced bare `object` receiver dispatches property reads and writes by its
/// concrete runtime class instead of resolving the pseudo-type as an empty class name.
#[test]
fn test_namespaced_generic_object_property_read_and_write_dispatch() {
    let out = compile_and_run(
        r#"<?php
namespace GenericObjectProperty;

class Box {
    public string $value = 'before';
}

function rewrite(object $object): string {
    $object->value = 'after';
    return $object->value;
}

echo rewrite(new Box());
"#,
    );
    assert_eq!(out, "after");
}

/// Verifies nullable bare-object receivers unbox and dispatch property reads and writes by the
/// concrete runtime class, including the stdClass dynamic-property fallback and a nullsafe read.
#[test]
fn test_nullable_generic_object_property_read_and_write_dispatch() {
    let out = compile_and_run(
        r#"<?php
namespace NullableGenericObjectProperty;

class Box {
    public string $value = 'before';
}

function rewrite(?object $object, string $value): mixed {
    $object->value = $value;
    return $object->value;
}

function peek(?object $object): mixed {
    return $object?->value;
}

$dynamic = new \stdClass();
$dynamic->value = 'initial';
echo rewrite(new Box(), 'class') . '|';
echo rewrite($dynamic, 'dynamic') . '|';
echo null === peek(null) ? 'null' : 'bad';
"#,
    );
    assert_eq!(out, "class|dynamic|null");
}

/// Verifies a read-only generic-object property is boxed and coerced at a declared return boundary.
#[test]
fn test_generic_object_property_read_with_declared_return() {
    let out = compile_and_run(
        r#"<?php
final class NamedPropertyObject {
    public string $name = 'ok';
}

function readName(object $value): string {
    return $value->name;
}

echo readName(new NamedPropertyObject());
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies keyed and append writes through bare `object` dispatch to an array property owner.
#[test]
fn test_generic_object_array_property_writes() {
    let out = compile_and_run(
        r#"<?php
final class GenericArrayPropertyBag {
    public array $entries = [];
}

function fillGenericArrayProperty(object $bag): string {
    $bag->entries['name'] = 'value';
    $bag->entries[] = 'tail';
    return $bag->entries['name'] . '|' . $bag->entries[0];
}

echo fillGenericArrayProperty(new GenericArrayPropertyBag());
"#,
    );
    assert_eq!(out, "value|tail");
}

/// Verifies a bare-object property implementing ArrayAccess receives keyed writes dynamically.
#[test]
fn test_generic_object_array_access_property_write() {
    let out = compile_and_run(
        r#"<?php
final class GenericArrayAccessPropertyBag {
    public object $slots;

    public function __construct() {
        $this->slots = new ArrayObject();
    }

    public function slotCount(): int {
        return $this->slots->count();
    }
}

function fillGenericArrayAccessProperty(object $bag): int {
    $bag->slots['name'] = 'value';
    return $bag->slotCount();
}

echo fillGenericArrayAccessProperty(new GenericArrayAccessPropertyBag());
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies bare-object returns are runtime-checked at nullable nominal object boundaries.
#[test]
fn test_generic_object_nominal_return_boundary() {
    let out = compile_and_run(
        r#"<?php
final class ExpectedReturnObject {}
final class UnexpectedReturnObject {}

function nominalReturn(object $value): ?ExpectedReturnObject {
    return $value;
}

echo nominalReturn(new ExpectedReturnObject()) instanceof ExpectedReturnObject ? 'ok' : 'bad';
try {
    nominalReturn(new UnexpectedReturnObject());
} catch (TypeError) {
    echo '|caught';
}
"#,
    );
    assert_eq!(out, "ok|caught");
}

/// Tests that a Child class inheriting Base's constructor properly specializes the
/// base class's string property type, so `new Child("Ada")` works without explicit
/// constructor in the child.
///
/// Fixture: Base with `$name` property and typed constructor; Child extends Base with
/// no constructor. Verifies greet() returns the passed name.
#[test]
fn test_inherited_constructor_specializes_base_string_property_type() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function greet() {
        return $this->name;
    }
}

class Child extends Base {}

$child = new Child("Ada");
echo $child->greet();
"#,
    );
    assert_eq!(out, "Ada");
}

/// Verifies constructor inference only specializes parameters that map to the same inherited
/// property, even when a sibling declares an unrelated constructor parameter at the same index.
#[test]
fn test_inherited_property_inference_does_not_retype_unrelated_constructor_parameter() {
    let out = compile_and_run(
        r#"<?php
class BaseValue {
    protected $shared;

    public function __construct($value) {
        $this->shared = $value;
    }

    public function shared() {
        return $this->shared;
    }
}

class InheritedValue extends BaseValue {}

class IndependentValue extends BaseValue {
    private $other;

    public function __construct($other) {
        $this->other = $other;
    }

    public function other() {
        return $this->other;
    }
}

$inherited = new InheritedValue(42);
$independent = new IndependentValue('ok');
echo $inherited->shared(), '|', $independent->other();
"#,
    );
    assert_eq!(out, "42|ok");
}

/// Tests that array literals can contain sibling objects that share a common parent
/// class, using a foreach loop to iterate and call a parent method.
///
/// Fixture: Animal base with `$name`; Dog and Cat subclasses; array literal with
/// `new Dog("Rex")` and `new Cat("Mia")`. Verifies both labels are printed.
#[test]
fn test_array_literal_allows_sibling_objects_with_common_parent() {
    let out = compile_and_run(
        r#"<?php
class Animal {
    public $name;

    public function __construct($name) {
        $this->name = $name;
    }

    public function label() {
        return $this->name;
    }
}

class Dog extends Animal {}
class Cat extends Animal {}

$animals = [new Dog("Rex"), new Cat("Mia")];
foreach ($animals as $animal) {
    echo $animal->label() . " ";
}
"#,
    );
    assert_eq!(out, "Rex Mia ");
}

/// Tests that a match expression without a default case produces a fatal error
/// ("unhandled match case") when the matched value has no corresponding arm.
///
/// Fixture: `$value = 3` matched against arms 1 and 2 only. Verifies fatal error
/// message is produced.
#[test]
fn test_match_without_default_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
$value = 3;
echo match($value) {
    1 => "one",
    2 => "two",
};
"#,
    );
    assert!(err.contains("unhandled match case"), "{err}");
}

/// Verifies the v017-trio example PHP fixture compiles and runs, asserting the
/// expected output "health:[ok]:missing".
///
/// Fixture: `examples/v017-trio/main.php` via `include_str!`. Regression guard for
/// trio example staying working.
#[test]
fn test_example_v017_trio_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/v017-trio/main.php"));
    assert_eq!(out, "health:[ok]:missing");
}

/// EC-10: `enum` is only a soft keyword — `class Enum {}` / `interface Enum` / `new Enum`
/// are legal PHP (vendor precedent: marc-mabe/php-enum). Byte-parity vs PHP 8.5.
#[test]
fn test_class_named_enum_declares() {
    let out = compile_and_run(
        "<?php class Enum { public function tag(): string { return 'e'; } } echo (new Enum())->tag();",
    );
    assert_eq!(out, "e");
}

/// Soft-keyword `enum` is also legal as an interface name.
#[test]
fn test_interface_named_enum_declares() {
    let out = compile_and_run(
        "<?php interface Enum { public function tag(): string; } class C implements Enum { public function tag(): string { return 'i'; } } echo (new C())->tag();",
    );
    assert_eq!(out, "i");
}
