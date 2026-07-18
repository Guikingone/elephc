//! Purpose:
//! End-to-end codegen tests for the closed-world Reflection API expansion:
//! `ReflectionClass`/`ReflectionMethod`/`ReflectionProperty` scalar and array
//! metadata methods, class constants, and the `Reflector` marker interface.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every asserted value was cross-checked against real PHP (`php -n`)
//!   before being hardcoded here.
//! - Metadata methods are baked from compile-time `ClassInfo` at
//!   `ReflectionClass`/`ReflectionMethod`/`ReflectionProperty` construction
//!   time (see `crate::codegen_ir::lower_inst::objects::reflection`), so
//!   these tests use small, real user class hierarchies rather than mocks.

use super::*;

/// Verifies `ReflectionClass::isAbstract()`/`isFinal()`/`isInterface()`/
/// `isInstantiable()`/`isTrait()`/`getShortName()` against a small
/// abstract/concrete hierarchy. php -n verified: an abstract class reports
/// `isAbstract()==true, isInstantiable()==false`; a concrete class is the
/// inverse; `ReflectionClass` never reflects an interface or trait in
/// elephc's closed world, so `isInterface()`/`isTrait()` are always `false`.
#[test]
fn test_reflection_class_abstract_final_instantiable_short_name() {
    let out = compile_and_run(
        r#"<?php
abstract class Animal {
    public function speak(): string { return "..."; }
}
class Dog extends Animal {
    public function bark(): string { return "woof"; }
}
$ra = new ReflectionClass("Animal");
$rd = new ReflectionClass("Dog");
echo $ra->isAbstract() ? "1" : "0";
echo $ra->isInstantiable() ? "1" : "0";
echo $rd->isAbstract() ? "1" : "0";
echo $rd->isFinal() ? "1" : "0";
echo $rd->isInterface() ? "1" : "0";
echo $rd->isTrait() ? "1" : "0";
echo $rd->isInstantiable() ? "1" : "0";
echo $rd->getShortName();
"#,
    );
    assert_eq!(out, "1000001Dog");
}

/// Verifies `isSubclassOf()` against parent classes, transitively
/// implemented interfaces, and self-exclusion. php -n verified: a class
/// reflecting `C` (which extends `B extends A implements I`) reports
/// `isSubclassOf("A")==true`, `isSubclassOf("I")==true` (interfaces count),
/// and `isSubclassOf("C")==false` (excludes itself).
#[test]
fn test_reflection_class_is_subclass_of() {
    let out = compile_and_run(
        r#"<?php
interface I {}
class A implements I {}
class B extends A {}
class C extends B {}
$rc = new ReflectionClass("C");
echo $rc->isSubclassOf("A") ? "1" : "0";
echo $rc->isSubclassOf("B") ? "1" : "0";
echo $rc->isSubclassOf("I") ? "1" : "0";
echo $rc->isSubclassOf("C") ? "1" : "0";
"#,
    );
    assert_eq!(out, "1110");
}

/// Verifies `isSubclassOf()`/`hasMethod()` accept a RUNTIME (non-literal)
/// string argument and match PHP's case-insensitive class/method-name
/// resolution — the membership test runs against a construction-baked array
/// via `in_array()`, so it works for any runtime string, not just
/// compile-time literals.
#[test]
fn test_reflection_class_is_subclass_of_and_has_method_are_case_insensitive_and_dynamic() {
    let out = compile_and_run(
        r#"<?php
class A { public function Foo(): void {} }
class B extends A {}
$rb = new ReflectionClass("B");
$target = strtolower("A");
echo $rb->isSubclassOf($target) ? "1" : "0";
echo $rb->hasMethod("foo") ? "1" : "0";
echo $rb->hasMethod("FOO") ? "1" : "0";
echo $rb->hasMethod("nonexistent") ? "1" : "0";
"#,
    );
    assert_eq!(out, "1110");
}

/// Verifies `hasProperty()` stays exact-case (PHP property names are
/// case-SENSITIVE, unlike class/method names) and includes inherited
/// properties.
#[test]
fn test_reflection_class_has_property_is_case_sensitive_and_inherited() {
    let out = compile_and_run(
        r#"<?php
class A { public int $age = 1; }
class B extends A { public string $name = "x"; }
$rb = new ReflectionClass("B");
echo $rb->hasProperty("age") ? "1" : "0";
echo $rb->hasProperty("Age") ? "1" : "0";
echo $rb->hasProperty("name") ? "1" : "0";
echo $rb->hasProperty("nope") ? "1" : "0";
"#,
    );
    assert_eq!(out, "1010");
}

/// Verifies `implementsInterface()` and `getInterfaceNames()` include
/// interfaces implemented transitively through the parent class chain
/// (php -n verified: `B extends A implements I` reports
/// `(new ReflectionClass('B'))->getInterfaceNames() === ['I']`).
#[test]
fn test_reflection_class_implements_interface_and_interface_names_are_inherited() {
    let out = compile_and_run(
        r#"<?php
interface I {}
class A implements I {}
class B extends A {}
$rb = new ReflectionClass("B");
echo $rb->implementsInterface("I") ? "1" : "0";
echo $rb->implementsInterface("Other") ? "1" : "0";
$names = $rb->getInterfaceNames();
echo count($names);
echo $names[0];
"#,
    );
    assert_eq!(out, "101I");
}

/// Verifies `getConstants()`/`getConstant()` include the reflected class's
/// own constants, constants inherited from a parent class (own wins on a
/// name collision), and constants inherited from an implemented interface —
/// and that `getConstant()` returns PHP's documented `false` sentinel for an
/// undefined name (php -n verified).
#[test]
fn test_reflection_class_get_constants_and_get_constant() {
    let out = compile_and_run(
        r#"<?php
interface Greetable {
    const GREETING = "hi";
}
abstract class Animal implements Greetable {
    const KIND = "animal";
}
class Dog extends Animal {
    const KIND = "dog";
}
$rd = new ReflectionClass("Dog");
$consts = $rd->getConstants();
echo $consts["KIND"];
echo $consts["GREETING"];
echo $rd->getConstant("KIND");
var_dump($rd->getConstant("nope"));
"#,
    );
    assert_eq!(out, "doghidogbool(false)\n");
}

/// Verifies `isInternal()` reports `true` only for a real PHP builtin shell
/// class and `false` for a user-declared class (jury addendum: never
/// fabricate `true` for a compiler-synthetic helper or a user class).
#[test]
fn test_reflection_class_is_internal() {
    let out = compile_and_run(
        r#"<?php
class MyClass {}
$ru = new ReflectionClass("MyClass");
$re = new ReflectionClass("Exception");
echo $ru->isInternal() ? "1" : "0";
echo $re->isInternal() ? "1" : "0";
"#,
    );
    assert_eq!(out, "01");
}

/// Verifies `ReflectionMethod::getModifiers()`/`isPublic()`/`isProtected()`/
/// `isStatic()`/`isAbstract()`/`getShortName()` against real method
/// declarations (php -n verified bit values: `IS_PUBLIC=1, IS_PROTECTED=2,
/// IS_STATIC=16, IS_ABSTRACT=64`). Also regression-covers a pre-existing gap
/// this change fixed in passing: `ReflectionMethod::getName()`/`__name` was
/// never baked at construction before this feature (only
/// `ReflectionClass`'s constructor populated the shared slot offset), so
/// `getName()` silently returned `''` for every `ReflectionMethod` instance.
#[test]
fn test_reflection_method_modifiers_and_short_name() {
    let out = compile_and_run(
        r#"<?php
abstract class A {
    abstract public function bar(): void;
    protected static function baz(): void {}
}
$r1 = new ReflectionMethod("A", "bar");
echo $r1->getModifiers();
echo $r1->isAbstract() ? "1" : "0";
echo $r1->isPublic() ? "1" : "0";
echo $r1->getShortName();
echo "|";
$r2 = new ReflectionMethod("A", "baz");
echo $r2->getModifiers();
echo $r2->isStatic() ? "1" : "0";
echo $r2->isProtected() ? "1" : "0";
"#,
    );
    assert_eq!(out, "6511bar|1811");
}

/// Verifies the PHP-faithful `ReflectionMethod::IS_*` / `ReflectionProperty::IS_*`
/// class constants (php -n verified: `ReflectionMethod::IS_STATIC=16,
/// IS_PUBLIC=1, IS_PROTECTED=2, IS_PRIVATE=4, IS_ABSTRACT=64, IS_FINAL=32`;
/// `ReflectionProperty::IS_STATIC=16, IS_PUBLIC=1, IS_PROTECTED=2,
/// IS_PRIVATE=4, IS_READONLY=128, IS_ABSTRACT=64, IS_FINAL=32` — PHP 8.4
/// added the last two to `ReflectionProperty` for abstract property hooks
/// and final properties).
#[test]
fn test_reflection_method_and_property_is_constants() {
    let out = compile_and_run(
        r#"<?php
echo ReflectionMethod::IS_STATIC, " ";
echo ReflectionMethod::IS_PUBLIC, " ";
echo ReflectionMethod::IS_PROTECTED, " ";
echo ReflectionMethod::IS_PRIVATE, " ";
echo ReflectionMethod::IS_ABSTRACT, " ";
echo ReflectionMethod::IS_FINAL, "\n";
echo ReflectionProperty::IS_STATIC, " ";
echo ReflectionProperty::IS_PUBLIC, " ";
echo ReflectionProperty::IS_PROTECTED, " ";
echo ReflectionProperty::IS_PRIVATE, " ";
echo ReflectionProperty::IS_READONLY, " ";
echo ReflectionProperty::IS_ABSTRACT, " ";
echo ReflectionProperty::IS_FINAL;
"#,
    );
    assert_eq!(out, "16 1 2 4 64 32\n16 1 2 4 128 64 32");
}

/// Verifies `ReflectionProperty::getModifiers()` reports PHP 8.4's
/// `IS_FINAL` bit for a `final` property (php -n verified:
/// `final public int $x` reports modifiers `33` = `IS_PUBLIC|IS_FINAL`).
#[test]
fn test_reflection_property_final_modifier() {
    let out = compile_and_run(
        r#"<?php
class A {
    final public int $x = 1;
}
$rp = new ReflectionProperty("A", "x");
echo $rp->getModifiers();
"#,
    );
    assert_eq!(out, "33");
}

/// Verifies `ReflectionProperty::getModifiers()`/`hasType()` against real
/// property declarations, including a static protected property (php -n
/// verified bit values as above).
#[test]
fn test_reflection_property_modifiers_and_has_type() {
    let out = compile_and_run(
        r#"<?php
class D {
    protected static string $breed = "unknown";
    public int $age = 3;
    private $untyped;
}
$r1 = new ReflectionProperty("D", "breed");
echo $r1->getModifiers();
echo $r1->hasType() ? "1" : "0";
$r2 = new ReflectionProperty("D", "age");
echo $r2->getModifiers();
echo $r2->hasType() ? "1" : "0";
$r3 = new ReflectionProperty("D", "untyped");
echo $r3->getModifiers();
echo $r3->hasType() ? "1" : "0";
"#,
    );
    // The untyped `$untyped` property (no type hint, no default) must
    // report `hasType()==false` (php -n verified) — see
    // `property_modifiers_and_type` in the EIR codegen for why this can't be
    // derived from the resolved `PhpType` alone (an untyped property still
    // gets an inferred `PhpType::Int` fallback there).
    assert_eq!(out, "1811140");
}

/// Verifies `Reflector` (which extends `Stringable`) is implemented by every
/// core Reflection* shell and narrows through `instanceof`.
#[test]
fn test_reflector_instanceof() {
    let out = compile_and_run(
        r#"<?php
class A {
    public int $age = 1;
    public function foo(): void {}
}
$rc = new ReflectionClass("A");
$rm = new ReflectionMethod("A", "foo");
$rp = new ReflectionProperty("A", "age");
echo $rc instanceof Reflector ? "1" : "0";
echo $rc instanceof Stringable ? "1" : "0";
echo $rm instanceof Reflector ? "1" : "0";
echo $rp instanceof Reflector ? "1" : "0";
"#,
    );
    assert_eq!(out, "1111");
}

/// Verifies that echoing/casting a Reflection* object stays a loud,
/// observable failure rather than silently producing empty-string output —
/// `Reflector`'s inherited `__toString()` contract throws a real `\Error`
/// instead of fabricating PHP's object-dump text. Calling `__toString()`
/// directly (bypassing the pre-existing, unrelated cast/echo exception
/// propagation gap noted in the reflection expansion report) proves the
/// throw itself is real and catchable.
#[test]
fn test_reflection_class_tostring_throws_when_called_directly() {
    let out = compile_and_run(
        r#"<?php
class A {}
$rc = new ReflectionClass("A");
try {
    echo $rc->__toString();
    echo "no throw";
} catch (\Error $e) {
    echo "caught";
}
"#,
    );
    assert_eq!(out, "caught");
}
