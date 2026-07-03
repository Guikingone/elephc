//! Purpose:
//! Regression tests for dynamic method dispatch on receivers whose static type
//! does not name a single class (a `Mixed` value, or a union of object classes).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Before the fix, a method call on such a receiver emitted no dispatch and
//!   left a garbage value in the result register. Dispatch now reads the
//!   receiver's runtime class id and selects the matching class's method, so
//!   these fixtures assert PHP-equivalent stdout.
//! - The `narrowed_interface_*` tests cover method calls on an interface-typed
//!   receiver where the method is NOT on the interface (the checker accepted it
//!   via `instanceof` narrowing to a concrete subtype). The backend falls back to
//!   the same runtime class-id dispatch, gating the `__call`-resolving case with a
//!   loud unsupported error so it can never miscompile.

use super::*;

/// Verifies a method call on a `: mixed`-returning value dispatches on the runtime
/// class id (the value is an object), both via a local and chained directly.
#[test]
fn test_mixed_receiver_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
class S {
    public int $v;
    public function __construct(int $x) { $this->v = $x; }
    public function doubled(): int { return $this->v * 2; }
}
class C {
    public function make(int $x): mixed {
        if ($x < 0) { return false; }
        return new S($x);
    }
}
$c = new C();
$s = $c->make(5);
echo $s->doubled() . ";" . $c->make(7)->doubled();
"#,
    );
    assert_eq!(out, "10;14");
}

/// Verifies a method call on an `object|false` union receiver dispatches correctly.
#[test]
fn test_object_or_false_union_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
class S {
    public int $v;
    public function __construct(int $x) { $this->v = $x; }
    public function doubled(): int { return $this->v * 2; }
}
class C {
    public function make(int $x): S|bool {
        if ($x < 0) { return false; }
        return new S($x);
    }
}
$c = new C();
$s = $c->make(5);
echo ($s === false) ? "F" : $s->doubled();
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies a dynamic-receiver method call passes its arguments correctly (the
/// receiver and arguments are evaluated once and dispatched together).
#[test]
fn test_mixed_receiver_method_with_arguments() {
    let out = compile_and_run(
        r#"<?php
class S {
    public function add(int $a, int $b): int { return $a + $b; }
}
class C {
    public function make(): mixed { return new S(); }
}
$c = new C();
echo $c->make()->add(3, 4);
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies dynamic dispatch selects the correct class at runtime when several
/// classes define the same method.
#[test]
fn test_mixed_receiver_multiple_candidate_classes() {
    let out = compile_and_run(
        r#"<?php
class Dog { public function speak(): string { return "woof"; } }
class Cat { public function speak(): string { return "meow"; } }
function animal(int $n): mixed { return ($n == 0) ? new Dog() : new Cat(); }
echo animal(0)->speak() . ":" . animal(1)->speak();
"#,
    );
    assert_eq!(out, "woof:meow");
}

/// Verifies a dynamic-receiver method call returning a string works end to end.
#[test]
fn test_mixed_receiver_string_return() {
    let out = compile_and_run(
        r#"<?php
class S {
    public string $n;
    public function __construct(string $n) { $this->n = $n; }
    public function greet(): string { return "hi " . $this->n; }
}
class C {
    public function make(string $n): mixed { return new S($n); }
}
$c = new C();
echo $c->make("ada")->greet();
"#,
    );
    assert_eq!(out, "hi ada");
}

/// Verifies a dynamic-receiver method call on a non-object runtime value fatals
/// (PHP "Call to a member function ... on a non-object") instead of miscompiling.
#[test]
fn test_mixed_receiver_non_object_fatals() {
    let out = compile_and_run_capture(
        r#"<?php
class S { public function d(): int { return 1; } }
function make(int $x): mixed { if ($x < 0) { return false; } return new S(); }
$v = make(-1);
echo $v->d();
"#,
    );
    assert!(!out.success);
    assert!(out.stderr.contains("Call to a member function d()"));
}

/// Regression: a user method whose name collides with a builtin method of a different arity (here
/// `add`, which `DateTime::add(DateInterval)` also defines) must still dispatch correctly for a
/// mixed receiver. The dispatch marshals arguments once with the first candidate's signature, so
/// candidates are filtered by argument arity; otherwise `DateTime::add` could be selected for this
/// 2-argument call depending on (nondeterministic) class-id ordering, corrupting the result.
#[test]
fn test_mixed_receiver_method_name_collides_with_builtin_arity() {
    let out = compile_and_run(
        r#"<?php
class Money { public function add(int $a, int $b): int { return $a + $b; } }
function make(): mixed { return new Money(); }
echo make()->add(40, 2);
"#,
    );
    assert_eq!(out, "42");
}

/// Core win: a method that exists only on a concrete implementor (`A::extra`) is called on an
/// interface-typed receiver after `instanceof A` narrowing. The backend can no longer see the
/// narrowing, so it dispatches by the receiver's runtime class id; `base()` (on the interface)
/// still uses ordinary interface dispatch. `new A()` takes the narrowed `extra()` branch and
/// `new B()` takes the `base()` branch.
#[test]
fn test_narrowed_interface_literal_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
interface I { function base(): string; }
class A implements I { function base(): string {return "A";} function extra(): string {return "X";} }
class B implements I { function base(): string {return "B";} }
function f(I $v): string { return $v instanceof A ? $v->extra() : $v->base(); }
echo f(new A()), f(new B());
"#,
    );
    assert_eq!(out, "XB");
}

/// A subclass that inherits the off-interface method (`C extends A` inherits `A::extra`) is
/// dispatched through the interface receiver: `new C()` narrows via `instanceof A` and the
/// class-id switch selects `C`, whose inherited `extra` runs `A::extra`.
#[test]
fn test_narrowed_interface_inherited_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
interface I { function base(): string; }
class A implements I { function base(): string {return "A";} function extra(): string {return "X";} }
class C extends A {}
function f(I $v): string { return $v instanceof A ? $v->extra() : $v->base(); }
echo f(new C());
"#,
    );
    assert_eq!(out, "X");
}

/// Virtual dispatch on the narrowed path: a subclass that OVERRIDES the off-interface method
/// (`C extends A` with its own `extra`) is called through the interface receiver. The runtime
/// class id selects `C::extra`, not the `A::extra` named by the `instanceof A` narrowing.
#[test]
fn test_narrowed_interface_overriding_subclass_dispatch() {
    let out = compile_and_run(
        r#"<?php
interface I { function base(): string; }
class A implements I { function base(): string {return "A";} function extra(): string {return "X";} }
class C extends A { function extra(): string {return "C";} }
function f(I $v): string { return $v instanceof A ? $v->extra() : $v->base(); }
echo f(new C());
"#,
    );
    assert_eq!(out, "C");
}

/// Gate boundary (deferred): a by-reference-returning off-interface method (`Box::&ref`) called
/// through an interface receiver MUST fail to compile with the loud unsupported error rather than
/// miscompile. The checker types the off-interface result as `Mixed`, but the by-ref candidate
/// returns a typed ref-cell pointer; storing it unboxed into the `Mixed` slot and rebinding it
/// silently yields a wrong value, so the gate keeps the loud error until a follow-up threads the
/// concrete return type through the narrowed ref-cell binding.
#[test]
#[should_panic(expected = "by-reference-returning method is not yet supported")]
fn test_narrowed_interface_by_reference_return_method_gate() {
    compile_and_run(
        r#"<?php
interface Container { function label(): string; }
class Box implements Container {
    public array $items = ['seed'];
    public function label(): string { return "box"; }
    public function &ref(): array { return $this->items; }
}
function grab(Container $c): string {
    if ($c instanceof Box) {
        $r = &$c->ref();
        $r[] = 'x';
        return implode(',', $r);
    }
    return "n";
}
echo grab(new Box());
"#,
    );
}

/// Nullable `?I` receiver: the narrowed dispatch runs through the nullable twin, which unboxes
/// the receiver and guards against PHP null. `g(new A())` narrows to `A` and calls `extra`;
/// `g(null)` never reaches the narrowed call because `instanceof A` is false.
#[test]
fn test_narrowed_nullable_interface_method_dispatch() {
    let out = compile_and_run(
        r#"<?php
interface I { function base(): string; }
class A implements I { function base(): string {return "A";} function extra(): string {return "X";} }
function g(?I $v): string { return ($v instanceof A) ? $v->extra() : "n"; }
echo g(new A()), g(null);
"#,
    );
    assert_eq!(out, "Xn");
}

/// Gate boundary (deferred to commit 2): an implementor that resolves the off-interface method
/// via `__call` rather than a literal method MUST still fail to compile with the loud unsupported
/// error. A literal-only class-id switch cannot dispatch `__call`, so the gate keeps the loud
/// error to guarantee the narrowed path never miscompiles.
#[test]
#[should_panic(expected = "__call-resolving implementor is not yet supported")]
fn test_narrowed_interface_magic_call_gate_still_errors() {
    compile_and_run(
        r#"<?php
interface I { function base(): string; }
class A implements I { function base(): string {return "A";} function __call($n, $a): string {return "m:$n";} }
function f(I $v): string { return $v instanceof A ? $v->extra() : $v->base(); }
echo f(new A());
"#,
    );
}
