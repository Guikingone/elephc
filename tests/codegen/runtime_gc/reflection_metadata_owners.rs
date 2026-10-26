//! Purpose:
//! Verifies that freshly emitted Reflection metadata transfers its heap ownership.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Functional metadata must remain usable while every discarded owner is released.
//! - User declarations and SPL classes cover small and interface-rich metadata graphs.

use crate::support::{compile_and_run_with_gc_stats, compile_and_run_with_heap_debug, parse_gc_stats};

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_reflection_metadata_owners_empty_class() {
    assert_clean(r#"<?php
class C {} $r=new ReflectionClass(C::class);echo $r->getName();
"#, "C");
}

#[test]
fn test_reflection_metadata_owners_class_constant() {
    assert_clean(r#"<?php
class C {const X=1;} $r=new ReflectionClassConstant(C::class,'X');echo $r->getName();
"#, "X");
}

#[test]
fn test_reflection_metadata_owners_typed_constant_flag() {
    assert_clean(r#"<?php
class C {const int X=1;} $r=new ReflectionClassConstant(C::class,'X');var_dump($r->hasType());
"#, "bool(true)\n");
}

#[test]
fn test_reflection_metadata_owners_typed_constant_value() {
    assert_clean(r#"<?php
class C {const int X=1;} $r=new ReflectionClassConstant(C::class,'X');echo (string)$r->getType();
"#, "int");
}

#[test]
fn test_reflection_metadata_owners_function_parameters() {
    assert_clean(r#"<?php
function f(int|string $x): int|string {return $x;} $r=new ReflectionFunction('f');echo get_class($r->getReturnType()),':',get_class($r->getParameters()[0]->getType());
"#, "ReflectionUnionType:ReflectionUnionType");
}

#[test]
fn test_reflection_metadata_owners_method_prototype() {
    assert_clean(r#"<?php
class P {function f(int $x):int{return $x;}} class C extends P {function f(int $x):int{return $x;}} $r=new ReflectionMethod(C::class,'f');echo $r->getPrototype()->getDeclaringClass()->getName();
"#, "P");
}

#[test]
fn test_reflection_metadata_owners_parent_constructor() {
    assert_clean(r#"<?php
class P {} class C extends P {function __construct(){}} $r=new ReflectionClass(C::class);echo $r->getParentClass()->getName(),':',$r->getConstructor()->getName();
"#, "P:__construct");
}

#[test]
fn test_reflection_metadata_owners_builtin_constant() {
    assert_clean(r#"<?php
$r=new ReflectionClassConstant(ReflectionMethod::class,'IS_PUBLIC');echo $r->getName(),':',$r->getValue();
"#, "IS_PUBLIC:1");
}

#[test]
fn test_reflection_metadata_owners_borrowed_static_payload() {
    assert_clean(r#"<?php
class Payload {function __destruct(){echo 'drop|';}} class C {public static $items=[];} C::$items=[new Payload()];$r=new ReflectionClass(C::class);$properties=$r->getStaticProperties();unset($r);echo count(C::$items),':',count($properties['items']),'|';unset($properties);echo count(C::$items),'|';C::$items=[];echo 'done';
"#, "1:1|1|drop|done");
}

#[test]
fn test_reflection_metadata_owners_property_named_union() {
    assert_clean(r#"<?php
class C {public int $x=1;public int|string $y=2;} $a=new ReflectionProperty(C::class,'x');$b=new ReflectionProperty(C::class,'y');echo get_class($a->getType()),':',get_class($b->getType());
"#, "ReflectionNamedType:ReflectionUnionType");
}

#[test]
fn test_reflection_metadata_owners_empty_backed_enum() {
    assert_clean(r#"<?php
enum E:int {} $r=new ReflectionEnum(E::class);echo $r->getName(),':',(string)$r->getBackingType();
"#, "E:int");
}

#[test]
fn test_reflection_metadata_owners_spl_default_heap() {
    // This interface-rich graph has tens of thousands of allocations. Checking
    // the entire free list on every release is quadratic in heap-debug mode;
    // validate its default-heap behavior and balanced owners with GC counters.
    let out = compile_and_run_with_gc_stats(r#"<?php
$r=new ReflectionClass(SplDoublyLinkedList::class);echo count($r->getProperties()),':',count($r->getDefaultProperties());
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0:0", "{}", out.stderr);
    let (allocations, frees) = parse_gc_stats(&out.stderr);
    assert!(allocations > 0);
    assert_eq!(allocations, frees, "{}", out.stderr);
}

#[test]
fn test_reflection_metadata_owners_empty_maps_grow_without_aliasing() {
    assert_clean(r#"<?php
class C {} $r=new ReflectionClass(C::class);$a=$r->getDefaultProperties();$b=$a;
for($i=0;$i<20;$i++){$a['item'.$i]='value'.$i;}
echo count($a),':',count($b),':',count($r->getDefaultProperties()),':',$a['item19'];
"#, "20:0:0:value19");
}


#[test]
fn test_reflection_metadata_owners_renderer_hides_parameter_backing_slots() {
    assert_clean(r#"<?php
$r=new ReflectionMethod('SplFileObject','fputcsv');
foreach ($r->getParameters() as $p) {
    ob_start(); var_dump($p); $view=ob_get_clean();
    echo $p->getName(), ':', substr_count($view, '["name"]'), ':',
        str_contains($view, '__name') ? 'backing' : 'public', "\n";
}
"#, "fields:1:public\nseparator:1:public\nenclosure:1:public\nescape:1:public\neol:1:public\n");
}

#[test]
fn test_reflection_metadata_owners_renderer_keeps_user_private_double_underscore() {
    assert_clean(r#"<?php
class UserBackingName { private string $__name='user'; }
ob_start(); var_dump(new UserBackingName()); $view=ob_get_clean();
echo str_contains($view, '["__name":"UserBackingName":private]') ? 'visible' : 'missing';
"#, "visible");
}
