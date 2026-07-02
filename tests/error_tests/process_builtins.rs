//! Purpose:
//! Recognition-layer tests for the process/system-control PHP builtins symfony/console needs:
//! `pcntl_signal`, `pcntl_alarm`, `pcntl_async_signals`, `pcntl_signal_get_handler`,
//! `proc_close`, `getmypid`, `cli_set_process_title`, `setproctitle`, `sapi_windows_cp_get`,
//! `sapi_windows_cp_set`, `sapi_windows_vt100_support`, `ini_get`, and `get_defined_constants`.
//! These are registered for type checking and first-class-callable resolution only; they are
//! runtime-dead for the console app and have no EIR/codegen lowering yet, so these tests assert
//! type-check recognition (never `compile_and_run`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Verifies arity diagnostics, PHP-accurate return types (the `int|false` / `string|false` /
//!   `int|string` unions and the `array` map), and first-class-callable syntax. A regressing
//!   catalog/signature entry surfaces here as a spurious "Undefined function" or a wrong
//!   arity/return-type error.

use super::*;

/// Verifies that every registered process/system builtin is recognized (no "Undefined function")
/// in a representative valid call form. `$h` stands in for an opaque proc/stream resource.
#[test]
fn test_process_builtins_recognized() {
    assert!(
        check_source(
            r#"<?php
$h = fopen("php://stdout", "w");
$a = pcntl_signal(2, "handler");
$b = pcntl_alarm(1);
$c = pcntl_async_signals(true);
$d = pcntl_signal_get_handler(2);
$e = proc_close($h);
$f = getmypid();
$g = cli_set_process_title("worker");
$i = setproctitle("worker");
$j = sapi_windows_cp_get();
$k = sapi_windows_cp_set(65001);
$l = sapi_windows_vt100_support($h, true);
$m = ini_get("memory_limit");
$n = get_defined_constants();
echo $b . $e;
"#
        )
        .is_ok(),
        "all registered process/system builtins should type-check",
    );
}

/// Verifies the union-return types: `getmypid`/`ini_get` fold into a strict `=== false` check
/// (`int|false` and `string|false`), while `pcntl_signal_get_handler` yields `int|string`.
#[test]
fn test_process_union_returns_recognized() {
    assert!(
        check_source(
            r#"<?php
$pid = getmypid();
if ($pid === false) { echo "no"; } else { echo $pid; }
$limit = ini_get("memory_limit");
if ($limit === false) { echo "no"; } else { echo $limit; }
$handler = pcntl_signal_get_handler(2);
echo $handler;
"#
        )
        .is_ok(),
        "process union return types (int|false, string|false, int|string) should type-check",
    );
}

/// Verifies that `get_defined_constants(true)` categorizes without complaint and its array
/// result can be indexed, exercising the optional `$categorize` argument and the `array` return.
#[test]
fn test_get_defined_constants_optional_arg_recognized() {
    assert!(
        check_source(
            "<?php $c = get_defined_constants(true); $x = $c[\"E_ALL\"]; echo $x;"
        )
        .is_ok(),
        "get_defined_constants(bool) should type-check and return an indexable array",
    );
}

/// Verifies that process/system builtins work through first-class-callable syntax, since
/// Symfony references such functions as callables. `ini_get`/`getmypid` cover the string- and
/// no-argument shapes respectively.
#[test]
fn test_process_first_class_callable_recognized() {
    assert!(
        check_source(
            "<?php $f = ini_get(...); $g = getmypid(...); echo is_callable($f) && is_callable($g);"
        )
        .is_ok(),
        "process/system builtins should be usable as first-class callables",
    );
}

expect_builtin_arity_error!(
    test_error_pcntl_alarm_wrong_args,
    "<?php pcntl_alarm();",
    "pcntl_alarm() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_getmypid_takes_no_args,
    "<?php getmypid(1);",
    "getmypid() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_ini_get_wrong_args,
    "<?php ini_get();",
    "ini_get() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_pcntl_signal_too_few_args,
    "<?php pcntl_signal(2);",
    "pcntl_signal() takes 2 or 3 arguments"
);
