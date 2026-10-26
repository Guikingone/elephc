//! Purpose:
//! Verifies call argument ownership when native scalar/void methods lack alias summaries.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - A builtin SPL method must not pollute an unrelated void method's alias result.
//! - Normal completion, exceptions and heap-return aliases retain their own owners.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out=compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_nonheap_method_argument_owners_void_call_with_spl_metadata() {
    assert_clean(r#"<?php
class A {function __unserialize(array $data):void {echo count($data);}}
function useBag(mixed $bag):void {$bag->__unserialize([[],['x'=>[3,4]]]);}
new SplObjectStorage();useBag(new A());
"#, "2");
}

#[test]
fn test_nonheap_method_argument_owners_throw_with_spl_metadata() {
    assert_clean(r#"<?php
class A {function __unserialize(array $data):void {throw new RuntimeException('boom');}}
function useBag(mixed $bag):void {$bag->__unserialize([[],['x'=>[3,4]]]);}
new SplObjectStorage();
try {useBag(new A());} catch (RuntimeException $e) {echo $e->getMessage();}
"#, "boom");
}

#[test]
fn test_nonheap_method_argument_owners_heap_result_keeps_its_payload() {
    assert_clean(r#"<?php
class Keeper {function value(array $data):array {return $data;}}
function read(mixed $bag):mixed {return $bag->value([[5]]);}
$result=read(new Keeper());echo $result[0][0];
"#, "5");
}


#[test]
fn test_nonheap_method_argument_owners_native_list_keeps_nested_payload() {
    assert_clean(r#"<?php
function fill(mixed $list):void {$list->push([['x']]);}
$list=new SplDoublyLinkedList();fill($list);
$value=$list->offsetGet(0);echo $value[0][0];
"#, "x");
}


#[test]
fn test_nonheap_method_argument_owners_list_mutators() {
    assert_clean(r#"<?php
function fill(mixed $l):void{$l->push([['x']]);$l->unshift([['y']]);$l->add(1,[['z']]);$l->offsetSet(2,[['w']]);}$l=new SplDoublyLinkedList();fill($l);for($i=0;$i<count($l);$i++){$v=$l->offsetGet($i);echo $v[0][0];}
"#, "yzw");
}


#[test]
fn test_nonheap_method_argument_owners_fixed_mutator() {
    assert_clean(r#"<?php
function fill(mixed $a):void{$a->offsetSet(0,[['x']]);}$a=new SplFixedArray(1);fill($a);$v=$a->offsetGet(0);echo $v[0][0];
"#, "x");
}


#[test]
fn test_nonheap_method_argument_owners_list_invalid() {
    assert_clean(r#"<?php
function fail(mixed $a):void{$a->offsetSet(-1,[['x']]);}try{fail(new SplDoublyLinkedList());}catch(Throwable $e){echo get_class($e);}
"#, "OutOfRangeException");
}


#[test]
fn test_nonheap_method_argument_owners_fixed_invalid() {
    assert_clean(r#"<?php
function fail(mixed $a):void{$a->offsetSet(-1,[['x']]);}try{fail(new SplFixedArray(1));}catch(Throwable $e){echo get_class($e);}
"#, "OutOfBoundsException");
}
