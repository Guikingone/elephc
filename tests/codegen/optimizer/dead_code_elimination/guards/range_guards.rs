//! Purpose:
//! End-to-end codegen coverage for integer range guard DCE.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Uses `$argc`-opaque inputs so AST constant folding does not erase the
//!   constructs before DCE; asserts dead marker strings are absent from assembly.
//! - Float parameters pin the discrete-domain safety regression around fractional gaps.
//! - `$argc` is 1 when the compiled binary runs with no CLI arguments.

use super::*;

/// Verifies transitive range pruning (`$x > 10` ⇒ `$x > 5`) and keeps live output.
#[test]
fn test_dead_code_elimination_prunes_nested_if_from_transitive_range_guard() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_transitive_range_guard");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run(int $x) {
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
function run(int $x) {
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

/// Verifies cumulative false bounds in an `elseif` chain isolate zero for an int parameter.
#[test]
fn test_dead_code_elimination_refines_integer_range_across_elseif_prefix() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_elseif_range_guard");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run(int $x) {
    if ($x < 0) {
        echo "n";
    } elseif ($x > 0) {
        echo "p";
    } else {
        if ($x === 0) {
            echo "z";
        } else {
            echo "dead-elseif-range";
        }
    }
}

run($argc - 1);
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
    assert_eq!(out, "z");
    assert!(
        !user_asm.contains("dead-elseif-range"),
        "range-excluded elseif branch should be absent from assembly"
    );
}

/// Verifies a float between adjacent integers keeps the nested fractional-gap branch live.
#[test]
fn test_dead_code_elimination_does_not_apply_integer_gap_to_float_parameter() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_float_range_domain");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run(float $x) {
    if ($x > 10) {
        if ($x >= 11) {
            echo "whole";
        } else {
            echo "fractional";
        }
    }
}

run(10.5);
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
    assert_eq!(out, "fractional");
    assert!(user_asm.contains("whole"));
    assert!(user_asm.contains("fractional"));
}

/// Verifies `foreach` clears an inherited int range before overwriting its value variable.
#[test]
fn test_dead_code_elimination_invalidates_range_for_foreach_value() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_foreach_range_domain");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function run(int $x) {
    if ($x > 10) {
        foreach ([10.5] as $x) {
            if ($x >= 11) {
                echo "whole";
            } else {
                echo "fractional";
            }
        }
    }
}

run(12);
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
    assert_eq!(out, "fractional");
    assert!(user_asm.contains("whole"));
    assert!(user_asm.contains("fractional"));
}

/// Verifies an exact-`int` typed local enables transitive range pruning without an int parameter.
#[test]
fn test_dead_code_elimination_seeds_range_from_typed_local() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_typed_local_range");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
int $x = $argc + 11;
if ($x > 10) {
    if ($x > 5) {
        echo "a";
    } else {
        echo "dead-typed-local-range";
    }
}

float $f = 10.5;
if ($f > 10) {
    if ($f >= 11) {
        echo "whole";
    } else {
        echo "fractional";
    }
}
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
    assert_eq!(out, "afractional");
    assert!(
        !user_asm.contains("dead-typed-local-range"),
        "typed int local should seed transitive range pruning"
    );
    assert!(user_asm.contains("whole"));
    assert!(user_asm.contains("fractional"));
}
