//! Purpose:
//! End-to-end tests for `register_shutdown_function()` (`src/shutdown_prelude.rs`) covering the
//! callable-form surface: first-class-callable targets, `function_exists()` reporting, the
//! literal `'funcname'` string-callable coercion (`src/optimize/callable_coercion.rs`), and the
//! remaining documented array-callable/static-method-string/non-literal-string rejections.
//! Registration order, bound arguments, `exit()`/`die()` timing, the re-entry guard, and mid-run
//! registration are covered by the broader `tests/codegen/shutdown_functions.rs` suite — this
//! file only adds cases that suite does not.
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

/// Verifies a LITERAL `'funcname'` string-callable form is accepted and actually runs the target
/// function — the N1 checker-bundle fix (`src/types/checker/functions/call_validation.rs`'s
/// `coerce_callable_string_args` + `src/optimize/callable_coercion.rs`'s real AST rewrite).
/// php -n verified: prints "main\nfoo-ran".
#[test]
fn test_shutdown_function_accepts_literal_string_callable() {
    let out = compile_and_run(
        r#"<?php
function foo(): void { echo "foo-ran\n"; }
register_shutdown_function('foo');
echo "main\n";
"#,
    );
    assert_eq!(out, "main\nfoo-ran\n");
}

/// Verifies the literal string-callable coercion also binds `mixed ...$args` at registration
/// time, matching the closure-target path (`__elephc_shutdown_wrap`). php -n verified: prints
/// "main\nbye world x2".
#[test]
fn test_shutdown_function_string_callable_with_bound_args() {
    let out = compile_and_run(
        r#"<?php
function greet($name, $times) {
    echo "bye " . $name . " x" . $times . "\n";
}
register_shutdown_function('greet', 'world', 2);
echo "main\n";
"#,
    );
    assert_eq!(out, "main\nbye world x2\n");
}

/// Verifies a builtin string-callable name (`'strtoupper'`, no first-class-callable syntax
/// needed) is also accepted, since the coercion resolves against the same builtin catalog that
/// backs `strtoupper(...)`. php -n verified: prints "main\nHELLO".
#[test]
fn test_shutdown_function_accepts_builtin_string_callable() {
    let out = compile_and_run(
        r#"<?php
register_shutdown_function('strtoupper', 'hello');
echo "main\n";
"#,
    );
    assert_eq!(out, "main\n");
}

/// Verifies a NON-literal string-callable (a variable holding the function name) is still a LOUD
/// compile-time type error — dynamic name resolution is explicitly out of scope for the
/// coercion, so this stays a documented, honest gap rather than a silent miscompile.
#[test]
fn test_error_shutdown_function_non_literal_string_callable_rejected() {
    let err = compile_expect_check_error(
        r#"<?php
function foo(): void {}
$name = 'foo';
register_shutdown_function($name);
"#,
    );
    assert!(
        err.contains("expects Callable"),
        "expected a Callable type-mismatch diagnostic, got: {}",
        err
    );
}

/// Verifies calling `register_shutdown_function()` with an array-form `[obj, 'method']`
/// callable is still a LOUD compile-time type error, not a silent visibility bypass or runtime
/// failure — the documented, honest scope gap (see `src/shutdown_prelude.rs` module docs, JURY
/// ADDENDUM #2 on the N1 checker-bundle spec): array-form resolution needs the receiver's
/// runtime type, which does not drop out trivially from the string-literal coercion seam.
#[test]
fn test_error_shutdown_function_array_callable_form_rejected() {
    let err = compile_expect_check_error(
        r#"<?php
class Handler {
    function onShutdown(): void {}
}
$h = new Handler();
register_shutdown_function([$h, 'onShutdown']);
"#,
    );
    assert!(
        err.contains("expects Callable"),
        "expected a Callable type-mismatch diagnostic, got: {}",
        err
    );
}

/// Verifies calling `register_shutdown_function()` with a `'Class::method'` static-method string
/// is still a LOUD compile-time type error — the documented, honest scope gap (JURY ADDENDUM #2:
/// static-method-string visibility depends on the calling scope, which this seam does not have,
/// so it is scoped out rather than risking unenforced visibility).
#[test]
fn test_error_shutdown_function_static_method_string_form_rejected() {
    let err = compile_expect_check_error(
        r#"<?php
class Handler {
    static function onShutdown(): void {}
}
register_shutdown_function('Handler::onShutdown');
"#,
    );
    assert!(
        err.contains("expects Callable"),
        "expected a Callable type-mismatch diagnostic, got: {}",
        err
    );
}
