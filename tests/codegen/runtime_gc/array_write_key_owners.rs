//! Purpose:
//! Exercises temporary array/hash key ownership on normal writes and RHS exceptions.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Mixed index cells are borrowed by storage mutation and must retire after their last use.
//! - An evaluated key remains protected while the value expression can throw.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_array_write_key_owners_hash_mixed_integer() {
    assert_clean("<?php function keyValue(): mixed {return 1;} $a=['seed'=>0];$a[keyValue()]=42;echo $a[1];", "42");
}

#[test]
fn test_array_write_key_owners_hash_mixed_string() {
    assert_clean("<?php function keyValue(): mixed {return 'x';} $a=['seed'=>0];$a[keyValue()]=42;echo $a['x'];", "42");
}

#[test]
fn test_array_write_key_owners_indexed_mixed_integer() {
    assert_clean("<?php function keyValue(): mixed {return 0;} $a=[];$a[keyValue()]=42;echo $a[0];", "42");
}

#[test]
fn test_array_write_key_owners_key_survives_throwing_rhs() {
    assert_clean("<?php function keyValue(): mixed {return 'x';} function fail(): int {throw new Exception('stop');} $a=['seed'=>0];try{$a[keyValue()]=fail();}catch(Exception $e){echo 'caught';}", "caught");
}

#[test]
fn test_array_write_key_owners_repeated_keys_retire_each_iteration() {
    assert_clean("<?php function keyValue(int $i): mixed {return 'k'.$i;} $a=['seed'=>0];for($i=0;$i<20;$i++){$a[keyValue($i)]=$i;}echo count($a);", "21");
}

#[test]
fn test_array_write_key_owners_borrowed_local_remains_owned() {
    assert_clean("<?php function keyValue(): mixed {return 'x';} $a=['seed'=>0];$key=keyValue();$a[$key]=42;echo $a['x'],':',$key;", "42:x");
}

#[test]
fn test_array_write_key_owners_string_snapshot_survives_rhs_scratch() {
    assert_clean("<?php $a=['seed'=>0];$a['key'.$argc]=str_repeat('x',65535).$argc;echo strlen($a['key1']);", "65536");
}

#[test]
fn test_array_write_key_owners_property_write_and_throw() {
    assert_clean("<?php class Holder {public array $data=['seed'=>0];} function keyValue():mixed{return 'x';} function fail():int{throw new Exception('stop');} $h=new Holder();$h->data[keyValue()]=42;try{$h->data[keyValue()]=fail();}catch(Exception $e){echo 'caught|';}echo $h->data['x'];", "caught|42");
}

#[test]
fn test_array_write_key_owners_static_property_write_and_throw() {
    assert_clean("<?php class Holder {public static array $data=['seed'=>0];} function keyValue():mixed{return 'x';} function fail():int{throw new Exception('stop');} Holder::$data[keyValue()]=42;try{Holder::$data[keyValue()]=fail();}catch(Exception $e){echo 'caught|';}echo Holder::$data['x'];", "caught|42");
}

#[test]
fn test_array_write_key_owners_nested_write_preserves_cow_and_unwind() {
    assert_clean("<?php function keyValue():mixed{return 'x';} function fail():int{throw new Exception('stop');} function makeData():mixed{return ['child'=>['seed'=>0]];} $a=makeData();$copy=$a;$a['child'][keyValue()]=42;try{$a['child'][keyValue()]=fail();}catch(Exception $e){echo 'caught|';}echo $a['child']['x'],':',count($copy['child']);", "caught|42:1");
}

#[test]
fn test_array_write_key_owners_boxed_target_retains_argument_protocol() {
    assert_clean("<?php function keyValue():mixed{return 'x';} function writeValue(&$a):void{$a[keyValue()]=42;} $a=['seed'=>0];writeValue($a);echo $a['x'];", "42");
}

#[test]
fn test_array_write_key_owners_computed_key_is_a_value_snapshot() {
    assert_clean("<?php function readKey(&$key):mixed{return $key;} $key='old';$a=['seed'=>0];$a[readKey($key)]=($key='new');echo $a['old'],':',$key;", "new:new");
}

#[test]
fn test_array_write_key_owners_array_access_argument() {
    assert_clean(r#"<?php
class Holder implements ArrayAccess {
 public array $data = [];
 public function offsetSet(mixed $key, mixed $value):void{$this->data[$key]=$value;}
 public function offsetGet(mixed $key):mixed{return $this->data[$key];}
 public function offsetExists(mixed $key):bool{return true;}
 public function offsetUnset(mixed $key):void{}
}
function keyValue():mixed{return 'x';}
$h=new Holder();$h[keyValue()]=42;echo $h['x'];
"#, "42");
}

#[test]
fn test_array_write_key_owners_string_local_stays_valid() {
    assert_clean("<?php $key=str_repeat('x',$argc);$a=['seed'=>0];$a[$key]=42;echo $key,':',$a[$key];", "x:42");
}

#[test]
fn test_array_write_key_owners_repeated_throw_clears_scoped_owner() {
    assert_clean("<?php function keyValue():mixed{return 'x';} function fail():int{throw new Exception('stop');} $a=['seed'=>0];for($i=0;$i<20;$i++){try{$a[keyValue()]=fail();}catch(Exception $e){}}echo count($a);", "1");
}

#[test]
fn test_array_write_key_owners_object_key_for_array_access() {
    assert_clean(r#"<?php
class Key {public function __destruct(){echo 'key|';}}
class Holder implements ArrayAccess {
 public function offsetSet(mixed $key,mixed $value):void{echo get_class($key),':',$value,'|';}
 public function offsetGet(mixed $key):mixed{return null;}
 public function offsetExists(mixed $key):bool{return false;}
 public function offsetUnset(mixed $key):void{}
}
$h=new Holder();$h[new Key()]=42;echo 'done';
"#, "Key:42|key|done");
}
