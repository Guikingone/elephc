//! Purpose:
//! PHP-equivalent rendering of computed magic debug views and their owned results.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent registration.
//!
//! Key details:
//! - Hooks execute once before headers and share recursion guards with nested dumps.
//! - Complete output checks include mangled keys, return-mode capture and destructor timing.

use crate::support::compile_and_run_with_heap_debug;

/// Checks the complete PHP rendering and final ownership of receiver and view storage.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Matches PHP for the computed debug view.
#[test]
fn test_object_debug_info_rendering_computed() {
    assert_clean(r#"<?php class C {public int $unused=8;function __debugInfo():array {echo 'hook|';return ['x'=>1+2];}}var_dump(new C());"#, "hook|object(C)#1 (1) {\n  [\"x\"]=>\n  int(3)\n}\n");
}

/// Matches PHP for the numeric keys debug view.
#[test]
fn test_object_debug_info_rendering_numeric_keys() {
    assert_clean(r#"<?php class C {function __debugInfo():array {return [7,'named'=>9];}}var_dump(new C());"#, "object(C)#1 (2) {\n  [0]=>\n  int(7)\n  [\"named\"]=>\n  int(9)\n}\n");
}

/// Matches PHP for the mangled keys debug view.
#[test]
fn test_object_debug_info_rendering_mangled_keys() {
    assert_clean(r#"<?php class C {function __debugInfo():array {return ["\0*\0p"=>7,"\0Owner\0q"=>8];}}var_dump(new C());"#, "object(C)#1 (2) {\n  [\"p\":protected]=>\n  int(7)\n  [\"q\":\"Owner\":private]=>\n  int(8)\n}\n");
}

/// Matches PHP for the recursive call debug view.
#[test]
fn test_object_debug_info_rendering_recursive_call() {
    assert_clean(r#"<?php class C {function __debugInfo():array {echo 'hook|';var_dump($this);return ['self'=>$this];}}var_dump(new C());"#, "hook|*RECURSION*\nobject(C)#1 (1) {\n  [\"self\"]=>\n  *RECURSION*\n}\n");
}

/// Matches PHP for the once per dump debug view.
#[test]
fn test_object_debug_info_rendering_once_per_dump() {
    assert_clean(r#"<?php class C {public int $calls=0;function __debugInfo():array {$this->calls++;echo 'hook|';return ['calls'=>$this->calls];}}$s=new C();var_dump($s);var_dump($s);echo 'total:',$s->calls;"#, "hook|object(C)#1 (1) {\n  [\"calls\"]=>\n  int(1)\n}\nhook|object(C)#1 (1) {\n  [\"calls\"]=>\n  int(2)\n}\ntotal:2");
}

/// Matches PHP for the fresh owner debug view.
#[test]
fn test_object_debug_info_rendering_fresh_owner() {
    assert_clean(r#"<?php class D {function __destruct(){echo 'drop|';}}class C {function __debugInfo():array {echo 'hook|';return ['fresh'=>new D()];}}var_dump(new C());echo 'done';"#, "hook|object(C)#1 (1) {\n  [\"fresh\"]=>\n  object(D)#2 (0) {\n  }\ndrop|}\ndone");
}

/// Matches PHP for the borrowed owner debug view.
#[test]
fn test_object_debug_info_rendering_borrowed_owner() {
    assert_clean(r#"<?php class C {public array $data=['k'=>'before'];function __debugInfo():array {echo 'hook|';return $this->data;}}$s=new C();var_dump($s);$s->data['k']='after';echo $s->data['k'];"#, "hook|object(C)#1 (1) {\n  [\"k\"]=>\n  string(6) \"before\"\n}\nafter");
}

/// Matches PHP for the null empty debug view.
#[test]
fn test_object_debug_info_rendering_null_empty() {
    assert_clean(r#"<?php error_reporting(0);class C {public int $hidden=8;function __debugInfo(){echo 'hook|';return null;}}var_dump(new C());"#, "hook|object(C)#1 (0) {\n}\n");
}

/// Matches PHP for the print r debug view.
#[test]
fn test_object_debug_info_rendering_print_r() {
    assert_clean(r#"<?php class C {public int $unused=8;function __debugInfo():array {echo 'hook|';return ['x'=>3];}}print_r(new C());"#, "hook|C Object\n(\n    [x] => 3\n)\n");
}

/// Matches PHP for the print r return debug view.
#[test]
fn test_object_debug_info_rendering_print_r_return() {
    assert_clean(r#"<?php class C {public int $unused=8;function __debugInfo():array {echo 'hook|';return ['x'=>3];}}$text=print_r(new C(),true);echo 'returned|',$text;"#, "hook|returned|C Object\n(\n    [x] => 3\n)\n");
}

/// Releases a receiver removed by its own hook before rendering the captured header metadata.
#[test]
fn test_object_debug_info_rendering_removed_nested_receiver() {
    assert_clean(r#"<?php
class Child {public int $n=7;function __destruct(){echo 'drop|';}function __debugInfo():array {echo 'hook|';$parent=Hold::$parent;unset($parent->child);return ['n'=>$this->n];}}
class ParentNode {public Child $child;function __construct(){$this->child=new Child();}}
class Hold {public static ParentNode $parent;}
Hold::$parent=new ParentNode();var_dump(Hold::$parent);echo 'done';
"#, "object(ParentNode)#1 (1) {\n  [\"child\"]=>\n  hook|drop|object(Child)#2 (1) {\n    [\"n\"]=>\n    int(7)\n  }\n}\ndone");
}

/// Closes the object before propagating an exception from its temporary debug properties.
#[test]
fn test_object_debug_info_rendering_snapshot_destructor_throws_after_close() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'drop|';throw new Exception('dtor');}}
class C {function __debugInfo():array {echo 'hook|';return ['fresh'=>new D()];}}
try {var_dump(new C());} catch(Throwable $e) {echo 'caught:',$e->getMessage();}
echo '|done';
"#, "hook|object(C)#1 (1) {\n  [\"fresh\"]=>\n  object(D)#2 (0) {\n  }\ndrop|}\ncaught:dtor|done");
}

/// Renders the empty view before propagating an exception from the null-return deprecation.
#[test]
fn test_object_debug_info_rendering_null_handler_throws_after_close() {
    assert_clean(r#"<?php
class C {function __debugInfo(){echo 'hook|';return null;}}
set_error_handler(function($errno,$message){echo 'deprecated|';throw new Exception('notice');});
try {var_dump(new C());} catch(Throwable $e){echo 'caught:',$e->getMessage();}
echo '|done';
"#, "hook|deprecated|object(C)#2 (0) {\n}\ncaught:notice|done");
}

/// Matches the pinned PHP oracle for mixed indexed receiver.
#[test]
fn test_object_debug_info_rendering_mixed_indexed_receiver() {
    assert_clean(r#"<?php class C {function __debugInfo():array {echo 'hook|';return ['x'=>1];}} var_dump([new C(),7]);echo 'done';"#, "array(2) {\n  [0]=>\n  hook|object(C)#1 (1) {\n    [\"x\"]=>\n    int(1)\n  }\n  [1]=>\n  int(7)\n}\ndone");
}

/// Matches the pinned PHP oracle for anonymous mangled key.
#[test]
fn test_object_debug_info_rendering_anonymous_mangled_key() {
    assert_clean(r#"<?php class C {function __debugInfo():array {return ["\0Owner\0file\0q"=>8];}} var_dump(new C());"#, "object(C)#1 (1) {\n  [\"q\":\"Owner\":private]=>\n  int(8)\n}\n");
}

/// Matches the pinned PHP oracle for malformed mangled keys.
#[test]
fn test_object_debug_info_rendering_malformed_mangled_keys() {
    assert_clean(r#"<?php error_reporting(0);class C {function __debugInfo():array {return ["\0"=>1,"\0\0p"=>2,"\0Owner"=>3];}}var_dump(new C());"#, "object(C)#1 (3) {\n  [\"\0\"]=>\n  int(1)\n  [\"\0\0p\"]=>\n  int(2)\n  [\"\0Owner\"]=>\n  int(3)\n}\n");
}

/// Matches the pinned PHP oracle for byref view snapshot.
#[test]
fn test_object_debug_info_rendering_byref_view_snapshot() {
    assert_clean(r#"<?php class Child {function __debugInfo():array {$object=Holder::$object;$object->data=['later'=>99];return ['ok'=>1];}}class C {public array $data;function &__debugInfo():array {return $this->data;}}class Holder {public static C $object;}$object=new C();Holder::$object=$object;$object->data=['child'=>new Child(),'tail'=>7];var_dump($object);echo implode(',',array_keys($object->data));"#, "object(C)#1 (2) {\n  [\"child\"]=>\n  object(Child)#2 (1) {\n    [\"ok\"]=>\n    int(1)\n  }\n  [\"tail\"]=>\n  int(7)\n}\nlater");
}

/// Hidden func_args metadata remains internal and receives an empty public argument list.
#[test]
fn test_object_debug_info_rendering_argument_introspection() {
    let out = crate::support::compile_and_run_with_heap_debug(r#"<?php class D {public function __debugInfo():array {return ['argc'=>func_num_args(),'args'=>func_get_args()];}}var_dump(new D());
"#);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "object(D)#1 (2) {\n  [\"argc\"]=>\n  int(0)\n  [\"args\"]=>\n  array(0) {\n  }\n}\n", "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
/// Inherited hooks receive their actual receiver before the synthesized hidden collector.
#[test]
fn test_object_debug_info_rendering_inherited_argument_introspection() {
    assert_clean(r#"<?php
class D {protected int $value=17; public function __debugInfo():array {return ['value'=>$this->value,'argc'=>func_num_args(),'args'=>func_get_args()];}}
class E extends D {}
var_dump(new E());
"#, "object(E)#1 (3) {\n  [\"value\"]=>\n  int(17)\n  [\"argc\"]=>\n  int(0)\n  [\"args\"]=>\n  array(0) {\n  }\n}\n");
}

/// Captures every byte beyond the former fixed-buffer limit and retires the full result.
#[test]
fn test_object_debug_info_rendering_print_r_large_capture() {
    let output = crate::support::compile_and_run_with_heap_debug(r#"<?php $s=print_r(str_repeat('a',70003),true);echo $s,'|end';"#);
    assert!(output.success, "{}", output.stderr);
    assert_eq!(output.stdout, format!("{}|end", "a".repeat(70003)), "{}", output.stderr);
    assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
}
