//! Purpose:
//! Integration tests for `?ptr<T>` values: a nullable typed pointer is one raw pointer word whose
//! null is address 0, across returns, parameters, locals, properties and call results.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers `=== null`, `!== null`, `is_null()`, `isset()`, `??`, `echo` and `gettype()` on null
//!   and set pointers, packed record addresses flowing into `?ptr<T>` sinks, instance and static
//!   property defaults (literal copy and initializer thunk), method, static method and closure
//!   returns, ternary and match merges with `null`, extern calls behind a null guard, and the
//!   null-pointer dereference fatal (task 6da89f60).

use crate::support::*;

/// Verifies a `?ptr<T>` returned by a function or passed as a parameter keeps the pointer and
/// reads as PHP null when it is null, through the comparison, null and output builtins.
#[test]
fn test_nullable_pointer_returns_and_parameters_compare_with_null() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
function take(?ptr<Vec2> $p): int { return $p === null ? -1 : $p->x; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 7;
$p = first($v, true);
if ($p !== null) { echo $p->x, "|"; }
$q = first($v, false);
echo $q === null ? "null" : "set", "|", is_null($q) ? "null" : "set", "|";
echo isset($q) ? "set" : "unset", "|", isset($p) ? "set" : "unset", "\n";
$r = $q ?? first($v, true);
echo $r->x, "|", take(null), "|", take($v[0]), "|", take(ptr_null()), "\n";
echo "[", $q, "]|", gettype($q), "|", gettype($p), "\n";
$local = first($v, true);
$local = null;
echo is_null($local) ? "cleared" : "kept", "\n";
"#,
    );
    assert_eq!(out, "7|null|null|unset|set\n7|-1|7|-1\n[]|NULL|pointer\ncleared\n");
}

/// Verifies `?ptr<T>` instance and static properties start null (through the literal default
/// copy and through the class initializer thunk), store a packed record address, and clear.
#[test]
fn test_nullable_pointer_properties_default_store_and_clear() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
final class Holder { public ?ptr<Vec2> $p = null; }
final class Lazy { public ?ptr<Vec2> $p = null; public bool $wide = PHP_INT_SIZE > 4 ? true : false; }
final class Box { public static ?ptr<Vec2> $s = null; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 9;
$h = new Holder();
$l = new Lazy();
echo $h->p === null ? "empty" : "full", "|", $l->p === null ? "empty" : "full", "|";
echo Box::$s === null ? "empty" : "full", "\n";
$h->p = $v[0];
Box::$s = $v[0];
echo $h->p === null ? "empty" : $h->p->x, "|", Box::$s === null ? "empty" : Box::$s->x, "\n";
$h->p = null;
Box::$s = null;
echo is_null($h->p) ? "empty" : "full", "|", is_null(Box::$s) ? "empty" : "full", "\n";
"#,
    );
    assert_eq!(out, "empty|empty|empty\n9|9\nempty|empty\n");
}

/// Verifies method, static method and closure calls returning `?ptr<T>`, and ternary and match
/// merges of a record address with `null`, keep the null member of the result type.
#[test]
fn test_nullable_pointer_method_closure_and_merge_results() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
final class Finder {
    public function __construct(private buffer<Vec2> $v) {}
    public function at(bool $ok): ?ptr<Vec2> { return $ok ? $this->v[0] : null; }
    public static function none(): ?ptr<Vec2> { return null; }
}
$v = buffer_new<Vec2>(1);
$v[0]->x = 5;
$f = new Finder($v);
$a = $f->at(true);
$b = $f->at(false);
echo $a === null ? "null" : $a->x, "|", $b === null ? "null" : "set", "|";
echo Finder::none() === null ? "null" : "set", "\n";
$get = function (bool $ok) use ($v): ?ptr<Vec2> { return $ok ? $v[0] : null; };
$c = $get(false);
$d = $get(true);
echo $c === null ? "null" : "set", "|", $d === null ? "null" : $d->x, "\n";
$t = $argc > 5 ? $v[0] : null;
$m = match ($argc) { 1 => null, default => $v[0] };
echo $t === null ? "null" : "set", "|", $m === null ? "null" : "set", "\n";
"#,
    );
    assert_eq!(out, "5|null|null\nnull|5\nnull|null\n");
}

/// Verifies a guarded `?ptr<T>` local passes to an extern `ptr<T>` parameter as the raw address.
#[test]
fn test_nullable_pointer_passes_to_extern_after_null_guard() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
extern function memset(ptr<Vec2> $target, int $byte, u64 $size): ptr;
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 3;
$p = first($v, true);
if ($p !== null) { memset($p, 0, 16); }
echo $v[0]->x, "\n";
"#,
    );
    assert_eq!(out, "0\n");
}

/// Verifies writing through a null `?ptr<T>` reports the null-pointer dereference fatal.
#[test]
fn test_nullable_pointer_null_dereference_reports_runtime_error() {
    let err = compile_and_run_expect_failure(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
$v = buffer_new<Vec2>(1);
$q = first($v, false);
$q->x = 1;
"#,
    );
    assert!(err.contains("Fatal error: null pointer dereference"));
}

/// Verifies freeing an object whose `?ptr<T>` property still holds an interior record address
/// leaves the memory before that address alone. The slot used to be described as a Mixed cell,
/// so object teardown decref'd the address and, on AArch64, rewrote the preceding record; the
/// property then appears in `json_encode` as its integer address like a `ptr` property.
#[test]
fn test_nullable_pointer_property_teardown_does_not_release_the_address() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
final class Holder { public ?ptr<Vec2> $p = null; }
function plant(buffer<Vec2> $v): void {
    $h = new Holder();
    $h->p = $v[1];
}
$v = buffer_new<Vec2>(2);
$v[0]->x = 0;
$v[0]->y = 1;
plant($v);
echo $v[0]->x, "|", $v[0]->y, "|", json_encode(new Holder()), "\n";
"#,
    );
    assert_eq!(out, "0|1|{\"p\":0}\n");
}

/// Verifies a `?ptr<T>` boxed into `mixed` (a parameter or a property) boxes a null pointer as
/// PHP null and a set one as its integer address, and that string conversions of a null
/// pointer give the empty string `echo` prints.
#[test]
fn test_nullable_pointer_boxes_and_converts_null_as_php_null() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
final class Bag { public mixed $m = 1; }
function show(mixed $m): void { echo $m === null ? "null" : gettype($m), "|"; }
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
$v = buffer_new<Vec2>(1);
show(first($v, false));
show(first($v, true));
$b = new Bag();
$b->m = first($v, false);
echo $b->m === null ? "null" : "set", "|";
$b->m = first($v, true);
echo $b->m === null ? "null" : "set", "\n";
$q = first($v, false);
echo "x{$q}y|", "a" . $q . "b|", (string)$q, "|", sprintf("%s", $q), "|", strlen((string)$q), "|";
echo strlen("a" . first($v, true)) > 3 ? "address" : "empty", "\n";
"#,
    );
    assert_eq!(out, "null|integer|null|set\nxy|ab|||0|address\n");
}

/// Verifies `(bool)`, `boolval()`, `xor` and `empty()` test a `?ptr<T>` word against 0 instead
/// of reading the record it points to as a Mixed cell (a live record of zeros read as falsy,
/// and one starting with 7 faulted).
#[test]
fn test_nullable_pointer_truthiness_and_empty_test_the_address() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
function second(buffer<Vec2> $v): ?ptr<Vec2> { return $v[1]; }
$v = buffer_new<Vec2>(2);
$v[1]->x = 7;
$v[1]->y = 1;
$zero = first($v, true);
$live = second($v);
$none = first($v, false);
echo ((bool)$zero) ? "y" : "n", boolval($live) ? "y" : "n", ((bool)$none) ? "y" : "n", "|";
echo empty($zero) ? "e" : "s", empty($live) ? "e" : "s", empty($none) ? "e" : "s", "|";
echo ($zero xor $none) ? "xor" : "same", "\n";
"#,
    );
    assert_eq!(out, "yyn|sse|xor\n");
}

/// Verifies a merge whose null arm is a `?ptr<T>` local, instance property or static property
/// (ternary, `??` and `?:`) stays nullable, so `=== null` and `echo` see PHP null.
#[test]
fn test_nullable_pointer_merges_keep_null_from_locals_and_properties() {
    let out = compile_and_run(
        r#"<?php
packed class Vec2 { public int $x; public int $y; }
final class Holder { public ?ptr<Vec2> $p = null; public static ?ptr<Vec2> $s = null; }
function first(buffer<Vec2> $v, bool $ok): ?ptr<Vec2> { return $ok ? $v[0] : null; }
$v = buffer_new<Vec2>(1);
$v[0]->x = 3;
$q = first($v, false);
$t = $argc > 5 ? $v[0] : $q;
echo $t === null ? "null" : "set", "[", $t, "]|";
$h = new Holder();
$u = $argc > 5 ? $v[0] : $h->p;
echo $u === null ? "null" : "set", "|";
$s = $argc > 5 ? $v[0] : Holder::$s;
echo $s === null ? "null" : "set", "|";
$w = $q ?? $q;
echo $w === null ? "null" : "set", "|";
$x = $q ?: null;
echo $x === null ? "null" : "set", "|";
$y = $q ?? $v[0];
echo $y === null ? "null" : $y->x, "\n";
"#,
    );
    assert_eq!(out, "null[]|null|null|null|null|3\n");
}
