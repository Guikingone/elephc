//! Purpose:
//! End-to-end coverage for PHP callable strings bound to a declared `callable` parameter
//! (`function apply(callable $f) {...} apply("strtoupper", ...)`), covering plain function
//! names, `"Class::method"` names, case-insensitive and namespaced spellings, and the named
//! argument form.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected value is verbatim `LC_ALL=C php` 8.4.20 stdout.
//! - elephc resolves callables statically, so only a compile-time-known string binds; the
//!   rejected shapes are pinned in `tests/error_tests/callables.rs`.

use crate::support::*;

/// Verifies a builtin function-name string binds to a declared `callable` parameter and is
/// invoked inside the callee — the repro from the parameter-typing audit.
#[test]
fn test_builtin_name_string_binds_to_callable_parameter() {
    let out = compile_and_run(
        r#"<?php
        function apply(callable $f, string $s) { return $f($s); }
        echo apply("strtoupper", "abc");
        "#,
    );
    assert_eq!(out, "ABC");
}

/// Verifies a user-defined function-name string binds to a declared `callable` parameter.
#[test]
fn test_user_function_name_string_binds_to_callable_parameter() {
    let out = compile_and_run(
        r#"<?php
        function decorate(string $s) { return "[" . $s . "]"; }
        function apply(callable $f, string $s) { return $f($s); }
        echo apply("decorate", "abc");
        "#,
    );
    assert_eq!(out, "[abc]");
}

/// Verifies PHP's case-insensitive function names and the fully qualified `\name` spelling
/// both resolve when passed as a callable string.
#[test]
fn test_callable_name_string_is_case_insensitive_and_accepts_leading_backslash() {
    let out = compile_and_run(
        r#"<?php
        function apply(callable $f, string $s) { return $f($s); }
        echo apply("STRTOUPPER", "ab"), apply("\\strtolower", "CD");
        "#,
    );
    assert_eq!(out, "ABcd");
}

/// Verifies a `"Class::method"` string binds to a declared `callable` parameter and dispatches
/// to the static method.
#[test]
fn test_static_method_name_string_binds_to_callable_parameter() {
    let out = compile_and_run(
        r#"<?php
        class Formatter {
            public static function wrap(string $s): string { return "<" . $s . ">"; }
        }
        function apply(callable $f, string $s) { return $f($s); }
        echo apply("Formatter::wrap", "abc");
        "#,
    );
    assert_eq!(out, "<abc>");
}

/// Verifies the binding also fires when the callable string is passed as a named argument,
/// which reaches EIR through the reordered named-argument path.
#[test]
fn test_callable_name_string_binds_through_named_argument() {
    let out = compile_and_run(
        r#"<?php
        function apply(callable $f, string $s) { return $f($s); }
        echo apply(s: "abc", f: "strtoupper");
        "#,
    );
    assert_eq!(out, "ABC");
}

/// Verifies a callable string bound to a method parameter behaves like the function case.
#[test]
fn test_callable_name_string_binds_to_method_parameter() {
    let out = compile_and_run(
        r#"<?php
        class Runner {
            public function run(callable $f, string $s) { return $f($s); }
        }
        echo (new Runner())->run("strtoupper", "abc");
        "#,
    );
    assert_eq!(out, "ABC");
}

/// Verifies a bound callable string carries its signature into the callee, so a call with the
/// wrong argument count is still rejected rather than silently accepted.
#[test]
fn test_bound_callable_string_keeps_working_alongside_first_class_callables() {
    let out = compile_and_run(
        r#"<?php
        function apply(callable $f, string $s) { return $f($s); }
        echo apply("strtoupper", "ab"), apply(strtolower(...), "CD"), apply(fn($x) => $x . "!", "e");
        "#,
    );
    assert_eq!(out, "ABcde!");
}
