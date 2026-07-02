//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of the `declare(...)`
//! statement, including its no-op statement form and its body-preserving block form.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `declare` directives (`strict_types`, `ticks`, `encoding`) are semantic no-ops in elephc:
//!   the statement form emits nothing and the block form runs its body normally. Outputs are
//!   cross-checked against `php` behavior.

use crate::support::*;

/// Verifies `declare(strict_types=1);` is a no-op: the statement itself emits nothing and
/// the following code runs normally. Matches `php -r 'declare(strict_types=1); echo 1+2;'`.
#[test]
fn test_declare_strict_types_is_noop() {
    let out = compile_and_run("<?php declare(strict_types=1); echo 1 + 2;");
    assert_eq!(out, "3");
}

/// Verifies `declare(ticks=1);` is a no-op: tick handlers are not supported, so the directive
/// is discarded and the following statement still runs.
#[test]
fn test_declare_ticks_is_noop() {
    let out = compile_and_run("<?php declare(ticks=1); echo \"hi\";");
    assert_eq!(out, "hi");
}

/// Verifies multiple comma-separated directives (`declare(strict_types=1, ticks=1);`) parse
/// and run as a single no-op, rather than being misread as assignment expressions.
#[test]
fn test_declare_multiple_directives_is_noop() {
    let out = compile_and_run("<?php declare(strict_types=1, ticks=1); echo \"ok\";");
    assert_eq!(out, "ok");
}

/// Verifies the block form `declare(ticks=1) { ... }` keeps and runs its body: the directive
/// list is discarded, but the block's statements execute like an ordinary block.
#[test]
fn test_declare_block_form_runs_body() {
    let out = compile_and_run("<?php declare(ticks=1) { echo \"hi\"; }");
    assert_eq!(out, "hi");
}

/// Verifies code that runs after a `declare(...);` statement (not just adjacent to it) still
/// executes, confirming the no-op splices cleanly into the surrounding statement list.
#[test]
fn test_declare_followed_by_multiple_statements() {
    let out = compile_and_run(
        r#"<?php
declare(strict_types=1);
$x = 5;
$y = 7;
echo $x + $y;
"#,
    );
    assert_eq!(out, "12");
}
