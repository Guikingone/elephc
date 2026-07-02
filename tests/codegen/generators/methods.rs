//! Purpose:
//! Generator methods — class instance/static methods whose bodies contain
//! `yield`/`yield from` must be recognized as generators and return a
//! `Generator` object (never `Void` or the declared `iterable`/`Generator`
//! hint value), mirroring how generator free-functions already behave.
//!
//! Called from:
//!  - `cargo test` via the integration test harness; aggregated under
//!    `tests::codegen::generators` in `tests/codegen/generators/mod.rs`.
//!
//! Key details:
//!  - Regression coverage for the method-return-type pass generator guard
//!    (`src/types/checker/method_pass.rs::update_method_return_type`). Before
//!    the guard, a generator method with a declared `: iterable` hint and no
//!    `return` wrongly fired "must return a value on every path" and its
//!    effective return type was overwritten to `Void`/the hint.

use crate::support::*;

/// Verifies a generator INSTANCE method declared `: iterable` with only `yield`
/// (no `return`) type-checks and iterates. Before the method-pass generator
/// guard this failed with "Method 'Bag::all' must return a value on every path".
/// Cross-checked against `php -r` → "123".
#[test]
fn test_generator_instance_method_with_iterable_hint() {
    let out = compile_and_run(
        r#"<?php
class Bag {
    private array $items = [1, 2, 3];
    public function all(): iterable { foreach ($this->items as $x) { yield $x; } }
}
$b = new Bag();
foreach ($b->all() as $x) { echo $x; }
"#,
    );
    assert_eq!(out, "123");
}

/// Verifies a generator method with NO declared return type + only `yield`
/// infers a `Generator` return (not `Void`), so the caller can iterate it.
/// Cross-checked against `php -r` → "12".
#[test]
fn test_generator_method_without_declared_return_type() {
    let out = compile_and_run(
        r#"<?php
class C {
    public function g() { yield 1; yield 2; }
}
$c = new C();
foreach ($c->g() as $x) { echo $x; }
"#,
    );
    assert_eq!(out, "12");
}

/// Verifies a STATIC generator method declared `: iterable` returns a
/// `Generator` object identically to an instance generator method — the
/// method-pass generator guard runs before the static/instance slot write.
/// Cross-checked against `php -r` → "12".
#[test]
fn test_static_generator_method_with_iterable_hint() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static function g(): iterable { yield 1; yield 2; }
}
foreach (C::g() as $x) { echo $x; }
"#,
    );
    assert_eq!(out, "12");
}

/// Repro D from the spec: `yield from $this->otherGeneratorMethod()`. The
/// generator-method RETURN fix makes `$this->all()` type as `Object("Generator")`
/// (verified: the diagnostic reads `got Object("Generator")`, not `got Iterable`),
/// and the `yield from` CONSUMER arm
/// (`src/types/checker/inference/expr/mod.rs`) now gates on the operand's TYPE
/// (array or Generator) rather than its syntactic kind, so a METHOD-CALL
/// operand returning a Generator is accepted and delegated via
/// `__rt_gen_delegate`. Cross-checked against `php -r` → "123".
#[test]
fn test_generator_method_yield_from_generator_method_call() {
    let out = compile_and_run(
        r#"<?php
class Bag {
    private array $items = [1, 2, 3];
    public function all(): iterable { foreach ($this->items as $x) { yield $x; } }
    public function combined(): iterable { yield from $this->all(); }
}
$b = new Bag();
foreach ($b->combined() as $x) { echo $x; }
"#,
    );
    assert_eq!(out, "123");
}

/// Verifies `yield from self::inner()` where a STATIC generator method delegates
/// to another static generator method. `self::inner()` types as
/// `Object("Generator")`, which the type-based `yield from` gate now accepts
/// (any syntactic form of a Generator-typed operand is delegated via
/// `__rt_gen_delegate`). Cross-checked against `php -r` → "12".
#[test]
fn test_generator_yield_from_static_method_delegation() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static function inner(): iterable { yield 1; yield 2; }
    public static function outer(): iterable { yield from self::inner(); }
}
foreach (C::outer() as $x) { echo $x; }
"#,
    );
    assert_eq!(out, "12");
}

/// Repro X1: a `: iterable` generator method that RECURSIVELY delegates to
/// itself via `yield from $this->upto(...)` (receiver `$this`, typed as the
/// declaring class, so the callee's return type is resolved from its own
/// signature). Before the `build_method_sig` generator seed fix, `upto`'s
/// signature was seeded from the `: iterable` hint, so while `upto`'s body was
/// being checked (before the method pass ran) the self-call `$this->upto(...)`
/// resolved as `Iterable` and `yield from` rejected it ("got Iterable").
/// Seeding `Generator` at the signature layer makes the self-recursive call
/// resolve as a delegatable Generator from the start.
/// Cross-checked against `php -r` → "321".
#[test]
fn test_generator_method_direct_self_recursion() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public function upto(int $n): iterable {
        yield $n;
        if ($n > 1) { yield from $this->upto($n - 1); }
    }
}
$c = new Counter();
foreach ($c->upto(3) as $v) { echo $v; }
"#,
    );
    assert_eq!(out, "321");
}

/// Repro Z1: two `: iterable` generator methods `a()`/`b()` that MUTUALLY
/// recurse via `yield from $this->b()` / `yield from $this->a()` with a
/// terminating guard. Each method's signature is now seeded `Generator`, so the
/// cross-references resolve as delegatable Generators from the start (before the
/// method pass runs) instead of the stale `Iterable` seed.
/// Cross-checked against `php -r` → "120340".
#[test]
fn test_generator_method_mutual_recursion() {
    let out = compile_and_run(
        r#"<?php
class Spec {
    public function a(int $n): iterable { yield $n; if ($n < 4) { yield from $this->b($n + 1); } }
    public function b(int $n): iterable { yield $n * 10; if ($n < 4) { yield from $this->a($n + 1); } }
}
$s = new Spec();
foreach ($s->a(1) as $v) { echo $v; }
"#,
    );
    assert_eq!(out, "120340");
}
