//! Purpose:
//! Integration or regression tests for diagnostic coverage of string builtins, including substr wrong args, strpos wrong args, and strpos false return rejects integer return type.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

expect_builtin_arity_error!(
    test_error_substr_replace_wrong_args,
    "<?php substr_replace(\"abc\", \"x\");",
    "substr_replace() takes 3 or 4 arguments"
);

expect_builtin_arity_error!(
    test_error_strcspn_wrong_args,
    "<?php strcspn(\"abc\");",
    "strcspn() takes 2 to 4 arguments"
);

expect_builtin_arity_error!(
    test_error_strspn_wrong_args,
    "<?php strspn(\"abc\");",
    "strspn() takes 2 to 4 arguments"
);

expect_builtin_arity_error!(
    test_error_strpbrk_wrong_args,
    "<?php strpbrk(\"abc\");",
    "strpbrk() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_hexdec_wrong_args,
    "<?php hexdec();",
    "hexdec() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_hexdec_too_many_args,
    "<?php hexdec(\"ff\", \"00\");",
    "hexdec() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_rawurlencode_wrong_args,
    "<?php rawurlencode();",
    "rawurlencode() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_base64_decode_wrong_args,
    "<?php base64_decode();",
    "base64_decode() takes 1 or 2 arguments"
);

expect_builtin_arity_error!(
    test_error_base64_decode_too_many_args,
    "<?php base64_decode(\"aGk=\", true, 1);",
    "base64_decode() takes 1 or 2 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_ereg_match_wrong_args,
    "<?php mb_ereg_match('ab');",
    "mb_ereg_match() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_mb_strlen_wrong_args,
    "<?php mb_strlen();",
    "mb_strlen() takes 1 or 2 arguments"
);

/// Verifies that `mb_strlen()` rejects a statically non-string value argument.
#[test]
fn test_error_mb_strlen_string_type() {
    expect_error(
        "<?php mb_strlen([1, 2]);",
        "mb_strlen() string argument must be string",
    );
}

/// Verifies that `mb_strlen()` accepts only string or null encoding arguments.
#[test]
fn test_error_mb_strlen_encoding_type() {
    expect_error(
        "<?php mb_strlen('abc', 123);",
        "mb_strlen() encoding argument must be string or null",
    );
}

/// Verifies that `mb_ereg_match()` rejects a non-string pattern.
#[test]
fn test_error_mb_ereg_match_pattern_type() {
    expect_error(
        "<?php mb_ereg_match(123, 'abc');",
        "mb_ereg_match() pattern argument must be string",
    );
}

/// Verifies that `mb_ereg_match()` rejects non-string, non-null options.
#[test]
fn test_error_mb_ereg_match_options_type() {
    expect_error(
        "<?php mb_ereg_match('ab', 'abc', 1);",
        "mb_ereg_match() options argument must be string or null",
    );
}

/// Verifies that `grapheme_strrev()` with no arguments produces the correct arity error.
#[test]
fn test_error_grapheme_strrev_wrong_args() {
    expect_error(
        "<?php grapheme_strrev();",
        "grapheme_strrev() takes exactly 1 argument",
    );
}

/// Verifies that `grapheme_strrev()` rejects statically non-string arguments.
#[test]
fn test_error_grapheme_strrev_non_string_argument() {
    expect_error(
        "<?php grapheme_strrev(123);",
        "grapheme_strrev() argument must be string",
    );
}

expect_builtin_arity_error!(
    test_error_crc32_wrong_args,
    "<?php crc32();",
    "crc32() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_ctype_digit_wrong_args,
    "<?php ctype_digit();",
    "ctype_digit() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_ctype_alnum_wrong_args,
    "<?php ctype_alnum();",
    "ctype_alnum() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_ctype_space_wrong_args,
    "<?php ctype_space();",
    "ctype_space() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_ctype_upper_wrong_args,
    "<?php ctype_upper();",
    "ctype_upper() takes exactly 1 argument"
);

/// Verifies `ctype_upper(mixed): bool` is recognized (no "Undefined function") and its bool
/// result flows into a boolean context. It is recognition-only (no runtime lowering yet).
#[test]
fn test_ctype_upper_recognized() {
    assert!(
        check_source("<?php $b = ctype_upper(\"ABC\"); echo $b ? \"y\" : \"n\";").is_ok(),
        "ctype_upper() should type-check as a recognized bool-returning builtin",
    );
}

expect_builtin_arity_error!(
    test_error_chop_wrong_args,
    "<?php chop();",
    "chop() takes 1 or 2 arguments"
);

/// Verifies that `substr()` with only one string argument produces the correct arity error.
#[test]
fn test_error_substr_wrong_args() {
    expect_error("<?php substr(\"hi\");", "substr() takes 2 or 3 arguments");
}

/// Verifies that `strpos()` with only one argument produces the correct arity error.
#[test]
fn test_error_strpos_wrong_args() {
    expect_error(
        "<?php strpos(\"hi\");",
        "strpos() takes 2 or 3 arguments",
    );
}

/// Verifies that `strpos()` with four arguments produces the correct arity error.
#[test]
fn test_error_strpos_too_many_args() {
    expect_error(
        "<?php strpos(\"hi\", \"i\", 0, 1);",
        "strpos() takes 2 or 3 arguments",
    );
}

/// Verifies that `strstr()` with four arguments produces the correct arity error.
#[test]
fn test_error_strstr_too_many_args() {
    expect_error(
        "<?php strstr(\"hi\", \"i\", true, 1);",
        "strstr() takes 2 or 3 arguments",
    );
}

/// Verifies that `octdec()` with no arguments produces the correct arity error.
#[test]
fn test_error_octdec_wrong_args() {
    expect_error("<?php octdec();", "octdec() takes exactly 1 argument");
}

/// Verifies that `octdec()` with two arguments produces the correct arity error.
#[test]
fn test_error_octdec_too_many_args() {
    expect_error(
        "<?php octdec(\"17\", \"77\");",
        "octdec() takes exactly 1 argument",
    );
}

/// Verifies that `substr_count()` with only one argument produces the correct arity error.
#[test]
fn test_error_substr_count_wrong_args() {
    expect_error(
        "<?php substr_count(\"hi\");",
        "substr_count() takes 2 to 4 arguments",
    );
}

/// Verifies that `substr_count()` with five arguments produces the correct arity error.
#[test]
fn test_error_substr_count_too_many_args() {
    expect_error(
        "<?php substr_count(\"hi\", \"i\", 0, 1, 2);",
        "substr_count() takes 2 to 4 arguments",
    );
}

/// Verifies returning `strpos()` (typed `Int|False`) from an `: int` function is rejected:
/// the `false` miss marker must be handled before the scalar return boundary.
#[test]
fn test_error_strpos_false_return_into_int_return_type() {
    expect_error(
        r#"<?php
function pos(): int {
    return strpos("abc", "z");
}
"#,
        "Function 'pos' return type expects Int, got Union([Int, False])",
    );
}

/// Verifies that `str_replace()` with only two arguments produces the correct arity error.
#[test]
fn test_error_str_replace_wrong_args() {
    expect_error(
        "<?php str_replace(\"a\", \"b\");",
        "str_replace() takes exactly 3 arguments",
    );
}

/// Verifies that `sprintf()` with no arguments produces the correct arity error.
#[test]
fn test_error_sprintf_no_args() {
    expect_error("<?php sprintf();", "sprintf() takes at least 1 argument");
}

/// Verifies that `printf()` with no arguments produces the correct arity error.
#[test]
fn test_error_printf_no_args() {
    expect_error("<?php printf();", "printf() takes at least 1 argument");
}

/// Verifies that `ord()` with no arguments produces the correct arity error.
#[test]
fn test_error_ord_wrong_args() {
    expect_error("<?php ord();", "ord() takes exactly 1 argument");
}

/// Verifies that `explode()` with only one argument produces the correct arity
/// error. `explode` accepts 2 or 3 arguments (the trailing `$limit` is optional).
/// Verifies that a pure-data registry builtin (`ord`) infers argument types so that
/// an undefined variable passed as an argument produces the correct diagnostic.
///
/// This is a regression test for Fix B: before the fix, the registry-first dispatch
/// branch skipped `infer_type` for builtins with no check hook, so undefined-variable
/// errors were silently dropped.
#[test]
fn test_error_ord_undefined_variable_arg() {
    expect_error(
        "<?php ord($undeclared);",
        "Undefined variable: $undeclared",
    );
}

/// Verifies that `explode()` with only one argument produces the correct arity error.
#[test]
fn test_error_explode_wrong_args() {
    expect_error("<?php explode(\",\");", "explode() takes 2 or 3 arguments");
}

/// Verifies that the optional `$limit` third argument (`explode(",", $s, 2)`)
/// type-checks cleanly now that the arity accepts 2 or 3 arguments.
#[test]
fn test_explode_with_limit_type_checks() {
    expect_ok("<?php $s = \"a,b,c\"; $p = explode(\",\", $s, 2);");
}

/// Verifies that `str_pad()` with only one argument produces the correct arity error.
#[test]
fn test_error_str_pad_wrong_args() {
    expect_error("<?php str_pad(\"x\");", "str_pad() takes 2 to 4 arguments");
}

/// Verifies that `md5()` with no arguments produces the correct arity error.
/// md5() accepts an optional `$binary` flag, so the message reports 1 or 2 args.
#[test]
fn test_error_md5_wrong_args() {
    expect_error("<?php md5();", "md5() takes 1 or 2 arguments");
}

/// Verifies that `sha1()` with no arguments produces the correct arity error.
/// sha1() accepts an optional `$binary` flag, so the message reports 1 or 2 args.
#[test]
fn test_error_sha1_wrong_args() {
    expect_error("<?php sha1();", "sha1() takes 1 or 2 arguments");
}

/// Verifies that `htmlspecialchars()` with no arguments produces the correct arity
/// error (PHP allows 1–4).
/// Verifies that `htmlspecialchars()` with no arguments produces the correct arity error.
/// htmlspecialchars() accepts optional `$flags` and `$encoding` arguments, so the message
/// reports 1 to 3 args.
#[test]
fn test_error_htmlspecialchars_wrong_args() {
    expect_error(
        "<?php htmlspecialchars();",
        "htmlspecialchars() takes 1 to 4 arguments",
    );
}

/// Verifies that `htmlspecialchars()` rejects five arguments (PHP allows 1–4: string,
/// flags, encoding, double_encode).
#[test]
fn test_error_htmlspecialchars_too_many_args() {
    expect_error(
        r#"<?php htmlspecialchars("x", 3, "UTF-8", true, 1);"#,
        "htmlspecialchars() takes 1 to 4 arguments",
    );
}

/// Verifies that `urlencode()` with no arguments produces the correct arity error.
#[test]
fn test_error_urlencode_wrong_args() {
    expect_error("<?php urlencode();", "urlencode() takes exactly 1 argument");
}

/// Verifies that `base64_encode()` with no arguments produces the correct arity error.
#[test]
fn test_error_base64_encode_wrong_args() {
    expect_error(
        "<?php base64_encode();",
        "base64_encode() takes exactly 1 argument",
    );
}

/// Verifies that `ctype_alpha()` with no arguments produces the correct arity error.
#[test]
fn test_error_ctype_alpha_wrong_args() {
    expect_error(
        "<?php ctype_alpha();",
        "ctype_alpha() takes exactly 1 argument",
    );
}

/// Verifies that `hash()` with only one argument produces the correct arity error.
/// `hash()` now accepts an optional third `$binary` argument, so the message
/// reports the 2-or-3 arity instead of the legacy fixed-2 wording.
#[test]
fn test_error_hash_wrong_args() {
    expect_error(r#"<?php hash("md5");"#, "hash() takes 2 or 3 arguments");
}

/// Verifies the remaining hash-family builtins reject invalid argument counts.
#[test]
fn test_error_hash_family_wrong_args() {
    for (source, message) in [
        (
            r#"<?php hash_hmac("sha256", "data");"#,
            "hash_hmac() takes 3 or 4 arguments",
        ),
        (
            r#"<?php hash_equals("known");"#,
            "hash_equals() takes exactly 2 arguments",
        ),
        (
            "<?php hash_algos(1);",
            "hash_algos() takes no arguments",
        ),
        (
            "<?php hash_init();",
            "hash_init() flags/HASH_HMAC streaming mode is not supported; use hash_hmac() for HMAC",
        ),
        (
            "<?php hash_update();",
            "hash_update() takes exactly 2 arguments",
        ),
        (
            "<?php hash_final();",
            "hash_final() takes 1 or 2 arguments",
        ),
        (
            "<?php hash_copy();",
            "hash_copy() takes exactly 1 argument",
        ),
    ] {
        expect_error(source, message);
    }
}

/// Verifies that `sscanf()` with only one argument produces the correct arity error.
#[test]
fn test_error_sscanf_wrong_args() {
    expect_error(
        r#"<?php sscanf("hi");"#,
        "sscanf() takes at least 2 arguments",
    );
}

// --- v0.5: I/O function errors ---

/// Verifies that `ptr_set()` rejects a string value, since ptr_set only accepts
/// int, bool, null, or pointer. This is an I/O function error regression test.
#[test]
fn test_error_ptr_set_requires_word_value() {
    expect_error(
        "<?php $p = ptr_null(); ptr_set($p, \"hello\");",
        "ptr_set() value must be int, bool, null, or pointer",
    );
}

/// Verifies the invalid-call diagnostic for error long2ip wrong args.
#[test]
fn test_error_long2ip_wrong_args() {
    expect_error("<?php long2ip();", "long2ip() takes exactly 1 argument");
}

/// Verifies the invalid-call diagnostic for error ip2long wrong args.
#[test]
fn test_error_ip2long_wrong_args() {
    expect_error("<?php ip2long();", "ip2long() takes exactly 1 argument");
}

/// Verifies the invalid-call diagnostic for error inet ntop wrong args.
#[test]
fn test_error_inet_ntop_wrong_args() {
    expect_error("<?php inet_ntop();", "inet_ntop() takes exactly 1 argument");
}

/// Verifies the invalid-call diagnostic for error inet pton wrong args.
#[test]
fn test_error_inet_pton_wrong_args() {
    expect_error("<?php inet_pton();", "inet_pton() takes exactly 1 argument");
}

/// Verifies the invalid-call diagnostic for error gzcompress wrong args.
#[test]
fn test_error_gzcompress_wrong_args() {
    expect_error("<?php gzcompress();", "gzcompress() takes 1 or 2 arguments");
}

/// Verifies the invalid-call diagnostic for error gzuncompress wrong args.
#[test]
fn test_error_gzuncompress_wrong_args() {
    expect_error("<?php gzuncompress();", "gzuncompress() takes 1 or 2 arguments");
}

/// Verifies the invalid-call diagnostic for error gzdeflate wrong args.
#[test]
fn test_error_gzdeflate_wrong_args() {
    expect_error("<?php gzdeflate();", "gzdeflate() takes 1 or 2 arguments");
}

/// Verifies the invalid-call diagnostic for error gzinflate wrong args.
#[test]
fn test_error_gzinflate_wrong_args() {
    expect_error("<?php gzinflate();", "gzinflate() takes 1 or 2 arguments");
}

/// Verifies the invalid-call diagnostic for error vsprintf wrong args.
#[test]
fn test_error_vsprintf_wrong_args() {
    expect_error(
        "<?php vsprintf(\"%d\");",
        "vsprintf() takes exactly 2 arguments",
    );
}

/// Verifies the invalid-call diagnostic for error vprintf wrong args.
#[test]
fn test_error_vprintf_wrong_args() {
    expect_error(
        "<?php vprintf(\"%d\", [1], 3);",
        "vprintf() takes exactly 2 arguments",
    );
}

expect_builtin_arity_error!(
    test_error_bindec_wrong_args,
    "<?php bindec();",
    "bindec() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_dechex_wrong_args,
    "<?php dechex();",
    "dechex() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_decoct_wrong_args,
    "<?php decoct();",
    "decoct() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_decbin_wrong_args,
    "<?php decbin();",
    "decbin() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_preg_last_error_msg_wrong_args,
    "<?php preg_last_error_msg(1);",
    "preg_last_error_msg() takes exactly 0 arguments"
);

// -- Recognition-layer coverage for newly registered string builtins --
// These builtins are recognized at type-check time (catalog + signature +
// checker return type + first-class-callable sig); their EIR/runtime lowering
// is deferred, so only type-check recognition is asserted here (no
// compile_and_run, which would fail at the deferred codegen stage).

/// Verifies that `strtr()` type-checks and returns a string in both the 3-arg
/// char-translation form and the 2-arg key=>value map form.
#[test]
fn test_strtr_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = strtr("hello", "el", "ip");
$b = strtr("hi", ["h" => "j"]);
echo $a . $b;
"#
        )
        .is_ok(),
        "strtr() should be recognized in both the 3-arg and 2-arg map forms",
    );
}

/// Verifies that `stripos()`/`strripos()` are recognized and return `int|false`,
/// which flows into an `int` return under gradual typing (false coerces to int),
/// mirroring the existing strpos/strrpos behavior.
#[test]
fn test_stripos_strripos_recognized() {
    assert!(
        check_source(
            r#"<?php
function a(): int { return stripos("Hello", "L"); }
function b(): int { return strripos("Hello", "l", 1); }
"#
        )
        .is_ok(),
        "stripos()/strripos() should be recognized and return int|false",
    );
}

/// Verifies that `strncmp`/`strncasecmp` type-check and return an int.
#[test]
fn test_strncmp_strncasecmp_recognized() {
    assert!(
        check_source(
            r#"<?php
$x = strncmp("abc", "abd", 2);
$y = strncasecmp("ABC", "abd", 2);
echo $x + $y;
"#
        )
        .is_ok(),
        "strncmp()/strncasecmp() should be recognized and return int",
    );
}

/// Verifies that `substr_compare` accepts both its 3-arg and full 5-arg forms.
#[test]
fn test_substr_compare_recognized() {
    assert!(
        check_source(
            r#"<?php
$x = substr_compare("Hello", "llo", 2);
$y = substr_compare("Hello", "LLO", 2, 3, true);
echo $x + $y;
"#
        )
        .is_ok(),
        "substr_compare() should be recognized in its 3-arg and 5-arg forms",
    );
}

/// Verifies that `strip_tags` (1- and 2-arg) and `levenshtein` (2- and 5-arg)
/// type-check.
#[test]
fn test_strip_tags_levenshtein_recognized() {
    assert!(
        check_source(
            r#"<?php
$t = strip_tags("<b>hi</b>");
$t2 = strip_tags("<b>hi</b>", "<b>");
$l = levenshtein("kitten", "sitting");
$l2 = levenshtein("a", "b", 1, 2, 1);
echo $t . $t2 . $l . $l2;
"#
        )
        .is_ok(),
        "strip_tags()/levenshtein() should be recognized",
    );
}

/// Verifies that `parse_str()` accepts an as-yet-undefined by-reference
/// `$result` out-parameter (PHP auto-vivifies it) without a spurious
/// "Undefined variable" diagnostic, and that `$result` is usable afterward —
/// mirroring the preg_match `$matches` out-parameter handling.
#[test]
fn test_parse_str_byref_out_param_recognized() {
    assert!(
        check_source(
            r#"<?php
parse_str("a=1&b=2", $result);
echo $result["a"];
"#
        )
        .is_ok(),
        "parse_str() should define its by-ref $result out-parameter",
    );
}

/// Verifies that `strtr` is usable through first-class-callable syntax so
/// callable-passing call sites (common in Symfony) type-check.
#[test]
fn test_strtr_first_class_callable_recognized() {
    assert!(
        check_source("<?php $f = strtr(...); echo is_callable($f);").is_ok(),
        "strtr(...) first-class callable syntax should type-check",
    );
}

expect_builtin_arity_error!(
    test_error_strtr_wrong_args,
    "<?php strtr(\"abc\", \"a\", \"b\", \"c\");",
    "strtr() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_strncmp_wrong_args,
    "<?php strncmp(\"a\", \"b\");",
    "strncmp() takes exactly 3 arguments"
);

expect_builtin_arity_error!(
    test_error_substr_compare_wrong_args,
    "<?php substr_compare(\"a\", \"b\");",
    "substr_compare() takes 3 to 5 arguments"
);

expect_builtin_arity_error!(
    test_error_levenshtein_wrong_args,
    "<?php levenshtein(\"a\");",
    "levenshtein() takes 2 to 5 arguments"
);

expect_builtin_arity_error!(
    test_error_parse_str_wrong_args,
    "<?php parse_str(\"a=1\");",
    "parse_str() takes exactly 2 arguments"
);

/// Verifies that `parse_str()` rejects a non-variable by-ref `$result` argument.
#[test]
fn test_error_parse_str_result_must_be_variable() {
    expect_error(
        "<?php parse_str(\"a=1\", [\"x\"]);",
        "parse_str() parameter $result must be passed a variable",
    );
}

expect_builtin_arity_error!(
    test_error_strval_wrong_args,
    "<?php strval(1, 2);",
    "strval() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_strrchr_wrong_args,
    "<?php strrchr(\"abc\");",
    "strrchr() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_addcslashes_wrong_args,
    "<?php addcslashes(\"abc\");",
    "addcslashes() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_stripcslashes_wrong_args,
    "<?php stripcslashes(\"a\", \"b\");",
    "stripcslashes() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_strnatcmp_wrong_args,
    "<?php strnatcmp(\"a\");",
    "strnatcmp() takes exactly 2 arguments"
);

expect_builtin_arity_error!(
    test_error_strnatcasecmp_wrong_args,
    "<?php strnatcasecmp(\"a\", \"b\", \"c\");",
    "strnatcasecmp() takes exactly 2 arguments"
);
