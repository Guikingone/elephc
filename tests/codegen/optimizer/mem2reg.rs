//! Purpose:
//! End-to-end regression coverage for scalar local promotion to EIR SSA.
//!
//! Called from:
//! - `cargo test --test codegen_tests optimizer::mem2reg`.
//!
//! Key details:
//! - Textual EIR proves local traffic becomes block parameters on joins and back edges.
//! - Native execution compares optimizer-on and optimizer-off PHP behavior.

use super::*;

/// Emits only the main function's textual EIR for one PHP program.
fn main_ir(source: &str, optimized: bool) -> String {
    let dir = make_cli_test_dir("elephc_mem2reg_ir");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write PHP fixture");
    let mut command = elephc_cli_command(&dir);
    command.arg("--emit-ir");
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let output = command.arg(&php_path).output().expect("emit EIR");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).expect("EIR is UTF-8");
    let main = text.split("  function main()").nth(1).expect("main function").to_string();
    fs::remove_dir_all(dir).expect("remove fixture directory");
    main
}

/// Compiles and executes one PHP program with the requested EIR optimization mode.
fn run_variant(source: &str, optimized: bool) -> String {
    run_variant_with_args(source, optimized, &[])
}

/// Compiles and executes one PHP program with the requested CLI arguments.
fn run_variant_with_args(source: &str, optimized: bool, args: &[&str]) -> String {
    let dir = make_cli_test_dir("elephc_mem2reg_run");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write PHP fixture");
    let mut command = elephc_cli_command(&dir);
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let compile = command.arg(&php_path).output().expect("compile PHP fixture");
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    let run = run_binary_with_args(&dir.join("main"), &dir, args);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let stdout = String::from_utf8(run.stdout).expect("stdout is UTF-8");
    fs::remove_dir_all(dir).expect("remove fixture directory");
    stdout
}

/// The example's counter and accumulator become loop parameters with back-edge values.
#[test]
fn test_mem2reg_promotes_loop_counter_and_accumulator() {
    let source = include_str!("../../../examples/mem2reg-loop/main.php");
    let plain = main_ir(source, false);
    let optimized = main_ir(source, true);
    for slot in ["slot[2]", "slot[3]"] {
        assert!(plain.contains(&format!("load_local {slot}")), "{plain}");
        assert!(!optimized.contains(&format!("load_local {slot}")), "{optimized}");
        assert!(!optimized.lines().any(|line| line.contains("store_local") && line.contains(slot)), "{optimized}");
    }
    assert!(optimized.contains("while.cond("), "loop header takes SSA values: {optimized}");
    assert!(optimized.contains("br bb1("), "back edge passes SSA values: {optimized}");
    assert_eq!(run_variant(source, false), "10");
    assert_eq!(run_variant(source, true), "10");
}

/// Distinct assignments on both sides of a branch merge through an EIR parameter.
#[test]
fn test_mem2reg_promotes_branch_join() {
    let source = "<?php $x = 0; if ($argc > 1) { $x = 11; } else { $x = 22; } echo $x; while ($x > 0) { $x = ($x - 1) & 255; } echo $x;";
    let optimized = main_ir(source, true);
    assert!(optimized.contains("if.merge("), "join takes an SSA parameter: {optimized}");
    for optimized in [false, true] {
        assert_eq!(run_variant_with_args(source, optimized, &[]), "220");
        assert_eq!(run_variant_with_args(source, optimized, &["extra"]), "110");
    }
}

/// Floating-point loop state is promoted without changing PHP's printed result.
#[test]
fn test_mem2reg_promotes_float_loop() {
    let source = "<?php $f = 0.5; $i = 0; while ($i < 3) { $f = $f + 0.25; $i++; } echo $f;";
    let plain = main_ir(source, false);
    let optimized = main_ir(source, true);
    assert!(plain.lines().any(|line| line.contains("php=float") && line.contains("load_local")), "{plain}");
    assert!(!optimized.lines().any(|line| line.contains("php=float") && line.contains("load_local")), "{optimized}");
    assert!(optimized.contains("while.cond("), "float state reaches the loop header: {optimized}");
    assert_eq!(run_variant(source, false), "1.25");
    assert_eq!(run_variant(source, true), "1.25");
}

/// A call-free bitwise loop keeps two scalar values live through its back edge.
#[test]
fn test_mem2reg_promotes_two_call_free_loop_values() {
    let source = "<?php $counter = $argc & 1; $acc = 0; do { $acc = $acc ^ ($counter | 4); $counter = $counter ^ 1; } while ($counter > 0); echo $acc;";
    let optimized = main_ir(source, true);
    assert!(optimized.contains("do.body(v"), "loop body takes both values as parameters: {optimized}");
    assert!(!optimized.lines().any(|line| line.contains("load_local slot[1]") || line.contains("load_local slot[2]")), "{optimized}");
    assert!(!optimized.contains("store_local"), "{optimized}");
    assert_eq!(run_variant(source, false), "5");
    assert_eq!(run_variant(source, true), "5");
}

/// By-reference mutation remains visible after the call in both optimizer modes.
#[test]
fn test_mem2reg_preserves_by_reference_local() {
    let source = "<?php function change(int &$x): void { $x = 7; } $x = 2; change($x); echo $x;";
    assert_eq!(run_variant(source, false), "7");
    assert_eq!(run_variant(source, true), "7");
}

/// Dynamic eval sees a scalar written after its call site on later loop iterations.
#[test]
fn test_mem2reg_keeps_dynamic_eval_visible_local_in_memory() {
    let source = r#"<?php
function repeat_eval(string $code): void {
    for ($i = 0; $i < 3; $i++) {
        if ($i > 0) { eval($code); }
        $n = 7;
        if (strlen($code) > 1000) { $n = 9; }
        echo "step\n";
    }
}
repeat_eval('echo "n=$n\\n";');
"#;
    let expected = "step\nn=7\nstep\nn=7\nstep\n";
    assert_eq!(run_variant(source, false), expected);
    assert_eq!(run_variant(source, true), expected);
}

/// Dead-store and immutable-load passes also retain a single later eval-visible store.
#[test]
fn test_mem2reg_keeps_single_store_visible_to_dynamic_eval() {
    let source = r#"<?php
function repeat_eval(string $code): void {
    for ($i = 0; $i < 3; $i++) {
        if ($i > 0) { eval($code); }
        $n = 7;
        echo "step\n";
    }
}
repeat_eval('echo "n=$n\\n";');
"#;
    let expected = "step\nn=7\nstep\nn=7\nstep\n";
    assert_eq!(run_variant(source, false), expected);
    assert_eq!(run_variant(source, true), expected);
}

/// A scalar defined before a loop remains promotable when its back edge has no store.
#[test]
fn test_mem2reg_promotes_read_only_loop_value() {
    let source = "<?php $x = $argc; if ($argc > 5) { $x = 3; } $i = 0; $s = 0; while ($i < 3) { $s = ($s + $x) & 255; $i++; } echo $s;";
    let optimized = main_ir(source, true);
    assert!(!optimized.contains("load_local slot[1]"), "loop-invariant scalar is promoted: {optimized}");
    assert_eq!(run_variant(source, false), "3");
    assert_eq!(run_variant(source, true), "3");
}

/// A promoted loop value survives runtime string work before its next use.
#[test]
fn test_mem2reg_preserves_value_across_runtime_call() {
    let source = "<?php $x = $argc; if ($argc > 5) { $x = 3; } $sum = 0; $i = 0; while ($i < 3) { echo strlen('z' . $argc); $sum = ($sum + $x) & 255; $i++; } echo $sum;";
    let optimized = main_ir(source, true);
    assert!(!optimized.contains("load_local slot[1]"), "loop value is promoted: {optimized}");
    assert_eq!(run_variant(source, false), "2223");
    assert_eq!(run_variant(source, true), "2223");
}

/// Loop-carried scalar swaps preserve parallel edge-copy semantics.
#[test]
fn test_mem2reg_preserves_swap_across_back_edge() {
    let source = "<?php $a = $argc; $b = 2; $i = 0; while ($i < 3) { $t = $a; $a = $b; $b = $t; $i++; } echo $a, ':', $b;";
    let optimized = main_ir(source, true);
    assert!(optimized.contains("while.cond("), "swap uses loop parameters: {optimized}");
    assert_eq!(run_variant(source, false), "2:1");
    assert_eq!(run_variant(source, true), "2:1");
}
