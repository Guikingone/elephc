//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of strings misc, including escaped dollar.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Tests that a backslash-escaped dollar (`\$`) inside a double-quoted string is
/// treated as a literal dollar sign, matching PHP's escape sequence behavior.
#[test]
fn test_string_escaped_dollar() {
    let out = compile_and_run(r#"<?php echo "price is \$5";"#);
    assert_eq!(out, "price is $5");
}

/// Tests that a multibyte UTF-8 string literal preceding ASCII digits round-trips
/// correctly through the compiler with no byte-level corruption or digit mishandling.
#[test]
fn test_multibyte_string_literal_before_ascii_digits_round_trips() {
    let out = compile_and_run("<?php echo '日本語123';");
    assert_eq!(out, "日本語123");
}

/// Verifies compiled PHP output for string control escape sequences.
#[test]
fn test_string_control_escape_sequences() {
    // \r, \v, \e, \f process to their ASCII control bytes (regression for
    // the lexer previously emitting a literal backslash for these escapes).
    let out = compile_and_run(
        r#"<?php echo strlen("a\r\nb") . "|" . ord("\r") . "|" . ord("\v") . "|" . ord("\e") . "|" . ord("\f");"#,
    );
    assert_eq!(out, "4|13|11|27|12");
}

// --- string offset assignment ($s[$i] = $c) ---

/// Verifies a basic string offset assignment replaces the byte at the given offset.
#[test]
fn test_string_offset_assign_basic() {
    let out = compile_and_run(r#"<?php $s = "abc"; $s[1] = "X"; echo $s;"#);
    assert_eq!(out, "aXc");
}

/// Verifies an offset past the end right-pads with spaces up to the offset (PHP semantics).
#[test]
fn test_string_offset_assign_pads_with_spaces() {
    let out = compile_and_run(r#"<?php $s = "ab"; $s[5] = "Z"; echo strlen($s), "|", $s;"#);
    assert_eq!(out, "6|ab   Z");
}

/// Verifies the write is copy-on-write: mutating one alias must not change the other.
///
/// `$b = $a` copies the string; `$a[0] = "X"` must produce a fresh value for `$a` while
/// `$b` keeps the original. A shared-string in-place mutation would print `Xbc Xbc`.
#[test]
fn test_string_offset_assign_is_copy_on_write() {
    let out = compile_and_run(r#"<?php $a = "abc"; $b = $a; $a[0] = "X"; echo $a, " ", $b;"#);
    assert_eq!(out, "Xbc abc");
}

/// Verifies only the first byte of a multibyte replacement is written (PHP semantics).
#[test]
fn test_string_offset_assign_uses_first_byte_only() {
    let out = compile_and_run(r#"<?php $s = "abc"; $s[0] = "YZ"; echo $s;"#);
    assert_eq!(out, "Ybc");
}

/// Verifies a negative offset writes relative to the end of the string.
#[test]
fn test_string_offset_assign_negative_offset() {
    let out = compile_and_run(r#"<?php $s = "abc"; $s[-1] = "X"; echo $s;"#);
    assert_eq!(out, "abX");
}

/// Verifies writing past the end of an empty string pads with spaces from offset zero.
#[test]
fn test_string_offset_assign_from_empty_pads() {
    let out = compile_and_run(r#"<?php $s = ""; $s[3] = "Z"; echo strlen($s), "|", $s;"#);
    assert_eq!(out, "4|   Z");
}

/// Verifies a boxed-Mixed offset (a widened `$s[++$j]` counter) coerces to int and writes.
///
/// This mirrors the `Symfony\Component\VarDumper\Dumper\AbstractDumper` UTF-8 encoder loop,
/// where the offset is typed `Mixed` and advanced with a pre-increment inside the index.
#[test]
fn test_string_offset_assign_mixed_offset_loop() {
    let out = compile_and_run(
        r#"<?php
function build(string $s): string {
    $s .= $s;
    $len = strlen($s);
    for ($i = 0, $j = 0; $i < $len; ++$i, ++$j) {
        $s[$j] = "A";
        $s[++$j] = "B";
    }
    return substr($s, 0, $j);
}
echo build("hi");"#,
    );
    assert_eq!(out, "ABABABAB");
}

// --- md5 / sha1 ---
