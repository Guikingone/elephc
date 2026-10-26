//! Purpose:
//! Pins the native SplObjectStorage serialization protocol and hydration ownership.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - PHP wire data combines interleaved object/info entries and mangled user properties.
//! - Round trips and explicit hook replacement must retain independent storage owners.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_storage_serialization_magic_pair() {
    assert_clean(r#"<?php
error_reporting(0);
class P {public int $value=7;}
$s=new SplObjectStorage(); $p=new P(); $s[$p]='info';
$copy=unserialize(serialize($s));
echo count($copy),':';
foreach($copy as $value){echo $value->value,':',$copy[$value];}
"#, "1:7:info");
}

#[test]
fn test_spl_object_storage_serialization_subclass_property_roundtrip() {
    assert_clean(r#"<?php
class A extends SplObjectStorage {
    protected $a=null;
    function __construct(){$this->a='123';}
    function getA(){return $this->a;}
}
$obj=unserialize(serialize(new A())); var_dump($obj->getA());
"#, "string(3) \"123\"\n");
}

#[test]
fn test_spl_object_storage_serialization_empty_wire_format() {
    assert_clean(r#"<?php echo serialize(new SplObjectStorage());"#,
        "O:16:\"SplObjectStorage\":2:{i:0;a:0:{}i:1;a:0:{}}");
}

#[test]
fn test_spl_object_storage_serialization_explicit_hook_merges_and_validates() {
    assert_clean(r#"<?php
error_reporting(0);
$a=new stdClass(); $b=new stdClass(); $s=new SplObjectStorage(); $s[$a]='a';
$s->__unserialize([[$b,'b'],[]]); echo count($s),':',$s[$a],':',$s[$b],"\n";
try {$s->__unserialize([[$b,'new',1,null],[]]);}
catch (UnexpectedValueException $e) {echo $e->getMessage(),':',count($s),':',$s[$b],"\n";}
try {$s->__unserialize([[$b],[]]);}
catch (UnexpectedValueException $e) {echo $e->getMessage(),':',count($s),"\n";}
try {$s->__unserialize([]);}
catch (UnexpectedValueException $e) {echo $e->getMessage(),"\n";}
try {$s->__unserialize([[],1]);}
catch (UnexpectedValueException $e) {echo $e->getMessage(),"\n";}
"#, "2:a:b\nNon-object key:2:new\nOdd number of elements:2\nIncomplete or ill-typed serialization data\nIncomplete or ill-typed serialization data\n");
}

#[test]
fn test_spl_object_storage_serialization_private_typed_property_replacement() {
    assert_clean(r#"<?php
class A extends SplObjectStorage {
    private string $label='old'; protected array $items=[1,2];
    function summary(){return $this->label.':'.implode(',', $this->items);}
}
$s=new A(); $copy=unserialize(serialize($s)); echo $copy->summary(),"\n";
$copy->__unserialize([[],["\0A\0label"=>'new',"\0*\0items"=>[3,4]]]);
echo $copy->summary();
"#, "old:1,2\nnew:3,4");
}

#[test]
fn test_spl_object_storage_serialization_shadowed_private_properties() {
    assert_clean(r#"<?php
class A extends SplObjectStorage {
    private string $label='parent'; function parentLabel(){return $this->label;}
}
class B extends A {
    private string $label='child'; function childLabel(){return $this->label;}
}
$s=unserialize(serialize(new B())); echo $s->parentLabel(),':',$s->childLabel();
"#, "parent:child");
}
