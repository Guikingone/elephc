//! Purpose:
//! Integration or regression tests for optimizer-sensitive codegen coverage of optimizer, dead-code elimination, switches tail paths, including dead code elimination collapses empty switch shell after branch dce, dead code elimination sinks tail into switch exit paths, and dead code elimination sinks tail into switch break paths.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled and run so folding, propagation, or pruning stays behavior-preserving.

use super::*;

/// Verifies that an empty switch shell (all branches dead after DCE) does not emit `switch_end`
/// in user assembly. Confirms "s!".
#[test]
fn test_dead_code_elimination_collapses_empty_switch_shell_after_branch_dce() {
    let dir = make_cli_test_dir("elephc_dead_code_elimination_empty_switch_shell");
    let (user_asm, _runtime_asm, required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
function poke() {
    echo "s";
    return 1;
}

switch (poke()) {
    case 1:
        strlen("abc");
        break;
}

echo "!";
"#,
        &dir,
        8_388_608,
        false,
        false,
    );

    assert!(
        !user_asm.contains("switch_end"),
        "empty switch shells should not survive user assembly after DCE:\n{}",
        user_asm
    );

    let out = assemble_and_run(
        &user_asm,
        get_runtime_obj(),
        &dir,
        &required_libraries,
        &default_link_paths(),
        &[],
    );
    assert_eq!(out, "s!");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies that tail code after a switch statement sinks correctly through all exit paths
/// when cases fall through to each other. Tests switch with three cases (1, 2, default)
/// where no branch has a break, so execution falls through: case 1 → case 2 → default.
/// Expected output: case 1 emits "abc", case 2 emits "bc", default emits "c", each followed by "!".
#[test]
fn test_dead_code_elimination_sinks_tail_into_switch_exit_paths() {
    let out = compile_and_run(
        r#"<?php
function run(int $flag) {
    switch ($flag) {
        case 1:
            echo "a";
        case 2:
            echo "b";
        default:
            echo "c";
    }
    echo "!";
}

run(1);
run(2);
run(3);
"#,
    );

    assert_eq!(out, "abc!bc!c!");
}

/// Regression: a switch **without a default** has an implicit no-match path that falls through
/// to the statements after the switch. The DCE tail-sinking optimization must not drop that tail
/// for the no-match path when it sinks the tail into the matching case bodies. Here `run(1)`
/// matches the single case (prints "a" then the sunk tail "!"), while `run(2)`/`run(3)` match no
/// case and must still reach the tail ("!"). Before the fix the no-match path skipped the tail
/// entirely (and, inside a function, fell straight to the epilogue).
#[test]
fn test_dead_code_elimination_keeps_tail_for_no_default_switch_no_match_path() {
    let out = compile_and_run(
        r#"<?php
function run(int $flag) {
    switch ($flag) {
        case 1:
            echo "a";
            break;
    }
    echo "!";
}

run(1);
run(2);
run(3);
"#,
    );

    assert_eq!(out, "a!!!");
}

/// Regression: a no-default switch whose only case **returns** (rather than breaks) must still
/// execute the code after the switch on the no-match path. `pick(1)` returns early ("one"), while
/// `pick(2)` matches nothing and falls through to `return "other"`. Before the fix the no-match
/// path jumped to the function epilogue and returned an uninitialized value.
#[test]
fn test_no_default_switch_returning_case_falls_through_on_no_match() {
    let out = compile_and_run(
        r#"<?php
function pick(int $flag): string {
    switch (true) {
        case 1 === $flag:
            return "one";
    }
    return "other";
}

echo pick(1), "|", pick(2), "|", pick(3);
"#,
    );

    assert_eq!(out, "one|other|other");
}

/// Verifies that code after a switch with a break in one case and fallthrough in another sinks
/// correctly. Confirms "a!bc!c!".
#[test]
fn test_dead_code_elimination_sinks_tail_into_switch_break_paths() {
    let out = compile_and_run(
        r#"<?php
function run(int $flag) {
    switch ($flag) {
        case 1:
            echo "a";
            break;
        case 2:
            echo "b";
        default:
            echo "c";
    }
    echo "!";
}

run(1);
run(2);
run(3);
"#,
    );

    assert_eq!(out, "a!bc!c!");
}
