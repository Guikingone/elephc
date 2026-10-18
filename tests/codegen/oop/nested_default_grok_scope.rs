//! Purpose:
//! Covers declaration scope in nested object defaults and free-function defaults.
//!
//! Called from:
//! - The OOP codegen integration suite.
//!
//! Key details:
//! - Constructor names never replace the lexical scope of their arguments.
//! - Free-function defaults cannot borrow the caller's class scope.

use crate::support::{compile_and_run_with_heap_debug, without_ir_opt};

/// Checks the observable value and retirement of every native owner.
fn check(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "stdout={}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Nested self factories bind to the declaring class or trait consumer across caller scopes.
#[test]
fn test_nested_default_grok_third_review_constructor_scope() {
    check(r#"<?php
class Wrapper { public function __construct(public object $inner) {} }
class Declaring {
    public function make(Wrapper $value = new Wrapper(new self)): Wrapper { return $value; }
    public function run(): void { echo get_class($this->make()->inner), '|'; }
}
class Child extends Declaring {}
trait Factory {
    public function make(Wrapper $value = new Wrapper(new self)): Wrapper { return $value; }
}
class Consumer { use Factory; }
$declaring = new Declaring();
$declaring->run();
echo get_class($declaring->make()->inner), '|';
echo get_class((new Child())->make()->inner), '|';
echo get_class((new Consumer())->make()->inner);
unset($declaring);
"#, "Declaring|Declaring|Declaring|Consumer");
}

/// Nested constructor scope does not depend on the EIR optimizer.
#[test]
fn test_nested_default_grok_third_review_constructor_scope_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_constructor_scope);
}

/// Constants inside constructor arguments use the surrounding declaration's scope.
#[test]
fn test_nested_default_grok_third_review_constructor_constant() {
    check(r#"<?php
class Wrapper { public function __construct(public string $name) {} }
class Declaring {
    public const LABEL = 'declaring';
    public function make(Wrapper $value = new Wrapper(self::LABEL)): Wrapper { return $value; }
}
echo (new Declaring())->make()->name;
"#, "declaring");
}

/// Nested constant arguments retain lexical scope without EIR optimization.
#[test]
fn test_nested_default_grok_third_review_constructor_constant_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_constructor_constant);
}

/// Calling a free function from a class never supplies scope to its relative default.
#[test]
fn test_nested_default_grok_third_review_free_function_scope() {
    check(r#"<?php
function make(object $value = new self): string { return get_class($value); }
function makeParent(object $value = new parent): string { return get_class($value); }
class Base {}
class Caller extends Base {
    public function run(): void {
        echo make(new stdClass()), '|';
        try { make(); } catch (Error $error) { echo $error->getMessage(), '|'; unset($error); }
        echo makeParent(new stdClass()), '|';
        try { makeParent(); } catch (Error $error) { echo $error->getMessage(); unset($error); }
    }
}
(new Caller())->run();
"#, "stdClass|Cannot access \"self\" when no class scope is active|stdClass|Cannot access \"parent\" when no class scope is active");
}

/// Free-function default deferral keeps the same behavior without EIR optimization.
#[test]
fn test_nested_default_grok_third_review_free_function_scope_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_free_function_scope);
}

/// Descriptor calls preserve the no-class scope of free-function object defaults.
#[test]
fn test_nested_default_grok_third_review_free_function_callbacks() {
    check(r#"<?php
function make(object $value = new self): string { return get_class($value); }
function untyped($value = new parent): string { return get_class($value); }
$factory = make(...);
$parent = untyped(...);
echo $factory(new stdClass()), '|';
try { $factory(); } catch (Error $error) { echo $error->getMessage(), '|'; unset($error); }
echo $parent(new stdClass()), '|';
try { $parent(); } catch (Error $error) { echo $error->getMessage(); unset($error); }
unset($factory, $parent);
"#, "stdClass|Cannot access \"self\" when no class scope is active|stdClass|Cannot access \"parent\" when no class scope is active");
}

/// Free-function descriptor defaults remain lazy without EIR optimization.
#[test]
fn test_nested_default_grok_third_review_free_function_callbacks_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_free_function_callbacks);
}

/// Opaque eval observes the same lazy no-class Error when invoking a native free function.
#[test]
fn test_nested_default_grok_third_review_free_function_eval() {
    check(r#"<?php
function make(object $value = new self): string { return get_class($value); }
function run(string $source): mixed { return eval($source); }
echo run('return make(new stdClass());'), '|';
echo run('try { return make(); } catch (Error $error) { return $error->getMessage(); }');
"#, "stdClass|Cannot access \"self\" when no class scope is active");
}

/// Native free-function default transport is independent of EIR optimization.
#[test]
fn test_nested_default_grok_third_review_free_function_eval_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_free_function_eval);
}

/// An unbound object factory raises before PHP checks a scalar parameter's type.
#[test]
fn test_nested_default_grok_third_review_scalar_unbound_factory() {
    check(r#"<?php
function make(int $value = new self): int { return $value; }
echo make(1), '|';
try { make(); } catch (Error $error) { echo $error->getMessage(); unset($error); }
"#, "1|Cannot access \"self\" when no class scope is active");
}

/// Lazy scalar-parameter scope failures remain independent of EIR optimization.
#[test]
fn test_nested_default_grok_third_review_scalar_unbound_factory_no_ir_opt() {
    without_ir_opt(test_nested_default_grok_third_review_scalar_unbound_factory);
}
