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
//! - Hash literals that mix pointers with `null` or with other targets are tested at top level,
//!   where `main` is lowered from the checker's finished environment.

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

/// Verifies a missing hash key of a pointer hash reads as the null pointer word: `=== null`
/// holds, a guarded field read is skipped, and `??` takes its default.
#[test]
fn test_pointer_hash_missing_key_reads_as_null() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$h = ["k" => $v[0]];
$miss = @$h["nope"];
echo $miss === null ? "null" : "wild", "|";
if ($miss !== null) { echo $miss->x; }
$d = $h["nope"] ?? $v[1];
echo $d->x, "\n";
"#
    ));
    assert_eq!(out, "null|11\n");
}

/// Verifies `array_pop()` and `array_shift()` of pointers give the pointer word back: field
/// access, a `?ptr<T>` parameter, an empty pop as null, and a literal element of the result.
#[test]
fn test_pointer_array_pop_and_shift_return_the_pointer() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"function show(?ptr<Vec2> $p): void { echo $p === null ? "null" : $p->x, "|"; }
$a = [at($v, 0), at($v, 1), at($v, 2)];
show(array_pop($a));
$first = array_shift($a);
echo $first->x, "|";
$last = array_pop($a);
echo $last->x, "|";
$none = array_pop($a);
echo $none === null ? "null" : "set", "|";
$b = [at($v, 2)];
$c = [array_pop($b)];
echo $c[0]->x, "\n";
"#
    ));
    assert_eq!(out, "12|10|11|null|12\n");
}

/// Verifies appending a pointer with another target keeps raw pointer storage (an untyped
/// `ptr` element) instead of boxing the array, so the stored addresses stay identical.
#[test]
fn test_pointer_array_of_mixed_targets_keeps_raw_addresses() {
    let out = compile_and_run(
        r#"<?php
packed class A { public int $x; }
packed class B { public int $y; }
$a = buffer_new<A>(1);
$b = buffer_new<B>(1);
$ptrs = [$a[0]];
$ptrs[] = $b[0];
function id(?ptr $p): ?ptr { return $p; }
echo id($ptrs[0]) === $a[0] ? "same" : "diff", "|", id($ptrs[1]) === $b[0] ? "same" : "diff";
echo "|", count($ptrs), "\n";
"#,
    );
    assert_eq!(out, "same|same|2\n");
}

/// Verifies `isset()` treats a stored null pointer element (indexed or hash) as unset and a
/// live one as set.
#[test]
fn test_pointer_array_isset_of_stored_null_is_false() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$b = [$v[0], null];
echo isset($b[1]) ? "isset" : "unset", "|", isset($b[0]) ? "isset" : "unset", "|";
$h = ["a" => $v[0]];
$h["b"] = null;
echo isset($h["b"]) ? "isset" : "unset", "|", isset($h["a"]) ? "isset" : "unset", "\n";
"#
    ));
    assert_eq!(out, "unset|isset|unset|isset\n");
}

/// Verifies `array_push()` of packed record addresses and `null` keeps raw pointer storage like
/// `$a[] = ...`, and a `null` appended to a pointer hash keeps the existing records reachable.
#[test]
fn test_pointer_array_push_of_records_and_null_and_hash_null_append() {
    let out = compile_and_run(&format!(
        "{PRELUDE}{}",
        r#"$a = [];
array_push($a, $v[1]);
$b = [$v[0]];
array_push($b, $v[2], null);
$a[0]->x = 21;
echo $v[1]->x, "|", $b[1]->x, "|", $b[2] === null ? "null" : "set", "|", count($b), "|";
$h = ["a" => $v[0]];
$h[] = null;
echo $h["a"]->x, "|", count($h), "\n";
"#
    ));
    assert_eq!(out, "21|12|null|3|10|2\n");
}

/// Verifies `array_push()` into a typed `array<ptr<T> >` property (the generic builtin path, not
/// the local fast path) stores a packed record address and `null` as pointer words, and that
/// the property's empty-array default has pointer storage.
#[test]
fn test_pointer_array_push_into_typed_property() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
class Bag { public array<ptr<Vec2> > $items = []; }
$v = buffer_new<Vec2>(2);
$v[0]->x = 5;
$b = new Bag();
array_push($b->items, $v[0]);
array_push($b->items, null);
echo count($b->items), "|", $b->items[0]->x, "|", $b->items[1] === null ? "null" : "set", "\n";
"#,
    );
    assert_eq!(out, "2|5|null\n");
}

/// Verifies a null-only literal bound to an `array<ptr<T>|null>` destination (a property
/// default, a static default, a parameter, a property or static assignment, a hash literal)
/// keeps pointer storage, so a later pointer append and both reads see the right words.
#[test]
fn test_pointer_arrays_from_null_only_literals() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
class Bag {
    public array<ptr<Vec2>|null> $items = [null];
    public static array<ptr<Vec2>|null> $shared = [null];
    public array<string, ptr<Vec2>|null> $named = ["a" => null];
}
function fill(array<ptr<Vec2>|null> $items): void {
    $v = buffer_new<Vec2>(1);
    $v[0]->x = 7;
    $items[] = $v[0];
    echo $items[0] === null ? "null" : "set", ":", $items[1]->x, "|";
}
$b = new Bag();
fill($b->items);
fill([null]);
$b->items = [null];
$v = buffer_new<Vec2>(1);
$v[0]->x = 8;
$b->items[] = $v[0];
echo $b->items[1]->x, "|";
Bag::$shared[] = $v[0];
echo Bag::$shared[0] === null ? "null" : "set", ":", Bag::$shared[1]->x, "|";
Bag::$shared = [null];
echo count(Bag::$shared), "|";
$b->named["b"] = $v[0];
echo $b->named["a"] === null ? "null" : "set", ":", $b->named["b"]->x, "\n";
"#,
    );
    assert_eq!(out, "null:7|null:7|8|null:8|1|null:8\n");
}

/// Verifies a top-level associative literal that mixes a pointer with `null`, or two pointer
/// targets, keeps pointer-word storage: the null element reads as PHP null, the live one reaches
/// its record, and two targets keep their addresses (an untyped `ptr` element).
///
/// Top level on purpose: `main` is lowered from the checker's finished environment, so a literal
/// the checker typed `array<string, mixed>` was converted element by element into mixed integers.
#[test]
fn test_pointer_hash_literals_mixing_null_and_targets_keep_pointer_words() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
packed class A { public int $x; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function id(?ptr $p): ?ptr { return $p; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 10;
$p = at($v, 0);
$h = ["a" => $p, "b" => null];
echo $h["b"] === null ? "null" : "wild", "|", $h["a"]->x, "|";
$a = buffer_new<A>(1);
$t = ["v" => $v[0], "a" => $a[0]];
echo id($t["v"]) === $v[0] ? "same" : "diff", id($t["a"]) === $a[0] ? "same" : "diff", "\n";
"#,
    );
    assert_eq!(out, "null|10|samesame\n");
}

/// Verifies spread and mixed literals keep pointer-word storage when they combine pointers with
/// different targets or packed record addresses: an associative spread beside a keyed or a plain
/// pointer keeps both addresses, and a packed address beside a spread reaches its record.
///
/// Top level on purpose, like the hash literal test: `main` follows the checker's types.
#[test]
fn test_pointer_spread_literals_keep_targets_and_record_addresses() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
packed class A { public int $x; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function ida(buffer<A> $a, int $i): ptr<A> { return $a[$i]; }
function id(?ptr $p): ?ptr { return $p; }
$v = buffer_new<Vec2>(2);
$v[0]->x = 4;
$v[1]->x = 5;
$a = buffer_new<A>(1);
$tail = ["a" => ida($a, 0)];
$h = ["v" => at($v, 0), ...$tail];
echo id($h["v"]) === $v[0] ? "same" : "diff", id($h["a"]) === $a[0] ? "same" : "diff", "|";
$b = [at($v, 0), ...$tail];
echo id($b[0]) === $v[0] ? "same" : "diff", id($b["a"]) === $a[0] ? "same" : "diff", "|";
$t = [$v[1]];
$c = [$v[0], ...$t];
echo $c[0]->x, $c[1]->x, "|";
$th = ["b" => $v[1]];
$d = ["a" => $v[0], ...$th];
echo $d["a"]->x, $d["b"]->x, "\n";
"#,
    );
    assert_eq!(out, "samesame|samesame|45|45\n");
}

/// Verifies `null` in a spread or mixed literal of pointers is the null pointer word: a spread of
/// a null-only literal (indexed or associative), a keyed `null` beside a spread, and a `null`
/// element before a spread of record addresses all read back as PHP null.
#[test]
fn test_pointer_spread_literals_store_null_as_the_null_pointer() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 10;
$p = at($v, 0);
$a = [$p, ...[null]];
echo $a[1] === null ? "null" : "wild", $a[0]->x, "|";
$h = ["a" => $p, ...["b" => null]];
echo $h["b"] === null ? "null" : "wild", $h["a"]->x, "|";
$tail = ["c" => $p];
$m = ["a" => $p, "b" => null, ...$tail];
echo $m["b"] === null ? "null" : "wild", $m["c"]->x, "|";
$n = [$v[0], null, ...[$v[0]]];
echo $n[1] === null ? "null" : "wild", $n[2]->x, "\n";
"#,
    );
    assert_eq!(out, "null10|null10|null10|null10\n");
}

/// Verifies an indexed pointer array spread into a pointer hash keeps its raw words: the
/// promotion to hash storage must not box them, so a `null` element that came from an indexed
/// spread is still unset for `isset()` (a boxed null cell is a set value), and record addresses
/// beside a key or after an associative spread read back correctly.
#[test]
fn test_indexed_pointer_spreads_into_pointer_hashes_keep_raw_words() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
$v = buffer_new<Vec2>(2);
$v[0]->x = 4;
$v[1]->x = 5;
$p = at($v, 0);
$t = [$v[1], null];
$d = ["a" => $v[0], ...$t];
echo $d["a"]->x, $d[0]->x, isset($d[1]) ? "set" : "unset", "|";
$h = ["a" => $p, ...[null]];
echo isset($h[0]) ? "set" : "unset", $h["a"]->x, "|";
$th = ["b" => $v[0]];
$e = [...$th, ...$t];
echo $e["b"]->x, $e[0]->x, isset($e[1]) ? "set" : "unset", "\n";
"#,
    );
    assert_eq!(out, "45unset|unset4|45unset\n");
}

/// Verifies a null-only literal passed as a named argument to an `array<ptr<T>|null>` parameter
/// keeps pointer storage like a positional one, so `isset()` of its null element is false for a
/// function and a method call, and beside an associative unpack (`f(...$bag, items: [null])`,
/// which lowers through the normalized-argument fallback) for an array and a hash parameter.
#[test]
fn test_pointer_array_named_argument_null_only_literal() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function probe(array<ptr<Vec2>|null> $items): string { return isset($items[0]) ? "set" : "unset"; }
class Box {
    public function probe(array<ptr<Vec2>|null> $items): string { return isset($items[0]) ? "set" : "unset"; }
}
function counted(int $n, array<ptr<Vec2>|null> $items): string { return isset($items[0]) ? "set" : "unset"; }
function keyed(int $n, array<string, ptr<Vec2>|null> $items): string { return isset($items["a"]) ? "set" : "unset"; }
echo probe([null]), probe(items: [null]), (new Box())->probe(items: [null]), "|";
$bag = ["n" => 1];
echo counted(...$bag, items: [null]), keyed(...$bag, items: ["a" => null]), "\n";
"#,
    );
    assert_eq!(out, "unsetunsetunset|unsetunset\n");
}

/// Verifies a `[null]` default of an `array<ptr<T>|null>` parameter keeps pointer storage when an
/// associative unpack leaves that parameter out: the call synthesizes a `key exists ? value :
/// default` ternary for it, whose default branch must still be built against the parameter, so
/// `isset()` of the null element is false with and without a trailing named argument.
#[test]
fn test_pointer_array_default_behind_an_associative_unpack() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function probe(array<ptr<Vec2>|null> $items = [null], array<ptr<Vec2>|null> $more = []): string {
    return (isset($items[0]) ? "set" : "unset") . count($more);
}
function tagged(array<ptr<Vec2>|null> $items = [null], array<ptr<Vec2>|null> $more = [], int $n = 0): string {
    return (isset($items[0]) ? "set" : "unset") . count($more) . $n;
}
$v = buffer_new<Vec2>(1);
$bag = ["more" => [at($v, 0)]];
echo probe(...$bag), "|", tagged(...$bag, n: 7), "\n";
"#,
    );
    assert_eq!(out, "unset1|unset17\n");
}

/// Verifies a pointer array or hash built by a conditional keeps its null words: a ternary, a
/// nested ternary, `?:`, `match` and `??` merging a record address with `[null]` hand the callee
/// an unset null element, field reads through either arm and a `foreach` reach the right values.
///
/// Top level on purpose (`$argc` is 1), so no branch folds away.
#[test]
fn test_pointer_array_conditional_literals_keep_null_words() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function probe(array<ptr<Vec2>|null> $items): string { return isset($items[0]) ? "set" : "unset"; }
function field(array<ptr<Vec2>|null> $items): string { $p = $items[0]; return $p === null ? "null" : (string) $p->x; }
function hprobe(array<string, ptr<Vec2>|null> $items): string { return isset($items["a"]) ? "set" : "unset"; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 9;
$maybe = $argc > 100 ? [at($v, 0)] : null;
echo probe($argc > 100 ? [at($v, 0)] : [null]),
    probe($argc > 0 ? ($argc > 100 ? [null] : [null]) : [null]),
    probe(($argc > 100 ? [at($v, 0)] : []) ?: [null]),
    probe(match ($argc) { 0 => [at($v, 0)], default => [null] }),
    probe($maybe ?? [null]),
    hprobe($argc > 100 ? ["a" => at($v, 0)] : ["a" => null]), "|";
echo field($argc > 0 ? [at($v, 0)] : [null]), field($argc > 100 ? [at($v, 0)] : [null]), "|";
foreach (($argc > 0 ? [at($v, 0), null] : [null]) as $q) { echo $q === null ? "N" : $q->x; }
echo "\n";
"#,
    );
    assert_eq!(out, "unsetunsetunsetunsetunsetunset|9null|9N\n");
}

/// Verifies `[null]` stays the null pointer word where it is returned, spread or assigned: a
/// `return [null]` from `: array<ptr<T>|null>` compiles, a returned ternary and a call result
/// read through `isset()` are unset, as are `[...[null]]` and a ternary assigned to a property.
#[test]
fn test_pointer_array_null_literals_through_returns_spreads_and_properties() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
class Bag { public array<ptr<Vec2>|null> $items = []; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function probe(array<ptr<Vec2>|null> $items): string { return isset($items[0]) ? "set" : "unset"; }
function make(buffer<Vec2> $v, int $n): array<ptr<Vec2>|null> { return $n > 100 ? [at($v, 0)] : [null]; }
function nul(): array<ptr<Vec2>|null> { return [null]; }
$v = buffer_new<Vec2>(1);
$b = new Bag();
$b->items = $argc > 100 ? [at($v, 0)] : [null];
echo probe([...[null]]), probe(make($v, $argc)), probe(nul()), "|";
echo isset(make($v, $argc)[0]) ? "set" : "unset", isset(nul()[0]) ? "set" : "unset", isset($b->items[0]) ? "set" : "unset", "\n";
"#,
    );
    assert_eq!(out, "unsetunsetunset|unsetunsetunset\n");
}

/// Verifies a null-only literal held in a local reaches a pointer-array slot as pointer words:
/// returned from `: array<ptr<T>|null>`, passed positionally or by name, or stored in a typed
/// property, its element reads, `isset()` and `foreach` all see the null pointer, not the box.
#[test]
fn test_pointer_array_slots_convert_boxed_null_locals() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
class Bag { public array<ptr<Vec2>|null> $items = []; }
function nul(): array<ptr<Vec2>|null> { $a = [null]; return $a; }
function walk(array<ptr<Vec2>|null> $items): string {
    $o = "";
    foreach ($items as $p) { $o .= $p === null ? "N" : "L"; }
    return $o;
}
function first(array<ptr<Vec2>|null> $items): string {
    $p = $items[0];
    return ($p === null ? "N" : "L") . (isset($items[0]) ? "s" : "u");
}
foreach (nul() as $p) { echo $p === null ? "N" : "L"; }
$a = [null];
$b = new Bag();
$b->items = $a;
echo "|", walk($a), walk(nul()), walk(items: $a), walk($b->items), "|", first($a), first(nul()), "\n";
"#,
    );
    assert_eq!(out, "N|NNNN|NuNu\n");
}

/// Verifies a pointer array or hash passed to an `array<mixed>` or `array<string, mixed>`
/// parameter is widened to boxed elements, so through that parameter a null element is PHP null
/// (unset, `json_encode` null) and a live one an integer address.
#[test]
fn test_pointer_containers_widen_into_mixed_element_parameters() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function at(buffer<Vec2> $v, int $i): ptr<Vec2> { return $v[$i]; }
function hm(array<string, mixed> $a): string {
    return (isset($a["a"]) ? "set" : "unset") . json_encode($a["a"]) . gettype($a["b"]);
}
function am(array<mixed> $a): string {
    return (isset($a[0]) ? "set" : "unset") . json_encode($a[0]) . gettype($a[1]);
}
$v = buffer_new<Vec2>(1);
$h = ["a" => null, "b" => at($v, 0)];
$l = [null, at($v, 0)];
echo hm($h), "|", am($l), "|", isset($h["a"]) ? "set" : "unset", $h["b"] === null ? "null" : "live", "\n";
"#,
    );
    assert_eq!(out, "unsetnullinteger|unsetnullinteger|unsetlive\n");
}
