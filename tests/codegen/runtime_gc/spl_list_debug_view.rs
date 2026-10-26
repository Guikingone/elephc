//! Purpose:
//! Checks native SPL list debug output without exposing storage properties.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Native fields follow user properties and keep physical list order.
//! - Capture mode, inherited debug handlers and user overrides retain PHP behavior.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_list_debug_view_base() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push(1);$c->push(2);$c->push(3);print_r($c);
"#, "SplDoublyLinkedList Object\n(\n    [flags:SplDoublyLinkedList:private] => 0\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => 1\n            [1] => 2\n            [2] => 3\n        )\n\n)\n");
}

#[test]
fn test_spl_list_debug_view_user_properties() {
    assert_clean(r#"<?php
class C extends SplStack {public int $visible=7;private int $secret=8;}
$c=new C();$c->push('a');print_r($c);
"#, "C Object\n(\n    [visible] => 7\n    [secret:C:private] => 8\n    [flags:SplDoublyLinkedList:private] => 6\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => a\n        )\n\n)\n");
}

#[test]
fn test_spl_list_debug_view_capture_keeps_property_views() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push('a');$text=print_r($c,true);echo $text;
echo 'views:',count((array)$c),':',count(get_object_vars($c)),':',(int)(strpos(var_export($c,true),'dllist')===false);
"#, "SplDoublyLinkedList Object\n(\n    [flags:SplDoublyLinkedList:private] => 0\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => a\n        )\n\n)\nviews:0:0:1");
}

#[test]
fn test_spl_list_debug_view_user_override() {
    assert_clean(r#"<?php
class C extends SplDoublyLinkedList {public int $visible=7;function __debugInfo():array{return ['only'=>$this->visible];}}
$c=new C();$c->push('a');print_r($c);
"#, "C Object\n(\n    [only] => 7\n)\n");
}

#[test]
fn test_spl_list_debug_view_dynamic_properties() {
    assert_clean(r#"<?php
#[AllowDynamicProperties] class C extends SplDoublyLinkedList {public int $visible=7;}
$c=new C();$c->extra=9;$c->push('a');print_r($c);
"#, "C Object\n(\n    [visible] => 7\n    [extra] => 9\n    [flags:SplDoublyLinkedList:private] => 0\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => a\n        )\n\n)\n");
}

#[test]
fn test_spl_list_debug_view_recursion() {
    assert_clean(r#"<?php
$c=new SplDoublyLinkedList();$c->push($c);print_r($c);unset($c);
"#, "SplDoublyLinkedList Object\n(\n    [flags:SplDoublyLinkedList:private] => 0\n    [dllist:SplDoublyLinkedList:private] => Array\n        (\n            [0] => SplDoublyLinkedList Object\n *RECURSION*\n        )\n\n)\n");
}
