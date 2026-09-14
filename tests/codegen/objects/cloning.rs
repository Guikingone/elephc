//! Purpose:
//! Integration or regression tests for PHP object cloning codegen.
//! Covers shallow object copies, declared property slots, stdClass dynamic properties, and `__clone` hooks.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures compile to native binaries and compare stdout against PHP clone semantics.

use super::*;

/// Verifies cloning declared scalar/string properties creates an independent object slot copy.
#[test]
fn test_clone_copies_declared_properties_independently() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public int $n = 1;
    public string $label = "one";
}
$a = new Item();
$b = clone $a;
$b->n = 2;
$b->label = "two";
echo $a->n . ":" . $a->label . "|" . $b->n . ":" . $b->label;
"#,
    );
    assert_eq!(out, "1:one|2:two");
}

/// Verifies `__clone()` is invoked after the shallow copy and mutates the clone, not the source.
#[test]
fn test_clone_invokes_magic_clone_on_the_copy() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public int $n = 1;
    public function __clone(): void {
        echo "hook;";
        $this->n = $this->n + 10;
    }
}
$a = new Counter();
$b = clone $a;
echo $a->n . "|" . $b->n;
"#,
    );
    assert_eq!(out, "hook;1|11");
}

/// Verifies `__clone()` can replace a string property without corrupting the source object.
#[test]
fn test_clone_persists_string_property_before_magic_clone_mutation() {
    let out = compile_and_run(
        r#"<?php
class LabelBox {
    public string $label = "A";
    public function __clone(): void {
        $this->label = $this->label . ":copy";
    }
}
$a = new LabelBox();
$b = clone $a;
echo $a->label . "|" . $b->label;
"#,
    );
    assert_eq!(out, "A|A:copy");
}

/// Verifies object-valued properties are shallow-copied, so nested object mutations remain shared.
#[test]
fn test_clone_keeps_nested_objects_shared() {
    let out = compile_and_run(
        r#"<?php
class Child {
    public int $x = 1;
}
class Boxed {
    public Child $child;
    public function __construct() {
        $this->child = new Child();
    }
}
$a = new Boxed();
$b = clone $a;
$b->child->x = 7;
echo $a->child->x . "|" . $b->child->x;
"#,
    );
    assert_eq!(out, "7|7");
}

/// Verifies stdClass dynamic properties are copied into a separate hash table during cloning.
#[test]
fn test_clone_copies_stdclass_dynamic_properties_independently() {
    let out = compile_and_run(
        r#"<?php
$a = new stdClass();
$a->name = "source";
$b = clone $a;
$b->name = "copy";
$b->extra = "new";
echo $a->name . "|" . $b->name . "|" . (isset($a->extra) ? "Y" : "N");
"#,
    );
    assert_eq!(out, "source|copy|N");
}

// --- PHP 8.5 `clone()` FUNCTION -------------------------------------------------------------
//
// These pin the FIRST AOT slice of the function form. It shares the boxed shallow-copy adapter
// with Magician, validates its argument at run time, and REFUSES the `$withProperties` overrides
// it cannot apply yet rather than dropping them silently.

/// Verifies the function form copies declared property slots into an independent object.
#[test]
fn test_clone_function_copies_declared_properties_independently() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public int $n = 1;
    public string $label = "one";
}
$a = new Item();
$b = clone($a);
$b->n = 2;
$b->label = "two";
echo $a->n . ":" . $a->label . "|" . $b->n . ":" . $b->label;
"#,
    );
    assert_eq!(out, "1:one|2:two");
}

/// Verifies the function form runs the runtime-selected `__clone()` hook on the copy.
#[test]
fn test_clone_function_invokes_magic_clone_on_the_copy() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public int $n = 1;
    public function __clone(): void {
        echo "hook;";
        $this->n = $this->n + 10;
    }
}
class Plain {
    public int $k = 5;
}
$a = new Counter();
$b = clone($a);
echo $a->n . "|" . $b->n . ";";
$p = new Plain();
$q = clone($p);
$q->k = 9;
echo $p->k . "|" . $q->k;
"#,
    );
    assert_eq!(out, "hook;1|11;5|9");
}

/// Verifies `clone(...)` reaches the same lowering through a runtime callable, with and
/// without an explicit empty override array.
#[test]
fn test_clone_function_works_through_a_runtime_callable() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public int $n = 1;
    public function bump(): void { $this->n = $this->n + 5; }
}
$fn = clone(...);
$a = new Box();
$b = $fn($a);
$b->bump();
echo get_class($b) . ":" . $a->n . "|" . $b->n . ";";
$c = $fn($a, []);
echo get_class($c) . ":" . $c->n;
"#,
    );
    assert_eq!(out, "Box:1|6;Box:1");
}

/// Verifies a non-object argument raises a catchable `TypeError`, whether the backend sees the
/// wrong type statically or only through a runtime-shaped value.
#[test]
fn test_clone_function_rejects_non_object_arguments() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 2; }
function pick(int $k): mixed { if ($k === 0) { return 5; } return new Box(); }
$fn = clone(...);
try {
    $x = $fn(5);
    echo "no throw;";
} catch (TypeError $e) {
    echo $e->getMessage() . ";";
}
try {
    $y = clone(pick(0));
    echo "no throw;";
} catch (TypeError $e) {
    echo $e->getMessage() . ";";
}
echo get_class(clone(pick(1)));
"#,
    );
    assert_eq!(
        out,
        "clone(): Argument #1 ($object) must be of type object, int given;\
clone(): Argument #1 ($object) must be of type object;Box"
    );
}

/// Verifies this slice REFUSES property overrides instead of ignoring them, both when the
/// operand is a literal and when it only becomes known at run time.
#[test]
fn test_clone_function_refuses_property_overrides() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 1; }
$a = new Box();
try {
    $b = clone($a, ["n" => 4]);
    echo "no throw;";
} catch (Error $e) {
    echo $e->getMessage() . ";";
}
$fn = clone(...);
try {
    $c = $fn($a, ["n" => 4]);
    echo "no throw";
} catch (Error $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "clone(): Argument #2 ($withProperties) property overrides are not supported yet;\
clone(): Argument #2 ($withProperties) property overrides are not supported yet"
    );
}

/// Verifies a class the shared adapter cannot copy raises a catchable `Error`.
#[test]
fn test_clone_function_reports_uncloneable_objects() {
    let out = compile_and_run(
        r#"<?php
$s = new SplFixedArray(2);
try {
    $c = clone($s);
    echo "no throw";
} catch (Error $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(out, "Trying to clone an uncloneable object");
}

/// Verifies hook visibility follows the CALLER's lexical scope, not the runtime class alone.
#[test]
fn test_clone_function_honors_private_hook_visibility() {
    let out = compile_and_run(
        r#"<?php
class Locked {
    public int $n = 1;
    private function __clone(): void { $this->n = 9; }
    public function copy(): Locked { return clone($this); }
}
$a = new Locked();
echo $a->copy()->n . ";";
try {
    $b = clone($a);
    echo "no throw";
} catch (Error $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "9;Call to private method Locked::__clone() from global scope"
    );
}

/// Verifies an escaped clone FCC uses each invocation site's scope across direct descriptor,
/// opaque callable, CUF and CUFA paths. `array_map()` remains outside this test because its
/// documented object-element limitation rejects the source array before callback invocation.
#[test]
fn test_clone_callable_uses_invocation_site_scope_across_dispatch_paths() {
    let out = compile_and_run(
        r#"<?php
class CloneScopeRoot {
    public function invokeAncestor(callable $cb, object $value) {
        return $cb($value);
    }
}
class CloneScopeDeclaring extends CloneScopeRoot {
    protected function __clone(): void { echo "hook;"; }
    public function makeCallable(): callable { return clone(...); }
    public function invokeDeclaringCuf(callable $cb, object $value) {
        return call_user_func($cb, $value);
    }
    public function invokeRuntimeCallback(callable $cb, object $value): void {
        array_filter([$value], $cb);
    }
}
class CloneScopeChild extends CloneScopeDeclaring {
    public function invokeDescendantCufa(callable $cb, object $value) {
        return call_user_func_array($cb, [$value]);
    }
}
class CloneScopeOther {
    public function invokeUnrelated(callable $cb, object $value) {
        return $cb($value);
    }
}
class ClonePrivateDeclaring {
    private function __clone(): void { echo "private;"; }
    public function makeCallable(): callable { return clone(...); }
    public function invokeDeclaring(callable $cb, object $value) { return $cb($value); }
}
class ClonePrivateChild extends ClonePrivateDeclaring {
    public function invokeChild(callable $cb, object $value) { return $cb($value); }
}
function invokeOpaqueClone(callable $cb, object $value) {
    return $cb($value);
}
function printCloneScopeError(callable $run): void {
    try {
        $run();
        echo "no throw;";
    } catch (Error $e) {
        echo $e->getMessage() . ";";
    }
}
$value = new CloneScopeChild();
$cb = $value->makeCallable();
printCloneScopeError(function() use ($cb, $value) { return $cb($value); });
printCloneScopeError(function() use ($cb, $value) { return invokeOpaqueClone($cb, $value); });
(new CloneScopeRoot())->invokeAncestor($cb, $value);
$value->invokeDeclaringCuf($cb, $value);
$value->invokeDescendantCufa($cb, $value);
$value->invokeRuntimeCallback($cb, $value);
printCloneScopeError(function() use ($cb, $value) {
    return (new CloneScopeOther())->invokeUnrelated($cb, $value);
});
$private = new ClonePrivateDeclaring();
$privateCb = $private->makeCallable();
$private->invokeDeclaring($privateCb, $private);
printCloneScopeError(function() use ($privateCb, $private) { return $privateCb($private); });
$privateChild = new ClonePrivateChild();
printCloneScopeError(function() use ($privateCb, $privateChild) {
    return $privateChild->invokeChild($privateCb, $privateChild);
});
"#,
    );
    assert_eq!(
        out,
        "Call to protected method CloneScopeDeclaring::__clone() from global scope;\
Call to protected method CloneScopeDeclaring::__clone() from global scope;\
hook;hook;hook;hook;\
Call to protected method CloneScopeDeclaring::__clone() from scope CloneScopeOther;\
private;Call to private method ClonePrivateDeclaring::__clone() from global scope;\
Call to private method ClonePrivateDeclaring::__clone() from scope ClonePrivateChild;"
    );
}

/// Verifies the fresh clone stays exception-owned: a throwing hook releases it exactly once,
/// leaving the source object to be destroyed normally at scope exit.
#[test]
fn test_clone_function_releases_the_clone_when_the_hook_throws() {
    let out = compile_and_run(
        r#"<?php
class Boom {
    public int $n = 1;
    public function __clone(): void { throw new RuntimeException("in hook"); }
    public function __destruct() { echo "gone;"; }
}
function run(): void {
    $a = new Boom();
    try {
        $b = clone($a);
        echo "no throw;";
    } catch (RuntimeException $e) {
        echo "caught:" . $e->getMessage() . ";";
    }
    echo "end;";
}
run();
echo "after";
"#,
    );
    assert_eq!(out, "gone;caught:in hook;end;gone;after");
}

/// Verifies the successful path transfers ownership exactly once: two live objects, two
/// destructor runs, with no leak and no double free.
#[test]
fn test_clone_function_transfers_ownership_of_the_copy() {
    let out = compile_and_run(
        r#"<?php
class Tracked {
    public int $n = 1;
    public function __destruct() { echo "gone;"; }
}
function useClone(): void {
    $a = new Tracked();
    $b = clone($a);
    echo "made:" . $b->n . ";";
}
useClone();
echo "after";
"#,
    );
    assert_eq!(out, "made:1;gone;gone;after");
}
