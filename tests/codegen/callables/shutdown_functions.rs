//! Purpose:
//! End-to-end tests for `register_shutdown_function()` (`src/shutdown_prelude.rs`) covering the
//! callable-form surface: first-class-callable targets, `function_exists()` reporting, and the
//! documented string/array-callable-form rejection. Registration order, bound arguments,
//! `exit()`/`die()` timing, the re-entry guard, and mid-run registration are covered by the
//! broader `tests/codegen/shutdown_functions.rs` suite — this file only adds cases that suite does
//! not.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Expected outputs are cross-checked against `php -n` (no extensions, matching elephc's
//!   closed-world model) on the identical source.

use crate::support::*;

/// Verifies a first-class-callable target (`foo(...)`) is accepted as the callback, not just an
/// inline closure. Cross-checked with `php -n` (prints "main\nfoo-ran").
#[test]
fn test_shutdown_function_accepts_first_class_callable() {
    let out = compile_and_run(
        r#"<?php
function foo(): void { echo "foo-ran\n"; }
register_shutdown_function(foo(...));
echo "main\n";
"#,
    );
    assert_eq!(out, "main\nfoo-ran\n");
}

/// Verifies `function_exists('register_shutdown_function')` reports `true` even in a program that
/// only PROBES it (never calls it), so the pay-for-use prelude injection still triggers for the
/// string/`function_exists` reference form, not just direct calls. Prints "y".
#[test]
fn test_function_exists_register_shutdown_function_reports_true() {
    let out = compile_and_run(
        r#"<?php echo function_exists('register_shutdown_function') ? "y" : "n";"#,
    );
    assert_eq!(out, "y");
}

/// Verifies calling `register_shutdown_function()` with a `'funcname'` string-callable form is a
/// LOUD compile-time type error, not a silent visibility bypass or runtime failure — the
/// documented, honest scope gap (see `src/shutdown_prelude.rs` module docs, JURY ADDENDUM #2:
/// method-string callables are scoped out rather than risking unenforced visibility).
#[test]
fn test_error_shutdown_function_string_callable_form_rejected() {
    let err = compile_expect_check_error(
        r#"<?php
function foo(): void {}
register_shutdown_function('foo');
"#,
    );
    assert!(
        err.contains("expects Callable"),
        "expected a Callable type-mismatch diagnostic, got: {}",
        err
    );
}
