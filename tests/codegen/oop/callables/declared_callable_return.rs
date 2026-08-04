//! Purpose:
//! Integration and regression tests for a DECLARED `: callable` return contract carrying each of
//! PHP's five runtime callable forms.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `PhpType::Callable` is one word: the address of a closure descriptor. PHP's `callable` is a
//!   union of five forms — a `Closure`, a function-name string, `'Class::method'`, an
//!   `[$object, 'method']` / `[Class::class, 'method']` pair, and an object with `__invoke`. Only
//!   the first fit the descriptor slot, so declaring `: callable` used to be strictly NARROWER
//!   than declaring nothing: the same body compiled and ran once the annotation was removed.
//! - The fix widens the EFFECTIVE return slot to boxed `Mixed`, which holds all five to the byte,
//!   and lets the caller invoke through the tag-dispatching dynamic path that already handled
//!   every form arriving from an untyped return.
//! - Widening the SLOT is what makes accepting these forms sound. Checker acceptance alone would
//!   write a string or array word into a descriptor slot — the caller would then invoke garbage.
//! - The negative controls live next to the other checker diagnostics, in
//!   `tests/error_tests/callables.rs`: `return 42;` from a `: callable` function is still a
//!   compile error, so the relaxation admits callable forms only.

use super::*;

/// A function-name string — the form that reads most like a plain value and least like a callable.
#[test]
fn test_declared_callable_return_carries_a_function_name_string() {
    let out = compile_and_run(
        r#"<?php
function pick(bool $upper): callable { return $upper ? 'strtoupper' : 'strtolower'; }
$c = pick(true);
echo $c("hi"), "\n";
$d = pick(false);
echo $d("HI"), "\n";
"#,
    );
    assert_eq!(out, "HI\nhi\n");
}

/// `'Class::method'` — a static method named by a single string.
#[test]
fn test_declared_callable_return_carries_a_static_method_string() {
    let out = compile_and_run(
        r#"<?php
class B { public static function m(string $s): string { return "S" . $s; } }
function get(): callable { return 'B::m'; }
$c = get();
echo $c("w"), "\n";
"#,
    );
    assert_eq!(out, "Sw\n");
}

/// The `[$object, 'method']` pair, returned from a method so `$this` is the receiver.
#[test]
fn test_declared_callable_return_carries_an_object_method_pair() {
    let out = compile_and_run(
        r#"<?php
class K {
    public function make(): callable { return [$this, 'triple']; }
    public function triple(int $n): int { return $n * 3; }
}
$k = new K();
$c = $k->make();
echo $c(5), "\n";
"#,
    );
    assert_eq!(out, "15\n");
}

/// An object with `__invoke` — callable without being a `Closure`.
#[test]
fn test_declared_callable_return_carries_an_invokable_object() {
    let out = compile_and_run(
        r#"<?php
class I { public function __invoke(string $s): string { return "!" . $s . "!"; } }
function get(): callable { return new I(); }
$c = get();
echo $c("q"), "\n";
"#,
    );
    assert_eq!(out, "!q!\n");
}

/// The form that already worked before the widening — kept so the change is proven not to have
/// traded one representable form for another.
#[test]
fn test_declared_callable_return_still_carries_a_closure() {
    let out = compile_and_run(
        r#"<?php
function pick(): callable { return fn (string $s): string => "<" . $s . ">"; }
$c = pick();
echo $c("z"), "\n";
"#,
    );
    assert_eq!(out, "<z>\n");
}

/// `?callable` — the union arm is no more representable than a bare one, so it widens too, and
/// `null` must survive the round trip.
#[test]
fn test_declared_nullable_callable_return_carries_null_and_a_callable() {
    let out = compile_and_run(
        r#"<?php
function get(bool $some): ?callable { return $some ? 'strrev' : null; }
var_dump(get(false));
$c = get(true);
echo $c("abc"), "\n";
"#,
    );
    assert_eq!(out, "NULL\ncba\n");
}

/// A method declaring `: callable` and returning the pair form — the Symfony shape
/// (`EarlyExpirationMessage::findCallback`, `ControllerEvent::getController`).
#[test]
fn test_declared_callable_return_on_a_method_reaches_the_caller_invocable() {
    let out = compile_and_run(
        r#"<?php
class Target { public function run(string $s): string { return "ran:" . $s; } }
class Holder {
    private array $callback;
    public function __construct(array $callback) { $this->callback = $callback; }
    public function findCallback(): callable { return $this->callback; }
}
$h = new Holder([new Target(), 'run']);
$cb = $h->findCallback();
echo $cb("x"), "\n";
"#,
    );
    assert_eq!(out, "ran:x\n");
}
