//! Purpose:
//! End-to-end codegen coverage for late-bound curated extension function calls (`apcu_exists`,
//! `opcache_invalidate`, `igbinary_serialize`, ...): a call site naming one of these compiles to
//! a catchable `\Error` throw with PHP's exact "Call to undefined function X()" message, instead
//! of a compile-time error, matching PHP's real late-bound function resolution.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - See `crate::types::checker::builtins::late_bound` (checker carve-out and curated allowlist)
//!   and `crate::ir_lower::expr::late_bound_call` (the throw lowering) for the implementation.
//! - `php -n` verified: PHP resolves a function call at `INIT_FCALL`, BEFORE any argument
//!   `SEND_*` opcode runs, so an undefined-function call's arguments are never evaluated — tests
//!   here assert a side-effecting argument's output is absent, not present.

use crate::support::*;

/// A guarded call to a curated late-bound name behind a runtime condition the optimizer cannot
/// constant-fold (a cached `extension_loaded()`-derived boolean, mirroring the Symfony
/// `isSupported()` pattern the design targets) never executes at runtime, so the program runs to
/// completion normally without ever reaching the throw.
#[test]
fn test_late_bound_guarded_call_never_executes() {
    let out = compile_and_run(
        "<?php
        class Cache {
            private static ?bool $apcuSupported = null;
            public static function isSupported(): bool {
                self::$apcuSupported ??= extension_loaded('apcu');
                return self::$apcuSupported;
            }
        }
        if (Cache::isSupported()) {
            apcu_exists('key');
            echo 'used-apcu';
        } else {
            echo 'fallback';
        }",
    );
    assert_eq!(out, "fallback");
}

/// A direct, unguarded call to a curated late-bound name throws a catchable `\Error` with PHP's
/// exact message, and execution continues normally after the `catch`.
#[test]
fn test_late_bound_direct_call_throws_catchable_error() {
    let out = compile_and_run(
        "<?php
        try {
            apcu_exists('key');
            echo 'unreachable';
        } catch (\\Error $e) {
            echo get_class($e), '|', $e->getMessage();
        }",
    );
    assert_eq!(out, "Error|Call to undefined function apcu_exists()");
}

/// The thrown value is exactly `\Error`, never `\Exception` — matches PHP: undefined-function
/// resolution failures are always `\Error`.
#[test]
fn test_late_bound_thrown_value_is_not_exception() {
    let out = compile_and_run(
        "<?php
        try {
            opcache_invalidate('f');
        } catch (\\Error $e) {
            echo ($e instanceof \\Exception) ? 'exception' : 'not-exception';
        }",
    );
    assert_eq!(out, "not-exception");
}

/// A call site written unqualified inside a namespace embeds the namespaced attempt name in the
/// thrown message, matching PHP's own "namespaced lookup failed, global fallback failed too"
/// message shape (`php -n` verified with `Foo\Bar\apcu_fetch()`-style probes).
#[test]
fn test_late_bound_namespaced_call_message() {
    let out = compile_and_run(
        "<?php
        namespace Foo\\Bar;
        function run() {
            try {
                igbinary_serialize([1]);
            } catch (\\Error $e) {
                echo $e->getMessage();
            }
        }
        run();",
    );
    assert_eq!(out, "Call to undefined function Foo\\Bar\\igbinary_serialize()");
}

/// PHP resolves an undefined-function call before evaluating its arguments (verified with
/// `php -n`: resolution fails at `INIT_FCALL`, before any `SEND_*` opcode runs, for a direct
/// call, a dynamic `$fn()` call, a spread, and a named argument alike) — a side-effecting
/// argument expression must never run.
#[test]
fn test_late_bound_call_does_not_evaluate_arguments() {
    let out = compile_and_run(
        "<?php
        function sideeffect($x) {
            echo 'evaluated:', $x;
            return $x;
        }
        try {
            apcu_store(sideeffect('a'), sideeffect('b'));
        } catch (\\Error $e) {
            echo 'caught';
        }",
    );
    assert_eq!(out, "caught");
}

/// `function_exists()` still reports `false` for a curated late-bound name — the checker
/// carve-out changes call-site compilation only, never the closed-world existence answer (the J3
/// pre-check fold already false-folds it).
#[test]
fn test_late_bound_function_exists_still_false() {
    let out = compile_and_run("<?php var_dump(function_exists('apcu_exists'));");
    assert_eq!(out, "bool(false)\n");
}

/// `is_callable()` likewise still reports `false` for a curated late-bound name.
#[test]
fn test_late_bound_is_callable_still_false() {
    let out = compile_and_run("<?php var_dump(is_callable('apcu_exists'));");
    assert_eq!(out, "bool(false)\n");
}

/// Every curated late-bound name throws the SAME catchable shape with ITS OWN exact name in the
/// message — a broader sweep across the full curated allowlist, not just one representative name.
#[test]
fn test_late_bound_call_covers_full_curated_allowlist() {
    let names = [
        "opcache_invalidate",
        "opcache_compile_file",
        "apcu_exists",
        "apcu_store",
        "apcu_delete",
        "xdebug_is_enabled",
        "xdebug_connect_to_client",
        "igbinary_serialize",
        "igbinary_unserialize",
        "frankenphp_handle_request",
    ];
    for name in names {
        let source = format!(
            "<?php
            try {{
                {name}();
            }} catch (\\Error $e) {{
                echo $e->getMessage();
            }}"
        );
        let out = compile_and_run(&source);
        assert_eq!(
            out,
            format!("Call to undefined function {name}()"),
            "curated name {name} did not throw the expected message"
        );
    }
}

/// Verifies the exact Symfony `FrankenPhpWorkerRunner::run()` shape: `xdebug_connect_to_client`
/// called doubly-guarded behind `extension_loaded('xdebug') && function_exists('xdebug_connect_to_client')`
/// inside a closure body (where the guard cannot be constant-folded away) compiles and runs to
/// completion without ever reaching the late-bound throw.
#[test]
fn test_late_bound_xdebug_connect_guarded_in_closure_never_executes() {
    let out = compile_and_run(
        "<?php
        $handler = function (): void {
            if (\\extension_loaded('xdebug') && \\function_exists('xdebug_connect_to_client')) {
                xdebug_connect_to_client();
            }
            echo 'ran';
        };
        $handler();",
    );
    assert_eq!(out, "ran");
}
