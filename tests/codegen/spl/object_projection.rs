//! Purpose:
//! Checks SPL wrapper array casts and PHP-visible own-property projections.
//!
//! Called from:
//! - The SPL codegen integration suite.
//!
//! Key details:
//! - Array casts select backing data or own properties according to STD_PROP_LIST.
//! - Internal container storage is absent from PHP property enumeration.

use crate::support::compile_and_run_with_heap_debug;

#[test]
fn test_spl_object_projection_cast_uses_backing_array() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$a = new ArrayObject([10, 20]);
$i = new ArrayIterator(['x' => 30]);
var_dump((array)$a, (array)$i);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "array(2) {\n  [0]=>\n  int(10)\n  [1]=>\n  int(20)\n}\narray(1) {\n  [\"x\"]=>\n  int(30)\n}\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_std_prop_list_selects_own_properties() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AO extends ArrayObject { public $own = 1; protected $hidden = 2; private $secret = 3; }
class AI extends ArrayIterator { public $own = 4; protected $hidden = 5; private $secret = 6; }
$a = new AO([10, 20], ArrayObject::STD_PROP_LIST);
$i = new AI([30, 40], ArrayIterator::STD_PROP_LIST);
$ac = (array)$a;
$ic = (array)$i;
echo count($ac), ':', $ac['own'], ':', $ac["\0*\0hidden"], ':', $ac["\0AO\0secret"], '|';
echo count($ic), ':', $ic['own'], ':', $ic["\0*\0hidden"], ':', $ic["\0AI\0secret"];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3:1:2:3|3:4:5:6", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_variables_hide_builtin_storage() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AO extends ArrayObject { public $own = 1; private $secret = 2; }
$a = new ArrayObject([10]);
$i = new ArrayIterator([20]);
$c = new AO([30]);
echo count(get_object_vars($a)), ':', count(get_mangled_object_vars($a)), '|';
echo count(get_object_vars($i)), ':', count(get_mangled_object_vars($i)), '|';
echo count(get_object_vars($c)), ':', count(get_mangled_object_vars($c));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0:0|0:0|1:2", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_cast_ignores_public_copy_override() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AO extends ArrayObject { public function getArrayCopy(): array { return ['wrong' => 99]; } }
$a = new AO(['right' => 7]);
$copy = (array)$a;
echo count($copy), ':', $copy['right'];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:7", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_mixed_receiver_cast() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function copyValue($value): array { return (array)$value; }
$a = new ArrayObject(['x' => 7]);
$i = new ArrayIterator(['y' => 8]);
$ac = copyValue($a);
$ic = copyValue($i);
echo count($ac), ':', $ac['x'], '|', count($ic), ':', $ic['y'];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:7|1:8", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_temporary_cast_keeps_array_values_alive() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Payload { public function __destruct() { echo 'payload|'; } }
class AO extends ArrayObject { public function __destruct() { echo 'wrapper|'; } }
$copy = (array)new AO([new Payload()]);
echo 'held:', count($copy), ':', get_class($copy[0]), '|';
unset($copy);
echo 'done';
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "wrapper|held:1:Payload|payload|done", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Flag declarations carry the same typed integer metadata as PHP's internal classes.
#[test]
fn test_spl_object_projection_flag_constant_types() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$a = new ReflectionClassConstant(ArrayObject::class, 'STD_PROP_LIST');
$i = new ReflectionClassConstant(ArrayIterator::class, 'ARRAY_AS_PROPS');
var_dump($a->hasType(), $i->hasType());
echo (string)$a->getType(), ':', (string)$i->getType();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "bool(true)\nbool(true)\nint:int", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_all_flag_combinations() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class AO extends ArrayObject { public $own = 7; }
class AI extends ArrayIterator { public $own = 8; }
foreach ([0, 1, 2, 3] as $flags) {
    $a = new AO(['data' => 10], $flags);
    $ac = (array)$a;
    $i = new AI(['data' => 20], $flags);
    $ic = (array)$i;
    echo $flags, ':', $ac['data'] ?? $ac['own'], ':', $ic['data'] ?? $ic['own'], '|';
}
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0:10:20|1:7:8|2:10:20|3:7:8|", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_recursive_iterator_inherits_native_view() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$i = new RecursiveArrayIterator(['x' => 7]);
$copy = (array)$i;
echo count($copy), ':', $copy['x'], ':', count(get_mangled_object_vars($i));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:7:0", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_append_view_uses_internal_copy_provider() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$a = new AppendIterator();
$a->append(new ArrayIterator([1]));
$view = $a->getArrayIterator();
$copy = (array)$view;
echo count($copy), ':', get_class($copy[0]), ':', count(get_mangled_object_vars($view));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:ArrayIterator:0", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_object_backing_keeps_mangled_properties() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class Backing { public $visible=1; protected $hidden=2; private $secret=3; }
$a=new ArrayObject(new Backing());
$copy=(array)$a;
echo count($copy), ':', $copy['visible'], ':', $copy["\0*\0hidden"], ':', $copy["\0Backing\0secret"];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3:1:2:3", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_nested_backing_ignores_inner_cast_flags() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class Inner extends ArrayObject { public $own=7; }
$inner=new Inner(['data'=>10], ArrayObject::STD_PROP_LIST);
$outer=new ArrayObject($inner, 0);
$copy=(array)$outer;
echo count($copy), ':', $copy['data'];
$inner->setFlags(0);
$copy=(array)$outer;
echo '|', count($copy), ':', $copy['data'];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:10|1:10", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_self_backing_uses_own_properties() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class SelfBacked extends ArrayObject {
    public $own=7; private $secret=8;
    public function __construct() { parent::__construct($this); }
}
$a=new SelfBacked();
$copy=(array)$a;
echo count($copy), ':', $copy['own'], ':', $copy["\0SelfBacked\0secret"];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "2:7:8", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_cast_normalizes_raw_numeric_property_keys() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
$o=new stdClass();
$a=new ArrayObject($o);
$a[]='integer';
$a[0]='named';
$copy=(array)$a;
$raw=$a->getArrayCopy();
echo count($copy), ':', $copy[0], '|', count($raw);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:named|2", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_spl_object_projection_list_backing_hides_native_slots() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
class C extends SplDoublyLinkedList {public int $visible=7;private int $secret=8;}
$c=new C();$a=new ArrayObject($c);$v=(array)$a;
echo count(get_object_vars($c)),':',count($v),':',$v['visible'],':',$v["\0C\0secret"];
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "1:2:7:8", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
