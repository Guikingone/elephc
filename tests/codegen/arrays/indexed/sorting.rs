//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of indexed array sorting, including asort, arsort, and ksort.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies `asort()` orders an indexed receiver's VALUES ascending.
///
/// TRACKED DIVERGENCE, and this fixture pins only the half that holds. php passes
/// `renumber = 0` to `zend_array_sort`, so it permutes the keys instead of rebuilding them:
/// `php -n -r '$a=[3,1,2]; asort($a); echo $a[0], json_encode($a);'` prints
/// `3{"1":1,"2":2,"0":3}`, where `$a[0]` is still the array's ORIGINAL first element. A packed
/// receiver has no room for that permutation, so it is reindexed here and `$a[0]` reads the
/// SMALLEST value instead. Preserving those keys means converting the receiver to a hash — the
/// conversion `natsort()`/`natcasesort()` now perform, see `crate::types::key_preserving_sort_promotes`
/// for why `asort` is not on it yet.
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

/// The descending spelling, with the same tracked divergence: php prints `1` for `$a[0]` after
/// `$a=[1,3,2]; arsort($a);` (the original first element), where a reindexed receiver reads `3`.
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
