//! Purpose:
//! End-to-end codegen tests for class, interface, trait, and enum constants.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Constant values are inlined by codegen rather than looked up at runtime.
//! - Inheritance and visibility cases cover schema/codegen agreement.

use super::*;

/// Verifies class constant int.
#[test]
fn test_class_constant_int() {
    //! Verifies integer class constant is inlined and accessible via ClassName::CONST.
    let out = compile_and_run(
        r#"<?php
class Math {
    const PI = 314;
}
echo Math::PI;
"#,
    );
    assert_eq!(out, "314");
}

/// Verifies class constant string.
#[test]
fn test_class_constant_string() {
    //! Verifies string class constant is inlined and accessible via ClassName::CONST.
    let out = compile_and_run(
        r#"<?php
class Greet {
    const HELLO = "hi";
}
echo Greet::HELLO;
"#,
    );
    assert_eq!(out, "hi");
}

/// Verifies class constant inherited from parent.
#[test]
fn test_class_constant_inherited_from_parent() {
    //! Verifies child class inherits parent constants via ClassName::CONST lookup.
    let out = compile_and_run(
        r#"<?php
class Base {
    const VERSION = 7;
}
class Child extends Base {}
echo Child::VERSION;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies class constant expression can reference self constant.
#[test]
fn test_class_constant_expression_can_reference_self_constant() {
    //! Verifies constant expressions can use self:: to reference other constants in the same class.
    let out = compile_and_run(
        r#"<?php
class Box {
    const A = 1;
    const B = self::A + 2;
}
echo Box::B;
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies inherited class constant expression keeps lexical self.
#[test]
fn test_inherited_class_constant_expression_keeps_lexical_self() {
    //! Verifies self:: in a parent constant expression refers to the defining class, not the runtime subclass.
    //! Regression: lexical self must not be replaced with runtime dynamic dispatch.
    let out = compile_and_run(
        r#"<?php
class Base {
    const A = 1;
    const B = self::A + 2;
}
class Child extends Base {
    const A = 10;
}
echo Child::B;
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies class constant expression can reference parent constant.
#[test]
fn test_class_constant_expression_can_reference_parent_constant() {
    //! Verifies constant expressions can use parent:: to access inherited constants.
    let out = compile_and_run(
        r#"<?php
class Base {
    const A = 1;
}
class Child extends Base {
    const B = parent::A + 2;
}
echo Child::B;
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies class constant expression can use self class.
#[test]
fn test_class_constant_expression_can_use_self_class() {
    //! Verifies self::class magic constant works inside a constant expression.
    let out = compile_and_run(
        r#"<?php
class Box {
    const NAME = self::class;
}
echo Box::NAME;
"#,
    );
    assert_eq!(out, "Box");
}

/// Verifies class constant self access inside method.
#[test]
fn test_class_constant_self_access_inside_method() {
    //! Verifies self::CONST inside an instance method resolves to the defining class constant.
    let out = compile_and_run(
        r#"<?php
class Box {
    const SIZE = 42;
    public function describe(): int { return self::SIZE; }
}
$b = new Box();
echo $b->describe();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies interface constant.
#[test]
fn test_interface_constant() {
    //! Verifies interface constants are accessible through implementing class and via ClassName::CONST.
    let out = compile_and_run(
        r#"<?php
interface Limits {
    const MAX = 100;
}
class Bound implements Limits {
    public function get(): int { return Limits::MAX; }
}
$b = new Bound();
echo $b->get();
"#,
    );
    assert_eq!(out, "100");
}

/// Verifies a typed int property default that references a class constant via `ClassName::CONST`
/// is folded to the constant's value and initialized correctly.
#[test]
fn test_property_default_class_constant_int() {
    let out = compile_and_run(
        r#"<?php
class A {
    const X = 10;
    public int $y = A::X;
}
$a = new A();
echo $a->y;
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies a typed property default referencing a constant via `self::CONST` resolves to the
/// same class's constant value.
#[test]
fn test_property_default_class_constant_self() {
    let out = compile_and_run(
        r#"<?php
class A {
    const X = 10;
    public int $y = self::X;
}
$a = new A();
echo $a->y;
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies a typed property default referencing a constant declared on a different class
/// (`Other::CONST`) is resolved across classes.
#[test]
fn test_property_default_class_constant_cross_class() {
    let out = compile_and_run(
        r#"<?php
class A {
    const X = 10;
}
class B {
    public int $y = A::X;
}
$b = new B();
echo $b->y;
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies a cross-class class-constant property default resolves even when the referencing class
/// is declared before the class that owns the constant (forward reference).
#[test]
fn test_property_default_class_constant_forward_reference() {
    let out = compile_and_run(
        r#"<?php
class B {
    public int $y = A::X;
}
class A {
    const X = 10;
}
$b = new B();
echo $b->y;
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies a class-constant property default resolved through `parent::CONST` uses the parent's
/// constant value.
#[test]
fn test_property_default_class_constant_parent() {
    let out = compile_and_run(
        r#"<?php
class P {
    const X = 7;
}
class C extends P {
    public int $y = parent::X;
}
$c = new C();
echo $c->y;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies a typed string property default referencing a string class constant.
#[test]
fn test_property_default_class_constant_string() {
    let out = compile_and_run(
        r#"<?php
class A {
    const S = "hi";
    public string $y = A::S;
}
$a = new A();
echo $a->y;
"#,
    );
    assert_eq!(out, "hi");
}

/// Verifies a typed float property default referencing a float class constant.
#[test]
fn test_property_default_class_constant_float() {
    let out = compile_and_run(
        r#"<?php
class A {
    const F = 1.5;
    public float $y = A::F;
}
$a = new A();
echo $a->y;
"#,
    );
    assert_eq!(out, "1.5");
}

/// Verifies a typed bool property default referencing a bool class constant.
#[test]
fn test_property_default_class_constant_bool() {
    let out = compile_and_run(
        r#"<?php
class A {
    const B = true;
    public bool $y = A::B;
}
$a = new A();
var_dump($a->y);
"#,
    );
    assert_eq!(out, "bool(true)\n");
}

/// Verifies an untyped property whose default references an integer class constant is inferred as
/// numeric (so arithmetic on it works), not as a string.
#[test]
fn test_property_default_class_constant_untyped_is_numeric() {
    let out = compile_and_run(
        r#"<?php
class A {
    const X = 10;
    public $y = A::X;
}
$a = new A();
echo $a->y + 5;
"#,
    );
    assert_eq!(out, "15");
}

/// Verifies a constructor parameter default referencing `self::CONST` resolves to the constant's
/// value when the argument is omitted at the call site.
#[test]
fn test_method_param_default_class_constant() {
    let out = compile_and_run(
        r#"<?php
class A {
    const M = 5;
    public int $n;
    public function __construct(int $n = self::M) {
        $this->n = $n;
    }
}
$a = new A();
echo $a->n;
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies class constant with attribute compiles.
#[test]
fn test_class_constant_with_attribute_compiles() {
    //! Verifies constants with PHP attributes compile without error; attribute is discarded.
    let out = compile_and_run(
        r#"<?php
class Cfg {
    #[Documented]
    const TIMEOUT = 30;
}
echo Cfg::TIMEOUT;
"#,
    );
    assert_eq!(out, "30");
}
