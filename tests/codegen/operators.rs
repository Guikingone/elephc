//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of operators, including addition, subtraction, and multiplication.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

// --- Phase 3: Arithmetic ---

/// Verifies integer addition with literal operands: 10 + 32 = 42.
#[test]
fn test_addition() {
    let out = compile_and_run("<?php echo 10 + 32;");
    assert_eq!(out, "42");
}

/// Verifies integer subtraction with literal operands: 100 - 58 = 42.
#[test]
fn test_subtraction() {
    let out = compile_and_run("<?php echo 100 - 58;");
    assert_eq!(out, "42");
}

/// Verifies integer multiplication with literal operands: 6 * 7 = 42.
#[test]
fn test_multiplication() {
    let out = compile_and_run("<?php echo 6 * 7;");
    assert_eq!(out, "42");
}

/// Verifies integer division with literal operands: 84 / 2 = 42.
#[test]
fn test_division() {
    let out = compile_and_run("<?php echo 84 / 2;");
    assert_eq!(out, "42");
}

/// Verifies arithmetic with variables: loads two integers from memory and adds them.
#[test]
fn test_arithmetic_with_variables() {
    let out = compile_and_run("<?php $a = 10; $b = 32; echo $a + $b;");
    assert_eq!(out, "42");
}

/// Verifies operator precedence: multiplication binds tighter than addition, so 2 + 3 * 4 = 14.
#[test]
fn test_operator_precedence() {
    let out = compile_and_run("<?php echo 2 + 3 * 4;");
    assert_eq!(out, "14");
}

/// Verifies parenthesized precedence: (2 + 3) * 4 = 20, confirming parentheses override default precedence.
#[test]
fn test_parenthesized_arithmetic() {
    let out = compile_and_run("<?php echo (2 + 3) * 4;");
    assert_eq!(out, "20");
}

/// Verifies a complex expression mixing parentheses, addition, multiplication, and subtraction: (10 + 5) * 2 - 7 = 23.
#[test]
fn test_complex_expression() {
    let out = compile_and_run("<?php echo (10 + 5) * 2 - 7;");
    assert_eq!(out, "23");
}

/// Verifies assignment of an arithmetic expression result to a variable, then echo: $a + $b → $c → output.
#[test]
fn test_arithmetic_assign_and_echo() {
    let out = compile_and_run("<?php $a = 10; $b = 32; $c = $a + $b; echo $c;");
    assert_eq!(out, "42");
}

/// Verifies subtraction producing a negative result: 3 - 10 = -7, confirming signed integer handling.
#[test]
fn test_subtraction_negative_result() {
    let out = compile_and_run("<?php echo 3 - 10;");
    assert_eq!(out, "-7");
}

/// Verifies left-associative chaining of addition: 1 + 2 + 3 + 4 = 10.
#[test]
fn test_nested_arithmetic() {
    let out = compile_and_run("<?php echo 1 + 2 + 3 + 4;");
    assert_eq!(out, "10");
}

/// Verifies that adding 1 to the maximum 64-bit integer constant overflows to float at compile time.
#[test]
fn test_constant_int_add_overflow_promotes_to_float() {
    let out = compile_and_run("<?php echo gettype(9223372036854775807 + 1);");
    assert_eq!(out, "double");
}

/// Verifies that squaring a large integer constant overflows to float at compile time.
#[test]
fn test_constant_int_multiply_overflow_promotes_to_float() {
    let out = compile_and_run("<?php echo gettype(3037000500 * 3037000500);");
    assert_eq!(out, "double");
}

/// Verifies that adding 1 to the maximum 64-bit integer at runtime overflows to float.
#[test]
fn test_runtime_int_add_overflow_promotes_to_float() {
    let out = compile_and_run("<?php function add_one(int $x) { return $x + 1; } echo gettype(add_one(9223372036854775807));");
    assert_eq!(out, "double");
}

/// Verifies that subtracting past the minimum 64-bit integer at runtime overflows to float.
#[test]
fn test_runtime_int_sub_overflow_promotes_to_float() {
    let out = compile_and_run("<?php function sub_two(int $x) { return $x - 2; } echo gettype(sub_two(-9223372036854775807));");
    assert_eq!(out, "double");
}

/// Verifies that squaring a large integer at runtime overflows to float.
#[test]
fn test_runtime_int_multiply_overflow_promotes_to_float() {
    let out = compile_and_run("<?php function mul_big(int $x) { return $x * 3037000500; } echo gettype(mul_big(3037000500));");
    assert_eq!(out, "double");
}

/// Verifies that runtime integer arithmetic without overflow remains integer, not float.
#[test]
fn test_runtime_int_arithmetic_without_overflow_stays_integer() {
    let out = compile_and_run("<?php function add_small(int $x) { return $x + 2; } $v = add_small(40); echo gettype($v) . ':' . $v;");
    assert_eq!(out, "integer:42");
}

/// Verifies that a runtime overflow result (float) participates correctly in subsequent arithmetic.
#[test]
fn test_runtime_overflow_result_participates_in_later_arithmetic() {
    let out = compile_and_run("<?php function add_one(int $x) { return $x + 1; } $c = add_one(9223372036854775807); echo gettype($c + 1);");
    assert_eq!(out, "double");
}

/// Verifies that pre-increment promotes an overflowing int local and returns the promoted value.
#[test]
fn test_runtime_pre_increment_overflow_promotes_local_to_float() {
    let out = compile_and_run("<?php function pre_inc(int $x) { $y = ++$x; echo gettype($y) . ':' . gettype($x); } pre_inc(9223372036854775807);");
    assert_eq!(out, "double:double");
}

/// Verifies that post-increment returns the old int while promoting the local for later reads.
#[test]
fn test_runtime_post_increment_overflow_returns_old_int_and_promotes_local() {
    let out = compile_and_run("<?php function post_inc(int $x) { $y = $x++; echo gettype($y) . ':' . gettype($x); } post_inc(9223372036854775807);");
    assert_eq!(out, "integer:double");
}

// --- Phase 3: Concatenation ---

/// Verifies string literal concatenation: "Hello, " . "World!" = "Hello, World!".
#[test]
fn test_concat_literals() {
    let out = compile_and_run("<?php echo \"Hello, \" . \"World!\";");
    assert_eq!(out, "Hello, World!");
}

/// Verifies string concatenation with variables: loads two strings from memory and concatenates.
#[test]
fn test_concat_variables() {
    let out = compile_and_run("<?php $a = \"Hello, \"; $b = \"World!\"; echo $a . $b;");
    assert_eq!(out, "Hello, World!");
}

/// Verifies left-associative chaining of string concatenation: "a" . "b" . "c" = "abc".
#[test]
fn test_concat_chain() {
    let out = compile_and_run("<?php echo \"a\" . \"b\" . \"c\";");
    assert_eq!(out, "abc");
}

/// Verifies concatenation assignment: $msg = "foo" . "bar"; echo $msg; = "foobar".
#[test]
fn test_concat_assign() {
    let out = compile_and_run("<?php $msg = \"foo\" . \"bar\"; echo $msg;");
    assert_eq!(out, "foobar");
}

/// Verifies concatenation with embedded newline escape: "hello" . "\n" outputs "hello\n".
#[test]
fn test_concat_with_newline() {
    let out = compile_and_run("<?php echo \"hello\" . \"\\n\";");
    assert_eq!(out, "hello\n");
}

/// Verifies that concatenating an array onto a string stringifies the array to the literal
/// "Array" (matching PHP's array-to-string conversion) for both an array literal and an
/// array-typed function result, instead of crashing by treating the array pointer as a string.
#[test]
fn test_concat_array_stringifies_to_array_literal() {
    let out = compile_and_run(
        r#"<?php
function makeArr() { return [1, 2, 3]; }
echo "a" . [4, 5];
echo "|";
echo "prefix" . makeArr();
"#,
    );
    assert_eq!(out, "aArray|prefixArray");
}

/// Verifies that echoing an array stringifies to the literal "Array" (matching PHP), routing
/// through the same string-coercion path as concatenation.
#[test]
fn test_echo_array_stringifies_to_array_literal() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; echo $a;");
    assert_eq!(out, "Array");
}

/// Verifies that interpolating an array into a double-quoted string stringifies it to the
/// literal "Array" (matching PHP) for both simple `$a` and complex `{$a}` interpolation.
#[test]
fn test_interpolated_array_stringifies_to_array_literal() {
    let out = compile_and_run("<?php $a = [1, 2, 3]; echo \"v=$a|w={$a}\";");
    assert_eq!(out, "v=Array|w=Array");
}

// --- Phase 3: Mixed-type concatenation ---

/// Verifies concatenation of string literal and integer literal: "Value: " . 42 = "Value: 42".
#[test]
fn test_concat_string_and_int() {
    let out = compile_and_run("<?php echo \"Value: \" . 42;");
    assert_eq!(out, "Value: 42");
}

/// Verifies concatenation of integer literal and string literal: 42 . " is the answer" = "42 is the answer".
#[test]
fn test_concat_int_and_string() {
    let out = compile_and_run("<?php echo 42 . \" is the answer\";");
    assert_eq!(out, "42 is the answer");
}

/// Verifies concatenation of two integer literals coerces to string: 1 . 2 = "12".
#[test]
fn test_concat_int_and_int() {
    let out = compile_and_run("<?php echo 1 . 2;");
    assert_eq!(out, "12");
}

/// Verifies concatenation of a string literal and a parenthesized expression result: "Result: " . ($a + $b) = "Result: 42".
#[test]
fn test_concat_expr_result() {
    let out = compile_and_run("<?php $a = 10; $b = 32; echo \"Result: \" . ($a + $b);");
    assert_eq!(out, "Result: 42");
}

/// Verifies mixed-type concatenation chaining left-to-right: "x=" . 5 . " y=" . 10 = "x=5 y=10".
#[test]
fn test_concat_chain_mixed() {
    let out = compile_and_run("<?php echo \"x=\" . 5 . \" y=\" . 10;");
    assert_eq!(out, "x=5 y=10");
}

/// Verifies concatenation with a negative integer: "num: " . -7 = "num: -7".
#[test]
fn test_concat_negative_int() {
    let out = compile_and_run("<?php echo \"num: \" . -7;");
    assert_eq!(out, "num: -7");
}

// --- Modulo ---

/// Verifies integer modulo: 10 % 3 = 1.
#[test]
fn test_modulo() {
    let out = compile_and_run("<?php echo 10 % 3;");
    assert_eq!(out, "1");
}

/// Verifies modulo with zero remainder: 15 % 5 = 0.
#[test]
fn test_modulo_zero_remainder() {
    let out = compile_and_run("<?php echo 15 % 5;");
    assert_eq!(out, "0");
}

// --- Comparison operators ---

/// Verifies loose equality comparison returning true: 1 == 1 outputs "1".
#[test]
fn test_equal_true() {
    let out = compile_and_run("<?php echo 1 == 1;");
    assert_eq!(out, "1");
}

/// Verifies loose equality comparison returning false: 1 == 2 outputs empty string (echo false prints nothing in PHP).
#[test]
fn test_equal_false() {
    let out = compile_and_run("<?php echo 1 == 2;");
    assert_eq!(out, ""); // echo false prints nothing in PHP
}

/// Verifies loose inequality returning true: 1 != 2 outputs "1".
#[test]
fn test_not_equal() {
    let out = compile_and_run("<?php echo 1 != 2;");
    assert_eq!(out, "1");
}

// --- Loose comparison across types ---

/// Verifies loose equality at compile time: empty string equals false, var_dump shows bool(true).
#[test]
fn test_loose_eq_empty_string_false() {
    let out = compile_and_run("<?php var_dump(\"\" == false);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies loose equality at compile time: integer 0 equals false, var_dump shows bool(true).
#[test]
fn test_loose_eq_zero_false() {
    let out = compile_and_run("<?php var_dump(0 == false);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies loose equality at compile time: integer 1 equals true, var_dump shows bool(true).
#[test]
fn test_loose_eq_one_true() {
    let out = compile_and_run("<?php var_dump(1 == true);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies loose equality at compile time: string "0" equals false (string zero is falsy), var_dump shows bool(true).
#[test]
fn test_loose_eq_string_vs_int() {
    let out = compile_and_run("<?php var_dump(\"0\" == false);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies loose inequality at compile time: empty string is not equal to true, var_dump shows bool(true).
#[test]
fn test_loose_neq_empty_string_true() {
    let out = compile_and_run("<?php var_dump(\"\" != true);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies loose equality at compile time: null equals false (null is falsy), var_dump shows bool(true).
#[test]
fn test_loose_eq_null_false() {
    let out = compile_and_run("<?php var_dump(null == false);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies compile-time loose equality of two non-numeric strings compares by byte sequence, not lexicographically.
#[test]
fn test_constant_loose_eq_non_numeric_strings_compare_by_bytes() {
    let out = compile_and_run("<?php var_dump(\"abc\" == \"def\");");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies compile-time loose equality of numeric strings ("0" == "00") compares numerically as equal.
#[test]
fn test_constant_loose_eq_numeric_strings_compare_numerically() {
    let out = compile_and_run("<?php var_dump(\"0\" == \"00\");");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies compile-time loose equality of number and non-numeric string is false: 0 == "abc" is bool(false).
#[test]
fn test_constant_loose_eq_number_and_non_numeric_string_is_false() {
    let out = compile_and_run("<?php var_dump(0 == \"abc\");");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies compile-time loose equality of number and numeric string is true: 10 == "1e1" both evaluate to 10.0.
#[test]
fn test_constant_loose_eq_number_and_numeric_string_is_true() {
    let out = compile_and_run("<?php var_dump(10 == \"1e1\");");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies runtime float comparisons against NaN match PHP: NaN is uncomparable, so `<`, `<=`,
/// `>`, `>=`, `==` are all false and `!=` is true, while `<=>` yields 1 in every direction
/// (including NaN<=>NaN). Operands come from `float`-returning calls so the optimizer cannot
/// constant-fold them, exercising the runtime comparison codegen rather than the folder.
#[test]
fn test_runtime_nan_comparisons() {
    let out = compile_and_run(
        r#"<?php
function nan_val(): float { return NAN; }
function one_val(): float { return 1.0; }
$nan = nan_val();
$one = one_val();
var_dump($nan < $one);
var_dump($nan <= $one);
var_dump($nan > $one);
var_dump($nan >= $one);
var_dump($nan == $one);
var_dump($nan != $one);
echo ($nan <=> $one), ($one <=> $nan), ($nan <=> $nan);
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\n111"
    );
}

/// Verifies runtime loose equality of two non-numeric strings compares by byte sequence.
#[test]
fn test_runtime_loose_eq_non_numeric_strings_compare_by_bytes() {
    let out = compile_and_run("<?php $a = \"abc\"; $b = \"def\"; var_dump($a == $b);");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies runtime loose equality of numeric strings "0" == "00" compares numerically as equal.
#[test]
fn test_runtime_loose_eq_numeric_strings_compare_numerically() {
    let out = compile_and_run("<?php $a = \"0\"; $b = \"00\"; var_dump($a == $b);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies runtime loose equality of number and non-numeric string is false: $n=0, $s="abc" → bool(false).
#[test]
fn test_runtime_loose_eq_number_and_non_numeric_string_is_false() {
    let out = compile_and_run("<?php $n = 0; $s = \"abc\"; var_dump($n == $s);");
    assert_eq!(out, "bool(false)\n");
}

/// Verifies runtime loose equality of number and numeric string is true: $n=10, $s="1e1" → bool(true).
#[test]
fn test_runtime_loose_eq_number_and_numeric_string_is_true() {
    let out = compile_and_run("<?php $n = 10; $s = \"1e1\"; var_dump($n == $s);");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies runtime loose equality of bool and string uses truthiness: true=="abc" is true (truthy), false=="abc" is false.
#[test]
fn test_runtime_loose_eq_bool_and_string_uses_truthiness() {
    let out = compile_and_run("<?php $s = \"abc\"; var_dump(true == $s); var_dump(false == $s);");
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// Verifies runtime loose equality of null and string uses empty-string rule: null=="" is true, null=="0" is false.
#[test]
fn test_runtime_loose_eq_null_and_string_uses_empty_string_rule() {
    let out = compile_and_run("<?php $empty = \"\"; $zero = \"0\"; var_dump(null == $empty); var_dump(null == $zero);");
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// Verifies integer less-than comparison: 1 < 2 outputs "1".
#[test]
fn test_less_than() {
    let out = compile_and_run("<?php echo 1 < 2;");
    assert_eq!(out, "1");
}

/// Verifies integer greater-than comparison: 2 > 1 outputs "1".
#[test]
fn test_greater_than() {
    let out = compile_and_run("<?php echo 2 > 1;");
    assert_eq!(out, "1");
}

/// Verifies integer less-than-or-equal comparison: 2 <= 2 outputs "1".
#[test]
fn test_less_equal() {
    let out = compile_and_run("<?php echo 2 <= 2;");
    assert_eq!(out, "1");
}

/// Verifies integer greater-than-or-equal comparison: 1 >= 2 outputs empty string (false).
#[test]
fn test_greater_equal() {
    let out = compile_and_run("<?php echo 1 >= 2;");
    assert_eq!(out, "");
}

/// Regression: a loose `==` between a plain integer and a boxed `Mixed` integer must hold in both
/// operand orders. Loading a Mixed operand unboxes it through a runtime call that clobbers the
/// scratch registers; without saving the already-loaded left operand, `Int == Mixed` lost its left
/// value and compared wrong, while `Mixed == Int` happened to work. The Mixed here comes from a
/// heterogeneous associative array element.
#[test]
fn test_loose_eq_int_and_mixed_both_orders() {
    let out = compile_and_run(
        r#"<?php
$h = ["n" => 100, "s" => "x"];
$m = $h["n"];
$i = 100;
echo ($i == $m ? "y" : "n"), ($m == $i ? "y" : "n"), ($i == $h["n"] ? "y" : "n"),
     ($i == 101 ? "y" : "n");
"#,
    );
    assert_eq!(out, "yyyn");
}

// --- PHP 8 ordered comparison of string/Mixed operands (`__rt_php_compare`) ---

/// Two numeric strings compare numerically, not lexicographically: `"10" < "9"` is false
/// because 10 is not less than 9 (a byte comparison would wrongly report `'1' < '9'`). The
/// string parameters keep the operands non-constant so the comparison reaches the runtime.
#[test]
fn test_str_lt_numeric_strings_compare_numerically() {
    let out = compile_and_run(
        r#"<?php
function cmp(string $a, string $b): string { return ($a < $b) ? "lt" : "ge"; }
echo cmp("10", "9"), "|", cmp("9", "10");
"#,
    );
    assert_eq!(out, "ge|lt");
}

/// Non-numeric strings compare lexicographically: `"abc" < "abd"` is true (`c` < `d`).
#[test]
fn test_str_lt_non_numeric_strings_compare_lexicographically() {
    let out = compile_and_run(
        r#"<?php
function cmp(string $a, string $b): string { return ($a < $b) ? "lt" : "ge"; }
echo cmp("abc", "abd"), "|", cmp("abd", "abc");
"#,
    );
    assert_eq!(out, "lt|ge");
}

/// A string vs int comparison follows PHP 8: a numeric string compares numerically (`"3" < 5`
/// is true), while a non-numeric string forces lexicographic comparison against the
/// stringified int (`"abc" < 5` compares `"abc"` with `"5"`, so it is false).
#[test]
fn test_str_lt_int_php8_semantics() {
    let out = compile_and_run(
        r#"<?php
function f(string $s): string { return ($s < 5) ? "lt" : "ge"; }
echo f("3"), "|", f("abc");
"#,
    );
    assert_eq!(out, "lt|ge");
}

/// Regression for the Symfony YAML blocker: `$scalar < PHP_INT_MIN` on a (numeric) string
/// operand must compare numerically. `"-9.99e18"` is less than `PHP_INT_MIN` (~-9.22e18).
#[test]
fn test_str_lt_php_int_min_numeric() {
    let out = compile_and_run(
        r#"<?php
function below_min(string $s): string { return ($s < PHP_INT_MIN) ? "below" : "ok"; }
echo below_min("-9.99e18"), "|", below_min("0");
"#,
    );
    assert_eq!(out, "below|ok");
}

/// A boxed `Mixed` operand (heterogeneous associative-array element) ordered against a string
/// goes through the same PHP 8 comparator: `"10"` vs `"9"` compares numerically.
#[test]
fn test_mixed_lt_str_numeric() {
    let out = compile_and_run(
        r#"<?php
$h = ["v" => "10", "s" => "x"];
echo ($h["v"] < "9") ? "lt" : "ge";
"#,
    );
    assert_eq!(out, "ge");
}

/// The concrete integer comparison fast path is unaffected: `1 < 2` is still true and
/// `3 < 2` is still false.
#[test]
fn test_int_lt_fast_path_unchanged() {
    let out = compile_and_run(
        r#"<?php
function ci(int $a, int $b): string { return ($a < $b) ? "lt" : "ge"; }
echo ci(1, 2), "|", ci(3, 2);
"#,
    );
    assert_eq!(out, "lt|ge");
}

/// The spaceship operator over string operands returns the PHP 8 three-way sign directly:
/// numeric strings compare numerically (`"10" <=> "9"` is 1), non-numeric strings
/// lexicographically (`"abc" <=> "abd"` is -1).
#[test]
fn test_spaceship_str_php8_sign() {
    let out = compile_and_run(
        r#"<?php
function sp(string $a, string $b): int { return $a <=> $b; }
echo sp("10", "9"), "|", sp("abc", "abd"), "|", sp("5", "5");
"#,
    );
    assert_eq!(out, "1|-1|0");
}

/// Verifies PHP 8 loose equality (`==`/`!=`) between a string and a Mixed operand, matching
/// `$a == $b` ⟺ `($a <=> $b) === 0`: numeric strings compare numerically, non-numeric
/// strings lexicographically, and `"" == null` is true. Regression for the symfony/yaml
/// `loose_eq for PHP types Str and Mixed` backend gap.
#[test]
fn test_string_vs_mixed_loose_equality() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n): mixed {
    if ($n === 1) return "1";
    if ($n === 2) return 5;
    if ($n === 3) return "abc";
    return null;
}
$m1 = pick(1); $m2 = pick(2); $m3 = pick(3); $m4 = pick(4);
echo ("1" == $m1) ? "T" : "F";
echo ("5" == $m2) ? "T" : "F";
echo ("abc" == $m3) ? "T" : "F";
echo ("x" == $m3) ? "T" : "F";
echo ("" == $m4) ? "T" : "F";
echo ("abc" != $m2) ? "T" : "F";
"#,
    );
    assert_eq!(out, "TTTFTT");
}

// --- PHP string bitwise operators (`&`/`|`/`^` bytewise on two strings) ---

/// Verifies `&` on two multi-byte string literals is bytewise (`min` length,
/// per-byte AND). `bin2hex("ABCD" & "\xff\x00\xff\x00")` == "41004300" in PHP.
#[test]
fn test_string_bitwise_and_multibyte() {
    let out = compile_and_run(r#"<?php echo bin2hex("ABCD" & "\xff\x00\xff\x00");"#);
    assert_eq!(out, "41004300");
}

/// Verifies `&` truncates to the shorter (right) operand's length: `"ABCD" & "\x0f"`
/// yields a single byte `0x41 & 0x0f == 0x01`, so strlen==1 and bin2hex=="01".
#[test]
fn test_string_bitwise_and_shorter_right() {
    let out = compile_and_run(r#"<?php echo strlen("ABCD" & "\x0f"), ":", bin2hex("ABCD" & "\x0f");"#);
    assert_eq!(out, "1:01");
}

/// Verifies `&` against an empty string produces an empty (len 0) string without crashing.
#[test]
fn test_string_bitwise_and_empty() {
    let out = compile_and_run(r#"<?php echo strlen("AB" & "");"#);
    assert_eq!(out, "0");
}

/// Verifies `|` uses `max` length and copies the longer operand's tail verbatim:
/// `bin2hex("AB" | "\x00\x00\x01\x02")` == "41420102" (overlap OR then tail 01 02).
#[test]
fn test_string_bitwise_or_tail_copy() {
    let out = compile_and_run(r#"<?php echo bin2hex("AB" | "\x00\x00\x01\x02");"#);
    assert_eq!(out, "41420102");
}

/// Verifies `^` is bytewise over the `min` length: `bin2hex("ABCD" ^ "\x01\x01")` == "4043".
#[test]
fn test_string_bitwise_xor_min() {
    let out = compile_and_run(r#"<?php echo bin2hex("ABCD" ^ "\x01\x01");"#);
    assert_eq!(out, "4043");
}

/// Verifies the real-world polyfill pattern: a length-1 `&` used as an array key.
/// `$s[0]` is `"\xF5"`, `"\xF5" & "\xF0"` is `"\xF0"`, so `$m["\xF0"]` is `3`.
#[test]
fn test_string_bitwise_polyfill_pattern() {
    let out = compile_and_run(
        r#"<?php $s="\xF5x"; $m=["\xF0"=>3]; echo $m[$s[0] & "\xF0"] ?? 0;"#,
    );
    assert_eq!(out, "3");
}

/// Verifies the self-alias case `$a & $a` does not double-free and produces the operand
/// bytes unchanged: `bin2hex("AB" & "AB")` == "4142".
#[test]
fn test_string_bitwise_self_alias() {
    let out = compile_and_run(r#"<?php $a="AB"; echo bin2hex($a & $a);"#);
    assert_eq!(out, "4142");
}

/// Verifies the runtime path (non-literal operands, so no constant folding) matches PHP
/// for both `&` (min length) and `|` (max length with tail copy).
#[test]
fn test_string_bitwise_runtime_operands() {
    let out = compile_and_run(
        r#"<?php $x="ABCD"; $y="\xff\x00\xff\x00"; $p="AB"; $q="\x00\x00\x01\x02";
echo bin2hex($x & $y), ":", bin2hex($p | $q);"#,
    );
    assert_eq!(out, "41004300:41420102");
}

/// Verifies that a non-both-string bitwise operator is unaffected: integer `&`/`|`/`^`
/// still take the integer path (6 & 3 == 2, 5 | 2 == 7, 5 ^ 1 == 4).
#[test]
fn test_integer_bitwise_unaffected_by_string_path() {
    let out = compile_and_run("<?php echo (6 & 3), ':', (5 | 2), ':', (5 ^ 1);");
    assert_eq!(out, "2:7:4");
}

// --- runtime-polymorphic Mixed bitwise (`Op::MixedBitwise` / `__rt_mixed_bitwise`) ---

/// A dynamic `Mixed` operand (a `mixed`-returning call whose runtime payload is a
/// string) `&` a string literal must take PHP's bytewise-string path at runtime:
/// `"\xF5" & "\xF0"` is `"\xF0"`. This is the QuestionHelper UTF-8 detection shape.
#[test]
fn test_mixed_bitwise_string_and_literal_bytewise() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return "\xF5"; } $x = m(); echo bin2hex($x & "\xF0");"#,
    );
    assert_eq!(out, "f0");
}

/// Both operands dynamic strings must be bytewise (`&`/`|`/`^`), and the results
/// match PHP: `"AB" & "AB"` == "AB", `"AB" | "  "` == "ab", `"AB" ^ "AB"` == 0000.
#[test]
fn test_mixed_bitwise_both_dynamic_strings() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return "AB"; }
$a = m(); $b = m();
echo ($a & $b), ":", ($a | "  "), ":", bin2hex($a ^ $b);"#,
    );
    assert_eq!(out, "AB:ab:0000");
}

/// A dynamic `Mixed` operand holding an int opposite a string literal is NOT both
/// strings, so PHP uses the integer path with string→int coercion: `5 & "3"` == 1,
/// `5 | "2"` == 7, `5 ^ "1"` == 4.
#[test]
fn test_mixed_bitwise_int_payload_vs_string_uses_int_path() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return 5; }
$x = m();
echo ($x & "3"), ":", ($x | "2"), ":", ($x ^ "1");"#,
    );
    assert_eq!(out, "1:7:4");
}

/// The Mixed bitwise result must be released on reassignment/discard: a tight loop
/// building bytewise-string results must not leak (verified here as a correctness
/// smoke — the heap-debug leak check runs in the manual verification).
#[test]
fn test_mixed_bitwise_loop_result_correct() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return "\xF5\xAA"; }
$acc = 0;
for ($i = 0; $i < 100; $i++) { $r = m() & "\xF0\x0F"; $acc += strlen($r); }
echo $acc;"#,
    );
    assert_eq!(out, "200");
}

// --- runtime-polymorphic unary Mixed bitwise NOT (`Op::MixedBitwiseNot` / `__rt_mixed_bitwise_not`) ---

/// A dynamic `Mixed` operand holding a string must take PHP's bytewise-NOT path at runtime:
/// `~"A"` is `"\xBE"` (each byte complemented), `~"AB"` is `"\xBE\xBD"`.
#[test]
fn test_mixed_bitwise_not_string_bytewise() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return "A"; } function n(): mixed { return "AB"; }
$a = m(); $b = n();
echo bin2hex(~$a), ":", bin2hex(~$b);"#,
    );
    assert_eq!(out, "be:bebd");
}

/// A dynamic `Mixed` operand holding an int must take the integer NOT path: `~5` == -6,
/// `~0` == -1, matching PHP's two's-complement bitwise NOT.
#[test]
fn test_mixed_bitwise_not_int_path() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return 5; } function z(): mixed { return 0; }
$a = m(); $b = z();
echo (~$a), ":", (~$b);"#,
    );
    assert_eq!(out, "-6:-1");
}

/// A concrete integer operand still takes the fast `Op::IBitNot` path (`~5` == -6), unaffected
/// by the runtime-polymorphic routing.
#[test]
fn test_int_bitwise_not_unaffected_by_mixed_path() {
    let out = compile_and_run("<?php $x = 5; echo ~$x, ':', ~0;");
    assert_eq!(out, "-6:-1");
}

/// The Mixed bitwise NOT result must be released on reassignment/discard: a tight loop
/// building bytewise-NOT string results must not leak (verified here as a correctness smoke).
#[test]
fn test_mixed_bitwise_not_loop_result_correct() {
    let out = compile_and_run(
        r#"<?php function m(): mixed { return "\xF5\xAA"; }
$acc = 0;
for ($i = 0; $i < 100; $i++) { $r = ~m(); $acc += strlen($r); }
echo $acc;"#,
    );
    assert_eq!(out, "200");
}

/// Regression for #397: loose equality with a Mixed operand holding a float
/// must not truncate the float to int before comparison. `1.5 == 1` must be
/// false, `1.5 == 1.5` must be true.
#[test]
fn test_loose_eq_mixed_float_vs_int() {
    let out = compile_and_run(
        r#"<?php
function check($m) {
    var_dump($m == 1);
    var_dump($m == 1.5);
    var_dump($m == 2);
}
check(1.5);
"#,
    );
    assert_eq!(out, "bool(false)\nbool(true)\nbool(false)\n");
}

/// Regression for #397: switch with a Mixed subject holding a float must use
/// loose equality, not integer truncation. `switch(1.5) { case 1: ...; case
/// 1.5: ... }` must match `case 1.5`.
#[test]
fn test_switch_mixed_float_subject() {
    let out = compile_and_run(
        r#"<?php
function classify($x) {
    switch ($x) {
        case 1:   return "int-one";
        case 1.5: return "onefive";
        default:  return "other";
    }
}
echo classify(1.5), "\n";
"#,
    );
    assert_eq!(out, "onefive\n");
}

/// Regression for #397: switch with a Mixed subject holding an int must still
/// match int cases correctly (no regression from the Mixed routing change).
#[test]
fn test_switch_mixed_int_subject() {
    let out = compile_and_run(
        r#"<?php
function classify($x) {
    switch ($x) {
        case 1:   return "int-one";
        case 1.5: return "onefive";
        default:  return "other";
    }
}
echo classify(1), "\n";
echo classify(2), "\n";
"#,
    );
    assert_eq!(out, "int-one\nother\n");
}

/// Regression for #397: `!=` (LooseNotEq) with a Mixed float operand must
/// also avoid truncation. `1.5 != 1` must be true.
#[test]
fn test_loose_neq_mixed_float_vs_int() {
    let out = compile_and_run(
        r#"<?php
function check($m) {
    var_dump($m != 1);
    var_dump($m != 1.5);
}
check(1.5);
"#,
    );
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// Regression: loose equality with a Mixed NaN payload must preserve PHP's
/// unordered-float rule. `NAN == 1` is false and `NAN != 1` is true, including
/// on x86_64 where unordered `ucomisd` comparisons set ZF.
#[test]
fn test_loose_eq_mixed_nan_vs_int() {
    let out = compile_and_run(
        r#"<?php
function check($m) {
    var_dump($m == 1);
    var_dump($m != 1);
}
check(NAN);
"#,
    );
    assert_eq!(out, "bool(false)\nbool(true)\n");
}

/// Regression: loose equality with a Mixed string payload must use PHP
/// numeric-string rules instead of `atof`-style casts. Non-numeric strings are
/// not equal to numbers, while numeric strings compare by parsed numeric value.
#[test]
fn test_loose_eq_mixed_string_vs_number_uses_numeric_string_rules() {
    let out = compile_and_run(
        r#"<?php
function check($m) {
    var_dump($m == 0);
    var_dump($m == 0.0);
    var_dump($m != 0);
    var_dump($m == 1.5);
}
check("abc");
check("1.5");
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(false)\nbool(true)\nbool(false)\nbool(false)\nbool(false)\nbool(true)\nbool(true)\n"
    );
}

/// Regression: loose equality between a Mixed boolean and a number compares by
/// PHP truthiness, not by comparing `true` as `1.0`.
#[test]
fn test_loose_eq_mixed_bool_vs_number_uses_truthiness() {
    let out = compile_and_run(
        r#"<?php
function check_true($m) {
    var_dump($m == 2);
    var_dump($m == 0.5);
    var_dump($m == 0);
}
function check_false($m) {
    var_dump($m == 0.0);
    var_dump($m == 1);
}
check_true(true);
check_false(false);
"#,
    );
    assert_eq!(
        out,
        "bool(true)\nbool(true)\nbool(false)\nbool(true)\nbool(false)\n"
    );
}

/// Regression: Mixed array payloads are not loosely equal to numeric operands.
#[test]
fn test_loose_eq_mixed_array_vs_number_is_false() {
    let out = compile_and_run(
        r#"<?php
function check($m) {
    var_dump($m == 0);
    var_dump($m != 1.0);
}
check([]);
check([1]);
"#,
    );
    assert_eq!(out, "bool(false)\nbool(true)\nbool(false)\nbool(true)\n");
}
