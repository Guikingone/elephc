//! Purpose:
//! Integration tests for the gradual-typing boundary model: Mixed/union values flowing into
//! concretely-typed parameters/returns get a runtime boundary guard, reassignment widens a
//! local's type via a flow-insensitive join, and by-reference arguments promote caller storage.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - The null-into-typed-object case asserts a runtime `TypeError` fatal (non-zero exit), so it
//!   uses `compile_and_run_capture`. Other cases assert coerced stdout via `compile_and_run`.
//! - `$argc` keeps a value runtime-unknown so AST folding cannot collapse the gradual construct.

use super::*;

/// A `Mixed` value (read from an associative array) flowing into a `string` parameter is
/// accepted and coerced to a native string at the call boundary.
#[test]
fn test_mixed_into_string_param_is_coerced() {
    let out = compile_and_run(
        "<?php
        function f(string $s) { return $s; }
        $m = [];
        $m[\"k\"] = \"hi\";
        $v = $m[\"k\"];
        echo f($v);
        ",
    );
    assert_eq!(out, "hi");
}

/// A `Mixed` integer value flowing into an `int` parameter is unboxed and coerced.
#[test]
fn test_mixed_into_int_param_is_coerced() {
    let out = compile_and_run(
        "<?php
        function f(int $n) { return $n + 1; }
        $m = [];
        $m[\"k\"] = 41;
        $v = $m[\"k\"];
        echo f($v);
        ",
    );
    assert_eq!(out, "42");
}

/// A union `?C` value whose runtime payload is a real object flows into a `C` parameter and is
/// unboxed to the underlying object pointer at the call boundary.
#[test]
fn test_union_object_into_class_param_non_null_works() {
    let out = compile_and_run(
        "<?php
        class C { public int $v = 7; }
        function maybe(int $n): ?C {
            if ($n > 0) { return new C(); }
            return null;
        }
        function take(C $c): int { return $c->v; }
        $x = maybe($argc);
        echo take($x);
        ",
    );
    assert_eq!(out, "7");
}

/// A union `?C` value whose runtime payload is null flowing into a non-nullable `C` parameter
/// fatals with a `TypeError` at the boundary guard rather than miscompiling.
#[test]
fn test_union_object_into_class_param_null_fatals() {
    let out = compile_and_run_capture(
        "<?php
        class C { public int $v = 7; }
        function maybe(int $n): ?C {
            if ($n > 100) { return new C(); }
            return null;
        }
        function take(C $c): int { return $c->v; }
        $x = maybe($argc);
        echo take($x);
        ",
    );
    assert!(
        !out.success,
        "null into a typed object parameter should fatal, stdout={:?}",
        out.stdout,
    );
    assert!(
        out.stderr.contains("TypeError"),
        "unexpected stderr: {:?}",
        out.stderr,
    );
}

/// Reassigning a local to incompatible types widens its slot to boxed `Mixed`; the final
/// string value renders correctly (PHP: `string(1) "a"`).
#[test]
fn test_reassign_widening_string_then_observed() {
    let out = compile_and_run(
        "<?php
        $x = 1;
        $x = null;
        $x = \"a\";
        var_dump($x);
        ",
    );
    assert_eq!(out, "string(1) \"a\"\n");
}

/// A local reassigned from `int` to `string` under a runtime-unknown branch compiles, and the
/// final string value renders correctly when the branch is taken.
#[test]
fn test_reassign_widening_conditional_compiles_and_runs() {
    let out = compile_and_run(
        "<?php
        $x = 1;
        if ($argc > 0) { $x = \"taken\"; }
        echo $x;
        ",
    );
    assert_eq!(out, "taken");
}

/// A `Mixed` value (read from a heterogeneous associative array) used as a string offset is
/// accepted under gradual typing and unboxed/coerced to an integer index at the access
/// boundary, reading the character at that offset.
#[test]
fn test_mixed_index_into_string_offset_is_coerced() {
    let out = compile_and_run(
        "<?php
        $a = [];
        $a[\"n\"] = 2;
        $a[\"s\"] = \"hi\";
        $i = $a[\"n\"];
        $s = \"hello\";
        echo $s[$i];
        ",
    );
    assert_eq!(out, "l");
}

/// A union (`int|false`) string-offset index — here a `strpos` result kept runtime-unknown by a
/// `string` parameter — is accepted under gradual typing and coerced to an integer at the
/// boundary, for both the bare index and an arithmetic offset of it.
#[test]
fn test_union_index_into_string_offset_is_coerced() {
    let out = compile_and_run(
        "<?php
        function f(string $m): string {
            $i = strpos($m, \":\", 0);
            return $m[$i] . $m[$i + 1];
        }
        echo f(\"ab:cd\");
        ",
    );
    assert_eq!(out, ":c");
}

/// A genuine `string` key into an array the checker inferred packed/int-keyed is accepted as a
/// PHP associative read: a non-numeric key misses to null, a canonical numeric-string key
/// coerces to the integer offset. The `array` parameter keeps the receiver packed and the key
/// runtime-unknown, so codegen routes through the boxed-Mixed reader, never packed pointer math.
#[test]
fn test_string_key_into_packed_array_reads_associatively() {
    let out = compile_and_run(
        "<?php
        function lk(array $a, string $k): string {
            $v = $a[$k];
            return is_null($v) ? \"null\" : (string)$v;
        }
        $a = [10, 20, 30];
        echo lk($a, \"foo\");
        echo \"|\";
        echo lk($a, \"1\");
        ",
    );
    assert_eq!(out, "null|20");
}

/// A `Mixed` key (read from a heterogeneous associative array) into a packed array is accepted
/// and dispatched at runtime: an integer payload hits the offset and reads the element.
#[test]
fn test_mixed_key_into_packed_array_reads_associatively() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"x\"] = 2;
        $h[\"y\"] = \"s\";
        $k = $h[\"x\"];
        $a = [10, 20, 30];
        echo $a[$k];
        ",
    );
    assert_eq!(out, "30");
}

/// Reading a packed array with `string`/`Mixed` keys inside a loop stays heap-balanced: the
/// transient `Mixed` box created to route the associative read retains and then releases the
/// array exactly once per access, leaving no leaked cells.
#[test]
fn test_packed_array_associative_read_is_heap_balanced() {
    let out = compile_and_run(
        "<?php
        function lk(array $a, string $k): void {
            $v = $a[$k];
            echo is_null($v) ? \"n\" : $v;
        }
        $a = [\"alpha\", \"beta\", \"gamma\"];
        $i = 0;
        while ($i < 3) {
            lk($a, \"x\");
            lk($a, \"1\");
            $i = $i + 1;
        }
        echo $a[1];
        ",
    );
    assert_eq!(out, "nbetanbetanbetabeta");
}
