//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP interfaces, including interface contract can be satisfied by concrete class, abstract base can defer method to concrete child, and class can implement multiple interfaces.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses checked-in example PHP fixtures through include_str! in addition to inline native-output assertions.

use super::*;

/// Verifies an UNTYPED parameter on a bodyless declaration stays `mixed`. The compiler starts an
/// untyped parameter at `int` and lets a concrete body refine it from its own call sites, but an
/// interface method has no body to refine from, so the starting point froze and became the
/// contract: a program that logged an int first then rejected the string call with
/// "parameter $level expects Int, got Str". PSR-3 declares `log($level, …)` exactly this way and
/// Symfony's `ErrorListener` calls it with a string level. PHP outputs "int:a|string:b|".
#[test]
fn test_untyped_interface_parameter_stays_mixed() {
    let out = compile_and_run(
        r#"<?php
interface Log {
    public function log($level, string $message): void;
}

abstract class BaseLog implements Log {
    abstract public function describe($level): string;
}

class Plain extends BaseLog {
    public function log($level, string $message): void {
        echo get_debug_type($level), ':', $message, '|';
    }

    public function describe($level): string { return get_debug_type($level); }
}

function emit(Log $logger, string $level): void {
    $logger->log($level, 'b');
}

$l = new Plain();
$l->log(3, 'a');
emit($l, 'critical');
echo $l->describe(1), ',', $l->describe('x');
"#,
    );
    assert_eq!(out, "int:a|string:b|int,string");
}

/// Verifies virtual/interface calls forward the hidden argc operand used by an overriding method.
#[test]
fn test_interface_dispatch_forwards_hidden_func_get_args_count() {
    let out = compile_and_run(
        r#"<?php
interface ArityLoader { public function load(mixed $resource, ?string $type = null): mixed; }
abstract class ArityLoaderBase implements ArityLoader {
    public function proxy(): int { return $this->load('base', null); }
}
final class ArityLoaderImpl extends ArityLoaderBase {
    public function load(mixed $resource, ?string $type = null): mixed {
        return func_num_args();
    }
}
function invokeArityLoader(ArityLoader $loader): int { return $loader->load('iface', null); }
$loader = new ArityLoaderImpl();
echo $loader->proxy(), '|', invokeArityLoader($loader);
"#,
    );
    assert_eq!(out, "2|2");
}

/// Verifies a generic `object` receiver dispatches a method over closed-world class candidates.
#[test]
fn test_generic_object_receiver_dispatches_known_method_candidate() {
    let out = compile_and_run(
        r#"<?php
final class GenericObjectStack {
    private array $values = [];
    public function push(int $value): void { $this->values[] = $value; }
    public function count(): int { return count($this->values); }
}
function eraseStackType(GenericObjectStack $stack): object { return $stack; }
$stack = eraseStackType(new GenericObjectStack());
$stack->push(4);
echo $stack->count();
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies an impossible narrowed-method branch compiles and remains unexecuted.
#[test]
fn test_impossible_instanceof_narrowed_method_branch_compiles() {
    let out = compile_and_run(
        r#"<?php
interface OptionalWarmer { public function warmUp(string $directory): array; }
class NonWarmableKernel {}
function optionallyWarm(NonWarmableKernel $kernel): array {
    return $kernel instanceof OptionalWarmer ? $kernel->warmUp('/tmp') : [];
}
echo 'cold';
"#,
    );
    assert_eq!(out, "cold");
}

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

/// Verifies object syntax dispatches a static interface method to the concrete receiver class.
#[test]
fn test_static_interface_method_dispatch_through_object_receiver() {
    let out = compile_and_run(
        r#"<?php
interface RuntimeStaticLabel {
    public static function label(string $suffix = "!"): string;
}
final class RuntimeStaticA implements RuntimeStaticLabel {
    public static function label(string $suffix = "!"): string { return "A" . $suffix; }
}
final class RuntimeStaticB implements RuntimeStaticLabel {
    public static function label(string $suffix = "!"): string { return "B" . $suffix; }
}
function runtimeStaticLabel(RuntimeStaticLabel $value): string {
    return $value->label("?");
}
echo runtimeStaticLabel(new RuntimeStaticA()), "|", runtimeStaticLabel(new RuntimeStaticB());
"#,
    );
    assert_eq!(out, "A?|B?");
}

/// Verifies static-syntax calls through an object invoke a static-only method.
///
/// PHP accepts `$object::method()` as well as `Class::method()`. The lowering
/// must not send this shape through the instance-only runtime callable-array
/// lookup when the receiver's class and its static method are known.
#[test]
fn test_static_method_dispatch_through_object_static_syntax() {
    let out = compile_and_run(
        r#"<?php
final class ObjectStaticSyntax {
    public static function label(string $suffix): string { return "static" . $suffix; }
}
function objectStaticSyntax(ObjectStaticSyntax $value): string {
    return $value::label("!");
}
echo objectStaticSyntax(new ObjectStaticSyntax());
"#,
    );
    assert_eq!(out, "static!");
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

/// Verifies covariant object returns do not depend on class metadata construction order.
#[test]
fn test_class_covariant_sibling_return_before_result_metadata() {
    let out = compile_and_run(
        r#"<?php
interface ResultContract {}
class ConcreteResult implements ResultContract {}
class BaseResult {}
class NarrowResult extends BaseResult {}
class ParentFactory {
    public function create(): BaseResult { return new BaseResult(); }
    public function createContract(): ResultContract { return new ConcreteResult(); }
}
class ChildFactory extends ParentFactory {
    public function create(): NarrowResult { return new NarrowResult(); }
    public function createContract(): ConcreteResult { return new ConcreteResult(); }
}
$factory = new ChildFactory();
echo ($factory->create() instanceof NarrowResult ? 'class' : 'bad');
echo '|';
echo ($factory->createContract() instanceof ConcreteResult ? 'interface' : 'bad');
"#,
    );
    assert_eq!(out, "class|interface");
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

/// Narrowed interface dispatch excludes unrelated same-name methods with incompatible ABIs.
#[test]
fn test_narrowed_interface_method_filters_unrelated_candidates() {
    let out = compile_and_run(
        r#"<?php
interface Runner {}
class StringRunner implements Runner {
    public function run(string $value): string { return $value; }
}
class Unrelated {
    public function run(?int $value): string { return 'wrong'; }
}
function invoke(Runner $runner): string {
    if ($runner instanceof StringRunner) {
        return $runner->run('ok');
    }
    return 'miss';
}
echo invoke(new StringRunner());
"#,
    );
    assert_eq!(out, "ok");
}


/// A class implements PHP's `Serializable`, which elephc never declared.
///
/// `TENTATIVE_RETURN_TYPE_METHODS` already named the interface, but it was absent from
/// `BUILTIN_INTERFACE_NAMES`, so a class implementing it sent the interpreter to `spl_autoload` —
/// and Composer cannot autoload a PHP built-in. `Twig\Profiler\Profile` is
/// `final class Profile implements \IteratorAggregate, \Serializable`, so the whole declaration
/// was refused with `class Twig\Profiler\Profile could not be declared`, killing the `--web`
/// worker while Symfony rendered its error page.
///
/// Deprecated since PHP 8.1 and still declared by the engine, which is what matters: userland code
/// implements it. Both its methods carry tentative return types, so the implementer below declares
/// a narrower `string` for `serialize()` and no type at all for `unserialize()`'s parameter.
///
/// Oracle: `php -n` prints the asserted line.
#[test]
fn test_class_implements_the_serializable_builtin_interface() {
    let out = compile_and_run(
        r#"<?php
final class Profile implements \IteratorAggregate, \Serializable {
    private array $items = ['a', 'b'];
    public function getIterator(): \Traversable { return new \ArrayIterator($this->items); }
    public function serialize(): string { return \serialize($this->__serialize()); }
    public function unserialize($data): void { $this->__unserialize(\unserialize($data)); }
    public function __serialize(): array { return [$this->items]; }
    public function __unserialize(array $data): void { [$this->items] = $data; }
}
$p = new Profile();
echo ($p instanceof Serializable ? 'yes' : 'no'), '|';
foreach ($p as $v) { echo $v; }
echo '|', $p->serialize();
"#,
    );
    assert_eq!(out, "yes|ab|a:1:{i:0;a:2:{i:0;s:1:\"a\";i:1;s:1:\"b\";}}");
}
