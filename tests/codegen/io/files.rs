//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of I/O files, including file put get contents, file get contents missing emits runtime warning, and file get contents missing is strict false.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies `file_put_contents` writes data and `file_get_contents` reads it back identically.
/// Fixture: creates `test.txt` with "hello world" via put, reads it back, asserts equality.
/// Cleans up the temp directory after the test.
#[test]
fn test_file_put_get_contents() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("test.txt", "hello world");
echo file_get_contents("test.txt");
"#,
    );
    assert_eq!(out, "hello world");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `file_get_contents` on a missing file emits a runtime warning to stderr and continues execution.
/// Fixture: tries to read "missing.txt" which does not exist.
/// Asserts: program exits successfully, stdout is "after" (execution continued), stderr contains the PHP warning.
/// This is a regression check for missing-file handling to ensure no fatal error is raised.
#[test]
fn test_file_get_contents_missing_emits_runtime_warning() {
    let out = compile_and_run_capture(
        r#"<?php
echo file_get_contents("missing.txt");
echo "after";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "after");
    assert!(
        out.stderr.contains("Warning: file_get_contents()"),
        "expected runtime warning, got stderr={}",
        out.stderr
    );
}

/// Verifies `file_get_contents` on a missing file returns strict `false` (not a falsy value).
/// Fixture: reads "missing.txt" with error suppression (`@`), stores result, compares with `=== false`.
/// Asserts: stdout is "false" (identity check passes), stderr is empty.
/// Covers the PHP semantics where missing file returns `false` not `""` or `0`.
#[test]
fn test_file_get_contents_missing_is_strict_false() {
    let out = compile_and_run_capture(
        r#"<?php
$value = @file_get_contents("missing.txt");
echo $value === false ? "false" : "string";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    assert_eq!(out.stderr, "");
}

/// Verifies `file_get_contents` on an existing file returns a truthy value, not `false`.
/// Fixture: creates `test.txt` with empty string via `file_put_contents`, then reads it back.
/// Asserts: identity comparison `$value === false` is false, confirming a string (not false) is returned.
/// Regression check: success path must not incorrectly return `false`.
#[test]
fn test_file_get_contents_success_is_not_false() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("test.txt", "");
$value = file_get_contents("test.txt");
echo $value === false ? "false" : "string";
"#,
    );
    assert_eq!(out, "string");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `file_exists` returns true for existing files and false for non-existent files.
/// Fixture: creates "exists.txt" with data, checks it; checks "nope.txt" which does not exist.
/// Asserts: "exists.txt" → yes, "nope.txt" → no, combined output is "yesno".
/// Cleans up the temp directory after the test.
#[test]
fn test_file_exists() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("exists.txt", "data");
if (file_exists("exists.txt")) {
    echo "yes";
}
if (!file_exists("nope.txt")) {
    echo "no";
}
"#,
    );
    assert_eq!(out, "yesno");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `filesize` returns the byte length of a file's content.
/// Fixture: creates "size.txt" containing "12345" (5 bytes).
/// Asserts: `filesize("size.txt")` equals 5.
/// Cleans up the temp directory after the test.
#[test]
fn test_filesize() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("size.txt", "12345");
echo filesize("size.txt");
"#,
    );
    assert_eq!(out, "5");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `is_file` and `is_dir` return correct booleans for files and directories.
/// Fixture: creates "afile.txt" and "adir" directory; checks both with is_file/is_dir and their negations.
/// Asserts: is_file("afile.txt")=true, is_dir("afile.txt")=false, is_dir("adir")=true, is_file("adir")=false.
/// Output sequence: "F!DD!F" (file→F, not dir→!D, dir→D, not file→!F).
/// Cleans up the directory (rmdir "adir") after the test.
#[test]
fn test_is_file_is_dir() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("afile.txt", "x");
mkdir("adir");
if (is_file("afile.txt")) { echo "F"; }
if (!is_dir("afile.txt")) { echo "!D"; }
if (is_dir("adir")) { echo "D"; }
if (!is_file("adir")) { echo "!F"; }
rmdir("adir");
"#,
    );
    assert_eq!(out, "F!DD!F");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `file()` reads a file and returns an array of lines (without newlines).
/// Fixture: creates "lines.txt" with "one\ntwo\nthree\n" (3 lines + trailing newline).
/// Asserts: `count($lines)` equals 3. Uses `unlink` to remove the file, then cleans up the temp dir.
#[test]
fn test_file_lines() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("lines.txt", "one\ntwo\nthree\n");
$lines = file("lines.txt");
echo count($lines);
unlink("lines.txt");
"#,
    );
    assert_eq!(out, "3");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `is_readable` and `is_writable` return true for a file the process can access.
/// Fixture: creates "perm.txt" with content, checks both predicates, then deletes it.
/// Asserts: "R" (readable) and "W" (writable) are both printed.
/// Platform assumption: current user has read/write permissions on the temp file.
/// Cleans up after the test by deleting the file.
#[test]
fn test_is_readable_writable() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("perm.txt", "x");
if (is_readable("perm.txt")) { echo "R"; }
if (is_writable("perm.txt")) { echo "W"; }
unlink("perm.txt");
"#,
    );
    assert_eq!(out, "RW");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `filemtime` returns a Unix timestamp greater than 1 billion for a recently created file.
/// Fixture: creates "ts.txt" with content, reads its modification time, asserts it is > 1,000,000,000.
/// Asserts: output is "ok". Uses `unlink` to remove the file, then cleans up the temp directory.
/// Regression check: filemtime must not return -1 or an invalid value for a freshly created file.
#[test]
fn test_filemtime() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("ts.txt", "x");
$t = filemtime("ts.txt");
if ($t > 1000000000) { echo "ok"; }
unlink("ts.txt");
"#,
    );
    assert_eq!(out, "ok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `file()`'s `$flags` bitmask over every combination PHP distinguishes.
///
/// The fixture writes a file with two empty lines so `FILE_IGNORE_NEW_LINES` and
/// `FILE_SKIP_EMPTY_LINES` are separable: PHP applies the newline trimming FIRST, so
/// `FILE_SKIP_EMPTY_LINES` alone drops nothing (a bare `"\n"` line still has length 1). Each line
/// is reported as `index/strlen/trimmed-content` so the trailing-terminator handling is visible.
/// `FILE_USE_INCLUDE_PATH` is accepted and has no effect, matching PHP's default empty
/// `include_path`. The expected values are verbatim `LC_ALL=C php` output from PHP 8.4.20.
#[test]
fn test_file_flags_combinations() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("f1.txt", "alpha\n\nbeta\n\ngamma");
function dump($label, $lines) { echo $label, ":", count($lines); foreach ($lines as $i => $l) { echo " ", $i, "/", strlen($l), "/", rtrim($l, "\r\n"); } echo "|"; }
dump("plain", file("f1.txt"));
dump("ignore", file("f1.txt", FILE_IGNORE_NEW_LINES));
dump("skip", file("f1.txt", FILE_SKIP_EMPTY_LINES));
dump("both", file("f1.txt", FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES));
dump("incpath", file("f1.txt", FILE_USE_INCLUDE_PATH));
unlink("f1.txt");
"#,
    );
    assert_eq!(
        out,
        "plain:5 0/6/alpha 1/1/ 2/5/beta 3/1/ 4/5/gamma|\
ignore:5 0/5/alpha 1/0/ 2/4/beta 3/0/ 4/5/gamma|\
skip:5 0/6/alpha 1/1/ 2/5/beta 3/1/ 4/5/gamma|\
both:3 0/5/alpha 1/4/beta 2/5/gamma|\
incpath:5 0/6/alpha 1/1/ 2/5/beta 3/1/ 4/5/gamma|"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `file()` accepts its `$flags` as a named argument and as a run-time value.
///
/// The flag is a plain bitmask rather than a shape-changing literal, so a variable must work.
#[test]
fn test_file_flags_named_and_runtime() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("f1.txt", "alpha\n\nbeta\n\ngamma");
function dump($label, $lines) { echo $label, ":", count($lines); foreach ($lines as $i => $l) { echo " ", $i, "/", strlen($l), "/", rtrim($l, "\r\n"); } echo "|"; }
dump("named", file(filename: "f1.txt", flags: FILE_IGNORE_NEW_LINES));
$f = FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES;
dump("runtime", file("f1.txt", $f));
unlink("f1.txt");
"#,
    );
    assert_eq!(
        out,
        "named:5 0/5/alpha 1/0/ 2/4/beta 3/0/ 4/5/gamma|runtime:3 0/5/alpha 1/4/beta 2/5/gamma|"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `FILE_IGNORE_NEW_LINES` removes a CRLF pair, not just the line feed.
#[test]
fn test_file_flags_strip_crlf() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("f2.txt", "a\r\nb\r\n\r\nc");
function dump($label, $lines) { echo $label, ":", count($lines); foreach ($lines as $i => $l) { echo " ", $i, "/", strlen($l), "/", rtrim($l, "\r\n"); } echo "|"; }
dump("crlf", file("f2.txt", FILE_IGNORE_NEW_LINES));
dump("crlfboth", file("f2.txt", FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES));
unlink("f2.txt");
"#,
    );
    assert_eq!(out, "crlf:4 0/1/a 1/1/b 2/0/ 3/1/c|crlfboth:3 0/1/a 1/1/b 2/1/c|");
    let _ = fs::remove_dir_all(&dir);
}
