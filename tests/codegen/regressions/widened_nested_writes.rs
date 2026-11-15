//! Purpose:
//! Regression tests for nested array writes through locals whose type changes across a scope
//! (`$c = [[...]]; ...; $c = ["a" => [...]];`), plus the copy-on-write separation those writes
//! rely on.
//!
//! Called from:
//! - `cargo test` through Rust's test harness (`tests/codegen/regressions.rs`).
//!
//! Key details:
//! - In the main scope a local's contextual type is its final checker type, so an earlier list
//!   literal used to be promoted to that later map type with its inner lists still typed as maps;
//!   the following nested write dispatched hash operations on list storage and crashed.
//! - The concrete-chain write-back released each hidden temp through a second load after the
//!   parent setter had already taken the temp's reference, freeing the child the parent stored.
//! - Indexed writes into a boxed-Mixed local leaked the previous box, and Mixed cells shared by a
//!   container copy (or by `$x = $arr[$k]`) were mutated through every holder.
//! - Every program runs with heap debugging and must finish with a clean leak summary.

use crate::support::*;

/// Asserts PHP-identical stdout and a clean heap-debug leak summary.
fn assert_clean_run(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, expected);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// The reported program: list-of-lists then map-of-maps through the same main-scope locals.
#[test]
fn test_widened_main_locals_nested_append_list_then_map() {
    assert_clean_run(
        r#"<?php
$c = [[1,[2,"x"]]]; $d = $c; $d[0][1][] = 9; echo json_encode($c), json_encode($d), "\n";
$c = ["a"=>["b"=>[1]]]; $d = $c; $d["a"]["b"][] = 9; echo json_encode($c), json_encode($d), "\n";
"#,
        "[[1,[2,\"x\"]]][[1,[2,\"x\",9]]]\n{\"a\":{\"b\":[1]}}{\"a\":{\"b\":[1,9]}}\n",
    );
}

/// Homogeneous list rows first, map later, in the main scope.
#[test]
fn test_widened_main_locals_concrete_list_rows_then_map() {
    assert_clean_run(
        r#"<?php
$c = [[1,2],[3]]; $d = $c; $d[0][] = 9; echo json_encode($c), json_encode($d), "\n";
$c = ["a"=>["b"=>[1]]]; $d = $c; $d["a"]["b"][] = 9; echo json_encode($c), json_encode($d), "\n";
"#,
        "[[1,2],[3]][[1,2,9],[3]]\n{\"a\":{\"b\":[1]}}{\"a\":{\"b\":[1,9]}}\n",
    );
}

/// Map first, list later (hash -> list retype), inside a function.
#[test]
fn test_widened_function_locals_map_then_list() {
    assert_clean_run(
        r#"<?php
function f() {
    $c = ["a"=>["b"=>[1]]]; $d = $c; $d["a"]["b"][] = 9; echo json_encode($c), json_encode($d), "\n";
    $c = [[1,2],[3]]; $d = $c; $d[0][] = 9; echo json_encode($c), json_encode($d), "\n";
}
f();
"#,
        "{\"a\":{\"b\":[1]}}{\"a\":{\"b\":[1,9]}}\n[[1,2],[3]][[1,2,9],[3]]\n",
    );
}

/// Depth-3 appends and depth-3 element writes through widened locals.
#[test]
fn test_widened_locals_depth_three_writes() {
    assert_clean_run(
        r#"<?php
$c = [[[1]],[[3]]]; $d = $c; $d[0][0][] = 9; echo json_encode($c), json_encode($d), "\n";
$c = ["a"=>["b"=>["c"=>[1]]]]; $d = $c; $d["a"]["b"]["c"][] = 9; echo json_encode($c), json_encode($d), "\n";
function g() {
    $c = [[[1,2]]]; $d = $c; $d[0][0][1] = 5; $d[0][0][] = 6; echo json_encode($c), json_encode($d), "\n";
    $c = ["a"=>["b"=>["c"=>"s"]]]; $d = $c; $d["a"]["b"]["c"] = "t"; $d["a"]["b"]["e"] = "u";
    echo json_encode($c), json_encode($d), "\n";
}
g();
"#,
        "[[[1]],[[3]]][[[1,9]],[[3]]]\n{\"a\":{\"b\":{\"c\":[1]}}}{\"a\":{\"b\":{\"c\":[1,9]}}}\n\
[[[1,2]]][[[1,5,6]]]\n{\"a\":{\"b\":{\"c\":\"s\"}}}{\"a\":{\"b\":{\"c\":\"t\",\"e\":\"u\"}}}\n",
    );
}

/// Two-level key writes into a concrete map: the parent must keep the written child alive.
#[test]
fn test_concrete_map_nested_key_write_keeps_child() {
    assert_clean_run(
        r#"<?php
function h() {
    $d = 5;
    $d = ["a"=>["b"=>[1]]]; $d["a"]["b"] = [7]; echo json_encode($d), "\n";
    $e = ["p"=>["q"=>1]]; $e["p"]["r"] = 7; print_r($e);
}
h();
$m = ["a"=>["b"=>1]]; $m["a"]["c"] = 7; echo json_encode($m), "\n";
"#,
        "{\"a\":{\"b\":[7]}}\nArray\n(\n    [p] => Array\n        (\n            [q] => 1\n            [r] => 7\n        )\n\n)\n{\"a\":{\"b\":1,\"c\":7}}\n",
    );
}

/// Indexed writes and appends into a boxed-Mixed local replace the box without leaking it.
#[test]
fn test_indexed_writes_into_widened_local_do_not_leak() {
    assert_clean_run(
        r#"<?php
function f() {
    $c = [[1],[2]]; $d = $c; $d[0] = [5]; $d[] = [6]; $d[1][0] = 4;
    echo json_encode($c), json_encode($d), "\n";
    $c = 1; $d = 2;
}
f();
"#,
        "[[1],[2]][[5],[4],[6]]\n",
    );
}

/// A leaf value whose type differs from the chain's concrete element type widens the root to
/// Mixed elements instead of storing a payload later reads would decode with the wrong layout.
#[test]
fn test_nested_write_with_mismatched_leaf_type_widens_root() {
    assert_clean_run(
        r#"<?php
function f() {
    $c = ["a"=>["b"=>[1]]]; $d = $c; $d["a"]["b"] = 9; $d["a"]["c"] = 5; $d["a"]["n"] = ["z"=>1];
    echo json_encode($c), json_encode($d), "\n";
    $c = [[1,2],[3]]; $d = $c; $d[0][1] = "s"; $d[1][] = [4];
    echo json_encode($c), json_encode($d), "\n";
    $m = ["a"=>["b"=>1]]; $m["a"]["s"] = "str" . 1; echo json_encode($m), "\n";
    $c = 1; $d = 2;
}
f();
$t = [[[1]],[[3]]]; $t[0][0][0] = "x"; echo json_encode($t), "\n";
$t = ["a"=>["b"=>["c"=>[1]]]]; $t["a"]["b"]["c"] = 9; echo json_encode($t), "\n";
"#,
        "{\"a\":{\"b\":[1]}}{\"a\":{\"b\":9,\"c\":5,\"n\":{\"z\":1}}}\n\
[[1,2],[3]][[1,\"s\"],[3,[4]]]\n{\"a\":{\"b\":1,\"s\":\"str1\"}}\n[[[\"x\"]],[[3]]]\n{\"a\":{\"b\":{\"c\":9}}}\n",
    );
}

/// A Mixed element copied into a local, or reached through a copied parent, separates on write.
#[test]
fn test_mixed_element_copies_separate_on_write() {
    assert_clean_run(
        r#"<?php
function f() {
    $a = [1, [2, "x"]]; $x = $a[1]; $x[] = 9; $x[0] = 7;
    echo json_encode($a), json_encode($x), "\n";
    $c = [[1,[2]],[3]]; $d = $c; $d[0][1][] = 9; $d[0][1][0] = 5;
    echo json_encode($c), json_encode($d), "\n";
    $h = ["k" => [1, "s"]]; $g = $h; $g["k"][] = 2;
    echo json_encode($h), json_encode($g), "\n";
}
f();
"#,
        "[1,[2,\"x\"]][7,\"x\",9]\n[[1,[2]],[3]][[1,[5,9]],[3]]\n{\"k\":[1,\"s\"]}{\"k\":[1,\"s\",2]}\n",
    );
}

/// Appending a value of another type to an associative local (directly or through a nested
/// append) widens the table to `mixed` values instead of storing a mistyped payload.
#[test]
fn test_mismatched_append_into_assoc_local_widens_values() {
    assert_clean_run(
        r#"<?php
function f() {
    $c = ["a"=>["b"=>[1]]]; $d = $c; $d["a"][] = 9; echo json_encode($c), json_encode($d), "\n";
    $h = ["b"=>[1]]; $h[] = 9; $h[] = [2]; echo json_encode($h), "\n";
}
f();
"#,
        "{\"a\":{\"b\":[1]}}{\"a\":{\"b\":[1],\"0\":9}}\n{\"b\":[1],\"0\":9,\"1\":[2]}\n",
    );
}

/// A nested list literal stored into a loop-widened `array<mixed>` parent is re-boxed, so the
/// following nested append reads real cells.
#[test]
fn test_nested_literal_in_loop_matches_widened_storage() {
    assert_clean_run(
        r#"<?php
function f() {
    for ($i = 0; $i < 3; $i++) { $d = ["a"=>["b"=>[$i]]]; $e = $d; $d["a"]["b"][] = $i * 10; echo json_encode($e), json_encode($d), "\n"; }
    for ($i = 0; $i < 2; $i++) { $l = [[$i], [$i + 1]]; $l[1][] = $i * 3; echo json_encode($l), "\n"; }
}
f();
for ($i = 0; $i < 2; $i++) { $d = ["a"=>["b"=>[$i]]]; $d["a"]["b"][] = $i * 10; echo json_encode($d), "\n"; }
"#,
        "{\"a\":{\"b\":[0]}}{\"a\":{\"b\":[0,0]}}\n{\"a\":{\"b\":[1]}}{\"a\":{\"b\":[1,10]}}\n\
{\"a\":{\"b\":[2]}}{\"a\":{\"b\":[2,20]}}\n[[0],[1,0]]\n[[1],[2,3]]\n{\"a\":{\"b\":[0,0]}}\n{\"a\":{\"b\":[1,10]}}\n",
    );
}

/// Mismatched nested writes inside loop bodies widen the root once, before the loop, while
/// homogeneous matrix updates keep their concrete layout.
#[test]
fn test_mismatched_nested_writes_in_loops_widen_root_before_loop() {
    assert_clean_run(
        r#"<?php
function f() {
    $d = ["a"=>["b"=>[1]]];
    for ($i = 0; $i < 3; $i++) { $d["a"]["b"] = $i; $d["a"]["c$i"] = "s$i"; }
    echo json_encode($d), "\n";
    $m = [[1,2],[3]];
    for ($i = 0; $i < 2; $i++) { $m[$i][0] = "x$i"; }
    echo json_encode($m), "\n";
    $s = [["a"], ["b"]];
    for ($i = 0; $i < 2; $i++) { $s[$i][0] = $i; }
    echo json_encode($s), "\n";
    $h = ["k" => ["x" => 1]];
    foreach (["p", "q"] as $n) { $h["k"][$n] = [$n]; }
    echo json_encode($h), "\n";
    $g = [[1, 2], [3, 4]];
    for ($i = 0; $i < 2; $i++) { for ($j = 0; $j < 2; $j++) { $g[$i][$j] = $g[$i][$j] + $i + $j; } }
    echo json_encode($g), "\n";
}
f();
"#,
        "{\"a\":{\"b\":2,\"c0\":\"s0\",\"c1\":\"s1\",\"c2\":\"s2\"}}\n[[\"x0\",2],[\"x1\"]]\n[[0],[1]]\n\
{\"k\":{\"x\":1,\"p\":[\"p\"],\"q\":[\"q\"]}}\n[[1,3],[4,6]]\n",
    );
}

/// `var_dump` prints nested associative arrays instead of `NULL` for the inner table.
#[test]
fn test_var_dump_nested_assoc_array() {
    assert_clean_run(
        r#"<?php
$d = ["a"=>["b"=>1]]; var_dump($d);
function f() { $h = ["k"=>["l"=>1.5]]; var_dump($h); }
f();
"#,
        "array(1) {\n  [\"a\"]=>\n  array(1) {\n    [\"b\"]=>\n    int(1)\n  }\n}\n\
array(1) {\n  [\"k\"]=>\n  array(1) {\n    [\"l\"]=>\n    float(1.5)\n  }\n}\n",
    );
}
