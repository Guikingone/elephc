//! Purpose:
//! Regresses native ownership of captures belonging to opaque eval closures.
//!
//! Called from:
//! - The runtime GC codegen suite through its parent module registration.
//!
//! Key details:
//! - Runtime-unknown source exercises Magician rather than AOT closure lowering.
//! - PHP-visible destruction timing and heap balance are checked together.

use crate::support::*;

/// Runs one opaque fragment and verifies both observable output and final ownership balance.
fn assert_capture_owners(fragment: &str, expected: &str) {
    let quoted = fragment.replace('\\', "\\\\").replace('\'', "\\'");
    let source = format!("<?php $source = '{quoted}' . ' // ' . $argc; eval($source); unset($source);");
    let (out, assembly) = compile_and_run_with_heap_debug_and_asm(&source);
    assert!(assembly.contains("__elephc_eval_execute"), "fragment must remain opaque eval");
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// The last closure owner releases its captured object before the following statement.
#[test]
fn test_core_eval_closure_capture_owners_last_release() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) {}; unset($d, $f); echo 'done';"#, "drop|done");
}

/// A bound copy keeps both its receiver and shared captured values alive independently.
#[test]
fn test_core_eval_closure_capture_owners_bound_copy() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) { echo 'call|'; };
$g = $f->bindTo(new stdClass()); unset($d, $f); $g(); echo 'before|'; unset($g); echo 'done';"#,
        "call|before|drop|done");
}

/// Reflection keeps the original closure identity alive as well as its captured values.
#[test]
fn test_core_eval_closure_capture_owners_reflection_identity() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) { echo 'call|'; };
$weak = WeakReference::create($f); $r = new ReflectionFunction($f); unset($d, $f);
echo $weak->get() === null ? 'lost|' : 'held|'; $r->invoke(); unset($r);
echo $weak->get() === null ? 'gone|' : 'held|'; unset($weak); echo 'done';"#,
        "held|call|drop|gone|done");
}

/// A captured reference wrapper owns the payload after its defining variable is unset.
#[test]
fn test_core_eval_closure_capture_owners_reference() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use(&$d) {echo 'call|';}; unset($d); $f(); unset($f); echo 'done';"#,
        "call|drop|done");
}

/// Collector-owned capture cells are retired once when the object points back to its closure.
#[test]
fn test_core_eval_closure_capture_owners_cycle() {
    assert_capture_owners(r#"class D { public $f; function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) {}; $d->f = $f;
unset($d, $f); gc_collect_cycles(); echo 'done';"#, "drop|done");
}

/// Retiring one of two closures does not destroy their shared captured cell too early.
#[test]
fn test_core_eval_closure_capture_owners_shared_value() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) {echo 'call|';}; $g = function() use($d) {echo 'call|';};
unset($d, $f); $g(); echo 'before|'; unset($g); echo 'done';"#, "call|before|drop|done");
}

/// Releasing a handler restored explicitly retires its captured object immediately.
#[test]
fn test_core_eval_closure_capture_owners_restored_handler() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); set_error_handler(function() use($d) {}); unset($d);
restore_error_handler(); echo 'done';"#, "drop|done");
}

/// Reflection constructors release their owned name argument and raw allocation owner.
#[test]
fn test_core_eval_closure_capture_owners_named_reflection() {
    assert_capture_owners("function f() {} $r = new ReflectionFunction(strtolower('F')); unset($r); echo 'done';", "done");
}

/// Preserves closure identity, callability, and captured owners through clone.
#[test]
fn test_core_eval_closure_capture_owners_clone() {
    assert_capture_owners(r#"class D{function __destruct(){echo'drop|';}}$d=new D();$f=function()use($d){echo'call|';};$g=clone $f;unset($d,$f);$g();echo'before|';unset($g);echo'done';"#, "call|before|drop|done");
}

/// Preserves closure identity, callability, and captured owners through reflection get closure.
#[test]
fn test_core_eval_closure_capture_owners_reflection_get_closure() {
    assert_capture_owners(r#"class D{function __destruct(){echo'drop|';}}$d=new D();$f=function()use($d){echo'call|';};$r=new ReflectionFunction($f);$g=$r->getClosure();echo $g===$f?'same|':'different|';unset($d,$f,$r);$g();unset($g);echo'done';"#, "same|call|drop|done");
}

/// Reflection-created callables use PHP's case-insensitive function lookup after the owner is released.
#[test]
fn test_core_eval_closure_capture_owners_reflection_named_callable() {
    assert_capture_owners(r#"function F() {echo 'call|';} $r = new ReflectionFunction('F');
$f = $r->getClosure(); unset($r); $f(); unset($f); echo 'done';"#, "call|done");
}

/// Existing closures pass through fromCallable without losing their identity or captured owner.
#[test]
fn test_core_eval_closure_capture_owners_from_callable_identity() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) {echo 'call|';}; $g = Closure::fromCallable($f);
echo $g === $f ? 'same|' : 'different|'; $w = WeakReference::create($f); unset($d, $f);
echo $w->get() === null ? 'lost|' : 'held|'; $g(); unset($g, $w); echo 'done';"#,
        "same|held|call|drop|done");
}

/// The Closure __invoke array trampoline returns that same object and owns its temporary array read.
#[test]
fn test_core_eval_closure_capture_owners_from_callable_invoke_array() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $f = function() use($d) {echo 'call|';};
$g = Closure::fromCallable([$f, '__INVOKE']); echo $g === $f ? 'same|' : 'different|';
unset($d, $f); $g(); unset($g); echo 'done';"#, "same|call|drop|done");
}

/// Temporary closure arguments transfer one independent return owner to fromCallable's caller.
#[test]
fn test_core_eval_closure_capture_owners_from_callable_temporary() {
    assert_capture_owners(r#"class D { function __destruct(){echo 'drop|';} }
$d = new D(); $g = Closure::fromCallable(function() use($d) {echo 'call|';});
unset($d); $g(); unset($g); echo 'done';"#, "call|drop|done");
}
