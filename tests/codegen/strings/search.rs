//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of strings search, including substr basic, substr with length, and substr negative offset.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies substr extracts the suffix starting at a positive offset.
/// Fixture: "Hello World" with offset 6 returns "World".
#[test]
fn test_substr_basic() {
    let out = compile_and_run(r#"<?php echo substr("Hello World", 6);"#);
    assert_eq!(out, "World");
}

/// Verifies substr respects a length parameter to limit the extraction.
/// Fixture: "Hello World" with offset 0 and length 5 returns "Hello".
#[test]
fn test_substr_with_length() {
    let out = compile_and_run(r#"<?php echo substr("Hello World", 0, 5);"#);
    assert_eq!(out, "Hello");
}

/// Verifies substr interprets a negative offset as distance from the end of the string.
/// Fixture: "Hello World" with offset -5 returns "World".
#[test]
fn test_substr_negative_offset() {
    let out = compile_and_run(r#"<?php echo substr("Hello World", -5);"#);
    assert_eq!(out, "World");
}

/// Verifies substr accepts a non-negative integer offset derived from a function return via addition.
/// Regression test: int-to-integer coercion path for the offset expression `$o + 1`.
/// Fixture: queries with `?` delimiter, strpos + intval, then substr with +1 offset.
#[test]
fn test_substr_coerces_mixed_numeric_offset_from_function_return_add() {
    let out = compile_and_run(
        r#"<?php
function get_index(string $s): int {
    $p = strpos($s, "?");
    return intval($p);
}
function slice_after(string $s): string {
    $o = get_index($s);
    $p = $o + 1;
    return substr($s, $p);
}
echo slice_after("/hello?name=elephc"), "\n";
echo substr("/hello?name=elephc", get_index("/hello?name=elephc") + 1), "\n";
"#,
    );
    assert_eq!(out, "name=elephc\nname=elephc\n");
}

/// Verifies strpos returns the integer byte offset when the needle is found.
/// Fixture: "Hello World" contains "World" starting at offset 6.
#[test]
fn test_strpos_found() {
    let out = compile_and_run(r#"<?php echo strpos("Hello World", "World");"#);
    assert_eq!(out, "6");
}

/// Verifies strpos returns empty string when the needle is absent.
/// Fixture: "Hello" does not contain "xyz".
#[test]
fn test_strpos_not_found() {
    let out = compile_and_run(r#"<?php echo strpos("Hello", "xyz");"#);
    assert_eq!(out, "");
}

/// Verifies strpos uses strict `=== false` comparison when the needle is not found.
/// Fixture: strpos on "Hello"/"xyz" is strict-false, not just falsy.
#[test]
fn test_strpos_not_found_is_strict_false() {
    let out = compile_and_run(r#"<?php echo strpos("Hello", "xyz") === false ? "miss" : "hit";"#);
    assert_eq!(out, "miss");
}

/// Verifies assignment of strpos result to a variable preserves strict-false semantics.
/// Fixture: `$pos = strpos(...)` then strict comparison against false.
#[test]
fn test_strpos_assigned_not_found_is_strict_false() {
    let out = compile_and_run(
        r#"<?php
$pos = strpos("Hello", "xyz");
echo $pos === false ? "miss" : "hit";
"#,
    );
    assert_eq!(out, "miss");
}

/// Verifies strpos returns 0 (not false) when the needle is at the start of the string.
/// Regression: zero is a valid offset and must not be confused with the false sentinel.
/// Fixture: "abc" contains "a" at offset 0, which is !== false.
#[test]
fn test_strpos_zero_offset_is_not_false() {
    let out = compile_and_run(r#"<?php echo strpos("abc", "a") === false ? "miss" : "zero";"#);
    assert_eq!(out, "zero");
}

/// Verifies strrpos finds the last occurrence of a needle.
/// Fixture: "abcabc" last "bc" starts at offset 4.
#[test]
fn test_strrpos() {
    let out = compile_and_run(r#"<?php echo strrpos("abcabc", "bc");"#);
    assert_eq!(out, "4");
}

/// Verifies strrpos returns strict false when the needle is absent.
/// Fixture: "abcabc" does not contain "zz".
#[test]
fn test_strrpos_not_found_is_strict_false() {
    let out = compile_and_run(r#"<?php echo strrpos("abcabc", "zz") === false ? "miss" : "hit";"#);
    assert_eq!(out, "miss");
}

/// Verifies strstr returns the portion of the string starting from the first needle occurrence.
/// Fixture: "user@example.com" split on "@" yields "@example.com".
#[test]
fn test_strstr_found() {
    let out = compile_and_run(r#"<?php echo strstr("user@example.com", "@");"#);
    assert_eq!(out, "@example.com");
}

/// Verifies strcmp returns 0 when two identical strings compare equal.
#[test]
fn test_strcmp_equal() {
    let out = compile_and_run(r#"<?php echo strcmp("abc", "abc");"#);
    assert_eq!(out, "0");
}

/// Verifies strcmp returns a negative value when the first string sorts before the second.
/// Fixture: "abc" < "abd" lexicographically.
#[test]
fn test_strcmp_less() {
    let out = compile_and_run(r#"<?php echo (strcmp("abc", "abd") < 0 ? "yes" : "no");"#);
    assert_eq!(out, "yes");
}

/// Verifies strcasecmp performs case-insensitive string comparison, returning 0 for equal strings.
#[test]
fn test_strcasecmp() {
    let out = compile_and_run(r#"<?php echo strcasecmp("Hello", "hello");"#);
    assert_eq!(out, "0");
}

/// Verifies str_contains returns 1 when the needle is present in the haystack.
/// Fixture: "Hello World" contains "World".
#[test]
fn test_str_contains_true() {
    let out = compile_and_run(r#"<?php echo str_contains("Hello World", "World");"#);
    assert_eq!(out, "1");
}

/// Verifies str_contains returns empty string when the needle is absent.
/// Fixture: "Hello" does not contain "xyz".
#[test]
fn test_str_contains_false() {
    let out = compile_and_run(r#"<?php echo str_contains("Hello", "xyz");"#);
    assert_eq!(out, "");
}

/// Verifies str_starts_with returns 1 when the haystack starts with the needle.
/// Fixture: "Hello World" starts with "Hello".
#[test]
fn test_str_starts_with_true() {
    let out = compile_and_run(r#"<?php echo str_starts_with("Hello World", "Hello");"#);
    assert_eq!(out, "1");
}

/// Verifies str_starts_with returns empty string when the haystack does not start with the needle.
/// Fixture: "Hello" does not start with "World".
#[test]
fn test_str_starts_with_false() {
    let out = compile_and_run(r#"<?php echo str_starts_with("Hello", "World");"#);
    assert_eq!(out, "");
}

/// Verifies str_ends_with returns 1 when the haystack ends with the needle.
/// Fixture: "Hello World" ends with "World".
#[test]
fn test_str_ends_with_true() {
    let out = compile_and_run(r#"<?php echo str_ends_with("Hello World", "World");"#);
    assert_eq!(out, "1");
}

/// Verifies str_ends_with returns empty string when the haystack does not end with the needle.
/// Fixture: "Hello" does not end with "xyz".
#[test]
fn test_str_ends_with_false() {
    let out = compile_and_run(r#"<?php echo str_ends_with("Hello", "xyz");"#);
    assert_eq!(out, "");
}

/// Verifies substr_replace replaces a substring at a given offset and length with the replacement string.
/// Fixture: "hello world" replaced at offset 6, length 5 with "PHP" yields "hello PHP".
#[test]
fn test_substr_replace() {
    let out = compile_and_run(r#"<?php echo substr_replace("hello world", "PHP", 6, 5);"#);
    assert_eq!(out, "hello PHP");
}

/// Verifies substr_replace replaces from offset to end of string when length is omitted.
/// Fixture: "hello world" replaced at offset 5 with "!" yields "hello!".
#[test]
fn test_substr_replace_no_length() {
    let out = compile_and_run(r#"<?php echo substr_replace("hello world", "!", 5);"#);
    assert_eq!(out, "hello!");
}

/// Verifies `substr_count()` counts non-overlapping occurrences.
/// `LC_ALL=C php` prints `2` for both `substr_count("hello world", "o")` and
/// `substr_count("aaaa", "aa")` — matches never overlap.
#[test]
fn test_substr_count_non_overlapping() {
    let out = compile_and_run(
        r#"<?php echo substr_count("hello world", "o"), "|", substr_count("aaaa", "aa"), "|", substr_count("hello", "z");"#,
    );
    assert_eq!(out, "2|2|0");
}

/// Verifies `substr_count()` honours the `$offset` argument, including a negative offset
/// measured back from the subject end. `LC_ALL=C php` prints `1` for both forms.
#[test]
fn test_substr_count_offset() {
    let out = compile_and_run(
        r#"<?php echo substr_count("hello world", "o", 5), "|", substr_count("hello world", "o", -5);"#,
    );
    assert_eq!(out, "1|1");
}

/// Verifies `substr_count()` honours `$length`, including the negative form measured back
/// from the subject end, and treats an explicit `null` like an omitted argument.
/// `LC_ALL=C php` prints `1`, `1`, `1`, `2`.
#[test]
fn test_substr_count_length() {
    let out = compile_and_run(
        r#"<?php
echo substr_count("hello world", "o", 0, 5), "|",
     substr_count("hello world", "o", 0, -5), "|",
     substr_count("hello world", "l", 3, 4), "|",
     substr_count("hello world", "o", 0, null);
"#,
    );
    assert_eq!(out, "1|1|1|2");
}

/// Verifies `substr_count()` resolves case-insensitively, through a namespace-qualified
/// call, and by named argument.
#[test]
fn test_substr_count_case_insensitive_namespaced_and_named_args() {
    let out = compile_and_run(
        r#"<?php
echo SUBSTR_COUNT("hello world", "o"), "|",
     \substr_count("hello world", "o"), "|",
     substr_count(haystack: "hello world", needle: "o", offset: 5);
"#,
    );
    assert_eq!(out, "2|2|1");
}

/// Verifies `substr_count()` raises php-src's catchable `ValueError`s for an empty needle
/// and for an `$offset`/`$length` pair that leaves the subject. Messages are verbatim
/// `LC_ALL=C php` 8.4 output.
#[test]
fn test_substr_count_value_errors() {
    let out = compile_and_run(
        r#"<?php
foreach ([["abc", "", 0, null], ["abc", "b", 5, null], ["abc", "b", 0, 9]] as $t) {
    try {
        substr_count($t[0], $t[1], $t[2], $t[3]);
    } catch (ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    );
    assert_eq!(
        out,
        "substr_count(): Argument #2 ($needle) must not be empty\n\
substr_count(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n\
substr_count(): Argument #4 ($length) must be contained in argument #1 ($haystack)\n"
    );
}

/// Verifies `strncmp()` compares only the first `$length` bytes and returns php-src's raw
/// byte difference. `LC_ALL=C php` prints `0`, `-12`, `-1`, `1`, `0` for these calls.
#[test]
fn test_strncmp_prefix_and_byte_difference() {
    let out = compile_and_run(
        r#"<?php
echo strncmp("Hello", "Hexxx", 2), "|",
     strncmp("Hello", "Hexxx", 3), "|",
     strncmp("abc", "abd", 3), "|",
     strncmp("abc", "ab", 3), "|",
     strncmp("abc", "abc", 10);
"#,
    );
    assert_eq!(out, "0|-12|-1|1|0");
}

/// Verifies `strncasecmp()` folds ASCII case before comparing the bounded prefix.
/// `LC_ALL=C php` prints `0`, `-1`, `1`.
#[test]
fn test_strncasecmp_ascii_folding() {
    let out = compile_and_run(
        r#"<?php
echo strncasecmp("HeLLo", "hellO", 5), "|",
     strncasecmp("ABC", "abd", 3), "|",
     strncasecmp("abc", "AB", 3);
"#,
    );
    assert_eq!(out, "0|-1|1");
}

/// Verifies both length-limited comparisons resolve case-insensitively, through a
/// namespace-qualified call, and by named argument.
#[test]
fn test_strncmp_case_insensitive_namespaced_and_named_args() {
    let out = compile_and_run(
        r#"<?php
echo STRNCMP("abc", "abd", 3), "|",
     \strncasecmp("ABC", "abc", 3), "|",
     strncmp(string1: "abc", string2: "abd", length: 2);
"#,
    );
    assert_eq!(out, "-1|0|0");
}

/// Verifies both length-limited comparisons raise php-src's catchable `ValueError` for a
/// negative `$length`. Messages are verbatim `LC_ALL=C php` 8.4 output.
#[test]
fn test_strncmp_negative_length_value_errors() {
    let out = compile_and_run(
        r#"<?php
try { strncmp("a", "b", -1); } catch (ValueError $e) { echo $e->getMessage(), "\n"; }
try { strncasecmp("a", "b", -1); } catch (ValueError $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(
        out,
        "strncmp(): Argument #3 ($length) must be greater than or equal to 0\n\
strncasecmp(): Argument #3 ($length) must be greater than or equal to 0\n"
    );
}
