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
