//! Purpose:
//! End-to-end tests for the `register_shutdown_function()` prelude
//! (`elephc::shutdown_prelude`): registration-ordered invocation at normal script end and before
//! `exit()`/`die()`, bound-argument support, mid-run registration, the re-entry guard, and the
//! user-declaration override.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Expected outputs are cross-checked against `php -n` (the CLI SAPI runs shutdown functions at
//!   the same two points elephc models: normal script end and `exit()`/`die()`).
//! - The exit-path fixtures assert `!success` (non-zero exit status) through
//!   `compile_and_run_capture`, since the harness's `ProgramOutput` exposes success, not the raw
//!   exit code; the exact code (3, 7) was verified manually against PHP during implementation.

use crate::support::*;

/// Verifies shutdown callbacks run at normal script end, AFTER main output, in registration
/// order, and that registration-time bound arguments (`mixed ...$args`) are delivered to the
/// callback. Cross-checked with `php -n` (prints "main\nfirst\nsecond: 1 2\n").
#[test]
fn test_register_shutdown_function_runs_at_script_end_in_order_with_args() {
    let out = compile_and_run(
        r#"<?php
register_shutdown_function(function () {
    echo "first\n";
});
register_shutdown_function(function ($a, $b) {
    echo "second: $a $b\n";
}, 1, 2);
echo "main\n";
"#,
    );
    assert_eq!(out, "main\nfirst\nsecond: 1 2\n");
}

/// Verifies shutdown callbacks also run before `exit(status)` terminates the process, and the
/// non-zero exit status is preserved after the callbacks run. Cross-checked with `php -n`
/// (prints "before exit\nshutdown ran\n", exit code 3).
#[test]
fn test_register_shutdown_function_runs_before_exit_and_preserves_status() {
    let out = compile_and_run_capture(
        r#"<?php
register_shutdown_function(function () {
    echo "shutdown ran\n";
});
echo "before exit\n";
exit(3);
echo "never\n";
"#,
    );
    assert_eq!(out.stdout, "before exit\nshutdown ran\n");
    assert!(!out.success, "exit(3) must yield a non-zero exit status");
}

/// Verifies the re-entry guard: a shutdown callback that itself calls `exit()` re-enters the
/// exit path (which calls the runner again), but the second entry is a no-op — remaining
/// callbacks are skipped and the callback's exit status wins, matching PHP. Cross-checked with
/// `php -n` (prints "main\none\n", exit code 7 — "two" never runs).
#[test]
fn test_register_shutdown_function_exit_inside_callback_skips_rest() {
    let out = compile_and_run_capture(
        r#"<?php
register_shutdown_function(function () {
    echo "one\n";
    exit(7);
});
register_shutdown_function(function () {
    echo "two\n";
});
echo "main\n";
"#,
    );
    assert_eq!(out.stdout, "main\none\n");
    assert!(!out.success, "exit(7) from a shutdown callback must yield a non-zero exit status");
}

/// Verifies a shutdown callback that registers ANOTHER shutdown function mid-run appends to the
/// live registry and the new entry runs after the current queue, matching PHP's live-queue
/// semantics. Cross-checked with `php -n` (prints "main\none\ntwo\nthree: late\n").
#[test]
fn test_register_shutdown_function_mid_run_registration_appends() {
    let out = compile_and_run(
        r#"<?php
register_shutdown_function(function () {
    echo "one\n";
    register_shutdown_function(function ($x) { echo "three: $x\n"; }, "late");
});
register_shutdown_function(function () {
    echo "two\n";
});
echo "main\n";
"#,
    );
    assert_eq!(out, "main\none\ntwo\nthree: late\n");
}

/// Verifies a program that declares its OWN `register_shutdown_function` keeps its definition
/// (the prelude is not injected, no redeclaration error) and no automatic end-of-script callback
/// invocation happens — the user's function is just an ordinary function.
#[test]
fn test_register_shutdown_function_user_declaration_wins() {
    let out = compile_and_run(
        r#"<?php
function register_shutdown_function(callable $cb): string {
    return "user-owned";
}
echo register_shutdown_function(function () { echo "never\n"; });
"#,
    );
    assert_eq!(out, "user-owned");
}
