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
fn test_krsort_assoc_preserves_key_value_pairs() {
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

/// Verifies `krsort()` reverses indexed-key iteration order while preserving each key/value pair.
#[test]
fn test_krsort_indexed_reverses_key_iteration() {
    let out = compile_and_run(
        r#"<?php
$a = [3, 1, 2];
krsort($a);
foreach ($a as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,");
}

/// Verifies `krsort()` publishes packed-to-hash promotion through an array-element reference.
#[test]
fn test_krsort_mutates_packed_array_element_reference() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["row" => [3, 1, 2]];
$original = $matrix;
krsort($matrix["row"]);
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($original["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|0:3,1:1,2:2,");
}

/// Verifies `krsort()` publishes packed-to-hash promotion through a typed property reference.
#[test]
fn test_krsort_mutates_packed_typed_property_reference() {
    let out = compile_and_run(
        r#"<?php
class SortFixture {
    public array $values = [3, 1, 2];
}

$fixture = new SortFixture();
krsort($fixture->values);
foreach ($fixture->values as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,");
}

/// Verifies `krsort()` preserves a typed property's representation and performs
/// copy-on-write when the property's packed value came from a non-literal assignment.
#[test]
fn test_krsort_cow_splits_non_literal_typed_property_assignment() {
    let out = compile_and_run(
        r#"<?php
class KrsortPropertyFixture {
    public array $items = [];
}

$fixture = new KrsortPropertyFixture();
$source = [3, 1, 2];
$fixture->items = $source;
$alias = $fixture->items;
krsort($fixture->items);
foreach ($fixture->items as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($alias as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|0:3,1:1,2:2,");
}

/// Verifies named `array:` arguments preserve packed-key promotion for locals and nested lvalues.
#[test]
fn test_krsort_named_array_argument_promotes_packed_lvalues() {
    let out = compile_and_run(
        r#"<?php
$local = [3, 1, 2];
krsort(array: $local);
foreach ($local as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
$matrix = ["row" => [3, 1, 2]];
krsort(array: $matrix["row"]);
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|2:2,1:1,0:3,");
}

/// Verifies repeated `krsort()` calls reuse an already-promoted nested hash safely.
#[test]
fn test_krsort_reuses_promoted_nested_element() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["row" => [3, 1, 2]];
for ($i = 0; $i < 2; $i++) {
    krsort($matrix["row"]);
}
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,");
}

/// Verifies `krsort()` accepts a nested element already stored as an associative hash.
#[test]
fn test_krsort_accepts_nested_element_already_stored_as_hash() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["row" => [3, 1, 2, "x" => 4]];
krsort($matrix["row"]);
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "x:4,2:2,1:1,0:3,");
}

/// Verifies nested associative hashes are COW-split and written back before key sorting.
#[test]
fn test_krsort_writes_back_shared_nested_associative_hash() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["row" => ["a" => 1, "b" => 2]];
$original = $matrix;
krsort($matrix["row"]);
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($original["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "b:2,a:1,|a:1,b:2,");
}

/// Verifies reverse key sorting promotes and writes back an element of a packed parent.
#[test]
fn test_krsort_mutates_packed_parent_element_reference() {
    let out = compile_and_run(
        r#"<?php
$grid = [[3, 1, 2], [9, 8, 7]];
$original = $grid;
krsort($grid[0]);
foreach ($grid[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($original[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|0:3,1:1,2:2,");
}

/// Verifies distinct nested key-sort statements reuse a widened packed parent.
#[test]
fn test_krsort_reuses_widened_packed_parent_across_statements() {
    let out = compile_and_run(
        r#"<?php
$grid = [[3, 1, 2], [9, 8, 7]];
krsort($grid[0]);
krsort($grid[1]);
foreach ($grid[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($grid[1] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|2:7,1:8,0:9,");
}

/// Verifies nested key sorting works when the packed parent starts with Mixed cells.
#[test]
fn test_krsort_nested_element_of_initially_mixed_packed_parent() {
    let out = compile_and_run(
        r#"<?php
$grid = [[3, 1, 2], "sentinel"];
krsort($grid[0]);
foreach ($grid[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|" . $grid[1];
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|sentinel");
}

/// Verifies a born-Mixed packed parent detaches before its nested cell is promoted.
#[test]
fn test_krsort_cow_splits_initially_mixed_packed_parent() {
    let out = compile_and_run(
        r#"<?php
$grid = [[3, 1, 2], "sentinel"];
$original = $grid;
krsort($grid[0]);
foreach ($grid[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($original[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,|0:3,1:1,2:2,");
}

/// Verifies an already-widened associative parent detaches before sorting a sibling cell.
#[test]
fn test_krsort_cow_splits_widened_associative_parent() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["left" => [3, 1, 2], "right" => [9, 8, 7]];
krsort($matrix["left"]);
$original = $matrix;
krsort($matrix["right"]);
foreach ($matrix["right"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
echo "|";
foreach ($original["right"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:7,1:8,0:9,|0:9,1:8,2:7,");
}

/// Verifies an integer-typed dynamic index reaches the nested Mixed-cell lowering path.
#[test]
fn test_krsort_nested_mixed_parent_with_dynamic_integer_index() {
    let out = compile_and_run(
        r#"<?php
$grid = [[3, 1, 2], "sentinel"];
$index = 0;
krsort($grid[$index]);
foreach ($grid[0] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,");
}

/// Verifies a scalar Mixed child remains a controlled krsort type error.
#[test]
fn test_krsort_scalar_child_of_mixed_packed_parent_reports_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
$grid = [[3, 1, 2], "sentinel"];
krsort($grid[1]);
"#,
    );
    assert!(!out.success, "scalar nested value should fail");
    assert!(
        out.stderr.contains("krsort()")
            && out.stderr.contains("Argument #1")
            && out.stderr.contains("array"),
        "expected a controlled krsort array TypeError, got: {}",
        out.stderr,
    );
}

/// Verifies a missing Mixed child remains a controlled krsort type error.
#[test]
fn test_krsort_missing_child_of_mixed_packed_parent_reports_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
$grid = [[3, 1, 2], "sentinel"];
krsort($grid[9]);
"#,
    );
    assert!(!out.success, "missing nested value should fail");
    assert!(
        out.stderr.contains("krsort()")
            && out.stderr.contains("Argument #1")
            && out.stderr.contains("array"),
        "expected a controlled krsort array TypeError, got: {}",
        out.stderr,
    );
}

/// Verifies separate nested key-sort statements reuse an already-promoted child hash.
#[test]
fn test_krsort_reuses_promoted_nested_element_across_statements() {
    let out = compile_and_run(
        r#"<?php
$matrix = ["row" => [3, 1, 2]];
krsort($matrix["row"]);
krsort($matrix["row"]);
foreach ($matrix["row"] as $key => $value) {
    echo $key . ":" . $value . ",";
}
"#,
    );
    assert_eq!(out, "2:2,1:1,0:3,");
}

/// Verifies a missing nested element fails with a PHP-level type error instead of crashing.
#[test]
fn test_krsort_missing_nested_element_reports_type_error() {
    let out = compile_and_run_capture(
        r#"<?php
$matrix = ["row" => [3, 1, 2]];
krsort($matrix["missing"]);
"#,
    );
    assert!(!out.success, "missing nested array should fail");
    assert!(
        out.stderr.contains("krsort()")
            && out.stderr.contains("Argument #1")
            && out.stderr.contains("array"),
        "expected a controlled krsort array TypeError, got: {}",
        out.stderr,
    );
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
