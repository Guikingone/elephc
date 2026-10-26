//! Purpose:
//! Keeps native SplObjectStorage contents observable to object equality.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Hidden native backing fields must participate independently of renderers.
//! - The unchanged php-src observer_007 fixture covers empty, shared and changed data.

use crate::support::compile_and_run_with_heap_debug;

#[test]
fn test_spl_object_storage_comparison_keeps_hidden_native_entries() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$a=new SplObjectStorage();$b=new SplObjectStorage();var_dump($a==$b);
$b[$b]=2;var_dump($a==$b);$a[$b]=2;var_dump($a==$b);
$a[$b]=3;var_dump($a==$b);
"#);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "bool(true)\nbool(false)\nbool(true)\nbool(false)\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_storage_comparison_matches_object_identity() {
    assert_clean(r#"<?php
$x=new stdClass();$y=new stdClass();$a=new SplObjectStorage();$b=new SplObjectStorage();
$a[$x]=1;$b[$y]=1;var_dump($a==$b);
$b=new SplObjectStorage();$b[$x]='1';var_dump($a==$b);
$b[$x]=2;var_dump($a==$b);
"#, "bool(false)\nbool(true)\nbool(false)\n");
}

#[test]
fn test_spl_object_storage_comparison_ignores_order_and_cursor() {
    assert_clean(r#"<?php
$x=new stdClass();$y=new stdClass();$a=new SplObjectStorage();$b=new SplObjectStorage();
$a[$x]=1;$a[$y]='2';$b[$y]=2;$b[$x]=1;var_dump($a==$b);
$a->next();var_dump($a==$b);$b[$x]=9;var_dump($a==$b);
"#, "bool(true)\nbool(true)\nbool(false)\n");
}

#[test]
fn test_spl_object_storage_comparison_distinct_subclasses_are_uncomparable() {
    assert_clean(r#"<?php
class S extends SplObjectStorage {public int $label=0;}
$a=new S();$b=new S();var_dump($a==$b);var_dump($a==$a);
$x=new stdClass();$a[$x]=1;$b[$x]=1;var_dump($a==$b);
"#, "bool(false)\nbool(true)\nbool(false)\n");
}

#[test]
fn test_spl_object_storage_comparison_leaves_dll_native_state_out_of_standard_properties() {
    assert_clean(r#"<?php
$a=new SplDoublyLinkedList();$b=new SplDoublyLinkedList();$a->push(1);$b->push(2);
var_dump($a==$b);$a->rewind();$a->next();var_dump($a==$b);
class D extends SplDoublyLinkedList {public string $label='first';}
$c=new D();$d=new D();$c->push(1);$d->push(2);var_dump($c==$d);
$d->label='other';var_dump($c==$d);
"#, "bool(true)\nbool(true)\nbool(true)\nbool(false)\n");
}

#[test]
fn test_spl_object_storage_comparison_keeps_nested_info_payloads_alive() {
    assert_clean(r#"<?php
class KeyObject {public string $name='key';}
$x=new KeyObject();$a=new SplObjectStorage();$b=new SplObjectStorage();
$a[$x]=['n'=>['kept']];$b[$x]=['n'=>['kept']];var_dump($a==$b);
echo $x->name,'|',$a[$x]['n'][0],':',$b[$x]['n'][0];
"#, "bool(true)\nkey|kept:kept");
}
