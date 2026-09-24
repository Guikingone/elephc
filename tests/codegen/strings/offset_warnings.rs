//! Purpose:
//! Regression tests for diagnostics on out-of-bounds string offset reads.
//!
//! Called from:
//! - `cargo test` through the codegen integration harness.
//!
//! Key details:
//! - Reads warn once; suppressed reads and in-bounds reads remain silent.

use crate::support::*;

/// Checks both signs retain the original offset in PHP's warning text.
#[test]
fn test_string_offset_out_of_bounds_warnings() {
    let out = compile_and_run_capture("<?php var_dump('abc'[99]); var_dump('abc'[-4]);");
    assert_eq!(out.stdout, "string(0) \"\"\nstring(0) \"\"\n");
    assert!(out.stderr.contains("Uninitialized string offset 99"), "{}", out.stderr);
    assert!(out.stderr.contains("Uninitialized string offset -4"), "{}", out.stderr);
}

/// Checks normal and suppressed string indexing keep their existing results.
#[test]
fn test_string_offset_warning_suppression_and_in_bounds() {
    let out = compile_and_run_capture("<?php echo 'abc'[1]; echo @'abc'[99];");
    assert_eq!(out.stdout, "b");
    assert_eq!(out.stderr, "");
}

/// Checks a warning handler receives one complete warning per failed read.
#[test]
fn test_string_offset_warning_handler_receives_complete_message() {
    let out = compile_and_run_capture(r#"<?php
set_error_handler(function($level, $message) { echo $level, ':', $message, '|'; });
$s = 'abc';
$unused = $s[99];
"#);
    assert_eq!(out.stdout, "2:Uninitialized string offset 99|");
    assert_eq!(out.stderr, "");
}

/// Checks an existence probe stays quiet for a missing string offset.
#[test]
fn test_string_offset_silent_probes() {
    let out = compile_and_run_capture("<?php $s = 'abc'; echo isset($s[99]) ? 'bad' : 'ok';");
    assert_eq!(out.stdout, "ok");
    assert_eq!(out.stderr, "");
}
