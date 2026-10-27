//! Purpose:
//! Groups the control flow integration test submodules into the parent suite.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Submodules group focused fixtures for branches and loops, alternative (`:` … `endif;`)
//!   syntax, functions, booleans, assignments, nulls, guarded reassignment, and related suites.

use crate::support::*;

#[path = "control_flow/alternative_syntax.rs"]
mod alternative_syntax;
#[path = "control_flow/branches_and_loops.rs"]
mod branches_and_loops;
#[path = "control_flow/functions.rs"]
mod functions;
#[path = "control_flow/booleans.rs"]
mod booleans;
#[path = "control_flow/assignments/mod.rs"]
mod assignments;
#[path = "control_flow/nulls.rs"]
mod nulls;
#[path = "control_flow/ternary.rs"]
mod ternary;
#[path = "control_flow/match_expressions.rs"]
mod match_expressions;
#[path = "control_flow/closures.rs"]
mod closures;
#[path = "control_flow/guarded_reassignment.rs"]
mod guarded_reassignment;

/// Verifies php's return-coverage rule is a RUN-TIME check, not a compile-time one.
///
/// php compiles `function foo(): int { }` as-is and raises a catchable
/// `TypeError: foo(): Return value must be of type int, none returned` at run time when a path
/// completes without a value. elephc used to refuse to compile.
#[test]
fn test_function_return_coverage_raises_runtime_type_error() {
    let out = compile_and_run(
        r#"<?php
function foo(): int { }
try {
    foo();
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "foo(): Return value must be of type int, none returned\n");
}

/// Verifies the partial-fallthrough path raises php's catchable TypeError at run time.
#[test]
fn test_function_partial_fallthrough_raises_runtime_type_error() {
    let out = compile_and_run(
        r#"<?php
function foo(bool $ok): int { if ($ok) { return 1; } }
try {
    var_dump(foo(true));
} catch (TypeError $e) {
    echo "unreachable\n";
}
try {
    var_dump(foo(false));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(
        out,
        "int(1)\nfoo(): Return value must be of type int, none returned\n"
    );
}

/// Verifies a switch `break` path raises php's catchable TypeError at run time.
#[test]
fn test_function_switch_break_path_raises_runtime_type_error() {
    let out = compile_and_run(
        r#"<?php
function foo(int $x): int { switch ($x) { case 1: if ($x > 0) { break; } return 1; default: return 2; } }
try {
    var_dump(foo(1));
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "foo(): Return value must be of type int, none returned\n");
}

/// Verifies a METHOD body that does not return on every path raises php's catchable TypeError
/// at run time, with php's `Class::method()` prefix.
#[test]
fn test_method_return_coverage_raises_runtime_type_error() {
    let out = compile_and_run(
        r#"<?php
class Box { public function value(): int { } }
try {
    (new Box())->value();
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(
        out,
        "Box::value(): Return value must be of type int, none returned\n"
    );
}

/// Verifies a CLOSURE body that does not return on every path raises php's catchable TypeError
/// at run time.
#[test]
fn test_closure_return_coverage_raises_runtime_type_error() {
    let out = compile_and_run(
        r#"<?php
$f = function(): int { };
try {
    $f();
} catch (TypeError $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert!(out.contains("Return value must be of type int, none returned"), "{out}");
}

/// Verifies a `: mixed` body that falls off the end compiles and raises php 8.5's catchable
/// `none returned` TypeError when that path runs: `mixed` accepts an explicit null, not an
/// implicit fall-through.
#[test]
fn test_mixed_return_coverage_accepts_implicit_null() {
    let out = compile_and_run(
        r#"<?php
class C { public function pick(): mixed {} }
try { (new C)->pick(); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
function f(): mixed { if (false) return 1; }
try { f(); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(
        out,
        "C::pick(): Return value must be of type mixed, none returned\n\
         f(): Return value must be of type mixed, none returned\n"
    );
}
