//! Purpose:
//! Checks print_r capture isolation and argument lifetime across native PHP callbacks.
//! Pins full output and heap cleanliness against the PHP 8.5.10 oracle.
//!
//! Called from:
//! - The codegen integration test harness through `runtime_gc`.
//!
//! Key details:
//! - Output handlers can reenter print_r or throw while an outer result stays owned.
//! - Resource formatting and argument destruction share the same scoped entry.

/// Executes a native fixture and verifies both visible bytes and complete retirement.
fn assert_print_r_scope(source: &str, expected: &str) {
    let output = crate::support::compile_and_run_with_heap_debug(source);
    assert!(output.success, "{} {}", output.stdout, output.stderr);
    assert_eq!(output.stdout, expected, "{}", output.stderr);
    assert!(output.stderr.contains("leak summary: clean"), "{}", output.stderr);
}

/// Matches the PHP oracle for output handler reentry.
#[test]
fn test_print_r_scope_output_handler_reentry() {
    assert_print_r_scope(r#"<?php ob_start(function($b){$inner=print_r(['n'=>2],true);return 'h:'.strlen($inner).'|'.$b;},1);print_r('abc');ob_end_flush();"#, "h:23|abch:23|");
}

/// Matches the PHP oracle for output handler throw.
#[test]
fn test_print_r_scope_output_handler_throw() {
    assert_print_r_scope(r#"<?php ob_start(function($b){throw new Exception('handler');},1);try{print_r(['x'=>1]);}catch(Throwable $e){echo 'caught:',$e->getMessage(),'|';}ob_end_clean();echo 'after';"#, "Array\n(\n    [x] => 1\n)\ncaught:handler|after");
}

/// Matches the PHP oracle for resource.
#[test]
fn test_print_r_scope_resource() {
    assert_print_r_scope(r#"<?php $r=fopen('php://memory','w');echo print_r($r,true),'|';print_r(['r'=>$r]);fclose($r);"#, "Resource id #5|Array\n(\n    [r] => Resource id #5\n)\n");
}

/// Matches the PHP oracle for argument destructor.
#[test]
fn test_print_r_scope_argument_destructor() {
    assert_print_r_scope(r#"<?php class D {function __destruct(){echo 'drop|';}}print_r(new D());echo 'done';"#, "D Object\n(\n)\ndrop|done");
}

/// Source mutation during output keeps the original mixed argument alive through emission.
#[test]
fn test_print_r_scope_mixed_source_mutation() {
    assert_print_r_scope(r#"<?php class H {public static mixed $value;}class D {public int $x=7;function __destruct(){echo 'drop|';}}H::$value=new D();ob_start(function($text){H::$value=null;return $text;},1);print_r(H::$value);ob_end_flush();echo 'done';"#, "D Object\n(\n    [x] => 7\n)\ndrop|done");
}

/// Native buffers preserve their hexadecimal address in echo, return and dynamic-flag modes.
#[test]
fn test_print_r_scope_native_buffer_address_format() {
    assert_print_r_scope(r#"<?php $value=buffer_new<int>(4);ob_start();print_r($value);$echoed=ob_get_clean();$returned=print_r($value,true);$dynamic=print_r($value,$argc>0);$digits=substr($returned,2);$valid=strlen($digits)>0;for($i=0;$i<strlen($digits);$i++){$byte=ord($digits[$i]);if(!(($byte>=48&&$byte<=57)||($byte>=65&&$byte<=70)||($byte>=97&&$byte<=102))){$valid=false;}}var_dump($echoed===$returned,$dynamic===$returned,substr($returned,0,2)==='0x',$valid);buffer_free($value);"#, "bool(true)\nbool(true)\nbool(true)\nbool(true)\n");
}
