//! Purpose:
//! End-to-end codegen tests for the new predefined-constant families (FILTER_*,
//! UPLOAD_ERR_*, PHP_URL_*, PHP_QUERY_*, PHP_MAXPATHLEN, T_START_HEREDOC/
//! T_END_HEREDOC, XML_DOCUMENT_TYPE_NODE, DATE_* format strings) and the real
//! `filter_var()` core semantics (FILTER_DEFAULT/UNSAFE_RAW passthrough,
//! VALIDATE_INT/FLOAT/BOOL(EAN), FILTER_NULL_ON_FAILURE).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every assertion mirrors a value/behavior independently verified with
//!   `php -n -r '...'` (PHP 8.5.6 local) — see the deliverable report for the
//!   full verification transcript. `FILTER_DEFAULT` in particular regression-
//!   tests the 518 -> 516 bug fix.
//! - Loud-diagnostic (unsupported filter/flags/non-literal) coverage lives in
//!   `tests/error_tests/misc_builtins.rs`, not here.

use crate::support::*;

// --- Part A: predefined constants -------------------------------------------

/// Regression test for the `FILTER_DEFAULT` bug fix: PHP reports 516 (the same
/// value as `FILTER_UNSAFE_RAW`), not the previously-hardcoded 518.
#[test]
fn test_filter_default_is_516() {
    let out = compile_and_run("<?php echo FILTER_DEFAULT;");
    assert_eq!(out, "516");
}

/// Verifies the core `filter_var()` filter ids and the null-on-failure/require-scalar flags.
#[test]
fn test_filter_constant_values() {
    let out = compile_and_run(
        "<?php echo FILTER_VALIDATE_INT, \",\", FILTER_VALIDATE_FLOAT, \",\", \
         FILTER_VALIDATE_BOOL, \",\", FILTER_VALIDATE_BOOLEAN, \",\", \
         FILTER_NULL_ON_FAILURE, \",\", FILTER_REQUIRE_SCALAR, \",\", FILTER_UNSAFE_RAW;",
    );
    assert_eq!(out, "257,259,258,258,134217728,33554432,516");
}

/// Verifies `\FILTER_VALIDATE_INT` (fully-qualified) and the bare form both resolve
/// to the same value (namespace-fallback resolution for the new constant family).
#[test]
fn test_filter_constant_namespace_fallback() {
    let out = compile_and_run("<?php echo FILTER_VALIDATE_INT === \\FILTER_VALIDATE_INT ? \"same\" : \"diff\";");
    assert_eq!(out, "same");
}

/// Verifies the `UPLOAD_ERR_*` family, including the gap at code 5.
#[test]
fn test_upload_err_constant_values() {
    let out = compile_and_run(
        "<?php echo UPLOAD_ERR_OK, \",\", UPLOAD_ERR_INI_SIZE, \",\", UPLOAD_ERR_NO_TMP_DIR, \",\", UPLOAD_ERR_EXTENSION;",
    );
    assert_eq!(out, "0,1,6,8");
}

/// Verifies the `PHP_URL_*`/`PHP_QUERY_*` families.
#[test]
fn test_url_constant_values() {
    let out = compile_and_run(
        "<?php echo PHP_URL_SCHEME, \",\", PHP_URL_FRAGMENT, \",\", PHP_QUERY_RFC1738, \",\", PHP_QUERY_RFC3986;",
    );
    assert_eq!(out, "0,7,1,2");
}

/// Verifies `PHP_MAXPATHLEN` materializes the macOS `PATH_MAX` (1024) on the
/// default local target (php-verified: `php -n -r 'var_dump(PHP_MAXPATHLEN);'`
/// prints `int(1024)` on macOS).
#[test]
fn test_php_maxpathlen_macos_value() {
    let out = compile_and_run("<?php echo PHP_MAXPATHLEN;");
    assert_eq!(out, "1024");
}

/// Verifies the heredoc/nowdoc boundary tokenizer constants.
#[test]
fn test_tokenizer_constant_values() {
    let out = compile_and_run("<?php echo T_START_HEREDOC, \",\", T_END_HEREDOC;");
    assert_eq!(out, "398,399");
}

/// Verifies `XML_DOCUMENT_TYPE_NODE`.
#[test]
fn test_xml_constant_value() {
    let out = compile_and_run("<?php echo XML_DOCUMENT_TYPE_NODE;");
    assert_eq!(out, "10");
}

/// Verifies the `DATE_*` format-string constants, including the documented
/// aliasing (`DATE_ATOM` == `DATE_RFC3339` == `DATE_W3C`).
#[test]
fn test_date_format_string_constants() {
    let out = compile_and_run(
        "<?php echo DATE_ATOM, \"|\", DATE_RFC2822, \"|\", DATE_RFC3339_EXTENDED, \"|\", (DATE_ATOM === DATE_W3C ? \"same\" : \"diff\");",
    );
    assert_eq!(
        out,
        "Y-m-d\\TH:i:sP|D, d M Y H:i:s O|Y-m-d\\TH:i:s.vP|same"
    );
}

// --- Part B: filter_var() core semantics ------------------------------------

/// Helper: renders a PHP `var_dump`-style boolean/expression echo for a
/// single `filter_var()` call and returns elephc's stdout.
fn fv(expr: &str) -> String {
    compile_and_run(&format!("<?php var_dump({});", expr))
}

// -- FILTER_VALIDATE_INT on string input (dedicated decimal-only parser) --

#[test]
fn test_filter_var_int_str_basic() {
    assert_eq!(fv("filter_var(\" 42 \", FILTER_VALIDATE_INT)"), "int(42)\n");
    assert_eq!(fv("filter_var(\"+42\", FILTER_VALIDATE_INT)"), "int(42)\n");
    assert_eq!(fv("filter_var(\"-42\", FILTER_VALIDATE_INT)"), "int(-42)\n");
    assert_eq!(fv("filter_var(\"0\", FILTER_VALIDATE_INT)"), "int(0)\n");
    assert_eq!(fv("filter_var(\"-0\", FILTER_VALIDATE_INT)"), "int(0)\n");
}

/// "0x1A" is rejected outright (decimal-only grammar, never hex); "012"/"00"
/// are rejected by the leading-zero rule (a leading `0` is only valid alone).
#[test]
fn test_filter_var_int_str_rejects_hex_and_leading_zero() {
    assert_eq!(fv("filter_var(\"0x1A\", FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"012\", FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"00\", FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"\", FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"12.0\", FILTER_VALIDATE_INT)"), "bool(false)\n");
}

/// Saturating overflow boundary: `PHP_INT_MAX`/`PHP_INT_MIN` succeed exactly;
/// one past either boundary fails.
#[test]
fn test_filter_var_int_str_overflow_boundary() {
    assert_eq!(
        fv("filter_var(\"9223372036854775807\", FILTER_VALIDATE_INT)"),
        "int(9223372036854775807)\n"
    );
    assert_eq!(
        fv("filter_var(\"9223372036854775808\", FILTER_VALIDATE_INT)"),
        "bool(false)\n"
    );
    assert_eq!(
        fv("filter_var(\"-9223372036854775808\", FILTER_VALIDATE_INT)"),
        "int(-9223372036854775808)\n"
    );
    assert_eq!(
        fv("filter_var(\"-9223372036854775809\", FILTER_VALIDATE_INT)"),
        "bool(false)\n"
    );
}

/// PHP-filter whitespace trims tab/LF/CR/VT/space but NOT form feed (`\x0c`).
#[test]
fn test_filter_var_int_str_whitespace_set() {
    assert_eq!(fv("filter_var(\"\\t1\", FILTER_VALIDATE_INT)"), "int(1)\n");
    assert_eq!(fv("filter_var(\"1\\n\", FILTER_VALIDATE_INT)"), "int(1)\n");
    assert_eq!(fv("filter_var(\"\\x0c42\", FILTER_VALIDATE_INT)"), "bool(false)\n");
}

// -- FILTER_VALIDATE_FLOAT on string input --

#[test]
fn test_filter_var_float_str_basic() {
    assert_eq!(fv("filter_var(\"1.5\", FILTER_VALIDATE_FLOAT)"), "float(1.5)\n");
    assert_eq!(fv("filter_var(\"1\", FILTER_VALIDATE_FLOAT)"), "float(1)\n");
    assert_eq!(fv("filter_var(\".5\", FILTER_VALIDATE_FLOAT)"), "float(0.5)\n");
    assert_eq!(fv("filter_var(\"-.5\", FILTER_VALIDATE_FLOAT)"), "float(-0.5)\n");
    assert_eq!(fv("filter_var(\"1.2e3\", FILTER_VALIDATE_FLOAT)"), "float(1200)\n");
    assert_eq!(fv("filter_var(\"1.2E-3\", FILTER_VALIDATE_FLOAT)"), "float(0.0012)\n");
}

#[test]
fn test_filter_var_float_str_rejects_non_grammar() {
    assert_eq!(fv("filter_var(\"0x1A\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\".\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"e3\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"1.5.5\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"INF\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(\"NAN\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
}

/// `strtod`-overflow (`"1e400"`) is detected and rejected rather than returning infinity.
#[test]
fn test_filter_var_float_str_overflow_rejected() {
    assert_eq!(fv("filter_var(\"1e400\", FILTER_VALIDATE_FLOAT)"), "bool(false)\n");
}

// -- FILTER_VALIDATE_BOOL(EAN) on string input --

#[test]
fn test_filter_var_bool_str_tokens() {
    for (tok, expected) in [
        ("1", "bool(true)\n"),
        ("true", "bool(true)\n"),
        ("on", "bool(true)\n"),
        ("yes", "bool(true)\n"),
        ("TRUE", "bool(true)\n"),
        ("ON", "bool(true)\n"),
        ("0", "bool(false)\n"),
        ("false", "bool(false)\n"),
        ("off", "bool(false)\n"),
        ("no", "bool(false)\n"),
        ("False", "bool(false)\n"),
        ("", "bool(false)\n"),
        ("abc", "bool(false)\n"),
        ("2", "bool(false)\n"),
        ("01", "bool(false)\n"),
    ] {
        let out = fv(&format!("filter_var(\"{}\", FILTER_VALIDATE_BOOLEAN)", tok));
        assert_eq!(out, expected, "token {:?}", tok);
    }
}

/// Surrounding whitespace is trimmed for bool tokens too.
#[test]
fn test_filter_var_bool_str_whitespace_trimmed() {
    assert_eq!(fv("filter_var(\" true \", FILTER_VALIDATE_BOOLEAN)"), "bool(true)\n");
    assert_eq!(fv("filter_var(\"true \", FILTER_VALIDATE_BOOLEAN)"), "bool(true)\n");
}

// -- FILTER_NULL_ON_FAILURE --

#[test]
fn test_filter_var_null_on_failure_int() {
    assert_eq!(
        fv("filter_var(\"abc\", FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE)"),
        "NULL\n"
    );
    assert_eq!(
        fv("filter_var(\"42\", FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE)"),
        "int(42)\n"
    );
}

/// `""` is a VALID false for the BOOL filter, not a failure — the flag must
/// not turn it into null.
#[test]
fn test_filter_var_bool_null_on_failure_empty_stays_false() {
    assert_eq!(
        fv("filter_var(\"\", FILTER_VALIDATE_BOOLEAN, FILTER_NULL_ON_FAILURE)"),
        "bool(false)\n"
    );
    assert_eq!(
        fv("filter_var(\"abc\", FILTER_VALIDATE_BOOLEAN, FILTER_NULL_ON_FAILURE)"),
        "NULL\n"
    );
}

/// `FILTER_REQUIRE_SCALAR` is accepted as a verified no-op (unblocks Symfony's
/// `InputBag::filter()` pattern of always setting it alongside `FILTER_NULL_ON_FAILURE`).
#[test]
fn test_filter_var_require_scalar_flag_accepted() {
    assert_eq!(
        fv("filter_var(\"42\", FILTER_VALIDATE_INT, FILTER_REQUIRE_SCALAR)"),
        "int(42)\n"
    );
    assert_eq!(
        fv("filter_var(\"abc\", FILTER_VALIDATE_INT, FILTER_REQUIRE_SCALAR | FILTER_NULL_ON_FAILURE)"),
        "NULL\n"
    );
}

// -- input polymorphism (non-string inputs) --

#[test]
fn test_filter_var_int_input_polymorphism() {
    assert_eq!(fv("filter_var(1, FILTER_VALIDATE_INT)"), "int(1)\n");
    assert_eq!(fv("filter_var(1.0, FILTER_VALIDATE_INT)"), "int(1)\n");
    assert_eq!(fv("filter_var(1.5, FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(true, FILTER_VALIDATE_INT)"), "int(1)\n");
    // A php-verified quirk: bool false ALWAYS fails VALIDATE_INT (unlike VALIDATE_BOOL).
    assert_eq!(fv("filter_var(false, FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var(null, FILTER_VALIDATE_INT)"), "bool(false)\n");
}

#[test]
fn test_filter_var_bool_input_polymorphism() {
    assert_eq!(fv("filter_var(true, FILTER_VALIDATE_BOOLEAN)"), "bool(true)\n");
    // Unlike VALIDATE_INT, bool false is ALWAYS a valid (not failing) result here.
    assert_eq!(fv("filter_var(false, FILTER_VALIDATE_BOOLEAN)"), "bool(false)\n");
    assert_eq!(fv("filter_var(1, FILTER_VALIDATE_BOOLEAN)"), "bool(true)\n");
    assert_eq!(fv("filter_var(0, FILTER_VALIDATE_BOOLEAN)"), "bool(false)\n");
    assert_eq!(fv("filter_var(2, FILTER_VALIDATE_BOOLEAN)"), "bool(false)\n");
}

/// Array input always fails (without `FILTER_REQUIRE_ARRAY`/`FILTER_FORCE_ARRAY`,
/// which stay loud), matching php exactly rather than crashing or mis-validating.
#[test]
fn test_filter_var_array_input_always_fails() {
    assert_eq!(fv("filter_var([1, 2], FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var([], FILTER_VALIDATE_INT)"), "bool(false)\n");
    assert_eq!(fv("filter_var([1, 2], FILTER_DEFAULT)"), "bool(false)\n");
}

// -- FILTER_DEFAULT / FILTER_UNSAFE_RAW passthrough --

#[test]
fn test_filter_var_default_passthrough() {
    let out = compile_and_run("<?php echo filter_var(\"hello\");");
    assert_eq!(out, "hello");
    let out = compile_and_run("<?php echo filter_var(\"hello\", FILTER_UNSAFE_RAW);");
    assert_eq!(out, "hello");
}

/// `FILTER_DEFAULT` stringifies non-string scalars exactly like a `(string)` cast.
#[test]
fn test_filter_var_default_stringifies_scalars() {
    assert_eq!(fv("filter_var(123, FILTER_DEFAULT)"), "string(3) \"123\"\n");
    assert_eq!(fv("filter_var(true, FILTER_DEFAULT)"), "string(1) \"1\"\n");
    assert_eq!(fv("filter_var(false, FILTER_DEFAULT)"), "string(0) \"\"\n");
    assert_eq!(fv("filter_var(null, FILTER_DEFAULT)"), "string(0) \"\"\n");
}

// -- Mixed-typed input dispatch (via a heterogeneous array element) --

#[test]
fn test_filter_var_mixed_dispatch() {
    let out = compile_and_run(
        r#"<?php
$data = ["s" => "42", "i" => 7, "f" => 3.5, "b" => true, "n" => null];
foreach ($data as $v) {
    echo filter_var($v, FILTER_VALIDATE_INT, FILTER_NULL_ON_FAILURE) ?? "null", ",";
}
"#,
    );
    assert_eq!(out, "42,7,null,1,null,");
}

/// 2-arg call using the fully-qualified `\FILTER_VALIDATE_INT` form.
#[test]
fn test_filter_var_fully_qualified_form() {
    let out = compile_and_run("<?php var_dump(filter_var(\"42\", \\FILTER_VALIDATE_INT));");
    assert_eq!(out, "int(42)\n");
}
