//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP, callables variadics, including first class callable variadic function call, closure variadic call, and first class callable variadic with regular param.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests variadic function called via first-class callable syntax `func(...)]`.
/// Verifies that positional arguments are collected into the variadic parameter
/// and count() returns the correct number.
#[test]
fn test_first_class_callable_variadic_function_call() {
    let out = compile_and_run(
        r#"<?php
function count_args(...$xs) {
    echo count($xs);
}

$f = count_args(...);
$f(1, 2, 3);
"#,
    );
    assert_eq!(out, "3");
}

/// Tests variadic closure called directly as a callable expression.
/// Verifies that positional arguments are collected into the variadic closure
/// parameter and count() returns the correct number.
#[test]
fn test_closure_variadic_call() {
    let out = compile_and_run(
        r#"<?php
$f = function (...$xs) {
    echo count($xs);
};

$f(1, 2, 3);
"#,
    );
    assert_eq!(out, "3");
}

/// Tests variadic function with a regular parameter before the variadic,
/// called via first-class callable syntax.
/// Verifies the regular parameter receives the first positional argument,
/// remaining arguments fill the variadic, and both are handled correctly.
#[test]
fn test_first_class_callable_variadic_with_regular_param() {
    let out = compile_and_run(
        r#"<?php
function head_and_count($a, ...$rest) {
    echo $a;
    echo ":";
    echo count($rest);
}

$f = head_and_count(...);
$f(7, 8, 9);
"#,
    );
    assert_eq!(out, "7:2");
}

/// Tests first-class callable syntax on builtin count() with a sequentially-keyed array.
/// Verifies builtin callables work with variadic-compatible signatures.
#[test]
fn test_first_class_callable_builtin_count_accepts_string_arrays() {
    let out = compile_and_run(
        r#"<?php
$f = count(...);
$xs = ["a", "b"];
echo $f($xs);
"#,
    );
    assert_eq!(out, "2");
}

/// Tests first-class callable syntax on builtin count() with an associative array.
/// Verifies builtin callables work with associative array inputs.
#[test]
fn test_first_class_callable_builtin_count_accepts_assoc_arrays() {
    let out = compile_and_run(
        r#"<?php
$f = count(...);
$xs = ["a" => 1, "b" => 2];
echo $f($xs);
"#,
    );
    assert_eq!(out, "2");
}

// -- Regression: variadic-method arity (`callable_wrapper_sig` defaults, N1 item 1) --
//
// `build_method_sig` unconditionally routes every class method's `FunctionSig` through
// `callable_wrapper_sig` to synthesize the trailing variadic slot's `params`/`defaults`
// entries in lockstep. `callable_wrapper_sig` used to push `None` for that synthesized
// slot's own default, and `required = sig.defaults.iter().filter(|d| d.is_none()).count()`
// (`call_validation::check_known_callable_call_with_options`) counts a `None` default as
// "this parameter is required" — so a bare variadic METHOD call falsely demanded at least
// one argument ("expects at least 1 arguments, got 0") even though PHP always allows a
// variadic method to be called with zero trailing arguments. Free functions never routed
// through `callable_wrapper_sig` at declaration time, so they were unaffected.
// php -n verified: `class M { function v(mixed ...$a): int { return count($a); } } (new
// M)->v();` → `0`; `(new M)->v(1, 2, 3);` → `3`.

/// Verifies a zero-arg call to an untyped variadic instance method matches PHP's `count($a)
/// === 0` instead of falsely erroring "expects at least 1 arguments, got 0".
#[test]
fn test_variadic_method_call_zero_args() {
    let out = compile_and_run(
        r#"<?php
class M {
    function v(mixed ...$a): int { return count($a); }
}
$m = new M();
echo $m->v();
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies an N-arg call to an untyped variadic instance method still collects every
/// argument (sibling to the zero-arg case above — guards against an off-by-one that only
/// shows up once at least one argument is passed).
#[test]
fn test_variadic_method_call_multiple_args() {
    let out = compile_and_run(
        r#"<?php
class M {
    function v(mixed ...$a): int { return count($a); }
}
$m = new M();
echo $m->v(1, 2, 3);
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies a zero-arg call to a STATIC variadic method also accepts zero arguments — the
/// `callable_wrapper_sig` fix applies uniformly across static and instance methods since
/// both route through the same `build_method_sig`.
#[test]
fn test_static_variadic_method_call_zero_args() {
    let out = compile_and_run(
        r#"<?php
class M {
    static function sv(int ...$a): int { return count($a); }
}
echo M::sv();
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies a TYPED variadic method (`int ...$a`, not `mixed ...$a`) also accepts zero
/// arguments and sums correctly with multiple arguments — the declared element type is
/// applied on top of `callable_wrapper_sig`'s output (see `build_method_sig`'s
/// `variadic_type` refinement), so this exercises both fixes together.
#[test]
fn test_typed_variadic_method_call_zero_and_multiple_args() {
    let out = compile_and_run(
        r#"<?php
class M {
    function tv(int ...$a): int { return array_sum($a); }
}
$m = new M();
echo $m->tv();
echo ":";
echo $m->tv(1, 2, 3);
"#,
    );
    assert_eq!(out, "0:6");
}

/// Verifies a variadic method with a REQUIRED leading parameter still fills the required
/// parameter and collects only the trailing arguments into the variadic tail. The
/// zero-trailing-args case (`$m->v("a")`) is the important regression guard here: it
/// exercises the SAME `callable_wrapper_sig`-synthesized variadic slot as the no-arg tests
/// above, but with a required parameter present, confirming the fix does not disturb
/// required-parameter arity enforcement (see the companion negative test in
/// `error_tests/callables.rs` for the omitted-required-parameter case).
#[test]
fn test_variadic_method_with_required_param_and_zero_trailing_args() {
    let out = compile_and_run(
        r#"<?php
class M {
    function v($first, ...$rest) {
        echo $first;
        echo ":";
        echo count($rest);
    }
}
$m = new M();
$m->v("a");
echo "|";
$m->v("a", "b", "c");
"#,
    );
    assert_eq!(out, "a:0|a:2");
}

/// Verifies static-method callable arrays route associative variadic tails through the descriptor invoker.
#[test]
fn test_static_method_callable_array_call_user_func_array_assoc_variadic_tail() {
    let out = compile_and_run(
        r#"<?php
class Formatter {
    public static function wrap($value = 7, ...$rest) {
        echo $value . ":";
        foreach ($rest as $key => $item) {
            echo $key . "=" . $item . ";";
        }
    }
}

$callback = [Formatter::class, "wrap"];
$args = ["value" => 3, "extra" => 9, "more" => 10];
call_user_func_array($callback, $args);
"#,
    );
    assert_eq!(out, "3:extra=9;more=10;");
}
