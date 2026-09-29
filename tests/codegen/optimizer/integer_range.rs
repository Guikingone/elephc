//! Purpose:
//! End-to-end coverage for EIR integer range and induction-variable analysis.
//!
//! Called from:
//! - `cargo test --test codegen_tests optimizer::integer_range`.
//!
//! Key details:
//! - Textual EIR proves bounded checked operations become scalar while unproven overflow stays.
//! - Native execution compares optimizer-on and optimizer-off PHP behavior.
//! - Assembly emission exercises both modes on every supported target.

use super::*;

/// Emits only the main function's textual EIR for one PHP program.
fn main_ir(source: &str, optimized: bool) -> String {
    let dir = make_cli_test_dir("elephc_integer_range_ir");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write integer range EIR fixture");
    let mut command = elephc_cli_command(&dir);
    command.arg("--emit-ir");
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let output = command.arg(&php_path).output().expect("emit integer range EIR");
    assert!(
        output.status.success(),
        "emit-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("EIR is UTF-8");
    let main = text
        .split("  function main()")
        .nth(1)
        .expect("main function")
        .to_string();
    fs::remove_dir_all(dir).expect("remove integer range EIR fixture");
    main
}

/// Compiles and executes a fixture with the selected optimizer mode.
fn run_variant(source: &str, optimized: bool) -> (String, String) {
    let dir = make_cli_test_dir("elephc_integer_range_run");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write integer range runtime fixture");
    let mut command = elephc_cli_command(&dir);
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let compile = command.arg(&php_path).output().expect("compile integer range fixture");
    assert!(
        compile.status.success(),
        "compile failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = run_binary(&dir.join("main"), &dir);
    assert!(
        run.status.success(),
        "integer range fixture failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8(run.stdout).expect("stdout is UTF-8");
    let stderr = String::from_utf8(run.stderr).expect("stderr is UTF-8");
    fs::remove_dir_all(dir).expect("remove integer range runtime fixture");
    (stdout, stderr)
}

/// Emits target-specific assembly with the selected optimizer mode.
fn target_assembly(source: &str, target: &str, optimized: bool) -> String {
    let dir = make_cli_test_dir("elephc_integer_range_target");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write integer range target fixture");
    let mut command = elephc_cli_command(&dir);
    command.arg("--emit-asm").arg("--target").arg(target);
    if target.starts_with("ios-") {
        command.arg("--emit").arg("staticlib");
    }
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let output = command.arg(&php_path).output().expect("emit target assembly");
    assert!(
        output.status.success(),
        "emit-asm for {target} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let assembly = fs::read_to_string(php_path.with_extension("s"))
        .expect("read integer range target assembly");
    fs::remove_dir_all(dir).expect("remove integer range target fixture");
    assembly
}

/// The example's bounded induction operations become unchecked scalar EIR.
#[test]
fn test_integer_range_rewrites_bounded_example() {
    let source = include_str!("../../../examples/integer-range/main.php");
    let plain = main_ir(source, false);
    let optimized = main_ir(source, true);
    let body = optimized
        .split("for.body:")
        .nth(1)
        .expect("optimized loop body")
        .split("for.update:")
        .next()
        .expect("body before update");
    let update = optimized
        .split("for.update:")
        .nth(1)
        .expect("optimized loop update")
        .split("for.exit:")
        .next()
        .expect("update before exit");

    assert!(plain.contains("= ichecked_mul "), "{plain}");
    assert!(plain.contains("= ichecked_sub "), "{plain}");
    assert!(body.contains("= imul "), "{optimized}");
    assert!(body.contains("= isub "), "{optimized}");
    assert!(body.contains("= iadd "), "{optimized}");
    assert!(!body.contains("= ichecked_"), "{optimized}");
    assert!(update.contains("= iadd "), "{optimized}");
    assert!(!update.contains("= ichecked_add_to_int "), "{optimized}");
    assert!(optimized.contains("origin: integer_range"), "{optimized}");

    let plain_run = run_variant(source, false);
    let optimized_run = run_variant(source, true);
    assert_eq!(plain_run, optimized_run);
    assert_eq!(optimized_run.0, "153");
}

/// A bounded expression observed directly as Mixed can still narrow to scalar output.
#[test]
fn test_integer_range_narrows_boxed_checked_output() {
    let source = "<?php echo ($argc & 255) + 1;";
    let plain = main_ir(source, false);
    let optimized = main_ir(source, true);
    assert!(plain.contains("= ichecked_add "), "{plain}");
    assert!(!optimized.contains("= ichecked_add "), "{optimized}");
    assert!(optimized.contains("= iadd "), "{optimized}");
    assert!(optimized.contains("origin: integer_range"), "{optimized}");
    assert_eq!(run_variant(source, false), run_variant(source, true));
}

/// Unbounded add, subtract, and multiply retain PHP overflow-to-float behavior.
#[test]
fn test_integer_range_keeps_unproven_overflow_checked() {
    let source = r#"<?php
$one = $argc;
echo PHP_INT_MAX + $one, "|";
echo PHP_INT_MIN - $one, "|";
echo PHP_INT_MAX * $one;
"#;
    let optimized = main_ir(source, true);
    assert!(optimized.contains("= ichecked_add "), "{optimized}");
    assert!(optimized.contains("= ichecked_sub "), "{optimized}");
    assert!(optimized.contains("= ichecked_mul "), "{optimized}");
    assert_eq!(run_variant(source, false), run_variant(source, true));
}

/// Every supported target emits both modes and removes only the bounded multiply.
#[test]
fn test_integer_range_all_supported_targets_compile_both_modes() {
    let source = r#"<?php
#[Export]
function bounded_checksum(): int {
    $limit = 9;
    $checksum = 0;
    for ($i = 0; $i < $limit; $i++) {
        $checksum = ($checksum + (($i * 3) & 65535)) & 65535;
    }
    return $checksum;
}
"#;
    for target in [
        "macos-aarch64",
        "ios-arm64",
        "ios-sim-arm64",
        "linux-aarch64",
        "linux-x86_64",
    ] {
        let plain = target_assembly(source, target, false);
        let optimized = target_assembly(source, target, true);
        assert!(
            plain.contains("op=ichecked_mul"),
            "unoptimized {target} assembly did not retain checked multiply"
        );
        assert!(
            optimized.contains("op=imul"),
            "optimized {target} assembly did not contain scalar multiply"
        );
        assert!(
            !optimized.contains("op=ichecked_mul"),
            "optimized {target} assembly retained bounded checked multiply"
        );
    }
}
