//! Purpose:
//! Integration or regression tests for the OWNERSHIP path of PHP's `(object)` cast: the source
//! is lowered into a synthetic local and handed to an ordinary user-function call, so a
//! refcounted array, string or boxed Mixed source must be released exactly once and an object
//! source must be neither copied nor released.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every fixture derives its value from `$argc` so constant folding cannot erase the
//!   allocation under test before lowering sees it, and loops the cast so a per-iteration leak
//!   shows up as a growing heap rather than a single stray block.
//! - Each assertion checks `HEAP DEBUG: leak summary: clean` alongside the output: a wrong
//!   ownership decision here is a silent leak, not a wrong answer.

use crate::support::*;

/// A temporary array source is released once: the cast consumes it into the stdClass's
/// properties and nothing keeps the array alive.
#[test]
fn test_object_cast_of_a_temporary_array_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$value = (object) ["name" => "n" . $argc, "port" => $argc];
echo $value->name, "|", $value->port;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "n1|1");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Casting in a LOOP is what turns a per-cast leak into a visible one: the array source, the
/// stdClass and its property boxes are all allocated each iteration.
#[test]
fn test_repeated_object_casts_leave_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$total = 0;
for ($i = 0; $i < 200; $i++) {
    $value = (object) ["k" => "v" . $i, "n" => $i + $argc];
    $total = $total + $value->n;
}
echo $total;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "20100");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// An owned STRING source reaches the `scalar` property and is released once.
#[test]
fn test_object_cast_of_an_owned_string_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($i = 0; $i < 200; $i++) {
    $value = (object) ("payload-" . $i . "-" . $argc);
    if (strlen($value->scalar) === 0) { echo "bad"; }
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// A runtime-typed (boxed `mixed`) source takes the dynamic helper, whose identity arm returns
/// the payload unchanged. The box must still be released exactly once on every arm.
#[test]
fn test_object_cast_of_a_mixed_source_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Tag { public string $s = "tag"; }
function pick(int $i): mixed {
    if ($i % 3 === 0) { return new Tag(); }
    if ($i % 3 === 1) { return ["k" => "a" . $i]; }
    return "s" . $i;
}
$count = 0;
for ($i = 0; $i < 200; $i++) {
    $value = (object) pick($i + $argc - 1);
    $count = $count + 1;
}
echo $count;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "200");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// An OBJECT source is the identity: no copy is allocated, and the cast must not release the
/// instance the program still holds under its own name.
#[test]
fn test_object_cast_of_an_object_neither_copies_nor_releases() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Box { public int $n = 0; }
for ($i = 0; $i < 200; $i++) {
    $box = new Box();
    $box->n = $i + $argc;
    $cast = (object) $box;
    $cast->n = $cast->n + 1;
    if ($box->n !== $i + $argc + 1) { echo "bad"; }
}
echo "ok";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ok");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// The cast result itself is an ordinary owned object: dropping it at the end of a function
/// releases the stdClass and every property box the conversion allocated.
#[test]
fn test_object_cast_result_dropped_in_a_function_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function width(array $values): int {
    $object = (object) $values;
    return strlen($object->k);
}
$total = 0;
for ($i = 0; $i < 200; $i++) {
    $total = $total + width(["k" => "v" . $i . $argc]);
}
echo $total;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "890");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}
