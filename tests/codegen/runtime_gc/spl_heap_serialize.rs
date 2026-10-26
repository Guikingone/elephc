//! Purpose:
//! Pins PHP 8.5.10 SPL heap serialization format and callback-time rejection.
//!
//! Called from:
//! - Runtime GC executable codegen regressions.
//!
//! Key details:
//! - Run both optimizer modes and verify complete output plus clean native ownership.
//! - Covers inherited user members, max/min/priority physical order and corrupted/locked heaps.

use crate::support::{compile_and_run_with_heap_debug, without_ir_opt};

/// Compares a full frozen oracle and checks ownership under both optimizer modes.
fn assert_serializer(source: &str, expected: &str) {
    let run = || {
        let output = compile_and_run_with_heap_debug(source);
        assert!(output.success, "{}", output.stderr);
        assert_eq!(output.stdout, expected, "{}", output.stderr);
        assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
    };
    run();
    without_ir_opt(run);
}

/// Replays the pinned PHP serializer oracle for formats.
#[test]
fn test_spl_heap_serialize_formats() {
    assert_serializer(r##"<?php $max=new SplMaxHeap();echo serialize($max),'|';foreach([100,50,200]as$v){$max->insert($v);}echo serialize($max),'|';$min=new SplMinHeap();foreach([10,30,20]as$v){$min->insert($v);}echo serialize($min),'|';$q=new SplPriorityQueue();$q->insert('low',1);$q->insert('high',10);$q->setExtractFlags(3);echo serialize($q);"##, "O:10:\"SplMaxHeap\":2:{i:0;a:0:{}i:1;a:2:{s:5:\"flags\";i:0;s:13:\"heap_elements\";a:0:{}}}|O:10:\"SplMaxHeap\":2:{i:0;a:0:{}i:1;a:2:{s:5:\"flags\";i:0;s:13:\"heap_elements\";a:3:{i:0;i:200;i:1;i:50;i:2;i:100;}}}|O:10:\"SplMinHeap\":2:{i:0;a:0:{}i:1;a:2:{s:5:\"flags\";i:0;s:13:\"heap_elements\";a:3:{i:0;i:10;i:1;i:30;i:2;i:20;}}}|O:16:\"SplPriorityQueue\":2:{i:0;a:0:{}i:1;a:2:{s:5:\"flags\";i:3;s:13:\"heap_elements\";a:2:{i:0;a:2:{s:4:\"data\";s:4:\"high\";s:8:\"priority\";i:10;}i:1;a:2:{s:4:\"data\";s:3:\"low\";s:8:\"priority\";i:1;}}}}");
}

/// Replays the pinned PHP serializer oracle for properties.
#[test]
fn test_spl_heap_serialize_properties() {
    assert_serializer(r##"<?php class H extends SplMaxHeap{public string $flags='user';protected string $label='l';private string $secret='s';}$h=new H();$h->insert(7);echo serialize($h),'|',serialize($h->__serialize());"##, "O:1:\"H\":2:{i:0;a:3:{s:5:\"flags\";s:4:\"user\";s:8:\"\0*\0label\";s:1:\"l\";s:9:\"\0H\0secret\";s:1:\"s\";}i:1;a:2:{s:5:\"flags\";i:0;s:13:\"heap_elements\";a:1:{i:0;i:7;}}}|a:2:{i:0;a:3:{s:5:\"flags\";s:4:\"user\";s:8:\"\0*\0label\";s:1:\"l\";s:9:\"\0H\0secret\";s:1:\"s\";}i:1;a:2:{s:5:\"flags\";i:0;s:13:\"heap_elements\";a:1:{i:0;i:7;}}}");
}

/// Replays the pinned PHP serializer oracle for write lock.
#[test]
fn test_spl_heap_serialize_write_lock() {
    assert_serializer(r##"<?php class H extends SplMaxHeap{public function compare(mixed $a,mixed $b):int{try{$this->__serialize();}catch(RuntimeException $e){echo $e->getMessage(),'|';}return parent::compare($a,$b);}}$h=new H();$h->insert(1);$h->insert(2);echo count($h);"##, "Cannot serialize heap while it is being modified.|2");
}

/// Replays the pinned PHP serializer oracle for corruption.
#[test]
fn test_spl_heap_serialize_corruption() {
    assert_serializer(r##"<?php class H extends SplMaxHeap{public function compare(mixed $a,mixed $b):int{throw new Exception('compare');}}$h=new H();$h->insert(1);try{$h->insert(2);}catch(Exception $e){}try{serialize($h);}catch(RuntimeException $e){echo $e->getMessage(),'|';}echo $h->isCorrupted()?'corrupt':'clean';"##, "Heap is corrupted, heap properties are no longer ensured.|corrupt");
}

/// Exposes the inherited native hook with zero public parameters on all four heap owners.
#[test]
fn test_spl_heap_serialize_reflection_arity_visibility() {
    assert_serializer(r##"<?php function show(ReflectionMethod $r):void{echo $r->getNumberOfParameters(),':',$r->isPublic()?'public':'bad','|';}show(new ReflectionMethod('SplHeap','__serialize'));show(new ReflectionMethod('SplMaxHeap','__serialize'));show(new ReflectionMethod('SplMinHeap','__serialize'));show(new ReflectionMethod('SplPriorityQueue','__serialize'));"##, "0:public|0:public|0:public|0:public|");
}
