//! Purpose:
//! Recognition-layer tests for the multibyte-string (`mb_*`) PHP builtins. These builtins are
//! registered for type checking and first-class-callable resolution only; runtime lowering is
//! deferred, so these tests assert type-check recognition (never `compile_and_run`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Verifies arity diagnostics, PHP-accurate return types (the `|false` unions), and
//!   first-class-callable syntax. A regressing catalog/signature entry surfaces here as a
//!   spurious "Undefined function" or a wrong arity/return-type error.

use super::*;

/// Verifies that every registered `mb_*` builtin is recognized (no "Undefined
/// function") in a representative valid call form.
///
/// `mb_convert_encoding` is deliberately absent from this list: it is no longer recognition-only,
/// it ships as an elephc-PHP prelude (`crate::mb_convert_encoding_prelude`) that actually converts,
/// and this harness type-checks WITHOUT the prelude-injection stage. Its contract is covered where
/// the behaviour lives — `codegen::strings::test_mb_convert_encoding_*`.
#[test]
fn test_mbstring_builtins_recognized() {
    assert!(
        check_source(
            r#"<?php
$sub = mb_substr("héllo", 1, 3);
$len = mb_strlen("héllo");
$ip = mb_stripos("héllo", "L");
$rp = mb_strripos("héllo", "L");
$ss = mb_strstr("héllo", "l");
$is = mb_stristr("héllo", "L");
$lo = mb_strtolower("HÉLLO");
$up = mb_strtoupper("héllo");
$cc = mb_convert_case("héllo world", 0);
$de = mb_detect_encoding("héllo");
$ie = mb_internal_encoding();
$or = mb_ord("é");
$sp = mb_str_split("héllo");
$w = mb_strwidth("héllo");
echo $sub . $len . $lo . $up . $cc . $w;
"#
        )
        .is_ok(),
        "all registered mb_* builtins should type-check",
    );
}

/// Verifies the `|false`-union return types: `mb_stripos`/`mb_ord` yield
/// `int|false` and `mb_strstr`/`mb_detect_encoding` yield `string|false`, so a
/// strict `=== false` check type-checks against the union.
#[test]
fn test_mbstring_union_returns_recognized() {
    assert!(
        check_source(
            r#"<?php
$pos = mb_stripos("abc", "b");
if ($pos === false) { echo "no"; } else { echo $pos; }
$ord = mb_ord("a");
if ($ord === false) { echo "no"; } else { echo $ord; }
$hit = mb_strstr("abc", "b");
if ($hit === false) { echo "no"; } else { echo $hit; }
$enc = mb_detect_encoding("abc");
if ($enc === false) { echo "no"; } else { echo $enc; }
"#
        )
        .is_ok(),
        "mb_* union return types (int|false, string|false) should type-check",
    );
}

/// Verifies that the fully-optional-tail forms type-check: `mb_substr` with an
/// explicit `$length`/`$encoding`, and `mb_internal_encoding` with a set argument.
#[test]
fn test_mbstring_optional_args_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = mb_substr("héllo", 1, 3, "UTF-8");
$b = mb_str_split("héllo", 2, "UTF-8");
$ok = mb_internal_encoding("UTF-8");
echo $a . $b[0];
echo $ok ? "y" : "n";
"#
        )
        .is_ok(),
        "mb_* optional trailing arguments should type-check",
    );
}

/// Verifies that `mb_strlen`/`mb_strtolower` work through first-class-callable
/// syntax, since Symfony passes these as callables (e.g. to `array_map`).
#[test]
fn test_mbstring_first_class_callable_recognized() {
    assert!(
        check_source(
            "<?php $f = mb_strlen(...); $g = mb_strtolower(...); echo is_callable($f) && is_callable($g);"
        )
        .is_ok(),
        "mb_strlen(...)/mb_strtolower(...) first-class callable syntax should type-check",
    );
}

/// Verifies that `mb_strpos`/`mb_strrpos` (unmasked in this batch) are recognized
/// and yield `int|false`, so a strict `=== false` check type-checks against the union.
#[test]
fn test_mb_strpos_recognized() {
    assert!(
        check_source(
            r#"<?php
$p = mb_strpos("héllo", "l");
if ($p === false) { echo "no"; } else { echo $p; }
$r = mb_strrpos("héllo", "l", 0, "UTF-8");
if ($r === false) { echo "no"; } else { echo $r; }
"#
        )
        .is_ok(),
        "mb_strpos/mb_strrpos should type-check with int|false return",
    );
}

expect_builtin_arity_error!(
    test_error_mb_strpos_wrong_args,
    "<?php mb_strpos(\"a\");",
    "mb_strpos() takes 2 to 4 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_substr_wrong_args,
    "<?php mb_substr(\"a\");",
    "mb_substr() takes 2 to 4 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_strlen_too_many_args,
    "<?php mb_strlen(\"a\", \"UTF-8\", \"x\");",
    "mb_strlen() takes 1 or 2 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_convert_case_wrong_args,
    "<?php mb_convert_case(\"a\");",
    "mb_convert_case() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_internal_encoding_too_many_args,
    "<?php mb_internal_encoding(\"UTF-8\", \"x\");",
    "mb_internal_encoding() takes 0 or 1 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_str_split_too_many_args,
    "<?php mb_str_split(\"a\", 1, \"UTF-8\", 2);",
    "mb_str_split() takes 1 to 3 arguments"
);
