//! Purpose:
//! End-to-end coverage for the allocation-free `(int)strpos()` sink.
//!
//! Called from:
//! - The codegen optimizer suite through `optimizer`.
//!
//! Key details:
//! - Matches, absent needles, positive/negative offsets and the empty needle compare
//!   against PHP; an out-of-range offset keeps the `ValueError`.
//! - The boxed `int|false` call stays in place when any observer other than the integer
//!   cast reads the result.

use crate::support::*;

/// Cast-observed byte offsets match PHP and stay heap-clean.
#[test]
fn strpos_int_sink_matches_php() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
echo (int)strpos('abcabc', 'b'), '|';
echo (int)strpos('abc', 'z'), '|';
echo (int)strpos('abcabc', 'b', 2), '|';
echo (int)strpos('abcabc', 'b', -3), '|';
echo (int)strpos('abc', ''), '|';
$s = str_repeat('a', 100) . 'needle';
$n = 0;
$iters = $argc + 64;
for ($i = 0; $i < $iters; $i++) { $n ^= (int)strpos($s, 'needle'); }
echo $n, '|';
try { echo (int)strpos('abc', 'a', 10); } catch (ValueError $e) { echo 'VE'; }
echo "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "1|0|4|4|0|100|VE\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// A strict-false comparison keeps the boxed `int|false` result.
#[test]
fn strpos_boxed_result_survives_false_comparison() {
    let out = compile_and_run(
        r#"<?php
$hit = strpos('abcabc', 'b');
$miss = strpos('abc', 'z');
echo $hit === false ? 'bad' : $hit, '|', $miss === false ? 'miss' : 'bad', "\n";
"#,
    );
    assert_eq!(out, "1|miss\n");
}

/// Disabling EIR optimization leaves the boxed call and produces the same output.
#[test]
fn strpos_int_sink_no_ir_opt_parity() {
    without_ir_opt(|| {
        let out = compile_and_run(
            r#"<?php
echo (int)strpos('abcabc', 'b'), '|', (int)strpos('abc', 'z'), '|';
echo (int)strpos('abcabc', 'b', -3), "\n";
"#,
        );
        assert_eq!(out, "1|0|4\n");
    });
}
