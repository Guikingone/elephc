//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP inheritance, including class protected members are accessible inside class methods, class protected static method is callable inside class, and inheritance dynamic dispatch uses child override.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies protected member `$value` and protected method `next()` are callable
/// from public method `reveal()` inside the same class, returning 42.
#[test]
fn test_class_protected_members_are_accessible_inside_class_methods() {
    let out = compile_and_run(
        r#"<?php
class SecretBox {
    protected $value = 41;

    protected function next() {
        return $this->value + 1;
    }

    public function reveal() {
        return $this->next();
    }
}

$box = new SecretBox();
echo $box->reveal();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies a parent scope can access protected members declared by a child class.
#[test]
fn test_parent_scope_can_access_child_protected_members() {
    let out = compile_and_run(
        r#"<?php
class ProtectedParent {
    public static function read(ProtectedChild $child): int {
        return $child->value + $child->extra();
    }
}
class ProtectedChild extends ProtectedParent {
    protected int $value = 7;

    protected function extra(): int {
        return 8;
    }
}
echo ProtectedParent::read(new ProtectedChild());
"#,
    );
    assert_eq!(out, "15");
}

/// Verifies protected static method `base()` is callable via fully-qualified name
/// `SecretMath::base()` from within public static method `answer()`, returning 42.
#[test]
fn test_class_protected_static_method_is_callable_inside_class() {
    let out = compile_and_run(
        r#"<?php
class SecretMath {
    protected static function base() {
        return 41;
    }

    public static function answer() {
        return SecretMath::base() + 1;
    }
}

echo SecretMath::answer();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies dynamic dispatch selects the `Dog::speak()` override when `$dog->run()`
/// calls `$this->speak()`, returning "dog".
#[test]
fn test_inheritance_dynamic_dispatch_uses_child_override() {
    let out = compile_and_run(
        r#"<?php
class Animal {
    public function speak() {
        return "animal";
    }

    public function run() {
        return $this->speak();
    }
}

class Dog extends Animal {
    public function speak() {
        return "dog";
    }
}

$dog = new Dog();
echo $dog->run();
"#,
    );
    assert_eq!(out, "dog");
}

/// Verifies private methods use lexical binding: `Base::reveal()` calls `Base::secret()`
/// even when the object is a `Child` instance, returning "base". Private methods are
/// not polymorphic and are resolved at the defining class at compile time.
#[test]
fn test_inheritance_parent_private_method_stays_lexically_bound() {
    let out = compile_and_run(
        r#"<?php
class Base {
    private function secret() {
        return "base";
    }

    public function reveal() {
        return $this->secret();
    }
}

class Child extends Base {
    public function secret() {
        return "child";
    }
}

$child = new Child();
echo $child->reveal();
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies `self::label()` is lexically bound to `Base::label()` even when called on
/// a `Child` instance, returning "base". Self resolves at the compile-time class.
#[test]
fn test_self_static_call_uses_lexical_class() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public static function label() {
        return "base";
    }

    public function reveal() {
        return self::label();
    }
}

class Child extends Base {
    public static function label() {
        return "child";
    }
}

$child = new Child();
echo $child->reveal();
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies `self::label()` resolves to `Base::label()` (lexical binding) even when
/// called on a `Child` instance via `Base::reveal()`, returning "base".
#[test]
fn test_self_instance_call_stays_lexically_bound() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function reveal() {
        return self::label();
    }

    public function label() {
        return "base";
    }
}

class Child extends Base {
    public function label() {
        return "child";
    }
}

$child = new Child();
echo $child->reveal();
"#,
    );
    assert_eq!(out, "base");
}

/// Verifies `static::who()` (late static binding) resolves to the actual runtime class
/// `Child` when called from an instance method `reveal()` on a `Child` object, returning "child".
#[test]
fn test_static_late_binding_uses_child_override_from_instance_method() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public static function who() {
        return "base";
    }

    public function reveal() {
        return static::who();
    }
}

class Child extends Base {
    public static function who() {
        return "child";
    }
}

$child = new Child();
echo $child->reveal();
"#,
    );
    assert_eq!(out, "child");
}

/// Verifies `static::who()` (late static binding) resolves to `Child` when called from
/// `Child::relay()`, returning "child". Late static binding works from static methods.
#[test]
fn test_static_late_binding_uses_child_override_from_static_method() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public static function who() {
        return "base";
    }

    public static function relay() {
        return static::who();
    }
}

class Child extends Base {
    public static function who() {
        return "child";
    }
}

echo Child::relay();
"#,
    );
    assert_eq!(out, "child");
}

/// Verifies named static call `A::who()` is non-forwarding (uses `A`'s vtable) while
/// `self::who()` is forwarding (resolved lexically to `A::who()`). Output is "A B".
#[test]
fn test_named_static_call_is_non_forwarding_but_self_is_forwarding() {
    let out = compile_and_run(
        r#"<?php
class A {
    public static function who() {
        return static::tag();
    }

    public static function relayNamed() {
        return A::who();
    }

    public static function relaySelf() {
        return self::who();
    }

    public static function tag() {
        return "A";
    }
}

class B extends A {
    public static function tag() {
        return "B";
    }
}

echo B::relayNamed() . " " . B::relaySelf();
"#,
    );
    assert_eq!(out, "A B");
}

/// Verifies `parent::who()` forwards the static call while still using runtime late binding
/// for `static::tag()`, returning "B". Parent:: forwards but does not reset the runtime class.
#[test]
fn test_parent_static_call_is_forwarding() {
    let out = compile_and_run(
        r#"<?php
class A {
    public static function who() {
        return static::tag();
    }

    public static function tag() {
        return "A";
    }
}

class B extends A {
    public static function relay() {
        return parent::who();
    }

    public static function tag() {
        return "B";
    }
}

echo B::relay();
"#,
    );
    assert_eq!(out, "B");
}

/// Verifies inherited properties (`$a`) and methods are accessible from child, and
/// `parent::greet()` calls the parent's version, returning "42 hi!".
#[test]
fn test_inheritance_parent_method_call_and_inherited_properties() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public $a = 40;

    public function greet() {
        return "hi";
    }
}

class Child extends Base {
    public $b = 2;

    public function total() {
        return $this->a + $this->b;
    }

    public function greet() {
        return parent::greet() . "!";
    }
}

$child = new Child();
echo $child->total() . " " . $child->greet();
"#,
    );
    assert_eq!(out, "42 hi!");
}

/// Verifies protected method `readValue()` and protected property `$value` are accessible
/// from a subclass via `$this`, returning 42.
#[test]
fn test_inheritance_protected_members_are_accessible_from_subclass() {
    let out = compile_and_run(
        r#"<?php
class Base {
    protected $value = 41;

    protected function readValue() {
        return $this->value;
    }
}

class Child extends Base {
    public function reveal() {
        return $this->readValue() + 1;
    }
}

$child = new Child();
echo $child->reveal();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies first-class callable syntax `MathBox::double(...)` compiles and calls the
/// static method correctly, returning 18.
#[test]
fn test_first_class_callable_static_method_indirect_call() {
    let out = compile_and_run(
        r#"<?php
class MathBox {
    public static function double($n) {
        return $n * 2;
    }
}

$fn = MathBox::double(...);
echo $fn(9);
"#,
    );
    assert_eq!(out, "18");
}

/// Verifies first-class callable on an untyped static method accepts string arguments
/// and returns "Hello World".
#[test]
fn test_first_class_callable_untyped_static_method_accepts_string_args() {
    let out = compile_and_run(
        r#"<?php
class Greeter {
    public static function greet($name) {
        return "Hello " . $name;
    }
}

$f = Greeter::greet(...);
echo $f("World");
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Verifies typed property redeclaration with an initializer overrides the parent's
/// default value: `Child::$x = 5` shadows `Base::$x = 1`, returning "5".
#[test]
fn test_property_redeclaration_concrete_overrides_default() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public int $x = 1;
}

class Child extends Base {
    public int $x = 5;
}

$c = new Child();
echo $c->x;
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies untyped property redeclaration with an initializer overrides the parent's
/// default value: `Child::$value = 2` shadows `Base::$value = 1`, returning "2".
#[test]
fn test_property_redeclaration_untyped() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public $value = 1;
}

class Child extends Base {
    public $value = 2;
}

$c = new Child();
echo $c->value;
"#,
    );
    assert_eq!(out, "2");
}

/// Verifies a child may declare a property whose name collides only with a PRIVATE
/// ancestor property, choosing its own (different) visibility and type. PHP does not
/// inherit private properties, so `Child::$secret` is a fresh, independent property, and
/// the class must type-check, compile, and run. The child reads its own redeclared
/// property, returning "y|y".
///
/// NOTE: the class layout currently keys property storage by name, so the child's
/// `$secret` and the ancestor's private `$secret` share one slot (a documented
/// type-check-only unblock); this test therefore only exercises the child-owned value,
/// not independent parent-private storage.
#[test]
fn test_property_shadows_private_parent_property() {
    let out = compile_and_run(
        r#"<?php
class Base {
    private int $secret = 1;
}

class Child extends Base {
    public string $secret = "y";

    public function reveal(): string {
        return $this->secret;
    }
}

$c = new Child();
echo $c->secret, "|", $c->reveal();
"#,
    );
    assert_eq!(out, "y|y");
}

/// Verifies property redeclaration can widen visibility from `protected` to `public`
/// while preserving the value, returning "20:20".
#[test]
fn test_property_redeclaration_widens_visibility() {
    let out = compile_and_run(
        r#"<?php
class Base {
    protected int $value = 10;

    public function get() {
        return $this->value;
    }
}

class Child extends Base {
    public int $value = 20;
}

$c = new Child();
echo $c->value;
echo ":";
echo $c->get();
"#,
    );
    assert_eq!(out, "20:20");
}

/// Verifies property redeclaration preserves the parent slot offset for non-redeclared
/// properties: `Child::$a = 10` redeclares `$a` but `$b` stays at Base's offset,
/// so `pair()` returns 12 (10+2), and direct access returns "12:10:2".
#[test]
fn test_property_redeclaration_preserves_slot_offset() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public int $a = 1;
    public int $b = 2;

    public function pair() {
        return $this->a + $this->b;
    }
}

class Child extends Base {
    public int $a = 10;
}

$c = new Child();
echo $c->pair();
echo ":";
echo $c->a;
echo ":";
echo $c->b;
"#,
    );
    assert_eq!(out, "12:10:2");
}

/// Verifies property redeclaration can add `readonly` to a typed property, returning "7".
#[test]
fn test_property_redeclaration_adds_readonly() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public int $value = 0;
}

class Child extends Base {
    public readonly int $value;

    public function __construct() {
        $this->value = 7;
    }
}

$c = new Child();
echo $c->value;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies multi-level property redeclaration: `Child::$value = 3` shadows both
/// `Parent_::$value = 2` and `GrandParent::$value = 1`, returning "3:3".
#[test]
fn test_property_redeclaration_multi_level_inheritance() {
    let out = compile_and_run(
        r#"<?php
class GrandParent {
    public int $value = 1;

    public function show() {
        return $this->value;
    }
}

class Parent_ extends GrandParent {
    public int $value = 2;
}

class Child extends Parent_ {
    public int $value = 3;
}

$c = new Child();
echo $c->value;
echo ":";
echo $c->show();
"#,
    );
    assert_eq!(out, "3:3");
}

/// Verifies property redeclaration can both widen visibility (`protected` to `public`)
/// and add `readonly` simultaneously, returning "9:9".
#[test]
fn test_property_redeclaration_widens_visibility_and_adds_readonly() {
    let out = compile_and_run(
        r#"<?php
class Base {
    protected int $value = 0;

    public function get() {
        return $this->value;
    }
}

class Child extends Base {
    public readonly int $value;

    public function __construct() {
        $this->value = 9;
    }
}

$c = new Child();
echo $c->value;
echo ":";
echo $c->get();
"#,
    );
    assert_eq!(out, "9:9");
}

/// Verifies property redeclaration works across trait application: `Child::$value = 5`
/// redeclares `HasValue::$value = 1` brought in via `Base`, returning "5".
#[test]
fn test_property_redeclaration_from_trait() {
    let out = compile_and_run(
        r#"<?php
trait HasValue {
    public int $value = 1;
}

class Base {
    use HasValue;
}

class Child extends Base {
    public int $value = 5;
}

$c = new Child();
echo $c->value;
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies a child class can redeclare a promoted property from a parent's constructor.
/// The parent's promoted property `$value` is initialized by the call (`new Child(42)`),
/// while the child's redeclared `$value = 7` is not used. The child also inherits the
/// parent's `show()` method which reads the parent's slot. Output is "42:42".
#[test]
fn test_property_redeclaration_redeclares_parent_promoted_property() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function __construct(public int $value = 1) {}

    public function show() {
        return $this->value;
    }
}

class Child extends Base {
    public int $value = 7;
}

$c = new Child(42);
echo $c->show();
echo ":";
echo $c->value;
"#,
    );
    assert_eq!(out, "42:42");
}

/// Verifies covariant return via `static`: an abstract parent method declared
/// `: static` is overridden by a concrete child also declared `: static`. The
/// override is legal (PHP 7.4+ covariance; `static` resolves to the child class,
/// a subtype of the parent), so the program compiles and `$b->m()` returns a `B`.
/// Regression guard for the checker falsely rejecting covariant returns because the
/// child's return-type class was not yet registered mid-schema-build.
#[test]
fn test_covariant_return_static_override_runs() {
    let out = compile_and_run(
        r#"<?php
abstract class A { abstract public function m(): static; }
class B extends A { public function m(): static { return $this; } }
$b = new B();
echo get_class($b->m());
"#,
    );
    assert_eq!(out, "B");
}

/// Verifies covariant return with concrete classes: a parent returning `Animal` is
/// overridden by a child returning `Dog` (a subclass). PHP accepts this covariant
/// narrowing, so the program compiles and calls the narrowed method's result.
#[test]
fn test_covariant_return_concrete_subclass_runs() {
    let out = compile_and_run(
        r#"<?php
class Animal { public function name(): string { return "animal"; } }
class Dog extends Animal { public function name(): string { return "dog"; } }
class Base { public function make(): Animal { return new Animal(); } }
class Sub extends Base { public function make(): Dog { return new Dog(); } }
$s = new Sub();
echo $s->make()->name();
"#,
    );
    assert_eq!(out, "dog");
}

/// Verifies nullable covariant return: parent `: ?Animal` overridden by child
/// `: ?Dog` is accepted (Dog <: Animal, and null <: null). The narrowed nullable
/// return still resolves to the concrete `Dog` at runtime.
#[test]
fn test_covariant_return_nullable_subclass_runs() {
    let out = compile_and_run(
        r#"<?php
class Animal { public function name(): string { return "animal"; } }
class Dog extends Animal { public function name(): string { return "dog"; } }
class Base { public function make(): ?Animal { return new Animal(); } }
class Sub extends Base { public function make(): ?Dog { return new Dog(); } }
$s = new Sub();
echo $s->make()->name();
"#,
    );
    assert_eq!(out, "dog");
}

/// Verifies covariant return when implementing an interface method: an interface
/// declares `f(): Animal` and a class implements it returning `Dog` (a subtype).
/// PHP accepts this, so the program compiles and dispatches to the concrete result.
#[test]
fn test_covariant_return_interface_method_runs() {
    let out = compile_and_run(
        r#"<?php
class Animal { public function name(): string { return "animal"; } }
class Dog extends Animal { public function name(): string { return "dog"; } }
interface I { public function f(): Animal; }
class C implements I { public function f(): Dog { return new Dog(); } }
$c = new C();
echo $c->f()->name();
"#,
    );
    assert_eq!(out, "dog");
}

/// Verifies covariant-return acceptance is order-independent: the overriding child
/// class and its parent are declared *before* the return-type classes (`Animal`,
/// `Dog`) in source, exercising the mid-schema-build path where the child's
/// return-type class is not yet registered in `checker.classes`. The subtype
/// relationship must still be resolved from the complete class map.
#[test]
fn test_covariant_return_order_independent_runs() {
    let out = compile_and_run(
        r#"<?php
class Base { public function make(): Animal { return new Animal(); } }
class Sub extends Base { public function make(): Dog { return new Dog(); } }
class Animal { public function name(): string { return "animal"; } }
class Dog extends Animal { public function name(): string { return "dog"; } }
$s = new Sub();
echo $s->make()->name();
"#,
    );
    assert_eq!(out, "dog");
}

/// Verifies LSP-legal parameter widening: a child override may add a trailing
/// optional parameter over a zero-parameter parent method. Both the defaulted
/// call `f()` and the explicit call `f(5)` must resolve to the child override.
/// Cross-checked with `php` (prints `2|5`).
#[test]
fn test_override_add_optional_trailing_param() {
    let out = compile_and_run(
        r#"<?php
class A { function f() { return 1; } }
class B extends A { function f($x = 2) { return $x; } }
$b = new B();
echo $b->f(), "|", $b->f(5);
"#,
    );
    assert_eq!(out, "2|5");
}

/// Verifies LSP-legal parameter widening: a child override may make a required
/// parent parameter optional (widening what it accepts). The defaulted call
/// `f()` uses the child's default and `f(7)` passes the argument through.
/// Cross-checked with `php` (prints `0|7`).
#[test]
fn test_override_make_required_param_optional() {
    let out = compile_and_run(
        r#"<?php
class A { function f($x) { return $x; } }
class B extends A { function f($x = 0) { return $x; } }
$b = new B();
echo $b->f(), "|", $b->f(7);
"#,
    );
    assert_eq!(out, "0|7");
}

/// Verifies LSP-legal parameter widening: a child override may add a variadic
/// tail over a fixed-arity parent method. The override is accepted by signature
/// validation and the variadic tail collects the extra arguments at runtime.
/// (Calls with two or more arguments and uses `count` to sidestep two unrelated
/// pre-existing instance-method-variadic gaps in call-count validation and
/// `array_sum(Array<Mixed>)` lowering.) Cross-checked with `php`: `3 + count([4,5])`
/// is `5`.
#[test]
fn test_override_add_variadic_over_fixed_parent() {
    let out = compile_and_run(
        r#"<?php
class A { function f($a) { return $a; } }
class B extends A { function f($a, ...$rest) { return $a + count($rest); } }
$b = new B();
echo $b->f(3, 4, 5);
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies the interface-implementation analog of parameter widening: a class
/// implementing an interface method may add a trailing optional parameter. The
/// same `validate_signature_compatibility` path serves interface implementation
/// via the "implementing" context. Cross-checked with `php` (prints `9`).
#[test]
fn test_interface_impl_add_optional_trailing_param() {
    let out = compile_and_run(
        r#"<?php
interface Doer { public function doIt(); }
class Worker implements Doer { public function doIt($n = 9) { return $n; } }
$w = new Worker();
echo $w->doIt();
"#,
    );
    assert_eq!(out, "9");
}
/// Verifies a child property can shadow a private parent property with a fresh
/// slot: parent methods keep reading the private slot while child/global reads
/// see the child property.
#[test]
fn test_private_parent_property_shadowing_uses_separate_slots() {
    let out = compile_and_run(
        r#"<?php
class Base {
    private int $value;

    public function __construct() {
        $this->value = 2;
    }

    public function parentValue() {
        return $this->value;
    }
}

class Child extends Base {
    public $value = "child";

    public function childValue() {
        return $this->value;
    }
}

$c = new Child();
echo $c->parentValue();
echo ":";
echo $c->childValue();
echo ":";
echo $c->value;
"#,
    );
    assert_eq!(out, "2:child:child");
}

/// Verifies a later non-private redeclaration updates the visible parent slot,
/// while an older private grandparent slot stays separate for grandparent methods.
#[test]
fn test_private_grandparent_property_shadowing_survives_later_redeclaration() {
    let out = compile_and_run(
        r#"<?php
class GrandParentBox {
    private int $value = 1;

    public function grandParentValue() {
        return $this->value;
    }
}

class ParentBox extends GrandParentBox {
    public int $value = 2;

    public function parentValue() {
        return $this->value;
    }
}

class ChildBox extends ParentBox {
    public int $value = 3;
}

$c = new ChildBox();
echo $c->grandParentValue();
echo ":";
echo $c->parentValue();
echo ":";
echo $c->value;
"#,
    );
    assert_eq!(out, "1:3:3");
}

/// Verifies an explicit ancestor class name can lexically invoke that ancestor's instance
/// implementation while keeping the current child object as `$this`.
#[test]
fn test_explicit_ancestor_instance_method_call_uses_current_this() {
    let out = compile_and_run(
        r#"<?php
class ExplicitAncestorBase {
    protected string $suffix = "!";

    public function render(string $value): string {
        return "base:" . $value . $this->suffix;
    }
}

class ExplicitAncestorChild extends ExplicitAncestorBase {
    public function render(string $value): string {
        return "child:" . $value;
    }

    public function run(): string {
        return ExplicitAncestorBase::render("ok");
    }
}

echo (new ExplicitAncestorChild())->run();
"#,
    );
    assert_eq!(out, "base:ok!");
}
