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

/// A `switch` without `default` still runs the code after it when no case matches. The tail is
/// sunk into every `break` path, and the no-match path (and the last case falling off the
/// switch) must reach it as well; before the fix both lost the tail, so `run(3)` printed nothing
/// (issue #877).
#[test]
fn test_dead_code_elimination_keeps_tail_on_switch_no_match_path_without_default() {
    let out = compile_and_run(
        r#"<?php
function run(int $flag) {
    switch ($flag) {
        case 1:
            echo "a";
            break;
        case 2:
            echo "b";
            break;
    }
    echo "!";
}
function fall(int $flag) {
    switch ($flag) {
        case 1:
            echo "a";
            break;
        case 2:
            echo "b";
    }
    echo "!";
}
$x = $argc + 2;
switch ($x) {
    case 1:
        echo "a";
        break;
    case 2:
        echo "b";
        break;
}
echo "|";
run(1);
run(2);
run(3);
echo "|";
fall(1);
fall(2);
fall(3);
"#,
    );

    assert_eq!(out, "|a!b!!|a!b!!");
}

/// A loop `break` / `continue` that follows a `switch` must keep targeting the loop: sunk into
/// a case body it would target the switch instead. The first loop exits after one iteration,
/// the second skips its echo only on the first iteration.
#[test]
fn test_dead_code_elimination_keeps_loop_exit_tail_outside_switch() {
    let out = compile_and_run(
        r#"<?php
$i = 0;
while ($i < 3) {
    $i++;
    switch ($argc) {
        case 1: echo "a"; break;
    }
    break;
}
echo "|", $i, "|";
$j = 0;
while ($j < 3) {
    $j++;
    switch ($argc) {
        case 1: echo "b"; break;
        default: echo "e"; break;
    }
    if ($j == 1) { continue; }
    echo $j;
}
echo "|", $j;
"#,
    );

    assert_eq!(out, "a|1|bb2b3|3");
}

/// A `default` written between cases that falls through must continue into the next case, and
/// the code after the switch must run only once control leaves it: for `$x = 3` PHP prints `db|`.
/// Before the fix the optimizer sank the tail into the default's fallthrough path and printed
/// `d|b|` (issue #881).
#[test]
fn test_dead_code_elimination_keeps_middle_default_fallthrough_into_next_case() {
    let out = compile_and_run(
        r#"<?php
function f($x) {
    switch ($x) {
        case 1: echo "a"; break;
        default: echo "d";
        case 2: echo "b"; break;
    }
    echo "|";
}
function g($x) {
    switch ($x) {
        case 1:
        default: echo "d";
        case 2: echo "b";
    }
    echo "|";
}
f($argc); f($argc + 1); f($argc + 2);
g($argc); g($argc + 1); g($argc + 2);
"#,
    );

    assert_eq!(out, "a|b|db|db|b|db|");
}

/// An empty `default:` written before another case falls through into that case for any
/// unmatched subject, exactly like PHP: `f(3)` prints `b|`. Before the fix the empty default had
/// no position, was placed last, and an unmatched subject left the switch (issue #881).
#[test]
fn test_dead_code_elimination_keeps_empty_middle_default_fallthrough_into_next_case() {
    let out = compile_and_run(
        r#"<?php
function f($x) {
    switch ($x) {
        case 1: echo "a"; break;
        default:
        case 2: echo "b"; break;
    }
    echo "|";
}
f($argc); f($argc + 1); f($argc + 2);
"#,
    );

    assert_eq!(out, "a|b|b|");
}
