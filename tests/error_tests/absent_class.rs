//! Purpose:
//! Diagnostic tests for the checker's absent-class tolerance: a class name that is unresolved
//! everywhere in the closed world degrades to `Mixed` with a warning (not a hard error) in every
//! reference position, while genuinely-malformed type syntax and reserved-word misuse keep erroring.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `expect_warning` asserts the source type-checks and carries the absent-class warning; the
//!   softened typo detection is the accepted cost of compiling frameworks with optional dependencies.
//! - Reserved-word/syntax errors must still hard-error, so the tolerance is not over-broad.

use super::*;

/// The message substring shared by every absent-class degradation warning.
const ABSENT_MESSAGE: &str = "treated as an absent optional dependency";

/// Verifies an unresolved class used as a parameter type hint is a warning, not an error: the
/// program type-checks and the absent-class warning is emitted (`array|\Process`-style hints on
/// uninstalled optional dependencies must compile).
#[test]
fn test_absent_class_type_hint_warns_not_errors() {
    expect_warning("<?php function f(\\No\\Such $x) { return $x; }", ABSENT_MESSAGE);
}

/// Verifies a union type hint mixing a known type with an absent class warns and compiles,
/// mirroring symfony's `array|\Process` optional-dependency signatures.
#[test]
fn test_absent_class_union_type_hint_warns_not_errors() {
    expect_warning(
        "<?php function f(array|\\No\\Such $cmd) { return $cmd; }",
        ABSENT_MESSAGE,
    );
}

/// Verifies `new AbsentClass()` is tolerated as `Mixed` with a warning instead of an
/// "Undefined class" error.
#[test]
fn test_absent_class_new_warns_not_errors() {
    expect_warning(
        "<?php function f() { return new \\No\\Such(); }",
        ABSENT_MESSAGE,
    );
}

/// Verifies a static call on an absent class (`Absent::from(...)`) is tolerated as `Mixed` with a
/// warning instead of an "Undefined class" error.
#[test]
fn test_absent_class_static_call_warns_not_errors() {
    expect_warning(
        "<?php function f() { return \\No\\Such::from(1); }",
        ABSENT_MESSAGE,
    );
}

/// Verifies `catch (AbsentException $e)` is tolerated with a warning instead of an
/// "Undefined class" error (an uninstalled optional-dependency exception simply never matches).
#[test]
fn test_absent_class_catch_warns_not_errors() {
    expect_warning(
        "<?php function f() { try { echo 1; } catch (\\No\\Such\\Exc $e) { echo 2; } }",
        ABSENT_MESSAGE,
    );
}

/// Guards against over-broad tolerance: `self` as a type outside a class context is a reserved-word
/// misuse that must still hard-error rather than degrade to `Mixed`.
#[test]
fn test_reserved_self_type_outside_class_still_errors() {
    expect_error(
        "<?php function f(self $x) { return $x; }",
        "Cannot use 'self' as a type outside of a class",
    );
}

/// Guards against over-broad tolerance: instantiating a class-like symbol that *is* known but is
/// not instantiable (an interface) still hard-errors. The tolerance only fires for a class name
/// absent everywhere in the closed world, so the interface guard runs before it.
#[test]
fn test_new_on_known_interface_still_errors() {
    expect_error(
        "<?php interface I {} function f() { return new I(); }",
        "Cannot instantiate interface",
    );
}
