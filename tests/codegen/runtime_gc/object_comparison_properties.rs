//! Purpose:
//! Ensures object equality reads physical properties independently of debug output.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Hidden/projected debug properties still participate in equality.
//! - Fresh comparison boxes borrow their receiver and own their payload lease.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_object_comparison_properties_empty_debug_projection() {
    assert_clean(r#"<?php
class A {public int $value=0;function __debugInfo(){return [];}}
$a=new A();$b=new A();$a->value=1;$b->value=2;
var_dump($a==$b);$b->value=1;var_dump($a==$b);
"#, "bool(false)\nbool(true)\n");
}

#[test]
fn test_object_comparison_properties_partial_debug_projection() {
    assert_clean(r#"<?php
class A {
 public int $shown=1;private string $secret='old';
 function setSecret(string $value):void{$this->secret=$value;}
 function __debugInfo(){return ['visible'=>$this->shown];}
}
$a=new A();$b=new A();$b->setSecret('new');var_dump($a==$b);
$b->setSecret('old');var_dump($a==$b);
"#, "bool(false)\nbool(true)\n");
}

#[test]
fn test_object_comparison_properties_array_payload_owners() {
    assert_clean(r#"<?php
class A {public array $items=[1,'kept'];function __debugInfo(){return [];}}
$a=new A();$b=new A();var_dump($a==$b);
$b->items=[2,'new'];var_dump($a==$b);$b->items=[1,'kept'];var_dump($a==$b);
echo $a->items[1],':',$b->items[1];
"#, "bool(true)\nbool(false)\nbool(true)\nkept:kept");
}

#[test]
fn test_object_comparison_properties_shadowed_private_slots() {
    assert_clean(r#"<?php
class A {private int $value=1;function parentValue(int $v):void{$this->value=$v;}}
class B extends A {private int $value=2;function childValue(int $v):void{$this->value=$v;}}
$a=new B();$b=new B();$b->parentValue(5);var_dump($a==$b);
$b->parentValue(1);var_dump($a==$b);$b->childValue(9);var_dump($a==$b);
"#, "bool(false)\nbool(true)\nbool(false)\n");
}

#[test]
fn test_object_comparison_properties_does_not_call_debug_hook() {
    assert_clean(r#"<?php
class A {
 public int $value=0;
 function __debugInfo(){echo 'hook';return [];}
}
$a=new A();$b=new A();$b->value=1;var_dump($a==$b);
"#, "bool(false)\n");
}
