//! Purpose:
//! Regression tests for a general EIR function-prologue bug: an early parameter
//! whose local storage widens to `Mixed` was boxed with `__rt_mixed_from_value`
//! *inside* the per-parameter materialization loop, before the following
//! parameters (still live in argument registers) had been spilled to their frame
//! slots. The boxing helper call clobbers the caller-saved argument registers, so
//! every later register parameter (including by-reference ref-cell pointers) was
//! read back as garbage.
//!
//! This surfaced compiling symfony/yaml: `Inline::evaluateScalar(ParserState,
//! string $scalar, int $flags, array &$references, ?bool &$isQuotedString)` boxed
//! its `string $scalar` parameter and then stored the clobbered x4/x5/x6 as
//! `$flags`, `&$references`, and `&$isQuotedString`, making the by-reference
//! pointer `1` and crashing on the first dereference.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Fix: `emit_function_prologue_with_label` spills every incoming parameter to
//!   its frame slot first, then boxes the widened parameters in a second pass, so
//!   the clobbering helper call runs only after all argument registers are saved.
//! - The `if ($s === "zzz")` branch is what widens `$s` (declared `string`) to
//!   `Mixed` local storage and triggers the in-prologue boxing; it never runs at
//!   runtime, so `$s` stays a string.

use crate::support::*;

/// Register parameters following a boxed (Mixed-widened) leading parameter keep
/// their values instead of being clobbered by the prologue boxing helper call.
#[test]
fn test_prologue_boxed_leading_param_preserves_later_register_params() {
    let out = compile_and_run(
        r#"<?php
function f(string $s, int $a, int $b, int $c, int $d, int $e): string {
    if ($s === "zzz") { $s = 42; }
    return "$a-$b-$c-$d-$e";
}
echo f("hi", 10, 20, 30, 40, 50);
"#,
    );
    assert_eq!(out, "10-20-30-40-50");
}

/// A by-reference parameter that follows a boxed leading parameter is passed a
/// valid ref-cell pointer (not a clobbered register): the callee both reads its
/// preceding register argument and writes back through the reference correctly.
#[test]
fn test_prologue_boxed_leading_param_preserves_following_by_ref() {
    let out = compile_and_run(
        r#"<?php
function g(string $s, int $a, ?bool &$q): string {
    if ($s === "zzz") { $s = 42; }
    $q = true;
    return "$a:" . ($q ? "T" : "F");
}
function drive(?bool $seed): string {
    $flag = $seed;
    $r = g("hi", 7, $flag);
    return $r . "|outer=" . ($flag === true ? "T" : "F");
}
echo drive(null);
"#,
    );
    assert_eq!(out, "7:T|outer=T");
}
