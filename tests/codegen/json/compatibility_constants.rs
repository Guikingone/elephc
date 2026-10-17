//! Purpose:
//! Covers the remaining PHP JSON constants and their observable encoder behavior.
//!
//! Called from:
//! - `cargo test --test codegen_tests json::compatibility_constants`.
//!
//! Key details:
//! - Fixed PHP-compatible expectations cover native compilation; interpreter tests cover eval.
//! - The line-terminator flag only applies together with unescaped Unicode.

use super::*;

/// Both missing constants must be visible through namespaced native lookup APIs.
#[test]
fn test_json_compatibility_constants_lookup() {
    let out = compile_and_run(
        r#"<?php
namespace JsonConstants;
echo \JSON_UNESCAPED_LINE_TERMINATORS . ":" . JSON_ERROR_NON_BACKED_ENUM . "\n";
echo constant("JSON_UNESCAPED_LINE_TERMINATORS") . ":" . constant("JSON_ERROR_NON_BACKED_ENUM") . "\n";
echo defined("JSON_UNESCAPED_LINE_TERMINATORS") ? "Y" : "N";
echo defined("JSON_ERROR_NON_BACKED_ENUM") ? "Y" : "N";
echo "\n";
"#,
    );
    assert_eq!(out, "2048:11\n2048:11\nYY\n");
}

/// Separators stay escaped unless both unescaped flags are set, including nested keys.
#[test]
fn test_json_unescaped_line_terminators_native() {
    let body = r#"
$text = "é\u{2028}\u{2029}😀";
foreach ([0, JSON_UNESCAPED_LINE_TERMINATORS, JSON_UNESCAPED_UNICODE,
          JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS] as $flags) {
    echo json_encode($text, $flags) . "\n";
    echo json_encode([$text => [$text]], $flags) . "\n";
}
"#;
    let source = format!("<?php\n{body}");
    let escaped = r#""\u00e9\u2028\u2029\ud83d\ude00""#;
    let unicode = "\"é\\u2028\\u2029😀\"";
    let literal = "\"é\u{2028}\u{2029}😀\"";
    let expected: String = [escaped, escaped, unicode, literal].iter()
        .map(|text| format!("{text}\n{{{text}:[{text}]}}\n"))
        .collect();
    assert_eq!(compile_and_run(&source), expected);
}

/// UTF-8 sanitization still respects the line-terminator flags for valid scalars.
#[test]
fn test_json_unescaped_line_terminators_with_utf8_flags() {
    let out = compile_and_run(
        r#"<?php
$text = "x\u{2028}\u{2029}y";
foreach ([JSON_INVALID_UTF8_IGNORE, JSON_INVALID_UTF8_SUBSTITUTE] as $sanitize) {
    echo json_encode($text, $sanitize | JSON_UNESCAPED_UNICODE) . "\n";
    echo json_encode($text, $sanitize | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_LINE_TERMINATORS) . "\n";
}
"#,
    );
    assert_eq!(out, "\"x\\u2028\\u2029y\"\n\"x\u{2028}\u{2029}y\"\n".repeat(2));
}

/// Unit enum encoding reports code 11, uses zero in partial output, and can throw.
#[test]
fn test_json_non_backed_enum_errors_native() {
    let body = r#"
enum JsonUnit { case Ready; }
var_dump(json_encode(JsonUnit::Ready));
echo json_last_error() . ":" . json_last_error_msg() . "\n";
echo json_encode([JsonUnit::Ready], JSON_PARTIAL_OUTPUT_ON_ERROR) . ":" . json_last_error() . "\n";
echo json_encode(["state" => JsonUnit::Ready], JSON_PARTIAL_OUTPUT_ON_ERROR | JSON_THROW_ON_ERROR) . "\n";
try {
    json_encode(JsonUnit::Ready, JSON_THROW_ON_ERROR);
} catch (JsonException $e) {
    echo $e->getCode() . ":" . $e->getMessage() . "\n";
}
echo json_encode("ok") . ":" . json_last_error() . "\n";
"#;
    let source = format!("<?php{body}");
    let expected = "bool(false)\n11:Non-backed enums have no default serialization\n[0]:11\n{\"state\":0}\n11:Non-backed enums have no default serialization\n\"ok\":0\n";
    assert_eq!(compile_and_run(&source), expected);

}

/// A custom JsonSerializable implementation retains precedence over the unit enum error.
#[test]
fn test_json_non_backed_enum_custom_serialization() {
    let out = compile_and_run(
        r#"<?php
enum JsonCustomUnit implements JsonSerializable {
    case Ready;
    public function jsonSerialize(): mixed { return "ready"; }
}
echo json_encode(JsonCustomUnit::Ready, JSON_THROW_ON_ERROR) . ":" . json_last_error();
"#,
    );
    assert_eq!(out, "\"ready\":0");
}
