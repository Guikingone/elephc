//! Purpose:
//! Verifies PHP handler return-value destruction, suspended registration state and argument ownership.
//!
//! Called from:
//! - The runtime GC codegen suite through explicit parent registration.
//!
//! Key details:
//! - The PHP return value dies before an old replaced handler, preserving exception-chain order.
//! - Native trigger_error argument containers are consumed even when callback cleanup throws.

use crate::support::compile_and_run_with_heap_debug;

/// Requires complete PHP output and no residual native argument or result owners.
fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Preserves PHP's order when both return-value destruction and old-handler destruction throw.
#[test]
fn test_handler_callback_result_before_replaced_owner() {
    assert_clean(r#"<?php
class OldOwner {function __destruct(){echo 'old|';throw new Exception('old');}}
class Returned {function __destruct(){echo 'returned|';throw new Exception('returned');}}
$old=new OldOwner();
set_error_handler(function($level,$message) use($old){set_error_handler(function(){return true;});return new Returned();});
unset($old);
try {trigger_error('test');} catch(Throwable $e){echo 'caught:',$e->getMessage(),'|';$p=$e->getPrevious();if($p!==null){echo 'previous:',$p->getMessage(),'|';}}
restore_error_handler();restore_error_handler();unset($e,$p);echo 'done';
"#, "returned|old|caught:old|previous:returned|done");
}

/// Leaves the original handler suspended while destroying its ignored object return.
#[test]
fn test_handler_callback_result_destructor_sees_suspended_registration() {
    assert_clean(r#"<?php
error_reporting(0);
class R {function __destruct(){echo 'returned|';trigger_error('nested');}}
set_error_handler(function(){echo 'handler|';return new R();});
trigger_error('outer');restore_error_handler();echo 'done';
"#, "handler|returned|done");
}

/// Retires the internally built argument array before a callback exception reaches the PHP catch.
#[test]
fn test_handler_callback_direct_throw_releases_arguments() {
    assert_clean(r#"<?php
set_error_handler(function(){throw new Exception('body');});
try {trigger_error('test');} catch(Throwable $e){echo 'caught:',$e->getMessage(),'|';}
restore_error_handler();unset($e);echo 'done';
"#, "caught:body|done");
}

/// Preserves the exact-false fallthrough rule after normalizing the ignored callback result.
#[test]
fn test_handler_callback_false_falls_through() {
    assert_clean(r#"<?php
error_reporting(0);set_error_handler(function(){echo 'handler|';return false;});
var_dump(trigger_error('test'));restore_error_handler();echo 'done';
"#, "handler|bool(true)\ndone");
}

/// Other false-like PHP values still suppress the default diagnostic.
#[test]
fn test_handler_callback_non_boolean_false_like_results_are_handled() {
    let out = compile_and_run_with_heap_debug(r#"<?php
set_error_handler(function(){echo 'h|';return 0;});trigger_error('zero-must-be-handled');restore_error_handler();
set_error_handler(function(){echo 'h|';return null;});trigger_error('null-must-be-handled');restore_error_handler();
set_error_handler(function(){echo 'h|';return '';});trigger_error('empty-must-be-handled');restore_error_handler();
set_error_handler(function(){echo 'h|';return [];});trigger_error('array-must-be-handled');restore_error_handler();echo 'done';
"#);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, "h|h|h|h|done");
    assert!(!out.stderr.contains("must-be-handled"), "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
