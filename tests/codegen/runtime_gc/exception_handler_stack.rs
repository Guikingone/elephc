//! Purpose:
//! Verifies PHP exception-handler suspension stacks and call-duration owner leases.
//!
//! Called from:
//! - The runtime GC codegen suite through explicit parent registration.
//!
//! Key details:
//! - Exception invocation pushes its original registration, unlike ordinary error invocation.
//! - Callback restores and replacements preserve both history and independently retained owners.

use crate::support::compile_and_run_with_heap_debug;

/// Requires the complete PHP output and retirement of every native owner.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Matches the pinned PHP suspension and lifetime oracle for restore inside.
#[test]
fn test_exception_handler_stack_restore_inside() {
    assert_clean(r#"<?php
class D {function __destruct(){throw new Exception('shutdown');}}
$d=new D();set_error_handler(function() use($d){});unset($d);
set_exception_handler(function($e){echo 'before:',is_null(get_exception_handler())?'none|':'set|';restore_exception_handler();echo 'after:',is_null(get_exception_handler())?'none|':'set|';});
echo 'body|';
"#, "body|before:none|after:set|");
}

/// Matches the pinned PHP suspension and lifetime oracle for replacement history.
#[test]
fn test_exception_handler_stack_replacement_history() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){throw new Exception($this->name);}}
function first($e){echo 'first:',$e->getMessage(),'|';set_exception_handler('second');}
function second($e){echo 'second:',$e->getMessage(),'|';restore_exception_handler();restore_exception_handler();restore_exception_handler();echo get_exception_handler()==='first'?'restored:first|':'bad|';}
$a=new D('A');set_error_handler(function() use($a){});unset($a);
$b=new D('B');set_error_handler(function() use($b){});unset($b);
set_exception_handler('first');echo 'body|';
"#, "body|first:A|second:B|restored:first|");
}

/// Matches the pinned PHP suspension and lifetime oracle for lease before return.
#[test]
fn test_exception_handler_stack_lease_before_return() {
    assert_clean(r#"<?php
class D {function __destruct(){throw new Exception('shutdown');}}
class O {function __destruct(){echo 'owner|';}}
class R {function __destruct(){echo 'returned|';}}
$d=new D();set_error_handler(function() use($d){});unset($d);
$o=new O();set_exception_handler(function($e) use($o){restore_exception_handler();restore_exception_handler();return new R();});unset($o);
echo 'body|';
"#, "body|owner|returned|");
}

/// Matches the pinned PHP suspension and lifetime oracle for replacement keeps capture.
#[test]
fn test_exception_handler_stack_replacement_keeps_capture() {
    assert_clean(r#"<?php
class D {function __construct(public string $name){}function __destruct(){throw new Exception($this->name);}}
class O {function __destruct(){echo 'owner|';}}
$a=new D('A');set_error_handler(function() use($a){});unset($a);
$b=new D('B');set_error_handler(function() use($b){});unset($b);
$owner=new O();$weak=WeakReference::create($owner);
set_exception_handler(function($e) use($owner,$weak){echo 'A|';set_exception_handler(function($e) use($weak){echo 'B:',$weak->get()===null?'dead|':'alive|';});});
unset($owner,$weak);echo 'body|';
"#, "body|A|B:alive|owner|");
}

/// Matches the pinned PHP suspension and lifetime oracle for error stack unchanged.
#[test]
fn test_exception_handler_stack_error_stack_unchanged() {
    assert_clean(r#"<?php
set_error_handler(function(){echo 'before:',is_null(get_error_handler())?'none|':'set|';restore_error_handler();echo 'after:',is_null(get_error_handler())?'none|':'set|';return true;});
trigger_error('test');restore_error_handler();echo 'done';
"#, "before:none|after:none|done");
}
