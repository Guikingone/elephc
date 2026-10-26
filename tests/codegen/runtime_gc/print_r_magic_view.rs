//! Purpose:
//! Verifies native print_r magic-view rendering and temporary ownership.
//! Compares exact PHP 8.5.10 output across callback, recursion, key and destructor cases.
//!
//! Called from:
//! - The codegen integration harness through `runtime_gc`.
//!
//! Key details:
//! - Every success case also requires a clean native heap.
//! - Fatal diagnostics and generic Fiber/definite-assignment gaps remain separate acceptance gates.

/// Checks the complete output and lifetime balance of a native magic view.
fn assert_magic(source:&str,expected:&str) {
    let out=crate::support::compile_and_run_with_heap_debug(source);
    assert!(out.success,"{} {}",out.stdout,out.stderr);
    assert_eq!(out.stdout,expected,"{}",out.stderr);
    assert!(out.stderr.contains("leak summary: clean"),"{}",out.stderr);
}

/// Matches PHP for nested echo.
#[test]
fn test_print_r_magic_nested_echo(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo():array {print_r(['inner'=>2]);return ['x'=>3];}}$s=print_r(['before'=>1,'object'=>new C(),'after'=>4],true);echo 'returned|',$s;"#, "Array\n(\n    [inner] => 2\n)\nreturned|Array\n(\n    [before] => 1\n    [object] => C Object\n        (\n            [x] => 3\n        )\n\n    [after] => 4\n)\n");
}

/// Matches PHP for nested capture.
#[test]
fn test_print_r_magic_nested_capture(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo():array {$s=print_r(['inner'=>2],true);echo 'nested:',strlen($s),'|';return ['x'=>3];}}$s=print_r(['before'=>1,'object'=>new C(),'after'=>4],true);echo 'returned|',$s;"#, "nested:27|returned|Array\n(\n    [before] => 1\n    [object] => C Object\n        (\n            [x] => 3\n        )\n\n    [after] => 4\n)\n");
}

/// Matches PHP for null.
#[test]
fn test_print_r_magic_null(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo(){return null;}}print_r(new C());"#, "C Object\n(\n)\n");
}

/// Matches PHP for null handler throws.
#[test]
fn test_print_r_magic_null_handler_throws(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo(){return null;}}set_error_handler(function($n,$m){echo 'deprecated|';throw new Exception('notice');});try{print_r(new C());}catch(Throwable $e){echo 'caught:',$e->getMessage();}echo '|done';"#, "deprecated|C Object\n(\n)\ncaught:notice|done");
}

/// Matches PHP for recursive hook.
#[test]
fn test_print_r_magic_recursive_hook(){
    assert_magic(r#"<?php error_reporting(0);class C {public int $n=0;function __debugInfo():array {$this->n++;echo 'hook:',$this->n,'|';if($this->n===1){print_r($this);}return ['n'=>$this->n];}}print_r(new C());"#, "hook:1|hook:2|C Object\n(\n    [n] => 2\n)\nC Object\n(\n    [n] => 2\n)\n");
}

/// Matches PHP for recursive view.
#[test]
fn test_print_r_magic_recursive_view(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo():array {echo 'hook|';return ['self'=>$this];}}print_r(new C());"#, "hook|C Object\n(\n    [self] => C Object\n *RECURSION*\n)\n");
}

/// Matches PHP for sibling.
#[test]
fn test_print_r_magic_sibling(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo():array {echo 'hook|';return ['x'=>3];}}$c=new C();print_r([$c,$c]);"#, "hook|hook|Array\n(\n    [0] => C Object\n        (\n            [x] => 3\n        )\n\n    [1] => C Object\n        (\n            [x] => 3\n        )\n\n)\n");
}

/// Matches PHP for fresh destructor.
#[test]
fn test_print_r_magic_fresh_destructor(){
    assert_magic(r#"<?php error_reporting(0);class D {function __destruct(){echo 'drop|';}}class C {function __debugInfo():array {echo 'hook|';return ['fresh'=>new D()];}}print_r(new C());echo 'done';"#, "hook|drop|C Object\n(\n    [fresh] => D Object\n        (\n        )\n\n)\ndone");
}

/// Matches PHP for destructor throws.
#[test]
fn test_print_r_magic_destructor_throws(){
    assert_magic(r#"<?php error_reporting(0);class D {function __destruct(){echo 'drop|';throw new Exception('dtor');}}class C {function __debugInfo():array {return ['fresh'=>new D()];}}try{print_r(new C());}catch(Throwable $e){echo 'caught:',$e->getMessage();}echo '|done';"#, "drop|C Object\n(\n    [fresh] => D Object\n        (\n        )\n\n)\ncaught:dtor|done");
}

/// Matches PHP for removed receiver.
#[test]
fn test_print_r_magic_removed_receiver(){
    assert_magic(r#"<?php error_reporting(0);class H {public static array $a;}class C {function __debugInfo():array {H::$a=[];return ['x'=>3];}function __destruct(){echo 'drop|';}}H::$a=[new C()];print_r(H::$a);echo 'done';"#, "Array\n(\n    [0] => C Object\n        (\n            [x] => 3\n        )\n\n)\ndrop|done");
}

/// Matches PHP for mangled.
#[test]
fn test_print_r_magic_mangled(){
    assert_magic(r#"<?php error_reporting(0);class C {function __debugInfo():array {return [2=>7,"\0C\0q"=>8,"\0*\0p"=>9];}}print_r(new C());"#, "C Object\n(\n    [2] => 7\n    [q:C:private] => 8\n    [p:protected] => 9\n)\n");
}

/// Shares canonical key decoding with var_dump for protected prefix.
#[test]
fn test_print_r_magic_key_protected_prefix(){
    assert_magic(r#"<?php class C {function __debugInfo():array{return ["\0*Other\0p"=>3];}}var_dump(new C());print_r(new C());"#, "object(C)#1 (1) {\n  [\"p\":protected]=>\n  int(3)\n}\nC Object\n(\n    [p:protected] => 3\n)\n");
}

/// Shares canonical key decoding with var_dump for anonymous.
#[test]
fn test_print_r_magic_key_anonymous(){
    assert_magic(r#"<?php class C {function __debugInfo():array{return ["\0Owner\0file\0q"=>8];}}var_dump(new C());print_r(new C());"#, "object(C)#1 (1) {\n  [\"q\":\"Owner\":private]=>\n  int(8)\n}\nC Object\n(\n    [q:Owner:private] => 8\n)\n");
}

/// Shares canonical key decoding with var_dump for malformed handler.
#[test]
fn test_print_r_magic_key_malformed_handler(){
    assert_magic(r#"<?php class C {function __debugInfo():array{return ["\0"=>3];}}set_error_handler(function($n,$m){echo 'notice|';throw new Exception('key');});try{print_r(new C());}catch(Throwable $e){echo 'caught:',$e->getMessage();}"#, "notice|C Object\n(\n    [\0] => 3\n)\ncaught:key");
}

/// Suppressed fatal reports still terminate after the PHP-visible callback prefix.
#[test]
fn test_print_r_magic_fatal_exception(){
    let out=crate::support::compile_and_run_with_heap_debug(r#"<?php error_reporting(0);class C {function __debugInfo():array {echo 'hook|';throw new Exception('x');}}try{$s=print_r(['before'=>1,new C()],true);}catch(Throwable $e){echo 'caught:',$e->getMessage(),'|';}echo 'after|';print_r(['ok'=>1]);"#);
    assert!(!out.success);
    assert_eq!(out.stdout,"hook|");
    assert!(out.stderr.is_empty(),"{}",out.stderr);
}

/// Suppressed fatal reports still terminate after the PHP-visible callback prefix.
#[test]
fn test_print_r_magic_fatal_deferred_destructor(){
    let out=crate::support::compile_and_run_with_heap_debug(r#"<?php error_reporting(0);class D {function __destruct(){echo 'drop|';throw new Exception('dtor');}}class C {function __debugInfo():array {echo 'first|';return ['fresh'=>new D()];}}class E {function __debugInfo():array {echo 'second|';return ['ok'=>2];}}try{print_r([new C(),new E(),'tail'=>7]);}catch(Throwable $e){echo 'caught:',$e->getMessage();}echo '|done';"#);
    assert!(!out.success);
    assert_eq!(out.stdout,"first|drop|");
    assert!(out.stderr.is_empty(),"{}",out.stderr);
}
