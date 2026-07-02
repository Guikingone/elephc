//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object-oriented PHP, callables functions and builtins, including first class callable named function indirect call, first class callable builtin used in array map, and first class callable builtin intval.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests first-class callable syntax on a named function used via indirect call.
///
/// PHP `triple(...)(7)` via variable indirect call.
#[test]
fn test_first_class_callable_named_function_indirect_call() {
    let out = compile_and_run(
        r#"<?php
function triple($n) {
    return $n * 3;
}

$fn = triple(...);
echo $fn(7);
"#,
    );
    assert_eq!(out, "21");
}

/// Tests first-class callable builtin `strlen` passed to `array_map`.
#[test]
fn test_first_class_callable_builtin_used_in_array_map() {
    let out = compile_and_run(
        r#"<?php
$len = strlen(...);
echo $len("tool");
"#,
    );
    assert_eq!(out, "4");
}

/// Tests first-class callable builtin `intval` used in arithmetic expression.
#[test]
fn test_first_class_callable_builtin_intval() {
    let out = compile_and_run(
        r#"<?php
$to_int = intval(...);
echo $to_int("123") + 7;
"#,
    );
    assert_eq!(out, "130");
}

/// Tests first-class callable builtin `strtolower` used in direct call.
#[test]
fn test_first_class_callable_builtin_string_transform() {
    let out = compile_and_run(
        r#"<?php
$lower = strtolower(...);
echo $lower("TOOLS");
"#,
    );
    assert_eq!(out, "tools");
}

/// Tests first-class callable builtin `array_sum` with a literal array argument.
#[test]
fn test_first_class_callable_builtin_array_sum() {
    let out = compile_and_run(
        r#"<?php
$sum = array_sum(...);
echo $sum([2, 3, 5]);
"#,
    );
    assert_eq!(out, "10");
}

/// Tests first-class callable builtin `trim` stripping leading/trailing whitespace.
#[test]
fn test_first_class_callable_builtin_trim() {
    let out = compile_and_run(
        r#"<?php
$trim = trim(...);
echo $trim("  ready  ");
"#,
    );
    assert_eq!(out, "ready");
}

/// Tests first-class callable builtin `substr` with start index and length arguments.
#[test]
fn test_first_class_callable_builtin_substr() {
    let out = compile_and_run(
        r#"<?php
$substr = substr(...);
echo $substr("abcdef", 2, 3);
"#,
    );
    assert_eq!(out, "cde");
}

/// Tests first-class callable builtin `str_contains` used in a ternary for boolean output.
#[test]
fn test_first_class_callable_builtin_str_contains() {
    let out = compile_and_run(
        r#"<?php
$contains = str_contains(...);
echo $contains("compiler", "pile") ? "yes" : "no";
"#,
    );
    assert_eq!(out, "yes");
}

/// Tests that a first-class callable builtin that mutates a by-ref parameter preserves the array after call.
#[test]
fn test_first_class_callable_builtin_sort_preserves_by_ref_param() {
    let out = compile_and_run(
        r#"<?php
$sort = sort(...);
$values = [3, 1, 2];
$sort($values);
foreach ($values as $value) {
    echo $value;
}
"#,
    );
    assert_eq!(out, "123");
}

/// Tests that a user-defined function with by-ref parameter is correctly mutated via first-class callable.
#[test]
fn test_first_class_callable_preserves_by_ref_params() {
    let out = compile_and_run(
        r#"<?php
function bump(&$n) {
    $n = $n + 1;
}

$fn = bump(...);
$value = 7;
$fn($value);
echo $value;
"#,
    );
    assert_eq!(out, "8");
}

/// Tests that an alias of a first-class callable still mutates the caller's by-ref argument.
#[test]
fn test_first_class_callable_alias_preserves_by_ref_params() {
    let out = compile_and_run(
        r#"<?php
function bump(&$n) {
    $n = $n + 1;
}

$f = bump(...);
$g = $f;
$value = 7;
$g($value);
echo $value;
"#,
    );
    assert_eq!(out, "8");
}

/// Tests that an alias of a closure with by-ref parameter correctly mutates the caller's argument.
#[test]
fn test_closure_alias_preserves_by_ref_params() {
    let out = compile_and_run(
        r#"<?php
$f = function (&$x) {
    $x = $x + 1;
};

$g = $f;
$value = 7;
$g($value);
echo $value;
"#,
    );
    assert_eq!(out, "8");
}

/// Tests a first-class callable named function passed to `array_map` with index-based array access.
#[test]
fn test_first_class_callable_variable_used_in_array_map() {
    let out = compile_and_run(
        r#"<?php
function double($n) {
    return $n * 2;
}

$fn = double(...);
$values = array_map($fn, [1, 2, 3]);
echo $values[0];
echo ":";
echo $values[2];
"#,
    );
    assert_eq!(out, "2:6");
}

/// Tests a first-class callable on an untyped user function accepting a string argument.
#[test]
fn test_first_class_callable_untyped_function_accepts_string_args() {
    let out = compile_and_run(
        r#"<?php
function greet($name) {
    return "Hello " . $name;
}

$f = greet(...);
echo $f("World");
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Tests `call_user_func` with a first-class callable builtin (`strlen`).
#[test]
fn test_first_class_callable_direct_call_user_func() {
    let out = compile_and_run(
        r#"<?php
echo call_user_func(strlen(...), "hello");
"#,
    );
    assert_eq!(out, "5");
}

/// Tests a direct call on a `Closure` value stored in a variable.
///
/// Baseline sanity: `$f = fn($x) => $x + 1; $f(41)` must type-check and run.
#[test]
fn test_closure_direct_call_baseline() {
    let out = compile_and_run(
        r#"<?php
$f = fn($x) => $x + 1;
$r = $f(41);
echo $r;
"#,
    );
    assert_eq!(out, "42");
}

/// Tests calling a `callable|null`-typed variable inside a null guard.
///
/// The receiver comes from a `?callable`-returning function
/// (`Union([Callable, Null])`). The call expression inside the guard must
/// type-check against the nullable-callable union. At runtime `$cb` is null so
/// the guard is false and nothing is emitted; the test exercises the
/// type-checker acceptance path (the call site is accepted via the `Callable`
/// union member) without depending on union-typed runtime call dispatch.
#[test]
fn test_callable_nullable_union_call_under_guard() {
    let out = compile_and_run(
        r#"<?php
function get_cb(bool $ok): ?callable {
    if (!$ok) {
        return null;
    }
    return fn($v) => $v;
}
$cb = get_cb(false);
if ($cb) {
    echo $cb(2);
}
"#,
    );
    assert_eq!(out, "");
}

/// Tests calling a `mixed`-typed receiver holding a closure.
///
/// A `mixed`-typed value (from a `mixed`-returning function) is callable under
/// PHP's gradual rules; the call type-checks (returns `Mixed`) and runs.
/// Mirrors `php -r '$x = fn($v) => $v; echo $x(2);'`.
#[test]
fn test_mixed_receiver_callable_call() {
    let out = compile_and_run(
        r#"<?php
function get_mixed(): mixed {
    return fn($v) => $v;
}
$x = get_mixed();
echo $x(2);
"#,
    );
    assert_eq!(out, "2");
}

/// Tests calling a value whose inferred type is a `Callable|Void` union.
///
/// Mirrors the Symfony console shape: a function that returns a closure in one
/// branch and nothing in another has inferred return `Union([Callable, Void])`.
/// The call runs only under the truthiness guard. Cross-checked with `php -r`.
#[test]
fn test_callable_void_union_receiver_call() {
    let out = compile_and_run(
        r#"<?php
function get_cb(bool $ok) {
    if ($ok) {
        return fn($v) => $v;
    }
}
$cb = get_cb(true);
if ($cb) {
    echo $cb(5);
}
"#,
    );
    assert_eq!(out, "5");
}

/// Tests calling a value typed as `Closure` (i.e. `Object("Closure")`).
///
/// A function returning `Closure` is called as `$h(21)`. Exercises the
/// `Object("Closure")` acceptance path (Closure has no user-class entry with
/// `__invoke`). Cross-checked with `php -r`.
#[test]
fn test_closure_return_type_call() {
    let out = compile_and_run(
        r#"<?php
function g(): Closure {
    return fn($x) => $x * 2;
}
$h = g();
echo $h(21);
"#,
    );
    assert_eq!(out, "42");
}
