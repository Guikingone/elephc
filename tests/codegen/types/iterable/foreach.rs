//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of types, iterable foreach, including iterable as parameter and return type, foreach over iterable hash emits keys and values, and foreach over iterable indexed emits keys and values.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies `iterable` can be used as a parameter and return type; an array passed through an
/// `identity(iterable $values): iterable` function is returned and `is_iterable()` confirms it.
#[test]
fn test_iterable_as_parameter_and_return_type() {
    let out = compile_and_run(
        "<?php
        function identity(iterable $values): iterable {
            return $values;
        }
        echo is_null(identity([1, 2])) ? 'null' : 'ok';
        ",
    );
    assert_eq!(out, "ok");
}

/// Verifies `foreach` over a hash (associative array) via a `iterable` parameter emits correct
/// string keys and values; `dump(['a' => 1, 'b' => 2, 'c' => 3])` outputs "a=1;b=2;c=3;".
#[test]
fn test_foreach_over_iterable_hash_emits_keys_and_values() {
    let out = compile_and_run(
        "<?php
        function dump(iterable $items): void {
            foreach ($items as $k => $v) {
                echo $k;
                echo '=';
                echo $v;
                echo ';';
            }
        }
        dump(['a' => 1, 'b' => 2, 'c' => 3]);
        ",
    );
    assert_eq!(out, "a=1;b=2;c=3;");
}

/// Verifies `foreach` over an indexed array via a `iterable` parameter emits correct integer keys
/// and values; `dump([10, 20, 30])` outputs "0=10;1=20;2=30;".
#[test]
fn test_foreach_over_iterable_indexed_emits_keys_and_values() {
    let out = compile_and_run(
        "<?php
        function dump(iterable $items): void {
            foreach ($items as $k => $v) {
                echo $k;
                echo '=';
                echo $v;
                echo ';';
            }
        }
        dump([10, 20, 30]);
        ",
    );
    assert_eq!(out, "0=10;1=20;2=30;");
}

/// Verifies `foreach` over an indexed string array via a `iterable` parameter uses runtime slot-width
/// integer keys; `dump(['red', 'blue'])` outputs "0:red;1:blue;".
#[test]
fn test_foreach_over_iterable_indexed_strings_uses_runtime_slot_width() {
    let out = compile_and_run(
        "<?php
        function dump(iterable $items): void {
            foreach ($items as $k => $v) {
                echo $k;
                echo ':';
                echo $v;
                echo ';';
            }
        }
        dump(['red', 'blue']);
        ",
    );
    assert_eq!(out, "0:red;1:blue;");
}

/// Verifies `foreach` over an untyped parameter receiving a `mixed` array works correctly;
/// `passthrough([1, 2, 3])` is passed to `dump($items)` and each value is echoed.
#[test]
fn test_foreach_over_untyped_parameter_with_mixed_runtime_array() {
    let out = compile_and_run(
        "<?php
        function passthrough(mixed $value): mixed {
            return $value;
        }
        function dump($items): void {
            foreach ($items as $value) {
                echo $value;
                echo ';';
            }
        }
        dump(passthrough([1, 2, 3]));
        ",
    );
    assert_eq!(out, "1;2;3;");
}

/// Verifies `foreach` over a `mixed` parameter containing an associative array with mixed types
/// outputs correct keys and string/integer values.
#[test]
fn test_foreach_over_mixed_parameter_assoc_array() {
    let out = compile_and_run(
        "<?php
        function dump(mixed $items): void {
            foreach ($items as $key => $value) {
                echo $key;
                echo '=';
                echo $value;
                echo ';';
            }
        }
        dump(['a' => 1, 'b' => 'two']);
        ",
    );
    assert_eq!(out, "a=1;b=two;");
}

/// Verifies `foreach` over a `mixed` result from `json_decode` with `true` (assoc) produces correct
/// integer keys and values; outputs "0=10;1=20;".
#[test]
fn test_foreach_over_mixed_json_decode_indexed_array() {
    let out = compile_and_run(
        r#"<?php
        $items = json_decode("[10, 20]", true);
        foreach ($items as $key => $value) {
            echo $key;
            echo '=';
            echo $value;
            echo ';';
        }
        "#,
    );
    assert_eq!(out, "0=10;1=20;");
}

/// Verifies `foreach` over a union-typed `array|bool` parameter dispatches to the array branch at runtime;
/// `dump(choose(true))` outputs "0:4;1:5;" where `choose` returns `[4, 5]`.
#[test]
fn test_foreach_over_union_parameter_array_runtime_value() {
    let out = compile_and_run(
        "<?php
        function choose(bool $flag): array|bool {
            if ($flag) {
                return [4, 5];
            }
            return false;
        }
        function dump(array|bool $items): void {
            foreach ($items as $key => $value) {
                echo $key;
                echo ':';
                echo $value;
                echo ';';
            }
        }
        dump(choose(true));
        ",
    );
    assert_eq!(out, "0:4;1:5;");
}

/// Verifies by-ref `foreach` over a `mixed` indexed array mutates the source array;
/// `[1, 2, 3]` becomes `[7, 7, 7]` after the by-ref loop.
#[test]
fn test_foreach_by_ref_over_mixed_indexed_array_updates_source() {
    let out = compile_and_run(
        "<?php
        function rewrite(mixed $items): void {
            foreach ($items as &$value) {
                $value = 7;
            }
            foreach ($items as $value) {
                echo $value;
                echo ';';
            }
        }
        rewrite([1, 2, 3]);
        ",
    );
    assert_eq!(out, "7;7;7;");
}

/// Verifies by-ref `foreach` over a `mixed` associative array mutates source values;
/// `['a' => 1, 'b' => 2]` becomes `['a' => 9, 'b' => 9]` after the loop.
#[test]
fn test_foreach_by_ref_over_mixed_assoc_array_updates_source() {
    let out = compile_and_run(
        "<?php
        function rewrite(mixed $items): void {
            foreach ($items as $key => &$value) {
                $value = 9;
            }
            foreach ($items as $key => $value) {
                echo $key;
                echo '=';
                echo $value;
                echo ';';
            }
        }
        rewrite(['a' => 1, 'b' => 2]);
        ",
    );
    assert_eq!(out, "a=9;b=9;");
}

/// Verifies that `unset($inner)` after a nested by-ref `foreach` resets its lifetime so the outer
/// `$v *= 2` mutation applies to the correct variable; outputs "42,4,6,".
#[test]
fn test_nested_by_ref_foreach_unset_inner_lifetime_reset() {
    let out = compile_and_run(
        "<?php
        $a = [1, 2, 3];
        foreach ($a as &$v) {
            foreach ($a as &$inner) {
                $inner += 10;
                break;
            }
            unset($inner);
            $v *= 2;
        }
        unset($v);
        foreach ($a as $x) {
            echo $x . ',';
        }
        ",
    );
    assert_eq!(out, "42,4,6,");
}

/// Verifies by-ref `foreach` on `mixed` with copy-on-write semantics: mutating `$b` via by-ref foreach
/// does not affect `$a`, but mutating `$c` does (COW split); outputs "a,b|a,b|a!,b!".
#[test]
fn test_mixed_by_ref_foreach_cow_split_preserves_aliases() {
    let out = compile_and_run(
        "<?php
        function mutate(mixed $x): mixed {
            foreach ($x as &$v) {
                $v .= '!';
            }
            unset($v);
            return $x;
        }
        $a = ['a', 'b'];
        $b = $a;
        $c = mutate($b);
        echo implode(',', $a) . '|' . implode(',', $b) . '|' . implode(',', $c);
        ",
    );
    assert_eq!(out, "a,b|a,b|a!,b!");
}

/// Verifies by-ref `foreach` over a `json_decode` assoc payload nested in a `mixed` container
/// mutates the source data; outputs "101|102" after adding 100 to each `n` field.
#[test]
fn test_by_ref_foreach_nested_json_decode_assoc_payloads() {
    let out = compile_and_run(
        r#"<?php
        $data = json_decode('{"rows":[{"n":1},{"n":2}]}', true);
        foreach ($data["rows"] as &$row) {
            $row["n"] = $row["n"] + 100;
        }
        unset($row);
        echo $data["rows"][0]["n"] . "|" . $data["rows"][1]["n"];
        "#,
    );
    assert_eq!(out, "101|102");
}

/// Verifies `foreach` over a non-iterable `mixed` WARNS and continues instead of aborting, and
/// that side effects sequenced before the loop are preserved: "S" is echoed by `side()` before
/// the foreach check runs.
///
/// This test used to pin the opposite behavior — a compiler-internal
/// `Fatal error: foreach over iterable with unsupported kind` and exit code 70. Reference PHP
/// 8.5.6 (`php -d xdebug.mode=off`) prints `S` on stdout,
/// `Warning: foreach() argument must be of type array|object, int given` on stderr, and exits 0;
/// elephc now matches, minus the ` in <file> on line <n>` tail it does not synthesize.
#[test]
fn test_mixed_foreach_over_non_iterable_warns_and_preserves_prior_side_effects() {
    let out = compile_and_run_capture(
        "<?php
        function side(): mixed {
            echo 'S';
            return 42;
        }
        $x = side();
        foreach ($x as $v) {
            echo $v;
        }
        ",
    );
    assert!(out.success, "program unexpectedly failed: {}", out.stderr);
    assert_eq!(out.stdout, "S");
    assert_eq!(
        out.stderr,
        "Warning: foreach() argument must be of type array|object, int given\n",
        "{}",
        out.stderr
    );
}

/// Verifies `foreach` over an `iterable`-typed `Iterator` implementation emits the correct keys and
/// values; `dump(new Range(2, 5))` outputs "0=2;1=3;2=4;" (key starts at 0 regardless of `current()`).
#[test]
fn test_foreach_over_iterable_iterator_object() {
    let out = compile_and_run(
        r#"<?php
class Range implements Iterator {
    private int $current;
    private int $end;
    public function __construct(int $start, int $end) {
        $this->current = $start;
        $this->end = $end;
    }
    public function rewind(): void {}
    public function valid(): bool { return $this->current < $this->end; }
    public function current(): int { return $this->current; }
    public function key(): int { return $this->current - 2; }
    public function next(): void { $this->current = $this->current + 1; }
}
function dump(iterable $items): void {
    foreach ($items as $k => $v) {
        echo $k;
        echo '=';
        echo $v;
        echo ';';
    }
}
dump(new Range(2, 5));
"#,
    );
    assert_eq!(out, "0=2;1=3;2=4;");
}

/// Verifies `foreach` over an `iterable`-typed `IteratorAggregate` implementation correctly delegates
/// to its `getIterator()` result; outputs "012".
#[test]
fn test_foreach_over_iterable_iterator_aggregate_object() {
    let out = compile_and_run(
        r#"<?php
class Range implements Iterator {
    private int $current;
    private int $end;
    public function __construct(int $start, int $end) {
        $this->current = $start;
        $this->end = $end;
    }
    public function rewind(): void {}
    public function valid(): bool { return $this->current < $this->end; }
    public function current(): int { return $this->current; }
    public function key(): int { return $this->current; }
    public function next(): void { $this->current = $this->current + 1; }
}
class Values implements IteratorAggregate {
    public function getIterator(): Iterator { return new Range(0, 3); }
}
function dump(iterable $items): void {
    foreach ($items as $v) {
        echo $v;
    }
}
dump(new Values());
"#,
    );
    assert_eq!(out, "012");
}

/// Verifies `getIterator()` on an `IteratorAggregate` is called exactly once per foreach iteration;
/// "G0011" confirms `getIterator()` echoes "G" once and the loop runs twice with keys "00" and "11".
#[test]
fn test_iterator_aggregate_get_iterator_side_effect_runs_once() {
    let out = compile_and_run(
        r#"<?php
class It implements Iterator {
    private int $i = 0;
    public function rewind(): void { $this->i = 0; }
    public function valid(): bool { return $this->i < 2; }
    public function current(): mixed { return $this->i; }
    public function key(): mixed { return $this->i; }
    public function next(): void { $this->i = $this->i + 1; }
}
class Bag implements IteratorAggregate {
    public function getIterator(): Iterator {
        echo "G";
        return new It();
    }
}
foreach (new Bag() as $k => $v) {
    echo $k . $v;
}
"#,
    );
    assert_eq!(out, "G0011");
}

/// Verifies `foreach` over a `iterable`-typed `Iterator` object can reuse the receiver variable name;
/// `consume(new Range(0, 3))` with `$items` as both the iterable and loop variable outputs "012".
#[test]
fn test_foreach_over_iterable_iterator_can_reuse_receiver_variable() {
    let out = compile_and_run(
        r#"<?php
class Range implements Iterator {
    private int $current;
    private int $end;
    public function __construct(int $start, int $end) {
        $this->current = $start;
        $this->end = $end;
    }
    public function rewind(): void {}
    public function valid(): bool { return $this->current < $this->end; }
    public function current(): int { return $this->current; }
    public function key(): int { return $this->current; }
    public function next(): void { $this->current = $this->current + 1; }
}
function consume(iterable $items): void {
    foreach ($items as $items) {
        echo $items;
    }
}
consume(new Range(0, 3));
"#,
    );
    assert_eq!(out, "012");
}

/// Verifies `foreach` over an empty `iterable`-typed `Iterator` preserves the existing value of the
/// receiver variable; `consume(new EmptyIteratorImpl())` echoes "old".
#[test]
fn test_foreach_over_empty_iterable_iterator_preserves_existing_value_variable() {
    let out = compile_and_run(
        r#"<?php
class EmptyIteratorImpl implements Iterator {
    public function rewind(): void {}
    public function valid(): bool { return false; }
    public function current(): int { return 1; }
    public function key(): int { return 2; }
    public function next(): void {}
}
function consume(iterable $items): void {
    $value = 'old';
    foreach ($items as $value) {
    }
    echo $value;
}
consume(new EmptyIteratorImpl());
"#,
    );
    assert_eq!(out, "old");
}

/// Verifies `foreach` over an indexed `iterable` can reuse the receiver variable name as the value variable;
/// `consume([10, 20, 30])` with `as $items` outputs "10;20;30;".
#[test]
fn test_foreach_over_iterable_indexed_can_reuse_receiver_variable() {
    let out = compile_and_run(
        "<?php
        function consume(iterable $items): void {
            foreach ($items as $items) {
                echo $items;
                echo ';';
            }
        }
        consume([10, 20, 30]);
        ",
    );
    assert_eq!(out, "10;20;30;");
}

/// Verifies `foreach` over an associative `iterable` can reuse the receiver variable as the key;
/// `consume(['a' => 1, 'b' => 2])` with `as $items => $value` outputs "a=1;b=2;".
#[test]
fn test_foreach_over_iterable_assoc_key_can_reuse_receiver_variable() {
    let out = compile_and_run(
        "<?php
        function consume(iterable $items): void {
            foreach ($items as $items => $value) {
                echo $items;
                echo '=';
                echo $value;
                echo ';';
            }
        }
        consume(['a' => 1, 'b' => 2]);
        ",
    );
    assert_eq!(out, "a=1;b=2;");
}

/// Verifies the loop key from `foreach` over an `iterable` remains `mixed` and retains the last key
/// value after the loop; `last_key(['a' => 1])` outputs "a" and `last_key([10, 20])` outputs "1".
#[test]
fn test_iterable_foreach_key_remains_mixed_after_runtime_branch() {
    let out = compile_and_run(
        "<?php
        function last_key(iterable $items): void {
            foreach ($items as $k => $v) {
            }
            echo $k;
        }
        last_key(['a' => 1]);
        echo '|';
        last_key([10, 20]);
        ",
    );
    assert_eq!(out, "a|1");
}

/// Verifies that a `mixed` iterable containing an inner indexed array (boxed via `iterable` return)
/// preserves `is_iterable() == true` inside the outer foreach; the inner array is not flattened.
#[test]
fn test_iterable_value_in_indexed_array_stays_boxed() {
    let out = compile_and_run(
        "<?php
        function id(iterable $items): iterable {
            return $items;
        }
        function show(iterable $items): void {
            foreach ($items as $value) {
                echo is_iterable($value) ? gettype($value) : 'no';
                echo ':';
                var_dump($value);
            }
        }
        show([id([1, 2])]);
        ",
    );
    // The inner array dumps its elements: the boxed Mixed unbox lands on the real
    // array and the walker reads its runtime value_type stamp. It used to print the
    // empty `array(2) {\n}\n` shell.
    assert_eq!(out, "array:array(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\n");
}

/// Verifies that a `mixed` iterable containing an inner associative array preserves
/// `is_iterable() == true` inside the outer foreach; the inner array is not flattened.
#[test]
fn test_iterable_value_in_assoc_array_stays_boxed() {
    let out = compile_and_run(
        "<?php
        function id(iterable $items): iterable {
            return $items;
        }
        $items = ['inner' => id([1, 2])];
        foreach ($items as $value) {
            echo is_iterable($value) ? gettype($value) : 'no';
            echo ':';
            var_dump($value);
        }
        ",
    );
    // The inner array dumps its elements: the boxed Mixed unbox lands on the real
    // array and the walker reads its runtime value_type stamp. It used to print the
    // empty `array(2) {\n}\n` shell.
    assert_eq!(out, "array:array(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\n");
}

/// Verifies an `iterable` value stored in a mixed associative array remains a boxed Mixed value
/// when read directly through `hash_get`.
#[test]
fn test_iterable_value_in_mixed_assoc_array_direct_read_stays_boxed() {
    let out = compile_and_run(
        "<?php
        function id(iterable $items): iterable {
            return $items;
        }
        $items = ['inner' => id([1, 2]), 'n' => 1];
        $value = $items['inner'];
        echo is_iterable($value) ? gettype($value) : 'no';
        echo ':';
        var_dump($value);
        ",
    );
    // The inner array dumps its elements: the boxed Mixed unbox lands on the real
    // array and the walker reads its runtime value_type stamp. It used to print the
    // empty `array(2) {\n}\n` shell.
    assert_eq!(out, "array:array(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\n");
}

/// Verifies an inner `iterable` array appended to a plain array stays boxed and `is_iterable()` is
/// true for it inside the outer foreach.
#[test]
fn test_iterable_value_appended_to_array_stays_boxed() {
    let out = compile_and_run(
        "<?php
        function id(iterable $items): iterable {
            return $items;
        }
        $items = [];
        $items[] = id(['a' => 1]);
        foreach ($items as $value) {
            echo is_iterable($value) ? gettype($value) : 'no';
            echo ':';
            var_dump($value);
        }
        ",
    );
    // The boxed Mixed value is the associative array ['a' => 1]; var_dump now walks the
    // hash after unboxing instead of printing the empty `array(1) {}` shell.
    assert_eq!(out, "array:array(1) {\n  [\"a\"]=>\n  int(1)\n}\n");
}

/// Verifies a `mixed` variadic parameter receiving an `iterable` value preserves it as a boxed array
/// inside the runtime variadic array; `collect(id([1, 2]))` outputs "[[1,2]]" via `json_encode`.
#[test]
fn test_iterable_variadic_arg_stays_boxed_in_runtime_array() {
    let out = compile_and_run(
        "<?php
        function id(iterable $items): iterable {
            return $items;
        }
        function collect(...$items): void {
            echo json_encode($items);
        }
        collect(id([1, 2]));
        ",
    );
    assert_eq!(out, "[[1,2]]");
}

// --- Issue #580: by-ref foreach over an array-element source ---
//
// `lower_foreach` read its source with a plain rvalue `lower_expr` regardless of the binding
// mode. For an array element that read is an `array_get`, which hands back the parent's own
// container *with a retain*: the element sits at refcount 2 while the loop runs, so `iter_start`
// copy-on-writes it, and the loop mutates a private copy and drops it. Every write was lost.
// A plain local source worked because loading a local yields the local's storage with no retain,
// leaving refcount 1 and no copy.
//
// The by-ref source now takes a fetch-for-write read, which does the copy-on-write split itself
// — receiver first, then element, publishing each back into the slot it came from — and returns
// the element borrowed. Note that merely dropping the retain would be WORSE than the original
// bug: `__rt_array_ensure_unique` consumes one reference from the shared source when it splits,
// so a read that never took one would have the split cannibalize the parent's own reference.
// The plain read's missing-key warning and null-container sentinel are reused unchanged.
//
// Both receiver kinds are covered. `Op::ArrayGetForWrite` reaches an indexed element slot with
// pointer arithmetic; `Op::HashGetForWrite` takes the matching entry's address from
// `__rt_hash_get` and splits the container that entry holds. The hash half never needed the
// reference BINDING the checker rejects for `$r = &$h['k'];` — that binds a hash slot into a
// local, whereas this only separates storage the parent already owns.

/// Regression for issue #580: by-ref `foreach` over an indexed element must mutate the parent
/// array in place. Pre-fix the loop ran and assigned `$v` but `$a[0]` stayed `[1, 2]`.
#[test]
fn test_regression_580_by_ref_foreach_over_indexed_element_source() {
    let out = compile_and_run(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out, "2,4");
}

/// Regression for the hash half of issue #580: a by-ref `foreach` over a HASH element must
/// mutate the parent too. The defect was the same one the indexed half had — `Op::HashGet`
/// hands back the parent's container with a retain, so `iter_start` copy-on-writes it — but the
/// fetch-for-write read was gated to indexed receivers, so this shape kept losing every write.
///
/// Note this never needed the reference *binding* the checker rejects for `$r = &$h['k'];`:
/// the fix separates the element and iterates the parent's own storage, it does not alias a
/// hash slot into a local.
#[test]
fn test_regression_580_by_ref_foreach_over_hash_element_source() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => [1, 2]];
foreach ($h['a'] as &$w) { $w = $w * 10; }
unset($w);
echo implode(',', $h['a']);
"#,
    );
    assert_eq!(out, "10,20");
}

/// Regression for issue #580: the mutation must be visible to the parent *during* the loop,
/// not merely published at the end — PHP's by-ref binding aliases the element's storage, so
/// reading the parent mid-loop already reflects the write.
#[test]
fn test_regression_580_by_ref_foreach_element_mutation_visible_during_loop() {
    let out = compile_and_run(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as &$v) { $v = $v * 2; echo $a[0][0], ';'; }
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out, "2;2;2,4");
}

/// Regression for issue #580: a two-level element source must alias through both levels.
#[test]
fn test_regression_580_by_ref_foreach_over_nested_element_source() {
    let out = compile_and_run(
        r#"<?php
$a = [[[1, 2]]];
foreach ($a[0][0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0][0]);
"#,
    );
    assert_eq!(out, "2,4");
}

/// Regression for issue #580: the key-and-value form takes the same source path, so it must
/// mutate the parent too.
#[test]
fn test_regression_580_by_ref_foreach_element_source_with_key() {
    let out = compile_and_run(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as $k => &$v) { $v = $v + $k; }
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out, "1,3");
}

/// Guard for issue #580: a by-VALUE `foreach` over an element source must NOT mutate the
/// parent. The fix gives only the by-ref source a borrowed read; the by-value source must keep
/// its retaining read, or this loop would start writing through to `$a[0]`.
#[test]
fn test_regression_580_by_value_foreach_over_element_source_does_not_mutate() {
    let out = compile_and_run(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as $v) { $v = $v * 2; }
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out, "1,2");
}

/// Guard for issue #580: the plain-local source already worked and must keep working.
#[test]
fn test_regression_580_by_ref_foreach_over_plain_local_still_mutates() {
    let out = compile_and_run(
        r#"<?php
$p = [1, 2];
foreach ($p as &$v) { $v = $v * 3; }
unset($v);
echo implode(',', $p);
"#,
    );
    assert_eq!(out, "3,6");
}

/// Guard for issue #580: iterating an element source by reference must stay heap-clean — the
/// borrowed read skips the retain, so a stray release would free storage the parent still owns
/// and a stray retain would leak the element.
#[test]
fn test_regression_580_by_ref_foreach_element_source_is_leak_free() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out.stdout, "2,4");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: the element must still reach the loop mutable when a SECOND owner
/// holds it, which is where a plain non-retaining read goes badly wrong.
///
/// The outer by-value loop boxes each row, so `$a[$k]` sits at refcount 2 while the inner loop
/// runs. `__rt_array_ensure_unique` splits on that, and it CONSUMES one reference from the
/// shared original when it does — a read that never took a reference of its own would therefore
/// have the split cannibalize the parent's own reference, leaving `$a[$k]` dangling and the next
/// iteration writing into freed storage (this program printed `6,8|3,4` from a reused block).
/// The fetch-for-write read does the split itself and publishes the separated container back
/// into the parent slot, so the parent keeps a live element and the writes land in it.
#[test]
fn test_regression_580_by_ref_foreach_element_source_shared_with_another_owner() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2], [3, 4]];
foreach ($a as $k => $row) {
    foreach ($a[$k] as &$v) { $v = $v * 2; }
    unset($v);
}
echo implode(',', $a[0]), '|', implode(',', $a[1]);
"#,
    );
    assert_eq!(out.stdout, "2,4|6,8");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: a HASH element of an indexed source must be separated with the
/// hash copy-on-write helper, not the indexed-array one.
///
/// `array<array{...}>` reaches the same fetch-for-write path as a nested indexed array, but its
/// element is hash storage: splitting it with `__rt_array_ensure_unique` would hand a hash
/// pointer to the indexed shallow-clone helper. Pairing the helper with the element's container
/// kind is what keeps this shape correct, and it is what lets the hash RECEIVER path reuse the
/// same helper selection.
#[test]
fn test_regression_580_by_ref_foreach_over_hash_element_of_indexed_source() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [['x' => 1, 'y' => 2]];
foreach ($a[0] as $k => &$v) { $v = $v * 2; }
unset($v);
echo $a[0]['x'], ',', $a[0]['y'];
"#,
    );
    assert_eq!(out.stdout, "2,4");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580: a missing index must keep warning and skipping the loop instead of
/// reaching the fetch-for-write path with an out-of-bounds slot address.
///
/// The fetch-for-write read reuses `array_get`'s bounds and null-container guards precisely so
/// this stays true: the miss takes the warning path and materializes the null-container
/// sentinel, which `iter_start` normalizes and `iter_next` reports as empty (issue #556). A
/// fresh read path that computed the element address first would write through a slot past the
/// end of the array.
#[test]
fn test_regression_580_by_ref_foreach_over_missing_element_index_warns_and_skips() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2]];
foreach ($a[7] as &$v) { $v = $v * 2; }
echo 'after:', implode(',', $a[0]);
"#,
    );
    assert_eq!(out.stdout, "after:1,2");
    assert!(
        out.stderr.contains("Undefined array key 7"),
        "expected the missing-index warning to survive the fetch-for-write read, got: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: the hash receiver must make the mutation visible to the parent
/// *during* the loop, not only after it, exactly like the indexed one.
#[test]
fn test_regression_580_by_ref_foreach_hash_element_mutation_visible_during_loop() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => [1, 2]];
foreach ($h['a'] as &$w) { $w = $w * 10; echo $h['a'][0], ';'; }
unset($w);
echo implode(',', $h['a']);
"#,
    );
    assert_eq!(out, "10;10;10,20");
}

/// Regression for issue #580: a HASH element of a HASH receiver must be separated with the hash
/// copy-on-write helper, pairing the split helper with the element's container kind on the hash
/// receiver path too.
#[test]
fn test_regression_580_by_ref_foreach_over_hash_element_of_hash_source() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$h = ['a' => ['x' => 1, 'y' => 2]];
foreach ($h['a'] as $k => &$w) { $w = $w * 10; }
unset($w);
echo $h['a']['x'], ',', $h['a']['y'];
"#,
    );
    assert_eq!(out.stdout, "10,20");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: a chain mixing hash and hash levels must fetch every level for
/// write, so the innermost split is published into storage the outer levels really share.
#[test]
fn test_regression_580_by_ref_foreach_over_nested_hash_element_source() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => ['b' => [1, 2]]];
foreach ($h['a']['b'] as &$w) { $w = $w * 10; }
unset($w);
echo implode(',', $h['a']['b']);
"#,
    );
    assert_eq!(out, "10,20");
}

/// Regression for issue #580: selecting a later key of a multi-entry hash must separate that
/// entry's own slot, not whichever entry the probe happens to land on first.
#[test]
fn test_regression_580_by_ref_foreach_hash_element_source_selects_the_right_entry() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => [1, 2], 'b' => [3, 4]];
foreach ($h['b'] as &$w) { $w = $w * 10; }
unset($w);
echo implode(',', $h['a']), '|', implode(',', $h['b']);
"#,
    );
    assert_eq!(out, "1,2|30,40");
}

/// Guard for issue #580: a by-VALUE `foreach` over a hash element must NOT mutate the parent.
/// Only the by-ref source takes the fetch-for-write read.
#[test]
fn test_regression_580_by_value_foreach_over_hash_element_does_not_mutate() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => [1, 2]];
foreach ($h['a'] as $w) { $w = $w * 10; }
echo implode(',', $h['a']);
"#,
    );
    assert_eq!(out, "1,2");
}

/// Guard for issue #580: a missing hash KEY must keep warning and skipping the loop instead of
/// separating a slot the probe never found. `__rt_hash_get` reports the miss with a null entry
/// address, which the fetch-for-write read routes to the same warning and null-container
/// sentinel `hash_get` produces.
#[test]
fn test_regression_580_by_ref_foreach_over_missing_hash_key_warns_and_skips() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$h = ['a' => [1, 2]];
foreach ($h['zz'] as &$w) { $w = $w * 10; }
echo 'after:', implode(',', $h['a']);
"#,
    );
    assert_eq!(out.stdout, "after:1,2");
    assert!(
        out.stderr.contains("Undefined array key \"zz\""),
        "expected the missing-key warning to survive the fetch-for-write read, got: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: the hash element must reach the loop mutable when a SECOND owner
/// holds it, the case where a plain non-retaining read would have the split cannibalize the
/// parent's own reference. Mirrors the indexed shared-owner regression.
#[test]
fn test_regression_580_by_ref_foreach_hash_element_shared_with_another_owner() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$h = ['a' => [1, 2]];
$keep = $h['a'];
foreach ($h['a'] as &$w) { $w = $w * 10; }
unset($w);
echo implode(',', $h['a']), '|', implode(',', $keep);
"#,
    );
    assert_eq!(out.stdout, "10,20|1,2");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: the loop's lifetime pin must cover the hash receiver path too.
///
/// `HashGetForWrite` hands the loop the entry's own container, borrowed, so dropping the parent
/// hash mid-body would leave the iterator on freed storage exactly as it did for an indexed
/// receiver. The pin is keyed on the source's `Borrowed` ownership rather than on the specific
/// op, so it applies here unchanged — this test is what holds that true.
#[test]
fn test_regression_580_by_ref_foreach_hash_element_source_survives_parent_replacement() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$h = ['a' => [1, 2]];
foreach ($h['a'] as &$w) {
    echo $w, ',';
    if ($w === 1) $h = [];
    $w *= 10;
}
unset($w);
echo 'done';
"#,
    );
    assert_eq!(out.stdout, "1,2,done");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580: separating the hash element must separate the RECEIVER first, so a
/// second owner of the hash itself keeps observing the unmutated storage.
#[test]
fn test_regression_580_by_ref_foreach_hash_element_separates_shared_receiver() {
    let out = compile_and_run(
        r#"<?php
$h = ['a' => [1, 2]];
$copy = $h;
foreach ($h['a'] as &$w) { $w = $w * 10; }
unset($w);
echo implode(',', $h['a']), '|', implode(',', $copy['a']);
"#,
    );
    assert_eq!(out, "10,20|1,2");
}

/// Regression for issue #580: the RECEIVER must be separated too, so the write is not visible
/// through an alias of the parent array.
///
/// PHP separates `$a` on the way to separating `$a[0]`, and elephc's element WRITE path already
/// does (`$a[0] = [9, 9]` splits the receiver inside `__rt_array_set_*`). Fetch-for-write
/// publishes a new element pointer into the receiver's payload, so it owes the same guarantee:
/// without it, `$b` — which shares the outer container — would observe the loop's writes.
#[test]
fn test_regression_580_by_ref_foreach_element_source_separates_aliased_parent() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2]];
$b = $a;
foreach ($a[0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0]), '|', implode(',', $b[0]);
"#,
    );
    assert_eq!(out.stdout, "2,4|1,2");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: an alias of the ELEMENT keeps the pre-loop value, because the
/// copy-on-write split gives the parent a fresh container and leaves the old one to its other
/// owner. This is the half a plain `array_get` read could never get right: it shared the element
/// with `$row` and then mutated a third, private copy that nobody could see.
#[test]
fn test_regression_580_by_ref_foreach_element_source_leaves_element_alias_untouched() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2]];
$row = $a[0];
foreach ($a[0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0]), '|', implode(',', $row);
"#,
    );
    assert_eq!(out.stdout, "2,4|1,2");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580: a subscript CHAIN is separated top-down, so an alias of an
/// intermediate level keeps its pre-loop value.
///
/// `foreach ($a[0][0] as &$v)` separates `$a`, then `$a[0]` into `$a`'s slot, then `$a[0][0]`
/// into `$a[0]`'s slot — the same order PHP uses. Lowering the intermediate `$a[0]` with a plain
/// read instead would leave it shared with `$mid`, and publishing the innermost split into it
/// would show up there.
#[test]
fn test_regression_580_by_ref_foreach_nested_element_source_separates_whole_chain() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[[1, 2]]];
$mid = $a[0];
foreach ($a[0][0] as &$v) { $v = $v * 2; }
unset($v);
echo implode(',', $a[0][0]), '|', implode(',', $mid[0]);
"#,
    );
    assert_eq!(out.stdout, "2,4|1,2");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580 (PR review follow-up): the loop must keep its borrowed element
/// source alive when the body drops the parent that owned it.
///
/// `ArrayGetForWrite` hands the loop the parent's own element, borrowed — the parent's slot is
/// the only owner. `$a = []` inside the body releases the outer container, which releases the
/// element with it, so the iterator was left pointing at freed storage: iteration stopped after
/// the first element (`1,done` instead of `1,2,done`) and `$v *= 2` wrote through a dangling
/// pointer. The loop therefore takes an explicit lifetime reference of its own, after the
/// copy-on-write separation so the pin cannot trigger a second split, and drops it on the way
/// out.
#[test]
fn test_regression_580_by_ref_foreach_element_source_survives_parent_replacement() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2]];
foreach ($a[0] as &$v) {
    echo $v, ',';
    if ($v === 1) $a = [];
    $v *= 2;
}
unset($v);
echo 'done';
"#,
    );
    assert_eq!(out.stdout, "1,2,done");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Regression for issue #580 (PR review follow-up): the same lifetime guarantee under a nested
/// source, where the dropped parent is an intermediate level of the subscript chain.
#[test]
fn test_regression_580_by_ref_foreach_nested_element_source_survives_parent_replacement() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[[1, 2]]];
foreach ($a[0][0] as &$v) {
    echo $v, ',';
    if ($v === 1) $a = [];
    $v *= 2;
}
unset($v);
echo 'done';
"#,
    );
    assert_eq!(out.stdout, "1,2,done");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580 (PR review follow-up): the lifetime pin must be dropped on a `break`,
/// not only on normal loop termination, or the element leaks.
#[test]
fn test_regression_580_by_ref_foreach_element_source_pin_released_on_break() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2, 3]];
foreach ($a[0] as &$v) {
    $v = $v * 2;
    if ($v === 4) { break; }
}
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out.stdout, "2,4,3");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580 (PR review follow-up): a `break 2` out of an outer loop skips the inner
/// loop's exit block, so the pin has to be dropped by the multi-level break cleanup path.
#[test]
fn test_regression_580_by_ref_foreach_element_source_pin_released_on_multi_level_break() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2, 3]];
$total = 0;
foreach ([1, 2] as $round) {
    foreach ($a[0] as &$v) {
        $total = $total + $v;
        if ($v === 2) { break 2; }
    }
    unset($v);
}
echo $total;
"#,
    );
    assert_eq!(out.stdout, "3");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580 (PR review follow-up): a `return` out of the loop body never reaches the
/// exit block, so the pin has to be dropped by the return cleanup path.
#[test]
fn test_regression_580_by_ref_foreach_element_source_pin_released_on_return() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function first_doubled(array $rows): int {
    foreach ($rows[0] as &$v) {
        $v = $v * 2;
        return $v;
    }
    return 0;
}
echo first_doubled([[5, 6]]);
"#,
    );
    assert_eq!(out.stdout, "10");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}

/// Guard for issue #580 (PR review follow-up): leaving the loop by throwing must drop the pin
/// too, so an exception caught outside the loop leaves a clean heap.
#[test]
fn test_regression_580_by_ref_foreach_element_source_pin_released_on_throw() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$a = [[1, 2, 3]];
try {
    foreach ($a[0] as &$v) {
        $v = $v * 2;
        if ($v === 4) { throw new Exception('stop'); }
    }
} catch (Exception $e) {
    echo $e->getMessage(), ':';
}
unset($v);
echo implode(',', $a[0]);
"#,
    );
    assert_eq!(out.stdout, "stop:2,4,3");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "expected a clean heap-debug leak summary, got: {}",
        out.stderr
    );
}
