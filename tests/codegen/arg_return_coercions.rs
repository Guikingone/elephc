//! Purpose:
//! Integration regressions for PHP call/return boundaries and flow-sensitive local storage.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures exercise runtime narrowing, representation changes, exceptions, value
//!   shape, and ownership behavior without relying on application-specific source patterns.

use crate::support::*;

/// Verifies concrete arrays and Traversable objects satisfy an `iterable` return by forwarding the
/// runtime payload instead of emitting a placeholder heap coercion.
#[test]
fn test_concrete_values_return_as_iterable() {
    let out = compile_and_run(
        r#"<?php
class ReturnIterator implements Iterator {
    private int $position = 0;
    public function current(): mixed { return 5; }
    public function key(): mixed { return $this->position; }
    public function next(): void { $this->position++; }
    public function rewind(): void { $this->position = 0; }
    public function valid(): bool { return $this->position < 1; }
}
function indexedValues(): iterable {
    return [1, 2];
}
function associativeValues(): iterable {
    return ['left' => 3, 'right' => 4];
}
function objectValues(): iterable {
    return new ReturnIterator();
}
foreach (indexedValues() as $value) {
    echo $value;
}
echo '|';
foreach (associativeValues() as $key => $value) {
    echo $key.$value;
}
echo '|';
foreach (objectValues() as $value) {
    echo $value;
}
"#,
    );
    assert_eq!(out, "12|left3right4|5");
}

/// Verifies an exception handler inside a statically infinite loop does not keep the loop's
/// synthetic exit reachable and materialize a zero-operand `iterable` return placeholder.
#[test]
fn test_infinite_loop_with_exception_handler_has_no_synthetic_iterable_fallthrough() {
    let out = compile_and_run(
        r#"<?php
function maybeThrow(bool $throw): void {
    if ($throw) {
        throw new Exception('boom');
    }
}
function values(bool $throw): iterable {
    while (true) {
        try {
            maybeThrow($throw);
            return ['ok' => 7];
        } catch (Exception $e) {
            return [];
        }
    }
}
foreach (values(false) as $key => $value) {
    echo $key.$value;
}
"#,
    );
    assert_eq!(out, "ok7");
}

/// Verifies an owning array temporary transferred to an `iterable` return remains balanced across
/// repeated calls and result cleanup.
#[test]
fn test_array_temporary_returned_as_iterable_has_balanced_ownership() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function returnedValues(int $value): iterable {
    return [$value, $value + 1];
}
$sum = 0;
for ($i = 0; $i < 100; $i++) {
    foreach (returnedValues($i) as $value) {
        $sum += $value;
    }
}
echo $sum;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "10000");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A PHP `array` return contract accepts associative storage even when the inferred values are
/// heterogeneous and therefore boxed. This is the shape produced by key-preserving
/// `array_unique(array_merge(...))` helpers.
#[test]
fn test_assoc_mixed_result_satisfies_untyped_array_return() {
    let out = compile_and_run(
        r#"<?php
function makeCandidateList(): array {
    $values = ["first" => "alpha", "number" => 7, "duplicate" => "alpha"];
    return array_unique($values);
}
$result = makeCandidateList();
echo count($result), "|", $result["first"], "|", $result["number"];
"#,
    );
    assert_eq!(out, "2|alpha|7");
}

/// Verifies a boxed object is checked against the declared parameter type and unboxed without
/// depending on the caller's source-level inference.
#[test]
fn test_mixed_object_argument_is_narrowed_at_parameter_boundary() {
    let out = compile_and_run(
        r#"<?php
final class Box {
    public function value(): int { return 7; }
}
function take(Box $box): int { return $box->value(); }
function relay(mixed $value): int { return take($value); }
echo relay(new Box());
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies a boxed value of the wrong runtime type raises a catchable `TypeError` at an object
/// parameter boundary.
#[test]
fn test_mixed_object_argument_rejects_wrong_runtime_type() {
    let out = compile_and_run(
        r#"<?php
final class Box {}
function take(Box $box): void { echo 'bad'; }
function relay(mixed $value): void { take($value); }
try {
    relay(42);
} catch (TypeError $error) {
    echo 'caught';
}
"#,
    );
    assert_eq!(out, "caught");
}

/// Verifies boxed unions use the same checked object boundary as a value declared directly as
/// `mixed`, including the catchable failure path for the union's null alternative.
#[test]
fn test_nullable_object_argument_is_narrowed_at_parameter_boundary() {
    let out = compile_and_run(
        r#"<?php
final class Box {
    public function value(): int { return 9; }
}
function take(Box $box): int { return $box->value(); }
function choose(bool $present): Box|null {
    return $present ? new Box() : null;
}
echo take(choose(true)), '|';
try {
    take(choose(false));
} catch (TypeError $error) {
    echo 'caught';
}
"#,
    );
    assert_eq!(out, "9|caught");
}

/// Verifies a boxed scalar union can cross a matching declared parameter boundary when its
/// runtime alternative satisfies that declaration.
#[test]
fn test_nullable_string_argument_uses_boxed_boundary_conversion() {
    let out = compile_and_run(
        r#"<?php
function take(string $value): void { echo $value; }
function choose(bool $present): string|null {
    return $present ? 'ok' : null;
}
take(choose(true));
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies an empty first argument does not freeze a declared generic array parameter to the
/// empty-literal placeholder and reject later populated arrays.
#[test]
fn test_generic_array_parameter_ignores_empty_literal_specialization() {
    let out = compile_and_run(
        r#"<?php
final class Counter {
    public function size(array $values): int { return count($values); }
}
$counter = new Counter();
echo $counter->size([]), '|', $counter->size([1, 2]);
"#,
    );
    assert_eq!(out, "0|2");
}

/// Verifies a declared bare array contract accepts indexed and associative values at every direct
/// call surface while representation normalization leaves the caller's containers readable.
#[test]
fn test_generic_array_parameter_accepts_multiple_runtime_shapes() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function firstValue(array $values): mixed {
    foreach ($values as $value) { return $value; }
    return null;
}
final class ArrayReader {
    public function first(array $values): mixed {
        foreach ($values as $value) { return $value; }
        return null;
    }
    public static function firstStatic(array $values): mixed {
        foreach ($values as $value) { return $value; }
        return null;
    }
}
$assoc = ['left' => 1];
$indexed = [2, 3];
$reader = new ArrayReader();
echo firstValue($assoc), firstValue($indexed), '|';
echo $reader->first($assoc), $reader->first($indexed), '|';
echo ArrayReader::firstStatic($assoc), ArrayReader::firstStatic($indexed), '|';
echo $assoc['left'], $indexed[0];
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "12|12|12|12");
}

/// Verifies boxed indexed and associative arrays satisfy a generic array return contract while
/// preserving their keys and values.
#[test]
fn test_mixed_array_return_is_narrowed_at_return_boundary() {
    let out = compile_and_run(
        r#"<?php
function relay(mixed $value): array { return $value; }
$indexed = relay([2, 3]);
$assoc = relay(['left' => 5]);
echo $indexed[0], $indexed[1], '|', $assoc['left'];
"#,
    );
    assert_eq!(out, "23|5");
}

/// Verifies a boxed non-array raises a catchable `TypeError` at a generic array return boundary.
#[test]
fn test_mixed_array_return_rejects_wrong_runtime_type() {
    let out = compile_and_run(
        r#"<?php
function relay(mixed $value): array { return $value; }
try {
    relay('not an array');
} catch (TypeError $error) {
    echo 'caught';
}
"#,
    );
    assert_eq!(out, "caught");
}

/// Verifies an ordinary PHP local may change representation across sequential assignments while
/// earlier and later reads continue to use the final boxed frame slot safely.
#[test]
fn test_untyped_local_can_be_reassigned_across_runtime_types() {
    let out = compile_and_run(
        r#"<?php
$value = 42;
echo $value, '|';
$value = 'text';
echo $value, '|';
$value = ['key' => 9];
echo $value['key'];
"#,
    );
    assert_eq!(out, "42|text|9");
}
