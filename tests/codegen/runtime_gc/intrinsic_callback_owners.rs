//! Purpose:
//! Covers borrowed arguments crossing runtime-backed SPL method wrappers.
//!
//! Called from:
//! - The runtime GC codegen integration suite through its explicit parent registration.
//!
//! Key details:
//! - Dynamic methods and array callbacks retain each native-consumed boxed argument.
//! - Errors must preserve their full message as well as balanced heap ownership.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_intrinsic_callback_owners_fixed_type_error() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key):void {try{$bag->offsetGet($key);}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
probe(new SplFixedArray(1),new stdClass());
"#, "TypeError:Cannot access offset of type stdClass on SplFixedArray");
}

#[test]
fn test_intrinsic_callback_owners_list_type_error() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key):void {try{$bag->offsetGet($key);}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
probe(new SplDoublyLinkedList(),['bad']);
"#, "TypeError:SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, array given");
}

#[test]
fn test_intrinsic_callback_owners_fixed_range_error() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key):void {try{$bag->offsetGet($key);}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
probe(new SplFixedArray(1),5);
"#, "OutOfBoundsException:Index invalid or out of range");
}

#[test]
fn test_intrinsic_callback_owners_fixed_success() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key,mixed $value):void {$bag->offsetSet($key,$value);$copy=$bag->offsetGet($key);echo $copy[0],':',$key,':',$value[0];}
probe(new SplFixedArray(1),0,['alive']);
"#, "alive:0:alive");
}

#[test]
fn test_intrinsic_callback_owners_list_success() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $value):void {$bag->push($value);$copy=$bag->offsetGet(0);echo $copy[0],':',$value[0];$bag->offsetSet(0,['replacement']);echo ':',$bag->offsetGet(0)[0];}
probe(new SplDoublyLinkedList(),['alive']);
"#, "alive:alive:replacement");
}

#[test]
fn test_intrinsic_callback_owners_fixed_write_type_error() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key,mixed $value):void {try{$bag->offsetSet($key,$value);}catch(Throwable $e){echo $e->getMessage(),':',$value[0];}}
probe(new SplFixedArray(1),new stdClass(),['alive']);
"#, "Cannot access offset of type stdClass on SplFixedArray:alive");
}

#[test]
fn test_intrinsic_callback_owners_list_write_type_error() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key,mixed $value):void {try{$bag->offsetSet($key,$value);}catch(Throwable $e){echo $e->getMessage(),':',$value[0];}}
probe(new SplDoublyLinkedList(),new stdClass(),['alive']);
"#, "SplDoublyLinkedList::offsetSet(): Argument #1 ($index) must be of type ?int, stdClass given:alive");
}

#[test]
fn test_intrinsic_callback_owners_fixed_callback_range() {
    assert_clean(r#"<?php
function probe(mixed $bag):void {try{$bag[5];}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
probe(new SplFixedArray(1));
"#, "OutOfBoundsException:Index invalid or out of range");
}

#[test]
fn test_intrinsic_callback_owners_fixed_exists_unset() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $key):void {$bag->offsetSet($key,['kept']);echo $bag->offsetExists($key)?'yes':'no';$bag->offsetUnset($key);echo ':',$bag->offsetExists($key)?'yes':'no',':',$key;}
probe(new SplFixedArray(1),'0');
"#, "yes:no:0");
}

#[test]
fn test_intrinsic_callback_owners_list_add() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $value):void {$bag->add(0,$value);$bag->unshift(['first']);echo $bag->offsetGet(0)[0],':',$bag->offsetGet(1)[0],':',$value[0];}
probe(new SplDoublyLinkedList(),['kept']);
"#, "first:kept:kept");
}

#[test]
fn test_intrinsic_callback_owners_queue_enqueue() {
    assert_clean(r#"<?php
function probe(mixed $bag,mixed $value):void {$bag->enqueue($value);echo $bag->dequeue()[0],':',$value[0];}
probe(new SplQueue(),['kept']);
"#, "kept:kept");
}

#[test]
fn test_intrinsic_callback_owners_fixed_local_key_type_error() {
    assert_clean(r#"<?php
function probe(mixed $bag):void {$key=new stdClass();try{$bag->offsetGet($key);}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
probe(new SplFixedArray(1));
"#, "TypeError:Cannot access offset of type stdClass on SplFixedArray");
}
