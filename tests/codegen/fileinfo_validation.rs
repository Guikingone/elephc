//! Purpose:
//! Checks fileinfo argument diagnostics and malformed runtime stream URL handling.
//!
//! Called from:
//! - The codegen integration harness on every executable supported target.
//!
//! Key details:
//! - PHP 8.4 and 8.5 differ for finfo filenames, but share database and MIME errors.
//! - Runtime-selected eval exercises the same prelude through Magician forwarding.

use crate::support::*;

/// Argument errors retain their public function names and the selected PHP exception class.
#[test]
fn test_fileinfo_php_profile_argument_errors() {
    if fileinfo_native::skip("test_fileinfo_php_profile_argument_errors") { return; }
    let source = r#"<?php
$f = new finfo(FILEINFO_MIME_TYPE);
try { mime_content_type("a\0b"); } catch (TypeError $e) { echo "mime-nul:", $e->getMessage(), "\n"; }
try { mime_content_type(""); } catch (ValueError $e) { echo "mime-empty:", $e->getMessage(), "\n"; }
try { $f->file("a\0b"); } catch (TypeError $e) { echo "type\n"; } catch (ValueError $e) { echo "value\n"; }
try { finfo_file($f, "a\0b"); } catch (TypeError $e) { echo "type:", $e->getMessage(), "\n"; } catch (ValueError $e) { echo "value:", $e->getMessage(), "\n"; }
try { finfo_file($f, ""); } catch (ValueError $e) { echo "empty:", $e->getMessage(), "\n"; }
try { new finfo(0, "a\0b"); } catch (ValueError $e) { echo "database-value\n"; }
try { finfo_open(0, "a\0b"); } catch (ValueError $e) { echo "open-value\n"; }
echo $f->buffer("hello"), "\n";
"#;
    for (profile, class) in [(elephc::php_version::PhpVersion::Php84, "type"), (elephc::php_version::PhpVersion::Php85, "value")] {
        assert_eq!(compile_and_run_with_php_version(source, profile), format!(
            "mime-nul:mime_content_type(): Argument #1 ($filename) must not contain any null bytes\nmime-empty:mime_content_type(): Argument #1 ($filename) must not be empty\n{class}\n{class}:finfo_file(): Argument #2 ($filename) must not contain any null bytes\nempty:finfo_file(): Argument #2 ($filename) must not be empty\ndatabase-value\nopen-value\ntext/plain\n"
        ));
    }
}

/// Invalid data metadata warns and returns false without changing accepted payload decoding.
#[test]
fn test_fileinfo_data_url_metadata_validation() {
    if fileinfo_native::skip("test_fileinfo_data_url_metadata_validation") { return; }
    let out = compile_and_run(r#"<?php
$f = new finfo(FILEINFO_MIME_TYPE);
$warnings = 0;
set_error_handler(function(int $level, string $message) use (&$warnings): bool { if ($level === E_WARNING) { $warnings++; } return true; });
foreach (["data://foo,HELLO", "data://text/plain;BASE64,SGVsbG8=", "data://;charset=UTF-8,HELLO", "data://text/plain;broken,HELLO", "data://text/plain;base64;x=y,SGVsbG8=", "data://text/plain;,HELLO"] as $url) {
    echo finfo_file($f, $url) === false ? "rejected\n" : "bad\n";
}
echo $warnings === 6 ? "warned\n" : "bad\n";
foreach (["data://,HELLO", "data://;base64,SEVMTE8=", "data://text/plain;charset=UTF-8;base64,SEVMTE8=", "data://text/plain;token=a=b,HELLO", "data://text/plain;base64=ordinary,HELLO"] as $url) {
    echo $f->file($url), "\n";
}
$code = $argc > 0 ? 'echo finfo_file($f, "data://foo,HELLO") === false ? "eval-rejected\n" : "bad\n"; echo $f->file("data://text/plain;BASE64,SGVsbG8=") === false ? "eval-rejected\n" : "bad\n"; echo $f->file("data://;base64,SEVMTE8="), "\n";' : '';
eval($code);
restore_error_handler();
"#);
    assert_eq!(out, "rejected\n".repeat(6) + "warned\n" + &"text/plain\n".repeat(5) + "eval-rejected\neval-rejected\ntext/plain\n");
}

/// Filter resource errors throw, while read and write selectors ignore ASCII case.
#[test]
fn test_fileinfo_filter_url_validation() {
    if fileinfo_native::skip("test_fileinfo_filter_url_validation") { return; }
    let out = compile_and_run(r#"<?php
$db = tempnam(sys_get_temp_dir(), "magic");
file_put_contents($db, "0 string HELLO filtered-document\n");
$f = new finfo(FILEINFO_NONE, $db);
try { finfo_file($f, "php://filter/read=string.rot13"); echo "bad\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; }
echo $f->file("php://filter/READ=string.rot13/resource=data://text/plain,URYYB"), "\n";
echo $f->file("php://filter/WrItE=string.rot13/resource=data://text/plain,HELLO"), "\n";
$code = $argc > 0 ? 'try { $f->file("php://filter/read=string.rot13"); echo "bad\n"; } catch (Error $e) { echo $e->getMessage(), "\n"; } echo finfo_file($f, "php://filter/ReAd=string.rot13/resource=data://text/plain,URYYB"), "\n";' : '';
eval($code);
unlink($db);
"#);
    assert_eq!(out, "No URL resource specified\nfiltered-document\nfiltered-document\nNo URL resource specified\nfiltered-document\n");
}
