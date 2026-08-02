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

/// Reaching the body of an `if (A || B)` proves at least one disjunct true, so a binding EVERY
/// disjunct constrains takes the union of their guard-true types. `Cc` is ruled out, so `$v->n` —
/// declared on `Aa` and `Bb` but not on `Cc` — resolves. Before this narrowing existed the union
/// stayed `Aa|Bb|Cc` and the access reported "Undefined property: Cc::n".
#[test]
fn test_or_chain_body_narrows_to_union_of_disjuncts() {
    expect_ok(
        "<?php \
         class Aa { public string $n = 'A'; } \
         class Bb { public string $n = 'B'; } \
         class Cc { public string $z = 'C'; } \
         function good(Aa|Bb|Cc $v): string { \
             if ($v instanceof Aa || $v instanceof Bb) { return $v->n; } \
             return 'cc'; \
         }",
    );
}

/// The `||` body union is admitted only when EVERY disjunct constrains the binding. Here the second
/// disjunct is an unrelated boolean, so reaching the body proves nothing about `$v` and the access
/// must still be rejected against the full declared union.
#[test]
fn test_or_chain_body_narrowing_requires_every_disjunct() {
    expect_error(
        "<?php \
         class Aa { public string $n = 'A'; } \
         class Bb { public string $other = 'B'; } \
         function bad(Aa|Bb $v, bool $ok): string { \
             if ($ok || $v instanceof Aa) { return $v->n; } \
             return 'x'; \
         }",
        "Undefined property: Bb::n",
    );
}

/// Falling past an `if (A && B)` proves `!A || !B`, representable here because both conjuncts
/// constrain `$s`: the fall-through type is `Sess|null`, so the later `null` test leaves `Sess` and
/// `$s->tag` resolves. This is Symfony `Request::getSession`'s shape. Before the `&&` complement
/// existed `$s` rejoined to its declared `Sess|Other|null` and the access reported
/// "Undefined property: Other::tag".
#[test]
fn test_and_chain_fall_through_narrows_when_every_conjunct_constrains() {
    expect_ok(
        "<?php \
         class Sess { public string $tag = 'S'; } \
         class Other { public string $other = 'O'; } \
         function pick(Sess|Other|null $s): string { \
             if (!$s instanceof Sess && null !== $s) { return 'other'; } \
             if ($s === null) { return 'null'; } \
             return $s->tag; \
         }",
    );
}

/// The `&&` fall-through complement is admitted only when EVERY conjunct constrains the binding.
/// `$flag` constrains nothing, so `$s` keeps its declared union past the `if`.
#[test]
fn test_and_chain_fall_through_narrowing_requires_every_conjunct() {
    expect_error(
        "<?php \
         class Sess { public string $tag = 'S'; } \
         class Other { public string $other = 'O'; } \
         function bad(Sess|Other|null $s, bool $flag): string { \
             if (!$s instanceof Sess && $flag) { return 'x'; } \
             return $s->tag; \
         }",
        "Undefined property: Other::tag",
    );
}

/// A chain complement may persist past the `if` only when no clause falls through. Here the body
/// falls through, so the path that ran it also reaches the following statement and `$s` must rejoin
/// to its declared union.
#[test]
fn test_and_chain_complement_dropped_when_body_falls_through() {
    expect_error(
        "<?php \
         class Sess { public string $tag = 'S'; } \
         class Other { public string $other = 'O'; } \
         function bad(Sess|Other|null $s): string { \
             if (!$s instanceof Sess && null !== $s) { echo 'other'; } \
             return $s->tag; \
         }",
        "Undefined property: Other::tag",
    );
}

/// A chain complement marks the construct as having applied a guard, which makes the post-`if`
/// restore keep the accumulated complement for an else-less chain whose every clause diverges.
/// This is the all-diverging shape: clause 1 contributes the `&&` complement, clause 2 diverges
/// without constraining anything, and `$s->tag` past the chain must see `Sess`.
#[test]
fn test_all_diverging_chain_keeps_accumulated_complement() {
    expect_ok(
        "<?php \
         class Sess { public string $tag = 'S'; } \
         class Other { public string $other = 'O'; } \
         function f(Sess|Other|null $s, int $n): string { \
             if (!$s instanceof Sess && null !== $s) { return 'a'; } \
             elseif ($n < 0) { return 'b'; } \
             if ($s === null) { return 'null'; } \
             return $s->tag; \
         }",
    );
}

/// Keeping the complement skips the post-`if` restore loop wholesale, so this pins that nothing
/// ELSE leaks through the skipped restore. `$t` is narrowed to `Aa` only inside clause 2's body by
/// the `&&` then-side; that narrowing is undone at the body's end and must not survive, because a
/// chain where only one conjunct constrains `$t` proves nothing about it on the fall-through edge.
#[test]
fn test_all_diverging_chain_does_not_leak_then_side_narrowing() {
    expect_error(
        "<?php \
         class Sess { public string $tag = 'S'; } \
         class Other { public string $other = 'O'; } \
         class Aa { public string $n = 'A'; } \
         class Bb { public string $z = 'B'; } \
         function h(Sess|Other|null $s, Aa|Bb $t, bool $flag): string { \
             if (!$s instanceof Sess && null !== $s) { return 'a'; } \
             elseif ($t instanceof Aa && $flag) { return 'b'; } \
             if ($s === null) { return 'n'; } \
             return $s->tag . $t->n; \
         }",
        "Undefined property: Bb::n",
    );
}

/// The De Morgan complement of a diverging single-clause `if (A || B)` is applied exactly once.
/// The clause loop now persists it on the fall-through edge, and the post-join single-clause block
/// that used to do the same work is suppressed, so the disjuncts' guard-false types are never
/// derived a second time against an already-narrowed environment. `$x` is `Foo` after the `if`.
#[test]
fn test_or_chain_diverging_complement_applied_once() {
    expect_ok(
        "<?php \
         class Foo { public string $name = 'F'; } \
         class Bar { public string $other = 'B'; } \
         function f(Foo|Bar|null $x, bool $b): string { \
             if (!$x instanceof Foo || $b) { return 'a'; } \
             return $x->name; \
         }",
    );
}

/// EVERY disjunct's complement survives, not just the last one. `!(A || B)` is `!A && !B`, so the
/// fall-through edge of `if ($x instanceof Rr || $x instanceof Ss)` must drop BOTH classes and
/// leave `string|null`, which `take()` accepts.
///
/// The per-disjunct complements used to be combined with `narrow_to`, whose union target is a
/// replacement proof rather than a set (see `guard_matches`): intersecting `string|Ss|null` with
/// `string|Rr|null` matched no member and fell back to returning the second operand whole, so
/// `Rr` came back and only `Ss` was ever subtracted. This is Symfony
/// `DependencyInjection\Definition::setFactory`'s shape, where the leaked `Reference` arm made the
/// `string|array|null` property store fail.
#[test]
fn test_or_chain_complement_subtracts_every_disjunct() {
    expect_ok(
        "<?php \
         class Rr {} \
         class Ss {} \
         class Sink { public function take(string|null $v): string { return $v ?? 'n'; } } \
         function f(string|Rr|Ss|null $x): string { \
             if ($x instanceof Rr || $x instanceof Ss) { return 'obj'; } \
             return (new Sink())->take($x); \
         }",
    );
}

/// The complement intersection may only REFINE: a disjunct that constrains a different binding
/// contributes no fact, so `$y` still carries its full declared union after the `if` and the
/// `Tt`-only property access stays rejected.
#[test]
fn test_or_chain_complement_does_not_invent_facts_for_other_bindings() {
    expect_error(
        "<?php \
         class Tt { public string $tag = 'T'; } \
         class Uu { public string $other = 'U'; } \
         function f(Tt|Uu $x, Tt|Uu $y): string { \
             if ($x instanceof Tt || $x instanceof Uu) { return 'x'; } \
             return $y->tag; \
         }",
        "Undefined property: Uu::tag",
    );
}
