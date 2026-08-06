//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of strings transform, including strtolower, strtoupper, and ucfirst.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies strtolower converts all alphabetic characters to lowercase.
#[test]
fn test_strtolower() {
    let out = compile_and_run(r#"<?php echo strtolower("Hello WORLD");"#);
    assert_eq!(out, "hello world");
}

/// Verifies strtoupper converts all alphabetic characters to uppercase.
#[test]
fn test_strtoupper() {
    let out = compile_and_run(r#"<?php echo strtoupper("Hello World");"#);
    assert_eq!(out, "HELLO WORLD");
}

/// Verifies ucfirst capitalizes the first character of a string.
#[test]
fn test_ucfirst() {
    let out = compile_and_run(r#"<?php echo ucfirst("hello");"#);
    assert_eq!(out, "Hello");
}

/// Verifies lcfirst lowercases the first character of a string.
#[test]
fn test_lcfirst() {
    let out = compile_and_run(r#"<?php echo lcfirst("Hello");"#);
    assert_eq!(out, "hello");
}

/// Verifies trim removes whitespace from both ends of a string.
#[test]
fn test_trim() {
    let out = compile_and_run("<?php echo trim(\"  hello  \");");
    assert_eq!(out, "hello");
}

/// Verifies ltrim removes whitespace from the left end of a string.
#[test]
fn test_ltrim() {
    let out = compile_and_run("<?php echo ltrim(\"  hello\");");
    assert_eq!(out, "hello");
}

/// Verifies rtrim removes whitespace from the right end of a string.
#[test]
fn test_rtrim() {
    let out = compile_and_run("<?php echo rtrim(\"hello  \");");
    assert_eq!(out, "hello");
}

/// Verifies str_repeat repeats a string the given number of times.
#[test]
fn test_str_repeat() {
    let out = compile_and_run(r#"<?php echo str_repeat("ab", 3);"#);
    assert_eq!(out, "ababab");
}

/// Verifies str_repeat handles large results that exceed the small-string inline buffer threshold (32768+ bytes), confirming the result is heap-allocated and its reported length is correct.
#[test]
fn test_str_repeat_large_heap_backed_result() {
    let out = compile_and_run(
        r#"<?php
echo strlen(str_repeat("ab", 32769));
echo ",";
$s = str_repeat("ab", 33000);
echo strlen($s);
"#,
    );
    assert_eq!(out, "65538,66000");
}

/// Verifies str_repeat emits a runtime error when given a negative count, matching PHP's behavior.
#[test]
fn test_str_repeat_negative_count_reports_runtime_error() {
    let err = compile_and_run_expect_failure(r#"<?php echo str_repeat("ab", -1);"#);
    assert!(err.contains(
        "Fatal error: str_repeat(): Argument #2 ($times) must be greater than or equal to 0"
    ));
}

/// Verifies strrev reverses the characters in a string.
#[test]
fn test_strrev() {
    let out = compile_and_run(r#"<?php echo strrev("Hello");"#);
    assert_eq!(out, "olleH");
}

/// Verifies grapheme_strrev reverses ASCII text like strrev while returning the PHP string|false shape.
#[test]
fn test_grapheme_strrev_ascii() {
    let out = compile_and_run(r#"<?php echo grapheme_strrev("ABCDE");"#);
    assert_eq!(out, "EDCBA");
}

/// Verifies grapheme_strrev keeps a combining mark attached to its base character.
#[test]
fn test_grapheme_strrev_combining_mark_cluster() {
    let out = compile_and_run("<?php echo grapheme_strrev(\"Ae\\u{0301}B\");");
    assert_eq!(out, "Be\u{0301}A");
}

/// Verifies grapheme_strrev keeps emoji modifiers and ZWJ sequences together as one cluster.
#[test]
fn test_grapheme_strrev_emoji_modifier_zwj_cluster() {
    let out = compile_and_run("<?php echo grapheme_strrev(\"A\\u{1F469}\\u{1F3FD}\\u{200D}\\u{1F4BB}B\");");
    assert_eq!(out, "B\u{1F469}\u{1F3FD}\u{200D}\u{1F4BB}A");
}

/// Verifies grapheme_strrev preserves embedded NUL bytes while reversing surrounding clusters.
#[test]
fn test_grapheme_strrev_preserves_nul_bytes() {
    let out = compile_and_run(r#"<?php echo grapheme_strrev("ab\0cd");"#);
    assert_eq!(out.as_bytes(), b"dc\0ba");
}

/// Verifies grapheme_strrev participates in builtin lookup, namespace fallback, and first-class callable syntax.
#[test]
fn test_grapheme_strrev_lookup_and_first_class_callable() {
    let out = compile_and_run(
        r#"<?php
namespace Demo;
echo function_exists("GrApHeMe_StRrEv") ? "1" : "0";
echo ":";
echo GrApHeMe_StRrEv("desk");
echo ":";
$rev = grapheme_strrev(...);
echo $rev("tool");
"#,
    );
    assert_eq!(out, "1:ksed:loot");
}

/// Verifies str_replace performs a simple find-and-replace on a string.
#[test]
fn test_str_replace() {
    let out = compile_and_run(r#"<?php echo str_replace("World", "PHP", "Hello World");"#);
    assert_eq!(out, "Hello PHP");
}

/// Verifies str_replace replaces all occurrences of a needle in a string.
#[test]
fn test_str_replace_multiple() {
    let out = compile_and_run(r#"<?php echo str_replace("o", "0", "Hello World");"#);
    assert_eq!(out, "Hell0 W0rld");
}

/// Verifies `strlen` applies PHP weak scalar coercion both directly and as an `array_map` callback.
#[test]
fn test_strlen_weak_scalar_coercion_including_callback() {
    let out = compile_and_run(
        r#"<?php
echo implode(",", array_map("strlen", [12, 345])), ":";
echo strlen(123), ":", strlen(1.5), ":", strlen(true), ":", strlen(false), ":", strlen(null);
"#,
    );
    assert_eq!(out, "2,3:3:3:1:0:0");
}

/// Verifies str_replace with an array `$search` and a single-string `$replace` replaces every
/// search needle with the one replacement, processing search elements in order.
#[test]
fn test_str_replace_array_search_single_replace() {
    let out = compile_and_run(r#"<?php echo str_replace(["a", "b"], "X", "abc");"#);
    assert_eq!(out, "XXc");
}

/// Verifies str_replace with array `$search`/`$replace` pairs each needle with its replacement.
#[test]
fn test_str_replace_array_search_array_replace() {
    let out = compile_and_run(r#"<?php echo str_replace(["a", "b"], ["1", "2"], "abc");"#);
    assert_eq!(out, "12c");
}

/// Verifies str_replace treats missing array-replacement elements as the empty string when the
/// replacement array is shorter than the search array.
#[test]
fn test_str_replace_array_replace_shorter() {
    let out = compile_and_run(r#"<?php echo str_replace(["a", "b", "c"], ["1"], "abc");"#);
    assert_eq!(out, "1");
}

/// Verifies an indexed string-array subject returns an indexed string array and can be unpacked.
#[test]
fn test_str_replace_array_subject_preserves_each_result() {
    let out = compile_and_run(
        r#"<?php
[$first, $second] = str_replace(["a", "b"], ["A", "B"], ["abc", "cab"]);
echo $first, ":", $second;
"#,
    );
    assert_eq!(out, "ABc:cAB");
}

/// Verifies str_replace applies array search elements iteratively, so a replacement introduced by an
/// earlier element is itself matched by a later search element (matching PHP ordering).
#[test]
fn test_str_replace_array_iterative_ordering() {
    let out = compile_and_run(r#"<?php echo str_replace(["a", "b"], ["b", "c"], "a");"#);
    assert_eq!(out, "c");
}

/// Verifies str_replace skips empty search-array elements as no-ops, matching PHP.
#[test]
fn test_str_replace_array_empty_search_element_skipped() {
    let out = compile_and_run(r#"<?php echo str_replace(["", "a"], ["X", "Y"], "abc");"#);
    assert_eq!(out, "Ybc");
}

/// Verifies str_ireplace supports the case-insensitive array-search/array-replace form.
#[test]
fn test_str_ireplace_array_search_array_replace() {
    let out = compile_and_run(r#"<?php echo str_ireplace(["A", "B"], ["1", "2"], "abAB");"#);
    assert_eq!(out, "1212");
}

/// Verifies explode splits a string on a delimiter and returns an indexed array.
#[test]
fn test_explode() {
    let out = compile_and_run(
        r#"<?php
$parts = explode(",", "a,b,c");
echo count($parts);
echo " ";
echo $parts[0] . " " . $parts[1] . " " . $parts[2];
"#,
    );
    assert_eq!(out, "3 a b c");
}

/// Verifies implode joins array elements into a string with a given separator.
#[test]
fn test_implode() {
    let out = compile_and_run(
        r#"<?php
$arr = ["Hello", "World"];
echo implode(" ", $arr);
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Verifies implode accepts the `Array(Void)` layout used for statically empty arrays.
#[test]
fn test_implode_empty_array_void_element_layout() {
    let out = compile_and_run(
        r#"<?php
echo "[", implode(",", []), "]";
echo "[", implode(",", array_merge([], [])), "]";
"#,
    );
    assert_eq!(out, "[][]");
}

/// `strncmp()` compares at most `$length` leading bytes and returns their raw byte difference —
/// NOT a normalized -1/0/1. Expectations pinned against `php -n` (PHP 8.5).
#[test]
fn test_strncmp_returns_php_byte_difference() {
    let out = compile_and_run(
        r#"<?php
echo strncmp("hello", "help", 3), "|";
echo strncmp("hello", "help", 4), "|";
echo strncmp("help", "hello", 4), "|";
echo strncmp("abc", "abc", 10), "|";
echo strncmp("abc", "ab", 3), "|";
echo strncmp("abc", "abd", 0);
"#,
    );
    assert_eq!(out, "0|-4|4|0|1|0");
}

/// A NEGATIVE `$offset` makes `strpos()` start `abs($offset)` bytes from the END of the haystack.
///
/// The native lowering applied the offset as `ptr += offset; len -= offset`, which for a negative
/// offset walked the haystack pointer BEFORE the string and answered as though the offset were
/// relative to the start — silently: `strpos("abcabc", "a", -3)` returned 0 where PHP returns 3,
/// and `strpos("hello", "l", -1)` returned 2 where PHP returns false. Pinned against `php -n`.
#[test]
fn test_strpos_negative_offset_counts_from_the_end() {
    let out = compile_and_run(
        r#"<?php
var_dump(strpos("hello", "l", -2));
var_dump(strpos("hello", "l", -3));
var_dump(strpos("hello", "l", -1));
var_dump(strpos("hello", "h", -5));
var_dump(strpos("abcabc", "a", -3));
"#,
    );
    assert_eq!(
        out,
        "int(3)\nint(2)\nbool(false)\nint(0)\nint(3)\n"
    );
}

/// An offset outside the haystack raises a `ValueError` in PHP 8; it used to return `false`.
#[test]
fn test_strpos_offset_outside_the_haystack_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
foreach ([-6, 6] as $offset) {
    try {
        strpos("hello", "l", $offset);
    } catch (\ValueError $e) {
        echo $e->getMessage(), "\n";
    }
}
"#,
    );
    assert_eq!(
        out,
        "strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n\
         strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)\n"
    );
}

/// A positive offset keeps working through the same path.
#[test]
fn test_strpos_positive_offset_still_searches_forward() {
    let out = compile_and_run(
        r#"<?php
var_dump(strpos("hello", "l", 0));
var_dump(strpos("hello", "l", 3));
var_dump(strpos("hello", "", 5));
"#,
    );
    assert_eq!(out, "int(2)\nint(3)\nint(5)\n");
}

/// `strrpos()`'s offset means something DIFFERENT from `strpos()`'s: a NEGATIVE offset requires the
/// match to START at or before `strlen + offset`, so the search window ends `strlen($needle)`
/// bytes further along. The native lowering shared `strpos`'s haystack adjustment and got this
/// wrong the same silent way. Pinned against `php -n`.
#[test]
fn test_strrpos_negative_offset_bounds_the_match_start() {
    let out = compile_and_run(
        r#"<?php
var_dump(strrpos("hello", "l", -1));
var_dump(strrpos("hello", "l", -2));
var_dump(strrpos("hello", "l", -3));
var_dump(strrpos("hello", "l", -5));
var_dump(strrpos("hello", "l", 3));
var_dump(strrpos("hello", "l", 4));
"#,
    );
    assert_eq!(
        out,
        "int(3)\nint(3)\nint(2)\nbool(false)\nint(3)\nbool(false)\n"
    );
}

/// `strripos()` is the case-insensitive form and inherits the same offset contract.
#[test]
fn test_strripos_folds_case_and_honours_the_offset() {
    let out = compile_and_run(
        r#"<?php
var_dump(strripos("HELLO", "l"));
var_dump(strripos("HELLO", "L", -3));
var_dump(strripos("HELLO", "z"));
"#,
    );
    assert_eq!(out, "int(3)\nint(2)\nbool(false)\n");
}

/// `strncasecmp()` folds ASCII case on both truncations. Pinned against `php -n` (PHP 8.5).
#[test]
fn test_strncasecmp_returns_php_byte_difference() {
    let out = compile_and_run(
        r#"<?php
echo strncasecmp("Hello", "HELP", 3), "|";
echo strncasecmp("Hello", "HELP", 4), "|";
echo strncasecmp("HELP", "Hello", 4);
"#,
    );
    assert_eq!(out, "0|-4|4");
}

/// `stripos()` is case-insensitive `strpos()`: ASCII folding preserves byte length, so positions
/// map 1:1 onto the original haystack. Covers the `false` miss, the empty needle and a NEGATIVE
/// offset — all pinned against `php -n`.
#[test]
fn test_stripos_matches_php_including_false_and_negative_offset() {
    let out = compile_and_run(
        r#"<?php
var_dump(stripos("Hello World", "o"));
var_dump(stripos("Hello World", "O", 5));
var_dump(stripos("Hello", "z"));
var_dump(stripos("Hello", ""));
var_dump(stripos("Hello", "L", -2));
"#,
    );
    assert_eq!(
        out,
        "int(4)\nint(7)\nbool(false)\nint(0)\nint(3)\n"
    );
}

/// PHP 8 rejects a negative `$length` with a `ValueError` rather than clamping it.
#[test]
fn test_strncmp_negative_length_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
try {
    strncmp("abc", "abd", -1);
} catch (\ValueError $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "strncmp(): Argument #3 ($length) must be greater than or equal to 0"
    );
}

/// Verifies `explode()`'s three-argument form against PHP's `$limit` semantics: a POSITIVE limit
/// caps the result at `$limit` elements, with the last one holding the entire unsplit remainder.
/// Expectations pinned against `php -n`.
#[test]
fn test_explode_positive_limit_merges_the_remainder() {
    let out = compile_and_run(
        r#"<?php
$parts = explode(",", "a,b,c,d", 2);
echo count($parts), "|", $parts[0], "|", $parts[1];
"#,
    );
    assert_eq!(out, "2|a|b,c,d");
}

/// A limit that meets or exceeds the natural segment count leaves the split untouched.
#[test]
fn test_explode_limit_larger_than_segment_count_is_a_full_split() {
    let out = compile_and_run(
        r#"<?php
$parts = explode(",", "a,b", 99);
echo count($parts), "|", implode("/", $parts);
"#,
    );
    assert_eq!(out, "2|a/b");
}

/// A NEGATIVE limit drops that many trailing segments outright — it does NOT merge them into the
/// last element the way a positive limit does.
#[test]
fn test_explode_negative_limit_drops_trailing_segments() {
    let out = compile_and_run(
        r#"<?php
$parts = explode(",", "a,b,c,d", -1);
echo count($parts), "|", implode("/", $parts);
$gone = explode(",", "a,b", -5);
echo "|", count($gone);
"#,
    );
    assert_eq!(out, "3|a/b/c|0");
}

/// PHP treats a ZERO limit as 1, so the whole subject comes back as a single element.
#[test]
fn test_explode_zero_limit_behaves_as_one() {
    let out = compile_and_run(
        r#"<?php
$parts = explode(",", "a,b,c", 0);
echo count($parts), "|", $parts[0];
"#,
    );
    assert_eq!(out, "1|a,b,c");
}

/// The limit is an ordinary runtime value, not a literal the compiler can fold away.
#[test]
fn test_explode_limit_from_a_runtime_variable() {
    let out = compile_and_run(
        r#"<?php
$n = 3;
$parts = explode("-", "w-x-y-z", $n);
echo count($parts), "|", implode("/", $parts);
"#,
    );
    assert_eq!(out, "3|w/x/y-z");
}

/// Verifies explode followed by implode produces the expected string transformation.
#[test]
fn test_explode_implode_roundtrip() {
    let out = compile_and_run(
        r#"<?php
$str = "one-two-three";
$parts = explode("-", $str);
echo implode(", ", $parts);
"#,
    );
    assert_eq!(out, "one, two, three");
}

/// REGRESSION for the N2 union-boxed-array READ SIGSEGV: `$u = $hosts ?: false;
/// implode(",", $u)` used to segfault (rc=139) — `--emit-ir` showed the Elvis join correctly
/// boxes both arms into a tagged `Heap(Mixed)` cell (proving the JOIN representation was sound),
/// but `implode`'s dynamic Mixed-array reader unconditionally called the generic `__rt_implode`
/// runtime routine after unboxing, which only correctly handles STRING (value_type 1) and
/// boxed-Mixed (value_type 7) element layouts — for `[1,2,3]`'s raw 8-byte int elements
/// (value_type 0) it misread each integer VALUE as a `{ptr,len}` string pair and dereferenced it
/// as a pointer. Fixed in `crate::codegen_ir::lower_inst::builtins::strings::lower_implode_dynamic`
/// by reading the array's OWN runtime element tag and branching to the already-existing, already
/// tag-correct `__rt_implode_int` helper instead. php -n verified: `1,2,3`.
#[test]
fn test_implode_union_array_false_idiom_int_array_no_longer_segfaults() {
    let out = compile_and_run(
        r#"<?php
$hosts = [1, 2, 3];
$u = $hosts ?: false;
echo implode(",", $u);
"#,
    );
    assert_eq!(out, "1,2,3");
}

/// Same repro shape as `test_implode_union_array_false_idiom_int_array_no_longer_segfaults` but
/// with a `array<string>`-element source array, exercising the OTHER branch of the runtime
/// element-tag dispatch (`__rt_implode`, value_type 1) instead of `__rt_implode_int`.
#[test]
fn test_implode_union_array_false_idiom_string_array() {
    let out = compile_and_run(
        r#"<?php
$parts = ["a", "b", "c"];
$u = $parts ?: false;
echo implode("-", $u);
"#,
    );
    assert_eq!(out, "a-b-c");
}

/// Verifies `implode(",", $u)` on the OTHER branch of the `$hosts = $x ?: false` idiom (an empty,
/// therefore falsy, source array collapses `$u` to boxed `false`) throws a catchable `\TypeError`
/// with PHP's EXACT wording instead of reading a null/zero payload as an array pointer. php -n
/// VERIFIED against PHP 8.5's real (nullable) `implode(string $separator, ?array $array)`
/// signature: `implode(): Argument #2 ($array) must be of type ?array, false given`.
#[test]
fn test_implode_union_false_tag_throws_byte_identical_type_error() {
    let out = compile_and_run(
        r#"<?php
$hosts = [];
$u = $hosts ?: false;
try {
    echo implode(",", $u);
} catch (\TypeError $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "implode(): Argument #2 ($array) must be of type ?array, false given"
    );
}

/// Sweeps the shared `union_type_guard` wrong-tag `\TypeError` dispatch (int/float/true/null)
/// through `implode()`'s union-array argument — the SAME dispatch `array_slice()`/`count()` reuse
/// for this family. php -n VERIFIED every message.
#[test]
fn test_implode_union_wrong_scalar_tags_throw_byte_identical_type_errors() {
    let out = compile_and_run(
        r#"<?php
function probe($v): string {
    $u = $v ?: false;
    try {
        return implode(",", $u);
    } catch (\TypeError $e) {
        return $e->getMessage();
    }
}
echo probe(0), "|";
echo probe(1.5), "|";
echo probe(true), "|";
echo probe(null);
"#,
    );
    assert_eq!(
        out,
        "implode(): Argument #2 ($array) must be of type ?array, false given|\
implode(): Argument #2 ($array) must be of type ?array, float given|\
implode(): Argument #2 ($array) must be of type ?array, true given|\
implode(): Argument #2 ($array) must be of type ?array, false given"
    );
}

/// Heap-cleanliness proof for the SIGSEGV fix: the union-boxed array payload is only BORROWED
/// (`crate::codegen_ir::lower_inst::builtins::arrays::union_type_guard::emit_borrow_array_or_type_error`
/// — no incref/decref/tag mutation on the source cell), so running the (formerly crashing) repro
/// under `--heap-debug` must report a clean heap with no leaked or double-freed blocks.
#[test]
fn test_implode_union_array_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$hosts = [1, 2, 3];
$u = $hosts ?: false;
echo implode(",", $u);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1,2,3");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

// --- v0.4 batch 2: more string functions ---

/// Verifies ucwords capitalizes the first character of each word in a string.
#[test]
fn test_ucwords() {
    let out = compile_and_run(r#"<?php echo ucwords("hello world foo");"#);
    assert_eq!(out, "Hello World Foo");
}

/// Verifies `ucwords($string, $separators)` honors a custom word-boundary set.
///
/// With separators `"-"` only the first character and the character after each
/// `-` are capitalized (a following space is no longer a boundary), matching
/// PHP's `ucwords("a-b c", "-") === "A-B c"`. A multi-character separators set
/// (`"|-"`) treats every listed byte as a boundary. Verified against `php -n`.
#[test]
fn test_ucwords_custom_separators() {
    let out = compile_and_run(
        r#"<?php
echo ucwords("a-b c", "-"), "\n";
echo ucwords("foo|bar-baz", "|-"), "\n";
$sep = "-";
echo ucwords("x-y-z", $sep), "\n";
"#,
    );
    assert_eq!(out, "A-B c\nFoo|Bar-Baz\nX-Y-Z\n");
}

/// Verifies str_ireplace performs case-insensitive find-and-replace.
#[test]
fn test_str_ireplace() {
    let out = compile_and_run(r#"<?php echo str_ireplace("WORLD", "PHP", "Hello World");"#);
    assert_eq!(out, "Hello PHP");
}

/// Verifies str_pad with default right-padding when pad_type is omitted.
#[test]
fn test_str_pad_right() {
    let out = compile_and_run(r#"<?php echo str_pad("hi", 5);"#);
    assert_eq!(out, "hi   ");
}

/// Verifies str_pad left-padding when pad_type is explicitly 0 (left).
#[test]
fn test_str_pad_left() {
    let out = compile_and_run(r#"<?php echo str_pad("hi", 5, " ", 0);"#);
    assert_eq!(out, "   hi");
}

/// Verifies str_pad with pad_type 2 (both sides) and a custom pad character.
#[test]
fn test_str_pad_both() {
    let out = compile_and_run(r#"<?php echo str_pad("hi", 6, "-", 2);"#);
    assert_eq!(out, "--hi--");
}

/// Verifies str_pad left-padding with a custom zero character.
#[test]
fn test_str_pad_custom_char() {
    let out = compile_and_run(r#"<?php echo str_pad("42", 5, "0", 0);"#);
    assert_eq!(out, "00042");
}

/// Verifies str_split splits a string into chunks of a given length.
#[test]
fn test_str_split() {
    let out = compile_and_run(
        r#"<?php
$parts = str_split("Hello", 2);
echo count($parts) . " " . $parts[0] . " " . $parts[1] . " " . $parts[2];
"#,
    );
    assert_eq!(out, "3 He ll o");
}

/// Verifies sprintf zero-pads an integer to a given width.
#[test]
fn test_sprintf_zero_padded_int() {
    let out = compile_and_run(r#"<?php echo sprintf("%05d", 42);"#);
    assert_eq!(out, "00042");
}

/// Regression: a string builtin applied to a boxed `Mixed` value inside a concatenation must
/// unbox the argument into the string ABI registers. Before the fix `strtoupper` read the stale
/// left-hand concat operand (`"L:"`) instead of the Mixed argument, producing `"L:L:"`.
#[test]
fn test_strtoupper_of_mixed_in_concatenation() {
    let out = compile_and_run(r#"<?php $j = json_decode('"widget"'); echo "L:" . strtoupper($j);"#);
    assert_eq!(out, "L:WIDGET");
}

/// Regression: the same unboxing applies across string-transform builtins taking a `Mixed`
/// argument (here `strtolower`, `strrev`, `ucfirst`), not just `strtoupper`.
#[test]
fn test_string_transforms_of_mixed_argument() {
    let out = compile_and_run(
        r#"<?php
        $h = json_decode('"HELLO"');
        $a = json_decode('"abc"');
        echo strtolower($h), "|", strrev($a), "|", ucfirst($a);
        "#,
    );
    assert_eq!(out, "hello|cba|Abc");
}

/// Regression: multi-argument string builtins must also unbox a `Mixed` string argument, whether
/// it is the subject (`str_replace` arg 3), the haystack (`strpos`), or the source (`explode`) —
/// not only the first argument. Before the fix these read stale string registers for a Mixed arg.
#[test]
fn test_multiarg_string_builtins_of_mixed_argument() {
    let out = compile_and_run(
        r#"<?php
        $m = json_decode('"hello world"');
        echo str_replace("o", "0", $m), "|", strpos($m, "world"), "|", implode(",", explode(" ", $m));
        "#,
    );
    assert_eq!(out, "hell0 w0rld|6|hello,world");
}

/// Regression: `$v = trim($v)` reassigning a HEAP string to a trimmed slice of ITSELF used to
/// corrupt `$v`. `trim` returns a borrowed slice into the source buffer; the store freed the old
/// buffer the slice still points into before copying it, so the value read back was garbage. This
/// is the exact shape of symfony/yaml `Inline::parse`'s `mixed` scalar return (a heap `$value`
/// trimmed and returned), which produced corruption once heap churn reused the freed buffer.
/// Persisting the trim slice to an owned copy fixes it. The first iteration used to be correct and
/// later iterations garbage; asserting every iteration is `elephc` catches the regression.
/// Output cross-checked with `php -r`.
#[test]
fn test_trim_self_reassign_mixed_return_loop_does_not_corrupt() {
    let out = compile_and_run(
        r#"<?php
function parseScalar(string $v): mixed {
    $v = trim($v);
    $refs = [];
    for ($i = 0; $i < 8; $i++) { $refs[] = $i * $i; }
    return $v;
}
$out = "";
$parts = ["ele", "phc"];
for ($k = 0; $k < 4; $k++) {
    $h = $parts[0] . $parts[1];
    $r = parseScalar($h);
    $out .= $r . ":" . strlen($r) . "|";
}
echo $out;
"#,
    );
    assert_eq!(out, "elephc:6|elephc:6|elephc:6|elephc:6|");
}

/// Regression: `$s = trim($s)` on a heap string with real leading/trailing whitespace persists the
/// interior slice to an owned copy, so a later read after heap churn still sees the trimmed value
/// instead of a freed/reused region. Guards the interior-pointer case of the trim persist fix.
#[test]
fn test_trim_self_reassign_interior_slice_survives_heap_churn() {
    let out = compile_and_run(
        r#"<?php
$parts = ["  ele", "phc  "];
$s = $parts[0] . $parts[1];
$s = trim($s);
$junk = [];
for ($i = 0; $i < 12; $i++) { $junk[] = str_repeat("q", $i + 1); }
echo $s, "|", strlen($s);
"#,
    );
    assert_eq!(out, "elephc|6");
}

/// Verifies strval coerces scalars to their PHP string representation.
/// Fixture: int/float/string via strval; each matches the `(string)` cast (php-verified).
#[test]
fn test_strval_scalars() {
    let out = compile_and_run(
        r#"<?php echo strval(42), "|", strval(1.5), "|", strval("x");"#,
    );
    assert_eq!(out, "42|1.5|x");
}

/// Verifies strval of a boolean true yields "1" and null yields the empty string.
/// Fixture: strval(true)="1", strval(null)="" (php-verified).
#[test]
fn test_strval_bool_and_null() {
    let out = compile_and_run(r#"<?php echo "[", strval(true), "][", strval(null), "]";"#);
    assert_eq!(out, "[1][]");
}

/// Verifies strval resolves through PHP's namespace fallback and case-insensitive lookup.
/// Fixture: inside a namespace, upper-case STRVAL() resolves to the builtin.
#[test]
fn test_strval_namespaced_case_insensitive_fallback() {
    let out = compile_and_run(r#"<?php namespace App; echo STRVAL(7);"#);
    assert_eq!(out, "7");
}

/// Verifies addcslashes backslash-prefixes printable set members, including `a..z` ranges.
/// Fixture: addcslashes("aBcDeF", "A..Z") escapes the upper-case letters (php-verified).
#[test]
fn test_addcslashes_range_and_printable() {
    let out = compile_and_run(
        r#"<?php echo addcslashes("aBcDeF", "A..Z"), "|", addcslashes("a.b", "."), "|", addcslashes('a"b', '"');"#,
    );
    assert_eq!(out, r#"a\Bc\De\F|a\.b|a\"b"#);
}

/// Verifies addcslashes emits C escapes for control bytes and octal for high bytes.
/// Fixture: NUL/TAB/LF/BEL/ESC via a `\0..\37` range, and chr(200) -> `\310` (php-verified).
#[test]
fn test_addcslashes_control_and_octal() {
    let out = compile_and_run(
        r#"<?php echo bin2hex(addcslashes(chr(0).chr(9).chr(10).chr(7).chr(27), "\0..\37")), "|", bin2hex(addcslashes(chr(200), chr(200)));"#,
    );
    assert_eq!(out, "5c3030305c745c6e5c615c303333|5c333130");
}

/// Verifies stripcslashes decodes C-style escapes as the inverse of addcslashes.
/// Fixture: `\t`/`\n` control escapes, `\101` octal -> 'A', `\x42` hex -> 'B' (php-verified).
#[test]
fn test_stripcslashes_decodes_escapes() {
    let out = compile_and_run(
        r#"<?php echo bin2hex(stripcslashes("a\\tb\\nc")), "|", stripcslashes("\\101\\x42");"#,
    );
    assert_eq!(out, "6109620a63|AB");
}

/// Verifies stripcslashes truncates octal to one byte and leaves unknown escapes literal.
/// Fixture: `\777` -> 0xff, and `\A`/`\z`/`\8` keep their literal characters (php-verified).
#[test]
fn test_stripcslashes_octal_overflow_and_literals() {
    let out = compile_and_run(
        r#"<?php echo bin2hex(stripcslashes("\\777")), "|", stripcslashes("\\A\\z\\8");"#,
    );
    assert_eq!(out, "ff|Az8");
}

/// Verifies stripcslashes resolves through PHP's namespace fallback and case-insensitive lookup.
/// Fixture: inside a namespace, upper-case StripCSlashes() resolves to the builtin.
#[test]
fn test_stripcslashes_namespaced_case_insensitive_fallback() {
    let out = compile_and_run(r#"<?php namespace App; echo StripCSlashes("q\\tr");"#);
    assert_eq!(out, "q\tr");
}
