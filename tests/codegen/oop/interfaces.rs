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

/// Verifies a class can satisfy a static interface method contract.
///
/// Fixture: interface `StaticMaker` declares `public static make(...)`;
/// `StaticWidget` implements it. The test also checks ReflectionClass and
/// ReflectionMethod expose the method as static.
#[test]
fn test_static_interface_method_contract_is_supported() {
    let out = compile_and_run(
        r#"<?php
interface StaticMaker {
    public static function make(string $name): string;
}

class StaticWidget implements StaticMaker {
    public static function make(string $name): string {
        return "W:" . $name;
    }
}

echo StaticWidget::make("box");
echo ":";
$interface = new ReflectionClass(StaticMaker::class);
echo $interface->hasMethod("make") ? "H" : "h";
echo ":";
$listed = $interface->getMethods()[0];
echo $listed->getName();
echo ":";
echo $listed->isStatic() ? "S" : "s";
echo ":";
echo $listed->getNumberOfParameters();
echo ":";
$method = new ReflectionMethod(StaticMaker::class, "make");
echo $method->isStatic() ? "S" : "s";
echo ":";
echo $method->getName();
echo ":";
echo (new ReflectionClass(StaticWidget::class))->implementsInterface(StaticMaker::class) ? "Y" : "N";
"#,
    );
    assert_eq!(out, "W:box:H:make:S:1:S:make:Y");
}

/// Verifies an abstract class may defer a static interface method to a concrete child.
///
/// Fixture: `AbstractStaticLabel` implements `StaticLabel` but leaves the
/// static contract abstract; `ConcreteStaticLabel` provides it and is callable.
#[test]
fn test_abstract_class_can_defer_static_interface_method_to_child() {
    let out = compile_and_run(
        r#"<?php
interface StaticLabel {
    public static function label(): string;
}

abstract class AbstractStaticLabel implements StaticLabel {
}

class ConcreteStaticLabel extends AbstractStaticLabel {
    public static function label(): string {
        return "ready";
    }
}

echo ConcreteStaticLabel::label();
"#,
    );
    assert_eq!(out, "ready");
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
    assert_eq!(out, "WIDGET\nproduct\nA-42\n1\n\n");
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
/// Verifies a PHP 8.3+ static interface method: an interface may declare a `static` method,
/// and an implementing class satisfies it with a static method, dispatched by class.
/// Fixture: interface `Previewable` with `static previews(): array`, final `C` implementing it.
#[test]
fn test_static_interface_method() {
    let out = compile_and_run(
        r#"<?php
interface Previewable {
    public static function previews(): array;
}

final class C implements Previewable {
    public static function previews(): array {
        return ['a', 'b', 'c'];
    }
}

echo implode(',', C::previews());
"#,
    );
    assert_eq!(out, "a,b,c");
}

/// Verifies a concrete child satisfies a static interface method when the interface is
/// implemented by an abstract parent class, and `#[\Override]` on the child's static
/// implementation resolves through the parent's inherited interfaces.
#[test]
fn test_static_interface_method_via_abstract_parent() {
    let out = compile_and_run(
        r#"<?php
interface Previewable {
    public static function previews(): array;
}

abstract class Base implements Previewable {
}

class C extends Base {
    #[\Override]
    public static function previews(): array {
        return ['x', 'y'];
    }
}

echo implode(',', C::previews());
"#,
    );
    assert_eq!(out, "x,y");
}

/// Verifies `#[\Override]` is accepted on a static interface-method implementation
/// (the override target is the interface's static method, matched via `InterfaceInfo.static_methods`).
#[test]
fn test_override_on_static_interface_method() {
    let out = compile_and_run(
        r#"<?php
interface Previewable {
    public static function previews(): array;
}

final class C implements Previewable {
    #[\Override]
    public static function previews(): array {
        return ['a', 'b'];
    }
}

echo implode(',', C::previews());
"#,
    );
    assert_eq!(out, "a,b");
}

/// An implementation may return a NARROWER type than the interface declares — the PSR-7 shape
/// `withX(): static` (resolving to the class) against an interface-typed return. The class
/// under validation is mid-construction when conformance runs, so the covariance is proven
/// from the conformance context itself. Byte-parity vs PHP 8.5.
#[test]
fn test_interface_covariant_self_return() {
    let out = compile_and_run(
        "<?php interface I { public function w(): I; } final class C implements I { public function w(): static { return $this; } } echo (new C())->w() instanceof C ? 'ok' : 'no';",
    );
    assert_eq!(out, "ok");
}

/// A static implementation may return its concrete class against an interface return contract.
#[test]
fn test_static_interface_covariant_self_return() {
    let out = compile_and_run(
        r#"<?php
interface Maker {
    public static function make(): Maker;
}
final class Product implements Maker {
    public static function make(): static { return new static(); }
}
echo Product::make() instanceof Product ? 'ok' : 'no';
"#,
    );
    assert_eq!(out, "ok");
}

/// Parent method returns the parent class; child may override with `static` / self (covariant).
#[test]
fn test_class_covariant_self_return_override() {
    let out = compile_and_run(
        "<?php class Base { public function w(): Base { return $this; } } class Child extends Base { public function w(): static { return $this; } } echo (new Child())->w() instanceof Child ? 'ok' : 'no';",
    );
    assert_eq!(out, "ok");
}

/// Verifies an inherited interface method returning `static` stays typed as the child interface.
#[test]
fn test_interface_late_static_return_stays_receiver() {
    let out = compile_and_run(
        r#"<?php
interface Message {
    public function withHeader(string $value): static;
}
interface Request extends Message {
    public function withMethod(string $method): static;
    public function method(): string;
}
final class Req implements Request {
    public function __construct(private string $method = 'GET') {}
    public function withHeader(string $value): static { return new static($this->method); }
    public function withMethod(string $method): static { return new static($method); }
    public function method(): string { return $this->method; }
}
function chain(Request $request): string {
    return $request->withHeader('x-trace')->withMethod('POST')->method();
}
echo chain(new Req());
"#,
    );
    assert_eq!(out, "POST");
}

/// Verifies an implementation may covariantly narrow `static|false` to `static`.
#[test]
fn test_interface_late_static_union_can_narrow_to_static() {
    let out = compile_and_run(
        r#"<?php
interface MaybeCopyable {
    public function copy(): static|false;
}
final class AlwaysCopyable implements MaybeCopyable {
    public function copy(): static { return $this; }
    public function label(): string { return "copy"; }
}
echo (new AlwaysCopyable())->copy()->label();
"#,
    );
    assert_eq!(out, "copy");
}

/// PHP-faithful lenient dispatch: a method call on an interface-typed receiver where the method is
/// declared only on implementors (absent from the interface) dispatches on the runtime class, and
/// each concrete implementor runs its OWN method. `speak()` is not on `Animal`; `Dog` and `Cat`
/// each declare it. The `Animal`-typed parameter dispatches by runtime class id to the correct
/// implementation (`php` verified: "woof|meow").
#[test]
fn test_interface_receiver_subtype_method_dispatches_per_runtime_class() {
    let out = compile_and_run(
        r#"<?php
interface Animal { public function name(): string; }
class Dog implements Animal { public function name(): string { return "dog"; } public function speak(): string { return "woof"; } }
class Cat implements Animal { public function name(): string { return "cat"; } public function speak(): string { return "meow"; } }
function sound(Animal $a): string { return $a->speak(); }
echo sound(new Dog()), "|", sound(new Cat());
"#,
    );
    assert_eq!(out, "woof|meow");
}

/// PHP-faithful lenient dispatch across a chained ancestor-typed return: `withHeader(): Message`
/// keeps its declared `Message` return (not refined to the implementor), and the chained
/// `->requestOnly()` on that `Message`-typed value dispatches on the runtime class — running when the
/// concrete object declares it. Here `Req::withHeader()` returns `$this` (a `Req`, which HAS
/// `requestOnly`), so it runs and prints "req" (`php` verified).
#[test]
fn test_interface_ancestor_return_chained_method_runs_when_present() {
    let out = compile_and_run(
        r#"<?php
interface Message { public function withHeader(): Message; }
interface Request extends Message { public function requestOnly(): string; }
class Req implements Request {
    public function withHeader(): Message { return $this; }
    public function requestOnly(): string { return "req"; }
}
function read(Request $request): string {
    return $request->withHeader()->requestOnly();
}
echo read(new Req());
"#,
    );
    assert_eq!(out, "req");
}

/// SAFETY GATE: when an interface-typed receiver holds a runtime object whose concrete class LACKS
/// the lenient-dispatched method, the call must fault CLEANLY (a catchable PHP-style `Error` /
/// controlled abort with a diagnostic and non-zero exit), never a SIGSEGV or silent garbage. Here
/// `radius()` lives only on `Circle`, but a `Square` (also a `Shape`) is passed: the runtime class
/// id matches no dispatch branch and falls through to the member-call fatal. `php` faults with "Call
/// to undefined method Square::radius()"; elephc emits the equivalent controlled member-call `Error`.
#[test]
fn test_interface_receiver_genuine_absence_faults_cleanly() {
    let err = compile_and_run_expect_failure(
        r#"<?php
interface Shape { public function area(): float; }
class Circle implements Shape { public function area(): float { return 3.14; } public function radius(): float { return 2.0; } }
class Square implements Shape { public function area(): float { return 4.0; } }
function getRadius(Shape $s): float { return $s->radius(); }
echo getRadius(new Square());
"#,
    );
    assert!(
        err.contains("Call to a member function radius() on null"),
        "expected a clean member-call Error for the genuinely-absent method, got: {err}"
    );
}

/// Verifies a property READ through an INTERFACE-typed parameter resolves against the runtime
/// class instead of failing codegen with `unknown class <Iface>`.
/// `Module::class_infos` and `Module::interface_infos` are separate maps, so every property path
/// with a named object receiver used to assume the name was a class; an interface receiver fell
/// through every branch and died in the backend even though PHP resolves such a read dynamically.
/// Fixture: `NodeI` implemented by `A` (declares `$depth`), read through a `NodeI` parameter.
#[test]
fn test_interface_typed_param_property_read_dispatches_on_runtime_class() {
    let out = compile_and_run(
        r#"<?php
interface NodeI { public function tag(): string; }
class A implements NodeI { public int $depth = 3; public function tag(): string { return "A"; } }
function r(NodeI $n): int { return $n->depth; }
echo r(new A());
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies an interface-typed receiver whose runtime class does NOT declare the property produces
/// PHP's own undefined-property warning and a null result, rather than a compile error.
/// `php -n` prints `Warning: Undefined property: B::$depth` followed by an empty string for the
/// null; only PHP's ` in FILE on line N` suffix is omitted, the campaign-wide AOT deviation.
#[test]
fn test_interface_typed_param_property_read_missing_property_warns() {
    let out = compile_and_run_capture(
        r#"<?php
interface NodeI { public function tag(): string; }
class B implements NodeI { public function tag(): string { return "B"; } }
function r(NodeI $n): void { echo $n->depth; echo "|end"; }
r(new B());
"#,
    );
    assert!(out.success, "program unexpectedly failed: {}", out.stderr);
    assert_eq!(out.stdout, "|end");
    assert!(
        out.stderr.contains("Warning: Undefined property: B::$depth"),
        "expected PHP's undefined-property warning on stderr, got: {}",
        out.stderr
    );
}

/// Verifies a NULLABLE interface receiver (`?Iface`) reads through the same implementor dispatch
/// and still emits PHP's null-receiver warning for a null argument.
/// An interface has no slot of its own, so `?Iface` cannot take `resolve_property_slot_for_class`
/// — that helper fails on its first statement with `unknown class <Iface>`.
#[test]
fn test_nullable_interface_param_property_read_handles_object_and_null() {
    let out = compile_and_run_capture(
        r#"<?php
interface NodeI { public function tag(): string; }
class A implements NodeI { public int $depth = 3; public function tag(): string { return "A"; } }
function r(?NodeI $n): void { echo $n->depth; echo "|"; }
r(new A());
r(null);
echo "end";
"#,
    );
    assert!(out.success, "program unexpectedly failed: {}", out.stderr);
    assert_eq!(out.stdout, "3||end");
    assert!(
        out.stderr
            .contains("Attempt to read property \"depth\" on null"),
        "expected PHP's null-receiver warning on stderr, got: {}",
        out.stderr
    );
}

/// Verifies the Symfony shape that motivated the fix: an interface-typed PROPERTY chained into a
/// property access under an `instanceof` guard.
/// Mirrors `Config\Definition\PrototypedArrayNode::233`, whose `instanceof` narrowing is discarded
/// before the backend, so the access really does reach an interface receiver.
#[test]
fn test_interface_typed_property_chained_under_instanceof_guard() {
    let out = compile_and_run(
        r#"<?php
interface PrototypeNodeI {}
class ArrayNodeX implements PrototypeNodeI { public array $normalizationClosures = []; }
class Holder {
    protected PrototypeNodeI $prototype;
    public function __construct(PrototypeNodeI $p) { $this->prototype = $p; }
    public function run(): int {
        if ($this->prototype instanceof ArrayNodeX) {
            $originalClosures = $this->prototype->normalizationClosures;
            return count($originalClosures);
        }
        return -1;
    }
}
echo (new Holder(new ArrayNodeX()))->run();
"#,
    );
    assert_eq!(out, "0");
}

/// NEGATIVE CONTROL for the interface property arm: a union receiver mixing an implementor and an
/// unrelated class must keep taking the boxed-Mixed candidate dispatch, not the interface arm.
/// Both members declare `$depth`, so a wrongly-routed read would still produce a number — the
/// values differ (3 vs 9) precisely so a mis-dispatch is visible rather than silently plausible.
#[test]
fn test_union_receiver_property_read_unaffected_by_interface_arm() {
    let out = compile_and_run(
        r#"<?php
interface NodeI { public function tag(): string; }
class A implements NodeI { public int $depth = 3; public function tag(): string { return "A"; } }
class Other { public int $depth = 9; }
function pick(int $k): Other|A { return $k === 0 ? new A() : new Other(); }
function r(Other|A $n): int { return $n->depth; }
echo r(pick(0)), ":", r(pick(1));
"#,
    );
    assert_eq!(out, "3:9");
}
