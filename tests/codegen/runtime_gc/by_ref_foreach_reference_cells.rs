//! Purpose:
//! Heap-debug regression tests for the managed reference cells that back by-reference
//! `foreach` over associative storage.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each fixture runs under `--heap-debug` and asserts `leak summary: clean`, which also
//!   trips on a double free or a write into a released block.
//! - A reference entry owns one count on its cell and each live local alias owns another, so
//!   an unbalanced bind/release would show up here as a leak or an invalid free rather than as
//!   silent corruption.
//! - The growth fixtures deliberately cross several table reallocations while an alias is live,
//!   which is the window where a stale iterator table pointer used to be reused after free.

use crate::support::compile_and_run_with_heap_debug;

/// Asserts the program printed `expected` and left a clean heap under heap debug.
fn assert_clean(out: crate::support::ProgramOutput, expected: &str) {
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// A plain by-reference loop binds and releases one cell per entry with no residue.
#[test]
fn test_by_ref_foreach_releases_every_reference_cell() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as &$v) {
    $v = $v + 1;
}
unset($v);
echo implode(",", $a), "\n";
"#,
    );
    assert_clean(out, "2,3,4\n");
}

/// Growth during the loop relocates the table many times and must not leak or double free.
#[test]
fn test_by_ref_foreach_growth_keeps_the_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, 2, 3];
$appended = 0;
foreach ($a as &$v) {
    if ($appended < 40) {
        $appended = $appended + 1;
        $a[] = 7;
    }
    $v = $v + 1;
}
unset($v);
echo count($a), "\n";
"#,
    );
    assert_clean(out, "43\n");
}

/// Leaving the loop through `break` retires the live alias on the cleanup path.
#[test]
fn test_by_ref_foreach_break_releases_the_live_alias() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as &$v) {
    $v = $v * 3;
    break;
}
unset($v);
echo implode(",", $a), "\n";
"#,
    );
    assert_clean(out, "3,2,3\n");
}

/// Overwriting the referenced key releases the previous cell payload exactly once.
#[test]
fn test_reference_write_through_releases_the_previous_payload() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["k" => "first", "j" => "second"];
foreach ($a as $key => &$v) {
    if ($key === "k") {
        $a["k"] = "replaced";
    }
}
unset($v);
echo $a["k"], "|", $a["j"], "\n";
"#,
    );
    assert_clean(out, "replaced|second\n");
}

/// A copy that shares a reference element releases the shared cell only when both owners drop it.
#[test]
fn test_shared_reference_entry_survives_one_owner_going_away() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, 2, 3];
foreach ($a as &$v) {
}
$c = $a;
$v = 99;
unset($a);
echo $c[2], "\n";
"#,
    );
    assert_clean(out, "99\n");
}

/// A closure keeps the cell alive after the source array is destroyed, then releases it once.
#[test]
fn test_escaped_reference_cell_is_released_by_its_last_owner() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [1, 2, 3];
$readers = [];
foreach ($a as &$v) {
    $readers[] = function () use (&$v) { return $v; };
    break;
}
unset($v);
unset($a);
$reader = $readers[0];
echo $reader(), "\n";
"#,
    );
    assert_clean(out, "1\n");
}

/// Unsetting the CURRENT key and then forcing relocation must never touch freed storage.
///
/// The successor-key anchor lets iteration continue here, and this fixture pins the ownership
/// half of that: the freed table is never read, the reference cell of the removed entry is
/// released exactly once, and every cell bound during the continued walk is retired. The
/// value-level continuation is asserted in tests/codegen/arrays/foreach_by_ref_growth.rs.
#[test]
fn unsetting_the_current_key_then_relocating_never_touches_freed_storage() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a" => 1, "b" => 2, "c" => 3];
foreach ($a as $k => &$v) {
    if ($k === "a") {
        unset($a["a"]);
        for ($i = 0; $i < 40; $i = $i + 1) {
            $a["g" . $i] = 0;
        }
    }
}
unset($v);
echo "done\n";
"#,
    );
    assert_clean(out, "done\n");
}

/// Relocating WITHOUT deleting the current key resumes on the successor, like PHP.
#[test]
fn relocating_without_deleting_the_current_key_resumes_on_the_successor() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a" => 1, "b" => 2, "c" => 3];
$seen = "";
foreach ($a as $k => &$v) {
    $seen = $seen . $k;
    if ($k === "a") {
        for ($i = 0; $i < 40; $i = $i + 1) {
            $a["g" . $i] = 0;
        }
    }
    $v = $v + 1;
}
unset($v);
echo substr($seen, 0, 3), "|", $a["a"], "|", $a["b"], "|", $a["c"], "\n";
"#,
    );
    assert_clean(out, "abc|2|3|4\n");
}

/// Owned string anchors survive immediate-successor deletion and are released after resync.
#[test]
fn deleted_immediate_successor_anchor_is_safe_and_balanced_across_growth() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a" => 1, "b" => 2, "c" => 3, "d" => 4];
$seen = "";
foreach ($a as $k => &$v) {
    $seen = $seen . $k;
    if ($k === "a") {
        unset($a["b"]);
        for ($i = 0; $i < 40; $i = $i + 1) {
            $a["g" . $i] = 0;
        }
    }
}
unset($v);
echo substr($seen, 0, 3), "\n";
"#,
    );
    assert_clean(out, "acd\n");
}

/// Breaking while string anchors are populated runs `IterEnd` and releases both retains.
#[test]
fn by_ref_foreach_break_releases_owned_string_anchors() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a" => 1, "b" => 2, "c" => 3];
foreach ($a as $k => &$v) {
    break;
}
unset($v);
unset($a);
echo "done\n";
"#,
    );
    assert_clean(out, "done\n");
}

/// Re-entering one lowered foreach state repeatedly starts from anchors cleared by `IterEnd`.
#[test]
fn repeated_foreach_state_reuse_keeps_string_anchor_ownership_balanced() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = ["a" => 1, "b" => 2, "c" => 3];
for ($i = 0; $i < 8; $i = $i + 1) {
    foreach ($a as $k => &$v) {
        break;
    }
    unset($v);
}
unset($a);
echo "done\n";
"#,
    );
    assert_clean(out, "done\n");
}

/// Returning from inside the loop runs the loop-frame `IterEnd` cleanup before function exit.
#[test]
fn by_ref_foreach_return_releases_owned_string_anchors() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function first(array $a): int {
    foreach ($a as $k => &$v) {
        return $v;
    }
    return 0;
}
echo first(["a" => 1, "b" => 2, "c" => 3]), "\n";
"#,
    );
    assert_clean(out, "1\n");
}
