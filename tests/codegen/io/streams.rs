//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of I/O streams, including stdin constant, stdout constant, and stderr constant.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies STDIN constant evaluates to the expected resource display string.
#[test]
fn test_stdin_constant() {
    let out = compile_and_run("<?php echo STDIN;");
    assert_eq!(out, "Resource id #1");
}

/// Verifies STDOUT constant evaluates to the expected resource display string.
#[test]
fn test_stdout_constant() {
    let out = compile_and_run("<?php echo STDOUT;");
    assert_eq!(out, "Resource id #2");
}

/// Verifies STDERR constant evaluates to the expected resource display string.
#[test]
fn test_stderr_constant() {
    let out = compile_and_run("<?php echo STDERR;");
    assert_eq!(out, "Resource id #3");
}

/// Verifies all three standard stream constants are typed as resources via gettype().
#[test]
fn test_standard_stream_constants_are_resources() {
    let out = compile_and_run(
        r#"<?php
echo gettype(STDIN) . "|";
echo gettype(STDOUT) . "|";
echo gettype(STDERR);
"#,
    );
    assert_eq!(out, "resource|resource|resource");
}

/// Verifies standard stream constants are resolved from the global scope inside a namespace block.
#[test]
fn test_standard_stream_constants_resolve_from_namespace() {
    let out = compile_and_run(
        r#"<?php
namespace App;
echo gettype(STDOUT) . "|";
echo STDOUT;
"#,
    );
    assert_eq!(out, "resource|Resource id #2");
}

/// Verifies fopen() returns a stream resource and that resource-to-string coercion produces the PHP display string.
#[test]
fn test_fopen_returns_stream_resource() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("resource.txt", "w");
echo gettype($f) . "|";
echo $f;
fclose($f);
unlink("resource.txt");
"#,
    );
    assert!(out.starts_with("resource|Resource id #"), "unexpected output: {out}");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fopen() returns false with a warning when opening a non-existent file for reading.
#[test]
fn test_fopen_missing_returns_false_and_warns() {
    let out = compile_and_run_capture(
        r#"<?php
$f = fopen("no_such_file.txt", "r");
echo $f === false ? "false" : "resource";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    // php-src puts the PATH inside the parentheses and the reason after it; the bare
    // `fopen()` this used to assert named neither.
    assert!(
        out.stderr.contains(
            "Warning: fopen(no_such_file.txt): Failed to open stream: No such file or directory"
        ),
        "expected the path and reason in the warning, got stderr={}",
        out.stderr
    );
}

/// Verifies @-suppression prevents the fopen() warning when opening a non-existent file.
#[test]
fn test_error_control_suppresses_fopen_missing_warning() {
    let out = compile_and_run_capture(
        r#"<?php
$f = @fopen("no_such_file.txt", "r");
echo gettype($f) . "|";
echo $f === false ? "false" : "resource";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "boolean|false");
    assert_eq!(out.stderr, "");
}

/// Verifies fopen() returns false for invalid or empty mode strings without emitting a warning.
#[test]
fn test_fopen_invalid_modes_return_false() {
    let out = compile_and_run_capture(
        r#"<?php
$bad = @fopen("bad_mode.txt", "z");
$empty = @fopen("empty_mode.txt", "");
echo ($bad === false ? "z" : "!");
echo ($empty === false ? "e" : "!");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ze");
    assert_eq!(out.stderr, "");
}

/// Verifies a stream resource passed through a mixed-type parameter preserves its resource type.
#[test]
fn test_mixed_file_handle_preserves_resource_type() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
function identity(mixed $value): mixed {
    return $value;
}
$f = fopen("mixed-resource.txt", "w");
$m = identity($f);
echo gettype($m) . "|";
echo $m;
fclose($f);
unlink("mixed-resource.txt");
"#,
    );
    assert!(out.starts_with("resource|Resource id #"), "unexpected output: {out}");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies stream resources use PHP's resource display string ("Resource id #N") in string concatenation.
#[test]
fn test_resource_concatenation_uses_php_display_string() {
    let out = compile_and_run("<?php echo \"stream=\" . STDOUT;");
    assert_eq!(out, "stream=Resource id #2");
}

/// Verifies stream resources are truthy and not empty according to PHP semantics, not raw file descriptor zero.
/// STDIN is always truthy even though its underlying fd is 0; regression for raw descriptor-based truthiness.
#[test]
fn test_resource_truthiness_does_not_use_raw_descriptor_zero() {
    let out = compile_and_run(
        r#"<?php
echo (bool)STDIN ? "truthy" : "falsy";
echo "|";
echo empty(STDIN) ? "empty" : "not-empty";
"#,
    );
    assert_eq!(out, "truthy|not-empty");
}

/// Verifies var_dump() emits the correct resource shape: "resource(N) of type (stream)".
#[test]
fn test_var_dump_resource_uses_stream_shape() {
    let out = compile_and_run("<?php var_dump(STDOUT);");
    assert_eq!(out, "resource(2) of type (stream)\n");
}

/// Verifies fopen/fwrite/fclose/fread round-trip: write "test data" to a file and read it back.
#[test]
fn test_fopen_fwrite_fclose_fread() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("rw.txt", "w");
fwrite($f, "test data");
fclose($f);
$f = fopen("rw.txt", "r");
$content = fread($f, 9);
fclose($f);
echo $content;
unlink("rw.txt");
"#,
    );
    assert_eq!(out, "test data");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fgets() reads one line from STDIN when piped input is provided.
#[test]
fn test_fgets_returns_false_at_eof() {
    // Regression: fgets() used to return PhpType::Str unconditionally,
    // so `while (($l = fgets($f)) !== false)` looped forever — the
    // !== false comparison always saw a string. fgets() now boxes its
    // result as Mixed: string on success, PHP false on zero-byte read
    // (EOF with no bytes accumulated).
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://memory", "r+");
fwrite($f, "line1\nline2\nline3\n");
rewind($f);
$count = 0;
while (($l = fgets($f)) !== false) {
    echo $l;
    $count++;
    if ($count > 10) { echo "OVERRUN"; break; }
}
echo "count=$count";
"#,
    );
    assert_eq!(out, "line1\nline2\nline3\ncount=3");
}

/// Verifies compiled PHP output for fgets stdin.
#[test]
fn test_fgets_stdin() {
    let out = compile_and_run_with_stdin(
        r#"<?php
$line = fgets(STDIN);
echo "got: " . $line;
"#,
        "hello\n",
    );
    assert_eq!(out, "got: hello\n");
}

/// Verifies fgets() raises a TypeError when passed false (e.g., from a failed fopen).
#[test]
fn test_fopen_false_stream_use_is_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
 $f = @fopen("no_such_file.txt", "r");
$line = fgets($f);
echo "done";
"#,
    );
    assert!(!out.success, "program unexpectedly succeeded");
    assert!(
        out.stderr.contains("TypeError: fgets()") && out.stderr.contains("false given"),
        "expected fgets TypeError, got stderr={}",
        out.stderr
    );
}

/// Verifies fgets() TypeError reports the actual runtime type when a non-stream is passed.
#[test]
fn test_stream_type_error_reports_runtime_string_type() {
    let out = compile_and_run_capture(
        r#"<?php
function identity(mixed $value): mixed {
    return $value;
}
fgets(identity("not a stream"));
"#,
    );
    assert!(!out.success, "program unexpectedly succeeded");
    assert!(
        out.stderr.contains("TypeError: fgets()") && out.stderr.contains("string given"),
        "expected string TypeError, got stderr={}",
        out.stderr
    );
}

/// Verifies fopen() result can be guarded with a false check before reading from it.
#[test]
fn test_fopen_guarded_resource_path_can_read() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("guarded.txt", "safe");
$f = fopen("guarded.txt", "r");
if ($f === false) {
    echo "fail";
} else {
    echo fread($f, 4);
    fclose($f);
}
unlink("guarded.txt");
"#,
    );
    assert_eq!(out, "safe");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies feof() is not incorrectly set stale when a file descriptor is closed and reopened.
#[test]
fn test_fopen_clears_stale_eof_for_reused_descriptor() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("first.txt", "x");
file_put_contents("second.txt", "y");
$f = fopen("first.txt", "r");
fread($f, 1);
fread($f, 1);
fclose($f);
$g = fopen("second.txt", "r");
echo feof($g) ? "eof" : "not-eof";
fclose($g);
unlink("first.txt");
unlink("second.txt");
"#,
    );
    assert_eq!(out, "not-eof");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fseek() positions and ftell() reports the correct offset; fread reads from the seek position.
#[test]
fn test_fseek_ftell() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("seek.txt", "abcdefghij");
$f = fopen("seek.txt", "r");
$result = fseek($f, 5);
echo $result;
echo ftell($f);
$data = fread($f, 5);
echo $data;
fclose($f);
unlink("seek.txt");
"#,
    );
    assert_eq!(out, "05fghij");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fseek() returns 0 on success and SEEK_SET/SEEK_CUR/SEEK_END constant modes work correctly.
#[test]
fn test_fseek_return_value() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("seek2.txt", "hello world");
$f = fopen("seek2.txt", "r");
$r1 = fseek($f, 0);
echo $r1;
$r2 = fseek($f, 3, 0);
echo $r2;
$r3 = fseek($f, 2, 1);
echo $r3;
echo ftell($f);
fclose($f);
unlink("seek2.txt");
"#,
    );
    assert_eq!(out, "0005");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fseek() clears the EOF flag after reading past end-of-file.
#[test]
fn test_fseek_clears_eof_after_successful_seek() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("seek-eof.txt", "x");
$f = fopen("seek-eof.txt", "r");
fread($f, 1);
fread($f, 1);
echo feof($f) ? "eof" : "not-eof";
fseek($f, 0);
echo "|" . (feof($f) ? "eof" : "not-eof");
fclose($f);
unlink("seek-eof.txt");
"#,
    );
    assert_eq!(out, "eof|not-eof");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fgetcsv() parses a single CSV row and access to the first field.
#[test]
fn test_fgetcsv() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("data.csv", "alice,30,NY\n");
$f = fopen("data.csv", "r");
$row = fgetcsv($f);
echo $row[0];
fclose($f);
unlink("data.csv");
"#,
    );
    assert_eq!(out, "alice");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fputcsv() writes a valid CSV line and file_get_contents() reads it back.
#[test]
fn test_fputcsv() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("out.csv", "w");
$data = ["hello", "world"];
fputcsv($f, $data);
fclose($f);
$content = file_get_contents("out.csv");
echo trim($content);
unlink("out.csv");
"#,
    );
    assert_eq!(out, "hello,world");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies fgetcsv() honors a custom separator.
#[test]
fn test_fgetcsv_custom_separator() {
    let (out, _dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("php://memory", "r+");
fwrite($f, "a;b;c\n1;2;3\n");
rewind($f);
$row1 = fgetcsv($f, 0, ";");
$row2 = fgetcsv($f, 0, ";");
echo $row1[0] . $row1[1] . $row1[2] . "\n";
echo $row2[0] . $row2[1] . $row2[2] . "\n";
"#,
    );
    assert_eq!(out, "abc\n123\n");
}

/// Verifies fgetcsv() honors a custom enclosure character.
#[test]
fn test_fgetcsv_custom_enclosure() {
    let (out, _dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("php://memory", "r+");
fwrite($f, "'a','b,c','d'\n");
rewind($f);
$row = fgetcsv($f, 0, ",", "'");
echo $row[0] . "|" . $row[1] . "|" . $row[2] . "\n";
"#,
    );
    assert_eq!(out, "a|b,c|d\n");
}

/// Verifies fgetcsv() with PHP 8.4 doubling mode (escape="").
#[test]
fn test_fgetcsv_php84_doubling() {
    let (out, _dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("php://memory", "r+");
fwrite($f, "\"a\"\"b\",\"c\"\n");
rewind($f);
$row = fgetcsv($f, 0, ",", "\"", "");
echo $row[0] . "|" . $row[1] . "\n";
"#,
    );
    assert_eq!(out, "a\"b|c\n");
}

/// Verifies fputcsv() honors custom separator and enclosure.
#[test]
fn test_fputcsv_custom_separator_enclosure() {
    let (out, _dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("php://memory", "r+");
fputcsv($f, ["a", "b;c", "d"], ";", "'");
rewind($f);
echo fread($f, 100);
"#,
    );
    assert_eq!(out, "a;'b;c';d\n");
}

/// Verifies fputcsv() honors a custom end-of-line string.
#[test]
fn test_fputcsv_custom_eol() {
    let (out, _dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("php://memory", "r+");
fputcsv($f, ["a", "b"], ",", "\"", "\\", "\r\n");
rewind($f);
echo bin2hex(fread($f, 100));
"#,
    );
    assert_eq!(out, "612c620d0a");
}

/// Verifies fputcsv+fgetcsv round-trip with custom delimiters and doubling mode.
#[test]
fn test_fputcsv_fgetcsv_roundtrip_custom() {
    let (out, _dir) = compile_and_run_in_dir(
        r##"<?php
$f = fopen("php://memory", "r+");
fputcsv($f, ["a;b", 'c"d'], ";", "#", "", "\n");
rewind($f);
$r = fgetcsv($f, 0, ";", "#", "");
echo $r[0] . "|" . $r[1] . "\n";
"##,
    );
    assert_eq!(out, "a;b|c\"d\n");
}

/// Verifies rewind() resets the read position to the start and data can be re-read.
#[test]
fn test_rewind() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("rw.txt", "abcdef");
$f = fopen("rw.txt", "r");
$first = fread($f, 3);
rewind($f);
$again = fread($f, 3);
fclose($f);
echo $first . "|" . $again;
unlink("rw.txt");
"#,
    );
    assert_eq!(out, "abc|abc");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies rewind() clears the EOF flag after reading past end-of-file.
#[test]
fn test_rewind_clears_eof_after_successful_seek() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("rewind-eof.txt", "x");
$f = fopen("rewind-eof.txt", "r");
fread($f, 1);
fread($f, 1);
echo feof($f) ? "eof" : "not-eof";
rewind($f);
echo "|" . (feof($f) ? "eof" : "not-eof");
fclose($f);
unlink("rewind-eof.txt");
"#,
    );
    assert_eq!(out, "eof|not-eof");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies feof() returns true only after reading past the end of a file.
#[test]
fn test_feof() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("eof.txt", "hi");
$f = fopen("eof.txt", "r");
$data = fread($f, 2);
$data = fread($f, 1);
if (feof($f)) { echo "eof"; }
fclose($f);
unlink("eof.txt");
"#,
    );
    assert_eq!(out, "eof");
    let _ = fs::remove_dir_all(&dir);
}

// --- resource & stream introspection (streams/sockets phase 1) ---

/// Verifies compiled PHP output for is resource true for stream.
#[test]
fn test_is_resource_true_for_stream() {
    let out = compile_and_run("<?php var_dump(is_resource(STDIN));");
    assert_eq!(out, "bool(true)\n");
}

/// Verifies compiled PHP output for is resource false for non resource.
#[test]
fn test_is_resource_false_for_non_resource() {
    let out = compile_and_run(
        r#"<?php
echo is_resource(42) ? "y" : "n";
echo is_resource("s") ? "y" : "n";
echo is_resource(null) ? "y" : "n";
"#,
    );
    assert_eq!(out, "nnn");
}

/// Verifies compiled PHP output for get resource type returns stream.
#[test]
fn test_get_resource_type_returns_stream() {
    let out = compile_and_run("<?php echo get_resource_type(STDOUT);");
    assert_eq!(out, "stream");
}

/// Verifies compiled PHP output for get resource id matches display marker.
#[test]
fn test_get_resource_id_matches_display_marker() {
    let out = compile_and_run(
        r#"<?php echo get_resource_id(STDIN) . "|" . get_resource_id(STDOUT) . "|" . get_resource_id(STDERR);"#,
    );
    assert_eq!(out, "1|2|3");
}

/// Verifies compiled PHP output for resource introspection is case insensitive.
#[test]
fn test_resource_introspection_is_case_insensitive() {
    let out = compile_and_run(
        r#"<?php echo IS_RESOURCE(STDIN) ? "y" : "n"; echo Get_Resource_Type(STDIN);"#,
    );
    assert_eq!(out, "ystream");
}

/// Verifies compiled PHP output for stream isatty false for regular file.
#[test]
fn test_stream_isatty_false_for_regular_file() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("tty_probe.txt", "w");
var_dump(stream_isatty($f));
fclose($f);
unlink("tty_probe.txt");
"#,
    );
    assert_eq!(out, "bool(false)\n");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream is local and supports lock are true.
#[test]
fn test_stream_is_local_and_supports_lock_are_true() {
    let out = compile_and_run(
        r#"<?php echo stream_is_local(STDIN) ? "L" : "l"; echo stream_supports_lock(STDIN) ? "S" : "s";"#,
    );
    assert_eq!(out, "LS");
}

/// Verifies `fgetcsv()` ends the manual's own read loop instead of spinning on it.
///
/// The runtime signals end-of-input with a null array pointer. Storing that raw left it
/// reading as `null`, and `null !== false` holds, so
/// `while (($row = fgetcsv($h)) !== false)` — the loop PHP's manual shows — ran forever;
/// a loop that guarded itself fatalled on `count(null)` instead. The counter here is the
/// point: a test that only checked the parsed fields passed throughout.
#[test]
fn test_fgetcsv_reports_false_at_end_of_input() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("csv_eof.csv", "a,b\nc,d\n");
$f = fopen("csv_eof.csv", "r");
$rows = 0;
while (($row = fgetcsv($f, 0, ",", "\"", "\\")) !== false) {
    $rows = $rows + 1;
    if ($rows > 8) { echo "RUNAWAY"; break; }
}
fclose($f);
echo $rows;
unlink("csv_eof.csv");
"#,
    );
    assert_eq!(out, "2");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a row read by `fgetcsv()` can be written straight back by `fputcsv()`.
///
/// This is the pair's whole point, and it is the shape that broke when `fgetcsv()` started
/// reporting `array<string>|false`: the row is stored boxed, and the writer accepted only
/// an unboxed string array, so the read-transform-write pipeline stopped COMPILING. The
/// union is what makes unwrapping safe — it guarantees the payload is a string array.
#[test]
fn test_fgetcsv_row_can_be_written_back_by_fputcsv() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("pipe_in.csv", "1,x\n2,\"y,z\"\n");
$in = fopen("pipe_in.csv", "r");
$out = fopen("pipe_out.csv", "w");
while (($rec = fgetcsv($in, 0, ",", "\"", "\\")) !== false) {
    fputcsv($out, $rec, ",", "\"", "\\");
}
fclose($in);
fclose($out);
echo file_get_contents("pipe_out.csv");
unlink("pipe_in.csv");
unlink("pipe_out.csv");
"#,
    );
    assert_eq!(out, "1,x\n2,\"y,z\"\n");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies writing an end-of-input `fgetcsv()` result raises php-src's own `TypeError`.
#[test]
fn test_fputcsv_rejects_a_false_fields_argument() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("empty_in.csv", "");
$in = fopen("empty_in.csv", "r");
$out = fopen("t_out.csv", "w");
$rec = fgetcsv($in, 0, ",", "\"", "\\");
try {
    fputcsv($out, $rec, ",", "\"", "\\");
} catch (TypeError $e) {
    echo $e->getMessage();
}
fclose($in);
fclose($out);
unlink("empty_in.csv");
unlink("t_out.csv");
"#,
    );
    assert_eq!(
        out,
        "fputcsv(): Argument #2 ($fields) must be of type array, false given"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a `php://filter` chain runs EVERY filter, in order.
///
/// Only the first name was applied, so `read=a|b` silently produced `a`'s output — which
/// looks plausible and is wrong. `convert.base64-encode` and `string.toupper` do not
/// commute, so swapping them proves the ORDER is right rather than just the count.
#[test]
fn test_php_filter_chain_applies_every_filter_in_order() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("fchain.txt", "Hello World");
$a = fopen("php://filter/read=convert.base64-encode|string.toupper/resource=fchain.txt", "r");
echo stream_get_contents($a), "|";
fclose($a);
$b = fopen("php://filter/read=string.toupper|convert.base64-encode/resource=fchain.txt", "r");
echo stream_get_contents($b), "|";
fclose($b);
$c = fopen("php://filter/read=string.toupper|no.such.filter/resource=fchain.txt", "r");
echo stream_get_contents($c);
fclose($c);
unlink("fchain.txt");
"#,
    );
    // The third case pins what an UNKNOWN name does: `php -n` skips it, keeps its
    // neighbours, and still opens. Cancelling the whole chain reads as just as plausible,
    // which is why it is measured rather than reasoned about.
    assert_eq!(out, "SGVSBG8GV29YBGQ=|SEVMTE8gV09STEQ=|HELLO WORLD");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a failed open names WHICH path failed and WHY, as php-src does.
///
/// The message was a bare `fopen(): Failed to open stream` — neither the path nor the
/// reason, which is most of what it exists for when several opens share a line. The
/// remaining difference from PHP is the ` in FILE on line N` suffix elephc never adds.
#[test]
fn test_failed_open_warning_names_the_path_and_the_reason() {
    let out = compile_and_run_capture(
        r#"<?php
$f = fopen("/no/such/dir/missing.txt", "r");
echo $f === false ? "false" : "open";
$c = file_get_contents("/no/such/dir/other.txt");
echo $c === false ? "|false" : "|read";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false|false");
    assert!(
        out.stderr.contains(
            "Warning: fopen(/no/such/dir/missing.txt): Failed to open stream: No such file or directory"
        ),
        "fopen warning lost the path or the reason, got stderr={}",
        out.stderr
    );
    assert!(
        out.stderr.contains(
            "Warning: file_get_contents(/no/such/dir/other.txt): Failed to open stream: No such file or directory"
        ),
        "file_get_contents warning lost the path or the reason, got stderr={}",
        out.stderr
    );
}

/// Verifies a filter name that resolves to nothing is REPORTED, naming the filter.
///
/// Returning `false` silently left a misspelled filter indistinguishable from one that
/// attached — the caller's data simply came through untransformed. php-src names both the
/// function and the filter, and `@` suppresses it like any warning.
#[test]
fn test_stream_filter_attach_warns_and_names_an_unknown_filter() {
    let out = compile_and_run_capture(
        r#"<?php
$h = fopen("php://memory", "w+");
var_dump(stream_filter_append($h, "no.such.filter"));
var_dump(stream_filter_prepend($h, "also.missing"));
var_dump(@stream_filter_append($h, "suppressed.one"));
fclose($h);
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "bool(false)\nbool(false)\nbool(false)\n");
    assert!(
        out.stderr
            .contains("Warning: stream_filter_append(): Unable to locate filter \"no.such.filter\""),
        "missing the append warning, got stderr={}",
        out.stderr
    );
    assert!(
        out.stderr
            .contains("Warning: stream_filter_prepend(): Unable to locate filter \"also.missing\""),
        "missing the prepend warning, got stderr={}",
        out.stderr
    );
    assert!(
        !out.stderr.contains("suppressed.one"),
        "`@` must suppress the warning, got stderr={}",
        out.stderr
    );
}

/// Verifies the CSV family deprecates an OMITTED `$escape`, and only an omitted one.
///
/// PHP 8.5 raises it because 9.0 changes the default from `"\\"` to `""`, which silently
/// changes how existing files parse. It keys on the argument being absent, so passing the
/// default explicitly stays quiet — the count is what pins that: three calls omit it and
/// three pass it, and exactly three notices come out.
#[test]
fn test_csv_family_deprecates_an_omitted_escape_argument() {
    let out = compile_and_run_capture(
        r#"<?php
file_put_contents("dep.csv", "a,b\n");
$r = fopen("dep.csv", "r");
fgetcsv($r);
fgetcsv($r, 0, ",", "\"", "\\");
fclose($r);
$w = fopen("dep_out.csv", "w");
fputcsv($w, ["a"]);
fputcsv($w, ["a"], ",", "\"", "\\");
fclose($w);
str_getcsv("a,b");
str_getcsv("a,b", ",", "\"", "\\");
echo "done";
unlink("dep.csv");
unlink("dep_out.csv");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    let notices = out.stderr.matches("the $escape parameter must be provided").count();
    assert_eq!(notices, 3, "expected three notices, got stderr={}", out.stderr);
    for name in ["fgetcsv", "fputcsv", "str_getcsv"] {
        assert!(
            out.stderr
                .contains(&format!("Deprecated: {name}(): the $escape parameter")),
            "missing the {name} notice, got stderr={}",
            out.stderr
        );
    }
}

/// Verifies `str_getcsv()` parses one record, with a newline as DATA rather than a break.
///
/// It is not `fgetcsv()` over a line, and the difference is not obvious: only a trailing
/// newline is structural, and php-src strips one in two separate places. `"a\nb"` is one
/// field containing a newline; `"a,b\n\n"` still yields two fields because both trailing
/// newlines go. The expectations come from `php -n` 8.5.6.
#[test]
fn test_str_getcsv_treats_an_interior_newline_as_data() {
    let out = compile_and_run(
        r#"<?php
$cases = ["a,b,\"c,d\"", "a,\"b\"\"c\",d", "a\nb", "a,b\n", "a,b\n\n", "\na,b", " \n", "a,b\r\n"];
foreach ($cases as $c) { echo json_encode(str_getcsv($c, ",", "\"", "\\")), "|"; }
"#,
    );
    assert_eq!(
        out,
        "[\"a\",\"b\",\"c,d\"]|[\"a\",\"b\\\"c\",\"d\"]|[\"a\\nb\"]|[\"a\",\"b\"]|[\"a\",\"b\"]|[\"\\na\",\"b\"]|[\" \"]|[\"a\",\"b\"]|"
    );
}

/// Verifies `str_getcsv()` answers the same through `eval()` as it does compiled.
#[test]
fn test_str_getcsv_matches_between_compiled_and_eval() {
    let out = compile_and_run(
        r#"<?php
echo json_encode(str_getcsv("a,\"b,c\",d", ",", "\"", "\\")), "|";
eval('echo json_encode(str_getcsv("a,\"b,c\",d", ",", "\"", "\\\\"));');
"#,
    );
    assert_eq!(out, "[\"a\",\"b,c\",\"d\"]|[\"a\",\"b,c\",\"d\"]");
}

/// Verifies a quoted CSV field may span newlines, as one field of one record.
///
/// The reader took one line at a time, so `1,"line one\nline two"` came back as two
/// records with the field cut in half and a stray quote left on the second — silent
/// corruption of a legal, common export shape. The record count is what pins it: a test
/// that only inspected the first row saw nothing wrong.
#[test]
fn test_fgetcsv_continues_a_quoted_field_across_newlines() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("ml.csv", "id,note\n1,\"line one\nline two\"\n2,plain\n");
$f = fopen("ml.csv", "r");
$rows = 0;
$note = "";
while (($row = fgetcsv($f, 0, ",", "\"", "\\")) !== false) {
    $rows = $rows + 1;
    if ($rows > 8) { echo "RUNAWAY"; break; }
    if ($rows == 2) { $note = $row[1]; }
}
fclose($f);
echo $rows, "|", strlen($note), "|", $note;
unlink("ml.csv");
"#,
    );
    assert_eq!(out, "3|17|line one\nline two");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `fputcsv()` doubles an embedded enclosure instead of backslash-escaping it.
///
/// elephc wrote `"with\"quote"` where PHP writes `"with""quote"` — not valid CSV, and PHP
/// itself reads it back as a different value. php-src also tracks whether the escape
/// character shielded the enclosure: `back\"quote` keeps its single quote rather than
/// gaining a doubled one, and the escape character is never doubled on output. The whole
/// existing fputcsv suite passed either way, because none of it wrote an embedded quote.
#[test]
fn test_fputcsv_doubles_an_embedded_enclosure() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$h = fopen("fp_dq.csv", "w");
fputcsv($h, ["with\"quote"], ",", "\"", "\\");
fputcsv($h, ["a\"b\"c"], ",", "\"", "\\");
fputcsv($h, ["back\\slash"], ",", "\"", "\\");
fputcsv($h, ["back\\\"shielded"], ",", "\"", "\\");
fclose($h);
echo file_get_contents("fp_dq.csv");
unlink("fp_dq.csv");
"#,
    );
    assert_eq!(
        out,
        "\"with\"\"quote\"\n\"a\"\"b\"\"c\"\n\"back\\slash\"\n\"back\\\"shielded\"\n"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `SplFileObject::fgetcsv()` still yields strings after `fgetcsv()` began boxing.
///
/// The SPL method body is synthesized, so it has no checked call-site type and takes the
/// EIR fallback instead. While that fallback still claimed `array<string>`, the boxed
/// `array|false` cell was read as a raw array pointer and every field came back as an
/// integer — a silent corruption no `fgetcsv()` test could see.
#[test]
fn test_spl_file_object_fgetcsv_reads_fields_not_pointers() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("spl_csv.csv", "a,b\nc,d\n");
$f = new SplFileObject("spl_csv.csv");
$seen = "";
while (!$f->eof()) {
    $row = $f->fgetcsv(",", "\"", "\\");
    if ($row === false) { break; }
    foreach ($row as $field) { $seen = $seen . $field; }
}
unset($f);
echo $seen;
unlink("spl_csv.csv");
"#,
    );
    assert_eq!(out, "abcd");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a refused write reports failure rather than its errno.
///
/// macOS returns a failed `write` as the POSITIVE errno with the carry flag set, which is
/// indistinguishable from a byte count: writing to a read-only handle answered `int(9)`
/// — EBADF — where PHP answers `false`. Asserting on the exact value matters, because
/// `9` is truthy and every `if (fwrite(...))` guard read it as success.
#[test]
fn test_fwrite_to_a_read_only_stream_reports_false() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("fw_ro.txt", "seed");
$h = fopen("fw_ro.txt", "r");
var_dump(@fwrite($h, "XY"));
fclose($h);
echo file_get_contents("fw_ro.txt");
unlink("fw_ro.txt");
"#,
    );
    assert_eq!(out, "bool(false)\nseed");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_is_local()` classifies a path that only exists at run time.
///
/// A literal is folded at compile time, so the loop is what exercises the runtime
/// classifier — before it existed this failed to compile rather than answering wrongly.
/// The expectations are `php -n` 8.5.6's: `data:` is remote with or without slashes,
/// scheme matching folds case, and the scheme needs its full `://`.
#[test]
fn test_stream_is_local_classifies_a_runtime_path() {
    let out = compile_and_run(
        r#"<?php
$cases = [
    "plain.txt", "/etc/hosts", "file:///etc/hosts",
    "http://example.com/x", "https://example.com/x",
    "ftp://example.com/x", "ftps://example.com/x",
    "php://memory", "glob://*.txt", "phar://a.phar/b.txt",
    "compress.zlib://a.gz", "data://text/plain,hello", "data:text/plain,hello",
    "HTTP://example.com/x", "hTTps://example.com", "FTP://x",
    "httpx://x", "http:/one-slash", "http", "my.http://x", "",
];
foreach ($cases as $c) { echo stream_is_local($c) ? "L" : "r"; }
"#,
    );
    assert_eq!(out, "LLLrrrrLLLLrrrrrLLLLL");
}

/// Verifies `stream_supports_lock()` answers per wrapper rather than always true.
///
/// php-src answers from the stream's ops: a descriptor-backed stream carries the lock
/// option, the memory and output wrappers do not. elephc answered a blanket `true`, which
/// told a caller that `flock()` on `php://memory` would serialise something. A descriptor
/// test cannot decide it, because elephc backs `php://memory` with a real temporary file.
#[test]
fn test_stream_supports_lock_is_false_for_the_memory_wrappers() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("lk.txt", "x");
echo stream_supports_lock(fopen("lk.txt", "r")) ? "L" : "n";
echo stream_supports_lock(fopen("php://memory", "w+")) ? "L" : "n";
echo stream_supports_lock(fopen("php://temp", "w+")) ? "L" : "n";
echo stream_supports_lock(fopen("php://output", "w")) ? "L" : "n";
echo stream_supports_lock(fopen("php://stdout", "w")) ? "L" : "n";
echo stream_supports_lock(tmpfile()) ? "L" : "n";
echo stream_supports_lock(STDIN) ? "L" : "n";
unlink("lk.txt");
"#,
    );
    assert_eq!(out, "LnnnLLL");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream get wrappers lists known wrappers.
#[test]
fn test_stream_get_wrappers_lists_known_wrappers() {
    // Full PHP-published wrapper list (Phase D: surface 100%). ftps,
    // compress.*, phar, glob are accepted at runtime but currently
    // return false from fopen — the listing is the PHP-spec surface.
    let out = compile_and_run(
        r#"<?php $w = stream_get_wrappers(); echo count($w) . ":" . $w[0] . "," . $w[3] . "," . $w[5];"#,
    );
    assert_eq!(out, "11:file,ftp,https");
}

/// Verifies compiled PHP output for stream get transports and filters.
#[test]
fn test_stream_get_transports_and_filters() {
    // The transport list is php-src's exactly: ten entries, tlsv1.0/1.1/1.2/1.3 routing
    // through the same enable_crypto path. `sslv2`/`sslv3` used to be listed and are not
    // any more — PHP 8.5.6 does not publish them and the protocols are dead.
    //
    // The filter list still diverges deliberately: PHP publishes nine WILDCARD names
    // (`zlib.*`, `convert.*`, …) while elephc publishes the fourteen concrete filters it
    // actually registers, so `stream_filter_append()` on any listed name succeeds.
    let out = compile_and_run(
        r#"<?php echo count(stream_get_transports()) . "," . count(stream_get_filters());"#,
    );
    assert_eq!(out, "10,14");
}

/// Verifies compiled PHP output for stream filter rot13 on read.
#[test]
fn test_stream_filter_rot13_on_read() {
    // A read-direction filter transforms bytes as they leave the stream.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "Hello World");
rewind($m);
stream_filter_append($m, "string.rot13", STREAM_FILTER_READ);
echo fread($m, 32);
fclose($m);
"#,
    );
    assert_eq!(out, "Uryyb Jbeyq");
}

/// Verifies compiled PHP output for stream filter toupper on write.
#[test]
fn test_stream_filter_toupper_on_write() {
    // A write-direction filter transforms bytes as they enter the stream.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
stream_filter_append($m, "string.toupper", STREAM_FILTER_WRITE);
fwrite($m, "written lower");
rewind($m);
echo fread($m, 32);
fclose($m);
"#,
    );
    assert_eq!(out, "WRITTEN LOWER");
}

/// Verifies compiled PHP output for php filter read toupper over temp.
#[test]
fn test_php_filter_read_toupper_over_temp() {
    // php://filter/read=F/resource=R opens R and attaches F to the read side.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://filter/read=string.toupper/resource=php://temp", "r+");
fwrite($f, "hello temp");
rewind($f);
echo fread($f, 64);
fclose($f);
"#,
    );
    assert_eq!(out, "HELLO TEMP");
}

/// Verifies compiled PHP output for php filter write rot13 over temp.
#[test]
fn test_php_filter_write_rot13_over_temp() {
    // php://filter/write=F transforms bytes as they enter the stream; reading
    // back raw (no filter) shows the rot13-encoded payload.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://filter/write=string.rot13/resource=php://temp", "r+");
fwrite($f, "hello");
rewind($f);
echo fread($f, 64);
fclose($f);
"#,
    );
    assert_eq!(out, "uryyb");
}

/// Verifies compiled PHP output for php filter bare filter applies to read.
#[test]
fn test_php_filter_bare_filter_applies_to_read() {
    // A bare filter (no read=/write=) is STREAM_FILTER_ALL, so it applies on read.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://filter/string.toupper/resource=php://temp", "r+");
fwrite($f, "both ways");
rewind($f);
echo fread($f, 64);
fclose($f);
"#,
    );
    assert_eq!(out, "BOTH WAYS");
}

/// Verifies compiled PHP output for php filter unknown filter returns unfiltered stream.
#[test]
fn test_php_filter_unknown_filter_returns_unfiltered_stream() {
    // PHP emits a warning but still returns the unfiltered stream for an unknown
    // filter (not false); reads pass through untransformed.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://filter/read=nope.bad/resource=php://temp", "r+");
echo ($f === false) ? "false" : "resource";
fwrite($f, "raw bytes");
rewind($f);
echo "|" . fread($f, 64);
fclose($f);
"#,
    );
    assert_eq!(out, "resource|raw bytes");
}

/// Verifies compiled PHP output for fprintf formats and writes to stream.
#[test]
fn test_fprintf_formats_and_writes_to_stream() {
    // fprintf = sprintf + fwrite: it formats the arguments and writes the result
    // to the stream, returning the byte count.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://temp", "r+");
$n = fprintf($f, "%s=%d (%.2f)", "x", 42, 3.14159);
rewind($f);
echo "n=$n|[" . stream_get_contents($f) . "]";
fclose($f);
"#,
    );
    assert_eq!(out, "n=11|[x=42 (3.14)]");
}

/// Verifies compiled PHP output for fscanf float via shared sscanf engine.
#[test]
fn test_fscanf_float_via_shared_sscanf_engine() {
    // fscanf shares __rt_sscanf, so the new %f branch must work through it too.
    let out = compile_and_run(
        r#"<?php
$g = fopen("php://temp", "r+");
fwrite($g, "9.99\n");
rewind($g);
$row = fscanf($g, "%f");
echo $row[0];
fclose($g);
"#,
    );
    assert_eq!(out, "9.99");
}

/// Verifies compiled PHP output for fscanf reads and parses line by line.
#[test]
fn test_fscanf_reads_and_parses_line_by_line() {
    // fscanf reads one line per call and parses it with the sscanf engine,
    // returning the matched fields as an array (2-argument form).
    let out = compile_and_run(
        r#"<?php
$g = fopen("php://temp", "r+");
fwrite($g, "alice 30\nbob 25\n");
rewind($g);
$r1 = fscanf($g, "%s %d");
echo $r1[0] . "=" . $r1[1] . "|";
$r2 = fscanf($g, "%s %d");
echo $r2[0] . "=" . $r2[1];
fclose($g);
"#,
    );
    assert_eq!(out, "alice=30|bob=25");
}

/// Verifies compiled PHP output for fprintf inside function returns int.
#[test]
fn test_fprintf_inside_function_returns_int() {
    // Exercises local-type inference: the fprintf result assigned to a local
    // inside a function must be an 8-byte Int slot (not a 16-byte str slot).
    let out = compile_and_run(
        r#"<?php
function emit($f): int { $n = fprintf($f, "[%d]", 7); return $n; }
$f = fopen("php://temp", "r+");
$c = emit($f);
rewind($f);
echo $c . ":" . stream_get_contents($f);
fclose($f);
"#,
    );
    assert_eq!(out, "3:[7]");
}

/// Verifies compiled PHP output for stream filter prepend and remove.
#[test]
fn test_stream_filter_prepend_and_remove() {
    // stream_filter_prepend attaches a filter; stream_filter_remove drops that one
    // filter and leaves the rest of the chain attached.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
stream_filter_prepend($m, "string.tolower", STREAM_FILTER_READ);
fwrite($m, "FIRST PASS");
rewind($m);
echo fread($m, 32);
echo "|";
$f = stream_filter_append($m, "string.rot13", STREAM_FILTER_READ);
stream_filter_remove($f);
rewind($m);
echo fread($m, 32);
fclose($m);
"#,
    );
    // The prepended `string.tolower` survives removing the appended `string.rot13`,
    // so the second read is still lowercased. The previous expectation of
    // "FIRST PASS" encoded the old two-slot table, whose removal cleared every
    // slot on the descriptor and so detached unrelated filters. Verified against
    // the PHP 8.5.6 CLI, which prints "first pass|first pass".
    assert_eq!(out, "first pass|first pass");
}

/// Verifies compiled PHP output for stream filter zlib deflate compresses.
#[test]
fn test_stream_filter_zlib_deflate_compresses() {
    // The zlib.deflate write filter deflate-compresses data into the stream;
    // the compressed output is non-empty and shorter than the input.
    let out = compile_and_run(
        r#"<?php
$w = fopen("zlib_filter_out.bin", "w");
stream_filter_append($w, "zlib.deflate", STREAM_FILTER_WRITE);
$data = str_repeat("stream filter compression test ", 30);
fwrite($w, $data);
fclose($w);
$packed = file_get_contents("zlib_filter_out.bin");
echo (strlen($packed) > 0 && strlen($packed) < strlen($data)) ? "compressed" : "FAIL";
"#,
    );
    assert_eq!(out, "compressed");
}

/// Verifies compiled PHP output for compress zlib wrapper round trips through deflate.
#[test]
fn test_compress_zlib_wrapper_round_trips_through_deflate() {
    // compress.zlib:// opens a file and attaches the zlib.inflate read filter
    // so subsequent reads see decompressed bytes. Pairs with zlib.deflate
    // write to round-trip a payload through the filesystem.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$w = fopen("czlib_rt.bin", "w");
stream_filter_append($w, "zlib.deflate", STREAM_FILTER_WRITE);
fwrite($w, "elephc compress.zlib round-trip payload");
fclose($w);
$r = fopen("compress.zlib://czlib_rt.bin", "r");
echo stream_get_contents($r);
fclose($r);
"#,
    );
    assert_eq!(out, "elephc compress.zlib round-trip payload");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for compress bzip2 wrapper decompresses file.
#[test]
fn test_compress_bzip2_wrapper_decompresses_file() {
    // compress.bzip2:// slurps the underlying file and runs libbz2's
    // BZ2_bzBuffToBuffDecompress over it before exposing the bytes through
    // the file descriptor. The hex payload below is `bzip2 -c < "elephc
    // bzip2 round-trip"` captured at fixture-generation time.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$raw = hex2bin("425a6839314159265359814f1ef10000039980400210001e65d610200031434d300050f440c9ea7a8c1e5b5022c8cab9a05c297c5dc914e14242053c7bc4");
file_put_contents("cbz2_rt.bin", $raw);
$f = fopen("compress.bzip2://cbz2_rt.bin", "r");
echo stream_get_contents($f);
fclose($f);
"#,
    );
    assert_eq!(out, "elephc bzip2 round-trip");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream filter bzip2 compress then decompress roundtrip.
#[test]
fn test_stream_filter_bzip2_compress_then_decompress_roundtrip() {
    // bzip2.compress (write) streams the payload through libbz2's BZ2_bzCompress
    // and flushes the tail at fclose; bzip2.decompress (read) one-shot
    // decompresses it back. The compressed file must be smaller and the restored
    // bytes must match the original exactly.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("bzip2 stream filter round-trip. ", 12);
$w = fopen("bz2rt.bin", "w");
stream_filter_append($w, "bzip2.compress", STREAM_FILTER_WRITE);
fwrite($w, $payload);
fclose($w);
$comp = filesize("bz2rt.bin");
$r = fopen("bz2rt.bin", "r");
stream_filter_append($r, "bzip2.decompress", STREAM_FILTER_READ);
$restored = stream_get_contents($r);
fclose($r);
echo (($comp < strlen($payload)) ? "smaller" : "NOTSMALLER");
echo ($restored === $payload) ? "|match" : "|MISMATCH";
"#,
    );
    assert_eq!(out, "smaller|match");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream filter params compression level round trips.
#[test]
fn test_stream_filter_params_compression_level_round_trips() {
    // The 4th stream_filter_append $params arg sets the compression level
    // (zlib.deflate) / blockSize (bzip2.compress). A bare int literal is honored
    // at codegen; both filters must still produce a valid stream that the matching
    // decompressor restores exactly. zlib uses level 9, bzip2 blockSize 1.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("stream filter params round-trip. ", 16);

$zw = fopen("zp.bin", "w");
stream_filter_append($zw, "zlib.deflate", STREAM_FILTER_WRITE, 9);
fwrite($zw, $payload);
fclose($zw);
$zr = fopen("compress.zlib://zp.bin", "r");
$zrestored = stream_get_contents($zr);
fclose($zr);

$bw = fopen("bp.bin", "w");
stream_filter_append($bw, "bzip2.compress", STREAM_FILTER_WRITE, 1);
fwrite($bw, $payload);
fclose($bw);
$br = fopen("bp.bin", "r");
stream_filter_append($br, "bzip2.decompress", STREAM_FILTER_READ);
$brestored = stream_get_contents($br);
fclose($br);

echo ($zrestored === $payload) ? "zok" : "zBAD";
echo ($brestored === $payload) ? "|bok" : "|bBAD";
"#,
    );
    assert_eq!(out, "zok|bok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream filter params array form round trips.
#[test]
fn test_stream_filter_params_array_form_round_trips() {
    // PHP's canonical $params shape is an associative array, not a bare int:
    // zlib.deflate reads ['level' => N] and bzip2.compress reads
    // ['blocks' => N, 'work' => N]. Both array forms must be honored at codegen
    // and still produce a stream the matching decompressor restores exactly.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("array-form stream filter params round-trip. ", 16);

$zw = fopen("zp.bin", "w");
stream_filter_append($zw, "zlib.deflate", STREAM_FILTER_WRITE, ['level' => 9]);
fwrite($zw, $payload);
fclose($zw);
$zr = fopen("compress.zlib://zp.bin", "r");
$zrestored = stream_get_contents($zr);
fclose($zr);

$bw = fopen("bp.bin", "w");
stream_filter_append($bw, "bzip2.compress", STREAM_FILTER_WRITE, ['blocks' => 1, 'work' => 30]);
fwrite($bw, $payload);
fclose($bw);
$br = fopen("bp.bin", "r");
stream_filter_append($br, "bzip2.decompress", STREAM_FILTER_READ);
$brestored = stream_get_contents($br);
fclose($br);

echo ($zrestored === $payload) ? "zok" : "zBAD";
echo ($brestored === $payload) ? "|bok" : "|bBAD";
"#,
    );
    assert_eq!(out, "zok|bok");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream filter bzip2 decompress reads real bzip2.
#[test]
fn test_stream_filter_bzip2_decompress_reads_real_bzip2() {
    // bzip2.decompress (the FILTER path, distinct from the compress.bzip2://
    // wrapper) must decode a genuine bzip2 stream. The hex payload is
    // `bzip2 -c < "elephc bzip2 round-trip"` captured at fixture-generation time.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$raw = hex2bin("425a6839314159265359814f1ef10000039980400210001e65d610200031434d300050f440c9ea7a8c1e5b5022c8cab9a05c297c5dc914e14242053c7bc4");
file_put_contents("bz2fix.bin", $raw);
$f = fopen("bz2fix.bin", "r");
stream_filter_append($f, "bzip2.decompress", STREAM_FILTER_READ);
echo stream_get_contents($f);
fclose($f);
"#,
    );
    assert_eq!(out, "elephc bzip2 round-trip");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for compress bzip2 wrapper missing file returns false.
#[test]
fn test_compress_bzip2_wrapper_missing_file_returns_false() {
    // compress.bzip2:// surfaces a missing-file failure as PHP false,
    // mirroring the compress.zlib:// fallback path.
    let out = compile_and_run(
        r#"<?php
$r = @fopen("compress.bzip2:///nonexistent/elephc/file.bz2", "r");
echo ($r === false) ? "FALSE" : "OTHER";
"#,
    );
    assert_eq!(out, "FALSE");
}

/// Verifies compiled PHP output for compress zlib wrapper missing file returns false.
#[test]
fn test_compress_zlib_wrapper_missing_file_returns_false() {
    // compress.zlib:// must surface a missing-file failure as PHP `false`,
    // not as a half-attached inflate stream.
    let out = compile_and_run(
        r#"<?php
$r = @fopen("compress.zlib:///nonexistent/elephc/file.bin", "r");
echo ($r === false) ? "FALSE" : "OTHER";
"#,
    );
    assert_eq!(out, "FALSE");
}

/// Verifies compiled PHP output for stream filter zlib inflate decompresses.
#[test]
fn test_stream_filter_zlib_inflate_decompresses() {
    // The zlib.inflate read filter decompresses a zlib.deflate-compressed
    // stream; the two filters round-trip a payload through a file.
    let out = compile_and_run(
        r#"<?php
$data = str_repeat("zlib stream filter round-trip ", 24);
$w = fopen("zlib_rt.bin", "w");
stream_filter_append($w, "zlib.deflate", STREAM_FILTER_WRITE);
fwrite($w, $data);
fclose($w);
$r = fopen("zlib_rt.bin", "r");
stream_filter_append($r, "zlib.inflate", STREAM_FILTER_READ);
$got = stream_get_contents($r);
fclose($r);
echo ($got === $data) ? "roundtrip-ok" : "FAIL";
"#,
    );
    assert_eq!(out, "roundtrip-ok");
}

/// Verifies compiled PHP output for stream filter iconv utf8 to utf16le.
#[test]
fn test_stream_filter_iconv_utf8_to_utf16le() {
    // convert.iconv.UTF-8/UTF-16LE transcodes the stream at attach time via
    // libc iconv. "Hi" → 4 bytes UTF-16LE: 'H',0,'i',0. UTF-8↔UTF-16LE is in
    // the charset set even musl's limited iconv supports.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "Hi");
rewind($m);
stream_filter_append($m, "convert.iconv.UTF-8/UTF-16LE", STREAM_FILTER_READ);
$u = fread($m, 64);
echo strlen($u) . ":" . ord($u[0]) . "," . ord($u[1]) . "," . ord($u[2]) . "," . ord($u[3]);
fclose($m);
"#,
    );
    assert_eq!(out, "4:72,0,105,0");
}

/// Verifies compiled PHP output for stream filter iconv utf16le to utf8 roundtrips.
#[test]
fn test_stream_filter_iconv_utf16le_to_utf8_roundtrips() {
    // The reverse direction: UTF-16LE bytes decode back to the UTF-8 source.
    // The UTF-16LE input is built with chr() since elephc's lexer does not
    // process \xHH escapes.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, chr(72) . chr(0) . chr(105) . chr(0) . chr(33) . chr(0));
rewind($m);
stream_filter_append($m, "convert.iconv.UTF-16LE/UTF-8", STREAM_FILTER_READ);
echo fread($m, 64);
fclose($m);
"#,
    );
    assert_eq!(out, "Hi!");
}

/// Verifies compiled PHP output for stream filter iconv write transcodes on fwrite.
#[test]
fn test_stream_filter_iconv_write_transcodes_on_fwrite() {
    // STREAM_FILTER_WRITE installs a streaming per-fwrite transcoder: "Hi"
    // written as UTF-8 lands in the stream as UTF-16LE (48 00 69 00).
    // stream_get_contents reads the raw stored bytes (it bypasses read filters),
    // so it returns the transcoded UTF-16LE form.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://temp", "r+");
stream_filter_append($f, "convert.iconv.UTF-8/UTF-16LE", STREAM_FILTER_WRITE);
fwrite($f, "Hi");
rewind($f);
echo bin2hex(stream_get_contents($f));
fclose($f);
"#,
    );
    assert_eq!(out, "48006900");
}

/// Verifies compiled PHP output for stream filter iconv write then read roundtrips.
#[test]
fn test_stream_filter_iconv_write_then_read_roundtrips() {
    // Write through the UTF-8->UTF-16LE write filter, then read back through the
    // UTF-16LE->UTF-8 read filter: the original text is recovered.
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$w = fopen("ic.bin", "w");
stream_filter_append($w, "convert.iconv.UTF-8/UTF-16LE", STREAM_FILTER_WRITE);
fwrite($w, "Hello");
fclose($w);
$r = fopen("ic.bin", "r");
stream_filter_append($r, "convert.iconv.UTF-16LE/UTF-8", STREAM_FILTER_READ);
echo fread($r, 64);
fclose($r);
"#,
    );
    assert_eq!(out, "Hello");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream filter iconv read still default on all mode.
#[test]
fn test_stream_filter_iconv_read_still_default_on_all_mode() {
    // Regression for the new mode dispatch: a bare append (no 3rd arg = ALL)
    // must keep the attach-time READ transform, not switch to write.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "Hi");
rewind($m);
stream_filter_append($m, "convert.iconv.UTF-8/UTF-16LE");
echo strlen(fread($m, 64));
fclose($m);
"#,
    );
    assert_eq!(out, "4");
}

/// Verifies compiled PHP output for stream filter base64 encode pads correctly.
#[test]
fn test_stream_filter_base64_encode_pads_correctly() {
    // The convert.base64-encode write filter encodes 3-byte groups into 4
    // base64 chars and pads the tail with '=' bytes. Tests all three
    // remainder cases (0/1/2 bytes leftover).
    let out = compile_and_run(
        r#"<?php
$m1 = fopen("php://memory", "r+");
stream_filter_append($m1, "convert.base64-encode", STREAM_FILTER_WRITE);
fwrite($m1, "Hello World");
rewind($m1);
echo fread($m1, 64);
fclose($m1);
echo "|";
$m2 = fopen("php://memory", "r+");
stream_filter_append($m2, "convert.base64-encode", STREAM_FILTER_WRITE);
fwrite($m2, "ab");
rewind($m2);
echo fread($m2, 64);
fclose($m2);
echo "|";
$m3 = fopen("php://memory", "r+");
stream_filter_append($m3, "convert.base64-encode", STREAM_FILTER_WRITE);
fwrite($m3, "a");
rewind($m3);
echo fread($m3, 64);
fclose($m3);
"#,
    );
    assert_eq!(out, "SGVsbG8gV29ybGQ=|YWI=|YQ==");
}

/// Verifies compiled PHP output for stream filter qp encode escapes non printables.
#[test]
fn test_stream_filter_qp_encode_escapes_non_printables() {
    // The convert.quoted-printable-encode write filter escapes bytes outside
    // ASCII 33..126 (and '=') as '=XX' hex escapes. Pass-through ASCII is
    // copied verbatim.
    let out = compile_and_run(
        r#"<?php
$s = "abc" . chr(195) . chr(169) . chr(10) . "=";
$m = fopen("php://memory", "r+");
stream_filter_append($m, "convert.quoted-printable-encode", STREAM_FILTER_WRITE);
fwrite($m, $s);
rewind($m);
echo fread($m, 64);
fclose($m);
"#,
    );
    assert_eq!(out, "abc=C3=A9=0A=3D");
}

/// Verifies compiled PHP output for stream filter base64 decode decompacts.
#[test]
fn test_stream_filter_base64_decode_decompacts() {
    // The convert.base64-decode read filter decodes 4-byte base64 quads
    // into 3 raw bytes. The runtime overwrites the buffer in place and
    // returns the shrunk byte count.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "SGVsbG8gV29ybGQ=");
rewind($m);
stream_filter_append($m, "convert.base64-decode", STREAM_FILTER_READ);
$s = fread($m, 64);
fclose($m);
echo "'" . $s . "' len=" . strlen($s);
"#,
    );
    assert_eq!(out, "'Hello World' len=11");
}

/// Verifies compiled PHP output for stream filter qp decode handles escapes and soft breaks.
#[test]
fn test_stream_filter_qp_decode_handles_escapes_and_soft_breaks() {
    // The convert.quoted-printable-decode read filter expands "=XX" hex
    // escapes into raw bytes and drops "=\r\n" / "=\n" soft line breaks.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "Caf=C3=A9 br=\n=C3=BBl=C3=A9");
rewind($m);
stream_filter_append($m, "convert.quoted-printable-decode", STREAM_FILTER_READ);
$s = fread($m, 64);
fclose($m);
echo "'" . $s . "' len=" . strlen($s);
"#,
    );
    assert_eq!(out, "'Café brûlé' len=13");
}

/// Verifies compiled PHP output for stream filter strip tags removes html.
#[test]
fn test_stream_filter_strip_tags_removes_html() {
    // The string.strip_tags read filter elides everything between '<' and '>'.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "<p>Hello <b>World</b></p>");
rewind($m);
stream_filter_append($m, "string.strip_tags", STREAM_FILTER_READ);
echo fread($m, 64);
fclose($m);
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Verifies compiled PHP output for stream filter dechunk parses chunked encoding.
#[test]
fn test_stream_filter_dechunk_parses_chunked_encoding() {
    // The dechunk read filter parses HTTP/1.1 chunked-transfer encoding:
    // hex size line, CRLF, payload, CRLF, then a zero chunk terminator.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "5\r\nHello\r\n6\r\n World\r\n0\r\n\r\n");
rewind($m);
stream_filter_append($m, "dechunk", STREAM_FILTER_READ);
echo fread($m, 64);
fclose($m);
"#,
    );
    assert_eq!(out, "Hello World");
}

/// Verifies compiled PHP output for stream get contents reads whole stream.
#[test]
fn test_stream_get_contents_reads_whole_stream() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgc.txt", "elephc stream contents");
$f = fopen("sgc.txt", "r");
echo stream_get_contents($f);
fclose($f);
unlink("sgc.txt");
"#,
    );
    assert_eq!(out, "elephc stream contents");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream get contents reads from current position.
#[test]
fn test_stream_get_contents_reads_from_current_position() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgc_pos.txt", "HEADERbody");
$f = fopen("sgc_pos.txt", "r");
fread($f, 6);
echo stream_get_contents($f);
fclose($f);
unlink("sgc_pos.txt");
"#,
    );
    assert_eq!(out, "body");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream get contents empty at eof.
#[test]
fn test_stream_get_contents_empty_at_eof() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgc_eof.txt", "x");
$f = fopen("sgc_eof.txt", "r");
fread($f, 10);
$rest = stream_get_contents($f);
echo "[" . $rest . "]" . strlen($rest);
fclose($f);
unlink("sgc_eof.txt");
"#,
    );
    assert_eq!(out, "[]0");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the optional `$length` and `$offset` arguments of
/// `stream_get_contents()`: a finite `$length` caps the read (`Hello`); an
/// `$offset >= 0` seeks before reading (`World` for length 5 from offset 7,
/// `World!` for read-all from offset 7); a negative/omitted `$length` reads to
/// EOF; and a capped read honors the current position after a prior `fread`
/// (`llo`). Output matches PHP 8.5 byte-for-byte (verified via `php -r`).
#[test]
fn test_stream_get_contents_length_and_offset() {
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "Hello, World!");
rewind($m);
echo "[" . stream_get_contents($m, 5) . "]";
rewind($m);
echo "[" . stream_get_contents($m, 5, 7) . "]";
rewind($m);
echo "[" . stream_get_contents($m, -1, 7) . "]";
rewind($m);
echo "[" . stream_get_contents($m) . "]";
rewind($m);
fread($m, 2);
echo "[" . stream_get_contents($m, 3) . "]";
fclose($m);
"#,
    );
    assert_eq!(out, "[Hello][World][World!][Hello, World!][llo]");
}

/// Verifies `stream_get_contents()` returns `false` when a positive offset
/// fails through a user wrapper's `stream_seek`, matching PHP's failure result.
#[test]
fn test_stream_get_contents_offset_seek_failure_is_false() {
    let out = compile_and_run(
        r#"<?php
class NoSeekGetW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_seek(int $offset, int $whence): bool { return false; }
    public function stream_read(int $n): string { return "abc"; }
    public function stream_eof(): bool { return true; }
}
stream_wrapper_register("noseekget", "NoSeekGetW");
$f = fopen("noseekget://x", "r");
$r = stream_get_contents($f, null, 1);
echo $r === false ? "false" : "got";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies finite `stream_get_contents()` on a user wrapper keeps reading
/// smaller chunks until the requested length is filled without draining the
/// rest of the wrapper stream.
#[test]
fn test_stream_get_contents_bounded_wrapper_read_fills_length() {
    let out = compile_and_run(
        r#"<?php
class SlowW {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="abcdefghi"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,min(2,$n)); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("slow","SlowW");
$f=fopen("slow://x","r");
echo stream_get_contents($f,5);
echo "|" . stream_get_contents($f);
fclose($f);
"#,
    );
    assert_eq!(out, "abcde|fghi");
}

/// Verifies a runtime-computed negative length follows PHP's read-all contract
/// instead of being treated as a finite negative cap.
#[test]
fn test_stream_get_contents_runtime_negative_length_reads_all() {
    let out = compile_and_run(
        r#"<?php
function neg_one(): int { return -1; }
$m = fopen("php://memory", "r+");
fwrite($m, "runtime-all");
rewind($m);
echo stream_get_contents($m, neg_one());
fclose($m);
"#,
    );
    assert_eq!(out, "runtime-all");
}

/// Verifies compiled PHP output for stream copy to stream copies all bytes.
#[test]
fn test_stream_copy_to_stream_copies_all_bytes() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("scts_src.txt", "copy me through a stream");
$from = fopen("scts_src.txt", "r");
$to = fopen("scts_dst.txt", "w");
$n = stream_copy_to_stream($from, $to);
fclose($from);
fclose($to);
echo $n . ":" . file_get_contents("scts_dst.txt");
unlink("scts_src.txt");
unlink("scts_dst.txt");
"#,
    );
    assert_eq!(out, "24:copy me through a stream");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream copy to stream resumes from position.
#[test]
fn test_stream_copy_to_stream_resumes_from_position() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("scts_p_src.txt", "SKIPkeep");
$from = fopen("scts_p_src.txt", "r");
fread($from, 4);
$to = fopen("scts_p_dst.txt", "w");
$n = stream_copy_to_stream($from, $to);
fclose($from);
fclose($to);
echo $n . ":" . file_get_contents("scts_p_dst.txt");
unlink("scts_p_src.txt");
unlink("scts_p_dst.txt");
"#,
    );
    assert_eq!(out, "4:keep");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream copy to stream empty source.
#[test]
fn test_stream_copy_to_stream_empty_source() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("scts_e_src.txt", "");
$from = fopen("scts_e_src.txt", "r");
$to = fopen("scts_e_dst.txt", "w");
echo stream_copy_to_stream($from, $to);
fclose($from);
fclose($to);
unlink("scts_e_src.txt");
unlink("scts_e_dst.txt");
"#,
    );
    assert_eq!(out, "0");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the optional `$length` and `$offset` arguments of
/// `stream_copy_to_stream()`: a finite `$length` caps the copy (`Hello`, 5
/// bytes); an `$offset >= 0` seeks the source first (`World` for length 5 from
/// offset 7); and a negative `$length` from an offset copies to EOF (`World!`,
/// 6 bytes). Byte counts and contents match PHP 8.5 (verified via `php -r`).
#[test]
fn test_stream_copy_to_stream_length_and_offset() {
    let out = compile_and_run(
        r#"<?php
$s = fopen("php://memory", "r+"); fwrite($s, "Hello, World!"); rewind($s);
$d = fopen("php://memory", "r+");
$n = stream_copy_to_stream($s, $d, 5);
rewind($d);
echo "[" . $n . ":" . stream_get_contents($d) . "]";

$s2 = fopen("php://memory", "r+"); fwrite($s2, "Hello, World!"); rewind($s2);
$d2 = fopen("php://memory", "r+");
$n2 = stream_copy_to_stream($s2, $d2, 5, 7);
rewind($d2);
echo "[" . $n2 . ":" . stream_get_contents($d2) . "]";

$s3 = fopen("php://memory", "r+"); fwrite($s3, "Hello, World!"); rewind($s3);
$d3 = fopen("php://memory", "r+");
$n3 = stream_copy_to_stream($s3, $d3, -1, 7);
rewind($d3);
echo "[" . $n3 . ":" . stream_get_contents($d3) . "]";
"#,
    );
    assert_eq!(out, "[5:Hello][5:World][6:World!]");
}

/// Verifies `stream_copy_to_stream()` returns `false` when a positive offset
/// fails through a user wrapper's `stream_seek`, matching PHP's failure result.
#[test]
fn test_stream_copy_to_stream_offset_seek_failure_is_false() {
    let out = compile_and_run(
        r#"<?php
class NoSeekCopyW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_seek(int $offset, int $whence): bool { return false; }
    public function stream_read(int $n): string { return "abc"; }
    public function stream_eof(): bool { return true; }
}
stream_wrapper_register("noseekcopy", "NoSeekCopyW");
$src = fopen("noseekcopy://x", "r");
$dst = fopen("php://memory", "r+");
$n = stream_copy_to_stream($src, $dst, null, 1);
echo $n === false ? "false" : "got";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies a runtime-computed negative length copies to EOF, matching PHP's
/// default `-1` length semantics.
#[test]
fn test_stream_copy_to_stream_runtime_negative_length_copies_all() {
    let out = compile_and_run(
        r#"<?php
function neg_one(): int { return -1; }
$s = fopen("php://memory", "r+");
$d = fopen("php://memory", "r+");
fwrite($s, "copy-runtime-all");
rewind($s);
$n = stream_copy_to_stream($s, $d, neg_one());
rewind($d);
echo $n . ":" . stream_get_contents($d);
fclose($s);
fclose($d);
"#,
    );
    assert_eq!(out, "16:copy-runtime-all");
}

/// Verifies finite `stream_copy_to_stream()` copies from a wrapper source that
/// returns smaller chunks than requested.
#[test]
fn test_stream_copy_to_stream_bounded_wrapper_read_fills_length() {
    let out = compile_and_run(
        r#"<?php
class SlowCopyW {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="abcdefghi"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,2); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("slowcopy","SlowCopyW");
$src=fopen("slowcopy://x","r");
$dst=fopen("php://memory","r+");
$n=stream_copy_to_stream($src,$dst,5);
rewind($dst);
echo $n . ":" . stream_get_contents($dst);
fclose($src);
fclose($dst);
"#,
    );
    assert_eq!(out, "5:abcde");
}

/// Verifies compiled PHP output for fopen php stdout writes to stdout.
#[test]
fn test_fopen_php_stdout_writes_to_stdout() {
    let out =
        compile_and_run(r#"<?php $h = fopen("php://stdout", "w"); fwrite($h, "via php-wrapper");"#);
    assert_eq!(out, "via php-wrapper");
}

/// Verifies closing a `php://stdout` handle leaves the program's own stdout usable.
///
/// The wrapper used to hand back descriptor 1 itself, so `fclose()` closed the process's
/// standard output: `after` was written to a closed descriptor and vanished, while the
/// program still exited 0 — output loss with no diagnostic anywhere. php-src duplicates
/// the descriptor in `php_fopen_wrapper.c`, and reference PHP 8.5.6 prints both lines.
///
/// The `before` line is asserted too: a wrapper that failed to open at all would drop
/// only the `via-handle` write and still print `after`, passing a test that pinned the
/// tail alone.
#[test]
fn test_closing_php_stdout_leaves_the_process_stdout_open() {
    let out = compile_and_run(
        r#"<?php
$h = fopen("php://stdout", "w");
echo "before\n";
fwrite($h, "via-handle\n");
fclose($h);
echo "after\n";
"#,
    );
    assert_eq!(out, "before\nvia-handle\nafter\n");
}

/// Verifies compiled PHP output for fopen php output is stdout alias.
#[test]
fn test_fopen_php_output_is_stdout_alias() {
    let out = compile_and_run(r#"<?php $h = fopen("php://output", "w"); fwrite($h, "aliased");"#);
    assert_eq!(out, "aliased");
}

/// Verifies compiled PHP output for fopen php stream yields resource.
#[test]
fn test_fopen_php_stream_yields_resource() {
    let out = compile_and_run(
        r#"<?php $h = fopen("php://stderr", "w"); echo is_resource($h) ? "y" : "n"; echo get_resource_type($h);"#,
    );
    assert_eq!(out, "ystream");
}

/// Verifies compiled PHP output for fopen php memory round trip.
#[test]
fn test_fopen_php_memory_round_trip() {
    // php://memory is a writable, seekable in-memory stream.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
fwrite($m, "memory contents");
rewind($m);
echo fread($m, 64);
fclose($m);
"#,
    );
    assert_eq!(out, "memory contents");
}

/// Verifies compiled PHP output for fopen php temp seek and tell.
#[test]
fn test_fopen_php_temp_seek_and_tell() {
    // php://temp behaves like php://memory; fseek/ftell work on it.
    let out = compile_and_run(
        r#"<?php
$t = fopen("php://temp", "w+");
fwrite($t, "0123456789");
fseek($t, 4);
echo fread($t, 3);
echo "|";
echo ftell($t);
fclose($t);
"#,
    );
    assert_eq!(out, "456|7");
}

/// Verifies compiled PHP output for fopen data uri base64.
#[test]
fn test_fopen_data_uri_base64() {
    // data:// with ;base64 decodes the payload at compile time.
    let out = compile_and_run(
        r#"<?php
$d = fopen("data://text/plain;base64,SGVsbG8gd29ybGQ=", "r");
echo fread($d, 64);
fclose($d);
"#,
    );
    assert_eq!(out, "Hello world");
}

/// Verifies compiled PHP output for fopen data uri percent encoded.
#[test]
fn test_fopen_data_uri_percent_encoded() {
    // A non-base64 data:// payload is percent-decoded (%HH and + → space).
    let out = compile_and_run(
        r#"<?php
$d = fopen("data://text/plain,Hello%20raw%2Bworld", "r");
echo fread($d, 64);
fclose($d);
"#,
    );
    assert_eq!(out, "Hello raw+world");
}

/// Verifies compiled PHP output for fopen data uri invalid returns false.
#[test]
fn test_fopen_data_uri_invalid_returns_false() {
    // A data:// URI without the mandatory comma fails like any bad fopen().
    let out = compile_and_run(
        r#"<?php $d = fopen("data://no-comma-here", "r"); echo is_bool($d) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// One PHAR entry for the test builder: archive name, recorded uncompressed
/// size, the bytes as stored in the data section, and the entry flag word.
struct TestPharEntry<'a> {
    name: &'a str,
    uncompressed_size: u32,
    stored: &'a [u8],
    flags: u32,
}

// Precomputed bzip2 blob for `"bzip2-compressed phar entry. "` repeated eight
// times. bzip2-rs is decode-only, so tests keep this stable fixture inline.
const BZIP2_PHAR_BLOB: &[u8] = &[
    0x42, 0x5a, 0x68, 0x39, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x61, 0x39,
    0xa6, 0xe8, 0x00, 0x00, 0x1f, 0x99, 0x80, 0x40, 0x03, 0x10, 0x00, 0x3e,
    0x63, 0xdc, 0x30, 0x20, 0x00, 0x70, 0x53, 0x09, 0xa6, 0x80, 0xd3, 0x10,
    0x2a, 0xa8, 0x0c, 0x43, 0x46, 0x1a, 0x9b, 0x0b, 0x0a, 0x0e, 0x46, 0x45,
    0xc5, 0x44, 0xc5, 0x05, 0x46, 0x06, 0xe3, 0xa1, 0x21, 0x03, 0x22, 0x42,
    0xc2, 0xe2, 0x63, 0x02, 0xe2, 0x82, 0x07, 0x82, 0x82, 0x05, 0x44, 0x0f,
    0xc5, 0xdc, 0x91, 0x4e, 0x14, 0x24, 0x18, 0x4e, 0x69, 0xba, 0x00,
];

/// Builds a native-format PHAR (PHP stub + manifest + data section) from
/// explicit per-entry stored bytes and flags, matching the byte layout PHP's
/// `Phar` class produces. crc32 and signature are omitted because the reader
/// ignores them. Lets the `phar://` codegen tests exercise uncompressed and
/// gzip (raw-DEFLATE) entries as deterministic, php-free fixtures.
fn build_phar(entries: &[TestPharEntry]) -> Vec<u8> {
    let mut manifest = Vec::new();
    manifest.extend_from_slice(&(entries.len() as u32).to_le_bytes()); // num_files
    manifest.extend_from_slice(&[0x11, 0x00]); // api version (1.1.0)
    manifest.extend_from_slice(&0u32.to_le_bytes()); // global bitmapped flags
    manifest.extend_from_slice(&0u32.to_le_bytes()); // alias length (none)
    manifest.extend_from_slice(&0u32.to_le_bytes()); // manifest metadata length (none)
    for e in entries {
        manifest.extend_from_slice(&(e.name.len() as u32).to_le_bytes());
        manifest.extend_from_slice(e.name.as_bytes());
        manifest.extend_from_slice(&e.uncompressed_size.to_le_bytes());
        manifest.extend_from_slice(&0u32.to_le_bytes()); // timestamp
        manifest.extend_from_slice(&(e.stored.len() as u32).to_le_bytes()); // compressed size
        manifest.extend_from_slice(&0u32.to_le_bytes()); // crc32 (ignored by the reader)
        manifest.extend_from_slice(&e.flags.to_le_bytes());
        manifest.extend_from_slice(&0u32.to_le_bytes()); // entry metadata length (none)
    }
    let mut out = Vec::new();
    out.extend_from_slice(b"<?php __HALT_COMPILER(); ?>\r\n");
    out.extend_from_slice(&(manifest.len() as u32).to_le_bytes()); // manifest length
    out.extend_from_slice(&manifest);
    for e in entries {
        out.extend_from_slice(e.stored); // data section: entries in manifest order
    }
    out
}

/// Convenience over [`build_phar`] for plain uncompressed entries (mode 0644).
fn build_minimal_phar(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let raw: Vec<TestPharEntry> = entries
        .iter()
        .map(|(name, content)| TestPharEntry {
            name,
            uncompressed_size: content.len() as u32,
            stored: content,
            flags: 0x0000_01a4,
        })
        .collect();
    build_phar(&raw)
}

/// Builds a minimal POSIX tar archive with regular-file entries.
fn build_tar_phar_container(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, content) in entries {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        let size = format!("{:011o}\0", content.len());
        header[124..124 + size.len()].copy_from_slice(size.as_bytes());
        header[156] = b'0';
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        for byte in &mut header[148..156] {
            *byte = b' ';
        }
        let checksum: u32 = header.iter().map(|&b| b as u32).sum();
        let checksum = format!("{:06o}\0 ", checksum);
        header[148..156].copy_from_slice(checksum.as_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(content);
        let padded_len = ((content.len() + 511) / 512) * 512;
        out.resize(out.len() + padded_len - content.len(), 0);
    }
    out.extend_from_slice(&[0u8; 1024]);
    out
}

/// Builds a ZIP archive with ordinary store/deflate entries and a central directory.
fn build_zip_phar_container(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, content, deflate) in entries {
        let local_offset = out.len() as u32;
        let stored = if *deflate {
            let mut encoder =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            std::io::Write::write_all(&mut encoder, content).unwrap();
            encoder.finish().unwrap()
        } else {
            content.to_vec()
        };
        let method = if *deflate { 8u16 } else { 0u16 };
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&method.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(stored.len() as u32).to_le_bytes());
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&stored);

        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&method.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&(stored.len() as u32).to_le_bytes());
        central.extend_from_slice(&(content.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&local_offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let central_offset = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Verifies compiled PHP output for fopen phar reads uncompressed entry.
#[test]
fn test_fopen_phar_reads_uncompressed_entry() {
    // fopen("phar://archive/entry") reads the named uncompressed entry out of the
    // archive at compile time and serves it as a readable stream. Covers a
    // top-level entry, a nested entry (exercising the cumulative data-offset
    // walk), and a missing entry lowering to false. The archive path must be a
    // literal, so the fixture is written to an absolute temp path embedded below.
    let phar = build_minimal_phar(&[
        ("hello.txt", b"Hello from phar!\n"),
        ("dir/inner.txt", b"inner content here"),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_m1_read_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php
$f = fopen("phar://{p}/hello.txt", "r");
echo fread($f, 100);
fclose($f);
$g = fopen("phar://{p}/dir/inner.txt", "r");
echo "[" . fread($g, 100) . "]";
fclose($g);
$m = @fopen("phar://{p}/nope.txt", "r");
echo "|" . ($m === false ? "false" : "open");
"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "Hello from phar!\n[inner content here]|false");
}

/// Verifies a literal `phar://` `file_get_contents()` honors PHP's `$offset`/`$length` window.
///
/// The entry bytes are extracted at COMPILE time and served from read-only `.data`, so the
/// windowing path — which trims its input in place and frees a failed read — must copy them into
/// an owned string first. Without that copy the trim would move and free a rodata pointer.
#[test]
fn test_file_get_contents_literal_phar_entry_honors_offset_and_length() {
    let phar = build_minimal_phar(&[("hello.txt", b"Hello from phar!\n")]);
    let path =
        std::env::temp_dir().join(format!("elephc_phar_fgc_range_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php
var_dump(file_get_contents("phar://{p}/hello.txt"));
var_dump(file_get_contents("phar://{p}/hello.txt", false, null, 6, 4));
var_dump(file_get_contents("phar://{p}/hello.txt", false, null, -6, 5));
var_dump(@file_get_contents("phar://{p}/hello.txt", false, null, -99));
"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(
        out,
        "string(17) \"Hello from phar!\n\"\nstring(4) \"from\"\nstring(5) \"phar!\"\nbool(false)\n"
    );
}

/// Runtime phar:// read: when the archive path arrives via a variable (not a
/// compile-time literal), `fopen` routes through `__rt_fopen_maybe_phar` →
/// `__rt_phar_read_entry`, which reads and parses the archive at run time and
/// materializes the entry as a readable stream. Reads the nested (2nd) entry to
/// validate the cumulative data-offset walk, and a missing entry → false.
#[test]
fn test_fopen_phar_runtime_path_reads_entry() {
    let phar = build_minimal_phar(&[
        ("hello.txt", b"Hello from phar!\n"),
        ("dir/inner.txt", b"inner content here"),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_m2_rt_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php
$p = "{p}";
$f = fopen("phar://" . $p . "/dir/inner.txt", "r");
echo fread($f, 100);
fclose($f);
$m = @fopen("phar://" . $p . "/nope.txt", "r");
echo "|" . ($m === false ? "false" : "open");
"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "inner content here|false");
}

/// phar:// write Milestone 1: `fopen("phar://...","w")` + `fwrite` + `fclose`
/// assembles a valid single-entry uncompressed phar that sets the
/// PHAR_HDR_SIGNATURE (0x10000) global flag and appends a SHA1 signature
/// trailer (`raw-sha1 ++ LE32(0x0002) ++ "GBMB"`), so real PHP — which requires
/// a hash by default — accepts the archive. elephc's own phar reader is
/// compile-time (it reads the archive during compilation), so a runtime-written
/// archive can't be read back in the same program; this test verifies the
/// on-disk bytes directly. (Manually confirmed that real PHP's `new Phar(...)`
/// reads the entry back.)
#[test]
fn test_fopen_phar_write_signs_single_entry() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("phar://out.phar/hello.txt", "w");
$n = fwrite($f, "payload-data");
echo (fclose($f) ? "ok" : "fail") . $n;
"#,
    );
    assert_eq!(out, "ok12");
    let bytes = fs::read(dir.join("out.phar")).expect("phar archive written");
    let _ = fs::remove_dir_all(&dir);
    // Global manifest flags carry PHAR_HDR_SIGNATURE (0x00010000) at offset 39
    // (29-byte stub + manifest_len(4) + num_files(4) + api_version(2)).
    assert_eq!(
        &bytes[39..43],
        &[0x00, 0x00, 0x01, 0x00],
        "PHAR_HDR_SIGNATURE flag not set"
    );
    // Signature trailer: <20 raw SHA1 bytes> ++ LE32(0x0002 = Phar::SHA1) ++ "GBMB".
    let tail = &bytes[bytes.len() - 8..];
    assert_eq!(&tail[0..4], &[0x02, 0x00, 0x00, 0x00], "signature type not SHA1");
    assert_eq!(&tail[4..8], b"GBMB", "phar magic missing");
}

/// `file_put_contents("phar://archive/entry", $data)` writes a signed
/// single-entry phar in one call (reusing the fopen-write runtime), returning
/// the byte count. Verifies the returned count and the on-disk signature bytes.
/// (Manually confirmed real PHP reads the entry back.)
#[test]
fn test_file_put_contents_phar_writes_signed_entry() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
echo file_put_contents("phar://out.phar/note.txt", "via fpc");
"#,
    );
    assert_eq!(out, "7"); // strlen("via fpc")
    let bytes = fs::read(dir.join("out.phar")).expect("phar archive written");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        &bytes[39..43],
        &[0x00, 0x00, 0x01, 0x00],
        "PHAR_HDR_SIGNATURE flag not set"
    );
    let tail = &bytes[bytes.len() - 8..];
    assert_eq!(&tail[0..4], &[0x02, 0x00, 0x00, 0x00], "signature type not SHA1");
    assert_eq!(&tail[4..8], b"GBMB", "phar magic missing");
}

/// EIR phar:// write streams seed the runtime PHAR writer instead of falling
/// through to a literal filesystem path named `phar://...`.
#[test]
fn test_fopen_phar_write_runtime_readback() {
    let out = compile_and_run(
        r#"<?php
$f = fopen("phar://streamed.phar/hello.txt", "w");
echo fwrite($f, "streamed") . "|";
echo (fclose($f) ? "closed" : "failed") . "|";
$archive = "streamed.phar";
echo file_get_contents("phar://" . $archive . "/hello.txt");
"#,
    );
    assert_eq!(out, "8|closed|streamed");
}

/// EIR one-shot phar:// writes use the same signed archive runtime as
/// `fopen()` + `fwrite()` + `fclose()` and are readable through a runtime URL.
#[test]
fn test_file_put_contents_phar_runtime_readback() {
    let out = compile_and_run(
        r#"<?php
echo file_put_contents("phar://single.phar/note.txt", "via fpc") . "|";
$archive = "single.phar";
echo file_get_contents("phar://" . $archive . "/note.txt");
"#,
    );
    assert_eq!(out, "7|via fpc");
}

/// Repeated phar:// file_put_contents() calls update a native PHAR in place,
/// preserving previously written entries instead of rewriting a single-entry archive.
#[test]
fn test_file_put_contents_phar_preserves_existing_entries() {
    let out = compile_and_run(
        r#"<?php
echo file_put_contents("phar://multi.phar/one.txt", "alpha") . "|";
echo file_put_contents("phar://multi.phar/dir/two.txt", "bravo") . "|";
echo file_put_contents("phar://multi.phar/one.txt", "updated") . "|";
$archive = "multi.phar";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/dir/two.txt");
"#,
    );
    assert_eq!(out, "5|5|7|updated|bravo");
}

/// fopen()+fwrite()+fclose() phar:// writes also use the native PHAR
/// read-modify-write bridge, so stream writes preserve earlier entries.
#[test]
fn test_fopen_phar_write_preserves_existing_entries() {
    let out = compile_and_run(
        r#"<?php
echo file_put_contents("phar://stream_multi.phar/one.txt", "alpha") . "|";
$f = fopen("phar://stream_multi.phar/two.txt", "w");
echo fwrite($f, "stream") . "|";
echo (fclose($f) ? "closed" : "failed") . "|";
$archive = "stream_multi.phar";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/two.txt");
"#,
    );
    assert_eq!(out, "5|6|closed|alpha|stream");
}

/// Runtime-built phar:// URLs passed to file_put_contents() route through the
/// native PHAR URL bridge instead of writing a literal filesystem path.
#[test]
fn test_file_put_contents_dynamic_phar_url_preserves_existing_entries() {
    let out = compile_and_run(
        r#"<?php
$archive = "dynamic_multi.phar";
echo file_put_contents("phar://" . $archive . "/one.txt", "alpha") . "|";
echo file_put_contents("phar://" . $archive . "/dir/two.txt", "bravo") . "|";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/dir/two.txt");
"#,
    );
    assert_eq!(out, "5|5|alpha|bravo");
}

/// Runtime-built phar:// URLs passed to write-mode fopen() preserve the full URL
/// until fclose(), then update the native PHAR through the URL bridge.
#[test]
fn test_fopen_dynamic_phar_write_preserves_existing_entries() {
    let out = compile_and_run(
        r#"<?php
$archive = "dynamic_stream.phar";
echo file_put_contents("phar://" . $archive . "/one.txt", "alpha") . "|";
$f = fopen("phar://" . $archive . "/dir/two.txt", "w");
echo fwrite($f, "stream") . "|";
echo (fclose($f) ? "closed" : "failed") . "|";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/dir/two.txt");
"#,
    );
    assert_eq!(out, "5|6|closed|alpha|stream");
}

/// Concurrent phar:// write streams keep independent payload buffers and
/// finalize through their own descriptors, including mixed literal/dynamic URLs.
#[test]
fn test_fopen_concurrent_phar_write_streams_preserve_entries() {
    let out = compile_and_run(
        r#"<?php
$archive = "concurrent_streams.phar";
$one = fopen("phar://concurrent_streams.phar/one.txt", "w");
$two = fopen("phar://" . $archive . "/two.txt", "w");
echo fwrite($two, "bravo") . "|";
echo fwrite($one, "alpha") . "|";
echo (fclose($one) ? "one" : "fail-one") . "|";
echo (fclose($two) ? "two" : "fail-two") . "|";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/two.txt");
"#,
    );
    assert_eq!(out, "5|5|one|two|alpha|bravo");
}

/// `phar://` writes to a `.tar` archive create/update a tar container through
/// the Rust bridge, and the runtime reader can read both entries back.
#[test]
fn test_file_put_contents_phar_tar_archive_runtime_readback() {
    let out = compile_and_run(
        r#"<?php
echo file_put_contents("phar://out.tar/one.txt", "alpha") . "|";
echo file_put_contents("phar://out.tar/dir/two.txt", "bravo") . "|";
$archive = "out.tar";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/dir/two.txt");
"#,
    );
    assert_eq!(out, "5|5|alpha|bravo");
}

/// `phar://` writes to a `.zip` archive create/update a ZIP container through
/// the Rust bridge, and the runtime reader can read both entries back.
#[test]
fn test_file_put_contents_phar_zip_archive_runtime_readback() {
    let out = compile_and_run(
        r#"<?php
echo file_put_contents("phar://out.zip/one.txt", "alpha") . "|";
echo file_put_contents("phar://out.zip/dir/two.txt", "bravo") . "|";
$archive = "out.zip";
echo file_get_contents("phar://" . $archive . "/one.txt") . "|";
echo file_get_contents("phar://" . $archive . "/dir/two.txt");
"#,
    );
    assert_eq!(out, "5|5|alpha|bravo");
}

/// `unlink("phar://...")` removes one archive entry while preserving sibling
/// entries across native PHAR, tar, and ZIP containers.
#[test]
fn test_unlink_phar_entries_preserves_siblings() {
    let out = compile_and_run(
        r#"<?php
file_put_contents("phar://delete.phar/one.txt", "alpha");
file_put_contents("phar://delete.phar/two.txt", "bravo");
echo (unlink("phar://delete.phar/one.txt") ? "u|" : "bad|");
$phar = "delete.phar";
echo (file_get_contents("phar://" . $phar . "/one.txt") === false ? "missing|" : "bad|");
echo file_get_contents("phar://" . $phar . "/two.txt") . "|";
file_put_contents("phar://delete.tar/one.txt", "tar-one");
file_put_contents("phar://delete.tar/two.txt", "tar-two");
echo (unlink("phar://delete.tar/one.txt") ? "u|" : "bad|");
$tar = "delete.tar";
echo file_get_contents("phar://" . $tar . "/two.txt") . "|";
file_put_contents("phar://delete.zip/one.txt", "zip-one");
file_put_contents("phar://delete.zip/two.txt", "zip-two");
echo (unlink("phar://delete.zip/one.txt") ? "u|" : "bad|");
$zip = "delete.zip";
echo file_get_contents("phar://" . $zip . "/two.txt") . "|";
echo (unlink("phar://delete.zip/missing.txt") ? "bad" : "missing");
"#,
    );
    assert_eq!(
        out,
        "u|missing|bravo|u|tar-two|u|zip-two|missing"
    );
}

/// `Phar` and `PharData` expose a minimal OOP ArrayAccess surface that maps
/// bracket reads/writes/isset to the existing runtime `phar://` reader/writer.
#[test]
fn test_phar_oop_array_access_read_write() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("oop.phar");
$p["one.txt"] = "alpha";
$p["dir/two.txt"] = "bravo";
echo class_exists("phar") ? "class|" : "missing|";
echo class_exists("pharfileinfo") ? "info-class|" : "missing-info|";
echo ($p instanceof ArrayAccess) ? "aa|" : "no-aa|";
$info = $p["one.txt"];
echo ($info instanceof SplFileInfo) ? "spl-info|" : "bad-info|";
echo get_class($info) . "|";
echo $info->getContent() . "|";
echo $info->getFilename() . "|";
echo $info->getPathname() . "|";
echo $p["dir/two.txt"]->getContent() . "|";
echo ($p["missing.txt"]->getContent() === false ? "missing|" : "bad|");
echo (isset($p["one.txt"]) ? "yes|" : "no|");
echo (isset($p["missing.txt"]) ? "bad|" : "no|");
$pd = new PharData("oop.tar");
$pd["note.txt"] = "tar";
echo $pd["note.txt"]->getContent() . "|";
echo Phar::GZ . "|" . PharData::TAR;
"#,
    );
    assert_eq!(
        out,
        "class|info-class|aa|spl-info|PharFileInfo|alpha|one.txt|phar://oop.phar/one.txt|bravo|missing|yes|no|tar|4096|2"
    );
}

/// `Phar::addFromString()` and `PharData::addFromString()` use the same runtime
/// writer as ArrayAccess assignment for native PHAR and tar containers.
#[test]
fn test_phar_oop_add_from_string_writes_entries() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("add.phar");
$p->addFromString("one.txt", "alpha");
$p->addFromString("dir/two.txt", "bravo");
echo $p["one.txt"]->getContent() . "|";
echo $p["dir/two.txt"]->getContent() . "|";
$pd = new PharData("add.tar");
$pd->addFromString("note.txt", "tar");
echo $pd["note.txt"]->getContent();
"#,
    );
    assert_eq!(out, "alpha|bravo|tar");
}

/// `Phar` and `PharData` expose object-level metadata, stub, and path helpers.
#[test]
fn test_phar_oop_metadata_stub_and_path_helpers() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("meta.phar");
echo ($p->hasMetadata() ? "bad|" : "no-meta|");
echo ($p->getMetadata() === null ? "null|" : "bad|");
$p->setMetadata("app:3");
echo ($p->hasMetadata() ? "has-meta|" : "bad|");
echo $p->getMetadata() . "|";
$p->setMetadata(["kind" => "app", "version" => 3]);
$meta = $p->getMetadata();
echo $meta["kind"] . ":" . $meta["version"] . "|";
$p->setMetadata(42);
echo $p->getMetadata() . "|";
$p->setMetadata(null);
echo ($p->hasMetadata() ? "has-null|" : "bad|");
echo ($p->getMetadata() === null ? "null-meta|" : "bad|");
echo ($p->delMetadata() ? "cleared|" : "bad|");
echo ($p->hasMetadata() ? "bad|" : "no-meta|");
$p->setStub("<?php echo 'stub'; __HALT_COMPILER(); ?>");
echo $p->getStub() . "|";
echo $p->getPath() . "|" . $p->getPathname() . "|" . $p->getFilename() . "|";
$pd = new PharData("meta.tar");
$pd->setMetadata("tar-meta");
echo $pd->getMetadata() . "|" . $pd->__toString();
"#,
    );
    assert_eq!(
        out,
        "no-meta|null|has-meta|app:3|app:3|42|has-null|null-meta|cleared|no-meta|<?php echo 'stub'; __HALT_COMPILER(); ?>|meta.phar|meta.phar|meta.phar|tar-meta|meta.tar"
    );
}

/// Global metadata and the stub persist into the archive and are read back by a fresh
/// `Phar`/`PharData` object across all three families (native, tar, zip).
#[test]
fn test_phar_oop_metadata_stub_persist_across_objects() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("persist.phar");
$p->addFromString("a.txt", "alpha");
$p->setMetadata(["v" => "1.0", "n" => 5]);
$q = new Phar("persist.phar");
$m = $q->getMetadata();
echo $m["v"], ":", $m["n"], ":", ($q->hasMetadata() ? "y" : "n"), "|";
$t = new PharData("persist.tar");
$t->addFromString("b.txt", "bravo");
$t->setMetadata("tar-meta");
$t->setStub("<?php __HALT_COMPILER(); ?>");
$t2 = new PharData("persist.tar");
echo $t2->getMetadata(), ":", $t2->getStub(), "|";
echo $t2->count(), "|";
$z = new PharData("persist.zip");
$z->addFromString("c.txt", "charlie");
$z->setMetadata(["zip" => 1]);
$z2 = new PharData("persist.zip");
$zm = $z2->getMetadata();
echo $zm["zip"];
"#,
    );
    assert_eq!(
        out,
        "1.0:5:y|tar-meta:<?php __HALT_COMPILER(); ?>|1|1"
    );
}

/// `PharFileInfo::setMetadata()`/`getMetadata()`/`hasMetadata()`/`delMetadata()`
/// persist per-file metadata into the archive and round-trip across fresh objects,
/// for native PHAR, tar, and zip, including a nested entry path and scalar metadata.
#[test]
fn test_phar_oop_per_file_metadata_persist() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("perfile.phar");
$p->addFromString("a.txt", "alpha");
$p->addFromString("dir/b.txt", "bravo");
$p["a.txt"]->setMetadata(["role" => "first", "n" => 3]);
$p["dir/b.txt"]->setMetadata("nested");
$q = new Phar("perfile.phar");
$am = $q["a.txt"]->getMetadata();
echo $am["role"], ":", $am["n"], "|";
echo $q["dir/b.txt"]->getMetadata(), "|";
echo ($q["a.txt"]->hasMetadata() ? "y" : "n"), ($q["dir/b.txt"]->hasMetadata() ? "y" : "n"), "|";
$t = new PharData("perfile.tar");
$t->addFromString("c.txt", "charlie");
$t->addFromString("d.txt", "delta");
$t["c.txt"]->setMetadata(["t" => 9]);
$t2 = new PharData("perfile.tar");
$tm = $t2["c.txt"]->getMetadata();
echo $tm["t"], ":", ($t2["c.txt"]->hasMetadata() ? "y" : "n"), ($t2["d.txt"]->hasMetadata() ? "y" : "n"), "|";
$z = new PharData("perfile.zip");
$z->addFromString("e.txt", "echo");
$z["e.txt"]->setMetadata(["z" => "v"]);
$z["e.txt"]->delMetadata();
$z2 = new PharData("perfile.zip");
echo ($z2["e.txt"]->hasMetadata() ? "y" : "n");
unlink("perfile.phar");
unlink("perfile.tar");
unlink("perfile.zip");
"#,
    );
    assert_eq!(out, "first:3|nested|yy|9:yn|n");
}

/// `PharData::compress()` produces a whole-archive `.tar.gz`/`.tar.bz2` that is read
/// back transparently, and `decompress()` reverses it — entries (including a nested
/// path) survive each step.
#[test]
fn test_phar_oop_tar_whole_archive_compression() {
    let out = compile_and_run(
        r#"<?php
$p = new PharData("wac.tar");
$p->addFromString("a.txt", "alpha");
$p->addFromString("dir/b.txt", "bravo");
$gz = $p->compress(Phar::GZ);
echo $gz->count(), ":", $gz["a.txt"]->getContent(), ":", $gz["dir/b.txt"]->getContent(), "|";
$bz = $p->compress(Phar::BZ2);
echo $bz->count(), ":", $bz["a.txt"]->getContent(), "|";
$back = $gz->decompress();
echo $back->count(), ":", $back["dir/b.txt"]->getContent();
unlink("wac.tar");
unlink("wac.tar.gz");
unlink("wac.tar.bz2");
"#,
    );
    assert_eq!(out, "2:alpha:bravo|2:alpha|2:bravo");
}

/// `Phar::setSignatureAlgorithm(Phar::OPENSSL, $key)` signs the archive with RSA-SHA1
/// and `getSignature()` reads it back as an OpenSSL signature; a hash algorithm
/// (`Phar::SHA256`) rewrites the trailer and reads back as SHA-256.
#[test]
fn test_phar_oop_signature_algorithms() {
    let out = compile_and_run(
        r#"<?php
$key = <<<'KEYEOF'
-----BEGIN PRIVATE KEY-----
MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBAOuAP7xZaVfhwn9l
BaMgxKPU1ODBpuT7Ybu6Fav03TJp1BKc1wUMiXnUPraUUI2R2JxoattDe7R/LcGk
jVoPiBGGPoxxTaByd5LJZJk6MJAiGBhzQT7bkK3OMDHLQqhziefqDFfnDLt/TN7+
umuMCPtLmuF6UUXiebMzyH21x7jvAgMBAAECgYBBhL+2rgVxzrxm5vsnhEFQ9zB2
i0ncYNey+7V1zr0PfoPi3cGwhOlmfJcqAp9ak534/c/kyqSK9esL+bTdvn5zIQqC
Swt2znffaW9nC6lM/pkZcvGLETt2m0L71n6pZVkMewsGBm9YrBQFA1krC7BV674U
mlOmmYpM3LPgzmRLwQJBAPm/G7O4Stmzu5xV5qtvYX1dNZ2gydkVyfK/AwCYpfbK
8ZXntKeWCt1BER1hNBSMPacHKb0LotK3j3LNNteLHCECQQDxZdNsXNLTHylWKA/X
dyM3SH9mM6ESZP07cU7Ifq6t9zJdTfGdiyxsAjaaXxDmShL+bAjU16iwaTAGcYTB
NrMPAkEAoUGwVV7Nlbvji5I7mr4UKKoikGDdc/oJp1+GRMBLiQqI6s3ta7gJ08rL
jjjRM+NJe6u4W4RD4eL8EJhIrOv5gQJAK4Tm+8c0PtmEU0L/sCGLWMEaLquqIy3P
tXK0+FJWXYiOLOILaBKaHJK9k1EGM+4wxGtnoC+M+tjLzq2SeF7LIwJAPdLUn2Qq
eGMK12chOVcx41RxYctqsOlEKCIt011yGsV2/Mdm9ljTXeyXvNXCVOVcnHaf1v5w
rNiobfy8sSb6iw==
-----END PRIVATE KEY-----
KEYEOF;
$p = new Phar("signed.phar");
$p->addFromString("a.txt", "alpha");
$p->setSignatureAlgorithm(Phar::OPENSSL, $key);
$s = $p->getSignature();
echo $s["hash_type"], ":", strlen($s["hash"]), "|";
$p->setSignatureAlgorithm(Phar::SHA256);
$s2 = $p->getSignature();
echo $s2["hash_type"], ":", strlen($s2["hash"]);
unlink("signed.phar");
"#,
    );
    // 1024-bit RSA signature = 128 bytes = 256 uppercase-hex chars; SHA-256 = 32 bytes = 64 hex.
    assert_eq!(out, "OpenSSL:256|SHA-256:64");
}

/// Tar and zip phars carry their signature in a `.phar/signature.bin` entry rather
/// than a trailer. `PharData::setSignatureAlgorithm()` signs both families (hash and
/// OpenSSL), `getSignature()` reads them back, and the signed archive still reads.
#[test]
fn test_phar_oop_tar_zip_signatures() {
    let out = compile_and_run(
        r#"<?php
$key = <<<'KEYEOF'
-----BEGIN PRIVATE KEY-----
MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBAOuAP7xZaVfhwn9l
BaMgxKPU1ODBpuT7Ybu6Fav03TJp1BKc1wUMiXnUPraUUI2R2JxoattDe7R/LcGk
jVoPiBGGPoxxTaByd5LJZJk6MJAiGBhzQT7bkK3OMDHLQqhziefqDFfnDLt/TN7+
umuMCPtLmuF6UUXiebMzyH21x7jvAgMBAAECgYBBhL+2rgVxzrxm5vsnhEFQ9zB2
i0ncYNey+7V1zr0PfoPi3cGwhOlmfJcqAp9ak534/c/kyqSK9esL+bTdvn5zIQqC
Swt2znffaW9nC6lM/pkZcvGLETt2m0L71n6pZVkMewsGBm9YrBQFA1krC7BV674U
mlOmmYpM3LPgzmRLwQJBAPm/G7O4Stmzu5xV5qtvYX1dNZ2gydkVyfK/AwCYpfbK
8ZXntKeWCt1BER1hNBSMPacHKb0LotK3j3LNNteLHCECQQDxZdNsXNLTHylWKA/X
dyM3SH9mM6ESZP07cU7Ifq6t9zJdTfGdiyxsAjaaXxDmShL+bAjU16iwaTAGcYTB
NrMPAkEAoUGwVV7Nlbvji5I7mr4UKKoikGDdc/oJp1+GRMBLiQqI6s3ta7gJ08rL
jjjRM+NJe6u4W4RD4eL8EJhIrOv5gQJAK4Tm+8c0PtmEU0L/sCGLWMEaLquqIy3P
tXK0+FJWXYiOLOILaBKaHJK9k1EGM+4wxGtnoC+M+tjLzq2SeF7LIwJAPdLUn2Qq
eGMK12chOVcx41RxYctqsOlEKCIt011yGsV2/Mdm9ljTXeyXvNXCVOVcnHaf1v5w
rNiobfy8sSb6iw==
-----END PRIVATE KEY-----
KEYEOF;
$tar = new PharData("sig.tar");
$tar->addFromString("doc.txt", "tarbody");
$tar->setSignatureAlgorithm(Phar::SHA256);
$ts = $tar->getSignature();
echo $ts["hash_type"], ":", strlen($ts["hash"]), "|";
$zip = new PharData("sig.zip");
$zip->addFromString("doc.txt", "zipbody");
$zip->setSignatureAlgorithm(Phar::OPENSSL, $key);
$zs = $zip->getSignature();
echo $zs["hash_type"], ":", strlen($zs["hash"]), "|";
echo $tar["doc.txt"]->getContent(), ":", $zip["doc.txt"]->getContent();
unlink("sig.tar");
unlink("sig.zip");
"#,
    );
    // SHA-256 digest = 32 bytes = 64 hex; OpenSSL 1024-bit RSA = 128 bytes = 256 hex.
    assert_eq!(out, "SHA-256:64|OpenSSL:256|tarbody:zipbody");
}

/// `PharData::setZipPassword()` decrypts traditional-PKWARE (ZipCrypto) encrypted
/// ZIP entries (a compiler extension). The embedded fixture was produced by the
/// `zip --encrypt` CLI; the correct password reads the payload, a wrong one yields
/// nothing.
#[test]
fn test_phar_oop_zipcrypto_password() {
    // A real `zip --encrypt -P hunter2` archive of "secret zipcrypto payload\n".
    let out = compile_and_run(
        r#"<?php
$zip = base64_decode("UEsDBAoACQAAACWR1Fy68T/DJQAAABkAAAAMABwAemNfcGxhaW4udHh0VVQJAAMluzZqJbs2anV4CwABBPUBAAAEAAAAAIX9cegIcalT/zcAGsBrKLo1vP/AI2DJ71z0w4OcxvSzLXaea0tQSwcIuvE/wyUAAAAZAAAAUEsBAh4DCgAJAAAAJZHUXLrxP8MlAAAAGQAAAAwAGAAAAAAAAQAAAKSBAAAAAHpjX3BsYWluLnR4dFVUBQADJbs2anV4CwABBPUBAAAEAAAAAFBLBQYAAAAAAQABAFIAAAB7AAAAAAA=");
file_put_contents("enc.zip", $zip);
$p = new PharData("enc.zip");
$p->setZipPassword("hunter2");
echo $p["zc_plain.txt"]->getContent();
$wrong = new PharData("enc.zip");
$wrong->setZipPassword("nope");
echo "|len=", strlen($wrong["zc_plain.txt"]->getContent());
unlink("enc.zip");
"#,
    );
    assert_eq!(out, "secret zipcrypto payload\n|len=0");
}

/// `PharData::setZipPassword()` also encrypts on write (a compiler extension): with a
/// password set before `addFromString`, the entry is ZipCrypto-encrypted on disk and
/// round-trips back through a fresh object with the correct password, while a fresh
/// object with a wrong password cannot decrypt it.
#[test]
fn test_phar_oop_zipcrypto_write_roundtrip() {
    let out = compile_and_run(
        r#"<?php
$p = new PharData("encw.zip");
$p->setZipPassword("hunter2");
$p->addFromString("a.txt", "secret payload");
// A fresh object with the correct password decrypts the written entry.
$ok = new PharData("encw.zip");
$ok->setZipPassword("hunter2");
echo $ok["a.txt"]->getContent();
// A fresh object with a wrong password cannot decrypt it.
$bad = new PharData("encw.zip");
$bad->setZipPassword("nope");
echo "|len=", strlen($bad["a.txt"]->getContent());
unlink("encw.zip");
"#,
    );
    assert_eq!(out, "secret payload|len=0");
}

/// `Phar` and `PharData` iterate over entries written through the OOP surface.
#[test]
fn test_phar_oop_iteration_tracks_written_entries() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("iter.phar");
$p->addFromString("one.txt", "alpha");
$p["two.txt"] = "bravo";
$p->addFromString("one.txt", "alpha2");
echo ($p instanceof Iterator) ? "iter|" : "no-iter|";
echo ($p instanceof Countable) ? "countable|" : "no-count|";
echo $p->count() . "|";
foreach ($p as $name => $info) {
    echo $name . "=" . $info->getContent() . "|";
}
$p->rewind();
echo get_class($p->current()) . "|";
unset($p["two.txt"]);
echo $p->count() . "|";
foreach ($p as $name => $info) {
    echo $name . "=" . $info->getContent() . "|";
}
$pd = new PharData("iter.tar");
$pd->addFromString("tar.txt", "tar");
foreach ($pd as $name => $info) {
    echo $name . "=" . $info->getContent();
}
unlink("iter.phar");
unlink("iter.tar");
"#,
    );
    assert_eq!(
        out,
        "iter|countable|2|one.txt=alpha2|two.txt=bravo|PharFileInfo|1|one.txt=alpha2|tar.txt=tar"
    );
}

/// `Phar` and `PharData` seed iteration from archives that already exist.
#[test]
fn test_phar_oop_iteration_scans_existing_archives() {
    let out = compile_and_run(
        r#"<?php
file_put_contents("phar://scan.phar/one.txt", "alpha");
file_put_contents("phar://scan.phar/two.txt", "bravo");
$p = new Phar("scan.phar");
echo $p->count() . "|";
foreach ($p as $name => $info) {
    echo $name . "=" . $info->getContent() . "|";
}
file_put_contents("phar://scan.tar/tar.txt", "tar");
$tar = new PharData("scan.tar");
echo $tar->count() . "|";
foreach ($tar as $name => $info) {
    echo $name . "=" . $info->getContent() . "|";
}
file_put_contents("phar://scan.zip/zip.txt", "zip");
$zip = new PharData("scan.zip");
echo $zip->count() . "|";
foreach ($zip as $name => $info) {
    echo $name . "=" . $info->getContent();
}
unlink("scan.phar");
unlink("scan.tar");
unlink("scan.zip");
"#,
    );
    assert_eq!(
        out,
        "2|one.txt=alpha|two.txt=bravo|1|tar.txt=tar|1|zip.txt=zip"
    );
}

/// `Phar::compressFiles()` and `decompressFiles()` rewrite native PHAR entry
/// compression while preserving readable payloads.
#[test]
fn test_phar_oop_compress_and_decompress_files() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("compress.phar");
$p->addFromString("one.txt", "alpha alpha alpha");
$p->addFromString("two.txt", "bravo bravo bravo");
$p->compressFiles(Phar::GZ);
echo $p["one.txt"]->getContent() . "|";
echo ($p->decompressFiles() ? "plain|" : "bad|");
echo $p["two.txt"]->getContent() . "|";
$zip = new PharData("compress.zip");
$zip->addFromString("zip.txt", "zip zip zip");
$zip->compressFiles(Phar::GZ);
echo $zip["zip.txt"]->getContent() . "|";
echo ($zip->decompressFiles() ? "zip-plain|" : "zip-bad|");
echo $zip["zip.txt"]->getContent() . "|";
echo (function_exists("__elephc_phar_set_compression") ? "visible" : "hidden");
"#,
    );
    assert_eq!(
        out,
        "alpha alpha alpha|plain|bravo bravo bravo|zip zip zip|zip-plain|zip zip zip|hidden"
    );
}

/// `Phar::delete()` and `PharData::delete()` remove archive entries through the
/// same PHAR-aware unlink path as ArrayAccess unset.
#[test]
fn test_phar_oop_delete_method_removes_entries() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("delete-method.phar");
$p->addFromString("one.txt", "alpha");
$p->addFromString("two.txt", "bravo");
echo ($p->delete("one.txt") ? "deleted|" : "bad|");
echo (isset($p["one.txt"]) ? "bad|" : "missing|");
echo $p["two.txt"]->getContent() . "|";
$pd = new PharData("delete-method.tar");
$pd->addFromString("one.txt", "tar-one");
$pd->addFromString("two.txt", "tar-two");
echo ($pd->delete("one.txt") ? "deleted|" : "bad|");
echo $pd["two.txt"]->getContent();
"#,
    );
    assert_eq!(out, "deleted|missing|bravo|deleted|tar-two");
}

/// ArrayAccess `unset()` on `Phar` and `PharData` deletes the archive entry and
/// leaves other entries readable.
#[test]
fn test_phar_oop_array_access_unset_deletes_entry() {
    let out = compile_and_run(
        r#"<?php
$p = new Phar("unset.phar");
$p["one.txt"] = "alpha";
$p["two.txt"] = "bravo";
unset($p["one.txt"]);
echo (isset($p["one.txt"]) ? "bad|" : "missing|");
echo $p["two.txt"]->getContent() . "|";
$pd = new PharData("unset.tar");
$pd["one.txt"] = "tar-one";
$pd["two.txt"] = "tar-two";
unset($pd["one.txt"]);
echo (isset($pd["one.txt"]) ? "bad|" : "missing|");
echo $pd["two.txt"]->getContent();
"#,
    );
    assert_eq!(out, "missing|bravo|missing|tar-two");
}

/// `file_get_contents()` of a literal `phar://` URL decodes the entry at compile
/// time (like the fopen read fast path) and returns its bytes as a string; a
/// missing entry returns `false`.
#[test]
fn test_file_get_contents_phar_literal_entry() {
    let phar = build_minimal_phar(&[
        ("hello.txt", b"Hello from phar!\n"),
        ("dir/inner.txt", b"inner content here"),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_fgc_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php
echo file_get_contents("phar://{p}/dir/inner.txt");
echo "|" . (file_get_contents("phar://{p}/nope.txt") === false ? "false" : "open");
"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "inner content here|false");
}

/// `file_get_contents()` of a NON-literal `phar://` URL reads the entry at run
/// time (via the `__rt_file_get_contents_maybe_phar` gate → runtime reader →
/// `stream_get_contents`): write a phar literally, then read it back through a
/// runtime path; a missing entry returns `false`.
#[test]
fn test_file_get_contents_phar_runtime_path() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$f = fopen("phar://fg.phar/data.txt", "w");
fwrite($f, "runtime fgc");
fclose($f);
$p = "fg.phar";
echo file_get_contents("phar://" . $p . "/data.txt");
echo "|" . (file_get_contents("phar://" . $p . "/missing.txt") === false ? "false" : "open");
"#,
    );
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(out, "runtime fgc|false");
}

/// Verifies compiled PHP output for fopen phar missing archive returns false.
#[test]
fn test_fopen_phar_missing_archive_returns_false() {
    // A phar:// URL whose archive file does not exist lowers to PHP false,
    // matching a failed fopen().
    let out = compile_and_run(
        r#"<?php $f = @fopen("phar:///nonexistent/elephc-missing.phar/x.txt", "r"); echo $f === false ? "false" : "open";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen phar reads gzip entry.
#[test]
fn test_fopen_phar_reads_gzip_entry() {
    // PHP stores gzip-compressed phar entries as raw DEFLATE; the compiler
    // inflates them at compile time. The fixture is compressed with the same
    // flate2 encoder the compiler decodes, so the round-trip is version-stable.
    let content = b"gzip-compressed phar entry payload, repeated for ratio. ".repeat(8);
    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, &content).unwrap();
    let stored = encoder.finish().unwrap();
    assert!(stored.len() < content.len(), "fixture should actually compress");
    let phar = build_phar(&[TestPharEntry {
        name: "z.txt",
        uncompressed_size: content.len() as u32,
        stored: &stored,
        flags: 0x0000_11a4, // gzip (0x1000) | 0644
    }]);
    let path = std::env::temp_dir().join(format!("elephc_phar_m2_gz_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php $f = fopen("phar://{p}/z.txt", "r"); $s = fread($f, 8192); fclose($f); echo strlen($s) . "|" . substr($s, 0, 4);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, format!("{}|gzip", content.len()));
}

/// Verifies compiled PHP output for dynamic fopen phar reads gzip entry.
#[test]
fn test_fopen_phar_runtime_path_reads_gzip_entry() {
    // The runtime phar reader must inflate gzip entries when the archive path
    // arrives through string concatenation instead of the compile-time literal
    // fast path.
    let content = b"gzip-compressed phar entry payload, repeated for ratio. ".repeat(8);
    let mut encoder =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, &content).unwrap();
    let stored = encoder.finish().unwrap();
    assert!(stored.len() < content.len(), "fixture should actually compress");
    let phar = build_phar(&[TestPharEntry {
        name: "z.txt",
        uncompressed_size: content.len() as u32,
        stored: &stored,
        flags: 0x0000_11a4, // gzip (0x1000) | 0644
    }]);
    let path = std::env::temp_dir().join(format!("elephc_phar_rt_gz_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php $p = "{p}"; $f = fopen("phar://" . $p . "/z.txt", "r"); $s = fread($f, 8192); fclose($f); echo strlen($s) . "|" . substr($s, 0, 4);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, format!("{}|gzip", content.len()));
}

/// Verifies compiled PHP output for fopen phar reads bzip2 entry.
#[test]
fn test_fopen_phar_reads_bzip2_entry() {
    // PHP stores bzip2 phar entries as a standard bzip2 stream ("BZh..."); the
    // compiler decompresses them at compile time via the pure-Rust bzip2-rs. A
    // pure-Rust decoder can't compress, so the fixture is a precomputed bzip2
    // blob of a known 232-byte string (`"bzip2-compressed phar entry. "` x8).
    const BZIP2_BLOB: &[u8] = &[
        0x42, 0x5a, 0x68, 0x39, 0x31, 0x41, 0x59, 0x26, 0x53, 0x59, 0x61, 0x39,
        0xa6, 0xe8, 0x00, 0x00, 0x1f, 0x99, 0x80, 0x40, 0x03, 0x10, 0x00, 0x3e,
        0x63, 0xdc, 0x30, 0x20, 0x00, 0x70, 0x53, 0x09, 0xa6, 0x80, 0xd3, 0x10,
        0x2a, 0xa8, 0x0c, 0x43, 0x46, 0x1a, 0x9b, 0x0b, 0x0a, 0x0e, 0x46, 0x45,
        0xc5, 0x44, 0xc5, 0x05, 0x46, 0x06, 0xe3, 0xa1, 0x21, 0x03, 0x22, 0x42,
        0xc2, 0xe2, 0x63, 0x02, 0xe2, 0x82, 0x07, 0x82, 0x82, 0x05, 0x44, 0x0f,
        0xc5, 0xdc, 0x91, 0x4e, 0x14, 0x24, 0x18, 0x4e, 0x69, 0xba, 0x00,
    ];
    let phar = build_phar(&[TestPharEntry {
        name: "b.txt",
        uncompressed_size: 232,
        stored: BZIP2_BLOB,
        flags: 0x0000_21a4, // bzip2 (0x2000) | 0644
    }]);
    let path = std::env::temp_dir().join(format!("elephc_phar_m2_bz_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php $f = fopen("phar://{p}/b.txt", "r"); $s = fread($f, 4096); fclose($f); echo strlen($s) . "|" . substr($s, 0, 26);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "232|bzip2-compressed phar entr");
}

/// Verifies compiled PHP output for dynamic file_get_contents phar reads bzip2 entry.
#[test]
fn test_file_get_contents_phar_runtime_path_reads_bzip2_entry() {
    // Dynamic file_get_contents() routes through the runtime phar reader, so it
    // must publish libbz2 and decompress bzip2-compressed entry payloads there.
    let phar = build_phar(&[TestPharEntry {
        name: "b.txt",
        uncompressed_size: 232,
        stored: BZIP2_PHAR_BLOB,
        flags: 0x0000_21a4, // bzip2 (0x2000) | 0644
    }]);
    let path = std::env::temp_dir().join(format!("elephc_phar_rt_bz_{}.phar", std::process::id()));
    std::fs::write(&path, &phar).unwrap();
    let src = format!(
        r#"<?php $p = "{p}"; $s = file_get_contents("phar://" . $p . "/b.txt"); echo strlen($s) . "|" . substr($s, 0, 26);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "232|bzip2-compressed phar entr");
}

/// Verifies a literal `fopen("phar://...")` URL can read a tar-based PHAR container.
#[test]
fn test_fopen_phar_literal_tar_entry() {
    let archive = build_tar_phar_container(&[
        ("plain.txt", b"plain"),
        ("dir/tar.txt", b"tar payload"),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_tar_lit_{}.tar", std::process::id()));
    std::fs::write(&path, &archive).unwrap();
    let src = format!(
        r#"<?php $f = fopen("phar://{p}/dir/tar.txt", "r"); echo fread($f, 64); fclose($f);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "tar payload");
}

/// Verifies a literal `file_get_contents("phar://...")` URL can read a deflated ZIP PHAR entry.
#[test]
fn test_file_get_contents_phar_literal_zip_deflate_entry() {
    let archive = build_zip_phar_container(&[
        ("plain.txt", b"stored", false),
        ("deflated.txt", b"deflated zip payload", true),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_zip_lit_{}.zip", std::process::id()));
    std::fs::write(&path, &archive).unwrap();
    let src = format!(
        r#"<?php echo file_get_contents("phar://{p}/deflated.txt");"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "deflated zip payload");
}

/// Verifies a dynamic `file_get_contents()` PHAR URL uses the runtime bridge for tar containers.
#[test]
fn test_file_get_contents_phar_runtime_tar_entry() {
    let archive = build_tar_phar_container(&[
        ("plain.txt", b"plain"),
        ("dir/runtime.txt", b"runtime tar payload"),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_tar_rt_{}.tar", std::process::id()));
    std::fs::write(&path, &archive).unwrap();
    let src = format!(
        r#"<?php $p = "{p}"; echo file_get_contents("phar://" . $p . "/dir/runtime.txt");"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "runtime tar payload");
}

/// Verifies a dynamic `fopen()` PHAR URL uses the runtime bridge for deflated ZIP entries.
#[test]
fn test_fopen_phar_runtime_zip_deflate_entry() {
    let archive = build_zip_phar_container(&[
        ("plain.txt", b"stored", false),
        ("dir/deflated.txt", b"runtime zip payload", true),
    ]);
    let path = std::env::temp_dir().join(format!("elephc_phar_zip_rt_{}.zip", std::process::id()));
    std::fs::write(&path, &archive).unwrap();
    let src = format!(
        r#"<?php $p = "{p}"; $f = fopen("phar://" . $p . "/dir/deflated.txt", "r"); echo fread($f, 64); fclose($f);"#,
        p = path.display()
    );
    let out = compile_and_run(&src);
    std::fs::remove_file(&path).ok();
    assert_eq!(out, "runtime zip payload");
}

/// Verifies compiled PHP output for stream socket server creates listening socket.
#[test]
fn test_stream_socket_server_creates_listening_socket() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:0");
echo is_resource($srv) ? "r" : "x";
echo get_resource_type($srv);
"#,
    );
    assert_eq!(out, "rstream");
}

/// Verifies compiled PHP output for stream socket client tcp nodelay does not crash.
#[test]
fn test_stream_socket_client_tcp_nodelay_does_not_crash() {
    // socket.tcp_nodelay = 1 triggers __rt_apply_socket_client_opts after
    // connect, which sets TCP_NODELAY via setsockopt. The setsockopt result
    // isn't observable from PHP (best-effort) but the helper must not blow
    // up the connection sequence.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:0");
$addr = stream_socket_get_name($srv, false);
stream_context_set_option(stream_context_get_default(), "socket", "tcp_nodelay", 1);
$client = stream_socket_client("tcp://" . $addr);
echo is_resource($client) ? "ok" : "fail";
if ($client) { fclose($client); }
fclose($srv);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket client so broadcast does not crash.
#[test]
fn test_stream_socket_client_so_broadcast_does_not_crash() {
    // socket.so_broadcast = 1 triggers __rt_apply_socket_client_opts, which sets
    // SO_BROADCAST on the UDP socket via setsockopt. Not observable from PHP
    // (best-effort) but the option must be accepted without breaking the socket.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("udp://127.0.0.1:0");
$addr = stream_socket_get_name($srv, false);
stream_context_set_option(stream_context_get_default(), "socket", "so_broadcast", 1);
$client = stream_socket_client("udp://" . $addr);
echo is_resource($client) ? "ok" : "fail";
if ($client) { fclose($client); }
fclose($srv);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket client bindto binds local address.
#[test]
fn test_stream_socket_client_bindto_binds_local_address() {
    // socket.bindto = "127.0.0.1:0" routes through __rt_apply_socket_bindto
    // before connect(). After connect, the local end of the client socket
    // must report 127.0.0.1 as its address. The :0 lets the kernel pick
    // any free local port — we only assert on the host prefix.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:0");
$addr = stream_socket_get_name($srv, false);
stream_context_set_option(stream_context_get_default(), "socket", "bindto", "127.0.0.1:0");
$client = stream_socket_client("tcp://" . $addr);
$local = stream_socket_get_name($client, false);
echo strpos($local, "127.0.0.1:") === 0 ? "ok" : "bad";
fclose($client);
fclose($srv);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket server ipv6 v6only does not crash.
#[test]
fn test_stream_socket_server_ipv6_v6only_does_not_crash() {
    // socket.ipv6_v6only = 1 is best-effort: the option only matters for
    // IPv6 sockets, and setsockopt fails silently on a v4 socket (EINVAL).
    // The bind/listen should still succeed.
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "socket", "ipv6_v6only", 1);
$srv = stream_socket_server("tcp://127.0.0.1:0");
echo is_resource($srv) ? "ok" : "fail";
if ($srv) { fclose($srv); }
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket server so reuseport does not crash.
#[test]
fn test_stream_socket_server_so_reuseport_does_not_crash() {
    // socket.so_reuseport = 1 triggers __rt_apply_socket_server_opts after
    // the socket() call but before bind(). The setsockopt call is best-
    // effort; this test only verifies the server still binds successfully.
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "socket", "so_reuseport", 1);
$srv = stream_socket_server("tcp://127.0.0.1:0");
echo is_resource($srv) ? "ok" : "fail";
if ($srv) { fclose($srv); }
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket server backlog accepts connection.
#[test]
fn test_stream_socket_server_backlog_accepts_connection() {
    // socket.backlog (read as a string, like ftp.resume_pos) feeds the listen()
    // backlog via __rt_socket_backlog instead of the hardcoded 128. A small
    // backlog must still bind, listen, and accept at least one connection.
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "socket", "backlog", "5");
$srv = stream_socket_server("tcp://127.0.0.1:0");
$addr = stream_socket_get_name($srv, false);
$client = stream_socket_client("tcp://" . $addr);
$conn = stream_socket_accept($srv);
echo is_resource($conn) ? "accepted" : "fail";
if ($conn) { fclose($conn); }
fclose($client);
fclose($srv);
"#,
    );
    assert_eq!(out, "accepted");
}

/// Verifies compiled PHP output for stream socket server backlog default when unset.
#[test]
fn test_stream_socket_server_backlog_default_when_unset() {
    // No backlog option set: __rt_socket_backlog falls back to the default 128
    // and the server still binds (regression for the miss path).
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:0");
echo is_resource($srv) ? "ok" : "fail";
if ($srv) { fclose($srv); }
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for unix socket server backlog does not crash.
#[test]
fn test_unix_socket_server_backlog_does_not_crash() {
    // Exercises the unix_socket_server backlog site (whose ARM64 path is a leaf
    // that now spills x30 around the __rt_socket_backlog call).
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_backlog_test.sock";
@unlink($path);
stream_context_set_option(stream_context_get_default(), "socket", "backlog", "3");
$srv = stream_socket_server("unix://" . $path);
echo is_resource($srv) ? "ok" : "fail";
if ($srv) { fclose($srv); }
@unlink($path);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream socket server rejects bad address.
#[test]
fn test_stream_socket_server_rejects_bad_address() {
    let out = compile_and_run(
        r#"<?php
echo stream_socket_server("garbage") === false ? "a" : "A";
echo stream_socket_server("tcp://999.1.2.3:80") === false ? "b" : "B";
"#,
    );
    assert_eq!(out, "ab");
}

/// Verifies compiled PHP output for stream socket client connects to server.
#[test]
fn test_stream_socket_client_connects_to_server() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54731");
$cli = stream_socket_client("tcp://127.0.0.1:54731");
echo is_resource($cli) ? "connected" : "failed";
"#,
    );
    assert_eq!(out, "connected");
}

/// Mechanism guard for the enable_crypto SNI auto-default (#84): stream_socket_client
/// now records the transport host per fd via __rt_stash_connect_host before boxing
/// the result. This must not disturb the normal connect path — verify a full
/// client→server→client round-trip still works over a named-loopback address, and
/// that a failed connect (fd = -1, stash passthrough) still returns false.
#[test]
fn test_stream_socket_client_host_stash_does_not_break_connect() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54838");
$cli = stream_socket_client("tcp://127.0.0.1:54838");
$conn = stream_socket_accept($srv);
fwrite($cli, "ping");
echo fread($conn, 4);
echo is_resource($cli) ? ":ok" : ":no";
$bad = stream_socket_client("tcp://127.0.0.1:1");
echo ($bad === false) ? ":closed" : ":open";
"#,
    );
    assert_eq!(out, "ping:ok:closed");
}

/// Verifies the socket error out-parameters carry the real failure, not a fixed guess.
///
/// The two outputs used to be a hardcoded `ECONNREFUSED` / `"Connection refused"` pair on
/// `fsockopen()` and nothing at all on `stream_socket_client()`. A `unix://` path that does not
/// exist pins the distinction: the answer must be `ENOENT`, which is 2 on both supported
/// platforms, rather than the connection-refused text a fixed guess would produce.
#[test]
fn test_socket_error_outputs_report_the_real_failure() {
    let out = compile_and_run(
        r#"<?php
$c = @stream_socket_client("unix:///nonexistent/elephc-probe.sock", $errno, $errstr, 1);
echo var_export($c === false, true), "|", $errno, "|", $errstr;
"#,
    );
    assert_eq!(out, "true|2|No such file or directory");
}

/// Verifies a successful call leaves the out-parameters at PHP's "nothing went wrong" values.
#[test]
fn test_socket_error_outputs_are_empty_after_a_successful_connect() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:0", $se, $ss);
$cli = stream_socket_client("tcp://" . stream_socket_get_name($srv, false), $ce, $cs, 5);
echo var_export($cli !== false, true), "|", $se, "|", var_export($ss, true);
echo "|", $ce, "|", var_export($cs, true);
fclose($cli);
fclose($srv);
"#,
    );
    assert_eq!(out, "true|0|''|0|''");
}

/// Verifies `stream_socket_server()` describes a bind failure the way php-src does.
///
/// php-src is measurably the odd one out here: it leaves `&$error_code` at `0` for every bind
/// and listen failure and puts the reason in `&$error_message` alone. Reporting the real `errno`
/// would be more informative and would not be PHP.
#[test]
fn test_stream_socket_server_reports_a_bind_failure_through_the_message_only() {
    let out = compile_and_run(
        r#"<?php
$first = stream_socket_server("tcp://127.0.0.1:0", $e1, $s1);
$taken = stream_socket_get_name($first, false);
$second = @stream_socket_server("tcp://" . $taken, $e2, $s2);
echo var_export($second === false, true), "|", $e2, "|", $s2;
fclose($first);
"#,
    );
    assert_eq!(out, "true|0|Address already in use");
}

/// Verifies an error number does not survive into the NEXT socket call.
///
/// The failure reason lives in one process-global, so a helper that never records one — the
/// `unix://` and IPv6 paths are reached by a tail call — would otherwise hand back whatever the
/// previous failure left there. The entry of each socket helper clears it for that reason.
#[test]
fn test_socket_error_outputs_do_not_leak_between_calls() {
    let out = compile_and_run(
        r#"<?php
$a = @stream_socket_client("tcp://127.0.0.1:1", $e1, $s1, 1);
$srv = stream_socket_server("tcp://127.0.0.1:0", $e2, $s2);
echo var_export($e1 !== 0, true), "|", $e2, "|", var_export($s2, true);
fclose($srv);
"#,
    );
    assert_eq!(out, "true|0|''");
}

/// Verifies compiled PHP output for stream socket client rejects closed port.
#[test]
fn test_stream_socket_client_rejects_closed_port() {
    let out =
        compile_and_run(r#"<?php var_dump(stream_socket_client("tcp://127.0.0.1:1") === false);"#);
    assert_eq!(out, "bool(true)\n");
}

/// Verifies compiled PHP output for stream socket accept exchanges data.
#[test]
fn test_stream_socket_accept_exchanges_data() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54732");
$cli = stream_socket_client("tcp://127.0.0.1:54732");
$conn = stream_socket_accept($srv);
echo is_resource($conn) ? "a" : "x";
fwrite($cli, "ping");
echo fread($conn, 16);
"#,
    );
    assert_eq!(out, "aping");
}

/// Verifies compiled PHP output for stream socket accept timeout returns false.
#[test]
fn test_stream_socket_accept_timeout_returns_false() {
    // With no client connecting, stream_socket_accept() must respect the
    // timeout and return false instead of blocking forever. 0 seconds
    // (poll) is the strictest test of the select() gate.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54933");
$conn = stream_socket_accept($srv, 0);
echo is_bool($conn) ? "timeout" : "got_conn";
"#,
    );
    assert_eq!(out, "timeout");
}

/// Verifies compiled PHP output for stream socket accept peer name inet.
#[test]
fn test_stream_socket_accept_peer_name_inet() {
    // The optional 3rd argument receives the peer A.B.C.D:port string for
    // IPv4 connections. The client's source port is ephemeral but the
    // host part is deterministic, so check the prefix.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54934");
$cli = stream_socket_client("tcp://127.0.0.1:54934");
$peer = "";
$conn = stream_socket_accept($srv, -1, $peer);
echo is_resource($conn) ? "ok|" : "fail|";
echo substr($peer, 0, 10);
"#,
    );
    assert_eq!(out, "ok|127.0.0.1:");
}

/// Verifies compiled PHP output for stream socket accept peer name unix.
#[test]
fn test_stream_socket_accept_peer_name_unix() {
    // Unix-domain peers are anonymous unless the client bound a name first,
    // which stream_socket_client() does not do — so the peer_name slot ends
    // up as an empty string (matching PHP for unnamed Unix peers).
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_accept_peer_test.sock";
unlink($path);
$srv = stream_socket_server("unix://" . $path);
$cli = stream_socket_client("unix://" . $path);
$peer = "preseed";
$conn = stream_socket_accept($srv, -1, $peer);
echo is_resource($conn) ? "ok|" : "fail|";
echo strlen($peer);
unlink($path);
"#,
    );
    assert_eq!(out, "ok|0");
}

/// Verifies compiled PHP output for stream get line splits on delimiter.
#[test]
fn test_stream_get_line_splits_on_delimiter() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl.txt", "alpha\nbeta\ngamma");
$f = fopen("sgl.txt", "r");
echo stream_get_line($f, 100, "\n") . "|";
echo stream_get_line($f, 100, "\n") . "|";
echo stream_get_line($f, 100, "\n");
fclose($f);
unlink("sgl.txt");
"#,
    );
    assert_eq!(out, "alpha|beta|gamma");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream get line respects length cap.
#[test]
fn test_stream_get_line_respects_length_cap() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl_cap.txt", "0123456789");
$f = fopen("sgl_cap.txt", "r");
echo stream_get_line($f, 4, "\n");
fclose($f);
unlink("sgl_cap.txt");
"#,
    );
    assert_eq!(out, "0123");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream get line loop terminates at eof.
///
/// The trailing newline leaves the stream positioned before EOF, so the loop runs a
/// third time and that read returns `false`. `false !== ""` holds, so reference PHP
/// counts three — the count is 3, not 2, and `php -n` agrees.
#[test]
fn test_stream_get_line_loop_terminates_at_eof() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl_eof.txt", "x\ny\n");
$f = fopen("sgl_eof.txt", "r");
$count = 0;
while (!feof($f)) {
    $line = stream_get_line($f, 100, "\n");
    if ($line !== "") { $count = $count + 1; }
}
echo $count;
fclose($f);
unlink("sgl_eof.txt");
"#,
    );
    assert_eq!(out, "3");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_get_line()` tells an empty segment apart from an exhausted stream.
///
/// A delimiter sitting at the read position strips the segment to nothing, which PHP
/// still reports as a string; only a stream with no byte left is false. Testing this
/// with `var_dump` rather than `.` concatenation is deliberate — string coercion turns
/// both answers into "" and the divergence disappears.
#[test]
fn test_stream_get_line_returns_false_only_once_nothing_remains() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl_empty.txt", "a||||b");
$f = fopen("sgl_empty.txt", "r");
var_dump(stream_get_line($f, 100, "||"));
var_dump(stream_get_line($f, 100, "||"));
var_dump(stream_get_line($f, 100, "||"));
var_dump(stream_get_line($f, 100, "||"));
fclose($f);
unlink("sgl_empty.txt");
"#,
    );
    assert_eq!(
        out,
        "string(1) \"a\"\nstring(0) \"\"\nstring(1) \"b\"\nbool(false)\n"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a zero `$length` reads php-src's default chunk instead of nothing.
#[test]
fn test_stream_get_line_treats_zero_length_as_the_default_chunk() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl_zero.txt", str_repeat("z", 9000));
$f = fopen("sgl_zero.txt", "r");
echo strlen(stream_get_line($f, 0)), "|", ftell($f);
fclose($f);
unlink("sgl_zero.txt");
"#,
    );
    assert_eq!(out, "8192|8192");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a negative `$length` raises php-src's verbatim `ValueError`.
#[test]
fn test_stream_get_line_rejects_a_negative_length() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("sgl_neg.txt", "data");
$f = fopen("sgl_neg.txt", "r");
try {
    stream_get_line($f, -1);
} catch (ValueError $e) {
    echo $e->getMessage();
}
fclose($f);
unlink("sgl_neg.txt");
"#,
    );
    assert_eq!(
        out,
        "stream_get_line(): Argument #2 ($length) must be greater than or equal to 0"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies compiled PHP output for stream set blocking toggles mode.
#[test]
fn test_stream_set_blocking_toggles_mode() {
    let out = compile_and_run(
        r#"<?php
echo stream_set_blocking(STDIN, false) ? "n" : "N";
echo stream_set_blocking(STDIN, true) ? "b" : "B";
"#,
    );
    assert_eq!(out, "nb");
}

/// Verifies nonblocking fread/fgets misses do not mark the stream EOF.
#[test]
fn test_nonblocking_socket_reads_do_not_mark_eof() {
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
stream_set_blocking($pair[0], false);
$first = fread($pair[0], 5);
echo $first === "" ? "empty" : "data";
echo "|";
echo feof($pair[0]) ? "eof" : "open";
echo "|";
$line = fgets($pair[0]);
echo $line === false ? "false" : "line";
echo "|";
echo feof($pair[0]) ? "eof" : "open";
echo "|";
$char = fgetc($pair[0]);
echo $char === false ? "false" : "char";
echo "|";
echo feof($pair[0]) ? "eof" : "open";
echo "|";
fwrite($pair[1], "hi\n");
echo fgets($pair[0]);
echo feof($pair[0]) ? "eof" : "open";
"#,
    );
    assert_eq!(out, "empty|open|false|open|false|open|hi\nopen");
}

/// Verifies `stream_get_line()` treats a nonblocking miss as transient instead of EOF.
#[test]
fn test_nonblocking_stream_get_line_does_not_mark_eof() {
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
stream_set_blocking($pair[0], false);
$miss = stream_get_line($pair[0], 8);
echo $miss === "" ? "empty" : "data";
echo "|";
echo feof($pair[0]) ? "eof" : "open";
echo "|";
fwrite($pair[1], "ready\n");
echo stream_get_line($pair[0], 8, "\n");
"#,
    );
    // A nonblocking miss consumed no byte, so it reads as false rather than "" — the
    // point of the test is the middle field: the miss must NOT latch EOF.
    assert_eq!(out, "data|open|ready");
}

/// Verifies compiled PHP output for stream socket shutdown on connection.
#[test]
fn test_stream_socket_shutdown_on_connection() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54733");
$cli = stream_socket_client("tcp://127.0.0.1:54733");
$conn = stream_socket_accept($srv);
echo stream_socket_shutdown($conn, 2) ? "down" : "fail";
"#,
    );
    assert_eq!(out, "down");
}

/// Verifies compiled PHP output for gethostname returns nonempty string.
#[test]
fn test_gethostname_returns_nonempty_string() {
    let out = compile_and_run(r#"<?php echo strlen(gethostname()) > 0 ? "named" : "empty";"#);
    assert_eq!(out, "named");
}

/// Verifies compiled PHP output for gethostbyname resolves localhost.
#[test]
fn test_gethostbyname_resolves_localhost() {
    // gethostbyname() resolves a host name to its IPv4 address; a numeric
    // address resolves to itself.
    let out = compile_and_run(
        r#"<?php echo gethostbyname("localhost"); echo "|"; echo gethostbyname("127.0.0.1");"#,
    );
    assert_eq!(out, "127.0.0.1|127.0.0.1");
}

/// Verifies compiled PHP output for gethostbyname unresolved returns input.
#[test]
fn test_gethostbyname_unresolved_returns_input() {
    // PHP returns the host name unchanged when it cannot be resolved.
    let out = compile_and_run(r#"<?php echo gethostbyname("no-such-host.invalid");"#);
    assert_eq!(out, "no-such-host.invalid");
}

/// Verifies compiled PHP output for gethostbyaddr resolves valid address.
#[test]
fn test_gethostbyaddr_resolves_valid_address() {
    // gethostbyaddr() reverse-resolves a valid IPv4 address to a host name,
    // or returns the address unchanged when no record exists.
    let out = compile_and_run(
        r#"<?php echo strlen(gethostbyaddr("127.0.0.1")) > 0 ? "named" : "empty";"#,
    );
    assert_eq!(out, "named");
}

/// Verifies compiled PHP output for gethostbyaddr malformed address is false.
#[test]
fn test_gethostbyaddr_malformed_address_is_false() {
    // A malformed address yields PHP false.
    let out = compile_and_run(
        r#"<?php echo gethostbyaddr("not-an-ip-address") === false ? "false" : "?";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for getprotobyname known protocols.
#[test]
fn test_getprotobyname_known_protocols() {
    let out = compile_and_run(
        r#"<?php
echo getprotobyname("tcp");
echo "|";
echo getprotobyname("udp");
echo "|";
echo getprotobyname("icmp");
"#,
    );
    assert_eq!(out, "6|17|1");
}

/// Verifies compiled PHP output for getprotobyname alias and missing.
#[test]
fn test_getprotobyname_alias_and_missing() {
    let out = compile_and_run(
        r#"<?php
echo getprotobyname("TCP");
echo "|";
echo getprotobyname("no_such_protocol") === false ? "false" : "?";
"#,
    );
    assert_eq!(out, "6|false");
}

/// Verifies compiled PHP output for getprotobynumber known numbers.
#[test]
fn test_getprotobynumber_known_numbers() {
    let out = compile_and_run(
        r#"<?php
echo getprotobynumber(6);
echo "|";
echo getprotobynumber(17);
echo "|";
echo getprotobynumber(1);
"#,
    );
    assert_eq!(out, "tcp|udp|icmp");
}

/// Verifies protocol zero and its host-defined name resolve in both directions.
#[test]
fn test_protocol_zero_host_name_round_trip() {
    // Protocol zero is named "ip" on some systems and "hopopt" on others.
    let out = compile_and_run(
        r#"<?php
$name = getprotobynumber(0);
echo $name . "|" . getprotobyname($name);
"#,
    );
    let (name, number) = out
        .split_once('|')
        .expect("expected protocol zero output in name|number format");
    assert!(!name.is_empty(), "expected a non-empty protocol name");
    assert_eq!(number, "0", "expected protocol name to round-trip to zero");
}

/// Verifies compiled PHP output for getprotobynumber persists across calls.
#[test]
fn test_getprotobynumber_persists_across_calls() {
    let out = compile_and_run(
        r#"<?php
$n = getprotobynumber(6);
$m = getprotobynumber(17);
echo $n . "/" . $m;
echo "|";
echo getprotobynumber(999) === false ? "false" : "?";
"#,
    );
    assert_eq!(out, "tcp/udp|false");
}

/// Verifies compiled PHP output for getservbyname known services.
#[test]
fn test_getservbyname_known_services() {
    let out = compile_and_run(
        r#"<?php
echo getservbyname("http", "tcp");
echo "|";
echo getservbyname("https", "tcp");
echo "|";
echo getservbyname("domain", "udp");
"#,
    );
    assert_eq!(out, "80|443|53");
}

/// Verifies compiled PHP output for getservbyname alias and missing.
#[test]
fn test_getservbyname_alias_and_missing() {
    let out = compile_and_run(
        r#"<?php
echo getservbyname("www", "tcp");
echo "|";
echo getservbyname("no_such_service", "tcp") === false ? "false" : "?";
"#,
    );
    assert_eq!(out, "80|false");
}

/// Verifies compiled PHP output for getservbyport known ports.
#[test]
fn test_getservbyport_known_ports() {
    let out = compile_and_run(
        r#"<?php
echo getservbyport(80, "tcp");
echo "|";
echo getservbyport(443, "tcp");
echo "|";
echo getservbyport(53, "udp");
"#,
    );
    assert_eq!(out, "http|https|domain");
}

/// Verifies compiled PHP output for getservbyport persists and missing.
#[test]
fn test_getservbyport_persists_and_missing() {
    let out = compile_and_run(
        r#"<?php
$a = getservbyport(80, "tcp");
$b = getservbyport(22, "tcp");
echo $a . "/" . $b;
echo "|";
echo getservbyport(80, "no_such_proto") === false ? "false" : "?";
"#,
    );
    assert_eq!(out, "http/ssh|false");
}

/// Verifies compiled PHP output for stream set timeout on socket.
#[test]
fn test_stream_set_timeout_on_socket() {
    // A short receive timeout makes the no-data fread() return instead of
    // blocking forever — the test completing at all proves it took effect.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54734");
$cli = stream_socket_client("tcp://127.0.0.1:54734");
$conn = stream_socket_accept($srv);
echo stream_set_timeout($conn, 0, 50000) ? "set" : "fail";
echo "|";
$data = fread($conn, 10);
echo "done";
"#,
    );
    assert_eq!(out, "set|done");
}

/// Verifies compiled PHP output for stream socket sendto connected.
#[test]
fn test_stream_socket_sendto_connected() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54736");
$cli = stream_socket_client("tcp://127.0.0.1:54736");
$conn = stream_socket_accept($srv);
echo stream_socket_sendto($cli, "ping");
echo "|";
echo fread($conn, 16);
"#,
    );
    assert_eq!(out, "4|ping");
}

/// Verifies compiled PHP output for stream socket recvfrom connected.
#[test]
fn test_stream_socket_recvfrom_connected() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54738");
$cli = stream_socket_client("tcp://127.0.0.1:54738");
$conn = stream_socket_accept($srv);
stream_socket_sendto($cli, "first");
$a = stream_socket_recvfrom($conn, 32);
stream_socket_sendto($cli, "second");
$b = stream_socket_recvfrom($conn, 32);
echo $a . "/" . $b;
"#,
    );
    assert_eq!(out, "first/second");
}

/// Verifies compiled PHP output for stream socket recvfrom address out param.
#[test]
fn test_stream_socket_recvfrom_address_out_param() {
    // The optional 4th argument receives the sender address by reference.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("udp://127.0.0.1:54745");
$cli = stream_socket_client("udp://127.0.0.1:54745");
fwrite($cli, "hello");
$addr = "";
$data = stream_socket_recvfrom($srv, 32, 0, $addr);
echo $data . "|" . substr($addr, 0, 10);
"#,
    );
    assert_eq!(out, "hello|127.0.0.1:");
}

/// Verifies compiled PHP output for stream socket recvfrom address overwrites slot.
#[test]
fn test_stream_socket_recvfrom_address_overwrites_slot() {
    // Regression: the address write-back must overwrite the variable's
    // string slot fully — pointer and length — so a pre-seeded value of a
    // different length cannot leak into the result.
    //
    // A `socketpair`-created Unix-domain socket has no bound name, so the
    // PHP-compatible sender address is the empty string. The pre-seeded
    // "PRESEED" length still has to be reset to 0 by the writeback.
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
fwrite($pair[0], "hi");
$addr = "PRESEED";
$data = stream_socket_recvfrom($pair[1], 8, 0, $addr);
echo $data . "|" . $addr . "|" . strlen($addr);
"#,
    );
    assert_eq!(out, "hi||0");
}

/// Verifies compiled PHP output for udp socket round trip.
#[test]
fn test_udp_socket_round_trip() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("udp://127.0.0.1:54740");
$cli = stream_socket_client("udp://127.0.0.1:54740");
fwrite($cli, "udp datagram");
echo fread($srv, 32);
"#,
    );
    assert_eq!(out, "udp datagram");
}

/// Verifies compiled PHP output for stream socket sendto to udp address.
#[test]
fn test_stream_socket_sendto_to_udp_address() {
    let out = compile_and_run(
        r#"<?php
$a = stream_socket_server("udp://127.0.0.1:54741");
$b = stream_socket_server("udp://127.0.0.1:54742");
echo stream_socket_sendto($b, "abc", 0, "udp://127.0.0.1:54741");
echo "|";
echo fread($a, 16);
"#,
    );
    assert_eq!(out, "3|abc");
}

/// Verifies compiled PHP output for unix socket round trip.
#[test]
fn test_unix_socket_round_trip() {
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_unix_codegen_test.sock";
unlink($path);
$srv = stream_socket_server("unix://" . $path);
$cli = stream_socket_client("unix://" . $path);
$conn = stream_socket_accept($srv);
fwrite($cli, "unix payload");
echo fread($conn, 32);
unlink($path);
"#,
    );
    assert_eq!(out, "unix payload");
}

/// Verifies compiled PHP output for udg socket round trip.
#[test]
fn test_udg_socket_round_trip() {
    // udg:// is the Unix-domain datagram transport: the server binds (no
    // listen/accept, since datagrams are connectionless), and the client's
    // connect() sets the default destination so fwrite can send a datagram.
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_udg_codegen_test.sock";
unlink($path);
$srv = stream_socket_server("udg://" . $path);
$cli = stream_socket_client("udg://" . $path);
fwrite($cli, "udg datagram");
echo fread($srv, 32);
unlink($path);
"#,
    );
    assert_eq!(out, "udg datagram");
}

/// Verifies compiled PHP output for stream socket sendto to udg address.
#[test]
fn test_stream_socket_sendto_to_udg_address() {
    // stream_socket_sendto() accepts a udg:// target: the sender must be a
    // bound Unix-domain datagram socket, but it doesn't have to be connected
    // to the receiver. The kernel routes the datagram by sockaddr_un path.
    let out = compile_and_run(
        r#"<?php
$srv_path = "/tmp/elephc_udg_sendto_srv.sock";
$cli_path = "/tmp/elephc_udg_sendto_cli.sock";
unlink($srv_path);
unlink($cli_path);
$srv = stream_socket_server("udg://" . $srv_path);
$cli = stream_socket_server("udg://" . $cli_path);
$n = stream_socket_sendto($cli, "udg-via-sendto", 0, "udg://" . $srv_path);
echo $n . "|" . fread($srv, 32);
unlink($srv_path);
unlink($cli_path);
"#,
    );
    assert_eq!(out, "14|udg-via-sendto");
}

/// Verifies compiled PHP output for stream socket sendto to unix address.
#[test]
fn test_stream_socket_sendto_to_unix_address() {
    // stream_socket_sendto() can also target a unix:// (SOCK_STREAM) listener
    // for connectionless writes from a separately-opened socket. The kernel
    // requires the sender's socket type and the target's type to be
    // compatible, so this exercises the Unix-domain sockaddr_un build through
    // the existing socketpair (SOCK_STREAM) sender.
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_unix_sendto_test.sock";
unlink($path);
$srv = stream_socket_server("unix://" . $path);
$cli = stream_socket_client("unix://" . $path);
$conn = stream_socket_accept($srv);
$n = stream_socket_sendto($cli, "unix-via-sendto", 0, "");
echo $n . "|" . fread($conn, 32);
unlink($path);
"#,
    );
    assert_eq!(out, "15|unix-via-sendto");
}

/// Minimal one-shot passive-mode FTP server for the `ftp://` codegen test.
/// Binds the control port immediately, then serves one client on a thread by
/// dispatching on each command verb (so any login command order is accepted).
fn spawn_ftp_server(port: u16, content: &'static [u8]) -> std::thread::JoinHandle<()> {
    use std::io::{Read, Write};
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", port)).expect("ftp test: bind control port");
    std::thread::spawn(move || {
        let (mut ctrl, _) = listener.accept().expect("ftp test: accept control");
        let read_line = |s: &mut std::net::TcpStream| {
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            while s.read(&mut byte).unwrap_or(0) == 1 {
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n") {
                    break;
                }
            }
            buf
        };
        ctrl.write_all(b"220 ready\r\n").unwrap();
        let mut data_listener: Option<std::net::TcpListener> = None;
        loop {
            let cmd = read_line(&mut ctrl);
            if cmd.is_empty() {
                break;
            }
            let verb = cmd
                .split(|&b| b == b' ' || b == b'\r')
                .next()
                .unwrap_or(b"")
                .to_ascii_uppercase();
            match verb.as_slice() {
                b"USER" => ctrl.write_all(b"331 need password\r\n").unwrap(),
                b"PASS" => ctrl.write_all(b"230 logged in\r\n").unwrap(),
                b"TYPE" => ctrl.write_all(b"200 type set\r\n").unwrap(),
                b"PASV" => {
                    let dl = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
                    let dport = dl.local_addr().unwrap().port();
                    ctrl.write_all(
                        format!(
                            "227 Entering Passive Mode (127,0,0,1,{},{})\r\n",
                            dport >> 8,
                            dport & 0xff
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                    data_listener = Some(dl);
                }
                b"RETR" => {
                    let dl = data_listener.take().expect("ftp test: RETR before PASV");
                    let (mut data, _) = dl.accept().unwrap();
                    ctrl.write_all(b"150 opening data connection\r\n").unwrap();
                    data.write_all(content).unwrap();
                    drop(data);
                    ctrl.write_all(b"226 transfer complete\r\n").unwrap();
                }
                b"QUIT" => {
                    ctrl.write_all(b"221 bye\r\n").unwrap();
                    break;
                }
                _ => ctrl.write_all(b"200 ok\r\n").unwrap(),
            }
        }
    })
}

/// FTP server variant that captures every control-channel command and
/// returns the captured-command log as the data-channel body so tests
/// can assert that specific commands (REST, etc.) were sent.
fn spawn_ftp_command_echo_server(port: u16) -> std::thread::JoinHandle<()> {
    use std::io::{Read, Write};
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", port)).expect("ftp test: bind control port");
    std::thread::spawn(move || {
        let (mut ctrl, _) = listener.accept().expect("ftp test: accept control");
        let read_line = |s: &mut std::net::TcpStream| {
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            while s.read(&mut byte).unwrap_or(0) == 1 {
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n") {
                    break;
                }
            }
            buf
        };
        ctrl.write_all(b"220 ready\r\n").unwrap();
        let mut data_listener: Option<std::net::TcpListener> = None;
        let mut commands: Vec<u8> = Vec::new();
        loop {
            let cmd = read_line(&mut ctrl);
            if cmd.is_empty() {
                break;
            }
            commands.extend_from_slice(&cmd);
            let verb = cmd
                .split(|&b| b == b' ' || b == b'\r')
                .next()
                .unwrap_or(b"")
                .to_ascii_uppercase();
            match verb.as_slice() {
                b"USER" => ctrl.write_all(b"331 need password\r\n").unwrap(),
                b"PASS" => ctrl.write_all(b"230 logged in\r\n").unwrap(),
                b"TYPE" => ctrl.write_all(b"200 type set\r\n").unwrap(),
                b"PASV" => {
                    let dl = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
                    let dport = dl.local_addr().unwrap().port();
                    ctrl.write_all(
                        format!(
                            "227 Entering Passive Mode (127,0,0,1,{},{})\r\n",
                            dport >> 8,
                            dport & 0xff
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                    data_listener = Some(dl);
                }
                b"REST" => ctrl.write_all(b"350 restarting\r\n").unwrap(),
                b"RETR" => {
                    let dl = data_listener.take().expect("ftp test: RETR before PASV");
                    let (mut data, _) = dl.accept().unwrap();
                    ctrl.write_all(b"150 opening data connection\r\n").unwrap();
                    data.write_all(&commands).unwrap();
                    drop(data);
                    ctrl.write_all(b"226 transfer complete\r\n").unwrap();
                }
                b"QUIT" => {
                    ctrl.write_all(b"221 bye\r\n").unwrap();
                    break;
                }
                _ => ctrl.write_all(b"200 ok\r\n").unwrap(),
            }
        }
    })
}

/// Verifies compiled PHP output for fopen ftp resume pos sends rest command.
#[test]
fn test_fopen_ftp_resume_pos_sends_rest_command() {
    // Phase 11 B2: stream_context_create(['ftp' => ['resume_pos' => '1024']])
    // makes __rt_ftp_open send "REST 1024\r\n" between PASV and RETR.
    // The echo server captures every command and returns the log as
    // the data-channel body, so the test sees REST in the response.
    let _server = spawn_ftp_command_echo_server(54994);
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ftp", "resume_pos", "1024");
$f = fopen("ftp://127.0.0.1:54994/pub/file.txt", "r");
$log = stream_get_contents($f);
fclose($f);
echo strpos($log, "REST 1024\r\n") !== false ? "has-rest" : "no-rest";
"#,
    );
    assert_eq!(out, "has-rest");
}

/// Verifies compiled PHP output for fopen ftp no resume pos skips rest.
#[test]
fn test_fopen_ftp_no_resume_pos_skips_rest() {
    // With no resume_pos in context, the runtime must NOT send REST.
    // (Sending REST 0 would still work but pollutes the protocol — the
    // builder skips the call entirely on a missed context lookup.)
    let _server = spawn_ftp_command_echo_server(54993);
    let out = compile_and_run(
        r#"<?php
$f = fopen("ftp://127.0.0.1:54993/pub/file.txt", "r");
$log = stream_get_contents($f);
fclose($f);
echo strpos($log, "REST") !== false ? "has-rest" : "no-rest";
"#,
    );
    assert_eq!(out, "no-rest");
}

/// Verifies compiled PHP output for fopen ftp retrieves file.
#[test]
fn test_fopen_ftp_retrieves_file() {
    // fopen("ftp://...") performs the anonymous passive-mode handshake and
    // returns the data connection as a readable stream.
    let _server = spawn_ftp_server(54965, b"contents fetched over ftp");
    let out = compile_and_run(
        r#"<?php
$f = fopen("ftp://127.0.0.1:54965/pub/file.txt", "r");
echo fread($f, 64);
fclose($f);
"#,
    );
    assert_eq!(out, "contents fetched over ftp");
}

/// `file_get_contents($url)` routes a runtime `ftp://` URL through the FTP
/// wrapper open path, then slurps the returned data connection.
#[test]
fn test_file_get_contents_dynamic_ftp_url() {
    let _server = spawn_ftp_server(54966, b"dynamic contents fetched over ftp");
    let out = compile_and_run(
        r#"<?php
$url = "ftp://127.0.0.1:54966/pub/file.txt";
echo file_get_contents($url);
"#,
    );
    assert_eq!(out, "dynamic contents fetched over ftp");
}

/// `file_get_contents($url)` routes a runtime `ftps://` URL through the FTP
/// TLS path; an unreachable control port deterministically returns PHP false
/// while still exercising TLS linkage and dynamic scheme dispatch.
#[test]
fn test_file_get_contents_dynamic_ftps_unreachable_is_false() {
    let out = compile_and_run(
        r#"<?php
$url = "ftps://127.0.0.1:1/pub/file.txt";
$r = @file_get_contents($url);
echo $r === false ? "false" : "got";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen ftp invalid url is false.
#[test]
fn test_fopen_ftp_invalid_url_is_false() {
    // An ftp:// URL without a path component fails like any bad fopen().
    let out = compile_and_run(
        r#"<?php $f = fopen("ftp://host-without-path", "r"); echo is_bool($f) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// Minimal one-shot HTTP/1.0 server for the `http://` codegen test. Binds an
/// ephemeral port immediately (returned alongside the handle, so parallel and
/// orphaned test processes can never collide), then serves one client on a
/// thread: it drains the request headers and writes a close-framed response
/// whose body is `content`.
fn spawn_http_server(content: &'static [u8]) -> (std::thread::JoinHandle<()>, u16) {
    use std::io::{Read, Write};
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("http test: bind port");
    let port = listener.local_addr().expect("http test: local addr").port();
    let handle = std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().expect("http test: accept");
        // Drain the request up to the blank line that ends the headers.
        let mut req = Vec::new();
        let mut byte = [0u8; 1];
        while sock.read(&mut byte).unwrap_or(0) == 1 {
            req.push(byte[0]);
            if req.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        let header = format!(
            "HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
            content.len()
        );
        sock.write_all(header.as_bytes()).unwrap();
        sock.write_all(content).unwrap();
        // Dropping the socket closes the connection so the client sees EOF.
    });
    (handle, port)
}

/// Same shape as `spawn_http_server` but echoes the received request bytes
/// back as the response body so tests can assert on the exact wire format
/// (method, path, headers, AND body) the elephc-built request produced.
fn spawn_http_echo_server() -> (std::thread::JoinHandle<()>, u16) {
    use std::io::{Read, Write};
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("http test: bind port");
    let port = listener.local_addr().expect("http test: local addr").port();
    let handle = std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().expect("http test: accept");
        let mut req = Vec::new();
        let mut byte = [0u8; 1];
        while sock.read(&mut byte).unwrap_or(0) == 1 {
            req.push(byte[0]);
            if req.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        // If a Content-Length header is present, also drain that many body
        // bytes — otherwise POST-style requests would never have their body
        // bytes echoed back, masking real propagation bugs in the client.
        if let Some(idx) = twoway_find(&req, b"\r\nContent-Length: ") {
            let start = idx + b"\r\nContent-Length: ".len();
            let end = req[start..]
                .iter()
                .position(|&b| b == b'\r')
                .map(|p| start + p)
                .unwrap_or(req.len());
            if let Ok(n) = std::str::from_utf8(&req[start..end])
                .unwrap_or("0")
                .trim()
                .parse::<usize>()
            {
                let mut body = vec![0u8; n];
                let _ = sock.read_exact(&mut body);
                req.extend_from_slice(&body);
            }
        }
        let header = format!(
            "HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
            req.len()
        );
        sock.write_all(header.as_bytes()).unwrap();
        sock.write_all(&req).unwrap();
    });
    (handle, port)
}

/// Serves two HTTP responses on the same port: the first is a 302 with a
/// `Location:` header pointing to `final_path` on the same `127.0.0.1:port`,
/// the second is a 200 with `body`. Used to exercise the follow_location
/// path through both relative and absolute Location values.
fn spawn_http_redirect_server(
    location: &str,
    final_path: &'static str,
    body: &'static [u8],
) -> (std::thread::JoinHandle<()>, u16) {
    use std::io::{Read, Write};
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("http redirect: bind port");
    let port = listener.local_addr().expect("http redirect: local addr").port();
    // the absolute-URL fixture needs the ephemeral port inside the Location header
    let location = location.replace("{PORT}", &port.to_string());
    let handle = std::thread::spawn(move || {
        let read_until_double_crlf = |sock: &mut std::net::TcpStream| {
            let mut req = Vec::new();
            let mut byte = [0u8; 1];
            while sock.read(&mut byte).unwrap_or(0) == 1 {
                req.push(byte[0]);
                if req.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            req
        };
        // Hop 1: respond 302 redirecting to `location`.
        let (mut s1, _) = listener.accept().expect("http redirect: accept hop 1");
        let _ = read_until_double_crlf(&mut s1);
        let r1 = format!(
            "HTTP/1.0 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            location
        );
        let _ = s1.write_all(r1.as_bytes());
        drop(s1);
        // Hop 2: serve the final body. Reject any unexpected path so the
        // assertion below pinpoints redirect-target bugs.
        let (mut s2, _) = listener.accept().expect("http redirect: accept hop 2");
        let req = read_until_double_crlf(&mut s2);
        let expected = format!("GET {} HTTP/1.0", final_path);
        if !req.starts_with(expected.as_bytes()) {
            let r2 = b"HTTP/1.0 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            let _ = s2.write_all(r2);
            return;
        }
        let r2 = format!(
            "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let _ = s2.write_all(r2.as_bytes());
        let _ = s2.write_all(body);
    });
    (handle, port)
}

/// Naive bytes-substring search — avoids pulling in extra crates for the
/// http test fixture.
fn twoway_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| &haystack[i..i + needle.len()] == needle)
}

const TEST_HTTPS_CERT_PEM: &str = "\
-----BEGIN CERTIFICATE-----
MIIDDTCCAfWgAwIBAgIUYwEnFCptGtZ9bISKGHSDDyDeR78wDQYJKoZIhvcNAQEL
BQAwFjEUMBIGA1UEAwwLZWxlcGhjLXRlc3QwHhcNMjYwNjAxMTQzMzMzWhcNMzYw
NTI5MTQzMzMzWjAWMRQwEgYDVQQDDAtlbGVwaGMtdGVzdDCCASIwDQYJKoZIhvcN
AQEBBQADggEPADCCAQoCggEBALEueBZ5lUAbSBPd5gj6DdreVaIUC1sTKaOtK32f
gEgo8f+OvI7x0xZSB75t07Kz4luusaq1iYKegF61P8gI0ZpaNkj6uLVowj+Pu8/+
AMPrr11i38P701YLNvcOf4QWOnoDlRsjyzR+w4XbQmeNRrT1yUwkUQf64rZ3OkrD
tk4+VLizdj/eeoEXezGO/HzEY4vyFHA0ZC4GDT0yfjh77NOi7rY+7yr1DdbYzon/
JkPw3fV25m7StGsgr/a3i4ghVXUze88XSAYHWANUMmyJc2kxX33EAWB30n5yy0DN
ikN8emJqsRhpVU4MwlnD+5tPVBz9rgdXE8++I5i5uUvX65UCAwEAAaNTMFEwHQYD
VR0OBBYEFKx0E1bLjEIQqIzIzj0qhgpMIg0WMB8GA1UdIwQYMBaAFKx0E1bLjEIQ
qIzIzj0qhgpMIg0WMA8GA1UdEwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADggEB
AKeskQbHp//yz/LEJWqa2uCKB+05Uutg/yauByw2JGvFIdpGMXtOeFYh6PlbhVQL
rijdbW0mI0W2slefK6xsCJxFGfQY3daL2pLgoJSU0nkW7WkZh0ao292letIR9vFR
8cULtOtZZUSl8lq6Xt51mdUcCvAJgNctEI/+58YyDZBrUf0hKSjAQ2MGuZsHr8xT
S5TYFmrdKicmU53hVXsNgsCDmqENsZqP99zgqikvcrd1qfJQ95N/7thuSJtBJydk
IxMlsDmy7cFWp8ts9w+WvdxpGeZAs1M7I2N2SqTuHYVh3SJCrdA1rwtJZKTsctUJ
rmggbINQyJdm1RdcppwbOqA=
-----END CERTIFICATE-----
";

const TEST_HTTPS_KEY_PEM: &str = "\
-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCxLngWeZVAG0gT
3eYI+g3a3lWiFAtbEymjrSt9n4BIKPH/jryO8dMWUge+bdOys+JbrrGqtYmCnoBe
tT/ICNGaWjZI+ri1aMI/j7vP/gDD669dYt/D+9NWCzb3Dn+EFjp6A5UbI8s0fsOF
20JnjUa09clMJFEH+uK2dzpKw7ZOPlS4s3Y/3nqBF3sxjvx8xGOL8hRwNGQuBg09
Mn44e+zTou62Pu8q9Q3W2M6J/yZD8N31duZu0rRrIK/2t4uIIVV1M3vPF0gGB1gD
VDJsiXNpMV99xAFgd9J+cstAzYpDfHpiarEYaVVODMJZw/ubT1Qc/a4HVxPPviOY
ublL1+uVAgMBAAECggEAKW0fAMo+njWCvbplHXYxpRnU1cdv/ERXuQA1KfMQEE8a
fdEGvzlFTHOzgc+17pNmel83BR3a3+JlSz9/gSqmrzsmdBvC8g9jU28sz22pCiXh
46jJfs4zVGvc1xjZsa1s0LhjtWvCCC0XVAW22fVLMeZBwX7AP2hmd5ka1P47csF2
aDIPRPuWWCMse7u/31bJIpLOTJwLe1KmOsrk8IaQcjPUYC+WCA84N3QUwVUMVXvR
31bYy2s2fLZ/pO4EYCHJ2TDXuUSL4JYQ9ru7FPNWyGQo8cuTBexDWMiRb8qxFYNl
U5pAJuk4Om2v3CqIgCLK2PQB/lPrJkcUPEN4P5SGgQKBgQDeZux9GFcYpwZKTAr2
4rPU7ovCNTgAGyNh+5u/xaJ/6zNYDKH+EQujM35JhZR114nHYvigTzUj2VyTPMEq
ncyYoG+7sj99QqMNqIXK+d22UeYWmbSw/jf1XDzC7UHWXASViw/kL1y/jP4NXSjf
dAxSahyRnP+aYYNXAsmRWsV2YQKBgQDL8rUFs1nzX6WfHRQ5zzcPAF9XAGwkVKzQ
OKHCHfyLN9sfCnJrSOd1DU3JEwWZ6Qzl+BwAavaqDHY8PsV0pMtKSfO77yDZVFeE
ZdrJeQMv44DszZjZK/J9Vd7JDR+6Yg49+P4l438KrMsbIp/PaEe34ApgwfzU1LB5
XOORMcPZtQKBgQCk7CAc1+rmbh19BQzwbca7dTYQi1R+x6EibOnfeRh60Zieh6es
90jw+iOBM9yW0oHqaJtEjdgzQGGlEd2Q07m/yOFyh8kLA1pUq46jqUzfgbYlNlBH
HA21FnQ8fKJg6pW/q4LaTMDzjwNqN5YytiTZDLUoygrFmeBCqt98uZpKoQKBgB7W
5pSkGDf7AJpc1VAgi1zTW5dWUwPzYeZiieNGkYejvJinBcI/VfCXQGnlXHV3jiHA
MMvHYOE53S8i9sy6lpr3L8n9UORMIqe8lybcC6VUK4yjUjeUs6hMMdIJEAEpDqpE
Wnn0OqOsmVHTHINKa33cfPVAoDC2sLDJYQf1lH35AoGAd0pIqclrFb1a4Fbpq8TM
jgOspoq2Sjj+5724t8sFeg7SRMdTkA/8M1t4FsY9TNhDSI2vi6cu9013EcfVGlUB
MYQgldWOaXCRMQsHgapn+orK7iF89zA+4UDACVNiHEYS9q8CGynLckruklWdiyi3
6NdfPEjH08mFJU5npyEEa7Q=
-----END PRIVATE KEY-----
";

/// Minimal one-shot HTTPS server for deterministic `https://` wrapper tests.
/// Binds an ephemeral port and returns it alongside the handle.
fn spawn_https_server(content: &'static [u8]) -> (std::thread::JoinHandle<()>, u16) {
    use std::io::{Read, Write};
    use std::sync::Arc;

    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 0)).expect("https test: bind port");
    let port = listener.local_addr().expect("https test: local addr").port();
    let handle = std::thread::spawn(move || {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let mut cert_reader = TEST_HTTPS_CERT_PEM.as_bytes();
        let certs = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .expect("https test: parse cert");
        let mut key_reader = TEST_HTTPS_KEY_PEM.as_bytes();
        let key = rustls_pemfile::private_key(&mut key_reader)
            .expect("https test: parse private key")
            .expect("https test: private key present");
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("https test: build server config");

        let (tcp, _) = listener.accept().expect("https test: accept");
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("https test: set read timeout");
        let conn =
            rustls::ServerConnection::new(Arc::new(config)).expect("https test: new connection");
        let mut tls = rustls::StreamOwned::new(conn, tcp);
        let mut request = [0u8; 1024];
        let _ = tls.read(&mut request);
        let headers = format!("HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n", content.len());
        tls.write_all(headers.as_bytes()).expect("https test: write headers");
        tls.write_all(content).expect("https test: write body");
        tls.flush().expect("https test: flush response");
    });
    (handle, port)
}

/// Verifies compiled PHP output for fopen http method default is get.
#[test]
fn test_fopen_http_method_default_is_get() {
    // Without a stream context, the request method falls back to "GET".
    // The echo server reflects the request bytes; the response body must
    // start with "GET /path HTTP/1.0\r\n".
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/echo", "r");
$req = stream_get_contents($f);
fclose($f);
echo substr($req, 0, 19);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "GET /echo HTTP/1.0\r");
}

/// Verifies compiled PHP output for fopen http method overrides via context.
#[test]
fn test_fopen_http_method_overrides_via_context() {
    // Phase 11 B2: stream_context_create(['http' => ['method' => 'POST']])
    // propagates through __rt_http_build_request → the request line
    // starts with "POST" instead of the default "GET".
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "method", "POST");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/api", "r");
$req = stream_get_contents($f);
fclose($f);
echo substr($req, 0, 21);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "POST /api HTTP/1.0\r\nH");
}

/// Verifies compiled PHP output for fopen http header inserted via context.
#[test]
fn test_fopen_http_header_inserted_via_context() {
    // Phase 11 B2: stream_context_create(['http' => ['header' => ...]])
    // propagates through __rt_http_build_request — the supplied header
    // line lands between the Host: line and the Connection: close line.
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "header", "X-Trace: abc");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/path", "r");
$req = stream_get_contents($f);
fclose($f);
echo strpos($req, "\r\nX-Trace: abc\r\n") !== false ? "has-header" : "no-header";
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "has-header");
}

/// Verifies compiled PHP output for fopen http content only emits body.
#[test]
fn test_fopen_http_content_only_emits_body() {
    // Reduced repro of the POST + content gap: set only ['http']['content']
    // without 'method'. If this passes, the bug is in set_option_4's two-call
    // sub-hash merge; if this fails, it's in the content lookup or emission.
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "content", "x=y");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/p", "r");
$req = stream_get_contents($f);
fclose($f);
$has_clen = strpos($req, "\r\nContent-Length: 3\r\n") !== false;
$has_body = strpos($req, "\r\n\r\nx=y") !== false;
echo ($has_clen ? "clen-ok" : "clen-MISSING") . "|" . ($has_body ? "body-ok" : "body-MISSING");
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "clen-ok|body-ok");
}

/// Verifies compiled PHP output for fopen http content post body with content length.
#[test]
fn test_fopen_http_content_post_body_with_content_length() {
    // Phase 11 B2 + post-deliverable: setting ['http']['content'] alongside
    // ['method' => 'POST'] propagates a Content-Length: N header and writes
    // the body bytes after the blank line. The echo server reflects the
    // raw request bytes so we can grep for both the header and the body.
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "method", "POST");
stream_context_set_option(stream_context_get_default(), "http", "content", "foo=bar&baz=qux");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/submit", "r");
$req = stream_get_contents($f);
fclose($f);
$has_clen = strpos($req, "\r\nContent-Length: 15\r\n") !== false;
$has_body = strpos($req, "\r\n\r\nfoo=bar&baz=qux") !== false;
echo ($has_clen ? "clen-ok" : "clen-MISSING") . "|" . ($has_body ? "body-ok" : "body-MISSING");
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "clen-ok|body-ok");
}

/// Verifies compiled PHP output for fopen http retrieves body.
#[test]
fn test_fopen_http_retrieves_body() {
    // fopen("http://...") issues an HTTP GET and exposes the response body
    // with the headers stripped as a readable stream.
    let (_server, port) = spawn_http_server(b"body delivered over http");
    let out = compile_and_run(&format!(
        r#"<?php
$f = fopen("http://127.0.0.1:{port}/page.txt", "r");
echo stream_get_contents($f);
fclose($f);
"#
    ));
    assert_eq!(out, "body delivered over http");
}

/// `file_get_contents("http://...")` opens the `http://` wrapper, slurps the
/// whole response body (headers stripped) into an owned string, and returns it
/// — equivalent to `fopen()` + `stream_get_contents()` + `fclose()` on the URL.
/// The owned-heap copy (via `__rt_str_persist`) survives the concat below.
#[test]
fn test_file_get_contents_over_http() {
    let (_server, port) = spawn_http_server(b"fgc over http body");
    let out = compile_and_run(&format!(
        r#"<?php
echo "[" . file_get_contents("http://127.0.0.1:{port}/page.txt") . "]";
"#
    ));
    assert_eq!(out, "[fgc over http body]");
}

/// `file_get_contents($url)` routes a runtime string beginning with `http://`
/// through the HTTP wrapper instead of the plain filesystem reader.
#[test]
fn test_file_get_contents_dynamic_http_url() {
    let (_server, port) = spawn_http_server(b"dynamic fgc over http");
    let out = compile_and_run(&format!(
        r#"<?php
$url = "http://127.0.0.1:{port}/page.txt";
echo "[" . file_get_contents($url) . "]";
"#
    ));
    assert_eq!(out, "[dynamic fgc over http]");
}

/// `file_get_contents("https://...")` succeeds against a local TLS server,
/// proving the literal HTTPS wrapper path returns an owned response body.
#[test]
fn test_file_get_contents_over_https_local_server() {
    let (_server, port) = spawn_https_server(b"fgc over local https");
    let out = compile_and_run(&format!(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "verify_peer", "0");
echo "[" . file_get_contents("https://127.0.0.1:{port}/page.txt") . "]";
"#
    ));
    assert_eq!(out, "[fgc over local https]");
}

/// `file_get_contents($url)` also succeeds when the runtime string uses
/// `https://`, covering the non-literal dynamic URL dispatcher.
#[test]
fn test_file_get_contents_dynamic_https_local_server() {
    let (_server, port) = spawn_https_server(b"dynamic fgc over local https");
    let out = compile_and_run(&format!(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "verify_peer", "0");
$url = "https://127.0.0.1:{port}/page.txt";
echo "[" . file_get_contents($url) . "]";
"#
    ));
    assert_eq!(out, "[dynamic fgc over local https]");
}

/// `file_get_contents($url)` routes a runtime `https://` URL through the HTTPS
/// wrapper dispatcher. A bad cafile fails before network I/O, making the TLS
/// path deterministic while still covering dynamic HTTPS linkage and parsing.
#[test]
fn test_file_get_contents_dynamic_https_cafile_bad_path_is_false() {
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "cafile", "/nonexistent/elephc/ca.pem");
$url = "https://127.0.0.1:9/";
$r = @file_get_contents($url);
echo $r === false ? "false" : "got";
"#,
    );
    assert_eq!(out, "false");
}

/// `file_get_contents()` of an unreachable `http://` URL returns PHP `false`
/// (the wrapper open fails, so the result boxes bool false).
#[test]
fn test_file_get_contents_over_http_failure_is_false() {
    let out = compile_and_run(
        r#"<?php
$r = file_get_contents("http://127.0.0.1:1/nope");
echo $r === false ? "false" : "got";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen http follow location relative path.
#[test]
fn test_fopen_http_follow_location_relative_path() {
    // 302 with a Location: /new redirects to the same host. The redirect
    // loop in __rt_http_open re-issues GET /new and serves the second body.
    let (_server, port) = spawn_http_redirect_server("/new", "/new", b"after-relative-redirect");
    let out = compile_and_run(&format!(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "follow_location", "1");
stream_context_set_option(stream_context_get_default(), "http", "max_redirects", "5");
$f = fopen("http://127.0.0.1:{port}/start", "r");
echo stream_get_contents($f);
fclose($f);
"#
    ));
    assert_eq!(out, "after-relative-redirect");
}

/// Verifies compiled PHP output for fopen http follow location absolute same host.
#[test]
fn test_fopen_http_follow_location_absolute_same_host() {
    // 302 with a Location: http://127.0.0.1:53902/final — same-host absolute
    // URLs are rewritten to /final and followed exactly like a relative
    // redirect. The fixture rejects any path other than /final, so this
    // test fails if the host:port parsing leaves stray prefix bytes in the
    // redirect path buffer.
    let (_server, port) = spawn_http_redirect_server(
        "http://127.0.0.1:{PORT}/final",
        "/final",
        b"after-absolute-redirect",
    );
    let out = compile_and_run(&format!(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "follow_location", "1");
stream_context_set_option(stream_context_get_default(), "http", "max_redirects", "5");
$f = fopen("http://127.0.0.1:{port}/start", "r");
echo stream_get_contents($f);
fclose($f);
"#
    ));
    assert_eq!(out, "after-absolute-redirect");
}

/// Verifies compiled PHP output for fopen http follow location cross host is not followed.
#[test]
fn test_fopen_http_follow_location_cross_host_is_not_followed() {
    // 302 with a Location: pointing to a different host:port is NOT followed
    // (cross-host redirect requires reconnecting, deferred for v1). The
    // initial 302 response is surfaced as-is; the body is empty because the
    // redirect response itself has Content-Length: 0.
    let (_server, port) = spawn_http_redirect_server(
        "http://other-host.invalid:80/whatever",
        "/never-reached",
        b"unreachable",
    );
    let out = compile_and_run(&format!(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "follow_location", "1");
stream_context_set_option(stream_context_get_default(), "http", "max_redirects", "5");
stream_context_set_option(stream_context_get_default(), "http", "ignore_errors", "1");
$f = fopen("http://127.0.0.1:{port}/start", "r");
echo strlen(stream_get_contents($f));
fclose($f);
"#
    ));
    assert_eq!(out, "0");
}

/// Verifies compiled PHP output for fopen ftps invalid url is false.
#[test]
fn test_fopen_ftps_invalid_url_is_false() {
    // An ftps:// URL with no authority fails at compile-time URL parsing,
    // mirroring the existing https:// invalid-URL test. The binary still
    // links elephc-tls, so a passing test exercises the whole linkage path
    // (TLS function-pointer slots, the runtime helper, and the runner's
    // -L target/debug wiring) before any real network IO.
    let out = compile_and_run(
        r#"<?php $f = fopen("ftps://", "r"); echo is_bool($f) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen ftps unreachable host is false.
#[test]
fn test_fopen_ftps_unreachable_host_is_false() {
    // ftps://127.0.0.1:1/foo — port 1 is unbound so __rt_stream_socket_client
    // returns -1 and __rt_ftp_open falls into the fail path. Returns false
    // without exploding the AUTH TLS dance.
    let out = compile_and_run(
        r#"<?php $f = @fopen("ftps://127.0.0.1:1/x", "r"); echo is_bool($f) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// `file_get_contents("ftps://...")` reuses the ftps:// wrapper open plus the
/// shared slurp path; an unreachable host fails the open so the result is PHP
/// false. Also exercises the elephc-tls linkage the checker requires for ftps.
#[test]
fn test_file_get_contents_over_ftps_unreachable_is_false() {
    let out = compile_and_run(
        r#"<?php $r = @file_get_contents("ftps://127.0.0.1:1/x"); echo $r === false ? "false" : "got";"#,
    );
    assert_eq!(out, "false");
}

/// `file_get_contents("ftp://...")` over an unreachable host returns PHP false
/// (the ftp:// wrapper open fails), completing the URL-scheme coverage next to
/// the http:// success test.
#[test]
fn test_file_get_contents_over_ftp_unreachable_is_false() {
    let out = compile_and_run(
        r#"<?php $r = @file_get_contents("ftp://127.0.0.1:1/x"); echo $r === false ? "false" : "got";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen http invalid url is false.
#[test]
fn test_fopen_http_invalid_url_is_false() {
    // An http:// URL with no authority fails like any bad fopen().
    let out = compile_and_run(
        r#"<?php $f = fopen("http://", "r"); echo is_bool($f) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen https invalid url is false.
#[test]
fn test_fopen_https_invalid_url_is_false() {
    // An https:// URL with no authority fails at compile-time URL parsing.
    // The binary still links against the elephc-tls staticlib, so a passing
    // test here verifies the whole linkage path (TLS function pointer slots,
    // the runtime helper, the runner's -L target/debug wiring) before any
    // real network IO is involved.
    let out = compile_and_run(
        r#"<?php $f = fopen("https://", "r"); echo is_bool($f) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen https cafile bad path is false.
#[test]
fn test_fopen_https_cafile_bad_path_is_false() {
    // ssl.cafile routes the connect through elephc_tls_connect_cafile, which
    // loads the CA bundle BEFORE any TCP connect. A nonexistent cafile fails to
    // load → the connect returns -1 → fopen() returns false. This exercises the
    // cafile dispatch branch + the elephc-tls linkage deterministically (no
    // network), since the failure happens during cafile load.
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "cafile", "/nonexistent/elephc/ca.pem");
$f = @fopen("https://127.0.0.1:9/", "r");
echo ($f === false) ? "false" : "open";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen https capath bad path is false.
#[test]
fn test_fopen_https_capath_bad_path_is_false() {
    // OOS Phase C: ssl.capath routes the connect through elephc_tls_connect_capath,
    // which scans the directory for CA certs BEFORE any TCP connect. A nonexistent
    // directory yields no certs → the connect returns -1 → fopen() returns false.
    // Exercises the capath dispatch branch + linkage deterministically (no network).
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "capath", "/nonexistent/elephc/cadir");
$f = @fopen("https://127.0.0.1:9/", "r");
echo ($f === false) ? "false" : "open";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen https peer name and relaxed options fail closed.
#[test]
fn test_fopen_https_peer_name_and_relaxed_options_fail_closed() {
    // OOS Phase C: ssl.peer_name routes through elephc_tls_connect_peer_name
    // (verify the cert for a different name), and ssl.allow_self_signed /
    // ssl.verify_peer_name = "0" route through the relaxed (insecure) verifier.
    // Each connects to an unreachable port, so the connect fails and fopen()
    // returns false — this exercises the new dispatch branches + the elephc-tls
    // linkage deterministically (no live TLS server needed).
    let out = compile_and_run(
        r#"<?php
$d = stream_context_get_default();
stream_context_set_option($d, "ssl", "peer_name", "example.com");
echo (@fopen("https://127.0.0.1:9/", "r") === false) ? "P" : "p";
stream_context_set_option($d, "ssl", "peer_name", "");
stream_context_set_option($d, "ssl", "allow_self_signed", "1");
echo (@fopen("https://127.0.0.1:9/", "r") === false) ? "S" : "s";
stream_context_set_option($d, "ssl", "allow_self_signed", "");
stream_context_set_option($d, "ssl", "verify_peer_name", "0");
echo (@fopen("https://127.0.0.1:9/", "r") === false) ? "V" : "v";
"#,
    );
    assert_eq!(out, "PSV");
}

/// End-to-end smoke against a real HTTPS host pinned to a custom CA bundle via
/// `ssl.cafile`. Requires outbound network plus a CA file on disk that signs
/// the host's chain, so it is `#[ignore]`d; it documents the manual
/// verification path for the cafile connect variant.
#[test]
#[ignore]
fn test_fopen_https_cafile_custom_bundle() {
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "cafile", "/etc/ssl/cert.pem");
$f = fopen("https://example.com/", "r");
echo substr(stream_get_contents($f), 0, 15);
fclose($f);
"#,
    );
    assert_eq!(out, "<!doctype html>");
}

/// End-to-end smoke against a real HTTPS host with `ssl.verify_peer = false`.
/// example.com obviously has a valid cert, so this just exercises the
/// dispatcher: with verify_peer disabled the runtime must pick the insecure
/// connect path and still return a usable body. `#[ignore]` because it
/// requires outbound network access.
#[test]
#[ignore]
fn test_fopen_https_real_example_com_with_verify_peer_disabled() {
    let out = compile_and_run(
        r#"<?php
stream_context_set_option(stream_context_get_default(), "ssl", "verify_peer", "0");
$f = fopen("https://example.com/", "r");
$body = stream_get_contents($f);
fclose($f);
echo substr($body, 0, 15);
"#,
    );
    assert_eq!(out, "<!doctype html>");
}

/// End-to-end smoke against a real HTTPS host. The test is `#[ignore]`d
/// because it needs outbound network access, just like the rustls-level test
/// in `crates/elephc-tls`; run with `cargo test -- --ignored` to exercise it.
#[test]
#[ignore]
fn test_fopen_https_real_example_com() {
    let out = compile_and_run(
        r#"<?php
$f = fopen("https://example.com/", "r");
$body = stream_get_contents($f);
fclose($f);
echo substr($body, 0, 15);
"#,
    );
    assert_eq!(out, "<!doctype html>");
}

/// End-to-end smoke for `file_get_contents("https://...")` against a real
/// HTTPS host. Ignored because it needs outbound network access and a currently
/// trusted public certificate chain.
#[test]
#[ignore]
fn test_file_get_contents_https_real_example_com() {
    let out = compile_and_run(
        r#"<?php
$body = file_get_contents("https://example.com/");
echo substr($body, 0, 15);
"#,
    );
    assert_eq!(out, "<!doctype html>");
}

/// End-to-end smoke for dynamic `file_get_contents($url)` over HTTPS. Ignored
/// for the same outbound-network reason as the fopen HTTPS smoke tests.
#[test]
#[ignore]
fn test_file_get_contents_dynamic_https_real_example_com() {
    let out = compile_and_run(
        r#"<?php
$url = "https://example.com/";
$body = file_get_contents($url);
echo substr($body, 0, 15);
"#,
    );
    assert_eq!(out, "<!doctype html>");
}

/// End-to-end real-TLS handshake through `stream_socket_enable_crypto`: open a
/// plain TCP socket to a real HTTPS host, promote it to TLS in place (SNI /
/// cert-name taken from the `ssl.peer_name` context), then exchange an encrypted
/// HTTP request/response over the upgraded fd. Proves the rustls
/// `elephc_tls_attach_fd` path and the fread/fwrite TLS routing actually work,
/// not just the return-shape mechanism the non-ignored tests pin. `#[ignore]`d
/// because it needs outbound network access; run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn test_stream_socket_enable_crypto_real_tls_handshake() {
    let out = compile_and_run(
        r#"<?php
stream_context_create(["ssl" => ["peer_name" => "example.com"]]);
$fp = stream_socket_client("tcp://example.com:443");
$ok = stream_socket_enable_crypto($fp, true, STREAM_CRYPTO_METHOD_TLS_CLIENT);
fwrite($fp, "GET / HTTP/1.0\r\nHost: example.com\r\nConnection: close\r\n\r\n");
$status = substr(fread($fp, 64), 0, 12);
fclose($fp);
echo ($ok ? "1" : "0") . "|" . $status;
"#,
    );
    assert_eq!(out, "1|HTTP/1.1 200");
}

/// End-to-end real-TLS teardown through `stream_socket_enable_crypto(false)`.
/// It upgrades a TCP socket to TLS, proves encrypted I/O works, then disables
/// crypto and closes the descriptor. Ignored because it needs outbound network.
#[test]
#[ignore]
fn test_stream_socket_enable_crypto_real_tls_disable_teardown() {
    let out = compile_and_run(
        r#"<?php
stream_context_create(["ssl" => ["peer_name" => "example.com"]]);
$fp = stream_socket_client("tcp://example.com:443");
$enabled = stream_socket_enable_crypto($fp, true, STREAM_CRYPTO_METHOD_TLS_CLIENT);
fwrite($fp, "GET / HTTP/1.0\r\nHost: example.com\r\nConnection: close\r\n\r\n");
$status = substr(fread($fp, 64), 0, 12);
$disabled = stream_socket_enable_crypto($fp, false);
fclose($fp);
echo ($enabled ? "1" : "0") . "|" . $status . "|" . ($disabled ? "1" : "0");
"#,
    );
    assert_eq!(out, "1|HTTP/1.1 200|1");
}

/// Minimal one-shot TCP server for the `fsockopen` codegen test. Binds the
/// port immediately, then serves one client on a thread by writing `content`
/// and closing the connection.
fn spawn_tcp_server(port: u16, content: &'static [u8]) -> std::thread::JoinHandle<()> {
    use std::io::Write;
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", port)).expect("tcp test: bind port");
    std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().expect("tcp test: accept");
        sock.write_all(content).unwrap();
        // Dropping the socket closes the connection so the client sees EOF.
    })
}

/// Minimal TCP server that writes two payload fragments with a pause between
/// them, forcing clients that request more bytes than the first fragment to
/// observe a short read before the rest of the payload arrives.
fn spawn_chunked_tcp_server(
    port: u16,
    first: &'static [u8],
    second: &'static [u8],
) -> std::thread::JoinHandle<()> {
    use std::io::Write;
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", port)).expect("tcp test: bind port");
    std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().expect("tcp test: accept");
        sock.write_all(first).unwrap();
        sock.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(150));
        sock.write_all(second).unwrap();
    })
}

/// Verifies finite `stream_get_contents()` loops across short socket reads
/// until the requested length is filled, then leaves the remaining socket bytes
/// available for the next read.
#[test]
fn test_stream_get_contents_bounded_socket_read_fills_length() {
    let _server = spawn_chunked_tcp_server(54989, b"ab", b"cdefghi");
    let out = compile_and_run(
        r#"<?php
$s = stream_socket_client("tcp://127.0.0.1:54989");
echo stream_get_contents($s, 5);
echo "|" . stream_get_contents($s);
fclose($s);
"#,
    );
    assert_eq!(out, "abcde|fghi");
}

/// Verifies compiled PHP output for fsockopen connects and reads.
#[test]
fn test_fsockopen_connects_and_reads() {
    // fsockopen() connects a TCP socket; on success the error outputs are
    // cleared and the connected stream is readable.
    let _server = spawn_tcp_server(54990, b"data over fsockopen");
    let out = compile_and_run(
        r#"<?php
$errno = -1;
$errstr = "unset";
$s = fsockopen("127.0.0.1", 54990, $errno, $errstr);
echo ($s === false) ? "FAIL" : "ok";
echo "|errno=" . $errno;
echo "|errstr=[" . $errstr . "]";
echo "|" . stream_get_contents($s);
fclose($s);
"#,
    );
    assert_eq!(out, "ok|errno=0|errstr=[]|data over fsockopen");
}

/// Verifies compiled PHP output for fsockopen refused sets error.
#[test]
fn test_fsockopen_refused_sets_error() {
    // A refused connection returns false and fills the by-reference error
    // outputs; the error code is non-zero and the message is set.
    let out = compile_and_run(
        r#"<?php
$errno = 0;
$errstr = "";
$s = fsockopen("127.0.0.1", 54991, $errno, $errstr);
echo ($s === false) ? "false" : "resource";
echo "|" . ($errno !== 0 ? "errno-set" : "errno-zero");
echo "|" . $errstr;
"#,
    );
    assert_eq!(out, "false|errno-set|Connection refused");
}

/// Verifies compiled PHP output for pfsockopen connects and reads.
#[test]
fn test_pfsockopen_connects_and_reads() {
    // pfsockopen() is an alias of fsockopen() — persistence is meaningless in a
    // standalone compiled binary, so it connects, reads, and clears the
    // by-reference error outputs identically to fsockopen().
    let _server = spawn_tcp_server(54992, b"data over pfsockopen");
    let out = compile_and_run(
        r#"<?php
$errno = -1;
$errstr = "unset";
$s = pfsockopen("127.0.0.1", 54992, $errno, $errstr);
echo ($s === false) ? "FAIL" : "ok";
echo "|errno=" . $errno;
echo "|errstr=[" . $errstr . "]";
echo "|" . stream_get_contents($s);
fclose($s);
"#,
    );
    assert_eq!(out, "ok|errno=0|errstr=[]|data over pfsockopen");
}

/// Verifies compiled PHP output for stream wrapper register records class.
#[test]
fn test_stream_wrapper_register_records_class() {
    // stream_wrapper_register() stores the user wrapper registration. v1
    // accepts up to 16 entries and returns true; the wrapper class is not
    // yet invoked by fopen.
    let out = compile_and_run(
        r#"<?php
class CustomWrapper {}
echo stream_wrapper_register("custom", "CustomWrapper") ? "true" : "false";
echo "|";
echo stream_wrapper_register("alt", "CustomWrapper", 0) ? "true" : "false";
"#,
    );
    assert_eq!(out, "true|true");
}

/// Verifies compiled PHP output for stream wrapper unregister round trip.
#[test]
fn test_stream_wrapper_unregister_round_trip() {
    // unregister removes a previously-registered protocol, then a fresh
    // register of the same protocol succeeds; unregistering an unknown
    // protocol returns false.
    let out = compile_and_run(
        r#"<?php
class W {}
stream_wrapper_register("foo", "W");
echo stream_wrapper_unregister("foo") ? "true" : "false";
echo "|";
echo stream_wrapper_unregister("foo") ? "true" : "false";
echo "|";
echo stream_wrapper_register("foo", "W") ? "true" : "false";
"#,
    );
    assert_eq!(out, "true|false|true");
}

/// Verifies `stream_wrapper_restore()` answers PHP's three cases, diagnostics included.
///
/// php 8.5.6 distinguishes them: a built-in that `stream_wrapper_unregister()` disabled is
/// restored silently and reports `true`; a built-in that was never disabled reports `true`
/// with a Notice; a scheme that never existed reports `false` with a Warning. The return
/// values already matched — the two diagnostics were missing.
///
/// Severity decides the stream, following what elephc does for every other diagnostic:
/// Notices go to stdout through the output-buffer funnel, Warnings to stderr through the
/// `@`-aware path. PHP CLI puts both on stdout; that divergence is repo-wide, not specific
/// to this builtin.
#[test]
fn test_stream_wrapper_restore_reports_phps_three_cases() {
    let out = compile_and_run_capture(
        r#"<?php
var_dump(stream_wrapper_restore("file"));
var_dump(stream_wrapper_restore("nosuch"));
stream_wrapper_unregister("file");
var_dump(stream_wrapper_restore("file"));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "Notice: stream_wrapper_restore(): file:// was never changed, nothing to restore\n\
         bool(true)\n\
         bool(false)\n\
         bool(true)\n"
    );
    assert_eq!(
        out.stderr,
        "Warning: stream_wrapper_restore(): nosuch:// never existed, nothing to restore\n"
    );
}

/// Verifies `@` suppresses the unknown-scheme Warning, as it does every runtime warning.
#[test]
fn test_stream_wrapper_restore_warning_is_suppressible() {
    let out = compile_and_run_capture(
        r#"<?php var_dump(@stream_wrapper_restore("nosuch"));"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "bool(false)\n");
    assert_eq!(out.stderr, "");
}

/// Verifies compiled PHP output for stream socket enable crypto reads peer name from context.
#[test]
fn test_stream_socket_enable_crypto_reads_peer_name_from_context() {
    // Phase 11 B3 follow-up: enable_crypto navigates
    // _stream_context_options["ssl"]["peer_name"] for the SNI hint via
    // __rt_get_ssl_peer_name. We can't reach a real TLS server in tests
    // (the rustls handshake needs a live remote), so the contract pinned
    // here is "this code path doesn't crash and still returns a bool" —
    // exercising the helper's two nested hash_get's plus its hit branch
    // (peer_name is in context). Also asserts the options round-trip
    // through stream_context_get_options.
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create(["ssl" => ["peer_name" => "example.com"]]);
$m = fopen("php://memory", "r+");
$r = stream_socket_enable_crypto($m, true);
echo is_bool($r) ? "bool|" : "non-bool|";
echo count(stream_context_get_options($ctx));
fclose($m);
"#,
    );
    assert_eq!(out, "bool|1");
}

/// Verifies compiled PHP output for stream socket enable crypto returns bool.
#[test]
fn test_stream_socket_enable_crypto_returns_bool() {
    // Phase 11 B3: stream_socket_enable_crypto invokes elephc_tls_attach_fd
    // on the fd. The rustls ClientConnection::new completes synchronously
    // (no I/O yet), so attach reports success even on degenerate fds like
    // php://memory; the failure surfaces on the first fread/fwrite when the
    // handshake actually runs. The shape of the return is the contract this
    // test pins — production code should also verify by attempting a read.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
$r = stream_socket_enable_crypto($m, true);
echo is_bool($r) ? "bool" : "non-bool";
fclose($m);
"#,
    );
    assert_eq!(out, "bool");
}

/// `stream_socket_enable_crypto($s, false)` unwinds a live TLS session: the
/// disable path reloads the fd and runs the shared `emit_tls_session_teardown`,
/// which (because the prior enable installed a non-zero `_tls_sessions[fd]`
/// handle) calls `_elephc_tls_close_fn` to send `close_notify` and zeroes the
/// slot, then reports `true`. The contract pinned here is that the enable→disable
/// sequence runs the real teardown branch without crashing and returns a `bool`
/// `true`; a plain-stream read-back is intentionally not asserted because the
/// `close_notify` record pollutes a degenerate `php://memory` backing buffer.
#[test]
fn test_stream_socket_enable_crypto_disable_tears_down_session() {
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
$a = stream_socket_enable_crypto($m, true);
$b = stream_socket_enable_crypto($m, false);
echo (is_bool($a) && is_bool($b) && $b === true) ? "ok" : "bad";
fclose($m);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies that the shared signature accepts the fourth named `session_stream` arg.
#[test]
fn test_stream_socket_enable_crypto_accepts_named_session_stream() {
    let out = compile_and_run(
        r#"<?php
function session_arg($stream) {
    echo "S";
    return $stream;
}
$m = fopen("php://memory", "r+");
$r = stream_socket_enable_crypto(stream: $m, enable: false, session_stream: session_arg($m));
echo $r ? "T" : "F";
fclose($m);
"#,
    );
    assert_eq!(out, "ST");
}

/// `ssl.local_cert` + `ssl.local_pk` select the mutual-TLS (client-certificate)
/// attach variant. A bogus cert/key path fails the client-auth config load
/// before any network I/O, so enable_crypto returns `false` — unlike the plain
/// server-auth attach, which reports `true` synchronously (see
/// `test_stream_socket_enable_crypto_returns_bool`). This pins that the
/// client-cert path is selected from the context and fails gracefully. A
/// successful client-cert handshake needs a client-auth-requiring server, so it
/// is covered by the `elephc-tls` crate unit tests instead.
#[test]
fn test_stream_socket_enable_crypto_client_cert_bad_path_fails() {
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create(['ssl' => ['local_cert' => '/nonexistent/elephc-cc.pem', 'local_pk' => '/nonexistent/elephc-cc-key.pem']]);
$m = fopen("php://memory", "r+");
$r = stream_socket_enable_crypto($m, true);
echo $r === false ? "no" : "yes";
fclose($m);
"#,
    );
    assert_eq!(out, "no");
}

/// Verifies compiled PHP output for stream context create returns resource.
#[test]
fn test_stream_context_create_returns_resource() {
    // Context creation and the lazy default each return a registry resource
    // whose ContextState owns its independently persisted options and notifier.
    let out = compile_and_run(
        r#"<?php
$c = stream_context_create(["http" => ["method" => "POST"]]);
$d = stream_context_get_default();
echo is_resource($c) ? "ok" : "FAIL";
echo "|";
echo is_resource($d) ? "ok" : "FAIL";
echo "|";
echo stream_context_set_option($c, "http", "method", "GET") ? "set-ok" : "FAIL";
"#,
    );
    assert_eq!(out, "ok|ok|set-ok");
}

/// Verifies compiled PHP output for stream context get options returns array.
#[test]
fn test_stream_context_get_options_returns_array() {
    // get_options returns the addressed ContextState's live COW snapshot, while
    // get_params reconstructs the exact notification/options parameter map.
    let out = compile_and_run(
        r#"<?php
$c = stream_context_create(["http" => ["method" => "POST"]]);
echo gettype(stream_context_get_options($c));
echo "|" . count(stream_context_get_options($c));
echo "|";
echo gettype(stream_context_get_params($c));
"#,
    );
    assert_eq!(out, "array|1|array");
}

/// Verifies compiled PHP output for fopen accepts 4 arg form with context.
#[test]
fn test_fopen_accepts_4_arg_form_with_context() {
    // Phase 11 B2: fopen($file, $mode, $use_include_path, $context) compiles
    // and runs. The 3rd and 4th args are evaluated for their side effects
    // (so e.g. dynamic-context PHP code typechecks) but the open path still
    // uses the global _stream_context_options slot for any consumer logic.
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create(["http" => ["method" => "GET"]]);
$m = fopen("php://memory", "r+", false, $ctx);
echo is_resource($m) ? "ok" : "fail";
fclose($m);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies that fopen() exposes its optional PHP parameter names to call planning.
#[test]
fn test_fopen_accepts_named_optional_args() {
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create(["http" => ["method" => "GET"]]);
$m = fopen(filename: "php://memory", mode: "r+", use_include_path: false, context: $ctx);
echo is_resource($m) ? "ok" : "fail";
fclose($m);
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies that literal fopen wrappers evaluate ignored optional args before opening.
#[test]
fn test_fopen_literal_wrapper_evaluates_optional_args_in_source_order() {
    let out = compile_and_run(
        r#"<?php
function mode_arg(): string { echo "M"; return "r+"; }
function use_include_path_arg(): bool { echo "U"; return false; }
function context_arg($ctx) { echo "C"; return $ctx; }
$ctx = stream_context_create();
$m = fopen("php://memory", mode_arg(), use_include_path_arg(), context_arg($ctx));
echo is_resource($m) ? "R" : "F";
fclose($m);
"#,
    );
    assert_eq!(out, "MUCR");
}

/// Verifies that non-literal fopen paths evaluate optional args before the open side effect.
#[test]
fn test_fopen_dynamic_path_evaluates_optional_args_before_open() {
    let out = compile_and_run(
        r#"<?php
function create_before_open(string $path): bool {
    echo "O";
    file_put_contents($path, "x");
    return false;
}
$path = tempnam(sys_get_temp_dir(), "elephc_fopen_order_");
unlink($path);
$f = fopen($path, "r", create_before_open($path));
echo is_resource($f) ? "R" : "F";
if ($f !== false) { fclose($f); }
unlink($path);
"#,
    );
    assert_eq!(out, "OR");
}

/// Verifies compiled PHP output for stream context set option four arg per option updates.
#[test]
fn test_stream_context_set_option_four_arg_per_option_updates() {
    // Phase 11 B2: the 4-arg form
    // stream_context_set_option(ctx, wrapper, opt, val) mutates the
    // persisted options[wrapper][opt] = val structure. Multiple calls
    // for the same wrapper accumulate options on the same sub-hash;
    // distinct wrappers grow the top-level hash.
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create();
stream_context_set_option($ctx, "http", "method", "POST");
stream_context_set_option($ctx, "http", "header", "X-Trace: 1");
stream_context_set_option($ctx, "ssl", "peer_name", "example.com");
$opts = stream_context_get_options($ctx);
$out = "wrappers:" . count($opts);
foreach ($opts as $w => $sub) {
    $out .= "|" . $w . ":" . count($sub);
}
echo $out;
"#,
    );
    assert_eq!(out, "wrappers:2|http:2|ssl:1");
}

/// Verifies compiled PHP output for TLS cipher/security-level options accepted as no-ops.
#[test]
fn test_stream_context_ssl_cipher_options_are_accepted_noops() {
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create();
$a = stream_context_set_option($ctx, "ssl", "ciphers", "DEFAULT@SECLEVEL=1");
$b = stream_context_set_option($ctx, "ssl", "security_level", "1");
$count = 0;
foreach (stream_context_get_options($ctx) as $wrapper => $sub) {
    if ($wrapper === "ssl") {
        $count = count($sub);
    }
}
echo ($a && $b ? "ok" : "FAIL") . "|" . $count;
"#,
    );
    assert_eq!(out, "ok|2");
}

/// Verifies the two-argument stream context option form merges wrapper maps.
#[test]
fn test_stream_context_set_option_two_arg_merges_options() {
    // The two-argument form merges incoming wrappers and each wrapper's option
    // map into the addressed ContextState, preserving entries absent from the patch.
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create(["http" => ["method" => "POST"]]);
echo count(stream_context_get_options($ctx)) . "|";
stream_context_set_option($ctx, ["ssl" => ["verify_peer" => false], "http" => ["method" => "GET"]]);
echo count(stream_context_get_options($ctx));
"#,
    );
    assert_eq!(out, "1|2");
}

/// Verifies compiled PHP output for stream context get options empty when no create.
#[test]
fn test_stream_context_get_options_empty_when_no_create() {
    // Before any stream_context_create, the persisted-options slot is
    // null; stream_context_get_options falls back to an empty hash.
    let out = compile_and_run(
        r#"<?php
$d = stream_context_get_default();
echo count(stream_context_get_options($d));
"#,
    );
    assert_eq!(out, "0");
}

/// Verifies compiled PHP output for stream set buffer stubs.
#[test]
fn test_stream_set_buffer_stubs() {
    // stream_set_chunk_size returns the previous chunk size (8192 default on the
    // first call); the read/write buffer setters return 0 ("success" — elephc
    // streams are unbuffered, so the size has no effect).
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
echo stream_set_chunk_size($m, 4096);
echo "|";
echo stream_set_read_buffer($m, 0);
echo "|";
echo stream_set_write_buffer($m, 0);
fclose($m);
"#,
    );
    assert_eq!(out, "8192|0|0");
}

/// `stream_set_chunk_size` returns the PREVIOUS per-fd chunk size (PHP's
/// observable contract): the first call reports the 8192 default, and each
/// subsequent call reports the value set by the previous call.
#[test]
fn test_stream_set_chunk_size_returns_previous() {
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
echo stream_set_chunk_size($m, 4096);
echo "|";
echo stream_set_chunk_size($m, 2048);
echo "|";
echo stream_set_chunk_size($m, 1024);
fclose($m);
"#,
    );
    assert_eq!(out, "8192|4096|2048");
}

/// Pins PHP's own out-parameter idiom: `&$errno` / `&$errstr` passed undeclared.
///
/// PHP auto-vivifies a variable bound to a by-reference parameter, which is why every manual
/// example writes the call this way and never declares the two error variables. The parameters
/// are declared `ref(Int)` / `ref(Str)` in the registry, so the checker treats those argument
/// positions as definition sites and gives each variable the type the builtin writes.
#[test]
fn test_socket_out_parameters_may_be_undeclared() {
    let out = compile_and_run(
        r#"<?php
$s = @stream_socket_client("tcp://127.0.0.1:1", $errno, $errstr, 1);
echo var_export($s === false, true), "|", gettype($errno), "|", gettype($errstr);
"#,
    );
    assert_eq!(out, "true|integer|string");
}

/// Pins that a NAMED out-parameter binds the parameter it names, not the one sharing its index.
///
/// `error_message:` is the third parameter but the second argument here, so resolving by position
/// would type `$why` as `int` and the runtime would then write a string pointer into an integer
/// slot. It also pins that omitting `$error_code` is allowed: normalization materialises the
/// parameter's `null` default at that position, which a by-reference argument check must accept.
#[test]
fn test_named_out_parameter_binds_the_parameter_it_names() {
    let out = compile_and_run(
        r#"<?php
$c = @stream_socket_client("unix:///nonexistent/elephc-probe.sock", error_message: $why);
echo gettype($why), "=", $why;
"#,
    );
    assert_eq!(out, "string=No such file or directory");
}

/// Pins that a by-ref output still refuses an argument with nowhere to write back into.
#[test]
fn test_out_parameter_rejects_an_argument_without_storage() {
    let error = compile_expect_type_error(
        r#"<?php
$c = @stream_socket_client("tcp://127.0.0.1:1", 0, $errstr, 1);
"#,
    );
    assert!(
        error.contains("parameter $error_code must be passed a variable"),
        "expected the by-reference storage diagnostic, got: {error}"
    );
}

/// Pins that the undeclared out-parameter also works in statement position, where the call's
/// result is discarded — `flock()` is the non-socket member of the same family.
#[test]
fn test_flock_would_block_out_parameter_may_be_undeclared() {
    let out = compile_and_run(
        r#"<?php
$h = fopen("php://memory", "r+");
flock($h, LOCK_SH, $would);
echo gettype($would), "=", var_export($would, true);
fclose($h);
"#,
    );
    assert_eq!(out, "integer=0");
}

/// Pins that a by-ref out-parameter whose variable already holds an incompatible type reports
/// elephc's ordinary reassignment error.
///
/// The write used to go straight into the caller's slot without consulting its representation:
/// an `int` landing in a `string` slot overwrote the pointer half with a small integer, and the
/// program segfaulted on the next read. Binding the out-parameter through the normal assignment
/// merge is what turns that silent corruption into a diagnostic.
#[test]
fn test_by_ref_out_parameter_rejects_an_incompatible_variable() {
    let error = compile_expect_type_error(
        r#"<?php
$would = "untouched";
$h = fopen("php://memory", "r+");
flock($h, LOCK_SH, $would);
"#,
    );
    assert!(
        error.contains("cannot reassign $would from string to int"),
        "expected a reassignment diagnostic, got: {error}"
    );
}

/// Verifies `fopen()` honours a `php://` scheme in a path built at RUN TIME, not only in a
/// literal.
///
/// The wrapper dispatch is a compile-time chain over the constant-folded filename, and the
/// dynamic path used to recognise `http://` alone — so every other scheme opened as a plain file
/// name, failed to find it, and answered `false`. That is the shape real code takes: a function
/// receives its path as a parameter, so the literal-only dispatch was invisible until a caller
/// passed one in. `__rt_php_wrapper_open` now makes the same choices from the run-time bytes.
///
/// Measured against php 8.5.6, which opens all of these.
#[test]
fn test_fopen_honours_a_php_scheme_built_at_run_time() {
    let out = compile_and_run(
        r#"<?php
function probe(string $label, string $path, string $mode): void {
    $h = @fopen($path, $mode);
    echo $label, "=", var_export($h !== false, true), " ";
    if ($h !== false) { fclose($h); }
}
$p = "php://";
probe("memory", $p . "memory", "r+");
probe("temp", $p . "temp", "r+");
probe("stdout", $p . "stdout", "w");
probe("stderr", $p . "stderr", "w");
probe("input", $p . "input", "r");
probe("output", $p . "output", "w");
probe("fd1", $p . "fd/1", "w");
probe("maxmemory", $p . "temp/maxmemory:16", "r+");
echo "|";
$m = fopen($p . "memory", "r+");
fwrite($m, "round trip");
rewind($m);
echo stream_get_contents($m);
fclose($m);
"#,
    );
    assert_eq!(
        out,
        "memory=true temp=true stdout=true stderr=true input=true output=true fd1=true \
         maxmemory=true |round trip"
    );
}

/// Pins that a run-time `php://` URL naming no stream answers `false` rather than opening
/// something.
///
/// The dispatcher walks a table and reports `-1` for anything it does not recognise, which boxes
/// as PHP's `false`. Without this the unknown case would be indistinguishable from the schemes
/// that work.
#[test]
fn test_fopen_rejects_an_unknown_php_scheme_built_at_run_time() {
    let out = compile_and_run(
        r#"<?php
$p = "php://";
echo var_export(@fopen($p . "nosuchstream", "r"), true), "|";
echo var_export(@fopen($p . "fd/notanumber", "r"), true), "|";
echo var_export(@fopen($p, "r"), true);
"#,
    );
    assert_eq!(out, "false|false|false");
}

/// Verifies a run-time `php://` handle behaves like a literal one in the ways most likely to
/// break: descriptor ownership, filters, and independence.
///
/// A descriptor-backed scheme must hand out a `dup()` — closing a `php://stdout` handle that WAS
/// descriptor 1 would take the program's own output with it. A run-time handle must also accept a
/// filter and honour the filtered-read buffer, and two handles to `php://temp` must not share a
/// buffer.
#[test]
fn test_a_run_time_php_handle_behaves_like_a_literal_one() {
    let out = compile_and_run(
        r#"<?php
$p = "php://";
$o = fopen($p . "stdout", "w");
fwrite($o, "via-handle ");
fclose($o);
echo "still-alive|";

$f = fopen($p . "memory", "r+");
fwrite($f, "abcdef");
rewind($f);
stream_filter_append($f, "string.toupper", STREAM_FILTER_READ);
$parts = [];
while (!feof($f)) {
    $c = fread($f, 2);
    if ($c === "") { break; }
    $parts[] = $c;
}
echo implode(",", $parts), "|";
fclose($f);

$a = fopen($p . "temp", "r+");
$b = fopen($p . "temp", "r+");
fwrite($a, "AAA");
fwrite($b, "BBB");
rewind($a);
rewind($b);
echo fread($a, 3), fread($b, 3);
fclose($a);
fclose($b);
"#,
    );
    assert_eq!(out, "via-handle still-alive|AB,CD,EF|AAABBB");
}

/// Verifies a `php://filter` URL built at RUN TIME opens and filters.
///
/// A filter URL is "open this, then filter it", so the parse hands the open path the RESOURCE and
/// the named filter is attached once the stream exists. That keeps the resource on whatever open
/// path it deserves — this covers both a plain file and a nested `php://temp`, and checks that a
/// plain open afterwards does not inherit the filter.
#[test]
fn test_php_filter_url_built_at_run_time_opens() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("pf.txt", "hello");
$url = "php://filter/read=string.toupper/resource=" . "pf.txt";
$f = fopen($url, "r");
echo "file=", stream_get_contents($f), "|";
fclose($f);

$nested = "php://filter/read=string.toupper/resource=php://" . "temp";
$g = fopen($nested, "r+");
fwrite($g, "abc");
rewind($g);
echo "nested=", stream_get_contents($g), "|";
fclose($g);

$h = fopen("pf" . ".txt", "r");
echo "plain=", stream_get_contents($h);
fclose($h);
"#,
    );
    assert_eq!(out, "file=HELLO|nested=ABC|plain=hello");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a run-time filter URL that names nothing usable opens the resource unfiltered or
/// fails, rather than opening something else.
///
/// An unknown filter name is what php-src also tolerates by opening the resource plain; a URL with
/// no `/resource=` names nothing at all.
#[test]
fn test_run_time_filter_url_edge_cases() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("pf.txt", "hello");
$unknown = "php://filter/read=no.such.filter/resource=" . "pf.txt";
$a = @fopen($unknown, "r");
echo "unknown=", var_export($a !== false, true);
if ($a !== false) { echo ":", stream_get_contents($a); fclose($a); }
$nores = "php://filter/read=string." . "toupper";
echo " noresource=", var_export(@fopen($nores, "r"), true);
$nested = "php://filter/read=string.toupper/resource=php://filter/read=string." . "tolower";
echo " nested=", var_export(@fopen($nested, "r"), true);
"#,
    );
    assert_eq!(out, "unknown=true:hello noresource=false nested=false");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a `data://` URI built at RUN TIME decodes and opens.
///
/// A literal URI is decoded during lowering and its bytes embedded, which left a run-time one
/// with no path at all. Decoding needed nothing new in the runtime: `__rt_base64_decode` and
/// `__rt_urldecode` already exist, and the latter's `+`-as-space rule is what the compile-time
/// decoder applies to these URIs too.
#[test]
fn test_fopen_honours_a_data_url_built_at_run_time() {
    let out = compile_and_run(
        r#"<?php
function probe(string $label, string $uri): void {
    $h = @fopen($uri, "r");
    echo $label, "=", var_export($h !== false, true);
    if ($h !== false) { echo ":", stream_get_contents($h); fclose($h); }
    echo " ";
}
$d = "data://";
probe("plain", $d . "text/plain,hi");
probe("pct", $d . "text/plain,a%20b%21");
probe("b64", $d . "text/plain;base64,aGVsbG8=");
probe("empty", $d . "text/plain,");
probe("nocomma", $d . "text/plain");
"#,
    );
    assert_eq!(out, "plain=true:hi pct=true:a b! b64=true:hello empty=true: nocomma=false ");
}

/// Verifies PHP's optional `fgets($handle, $length)`, which bounds the line.
///
/// php 8.5.6 reads at most `$length - 1` bytes, leaves the remainder for the next read, answers
/// `false` when the bound leaves room for nothing, and rejects a non-positive bound with a
/// `ValueError`. The builtin used to take a single parameter, so `fgets($conn, 1024)` — the
/// ordinary way to read a request line — did not compile at all.
#[test]
fn test_fgets_accepts_phps_length_bound() {
    let out = compile_and_run(
        r#"<?php
$h = fopen("php://memory", "r+");
fwrite($h, "abcdefghij\nsecond\n");
rewind($h);
echo var_export(fgets($h, 5), true), "|", var_export(fgets($h), true), "|";
rewind($h);
echo var_export(fgets($h, 2), true), "|", var_export(fgets($h, 1), true), "|";
rewind($h);
echo var_export(fgets($h, 100), true);
fclose($h);
"#,
    );
    assert_eq!(out, "'abcd'|'efghij\n'|'a'|false|'abcdefghij\n'");
}

/// Verifies a non-positive `$length` raises php-src's `ValueError` rather than reading unbounded.
///
/// Zero is what an omitted argument means to the runtime helper, so a caller-supplied zero has to
/// be rejected before it reaches it — otherwise `fgets($h, 0)` would quietly read a whole line.
#[test]
fn test_fgets_rejects_a_non_positive_length() {
    let out = compile_and_run(
        r#"<?php
$h = fopen("php://memory", "r+");
fwrite($h, "abcdefghij\n");
rewind($h);
foreach ([0, -1] as $len) {
    try {
        fgets($h, $len);
        echo "no-throw|";
    } catch (ValueError $e) {
        echo $e->getMessage(), "|";
    }
}
fclose($h);
"#,
    );
    assert_eq!(
        out,
        "fgets(): Argument #2 ($length) must be greater than 0|\
         fgets(): Argument #2 ($length) must be greater than 0|"
    );
}

/// Verifies `data://` refuses a media type php-src does not accept, and reads `;base64` the way
/// php-src reads it.
///
/// elephc used to accept ANY media type and look for a `;base64` suffix case-insensitively, so it
/// opened URIs php-src refuses and base64-decoded a `;BASE64` php-src would not. Measuring the
/// real rule was the point of this fixture, and it is narrower than "charset is special":
///
/// - the type is empty, or it must carry a `/` — `text` alone is refused;
/// - every parameter must be `name=value`, whatever the name — `;bogus=1` is ACCEPTED, `;bogus`
///   and a trailing empty `;` are not;
/// - `base64` counts only as the LAST parameter and only in lower case, so
///   `;charset=utf-8;base64` decodes while `;base64;charset=utf-8` is refused outright.
///
/// The rule lives twice — in `data_uri_media_type_is_valid` for a literal URI resolved at compile
/// time, and in `__rt_data_uri_meta_ok` for one built at run time. Neither can serve both, so both
/// forms are exercised here and a divergence fails this test.
#[test]
fn test_data_url_rejects_a_media_type_php_refuses() {
    let out = compile_and_run(
        r#"<?php
function probe(string $label, string $uri): void {
    $h = @fopen($uri, "r");
    echo $label, "=", var_export($h !== false, true);
    if ($h !== false) { echo ":", stream_get_contents($h); fclose($h); }
    echo " ";
}
// Run-time URIs go through the runtime validator.
$d = "data://";
probe("noslash", $d . "text,aGVsbG8=");
probe("emptyparam", $d . "text/plain;,aGVsbG8=");
probe("b64notlast", $d . "text/plain;base64;charset=utf-8,aGVsbG8=");
probe("upper", $d . "text/plain;BASE64,aGVsbG8=");
probe("namedparam", $d . "text/plain;bogus=1,aGVsbG8=");
probe("b64last", $d . "text/plain;charset=utf-8;base64,aGVsbG8=");
echo "|";
// The same shapes as literals, which the compile-time decoder resolves instead.
probe("lit-noslash", "data://text,aGVsbG8=");
probe("lit-b64notlast", "data://text/plain;base64;charset=utf-8,aGVsbG8=");
probe("lit-namedparam", "data://text/plain;bogus=1,aGVsbG8=");
probe("lit-b64last", "data://text/plain;charset=utf-8;base64,aGVsbG8=");
"#,
    );
    assert_eq!(
        out,
        "noslash=false emptyparam=false b64notlast=false upper=false \
         namedparam=true:aGVsbG8= b64last=true:hello \
         |lit-noslash=false lit-b64notlast=false \
         lit-namedparam=true:aGVsbG8= lit-b64last=true:hello "
    );
}

/// Pins that `fread($f, $n)` never hands back more than `$n` bytes through a filter.
///
/// IGNORED because elephc has no filtered-read buffer: a read filter that expands its input
/// has its whole output returned in one go, so `fread($f, 2)` over a filter tripling `"ab"`
/// answers the 6-byte `"ababab"` where php 8.5.6 answers `ab`, `ab`, `ab` — it caps the
/// result at `$n` and keeps the remainder on the stream for the next read.
///
/// This is INDEPENDENT of [`test_user_filter_psfs_feed_me_buffers_across_dispatches`]: the
/// filter here answers `PSFS_PASS_ON` on every dispatch, so no FEED_ME handling is involved.
/// It is also that fixture's prerequisite — without somewhere to park the remainder, a
/// FEED_ME fix cannot hand back the right chunk sizes either.
///
/// Returning more bytes than requested is a contract break in its own right: a caller that
/// sized a buffer from `$n` gets more than it asked for.
#[test]
fn test_fread_caps_a_filtered_read_at_the_requested_length() {
    let out = compile_and_run(
        r#"<?php
class ExpandThrice extends php_user_filter {
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $consumed += $b->datalen;
            $ob = stream_bucket_new($this->stream, str_repeat($b->data, 3));
            stream_bucket_append($out, $ob);
        }
        return PSFS_PASS_ON;
    }
}
stream_filter_register("expand.thrice", "ExpandThrice");
$f = fopen("php://memory", "r+");
fwrite($f, "ab");
rewind($f);
stream_filter_append($f, "expand.thrice", STREAM_FILTER_READ);
$parts = [];
while (!feof($f)) {
    $c = fread($f, 2);
    if ($c === "" || $c === false) { break; }
    $parts[] = $c;
}
echo implode("|", $parts);
"#,
    );
    assert_eq!(out, "ab|ab|ab");
}

/// Pins PHP's `PSFS_FEED_ME` contract for a filter that buffers across dispatches.
///
/// IGNORED because elephc does not implement it yet, and the current behaviour is a
/// SILENT one: `PSFS_FEED_ME` passes the RAW input through, so this filter leaks
/// unfiltered bytes to the caller — `<abc><ABCDEF><ghi>` where php 8.5.6 answers
/// `<ABC><DEF><GHI>`. A filter that returns PSFS_PASS_ON on every dispatch is
/// unaffected, which is why the rest of the filter suite stays green.
///
/// Fixing it takes THREE changes that must land together:
///   1. `PSFS_FEED_ME` must return nothing rather than the original input;
///   2. `__rt_fread` must then fetch more input and dispatch again instead of
///      reporting a short read — with (1) alone, `fread()` returns "" and every
///      caller written as `if ($chunk === "") break;` stops early, turning a data
///      LEAK into data LOSS;
///   3. the StreamState needs a filtered-read buffer plus a closing flush at EOF.
///      Measured against php 8.5.6: a filter that triples `"ab"` answers three
///      `fread($f, 2)` calls with `ab|ab|ab`, so PHP caps the filtered result at
///      `$length` and keeps the remainder; and a filter still holding bytes when the
///      stream ends gets a `$closing` dispatch whose output reaches the reader. With
///      only (1)+(2) this fixture prints `<ABCDEF>` — the leak becomes a loss.
#[test]
fn test_user_filter_psfs_feed_me_buffers_across_dispatches() {
    let out = compile_and_run(
        r#"<?php
class FeedMeCollect extends php_user_filter {
    private string $buf = "";
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $consumed += $b->datalen;
            $this->buf .= $b->data;
        }
        if (strlen($this->buf) < 6) {
            return PSFS_FEED_ME;
        }
        $ob = stream_bucket_new($this->stream, strtoupper($this->buf));
        stream_bucket_append($out, $ob);
        $this->buf = "";
        return PSFS_PASS_ON;
    }
}
stream_filter_register("feedme.collect", "FeedMeCollect");
$f = fopen("php://memory", "r+");
fwrite($f, "abcdefghi");
rewind($f);
stream_filter_append($f, "feedme.collect", STREAM_FILTER_READ);
$out = "";
while (!feof($f)) {
    $chunk = fread($f, 3);
    if ($chunk === "" || $chunk === false) { break; }
    $out .= "<" . $chunk . ">";
}
echo $out;
"#,
    );
    assert_eq!(out, "<ABC><DEF><GHI>");
}

/// Pins the third measured property of PHP's filtered reads: end of input triggers a `$closing`
/// dispatch whose output reaches the reader.
///
/// IGNORED because nothing flushes a read filter at EOF. A filter holding every byte until
/// `$closing` therefore never emits its result — and because `PSFS_FEED_ME` currently passes its
/// input through, the reader gets the RAW `xyz` instead of the filter's `[xyz]`. Measured against
/// php 8.5.6.
///
/// Kept separate from [`test_user_filter_psfs_feed_me_buffers_across_dispatches`] so the three
/// properties can be fixed and verified one at a time: FEED_ME returning nothing, `fread()`
/// capping and parking the remainder, and this closing flush. Landing the first two without this
/// one turns the leak into silent data loss, so all three ship together.
#[test]
fn test_read_filter_is_flushed_when_the_stream_ends() {
    let out = compile_and_run(
        r#"<?php
class HoldUntilClose extends php_user_filter {
    private string $buf = "";
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $consumed += $b->datalen;
            $this->buf .= $b->data;
        }
        if (!$closing) {
            return PSFS_FEED_ME;
        }
        stream_bucket_append($out, stream_bucket_new($this->stream, "[" . $this->buf . "]"));
        return PSFS_PASS_ON;
    }
}
stream_filter_register("hold.until.close", "HoldUntilClose");
$f = fopen("php://memory", "r+");
fwrite($f, "xyz");
rewind($f);
stream_filter_append($f, "hold.until.close", STREAM_FILTER_READ);
echo stream_get_contents($f);
fclose($f);
"#,
    );
    assert_eq!(out, "[xyz]");
}

/// Verifies `php_user_filter` declares the properties PHP declares.
///
/// Only `$params` existed, so the manual's own filter idiom — building an output bucket
/// with `stream_bucket_new($this->stream, ...)` — did not compile at all.
#[test]
fn test_user_filter_base_class_declares_filtername_and_stream() {
    let out = compile_and_run(
        r#"<?php
class PropProbeFilter extends php_user_filter {
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $consumed += $b->datalen;
            $ob = stream_bucket_new($this->stream, strtoupper($b->data));
            stream_bucket_append($out, $ob);
        }
        return PSFS_PASS_ON;
    }
}
stream_filter_register("prop.probe", "PropProbeFilter");
$f = fopen("php://memory", "r+");
fwrite($f, "hello");
rewind($f);
stream_filter_append($f, "prop.probe", STREAM_FILTER_READ);
echo stream_get_contents($f);
echo "|", var_export(property_exists("PropProbeFilter", "filtername"), true);
"#,
    );
    assert_eq!(out, "HELLO|true");
}

/// Verifies compiled PHP output for user stream filter write transforms payload.
#[test]
fn test_user_stream_filter_write_transforms_payload() {
    // Phase 10 tier 3: a user-registered filter class attached in write
    // direction transforms fwrite payloads. The filter's filter() method
    // receives the raw bytes and returns the bytes that actually hit the
    // underlying stream — so reading them back yields the transformed
    // payload.
    let out = compile_and_run(
        r#"<?php
class UpperFilter {
    public function filter(string $data): string {
        return strtoupper($data);
    }
}
stream_filter_register("user.upper", "UpperFilter");
$f = fopen("php://memory", "r+");
stream_filter_append($f, "user.upper", STREAM_FILTER_WRITE);
fwrite($f, "hello world");
rewind($f);
echo fread($f, 64);
"#,
    );
    assert_eq!(out, "HELLO WORLD");
}

/// Verifies compiled PHP output for user stream filter registered class is case insensitive.
#[test]
fn test_user_stream_filter_registered_class_is_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
class CaseFilter {
    public function filter(string $data): string {
        return strtoupper($data);
    }
}
stream_filter_register("case.upper", "casefilter");
$f = fopen("php://memory", "r+");
stream_filter_append($f, "case.upper", STREAM_FILTER_WRITE);
fwrite($f, "hello");
rewind($f);
echo fread($f, 64);
"#,
    );
    assert_eq!(out, "HELLO");
}

/// Verifies compiled PHP output for user stream filter read transforms payload.
#[test]
fn test_user_stream_filter_read_transforms_payload() {
    // Phase 10 tier 3: a user-registered filter class attached in read
    // direction transforms bytes returned by fread. The raw on-stream
    // bytes are unchanged; only the read path sees the filtered result.
    let out = compile_and_run(
        r#"<?php
class LowerFilter {
    public function filter(string $data): string {
        return strtolower($data);
    }
}
stream_filter_register("user.lower", "LowerFilter");
$f = fopen("php://memory", "r+");
fwrite($f, "HELLO WORLD");
rewind($f);
stream_filter_append($f, "user.lower", STREAM_FILTER_READ);
echo fread($f, 64);
"#,
    );
    assert_eq!(out, "hello world");
}

/// Verifies compiled PHP output for user stream filter params exposed on `$this`.
#[test]
fn test_user_stream_filter_params_are_exposed_on_this() {
    let out = compile_and_run(
        r#"<?php
class ParamFilter extends php_user_filter {
    public function onCreate(): bool {
        echo $this->params["prefix"];
        return true;
    }

    public function filter(string $data): string {
        return $data . $this->params["suffix"];
    }
}
stream_filter_register("user.params", "ParamFilter");
$f = fopen("php://memory", "r+");
stream_filter_append($f, "user.params", STREAM_FILTER_WRITE, ["prefix" => "<", "suffix" => ">"]);
fwrite($f, "hello");
rewind($f);
echo "|" . fread($f, 64);
"#,
    );
    assert_eq!(out, "<|hello>");
}

/// Verifies compiled PHP output for user stream filter unknown name returns false.
#[test]
fn test_user_stream_filter_unknown_name_returns_false() {
    // stream_filter_append on an unknown user-filter name resolves the
    // ID to 0 through the registry scan; the helper short-circuits and
    // the builtin emitter boxes PHP false. No state mutation happens.
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://memory", "r+");
$r = stream_filter_append($f, "this.does.not.exist");
echo $r === false ? "false" : "open";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for stream filter user onclose fires on remove.
#[test]
fn test_stream_filter_user_onclose_fires_on_remove() {
    // Phase 11 B4 (partial): stream_filter_remove() now shares the same
    // onClose-then-clear teardown as fclose(). Removing a filter that
    // declared onClose fires the hook before subsequent fwrites bypass
    // the (now-detached) filter.
    let out = compile_and_run(
        r#"<?php
class TraceFilter {
    public function filter(string $data): string {
        return strtoupper($data);
    }
    public function onClose(): void {
        echo "|closed";
    }
}
stream_filter_register("trace.upper", "TraceFilter");
$m = fopen("php://memory", "r+");
$f = stream_filter_append($m, "trace.upper", STREAM_FILTER_WRITE);
fwrite($m, "a");
stream_filter_remove($f);
fwrite($m, "b");
rewind($m);
echo stream_get_contents($m);
fclose($m);
"#,
    );
    // Filtered "a" → "A", then onClose fires before the second write
    // bypasses the filter, so the final memory holds "Ab" and the
    // closed-marker lands between them in the output.
    assert_eq!(out, "|closedAb");
}

/// Verifies compiled PHP output for stream bucket new returns object with data and datalen.
#[test]
fn test_stream_bucket_new_returns_object_with_data_and_datalen() {
    // Phase 11 B4 (API-surface delivery): stream_bucket_new($stream, $data)
    // returns a real PHP object (stdClass-backed) with public `data` and
    // `datalen` properties, matching PHP's documented bucket shape. The
    // bucket is decoupled from the filter dispatch — it's a stand-alone
    // primitive that filter() implementations using the PHP-standard
    // 4-arg signature can call (the dispatch refactor itself is the
    // separate increment).
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
$b = stream_bucket_new($m, "hello world");
echo gettype($b) . "|" . $b->data . "|" . $b->datalen;
fclose($m);
"#,
    );
    assert_eq!(out, "object|hello world|11");
}

/// Verifies compiled PHP output for stream bucket make writeable returns null for empty brigade.
#[test]
fn test_stream_bucket_make_writeable_returns_null_for_empty_brigade() {
    // Phase 11 B4: stream_bucket_make_writeable on an empty brigade
    // returns null per PHP's documented behaviour. v1 always returns
    // null since the filter dispatch hasn't been wired to seed brigade
    // state yet.
    let out = compile_and_run(
        r#"<?php
$brigade = new stdClass();
$b = stream_bucket_make_writeable($brigade);
echo is_null($b) ? "null" : "non-null";
"#,
    );
    assert_eq!(out, "null");
}

/// Verifies compiled PHP output for stream filter user oncreate refusal blocks attach.
#[test]
fn test_stream_filter_user_oncreate_refusal_blocks_attach() {
    // Phase 11 B4 (partial): if a user-filter class's onCreate() returns
    // false, the filter is refused and stream_filter_append returns false.
    // No filter is recorded against the fd, so subsequent fwrites pass
    // through unchanged.
    let out = compile_and_run(
        r#"<?php
class RefuseFilter {
    public function onCreate(): bool {
        return false;
    }
    public function filter(string $data): string {
        return "should not run";
    }
}
stream_filter_register("trace.refuse", "RefuseFilter");
$m = fopen("php://memory", "r+");
$r = stream_filter_append($m, "trace.refuse", STREAM_FILTER_WRITE);
echo "attach=" . ($r === false ? "false" : "ok") . "|";
fwrite($m, "hi");
rewind($m);
echo stream_get_contents($m);
fclose($m);
"#,
    );
    assert_eq!(out, "attach=false|hi");
}

/// Verifies compiled PHP output for stream filter user oncreate and onclose fire.
#[test]
fn test_stream_filter_user_oncreate_and_onclose_fire() {
    // Phase 11 B4 (partial): onCreate() runs at attach time (so its
    // side effect of pre-loading state is visible to the first filter()
    // call), and onClose() runs at fclose() time (so cleanup like a
    // final flush can happen). When the method is absent in the class,
    // the attach / close still works — only the implemented hooks
    // fire.
    let out = compile_and_run(
        r#"<?php
class CountingFilter {
    public string $prefix = "";
    public function onCreate(): bool {
        $this->prefix = ">>";
        return true;
    }
    public function filter(string $data): string {
        return $this->prefix . $data;
    }
    public function onClose(): void {
        echo "|closed";
    }
}
stream_filter_register("count.upper", "CountingFilter");
$m = fopen("php://memory", "r+");
stream_filter_append($m, "count.upper", STREAM_FILTER_WRITE);
fwrite($m, "x");
rewind($m);
echo stream_get_contents($m);
fclose($m);
"#,
    );
    assert_eq!(out, ">>x|closed");
}

/// Verifies compiled PHP output for stream filter register accepts registration.
#[test]
fn test_stream_filter_register_accepts_registration() {
    // v1 stub: stream_filter_register() accepts the registration and reports
    // true. The user-defined filter class is not yet invoked on read/write.
    let out = compile_and_run(
        r#"<?php
class CustomFilter {}
echo stream_filter_register("custom.filter", "CustomFilter") ? "true" : "false";
"#,
    );
    assert_eq!(out, "true");
}

/// Verifies compiled PHP output for fopen silent fail for registered user wrapper.
#[test]
fn test_fopen_silent_fail_for_registered_user_wrapper() {
    // Phase 10 dispatch v1: __rt_fopen recognises paths whose scheme matches
    // a registered user wrapper. When the wrapper class does not implement
    // `stream_open`, the runtime fails silently (no "Failed to open stream"
    // warning) instead of attempting to open the literal path.
    let out = compile_and_run_capture(
        r#"<?php
class CustomWrapper {}
stream_wrapper_register("custom", "CustomWrapper");
$f = fopen("custom://anywhere", "r");
echo $f === false ? "false" : "open";
"#,
    );
    assert_eq!(out.stdout, "false");
    assert!(
        !out.stderr.contains("Failed to open"),
        "registered user wrapper should not produce the failed-to-open warning, got stderr: {:?}",
        out.stderr,
    );
}

/// Verifies compiled PHP output for fopen user wrapper stream open true returns resource.
#[test]
fn test_fopen_user_wrapper_stream_open_true_returns_resource() {
    // Phase 10 step 3: when the wrapper class implements `stream_open` and
    // returns true, fopen() returns a resource backed by a synthetic
    // descriptor stored in `_user_wrapper_handles`. The wrapper object
    // itself is retained for later fread/fwrite/fclose dispatch.
    let out = compile_and_run(
        r#"<?php
class MyW {
    public function stream_open($path, $mode, $options, &$opened): bool {
        return true;
    }
}
stream_wrapper_register("my", "MyW");
$f = fopen("my://anywhere", "r");
echo is_resource($f) ? "ok" : "fail";
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen user wrapper registered class is case insensitive.
#[test]
fn test_fopen_user_wrapper_registered_class_is_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
class CaseWrapper {
    public function stream_open($path, $mode, $options, &$opened): bool {
        return true;
    }
}
stream_wrapper_register("casew", "casewrapper");
$f = fopen("casew://anywhere", "r");
echo is_resource($f) ? "ok" : "fail";
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen user wrapper round trip read write close.
#[test]
fn test_fopen_user_wrapper_round_trip_read_write_close() {
    // Phase 10 step 4: fread/fwrite/fclose dispatch into the wrapper class's
    // stream_read/stream_write/stream_close on a synthetic fd. The method
    // contracts are: stream_read returns string, stream_write returns int,
    // stream_close returns void, stream_eof returns bool.
    let out = compile_and_run(
        r#"<?php
class MyW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_read(int $count): string { return "hello"; }
    public function stream_write(string $data): int { return strlen($data); }
    public function stream_close(): void {}
    public function stream_eof(): bool { return false; }
}
stream_wrapper_register("my", "MyW");
$f = fopen("my://x", "r");
echo fread($f, 100);
echo "|";
echo fwrite($f, "abc");
echo "|";
echo feof($f) ? "1" : "0";
echo "|";
echo fclose($f) ? "1" : "0";
"#,
    );
    assert_eq!(out, "hello|3|0|1");
}

/// Verifies the final owner of an abandoned wrapper stream closes it on unset.
#[test]
fn test_fopen_user_wrapper_closes_on_final_owner_unset() {
    let out = compile_and_run(
        r#"<?php
class ScopeCloseWrapper {
    public function stream_open($path, $mode, $options, &$openedPath): bool {
        return true;
    }

    public function stream_close(): void {
        echo "closed|";
    }
}

stream_wrapper_register("scopecl", "ScopeCloseWrapper");
$stream = fopen("scopecl://resource", "r");
echo is_resource($stream) ? "open|" : "failed|";
unset($stream);
echo "after";
"#,
    );
    assert_eq!(out, "open|closed|after");
}

/// Verifies compiled PHP output for fopen user wrapper fputcsv routes through stream write.
#[test]
fn test_fopen_user_wrapper_fputcsv_routes_through_stream_write() {
    // fputcsv() on a userspace-wrapper resource must route its field/separator/
    // quote/newline segments into the wrapper's stream_write (via __rt_fd_write's
    // synthetic-fd dispatch) instead of a raw write to a real fd. The wrapper
    // echoes each chunk, so stdout reconstructs the exact CSV bytes: a plain row,
    // then a row whose first field embeds a comma and is therefore CSV-quoted.
    let out = compile_and_run(
        r#"<?php
class CsvW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_write(string $data): int { echo $data; return strlen($data); }
    public function stream_close(): void {}
}
stream_wrapper_register("csv", "CsvW");
$f = fopen("csv://x", "w");
fputcsv($f, ["a", "b", "c"]);
fputcsv($f, ["x,y", "z"]);
fclose($f);
"#,
    );
    assert_eq!(out, "a,b,c\n\"x,y\",z\n");
}

/// A user wrapper's negative `stream_write()` result is the runtime failure
/// sentinel and must surface from PHP `fwrite()` as boolean false, never integer
/// `-1`; successful writes remain integer byte counts.
#[test]
fn test_fwrite_user_wrapper_negative_result_is_false() {
    let out = compile_and_run(
        r#"<?php
class FailWriteWrapper {
    public function stream_open($path, $mode, $options, &$opened): bool { return true; }
    public function stream_write(string $data): int { return -1; }
}
stream_wrapper_register("failwrite", "FailWriteWrapper");
$stream = fopen("failwrite://x", "r+");
$result = fwrite($stream, "x");
echo ($result === false) ? "false" : gettype($result) . ":" . $result;
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for fopen user wrapper fgetc and rewind dispatch.
#[test]
fn test_fopen_user_wrapper_fgetc_and_rewind_dispatch() {
    // fgetc() reads a single byte via the wrapper's stream_read; rewind()
    // dispatches stream_seek(0, SEEK_SET) so a subsequent read restarts from
    // the beginning. (rewind previously lseek'd the synthetic fd and no-op'd.)
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="ABCDE"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_seek($o,$w): bool { $this->pos=$o; return true; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$f=fopen("w://x","r");
echo fgetc($f) . fgetc($f);
rewind($f);
echo fgetc($f);
fclose($f);
"#,
    );
    assert_eq!(out, "ABA");
}

/// Verifies compiled PHP output for fopen user wrapper applies property defaults.
#[test]
fn test_fopen_user_wrapper_applies_property_defaults() {
    // A registered wrapper instantiated by __rt_new_by_name now receives its
    // declared property defaults (via the _class_propinit_<id> thunk), so a
    // stream_open that relies on a default without assigning it works.
    let out = compile_and_run(
        r#"<?php
class W {
    public string $prefix = "PFX:";
    public string $data;
    public int $pos;
    public function stream_open($p, $m, $o, &$op): bool { $this->data = $this->prefix . "body"; $this->pos = 0; return true; }
    public function stream_read($n): string { $c = substr($this->data, $this->pos, $n); $this->pos += strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos >= strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w", "W");
$h = fopen("w://x", "r");
echo fread($h, 100);
fclose($h);
"#,
    );
    assert_eq!(out, "PFX:body");
}

/// Verifies compiled PHP output for fopen user wrapper stream get contents drains.
#[test]
fn test_fopen_user_wrapper_stream_get_contents_drains() {
    // stream_get_contents() on a synthetic wrapper fd drains via a compiled,
    // feof-gated fread loop: it checks the wrapper's stream_eof before each
    // read, so it never makes the EOF read whose empty substr result freed the
    // caller's resource cell. The result is assigned and the stream closed —
    // the exact pattern that previously SIGSEGV'd / corrupted $f.
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="hello, world!"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$f=fopen("w://x","r");
$x = stream_get_contents($f);
echo "[$x]";
fclose($f);
echo "|t=" . gettype($f);
"#,
    );
    assert_eq!(out, "[hello, world!]|t=resource");
}

/// Verifies compiled PHP output for fopen user wrapper fpassthru writes and counts.
#[test]
fn test_fopen_user_wrapper_fpassthru_writes_and_counts() {
    // fpassthru() on a wrapper fd uses the same feof-gated loop: it streams each
    // chunk to stdout, returns the byte count, and leaves the resource intact so
    // a following fclose() still sees a resource (not a freed/int cell).
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="Hello, world!"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$f=fopen("w://x","r");
$n=fpassthru($f);
echo "|n=$n";
fclose($f);
echo "|t=" . gettype($f);
"#,
    );
    assert_eq!(out, "Hello, world!|n=13|t=resource");
}

/// Verifies compiled PHP output for fopen user wrapper fgets reads lines.
#[test]
fn test_fopen_user_wrapper_fgets_reads_lines() {
    // fgets() on a wrapper fd reads one line at a time through a feof-gated
    // 1-byte loop, keeping the trailing newline and stopping at EOF. The
    // `!== false` loop must terminate cleanly and leave the resource intact.
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="line1\nline2\nlast"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$f=fopen("w://x","r");
while (($l = fgets($f)) !== false) { echo "[" . rtrim($l, "\n") . "]"; }
fclose($f);
echo "|t=" . gettype($f);
"#,
    );
    assert_eq!(out, "[line1][line2][last]|t=resource");
}

/// Verifies compiled PHP output for fopen user wrapper fscanf reads through stream read.
#[test]
fn test_fopen_user_wrapper_fscanf_reads_through_stream_read() {
    // fscanf() reads its line via __rt_fgets, which gained a wrapper-fd branch in
    // the userspace-wrapper coverage work, so fscanf() transparently parses a line
    // drained from the wrapper's stream_read. The conformant wrapper honors $count.
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="42 3.14 hi\n"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$f=fopen("w://x","r");
$r = fscanf($f, "%d %f %s");
echo $r[0] . "|" . $r[1] . "|" . $r[2];
fclose($f);
"#,
    );
    assert_eq!(out, "42|3.14|hi");
}

/// Verifies compiled PHP output for fopen user wrapper stream copy to stream drains.
#[test]
fn test_fopen_user_wrapper_stream_copy_to_stream_drains() {
    // stream_copy_to_stream() with a wrapper source uses the feof-gated loop:
    // each chunk is read via __rt_fread and written to the destination via
    // __rt_fwrite (here a real php://temp fd). The source resource must survive.
    let out = compile_and_run(
        r#"<?php
class W {
    public $data; public $pos;
    public function stream_open($p,$m,$o,&$op): bool { $this->data="copy-me-over!"; $this->pos=0; return true; }
    public function stream_read($n): string { $c=substr($this->data,$this->pos,$n); $this->pos+=strlen($c); return $c; }
    public function stream_eof(): bool { return $this->pos>=strlen($this->data); }
    public function stream_close(): void {}
}
stream_wrapper_register("w","W");
$src=fopen("w://x","r");
$dst=fopen("php://temp","r+");
$n=stream_copy_to_stream($src,$dst);
rewind($dst);
echo "n=$n|got=[" . stream_get_contents($dst) . "]";
fclose($src); fclose($dst);
echo "|st=" . gettype($src);
"#,
    );
    assert_eq!(out, "n=13|got=[copy-me-over!]|st=resource");
}

/// Verifies compiled PHP output for fopen user wrapper ftell dispatches to stream tell.
#[test]
fn test_fopen_user_wrapper_ftell_dispatches_to_stream_tell() {
    // Phase 10 follow-up: ftell() dispatches into the wrapper's stream_tell
    // and returns the int it reports. Without stream_tell, the helper falls
    // through to -1 (PHP's ftell failure sentinel).
    let out = compile_and_run(
        r#"<?php
class TellW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_tell(): int { return 42; }
}
class NoTellW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("tellw", "TellW");
stream_wrapper_register("notell", "NoTellW");
$f = fopen("tellw://x", "r");
echo ftell($f);
echo "|";
$g = fopen("notell://x", "r");
echo ftell($g);
"#,
    );
    assert_eq!(out, "42|-1");
}

/// Verifies compiled PHP output for fopen user wrapper fstat dispatches to stream stat.
#[test]
fn test_fopen_user_wrapper_fstat_dispatches_to_stream_stat() {
    // OOS Phase E: fstat() on a synthetic wrapper fd dispatches into the
    // wrapper's stream_stat() (vtable slot 8) and returns the associative stat
    // array it builds, so fstat($f)['size'] / ['mode'] read through the boxed
    // Mixed cell. The stat method is declared WITHOUT a return type so its
    // assoc array round-trips as a Mixed (a `: array` return would be
    // integer-keyed and reject the string keys). A wrapper without stream_stat
    // falls through to boxed false, matching PHP's fstat() failure.
    let out = compile_and_run(
        r#"<?php
class StatW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_read($c): string { return ""; }
    public function stream_eof(): bool { return true; }
    public function stream_stat() {
        return ['dev'=>0,'ino'=>0,'mode'=>33188,'nlink'=>1,'uid'=>0,'gid'=>0,
                'rdev'=>0,'size'=>5,'atime'=>0,'mtime'=>0,'ctime'=>0,
                'blksize'=>4096,'blocks'=>1];
    }
}
class NoStatW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_read($c): string { return ""; }
    public function stream_eof(): bool { return true; }
}
stream_wrapper_register("statw", "StatW");
stream_wrapper_register("nostatw", "NoStatW");
$f = fopen("statw://x", "r");
$s = fstat($f);
echo gettype($s) . ":" . $s['size'] . ":" . $s['mode'];
fclose($f);
echo "|";
$g = fopen("nostatw://y", "r");
$r = fstat($g);
echo ($r === false) ? "false" : "arr";
fclose($g);
"#,
    );
    assert_eq!(out, "array:5:33188|false");
}

/// Verifies compiled PHP output for file exists dispatches to wrapper url stat.
#[test]
fn test_file_exists_dispatches_to_wrapper_url_stat() {
    // OOS Phase E: file_exists("scheme://...") on a registered userspace wrapper
    // routes through __rt_user_wrapper_url_stat, instantiates the class, and
    // calls url_stat(string $path, int $flags). The path exists iff url_stat
    // returns a stat array (not false). A non-wrapper path falls back to the
    // real filesystem stat. url_stat must declare `string $path` (PHP's actual
    // signature) — an untyped param infers as Mixed and rejects string ops.
    let out = compile_and_run(
        r#"<?php
class SW {
    public function url_stat(string $path, int $flags) {
        if (strpos($path, "yes") !== false) {
            return ['dev'=>0,'ino'=>0,'mode'=>33188,'nlink'=>1,'uid'=>0,'gid'=>0,
                    'rdev'=>0,'size'=>10,'atime'=>0,'mtime'=>0,'ctime'=>0,
                    'blksize'=>4096,'blocks'=>1];
        }
        return false;
    }
}
stream_wrapper_register("sw", "SW");
file_put_contents("probe.txt", "x");
echo file_exists("sw://yes") ? "Y" : "N";
echo file_exists("sw://no") ? "Y" : "N";
echo file_exists("probe.txt") ? "Y" : "N";
echo file_exists("no_such_elephc_probe.txt") ? "Y" : "N";
"#,
    );
    assert_eq!(out, "YNYN");
}

/// Verifies compiled PHP output for filesize and is file dispatch to wrapper url stat.
#[test]
fn test_filesize_and_is_file_dispatch_to_wrapper_url_stat() {
    // OOS Phase E: filesize()/is_file() on a registered wrapper route through
    // __rt_user_wrapper_url_stat_field, which calls url_stat(string $path, int
    // $flags) and extracts the int 'size' (filesize) or 'mode' (is_file, then a
    // S_IFMT==S_IFREG check). Non-wrapper paths fall back to the real
    // filesystem. The url_stat result is a Mixed array; ['size']/['mode'] are
    // read via __rt_mixed_array_get and the boxes are released.
    let out = compile_and_run(
        r#"<?php
class SW {
    public function url_stat(string $path, int $flags) {
        if (strpos($path, "file") !== false) { return ['size'=>123, 'mode'=>33188]; }
        if (strpos($path, "dir")  !== false) { return ['size'=>0,   'mode'=>16877]; }
        return false;
    }
}
stream_wrapper_register("sw", "SW");
file_put_contents("real.txt", "abcde");
echo filesize("sw://file");
echo ":" . filesize("real.txt");
echo ":" . (is_file("sw://file") ? "Y" : "N");
echo ":" . (is_file("sw://dir") ? "Y" : "N");
echo ":" . (is_file("sw://nope") ? "Y" : "N");
echo ":" . (is_file("real.txt") ? "Y" : "N");
echo ":" . (is_file("no_such_elephc_probe") ? "Y" : "N");
"#,
    );
    assert_eq!(out, "123:5:Y:N:N:Y:N");
}

/// Verifies compiled PHP output for readfile dispatches to wrapper.
#[test]
fn test_readfile_dispatches_to_wrapper() {
    // OOS Phase E: readfile("scheme://...") on a registered wrapper routes
    // through __rt_readfile_wrapper (fopen + feof-gated fread drain to stdout +
    // close), echoing the wrapper's contents and returning the byte count. A
    // non-wrapper path falls back to __rt_readfile (raw open + stream), which
    // preserves the directory read-error semantics.
    let out = compile_and_run(
        r#"<?php
class RW {
    public $pos = 0;
    public function stream_open(string $p, string $m, int $o, &$op): bool { return true; }
    public function stream_read(int $count): string { if ($this->pos >= 5) { return ""; } $this->pos = 5; return "HELLO"; }
    public function stream_eof(): bool { return $this->pos >= 5; }
}
stream_wrapper_register("rw", "RW");
file_put_contents("rfr.txt", "abc");
$n = readfile("rw://x");
echo "|" . $n . "|";
$m = readfile("rfr.txt");
echo "|" . $m;
"#,
    );
    assert_eq!(out, "HELLO|5|abc|3");
}

/// Verifies compiled PHP output for fgetcsv and stream get line on wrapper.
#[test]
fn test_fgetcsv_and_stream_get_line_on_wrapper() {
    // OOS Phase E: fgetcsv() and stream_get_line() read from a wrapper fd.
    // fgetcsv goes through __rt_fgetcsv -> __rt_fgets, and stream_get_line
    // through __rt_stream_get_line; both gained a feof-gated, 1-byte __rt_fread
    // loop that accumulates into _user_wrapper_drain_buf (NOT _concat_buf, which
    // each __rt_fread result may occupy). The wrapper's stream_read honors
    // $count (returns a substr), matching PHP's stream_read contract.
    let out = compile_and_run(
        r#"<?php
class LW {
    public $data = "a,b,c\n1,2,3\n";
    public $pos = 0;
    public function stream_open(string $p, string $m, int $o, &$op): bool { $this->pos = 0; return true; }
    public function stream_read(int $count): string {
        $chunk = substr($this->data, $this->pos, $count);
        $this->pos = $this->pos + strlen($chunk);
        return $chunk;
    }
    public function stream_eof(): bool { return $this->pos >= strlen($this->data); }
}
stream_wrapper_register("lw", "LW");
$g = fopen("lw://x", "r");
$r1 = fgetcsv($g);
$r2 = fgetcsv($g);
echo implode("|", $r1) . ":" . implode("|", $r2);
fclose($g);
echo "/";
$h = fopen("lw://y", "r");
echo trim(stream_get_line($h, 100, "\n"));
echo ",";
echo trim(stream_get_line($h, 100, "\n"));
fclose($h);
"#,
    );
    assert_eq!(out, "a|b|c:1|2|3/a,b,c,1,2,3");
}

/// Verifies compiled PHP output for fopen user wrapper fflush dispatches to stream flush.
#[test]
fn test_fopen_user_wrapper_fflush_dispatches_to_stream_flush() {
    // Phase 10 follow-up: fflush() dispatches into the wrapper's stream_flush
    // and returns its bool result. Without stream_flush, the helper reports
    // success — "nothing to flush" is a benign default.
    let out = compile_and_run(
        r#"<?php
class FlushW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_flush(): bool { return true; }
}
class NoFlushW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("flushw", "FlushW");
stream_wrapper_register("noflush", "NoFlushW");
$f = fopen("flushw://x", "r");
echo fflush($f) ? "1" : "0";
echo "|";
$g = fopen("noflush://x", "r");
echo fflush($g) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1|1");
}

/// Verifies compiled PHP output for fopen user wrapper fseek dispatches to stream seek.
#[test]
fn test_fopen_user_wrapper_fseek_dispatches_to_stream_seek() {
    // Phase 10 step 4: fseek dispatches into the wrapper's stream_seek and
    // maps a `true` return to 0, anything else (including a missing method)
    // to -1 — matching PHP's int fseek() result.
    let out = compile_and_run(
        r#"<?php
class SeekW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_seek(int $offset, int $whence): bool { return true; }
}
stream_wrapper_register("seek", "SeekW");
$f = fopen("seek://x", "r");
echo fseek($f, 10);
echo "|";
echo fseek($f, 0, 2);
"#,
    );
    assert_eq!(out, "0|0");
}

/// Verifies compiled PHP output for fopen user wrapper fseek missing method returns minus one.
#[test]
fn test_fopen_user_wrapper_fseek_missing_method_returns_minus_one() {
    // Phase 10 step 4: when the wrapper class does not implement stream_seek,
    // the user-wrapper helper falls through to the PHP -1 failure sentinel.
    let out = compile_and_run(
        r#"<?php
class NoSeekW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("noseek", "NoSeekW");
$f = fopen("noseek://x", "r");
echo fseek($f, 10);
"#,
    );
    assert_eq!(out, "-1");
}

/// Verifies stream_set_blocking() and stream_set_timeout() on a registered
/// userspace-wrapper stream dispatch into the wrapper's stream_set_option(),
/// threading the option code and value; a wrapper without stream_set_option
/// returns false.
#[test]
fn test_stream_set_option_wrapper_dispatch() {
    // G1: stream_set_blocking($fp, $mode) → stream_set_option(STREAM_OPTION_BLOCKING=1,
    // mode?1:0, 0); stream_set_timeout($fp, $sec) → stream_set_option(
    // STREAM_OPTION_READ_TIMEOUT=4, sec, 0) — both via vtable slot 13 on a
    // synthetic wrapper fd. A wrapper missing stream_set_option yields false.
    let out = compile_and_run(
        r#"<?php
class OptW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_set_option(int $option, int $arg1, int $arg2): bool {
        if ($option === STREAM_OPTION_BLOCKING)     return $arg1 === 0;
        if ($option === STREAM_OPTION_READ_TIMEOUT) return $arg1 === 7;
        return false;
    }
}
class NoOptW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("opt", "OptW");
stream_wrapper_register("noopt", "NoOptW");
$f = fopen("opt://x", "r");
echo stream_set_blocking($f, false) ? "1" : "0";
echo stream_set_blocking($f, true)  ? "1" : "0";
echo stream_set_timeout($f, 7)      ? "1" : "0";
echo stream_set_timeout($f, 3)      ? "1" : "0";
echo "|";
$g = fopen("noopt://x", "r");
echo stream_set_blocking($g, false) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1010|0");
}

/// Verifies chmod() on a registered userspace-wrapper scheme dispatches into the
/// wrapper's stream_metadata($path, STREAM_META_ACCESS, $mode), threading the
/// option and mode through; a wrapper without stream_metadata returns false.
#[test]
fn test_chmod_wrapper_dispatches_to_stream_metadata() {
    // G1: chmod("scheme://path", $mode) on a registered wrapper routes to
    // stream_metadata (vtable slot 14) with option STREAM_META_ACCESS (6) and
    // value = $mode. A non-wrapper path keeps the libc chmod; a wrapper missing
    // stream_metadata yields false.
    let out = compile_and_run(
        r#"<?php
class MetaW {
    public function stream_metadata(string $path, int $option, mixed $value): bool {
        return $path === "mw://f" && $option === STREAM_META_ACCESS && $value === 0644;
    }
}
class NoMetaW {}
stream_wrapper_register("mw", "MetaW");
stream_wrapper_register("nm", "NoMetaW");
echo chmod("mw://f", 0644) ? "1" : "0";
echo chmod("mw://f", 0700) ? "1" : "0";
echo chmod("nm://f", 0644) ? "1" : "0";
"#,
    );
    assert_eq!(out, "100");
}

/// Verifies unlink()/mkdir()/rmdir() on a registered userspace-wrapper scheme
/// dispatch into the wrapper's matching path method, and that a wrapper without
/// the method (or a non-wrapper path) does not take the wrapper branch.
#[test]
fn test_user_wrapper_path_ops_dispatch() {
    // G1: unlink/mkdir/rmdir on a "scheme://" path matching a registered wrapper
    // route to the wrapper's unlink()/mkdir()/rmdir() (vtable slots 15/17/18),
    // returning their bool result; a wrapper missing the method yields false.
    let out = compile_and_run(
        r#"<?php
class PathW {
    public function unlink(string $path): bool { return $path === "pw://gone"; }
    public function mkdir(string $path): bool { return $path === "pw://newdir"; }
    public function rmdir(string $path): bool { return $path === "pw://olddir"; }
}
class BareW {}
stream_wrapper_register("pw", "PathW");
stream_wrapper_register("bare", "BareW");
echo unlink("pw://gone") ? "1" : "0";
echo mkdir("pw://newdir") ? "1" : "0";
echo rmdir("pw://olddir") ? "1" : "0";
echo "|";
echo unlink("pw://other") ? "1" : "0";
echo unlink("bare://x") ? "1" : "0";
"#,
    );
    assert_eq!(out, "111|00");
}

/// Verifies rename() on a registered userspace-wrapper source scheme dispatches
/// into the wrapper's rename(), threading both the source and destination URLs,
/// and that a wrapper without rename() returns false.
#[test]
fn test_user_wrapper_rename_dispatch() {
    // G1: rename($from, $to) where $from is a registered "scheme://" path routes
    // to the wrapper's rename() (vtable slot 16), passing both full URLs.
    let out = compile_and_run(
        r#"<?php
class MoveW {
    public function rename(string $from, string $to): bool {
        return $from === "mw://a" && $to === "mw://b";
    }
}
class NoMoveW {}
stream_wrapper_register("mw", "MoveW");
stream_wrapper_register("nm", "NoMoveW");
echo rename("mw://a", "mw://b") ? "1" : "0";
echo rename("mw://a", "mw://wrong") ? "1" : "0";
echo rename("nm://a", "nm://b") ? "1" : "0";
"#,
    );
    assert_eq!(out, "100");
}

/// Verifies flock() on a userspace-wrapper stream dispatches into the wrapper's
/// stream_lock(), threading the lock operation through, and returns its bool
/// result; a wrapper that does not implement stream_lock yields false.
#[test]
fn test_fopen_user_wrapper_flock_dispatches_to_stream_lock() {
    // G1: flock($fp, $op) on a synthetic wrapper fd routes to stream_lock($op).
    // The wrapper reports whether it received LOCK_EX, proving the operation is
    // threaded through; a wrapper missing stream_lock falls through to false.
    let out = compile_and_run(
        r#"<?php
class LockW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_lock(int $operation): bool { return $operation === LOCK_EX; }
}
class NoLockW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("lockw", "LockW");
stream_wrapper_register("nolock", "NoLockW");
$f = fopen("lockw://x", "r");
echo flock($f, LOCK_EX) ? "1" : "0";
echo "|";
echo flock($f, LOCK_SH) ? "1" : "0";
echo "|";
$g = fopen("nolock://x", "r");
echo flock($g, LOCK_EX) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1|0|0");
}

/// Verifies ftruncate() on a userspace-wrapper stream dispatches into the
/// wrapper's stream_truncate(), threading the new size through, and returns its
/// bool result; a wrapper that does not implement stream_truncate yields false.
#[test]
fn test_fopen_user_wrapper_ftruncate_dispatches_to_stream_truncate() {
    // G1: ftruncate($fp, $size) on a synthetic wrapper fd routes to
    // stream_truncate($new_size). The wrapper reports whether it received 42,
    // proving the size is threaded; a wrapper missing stream_truncate is false.
    let out = compile_and_run(
        r#"<?php
class TruncW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
    public function stream_truncate(int $new_size): bool { return $new_size === 42; }
}
class NoTruncW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("truncw", "TruncW");
stream_wrapper_register("notrunc", "NoTruncW");
$f = fopen("truncw://x", "w");
echo ftruncate($f, 42) ? "1" : "0";
echo "|";
echo ftruncate($f, 7) ? "1" : "0";
echo "|";
$g = fopen("notrunc://x", "w");
echo ftruncate($g, 42) ? "1" : "0";
"#,
    );
    assert_eq!(out, "1|0|0");
}

/// Verifies compiled PHP output for fopen user wrapper stream open receives opened path arg.
#[test]
fn test_fopen_user_wrapper_stream_open_receives_opened_path_arg() {
    // Phase 10 follow-up: stream_open is now called with the 5th
    // `?string &$opened_path` argument (a writable scratch slot). Wrappers
    // that declare the PHP-faithful 5-arg signature must dispatch
    // correctly. The value the wrapper writes back is not surfaced to the
    // caller (v1 limitation), but the wrapper must be able to write
    // without crashing.
    let out = compile_and_run(
        r#"<?php
class OpenedW {
    public bool $touched_opened_path = false;
    public function stream_open(string $path, string $mode, int $options, ?string &$opened_path): bool {
        $opened_path = "/resolved/" . $path;
        $this->touched_opened_path = true;
        return true;
    }
    public function stream_eof(): bool { return false; }
}
stream_wrapper_register("opened", "OpenedW");
$f = fopen("opened://x", "r");
echo is_resource($f) ? "ok" : "fail";
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen user wrapper handles above old cap.
#[test]
fn test_fopen_user_wrapper_handles_above_old_cap() {
    // Phase 10 follow-up: bumped USER_WRAPPER_HANDLES_CAP from 64 to 256.
    // Opens 100 concurrent wrapper handles, each backed by a no-op stream_open
    // that returns true. Used to overflow the 64-slot table; now succeeds.
    let out = compile_and_run(
        r#"<?php
class CapW {
    public function stream_open($p, $m, $o, &$op): bool { return true; }
}
stream_wrapper_register("cap", "CapW");
$handles = [];
for ($i = 0; $i < 100; $i++) {
    $h = fopen("cap://x", "r");
    if (!is_resource($h)) { echo "fail@" . $i; return; }
    $handles[] = $h;
}
echo "ok-" . count($handles);
"#,
    );
    assert_eq!(out, "ok-100");
}

/// Verifies compiled PHP output for fopen user wrapper failure does not leak.
#[test]
fn test_fopen_user_wrapper_failure_does_not_leak() {
    // Phase 10 follow-up: after stream_open returns false, the runtime
    // helper releases the wrapper object via __rt_object_free_deep so
    // long-running programs that probe many failing wrappers do not
    // accumulate one heap object per attempt. Loops 256 fopen calls and
    // checks the loop completes (a stress signal — the leak path itself
    // is verified by the deep-free call being on the path).
    let out = compile_and_run(
        r#"<?php
class MyW {
    public function stream_open($p, $m, $o, &$op): bool { return false; }
}
stream_wrapper_register("leak", "MyW");
for ($i = 0; $i < 256; $i++) {
    $f = fopen("leak://x", "r");
    if ($f !== false) {
        echo "leaked"; return;
    }
}
echo "ok";
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen user wrapper stream open false returns false.
#[test]
fn test_fopen_user_wrapper_stream_open_false_returns_false() {
    // Phase 10 step 3: when the wrapper class's stream_open returns false,
    // fopen() reports failure (PHP `false`) without emitting the standard
    // "Failed to open stream" warning.
    let out = compile_and_run_capture(
        r#"<?php
class MyW {
    public function stream_open($path, $mode, $options, &$opened): bool {
        return false;
    }
}
stream_wrapper_register("my", "MyW");
$f = fopen("my://anywhere", "r");
echo $f === false ? "false" : "open";
"#,
    );
    assert_eq!(out.stdout, "false");
    assert!(
        !out.stderr.contains("Failed to open"),
        "wrapper stream_open returning false should not emit the failed-to-open warning, got stderr: {:?}",
        out.stderr,
    );
}

/// Verifies compiled PHP output for stream socket get name.
#[test]
fn test_stream_socket_get_name() {
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54743");
echo stream_socket_get_name($srv, false);
echo "|";
$cli = stream_socket_client("tcp://127.0.0.1:54743");
echo stream_socket_get_name($cli, true);
"#,
    );
    assert_eq!(out, "127.0.0.1:54743|127.0.0.1:54743");
}

/// Verifies compiled PHP output for stream socket client resolves hostname.
#[test]
fn test_stream_socket_client_resolves_hostname() {
    // A non-numeric host in a socket address is resolved through gethostbyname.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://127.0.0.1:54920");
$cli = stream_socket_client("tcp://localhost:54920");
$conn = stream_socket_accept($srv);
fwrite($cli, "resolved");
echo fread($conn, 16);
"#,
    );
    assert_eq!(out, "resolved");
}

/// Verifies compiled PHP output for stream socket server resolves hostname.
#[test]
fn test_stream_socket_server_resolves_hostname() {
    // Host-name resolution applies to the server bind address too.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://localhost:54921");
$cli = stream_socket_client("tcp://127.0.0.1:54921");
$conn = stream_socket_accept($srv);
fwrite($cli, "bound by name");
echo fread($conn, 32);
"#,
    );
    assert_eq!(out, "bound by name");
}

/// Verifies compiled PHP output for stream socket client ipv6 hostname via dns.
#[test]
fn test_stream_socket_client_ipv6_hostname_via_dns() {
    // Phase 11 B1: tcp://[hostname]:port now resolves the bracketed token
    // through getaddrinfo with AF_INET6 hint when inet_pton rejects the
    // literal. `localhost` resolves to ::1 on every supported system, so
    // a server bound to [::1] accepts the client built from
    // [localhost]:port end-to-end without any literal-IPv6 input.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://[::1]:55821");
echo is_resource($srv) ? "srv|" : "srv_fail|";
$cli = stream_socket_client("tcp://[localhost]:55821");
echo is_resource($cli) ? "cli|" : "cli_fail|";
$conn = stream_socket_accept($srv);
fwrite($cli, "v6-dns");
echo fread($conn, 16);
fclose($conn); fclose($cli); fclose($srv);
"#,
    );
    assert_eq!(out, "srv|cli|v6-dns");
}

/// Verifies compiled PHP output for stream socket server ipv6 literal roundtrip.
#[test]
fn test_stream_socket_server_ipv6_literal_roundtrip() {
    // Full PHP-side IPv6 round-trip: stream_socket_server binds [::1]:port,
    // stream_socket_client connects, fwrite/fread carry the payload. This
    // exercises both __rt_stream_socket_server_v6 and the client's IPv6
    // dispatch in the same binary.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://[::1]:54937");
echo is_resource($srv) ? "srv|" : "srv_fail|";
$cli = stream_socket_client("tcp://[::1]:54937");
echo is_resource($cli) ? "cli|" : "cli_fail|";
$conn = stream_socket_accept($srv);
fwrite($cli, "v6-ping");
echo fread($conn, 16);
"#,
    );
    assert_eq!(out, "srv|cli|v6-ping");
}

/// Verifies compiled PHP output for udp ipv6 round trip.
#[test]
fn test_udp_ipv6_round_trip() {
    // UDP over IPv6: stream_socket_server binds [::1]:port with SOCK_DGRAM
    // (no listen), stream_socket_client connects (sets default target),
    // fwrite/fread carry one datagram each way. This exercises the
    // udp:// scheme detection in both v6 dispatchers.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("udp://[::1]:54939");
echo is_resource($srv) ? "srv|" : "srv_fail|";
$cli = stream_socket_client("udp://[::1]:54939");
echo is_resource($cli) ? "cli|" : "cli_fail|";
fwrite($cli, "v6-udp");
echo fread($srv, 16);
"#,
    );
    assert_eq!(out, "srv|cli|v6-udp");
}

/// Verifies compiled PHP output for stream socket get name ipv6.
#[test]
fn test_stream_socket_get_name_ipv6() {
    // stream_socket_get_name on an AF_INET6 socket should surface the peer
    // as `[ipv6]:port`. The local server's bound port is deterministic; the
    // client's source port is ephemeral, so check that the result starts
    // with the bracketed IPv6 prefix.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("tcp://[::1]:54938");
echo stream_socket_get_name($srv, false) . "\n";
$cli = stream_socket_client("tcp://[::1]:54938");
echo stream_socket_get_name($cli, true) . "\n";
echo substr(stream_socket_get_name($cli, false), 0, 5);
"#,
    );
    assert_eq!(out, "[::1]:54938\n[::1]:54938\n[::1]");
}

/// Verifies compiled PHP output for stream socket client ipv6 literal roundtrip.
#[test]
fn test_stream_socket_client_ipv6_literal_roundtrip() {
    // tcp://[::1]:port routes through the IPv6 dispatch: __rt_inet6_pton
    // parses the bracketed literal, the helper builds a sockaddr_in6, and
    // connects via AF_INET6. The Rust-side listener binds to ::1 so we
    // exercise the full IPv6 socket stack without any DNS dependency.
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("[::1]:54936")
        .expect("ipv6 test: bind [::1]:54936");
    let handle = std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().expect("ipv6 test: accept");
        let mut buf = [0u8; 4];
        sock.read_exact(&mut buf).expect("ipv6 test: read");
        sock.write_all(b"PONG").expect("ipv6 test: write");
        buf
    });
    let out = compile_and_run(
        r#"<?php
$cli = stream_socket_client("tcp://[::1]:54936");
echo is_resource($cli) ? "ok|" : "fail|";
fwrite($cli, "PING");
echo fread($cli, 4);
"#,
    );
    let read_buf = handle.join().expect("ipv6 test: join");
    assert_eq!(&read_buf, b"PING");
    assert_eq!(out, "ok|PONG");
}

/// Verifies compiled PHP output for stream socket client unresolvable host is false.
#[test]
fn test_stream_socket_client_unresolvable_host_is_false() {
    // An unresolvable host fails the connection like any bad address.
    let out = compile_and_run(
        r#"<?php $c = stream_socket_client("tcp://no-such-host.invalid:1234"); echo is_bool($c) ? "false" : "resource";"#,
    );
    assert_eq!(out, "false");
}

/// Verifies compiled PHP output for stream socket pair unsupported domain is false.
#[test]
fn test_stream_socket_pair_unsupported_domain_is_false() {
    // socketpair() refuses STREAM_PF_INET on every platform we target.
    // PHP's contract is `array|false`, so the return must be strictly
    // false (not an empty array) for === comparisons to work.
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_INET, STREAM_SOCK_STREAM, 0);
echo gettype($pair);
echo "|";
echo ($pair === false) ? "strict_false" : "not_false";
"#,
    );
    assert_eq!(out, "boolean|strict_false");
}

/// Verifies compiled PHP output for stream socket pair round trip.
#[test]
fn test_stream_socket_pair_round_trip() {
    // Also a regression test for indexed reads of an array<resource>:
    // $pair[0] / $pair[1] must yield the stored descriptors, not the index.
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
echo count($pair);
echo "|";
fwrite($pair[0], "ping");
echo fread($pair[1], 16);
echo "|";
fwrite($pair[1], "pong");
echo fread($pair[0], 16);
"#,
    );
    assert_eq!(out, "2|ping|pong");
}

/// Verifies socket-pair elements own opaque registry handles after the result array is released.
#[test]
fn test_stream_socket_pair_handles_survive_result_array_release() {
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
$left = $pair[0];
$right = $pair[1];
$distinct = get_resource_id($left) !== get_resource_id($right);
unset($pair);
echo get_resource_type($left) . "|" . get_resource_type($right) . "|";
echo $distinct ? "distinct|" : "same|";
fwrite($left, "owned");
echo fread($right, 5);
"#,
    );
    assert_eq!(out, "stream|stream|distinct|owned");
}

/// Verifies compiled PHP output for stream socket get name udp.
#[test]
fn test_stream_socket_get_name_udp() {
    // Phase 5 audit: stream_socket_get_name on a UDP socket must format the
    // bound address as A.B.C.D:port, just like the TCP case. Both the local
    // (server) and peer (client) sides should report the bound port.
    let out = compile_and_run(
        r#"<?php
$srv = stream_socket_server("udp://127.0.0.1:54928");
echo stream_socket_get_name($srv, false);
echo "|";
$cli = stream_socket_client("udp://127.0.0.1:54928");
echo stream_socket_get_name($cli, true);
"#,
    );
    assert_eq!(out, "127.0.0.1:54928|127.0.0.1:54928");
}

/// Verifies compiled PHP output for stream socket get name unix.
#[test]
fn test_stream_socket_get_name_unix() {
    // Phase 5 audit: stream_socket_get_name on a Unix-domain socket must
    // surface the filesystem path, not garbage parsed out of a sockaddr_in.
    // Use a process-unique path so parallel tests do not collide.
    let out = compile_and_run(
        r#"<?php
$path = "/tmp/elephc_unix_getname_test.sock";
unlink($path);
$srv = stream_socket_server("unix://" . $path);
echo stream_socket_get_name($srv, false);
unlink($path);
"#,
    );
    assert_eq!(out, "/tmp/elephc_unix_getname_test.sock");
}

/// Verifies compiled PHP output for popen read mode.
#[test]
fn test_popen_read_mode() {
    let out = compile_and_run(
        r#"<?php
$p = popen("printf abc", "r");
echo fread($p, 16);
echo "|";
echo pclose($p);
"#,
    );
    assert_eq!(out, "abc|0");
}

/// Verifies compiled PHP output for opendir readdir iterates directory.
#[test]
fn test_opendir_readdir_iterates_directory() {
    let out = compile_and_run(
        r#"<?php
mkdir("sub");
file_put_contents("sub/alpha.txt", "a");
$d = opendir("sub");
$count = 0;
$found = 0;
while (($e = readdir($d)) !== false) {
    $count = $count + 1;
    if ($e === "alpha.txt") { $found = 1; }
}
closedir($d);
echo $count . ":" . $found;
"#,
    );
    assert_eq!(out, "3:1");
}

/// Verifies compiled PHP output for opendir invalid path returns false.
#[test]
fn test_opendir_invalid_path_returns_false() {
    let out = compile_and_run(
        r#"<?php
var_dump(opendir("/nonexistent/path/elephc-xyz") === false);
"#,
    );
    assert_eq!(out, "bool(true)\n");
}

/// Verifies compiled PHP output for readdir returns false at end of directory.
#[test]
fn test_readdir_returns_false_at_end_of_directory() {
    let out = compile_and_run(
        r#"<?php
mkdir("ed");
$d = opendir("ed");
$a = readdir($d);
$b = readdir($d);
$x = readdir($d);
closedir($d);
echo (is_string($a) ? "s" : "?");
echo (is_string($b) ? "s" : "?");
echo ($x === false ? "F" : "?");
"#,
    );
    assert_eq!(out, "ssF");
}

/// Verifies compiled PHP output for rewinddir restarts iteration.
#[test]
fn test_rewinddir_restarts_iteration() {
    let out = compile_and_run(
        r#"<?php
mkdir("rd");
$d = opendir("rd");
$first = readdir($d);
readdir($d);
$end = readdir($d);
rewinddir($d);
$again = readdir($d);
closedir($d);
echo ($end === false ? "1" : "0");
echo ($again === $first ? "1" : "0");
"#,
    );
    assert_eq!(out, "11");
}

/// Verifies `closedir` invalidates the old PHP resource while a new handle remains usable.
#[test]
fn test_closedir_allows_directory_handle_reuse() {
    let out = compile_and_run(
        r#"<?php
mkdir("cd");
$d1 = opendir("cd");
closedir($d1);
$d2 = opendir("cd");
$e = readdir($d2);
closedir($d2);
echo (is_resource($d2) ? "r" : "?");
echo (is_string($e) ? "ok" : "no");
"#,
    );
    assert_eq!(out, "?ok");
}

/// Verifies compiled PHP output for array literal of resources round trips.
#[test]
fn test_array_literal_of_resources_round_trips() {
    let out = compile_and_run(
        r#"<?php
$arr = [STDIN, STDOUT, STDERR];
echo $arr[0] . "|" . $arr[1] . "|" . $arr[2];
"#,
    );
    assert_eq!(out, "Resource id #1|Resource id #2|Resource id #3");
}

/// Verifies associative array literals preserve resource value metadata.
#[test]
fn test_assoc_array_literal_of_resources_round_trips() {
    let out = compile_and_run(
        r#"<?php
$arr = ["in" => STDIN, "out" => STDOUT, "err" => STDERR];
echo $arr["in"] . "|" . $arr["out"] . "|" . $arr["err"];
"#,
    );
    assert_eq!(out, "Resource id #1|Resource id #2|Resource id #3");
}

/// Verifies compiled PHP output for stream get meta data describes file stream.
#[test]
fn test_stream_get_meta_data_describes_file_stream() {
    let out = compile_and_run(
        r#"<?php
$f = fopen("meta.txt", "w");
$m = stream_get_meta_data($f);
echo "mode=" . $m["mode"];
echo " seekable=" . ($m["seekable"] ? "1" : "0");
echo " eof=" . ($m["eof"] ? "1" : "0");
echo " type=" . $m["stream_type"];
echo " wrap=" . $m["wrapper_type"];
echo " blocked=" . ($m["blocked"] ? "1" : "0");
echo " unread=" . $m["unread_bytes"];
echo " timed_out=" . ($m["timed_out"] ? "1" : "0");
fclose($f);
"#,
    );
    assert_eq!(
        out,
        "mode=w seekable=1 eof=0 type=STDIO wrap=plainfile blocked=1 unread=0 timed_out=0"
    );
}

/// Verifies the `data:` wrapper reports PHP's name for it, `RFC2397`.
///
/// elephc answered `data` — the scheme, not the wrapper. Reference PHP 8.5.6 names it
/// after the RFC that defines `data:` URLs, and a program branching on `wrapper_type`
/// (as PSR-7 and Flysystem adapters do) saw a name that exists nowhere in PHP.
#[test]
fn test_stream_get_meta_data_names_the_data_wrapper_rfc2397() {
    let out = compile_and_run(
        r#"<?php
$d = fopen("data://text/plain,hi", "r");
echo stream_get_meta_data($d)["wrapper_type"];
"#,
    );
    assert_eq!(out, "RFC2397");
}

/// Verifies compiled PHP output for stream get meta data reports eof consistently with feof.
#[test]
fn test_stream_get_meta_data_reports_eof_consistently_with_feof() {
    let out = compile_and_run(
        r#"<?php
file_put_contents("meta2.txt", "ab");
$f = fopen("meta2.txt", "r");
fread($f, 10);
fread($f, 10);
$m = stream_get_meta_data($f);
echo ($m["eof"] ? "eof" : "no");
echo ":";
echo ($m["eof"] === feof($f) ? "consistent" : "differ");
fclose($f);
"#,
    );
    assert_eq!(out, "eof:consistent");
}

/// Verifies compiled PHP output for readdir loop collects results into array.
#[test]
fn test_readdir_loop_collects_results_into_array() {
    // Regression: appending a string|false value to an array inside a loop
    // re-ran the indexed-to-mixed conversion every iteration, corrupting the
    // already-boxed earlier elements.
    let out = compile_and_run(
        r#"<?php
mkdir("collectdir");
file_put_contents("collectdir/x.txt", "1");
$d = opendir("collectdir");
$names = [];
while (($e = readdir($d)) !== false) { $names[] = $e; }
closedir($d);
echo count($names);
echo is_string($names[0]) ? "s" : "?";
echo is_string($names[1]) ? "s" : "?";
echo is_string($names[2]) ? "s" : "?";
"#,
    );
    assert_eq!(out, "3sss");
}

/// Verifies compiled PHP output for stream select detects ready socket.
#[test]
fn test_stream_select_detects_ready_socket() {
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(1, 1, 0);
$a = $pair[0];
$b = $pair[1];
fwrite($a, "ping");
$r1 = [$b]; $w1 = []; $e1 = [];
$n1 = stream_select($r1, $w1, $e1, 0, 0);
$r2 = [$a]; $w2 = []; $e2 = [];
$n2 = stream_select($r2, $w2, $e2, 0, 0);
echo "n1=" . $n1 . " r1=" . count($r1) . " n2=" . $n2 . " r2=" . count($r2);
"#,
    );
    assert_eq!(out, "n1=1 r1=1 n2=0 r2=0");
}

/// Verifies compiled PHP output for stream select compacts to ready subset.
#[test]
fn test_stream_select_compacts_to_ready_subset() {
    let out = compile_and_run(
        r#"<?php
$p1 = stream_socket_pair(1, 1, 0);
$p2 = stream_socket_pair(1, 1, 0);
fwrite($p1[0], "x");
$r = [$p1[1], $p2[1]];
$w = [];
$e = [];
$n = stream_select($r, $w, $e, 0, 0);
echo $n . ":" . count($r);
"#,
    );
    assert_eq!(out, "1:1");
}

/// Verifies compiled PHP output for stream bucket append then pop in order.
#[test]
fn test_stream_bucket_append_then_pop_in_order() {
    // Phase 11 B4 v2: stream_bucket_append actually appends to the
    // brigade's _buckets indexed-array property; stream_bucket_make_writeable
    // actually pops the head. With three appends and three pops in a row
    // we should observe FIFO order matching what PHP's bucket brigade
    // semantics guarantee.
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
$brigade = new stdClass();
stream_bucket_append($brigade, stream_bucket_new($m, "alpha"));
stream_bucket_append($brigade, stream_bucket_new($m, "beta"));
stream_bucket_append($brigade, stream_bucket_new($m, "gamma"));
while (true) {
    $b = stream_bucket_make_writeable($brigade);
    if (is_null($b)) break;
    echo "[" . $b->data . "]";
}
echo "|done";
fclose($m);
"#,
    );
    assert_eq!(out, "[alpha][beta][gamma]|done");
}

/// Verifies prepend order and brigade growth beyond the initial bucket-array capacity.
#[test]
fn test_stream_bucket_prepend_then_pop_in_reverse_insertion_order() {
    let out = compile_and_run(
        r#"<?php
$m = fopen("php://memory", "r+");
$brigade = new stdClass();
stream_bucket_prepend($brigade, stream_bucket_new($m, "alpha"));
stream_bucket_prepend($brigade, stream_bucket_new($m, "beta"));
stream_bucket_prepend($brigade, stream_bucket_new($m, "gamma"));
stream_bucket_prepend($brigade, stream_bucket_new($m, "delta"));
stream_bucket_prepend($brigade, stream_bucket_new($m, "epsilon"));
stream_bucket_prepend($brigade, stream_bucket_new($m, "zeta"));
while (true) {
    $b = stream_bucket_make_writeable($brigade);
    if (is_null($b)) break;
    echo "[" . $b->data . "]";
}
echo "|done";
fclose($m);
"#,
    );
    assert_eq!(out, "[zeta][epsilon][delta][gamma][beta][alpha]|done");
}

/// Verifies compiled PHP output for user filter 4arg brigade dispatch.
#[test]
fn test_user_filter_4arg_brigade_dispatch() {
    // Phase 11 B4 v2: when a user filter class's filter() method has 4
    // parameters, the runtime dispatcher seeds an input brigade with one
    // bucket (the just-read stream bytes), calls
    // `filter($in, $out, &$consumed, $closing)`, then walks the output
    // brigade's `_buckets` indexed-array and concatenates each
    // `$bucket->data` string into the post-filter buffer.
    //
    // Simplest end-to-end check: a "pass-through" filter that pops the
    // input bucket and appends it to the output brigade. The fread()
    // result is the original file bytes routed through the brigade
    // pipeline.
    let out = compile_and_run(
        r#"<?php
class PassThrough {
    public function filter($in, $out, $consumed, $closing): int {
        $b = stream_bucket_make_writeable($in);
        stream_bucket_append($out, $b);
        return 2;  // PSFS_PASS_ON
    }
}
stream_filter_register("pass.test", "PassThrough");
$path = tempnam(sys_get_temp_dir(), "elephc_brigade_e2e_");
file_put_contents($path, "hello brigade");
$f = fopen($path, "r");
stream_filter_append($f, "pass.test");
$content = fread($f, 64);
echo $content;
fclose($f);
unlink($path);
"#,
    );
    assert_eq!(out, "hello brigade");
}

/// Verifies compiled PHP output for user filter 4arg brigade transforms via while loop.
#[test]
fn test_user_filter_4arg_brigade_transforms_via_while_loop() {
    // Regression for two pre-existing Mixed bugs that blocked the canonical
    // PHP brigade-filter idiom (both fixed alongside this test):
    //   1. `while ($b = stream_bucket_make_writeable($in))` — the loop
    //      condition evaluates a Mixed(object) for truthiness;
    //      __rt_mixed_cast_bool used to treat tag-6 (object) as falsy, so the
    //      loop body never ran.
    //   2. `strtoupper($b->data)` — strtoupper/strtolower read a Mixed operand
    //      via a bare emit_expr and left a boxed cell in x0 with stale string
    //      registers, yielding an empty result; they now route through
    //      emit_string_arg (coerce_to_string → __rt_mixed_cast_string).
    // Together they make a transforming 4-arg brigade filter round-trip.
    let out = compile_and_run(
        r#"<?php
class UpBrigade {
    public $context;
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $b->data = strtoupper($b->data);
            $consumed += $b->datalen;
            stream_bucket_append($out, $b);
        }
        return PSFS_PASS_ON;
    }
}
stream_filter_register("up.brigade", "UpBrigade");
$w = fopen("php://temp", "w+");
stream_filter_append($w, "up.brigade", STREAM_FILTER_WRITE);
fwrite($w, "hello brigade");
rewind($w);
echo fread($w, 64);
"#,
    );
    assert_eq!(out, "HELLO BRIGADE");
}

/// Verifies a user filter returning PSFS_ERR_FATAL yields an empty read result.
#[test]
fn test_user_filter_psfs_err_fatal() {
    let out = compile_and_run(
        r#"<?php
class FatalFilter extends php_user_filter {
    public function filter($in, $out, &$consumed, $closing): int {
        return PSFS_ERR_FATAL;
    }
}
stream_filter_register("fatal", "FatalFilter");
$f = fopen("php://memory", "r+");
fwrite($f, "hello\n");
rewind($f);
stream_filter_append($f, "fatal");
$r = fread($f, 100);
echo "len=" . strlen($r) . "|";
"#,
    );
    assert_eq!(out, "len=0|");
}

/// Verifies a user filter that only ever answers `PSFS_FEED_ME` yields NOTHING.
///
/// This fixture used to assert `"hello\n"` — it pinned the defect. `PSFS_FEED_ME` means the
/// filter took the input and has no output yet, so passing the input through handed the caller
/// raw, unfiltered bytes. Measured against php 8.5.6, which answers the empty string here (plus
/// a "Unprocessed filter buckets remaining on input brigade" warning elephc does not emit).
#[test]
fn test_user_filter_psfs_feed_me() {
    let out = compile_and_run(
        r#"<?php
class FeedMeFilter extends php_user_filter {
    public function filter($in, $out, &$consumed, $closing): int {
        return PSFS_FEED_ME;
    }
}
stream_filter_register("feedme", "FeedMeFilter");
$f = fopen("php://memory", "r+");
fwrite($f, "hello\n");
rewind($f);
stream_filter_append($f, "feedme");
$r = fread($f, 100);
echo "len=", strlen($r);
"#,
    );
    assert_eq!(out, "len=0");
}

/// Verifies a user filter returning PSFS_PASS_ON transforms the output (control).
#[test]
fn test_user_filter_psfs_pass_on_control() {
    let out = compile_and_run(
        r#"<?php
class UpperFilter extends php_user_filter {
    public function filter($in, $out, &$consumed, $closing): int {
        while ($b = stream_bucket_make_writeable($in)) {
            $b->data = strtoupper($b->data);
            stream_bucket_append($out, $b);
        }
        return PSFS_PASS_ON;
    }
}
stream_filter_register("upper", "UpperFilter");
$f = fopen("php://memory", "r+");
fwrite($f, "hello\n");
rewind($f);
stream_filter_append($f, "upper");
echo fread($f, 100);
"#,
    );
    assert_eq!(out, "HELLO\n");
}

/// Verifies compiled PHP output for mixed object is truthy.
#[test]
fn test_mixed_object_is_truthy() {
    // Regression: a Mixed cell holding an object (tag 6) must be truthy in a
    // boolean context, matching PHP. __rt_mixed_cast_bool previously fell
    // through to the falsy default for tag 6 (only int/string/float/bool/
    // array/resource were handled). A Mixed(null) stays falsy.
    let out = compile_and_run(
        r#"<?php
class C { public $x = 1; }
function mk(): mixed { return new C(); }
function nope(): mixed { return null; }
$o = mk();
echo ($o ? "obj-truthy" : "obj-falsy");
$n = nope();
echo ($n ? "|null-truthy" : "|null-falsy");
"#,
    );
    assert_eq!(out, "obj-truthy|null-falsy");
}

/// Verifies compiled PHP output for fopen http content emits content length header.
#[test]
#[ignore = "test is reliable standalone but flakes in parallel sweep (port-binding race); the underlying Content-Length emission is verified by ad-hoc Ruby + standalone elephc runs — see the http_build_request.rs commit body for the reproduction"]
fn test_fopen_http_content_emits_content_length_header() {
    // Phase 11 B2 polish: when $ctx['http']['content'] is set, the request
    // line carries a `Content-Length: <N>\r\n` header so the receiving
    // server knows how many body bytes to read. (The earlier B2 commit
    // landed the body append but left the Content-Length emission stubbed
    // with a TEMPORARILY-DISABLED branch on ARM64; this verifies the
    // re-enabled path puts the right bytes on the wire.)
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "method", "POST");
stream_context_set_option(stream_context_get_default(), "http", "content", "hello body");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/", "r");
$req = stream_get_contents($f);
fclose($f);
// The echo server replays the request headers (bytes up to the blank
// line) as the response body. Substr-based search instead of strpos
// to dodge any `!== false` quirks on Mixed return values.
$found = false;
$needle = "Content-Length: 10";
$nlen = strlen($needle);
for ($i = 0; $i + $nlen <= strlen($req); $i++) {
    if (substr($req, $i, $nlen) === $needle) { $found = true; break; }
}
echo $found ? "ok" : "MISS:" . strlen($req);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream context set default returns resource.
#[test]
fn test_stream_context_set_default_returns_resource() {
    let out = compile_and_run(
        r#"<?php
$r = stream_context_set_default(["http" => ["method" => "POST"]]);
echo is_resource($r) ? "resource" : "no";
"#,
    );
    assert_eq!(out, "resource");
}

/// Verifies compiled PHP output for stream context set params returns true.
#[test]
fn test_stream_context_set_params_returns_true() {
    let out = compile_and_run(
        r#"<?php
$ctx = stream_context_create();
echo stream_context_set_params($ctx, []) ? "ok" : "FAIL";
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for stream resolve include path existing and missing.
#[test]
fn test_stream_resolve_include_path_existing_and_missing() {
    let out = compile_and_run(
        r#"<?php
$r = stream_resolve_include_path("/tmp");
$miss = stream_resolve_include_path("/non/existent/xyz");
echo (is_string($r) ? "s" : "n") . "|" . ($miss === false ? "f" : "x");
"#,
    );
    assert_eq!(out, "s|f");
}

/// Verifies compiled PHP output for fopen http user agent in request.
#[test]
fn test_fopen_http_user_agent_in_request() {
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "user_agent", "MyApp/2.0");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/", "r");
$req = stream_get_contents($f);
fclose($f);
$needle = "User-Agent: MyApp/2.0";
$nlen = strlen($needle);
$found = false;
for ($i = 0; $i + $nlen <= strlen($req); $i++) {
    if (substr($req, $i, $nlen) === $needle) { $found = true; break; }
}
echo $found ? "ok" : "MISS";
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen http protocol version 1 1.
#[test]
fn test_fopen_http_protocol_version_1_1() {
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "protocol_version", "1.1");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/", "r");
$req = stream_get_contents($f);
fclose($f);
$needle = "HTTP/1.1";
$nlen = strlen($needle);
$found = false;
for ($i = 0; $i + $nlen <= strlen($req); $i++) {
    if (substr($req, $i, $nlen) === $needle) { $found = true; break; }
}
echo $found ? "ok" : "MISS";
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "ok");
}

/// Verifies compiled PHP output for fopen php fd n writes to descriptor.
#[test]
fn test_fopen_php_fd_n_writes_to_descriptor() {
    let out = compile_and_run(
        r#"<?php
$f = fopen("php://fd/1", "w");
fwrite($f, "fd-route");
fclose($f);
"#,
    );
    assert_eq!(out, "fd-route");
}

/// Verifies compiled PHP output for fopen http request fulluri in request line.
#[test]
fn test_fopen_http_request_fulluri_in_request_line() {
    let (_server, port) = spawn_http_echo_server();
    let out = compile_and_run(
        &r#"<?php
stream_context_set_option(stream_context_get_default(), "http", "request_fulluri", "1");
$f = fopen("http://127.0.0.1:PHP_TEST_PORT/path", "r");
$req = stream_get_contents($f);
fclose($f);
$needle = "GET http://127.0.0.1:PHP_TEST_PORT/path HTTP/1.0";
$nlen = strlen($needle);
$found = false;
for ($i = 0; $i + $nlen <= strlen($req); $i++) {
    if (substr($req, $i, $nlen) === $needle) { $found = true; break; }
}
echo $found ? "ok" : "MISS";
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "ok");
}

/// Verifies opendir()/readdir()/rewinddir()/closedir() on a registered userspace
/// wrapper dispatch to dir_opendir/dir_readdir/dir_rewinddir/dir_closedir (vtable
/// slots 19-22) through a synthetic handle fd, with object state (the read
/// cursor) persisting across the readdir() calls and surviving a rewinddir().
#[test]
fn test_opendir_readdir_wrapper_dispatch() {
    let out = compile_and_run(
        r#"<?php
class MyDir {
    public $context;
    public $pos = 0;
    public function dir_opendir($path, $options): bool {
        $this->pos = 0;
        return true;
    }
    public function dir_readdir(): string {
        $names = ["a.txt", "b.txt"];
        if ($this->pos >= 2) {
            return "";
        }
        $n = $names[$this->pos];
        $this->pos = $this->pos + 1;
        return $n;
    }
    public function dir_rewinddir(): bool {
        $this->pos = 0;
        return true;
    }
    public function dir_closedir(): bool {
        echo "closed\n";
        return true;
    }
}
stream_wrapper_register("mydir", "MyDir");
$dh = opendir("mydir://x");
while (($f = readdir($dh)) !== false) {
    echo "$f\n";
}
rewinddir($dh);
$g = readdir($dh);
echo "rewound:$g\n";
closedir($dh);
echo "done\n";
"#,
    );
    assert_eq!(out, "a.txt\nb.txt\nrewound:a.txt\nclosed\ndone\n");
}

/// A registered wrapper that does not implement dir_opendir makes opendir()
/// return false (the matched-but-failed path) rather than a directory handle.
#[test]
fn test_opendir_wrapper_without_dir_opendir_returns_false() {
    let out = compile_and_run(
        r#"<?php
class NoDir {
    public $context;
    public function stream_open($path, $mode, $options, &$opened): bool {
        return true;
    }
}
stream_wrapper_register("ndir", "NoDir");
$dh = opendir("ndir://x");
if ($dh === false) {
    echo "false\n";
} else {
    echo "opened\n";
}
"#,
    );
    assert_eq!(out, "false\n");
}

/// chown()/chgrp() with an integer uid/gid on a registered userspace wrapper
/// dispatch to the wrapper's stream_metadata($path, STREAM_META_OWNER/GROUP,
/// $value) (vtable slot 14) instead of libc chown(2).
#[test]
fn test_chown_chgrp_int_wrapper_dispatch() {
    let out = compile_and_run(
        r#"<?php
class MetaWrapper {
    public $context;
    public function stream_metadata(string $path, int $option, mixed $value): bool {
        echo "meta:" . $option . ":" . $value . "\n";
        return true;
    }
}
stream_wrapper_register("metaw", "MetaWrapper");
$a = chown("metaw://x", 1000);
$b = chgrp("metaw://y", 2000);
echo ($a ? "ok" : "no") . "\n";
echo ($b ? "ok" : "no") . "\n";
"#,
    );
    assert_eq!(out, "meta:3:1000\nmeta:5:2000\nok\nok\n");
}

/// chown()/chgrp() with a STRING user/group name on a registered userspace wrapper
/// dispatch to stream_metadata($path, STREAM_META_OWNER_NAME/GROUP_NAME, $value)
/// (vtable slot 14) with the name boxed as a mixed value, instead of libc
/// getpwnam/getgrnam. A non-wrapper path keeps the libc name-resolving helpers.
#[test]
fn test_chown_chgrp_name_wrapper_dispatch() {
    let out = compile_and_run(
        r#"<?php
class NameWrapper {
    public $context;
    public function stream_metadata(string $path, int $option, mixed $value): bool {
        echo "meta:" . $option . ":" . $value . "\n";
        return true;
    }
}
stream_wrapper_register("namew", "NameWrapper");
$a = chown("namew://x", "www-data");
$b = chgrp("namew://y", "staff");
echo ($a ? "ok" : "no") . "\n";
echo ($b ? "ok" : "no") . "\n";
"#,
    );
    assert_eq!(out, "meta:2:www-data\nmeta:4:staff\nok\nok\n");
}

/// touch() on a registered userspace wrapper dispatches to
/// stream_metadata($path, STREAM_META_TOUCH, [mtime, atime]); the value is a
/// 2-element int array. A non-wrapper path keeps libc touch.
#[test]
fn test_touch_wrapper_dispatch() {
    let out = compile_and_run(
        r#"<?php
class TouchW {
    public $context;
    public function stream_metadata(string $path, int $option, mixed $value): bool {
        echo "opt=" . $option . " n=" . count($value) . " m=" . $value[0] . " a=" . $value[1] . "\n";
        return true;
    }
}
stream_wrapper_register("touchw", "TouchW");
$r = touch("touchw://f", 100, 200);
echo ($r ? "ok" : "no") . "\n";
"#,
    );
    assert_eq!(out, "opt=1 n=2 m=100 a=200\nok\n");
}

/// Regression: two `stream_context_create` calls in one program must
/// assemble. The no-options clear path previously used a fixed
/// `scc_store_zero` label that was defined twice (once per call), so any
/// program creating more than one context failed to assemble.
#[test]
fn test_stream_context_create_twice_assembles() {
    let out = compile_and_run(
        r#"<?php
$a = stream_context_create([]);
$b = stream_context_create([]);
echo "ok";
"#,
    );
    assert_eq!(out, "ok");
}

/// An explicitly supplied stream-context notifier fires STREAM_NOTIFY_CONNECT
/// (code 2) while opening a successful loopback HTTP stream.
#[test]
fn test_stream_notification_callback_fires_connect_for_explicit_context() {
    let (_server, port) = spawn_http_server(b"ok");
    let out = compile_and_run(
        &r#"<?php
$ctx = stream_context_create([], ['notification' => function($code, $sev, $msg, $mc, $bt, $bm) {
    if ($code === 2) echo "N" . $code . ";";
}]);
$f = fopen('http://127.0.0.1:PHP_TEST_PORT/', 'r', false, $ctx);
echo $f === false ? "closed" : "open";
fclose($f);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "N2;open");
}

/// v1 captures only a literal closure / first-class-callable `notification`
/// value. A string function-name callback is not a callable descriptor (no
/// invoker at offset 56), so it is not fired and the global is cleared
/// instead; the refused open must still complete without crashing.
#[test]
fn test_stream_notification_string_callback_not_fired_in_v1() {
    let out = compile_and_run(
        r#"<?php
function my_notify($code) { echo "S" . $code; }
$ctx = stream_context_create([], ['notification' => 'my_notify']);
$f = fopen('http://127.0.0.1:1/', 'r', false, $ctx);
echo $f === false ? "ok" : "bad";
"#,
    );
    assert_eq!(out, "ok");
}

/// An explicit empty context masks the request-default notification callback.
#[test]
fn test_stream_notification_empty_explicit_context_masks_default() {
    let (_server, port) = spawn_http_server(b"ok");
    let out = compile_and_run(
        &r#"<?php
$default = stream_context_get_default();
stream_context_set_params($default, ['notification' => function($code) {
    if ($code === 2) echo "default-fired";
}]);
$empty = stream_context_create([], ['other' => 1]);
$f = fopen('http://127.0.0.1:PHP_TEST_PORT/', 'r', false, $empty);
echo $f === false ? "bad" : "ok";
fclose($f);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "ok");
}

/// `stream_context_set_params` updates the explicitly addressed context notifier.
#[test]
fn test_stream_notification_callback_via_set_params() {
    let (_server, port) = spawn_http_server(b"ok");
    let out = compile_and_run(
        &r#"<?php
$ctx = stream_context_create([]);
stream_context_set_params($ctx, ['notification' => function($code) {
    if ($code === 2) echo "P" . $code . ";";
}]);
$f = fopen('http://127.0.0.1:PHP_TEST_PORT/', 'r', false, $ctx);
echo $f === false ? "closed" : "open";
fclose($f);
"#
        .replace("PHP_TEST_PORT", &port.to_string()),
    );
    assert_eq!(out, "P2;open");
}

/// A userspace wrapper whose `stream_cast()` (vtable slot 10) returns a real
/// underlying socket fd becomes select()-able: `stream_select` resolves the
/// synthetic wrapper fd to that real fd (STREAM_CAST_FOR_SELECT) and reports it
/// ready once data arrives. The wrapper connects to a same-process server
/// inside `stream_open`, and the server side writes to make it readable.
#[test]
fn test_stream_select_wrapper_stream_cast_detects_ready() {
    let out = compile_and_run(
        r#"<?php
class SockW {
    public $context;
    public $inner;
    public function stream_open($path, $mode, $options, &$opened): bool {
        $this->inner = stream_socket_client("tcp://127.0.0.1:55050");
        return $this->inner !== false;
    }
    public function stream_cast($cast_as) { return $this->inner; }
    public function stream_eof(): bool { return false; }
    public function stream_read(int $n): string { return ""; }
}
stream_wrapper_register("sockw", "SockW");
$srv = stream_socket_server("tcp://127.0.0.1:55050");
$w = fopen("sockw://x", "r");
$conn = stream_socket_accept($srv);
fwrite($conn, "ping");
$r = [$w]; $wr = []; $e = [];
$n = stream_select($r, $wr, $e, 1, 0);
echo "n=" . $n . " kept=" . count($r);
"#,
    );
    assert_eq!(out, "n=1 kept=1");
}

/// A resource keeps its PHP kind name when it travels through an untyped parameter.
///
/// `stream_context_create()` is statically `Resource`, so passing it to `mixed $r` boxes it
/// through the generic value boxer — which writes ownership marker 0. The registry lookup
/// only ran for markers 1/3/4/9, so a context answered `"stream"` while a filter (boxed by
/// the legacy fd path, marker 3) answered correctly. Same emitted code for both, so the
/// divergence was purely the marker. Oracle: php 8.5.6.
#[test]
fn test_resource_kind_name_survives_an_untyped_parameter() {
    let out = compile_and_run(
        r#"<?php
function kind($r) { return get_resource_type($r); }
function open_p($r) { return var_export(is_resource($r), true); }
$ctx = stream_context_create([]);
$f   = fopen("php://memory", "r+");
$fl  = stream_filter_append($f, "string.toupper", STREAM_FILTER_WRITE);
echo kind($ctx), "|", kind($fl), "|", kind($f), "|", open_p($ctx);
fclose($f);
echo "|", kind($f), "|", open_p($f);
"#,
    );
    assert_eq!(out, "stream-context|stream filter|stream|true|Unknown|false");
}

/// `stream_select()` must actually wait for its timeout.
///
/// The timeout arrives in caller-saved registers (x3/x4, rcx/r8) and the pollfd build
/// calls `__rt_stream_fd` for every entry, so the computed timeout was whatever those
/// registers happened to hold afterwards. On macOS that garbage rounded to zero and the
/// call returned instantly; on Linux it hit the "negative seconds means infinite" arm and
/// `poll(-1)` blocked forever, which is what timed this suite's wrapper test out at 60s.
/// The lower bound is deliberately loose — the bug produced 0 ms, not 190 ms.
#[test]
fn test_stream_select_waits_for_its_timeout() {
    let out = compile_and_run(
        r#"<?php
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, 0);
$r = [$pair[0]]; $w = []; $e = [];
$t0 = microtime(true);
$n = stream_select($r, $w, $e, 0, 200000);
$ms = (int) round((microtime(true) - $t0) * 1000);
echo "n=", var_export($n, true), " waited=", var_export($ms >= 150, true);
"#,
    );
    assert_eq!(out, "n=0 waited=true");
}

/// A userspace wrapper that does not implement `stream_cast` cannot be
/// represented as a select()-able descriptor, so `stream_select` excludes its
/// synthetic fd (matching PHP) and drops it from the ready set without crashing.
#[test]
fn test_stream_select_wrapper_without_stream_cast_excluded() {
    let out = compile_and_run(
        r#"<?php
class NoCast {
    public $context;
    public function stream_open($path, $mode, $options, &$opened): bool { return true; }
    public function stream_eof(): bool { return false; }
    public function stream_read(int $n): string { return ""; }
}
stream_wrapper_register("nocast", "NoCast");
$w = fopen("nocast://x", "r");
$r = [$w]; $wr = []; $e = [];
$n = stream_select($r, $wr, $e, 0, 0);
echo "n=" . $n . " kept=" . count($r);
"#,
    );
    assert_eq!(out, "n=0 kept=0");
}

/// Verifies `fread()` of a payload larger than the 64 KiB concat scratch buffer returns the whole
/// string AND leaves the stream-handle table intact, so the following `fclose()` still sees a
/// valid resource. Before the reservation fix the read ran past `_concat_buf` into the adjacent
/// BSS globals and `fclose()` failed with a bogus TypeError.
#[test]
fn test_fread_larger_than_concat_scratch_keeps_stream_table_intact() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("0123456789", 10000);
file_put_contents("big_fread.bin", $payload);
$f = fopen("big_fread.bin", "r");
$data = fread($f, 100000);
echo strlen($data), "|", substr($data, 0, 5), "|", substr($data, -5), "|";
echo ($data === $payload ? "same" : "DIFF"), "|";
echo (is_resource($f) ? "res" : "broken"), "|";
fclose($f);
unlink("big_fread.bin");
echo "closed";
"#,
    );
    assert_eq!(out, "100000|01234|56789|same|res|closed");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_get_contents()` drains a stream larger than the 64 KiB concat scratch buffer
/// into one contiguous, byte-exact result through the growable reservation.
#[test]
fn test_stream_get_contents_larger_than_concat_scratch() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("abcdefghij", 10000);
file_put_contents("big_sgc.bin", $payload);
$f = fopen("big_sgc.bin", "r");
$data = stream_get_contents($f);
fclose($f);
unlink("big_sgc.bin");
echo strlen($data), "|", ($data === $payload ? "same" : "DIFF");
"#,
    );
    assert_eq!(out, "100000|same");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the bounded `stream_get_contents($f, $length)` form also honours a cap larger than the
/// 64 KiB concat scratch buffer without overrunning it.
#[test]
fn test_stream_get_contents_bounded_larger_than_concat_scratch() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$payload = str_repeat("abcdefghij", 10000);
file_put_contents("big_sgc_b.bin", $payload);
$f = fopen("big_sgc_b.bin", "r");
$data = stream_get_contents($f, 70000);
fclose($f);
unlink("big_sgc_b.bin");
echo strlen($data), "|", ($data === substr($payload, 0, 70000) ? "same" : "DIFF");
"#,
    );
    assert_eq!(out, "70000|same");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `fgets()` returns a line longer than the 64 KiB concat scratch buffer intact: the line
/// accumulator grows into owned heap storage instead of writing past `_concat_buf`.
#[test]
fn test_fgets_line_larger_than_concat_scratch() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$w = fopen("big_line.txt", "w");
fwrite($w, "first\n");
fwrite($w, str_repeat("Z", 200000));
fwrite($w, "\nlast\n");
fclose($w);
$f = fopen("big_line.txt", "r");
$a = fgets($f);
$b = fgets($f);
$c = fgets($f);
fclose($f);
unlink("big_line.txt");
echo rtrim($a), "|", strlen($b), "|", substr($b, 0, 3), "|", rtrim($c);
"#,
    );
    assert_eq!(out, "first|200001|ZZZ|last");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_get_line()` honours a byte budget larger than the 64 KiB concat scratch
/// buffer, returning the full delimiter-stripped line from the reserved destination.
#[test]
fn test_stream_get_line_budget_larger_than_concat_scratch() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$long = str_repeat("Q", 150000);
file_put_contents("big_sgl.txt", $long . "|tail");
$f = fopen("big_sgl.txt", "r");
$a = stream_get_line($f, 200000, "|");
$b = stream_get_line($f, 200000, "|");
fclose($f);
unlink("big_sgl.txt");
echo strlen($a), "|", substr($a, 0, 3), "|", $b;
"#,
    );
    assert_eq!(out, "150000|QQQ|tail");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_get_meta_data()` reports the mode string the caller passed, not one derived
/// from the descriptor's access bits.
///
/// The derivation could only ever answer `r`, `w` or `r+`: it read `F_GETFL`, which knows nothing
/// of `a` (reported `w`), of `+` past a `b` flag, or of the `b` flag itself. A library that
/// branches on `$meta['mode'][0] === 'a'` to decide whether a handle appends saw `w` and rewound.
#[test]
fn test_stream_get_meta_data_reports_the_mode_the_caller_passed() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("modes.txt", "seed");
foreach (["r", "rb", "r+", "r+b", "w", "w+", "a", "a+", "c"] as $mode) {
    $h = fopen("modes.txt", $mode);
    echo stream_get_meta_data($h)["mode"], " ";
    fclose($h);
}
"#,
    );
    assert_eq!(out, "r rb r+ r+b w w+ a a+ c ");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the memory wrappers report the mode of the stream PHP built for them.
///
/// `php://memory` and `php://temp` do not echo the caller's mode: a read-only mode answers `rb`,
/// an append mode `a+b`, and anything asking for write access `w+b`. Reference PHP 8.5.6 was the
/// oracle for each of these.
#[test]
fn test_stream_get_meta_data_maps_the_memory_wrapper_modes() {
    let out = compile_and_run(
        r#"<?php
foreach (["r", "rb", "r+", "w", "w+", "a", "c"] as $mode) {
    $h = fopen("php://memory", $mode);
    echo stream_get_meta_data($h)["mode"], " ";
    fclose($h);
}
$t = fopen("php://temp", "r");
echo stream_get_meta_data($t)["mode"], " ";
fclose($t);
$o = fopen("php://output", "w");
echo stream_get_meta_data($o)["mode"];
"#,
    );
    assert_eq!(out, "rb rb w+b w+b w+b a+b rb rb wb");
}

/// Verifies repeated `stream_get_meta_data()` calls keep reporting the same URI.
///
/// The array releases its string values, so handing it the StreamState's own URI allocation freed
/// the state's copy. The first two calls still read the right bytes; by the third, the hash keys
/// of the arrays built in between had reused the block, and `uri` came back as a fragment of
/// `seekable` or `blocked`. The state's pointer was also left dangling for its own teardown.
#[test]
fn test_stream_get_meta_data_uri_survives_repeated_reads() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("uri_meta.txt", "seed");
$h = fopen("uri_meta.txt", "r");
echo stream_get_meta_data($h)["uri"], "|";
echo stream_get_meta_data($h)["uri"], "|";
echo stream_get_meta_data($h)["uri"], "|";
echo stream_get_meta_data($h)["uri"];
fclose($h);
"#,
    );
    assert_eq!(out, "uri_meta.txt|uri_meta.txt|uri_meta.txt|uri_meta.txt");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `c` opens a file for writing without truncating it, and creates it when absent.
///
/// The mode parser accepted only `r`, `w` and `a`, so `c` — which PHP added precisely to let a
/// caller take an advisory lock before deciding to truncate — returned `false` with a warning.
#[test]
fn test_fopen_c_mode_creates_without_truncating() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("c_mode.txt", "abcdef");
$h = fopen("c_mode.txt", "c");
fwrite($h, "XY");
fclose($h);
echo file_get_contents("c_mode.txt"), "|";
$fresh = fopen("c_mode_new.txt", "c");
echo ($fresh === false ? "false" : "resource");
fclose($fresh);
"#,
    );
    assert_eq!(out, "XYcdef|resource");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `x` creates a file exclusively and refuses one that already exists.
#[test]
fn test_fopen_x_mode_refuses_an_existing_file() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
$fresh = fopen("x_mode.txt", "x");
echo ($fresh === false ? "false" : "resource"), "|";
fwrite($fresh, "new");
fclose($fresh);
$again = @fopen("x_mode.txt", "x");
echo ($again === false ? "false" : "resource"), "|";
echo file_get_contents("x_mode.txt");
"#,
    );
    assert_eq!(out, "resource|false|new");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a `+` after the `b` flag still opens the file for both reading and writing.
///
/// The parser only inspected the second mode byte, so `rb+` — an idiom PHP accepts and the manual
/// spells out — stayed read-only and its writes failed silently.
#[test]
fn test_fopen_plus_is_honoured_after_the_b_flag() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("plus_after_b.txt", "abcdef");
$h = fopen("plus_after_b.txt", "rb+");
fwrite($h, "ZZ");
fclose($h);
echo file_get_contents("plus_after_b.txt");
"#,
    );
    assert_eq!(out, "ZZcdef");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `stream_socket_client()` warns when the connection is refused.
///
/// PHP raises this Warning whether or not the caller passed `&$errno`/`&$errstr`; elephc filled
/// the out-parameters and printed nothing, so a script that watched the warning to notice a dead
/// endpoint saw a silent `false`. Port 9 (discard) is not served on a CI host.
#[test]
fn test_stream_socket_client_warns_when_the_connection_is_refused() {
    let out = compile_and_run_capture(
        r#"<?php
$c = stream_socket_client("tcp://127.0.0.1:9");
echo ($c === false ? "false" : "resource");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    assert!(
        out.stderr
            .contains("Warning: stream_socket_client(): Unable to connect to tcp://127.0.0.1:9 ("),
        "expected PHP's connect warning, got stderr={}",
        out.stderr
    );
}

/// Verifies `@` suppresses the connect-failure warning, as it does every other PHP diagnostic.
#[test]
fn test_error_control_suppresses_the_connect_failure_warning() {
    let out = compile_and_run_capture(
        r#"<?php
$c = @stream_socket_client("tcp://127.0.0.1:9");
echo ($c === false ? "false" : "resource");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    assert_eq!(out.stderr, "");
}

/// Verifies `stream_get_meta_data()['stream_type']` names the wrapper, not the descriptor.
///
/// It was derived from whether `lseek` worked, which is not what php-src reports: a memory stream
/// came back as STDIO, `php://output` as a socket, and a `popen()` pipe as a socket too. The name
/// is a wrapper and backend identity, so it comes from the recorded identity now.
#[test]
fn test_stream_get_meta_data_names_the_wrapper_not_the_descriptor() {
    let (out, dir) = compile_and_run_in_dir(
        r#"<?php
file_put_contents("stype.txt", "seed");
$names = [];
$h = fopen("stype.txt", "r");
$names[] = stream_get_meta_data($h)["stream_type"];
fclose($h);
$h = fopen("php://memory", "r+");
$names[] = stream_get_meta_data($h)["stream_type"];
fclose($h);
$h = fopen("php://temp", "r+");
$names[] = stream_get_meta_data($h)["stream_type"];
fclose($h);
$h = fopen("php://output", "w");
$names[] = stream_get_meta_data($h)["stream_type"];
$h = fopen("php://input", "r");
$names[] = stream_get_meta_data($h)["stream_type"];
$h = fopen("data://text/plain,hi", "r");
$names[] = stream_get_meta_data($h)["stream_type"];
fclose($h);
$p = popen("printf hi", "r");
$names[] = stream_get_meta_data($p)["stream_type"];
pclose($p);
$d = opendir(".");
$names[] = stream_get_meta_data($d)["stream_type"];
closedir($d);
echo implode("|", $names);
"#,
    );
    assert_eq!(out, "STDIO|MEMORY|TEMP|Output|Input|RFC2397|STDIO|dir");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies each socket transport is named the way php-src names it.
///
/// A TCP, UDP, Unix-domain and paired socket are all non-seekable descriptors, so nothing about
/// them distinguishes the four names php-src gives them. The transport is recorded from the
/// address the caller wrote, and an accepted connection takes its listener's.
#[test]
fn test_stream_get_meta_data_names_each_socket_transport() {
    let out = compile_and_run(
        r#"<?php
$names = [];
$s = stream_socket_server("tcp://127.0.0.1:0");
$names[] = stream_get_meta_data($s)["stream_type"];
$c = stream_socket_client("tcp://" . stream_socket_get_name($s, false));
$names[] = stream_get_meta_data($c)["stream_type"];
$a = stream_socket_accept($s);
$names[] = stream_get_meta_data($a)["stream_type"];
fclose($a);
fclose($c);
fclose($s);
$u = stream_socket_server("udp://127.0.0.1:0", $e, $m, STREAM_SERVER_BIND);
$names[] = stream_get_meta_data($u)["stream_type"];
fclose($u);
$path = "/tmp/elephc_stype_transport.sock";
@unlink($path);
$x = stream_socket_server("unix://" . $path);
$names[] = stream_get_meta_data($x)["stream_type"];
fclose($x);
@unlink($path);
$pair = stream_socket_pair(STREAM_PF_UNIX, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
$names[] = stream_get_meta_data($pair[0])["stream_type"];
fclose($pair[0]);
fclose($pair[1]);
echo implode("|", $names);
"#,
    );
    assert_eq!(
        out,
        "tcp_socket/ssl|tcp_socket/ssl|tcp_socket/ssl|udp_socket|unix_socket|generic_socket"
    );
}

/// Verifies an unresolvable host produces the message php-src composes for it.
///
/// This failure has no `errno` — php-src builds the text itself, which is why `&$error_code` stays
/// `0` — so elephc, which only ever described an `errno`, left `&$error_message` empty and the
/// caller had nothing but `false` to go on. `.invalid` is reserved by RFC 2606 and never resolves.
#[test]
fn test_socket_error_outputs_describe_an_unresolvable_host() {
    let out = compile_and_run_capture(
        r#"<?php
$c = @stream_socket_client("tcp://no-such-host.invalid:80", $errno, $errstr);
echo ($c === false ? "false" : "resource"), "|", $errno, "|", $errstr;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert!(
        out.stdout.starts_with(
            "false|0|php_network_getaddresses: getaddrinfo for no-such-host.invalid failed: "
        ),
        "expected php-src's composed resolver message, got stdout={}",
        out.stdout
    );
}

/// Verifies an unresolvable host raises the two Warnings PHP raises, in PHP's order.
///
/// php-src reports the resolver's own message first, then the connect line that repeats it as the
/// reason.
#[test]
fn test_unresolvable_host_warns_twice_like_php() {
    let out = compile_and_run_capture(
        r#"<?php
$c = stream_socket_client("tcp://no-such-host.invalid:80");
echo ($c === false ? "false" : "resource");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    let lines: Vec<&str> = out.stderr.lines().collect();
    assert_eq!(lines.len(), 2, "expected two warnings, got stderr={}", out.stderr);
    assert!(
        lines[0].starts_with(
            "Warning: stream_socket_client(): php_network_getaddresses: getaddrinfo for \
             no-such-host.invalid failed: "
        ),
        "unexpected first warning: {}",
        lines[0]
    );
    assert!(
        lines[1].starts_with(
            "Warning: stream_socket_client(): Unable to connect to tcp://no-such-host.invalid:80 \
             (php_network_getaddresses: getaddrinfo for no-such-host.invalid failed: "
        ),
        "unexpected second warning: {}",
        lines[1]
    );
}

/// Verifies `fsockopen()` spells its refused endpoint the way PHP does, as `host:port`.
#[test]
fn test_fsockopen_warns_with_the_host_and_port() {
    let out = compile_and_run_capture(
        r#"<?php
$c = fsockopen("127.0.0.1", 9);
echo ($c === false ? "false" : "resource");
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "false");
    assert!(
        out.stderr
            .contains("Warning: fsockopen(): Unable to connect to 127.0.0.1:9 ("),
        "expected PHP's connect warning, got stderr={}",
        out.stderr
    );
}
