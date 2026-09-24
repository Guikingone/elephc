//! Purpose:
//! Regression tests for PHP diagnostics when float values become array keys.
//!
//! Called from:
//! - `cargo test` through the codegen integration harness.
//!
//! Key details:
//! - Covers typed and boxed keys, exact floats, and diagnostic dispatch.

use crate::support::*;

/// Checks fractional reads and writes warn with the original float value.
#[test]
fn test_float_array_key_fractional_read_and_write_warn() {
    let out = compile_and_run_capture("<?php $a = [1 => 'x']; echo $a[1.9]; $a[-0.9] = 'y'; echo $a[0];");
    assert_eq!(out.stdout, "xy");
    assert!(out.stderr.contains("Implicit conversion from float 1.9 to int loses precision"), "{}", out.stderr);
    assert!(out.stderr.contains("Implicit conversion from float -0.9 to int loses precision"), "{}", out.stderr);
}

/// A missing hash read reports one float-key deprecation before the undefined-key warning.
#[test]
fn test_float_array_key_missing_hash_read_warns_once() {
    let out = compile_and_run_capture("<?php $a = ['name' => 'x']; var_dump($a[1.9]);");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
    assert_eq!(out.stderr.matches("Undefined array key 1").count(), 1, "{}", out.stderr);
}

/// A boxed float key on a missing hash read emits one deprecation and one warning.
#[test]
fn test_float_array_key_mixed_missing_hash_read_warns_once() {
    let out = compile_and_run_capture("<?php $key = $argc > 0 ? 1.9 : 'one'; $a = ['name' => 'x']; var_dump($a[$key]);");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
    assert_eq!(out.stderr.matches("Undefined array key 1").count(), 1, "{}", out.stderr);
}

/// A missing hash entry inserted through a reference normalizes its float key once.
#[test]
fn test_float_array_key_missing_reference_warns_once() {
    let out = compile_and_run_capture("<?php $a = ['name' => 'x']; $r =& $a[1.9]; $r = 'y'; echo $a[1];");
    assert_eq!(out.stdout, "y");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// A nested write does not rediagnose the key while promoting its hash entry.
#[test]
fn test_float_array_key_nested_write_warns_once() {
    let out = compile_and_run_capture("<?php $a = [1 => ['name' => 1], 'other' => 's']; $a[1.9]['x'] = 2; echo $a[1]['x'];");
    assert_eq!(out.stdout, "2");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks array-literal insertion reports the float key before later reads.
#[test]
fn test_float_array_literal_key_warns() {
    let out = compile_and_run_capture("<?php $a = [1.9 => 'x']; echo $a[1];");
    assert_eq!(out.stdout, "x");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks integral float keys and suppressed reads remain silent.
#[test]
fn test_float_array_key_exact_and_suppressed_are_silent() {
    let out = compile_and_run_capture("<?php $a = [2 => 'x']; echo $a[2.0]; echo @$a[2.9];");
    assert_eq!(out.stdout, "xx");
    assert_eq!(out.stderr, "");
}

/// Checks a dynamically typed float key emits one complete deprecation.
#[test]
fn test_float_array_key_mixed_handler_receives_complete_message() {
    let out = compile_and_run_capture(r#"<?php
set_error_handler(function($level, $message) { echo $level, ':', $message, '|'; });
$key = $argc > 0 ? 1.9 : 'one';
$a = [1 => 'x'];
echo $a[$key];
"#);
    assert_eq!(out.stdout, "8192:Implicit conversion from float 1.9 to int loses precision|x");
    assert_eq!(out.stderr, "");
}

/// Checks dynamically typed writes diagnose the key before indexing the array.
#[test]
fn test_float_array_key_mixed_write_warns_once() {
    let out = compile_and_run_capture(r#"<?php
$key = $argc > 0 ? 1.9 : 'one';
$a = [];
$a[$key] = 'x';
echo $a[1];
"#);
    assert_eq!(out.stdout, "x");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks both list and hash existence probes apply PHP's float-key conversion.
#[test]
fn test_float_array_key_exists_warns() {
    let out = compile_and_run_capture(r#"<?php
$list = ['a', 'b'];
$hash = ['name' => 'n', 1 => 'x'];
echo array_key_exists(1.9, $list) ? 'L' : '?';
echo array_key_exists(1.9, $hash) ? 'H' : '?';
"#);
    assert_eq!(out.stdout, "LH");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 2, "{}", out.stderr);
}

/// Checks a boxed float key follows the same existence-probe diagnostics.
#[test]
fn test_float_array_key_exists_mixed_warns() {
    let out = compile_and_run_capture(r#"<?php
$key = $argc > 0 ? 1.9 : 'one';
$a = [1 => 'x'];
echo array_key_exists($key, $a) ? 'yes' : 'no';
"#);
    assert_eq!(out.stdout, "yes");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks NaN and infinity receive PHP's range warning and NaN's extra deprecation.
#[test]
fn test_float_array_key_nan_and_infinity_diagnostics() {
    let out = compile_and_run_capture("<?php $a = [0 => 'z']; echo $a[NAN], $a[INF];");
    assert_eq!(out.stdout, "zz");
    assert_eq!(out.stderr.matches("The float NAN is not representable as an int, cast occurred").count(), 1, "{}", out.stderr);
    assert_eq!(out.stderr.matches("Implicit conversion from float NAN to int loses precision").count(), 1, "{}", out.stderr);
    assert_eq!(out.stderr.matches("The float INF is not representable as an int, cast occurred").count(), 1, "{}", out.stderr);
}

/// Checks an existence probe diagnoses a float key even without a value read.
#[test]
fn test_float_array_key_isset_warns_once() {
    let out = compile_and_run_capture("<?php $a = [1 => 'x']; echo isset($a[1.9]) ? 'yes' : 'no';");
    assert_eq!(out.stdout, "yes");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks reference insertion converts its source float key only once.
#[test]
fn test_float_array_key_reference_write_warns_once() {
    let out = compile_and_run_capture("<?php $a = [1 => 'x']; $r =& $a[1.9]; $r = 'y'; echo $a[1];");
    assert_eq!(out.stdout, "y");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}

/// Checks iterator-to-array key normalization reports fractional generator keys.
#[test]
fn test_float_array_key_from_iterator_warns() {
    let out = compile_and_run_capture("<?php function g() { yield 1.9 => 'x'; } $a = iterator_to_array(g()); echo $a[1];");
    assert_eq!(out.stdout, "x");
    assert_eq!(out.stderr.matches("Implicit conversion from float 1.9 to int loses precision").count(), 1, "{}", out.stderr);
}
