//! Purpose:
//! Recognition-layer tests for the misc/error-handling/process PHP builtins symfony/console and
//! symfony/string need: `method_exists`, `trigger_error`, `set_error_handler`,
//! `restore_error_handler`, `set_exception_handler`, `preg_quote`, `preg_grep`, `version_compare`,
//! `unpack`, `random_bytes`, `http_build_query`, `escapeshellarg`, `assert`,
//! `sapi_windows_cp_conv`, and `posix_kill`. These are registered for type checking and
//! first-class-callable resolution only; they have no EIR/codegen lowering yet, so these tests
//! assert type-check recognition (never `compile_and_run`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Verifies arity diagnostics, PHP-accurate return types (the `array|false` / `int|bool` /
//!   `?string` unions, the `string` transforms, and the `bool` predicates), and first-class-callable
//!   syntax. A regressing catalog/signature entry surfaces here as a spurious "Undefined function"
//!   or a wrong arity/return-type error.

use super::*;

/// Verifies that every registered misc builtin is recognized (no "Undefined function") in a
/// representative valid call form. Only the string-returning results are echoed; array/bool/mixed
/// results are merely bound to exercise their inferred types.
#[test]
fn test_misc_builtins_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = method_exists("Foo", "bar");
$b = trigger_error("deprecated");
$c = trigger_error("deprecated", 1024);
$d = set_error_handler(null);
$e = restore_error_handler();
$f = set_exception_handler(null);
$q = preg_quote("a.b*c");
$q2 = preg_quote("a/b", "/");
$grep = preg_grep("/^a/", ["apple", "banana"]);
$vc = version_compare("1.0.0", "2.0.0");
$un = unpack("Nlen", "\x00\x00\x00\x05");
$rb = random_bytes(16);
$hq = http_build_query(["a" => 1, "b" => 2]);
$es = escapeshellarg("some arg");
$as = assert(true);
$cp = sapi_windows_cp_conv(65001, 1252, "text");
$pk = posix_kill(12345, 9);
echo $q . $q2 . $rb . $hq . $es;
"#
        )
        .is_ok(),
        "all registered misc/error-handling/process builtins should type-check",
    );
}

/// Verifies the union-return types: `preg_grep`/`unpack` yield an `array|false` that is iterable
/// under an `is_array` guard, `version_compare` yields `int|bool` (echoable / truthy-testable), and
/// `sapi_windows_cp_conv` yields a `?string` that narrows through `=== null`.
#[test]
fn test_misc_union_returns_recognized() {
    assert!(
        check_source(
            r#"<?php
$grep = preg_grep("/x/", ["x1", "y2"]);
if (is_array($grep)) { foreach ($grep as $v) { echo $v; } }
$un = unpack("Nlen", "abcd");
if (is_array($un)) { foreach ($un as $val) { echo $val; } }
$vc = version_compare("1.0", "2.0");
echo $vc;
$vc2 = version_compare("1.0", "2.0", ">=");
if ($vc2) { echo "yes"; }
$cp = sapi_windows_cp_conv(1, 2, "s");
if ($cp === null) { echo "null"; } else { echo $cp; }
"#
        )
        .is_ok(),
        "misc union return types (array|false, int|bool, ?string) should type-check",
    );
}

/// Verifies that misc builtins work through first-class-callable syntax, since Symfony references
/// such functions as callables. `preg_quote`/`method_exists`/`version_compare` cover the string,
/// mixed-plus-string, and dual-string parameter shapes.
#[test]
fn test_misc_first_class_callable_recognized() {
    assert!(
        check_source(
            "<?php $f = preg_quote(...); $g = method_exists(...); $h = version_compare(...); \
             echo is_callable($f) && is_callable($g) && is_callable($h);"
        )
        .is_ok(),
        "misc builtins should be usable as first-class callables",
    );
}

expect_builtin_arity_error!(
    test_error_method_exists_wrong_args,
    "<?php method_exists(\"Foo\");",
    "method_exists() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_restore_error_handler_takes_no_args,
    "<?php restore_error_handler(1);",
    "restore_error_handler() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_posix_kill_wrong_args,
    "<?php posix_kill(1);",
    "posix_kill() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_version_compare_too_few_args,
    "<?php version_compare(\"1\");",
    "version_compare() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_random_bytes_wrong_args,
    "<?php random_bytes();",
    "random_bytes() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_assert_too_many_args,
    "<?php assert(true, \"desc\", 1);",
    "assert() takes 1 or 2 arguments"
);
