//! Purpose:
//! Checks string-key writes and nested appends on lists and reference variadic copies.
//!
//! Called from:
//! - The runtime GC codegen integration test module.
//!
//! Key details:
//! - Reference markers survive container copies and promotion without becoming scalar values.
//! - Missing nested parents are installed in the mutated container, with balanced ownership.

use crate::support::compile_and_run_with_heap_debug;

/// Requires PHP-visible values, silent legal autovivification and a clean final heap.
fn assert_key_write(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(!out.stderr.contains("Undefined array key"), "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A by-value variadic copy retains referenced caller slots and reads their current values.
#[test]
fn test_variadic_third_review_copy_numeric_string_key() {
    assert_key_write(r#"<?php
function writeCopy(&...$items): void {
    $copy = $items;
    $copy["1"] = "Q";
    echo $copy[0], "/", $copy[1], "/", $copy[2], "|", json_encode($copy), "|";
}
$a = "A"; $b = "B"; $c = "C";
$write = writeCopy(...); $write($a, $b, $c);
echo $a, "/", $b, "/", $c, "\n";
"#, "A/Q/C|[\"A\",\"Q\",\"C\"]|A/Q/C\n");
}

/// Creating a named nested parent does not overwrite an unrelated variadic caller slot.
#[test]
fn test_variadic_third_review_nested_string_parent() {
    assert_key_write(r#"<?php
function writeNested(&...$items): void {
    $items["key"]["x"] = 1;
    echo $items[0], "/", $items[1], "/", $items[2], "|", json_encode($items), "|";
}
$a = "A"; $b = "B"; $c = "C";
$write = writeNested(...); $write($a, $b, $c);
echo $a, "/", $b, "/", $c, "\n";
"#, "A/B/C|{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":{\"x\":1}}|A/B/C\n");
}

/// The same nested parent is attached to a plain list without changing its COW sibling.
#[test]
fn test_variadic_third_review_plain_list_nested_parent() {
    assert_key_write(r#"<?php
$items = ["A", "B", "C"]; $copy = $items;
$items["key"]["x"] = 1;
echo json_encode($items), "|", json_encode($copy), "\n";
"#, "{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":{\"x\":1}}|[\"A\",\"B\",\"C\"]\n");
}

/// A string-key nested append can probe and grow a reference variadic container.
#[test]
fn test_variadic_third_review_nested_string_append() {
    assert_key_write(r#"<?php
function appendNested(&...$items): void {
    $items["key"][] = 1; $items["key"][] = 2;
    echo $items[0], "/", $items[1], "/", $items[2], "|", json_encode($items), "|";
}
$a = "A"; $b = "B"; $c = "C";
$write = appendNested(...); $write($a, $b, $c);
echo $a, "/", $b, "/", $c, "\n";
"#, "A/B/C|{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":[1,2]}|A/B/C\n");
}

/// Repeated nested appends on a plain list preserve both prior values and its copied sibling.
#[test]
fn test_variadic_third_review_plain_list_nested_append() {
    assert_key_write(r#"<?php
$items = ["A", "B", "C"]; $copy = $items;
$items["key"][] = 1; $items["key"][] = 2;
echo json_encode($items), "|", json_encode($copy), "\n";
"#, "{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":[1,2]}|[\"A\",\"B\",\"C\"]\n");
}

/// A nested write into a by-value variadic copy adds only its private named parent.
#[test]
fn test_variadic_third_review_copy_nested_string_parent() {
    assert_key_write(r#"<?php
function writeCopyNested(&...$items): void {
    $copy = $items; $copy["key"]["x"] = 1;
    echo json_encode($copy), "|", json_encode($items), "|";
}
$a = "A"; $b = "B"; $c = "C";
$write = writeCopyNested(...); $write($a, $b, $c);
echo $a, "/", $b, "/", $c, "\n";
"#, "{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":{\"x\":1}}|[\"A\",\"B\",\"C\"]|A/B/C\n");
}

/// Runtime string-key isset probes stay silent, recognize null and retire their result cells.
#[test]
fn test_variadic_third_review_mixed_key_isset_control() {
    assert_key_write(r#"<?php
function probeKeys(array $items, mixed $key): void {
    echo isset($items[$key]) ? "yes" : "no", ":";
    echo isset($items["missing"]) ? "yes" : "no", "\n";
}
$items = [null, "B"]; $key = $argc == 1 ? "1" : 1;
probeKeys($items, $key); probeKeys($items, "0");
$items["key"] = "V"; probeKeys($items, "key");
$items["key"] = null; probeKeys($items, "key");
"#, "yes:no\nno:no\nyes:no\nno:no\n");
}

/// An owned computed string parent is used once and freed after repeated variadic writes.
#[test]
fn test_variadic_third_review_owned_string_parent() {
    assert_key_write(r#"<?php
function parentKey(string $suffix): string { return "k" . $suffix; }
function writeOwnedKeys(string $suffix, &...$items): void {
    $items[parentKey($suffix)]["x"] = 1;
    $items[parentKey($suffix)]["y"] = 2;
    echo json_encode($items), "|";
}
$a = "A"; $b = "B"; $c = "C";
$write = writeOwnedKeys(...); $write("ey", $a, $b, $c);
echo $a, "/", $b, "/", $c, "\n";
"#, "{\"0\":\"A\",\"1\":\"B\",\"2\":\"C\",\"key\":{\"x\":1,\"y\":2}}|A/B/C\n");
}
