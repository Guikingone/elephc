//! Purpose:
//! Regression tests for transient `_concat_buf` string results passed as call arguments.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A string builtin (`strtoupper`/`md5`/…) or a runtime concatenation materializes its
//!   result into the shared `_concat_buf` scratch and returns a borrowed slice. Before the
//!   fix, passing such a slice as a function/method/closure argument produced garbage: the
//!   callee's first statement reset `_concat_off` to 0 and the callee's own concats
//!   overwrote the caller's slice bytes. The fix has each frame reset `_concat_off` to the
//!   base it inherited from the caller (the high-water mark below which the caller's slice
//!   lives), so the callee appends above the caller's slice instead of clobbering it.
//!   These assertions fail on the pre-fix codegen (garbage output) and pass after it.

use super::*;

/// A `_concat_buf`-slice builtin result (`strtoupper`) passed to a user function must
/// survive the callee's per-statement concat reset and read back as the correct value.
#[test]
fn test_transient_builtin_string_arg_to_function() {
    let out = compile_and_run(
        r#"<?php function t($v){ return "[".$v."]"; } echo t(strtoupper("xy"));"#,
    );
    assert_eq!(out, "[XY]");
}

/// A runtime-concatenation result (`"ab".$d`) passed as an argument must survive the
/// callee's concat reset; this is the canonical minimal repro of the original bug.
#[test]
fn test_runtime_concat_string_arg_to_function() {
    let out = compile_and_run(
        r#"<?php function t($v){ return "[".$v."]"; } $d = "cd"; echo t("ab".$d);"#,
    );
    assert_eq!(out, "[abcd]");
}

/// The fix must cover method calls (vtable dispatch), not just plain functions: a
/// transient string argument to a method must survive the method body's concat reset.
#[test]
fn test_transient_string_arg_to_method() {
    let out = compile_and_run(
        r#"<?php class R { public function h(string $s): string { return "<".$s.">"; } } $r = new R(); echo $r->h(strtoupper("hi"));"#,
    );
    assert_eq!(out, "<HI>");
}

/// The fix must cover closures: a transient string argument to a closure must survive the
/// closure body's concat reset.
#[test]
fn test_transient_string_arg_to_closure() {
    let out = compile_and_run(
        r#"<?php $f = function($x){ return "{".$x."}"; }; echo $f(strtolower("ZZ"));"#,
    );
    assert_eq!(out, "{zz}");
}

/// Chained pass-through: `outer` preserves its inherited slice, then passes it to `inner`,
/// which preserves it again above `outer`'s region. Verifies the base is per-frame.
#[test]
fn test_transient_string_arg_chained_passthrough() {
    let out = compile_and_run(
        r#"<?php function inner($a){ return "(".$a.")"; } function outer($b){ return inner($b); } echo outer(strtoupper("qq"));"#,
    );
    assert_eq!(out, "(QQ)");
}

/// A longer `_concat_buf` builtin result (`md5`, 32 bytes) passed as an argument must be
/// read back intact, guarding against partial-overwrite variants of the bug.
#[test]
fn test_transient_md5_string_arg_to_function() {
    let out = compile_and_run(
        r#"<?php function t($v){ return "[".$v."]"; } echo t(md5("x"));"#,
    );
    assert_eq!(out, "[9dd4e461268c8034f5c8564e155c67a6]");
}

/// Two transient string arguments in one call must both survive: the callee's base sits
/// above both caller slices, so neither is clobbered.
#[test]
fn test_two_transient_string_args() {
    let out = compile_and_run(
        r#"<?php function j($a, $b){ return $a."-".$b; } echo j(strtoupper("ab"), strtolower("CD"));"#,
    );
    assert_eq!(out, "AB-cd");
}

// --- Issue #614: numeric stringification inside `__rt_implode`'s mixed loop ---
//
// End-to-end coverage for the CURSOR invariant documented in
// `src/codegen_support/runtime/strings/implode.rs`, fixed in a5a6ac6e4. `__rt_implode` keeps
// its write cursor in a register; a `mixed` element routes through `__rt_mixed_cast_string`,
// which stringifies floats with `__rt_ftoa` and ints with `__rt_itoa`, and both format into
// `_concat_buf` at whatever `_concat_off` holds. With the offset parked at the join's start,
// they overwrite the bytes already joined.
//
// The two directions corrupt differently, which is why the original report believed integers
// were immune: floats corrupt from byte 0 (`snprintf` writes forward from the offset), ints
// from byte 20 backwards (`__rt_itoa` writes right-to-left from `offset+20`), so an int only
// shows damage once the joined prefix reaches that window. The emitter's own unit tests pin
// the emitted instruction sequence; these pin the observable PHP behaviour, including that
// integer case.

/// Regression for issue #614: a float element after a string element must not overwrite the
/// bytes already joined. Pre-fix this printed `1.1.1`.
#[test]
fn test_regression_614_implode_float_after_string() {
    let out = compile_and_run(r#"<?php $r = ["x", 1.5]; echo implode(",", $r);"#);
    assert_eq!(out, "x,1.5");
}

/// Regression for issue #614: a float between two string elements must leave both intact.
#[test]
fn test_regression_614_implode_float_between_strings() {
    let out = compile_and_run(r#"<?php $r = ["x", 1.5, "y"]; echo implode(",", $r);"#);
    assert_eq!(out, "x,1.5,y");
}

/// Regression for issue #614: consecutive float elements each stringify into the shared
/// buffer, so two casts in a row must not clobber each other or the joined prefix.
/// (A homogeneous all-float array is a different, pre-existing defect — it never reaches
/// this mixed loop at all — so the array is kept heterogeneous here on purpose.)
#[test]
fn test_regression_614_implode_consecutive_floats() {
    let out = compile_and_run(r#"<?php $r = ["a", 1.5, 2.5]; echo implode(",", $r);"#);
    assert_eq!(out, "a,1.5,2.5");
}

/// Regression for issue #614: a leading float element was the one order that happened to
/// work before the fix (the cast wrote exactly where the cursor already pointed). Pinned so
/// the fix does not regress the accidental-pass case into a real failure.
#[test]
fn test_regression_614_implode_leading_float() {
    let out = compile_and_run(r#"<?php $r = [1.5, "x"]; echo implode(",", $r);"#);
    assert_eq!(out, "1.5,x");
}

/// Regression for issue #614: integers are corrupted by the same defect once the joined
/// prefix reaches `__rt_itoa`'s 21-byte scratch window. With 25 leading characters the digit
/// landed at byte 20 (`xxxxxxxxxxxxxxxxxxxx1xxxx,1`), so the int path needs its own guard.
#[test]
fn test_regression_614_implode_int_past_itoa_scratch_window() {
    let out = compile_and_run(
        r#"<?php $r = ["xxxxxxxxxxxxxxxxxxxxxxxxx", 1]; echo implode(",", $r);"#,
    );
    assert_eq!(out, "xxxxxxxxxxxxxxxxxxxxxxxxx,1");
}

/// Regression for issue #614: an interleaved join long enough to cross the scratch window in
/// several places must match PHP exactly, covering float and int casts in the same result.
#[test]
fn test_regression_614_implode_mixed_numeric_long_join() {
    let out = compile_and_run(
        r#"<?php $r = ["alpha-bravo-charlie", 1.5, "delta-echo-foxtrot", 42, 2.25]; echo implode("|", $r);"#,
    );
    assert_eq!(out, "alpha-bravo-charlie|1.5|delta-echo-foxtrot|42|2.25");
}

/// Regression for issue #614: the joined bytes must still be correct under heap debug, whose
/// allocation guards and poisoning shift the surrounding heap layout.
///
/// This asserts output only. The mixed-element join also leaks exactly one 48-byte block, but
/// that is a separate pre-existing defect: it reproduces identically with an `int` element
/// (`["x", 1]`) and on the pre-fix build, so it is not part of this corruption fix.
#[test]
fn test_regression_614_implode_float_after_string_under_heap_debug() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$r = ["x", 1.5];
echo implode(",", $r);
"#,
    );
    assert_eq!(out.stdout, "x,1.5");
}
