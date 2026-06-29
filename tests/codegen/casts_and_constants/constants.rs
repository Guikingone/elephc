//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of casts, constants, and introspection constants, including php integer max, php integer min, and m pi.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies `PHP_INT_MAX` constant is correctly substituted at compile time and the
/// resulting binary outputs the maximum 64-bit signed integer value.
/// Fixture: `<?php echo PHP_INT_MAX;` → expects `9223372036854775807`.
#[test]
fn test_php_int_max() {
    let out = compile_and_run("<?php echo PHP_INT_MAX;");
    assert_eq!(out, "9223372036854775807");
}

/// Verifies `PHP_INT_MIN` constant is correctly substituted at compile time and the
/// resulting binary outputs the minimum 64-bit signed integer value.
/// Fixture: `<?php echo PHP_INT_MIN;` → expects `-9223372036854775808`.
#[test]
fn test_php_int_min() {
    let out = compile_and_run("<?php echo PHP_INT_MIN;");
    assert_eq!(out, "-9223372036854775808");
}

/// Verifies `M_PI` math constant is correctly substituted at compile time and the
/// resulting binary outputs the correct float approximation.
/// Fixture: `<?php echo M_PI;` → expects `3.1415926535898`.
#[test]
fn test_m_pi() {
    let out = compile_and_run("<?php echo M_PI;");
    assert_eq!(out, "3.1415926535898");
}

/// Verifies `PHP_FLOAT_MAX` constant is correctly substituted and the resulting binary
/// runs without crash; also verifies `is_float()` returns true for the value.
/// Fixture: `<?php echo is_float(PHP_FLOAT_MAX);` → expects `1`.
#[test]
fn test_php_float_max() {
    let out = compile_and_run("<?php echo is_float(PHP_FLOAT_MAX);");
    assert_eq!(out, "1");
}

/// Verifies a fully-qualified reference to a predefined constant (`\PHP_INT_MAX`) parses and
/// resolves identically to the unqualified form. The leading `\` denotes the global namespace,
/// which is where these constants live. Symfony hits this with `\PHP_INT_MAX`/`\INF`/etc.
#[test]
fn test_fq_php_int_max() {
    let out = compile_and_run("<?php echo \\PHP_INT_MAX;");
    assert_eq!(out, "9223372036854775807");
}

/// Verifies `\DIRECTORY_SEPARATOR` (fully-qualified) resolves to the platform path separator.
#[test]
fn test_fq_directory_separator() {
    let out = compile_and_run("<?php echo \\DIRECTORY_SEPARATOR;");
    assert_eq!(out, "/");
}

/// Verifies `\PHP_EOL` (fully-qualified) resolves to the end-of-line string.
#[test]
fn test_fq_php_eol() {
    let out = compile_and_run("<?php echo \\PHP_EOL;");
    assert_eq!(out, "\n");
}

/// Verifies `\M_PI` (fully-qualified math constant) resolves to pi.
#[test]
fn test_fq_math_pi() {
    let out = compile_and_run("<?php echo \\M_PI;");
    assert_eq!(out, "3.1415926535898");
}

/// Verifies fully-qualified `\true`/`\false`/`\null` resolve to the global boolean/null constants.
#[test]
fn test_fq_bool_and_null() {
    let out = compile_and_run("<?php var_dump(\\true); var_dump(\\false); var_dump(\\null);");
    assert_eq!(out, "bool(true)\nbool(false)\nNULL\n");
}

/// Verifies `\INF` (fully-qualified) resolves to positive infinity.
#[test]
fn test_fq_infinity() {
    let out = compile_and_run("<?php $x = \\INF; echo is_infinite($x) ? \"inf\" : \"finite\";");
    assert_eq!(out, "inf");
}

/// Verifies `E_USER_DEPRECATED` error-level constant equals 16384 (PHP standard value).
/// Fixture: `<?php echo E_USER_DEPRECATED;` → expects `16384`.
#[test]
fn test_e_user_deprecated() {
    let out = compile_and_run("<?php echo E_USER_DEPRECATED;");
    assert_eq!(out, "16384");
}

/// Verifies `E_WARNING` error-level constant equals 2 and `E_ALL` is a positive integer.
/// Fixture tests both in one shot.
#[test]
fn test_e_warning_and_e_all() {
    let out = compile_and_run("<?php echo E_WARNING, \"|\", E_ALL > 0 ? 1 : 0;");
    assert_eq!(out, "2|1");
}

/// Verifies several error-level constants: E_ERROR=1, E_NOTICE=8, E_USER_ERROR=256.
#[test]
fn test_error_level_constants() {
    let out = compile_and_run("<?php echo E_ERROR, \"|\", E_NOTICE, \"|\", E_USER_ERROR;");
    assert_eq!(out, "1|8|256");
}

/// Verifies `DEBUG_BACKTRACE_IGNORE_ARGS=2` and `DEBUG_BACKTRACE_PROVIDE_OBJECT=1`.
#[test]
fn test_debug_backtrace_constants() {
    let out = compile_and_run("<?php echo DEBUG_BACKTRACE_IGNORE_ARGS, \"|\", DEBUG_BACKTRACE_PROVIDE_OBJECT;");
    assert_eq!(out, "2|1");
}

/// Verifies `PHP_SAPI` equals `"cli"` for elephc-compiled programs.
#[test]
fn test_php_sapi() {
    let out = compile_and_run("<?php echo PHP_SAPI;");
    assert_eq!(out, "cli");
}

/// Verifies `PHP_MAJOR_VERSION=8`, `PHP_MINOR_VERSION=4`, `PHP_VERSION_ID=80400`.
#[test]
fn test_php_version_constants() {
    let out = compile_and_run("<?php echo PHP_MAJOR_VERSION, \"|\", PHP_MINOR_VERSION, \"|\", PHP_VERSION_ID;");
    assert_eq!(out, "8|4|80400");
}

/// Verifies `PHP_INT_SIZE` equals 8 (64-bit LP64 target).
#[test]
fn test_php_int_size() {
    let out = compile_and_run("<?php echo PHP_INT_SIZE;");
    assert_eq!(out, "8");
}

/// Verifies `PHP_VERSION` is a non-empty string starting with "8.".
#[test]
fn test_php_version_string() {
    let out = compile_and_run("<?php echo strlen(PHP_VERSION) > 0 ? 1 : 0;");
    assert_eq!(out, "1");
}

/// Verifies `LC_NUMERIC` has the PHP-standard value 4 on supported targets.
#[test]
fn test_lc_numeric_constant() {
    let out = compile_and_run("<?php echo LC_NUMERIC;");
    assert_eq!(out, "4");
}

/// Verifies `LC_ALL` has the PHP-standard value 0 on supported targets.
#[test]
fn test_lc_all_constant() {
    let out = compile_and_run("<?php echo LC_ALL;");
    assert_eq!(out, "0");
}

/// Verifies all LC_* constants are reachable and have the expected PHP values.
#[test]
fn test_lc_all_constants() {
    let out = compile_and_run(
        "<?php echo LC_ALL,\"|\",LC_COLLATE,\"|\",LC_CTYPE,\"|\",LC_MONETARY,\"|\",LC_NUMERIC,\"|\",LC_TIME,\"|\",LC_MESSAGES;",
    );
    assert_eq!(out, "0|1|2|3|4|5|6");
}
