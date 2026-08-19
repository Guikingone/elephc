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

/// Runtime-name fixture whose unconstrained string parameter requires the open callable universe.
const RUNTIME_STRING_CALLABLE_SOURCE: &str = r#"<?php
function return_named_callable(string $name): callable {
    return $name;
}
$callback = return_named_callable($argc > 0 ? "strtoupper" : "strtolower");
echo $callback("Mixed");
"#;

/// Open runtime-name fixture whose selector originates outside the compiler's finite literal set.
const OPEN_RUNTIME_STRING_CALLABLE_SOURCE: &str = r#"<?php
function return_open_named_callable(string $name): callable {
    return $name;
}
$callback = return_open_named_callable($argv[0]);
echo $callback("Mixed");
"#;

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

/// Verifies a runtime string name is validated and materialized at a callable return boundary.
#[test]
fn test_runtime_string_satisfies_callable_return_type() {
    let out = compile_and_run(RUNTIME_STRING_CALLABLE_SOURCE);
    assert_eq!(out, "MIXED");
}

/// Verifies an open runtime string universe emits one hash resolver call instead of a candidate ladder.
#[test]
fn test_runtime_string_callable_uses_compact_lookup_table() {
    let dir = make_cli_test_dir("elephc_runtime_string_callable_lookup_table");
    let (user_asm, _runtime_asm, _required_libraries) = compile_source_to_asm_with_options(
        OPEN_RUNTIME_STRING_CALLABLE_SOURCE,
        &dir,
        8_388_608,
        false,
        false,
    );
    assert!(
        user_asm.contains("__rt_callable_lookup_string_linear")
            || user_asm.contains("__rt_callable_lookup_string_hash"),
        "open string-callable dispatch should call a compact table resolver"
    );
    assert!(
        !user_asm.contains("mixed_string_descriptor_next"),
        "open string-callable dispatch must not emit a per-candidate descriptor ladder"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// Verifies a gradual receiver/method pair is validated and materialized as a callable return.
#[test]
fn test_gradual_array_satisfies_callable_return_type() {
    let out = compile_and_run(
        r#"<?php
class ReturnedStaticCallable {
    public static function decorate(string $value): string {
        return "[" . $value . "]";
    }
}
function return_array_callable(mixed $class, mixed $method): callable {
    return [$class, $method];
}
$callback = return_array_callable(ReturnedStaticCallable::class, "decorate");
echo $callback("ok");
"#,
    );
    assert_eq!(out, "[ok]");
}
