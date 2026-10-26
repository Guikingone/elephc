//! Purpose:
//! Verifies that boxed ArrayAccess receivers retain the original PHP offset values.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Object dispatch precedes array-key coercion and uses the shared method ABI.
//! - Ordinary packed/hash receivers keep PHP key conversion and cleanup behavior.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out=compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_array_access_mixed_keys_read_preserves_object_identity() {
    assert_clean(r#"<?php
class A implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{echo gettype($key),':';return $key;}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function read(mixed $bag,mixed $key):mixed{return $bag[$key];}
$key=new stdClass();$value=read(new A(),$key);echo $value===$key?'same':'different';
"#, "object:same");
}

#[test]
fn test_array_access_mixed_keys_write_preserves_offset_and_value() {
    assert_clean(r#"<?php
class A implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return null;}
 function offsetSet(mixed $key,mixed $value):void{echo gettype($key),':',gettype($value);}
 function offsetUnset(mixed $key):void{}
}
function write(mixed $bag,mixed $key,mixed $value):void{$bag[$key]=$value;}
write(new A(),new stdClass(),['x']);
"#, "object:array");
}

#[test]
fn test_array_access_mixed_keys_array_fallback_still_converts_keys() {
    assert_clean(r#"<?php
error_reporting(0);
function read(mixed $bag,mixed $key):mixed{return $bag[$key];}
echo read(['a'=>3,1=>4],'a'),':',read(['a'=>3,1=>4],true);
"#, "3:4");
}


#[test]
fn test_array_access_mixed_keys_read_keeps_all_php_offset_types() {
    assert_clean(r#"<?php
class A implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{echo gettype($key),';';return null;}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function read(mixed $bag,mixed $key):void{$bag[$key];}
foreach([true,1.5,null,['x'],new stdClass(),'12','02'] as $key){read(new A(),$key);}
"#, "boolean;double;NULL;array;object;string;string;");
}


#[test]
fn test_array_access_mixed_keys_literal_offsets_release_conversion_cells() {
    assert_clean(r#"<?php
class A implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{return $key;}
 function offsetSet(mixed $key,mixed $value):void{echo $key,':',$value,';';}
 function offsetUnset(mixed $key):void{}
}
function useBag(mixed $bag):void{$bag['write']='value';echo $bag['read'];}
useBag(new A());
"#, "write:value;read");
}


#[test]
fn test_array_access_mixed_keys_nested_fetch_preserves_offset() {
    assert_clean(r#"<?php
error_reporting(0);
class A implements ArrayAccess {
 function offsetExists(mixed $key):bool{return true;}
 function offsetGet(mixed $key):mixed{echo gettype($key);return ['x'=>0];}
 function offsetSet(mixed $key,mixed $value):void{}
 function offsetUnset(mixed $key):void{}
}
function change(mixed $bag,mixed $key):void{$bag[$key]['x']=7;}
change(new A(),new stdClass());
"#, "object");
}
