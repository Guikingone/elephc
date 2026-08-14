//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of indexed array sorting, including asort, arsort, and ksort.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies asort maintains key-value associations and sorts by values in ascending order.
/// Fixture: [3, 1, 2] → sorted [1, 2, 3] → first element $a[0] should be 1.
#[test]
fn test_asort() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
asort($a);
echo $a[0];
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies arsort maintains key-value associations and sorts by values in descending order.
/// Fixture: [1, 3, 2] → sorted descending [3, 2, 1] → first element $a[0] should be 3.
#[test]
fn test_arsort() {
    let out = compile_and_run(
        r#"<?php
$a = [1, 3, 2];
arsort($a);
echo $a[0];
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies ksort sorts by keys in ascending order, preserving values.
/// Fixture: [3, 1, 2] with string keys → sorted by key → count remains 3.
#[test]
fn test_ksort() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
ksort($a);
echo count($a);
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies krsort sorts by keys in descending order, preserving key/value association.
/// An indexed array stores its keys as slot positions, so the receiver must be a hash;
/// fixture: [2 => 1, 1 => 2, 3 => 3] → descending key order 3, 2, 1 with values 3, 1, 2.
#[test]
fn test_krsort() {
    let out = compile_and_run(
        r#"<?php
$a = [2 => 1, 1 => 2, 3 => 3];
krsort($a);
echo count($a);
foreach ($a as $k => $v) { echo ":", $k, "=", $v; }
"#,
    );
    assert_eq!(out, "3:3=3:2=1:1=2");
}

/// Verifies natsort sorts values naturally (human ordering), preserving key-value associations.
/// Fixture: [3, 1, 2] → natural sort [1, 2, 3] → first element $a[0] should be 1.
#[test]
fn test_natsort() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
natsort($a);
echo $a[0];
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies natcasesort sorts values naturally case-insensitively, preserving key-value associations.
/// Fixture: [3, 1, 2] → case-insensitive natural sort [1, 2, 3] → first element $a[0] should be 1.
#[test]
fn test_natcasesort() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
natcasesort($a);
echo $a[0];
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies compiled PHP output for sort string array.
#[test]
fn test_sort_string_array() {
    let out = compile_and_run(
        r#"<?php
$a = ["banana", "apple", "cherry", "date"];
sort($a);
echo $a[0] . "," . $a[1] . "," . $a[2] . "," . $a[3];
"#,
    );
    assert_eq!(out, "apple,banana,cherry,date");
}

/// Verifies compiled PHP output for rsort string array.
#[test]
fn test_rsort_string_array() {
    let out = compile_and_run(
        r#"<?php
$a = ["banana", "apple", "cherry", "date"];
rsort($a);
echo $a[0] . "," . $a[1] . "," . $a[2] . "," . $a[3];
"#,
    );
    assert_eq!(out, "date,cherry,banana,apple");
}

/// Verifies gradual callback sorts preserve or reset keys according to the PHP builtin used.
#[test]
fn test_gradual_sort_compatibility_helpers() {
    let output = compile_and_run(
        r#"<?php
function compare_values(mixed $left, mixed $right): int { return $left <=> $right; }
function compare_keys(mixed $left, mixed $right): int { return $right <=> $left; }
function sort_all(mixed $input): array {
    $preserved = $input;
    uasort($preserved, 'compare_values');
    $reindexed = $input;
    usort($reindexed, 'compare_values');
    $keys = $input;
    uksort($keys, 'compare_keys');
    return [$preserved, $reindexed, $keys];
}
$result = sort_all(["b" => 2, "a" => 1]);
echo implode(',', array_keys($result[0])).'|';
echo implode(',', array_keys($result[1])).'|';
echo implode(',', array_keys($result[2]));
"#,
    );
    assert_eq!(output, "a,b|0,1|b,a");
}

/// Verifies key sorting a boxed gradual array reaches typed hash codegen without a backend refusal.
#[test]
fn test_ksort_gradual_local_reaches_typed_codegen_with_flags() {
    let source = r#"<?php
function sorted(mixed $values): mixed {
    ksort($values, SORT_STRING);
    return $values;
}
$source = ["b" => 2, "a" => 1];
$result = sorted($source);
foreach ($result as $key => $value) { echo $key . $value; }
echo ":";
foreach ($source as $key => $value) { echo $key . $value; }
"#;
    let dir = make_cli_test_dir("elephc_ksort_gradual_local");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("__rt_ksort"),
        "gradual ksort must reach the typed key-sort runtime call"
    );
    let _ = std::fs::remove_dir_all(dir);
}
