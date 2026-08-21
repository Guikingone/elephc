//! Purpose:
//! Integration tests for an EXPLICIT `null` passed to a builtin parameter php spells
//! `?T $x = null`. php reads it as "use the default", which for a trailing argument is exactly
//! the shorter call — not as the zero value of the parameter's declared scalar type.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - The lowering used to COERCE such a null into the declared scalar's zero, because it read
//!   `param.ty` and not `param.default`. That coercion is correct for a NON-nullable scalar
//!   (`strlen(null)` is `int(0)` in php, after a deprecation) and silently wrong for a nullable
//!   one: `substr("hello", 1, null)` answered `""`, `stream_get_contents($h, null)` answered
//!   `""`, and `stream_copy_to_stream($a, $b, null)` copied nothing. Nothing warned.
//! - Two shapes are pinned separately because two different mechanisms serve them. A TRAILING
//!   null is dropped in lowering, so the builtin takes the omitted-argument branch it already
//!   has. A null in a MIDDLE position cannot be dropped — a later argument still needs its
//!   position — so it reaches codegen, which materialises the word that site reads as "no bound".
//!   `stream_copy_to_stream($a, $b, null, 4)` is the case that only the second mechanism covers.
//! - Every expectation was MEASURED on `php -n` 8.5.6.
//! - The stream cases use `php://memory` rather than a temporary file, so the suite carries no
//!   filesystem state and the rule is what is under test.

use crate::support::*;

/// Verifies a trailing null length reads to the end of the string, not to byte zero.
///
/// `php -n` 8.5.6 answers `"ello"`. The coercion made this `substr("hello", 1, 0)`, whose `""`
/// is a legitimate answer to a different call — which is why nothing looked wrong.
#[test]
fn test_substr_with_a_null_length_reads_to_the_end() {
    let out = compile_and_run_capture("<?php var_dump(substr(\"hello\", 1, null));\n");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(4) \"ello\"\n");
}

/// Verifies the same rule holds when the offset is negative, so the fix is about the LENGTH.
#[test]
fn test_substr_with_a_negative_offset_and_null_length() {
    let out = compile_and_run_capture("<?php var_dump(substr(\"hello\", -3, null));\n");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(3) \"llo\"\n");
}

/// Verifies a null length on `stream_get_contents` reads the whole stream.
#[test]
fn test_stream_get_contents_with_a_null_length_reads_everything() {
    let out = compile_and_run_capture(
        r#"<?php
$h = fopen("php://memory", "w+");
fwrite($h, "one\ntwo\n");
rewind($h);
var_dump(stream_get_contents($h, null));
fclose($h);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(8) \"one\ntwo\n\"\n");
}

/// Verifies a null length on `stream_copy_to_stream` copies to EOF.
#[test]
fn test_stream_copy_to_stream_with_a_null_length_copies_everything() {
    let out = compile_and_run_capture(
        r#"<?php
$a = fopen("php://memory", "w+");
fwrite($a, "one\ntwo\n");
rewind($a);
$b = fopen("php://memory", "w+");
var_dump(stream_copy_to_stream($a, $b, null));
rewind($b);
var_dump(stream_get_contents($b));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(8)\nstring(8) \"one\ntwo\n\"\n");
}

/// Verifies a null length FOLLOWED by an offset still means "no bound".
///
/// This is the shape the trailing-null drop cannot reach: the `4` after it holds the position, so
/// the null has to survive into codegen and be materialised there. `php -n` 8.5.6 copies the four
/// bytes from offset 4 to the end.
#[test]
fn test_stream_copy_to_stream_with_a_null_length_and_an_offset() {
    let out = compile_and_run_capture(
        r#"<?php
$a = fopen("php://memory", "w+");
fwrite($a, "one\ntwo\n");
rewind($a);
$b = fopen("php://memory", "w+");
var_dump(stream_copy_to_stream($a, $b, null, 4));
rewind($b);
var_dump(stream_get_contents($b));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(4)\nstring(4) \"two\n\"\n");
}

/// Verifies `fgets($h, null)` reads a whole line where it used to REFUSE the program.
///
/// The refusal — `unsupported EIR backend feature: fgets length for PHP type Void` — is the loud
/// half of the same defect: forwarding an optional argument, which is what php's own
/// `?int $length = null` signature invites, did not compile at all.
#[test]
fn test_fgets_with_a_null_length_reads_a_line() {
    let out = compile_and_run_capture(
        r#"<?php
$h = fopen("php://memory", "w+");
fwrite($h, "one\ntwo\n");
rewind($h);
var_dump(fgets($h, null));
fclose($h);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "string(4) \"one\n\"\n");
}

/// Verifies a non-nullable scalar parameter still gets php's null coercion.
///
/// This is the rule the fix must NOT break: `strlen(null)` is `int(0)` in php because `$string`
/// is a plain `string`, with no null default. A fix that skipped every null would turn this into
/// a compile failure.
///
/// php ALSO prints `Deprecated: strlen(): Passing null to parameter #1 ($string) of type string is
/// deprecated`, which elephc does not — a separate, already-recorded gap, so only the VALUE is
/// asserted here. Asserting the diagnostics too would make this test about the notice.
#[test]
fn test_a_non_nullable_scalar_parameter_still_coerces_null() {
    let out = compile_and_run_capture("<?php $x = null; var_dump(strlen($x));\n");
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(0)\n");
}

/// Verifies a trailing null on a non-stream builtin takes the omitted-argument branch too.
///
/// `umask(null)` READS the mask in php; the coercion made it `umask(0)`, which SETS it to zero —
/// a wrong answer and a process-wide side effect. It refused to compile in between, which is how
/// it was found.
#[test]
fn test_umask_with_a_null_mask_reads_rather_than_sets() {
    let out = compile_and_run_capture(
        r#"<?php
$before = umask(null);
var_dump($before === umask(null));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "bool(true)\n");
}
