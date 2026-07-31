//! Purpose:
//! End-to-end codegen coverage for cross-variable relational guard DCE.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Runtime-unknown inputs keep the outer guards alive past AST folding; dead
//!   marker strings must not appear in generated assembly.
//! - `$argc` is 1 when the compiled binary runs with no CLI arguments.

use super::*;

/// Verifies `$x === $y` prunes nested `$x !== $y` before codegen.
#[test]
fn test_dead_code_elimination_prunes_nested_if_from_cross_var_strict_eq() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_cross_var_strict_eq");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run($x, $y) {
    if ($x === $y) {
        if ($x !== $y) {
            echo "dead-relvar";
        } else {
            echo "a";
        }
    } else {
        echo "b";
    }
}

run($argc, $argc);
run($argc, $argc + 1);
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
        !user_asm.contains("dead-relvar"),
        "dead relational branch should be absent from assembly"
    );
}

/// Verifies exact substitution `$x === 3` + `$y > $x` prunes `$y <= 3`.
#[test]
fn test_dead_code_elimination_prunes_nested_if_from_relational_exact_substitution() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_relational_exact_subst");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run($x, $y) {
    if ($x === 3) {
        if ($y > $x) {
            if ($y <= 3) {
                echo "dead-relvar";
            } else {
                echo "a";
            }
        } else {
            echo "b";
        }
    } else {
        echo "c";
    }
}

run(3, $argc + 4);
run(3, $argc);
run($argc, 9);
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
    // argc=1: run(3,5) → a; run(3,1) → b; run(1,9) → c
    assert_eq!(out, "abc");
    assert!(
        !user_asm.contains("dead-relvar"),
        "dead substituted relational branch should be absent from assembly"
    );
}
