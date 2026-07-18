//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP interfaces, including interface contract can be satisfied by concrete class, abstract base can defer method to concrete child, and class can implement multiple interfaces.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses checked-in example PHP fixtures through include_str! in addition to inline native-output assertions.

use super::*;

/// Verifies a concrete class can satisfy an interface contract by implementing all required methods.
/// Fixture: interface `Named` with method `name()`, concrete `User` implementing `Named`.
/// Asserts the method call on the concrete instance returns the expected string.
#[test]
fn test_interface_contract_can_be_satisfied_by_concrete_class() {
    let out = compile_and_run(
        r#"<?php
interface Named {
    public function name();
}

class User implements Named {
    public function name() {
        return "Ada";
    }
}

$user = new User();
echo $user->name();
"#,
    );
    assert_eq!(out, "Ada");
}

/// Verifies an abstract class can defer interface method implementation to a concrete child class.
/// Fixture: abstract `BaseGreeter` with abstract method `label()` and concrete `PersonGreeter`.
/// Asserts calling `greet()` on the concrete child triggers `label()` via `$this->label()`.
#[test]
fn test_abstract_base_can_defer_method_to_concrete_child() {
    let out = compile_and_run(
        r#"<?php
abstract class BaseGreeter {
    abstract public function label();

    public function greet() {
        return "hi " . $this->label();
    }
}

class PersonGreeter extends BaseGreeter {
    public function label() {
        return "world";
    }
}

$g = new PersonGreeter();
echo $g->greet();
"#,
    );
    assert_eq!(out, "hi world");
}

/// Verifies a class can implement multiple interfaces simultaneously.
/// Fixture: `Named` and `Tagged` interfaces, `Item` implementing both.
/// Asserts chained method calls resolve to the correct interface method on the same instance.
#[test]
fn test_class_can_implement_multiple_interfaces() {
    let out = compile_and_run(
        r#"<?php
interface Named {
    public function name();
}

interface Tagged {
    public function tag();
}

class Item implements Named, Tagged {
    public function name() {
        return "box";
    }

    public function tag() {
        return "BX";
    }
}

$item = new Item();
echo $item->name() . ":" . $item->tag();
"#,
    );
    assert_eq!(out, "box:BX");
}

/// Verifies transitive interface extension is enforced: a class must satisfy the full chain.
/// Fixture: `Labeled extends Named`, `Product implements Labeled`. Uses `strtoupper($this->name())`.
/// Asserts the method call correctly resolves through the transitive interface hierarchy.
#[test]
fn test_transitive_interface_extends_is_enforced() {
    let out = compile_and_run(
        r#"<?php
interface Named {
    public function name();
}

interface Labeled extends Named {
    public function label();
}

class Product implements Labeled {
    public function name() {
        return "widget";
    }

    public function label() {
        return strtoupper($this->name());
    }
}

$product = new Product();
echo $product->label();
"#,
    );
    assert_eq!(out, "WIDGET");
}

/// Verifies the checked-in example at `examples/interfaces/main.php` compiles and runs end-to-end.
/// Loads the PHP fixture via `include_str!`, asserts stdout matches expected multi-line output.
#[test]
fn test_example_interfaces_compiles_and_runs() {
    let out = compile_and_run(include_str!("../../../examples/interfaces/main.php"));
    // `isset(...) . "\n"`: a bool false stringifies to "" (not "0") in PHP, so the
    // post-unset isset line is empty.
    assert_eq!(out, "WIDGET\nA-42\n1\n\n");
}

/// Verifies an interface with a read-only property (`get;`) can be satisfied by a concrete property.
/// Fixture: interface `HasId` with `public int $id { get; }`, concrete `User` with int field.
/// Asserts reading the property on the concrete instance returns the expected value.
#[test]
fn test_interface_get_property_contract_is_satisfied_by_concrete_property() {
    let out = compile_and_run(
        r#"<?php
interface HasId {
    public int $id { get; }
}

class User implements HasId {
    public int $id = 42;
}

$user = new User();
echo $user->id;
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies interface property setters allow contravariant type (subclass) in implementing class.
/// Fixture: `Dog extends Animal`, interface `DogSink` with `public Dog $pet { set; }`,
/// implementing `Kennel` declares `public Animal $pet`. Sets a `Dog` instance and checks `instanceof Animal`.
/// Asserts contravariant property types are accepted per PHP semantics.
#[test]
fn test_interface_set_property_contract_allows_contravariant_type() {
    let out = compile_and_run(
        r#"<?php
class Animal {}
class Dog extends Animal {}

interface DogSink {
    public Dog $pet { set; }
}

class Kennel implements DogSink {
    public Animal $pet;
}

$kennel = new Kennel();
$kennel->pet = new Dog();
echo $kennel->pet instanceof Animal;
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies an abstract class can defer interface property implementation to a concrete child.
/// Fixture: interface `HasName` with `string $name { get; set; }`, abstract `NamedBase implements HasName`,
/// concrete `Product extends NamedBase` with a default field initializer.
/// Asserts reading the property on the concrete child resolves via the abstract's interface contract.
#[test]
fn test_abstract_class_can_defer_interface_property_to_child() {
    let out = compile_and_run(
        r#"<?php
interface HasName {
    public string $name { get; set; }
}

abstract class NamedBase implements HasName {
}

class Product extends NamedBase {
    public string $name = "widget";
}

$product = new Product();
echo $product->name;
"#,
    );
    assert_eq!(out, "widget");
}

/// Verifies a static interface method (PHP 8 `public static function x(): T;`) is satisfied by a
/// concrete class and callable through `C::method()`. Mirrors the shape of Symfony's
/// `EnvVarProcessorInterface::getProvidedTypes(): array` pattern that motivated this feature.
#[test]
fn test_static_interface_method_satisfied_by_concrete_class() {
    let out = compile_and_run(
        r#"<?php
interface EnvVarProcessorInterface {
    public static function getProvidedTypes(): array;
}

class EnvVarProcessor implements EnvVarProcessorInterface {
    public static function getProvidedTypes(): array {
        return ["string" => 1, "bool" => 2];
    }
}

$types = EnvVarProcessor::getProvidedTypes();
echo count($types) . ":" . $types["string"] . ":" . $types["bool"];
"#,
    );
    assert_eq!(out, "2:1:2");
}

/// Verifies an abstract class can defer a static interface method implementation to a concrete
/// child class, mirroring `test_abstract_base_can_defer_method_to_concrete_child` but for the
/// static contract.
#[test]
fn test_abstract_base_can_defer_static_method_to_concrete_child() {
    let out = compile_and_run(
        r#"<?php
interface Factory {
    public static function make(): int;
}

abstract class BaseFactory implements Factory {
}

class ConcreteFactory extends BaseFactory {
    public static function make(): int {
        return 7;
    }
}

echo ConcreteFactory::make();
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies covariant `self`/`static` return forms on a static interface method: the interface
/// declares `: static` and the implementor returns `new self()`, matching PHP's LSP rule that
/// `static` covariantly narrows to the receiver (`php -n` verified against the interpreter).
#[test]
fn test_static_interface_method_covariant_static_return() {
    let out = compile_and_run(
        r#"<?php
interface Buildable {
    public static function make(): static;
}

class Widget implements Buildable {
    public int $val = 0;

    public static function make(): static {
        $w = new self();
        $w->val = 9;
        return $w;
    }
}

$w = Widget::make();
echo $w->val;
"#,
    );
    assert_eq!(out, "9");
}

/// Verifies late static binding (`static::`/`new static()`) works inside a static interface
/// method's implementation, and resolves to the calling subclass rather than the declaring
/// class — the same LSB rule as any other static method, unaffected by the interface contract.
#[test]
fn test_static_interface_method_late_static_binding() {
    let out = compile_and_run(
        r#"<?php
interface Buildable {
    public static function make(): static;
}

class Base implements Buildable {
    public static function make(): static {
        return new static();
    }

    public function name(): string {
        return static::class;
    }
}

class Derived extends Base {
}

$b = Base::make();
$d = Derived::make();
echo $b->name() . ":" . $d->name();
"#,
    );
    assert_eq!(out, "Base:Derived");
}

/// Verifies static interface method calls are case-insensitive on both the class name and the
/// method name, matching PHP's case-insensitive symbol lookup rules for classes and methods.
#[test]
fn test_static_interface_method_call_is_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
interface I {
    public static function F(): int;
}

class C implements I {
    public static function f(): int {
        return 5;
    }
}

echo C::F() . ":" . c::f();
"#,
    );
    assert_eq!(out, "5:5");
}
