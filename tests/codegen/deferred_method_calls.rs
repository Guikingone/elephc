//! Purpose:
//! Regressions for method calls the receiver's declared class or interface does not declare.
//!
//! Called from:
//! - `cargo test --test codegen_tests deferred_method_calls`.
//!
//! Key details:
//! - php looks a method up on the object the call meets: a subclass or an implementation may
//!   declare it, and an object without it raises `Error: Call to undefined method C::m()`.
//! - The checker defers such calls and their receiver is boxed so lowering dispatches on the
//!   runtime class; the box must not leak.

use crate::support::*;

/// A subclass method is reachable through a parent-typed receiver, an interface-typed receiver
/// reaches its implementation's extra methods, and a missing method is php's catchable `Error`.
#[test]
fn test_unknown_method_on_declared_type_dispatches_at_run_time() {
    let run = compile_and_run_with_heap_debug(r#"<?php
class A { public function bar() { return "bar"; } }
class B extends A { public function foo() { return "foo"; } }
function call(A $a) { return $a->foo(); }
echo call(new B), "\n";
try { echo call(new A), "\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; }
$a = new A;
try { echo $a->nope(), "\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; }
function children(Iterator $it) { return $it->hasChildren() ? "has" : "none"; }
echo children(new RecursiveArrayIterator([[1], 2])), "\n";
interface Named {}
class N implements Named { public function getName() { return "n"; } }
function nm(Named $x) { return $x->getName(); }
echo nm(new N), "\n";
$n = $argc > 5 ? null : new N;
echo $n?->getName(), "\n";
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "foo\nCall to undefined method A::foo()\nCall to undefined method A::nope()\nhas\nn\nn\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}

/// A receiver typed by a declared return (`Message`, `Account`, `static|Choice`, `A|B`) is never
/// narrowed from a method name; each call dispatches on the object it meets, so only the
/// objects that really lack the method raise php's `Error`. The throwing callers are untyped:
/// a throw out of a Mixed-receiver call inside a typed-return function leaks one block on its
/// own (bead elephc-4ml.17), which this test does not cover.
#[test]
fn test_declared_return_receivers_are_not_narrowed() {
    let run = compile_and_run_with_heap_debug(r#"<?php
class A1 { function only_a() { return "a"; } }
class B1 {}
function make(bool $b): A1|B1 { return $b ? new A1() : new B1(); }
echo make(true)->only_a(), "\n";
try { make(false)->only_a(); } catch (Error $e) { echo $e->getMessage(), "\n"; }
interface Message { public function withHeader(): Message; }
interface Request extends Message { public function requestOnly(): string; }
class Plain implements Message { public function withHeader(): Message { return $this; } }
class Req implements Request {
    public function withHeader(): Message { return new Plain(); }
    public function requestOnly(): string { return "req"; }
}
function read(Request $request) { return $request->withHeader()->requestOnly(); }
try { echo read(new Req), "\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; }
interface Account { public function withdraw(int $amount): Account; }
interface Savings extends Account { public function interestRate(): int; }
final class SavingsAccount implements Savings {
    public function withdraw(int $amount): Account { return $this; }
    public function interestRate(): int { return 4; }
}
function rate(Savings $account): int { return $account->withdraw(10)->interestRate(); }
echo rate(new SavingsAccount()), "\n";
class Choice {
    public function choose(bool $same): static|Choice { return $same ? $this : new Choice(); }
}
class SpecialChoice extends Choice { public function special(): string { return "special"; } }
function render(SpecialChoice $choice, bool $same) { return $choice->choose($same)->special(); }
echo render(new SpecialChoice, true), "\n";
try { echo render(new SpecialChoice, false), "\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; }
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "a\nCall to undefined method B1::only_a()\nCall to undefined method Plain::requestOnly()\n4\n\
         special\nCall to undefined method Choice::special()\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}

/// Deferred calls and undefined constants inside method bodies are rewritten too: lowering emits
/// methods from the checker's stored declarations, not from the class statement.
#[test]
fn test_deferred_calls_inside_method_bodies() {
    let run = compile_and_run_with_heap_debug(r#"<?php
class T {
    public function f() { return NOPE; }
    public function g(T $other): void { $this->nope(); }
    public function h(T $other): void { $other->missing(1); }
}
$t = new T();
try { $t->f(); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { $t->g($t); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { $t->h($t); } catch (Error $e) { echo $e->getMessage(), "\n"; }
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "Undefined constant \"NOPE\"\nCall to undefined method T::nope()\nCall to undefined method T::missing()\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}
