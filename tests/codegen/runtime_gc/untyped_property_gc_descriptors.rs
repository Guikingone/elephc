//! Purpose:
//! Covers owners and JSON values in untyped properties restored from wire data.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - No source assignment refines the defaultless property's logical Void type.
//! - Collection and object destruction must visit its physical Mixed cell.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_untyped_property_gc_descriptors_restored_string() {
    assert_clean(r#"<?php
class Plain {public $value;}
$copy=unserialize('O:5:"Plain":1:{s:5:"value";s:4:"kept";}');
echo serialize($copy);
"#, "O:5:\"Plain\":1:{s:5:\"value\";s:4:\"kept\";}");
}

#[test]
fn test_untyped_property_gc_descriptors_self_cycle() {
    assert_clean(r#"<?php
class Plain {public $self;}
$copy=unserialize('O:5:"Plain":1:{s:4:"self";r:1;}');
echo serialize($copy),'|';unset($copy);gc_collect_cycles();echo 'done';
"#, "O:5:\"Plain\":1:{s:4:\"self\";r:1;}|done");
}

#[test]
fn test_untyped_property_gc_descriptors_restored_array() {
    assert_clean(r#"<?php
class Plain {public $value;}
$copy=unserialize('O:5:"Plain":1:{s:5:"value";a:2:{i:0;i:1;i:1;s:4:"kept";}}');
echo serialize($copy);
"#, "O:5:\"Plain\":1:{s:5:\"value\";a:2:{i:0;i:1;i:1;s:4:\"kept\";}}");
}

#[test]
fn test_untyped_property_gc_descriptors_json_string() {
    assert_clean(r#"<?php
class Plain {public $value;}
$copy=unserialize('O:5:"Plain":1:{s:5:"value";s:4:"kept";}');
echo json_encode($copy);
"#, "{\"value\":\"kept\"}");
}

#[test]
fn test_untyped_property_gc_descriptors_implicit_null() {
    assert_clean(r#"<?php
class Plain {public $value;}
$copy=new Plain();echo json_encode($copy);
"#, "{\"value\":null}");
}

#[test]
fn test_untyped_property_gc_descriptors_explicit_null_default() {
    assert_clean(r#"<?php
class Plain {public $value=null;}
$copy=new Plain();echo json_encode($copy);
"#, "{\"value\":null}");
}

#[test]
fn test_untyped_property_gc_descriptors_null_sentinel_during_cycle_collection() {
    assert_clean(r#"<?php
class Plain {public $value=null;public mixed $self=null;}
$copy=new Plain();$copy->self=$copy;unset($copy);gc_collect_cycles();echo 'done';
"#, "done");
}

#[test]
fn test_untyped_property_gc_descriptors_mixed_null_encoding() {
    assert_clean(r#"<?php
$value=$argc>0?null:'kept';echo json_encode($value);
"#, "null");
}
