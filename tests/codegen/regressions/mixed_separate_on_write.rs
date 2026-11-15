//! Purpose:
//! Regression tests for PHP value semantics of shared boxed `mixed` values: assigning a Mixed
//! value (`$x = $y`, reading `$e = $m[$k]` from Mixed storage, passing to a by-value `mixed`
//! parameter) only shares the boxed cell, so every in-place write must separate the cell first
//! instead of mutating the value seen by the other owners.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers appends, keyed sets, nested writes (indexed and associative parents), by-value
//!   `mixed` parameters, static locals, element reads, and `unset($mixed[$k])`.
//! - Heap-debug variants build Mixed values without `json_decode()` and assert a clean leak
//!   summary, so the separation's clone/release pairs stay balanced.

use crate::support::*;

/// Runs `source` under heap debug and asserts the expected stdout and a clean leak summary.
fn assert_heap_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, expected);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Appending to a copy of a decoded `mixed` array must not change the original.
#[test]
fn test_mixed_copy_append_does_not_mutate_original() {
    let out = compile_and_run(
        r#"<?php
$y = json_decode('[1]', true); $x = $y; $x[] = 2; echo json_encode($y), json_encode($x);
"#,
    );
    assert_eq!(out, "[1][1,2]");
}

/// Writing through an element read out of an `array<mixed>` must not change the source array.
#[test]
fn test_mixed_element_read_then_append_does_not_mutate_source() {
    let out = compile_and_run(
        r#"<?php
$m = [1, [5]]; $e = $m[1]; $e[] = 6; echo json_encode($m), json_encode($e);
"#,
    );
    assert_eq!(out, "[1,[5]][5,6]");
}

/// A nested append through a copied `array<mixed>` must separate the shared element cell.
#[test]
fn test_mixed_array_copy_nested_append_does_not_mutate_original() {
    let out = compile_and_run(
        r#"<?php
$m = [1, [5]]; $n = $m; $n[1][] = 7; echo json_encode($m), json_encode($n);
"#,
    );
    assert_eq!(out, "[1,[5]][1,[5,7]]");
}

/// A by-value `mixed` parameter owns a separated value inside the callee.
#[test]
fn test_mixed_by_value_param_append_does_not_mutate_caller() {
    let out = compile_and_run(
        r#"<?php
function f(mixed $v) { $v[] = 3; return $v; }
$q = json_decode('[1]', true); $r = f($q); echo json_encode($q), json_encode($r);
"#,
    );
    assert_eq!(out, "[1][1,3]");
}

/// Keyed sets on a copied `mixed` array (integer and string keys) separate the cell.
#[test]
fn test_mixed_copy_keyed_set_does_not_mutate_original() {
    let out = compile_and_run(
        r#"<?php
$y = json_decode('[1]', true); $x = $y; $x[0] = 5; echo json_encode($y), json_encode($x), "|";
$y = json_decode('{"a":1}', true); $x = $y; $x["b"] = 5; echo json_encode($y), json_encode($x);
"#,
    );
    assert_eq!(out, r#"[1][5]|{"a":1}{"a":1,"b":5}"#);
}

/// Nested writes through a copied `mixed` value separate the root and every shared level.
#[test]
fn test_mixed_copy_nested_writes_do_not_mutate_original() {
    let out = compile_and_run(
        r#"<?php
$y = json_decode('[[1]]', true); $x = $y; $x[0][0] = 5; echo json_encode($y), json_encode($x), "|";
$c = [[1,[2,3]]]; $d = $c; $d[0][1][] = 9; echo json_encode($c), json_encode($d), "|";
$y = json_decode('{"a":{"b":1}}', true); $x = $y; $x["a"]["c"] = 5; echo json_encode($y), json_encode($x), "|";
$m = json_decode('{"k":[1,{"z":2}]}', true); $e = $m["k"]; $e[1]["z"] = 3; echo json_encode($m), json_encode($e);
"#,
    );
    assert_eq!(
        out,
        r#"[[1]][[5]]|[[1,[2,3]]][[1,[2,3,9]]]|{"a":{"b":1}}{"a":{"b":1,"c":5}}|{"k":[1,{"z":2}]}[1,{"z":3}]"#
    );
}

/// Writing to the original after a copy leaves the copy untouched, and repeated copies in a loop
/// stay independent.
#[test]
fn test_mixed_original_write_and_loop_copies_stay_independent() {
    let out = compile_and_run(
        r#"<?php
$y = json_decode('[1]', true); $x = $y; $y[] = 7; echo json_encode($y), json_encode($x), "|";
$base = json_decode('[]', true);
$copies = [];
for ($i = 0; $i < 3; $i++) { $c = $base; $c[] = $i; $copies[] = $c; }
echo json_encode($base), json_encode($copies);
"#,
    );
    assert_eq!(out, "[1,7][1]|[][[0],[1],[2]]");
}

/// `unset($mixed[$key])` compiles for boxed `mixed` locals and separates a shared cell.
#[test]
fn test_mixed_copy_unset_does_not_mutate_original() {
    let out = compile_and_run(
        r#"<?php
$y = json_decode('{"a":1,"b":2}', true); $x = $y; unset($x["a"]); echo json_encode($y), json_encode($x), "|";
$y = json_decode('[1,2,3]', true); $x = $y; unset($x[1]); echo json_encode($y), json_encode($x), "|";
$y = json_decode('[1,2]', true); $x = $y; unset($x[5]); unset($x["k"]); echo json_encode($x), "|";
$z = json_decode('{"a":1}', true); unset($z["missing"]); unset($z["a"]); echo json_encode($z), count($z);
"#,
    );
    assert_eq!(
        out,
        r#"{"a":1,"b":2}{"b":2}|[1,2,3]{"0":1,"2":3}|[1,2]|[]0"#
    );
}

/// Separation of local, parameter, and element cells keeps the heap balanced.
#[test]
fn test_mixed_separate_on_write_is_heap_clean() {
    assert_heap_clean(
        r#"<?php
function mk(): mixed { return [1, [5], "s"]; }
function f(mixed $v) { $v[] = 3; $v[1][] = 4; return $v; }
function run() {
    $y = mk(); $x = $y; $x[] = 2; $x[0] = "t"; $x[1][] = 6;
    echo json_encode($y), json_encode($x), "|";
    $q = mk(); $r = f($q); echo json_encode($q), json_encode($r), "|";
    $m = [1, [5]]; $e = $m[1]; $e[] = 6; echo json_encode($m), json_encode($e), "|";
    $n = $m; $n[1][] = 7; echo json_encode($m), json_encode($n), "|";
    $c = [[1, [2, 3]]]; $d = $c; $d[0][1][] = 9; echo json_encode($c), json_encode($d);
}
run();
"#,
        r#"[1,[5],"s"]["t",[5,6],"s",2]|[1,[5],"s"][1,[5,4],"s",3]|[1,[5]][5,6]|[1,[5]][1,[5,7]]|[[1,[2,3]]][[1,[2,3,9]]]"#,
    );
}

/// Separation of hash-backed `mixed` values (keyed sets, nested writes, unset) is heap clean.
#[test]
fn test_mixed_separate_on_write_hash_paths_are_heap_clean() {
    assert_heap_clean(
        r#"<?php
function mk(): mixed { return ["a" => ["b" => 1], "c" => 2]; }
function run() {
    $y = mk(); $x = $y; $x["d"] = 3; $x["a"]["e"] = 4; unset($x["c"]);
    echo json_encode($y), json_encode($x), "|";
    for ($i = 0; $i < 3; $i++) { $z = $y; $z["a"]["i"] = $i; unset($z["c"]); echo json_encode($z); }
    echo "|", json_encode($y);
}
run();
"#,
        r#"{"a":{"b":1},"c":2}{"a":{"b":1,"e":4},"d":3}|{"a":{"b":1,"i":0}}{"a":{"b":1,"i":1}}{"a":{"b":1,"i":2}}|{"a":{"b":1},"c":2}"#,
    );
}
