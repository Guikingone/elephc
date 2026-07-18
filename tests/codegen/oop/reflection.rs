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

// --- Dynamic-name `new ReflectionClass($runtimeName)` construction (PART C) ---
//
// The reflected-name argument must be derived from `$argc` (or similar runtime-unknown state)
// rather than a plain `$x = "Literal"; new ReflectionClass($x);` — the AST-level constant folder
// runs before the checker and would otherwise fold such an assignment back into a literal
// argument, silently exercising the pre-existing literal path instead of the new dynamic
// dispatcher this section tests. Mirrors the established idiom in
// `tests/codegen/casts_and_constants/introspection.rs`'s non-literal `class_exists()` tests.

/// Verifies dynamic `new ReflectionClass($runtimeName)` construction end to end: the reflected
/// class is resolved at runtime, and every A1 metadata method (`getName`, `getShortName`,
/// `isAbstract`, `isSubclassOf`, `hasMethod`) returns the SAME per-class values the literal path
/// would — proving the dynamic dispatcher's construction branch populates the exact same
/// closed-world metadata slots as `lower_reflection_owner_new`'s literal-argument path (php -n
/// verified expected values).
#[test]
fn test_reflection_class_dynamic_construction_valid_name() {
    let out = compile_and_run(
        r#"<?php
abstract class ElephcDynAnimal { public int $legs = 4; }
class ElephcDynDog extends ElephcDynAnimal {
    public function bark(): string { return "woof"; }
}
$name = $argc > 0 ? "ElephcDynDog" : "NOPE";
$r = new ReflectionClass($name);
echo $r->getName();
echo "|";
echo $r->getShortName();
echo "|";
echo $r->isAbstract() ? "1" : "0";
echo "|";
echo $r->isSubclassOf("ElephcDynAnimal") ? "1" : "0";
echo "|";
echo $r->hasMethod("bark") ? "1" : "0";
echo $r->hasMethod("meow") ? "1" : "0";
"#,
    );
    assert_eq!(out, "ElephcDynDog|ElephcDynDog|0|1|10");
}

/// Verifies a dynamic `new ReflectionClass($runtimeName)` construction with an unknown class name
/// throws a REAL, CATCHABLE `\ReflectionException` (not a fatal) — php -n verified message format
/// `Class "NAME" does not exist`, echoing the original queried name unmodified. This is the
/// behavior Symfony's DI container and autoloading fallbacks depend on (`try { new
/// ReflectionClass(...) } catch (\ReflectionException $e) { ... }`).
#[test]
fn test_reflection_class_dynamic_construction_unknown_name_throws_reflection_exception() {
    let out = compile_and_run(
        r#"<?php
$name = $argc > 0 ? "ElephcDynNoSuchClass" : "NOPE";
try {
    $r = new ReflectionClass($name);
    echo "no throw";
} catch (\ReflectionException $e) {
    echo "caught:";
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(out, "caught:Class \"ElephcDynNoSuchClass\" does not exist");
}

/// Verifies the dynamic dispatcher's class-name comparison is case-INSENSITIVE, matching PHP
/// class-name semantics (php -n verified: `new ReflectionClass("elephcdyndog")` resolves the
/// declared `ElephcDynDog` class and `getName()` returns its canonical declared-case spelling).
#[test]
fn test_reflection_class_dynamic_construction_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
class ElephcDynDog {
    public function bark(): string { return "woof"; }
}
$name = $argc > 0 ? "elephcdyndog" : "NOPE";
$r = new ReflectionClass($name);
echo $r->getName();
"#,
    );
    assert_eq!(out, "ElephcDynDog");
}

/// Regression: the pre-existing literal-argument `new ReflectionClass("Name")` construction path
/// stays byte-for-byte unchanged when a dynamic-name call site is ALSO present in the same
/// program (both routes are compiled: the dynamic dispatcher is emitted once, and every literal
/// call site still takes the direct compile-time metadata bake — see the `is_const_string_or_
/// class_value` gate at the top of `lower_reflection_owner_new`).
#[test]
fn test_reflection_class_literal_construction_unaffected_by_dynamic_dispatcher_presence() {
    let out = compile_and_run(
        r#"<?php
abstract class ElephcDynAnimal { public int $legs = 4; }
class ElephcDynDog extends ElephcDynAnimal {
    public function bark(): string { return "woof"; }
}
$name = $argc > 0 ? "ElephcDynDog" : "NOPE";
$dynamic = new ReflectionClass($name);
$literal = new ReflectionClass("ElephcDynAnimal");
echo $dynamic->getName();
echo "|";
echo $literal->getName();
echo "|";
echo $literal->isAbstract() ? "1" : "0";
"#,
    );
    assert_eq!(out, "ElephcDynDog|ElephcDynAnimal|1");
}

// -- getFileName() (Part D: declaring-file plumbing) + getParentClass() (Part C) --
//
// `getFileName()` is baked from `crate::pipeline::scan_reflection_source_files`'s snapshot of
// the entry file's OWN top-level declarations (see
// `crate::codegen_ir::lower_inst::objects::reflection::reflection_class_extra_metadata`); the
// codegen test harness (`tests/codegen/support/compiler.rs`) mirrors that exact placement using
// its synthetic `<temp>/test.php` main file, so — like the existing `__FILE__` tests in
// `tests/codegen/magic_constants.rs` — these assertions check the path SHAPE (absolute, ends
// with `test.php`) rather than an exact string.

/// Verifies `ReflectionClass::getFileName()` returns the SAME absolute path for a literal- and a
/// dynamically-constructed receiver of classes declared in the same (synthetic) file, and that
/// `ReflectionFunction::getFileName()` agrees too — all three point at one physical file.
#[test]
fn test_reflection_get_file_name_literal_dynamic_and_function_agree() {
    let out = compile_and_run(
        r#"<?php
class ElephcFileA {}
class ElephcFileB {}
function elephcFileFn(): string { return "x"; }

$literal = new ReflectionClass("ElephcFileA");
$name = $argc > 0 ? "ElephcFileB" : "NOPE";
$dynamic = new ReflectionClass($name);
$rf = new ReflectionFunction("elephcFileFn");

$file1 = $literal->getFileName();
$file2 = $dynamic->getFileName();
$file3 = $rf->getFileName();
echo ($file1 === $file2 && $file2 === $file3) ? "same" : "different";
echo "|";
echo (str_starts_with($file1, "/") && str_ends_with($file1, "test.php")) ? "shaped" : "unshaped";
"#,
    );
    assert_eq!(out, "same|shaped");
}

/// Verifies `ReflectionClass::getFileName()` on a builtin/internal class returns PHP's `false`
/// sentinel (php -n verified: `(new ReflectionClass('stdClass'))->getFileName() === false`) —
/// the `__file` slot's empty-string sentinel correctly surfaces as `false`, not an empty string.
#[test]
fn test_reflection_class_get_file_name_builtin_class_returns_false() {
    let out = compile_and_run(
        r#"<?php
$r = new ReflectionClass("stdClass");
var_dump($r->getFileName());
"#,
    );
    assert_eq!(out, "bool(false)\n");
}

/// Verifies `ReflectionMethod::getFileName()` resolves to the file of the class that ACTUALLY
/// DECLARES the method, not the constructor's `class_name` argument (php -n verified:
/// `(new ReflectionMethod('Dog', 'speak'))->getFileName()` for a `speak()` inherited from
/// `Animal` reports `Animal`'s file) — both classes are declared in the same synthetic file
/// here, so this specifically exercises the `method_declaring_classes` resolution rather than
/// the (separately covered) cross-file case.
#[test]
fn test_reflection_method_get_file_name_matches_declaring_class() {
    let out = compile_and_run(
        r#"<?php
class ElephcFileAnimal {
    public function speak(): string { return "..."; }
}
class ElephcFileDog extends ElephcFileAnimal {
    public function bark(): string { return "woof"; }
}
$rcAnimal = new ReflectionClass("ElephcFileAnimal");
$rmInherited = new ReflectionMethod("ElephcFileDog", "speak");
$rmOwn = new ReflectionMethod("ElephcFileDog", "bark");
echo $rmInherited->getFileName() === $rcAnimal->getFileName() ? "inherited-matches" : "inherited-mismatch";
echo "|";
echo $rmOwn->getFileName() === $rcAnimal->getFileName() ? "own-matches" : "own-mismatch";
"#,
    );
    assert_eq!(out, "inherited-matches|own-matches");
}

/// Verifies `ReflectionClass::getParentClass()` on a class WITH a parent returns a real,
/// usable `ReflectionClass` for that parent (php -n verified: same class name, same
/// `getFileName()` since both classes here share one file) — for BOTH a literal- and a
/// dynamically-constructed receiver, proving the single PHP-level shell body
/// (`$this->__parent_name === '' ? false : new ReflectionClass($this->__parent_name)`) serves
/// both construction paths identically.
#[test]
fn test_reflection_class_get_parent_class_returns_parent_reflection_class() {
    let out = compile_and_run(
        r#"<?php
class ElephcParentAnimal {}
class ElephcParentDog extends ElephcParentAnimal {}

$literal = new ReflectionClass("ElephcParentDog");
$literalParent = $literal->getParentClass();
echo ($literalParent !== false) ? $literalParent->getName() : "false";

$name = $argc > 0 ? "ElephcParentDog" : "NOPE";
$dynamic = new ReflectionClass($name);
$dynamicParent = $dynamic->getParentClass();
echo "|";
echo ($dynamicParent !== false) ? $dynamicParent->getName() : "false";
echo "|";
echo ($literalParent !== false && $dynamicParent !== false && $literalParent->getFileName() === $dynamicParent->getFileName()) ? "same-file" : "different-file";
"#,
    );
    assert_eq!(out, "ElephcParentAnimal|ElephcParentAnimal|same-file");
}

/// Verifies `ReflectionClass::getParentClass()` on a class with NO parent returns PHP's `false`
/// sentinel (php -n verified: `(new ReflectionClass('ElephcNoParent'))->getParentClass() ===
/// false`), for both a literal- and a dynamically-constructed receiver.
#[test]
fn test_reflection_class_get_parent_class_no_parent_returns_false() {
    let out = compile_and_run(
        r#"<?php
class ElephcNoParent {}
$literal = new ReflectionClass("ElephcNoParent");
var_dump($literal->getParentClass());

$name = $argc > 0 ? "ElephcNoParent" : "NOPE";
$dynamic = new ReflectionClass($name);
var_dump($dynamic->getParentClass());
"#,
    );
    assert_eq!(out, "bool(false)\nbool(false)\n");
}

// -- Mixed-typed dynamic ReflectionClass argument (Part A): PHP's real
// `__construct(object|string $objectOrClass)` signature — `new ReflectionClass($obj)` is legal
// PHP (php -n verified) and reflects the OBJECT'S OWN runtime class, not necessarily the
// receiving variable's static type. See
// `crate::codegen_ir::lower_inst::objects::reflection::lower_reflection_class_new_dynamic` for
// the runtime tag dispatch this backs.

/// Verifies `new ReflectionClass($obj)` reflects the object's own concrete runtime class (php -n
/// verified, using a subclass instance to prove it is NOT just trusting a static type name).
#[test]
fn test_reflection_class_dynamic_construction_from_object_argument() {
    let out = compile_and_run(
        r#"<?php
class ElephcMixedAnimal {}
class ElephcMixedDog extends ElephcMixedAnimal {}
$obj = $argc > 0 ? new ElephcMixedDog() : new ElephcMixedAnimal();
$r = new ReflectionClass($obj);
echo $r->getName();
"#,
    );
    assert_eq!(out, "ElephcMixedDog");
}

/// Verifies `new ReflectionClass($mixed)` where the runtime value is a boxed STRING (Mixed tag 1)
/// resolves exactly like a plain `Str`-typed dynamic argument (php -n verified).
#[test]
fn test_reflection_class_dynamic_construction_from_mixed_string_argument() {
    let out = compile_and_run(
        r#"<?php
class ElephcMixedStr {}
function pick(bool $b): mixed { return $b ? "ElephcMixedStr" : 42; }
$name = pick($argc > 0);
$r = new ReflectionClass($name);
echo $r->getName();
"#,
    );
    assert_eq!(out, "ElephcMixedStr");
}

/// Verifies `new ReflectionClass($mixed)` where the runtime value is a boxed OBJECT (Mixed tag 6)
/// resolves the object's own class (php -n verified) — the Mixed-boxed counterpart of
/// `test_reflection_class_dynamic_construction_from_object_argument`.
#[test]
fn test_reflection_class_dynamic_construction_from_mixed_object_argument() {
    let out = compile_and_run(
        r#"<?php
class ElephcMixedObj {}
function pick(bool $b): mixed { return $b ? new ElephcMixedObj() : "nope"; }
$obj = pick($argc > 0);
$r = new ReflectionClass($obj);
echo $r->getName();
"#,
    );
    assert_eq!(out, "ElephcMixedObj");
}

/// Verifies PHP's real WEAK-TYPING scalar coercion for `new ReflectionClass($scalar)` (php -n
/// verified — NOT a `TypeError`, contrary to a naive reading of the `object|string` signature):
/// int/float/bool/null are all coerced to their `(string)` cast and routed through the SAME
/// closed-world class lookup as a literal string, producing PHP's exact
/// `ReflectionException: Class "X" does not exist` message and remaining fully catchable.
#[test]
fn test_reflection_class_dynamic_construction_scalar_weak_coercion_matches_php() {
    let out = compile_and_run(
        r#"<?php
function results(bool $b): array {
    $out = [];
    foreach ([42, 4.2, true, false, null] as $v) {
        $x = $b ? $v : "unused";
        try {
            new ReflectionClass($x);
            $out[] = "no-throw";
        } catch (\ReflectionException $e) {
            $out[] = $e->getMessage();
        }
    }
    return $out;
}
foreach (results($argc > 0) as $line) {
    echo $line, "|";
}
"#,
    );
    assert_eq!(
        out,
        "Class \"42\" does not exist|Class \"4.2\" does not exist|Class \"1\" does not exist|Class \"\" does not exist|Class \"\" does not exist|"
    );
}

/// Verifies PHP's real behavior for a genuinely non-coercible `new ReflectionClass($x)` argument
/// (php -n verified: an `array` argument throws a real, CATCHABLE `\TypeError` — never a
/// `ReflectionException`, and the construction never proceeds with garbage).
#[test]
fn test_reflection_class_dynamic_construction_array_argument_throws_type_error() {
    let out = compile_and_run(
        r#"<?php
$value = $argc > 0 ? [1, 2] : "unused";
try {
    new ReflectionClass($value);
    echo "no-throw";
} catch (\TypeError $e) {
    echo "caught:", get_class($e);
}
"#,
    );
    assert_eq!(out, "caught:TypeError");
}

/// Regression: `new ReflectionClass($x)` where `$x` is a STATICALLY int/float/bool-typed local
/// (not boxed as `Mixed` — the checker knows the concrete scalar type at compile time) must get
/// the SAME PHP weak-coercion treatment as a `Mixed`-boxed scalar (php -n verified — this is
/// PART A's uniform runtime-tag design, but a plain, non-`Mixed` scalar operand takes a
/// DIFFERENT, unboxed codegen path in `lower_reflection_class_new_dynamic`; this caught a real
/// gap where that path fell through to an "unsupported EIR backend feature" internal compiler
/// error instead of PHP's `ReflectionException`).
#[test]
fn test_reflection_class_dynamic_construction_plain_scalar_locals_weak_coerce() {
    let out = compile_and_run(
        r#"<?php
$intLocal = 42;
try {
    new ReflectionClass($intLocal);
    echo "no-throw";
} catch (\ReflectionException $e) {
    echo $e->getMessage();
}
echo "|";
$floatLocal = 4.2;
try {
    new ReflectionClass($floatLocal);
    echo "no-throw";
} catch (\ReflectionException $e) {
    echo $e->getMessage();
}
echo "|";
$boolLocal = true;
try {
    new ReflectionClass($boolLocal);
    echo "no-throw";
} catch (\ReflectionException $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "Class \"42\" does not exist|Class \"4.2\" does not exist|Class \"1\" does not exist"
    );
}
