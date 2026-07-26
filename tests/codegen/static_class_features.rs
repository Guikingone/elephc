//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of static class features, including class class named, class class namespaced, and class class self inside method.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

// --- ::class magic constant ---

/// Verifies `ClassName::class` resolves to the unqualified class name `C`.
#[test]
fn test_class_class_named() {
    let out = compile_and_run(
        "<?php class C { public int $x = 0; } echo C::class;",
    );
    assert_eq!(out, "C");
}

/// Verifies `$obj::class` on an object expression resolves to the runtime class name,
/// equivalent to `get_class($obj)` (PHP 8 dynamic `::class`).
#[test]
fn test_class_class_dynamic_object_receiver() {
    let out = compile_and_run(
        "<?php class C {} $o = new C(); echo $o::class;",
    );
    assert_eq!(out, "C");
}

/// Verifies `ClassName::class` inside a namespace resolves to the fully-qualified name `App\C`.
#[test]
fn test_class_class_namespaced() {
    let out = compile_and_run(
        "<?php namespace App; class C { public int $x = 0; } echo C::class;",
    );
    assert_eq!(out, "App\\C");
}

/// Verifies `self::class` inside a method resolves to the lexical (defining) class `C`, not the runtime-called subclass.
#[test]
fn test_class_class_self_inside_method() {
    let out = compile_and_run(
        "<?php\nclass C {\n    public static function name() { return self::class; }\n}\necho C::name();\n",
    );
    assert_eq!(out, "C");
}

/// Verifies `parent::class` inside a child class resolves to the parent class `Base`.
#[test]
fn test_class_class_parent_inside_child() {
    let out = compile_and_run(
        "<?php\nclass Base { public int $x = 0; }\nclass Child extends Base {\n    public static function parent_name() { return parent::class; }\n}\necho Child::parent_name();\n",
    );
    assert_eq!(out, "Base");
}

/// Verifies `static::class` uses late static binding — resolves to the runtime class `Child` even when called from base method.
#[test]
fn test_class_class_static_uses_late_static_binding() {
    let out = compile_and_run(
        "<?php\nclass Base {\n    public static function name() { return static::class; }\n}\nclass Child extends Base {}\necho Child::name();\n",
    );
    assert_eq!(out, "Child");
}

/// Verifies `::class` can be used in string concatenation expressions.
#[test]
fn test_class_class_concat_in_message() {
    let out = compile_and_run(
        "<?php class Logger { public int $x = 0; } echo \"From: \" . Logger::class;",
    );
    assert_eq!(out, "From: Logger");
}

/// Verifies `$object::class` returns the object's fully-qualified runtime class name.
#[test]
fn test_object_class_name_returns_runtime_fqn() {
    let out = compile_and_run(
        "<?php namespace App; class Pluto {} $pippo = new Pluto(); echo $pippo::class;",
    );
    assert_eq!(out, "App\\Pluto");
}

/// Verifies `$object::class` reads the concrete subclass even when the expression is typed as its base class.
#[test]
fn test_object_class_name_preserves_concrete_subclass() {
    let out = compile_and_run(
        "<?php class Base {} class Child extends Base {} function pick(bool $base): Base { return $base ? new Base() : new Child(); } echo pick(false)::class;",
    );
    assert_eq!(out, "Child");
}

/// Verifies `::class` accepts object-only unions and dispatches using the selected runtime object.
#[test]
fn test_object_class_name_accepts_object_union() {
    let out = compile_and_run(
        "<?php class Left {} class Right {} function pick(bool $left): Left|Right { return $left ? new Left() : new Right(); } echo pick(false)::class;",
    );
    assert_eq!(out, "Right");
}

/// Verifies an object-valued expression before `::class` is evaluated exactly once.
#[test]
fn test_object_class_name_evaluates_receiver_once() {
    let out = compile_and_run(
        "<?php class Probe {} function make_probe(): Probe { echo 'once|'; return new Probe(); } echo make_probe()::class;",
    );
    assert_eq!(out, "once|Probe");
}

// --- new self() / new static() / new parent() ---

/// Verifies `new self()` inside a static method returns an instance of the lexical (defining) class `Box` and that fields are accessible.
#[test]
fn test_new_self_returns_instance_of_lexical_class() {
    let out = compile_and_run(
        "<?php\nclass Box {\n    public string $label = \"hello\";\n    public static function make(): Box { return new self(); }\n}\n$b = Box::make();\necho $b->label;\n",
    );
    assert_eq!(out, "hello");
}

/// Verifies `new static()` uses late static binding — returns an instance of the runtime-called class `Child` when called via `Child::make()`.
#[test]
fn test_new_static_returns_instance_of_called_class() {
    let out = compile_and_run(
        "<?php\nclass Base {\n    public static function make(): Base { return new static(); }\n    public function name(): string { return self::class; }\n}\nclass Child extends Base {\n    public function name(): string { return self::class; }\n}\n$b = Child::make();\necho $b->name();\n",
    );
    assert_eq!(out, "Child");
}

/// Verifies a static child override may narrow a parent-class return to `static`.
#[test]
fn test_static_override_covariant_self_return() {
    let out = compile_and_run(
        "<?php class Base { public static function make(): Base { return new static(); } } class Child extends Base { public static function make(): static { return new static(); } } echo Child::make() instanceof Child ? 'ok' : 'no';",
    );
    assert_eq!(out, "ok");
}

/// Verifies `new parent()` inside a child class returns an instance of the parent class `Base`.
#[test]
fn test_new_parent_returns_instance_of_parent_class() {
    let out = compile_and_run(
        "<?php\nclass Base {\n    public string $tag = \"base\";\n}\nclass Child extends Base {\n    public static function makeBase(): Base { return new parent(); }\n}\n$b = Child::makeBase();\necho $b->tag;\n",
    );
    assert_eq!(out, "base");
}

/// Verifies `new self`, `new static`, and `new parent` work without constructor parentheses.
#[test]
fn test_new_relative_receivers_without_constructor_parentheses() {
    let out = compile_and_run(
        "<?php\nclass Base {\n    public function who(): string { return \"base\"; }\n    public static function makeStatic() { return new static; }\n}\nclass Child extends Base {\n    public function who(): string { return \"child\"; }\n    public static function makeSelf() { return new self; }\n    public static function makeParent() { return new parent; }\n}\necho Child::makeSelf()->who(), \"|\", Child::makeStatic()->who(), \"|\", Child::makeParent()->who();\n",
    );
    assert_eq!(out, "child|child|base");
}

/// Verifies `new self()` passes constructor arguments correctly.
#[test]
fn test_new_self_with_constructor_args() {
    let out = compile_and_run(
        "<?php\nclass Greeter {\n    public string $name;\n    public function __construct(string $n) { $this->name = $n; }\n    public static function make(string $n): Greeter { return new self($n); }\n}\n$g = Greeter::make(\"Alice\");\necho $g->name;\n",
    );
    assert_eq!(out, "Alice");
}

// --- Static closures ---

/// Verifies static anonymous functions (closures) can be created and invoked with positional arguments.
#[test]
fn test_static_closure_runs() {
    let out = compile_and_run(
        "<?php $f = static function($a, $b) { return $a + $b; }; echo $f(3, 4);",
    );
    assert_eq!(out, "7");
}

/// Verifies static arrow functions (fn) can be created and invoked.
#[test]
fn test_static_arrow_function_runs() {
    let out = compile_and_run("<?php $g = static fn($x) => $x * 2; echo $g(5);");
    assert_eq!(out, "10");
}

/// Verifies a nullable-int (`?int`, TaggedScalar) value stored into a plain-`int` static
/// property slot narrows correctly. The `if (null !== $v)` guard is not narrowed by the
/// checker, so `$v` reaches the store still typed `?int`; the backend must unwrap the tagged
/// scalar (null→0) before the int slot store. Regression for the symfony/yaml
/// `Inline::$parsedLineNumber` static-property gap.
#[test]
fn test_tagged_scalar_value_into_int_static_property() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static int $n = 0;
    public static function init(?int $v): void {
        if (null !== $v) {
            self::$n = $v;
        }
    }
}
C::init(7);
echo C::$n;
C::init(null);
echo "|", C::$n;
"#,
    );
    assert_eq!(out, "7|7");
}

// --- prefix ++/-- on static-property l-values (parser gate #3) ---

/// Verifies prefix `++self::$n` in a static method yields the new value (PHP semantics),
/// mutating the static property in place. Previously failed with "Expected variable after '++'".
#[test]
fn test_prefix_increment_static_property_yields_new_value() {
    let out = compile_and_run(
        "<?php class C { public static int $n = 5; static function t(){ return ++self::$n; } } echo C::t();",
    );
    assert_eq!(out, "6");
}

/// Verifies prefix `--self::$n` decrements the static property and yields the new value.
#[test]
fn test_prefix_decrement_static_property_yields_new_value() {
    let out = compile_and_run(
        "<?php class C { public static int $n = 5; static function t(){ return --self::$n; } } echo C::t();",
    );
    assert_eq!(out, "4");
}

/// Verifies the postfix-vs-prefix sequence on a static property: `self::$n++` yields the old
/// value, `++self::$n` yields the new value, and both mutate in place. Cross-checked against
/// `php -r` (output `5,7,7`).
#[test]
fn test_static_property_post_then_prefix_incdec_sequence() {
    let out = compile_and_run(
        "<?php class C{ public static int $n=5; static function t(){ $a=self::$n++; $b=++self::$n; return \"$a,$b,\".self::$n; } } echo C::t();",
    );
    assert_eq!(out, "5,7,7");
}

/// Verifies prefix `++self::$n;` as a bare (result-discarded) statement mutates the static
/// property in place — the statement-position path that the Symfony ErrorHandler exercises.
#[test]
fn test_prefix_increment_static_property_statement_position() {
    let out = compile_and_run(
        "<?php class C { public static int $n = 1; static function bump(){ ++self::$n; } } C::bump(); C::bump(); echo C::$n;",
    );
    assert_eq!(out, "3");
}

// --- $obj::CONST dynamic class-constant access ---

/// Verifies `$o::K` reads the class constant on the object's static class (`C::K`).
/// Cross-checked against `php -r` (output `42`).
#[test]
fn test_dynamic_class_constant_on_typed_object() {
    let out = compile_and_run(
        "<?php class C { const K = 42; } function f(C $o): int { return $o::K; } echo f(new C());",
    );
    assert_eq!(out, "42");
}

/// Verifies `$o::K` resolves an *inherited* class constant through the parent chain.
/// Cross-checked against `php -r` (output `7`).
#[test]
fn test_dynamic_class_constant_inherited() {
    let out = compile_and_run(
        "<?php class B { const K = 7; } class C extends B {} function f(C $o): int { return $o::K; } echo f(new C());",
    );
    assert_eq!(out, "7");
}

/// Verifies `$o::K` resolves an *interface* constant when the receiver is typed by the
/// interface. Cross-checked against `php -r` (output `3`).
#[test]
fn test_dynamic_class_constant_interface() {
    let out = compile_and_run(
        "<?php interface I { const K = 3; } class C implements I {} function f(I $o): int { return $o::K; } echo f(new C());",
    );
    assert_eq!(out, "3");
}

/// Verifies a string class constant read through an object composes in an expression.
/// Cross-checked against `php -r` (output `ceex`).
#[test]
fn test_dynamic_class_constant_string_in_expression() {
    let out = compile_and_run(
        "<?php class C { const NAME = \"cee\"; } function f(C $o): string { return $o::NAME . \"x\"; } echo f(new C());",
    );
    assert_eq!(out, "ceex");
}

/// Verifies the object expression is evaluated exactly once for its side effects when its
/// class constant is read (`mk()::K` prints `m` once, then the constant). Cross-checked
/// against `php -r` (output `m42`).
#[test]
fn test_dynamic_class_constant_object_evaluated_once() {
    let out = compile_and_run(
        "<?php class C { const K = 42; } function mk(): C { echo \"m\"; return new C(); } echo mk()::K;",
    );
    assert_eq!(out, "m42");
}

/// Regression test (Symfony routing `AddTrait`): a PROTECTED property accessed on `$this` narrowed
/// via `instanceof` to a class the current class can never be (single-inheritance siblings) is a
/// statically-dead branch. PHP checks visibility only when the fetch runs, and that fetch never
/// runs here, so the checker must accept it (not a spurious "Cannot access protected property").
/// The `AddTrait` is composed into `CollectionConfigurator`, whose `$this instanceof
/// RouteConfigurator` guard is always false — the protected `RouteConfigurator::$parentConfigurator`
/// read is unreachable. php-verified: prints `r1:coll`.
#[test]
fn test_this_instanceof_incompatible_sibling_protected_read_dead_branch() {
    let out = compile_and_run(
        r#"<?php
class RouteConfigurator {
    public function __construct(protected ?RouteConfigurator $parentConfigurator = null) {}
}
class CollectionConfigurator {
    use AddTrait;
    public function __construct(private ?CollectionConfigurator $parentConfigurator = null) {}
    public function hasParent(): bool { return $this->parentConfigurator !== null; }
}
trait AddTrait {
    public function add(string $n): string {
        $p = $this instanceof CollectionConfigurator
            ? "coll"
            : ($this instanceof RouteConfigurator
                ? ($this->parentConfigurator !== null ? "has" : "no")
                : "null");
        return $n . ":" . $p;
    }
}
echo (new CollectionConfigurator())->add("r1");
"#,
    );
    assert_eq!(out, "r1:coll");
}
