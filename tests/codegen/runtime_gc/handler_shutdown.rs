//! Purpose:
//! Verifies native CLI handler-root retirement against PHP shutdown output and heap ownership.
//!
//! Called from:
//! - The runtime GC codegen suite through its explicit parent module registration.
//!
//! Key details:
//! - Captured objects outlive ordinary locals and keep active handlers available to destructors.
//! - Destruction follows live PHP handles rather than registry stack order.

use crate::support::{compile_and_run_with_heap_debug, compile_and_run_with_heap_size};

/// Requires the complete PHP-observable output and a fully retired native heap.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Retires a registration without requiring an explicit restore from the PHP program.
#[test]
fn test_handler_shutdown_error_registration() {
    assert_clean("<?php set_error_handler(function($level,$message){});echo 'done';", "done");
}

/// Retires both handler kinds and their differently shaped previous-registration nodes.
#[test]
fn test_handler_shutdown_both_registrations() {
    assert_clean("<?php set_error_handler(function($level,$message){});set_exception_handler(function($e){});echo 'done';", "done");
}

/// Runs a captured destructor after the ordinary main local has been destroyed.
#[test]
fn test_handler_shutdown_capture_after_local() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){echo $this->name,'|';}}
$d=new D('handler');set_error_handler(function() use($d){});unset($d);
$local=new D('local');echo 'body|';
"#, "body|local|handler|");
}

/// Visits the oldest remaining object before an object retained by the current handler.
#[test]
fn test_handler_shutdown_nested_captures() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){echo $this->name,'|';}}
$a=new D('first');set_error_handler(function() use($a){});unset($a);
$b=new D('second');set_error_handler(function() use($b){});unset($b);echo 'body|';
"#, "body|first|second|");
}

/// Preserves object order when it is the reverse of handler registration order.
#[test]
fn test_handler_shutdown_reversed_capture_creation() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){echo $this->name,'|';}}
$b=new D('created-first');$a=new D('created-second');
set_error_handler(function() use($a){});unset($a);
set_error_handler(function() use($b){});unset($b);echo 'body|';
"#, "body|created-first|created-second|");
}

/// Leaves the active error callback available while an ordinary local destructor emits a warning.
#[test]
fn test_handler_shutdown_callback_available_to_destructors() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'handler|';}}
class E {function __destruct(){trigger_error('shutdown');}}
$d=new D();set_error_handler(function() use($d){echo 'handled|';});unset($d);
$local=new E();echo 'body|';
"#, "body|handled|handler|");
}

/// Uses the same shutdown path for an early top-level return.
#[test]
fn test_handler_shutdown_top_level_return() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'handler|';}}
$d=new D();set_error_handler(function() use($d){});unset($d);
echo 'body|';return;echo 'unreachable';
"#, "body|handler|");
}

/// Does not run an explicitly restored handler's capture destructor twice.
#[test]
fn test_handler_shutdown_explicit_restore() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'released|';}}
$d=new D();set_error_handler(function() use($d){});unset($d);
restore_error_handler();echo 'done';
"#, "released|done");
}

/// Allows the current destructor to restore its own captured registration.
#[test]
fn test_handler_shutdown_self_restore() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'drop|';restore_error_handler();}}
$d=new D();set_error_handler(function() use($d){});unset($d);echo 'body|';
"#, "body|drop|");
}

/// Does not pin later objects whose handler is removed by the current destructor.
#[test]
fn test_handler_shutdown_drops_future_capture() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){echo $this->name,'-start|';if($this->name==='A'){restore_error_handler();}echo $this->name,'-end|';}}
$a=new D('A');set_error_handler(function() use($a){});unset($a);
$b=new D('B');set_error_handler(function() use($b){});unset($b);echo 'body|';
"#, "body|A-start|B-start|B-end|A-end|");
}

/// Collects a captured self-cycle after its destructor has run exactly once.
#[test]
fn test_handler_shutdown_captured_cycle() {
    assert_clean(r#"<?php
class D {public mixed $self=null;function __destruct(){echo 'drop|';}}
$d=new D();$d->self=$d;set_error_handler(function() use($d){});unset($d);echo 'body|';
"#, "body|drop|");
}

/// Dispatches a destructor exception before retiring its active exception handler.
#[test]
fn test_handler_shutdown_destructor_exception_handler() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'drop|';throw new Exception('shutdown');}}
$d=new D();set_error_handler(function() use($d){});unset($d);
set_exception_handler(function($e){echo 'caught:',$e->getMessage(),'|';});echo 'body|';
"#, "body|drop|caught:shutdown|");
}

/// Handles each destructor exception before visiting the next object.
#[test]
fn test_handler_shutdown_separate_destructor_exceptions() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){echo $this->name,'|';throw new Exception($this->name);}}
$a=new D('A');set_error_handler(function() use($a){});unset($a);
$b=new D('B');set_error_handler(function() use($b){});unset($b);
set_exception_handler(function($e){echo 'caught:',$e->getMessage(),'|';});echo 'body|';
"#, "body|A|caught:A|B|caught:B|");
}

/// Keeps the current receiver alive while its exception handler removes the registry owner.
#[test]
fn test_handler_shutdown_exception_receiver_lease() {
    assert_clean(r#"<?php
class D {function __destruct(){echo 'drop|';throw new Exception('shutdown');}}
$d=new D();$weak=WeakReference::create($d);
set_error_handler(function() use($d){});unset($d);
set_exception_handler(function($e) use($weak){restore_error_handler();echo $weak->get()===null?'dead|':'alive|';});
echo 'body|';
"#, "body|drop|alive|");
}

/// Retires static values created by the handler destructor phase.
#[test]
fn test_handler_shutdown_destructor_writes_static() {
    assert_clean(r#"<?php
class Holder {public static mixed $value=null;}
class Late {function __destruct(){echo 'late|';}}
class D {function __destruct(){echo 'handler|';Holder::$value=new Late();}}
$d=new D();set_error_handler(function() use($d){});unset($d);echo 'body|';
"#, "body|handler|late|");
}

/// Forces inverse-table growth while constructor integer, string and floating arguments remain live.
#[test]
fn test_handler_shutdown_growth_without_handle_reuse() {
    let out = compile_and_run_with_heap_size(r#"<?php
class P {function __construct(public float $value, public int $index, public string $text){}}
class D {
    function __construct(public int $n){}
    function __destruct(){
        $value=$this->n+0.25;$last=0;
        for($i=0;$i<$this->n;$i++){
            $p=new P($value,$i,'kept');$last=spl_object_id($p);
            if($p->value!==$value||$p->index!==$i||$p->text!=='kept'){echo 'corrupt|';}
            unset($p);
        }
        echo 'grown:',$last,':',$value,'|';
    }
}
$d=new D($argc+2000);set_error_handler(function() use($d){});unset($d);echo 'body|';
"#, 8192);
    assert_eq!(out, "body|grown:2003:2001.25|");
}
