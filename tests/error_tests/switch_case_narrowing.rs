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
/// is therefore NOT narrowed, and `$x->bOnly()` (a `B`-only method) still errors on the `A` member
/// of the `A|B` union. If the gate were broken and the case were narrowed to `B`, this would wrongly
/// type-check.
#[test]
fn test_switch_true_fall_through_case_is_not_narrowed() {
    expect_error(
        "<?php \
         class A {} \
         class B { public function bOnly(): int { return 1; } } \
         function g(A|B $x): int { \
             switch (true) { \
                 case $x instanceof A: \
                 case $x instanceof B: return $x->bOnly(); \
             } \
             return 0; \
         }",
        "Undefined method: A::bOnly",
    );
}

/// Positive control for the gate: when the `A` case terminates (`return 0`), the `B` case can no
/// longer be reached by fall-through, so it IS fall-in-safe and its body is narrowed to `B`. The
/// `$x->bOnly()` call then type-checks. This proves the gate narrows the safe case rather than
/// refusing all narrowing.
#[test]
fn test_switch_true_terminating_predecessor_allows_narrowing() {
    expect_ok(
        "<?php \
         class A {} \
         class B { public function bOnly(): int { return 1; } } \
         function g(A|B $x): int { \
             switch (true) { \
                 case $x instanceof A: return 0; \
                 case $x instanceof B: return $x->bOnly(); \
             } \
             return 0; \
         }",
    );
}

/// The case-body narrowing does not leak past the switch: the in-case `$q->m()` type-checks under
/// the narrowed `CQ`, but the `$q->m()` after the switch sees `$q` at its original declared `Q` and
/// errors. If narrowing leaked to the outer scope, the post-switch call would wrongly type-check.
#[test]
fn test_switch_true_narrowing_does_not_leak_past_switch() {
    expect_error(
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
        "Undefined method: Q::m",
    );
}

/// Non-regression: a value switch (`switch ($flag)`, subject is not the literal `true`) is never
/// narrowed — its case values are compared to the subject, not evaluated as guards. `$q->m()` in the
/// case body therefore sees `$q` as `Q` and errors, exactly as before this feature.
#[test]
fn test_value_switch_subject_does_not_narrow() {
    expect_error(
        "<?php \
         class Q {} \
         class CQ extends Q { public function m(): int { return 1; } } \
         function f(Q $q, bool $flag): int { \
             switch ($flag) { \
                 case $q instanceof CQ: return $q->m(); \
                 default: return 0; \
             } \
         }",
        "Undefined method: Q::m",
    );
}
