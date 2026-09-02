//! Purpose:
//! Integration tests for list destructuring from narrowed and associative array values.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Null guards ending in `continue` or `break` must preserve the non-null complement.
//! - Associative-array storage remains a valid RHS when positional integer keys are present.

use crate::support::*;

/// Verifies a null guard ending in `continue` narrows `?array` to `Array` before list unpacking.
#[test]
fn test_null_guard_continue_narrows_list_unpack_rhs() {
    let out = compile_and_run(
        r#"<?php
final class R {
    private function mk(int $n): ?array {
        if ($n < 0) { return null; }
        return ["k" . $n, "v" . $n];
    }
    public function run(): string {
        $out = "";
        foreach ([1, -1, 2] as $n) {
            $entry = $this->mk($n);
            if ($entry === null) { continue; }
            [$key, $value] = $entry;
            $out .= $key . "=" . $value . ";";
        }
        return $out;
    }
}
echo (new R())->run();
"#,
    );
    assert_eq!(out, "k1=v1;k2=v2;");
}

/// Verifies a null guard ending in `break` narrows `?array` to `Array` before list unpacking.
#[test]
fn test_null_guard_break_narrows_list_unpack_rhs() {
    let out = compile_and_run(
        r#"<?php
function row(int $n): ?array {
    if ($n < 0) { return null; }
    return ["k" . $n, "v" . $n];
}
$out = "";
foreach ([1, -1, 2] as $n) {
    $entry = row($n);
    if ($entry === null) { break; }
    [$key, $value] = $entry;
    $out .= $key . "=" . $value . ";";
}
echo $out;
"#,
    );
    assert_eq!(out, "k1=v1;");
}

/// Verifies positional list unpacking accepts associative storage with integer keys.
#[test]
fn test_list_unpack_assoc_array_rhs() {
    let out = compile_and_run(
        r#"<?php
$row = [0 => "left", 1 => "right", "label" => "ignored"];
[$left, $right] = $row;
echo $left . ":" . $right;
"#,
    );
    assert_eq!(out, "left:right");
}

/// Verifies positional list unpacking validates and reads a boxed Mixed array result at runtime.
#[test]
fn test_list_unpack_mixed_function_result() {
    let out = compile_and_run(
        r#"<?php
function resolveRuntime(): mixed {
    return ["app", "args"];
}
[$app, $args] = resolveRuntime();
echo $app . ":" . $args;
"#,
    );
    assert_eq!(out, "app:args");
}

/// Verifies a nullable array can be destructured without prior narrowing and that a null value
/// follows PHP's missing-element behavior instead of being rejected statically.
#[test]
fn test_list_unpack_nullable_array_without_guard() {
    let out = compile_and_run(
        r#"<?php
function row(bool $present): ?array {
    return $present ? ["left", "right"] : null;
}
[$left, $right] = row(true);
echo $left . ":" . $right . ";";
[$missingLeft, $missingRight] = row(false);
var_dump($missingLeft, $missingRight);
"#,
    );
    assert_eq!(out, "left:right;NULL\nNULL\n");
}

/// Verifies destructuring is valid inside a condition, evaluates its RHS once, and yields that
/// original array before the surrounding boolean operators inspect it.
#[test]
fn test_list_unpack_assignment_expression_in_condition() {
    let out = compile_and_run(
        r#"<?php
function serviceEntry(): array {
    echo "E";
    return [null, "ok"];
}
if (!([$file, $code] = serviceEntry()) || null !== $file) {
    echo "bad";
} else {
    echo $code;
}
"#,
    );
    assert_eq!(out, "Eok");
}

/// Verifies `foreach` value destructuring in both spellings PHP accepts (`[...]` and
/// `list(...)`), plus the `$key => [...]` form. Expected output matches `php -r` on 8.4.
#[test]
fn test_foreach_value_destructuring() {
    let out = compile_and_run(
        r#"<?php
$m = [[1, 2], [3, 4]];
foreach ($m as [$a, $b]) { echo $a, "-", $b, ";"; }
foreach ($m as list($a, $b)) { echo $a, "+", $b, ";"; }
foreach ($m as $k => [$a, $b]) { echo $k, ":", $a, ",", $b, ";"; }
"#,
    );
    assert_eq!(out, "1-2;3-4;1+2;3+4;0:1,2;1:3,4;");
}

/// Verifies keyed, skipped-element, and nested `foreach` destructuring patterns, which all
/// reuse the same lowering as a standalone `[...] = $value;` assignment.
#[test]
fn test_foreach_destructuring_keyed_skipped_and_nested() {
    let out = compile_and_run(
        r#"<?php
$pairs = [["name" => "ann", "age" => 30], ["name" => "bob", "age" => 40]];
foreach ($pairs as ["name" => $n, "age" => $g]) { echo $n, "=", $g, ";"; }
$skip = [[1, 2, 3], [4, 5, 6]];
foreach ($skip as [, $second]) { echo $second, ";"; }
$nested = [[1, [2, 3]], [4, [5, 6]]];
foreach ($nested as [$x, [$y, $z]]) { echo $x, $y, $z, ";"; }
"#,
    );
    assert_eq!(out, "ann=30;bob=40;2;5;123;456;");
}

/// Verifies that a heterogeneous tuple yielded after a callback sort stays boxed while a
/// `foreach` destructuring pattern reads its later positional elements.
///
/// The iterator cannot expose the tuple's raw array representation to the synthetic
/// destructuring local: that local is gradual because array elements are refcounted.  Each
/// generated `$row[$offset]` therefore must receive a boxed `Mixed` cell, exactly as a direct
/// gradual list-unpack read does.
#[test]
fn test_foreach_destructuring_after_uasort_of_heterogeneous_tuples() {
    let out = compile_and_run(
        r#"<?php
$services = [];
$services[] = [0, 0, null, "first", null];
$services[] = [1, 1, "named", "second", "ServiceClass"];
uasort($services, static fn ($a, $b) => $b[0] <=> $a[0] ?: $a[1] <=> $b[1]);
foreach ($services as [, , $index, $serviceId, $class]) {
    echo ($index ?? "none"), ":", $serviceId, ":", ($class ?? "none"), ";";
}
"#,
    );
    assert_eq!(out, "named:second:ServiceClass;none:first:none;");
}

/// Verifies destructuring `foreach` loops nest, and that the pattern also works with a
/// single-statement body and inside a function over an `array`-hinted parameter.
#[test]
fn test_foreach_destructuring_nested_loops_and_bodies() {
    let out = compile_and_run(
        r#"<?php
$m = [[1, 2], [3, 4]];
foreach ($m as [$a, $b]) { foreach ($m as [$c, $d]) { echo $a, $b, $c, $d, "|"; } }
foreach ($m as [$a, $b]) echo $a, $b, ";";
function total(array $rows): int { $t = 0; foreach ($rows as [$x, $y]) { $t += $x * $y; } return $t; }
echo total([[1, 2], [3, 4]]);
"#,
    );
    assert_eq!(out, "1212|1234|3412|3434|12;34;14");
}
