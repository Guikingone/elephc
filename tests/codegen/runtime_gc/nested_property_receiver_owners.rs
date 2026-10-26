//! Purpose:
//! Verifies owned receivers survive property stores and retire on success or unwind.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - A property-chain read may produce an independent object lease.
//! - Ordinary, temporary and magic receivers need the same bounded ownership scope.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source:&str, expected:&str) {
    let out=compile_and_run_with_heap_debug(source);
    assert!(out.success,"{} {}",out.stdout,out.stderr);
    assert_eq!(out.stdout,expected,"{}",out.stderr);
    assert!(out.stderr.contains("leak summary: clean"),"{}",out.stderr);
}

#[test]
fn test_nested_property_receiver_owners_chain_assignment() {
    assert_clean(r#"<?php
class Peer {public string $value='old';}
class P {public Peer $a;public Peer $b;}
$p=new P();$p->a=new Peer();$p->a->value='new';$p->b=$p->a;echo 'ok';
"#,"ok");
}

#[test]
fn test_nested_property_receiver_owners_rhs_throw() {
    assert_clean(r#"<?php
class Peer {public string $value='old';function __destruct(){echo 'drop|';}}
class P {public Peer $a;}
function fail():string {throw new RuntimeException('bad');}
$p=new P();$p->a=new Peer();
try {$p->a->value=fail();} catch(RuntimeException $e){echo 'caught|';}
unset($p);
"#,"caught|drop|");
}

#[test]
fn test_nested_property_receiver_owners_temporary_receiver() {
    assert_clean(r#"<?php
class Peer {public string $value='old';function __destruct(){echo 'drop|';}}
function make():Peer{return new Peer();}
make()->value='new';echo 'done';
"#,"drop|done");
}

#[test]
fn test_nested_property_receiver_owners_magic_receiver() {
    assert_clean(r#"<?php
final class Magic {
 function __set(string $name,mixed $value):void{echo 'set|';}
 function __destruct(){echo 'drop|';}
}
function make():Magic{return new Magic();}
make()->value='new';echo 'done';
"#,"set|drop|done");
}


#[test]
fn test_nested_property_receiver_owners_store_type_error() {
    assert_clean(r#"<?php
class Peer {public int $value=1;function __destruct(){echo 'drop|';}}
function make():Peer{return new Peer();}
function bad():mixed{return ['bad'];}
try {make()->value=bad();} catch(TypeError $e){echo 'caught';}
"#,"drop|caught");
}

#[test]
fn test_nested_property_receiver_owners_magic_throw() {
    assert_clean(r#"<?php
final class Magic {
 function __set(string $name,mixed $value):void{echo 'set|';throw new RuntimeException('bad');}
 function __destruct(){echo 'drop|';}
}
function make():Magic{return new Magic();}
try {make()->value='new';} catch(RuntimeException $e){echo 'caught';}
"#,"set|drop|caught");
}
