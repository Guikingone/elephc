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

/// A `Mixed` integer value (read from a heterogeneous associative array) is accepted by the
/// increment/decrement operators under gradual typing: EIR lowering unboxes and coerces the
/// boxed cell to an integer before applying `++`/`--`, so the stored result is the correct
/// incremented/decremented number for both pre- and post-forms.
#[test]
fn test_inc_dec_on_mixed_unboxes_and_coerces() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"x\"] = 41;
        $h[\"y\"] = \"s\";
        $v = $h[\"x\"];
        $v++;
        echo $v;
        echo \"|\";
        $w = $h[\"x\"];
        echo ++$w;
        echo \"|\";
        $d = $h[\"x\"];
        $d--;
        echo $d;
        ",
    );
    assert_eq!(out, "42|42|40");
}

/// A numeric-coercible union value (`int|false`, here a `strpos` result kept runtime-unknown by a
/// `string` parameter) is accepted by `++` under gradual typing and coerced to an integer before
/// the increment, so the false-or-int operand increments as a number.
#[test]
fn test_inc_on_numeric_union_coerces() {
    let out = compile_and_run(
        "<?php
        function f(string $m): int {
            $i = strpos($m, \":\", 0);
            $i++;
            return $i;
        }
        echo f(\"ab:cd\");
        ",
    );
    assert_eq!(out, "3");
}

/// A `Mixed`/union-containing-array haystack (read from a heterogeneous associative array) is
/// accepted by `in_array()` under gradual typing: the boxed array is converted to an owned hash
/// at the runtime boundary, so a present needle hits and an absent needle misses.
#[test]
fn test_gradual_in_array_with_mixed_haystack() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"a\"] = [1, 2, 3];
        $h[\"b\"] = \"s\";
        $hay = $h[\"a\"];
        var_dump(in_array(2, $hay));
        var_dump(in_array(9, $hay));
        ",
    );
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// A `Mixed`/union-containing-array haystack is accepted by `array_key_exists()` under gradual
/// typing: the boxed array is converted to an owned hash at the runtime boundary, so a present
/// key hits and an absent key misses.
#[test]
fn test_gradual_array_key_exists_with_mixed_haystack() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"a\"] = [\"x\" => 1];
        $h[\"b\"] = \"s\";
        $hay = $h[\"a\"];
        var_dump(array_key_exists(\"x\", $hay));
        var_dump(array_key_exists(\"z\", $hay));
        ",
    );
    assert_eq!(out, "bool(true)\nbool(false)\n");
}

/// An array union `$a + $b` where one operand is a concrete array literal and the other is a
/// `Mixed` value (read from a heterogeneous associative array) is accepted under gradual typing:
/// the boxed operand is unboxed and asserted to be an array, then unioned with left-operand keys
/// winning, matching PHP's `[1, 2] + [3, 4, 5] == [1, 2, 5]`.
#[test]
fn test_gradual_array_union_with_mixed_operand() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"a\"] = [1, 2];
        $h[\"b\"] = \"s\";
        $m = $h[\"a\"];
        $r = $m + [3, 4, 5];
        echo $r[0] . $r[1] . $r[2];
        ",
    );
    assert_eq!(out, "125");
}

/// Regression: a `string` parameter reassigned to an `int` on the **then** branch of an `if`
/// (which returns) must not leak its `int` type onto the else/fall-through path where the
/// reassignment never ran. Before branch-scoped local-type tracking, the else path read the
/// still-string parameter with the leaked `int` representation and printed `0` instead of the
/// original string. Matches PHP, which prints the parameter unchanged on the else path.
#[test]
fn test_string_param_reassigned_int_on_returning_branch_does_not_leak_type() {
    let out = compile_and_run(
        r#"<?php
function f(string $s): string {
    if ($s === "zzz") {
        $s = 42;
        return "then:" . $s;
    }
    return "else:" . $s;
}
echo f("zzz"), "|", f("elephc"), "|", f("abc");
"#,
    );
    assert_eq!(out, "then:42|else:elephc|else:abc");
}

/// Regression: when a `string` local is conditionally reassigned to an `int` and the branches
/// merge (no early return), the post-`if` value must carry each branch's own value with the
/// correct representation — `int(42)` when the branch ran, the original string otherwise — with
/// `gettype()` reporting `integer`/`string` respectively, matching PHP.
#[test]
fn test_conditional_string_to_int_reassignment_merges_per_branch() {
    let out = compile_and_run(
        r#"<?php
function f(string $s): string {
    if ($s === "zzz") {
        $s = 42;
    }
    return $s . ":" . gettype($s);
}
echo f("zzz"), "|", f("elephc");
"#,
    );
    assert_eq!(out, "42:integer|elephc:string");
}

/// Regression: the leak also travels through a `switch`. A `string` value reassigned to an `int`
/// inside a case body that **returns** must not change the type seen by the code after the switch
/// on the no-match path, where the value is still the original string. Before switch body type
/// mutations were folded back against the no-match state, `s("elephc")` printed `0`.
#[test]
fn test_string_reassigned_int_in_returning_switch_case_does_not_leak_after_switch() {
    let out = compile_and_run(
        r#"<?php
function s(string $v): string {
    switch (true) {
        case ctype_digit($v):
            $v = (int) $v;
            return "digit:" . $v;
    }
    return (string) $v;
}
echo s("7"), "|", s("elephc"), "|", s("abc");
"#,
    );
    assert_eq!(out, "digit:7|elephc|abc");
}

/// Regression: a `string` parameter reassigned to an `int` inside a `do…while` body must be
/// read with the widened representation after the loop. The reassignment widens the slot to
/// `Mixed` for the whole function, so the post-loop `"v:" . $s` read must load it as `Mixed`
/// and coerce, matching PHP: the reassignment never runs when the loop iterates once, but the
/// widened storage is still correct on the multi-iteration path.
#[test]
fn test_do_while_string_to_int_widening_reads_after_loop() {
    let out = compile_and_run(
        r#"<?php
function f(string $s, int $n): string {
    $i = 0;
    do {
        if ($i === 1) { $s = 42; }
        $i++;
    } while ($i < $n);
    return "v:" . $s;
}
echo f("elephc", 1), "|", f("elephc", 3);
"#,
    );
    assert_eq!(out, "v:elephc|v:42");
}

/// Regression for the loop back-edge type leak: a `string` local read at the top of a `while`
/// body, before it is reassigned to an `int` later in the same body, is live with the widened
/// `Mixed` value on iterations past the first. Without loop-scoped pre-widening the read stayed
/// typed `string`, so `gettype()` reported `string` on every iteration (the `Mixed` cell coerced
/// back to a string) instead of `integer` once the reassignment had run. Matches PHP.
#[test]
fn test_while_body_read_before_reassignment_sees_widened_type() {
    let out = compile_and_run(
        r#"<?php
function g(string $s, int $n): string {
    $i = 0;
    $out = "";
    while ($i < $n) {
        $out .= gettype($s) . ":";
        if ($i === 0) { $s = 42; }
        $i++;
    }
    return $out;
}
echo g("elephc", 1), "|", g("elephc", 3);
"#,
    );
    assert_eq!(out, "string:|string:integer:integer:");
}

/// Regression: the loop back-edge leak also travels through `foreach`. A `string` local read at
/// the top of the body, then reassigned to the (int) element on each pass, must report the type
/// of the value carried from the previous iteration. Before loop pre-widening it read `string`
/// throughout; PHP reports `string` on the first pass and `integer` afterward.
#[test]
fn test_foreach_body_read_before_reassignment_sees_widened_type() {
    let out = compile_and_run(
        r#"<?php
function h(array $items, string $s): string {
    $out = "";
    foreach ($items as $x) {
        $out .= gettype($s) . ":";
        $s = $x;
    }
    return $out;
}
echo h([1, 2], "elephc");
"#,
    );
    assert_eq!(out, "string:integer:");
}

/// Regression guard against over-widening: a loop counter reassigned to the **same** type
/// (`$x = $x + 1`, int→int) must NOT be promoted to `Mixed`. If the pre-scan wrongly widened it,
/// each `$acc[] = $x` would store a boxed `Mixed` pointer into the `array<int>` container and
/// `implode` would print raw addresses instead of the integers. Correct output keeps the counter
/// narrow, matching PHP.
#[test]
fn test_same_type_loop_counter_is_not_over_widened() {
    let out = compile_and_run(
        r#"<?php
function counter(int $n): string {
    $x = 10;
    $acc = [];
    $i = 0;
    while ($i < $n) {
        $acc[] = $x;
        $x = $x + 1;
        $i++;
    }
    return implode(",", $acc);
}
echo counter(3);
"#,
    );
    assert_eq!(out, "10,11,12");
}
