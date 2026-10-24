//! Purpose:
//! Exercises the complete PHP fileinfo surface, including native and eval ownership.
//!
//! Called from:
//! - The codegen integration harness, on every executable supported target.
//!
//! Key details:
//! - Requires the pinned managed libmagic artifact; CI refuses silent skips.

use crate::support::*;

/// Both PHP call styles share MIME, encoding and description modes, including binary bytes.
#[test]
fn test_fileinfo_buffer_flags_and_aliases() {
    if fileinfo_native::skip("test_fileinfo_buffer_flags_and_aliases") { return; }
    let out = compile_and_run(r#"<?php
$f = new \FiNfO(FILEINFO_MIME_TYPE);
echo finfo_buffer(finfo: $f, string: "hello"), "\n";
echo $f->buffer("hello", FILEINFO_MIME_ENCODING), "\n";
echo $f->buffer("hello"), "\n";
echo finfo_buffer($f, "hello", FILEINFO_MIME), "\n";
echo $f->buffer("\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x02\0\0\0"), "\n";
echo $f->buffer(""), "\n";
echo finfo_set_flags($f, FILEINFO_NONE) ? "set\n" : "bad\n";
echo $f->buffer("hello"), "\n";
echo finfo_close($f) ? "closed\n" : "bad\n";
echo $f->buffer("hello", FILEINFO_MIME_TYPE), "\n";
"#);
    assert_eq!(out, "text/plain\nus-ascii\ntext/plain\ntext/plain; charset=us-ascii\nimage/png\napplication/x-empty\nset\nASCII text, with no line terminators\nclosed\ntext/plain\n");
}

/// Files, stream URLs and resource inputs preserve PHP results and stream position.
#[test]
fn test_fileinfo_files_and_streams() {
    if fileinfo_native::skip("test_fileinfo_files_and_streams") { return; }
    let out = compile_and_run(r#"<?php
$f = finfo_open(FILEINFO_MIME_TYPE);
$p = tempnam(sys_get_temp_dir(), "finfo");
file_put_contents($p, "hello\n");
echo finfo_file($f, $p), "\n";
echo mime_content_type($p), "\n";
$s = fopen($p, "rb");
fseek($s, 3);
echo mime_content_type($s), ":", ftell($s), "\n";
fclose($s);
echo $f->file("data://text/plain;base64,aGVsbG8="), "\n";
echo $f->file(sys_get_temp_dir()), "\n";
echo $f->file("/dev/null"), "\n";
unlink($p);
"#);
    assert_eq!(out, "text/plain\ntext/plain\ntext/plain:3\ntext/plain\ndirectory\napplication/x-empty\n");
}

/// User databases support descriptions, MIME annotations, extensions and persistent options.
#[test]
fn test_fileinfo_custom_database() {
    if fileinfo_native::skip("test_fileinfo_custom_database") { return; }
    let out = compile_and_run(r#"<?php
$p = tempnam(sys_get_temp_dir(), "magic");
file_put_contents($p, "0 string ELEPHC Elephc document\n!:mime application/x-elephc\n!:ext elph\n");
$f = new finfo(FILEINFO_NONE, $p);
echo $f->buffer("ELEPHC\0body"), "\n";
echo $f->buffer("ELEPHC\0body", FILEINFO_MIME_TYPE), "\n";
echo $f->buffer("ELEPHC\0body", FILEINFO_EXTENSION), "\n";
echo $f->buffer("ELEPHC\0body"), "\n";
unlink($p);
"#);
    assert_eq!(out, "Elephc document\napplication/x-elephc\nelph\nElephc document\n");
}

/// Native failures and invalid object operations are recoverable without invalidating peers.
#[test]
fn test_fileinfo_failures_clone_and_serialization() {
    if fileinfo_native::skip("test_fileinfo_failures_clone_and_serialization") { return; }
    let out = compile_and_run(r#"<?php
$f = new finfo(FILEINFO_MIME_TYPE);
echo @$f->file("/elephc-fileinfo-missing/file") === false ? "missing\n" : "bad\n";
echo @finfo_open(0, "/elephc-fileinfo-missing/magic") === false ? "database\n" : "bad\n";
try { new finfo(0, "/elephc-fileinfo-missing/magic"); } catch (Exception $e) { echo "constructor\n"; }
try { $f->file(""); } catch (ValueError $e) { echo "empty\n"; }
try { $f->file("a\0b"); } catch (ValueError $e) { echo "nul\n"; }
try { $g = clone $f; } catch (Error $e) { echo "clone\n"; }
try { serialize($f); } catch (Exception $e) { echo "serialize\n"; }
echo $f->buffer("hello"), "\n";
"#);
    assert_eq!(out, "missing\ndatabase\nconstructor\nempty\nnul\nclone\nserialize\ntext/plain\n");
}

/// Runtime-unknown eval, named arguments and callable aliases use the host implementation.
#[test]
fn test_fileinfo_eval_bundle() {
    if fileinfo_native::skip("test_fileinfo_eval_bundle") { return; }
    let out = compile_and_run(r#"<?php
$f = new finfo(FILEINFO_MIME_TYPE);
$code = $argc > 0 ? '$x = finfo_open(flags: FILEINFO_MIME_TYPE); echo get_class($x), ":", finfo_buffer(string: "hello", finfo: $x), "\n"; $fn = "finfo_buffer"; echo $fn($x, "hello"), "\n"; echo $x->buffer("hello", FILEINFO_MIME_ENCODING), "\n"; finfo_close($x); echo $x->buffer("hello"), "\n";' : '';
eval($code);
"#);
    assert_eq!(out, "finfo:text/plain\ntext/plain\nus-ascii\ntext/plain\n");
}

/// Repeated object construction and destruction balance compiler heap ownership.
#[test]
fn test_fileinfo_heap_ownership() {
    if fileinfo_native::skip("test_fileinfo_heap_ownership") { return; }
    let out = compile_and_run_with_heap_debug(r#"<?php
for ($i = 0; $i < 10; $i++) {
    $f = new finfo(FILEINFO_MIME_TYPE);
    $s = $f->buffer("hello");
    $g = $f;
    finfo_close($g);
    unset($g);
    unset($f);
}
echo "ok\n";
"#);
    assert_eq!(out.stdout, "ok\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// All eleven fileinfo constants resolve identically without enabling the native bridge.
#[test]
fn test_fileinfo_constant_catalog() {
    let out = compile_and_run(r#"<?php
echo FILEINFO_NONE, ",", FILEINFO_SYMLINK, ",", FILEINFO_DEVICES, ",", FILEINFO_MIME_TYPE, ",", FILEINFO_CONTINUE, ",", FILEINFO_PRESERVE_ATIME, ",", FILEINFO_RAW, ",", FILEINFO_MIME_ENCODING, ",", FILEINFO_MIME, ",", FILEINFO_APPLE, ",", FILEINFO_EXTENSION;
"#);
    assert_eq!(out, "0,2,8,16,32,128,256,1024,1040,2048,16777216");
}
