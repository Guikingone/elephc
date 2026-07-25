//! Purpose:
//! Regression tests for `switch (true)` case-body type-guard narrowing. Each fall-in-safe,
//! single-value case body sees its guard's narrowing (single guard or pure `&&` chain), while
//! narrowing is refused for a case reachable by fall-through from a non-terminating case and never
//! leaks past the switch.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `expect_ok` snippets place the guarded method call inside a fall-in-safe case body so the
//!   checker must see the narrowed subclass; the same call errors without narrowing (proven by the
//!   value-switch and fall-through tests). `expect_error` covers the two soundness guarantees:
//!   a fall-through case is NOT narrowed, and a value switch (`switch ($x)`, not `switch (true)`)
//!   is never narrowed.
//! - The instanceof-subtype-method-call shape type-checks here but is not yet lowerable by the EIR
//!   backend (checker-only narrowing does not reach codegen dispatch), so these stay checker-level.

use super::*;

/// A fall-in-safe `switch (true)` case body is narrowed by its guard, for both a compound `&&`
/// guard (`$q instanceof CQ && $b` — the instanceof contributes the narrowing, the bool operand
/// none) and a plain `instanceof` guard: `$q->m()` in each body resolves to `CQ::m`. Without
/// narrowing, `$q->m()` errors "Undefined method: Q::m".
#[test]
fn test_switch_true_case_body_narrows_single_and_compound_guards() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q, bool $b): int { \
             switch (true) { \
                 case $q instanceof CQ && $b: return $q->m(); \
                 case $q instanceof CQ: return $q->m(); \
                 default: return 0; \
             } \
         }",
    );
}

/// Fall-through soundness (the gate): the `A` case has no `break` and falls into the `B` case, so
/// the `B` case is reachable at runtime with `$x` being an `A` (its guard false). The `B` case body
/// is therefore NOT narrowed and still sees `$x` at its `A|B` union type.
///
/// SPEC G1 updated this gate's discriminator: PHP-faithful union method dispatch now type-checks
/// `$x->bOnly(...)` as long as AT LEAST ONE union member declares `bOnly` (previously the checker
/// required every member to declare it, so an un-narrowed `A|B` receiver calling a `B`-only method
/// always errored — that blunt signal no longer exists). To keep a compile-time discriminator, both
/// `A` and `B` now declare `bOnly` but with INCOMPATIBLE parameter types (`int` vs `string`): under
/// the new rule, two-or-more resolving members must ALL accept the call's arguments (JURY ADDENDUM
/// #1), so an un-narrowed `A|B` receiver calling `bOnly("s")` still errors (loud, on `A`'s
/// mismatched parameter) — but a receiver correctly narrowed to `B` alone (single resolving member,
/// see the positive control below) accepts it cleanly. If the gate were broken and this case were
/// wrongly narrowed to `B`, the call would exercise the single-member path and wrongly type-check.
#[test]
fn test_switch_true_fall_through_case_is_not_narrowed() {
    expect_error(
        "<?php \
         class A { public function bOnly(int $v): int { return 1; } } \
         class B { public function bOnly(string $v): int { return 2; } } \
         function g(A|B $x): int { \
             switch (true) { \
                 case $x instanceof A: \
                 case $x instanceof B: return $x->bOnly(\"s\"); \
             } \
             return 0; \
         }",
        "Method A::bOnly parameter $v expects Int, got Str",
    );
}

/// Positive control for the gate: when the `A` case terminates (`return 0`), the `B` case can no
/// longer be reached by fall-through, so it IS fall-in-safe and its body is narrowed to plain `B`
/// (not the `A|B` union). `$x->bOnly("s")` then resolves against `B::bOnly(string)` alone and
/// type-checks, even though `A::bOnly` (int-typed) would have rejected the same call — proving the
/// gate narrows the safe case (single-member dispatch) rather than refusing all narrowing.
#[test]
fn test_switch_true_terminating_predecessor_allows_narrowing() {
    expect_ok(
        "<?php \
         class A { public function bOnly(int $v): int { return 1; } } \
         class B { public function bOnly(string $v): int { return 2; } } \
         function g(A|B $x): int { \
             switch (true) { \
                 case $x instanceof A: return 0; \
                 case $x instanceof B: return $x->bOnly(\"s\"); \
             } \
             return 0; \
         }",
    );
}

/// The case-body narrowing does not leak past the switch: the post-switch `$q->m()` sees `$q` at its
/// original declared `Q`, NOT the in-case narrowed `CQ` (the non-leakage this test guards). That base
/// `Q` receiver no longer rejects the call at compile time, though: PHP performs no compile-time
/// method-existence check on a base-typed receiver and dispatches `$q->m()` on the runtime class, and
/// a concrete subclass (`CQ`, which IS-A `Q`) declares `m`, so the call is accepted and dispatched at
/// runtime — faulting cleanly with a PHP-style `Error` only when `$q` is really a bare `Q` (`php`
/// verified). So both the narrowed in-case call and the un-narrowed post-switch call COMPILE; the
/// non-leakage is preserved in the receiver TYPE (`Q`, not `CQ`), and the previous compile-time
/// rejection of the post-switch call is superseded by PHP-faithful runtime dispatch.
#[test]
fn test_switch_true_narrowing_does_not_leak_past_switch() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q): int { \
             switch (true) { \
                 case $q instanceof CQ: return $q->m(); \
                 default: break; \
             } \
             return $q->m(); \
         }",
    );
}

/// Non-regression: a value switch (`switch ($flag)`, subject is not the literal `true`) is never
/// narrowed — its case values are compared to the subject, not evaluated as guards — so `$q->m()` in
/// the case body sees `$q` as `Q`, not `CQ` (the non-narrowing this test guards). That base `Q`
/// receiver is accepted via PHP-faithful lenient dispatch (PHP does no compile-time method-existence
/// check; the concrete subclass `CQ` IS-A `Q` declares `m`, so the call dispatches on the runtime
/// class and faults cleanly only for a bare `Q`). The value-switch subject is still not narrowed —
/// the receiver stays `Q` — but the call now COMPILES and dispatches at runtime instead of being
/// rejected at compile time.
#[test]
fn test_value_switch_subject_does_not_narrow() {
    expect_ok(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q, bool $flag): int { \
             switch ($flag) { \
                 case $q instanceof CQ: return $q->m(); \
                 default: return 0; \
             } \
         }",
    );
}
