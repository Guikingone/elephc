//! Purpose:
//! Regression tests for WIN B member-path type narrowing inside a ternary guard
//! (`$this->prop instanceof X ? $this->prop->methodOnX() : …`). The guarded
//! property receiver is narrowed to the `instanceof` subtype only inside the
//! ternary's then-branch, so a subtype-only method resolves there while the
//! declared (interface) type is still used everywhere else.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Narrowing is TERNARY-ONLY and THEN-BRANCH-ONLY: the else-branch, a use after
//!   the ternary, and an `if`-block guard all keep the declared type (proven by the
//!   negative controls). The variable-rooted ternary (WIN A) must stay green too.
//! - Acceptance means no `Undefined method` diagnostic; the narrowed subtype-method
//!   call type-checks here but is not yet lowerable by the EIR backend (checker-only
//!   narrowing does not reach codegen dispatch), so these stay checker-level.

use super::*;

/// A fixture with an interface `I`, a sub-interface `J` declaring an extra method,
/// and a class `C` holding a `private I $x`. The `{BODY}` placeholder is replaced
/// with the method body under test.
fn fixture(body: &str) -> String {
    format!(
        "<?php \
         interface I {{ public function base(): void; }} \
         interface J extends I {{ public function extra(): void; }} \
         class C {{ \
             private I $x; \
             public function __construct(I $x) {{ $this->x = $x; }} \
             public function m() {{ {} }} \
         }}",
        body
    )
}

/// Core WIN: a ternary guard `$this->x instanceof J` narrows the property receiver to
/// `J` inside the then-branch, so `$this->x->extra()` (a `J`-only method) resolves
/// instead of erroring on the declared interface `I`.
#[test]
fn test_ternary_member_path_narrows_then_branch() {
    expect_ok(&fixture(
        "$y = $this->x instanceof J ? $this->x->extra() : null;",
    ));
}

/// Negative control — no leak past the ternary: after the guarded ternary, a second
/// UNGUARDED `$this->x->extra()` still sees the declared `I` and must error. Proves
/// the overlay lives only in the ternary's discarded then-env.
#[test]
fn test_ternary_member_path_does_not_leak_past_ternary() {
    expect_error(
        &fixture(
            "$y = $this->x instanceof J ? $this->x->extra() : null; $this->x->extra();",
        ),
        "Undefined method",
    );
}

/// Negative control — the else-branch is NOT narrowed: `$this->x->extra()` in the
/// else arm keeps the declared `I` and must error. WIN B narrows the then-side only.
#[test]
fn test_ternary_member_path_else_branch_not_narrowed() {
    expect_error(
        &fixture("$y = $this->x instanceof J ? null : $this->x->extra();"),
        "Undefined method",
    );
}

/// Verifies `if`-block member-path narrowing now works: the property-fact narrowing
/// (landed on main) narrows `$this->x` inside the guarded block, so
/// `if ($this->x instanceof J) { $this->x->extra(); }` type-checks. This was previously
/// a negative control while if-block member narrowing was deferred.
#[test]
fn test_if_block_member_path_not_narrowed() {
    expect_ok(&fixture("if ($this->x instanceof J) { $this->x->extra(); }"));
}

/// Verifies a divergent negated property guard keeps its complement through a nested `if`.
/// This is the common framework shape that validates a specialized output before using it.
#[test]
fn test_divergent_negated_property_guard_narrows_nested_if() {
    expect_ok(
        "<?php \
         interface BaseOutput { public function write(): void; } \
         class SectionOutput implements BaseOutput { \
             public function write(): void {} \
             public function clear(int $lines): void {} \
         } \
         class NestedPropertyGuard { \
             public function __construct(private BaseOutput $output) {} \
             private function countLines(): int { return 1; } \
             public function render(bool $alreadyRendered): void { \
                 if (!$this->output instanceof SectionOutput) { throw new Exception('bad'); } \
                 if ($alreadyRendered) { $this->output->clear($this->countLines()); } \
             } \
         }",
    );
}

/// Verifies a function-call argument keeps the type computed before that argument's method call invalidates property facts.
#[test]
fn test_guarded_property_method_argument_keeps_evaluated_type() {
    expect_ok(
        "<?php \
         interface BaseOutput {} \
         final class SectionOutput implements BaseOutput { \
             public function lineCount(): int { return 1; } \
         } \
         function consume(int $count, string $message): void {} \
         final class Renderer { \
             public function __construct(private BaseOutput $output) {} \
             public function render(string $message): void { \
                 if ($this->output instanceof SectionOutput) { \
                     consume($this->output->lineCount(), $message); \
                 } \
             } \
         }",
    );
}

/// Regression — the variable-rooted ternary narrowing (WIN A) still works: a plain
/// `$v instanceof J ? $v->extra() : null` accepts, proving the added member-path
/// branch did not disturb the existing variable-guard path.
#[test]
fn test_ternary_variable_narrowing_still_works() {
    expect_ok(
        "<?php \
         interface I { public function base(): void; } \
         interface J extends I { public function extra(): void; } \
         function f(I $v) { $y = $v instanceof J ? $v->extra() : null; }",
    );
}
