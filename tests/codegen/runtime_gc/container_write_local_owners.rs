//! Purpose:
//! Verifies temporary local read ownership across container writes after foreach widens storage.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - A refined array/object can still be loaded from a physically boxed local slot.
//! - Mutation must retire the extracted reference on normal and exceptional paths.

use crate::support::compile_and_run_with_heap_debug;

#[test]
fn test_container_write_local_owners_indexed_append() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class A {} class B {}
foreach ([new A(), new B()] as $value) {}
$value = [];
$value[] = 42;
echo count($value);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

fn object_write(throwing: bool) {
    let source = r#"<?php
class A {} class B {}
class Writer implements ArrayAccess {
    public array $values = [];
    public function offsetSet(mixed $key, mixed $value): void {
        $this->values[$key] = $value;
        if (THROWING) { throw new Exception('stop'); }
    }
    public function offsetGet(mixed $key): mixed { return $this->values[$key]; }
    public function offsetExists(mixed $key): bool { return true; }
    public function offsetUnset(mixed $key): void {}
}
foreach ([new A(), new B()] as $value) {}
$value = new Writer();
try { $value['a'] = 42; } catch (Exception $error) { echo 'caught|'; }
echo $value['a'];
"#.replace("THROWING", if throwing { "true" } else { "false" });
    let out = compile_and_run_with_heap_debug(&source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, if throwing { "caught|42" } else { "42" }, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_container_write_local_owners_object_offset() { object_write(false); }

#[test]
fn test_container_write_local_owners_object_throw() { object_write(true); }

#[test]
fn test_container_write_local_owners_argument_throw() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class A {} class B {}
class Writer implements ArrayAccess {
    public array $values = [];
    public function offsetSet(mixed $key, mixed $value): void {
        $this->values[$key] = $value;
        if (false) { throw new Exception('stop'); }
    }
    public function offsetGet(mixed $key): mixed { return $this->values[$key]; }
    public function offsetExists(mixed $key): bool { return true; }
    public function offsetUnset(mixed $key): void {}
}
foreach ([new A(), new B()] as $value) {}
$value = new Writer();
function fail(): int { throw new Exception('stop'); }
try { $value[fail()] = 42; } catch (Exception $error) { echo 'caught|'; }
echo count($value->values);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "caught|0", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_container_write_local_owners_fresh_rhs() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class A {} class B {}
class Writer implements ArrayAccess {
    public array $values = [];
    public function offsetSet(mixed $key, mixed $value): void {
        $this->values[$key] = $value;
        if (false) { throw new Exception('stop'); }
    }
    public function offsetGet(mixed $key): mixed { return $this->values[$key]; }
    public function offsetExists(mixed $key): bool { return true; }
    public function offsetUnset(mixed $key): void {}
}
foreach ([new A(), new B()] as $value) {}
$value = new Writer();
class Payload { public function __destruct() { echo 'drop|'; } }
$value['a'] = ['child' => new Payload()];
echo 'held|';
unset($value); echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "held|drop|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_container_write_local_owners_key_before_throwing_value() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class A {} class B {}
class Writer implements ArrayAccess {
    public array $values = [];
    public function offsetSet(mixed $key, mixed $value): void {
        $this->values[$key] = $value;
        if (false) { throw new Exception('stop'); }
    }
    public function offsetGet(mixed $key): mixed { return $this->values[$key]; }
    public function offsetExists(mixed $key): bool { return true; }
    public function offsetUnset(mixed $key): void {}
}
foreach ([new A(), new B()] as $value) {}
$value = new Writer();
function keyValue(): string { return str_repeat('k', 64); }
function fail(): int { throw new Exception('stop'); }
try { $value[keyValue()] = fail(); } catch (Exception $error) { echo 'caught|'; }
echo count($value->values);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "caught|0", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_container_write_local_owners_php_key_evaluation_order() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Writer implements ArrayAccess {
    public function offsetSet(mixed $key, mixed $value): void { echo $key, ':', $value, '|'; }
    public function offsetGet(mixed $key): mixed { return null; }
    public function offsetExists(mixed $key): bool { return false; }
    public function offsetUnset(mixed $key): void {}
}
function keyValue(): string { echo 'key|'; return 'a'; }
function rhsValue(): string { echo 'rhs|'; return 'b'; }
$writer = new Writer();
$key = 'a';
$writer[$key] = ($key = 'b');
$writer[keyValue()] = rhsValue();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "b:b|key|rhs|a:b|", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_container_write_local_owners_native_spl_setters() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Payload { public function __destruct() { echo 'drop|'; } }
$fixed = new SplFixedArray(1);
$fixed[0] = new Payload();
$list = new SplDoublyLinkedList();
$list->push(0);
$list[0] = new Payload();
$stack = new SplStack();
$stack->push(0);
$stack[0] = new Payload();
$queue = new SplQueue();
$queue->push(0);
$queue[0] = new Payload();
unset($fixed, $list, $stack, $queue);
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "drop|drop|drop|drop|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
