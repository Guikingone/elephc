//! Purpose:
//! End-to-end coverage for the raw-int fused map+sum sink.
//!
//! Called from:
//! - The codegen optimizer suite through `optimizer`.
//!
//! Key details:
//! - Positive, negative and overflow-promoted map offsets exercise the vector, scalar and
//!   warned float-to-int paths without boxing the intermediate result.
//! - `--ir-opt=off` keeps the exact boxed map+sum path and must produce the same output.

use crate::support::*;

/// Emits the `main` function body for one source fixture in the requested optimizer mode.
fn emit_main_ir(source: &str, ir_opt: bool) -> String {
    let dir = make_cli_test_dir("elephc_map_sum_raw_ir");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write map-sum IR fixture");
    let mode = if ir_opt { "--ir-opt=on" } else { "--ir-opt=off" };
    let output = elephc_cli_command(&dir)
        .arg("--emit-ir")
        .arg(mode)
        .arg(&php_path)
        .output()
        .expect("run elephc --emit-ir");
    assert!(
        output.status.success(),
        "emit-ir failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("EIR is UTF-8");
    let main = text
        .split("function main(")
        .nth(1)
        .expect("EIR contains main")
        .split("\n  function ")
        .next()
        .expect("main body")
        .to_string();
    let _ = fs::remove_dir_all(dir);
    main
}

const FIXTURE: &str = r#"<?php
$a = range(1, 100);
$n = 0;
for ($i = 0; $i < 2001; $i++) { $n ^= array_sum(array_map(fn($x) => $x + 3, $a)); }
echo $n, '|';
$b = [1, 2, 3];
echo array_sum(array_map(fn($x) => $x - 1000, $b)), '|';
$c = range(PHP_INT_MAX, PHP_INT_MAX);
$n = 0;
$n ^= array_sum(array_map(fn($x) => $x + 1, $c));
echo $n, "\n";
"#;

const EXPECTED: &str = "5350|-2994|-9223372036854775808\n";

/// The raw sink returns the same values as PHP and leaves no heap allocations behind.
#[test]
fn array_map_sum_add_int_sink_matches_php() {
    let out = compile_and_run_with_heap_debug(FIXTURE);
    assert!(out.success, "program crashed: {}", out.stderr);
    assert_eq!(out.stdout, EXPECTED);
    assert!(
        out.stderr
            .contains("The float 9.223372036854776E+18 is not representable as an int"),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Disabling EIR optimization keeps the boxed map+sum path with the same output.
#[test]
fn array_map_sum_add_int_sink_no_ir_opt_parity() {
    let out = without_ir_opt(|| compile_and_run(FIXTURE));
    assert_eq!(out, EXPECTED);
}

/// The optimized main function calls the raw kernel directly, without a cast of its result.
#[test]
fn array_map_sum_add_int_sink_removes_the_cast() {
    let ir = emit_main_ir(FIXTURE, true);
    assert!(ir.contains("runtime.array_map_sum_add_int"), "{ir}");
    assert!(!ir.contains("runtime.array_map_sum_add "), "{ir}");
    let call = ir
        .lines()
        .find(|line| line.contains("runtime.array_map_sum_add_int"))
        .expect("raw map+sum line");
    let result = call
        .split(':')
        .next()
        .expect("result value")
        .trim()
        .to_string();
    assert!(
        !ir.lines()
            .any(|line| line.contains("= cast") && line.contains(&result)),
        "{ir}"
    );
}
