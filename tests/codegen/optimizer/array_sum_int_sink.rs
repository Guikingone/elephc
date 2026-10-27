//! Purpose:
//! End-to-end coverage for the allocation-free `array_sum()` int sink.
//!
//! Called from:
//! - The codegen optimizer suite through `optimizer`.
//!
//! Key details:
//! - Runtime-length packed int arrays keep the SIMD gate available; negative values and
//!   overflow-promoted sums exercise the raw scalar/float fallbacks.
//! - Float and associative receivers stay on the boxed aggregate path.

use crate::support::*;

/// Cast-observed packed int sums return the same values with and without heap boxing.
#[test]
fn array_sum_int_sink_matches_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = range(1, $argc + 100);
$n = 0;
for ($i = 0; $i < 2001; $i++) { $n ^= array_sum($a); }
echo $n, '|';
$negative = [-3, 7, -11, 5];
echo array_sum($negative) ^ 0, '|';
$wide = [4294967296, 1, 2, 3, 4, 5, 6, 7, 8];
echo array_sum($wide) ^ 0, '|';
$float = [1.5, 2.5];
echo array_sum($float) ^ 0, '|';
$hash = ['a' => 1, 'b' => 2];
echo array_sum($hash) ^ 0, "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "5151|-2|4294967332|4|3\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Overflow-promoted sums keep PHP's warned float-to-int cast behavior.
#[test]
fn array_sum_int_sink_preserves_float_cast_warning() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$overflow = [PHP_INT_MAX, PHP_INT_MAX];
$s = 0;
for ($i = 0; $i < 3; $i++) { $s ^= array_sum($overflow); }
echo $s, "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "0\n");
    assert!(
        out.stderr
            .contains("The float 1.8446744073709552E+19 is not representable as an int"),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Disabling EIR optimization leaves the boxed call and must produce the same output.
#[test]
fn array_sum_int_sink_no_ir_opt_parity() {
    without_ir_opt(|| {
        let out = compile_and_run(
            r#"<?php
$a = range(1, $argc + 100);
$n = 0;
for ($i = 0; $i < 2001; $i++) { $n ^= array_sum($a); }
$negative = [-3, 7, -11, 5];
echo $n, '|', array_sum($negative) ^ 0, "\n";
"#,
        );
        assert_eq!(out, "5151|-2\n");
    });
}

/// The declared-int preflight branch keeps the same output with EIR optimization disabled.
#[test]
fn typed_map_sum_preflight_no_ir_opt_parity() {
    without_ir_opt(|| {
        let out = compile_and_run(
            r#"<?php
$ok = range(1, $argc + 100);
$n = 0;
for ($i = 0; $i < 2001; $i++) { $n ^= array_sum(array_map(fn(int $x): int => $x + 1, $ok)); }
echo $n, "\n";
"#,
        );
        assert_eq!(out, "5252\n");
    });
}
