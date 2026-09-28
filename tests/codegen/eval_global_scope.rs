//! Purpose:
//! Checks global symbol-table identity across opaque eval and declared functions.
//!
//! Called from:
//! - The codegen integration test harness.
//!
//! Key details:
//! - Runtime-selected source keeps these cases on the interpreter bridge.

use crate::support::*;

/// Shares names created by main-scope eval with eval-declared function globals.
#[test]
fn test_eval_main_global_scope_dynamic_names() {
    let source = r#"<?php
$source = $argc > 0 ? '$number = 7; function increment_dynamic_global() { global $number; $number++; } increment_dynamic_global(); echo $number, "\n";' : '';
eval($source);
$next = $argc > 0 ? 'increment_dynamic_global(); echo $number, "\n";' : '';
eval($next);
"#;
    let output = compile_and_run_with_heap_debug(source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "8\n9\n");
}

/// Keeps function-local eval variables separate from a declared global binding.
#[test]
fn test_eval_function_global_scope_keeps_locals_separate() {
    let source = r#"<?php
$number = 7;
function read_number(): int { global $number; return $number; }
function invoke_local_eval(string $source): void {
    $number = 20;
    eval($source);
    echo $number, "\n";
}
$source = $argc > 0 ? 'function increment_local_global() { global $number; $number++; } increment_local_global(); echo $number, "\n";' : '';
invoke_local_eval($source);
echo read_number(), "\n";
"#;
    assert_eq!(compile_and_run(source), "20\n20\n8\n");
}

/// Gives eval code PHP's CLI superglobals when the compiled program names none of them (#935).
///
/// Eval found `$_ENV` and `$_SERVER` undefined unless the program spelled them. The fragment
/// runs from the top level and inside a function; each sees the environment in `$_ENV`, the
/// CLI keys in `$_SERVER`, and empty request arrays. Heap debugging proves the created arrays
/// are released with their eval scopes.
#[test]
fn test_eval_creates_cli_superglobals_the_program_never_names() {
    let source = r#"<?php
$code = $argc > 0 ? 'echo array_key_exists("PATH", $_ENV) ? "env" : "no-env", "|", $_SERVER["argc"], "|", $_SERVER["PHP_SELF"] === $_SERVER["argv"][0] ? "self" : "bad", "|", $_SERVER["REQUEST_TIME"] > 0 ? "time" : "no-time", "|", count($_GET) + count($_REQUEST), "\n";' : '';
function probe(string $code): void { eval($code); }
eval($code);
probe($code);
"#;
    let output = compile_and_run_with_heap_debug(source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "env|1|self|time|0\nenv|1|self|time|0\n");
    assert!(
        output.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "eval-created superglobals leaked: {}",
        output.stderr
    );
}

/// Lets a function-scope eval read and write the superglobals the compiled program seeded,
/// without a `global` statement (#935).
///
/// Superglobals resolved in the eval's local scope, so a function-scope eval saw neither the
/// program's `$_SERVER` nor its `$_ENV`. They now resolve through the synchronized global
/// scope: the native write is visible inside eval, and eval's write reaches native code.
#[test]
fn test_eval_function_scope_shares_program_superglobals() {
    let source = r#"<?php
$_SERVER["FROM_NATIVE"] = "n";
$code = $argc > 0 ? 'echo $_SERVER["FROM_NATIVE"], "|", array_key_exists("PATH", $_ENV) ? "env" : "no-env", "\n"; $_ENV["FROM_EVAL"] = "e";' : '';
function probe(string $code): void { eval($code); }
probe($code);
echo array_key_exists("FROM_EVAL", $_ENV) ? $_ENV["FROM_EVAL"] : "lost", "\n";
"#;
    let output = compile_and_run_with_heap_debug(source);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, "n|env\ne\n");
    assert!(
        output.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "shared superglobals leaked: {}",
        output.stderr
    );
}
