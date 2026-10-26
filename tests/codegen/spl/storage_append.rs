//! Purpose:
//! Differential regressions for SPL storage appends, overrides and object-backed refusals.
//!
//! Called from:
//! - `cargo test --test codegen_tests storage_append`.
//!
//! Key details:
//! - PHP resolves nested backing stores without invoking inner user overrides.
//! - Public append honors outer offsetSet overrides and refuses object properties.

use crate::support::*;

/// Nested array storage receives null-offset writes without calling the inner public overrides.
#[test]
fn test_nested_storage_append_bypasses_inner_overrides() {
    let out = compile_and_run(
        r#"<?php
error_reporting(0);
class Inner extends ArrayObject {
    public function append(mixed $value): void { echo "inner-append\n"; }
    public function offsetSet(mixed $key, mixed $value): void { echo "inner-set\n"; }
}
$inner = new Inner([1]);
$outer = new ArrayObject($inner);
$outer[] = 2;
$outer->append(3);
$outer->offsetSet(null, 4);
var_dump($inner->getArrayCopy());
"#,
    );
    assert_eq!(out, "array(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n  [3]=>\n  int(4)\n}\n");
}

/// Dimension append bypasses append overrides while public parent append calls offsetSet overrides.
#[test]
fn test_append_dispatch_honors_only_the_php_outer_override() {
    let out = compile_and_run(
        r#"<?php
class AppendOwner extends ArrayObject {
    public function append(mixed $value): void { echo "append\n"; parent::append($value); }
    public function offsetSet(mixed $key, mixed $value): void {
        echo "set\n"; parent::offsetSet($key, $value);
    }
}
$owner = new AppendOwner([1]);
$owner[] = 2;
$owner->append(3);
var_dump($owner->getArrayCopy());
"#,
    );
    assert_eq!(out, "set\nappend\nset\narray(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n}\n");
}

/// Public append refuses terminal object stores, including nested and self-backed wrappers.
#[test]
fn test_public_append_object_storage_refusal_names_outer_class() {
    let out = compile_and_run(
        r#"<?php
error_reporting(0);
class PropertyStore { public int $x = 5; }
$p = new PropertyStore();
$owner = new ArrayObject($p);
$iterator = new ArrayIterator($owner);
try { $owner->append(2); } catch (Error $e) { echo $e->getMessage(), "\n"; }
try { $iterator->append(3); } catch (Error $e) { echo $e->getMessage(), "\n"; }
class SelfStore extends ArrayObject {
    public function __construct() { parent::__construct($this); }
}
$self = new SelfStore();
try { $self->append(4); } catch (Error $e) { echo $e->getMessage(), "\n"; }
var_dump($p->x);
"#,
    );
    assert_eq!(out, "Cannot append properties to objects, use ArrayObject::offsetSet() instead\nCannot append properties to objects, use ArrayIterator::offsetSet() instead\nCannot append properties to objects, use SelfStore::offsetSet() instead\nint(5)\n");
}

/// A user ArrayAccess implementation receives a null offset, regardless of its append method.
#[test]
fn test_array_access_append_uses_null_offset_set() {
    let out = compile_and_run(
        r#"<?php
class OffsetProbe implements ArrayAccess {
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetSet(mixed $key, mixed $value): void { var_dump($key, $value); }
    public function offsetUnset(mixed $key): void {}
    public function append(mixed $value): void { echo "wrong-append"; }
}
$probe = new OffsetProbe();
$probe[] = 2;
"#,
    );
    assert_eq!(out, "NULL\nint(2)\n");
}

/// Appending heap values through both wrapper kinds preserves input COW and releases all owners.
#[test]
fn test_nested_storage_append_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
error_reporting(0);
for ($i = 0; $i < 20; $i++) {
    $input = [str_repeat("a", 40)];
    $inner = new ArrayObject($input);
    $outer = new ArrayIterator($inner);
    $outer[] = str_repeat("b", 40);
    $outer->append(str_repeat("c", 40));
    $copy = $inner->getArrayCopy();
    $copy[0] = str_repeat("d", 40);
    if (count($copy) !== 3 || count($input) !== 1 || $inner[0] !== $input[0]
        || $copy[1] !== str_repeat("b", 40) || $copy[2] !== str_repeat("c", 40)) { echo "wrong"; }
    unset($copy, $outer, $inner, $input);
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "nested append leaked: {}", out.stderr);
}

/// Refusing append on a self-backed store releases method-frame owners before cycle collection.
#[test]
fn test_self_backed_append_refusal_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
error_reporting(0);
class SelfStore extends ArrayObject {
    public function __construct() { parent::__construct($this); }
}
for ($i = 0; $i < 20; $i++) {
    $self = new SelfStore();
    try { $self->append(4); } catch (Error $e) {}
    unset($e, $self);
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "refused self-backed append leaked: {}", out.stderr);
}
