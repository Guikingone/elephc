//! Purpose:
//! End-to-end codegen coverage for integer range guard DCE.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses `$argc`-opaque inputs so AST constant folding does not erase the
//!   constructs before DCE; asserts dead marker strings are absent from assembly.
//! - `$argc` is 1 when the compiled binary runs with no CLI arguments.

use super::*;

/// Verifies transitive range pruning (`$x > 10` ⇒ `$x > 5`) and keeps live output.
#[test]
fn test_dead_code_elimination_prunes_nested_if_from_transitive_range_guard() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_transitive_range_guard");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run($x) {
    if ($x > 10) {
        if ($x > 5) {
            echo "a";
        } else {
            echo "dead-range";
        }
    } else {
        echo "b";
    }
}

run($argc + 11);
run($argc);
"#,
        &dir,
        8_388_608,
        false,
        false,
    );

    let out = assemble_and_run(
        &user_asm,
        get_runtime_obj(),
        &dir,
        &required_libraries,
        &default_link_paths(),
        &[],
    );
    assert_eq!(out, "ab");
    assert!(
        !user_asm.contains("dead-range"),
        "dead range branch should be absent from assembly"
    );
}

/// Verifies switch int cases outside an outer range are dropped before codegen.
#[test]
fn test_dead_code_elimination_drops_switch_cases_outside_range_guard() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_switch_range_guard");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run($x) {
    if ($x > 5) {
        switch ($x) {
            case 0:
                echo "dead-range-case";
                break;
            case 6:
                echo "a";
                break;
            default:
                echo "b";
                break;
        }
    } else {
        echo "c";
    }
}

run($argc + 5); // argc=1 → 6, hits case 6
run($argc);
"#,
        &dir,
        8_388_608,
        false,
        false,
    );

    let out = assemble_and_run(
        &user_asm,
        get_runtime_obj(),
        &dir,
        &required_libraries,
        &default_link_paths(),
        &[],
    );
    assert_eq!(out, "ac");
    assert!(
        !user_asm.contains("dead-range-case"),
        "impossible switch case should be absent from assembly"
    );
}
