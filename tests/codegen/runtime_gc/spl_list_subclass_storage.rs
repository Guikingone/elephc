//! Purpose:
//! Checks inherited SPL list storage allocation and owner lifetimes.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Native list state must coexist with user properties and custom constructors.
//! - Ordinary, dynamic and inherited calls retain the concrete receiver identity.

use crate::support::{compile_and_run, compile_and_run_with_heap_debug};

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_list_subclass_storage_custom_constructor() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {function __construct(int $n){$this->push($n);}} $c=new C(3);echo $c->count(),':',$c->top();
"#, "1:3");
}

#[test]
fn test_spl_list_subclass_storage_dynamic_new() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} class D extends C {} $name=$argc===1?C::class:D::class;$c=new $name();$c->push(1);echo get_class($c),':',$c->count();
"#, "C:1");
}

#[test]
fn test_spl_list_subclass_storage_inherited_modes() {
    assert_clean(r#"<?php
class S extends SplStack {} class Q extends SplQueue {} $s=new S();$q=new Q();echo $s->getIteratorMode(),':',$q->getIteratorMode();
"#, "6:4");
}

#[test]
fn test_spl_list_subclass_storage_inherited_push() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c->push('x');echo $c->count(),':',$c->top();
"#, "1:x");
}

#[test]
fn test_spl_list_subclass_storage_override_count() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {function count():int{return -parent::count();}} $c=new C();$c->push(1);echo $c->count(),':',count($c);
"#, "-1:-1");
}

#[test]
fn test_spl_list_subclass_storage_payload_lifetime() {
    assert_clean(r#"<?php
class P {function __destruct(){echo 'drop|';}} class C extends SplDoublyLinkedList {} $c=new C();$c->push(new P());echo 'held|';unset($c);echo 'done';
"#, "held|drop|done");
}

#[test]
fn test_spl_list_subclass_storage_user_properties() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {public int $number=7;public string $label='keep';} $c=new C();$c->push(9);echo $c->number,':',$c->label,':',$c->top(),':',count(get_object_vars($c));
"#, "7:keep:9:2");
}

#[test]
fn test_spl_list_subclass_storage_property_views() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {public int $visible=7;private int $secret=8;} $c=new C();$a=(array)$c;echo count(get_object_vars($c)),':',count(get_mangled_object_vars($c)),':',count($a),':',$a['visible'],':',$a["\0C\0secret"];
"#, "1:2:2:7:8");
}

#[test]
fn test_spl_list_subclass_storage_base_property_views() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push(3);echo count(get_object_vars($c)),':',count(get_mangled_object_vars($c)),':',count((array)$c),':',$c->top();
"#, "0:0:0:3");
}

#[test]
fn test_spl_list_subclass_storage_reflection_properties() {
    let out=compile_and_run(r#"<?php
$r=new ReflectionClass(SplDoublyLinkedList::class);echo count($r->getProperties()),':',count($r->getDefaultProperties());
"#);
    assert_eq!(out, "0:0");
}

#[test]
fn test_spl_list_subclass_storage_dynamic_base_modes() {
    assert_clean(r#"<?php
$name=$argc===1?SplStack::class:SplQueue::class;$c=new $name();$c->push('x');echo get_class($c),':',$c->top(),':',$c->getIteratorMode();
"#, "SplStack:x:6");
}

#[test]
fn test_spl_list_subclass_storage_base_and_child() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {public string $label='keep';}$a=new SplDoublyLinkedList();$b=new C();$a->push('a');$b->push('b');echo $a->top(),':',$b->top(),':',$b->label;
"#, "a:b:keep");
}

#[test]
fn test_spl_list_subclass_storage_clone_independent_nodes() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c->push('first');$c->push('second');$d=clone $c;echo $d->pop(),':',$c->count(),':',$d->count(),':',$c->top(),':',$d->top();
"#, "second:2:1:second:first");
}

#[test]
fn test_spl_list_subclass_storage_clone_resets_cursor() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c->push('first');$c->push('second');$c->rewind();$c->next();$d=clone $c;echo $c->key(),':',$d->key(),':',$c->current(),':',$d->current();
"#, "1:0:second:first");
}

#[test]
fn test_spl_list_subclass_storage_base_clone() {
    assert_clean(r#"<?php
$c=new SplStack();$c->push('first');$c->push('second');$d=clone $c;echo $d->pop(),':',$c->count(),':',$d->count(),':',$c->top(),':',$d->top(),':',$d->getIteratorMode();
"#, "second:2:1:second:first:6");
}

#[test]
fn test_spl_list_subclass_storage_clone_hook_sees_initial_nodes() {
    assert_clean(r#"<?php
class C extends SplStack {function __clone(){echo $this->count(),':',$this->key(),':',$this->current(),'|';$this->pop();}}
$c=new C();$c->push('a');$c->push('b');$c->rewind();$c->next();$d=clone $c;echo $c->top(),':',$d->top();
"#, "2:0:a|b:a");
}

#[test]
fn test_spl_list_subclass_storage_clone_keeps_payload_alive() {
    assert_clean(r#"<?php
class P {function __destruct(){echo 'drop|';}} class C extends SplDoublyLinkedList {} $c=new C();$c->push(new P());$d=clone $c;unset($c);echo 'held|';unset($d);echo 'done';
"#, "held|drop|done");
}

#[test]
fn test_spl_list_subclass_storage_empty_stack_clone_cursor() {
    assert_clean(r#"<?php
$c=new SplStack();$d=clone $c;echo $d->count(),':',$d->key(),':',(int)$d->valid();
"#, "0:0:0");
}

#[test]
fn test_spl_list_subclass_storage_callable_push_keeps_payload_alive() {
    assert_clean(r#"<?php
class P {function __destruct(){echo 'drop|';}} class C extends SplDoublyLinkedList {} $c=new C();$push=$c->push(...);$push(new P());unset($push);echo 'held|';unset($c);echo 'done';
"#, "held|drop|done");
}

#[test]
fn test_spl_list_subclass_storage_key_after_invalid_cursor() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push('a');$c->rewind();$c->next();echo $c->key(),':',(int)$c->valid(),'|';
$s=new SplStack();$s->push('a');$s->push('b');$d=clone $s;$d->next();echo $d->key(),':',(int)$d->valid();
"#, "1:0|-1:0");
}

#[test]
fn test_spl_list_subclass_storage_self_cycle() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {function __destruct(){echo 'drop|';}} $c=new C();$c->push($c);unset($c);echo 'done|';
"#, "done|drop|");
}

#[test]
fn test_spl_list_subclass_storage_callable_serialize_owns_string() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c->push(7);$serialize=$c->serialize(...);echo $serialize();
"#, "i:0;:i:7;");
}

#[test]
fn test_spl_list_subclass_storage_cycle_storage_owners() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c->push($c);unset($c);echo 'done';
"#, "done");
}

#[test]
fn test_spl_list_subclass_storage_iterator_exhausted_positions() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push('a');$c->rewind();$c->next();echo $c->key(),':';$c->next();echo $c->key(),':';$c->prev();echo $c->key(),'|';
$s=new SplStack();$s->push('a');$s->rewind();$s->next();echo $s->key(),':';$s->next();echo $s->key(),':';$s->prev();echo $s->key(),'|';
$p=new SplDoublyLinkedList();$p->push('a');$p->rewind();$p->prev();echo $p->key(),':';$p->prev();echo $p->key();
"#, "1:1:1|-1:-1:-1|-1:-1");
}

#[test]
fn test_spl_list_subclass_storage_iterator_delete_exhausted_position() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push('a');$c->setIteratorMode(3);$c->rewind();$c->next();echo $c->count(),':',$c->key(),':',(int)$c->valid(),'|';$c->next();echo $c->key(),'|';
$d=new SplDoublyLinkedList();$d->setIteratorMode(1);$d->next();echo $d->key(),':',(int)$d->valid();
"#, "0:-1:0|-1|0:0");
}

#[test]
fn test_spl_list_subclass_storage_append() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {} $c=new C();$c[]=5;echo $c[0],':',count($c);
"#, "5:1");
}

#[test]
fn test_spl_list_subclass_storage_builtin_unserialize_dispatch() {
    assert_clean(r#"<?php
$q=new SplQueue();$q->enqueue('a');$q->enqueue('b');$copy=unserialize(serialize($q));
echo get_class($copy),':',count($copy),':',$copy->bottom(),':',$copy->top(),':',$copy->getIteratorMode();
"#, "SplQueue:2:a:b:4");
}

#[test]
fn test_spl_list_subclass_storage_wire_only_unserialize_dispatch() {
    assert_clean(r#"<?php
$q=unserialize('O:8:"SplQueue":3:{i:0;i:4;i:1;a:2:{i:0;s:1:"a";i:1;s:1:"b";}i:2;a:0:{}}');
echo get_class($q),':',count($q),':',$q->bottom(),':',$q->top(),':',$q->getIteratorMode();
"#, "SplQueue:2:a:b:4");
}


#[test]
fn test_spl_list_subclass_storage_native_mixed_lvalue() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$v=$argc===1?'a':7;$c->push($v);unset($v);echo $c->count(),':',$c->top();
"#, "1:a");
}


#[test]
fn test_spl_list_subclass_storage_native_mixed_lvalue_payload() {
    assert_clean(r#"<?php
class P {function __destruct(){echo 'drop|';}} $c=new SplDoublyLinkedList();$v=$argc===1?new P():7;$c->push($v);unset($v);echo 'held|';unset($c);echo 'done';
"#, "held|drop|done");
}


#[test]
fn test_spl_list_subclass_storage_snapshot_owns_payload() {
    assert_clean(r#"<?php
class P {function __destruct(){echo 'drop|';}} $c=new SplDoublyLinkedList();$c->push(new P());$state=$c->__serialize();unset($c);echo 'held|';unset($state);echo 'done';
"#, "held|drop|done");
}


#[test]
fn test_spl_list_subclass_storage_snapshot_is_independent() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push('a');$state=$c->__serialize();$c->pop();echo count($c),':',$state[1][0];
"#, "0:a");
}
