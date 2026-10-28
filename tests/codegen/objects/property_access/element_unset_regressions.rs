//! Purpose:
//! Regression tests for property element unset across readonly and runtime-typed storage.
//!
//! Called from:
//! - The object property codegen integration suite.
//!
//! Key details:
//! - ArrayAccess receives original offsets, while SPL applies its own conversion rules.
//! - Readonly permits object mutation, and false-offset diagnostics may replace the property.

use super::*;

/// An uninitialized static array skips removal after evaluating and retiring its computed key.
#[test]
fn test_unset_uninitialized_static_array_property_is_quiet() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class QuietStaticOffsetHolder { public static array $items; }
function quiet_static_offset_key(): string { echo "key|"; return str_repeat("k", 2); }
unset(QuietStaticOffsetHolder::$items[quiet_static_offset_key()]);
echo isset(QuietStaticOffsetHolder::$items) ? "initialized|" : "unset|";
QuietStaticOffsetHolder::$items = ["kk" => 1, "keep" => 2];
unset(QuietStaticOffsetHolder::$items[quiet_static_offset_key()]);
echo json_encode(QuietStaticOffsetHolder::$items), "\n";
"#);
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "key|unset|key|{\"keep\":2}\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A readonly mixed property permits ArrayAccess mutation and refuses array mutation.
#[test]
fn test_unset_readonly_mixed_property_dispatches_object_offsets() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ReadonlyOffsetBag implements ArrayAccess {
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetSet(mixed $key, mixed $value): void {}
    public function offsetUnset(mixed $key): void { echo "removed:", $key, "\n"; }
}
class ReadonlyOffsetHolder {
    public readonly mixed $bag;
    public function __construct(mixed $bag) { $this->bag = $bag; }
}
$object = new ReadonlyOffsetHolder(new ReadonlyOffsetBag());
unset($object->bag["key"]);
$array = new ReadonlyOffsetHolder(["key" => 1]);
try { unset($array->bag["key"]); }
catch (Error $error) { echo $error->getMessage(), "\n"; }
echo count($array->bag), "\n";
"#);
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "removed:key\nCannot indirectly modify readonly property ReadonlyOffsetHolder::$bag\n1\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Boxed user ArrayAccess objects accept array and object offsets without array-key conversion.
#[test]
fn test_unset_boxed_array_access_accepts_array_and_object_keys() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class OriginalOffsetBag implements ArrayAccess {
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetSet(mixed $key, mixed $value): void {}
    public function offsetUnset(mixed $key): void { echo gettype($key), "\n"; }
}
class OriginalOffsetHolder { public mixed $bag; }
function original_array_offset(): array { return [1, 2]; }
$holder = new OriginalOffsetHolder();
$holder->bag = new OriginalOffsetBag();
unset($holder->bag[original_array_offset()], $holder->bag[new stdClass()]);
unset($holder);
"#);
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "array\nobject\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// SPL sees the original offset so null errors and fractional-offset deprecations keep PHP types.
#[test]
fn test_unset_boxed_spl_uses_original_offset_conversion() {
    let out = compile_and_run(r#"<?php
class OriginalSplHolder { public mixed $bag; }
$holder = new OriginalSplHolder();
$holder->bag = new SplFixedArray(3);
try { unset($holder->bag[null]); }
catch (TypeError $error) { echo $error->getMessage(), "\n"; }
set_error_handler(function (int $severity, string $message): bool {
    echo $severity, ":", $message, "\n";
    return true;
});
unset($holder->bag[1.5]);
echo "removed\n";
"#);
    assert_eq!(out, "Cannot access offset of type null on SplFixedArray\n8192:Implicit conversion from float 1.5 to int loses precision\nremoved\n");
}

/// A false-offset deprecation observes false and a handler's replacement array survives unchanged.
#[test]
fn test_unset_false_property_reports_deprecation_before_handler_replacement() {
    let out = compile_and_run(r#"<?php
class FalseOffsetHolder { public mixed $value = false; }
$holder = new FalseOffsetHolder();
set_error_handler(function (int $severity, string $message) use ($holder): bool {
    echo $severity, ":", $message, ":", gettype($holder->value), "\n";
    $holder->value = ["key" => 1];
    return true;
});
unset($holder->value["key"]);
echo json_encode($holder->value), "\n";
"#);
    assert_eq!(out, "8192:Automatic conversion of false to array is deprecated:boolean\n{\"key\":1}\n");
}

/// A runtime array receiver rejects array and object offsets without removing integer key zero.
#[test]
fn test_unset_mixed_array_property_rejects_array_and_object_keys() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class InvalidOffsetMarker {}
class InvalidOffsetHolder { public mixed $value = [0 => "keep"]; }
function invalid_unset_key(int $n): mixed { return $n > 0 ? new InvalidOffsetMarker() : [1]; }
$holder = new InvalidOffsetHolder();
for ($i = 0; $i < 2; $i++) {
    try { unset($holder->value[invalid_unset_key($i)]); }
    catch (TypeError $error) { echo $error->getMessage(), "\n"; }
}
echo json_encode($holder->value), "\n";
"#);
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "Cannot unset offset of type array on array\nCannot unset offset of type InvalidOffsetMarker on array\n[\"keep\"]\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A callback may replace its property while the selected object remains alive until return.
#[test]
fn test_unset_boxed_object_keeps_receiver_across_callback() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ReplacedOffsetHolder { public mixed $bag; }
class ReplacedOffsetBag implements ArrayAccess {
    public function __construct(public ReplacedOffsetHolder $holder, public string $name) {}
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetSet(mixed $key, mixed $value): void {}
    public function offsetUnset(mixed $key): void {
        $this->holder->bag = null;
        echo $this->name, "|";
    }
    public function __destruct() { echo "drop|"; }
}
$holder = new ReplacedOffsetHolder();
for ($i = 0; $i < 20; $i++) {
    $holder->bag = new ReplacedOffsetBag($holder, "kept");
    unset($holder->bag["key"]);
    echo "after|";
}
echo "done\n";
"#);
    assert!(out.success, "stdout: {}\nstderr: {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "kept|drop|after|".repeat(20) + "done\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
