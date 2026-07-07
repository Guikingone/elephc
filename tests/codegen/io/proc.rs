//! Purpose:
//! Integration tests for the `proc_open`/`proc_close` builtins (C1a surface).
//!
//! Called from:
//! - `cargo test` through the codegen test harness, via `tests/codegen/io.rs`.
//!
//! Key details:
//! - C1a ships runtime stubs returning -1, so `proc_open` boxes as PHP false.
//! - `proc_close` run behavior is validated in C1b; here it is compile-verified only.

use super::*;

/// Verifies proc_open compiles, links, and boxes the C1a stub result as false.
#[test]
fn test_proc_open_stub_returns_false() {
    let out = compile_and_run(
        r#"<?php
$pipes = [];
$r = proc_open([], "echo hi", $pipes);
if ($r === false) {
    echo "false";
} else {
    echo "resource";
}
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies a full 3-arg proc_open with a descriptor spec lowers and links, and
/// the C1a stub still boxes the result as PHP false.
#[test]
fn test_proc_open_compile_only_with_pipes() {
    let out = compile_and_run(
        r#"<?php
$pipes = [];
$r = proc_open([0 => ["pipe", "r"], 1 => ["pipe", "w"]], "echo hi", $pipes);
echo $r === false ? "false" : "resource";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies proc_close compiles, links, and runs the resource type check against
/// the C1a stub result. Because the `proc_open` stub returns `false`, `proc_close`
/// raises a PHP `TypeError` at runtime (matching PHP's own behavior). This proves
/// the lowering links and the resource-guard path fires; the success path is
/// exercised in C1b once the real runtime lands.
#[test]
fn test_proc_close_compile_only() {
    let stderr = compile_and_run_expect_failure(
        r#"<?php
$pipes = [];
$r = proc_open([], "echo hi", $pipes);
proc_close($r);
"#,
    );
    assert!(stderr.contains("proc_close"), "stderr was: {}", stderr);
}