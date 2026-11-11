//! Purpose:
//! Integration tests for the EIR common-subexpression elimination pass (`cse`)
//! driven by the fixed-point pass driver. Fixtures use a local seeded from the
//! runtime `$argc` value so the repeated subexpressions are not constant-folded
//! away at the AST level and reach EIR, where peephole load forwarding collapses
//! the repeated reads to one value and CSE then deduplicates the identical pure
//! computations built on it.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - `$argc` is 1 with no CLI arguments, so each fixture's expected output is the
//!   value the un-deduplicated program would also compute — CSE must be behavior
//!   preserving.

use super::*;

/// A repeated `$n + 1` subexpression is computed once: `($n + 1) * ($n + 1)` with
/// `$n == 1` is `4`.
#[test]
fn test_cse_repeated_arithmetic() {
    let out = compile_and_run("<?php $n = $argc; echo ($n + 1) * ($n + 1);");
    assert_eq!(out, "4");
}

/// A larger common subexpression `$n * 3 + 7` is shared across the addition: with
/// `$n == 1` the result is `(3 + 7) + (3 + 7) == 20`.
#[test]
fn test_cse_larger_common_subexpression() {
    let out = compile_and_run("<?php $n = $argc; echo ($n * 3 + 7) + ($n * 3 + 7);");
    assert_eq!(out, "20");
}

/// A repeated comparison `$n > 5` is computed once and reused by both the
/// assignment and the conditional; with `$n == 1` the branch is the false arm.
#[test]
fn test_cse_repeated_comparison() {
    let out = compile_and_run("<?php $n = $argc; $x = $n > 5; echo ($n > 5) ? \"a\" : \"b\";");
    assert_eq!(out, "b");
}

/// CSE must not change behavior when the operands differ: `($n + 1) * ($n + 2)`
/// has two distinct subexpressions and stays `2 * 3 == 6` for `$n == 1`.
#[test]
fn test_cse_distinct_subexpressions_preserved() {
    let out = compile_and_run("<?php $n = $argc; echo ($n + 1) * ($n + 2);");
    assert_eq!(out, "6");
}

/// Structural proof that CSE actually deduplicates `($n + 1) * ($n + 1)` (the
/// previous output-only tests passed even when it did not fire): the constant `1`
/// operands are unified by value, so `main` contains exactly one `iadd` with ir-opt
/// on (two without). `$n = $argc` is bounded (argv holds at most `i32::MAX` entries),
/// so `crate::int_range` proves `$n + 1` overflow-free and it lowers to a pure `iadd`.
/// An unbounded operand keeps the checked, allocating `ichecked_add`, which CSE must not
/// collapse. Guards the constant-operand canonicalization against regression.
#[test]
fn test_cse_collapses_constant_operand_subexpression() {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    let dir = std::env::temp_dir().join(format!(
        "elephc_cse_struct_{}_{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let elephc = elephc_cli_bin();

    let count_adds = |source: &str, name: &str, extra: &[&str]| -> (usize, usize) {
        let php_path: PathBuf = dir.join(name);
        fs::write(&php_path, source).unwrap();
        let mut cmd = Command::new(&elephc);
        cmd.arg("--emit-ir");
        cmd.args(extra);
        cmd.arg(&php_path);
        let out = cmd.output().expect("emit-ir");
        assert!(out.status.success(), "emit-ir failed");
        let text = String::from_utf8_lossy(&out.stdout);
        let main_body = text.split("function main(").nth(1).expect("main present");
        let main_body = main_body.split("\n  function ").next().unwrap_or(main_body);
        (
            main_body.matches("= iadd ").count(),
            main_body.matches("= ichecked_add ").count(),
        )
    };

    // Proven overflow-free: pure `iadd`, collapsed by CSE.
    let bounded = "<?php $n = $argc; echo ($n + 1) * ($n + 1);\n";
    assert_eq!(count_adds(bounded, "bounded.php", &["--no-ir-opt"]), (2, 0), "unoptimized keeps both adds");
    assert_eq!(count_adds(bounded, "bounded.php", &[]), (1, 0), "CSE collapses the proven pure add");

    // Unbounded operand: `ichecked_add` allocates a Mixed box (not pure), so CSE keeps both.
    let unbounded = "<?php $n = intval($argv[0]); echo ($n + 1) * ($n + 1);\n";
    assert_eq!(count_adds(unbounded, "unbounded.php", &["--no-ir-opt"]), (0, 2), "unoptimized keeps both checked adds");
    assert_eq!(count_adds(unbounded, "unbounded.php", &[]), (0, 2), "CSE cannot collapse checked adds (not pure)");
}
