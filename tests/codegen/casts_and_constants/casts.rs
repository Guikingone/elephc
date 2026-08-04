//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of casts, constants, and introspection casts, including cast integer from float, cast integer from string, and cast integer from boolean.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Compiles `<?php echo (int)3.7;` and asserts stdout is `"3"` — truncates float toward zero.
#[test]
fn test_cast_int_from_float() {
    let out = compile_and_run("<?php echo (int)3.7;");
    assert_eq!(out, "3");
}

/// Compiles `<?php echo (int)"42";` and asserts stdout is `"42"` — parses decimal integer from string.
#[test]
fn test_cast_int_from_string() {
    let out = compile_and_run("<?php echo (int)\"42\";");
    assert_eq!(out, "42");
}

/// Verifies PHP numeric-string rules for int casts, `intval()`, and Mixed string payloads.
#[test]
fn test_cast_int_from_numeric_strings_uses_php_conversion_rules() {
    let out = compile_and_run(
        r#"<?php
echo (int)" 42", ":", (float)" 42", "\n";
echo (int)"1e2", ":", (float)"1e2", "\n";
echo (int)"  +7", ":", (float)"  +7", "\n";
echo (int)"1.9", ":", (float)"1.9", "\n";
echo (int)"1.9e2", ":", (float)"1.9e2", "\n";
echo (int)"1e2abc", ":", (float)"1e2abc", "\n";
echo (int)"  -2.7e1", ":", (float)"  -2.7e1", "\n";
echo "intval:", intval("  +7"), ":", intval("1e2"), "\n";
$map = ["exp" => "1e2", "plus" => "  +7", "n" => 5];
echo "mixed:", (int)$map["exp"], ":", (int)$map["plus"];
"#,
    );
    assert_eq!(
        out,
        "42:42\n100:100\n7:7\n1:1.9\n190:190\n100:100\n-27:-27\nintval:7:100\nmixed:100:7"
    );
}

/// Regression: a cast binds tighter than the following binary operator (PHP precedence).
/// `(int)$x + 3` is `((int)$x) + 3` (not `(int)($x + 3)`), `(int)$x * 2` likewise, and
/// `(int)$n . "x"` concatenates the cast result. Before the parser fix the cast operand was
/// parsed at too-low a binding power and swallowed the trailing operator: arithmetic forms were
/// rejected as "non-numeric operands" and the concat form silently dropped the suffix.
#[test]
fn test_cast_precedence_binds_tighter_than_binary_ops() {
    let out = compile_and_run(
        r#"<?php
$x = "5";
$n = 5;
echo (int)$x + 3, "|", (int)$x * 2, "|", (float)"2.5" + 1, "|", (int)$n . "x";
"#,
    );
    assert_eq!(out, "8|10|3.5|5x");
}

/// Compiles `<?php echo (int)true;` and asserts stdout is `"1"` — true becomes 1.
#[test]
fn test_cast_int_from_bool() {
    let out = compile_and_run("<?php echo (int)true;");
    assert_eq!(out, "1");
}

/// Compiles `<?php echo (float)42;` and asserts stdout is `"42"` — int widens to float without truncation.
#[test]
fn test_cast_float_from_int() {
    let out = compile_and_run("<?php echo (float)42;");
    assert_eq!(out, "42");
}

/// Compiles `<?php echo (float)'3.14';` and asserts stdout is `"3.14"` — parses float from numeric string.
#[test]
fn test_cast_float_from_string() {
    let out = compile_and_run("<?php echo (float)'3.14';");
    assert_eq!(out, "3.14");
}

/// Compiles `<?php echo (float)'42';` and asserts stdout is `"42"` — string integer widens to float.
#[test]
fn test_cast_float_from_string_integer() {
    let out = compile_and_run("<?php echo (float)'42';");
    assert_eq!(out, "42");
}

/// Compiles `<?php echo (float)'abc';` and asserts stdout is `"0"` — non-numeric string becomes 0.
#[test]
fn test_cast_float_from_string_non_numeric() {
    let out = compile_and_run("<?php echo (float)'abc';");
    assert_eq!(out, "0");
}

/// Compiles `<?php echo (string)42;` and asserts stdout is `"42"`.
#[test]
fn test_cast_string_from_int() {
    let out = compile_and_run("<?php echo (string)42;");
    assert_eq!(out, "42");
}

/// Compiles `<?php echo (string)3.14;` and asserts stdout is `"3.14"`.
#[test]
fn test_cast_string_from_float() {
    let out = compile_and_run("<?php echo (string)3.14;");
    assert_eq!(out, "3.14");
}

/// Compiles `<?php echo (string)true;` and asserts stdout is `"1"`.
#[test]
fn test_cast_string_from_bool_true() {
    let out = compile_and_run("<?php echo (string)true;");
    assert_eq!(out, "1");
}

/// Compiles `<?php echo (string)false;` and asserts stdout is `""` — false becomes empty string.
#[test]
fn test_cast_string_from_bool_false() {
    let out = compile_and_run("<?php echo (string)false;");
    assert_eq!(out, "");
}

/// Compiles `<?php echo (bool)0;` and asserts stdout is `""` — zero is falsy.
#[test]
fn test_cast_bool_from_int_zero() {
    let out = compile_and_run("<?php echo (bool)0;");
    assert_eq!(out, "");
}

/// Compiles `<?php echo (bool)42;` and asserts stdout is `"1"` — non-zero int is truthy.
#[test]
fn test_cast_bool_from_int_nonzero() {
    let out = compile_and_run("<?php echo (bool)42;");
    assert_eq!(out, "1");
}

/// Compiles `<?php echo (bool)"";` and asserts stdout is `""` — empty string is falsy.
#[test]
fn test_cast_bool_from_string_empty() {
    let out = compile_and_run("<?php echo (bool)\"\";");
    assert_eq!(out, "");
}

/// Compiles `<?php echo (bool)"hello";` and asserts stdout is `"1"` — non-empty string is truthy.
#[test]
fn test_cast_bool_from_string_nonempty() {
    let out = compile_and_run("<?php echo (bool)\"hello\";");
    assert_eq!(out, "1");
}

/// Verifies casts unbox PhpMixed payload correctly: float→int truncation, string→int parse,
/// int→bool truthiness, true→string "1", null→string "", and int→string decimal.
#[test]
fn test_cast_mixed_unboxes_payload() {
    let out = compile_and_run(
        r#"<?php
$map = [
    "int" => 42,
    "float" => 3.75,
    "true" => true,
    "false" => false,
    "null" => null,
    "text" => "27",
];
echo (int)$map["float"];
echo "|";
echo (int)$map["text"];
echo "|";
echo (bool)$map["int"] ? "1" : "0";
echo (bool)$map["false"] ? "1" : "0";
echo "|";
echo (string)$map["true"];
echo "|";
echo (string)$map["null"];
echo "|";
echo (string)$map["int"];
"#,
    );
    assert_eq!(out, "3|27|10|1||42");
}

/// Compiles `<?php echo (integer)3.7;` and asserts stdout is `"3"` — (integer) is a PHP alias for (int).
#[test]
fn test_cast_integer_alias() {
    let out = compile_and_run("<?php echo (integer)3.7;");
    assert_eq!(out, "3");
}

/// Compiles `<?php echo (double)42;` and asserts stdout is `"42"` — (double) is a PHP alias for (float).
#[test]
fn test_cast_double_alias() {
    let out = compile_and_run("<?php echo (double)42;");
    assert_eq!(out, "42");
}

/// Compiles `<?php echo (boolean)1;` and asserts stdout is `"1"` — (boolean) is a PHP alias for (bool).
#[test]
fn test_cast_boolean_alias() {
    let out = compile_and_run("<?php echo (boolean)1;");
    assert_eq!(out, "1");
}

/// Verifies cast keywords are case-insensitive: INTEGER, DOUBLE, STRING, BOOLEAN all work.
#[test]
fn test_cast_keywords_are_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
echo (INTEGER)3.7;
echo ":";
echo (DOUBLE)"2.5";
echo ":";
echo (STRING)42;
echo ":";
echo (BOOLEAN)0 ? "true" : "false";
"#,
    );
    assert_eq!(out, "3:2.5:42:false");
}

// --- gettype ---

// --- PHP float->int cast edge cases ---

/// Verifies `(int)` and `intval()` on NaN/±INF return `0` like PHP, on both supported targets.
///
/// AArch64 `fcvtzs` saturates and x86_64 `cvttsd2si` returns `INT64_MIN`, so both targets used
/// to disagree with PHP and with each other. Values go through `$argc` so the folders cannot
/// evaluate the cast at compile time.
#[test]
fn test_cast_int_from_nan_and_infinity_is_zero() {
    let out = compile_and_run(
        r#"<?php
$n = $argc;
echo (int)(NAN * $n), ':', (int)(INF * $n), ':', (int)(-INF * $n);
echo ':', intval(NAN * $n), ':', intval(INF * $n);
"#,
    );
    assert_eq!(out, "0:0:0:0:0");
}

/// Verifies `(int)` of a float far outside the int64 range matches PHP's modulo-2^64 result.
#[test]
fn test_cast_int_from_out_of_range_float() {
    let out = compile_and_run(
        r#"<?php
$n = $argc;
echo (int)(1e300 * $n), ':', (int)(-1e300 * $n), ':', (int)(1.5e19 * $n);
"#,
    );
    assert_eq!(out, "0:0:-3446744073709551616");
}

/// Verifies RUNTIME `(float)` / `(int)` string casts follow PHP's numeric-string grammar
/// rather than libc `strtod`'s, so they agree with the compile-time folder in
/// `crate::optimize::fold::casts`. PHP has no `INF`/`NAN` spelling (`(float)"INF"` is
/// `0.0`), no hexadecimal form (`(int)"0x1A"` is `0`, not `26`) and no underscore
/// separator (`(float)"1_000"` is `1.0`); leading/trailing whitespace is allowed, a bare
/// `"1e"` stops before the exponent, and only the leading numeric run counts.
/// The strings come out of a `foreach`, so the runtime helpers are what is exercised.
#[test]
fn test_runtime_string_casts_follow_php_numeric_string_grammar() {
    let out = compile_and_run(
        r#"<?php
$strings = ["INF", "-INF", "nan", "infinity", "0x1A", "1e400", " 42 ", "+.5e-2", "1_000", "5.", ".5", "1e", "12abc", "1.2.3", "", "9223372036854775808"];
foreach ($strings as $s) { echo (float)$s, ",", (int)$s, ",", is_numeric($s) ? "T" : "F", "|"; }
"#,
    );
    assert_eq!(
        out,
        "0,0,F|0,0,F|0,0,F|0,0,F|0,0,F|INF,0,T|42,42,T|0.005,0,T|1,1,F|5,5,T|0.5,0,T|1,1,F|12,12,F|1.2,1,F|0,0,F|9.2233720368548E+18,9223372036854775807,T|"
    );
}

/// Verifies a string literal and the same string reaching the cast at runtime produce the
/// SAME value: the constant folder and the runtime helper must implement one grammar.
/// A mismatch here means `(float)"0x1A"` answers `0.0` in one path and `26.0` in the other.
#[test]
fn test_string_cast_literal_and_runtime_paths_agree() {
    let out = compile_and_run(
        r#"<?php
$runtime = ["INF", "0x1A", "1_000", " 42 ", "1e", "12abc"];
echo (float)"INF", ",", (int)"INF", "|";
echo (float)"0x1A", ",", (int)"0x1A", "|";
echo (float)"1_000", ",", (int)"1_000", "|";
echo (float)" 42 ", ",", (int)" 42 ", "|";
echo (float)"1e", ",", (int)"1e", "|";
echo (float)"12abc", ",", (int)"12abc", "|";
echo "\n";
foreach ($runtime as $s) { echo (float)$s, ",", (int)$s, "|"; }
"#,
    );
    let (folded, runtime) = out.split_once('\n').expect("two output lines");
    assert_eq!(folded, runtime);
    assert_eq!(folded, "0,0|0,0|1,1|42,42|1,1|12,12|");
}
