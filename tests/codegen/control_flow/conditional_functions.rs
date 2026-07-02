//! Purpose:
//! Regression tests for hoisting conditionally-declared functions to the top level so they are
//! registered like ordinary functions. Covers `if (!function_exists('X')) { function X ... }`
//! polyfill guards (the mbstring/intl-normalizer/intl-grapheme pattern), the guard's skip-when-
//! already-provided behavior, plain conditional and nested declarations, and cross-file discovery.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each fixture's expected output is cross-checked against PHP. The guard-skip cases (a builtin or
//!   a first-declared function) must keep the pre-existing definition and never redeclare, matching
//!   PHP's runtime `function_exists` behavior.

use super::*;

/// A function guarded by `if (!function_exists('X'))` where `X` is not otherwise defined is hoisted
/// and registered, so a later call resolves — the core mbstring/intl polyfill pattern.
#[test]
fn test_function_exists_guarded_declaration_is_registered() {
    let out =
        compile_and_run("<?php if (!function_exists('myfoo')) { function myfoo() { return 42; } } echo myfoo();");
    assert_eq!(out, "42");
}

/// A `!function_exists('X')` guard around a redefinition of an existing builtin is statically false,
/// so the nested definition is skipped and the real builtin wins (no "cannot redeclare" error).
/// PHP behaves identically: the builtin exists, so the guarded body never runs.
#[test]
fn test_function_exists_guard_skips_builtin_redefinition() {
    let out = compile_and_run(
        "<?php if (!function_exists('strlen')) { function strlen($s) { return 999; } } echo strlen('abc');",
    );
    assert_eq!(out, "3");
}

/// A function declared inside a plain `if (true) { ... }` (no `function_exists` guard) is discovered
/// and registered, so a subsequent call resolves.
#[test]
fn test_plain_true_branch_declaration_is_registered() {
    let out = compile_and_run("<?php if (true) { function h() { return 7; } } echo h();");
    assert_eq!(out, "7");
}

/// A guarded declaration nested two container levels deep (`if (true) { if (!function_exists(...))
/// { function ... } }`) is still hoisted to the top level and registered.
#[test]
fn test_nested_two_level_guarded_declaration_is_registered() {
    let out = compile_and_run(
        "<?php if (true) { if (!function_exists('deep')) { function deep() { return 5; } } } echo deep();",
    );
    assert_eq!(out, "5");
}

/// A `!function_exists('X') && ...` guard chain still recognizes the guarded name, so the function
/// is hoisted and registered.
#[test]
fn test_function_exists_guard_in_and_chain_is_registered() {
    let out = compile_and_run(
        "<?php if (!function_exists('foo9') && true) { function foo9() { return 9; } } echo foo9();",
    );
    assert_eq!(out, "9");
}

/// Two guarded declarations of the same name: the first is hoisted, and the second — now guarded by
/// a name that is already known — is skipped, so the first definition wins (matching PHP, where the
/// second `function_exists` guard is false). No duplicate-declaration error is produced.
#[test]
fn test_duplicate_guarded_declaration_keeps_first() {
    let out = compile_and_run(
        "<?php if (!function_exists('dup')) { function dup() { return 1; } } if (!function_exists('dup')) { function dup() { return 2; } } echo dup();",
    );
    assert_eq!(out, "1");
}

/// A guarded function declared inside an included file is discovered across the include boundary and
/// registered, so the including file can call it — the polyfill `bootstrap` require pattern.
#[test]
fn test_guarded_declaration_in_included_file_is_registered() {
    let out = compile_and_run_files(
        &[
            (
                "main.php",
                "<?php\nrequire __DIR__ . '/inc.php';\necho g();\n",
            ),
            (
                "inc.php",
                "<?php\nif (!function_exists('g')) { function g() { return 7; } }\n",
            ),
        ],
        "main.php",
    );
    assert_eq!(out, "7");
}

/// A function guarded inside a statically-dead `if (false) { ... }` branch is not hoisted, so it is
/// not registered and `function_exists` reports it absent — matching PHP, where the branch never
/// runs and never declares the function.
#[test]
fn test_dead_false_branch_declaration_is_not_registered() {
    let out = compile_and_run(
        "<?php if (false) { function never_defined() { return 1; } } echo function_exists('never_defined') ? 'yes' : 'no';",
    );
    assert_eq!(out, "no");
}
