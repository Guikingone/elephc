//! Purpose:
//! Integration tests for arrays and hashes of pointers: `ptr<T>` values, packed record addresses
//! and `null` stored as raw pointer words in indexed arrays and hashes.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers literals, appends, index and key writes, foreach over arrays and hashes, `null`
//!   elements and missing keys (both read back as PHP null), untyped `ptr` elements, and
//!   `array_push`/`array_pop` (task 6da89f60).

use crate::support::*;

/// Packed record and buffer setup shared by the pointer array tests.
const PRELUDE: &str = r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
$v = buffer_new<Vec2>(3);
$v[0]->x = 10; $v[1]->x = 11; $v[2]->x = 12;
"#;

/// Verifies an array literal of `ptr<T>` values and packed record addresses keeps the records
/// reachable through element reads and foreach, and writes through an element reach the buffer.
#[test]
fn test_pointer_array_literals_reach_their_records() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$p = at($v, 0); $q = at($v, 1);
$a = [$p, $q];
echo count($a), "|", $a[1]->x, "|", $a[0]->x, "\n";
$b = [$v[0], $v[1], $v[2]];
foreach ($b as $i => $r) { echo $i, ":", $r->x, " "; }
echo "\n";
$b[1]->x = 99;
echo $v[1]->x, "\n";
"#
    ));
    assert_eq!(out, "2|11|10\n0:10 1:11 2:12 \n99\n");
}

/// Verifies appends and index writes of pointers and packed record addresses keep pointer
/// storage instead of widening the array to boxed `mixed`.
#[test]
fn test_pointer_array_appends_and_index_writes() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$a = [];
$a[] = at($v, 0);
$a[] = $v[1];
$a[] = $v[2];
echo count($a), "|", $a[1]->x, "|", $a[2]->x, "\n";
$a[0] = $v[2];
echo $a[0]->x, "\n";
"#
    ));
    assert_eq!(out, "3|11|12\n12\n");
}

/// Verifies hashes of pointers store literal, call and packed values, accept key writes of a
/// record address and `null`, and bind the pointer (or null) in foreach.
#[test]
fn test_pointer_hashes_store_and_iterate() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$h = ["k" => $v[0], "j" => at($v, 1)];
$h["m"] = $v[2];
echo count($h), "|", $h["k"]->x, "|", $h["j"]->x, "|", $h["m"]->x, "\n";
$g = ["a" => $v[0]];
$g["b"] = null;
foreach ($g as $k => $p) { echo $k, ":", $p === null ? "null" : $p->x, " "; }
echo "\n";
"#
    ));
    assert_eq!(out, "3|10|11|12\na:10 b:null \n");
}

/// Verifies `null` elements beside pointers stay pointer storage and read back as PHP null, as
/// do `?ptr<T>` call results and a missing index, while live elements keep their records.
#[test]
fn test_pointer_arrays_hold_null_elements() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"function maybe(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
$a = [maybe($v, true), maybe($v, false)];
echo $a[0] === null ? "null" : $a[0]->x, "|", $a[1] === null ? "null" : "set", "|";
echo is_null($a[1]) ? "null" : "set", "\n";
$b = [$v[0], null];
echo $b[1] === null ? "null" : "set", "|", $b[0]->x, "\n";
$c = [$v[0], $v[1]];
$c[] = null;
echo count($c), "|", $c[2] === null ? "null" : "set", "|", $c[0]->x, "|";
$miss = @$c[7];
echo $miss === null ? "miss" : "hit", "|", isset($c[7]) ? "isset" : "unset", "|";
echo isset($c[1]) ? "isset" : "unset", "\n";
"#
    ));
    assert_eq!(out, "10|null|null\nnull|10\n3|null|10|miss|unset|isset\n");
}

/// Verifies arrays and hashes of untyped `ptr` values from `ptr_null()` and an extern call keep
/// the raw addresses (the null pointer reads back as PHP null).
#[test]
fn test_untyped_pointer_arrays_keep_raw_addresses() {
    let out = compile_and_run(
        r#"<?php
extern function malloc(u64 $size): ptr;
extern function free(ptr $p): void;
$raw = malloc(8);
$a = [ptr_null(), $raw];
echo ptr_is_null($a[0]) ? "null" : "set", "|", ptr_is_null($a[1]) ? "null" : "set", "|";
echo $a[0] === null ? "null" : "set", "\n";
$h = ["r" => $raw];
echo $h["r"] === $raw ? "same" : "diff", "\n";
free($raw);
"#,
    );
    assert_eq!(out, "null|set|null\nsame\n");
}

/// Verifies `array_push()` and `array_pop()` on an array of pointers move the raw words, and
/// the popped value is the pointer.
#[test]
fn test_pointer_arrays_push_and_pop_builtins() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$a = [at($v, 0)];
array_push($a, at($v, 1), at($v, 2));
echo count($a), "|", $a[2]->x, "|";
$last = array_pop($a);
echo count($a), "|", $last === null ? "null" : "set", "\n";
"#
    ));
    assert_eq!(out, "3|12|2|set\n");
}

/// Verifies a foreach over pointers with a `null` element binds `?ptr<T>`, so a `!== null`
/// guard skips the null element and the others write through to their records.
#[test]
fn test_pointer_array_foreach_skips_null_elements() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
$points = buffer_new<Vec2>(3);
$picked = [$points[0], $points[2]];
$picked[] = null;
foreach ($picked as $p) {
    if ($p !== null) {
        $p->x += 1;
    }
}
echo $points[0]->x, "|", $points[1]->x, "|", $points[2]->x, "\n";
"#,
    );
    assert_eq!(out, "1|0|1\n");
}
