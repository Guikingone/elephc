//! Purpose:
//! Pins ordinary unserialize property hydration and its temporary/default owners.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Parsed value boxes and overwritten defaults must retire independently.
//! - These cases use ordinary objects with no SPL hook or iterator involvement.

use crate::support::compile_and_run_with_heap_debug;

fn assert_value(source: &str, expected: &str) {
    let out=compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

fn assert_clean(source: &str) { assert_value(source, "ok"); }

#[test]
fn test_unserialize_property_owners_empty_control() {
    assert_clean("<?php class P {} $s=new P();$copy=unserialize(serialize($s));echo 'ok';");
}

#[test]
fn test_unserialize_property_owners_typed_integer() {
    assert_clean("<?php class P {public int $value=7;} $s=new P();$copy=unserialize(serialize($s));echo 'ok';");
}

#[test]
fn test_unserialize_property_owners_typed_string_default() {
    assert_clean("<?php class P {public string $value='old';} $s=new P();$copy=unserialize(serialize($s));echo 'ok';");
}

#[test]
fn test_unserialize_property_owners_mixed_default() {
    assert_clean("<?php class P {public mixed $value=7;} $s=new P();$copy=unserialize(serialize($s));echo 'ok';");
}


#[test]
fn test_unserialize_property_owners_changed_scalar_values() {
    assert_value(r#"<?php
class P {public int $i=7;public string $s='old';public mixed $m=7;}
$p=new P();$p->i=9;$p->s='new';$p->m=['x'];
$copy=unserialize(serialize($p));echo $copy->i,':',$copy->s,':',$copy->m[0];
"#, "9:new:x");
}

#[test]
fn test_unserialize_property_owners_array_default_replacement() {
    assert_value(r#"<?php
class P {public array $items=[0];}
$p=new P();$p->items=['a','b'];$copy=unserialize(serialize($p));
echo implode(',',$copy->items);
"#, "a,b");
}

#[test]
fn test_unserialize_property_owners_typed_object_backreference() {
    assert_value(r#"<?php
class Peer {public string $value='old';}
class P {public Peer $a;public Peer $b;}
$p=new P();$p->a=new Peer();$p->a->value='new';$p->b=$p->a;
$copy=unserialize(serialize($p));echo $copy->a===$copy->b?'same:':'different:';
echo $copy->a->value;
"#, "same:new");
}


#[test]
fn test_unserialize_property_owners_scalar_and_nullable_storage() {
    assert_value(r#"<?php
class P {public bool $b=false;public float $f=0.0;public ?int $n=null;}
$p=new P();$p->b=true;$p->f=1.5;$p->n=7;
$copy=unserialize(serialize($p));var_dump($copy->b,$copy->f,$copy->n);
$copy->n=null;$other=unserialize(serialize($copy));var_dump($other->n);
"#, "bool(true)\nfloat(1.5)\nint(7)\nNULL\n");
}

#[test]
fn test_unserialize_property_owners_private_shadow_wire_slots() {
    assert_value(r#"<?php
class A {private string $v='parent';function parentValue():string{return $this->v;}}
class B extends A {private string $v='child';function childValue():string{return $this->v;}}
$wire='O:1:"B":2:{s:4:"'."\0A\0v".'";s:3:"one";s:4:"'."\0B\0v".'";s:3:"two";}';
$copy=unserialize($wire);echo $copy->parentValue(),':',$copy->childValue();
"#, "one:two");
}


#[test]
fn test_unserialize_property_owners_nullable_int_rejects_wire_types() {
    assert_value(r#"<?php
class C {public ?int $n=null;}
foreach(['s:3:"abc";','s:1:"7";','d:1.5;','b:1;','b:0;','a:0:{}','O:8:"stdClass":0:{}'] as $payload){
 try {$o=unserialize('O:1:"C":1:{s:1:"n";'.$payload.'}');}
 catch(TypeError $e){echo $e->getMessage(),"\n";}
}
"#, concat!(
"Cannot assign string to property C::$n of type ?int\n",
"Cannot assign string to property C::$n of type ?int\n",
"Cannot assign float to property C::$n of type ?int\n",
"Cannot assign true to property C::$n of type ?int\n",
"Cannot assign false to property C::$n of type ?int\n",
"Cannot assign array to property C::$n of type ?int\n",
"Cannot assign stdClass to property C::$n of type ?int\n"));
}

#[test]
fn test_unserialize_property_owners_nested_type_error_preserves_outer_context() {
    assert_value(r#"<?php
class C {public ?int $n=null;}
class Outer {
 public string $value='outer';
 function __wakeup():void {
  try {unserialize('O:1:"C":1:{s:1:"n";s:3:"bad";}');}
  catch(TypeError $e){echo "caught:";}
 }
}
$values=unserialize('a:2:{i:0;O:5:"Outer":1:{s:5:"value";s:4:"kept";}i:1;r:2;}');
echo $values[0]===$values[1]?'same:':'different:';echo $values[0]->value;
"#, "caught:same:kept");
}

#[test]
fn test_unserialize_property_owners_eval_bridge_writer_labels() {
    assert_value(r#"<?php
class P {public ?int $n=null;}
$code=$argc>0?'return 1;':'return 0;';echo eval($code),':';
$p=unserialize('O:1:"P":1:{s:1:"n";i:7;}');var_dump($p->n);
"#, "1:int(7)\n");
}
