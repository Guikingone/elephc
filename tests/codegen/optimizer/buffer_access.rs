//! Purpose:
//! End-to-end regression coverage for `buffer<T>` hot-loop optimization: exact packed field
//! effects.
//!
//! Called from:
//! - `cargo test --test codegen_tests optimizer::buffer_access`.
//!
//! Key details:
//! - Textual EIR pins the optimized loop shape; every fixture also runs natively with the
//!   optimizer on and off and must print the same result.

use super::*;

/// One packed-record integration step in the shape of Ivory's `bench/integrate` hot loop.
const INTEGRATE_SOURCE: &str = r#"<?php
packed class Body { public float $x; public float $vx; }
function step(buffer<Body> $b, int $n, float $dt): void {
    for ($i = 0; $i < $n; $i++) {
        $x = $b[$i]->x + $b[$i]->vx * $dt;
        if ($x < 0.0) {
            $x = -$x;
            $b[$i]->vx = -$b[$i]->vx;
        }
        $b[$i]->x = $x;
    }
}
$b = buffer_new<Body>(3);
$b[0]->vx = 1.5;
$b[1]->vx = -2.0;
$b[2]->x = 1.0;
$b[2]->vx = 0.25;
step($b, 3, 2.0);
echo $b[0]->x, " ", $b[0]->vx, " ", $b[1]->x, " ", $b[1]->vx, " ", $b[2]->x, "\n";
"#;

/// The output both optimizer modes must print for `INTEGRATE_SOURCE`.
const INTEGRATE_OUTPUT: &str = "3 1.5 4 2 1.5\n";

/// Emits one named function's textual EIR, optimized or not, without other function bodies.
fn function_ir(source: &str, optimized: bool, name: &str) -> String {
    let dir = make_cli_test_dir("elephc_buffer_access_ir");
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
    let header = format!("  function {name}(");
    let body = text.split_once(&header).expect("named function").1;
    let function = body.split("\n  function ").next().expect("function body").to_string();
    fs::remove_dir_all(dir).expect("remove fixture directory");
    function
}

/// Compiles and runs one PHP program with the EIR optimizer on or off and returns stdout.
fn run_variant(source: &str, optimized: bool) -> String {
    let dir = make_cli_test_dir("elephc_buffer_access_run");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("write PHP fixture");
    let mut command = elephc_cli_command(&dir);
    if !optimized {
        command.arg("--no-ir-opt");
    }
    let compile = command.arg(&php_path).output().expect("compile PHP fixture");
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    let run = run_binary(&dir.join("main"), &dir);
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
    let stdout = String::from_utf8(run.stdout).expect("program stdout is UTF-8");
    fs::remove_dir_all(dir).expect("remove fixture directory");
    stdout
}

/// Returns the effect list printed for one EIR instruction line, or an empty string.
fn printed_effects(line: &str) -> &str {
    line.split(" ; ")
        .find_map(|part| part.strip_prefix("effects: "))
        .unwrap_or("")
}

/// Packed field reads and writes carry their exact effects instead of the object-property
/// contract, so the read no longer advertises warnings (which published the diagnostic source
/// location before every field load) and the write no longer advertises refcounting or throws.
#[test]
fn test_packed_field_accesses_carry_exact_effects() {
    let ir = function_ir(INTEGRATE_SOURCE, true, "step");
    let reads: Vec<&str> = ir.lines().filter(|line| line.contains("= prop_get ")).collect();
    let writes: Vec<&str> = ir.lines().filter(|line| line.contains(" prop_set ")).collect();
    assert!(!reads.is_empty() && !writes.is_empty(), "{ir}");
    for line in reads {
        assert_eq!(printed_effects(line), "reads_heap", "{line}");
    }
    for line in writes {
        assert_eq!(printed_effects(line), "writes_heap, may_fatal", "{line}");
    }
    assert_eq!(run_variant(INTEGRATE_SOURCE, true), INTEGRATE_OUTPUT);
    assert_eq!(run_variant(INTEGRATE_SOURCE, false), INTEGRATE_OUTPUT);
}
