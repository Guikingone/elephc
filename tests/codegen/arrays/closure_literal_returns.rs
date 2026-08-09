//! Purpose:
//! Regression tests for the array-storage type stamped on an array literal that a closure
//! returns directly: nested literals, an array-typed parameter, an associative literal, and a
//! literal containing a spread.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected value is verbatim `LC_ALL=C php` output from PHP 8.4.20.
//! - The closure return-type inference in `src/ir_lower/function.rs` and the literal typing in
//!   `src/ir_lower/expr` (`array_literal_type_for_ir` / `assoc_array_literal_type_for_ir`) type
//!   the same literal and must agree: `lower_return_expr` feeds the inferred return element type
//!   back into `lower_array_literal_with_expected_type`, and the caller reads the returned
//!   array through the same signature metadata. When the inference fell back to the syntactic
//!   `int` default, `function (array $xs) { return [$xs]; }` returned `[1]` instead of the
//!   nested array and `function (string $s) { return ['k' => $s]; }` read the string payload
//!   back as a raw pointer-sized integer.
//! - The spread fixture pins the caller-side stamp: the body already built `array<mixed>`
//!   through the spread lowering while the signature still advertised `array<int>`.

use crate::support::*;

/// Verifies a nested array literal inside a closure-returned literal keeps its inner element
/// types, including a string in the inner array and at the outer level.
#[test]
fn test_closure_returns_nested_array_literal_of_mixed_params() {
    let out = compile_and_run(
        r#"<?php
$f = function (mixed $a, mixed $b) { return [[$a, $b], $b]; };
var_dump($f(1, "z"));
"#,
    );
    assert_eq!(
        out,
        "array(2) {\n  [0]=>\n  array(2) {\n    [0]=>\n    int(1)\n    [1]=>\n    string(1) \"z\"\n  }\n  [1]=>\n  string(1) \"z\"\n}\n"
    );
}

/// Verifies an `array`-typed parameter wrapped in a returned literal stays an array instead of
/// being cast to the syntactic `int` default.
#[test]
fn test_closure_returns_array_literal_wrapping_array_param() {
    let out = compile_and_run(
        r#"<?php
$f = function (array $xs) { return [$xs]; };
var_dump($f([1, 2]));
"#,
    );
    assert_eq!(
        out,
        "array(1) {\n  [0]=>\n  array(2) {\n    [0]=>\n    int(1)\n    [1]=>\n    int(2)\n  }\n}\n"
    );
}

/// Verifies an associative literal returned directly from a closure stamps its value slot from
/// the parameter type, so reading the key back yields the string rather than a raw integer.
#[test]
fn test_closure_returns_assoc_literal_of_typed_param() {
    let out = compile_and_run(
        r#"<?php
$f = function (string $s) { return ['k' => $s]; };
$r = $f("yo");
var_dump($r['k']);
var_dump($r);
"#,
    );
    assert_eq!(
        out,
        "string(2) \"yo\"\narray(1) {\n  [\"k\"]=>\n  string(2) \"yo\"\n}\n"
    );
}

/// Verifies a returned literal that spreads an array parameter and appends a `mixed` argument
/// keeps the appended string, pinning agreement between the callee's spread lowering and the
/// signature the caller reads.
#[test]
fn test_closure_returns_spread_array_literal() {
    let out = compile_and_run(
        r#"<?php
$f = function (array $xs, mixed $y) { return [...$xs, $y]; };
var_dump($f([1, 2], "s"));
"#,
    );
    assert_eq!(
        out,
        "array(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  string(1) \"s\"\n}\n"
    );
}
