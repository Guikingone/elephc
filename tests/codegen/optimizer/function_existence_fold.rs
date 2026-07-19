//! Purpose:
//! End-to-end tests for the AST optimizer's closed-world `function_exists` fold, covering both the
//! string-literal argument form (`function_exists('X')`) and the `Name::class` argument form
//! (`function_exists(X::class)`), the latter sharing the `::class`-to-FQN resolver with
//! `class_existence` so a polyfill guard written with `::class` folds the same way as one written
//! with a string literal. Also covers the PRE-checker curated-extension-allowlist fold
//! (`FunctionExistenceSet::for_pre_check`, wired into `crate::pipeline::compile` right after
//! `fold_constants`), which false-folds `function_exists`/`extension_loaded` for a small set of
//! PHP extension names elephc never provides so a Composer runtime's guard around an unresolvable
//! extension function (`fastcgi_finish_request`, `igbinary_serialize`, ...) never reaches the
//! checker.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Expected outputs are cross-checked against `php -r`. The `::class` cases mirror the
//!   `class_exists` fold fixtures: a guarded body whose guard folds to a constant boolean is
//!   pruned by DCE, and a `::class` argument that defers (a checked user function) is rewritten to
//!   its FQN string literal so codegen's `lower_function_exists` receives a lowerable string.
//! - The pre-check cases exercise the aggressive short-circuit/ternary drop
//!   (`try_fold_precheck_short_circuit`/`try_fold_precheck_ternary` in
//!   `src/optimize/function_existence.rs`): the guard combines the curated-false fold with a
//!   runtime-only operand (`&&`, `?:`) that the general fold engine could never prune away, since
//!   that operand is not itself a compile-time literal.

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

/// Verifies the pre-checker curated-extension fold clears `symfony/runtime`'s
/// `HttpKernelRunner`/`symfony/http-foundation` `Response::send()` shape: an `if`/`elseif` chain
/// guarding two never-provided SAPI functions, each `&&`-combined with a runtime-only operand the
/// general fold engine cannot reduce to a literal on its own. Without the pre-checker fold this
/// would previously fail to compile with "Undefined function: fastcgi_finish_request" — the
/// checker sees the call regardless of the (dynamic-looking) surrounding condition. Cross-checked
/// with `php -n -r` (prints "done": neither extension function exists under the CLI SAPI either).
#[test]
fn test_precheck_fold_prunes_fastcgi_litespeed_guard_with_runtime_operand() {
    let out = compile_and_run(
        r#"<?php
class Runner {
    private bool $debug = false;
    function finish(): void {
        if (\function_exists('fastcgi_finish_request') && !$this->debug) {
            fastcgi_finish_request();
        } elseif (\function_exists('litespeed_finish_request') && !$this->debug) {
            litespeed_finish_request();
        }
        echo "done";
    }
}
(new Runner())->finish();
"#,
    );
    assert_eq!(out, "done");
}

/// Verifies the pre-checker curated-extension fold also covers `extension_loaded()`, and drops the
/// untaken ternary branch even though that branch (a call to an unresolvable extension function) is
/// not itself a pure/literal expression — the aggressive pre-check-only ternary drop
/// (`try_fold_precheck_ternary`) is required here; the general ternary fold requires both branches
/// to already be scalar literals. Mirrors `symfony/cache`'s `DefaultMarshaller::marshall()` guard.
/// Cross-checked with `php -n -r` (prints "not-loaded": igbinary is absent under the CLI SAPI too).
#[test]
fn test_precheck_fold_prunes_extension_loaded_ternary_impure_branch() {
    let out = compile_and_run(
        r#"<?php
$value = \extension_loaded('igbinary') ? igbinary_serialize([1, 2]) : 'not-loaded';
echo $value;
"#,
    );
    assert_eq!(out, "not-loaded");
}

/// Regression test for a bug the pre-checker prune pass surfaced (fixed in
/// `src/optimize/control/prune/statements.rs`): `prune_stmt` used to reconstruct every statement
/// with a hardcoded `attributes: Vec::new()`, silently dropping `#[Attribute(...)]` metadata. That
/// was invisible as long as `prune_constant_control_flow` only ran AFTER the checker had already
/// snapshotted attributes elsewhere — but this pre-checker fold now runs `prune_constant_control_flow`
/// BEFORE the checker, so a program with an unrelated curated-extension guard anywhere would have
/// silently erased reflection attributes on every statement the pass touched. Verifies a
/// `#[MyAttr]`-attributed class declaration survives intact (`ReflectionClass::getAttributes()`
/// still finds it) in a program that also contains a pruned `fastcgi_finish_request` guard.
#[test]
fn test_precheck_fold_prune_preserves_statement_attributes() {
    let out = compile_and_run(
        r#"<?php
#[Attribute]
class MyAttr {}

if (\function_exists('fastcgi_finish_request') && true) {
    fastcgi_finish_request();
}

#[MyAttr]
class Target {}

$r = new ReflectionClass('Target');
$attrs = $r->getAttributes();
echo count($attrs), ":", $attrs[0]->getName();
"#,
    );
    assert_eq!(out, "1:MyAttr");
}

/// Verifies the pre-checker fold NEVER shadows a real (even same-named) user-declared function:
/// when the program itself defines `fastcgi_finish_request()` (a real, callable polyfill rather
/// than a dead guard body), `function_exists('fastcgi_finish_request')` must stay unfolded pre-
/// checker so the guard is decided normally and the user's own function runs. Cross-checked with
/// `php -n -r` on the same source (prints "polyfill": PHP has no such extension function either, so
/// the guarded user declaration is never redeclared and `function_exists` sees only the user's own
/// definition, which is present unconditionally here).
#[test]
fn test_precheck_fold_never_shadows_user_declared_curated_name() {
    let out = compile_and_run(
        r#"<?php
function fastcgi_finish_request(): string {
    return "polyfill";
}
if (\function_exists('fastcgi_finish_request')) {
    echo fastcgi_finish_request();
} else {
    echo "absent";
}
"#,
    );
    assert_eq!(out, "polyfill");
}