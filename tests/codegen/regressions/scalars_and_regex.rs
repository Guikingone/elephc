//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of regressions scalars and regex, including null byte in string, not empty string is true, and not nonempty string is false.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests that `strlen()` correctly counts bytes including embedded null characters.
/// Fixture: a string with an embedded null byte (`"ab\0cd"` has 5 bytes).
#[test]
fn test_null_byte_in_string() {
    let out = compile_and_run(r#"<?php echo strlen("ab\0cd");"#);
    assert_eq!(out, "5");
}

// -- Issue #26: empty string should be falsy --

/// Verifies that an empty string is falsy (double negation yields empty string = false).
/// Regression for issue #26.
#[test]
fn test_not_empty_string_is_true() {
    let out = compile_and_run(r#"<?php echo !!"";"#);
    assert_eq!(out, "");
}

/// Verifies that a non-empty string is truthy (double negation yields "1").
/// Regression for issue #26.
#[test]
fn test_not_nonempty_string_is_false() {
    let out = compile_and_run(r#"<?php echo !!"hello";"#);
    assert_eq!(out, "1");
}

// -- Issue #27: is_numeric() should work for numeric strings --

/// Verifies that `is_numeric()` returns true for decimal digit strings.
/// Regression for issue #27.
#[test]
fn test_is_numeric_string_digits() {
    let out = compile_and_run(r#"<?php if (is_numeric("42")) { echo "yes"; } else { echo "no"; }"#);
    assert_eq!(out, "yes");
}

/// Verifies that `is_numeric()` returns true for floating-point strings.
/// Regression for issue #27.
#[test]
fn test_is_numeric_string_float() {
    let out =
        compile_and_run(r#"<?php if (is_numeric("3.14")) { echo "yes"; } else { echo "no"; }"#);
    assert_eq!(out, "yes");
}

/// Verifies that `is_numeric()` returns true for negative numeric strings.
/// Regression for issue #27.
#[test]
fn test_is_numeric_string_negative() {
    let out = compile_and_run(r#"<?php if (is_numeric("-5")) { echo "yes"; } else { echo "no"; }"#);
    assert_eq!(out, "yes");
}

/// Verifies that `is_numeric()` returns false for non-numeric strings.
/// Regression for issue #27.
#[test]
fn test_is_numeric_string_not_numeric() {
    let out =
        compile_and_run(r#"<?php if (is_numeric("abc")) { echo "yes"; } else { echo "no"; }"#);
    assert_eq!(out, "no");
}

// -- Issue #29: function_exists() should recognize builtins --

/// Verifies that `preg_split()` correctly handles the `\s+` regex pattern (whitespace splitting).
/// Regression for issue #29 (function_exists() recognizing builtins).
#[test]
fn test_preg_split_backslash_s() {
    let out = compile_and_run(
        r#"<?php
$parts = preg_split("/\s+/", "hello  world");
echo $parts[1];
"#,
    );
    assert_eq!(out, "world");
}

/// Verifies that `preg_split()` correctly handles the `\d+` regex pattern (digit splitting).
/// Regression for issue #29 (function_exists() recognizing builtins).
#[test]
fn test_preg_split_backslash_d() {
    let out = compile_and_run(
        r#"<?php
$parts = preg_split("/\d+/", "abc123def456ghi");
echo count($parts) . "|" . $parts[0] . "|" . $parts[1] . "|" . $parts[2];
"#,
    );
    assert_eq!(out, "3|abc|def|ghi");
}

/// Verifies that `preg_match()` correctly handles the `\s` regex pattern (single whitespace).
/// Regression for issue #29 (function_exists() recognizing builtins).
#[test]
fn test_preg_match_backslash_s() {
    let out = compile_and_run(r#"<?php echo preg_match("/\s/", "hello world");"#);
    assert_eq!(out, "1");
}

/// Verifies that `preg_match()` correctly handles the `\d+` regex pattern (digits).
/// Regression for issue #29 (function_exists() recognizing builtins).
#[test]
fn test_preg_match_backslash_d() {
    let out = compile_and_run(r#"<?php echo preg_match("/\d+/", "abc123");"#);
    assert_eq!(out, "1");
}

/// Verifies that `preg_match()` correctly handles the `\w+` regex pattern (word characters).
/// Regression for issue #29 (function_exists() recognizing builtins).
#[test]
fn test_preg_match_backslash_w() {
    let out = compile_and_run(r#"<?php echo preg_match("/^\w+$/", "hello_world");"#);
    assert_eq!(out, "1");
}

// --- Issue #14: hex integer literals ---

/// Verifies that lowercase hex literal `0xff` is parsed and evaluates to 255.
/// Regression for issue #14 (hex integer literals).
#[test]
fn test_hex_literal_0xff() {
    let out = compile_and_run("<?php echo 0xFF;");
    assert_eq!(out, "255");
}

/// Verifies that mixed-case hex literal `0x1a` is parsed case-insensitively and evaluates to 26.
/// Regression for issue #14 (hex integer literals).
#[test]
fn test_hex_literal_0x1a() {
    let out = compile_and_run("<?php echo 0x1A;");
    assert_eq!(out, "26");
}

/// Verifies that zero hex literal `0x0` is parsed and evaluates to 0.
/// Regression for issue #14 (hex integer literals).
#[test]
fn test_hex_literal_0x0() {
    let out = compile_and_run("<?php echo 0x0;");
    assert_eq!(out, "0");
}

/// Verifies that the `0X` prefix (uppercase X) is also accepted for hex literals.
/// Regression for issue #14 (hex integer literals).
#[test]
fn test_hex_literal_uppercase_prefix() {
    let out = compile_and_run("<?php echo 0XFF;");
    assert_eq!(out, "255");
}

/// Verifies that hex literals can be used in arithmetic expressions.
/// Regression for issue #14 (hex integer literals).
#[test]
fn test_hex_literal_arithmetic() {
    let out = compile_and_run("<?php echo 0xFF + 1;");
    assert_eq!(out, "256");
}

// --- Issue #23: modulo by zero ---

/// Verifies normal modulo behavior: `5 % 1` returns 0 (no remainder).
/// Regression for issue #23 (modulo by zero).
#[test]
fn test_modulo_normal() {
    let out = compile_and_run("<?php echo 5 % 1;");
    assert_eq!(out, "0");
}

/// Verifies that modulo by zero returns 0 (no crash).
/// Regression for issue #23 (modulo by zero).
#[test]
fn test_modulo_by_zero() {
    let out = compile_and_run("<?php echo 5 % 0;");
    assert_eq!(out, "0");
}

/// Verifies normal modulo remainder: `7 % 3` returns 1.
/// Regression for issue #23 (modulo by zero).
#[test]
fn test_modulo_normal_remainder() {
    let out = compile_and_run("<?php echo 7 % 3;");
    assert_eq!(out, "1");
}

// --- Issue #24: negative array index ---

/// Verifies that `fmod(-10, 3)` returns `-1` (negative dividend modulo).
/// Regression for issue #24 (negative array index / float modulo).
#[test]
fn test_fmod_negative_dividend() {
    let out = compile_and_run("<?php echo fmod(-10, 3);");
    assert_eq!(out, "-1");
}

/// Verifies that float modulo with a negative dividend returns a negative remainder.
/// Regression for issue #24 (negative array index / float modulo).
#[test]
fn test_float_modulo_negative() {
    let out = compile_and_run("<?php echo -10.0 % 3;");
    assert_eq!(out, "-1");
}

// --- Bug fix: string "0" is falsy ---

/// Verifies that the string `"0"` is falsy in an if statement.
/// Bug fix: string "0" is falsy.
#[test]
fn test_string_zero_falsy_if() {
    let out = compile_and_run(
        r#"<?php
if ("0") { echo "bad"; } else { echo "good"; }
"#,
    );
    assert_eq!(out, "good");
}

/// Verifies that the string `"0"` is falsy in a ternary expression.
/// Bug fix: string "0" is falsy.
#[test]
fn test_string_zero_falsy_ternary() {
    let out = compile_and_run(r#"<?php echo "0" ? "truthy" : "falsy";"#);
    assert_eq!(out, "falsy");
}

/// Verifies that negation of the string `"0"` is truthy.
/// Bug fix: string "0" is falsy.
#[test]
fn test_string_zero_falsy_not() {
    let out = compile_and_run(r#"<?php echo !"0" ? "yes" : "no";"#);
    assert_eq!(out, "yes");
}

/// Verifies that a non-empty string is truthy.
#[test]
fn test_string_nonempty_truthy() {
    let out = compile_and_run(r#"<?php echo "hello" ? "yes" : "no";"#);
    assert_eq!(out, "yes");
}

/// Verifies that an empty string is falsy.
#[test]
fn test_string_empty_falsy() {
    let out = compile_and_run(r#"<?php echo "" ? "yes" : "no";"#);
    assert_eq!(out, "no");
}

/// Verifies that `preg_match` with a not-yet-supported Mixed subject still compiles.
///
/// The EIR regex bridge does not yet coerce a non-string (Mixed) subject, so it
/// lowers that case to a runtime fatal instead of failing codegen. This keeps a
/// runtime-dead branch (here guarded by `$run = false`) compilable; the program
/// links and runs without firing the fatal. Mirrors Symfony YAML's block-scalar
/// `preg_replace('#\d+#', '', $modifiers)` path, which the parser never reaches
/// for a simple mapping.
#[test]
fn test_preg_match_mixed_subject_dead_branch_compiles() {
    let out = compile_and_run(
        r#"<?php
function maybe_match(mixed $subject, bool $run): int {
    $n = 0;
    if ($run) {
        $n = preg_match('/x/', $subject);
    }
    return $n;
}
echo maybe_match("abc", false);
"#,
    );
    assert_eq!(out, "0");
}

// --- by-ref IN-OUT params must NOT be re-typed by the out-only overwrite change ---

/// Verifies an IN-OUT by-ref call (`sort`) mutates its array argument in place without
/// destroying the caller's element type: `$a` stays `array<int>`, so `array_sum($a)`
/// after `sort($a)` still type-checks and computes the sum. This guards the OUT-only vs
/// IN-OUT distinction — sort is deliberately NOT classified out-only, so an already-defined
/// `array<int>` local keeps its type. PHP: `sort([3,1,2])` -> `[1,2,3]`, sum `6`.
#[test]
fn test_sort_in_out_preserves_array_element_type() {
    let out = compile_and_run(r#"<?php $a = [3, 1, 2]; sort($a); echo array_sum($a);"#);
    assert_eq!(out, "6");
}

/// Verifies `array_push` (IN-OUT by-ref) keeps its target's `array<int>` type so a
/// following `array_sum` still type-checks. PHP: push 4 onto `[1,2,3]`, sum `10`.
#[test]
fn test_array_push_in_out_preserves_array_element_type() {
    let out = compile_and_run(
        r#"<?php $b = [1, 2, 3]; array_push($b, 4); echo array_sum($b);"#,
    );
    assert_eq!(out, "10");
}

/// Verifies `array_shift` (IN-OUT by-ref) mutates in place and preserves the remaining
/// array's element type. PHP: shift from `[10,20,30]` yields `10`, remaining sum `50`.
#[test]
fn test_array_shift_in_out_preserves_array_element_type() {
    let out = compile_and_run(
        r#"<?php $c = [10, 20, 30]; $first = array_shift($c); echo $first, "-", array_sum($c);"#,
    );
    assert_eq!(out, "10-50");
}

// --- Bug fix: compound assignment in for-loop update ---

/// EC-2 (#485): mb_ereg_match() — start-anchored regex match, reusing the PCRE2 engine and
/// enforcing the start-anchor via regmatch[0].rm_so == 0. Verified vs PHP 8.5: it is a prefix
/// match at offset 0 (`'ab'`/`'abc'` = true, `'bc'`/`'abc'` = false); `\z` forces end-anchoring.
#[test]
fn test_mb_ereg_match_start_anchored() {
    let out = compile_and_run(
        "<?php echo (int)mb_ereg_match('ab','abc'), (int)mb_ereg_match('bc','abc'), (int)mb_ereg_match('^[A-Z][A-Za-z0-9]*$','Foo'), (int)mb_ereg_match('[a-z]+\\z','abc123');",
    );
    assert_eq!(out, "1010");
}

/// Verifies that `mb_ereg_match()` accepts the optional options argument and honors `i`.
#[test]
fn test_mb_ereg_match_options_case_insensitive() {
    let out = compile_and_run(
        "<?php echo (int)mb_ereg_match('ab','AB'), (int)mb_ereg_match('ab','AB','i'), (int)mb_ereg_match('ab','AB', null);",
    );
    assert_eq!(out, "010");
}
