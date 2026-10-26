//! Purpose:
//! Compares SPL array-container debug views with the pinned PHP 8.5.10 behavior.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - Internal writes, reverse writes, growth, promotion, sorting, copying and cloning differ.
//! - Each full output assertion also checks that all native owners were retired.

use crate::support::compile_and_run_with_heap_debug;

/// Checks the PHP output together with normal termination and complete native cleanup.
fn check(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_set() {
    check(r#"<?php $a=new ArrayObject(['x'=>1]);$d=$a->__debugInfo();$a['x']=2;echo json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_append() {
    check(r#"<?php $a=new ArrayObject([1]);$d=$a->__debugInfo();$a->append(2);echo json_encode($d["\0ArrayObject\0storage"]);
"#, "[1,2]");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_append_grow() {
    check(r#"<?php $a=new ArrayObject([]);$d=$a->__debugInfo();for($i=0;$i<40;$i++){$a[]=$i;}echo count($d["\0ArrayObject\0storage"]);
"#, "40");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_unset() {
    check(r#"<?php $a=new ArrayObject(['x'=>1,'y'=>2]);$d=$a->__debugInfo();unset($a['x']);echo json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"y\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_sort() {
    check(r#"<?php $a=new ArrayObject(['a'=>2,'b'=>1]);$d=$a->__debugInfo();$a->asort();echo json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"a\":2,\"b\":1}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_promote() {
    check(r#"<?php $a=new ArrayObject([1]);$d=$a->__debugInfo();$a['x']=2;echo json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"0\":1,\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_reverse_write() {
    check(r#"<?php $a=new ArrayObject(['x'=>1]);$d=$a->__debugInfo();$d["\0ArrayObject\0storage"]['x']=2;echo json_encode($a->getArrayCopy()).'|'.json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"x\":1}|{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_clone() {
    check(r#"<?php $a=new ArrayObject(['x'=>1]);$d=$a->__debugInfo();$b=clone $a;$b['x']=2;echo json_encode($a->getArrayCopy()).'|'.json_encode($b->getArrayCopy()).'|'.json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"x\":1}|{\"x\":2}|{\"x\":1}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_copy() {
    check(r#"<?php $a=new ArrayObject(['x'=>1]);$d=$a->__debugInfo();$copy=$a->getArrayCopy();$a['x']=2;echo json_encode($copy).'|'.json_encode($d["\0ArrayObject\0storage"]);
"#, "{\"x\":1}|{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayobject_detached_view() {
    check(r#"<?php $a=new ArrayObject(['x'=>1]);$d=$a->__debugInfo();$d["\0ArrayObject\0storage"]['x']=2;$a['x']=3;echo json_encode($d["\0ArrayObject\0storage"]).'|'.json_encode($a->getArrayCopy());
"#, "{\"x\":2}|{\"x\":3}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_set() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1]);$d=$a->__debugInfo();$a['x']=2;echo json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_append() {
    check(r#"<?php $a=new ArrayIterator([1]);$d=$a->__debugInfo();$a->append(2);echo json_encode($d["\0ArrayIterator\0storage"]);
"#, "[1,2]");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_append_grow() {
    check(r#"<?php $a=new ArrayIterator([]);$d=$a->__debugInfo();for($i=0;$i<40;$i++){$a[]=$i;}echo count($d["\0ArrayIterator\0storage"]);
"#, "40");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_unset() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1,'y'=>2]);$d=$a->__debugInfo();unset($a['x']);echo json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"y\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_sort() {
    check(r#"<?php $a=new ArrayIterator(['a'=>2,'b'=>1]);$d=$a->__debugInfo();$a->asort();echo json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"a\":2,\"b\":1}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_promote() {
    check(r#"<?php $a=new ArrayIterator([1]);$d=$a->__debugInfo();$a['x']=2;echo json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"0\":1,\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_reverse_write() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1]);$d=$a->__debugInfo();$d["\0ArrayIterator\0storage"]['x']=2;echo json_encode($a->getArrayCopy()).'|'.json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"x\":1}|{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_clone() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1]);$d=$a->__debugInfo();$b=clone $a;$b['x']=2;echo json_encode($a->getArrayCopy()).'|'.json_encode($b->getArrayCopy()).'|'.json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"x\":2}|{\"x\":2}|{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_copy() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1]);$d=$a->__debugInfo();$copy=$a->getArrayCopy();$a['x']=2;echo json_encode($copy).'|'.json_encode($d["\0ArrayIterator\0storage"]);
"#, "{\"x\":1}|{\"x\":2}");
}

/// Preserves the complete pinned PHP storage-view result and ownership.
#[test]
fn test_spl_debug_arrayiterator_detached_view() {
    check(r#"<?php $a=new ArrayIterator(['x'=>1]);$d=$a->__debugInfo();$d["\0ArrayIterator\0storage"]['x']=2;$a['x']=3;echo json_encode($d["\0ArrayIterator\0storage"]).'|'.json_encode($a->getArrayCopy());
"#, "{\"x\":2}|{\"x\":3}");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayobject_throw_set() {
    check(r#"<?php class D {public static mixed $owner=null;function __destruct(){echo 'drop|';throw new Exception('drop');}}$a=new ArrayObject(['x'=>new D()]);$d=$a->__debugInfo();try{$a['x']=2;}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayObject\0storage"]),':',count($a),'|';unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|{\"x\":2}:1|done");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayobject_throw_unset() {
    check(r#"<?php class D {public static mixed $owner=null;function __destruct(){echo 'drop|';throw new Exception('drop');}}$a=new ArrayObject(['x'=>new D()]);$d=$a->__debugInfo();try{unset($a['x']);}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayObject\0storage"]),':',count($a),'|';unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|[]:0|done");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayiterator_throw_set() {
    check(r#"<?php class D {public static mixed $owner=null;function __destruct(){echo 'drop|';throw new Exception('drop');}}$a=new ArrayIterator(['x'=>new D()]);$d=$a->__debugInfo();try{$a['x']=2;}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayIterator\0storage"]),':',count($a),'|';unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|{\"x\":2}:1|done");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayiterator_throw_unset() {
    check(r#"<?php class D {public static mixed $owner=null;function __destruct(){echo 'drop|';throw new Exception('drop');}}$a=new ArrayIterator(['x'=>new D()]);$d=$a->__debugInfo();try{unset($a['x']);}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayIterator\0storage"]),':',count($a),'|';unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|[]:0|done");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayobject_throw_reenter_typed() {
    check(r#"<?php class D {public static ?ArrayObject $owner=null;function __destruct(){echo 'drop|';$owner=self::$owner;if($owner!==null){$owner['y']=9;}throw new Exception('drop');}}$a=new ArrayObject(['x'=>new D()]);D::$owner=$a;$d=$a->__debugInfo();try{$a['x']=2;}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayObject\0storage"]),':',count($a),'|';D::$owner=null;unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|{\"x\":2,\"y\":9}:2|done");
}

/// Retires the live-cell lease when destruction throws or mutates the same container.
#[test]
fn test_spl_debug_exception_arrayiterator_throw_reenter_typed() {
    check(r#"<?php class D {public static ?ArrayIterator $owner=null;function __destruct(){echo 'drop|';$owner=self::$owner;if($owner!==null){$owner['y']=9;}throw new Exception('drop');}}$a=new ArrayIterator(['x'=>new D()]);D::$owner=$a;$d=$a->__debugInfo();try{$a['x']=2;}catch(Exception $e){echo 'caught|';}echo json_encode($d["\0ArrayIterator\0storage"]),':',count($a),'|';D::$owner=null;unset($d,$a);gc_collect_cycles();echo 'done';
"#, "drop|caught|{\"x\":2,\"y\":9}:2|done");
}
