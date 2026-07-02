//! Purpose:
//! End-to-end tests for the AST optimizer's closed-world `function_exists` fold, covering both the
//! string-literal argument form (`function_exists('X')`) and the `Name::class` argument form
//! (`function_exists(X::class)`), the latter sharing the `::class`-to-FQN resolver with
//! `class_existence` so a polyfill guard written with `::class` folds the same way as one written
//! with a string literal.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Expected outputs are cross-checked against `php -r`. The `::class` cases mirror the
//!   `class_exists` fold fixtures: a guarded body whose guard folds to a constant boolean is
//!   pruned by DCE, and a `::class` argument that defers (a checked user function) is rewritten to
//!   its FQN string literal so codegen's `lower_function_exists` receives a lowerable string.

use super::*;

/// Verifies `function_exists('X')` on a name elephc provides as a builtin folds to `true`, so the
/// guarded redefinition body is DCE'd and the real builtin wins. PHP prints "3".
#[test]
fn test_function_exists_string_literal_builtin_folds_true() {
    let out = compile_and_run(
        "<?php if (!function_exists('strlen')) { function strlen($s) { return 999; } } echo strlen('abc');",
    );
    assert_eq!(out, "3");
}

/// Verifies `function_exists(\Builtin::class)` on a builtin folds to `true`, so the guarded
/// redefinition body is DCE'd and the real builtin wins. `\strlen::class` resolves to `"strlen"`,
/// which is a PHP-visible builtin. Cross-checked with
/// `php -r 'if(!function_exists(\strlen::class)){function strlen($s){return 999;}} echo strlen("abc");'`
/// (prints "3").
#[test]
fn test_function_exists_class_const_builtin_folds_true() {
    let out = compile_and_run(
        r#"<?php
if (!\function_exists(\strlen::class)) {
    function strlen($s) { return 999; }
}
echo strlen("abc");
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies `function_exists(Name::class)` on a name NOT in the closed world folds to `false`, so
/// the `!function_exists(s::class)` guard folds to `true` and the guarded `function s` declaration
/// stays live, is hoisted/registered, and the later call resolves. Cross-checked with
/// `php -r 'namespace N; if(!function_exists(s::class)){function s(){return "hi";}} echo s();'`
/// (prints "hi").
#[test]
fn test_function_exists_class_const_unknown_folds_false_body_stays() {
    let out = compile_and_run(
        r#"<?php
namespace Symfony\Component\String;
if (!\function_exists(s::class)) {
    function s(?string $string = 'hi'): string { return $string; }
}
echo s();
"#,
    );
    assert_eq!(out, "hi");
}

/// Verifies a bare (unqualified) `function_exists(s::class)` inside a namespace resolves `s` to the
/// namespace FQN for the fold, matching PHP's name resolution for `::class` on an unqualified name.
/// Here `s` is a checked user function, so the call is rewritten to the FQN string and codegen
/// lowers it to a static `true`. Cross-checked with
/// `php -r 'namespace N; function s(){return "x";} echo function_exists(s::class)?1:0;'` (prints "1").
#[test]
fn test_function_exists_class_const_bare_in_namespace_resolves_fqn() {
    let out = compile_and_run(
        r#"<?php
namespace N;
function s(): string { return "x"; }
echo function_exists(s::class) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies `function_exists(Absent::class)` on an absent name folds to `false`, so the guarded
/// `echo` body is DCE'd and the `else` branch runs. The guarded body must not reference any
/// undefined symbol, since type checking runs before the optimizer fold. PHP prints "absent".
#[test]
fn test_function_exists_class_const_absent_folds_false_else_branch() {
    let out = compile_and_run(
        r#"<?php
namespace N;
if (\function_exists(NoSuchFn::class)) {
    echo "present";
} else {
    echo "absent";
}
"#,
    );
    assert_eq!(out, "absent");
}

/// Verifies the `::class`-to-string rewrite composes with the resolver's conditional-function
/// hoisting: the spec's Symfony `symfony/string` polyfill shape (`if (!function_exists(s::class))
/// { function s(...) }`) compiles and the hoisted `s` is callable, with no "non-literal function
/// name" codegen error. Cross-checked with `php -r` on the same source (prints "hi").
#[test]
fn test_function_exists_class_const_polyfill_guard_compiles_and_runs() {
    let out = compile_and_run(
        r#"<?php
namespace Symfony\Component\String;
if (!\function_exists(s::class)) {
    function s(?string $string = 'hi'): string { return $string ?? ''; }
}
echo s();
"#,
    );
    assert_eq!(out, "hi");
}