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

/// A `Mixed`/union-containing-array argument is accepted by `array_flip()` under gradual
/// typing: the boxed array is converted to an owned hash at the runtime boundary and flipped
/// through `__rt_array_flip_mixed`, so keys and values swap (string keys become values and
/// integer values become keys), matching PHP.
#[test]
fn test_gradual_array_flip_with_mixed_argument() {
    let out = compile_and_run(
        "<?php
        $h = [];
        $h[\"a\"] = [\"x\" => 10, \"y\" => 20];
        $h[\"b\"] = \"s\";
        $src = $h[\"a\"];
        var_dump(array_flip($src));
        ",
    );
    assert_eq!(
        out,
        "array(2) {\n  [10]=>\n  string(1) \"x\"\n  [20]=>\n  string(1) \"y\"\n}\n"
    );
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

/// Two locals whose control-flow types are unions made exclusively of indexed/associative array
/// shapes remain valid operands for PHP's left-biased `+=` array union. Both locals use boxed
/// storage, so this also verifies that the two temporary hash conversions are released.
#[test]
fn test_array_only_union_operands_support_compound_array_union() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$left = ['shared' => 'left', 'left' => 'kept'];
if ($argc > 10) {
    $left = [0 => 'left-zero'];
}

$right = ['shared' => 'right', 'right' => 'added'];
if ($argc > 10) {
    $right = [0 => 'right-zero'];
}

$left += $right;
echo $left['shared'], ':', $left['left'], ':', $left['right'];
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "left:kept:added");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Two `array|false` operands use PHP's runtime meaning of `+`: two array payloads form a
/// left-biased union, while two false payloads remain numeric. The boxed conversions and result
/// must also leave the heap balanced.
#[test]
fn test_array_or_false_union_operands_dispatch_array_or_numeric_addition() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function maybe_array(bool $returnArray, string $key): array|false {
    if ($returnArray) {
        return [$key => $key];
    }
    return false;
}

$left = maybe_array(true, 'left');
$right = maybe_array(true, 'right');
$union = $left + $right;
$zero = maybe_array(false, 'unused-left') + maybe_array(false, 'unused-right');
echo $union['left'], ':', $union['right'], ':', $zero;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "left:right:0");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// An `array|false` runtime mismatch must preserve PHP's `TypeError` instead of numerically
/// coercing the array payload through the generic Mixed integer cast.
#[test]
fn test_array_or_false_union_mismatched_addition_fatals() {
    let out = compile_and_run_capture(
        r#"<?php
function maybe_array(bool $returnArray): array|false {
    return $returnArray ? ['value'] : false;
}

echo maybe_array(true) + maybe_array(false);
"#,
    );
    assert!(
        !out.success,
        "array plus false should fatal, stdout={:?}",
        out.stdout
    );
    assert!(
        out.stderr.contains("TypeError"),
        "unexpected stderr: {:?}",
        out.stderr
    );
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

/// Verifies a terminated switch case cannot widen a local used by a sibling case.
#[test]
fn test_break_terminated_switch_case_assignment_does_not_leak_to_sibling_case() {
    let out = compile_and_run(
        r#"<?php
function renderChoice(
    int $mode,
    array $choices,
    string|bool|int|float|null $default,
): string {
    switch ($mode) {
        case 1:
            $default = explode(",", (string) $default);
            break;
        case 2:
            return (string) ($choices[$default] ?? $default);
        default:
            return "";
    }

    return "fallback";
}

echo renderChoice(2, ["primary" => "green"], "primary");
"#,
    );
    assert_eq!(out, "green");
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

/// Verifies arithmetic on two `Mixed` operands computes at runtime like PHP:
/// `mixed + mixed` boxes/unboxes through the numeric dispatch path. PHP: `2 + 3 == 5`.
#[test]
fn test_gradual_mixed_arithmetic_add() {
    let out = compile_and_run(
        r#"<?php
function add(mixed $a, mixed $b) { return $a + $b; }
echo add(2, 3);
"#,
    );
    assert_eq!(out, "5");
}

/// Verifies ordered comparison (`<`) on two `Mixed` operands lowers through the runtime
/// comparator and yields PHP-identical results: `2 < 3` is true, `5 < 3` is false.
#[test]
fn test_gradual_mixed_comparison_lt() {
    let out = compile_and_run(
        r#"<?php
function cmp(mixed $a, mixed $b) { return $a < $b ? "lt" : "ge"; }
echo cmp(2, 3), cmp(5, 3);
"#,
    );
    assert_eq!(out, "ltge");
}

/// Verifies the spaceship operator on `Mixed` operands: `2 <=> 3 == -1`, matching PHP.
#[test]
fn test_gradual_mixed_spaceship() {
    let out = compile_and_run(
        r#"<?php
function sp(mixed $a, mixed $b) { return $a <=> $b; }
echo sp(2, 3);
"#,
    );
    assert_eq!(out, "-1");
}

/// Verifies a nullsafe method call on a `Mixed` receiver: a non-null object dispatches the
/// method (`42`), while a null-holding `Mixed` cell short-circuits to null so `?? "N"`
/// yields `"N"`. This exercises the runtime null-guard for boxed `?->` receivers.
#[test]
fn test_gradual_nullsafe_method_on_mixed_receiver() {
    let out = compile_and_run(
        r#"<?php
class Box { public function val(): int { return 42; } }
function get(mixed $x): mixed { return $x?->val(); }
$b = new Box();
echo (get($b) ?? "N"), "|", (get(null) ?? "N");
"#,
    );
    assert_eq!(out, "42|N");
}

/// Verifies a nullsafe property read on a `Mixed` receiver short-circuits to null when the
/// receiver is null (`?? "N"` → `"N"`) and returns the property otherwise.
#[test]
fn test_gradual_nullsafe_property_on_mixed_receiver() {
    let out = compile_and_run(
        r#"<?php
class Holder { public int $n = 7; }
function read(mixed $x): mixed { return $x?->n; }
$h = new Holder();
echo (read($h) ?? "N"), "|", (read(null) ?? "N");
"#,
    );
    assert_eq!(out, "7|N");
}

/// Re-enabled after the call-argument spread runtime guard landed: spreading a `mixed`-typed
/// parameter that holds an array into a variadic callee now materializes a concrete packed
/// `array<mixed>` through the runtime array guard (`Op::MixedToHash` + `array_values()`) before
/// the variadic tail is built, so it unpacks correctly. (Correction round 1 had reverted the
/// original acceptance because the lowering lacked that guard.)
#[test]
fn test_gradual_spread_mixed_into_variadic() {
    let out = compile_and_run(
        r#"<?php
function total(...$xs) { $t = 0; foreach ($xs as $x) { $t += $x; } return $t; }
function go(mixed $a) { return total(...$a); }
echo go([1, 2, 3, 4]);
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies list-unpacking a `Mixed` right-hand side that holds an array binds each
/// positional target. PHP: `[$a, $b] = [10, 20]` gives `$a + $b == 30`.
#[test]
fn test_gradual_list_unpack_mixed_rhs() {
    let out = compile_and_run(
        r#"<?php
function take(mixed $arr): int { [$a, $b] = $arr; return $a + $b; }
echo take([10, 20]);
"#,
    );
    assert_eq!(out, "30");
}

// --- Family A: PHP's monolithic `array` hint accepts any array-family value ---

/// A differently-typed array flows into a parameter whose element type the checker specialized
/// to `Array(Object(C))` from an earlier call: PHP's `array` hint carries no element type, so
/// `sz([1, 2, 3])` after `sz([new C(), new C()])` is accepted and runs (`count` = 3), and the
/// object-array call still counts 2. Pre-relaxation this reported "expects Array(Object(\"C\")),
/// got Array(Int)". Codegen is sound because all PHP arrays share one pointer representation.
#[test]
fn test_array_family_differently_typed_array_into_specialized_param() {
    let out = compile_and_run(
        r#"<?php
class C { public int $v = 1; }
function sz(array $items): int { return count($items); }
$objs = [new C(), new C()];
echo sz($objs);
echo "|";
$nums = [1, 2, 3];
echo sz($nums);
"#,
    );
    assert_eq!(out, "2|3");
}

/// An associative array flows into a parameter specialized to `Array(Object(Def))`: PHP accepts
/// any array shape for an `array` hint, so `setArgs(["k" => "v", "m" => "n"])` runs (`count` = 2).
#[test]
fn test_array_family_assoc_array_into_object_array_param() {
    let out = compile_and_run(
        r#"<?php
class Def {}
function setArgs(array $a): int { return count($a); }
function build(): array { $r = []; $r[] = new Def(); return $r; }
$typed = build();
echo setArgs($typed);
echo "|";
echo setArgs(["k" => "v", "m" => "n"]);
"#,
    );
    assert_eq!(out, "1|2");
}

// --- Family C (foreach key): a genuine int key still supports arithmetic ---

/// A genuine int key of a list supports arithmetic: `$sum += $i` over `[5, 6, 7]` keys (0, 1, 2)
/// sums to 3, matching PHP. Foreach keys are typed `Int`; this guards integer-key arithmetic
/// against regression.
#[test]
fn test_foreach_int_key_arithmetic() {
    let out = compile_and_run(
        r#"<?php
function mk(): array { return [5, 6, 7]; }
$sum = 0;
foreach (mk() as $i => $v) { $sum += $i; }
echo $sum;
"#,
    );
    assert_eq!(out, "3");
}

// --- Family C (box variadic): a declared `mixed ...$args` is never narrowed ---

/// A `mixed ...$args` variadic accepts heterogeneous arguments (string, int, float, array): the
/// declared `Mixed` element type must not be re-specialized to the first argument's type. Pre-fix
/// this reported "variadic parameter $args expects Str, got Int" for the second argument.
#[test]
fn test_mixed_variadic_accepts_heterogeneous_args() {
    let out = compile_and_run(
        r#"<?php
class Fs {
    public function box(string $func, mixed ...$args): string {
        return $func . ":" . count($args);
    }
}
$fs = new Fs();
echo $fs->box("f", "s", 1, 2.5, [9]);
"#,
    );
    assert_eq!(out, "f:4");
}

/// A static `mixed ...$args` variadic keeps its declared gradual element type across repeated
/// heterogeneous calls instead of specializing to the first call's string argument.
#[test]
fn test_static_mixed_variadic_accepts_heterogeneous_args() {
    let out = compile_and_run(
        r#"<?php
class StaticFs {
    private static function box(string $func, mixed ...$args): string {
        return $func . ":" . count($args);
    }

    public static function run(): string {
        return self::box("first", "value") . "|" . self::box("second", 7, false);
    }
}

echo StaticFs::run();
"#,
    );
    assert_eq!(out, "first:1|second:2");
}

// --- Family G (sound covariance): a union of subtypes into their common base compiles + runs ---

/// A union of two SUBTYPES (`A|C`, both extending `Base`) flowing into a `Base`-typed parameter
/// compiles and runs php-identically: every union member is a proven subtype of the target, so the
/// subtype direction of `gradual_union_flows_into` matches each member. This must stay green after
/// the round-3 exclusion of the object supertype direction, which only affects base→derived flows.
#[test]
fn test_union_of_subtypes_into_common_base_param_runs() {
    let out = compile_and_run(
        r#"<?php
class Base { public function who(): string { return "b"; } }
class A extends Base { public function who(): string { return "a"; } }
class C extends Base { public function who(): string { return "c"; } }
function take(Base $x): string { return $x->who(); }
function pick(int $n): A|C { return new A(); }
echo take(pick($argc));
"#,
    );
    assert_eq!(out, "a");
}

// --- Family H1-B (scalar-receiver index reads): campaign H1 PART B ---
//
// PHP treats indexing a bare scalar (`false[0]`, `null["k"]`, `5[0]`, `5.5[0]`) as a miss:
// a `Warning` plus `null`, never a fatal (php -n verified). These were previously rejected
// with "Cannot index non-array"; elephc now accepts them and returns `Mixed` (boxed null at
// runtime), matching PHP's value output (the `stderr` warning text is intentionally omitted).

/// A bare `bool` (`false`) receiver read at an integer offset misses to `null`, matching PHP.
#[test]
fn test_gradual_scalar_bool_receiver_index_read_misses_to_null() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n): bool { return false; }
$x = pick($argc);
var_dump($x[0]);
"#,
    );
    assert_eq!(out, "NULL\n");
}

/// A bare `null` (`Void`) receiver read at a string key misses to `null`, matching PHP.
#[test]
fn test_gradual_scalar_null_receiver_index_read_misses_to_null() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n) { return null; }
$x = pick($argc);
var_dump($x["k"]);
"#,
    );
    assert_eq!(out, "NULL\n");
}

/// Bare `int`/`float` receivers also miss to `null` on indexing (php -n verified), matching
/// the bool/null cases above rather than fataling.
#[test]
fn test_gradual_scalar_int_float_receiver_index_read_misses_to_null() {
    let out = compile_and_run(
        r#"<?php
function pick_int(int $n): int { return $n + 5; }
function pick_float(int $n): float { return $n + 5.5; }
$i = pick_int($argc);
$f = pick_float($argc);
var_dump($i[0]);
var_dump($f[0]);
"#,
    );
    assert_eq!(out, "NULL\nNULL\n");
}

/// A scalar-only union (`int|false`) miss both branches (int and false) to `null` on indexing.
#[test]
fn test_gradual_int_or_false_union_receiver_index_read_misses_to_null() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n): int|false {
    if ($n > 100) { return $n; }
    return false;
}
$x = pick($argc);
var_dump($x[0]);
"#,
    );
    assert_eq!(out, "NULL\n");
}

/// A nested read through a scalar-vivified `Mixed(null)` miss (`$x[0]["k"]`) composes correctly:
/// the outer miss already yields `Mixed(null)`, and indexing that (already-covered Mixed path)
/// also misses to `null`, matching PHP.
#[test]
fn test_gradual_scalar_receiver_nested_index_read_misses_to_null() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n): bool { return false; }
$x = pick($argc);
var_dump($x[0]["k"]);
"#,
    );
    assert_eq!(out, "NULL\n");
}

/// Regression: an expression-position destructure of a NON-constant-foldable `null` source used
/// directly as a condition (`if ([, $b] = $src) {...}`) must be falsy, matching PHP. This exercises
/// `ctx.truthy()`'s `Void`-under-`IrType::I64` special case — the compiler-synthesized list-unpack
/// temp is introduced after AST-level constant folding, so `Op::ConstNull`'s `NULL_SENTINEL`
/// encoding (not literal 0) must route through `Op::IsTruthy` rather than being tested by raw
/// nonzero-ness, or `null` reads as truthy. Only reachable once scalar-receiver index reads
/// (this file's Family H1-B) let `[, $b] = null` type-check as a list-unpack read instead of
/// erroring "Cannot index non-array".
#[test]
fn test_gradual_expr_destructure_null_source_condition_is_falsy() {
    let out = compile_and_run(
        r#"<?php
function pick(int $n) { return null; }
$src = pick($argc);
if ([, $b] = $src) { echo "t"; } else { echo "f"; }
var_dump($b);
"#,
    );
    assert_eq!(out, "fNULL\n");
}
