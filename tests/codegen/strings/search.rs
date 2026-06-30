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

/// Verifies `strcspn` returns the length of the leading run not containing any listed character.
/// Fixture: "hello world" stops at the 'o' (index 4) found in "wo".
#[test]
fn test_strcspn_basic() {
    let out = compile_and_run(r#"<?php echo strcspn("hello world", "wo");"#);
    assert_eq!(out, "4");
}

/// Verifies `strcspn` returns the full length when no listed character is present.
#[test]
fn test_strcspn_no_match_returns_full_length() {
    let out = compile_and_run(r#"<?php echo strcspn("abc", "xyz");"#);
    assert_eq!(out, "3");
}

/// Verifies `strspn` returns the length of the leading run consisting entirely of listed characters.
/// Fixture: "42 is the answer" matches "42" against the digit set, stopping at the space.
#[test]
fn test_strspn_basic() {
    let out = compile_and_run(r#"<?php echo strspn("42 is the answer", "1234567890");"#);
    assert_eq!(out, "2");
}

/// Verifies `strspn` returns 0 when the first byte is not in the character set.
#[test]
fn test_strspn_no_leading_match() {
    let out = compile_and_run(r#"<?php echo strspn("abc", "0123456789");"#);
    assert_eq!(out, "0");
}

/// Verifies the 3-argument `strcspn`/`strspn` forms scan a substr-style window
/// starting at a non-negative offset (the form symfony/yaml relies on).
///
/// `strcspn("hello world", " \r\n", 1)` scans "ello world" and stops at the
/// space (index 4); `strspn("   abc", " ", 1)` counts the two remaining spaces.
#[test]
fn test_strcspn_strspn_with_offset() {
    let out = compile_and_run(
        r#"<?php echo strcspn("hello world", " \r\n", 1), "|", strspn("   abc", " ", 1);"#,
    );
    assert_eq!(out, "4|2");
}

/// Verifies the 4-argument `strcspn` form clamps the scan to an explicit window,
/// including PHP's tail-relative negative length.
///
/// `strcspn("hello", "xyz", 2, 1)` scans only the 1-byte window "l" (no member,
/// span 1); `strcspn("hello world", "o", 0, -3)` scans "hello wo" (length
/// 11 - 3) and stops at 'o' (index 4). Cross-checked against `php -r`.
#[test]
fn test_strcspn_with_offset_and_length() {
    let out = compile_and_run(
        r#"<?php echo strcspn("hello", "xyz", 2, 1), "|", strcspn("hello world", "o", 0, -3);"#,
    );
    assert_eq!(out, "1|4");
}

/// Verifies `strpbrk` returns the suffix beginning at the first character found in the set.
/// Fixture: "hello" with set "lo" first matches 'l' at index 2, yielding "llo".
#[test]
fn test_strpbrk_found() {
    let out = compile_and_run(r#"<?php var_dump(strpbrk("hello", "lo"));"#);
    assert_eq!(out, "string(3) \"llo\"\n");
}

/// Verifies `strpbrk` returns boolean `false` when no character in the set occurs in the string.
#[test]
fn test_strpbrk_not_found_returns_false() {
    let out = compile_and_run(r#"<?php var_dump(strpbrk("hello", "xyz"));"#);
    assert_eq!(out, "bool(false)\n");
}

/// Verifies `strcspn`/`strspn` resolve case-insensitively (PHP builtin names are case-insensitive).
#[test]
fn test_strcspn_strspn_case_insensitive() {
    let out = compile_and_run(r#"<?php echo StrCsPn("hello", "l"), "|", STRSPN("aaab", "a");"#);
    assert_eq!(out, "2|3");
}

/// Verifies octdec converts octal string "17" to decimal 15 (1*8 + 7).
#[test]
fn test_octdec_basic() {
    let out = compile_and_run(r#"<?php echo octdec("17");"#);
    assert_eq!(out, "15");
}

/// Verifies octdec converts "777" (3 octal sevens) to decimal 511.
#[test]
fn test_octdec_three_digits() {
    let out = compile_and_run(r#"<?php echo octdec("777");"#);
    assert_eq!(out, "511");
}

/// Verifies octdec stops parsing at the first non-octal character.
/// Fixture: "18" stops at '8' (not an octal digit), returning 1.
#[test]
fn test_octdec_stops_at_non_octal() {
    let out = compile_and_run(r#"<?php echo octdec("18");"#);
    assert_eq!(out, "1");
}

/// Verifies octdec returns 0 for an empty string.
#[test]
fn test_octdec_empty_string() {
    let out = compile_and_run(r#"<?php echo octdec("");"#);
    assert_eq!(out, "0");
}

/// Verifies substr_count counts all non-overlapping occurrences of the needle.
/// Fixture: "hello world hello" contains "hello" twice.
#[test]
fn test_substr_count_basic() {
    let out = compile_and_run(r#"<?php echo substr_count("hello world hello", "hello");"#);
    assert_eq!(out, "2");
}

/// Verifies substr_count returns 1 when there is exactly one occurrence.
#[test]
fn test_substr_count_single() {
    let out = compile_and_run(r#"<?php echo substr_count("abcdef", "cd");"#);
    assert_eq!(out, "1");
}

/// Verifies substr_count returns 0 when the needle is absent.
#[test]
fn test_substr_count_not_found() {
    let out = compile_and_run(r#"<?php echo substr_count("hello", "xyz");"#);
    assert_eq!(out, "0");
}

/// Verifies substr_count does not count overlapping occurrences.
/// Fixture: "aaaa" has two non-overlapping "aa" occurrences (positions 0 and 2).
#[test]
fn test_substr_count_non_overlapping() {
    let out = compile_and_run(r#"<?php echo substr_count("aaaa", "aa");"#);
    assert_eq!(out, "2");
}

/// Verifies strstr with before_needle=true returns the prefix before the first occurrence.
/// Fixture: "user@example.com" split on "@" returns "user".
#[test]
fn test_strstr_before_needle_true() {
    let out = compile_and_run(r#"<?php echo strstr("user@example.com", "@", true);"#);
    assert_eq!(out, "user");
}

/// Verifies strstr with before_needle=false returns the suffix starting at the needle.
/// Fixture: "user@example.com" split on "@" returns "@example.com".
#[test]
fn test_strstr_before_needle_false() {
    let out = compile_and_run(r#"<?php echo strstr("user@example.com", "@", false);"#);
    assert_eq!(out, "@example.com");
}

/// Verifies strstr with before_needle=true returns the empty (not-found) result when the
/// needle is absent. elephc models strstr as `Str`, so the miss path yields an empty string
/// (mirroring `test_strpos_not_found`); the prefix reconstruction must not corrupt that path.
#[test]
fn test_strstr_before_needle_miss() {
    let out = compile_and_run(r#"<?php echo "[", strstr("hello", "@", true), "]";"#);
    assert_eq!(out, "[]");
}

/// Verifies strpos with a starting offset skips the first occurrence and finds the second.
/// Fixture: "abcabc" contains "c" at both offset 2 and 5; starting from offset 3 finds offset 5.
#[test]
fn test_strpos_with_offset() {
    let out = compile_and_run(r#"<?php echo strpos("abcabc", "c", 3);"#);
    assert_eq!(out, "5");
}

/// Verifies strpos with an offset past all occurrences returns strict false.
#[test]
fn test_strpos_with_offset_not_found() {
    let out = compile_and_run(
        r#"<?php echo strpos("abcabc", "c", 6) === false ? "miss" : "hit";"#,
    );
    assert_eq!(out, "miss");
}

/// Verifies strpos with offset 0 behaves identically to the 2-arg form.
#[test]
fn test_strpos_with_zero_offset() {
    let out = compile_and_run(r#"<?php echo strpos("abcabc", "c", 0);"#);
    assert_eq!(out, "2");
}
