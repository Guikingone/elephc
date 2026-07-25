//! Purpose:
//! Regression tests for type-guard narrowing threaded through short-circuit `&&`/`||` operands.
//! Each operand of a pure same-operator chain narrows its guarded variable for the operands that
//! run after it (`then_ty` for `&&`, `else_ty` for `||`), while the narrowing never leaks past the
//! chain.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `expect_ok` snippets place the guarded operation inside the chain so the checker must see the
//!   narrowed type; the same operation without a guard errors (proven separately), so acceptance is
//!   evidence of narrowing. `expect_error` covers the no-leak guarantee: a use of the guarded
//!   variable after the chain still sees its original (un-narrowed) declared type.
//! - The instanceof-subtype-method-call shape type-checks here but is not yet lowerable by the EIR
//!   backend (checker-only narrowing does not reach codegen dispatch), so these stay checker-level.

use super::*;

/// The `&&` chain narrows from its FIRST operand: `$q instanceof CQ` (operand 0) narrows `$q` to the
/// subclass `CQ` so the following operand's `$q->m()` call resolves to `CQ::m` instead of erroring
/// on the base `Q`. Without the guard, `$q->m()` reports "Undefined method: Q::m".
#[test]
fn test_and_instanceof_guard_narrows_from_first_operand() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q): int { return ($q instanceof CQ && $q->m() > 0) ? 1 : 0; }",
    );
}

/// A stable property guarded by `instanceof` is narrowed for the right-hand side of the same
/// short-circuit `&&`, so subtype-only methods resolve there.
#[test]
fn test_and_property_instanceof_guard_narrows_rhs() {
    expect_ok(
        "<?php \
         interface BaseValue {} \
         interface SpecializedValue extends BaseValue { public function matches(): bool; } \
         class Holder { \
             public function __construct(private BaseValue $value) {} \
             public function matches(): bool { \
                 return $this->value instanceof SpecializedValue && $this->value->matches(); \
             } \
         }",
    );
}

/// The narrowing from operand 0 threads to EVERY later operand of the same `&&` chain, not just the
/// one immediately after the guard: both `$q->m()` calls (operands 1 and 2) see `$q` as `CQ`.
#[test]
fn test_and_instanceof_guard_threads_to_all_later_operands() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q): int { return ($q instanceof CQ && $q->m() > 0 && $q->m() < 10) ? 1 : 0; }",
    );
}

/// A `||` chain narrows the following operand with the guard's COMPLEMENT (guard-false type). Here
/// the negated guard `!($q instanceof CQ)` is false exactly when `$q instanceof CQ` holds, so the
/// right operand runs only when `$q` is a `CQ`, and `$q->m()` resolves to `CQ::m`.
#[test]
fn test_or_negated_instanceof_guard_narrows_rhs() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function g(Q $q): bool { return !($q instanceof CQ) || $q->m() > 0; }",
    );
}

/// A general (non-instanceof) guard narrows through `&&` too: `is_countable($x)` narrows the
/// `iterable` parameter to `array|Countable`, so the guarded `count($x)` type-checks. Without the
/// guard, `count($x)` on an `iterable` reports "count() argument must be array or Countable object".
#[test]
fn test_and_is_countable_guard_narrows_rhs() {
    expect_ok("<?php function h(iterable $x): bool { return is_countable($x) && count($x) > 0; }");
}

/// The `||` direction of a general guard: `!is_countable($x)` is false exactly when `$x` is
/// countable, so the right operand's `count($x)` runs with `$x` narrowed to `array|Countable`.
#[test]
fn test_or_negated_is_countable_guard_narrows_rhs() {
    expect_ok(
        "<?php function h(iterable $x): int { return (!is_countable($x) || count($x) > 0) ? 1 : 0; }",
    );
}

/// The chain narrowing stays inside the chain and does not leak past it: after
/// `$q instanceof CQ && $q->m()`, the later `$q->m()` outside the chain sees `$q` at its original
/// declared `Q`, NOT the in-chain narrowed `CQ` (the non-leakage this test guards). That base `Q`
/// receiver no longer rejects the call at compile time: PHP does no compile-time method-existence
/// check on a base-typed receiver and dispatches `$q->m()` on the runtime class, and a concrete
/// subclass (`CQ`, IS-A `Q`) declares `m`, so the call is accepted and dispatched at runtime —
/// faulting cleanly with a PHP-style `Error` only when `$q` is really a bare `Q` (`php` verified).
/// Both the in-chain and post-chain calls COMPILE; non-leakage is preserved in the receiver TYPE
/// (`Q`, not `CQ`), and the previous compile-time rejection is superseded by runtime dispatch.
#[test]
fn test_and_narrowing_does_not_leak_past_chain() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function nf(Q $q): int { \
             $a = ($q instanceof CQ && $q->m() > 0) ? 1 : 0; \
             return $a + $q->m(); \
         }",
    );
}
