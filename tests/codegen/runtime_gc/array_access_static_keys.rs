//! Purpose:
//! Covers concrete PHP offsets on boxed ArrayAccess receivers and real array fallbacks.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - Legal object/array callback offsets retain their original PHP type and identity.
//! - Real arrays reject illegal keys after RHS evaluation without leaking owners.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_array_access_static_keys_object_write_keeps_payload() {
    assert_clean(r#"<?php
class Key {public string $name='kept';}
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return null;}
 function offsetSet(mixed $key,mixed $value):void{echo $key->name,':',$value;}
 function offsetUnset(mixed $key):void{}
}
function write(mixed $bag):void{$key=new Key();$bag[$key]=7;echo ':',$key->name;}
write(new Bag());
"#, "kept:7:kept");
}

#[test]
fn test_array_access_static_keys_object_read_keeps_identity() {
    assert_clean(r#"<?php
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return $key;}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function read(mixed $bag):void{$key=new stdClass();$value=$bag[$key];echo $key===$value?'same':'different';}
read(new Bag());
"#, "same");
}

#[test]
fn test_array_access_static_keys_array_offset_reaches_callback() {
    assert_clean(r#"<?php
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return $key;}
 function offsetSet(mixed $key,mixed $value):void{echo gettype($key),':',$key[0],':',$value,';';}
 function offsetUnset(mixed $key):void{}
}
function useBag(mixed $bag):void{$key=['offset'];$bag[$key]='value';$copy=$bag[$key];echo $copy[0];}
useBag(new Bag());
"#, "array:offset:value;offset");
}

#[test]
fn test_array_access_static_keys_array_fallback_rejects_object_after_rhs() {
    assert_clean(r#"<?php
class Key {}
function rhs():array{echo 'rhs|';return ['kept'];}
function write(mixed $bag):void {
 $key=new Key();
 try{$bag[$key]=rhs();}catch(Throwable $e){echo $e::class,':',$e->getMessage(),'|';}
 echo 'done';
}
write([]);
"#, "rhs|TypeError:Cannot access offset of type Key on array|done");
}

#[test]
fn test_array_access_static_keys_boxed_illegal_types_do_not_become_zero() {
    assert_clean(r#"<?php
function read(mixed $bag,mixed $key):void {
 try{echo $bag[$key];}catch(Throwable $e){echo $e->getMessage(),';';}
}
read([0=>'wrong'],new stdClass());read(['x'=>'wrong'],['bad']);
"#, "Cannot access offset of type stdClass on array;Cannot access offset of type array on array;");
}

#[test]
fn test_array_access_static_keys_bad_receiver_precedes_bad_key() {
    assert_clean(r#"<?php
function read(mixed $bag,mixed $key):void {
 try{$bag[$key];}catch(Throwable $e){echo $e::class,':',$e->getMessage();}
}
read(new stdClass(),new stdClass());
"#, "Error:Cannot use object of type stdClass as array");
}

#[test]
fn test_array_access_static_keys_native_fixed_array_validates_original_key() {
    assert_clean(r#"<?php
function read(mixed $bag):void {
 $key=new stdClass();
 try{$bag[$key];}catch(Throwable $e){echo $e->getMessage();}
}
read(new SplFixedArray(1));
"#, "Cannot access offset of type stdClass on SplFixedArray");
}

#[test]
fn test_array_access_static_keys_nested_fetch_keeps_object_offset() {
    assert_clean(r#"<?php
error_reporting(0);
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{echo gettype($key);return ['nested'=>0];}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function change(mixed $bag):void{$key=new stdClass();$bag[$key]['nested']=7;}
change(new Bag());
"#, "object");
}

#[test]
fn test_array_access_static_keys_nested_array_rejects_object_offset() {
    assert_clean(r#"<?php
function change(mixed $bag):void {
 $key=new stdClass();
 try{$bag[$key]['nested']=7;}catch(Throwable $e){echo $e->getMessage();}
}
change([]);
"#, "Cannot access offset of type stdClass on array");
}

#[test]
fn test_array_access_static_keys_null_write_autovivifies_before_key_error() {
    assert_clean(r#"<?php
function change(mixed $bag):void {
 $key=new stdClass();
 try{$bag[$key]=7;}catch(Throwable $e){echo $e->getMessage(),':';}
 echo gettype($bag),':',count($bag);
}
change(null);
"#, "Cannot access offset of type stdClass on array:array:0");
}

#[test]
fn test_array_access_static_keys_closure_offset_keeps_identity() {
    assert_clean(r#"<?php
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return $key;}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function read(mixed $bag):void{$key=static fn()=>7;$copy=$bag[$key];echo $key===$copy?'same':'different';}
read(new Bag());
"#, "same");
}

#[test]
fn test_array_access_static_keys_real_array_rejects_closure_offset() {
    assert_clean(r#"<?php
function read(mixed $bag):void {
 $key=static fn()=>7;
 try{$bag[$key];}catch(Throwable $e){echo $e->getMessage();}
}
read([]);
"#, "Cannot access offset of type Closure on array");
}

#[test]
fn test_array_access_static_keys_associative_offset_keeps_entries() {
    assert_clean(r#"<?php
class Bag implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return $key;}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function read(mixed $bag):void{$key=['name'=>'kept'];$copy=$bag[$key];echo $copy['name'];}
read(new Bag());
"#, "kept");
}

#[test]
fn test_array_access_static_keys_rhs_normal_destructor() {
    assert_clean(r#"<?php
class Value {function __destruct(){echo 'drop|';}}
function rhs():Value{echo 'rhs|';return new Value();}
function write(mixed $bag):void {try{$bag[new stdClass()]=rhs();}catch(Throwable $e){echo $e::class,':',$e->getMessage();}}
write([]);
"#, "rhs|drop|TypeError:Cannot access offset of type stdClass on array");
}

#[test]
fn test_array_access_static_keys_rhs_throwing_destructor() {
    assert_clean(r#"<?php
class Value {function __destruct(){echo 'drop|';throw new Exception('destructor');}}
function rhs():Value{echo 'rhs|';return new Value();}
function write(mixed $bag):void {try{$bag[new stdClass()]=rhs();}catch(Throwable $e){echo $e::class,':',$e->getMessage(),'|';$previous=$e->getPrevious();echo $previous===null?'none':$previous->getMessage();}}
write([]);
"#, "rhs|drop|Exception:destructor|Cannot access offset of type stdClass on array");
}
