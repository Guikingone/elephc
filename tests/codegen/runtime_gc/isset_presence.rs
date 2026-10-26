//! Purpose:
//! Regression coverage for typed locals whose assignments may throw before binding.
//! Separates PHP presence from empty, zero and false values.
//!
//! Called from:
//! - The codegen integration harness through runtime_gc.
//!
//! Key details:
//! - Every fixture requires exact PHP output and a clean native heap.

/// A typed throwing RHS must not make a new string binding visible to isset.
#[test]
fn test_isset_presence_throwing_string_assignment() {
    let out=crate::support::compile_and_run_with_heap_debug(r#"<?php function failing():string{throw new Exception('x');}try{$value=failing();}catch(Throwable $e){echo 'caught|';}echo isset($value)?'set':'unset';"#);
    assert!(out.success,"{}",out.stderr);
    assert_eq!(out.stdout,"caught|unset","{}",out.stderr);
    assert!(out.stderr.contains("leak summary: clean"),"{}",out.stderr);
}

/// Compares all visible values and ownership balance for a local presence fixture.
fn assert_presence(source:&str,expected:&str) {
    let out=crate::support::compile_and_run_with_heap_debug(source);
    assert!(out.success,"{} {}",out.stdout,out.stderr);
    assert_eq!(out.stdout,expected,"{}",out.stderr);
    assert!(out.stderr.contains("leak summary: clean"),"{}",out.stderr);
}

/// Preserves PHP presence for throw int.
#[test]
fn test_isset_presence_throw_int(){
    assert_presence(r#"<?php function failing():int{throw new Exception('x');}try{$value=failing();}catch(Throwable $e){}var_dump(isset($value));"#, "bool(false)\n");
}

/// Preserves PHP presence for throw bool.
#[test]
fn test_isset_presence_throw_bool(){
    assert_presence(r#"<?php function failing():bool{throw new Exception('x');}try{$value=failing();}catch(Throwable $e){}var_dump(isset($value));"#, "bool(false)\n");
}

/// Preserves PHP presence for throw float.
#[test]
fn test_isset_presence_throw_float(){
    assert_presence(r#"<?php function failing():float{throw new Exception('x');}try{$value=failing();}catch(Throwable $e){}var_dump(isset($value));"#, "bool(false)\n");
}

/// Preserves PHP presence for throw array.
#[test]
fn test_isset_presence_throw_array(){
    assert_presence(r#"<?php function failing():array{throw new Exception('x');}try{$value=failing();}catch(Throwable $e){}var_dump(isset($value));"#, "bool(false)\n");
}

/// Preserves PHP presence for previous value.
#[test]
fn test_isset_presence_previous_value(){
    assert_presence(r#"<?php function failing():string{throw new Exception('x');}$value='old';try{$value=failing();}catch(Throwable $e){}echo isset($value)?$value:'unset';"#, "old");
}

/// Preserves PHP presence for success empty zero.
#[test]
fn test_isset_presence_success_empty_zero(){
    assert_presence(r#"<?php $s='';$i=0;$b=false;$f=0.0;var_dump(isset($s,$i,$b,$f));"#, "bool(true)\n");
}

/// Preserves PHP presence for null unset.
#[test]
fn test_isset_presence_null_unset(){
    assert_presence(r#"<?php $x=null;var_dump(isset($x));$x=0;var_dump(isset($x));unset($x);var_dump(isset($x));"#, "bool(false)\nbool(true)\nbool(false)\n");
}

/// Preserves PHP presence for conditional.
#[test]
fn test_isset_presence_conditional(){
    assert_presence(r#"<?php if($argc>1){$x=7;}var_dump(isset($x));"#, "bool(false)\n");
}

/// Preserves PHP presence for parameters.
#[test]
fn test_isset_presence_parameters(){
    assert_presence(r#"<?php function f(string $s,int $i,bool $b){var_dump(isset($s,$i,$b));unset($s);var_dump(isset($s));}f('',0,false);"#, "bool(true)\nbool(false)\n");
}

/// Preserves PHP presence for parameter rebind throw.
#[test]
fn test_isset_presence_parameter_rebind_throw(){
    assert_presence(r#"<?php function fail_string():string{throw new Exception('x');}function test(string $s){unset($s);try{$s=fail_string();}catch(Throwable $e){}var_dump(isset($s));}test('old');"#, "bool(false)\n");
}

/// Preserves PHP presence for function conditional.
#[test]
fn test_isset_presence_function_conditional(){
    assert_presence(r#"<?php function test(bool $set){if($set){$x=0;}var_dump(isset($x));}test(false);test(true);"#, "bool(false)\nbool(true)\n");
}

/// Preserves PHP presence for string append.
#[test]
fn test_isset_presence_string_append(){
    assert_presence(r#"<?php error_reporting(0);$x.='ok';var_dump(isset($x));"#, "bool(true)\n");
}

/// Preserves PHP presence for foreach binding.
#[test]
fn test_isset_presence_foreach_binding(){
    assert_presence(r#"<?php foreach([0,false,''] as $x){var_dump(isset($x));}"#, "bool(true)\nbool(true)\nbool(true)\n");
}

/// Preserves PHP presence for lazy later argument.
#[test]
fn test_isset_presence_lazy_later_argument(){
    assert_presence(r#"<?php error_reporting(0);function trace_key(){echo 'bad';return 0;}$a=[1];var_dump(isset($missing,$a[trace_key()]));"#, "bool(false)\n");
}

/// Preserves PHP presence for loop after unset.
#[test]
fn test_isset_presence_loop_after_unset(){
    assert_presence(r#"<?php $x=null;unset($x);for($i=0;$i<3;$i++){var_dump(isset($x));$x=$i;}"#, "bool(false)\nbool(true)\nbool(true)\n");
}

/// Preserves PHP presence for ref existing alias.
#[test]
fn test_isset_presence_ref_existing_alias(){
    assert_presence(r#"<?php $a=null;$b=&$a;var_dump(isset($a),isset($b));$b=0;var_dump(isset($a),isset($b));unset($a);var_dump(isset($a),isset($b));"#, "bool(false)\nbool(false)\nbool(true)\nbool(true)\nbool(false)\nbool(true)\n");
}

/// Preserves PHP presence for ref existing write throw.
#[test]
fn test_isset_presence_ref_existing_write_throw(){
    assert_presence(r#"<?php function write_then_throw(&$value){$value=0;throw new Exception('x');}$x=null;try{write_then_throw($x);}catch(Throwable $e){}var_dump(isset($x));"#, "bool(true)\n");
}

/// A forward probe must observe each loop iteration's executed binding writes.
#[test]
fn test_isset_presence_loop_forward_probe() {
    assert_presence("<?php for($i=0;$i<3;$i++){var_dump(isset($x));$x=$i;}", "bool(false)\nbool(true)\nbool(true)\n");
}
