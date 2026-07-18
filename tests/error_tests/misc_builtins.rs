//! Purpose:
//! Recognition-layer tests for the misc/error-handling/process PHP builtins symfony/console and
//! symfony/string need: `method_exists`, `trigger_error`, `set_error_handler`,
//! `restore_error_handler`, `restore_exception_handler`, `set_exception_handler`, `preg_quote`,
//! `preg_grep`, `version_compare`, `unpack`, `random_bytes`, `http_build_query`, `escapeshellarg`,
//! `assert`, `sapi_windows_cp_conv`, `posix_kill`, and `filter_var` (plus the array builtin
//! `array_key_last`). Most of these are registered for type checking and first-class-callable
//! resolution only, with no EIR/codegen lowering yet, so those tests assert type-check
//! recognition (never `compile_and_run`).
//! `filter_var` is the exception: its core filters (`FILTER_DEFAULT`/`FILTER_UNSAFE_RAW`,
//! `FILTER_VALIDATE_INT`/`FLOAT`/`BOOL(EAN)`) DO have full EIR/runtime lowering when the filter id
//! is a compile-time literal — see `tests/codegen/filter_var.rs` for the runtime parity matrix.
//! The tests here cover the CHECKER-level loud diagnostics for filter_var's unsupported scope
//! (non-literal filter id, unsupported filter/flags, array-form `$options`) and the still-
//! recognition-only `array_key_last` builtin. `var_export` is intentionally NOT registered here:
//! it already has a runtime via the `var_export_prelude` injection, and catalog registration would
//! conflict with that prelude (see the deliverable report for the var_export conflict).
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
$fv = filter_var("user@example.com", 257);
$kl = array_key_last(["a" => 1, "b" => 2]);
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

/// Verifies `filter_var` is usable through first-class-callable syntax, since Symfony
/// references validation functions as callables. Exercises the mixed/int/mixed parameter
/// shape and the `Mixed` return type through `is_callable`.
#[test]
fn test_filter_var_first_class_callable_recognized() {
    assert!(
        check_source("<?php $f = filter_var(...); echo is_callable($f);").is_ok(),
        "filter_var should be usable as a first-class callable",
    );
}

/// Verifies `array_key_last` is recognized and its `string|int|null` return union narrows
/// through an `=== null` guard, mirroring `array_key_first`. Pins the new array builtin's
/// recognition and the union return type.
#[test]
fn test_array_key_last_return_union_type_checks() {
    assert!(
        check_source(
            r#"<?php
$k = array_key_last(["a" => 1, "b" => 2]);
if ($k === null) { echo "empty"; } else { echo $k; }
"#
        )
        .is_ok(),
        "array_key_last should type-check with a string|int|null return union",
    );
}

expect_builtin_arity_error!(
    test_error_restore_exception_handler_takes_no_args,
    "<?php restore_exception_handler(1);",
    "restore_exception_handler() takes no arguments"
);

expect_builtin_arity_error!(
    test_error_filter_var_too_many_args,
    "<?php filter_var(1, 2, 3, 4);",
    "filter_var() takes 1 to 3 arguments"
);

expect_builtin_arity_error!(
    test_error_array_key_last_takes_one_arg,
    "<?php array_key_last();",
    "array_key_last() takes exactly 1 argument"
);

// -- filter_var() scope-boundary loud diagnostics ---------------------------
// Locked decisions (see the deliverable report): a non-literal $filter is kept
// loud at the checker (simplest sound option); VALIDATE_IP/EMAIL/URL/MAC/DOMAIN/
// REGEXP, array-form $options, and unsupported flags (REQUIRE_ARRAY/FORCE_ARRAY/
// CALLBACK) are all kept loud rather than mis-validated. FILTER_REQUIRE_SCALAR is
// accepted as a verified no-op (see `test_filter_var_require_scalar_flag_accepted`
// in `tests/codegen/filter_var.rs`).

expect_builtin_arity_error!(
    test_error_filter_var_non_literal_filter_id,
    "<?php function f(int $filter) { return filter_var(\"42\", $filter); }",
    "filter_var(): a dynamic (non-compile-time-constant) $filter is not supported yet"
);

expect_builtin_arity_error!(
    test_error_filter_var_unsupported_validate_email,
    "<?php filter_var(\"a@b.com\", FILTER_VALIDATE_EMAIL);",
    "filter_var(): filter 274 is not supported yet"
);

expect_builtin_arity_error!(
    test_error_filter_var_unsupported_validate_ip,
    "<?php filter_var(\"127.0.0.1\", FILTER_VALIDATE_IP);",
    "filter_var(): filter 275 is not supported yet"
);

expect_builtin_arity_error!(
    test_error_filter_var_unsupported_flag_force_array,
    "<?php filter_var(\"42\", FILTER_VALIDATE_INT, FILTER_FORCE_ARRAY);",
    "filter_var(): flag combination"
);

expect_builtin_arity_error!(
    test_error_filter_var_unsupported_flag_callback,
    "<?php filter_var(\"42\", FILTER_CALLBACK);",
    "filter_var(): filter 1024 is not supported yet"
);

expect_builtin_arity_error!(
    test_error_filter_var_array_form_options,
    "<?php filter_var(\"42\", FILTER_VALIDATE_INT, ['flags' => FILTER_NULL_ON_FAILURE]);",
    "filter_var(): array-form $options"
);

expect_builtin_arity_error!(
    test_error_filter_var_non_literal_options,
    "<?php function f(int $opts) { return filter_var(\"42\", FILTER_VALIDATE_INT, $opts); }",
    "filter_var(): a dynamic (non-compile-time-constant) $options is not supported yet"
);

/// Verifies a `FILTER_NULL_ON_FAILURE | FILTER_REQUIRE_SCALAR` combined-flags
/// expression resolves statically (bitwise-OR of two known filter constants),
/// exercising `filter_static_int_value`'s `BinOp::BitOr` support.
#[test]
fn test_filter_var_combined_flags_resolve_statically() {
    assert!(
        check_source(
            "<?php $r = filter_var(\"42\", FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE | FILTER_REQUIRE_SCALAR); echo $r;"
        )
        .is_ok(),
        "a statically-resolvable combined flags expression should type-check",
    );
}

/// Verifies `\FILTER_VALIDATE_INT` (fully-qualified) resolves identically to
/// the bare form for the checker's static filter-id evaluation.
#[test]
fn test_filter_var_fully_qualified_constant_recognized() {
    assert!(
        check_source("<?php $r = filter_var(\"42\", \\FILTER_VALIDATE_INT); echo $r;").is_ok(),
        "a fully-qualified FILTER_VALIDATE_INT should resolve statically",
    );
}
