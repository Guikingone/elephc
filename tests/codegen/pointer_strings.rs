//! Purpose:
//! Regression tests for converting raw pointers to PHP strings: concatenation, interpolation,
//! string casts, `strval()`, printf-family `%s` (literal or runtime format) and string builtins
//! must print the same lowercase "0x..." text as `echo $ptr`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Live addresses change between runs, so those fixtures capture `echo $p` with output
//!   buffering and compare it with each conversion inside the program instead of hard-coding it.
//! - Fixed values built with `ptr_offset(ptr_null(), N)` pin the exact hex digits, which also
//!   covers the linux-x86_64 nibble alignment of `__rt_ptoa`.

use crate::support::*;

/// Tests that every string conversion of a null pointer prints `0x0`, exactly like `echo`.
#[test]
fn test_null_pointer_string_conversions_match_echo() {
    let out = compile_and_run(
        r#"<?php
$n = ptr_null();
echo $n, "\n";
echo "concat: " . $n . "\n";
echo "interp: $n\n";
echo "braces: {$n}\n";
echo (string)$n, "\n";
echo strval($n), "\n";
echo sprintf("[%s]", $n), "\n";
$s = "append:";
$s .= $n;
echo $s, "\n";
$h = <<<EOT
heredoc: $n
EOT;
echo $h, "\n";
print $n;
print "\n";
"#,
    );
    assert_eq!(
        out,
        "0x0\nconcat: 0x0\ninterp: 0x0\nbraces: 0x0\n0x0\n0x0\n[0x0]\nappend:0x0\nheredoc: 0x0\n0x0\n"
    );
}

/// Tests that pointer hex digits are exact for values whose top set bit is anywhere in its nibble.
///
/// Before the fix linux-x86_64 shifted one nibble too few unless the most significant set bit
/// was the lowest bit of its nibble, so `0xf` printed `0x0` and `0x7fffffffffffffff` printed
/// `0xf000000000000000`; the conversions themselves did not compile on any target.
#[test]
fn test_pointer_hex_digits_are_exact_for_every_nibble_alignment() {
    let out = compile_and_run(
        r#"<?php
$values = [1, 0xf, 0x10, 0xff, 0x100, 0x1234, 0x7ffebc14564, 0x7fffffffffffffff, -1];
foreach ($values as $v) {
    $p = ptr_offset(ptr_null(), $v);
    echo $p, " ", "" . $p, " ", "$p", "\n";
}
"#,
    );
    let expected: String = [
        "0x1",
        "0xf",
        "0x10",
        "0xff",
        "0x100",
        "0x1234",
        "0x7ffebc14564",
        "0x7fffffffffffffff",
        "0xffffffffffffffff",
    ]
    .iter()
    .map(|hex| format!("{hex} {hex} {hex}\n"))
    .collect();
    assert_eq!(out, expected);
}

/// Tests that live, typed and extern-returned pointers convert to the text `echo` prints.
///
/// The fixture captures `echo` through output buffering so the varying address never appears
/// in the expected output; each line reports whether one conversion path matched it.
#[test]
fn test_live_pointer_string_conversions_match_echo() {
    let out = compile_and_run(
        r#"<?php
extern function malloc(u64 $size): ptr;
extern function free(ptr $p): void;
extern class Pair { public int $a; public int $b; }

$x = 42;
$p = ptr($x);
$typed = ptr_cast<Pair>(malloc(16));
$heap = malloc(32);
ob_start();
echo $p;
$cases = ["ptr" => ob_get_clean()];
ob_start();
echo $typed;
$cases["typed"] = ob_get_clean();
ob_start();
echo $heap;
$cases["extern"] = ob_get_clean();
echo strlen($cases["ptr"]) > 3 && str_starts_with($cases["ptr"], "0x") ? "hex\n" : "not hex\n";
echo ("" . $p) === $cases["ptr"] ? "concat ok\n" : "concat bad\n";
echo "$p" === $cases["ptr"] ? "interp ok\n" : "interp bad\n";
echo (string)$p === $cases["ptr"] ? "cast ok\n" : "cast bad\n";
echo strval($p) === $cases["ptr"] ? "strval ok\n" : "strval bad\n";
echo sprintf("%s", $p) === $cases["ptr"] ? "sprintf ok\n" : "sprintf bad\n";
echo ("" . $typed) === $cases["typed"] ? "typed ok\n" : "typed bad\n";
echo "{$heap}" === $cases["extern"] ? "extern ok\n" : "extern bad\n";
echo str_contains("at " . $heap . "!", $cases["extern"]) ? "builtin ok\n" : "builtin bad\n";
free($heap);
free($typed);
"#,
    );
    assert_eq!(
        out,
        "hex\nconcat ok\ninterp ok\ncast ok\nstrval ok\nsprintf ok\ntyped ok\nextern ok\nbuiltin ok\n"
    );
}

/// Tests that printf-family calls with a runtime format string print pointers like `echo`.
///
/// Only a literal format let the backend pick the `%s` string conversion; a format held in a
/// local or a parameter packed the raw pointer type and failed with "sprintf format argument
/// PHP type Pointer". The two pointers in one call also check that each "0x..." operand keeps
/// its own concat scratch bytes.
#[test]
fn test_pointer_printf_family_with_runtime_format_matches_echo() {
    let out = compile_and_run(
        r#"<?php
function fmt_ptr(string $format, ptr $p): string {
    return sprintf($format, $p);
}
$fmt = "%s";
$pair = "%s|%s";
$n = ptr_null();
$f = ptr_offset($n, 255);
echo sprintf($fmt, $n), "\n";
echo sprintf($pair, $f, $n), "\n";
echo fmt_ptr("<%s>", $f), "\n";
$count = printf($fmt, $f);
echo "\n", $count, "\n";
echo sprintf("%s-%s", $f, $n), "\n";
"#,
    );
    assert_eq!(out, "0x0\n0xff|0x0\n<0xff>\n0xff\n4\n0xff-0x0\n");
}

/// Tests that a pointer formatted in the middle of a concatenation keeps the earlier pieces.
///
/// `__rt_ptoa` used to write at the start of the concat scratch buffer, which is where a
/// partially built concatenation lives; it now appends at the scratch cursor like `__rt_itoa`.
#[test]
fn test_pointer_in_the_middle_of_a_concatenation_keeps_earlier_pieces() {
    let out = compile_and_run(
        r#"<?php
$n = ptr_null();
$f = ptr_offset($n, 255);
$s = "1" . 2 . $n . 3 . $f . "z";
echo $s, "\n";
echo "a" . 12 . "b" . $f . "c" . $n, "\n";
"#,
    );
    assert_eq!(out, "120x030xffz\na12b0xffc0x0\n");
}

/// Tests that pointer string conversions keep the heap clean under heap debugging.
#[test]
fn test_pointer_string_conversions_are_leak_free() {
    let output = compile_and_run_with_heap_debug(
        r#"<?php
$x = 7;
$p = ptr($x);
$parts = [];
for ($i = 0; $i < 4; $i++) {
    $parts[] = "p" . $i . "=" . $p;
}
$joined = implode(",", $parts);
echo strlen($joined) > 0 ? "ok" : "empty", "\n";
echo "null=" . ptr_null(), "\n";
"#,
    );
    assert!(output.success, "{}\n{}", output.stdout, output.stderr);
    assert_eq!(output.stdout, "ok\nnull=0x0\n");
    assert!(output.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", output.stderr);
}
