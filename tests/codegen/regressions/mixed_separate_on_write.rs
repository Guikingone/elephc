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
//!   `mixed` parameters, static locals, element reads, and `unset($mixed[$k])` (including its
//!   `ArrayAccess`/string/scalar runtime dispatch).
//! - Nested appends through `mixed` elements must stay amortized O(1); that is asserted through
//!   heap-debug allocation counts rather than timings.
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

/// Nested appends through a boxed `mixed` element (`$a["l"][] = $v`) write in place through
/// fetch-for-write, but copies of the root taken before the write (typed hash, list, and `mixed`
/// roots, depth 2 and 3, property roots, by-value `mixed` parameters) never observe the write.
#[test]
fn test_mixed_nested_append_keeps_copies_independent() {
    assert_heap_clean(
        r#"<?php
function mk(): mixed { return ['l' => [1], 'n' => ['x' => [2]]]; }
function run() {
    // typed hash root, depth 2 and 3
    $loc = ['l' => [], 'm' => 'x', 'n' => ['k' => []]];
    $copy = $loc; $loc['l'][] = 1; $loc['n']['k'][] = 2;
    echo json_encode($copy), json_encode($loc), "|";
    // list root
    $lst = [[1], 'x', [[2]]];
    $c2 = $lst; $lst[0][] = 3; $lst[2][0][] = 4;
    echo json_encode($c2), json_encode($lst), "|";
    // mixed root
    $m = mk(); $c3 = $m; $m['l'][] = 5; $m['n']['x'][] = 6; $m['new'][] = 7;
    echo json_encode($c3), json_encode($m), "|";
    // element read keeps its value
    $e = $m['l']; $m['l'][] = 8; echo json_encode($e), json_encode($m['l']), "|";
    // loop of appends, then copy in between
    $acc = ['l' => []];
    for ($i = 0; $i < 4; $i++) { $acc['l'][] = $i; if ($i == 1) { $snap = $acc; } }
    echo json_encode($snap), json_encode($acc), "|";
    // value expression reading the same element
    $self = ['l' => [1]]; $self['l'][] = $self['l']; echo json_encode($self);
}
run();
echo "\n";
class H { public array $d; function __construct() { $this->d = ['l' => [], 'm' => 'x']; }
  function add($v) { $this->d['l'][] = $v; } }
$h = new H; $h->add(1); $snap = $h->d; $h->add(2); $h->d['l'][] = 3;
echo json_encode($snap), json_encode($h->d), "\n";
function f(mixed $cfg) { $cfg['items'][] = [1]; return $cfg; }
$cfg = ['items' => []]; $r = f($cfg); echo json_encode($cfg), json_encode($r), "\n";
"#,
        r#"{"l":[],"m":"x","n":{"k":[]}}{"l":[1],"m":"x","n":{"k":[2]}}|[[1],"x",[[2]]][[1,3],"x",[[2,4]]]|{"l":[1],"n":{"x":[2]}}{"l":[1,5],"n":{"x":[2,6]},"new":[7]}|[1,5][1,5,8]|{"l":[0,1]}{"l":[0,1,2,3]}|{"l":[1,[1]]}
{"l":[1],"m":"x"}{"l":[1,2,3],"m":"x"}
{"items":[]}{"items":[[1]]}
"#,
    );
}

/// A nested append through a boxed `mixed` element must stay amortized O(1): it must not clone
/// the element cell and copy its whole payload on every append. Counted through heap-debug
/// allocations, so the check is independent of machine speed (a per-append copy would allocate
/// at least three blocks per append instead of one or two).
#[test]
fn test_mixed_nested_append_does_not_copy_payload_per_append() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class H { public array $d; function __construct() { $this->d = ["l" => [], "m" => "x"]; } }
function cfg(mixed $cfg) { for ($i = 0; $i < 1000; $i++) { $cfg["items"][] = $i; } return count($cfg["items"]); }
function run() {
    $loc = ["l" => [], "m" => "x"];
    for ($i = 0; $i < 1000; $i++) { $loc["l"][] = $i; }
    $k = ["n" => ["a" => ["b" => []]]];
    for ($i = 0; $i < 1000; $i++) { $k["n"]["a"]["b"][] = $i; }
    $h = new H;
    for ($i = 0; $i < 1000; $i++) { $h->d["l"][] = $i; }
    echo count($loc["l"]), ",", count($k["n"]["a"]["b"]), ",", count($h->d["l"]), ",", cfg(["items" => []]);
}
run();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1000,1000,1000,1000");
    let (allocs, _frees) = parse_heap_debug_allocs(&out.stderr);
    assert!(
        allocs < 10_000,
        "nested appends allocated {allocs} blocks for 4000 appends: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// Parses `HEAP DEBUG: allocs=N frees=M` from heap-debug stderr.
fn parse_heap_debug_allocs(stderr: &str) -> (u64, u64) {
    let line = stderr
        .lines()
        .find(|line| line.starts_with("HEAP DEBUG: allocs="))
        .unwrap_or_else(|| panic!("missing heap debug stats: {stderr}"));
    let field = |name: &str| -> u64 {
        line.split(name)
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| panic!("bad heap debug stats: {line}"))
    };
    (field("allocs="), field("frees="))
}

/// `unset($mixed[$k])` dispatches on the runtime payload like PHP: `ArrayAccess` objects call
/// `offsetUnset`, arrays drop the key, strings and other scalars throw the matching `Error`,
/// non-`ArrayAccess` objects throw, and `null`/`false` stay quiet no-ops.
#[test]
fn test_mixed_unset_dispatches_on_runtime_payload() {
    let out = compile_and_run(
        r#"<?php
class AA implements ArrayAccess {
    function offsetExists(mixed $o): bool { return true; }
    function offsetGet(mixed $o): mixed { return 1; }
    function offsetSet(mixed $o, mixed $v): void {}
    function offsetUnset(mixed $o): void { echo "unset $o\n"; }
}
function g(mixed $m, mixed $k) { unset($m[$k]); return $m; }
g(new AA, 'a');
function h(AA|array $m) { unset($m['zz']); return $m; }
h(new AA);
echo json_encode(h(['zz' => 1, 'y' => 2])), "\n";
try { g("str", 0); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { g(12, 0); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { g(1.5, 0); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { g(true, 0); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { g(new stdClass, 0); } catch (Error $e) { echo $e->getMessage(), "\n"; }
var_dump(g(null, 0));
var_dump(g(false, 0));
echo json_encode(g([1, 2, 3], 1)), json_encode(g(['a' => 1, 'b' => 2], 'a')), "\n";
"#,
    );
    assert_eq!(out, r#"unset a
unset zz
{"y":2}
Cannot unset string offsets
Cannot unset offset in a non-array variable
Cannot unset offset in a non-array variable
Cannot unset offset in a non-array variable
Cannot use object of type stdClass as array
NULL
bool(false)
{"0":1,"2":3}{"b":2}
"#);
}
