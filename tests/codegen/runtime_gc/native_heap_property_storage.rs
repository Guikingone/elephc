//! Purpose:
//! Freezes ordinary PHP property behavior while native SPL user slots gain zval storage.
//!
//! Called from:
//! - Runtime GC codegen integration tests on executable supported targets.
//!
//! Key details:
//! - Both optimizer modes preserve weak coercions, literal false, inheritance and reference aliases.
//! - Complete output is pinned to PHP 8.5.10 and every execution must release all native owners.

use crate::support::{compile_and_run_with_heap_debug, without_ir_opt};

/// Preserves observable property behavior and ownership through native storage refinement.
fn assert_native_property(source: &str, expected: &str) {
    let run = || {
        let output = compile_and_run_with_heap_debug(source);
        assert!(output.success, "{}", output.stderr);
        assert_eq!(output.stdout, expected, "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    };
    run();
    without_ir_opt(run);
}

/// Replays the pinned PHP ordinary-property control for scalar coercions.
#[test]
fn test_native_heap_property_storage_scalar_coercions() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public int $i=1;public float $f=1.0;public string $s='a';public bool $b=false;}function put(H $h,string $key,mixed $value):void{$h->$key=$value;}$h=new H();put($h,'i','7');put($h,'f',2);put($h,'s',3);put($h,'b',0);foreach(get_mangled_object_vars($h)as$k=>$v){echo $k,':',gettype($v),':',json_encode($v),'|';}"##, "i:integer:7|f:double:2|s:string:\"3\"|b:boolean:false|");
}

/// Replays the pinned PHP ordinary-property control for rejected values.
#[test]
fn test_native_heap_property_storage_rejected_values() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public int $i=1;public false $f=false;}function put(H $h,string $key,mixed $value):void{$h->$key=$value;}$h=new H();try{put($h,'i','bad');}catch(TypeError $e){echo $e->getMessage(),'|';}try{put($h,'f',true);}catch(TypeError $e){echo $e->getMessage(),'|';}try{put($h,'f',0);}catch(TypeError $e){echo $e->getMessage(),'|';}echo $h->i,':',$h->f?'true':'false';"##, "Cannot assign string to property H::$i of type int|Cannot assign true to property H::$f of type false|Cannot assign int to property H::$f of type false|1:false");
}

/// Replays the pinned PHP ordinary-property control for array and object.
#[test]
fn test_native_heap_property_storage_array_and_object() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public array $a=[1];public ?stdClass $o=null;public ?string $s=null;}function put(H $h,string $key,mixed $value):void{$h->$key=$value;}$h=new H();$copy=$h->a;$h->a[]=2;put($h,'o',new stdClass());put($h,'s','ok');echo json_encode($copy),'|',json_encode($h->a),'|',get_class($h->o),'|',$h->s;"##, "[1]|[1,2]|stdClass|ok");
}

/// Replays the pinned PHP ordinary-property control for private shadow.
#[test]
fn test_native_heap_property_storage_private_shadow() {
    assert_native_property(r##"<?php class P extends SplMaxHeap{private int $value=1;function parentPut(mixed $v):void{$this->value=$v;}function parentValue():mixed{return $this->value;}}class H extends P{private string $value='x';function childPut(mixed $v):void{$this->value=$v;}function childValue():mixed{return $this->value;}}$h=new H();$h->parentPut('7');$h->childPut(8);echo gettype($h->parentValue()),':',$h->parentValue(),'|',gettype($h->childValue()),':',$h->childValue();"##, "integer:7|string:8");
}

/// Replays the pinned PHP ordinary-property control for promoted reference.
#[test]
fn test_native_heap_property_storage_promoted_reference() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{function __construct(public int &$value){}}$x=1;$h=new H($x);$x=2;echo $h->value,'|';$h->value=3;echo $x;"##, "2|3");
}

/// Replays the pinned PHP ordinary-property control for stringable.
#[test]
fn test_native_heap_property_storage_stringable() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public string $value='x';}class S{function __toString():string{global $calls;$calls++;return 'converted';}}function put(H $h,string $key,mixed $value):void{$h->$key=$value;}$calls=0;$h=new H();put($h,'value',new S());echo $calls,':',$h->value;"##, "1:converted");
}

/// Preserves the pinned PHP container behavior for array key writes.
#[test]
fn test_native_heap_property_storage_array_key_writes() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public array $a=[1];}$h=new H();$copy=$h->a;$h->a['x']=2;$h->a[0]=3;echo json_encode($copy),'|',json_encode($h->a);"##, "[1]|{\"0\":3,\"x\":2}");
}

/// Preserves the pinned PHP container behavior for nullable array.
#[test]
fn test_native_heap_property_storage_nullable_array() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public ?array $a=null;}$h=new H();$h->a[]=1;$h->a['x']=2;echo json_encode($h->a);"##, "{\"0\":1,\"x\":2}");
}

/// Preserves successful exact-tag writes, including a literal false payload.
#[test]
fn test_native_heap_property_storage_matching_values() {
    assert_native_property(r##"<?php class H extends SplMaxHeap{public int $i=0;public float $d=0.0;public string $s='';public bool $b=false;public false $f=false;}function put(H $h,string $key,mixed $value):void{$h->$key=$value;}$h=new H();put($h,'i',7);put($h,'d',1.5);put($h,'s','v');put($h,'b',true);put($h,'f',false);foreach(get_mangled_object_vars($h)as$k=>$v){echo $k,':',gettype($v),':',json_encode($v),'|';}"##, "i:integer:7|d:double:1.5|s:string:\"v\"|b:boolean:true|f:boolean:false|");
}
