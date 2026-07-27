//! Purpose:
//! Regression tests for issue #602: `count()` on a boxed `Mixed` receiver whose payload was the
//! null-container sentinel (produced by a missed array read forwarded through a ternary merge)
//! used to dereference the sentinel and segfault (SIGSEGV 139). The null-container
//! normalization introduced with issue #585 removed the crash by turning the sentinel into a
//! real null before `count()` ever sees it, so these tests lock the resulting behavior in place
//! rather than re-implementing a guard.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `$argc` keeps the ternary runtime-unknown so the merge (and the missed-read arm) survive
//!   AST folding; the tests run with no arguments, so the missed-read arm executes.
//! - A sentinel-carrying cell, a plain null pointer, and a boxed null cell (tag 8, e.g.
//!   `json_decode("null")`) all count as `0`, matching the codebase's off-web
//!   `count($_SERVER) == 0` convention.
//! - That quiet `0` is deliberately *not* PHP parity: PHP raises
//!   `count(): Argument #1 ($value) must be of type Countable|array, null given` for every null
//!   receiver. Closing that gap without breaking the off-web convention is tracked in issue
//!   #617; these tests document the divergence so a future parity change is a conscious one.

use crate::support::*;

/// Issue #602: the sentinel-carrying Mixed cell must not crash. Before the #585 normalization
/// this program died with SIGSEGV 139; it now reports the missed read and counts as `0`.
#[test]
fn test_mixed_count_null_container_sentinel_does_not_crash() {
    let out = compile_and_run_capture(
        r#"<?php
$rows = [[1, 2]];
$r = $argc == 1 ? $rows[5] : ["a", "b"];
echo count($r), "\n";
"#,
    );
    assert!(
        out.success,
        "the sentinel receiver must not crash count(), stdout={:?} stderr={:?}",
        out.stdout, out.stderr
    );
    assert_eq!(out.stdout, "0\n");
    assert!(
        out.stderr.contains("Warning: Undefined array key 5"),
        "missing the missed-read warning, stderr={:?}",
        out.stderr
    );
}

/// Issue #602 / #617 divergence: no `TypeError` is raised, so a `catch (TypeError)` around the
/// call never fires and execution simply continues. PHP would take the catch branch here. This
/// test exists to make that difference explicit and to fail loudly if the behavior is changed
/// without updating #617.
#[test]
fn test_mixed_count_null_container_sentinel_raises_no_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
$rows = [[1, 2]];
$r = $argc == 1 ? $rows[5] : ["a", "b"];
try {
    echo count($r), "\n";
} catch (TypeError $e) {
    echo "caught: " . $e->getMessage() . "\n";
}
echo "after\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(
        out.stdout, "0\nafter\n",
        "elephc counts a null receiver as 0; PHP raises a TypeError here (issue #617)"
    );
    assert!(
        out.stderr.contains("Warning: Undefined array key 5"),
        "missing the missed-read warning, stderr={:?}",
        out.stderr
    );
}

/// Issue #602: the sentinel path is leak-free under heap debug — the Mixed cell carrying the
/// missed read is released cleanly even though no container is ever materialized.
#[test]
fn test_mixed_count_null_container_sentinel_is_leak_free() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$rows = [[1, 2]];
$r = $argc == 1 ? $rows[5] : ["a", "b"];
echo count($r), "\n";
echo "after\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert!(
        out.stdout.contains("after"),
        "expected execution to continue past count(), stdout={:?}",
        out.stdout
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap on the sentinel count() path, stderr={}",
        out.stderr
    );
}

/// Issue #602 boundary: a Mixed cell holding a real null value (tag 8, from
/// `json_decode("null")`) counts as `0` on the same quiet path as the sentinel, so the two
/// forms cannot drift apart unnoticed.
#[test]
fn test_mixed_count_real_null_cell_is_legacy_zero() {
    let out = compile_and_run_capture(r#"<?php echo count(json_decode("null")), "\n";"#);
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "0\n");
}

/// Issue #602 control: when the same ternary merge delivers a real array, `count()` returns the
/// correct length — the missed-read handling must not disturb the populated path.
#[test]
fn test_mixed_count_real_array_via_ternary_merge() {
    let out = compile_and_run_capture(
        r#"<?php
$rows = [[1, 2, 3]];
$r = $argc == 1 ? $rows[0] : ["a", "b"];
echo count($r), "\n";
"#,
    );
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "3\n");
}

/// Issue #602 control: an empty container in a Mixed cell still counts as `0`, so a legitimate
/// zero count stays distinguishable from the missing-container path by the absence of a warning.
#[test]
fn test_mixed_count_empty_array_cell_is_zero() {
    let out = compile_and_run_capture(r#"<?php echo count(json_decode("[]")), "\n";"#);
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, "0\n");
    assert!(
        !out.stderr.contains("Warning"),
        "an empty array must not warn, stderr={:?}",
        out.stderr
    );
}
