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

/// Verifies a RUNTIME override array is applied to the clone and leaves the source untouched.
///
/// The array is a plain local, not a literal the backend can read at compile time, which is the
/// shape every non-trivial call site has.
#[test]
fn test_clone_function_applies_a_runtime_override_array() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 1; public string $s = "a"; }
$a = new Box();
$ov = ["n" => 4, "s" => "z"];
$b = clone($a, $ov);
echo $a->n . ":" . $a->s . "|" . $b->n . ":" . $b->s;
"#,
    );
    assert_eq!(out, "1:a|4:z");
}

/// Verifies a spread-built override array reaches the applicator with its runtime entries.
#[test]
fn test_clone_function_applies_a_spread_built_override_array() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 1; public string $s = "a"; }
$rest = ["n" => 3, "s" => "q"];
$b = clone(new Box(), [...$rest]);
echo $b->n . ":" . $b->s;
"#,
    );
    assert_eq!(out, "3:q");
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
/// Verifies a first-class callable carries the INVOCATION SITE's scope into property resolution.
///
/// The same `clone(...)` value reaches a private property from inside the class and is refused
/// from global scope, so the scope cannot have been baked in where the callable was created.
#[test]
fn test_clone_function_resolves_override_scope_through_a_first_class_callable() {
    let out = compile_and_run(
        r#"<?php
class S {
    private int $x = 1;
    public function get(): int { return $this->x; }
    public static function inside(S $o, array $ov) { $f = clone(...); return $f($o, $ov); }
}
$s = new S();
echo S::inside($s, ["x" => 8])->get() . ";";
$escaped = clone(...);
try { $escaped($s, ["x" => 9]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "8;Cannot access private property S::$x");
}

/// Verifies `call_user_func()` and `call_user_func_array()` transport the same invocation scope.
#[test]
fn test_clone_function_resolves_override_scope_through_call_user_func_variants() {
    let out = compile_and_run(
        r#"<?php
class S {
    private int $x = 1;
    public function get(): int { return $this->x; }
    public static function cuf(S $o, array $ov) { return call_user_func('clone', $o, $ov); }
    public static function cufa(S $o, array $ov) { return call_user_func_array('clone', [$o, $ov]); }
}
$s = new S();
echo S::cuf($s, ["x" => 4])->get() . ";" . S::cufa($s, ["x" => 5])->get() . ";";
try { call_user_func('clone', $s, ["x" => 6]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "4;5;Cannot access private property S::$x");
}

/// Verifies two same-named private slots stay apart: the SCOPE picks which one an override writes.
///
/// `P` writes the parent's slot on a `C` instance and `C` writes its own, which a resolver keyed
/// on the runtime class alone cannot express. Global scope reaches neither.
#[test]
fn test_clone_function_selects_the_scope_private_slot_on_a_child_clone() {
    let out = compile_and_run(
        r#"<?php
class P {
    private int $x = 1;
    public function show(): int { return $this->x; }
    public static function fromP(C $o, array $ov): void { $r = clone($o, $ov); echo $r->show() . "/" . $r->show2() . ";"; }
}
class C extends P {
    private int $x = 2;
    public function show2(): int { return $this->x; }
    public static function fromC(C $o, array $ov): void { $r = clone($o, $ov); echo $r->show() . "/" . $r->show2() . ";"; }
}
$c = new C();
P::fromP($c, ["x" => 55]);
C::fromC($c, ["x" => 66]);
try { clone($c, ["x" => 77]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "55/2;1/66;Cannot access private property C::$x");
}

/// Verifies protected access follows php's ancestor-OR-descendant rule in both directions.
#[test]
fn test_clone_function_honors_protected_visibility_in_both_ancestry_directions() {
    let out = compile_and_run(
        r#"<?php
class A {
    protected int $p = 1;
    public function get(): int { return $this->p; }
    public static function fromA(A $o, array $ov): int { $r = clone($o, $ov); return $r->get(); }
}
class B extends A {
    public static function fromB(A $o, array $ov): int { $r = clone($o, $ov); return $r->get(); }
}
echo A::fromA(new B(), ["p" => 3]) . ";" . B::fromB(new A(), ["p" => 4]) . ";";
try { clone(new A(), ["p" => 5]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "3;4;Cannot access protected property A::$p");
}

/// Verifies PHP 8.4 asymmetric write visibility decides overrides, naming the DECLARING class.
#[test]
fn test_clone_function_honors_asymmetric_set_visibility() {
    let out = compile_and_run(
        r#"<?php
class R {
    public private(set) int $ps = 0;
    public protected(set) int $pr = 0;
    public static function inner(R $o, array $ov): R { return clone($o, $ov); }
}
class RC extends R {
    public static function child(R $o, array $ov): R { return clone($o, $ov); }
}
$r = new R();
echo R::inner($r, ["ps" => 9])->ps . ";";
try { RC::child($r, ["ps" => 7]); echo "no throw;"; } catch (Error $e) { echo $e->getMessage() . ";"; }
echo RC::child($r, ["pr" => 6])->pr . ";";
try { clone($r, ["pr" => 3]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "9;Cannot modify private(set) property R::$ps from scope RC;\
6;Cannot modify protected(set) property R::$pr from global scope"
    );
}

/// Verifies `readonly` is REINITIALIZED by an override in scope and refused out of scope.
///
/// `readonly` carries an implicit `protected(set)`, which is why the refusal says so.
#[test]
fn test_clone_function_reinitializes_readonly_properties_only_in_scope() {
    let out = compile_and_run(
        r#"<?php
class R {
    public readonly int $ro;
    public function __construct() { $this->ro = 1; }
    public function inner(array $ov): R { return clone($this, $ov); }
}
$r = new R();
echo $r->inner(["ro" => 9])->ro . ";";
try { clone($r, ["ro" => 5]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "9;Cannot modify protected(set) readonly property R::$ro from global scope"
    );
}

/// Verifies overrides go through the ordinary typed-property pipeline: weak coercion, then a
/// `TypeError` for a value no coercion accepts.
#[test]
fn test_clone_function_coerces_and_rejects_typed_override_values() {
    let out = compile_and_run(
        r#"<?php
class T { public int $n = 0; public float $f = 0.0; public string $s = ""; }
$ov = ["n" => "42", "f" => 3, "s" => 7];
$c = clone(new T(), $ov);
echo $c->n . ":" . $c->f . ":" . $c->s . ";";
$bad = ["n" => "abc"];
try { clone(new T(), $bad); echo "no throw"; } catch (TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "42:3:7;Cannot assign string to property T::$n of type int");
}

/// Verifies an override on a hooked property runs the `set` hook rather than the backing slot.
#[test]
fn test_clone_function_runs_set_hooks_for_overrides() {
    let out = compile_and_run(
        r#"<?php
class H { public int $n = 0 { set(int $v) { echo "hook(" . $v . ");"; $this->n = $v * 2; } } }
$ov = ["n" => 5];
$c = clone(new H(), $ov);
echo $c->n;
"#,
    );
    assert_eq!(out, "hook(5);10");
}

/// Verifies a `set` hook that throws during an override leaves no clone behind.
#[test]
fn test_clone_function_releases_the_clone_when_a_set_hook_throws() {
    let out = compile_and_run(
        r#"<?php
class HB {
    public int $n = 0 { set(int $v) { if ($v > 5) { throw new RuntimeException("too big"); } $this->n = $v; } }
    public function __destruct() { echo "gone;"; }
}
function run(): void {
    $a = new HB();
    $ov = ["n" => 9];
    try { clone($a, $ov); echo "no throw;"; } catch (RuntimeException $e) { echo "caught:" . $e->getMessage() . ";"; }
}
run();
echo "after";
"#,
    );
    assert_eq!(out, "gone;caught:too big;gone;after");
}

/// Verifies a `clone()` override nested inside a `set` hook that itself runs under another
/// `clone()` override keeps both applications straight.
#[test]
fn test_clone_function_supports_overrides_nested_inside_a_set_hook() {
    let out = compile_and_run(
        r#"<?php
class Inner { public int $v = 0 { set(int $x) { echo "inner(" . $x . ");"; $this->v = $x; } } }
class Outer {
    public int $n = 0 {
        set(int $x) {
            echo "outer(" . $x . ");";
            $c = clone(new Inner(), ["v" => $x + 1]);
            echo "got(" . $c->v . ");";
            $this->n = $x;
        }
    }
}
$ov = ["n" => 4];
$o = clone(new Outer(), $ov);
echo $o->n;
"#,
    );
    assert_eq!(out, "outer(4);inner(5);got(5);4");
}

/// Verifies an inaccessible name and an unknown name both reach `__set()`, as php does.
#[test]
fn test_clone_function_routes_inaccessible_and_unknown_names_to_magic_set() {
    let out = compile_and_run(
        r#"<?php
class M {
    private int $p = 1;
    public function __set($k, $v) { echo "set(" . $k . "=" . $v . ");"; }
}
$ov = ["p" => 5, "zz" => 6];
clone(new M(), $ov);
echo "done";
"#,
    );
    assert_eq!(out, "set(p=5);set(zz=6);done");
}

/// Verifies integer keys become the decimal property names php uses and a NUL-prefixed name throws.
#[test]
fn test_clone_function_converts_numeric_keys_and_rejects_nul_names() {
    let out = compile_and_run(
        r#"<?php
$ov = [0 => "zero", 7 => "seven"];
$c = clone(new stdClass(), $ov);
echo $c->{"0"} . ":" . $c->{"7"} . ";";
$bad = ["\0hidden" => 1];
try { clone(new stdClass(), $bad); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "zero:seven;Cannot access property starting with \"\\0\"");
}

/// Verifies a class with dynamic-property storage takes an unknown name, and one without
/// REPORTS instead of dropping the write.
///
/// php 8.5 deprecates and stores on the plain class. This compiler has no per-instance property
/// hash unless the class opted in, and already refuses `$plain->undeclared = 1` at compile time,
/// so the override raises php's `Cannot create dynamic property` `Error` rather than vanishing.
#[test]
fn test_clone_function_stores_dynamic_properties_only_where_storage_exists() {
    let out = compile_and_run(
        r#"<?php
#[AllowDynamicProperties] class D { public int $n = 1; }
class Plain { public int $n = 1; }
$ov = ["n" => 2, "zz" => "x"];
$d = clone(new D(), $ov);
echo $d->n . ":" . $d->zz . ";";
try { clone(new Plain(), ["zz" => 1]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "2:x;Cannot create dynamic property Plain::$zz");
}

/// Verifies both arguments are evaluated once in source order before anything is applied, and
/// that the first failing key stops the rest.
///
/// The source object is inspected afterwards to prove the refused run touched only the clone.
#[test]
fn test_clone_function_evaluates_arguments_in_order_and_stops_at_the_first_error() {
    let out = compile_and_run(
        r#"<?php
class Z {
    public int $a = 0;
    private int $b = 0;
    public int $c = 0;
    public function all(): string { return $this->a . "/" . $this->b . "/" . $this->c; }
}
function mk(string $tag, $v) { echo "arg(" . $tag . ");"; return $v; }
$z = new Z();
$o = mk("obj", $z);
$ov = mk("ov", ["a" => 1, "b" => 2, "c" => 3]);
try { clone($o, $ov); echo "no throw;"; } catch (Error $e) { echo $e->getMessage() . ";"; }
echo $z->all();
"#,
    );
    assert_eq!(
        out,
        "arg(obj);arg(ov);Cannot access private property Z::$b;0/0/0"
    );
}

/// Verifies an enum case is uncloneable, with or without overrides.
#[test]
fn test_clone_function_refuses_to_clone_enum_cases() {
    let out = compile_and_run(
        r#"<?php
enum Suit: string { case Hearts = 'H'; }
try { clone(Suit::Hearts, ["value" => "S"]); echo "no throw"; } catch (Error $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(out, "Trying to clone an uncloneable object of class Suit");
}

/// Verifies a refused override releases the clone exactly once and leaves the source intact.
#[test]
fn test_clone_function_releases_the_clone_when_an_override_is_refused() {
    let out = compile_and_run(
        r#"<?php
class Tracked {
    public int $n = 1;
    private int $secret = 0;
    public function __destruct() { echo "gone;"; }
}
function run(): void {
    $a = new Tracked();
    $ov = ["n" => 2, "secret" => 3];
    try { clone($a, $ov); echo "no throw;"; } catch (Error $e) { echo "caught:" . $e->getMessage() . ";"; }
    echo "src=" . $a->n . ";";
}
run();
echo "after";
"#,
    );
    assert_eq!(
        out,
        "gone;caught:Cannot access private property Tracked::$secret;src=1;gone;after"
    );
}
