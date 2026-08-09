//! Purpose:
//! Regression tests for mutating array/hash builtins whose by-reference receiver is a
//! by-reference PARAMETER (`function f(array &$a)`). Every one of these lost the write-back:
//! the backend's slot resolver only recognized `load_local`, so a receiver read with
//! `load_ref_cell` had nowhere to publish the copy-on-write split or the growth relocation.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected value is verbatim `LC_ALL=C php` 8.4 output for the same fixture.
//! - The `$alias = $x;` lines are load-bearing: they make the receiver shared, so the runtime's
//!   ensure-unique separates a private copy. Without the write-back that copy was mutated and
//!   thrown away, and the caller observed the original array — a silent wrong answer.
//! - `array_unshift` fails even WITHOUT an alias, because prepending reaches `__rt_array_grow`
//!   and the caller then held a pointer to storage the growth had already freed. The
//!   nine-value fixture forces that growth.
//! - The heap-debug assertion pins that republishing the relocated pointer does not double
//!   release the previous storage.

use crate::support::*;

/// Verifies `array_unshift()` on a by-reference parameter reaches the caller's array.
///
/// Nine prepends into a two-element array force at least one `__rt_array_grow`, so this is the
/// use-after-free case: the caller used to print nothing at all because it read the freed
/// pre-growth storage.
#[test]
fn test_array_unshift_on_by_ref_parameter_reaches_caller() {
    let out = compile_and_run(
        r#"<?php
function f(array &$a) { array_unshift($a, 9,8,7,6,5,4,3,2,1); }
$x = [1,2]; f($x); echo implode(",", $x), "\n";
"#,
    );
    assert_eq!(out, "9,8,7,6,5,4,3,2,1,1,2\n");
}

/// Verifies the shape-changing indexed builtins publish their copy-on-write split through a
/// by-reference parameter, and that an alias taken beforehand keeps the original order.
#[test]
fn test_shape_changing_builtins_on_by_ref_parameter_match_php() {
    let out = compile_and_run(
        r#"<?php
function g(array &$a) { $v = array_shift($a); echo $v, "\n"; }
$y = [1,2,3]; $ya = $y; g($y); echo implode(",", $y), "|", implode(",", $ya), "\n";
function h(array &$a) { echo array_pop($a), "\n"; }
$z = [1,2,3]; $za = $z; h($z); echo implode(",", $z), "|", implode(",", $za), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"1
2,3|1,2,3
3
1,2|1,2,3
"#
    );
}

/// Verifies the sort family publishes its copy-on-write split through a by-reference parameter.
///
/// `sort`, `usort`, `ksort` (an insertion-order relink), and `array_multisort` all resolve their
/// receiver the same way, so one missing case would leave the caller unsorted with no diagnostic.
#[test]
fn test_sort_family_on_by_ref_parameter_matches_php() {
    let out = compile_and_run(
        r#"<?php
function s(array &$a) { sort($a); }
$w = [3,1,2]; $wa = $w; s($w); echo implode(",", $w), "|", implode(",", $wa), "\n";
function u(array &$a) { usort($a, fn(int $p, int $q): int => $q <=> $p); }
$v = [3,1,2]; $va = $v; u($v); echo implode(",", $v), "|", implode(",", $va), "\n";
function k(array &$a) { ksort($a); }
$m = ["b"=>2,"a"=>1]; $ma = $m; k($m); echo implode(",", array_keys($m)), "|", implode(",", array_keys($ma)), "\n";
function ms(array &$p, array &$q) { array_multisort($p, $q); }
$o = [3,1,2]; $oo = [30,10,20]; $oa = $o; ms($o, $oo); echo implode(",", $o), "|", implode(",", $oa), "\n";
"#,
    );
    assert_eq!(
        out,
        r#"1,2,3|3,1,2
3,2,1|3,1,2
a,b|b,a
1,2,3|3,1,2
"#
    );
}

/// Verifies an associative insert through a by-reference parameter reaches the caller's table.
///
/// `$a["c"] = 3` splits the shared table with `__rt_hash_ensure_unique` and can reallocate it,
/// so the hash lowering needs the same ref-cell write-back the indexed builtins do.
#[test]
fn test_hash_insert_on_by_ref_parameter_matches_php() {
    let out = compile_and_run(
        r#"<?php
function hs(array &$a) { $a["c"] = 3; }
$n = ["a"=>1,"b"=>2]; $na = $n; hs($n); echo implode(",", array_keys($n)), "|", implode(",", array_keys($na)), "\n";
"#,
    );
    assert_eq!(out, "a,b,c|a,b\n");
}

/// Verifies the whole by-reference receiver matrix leaves the heap balanced.
///
/// Republishing a relocated pointer through a ref cell releases whatever the slot held before,
/// so a write-back that dropped or double-counted the previous owner shows up here.
#[test]
fn test_by_ref_parameter_receivers_leave_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f(array &$a) { array_unshift($a, 9,8,7,6,5,4,3,2,1); }
$x = [1,2]; f($x); echo implode(",", $x), "\n";
function g(array &$a) { $v = array_shift($a); echo $v, "\n"; }
$y = [1,2,3]; $ya = $y; g($y); echo implode(",", $y), "|", implode(",", $ya), "\n";
function h(array &$a) { echo array_pop($a), "\n"; }
$z = [1,2,3]; $za = $z; h($z); echo implode(",", $z), "|", implode(",", $za), "\n";
function s(array &$a) { sort($a); }
$w = [3,1,2]; $wa = $w; s($w); echo implode(",", $w), "|", implode(",", $wa), "\n";
"#,
    );
    assert_eq!(
        out.stdout,
        r#"9,8,7,6,5,4,3,2,1,1,2
1
2,3|1,2,3
3
1,2|1,2,3
1,2,3|3,1,2
"#,
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}
