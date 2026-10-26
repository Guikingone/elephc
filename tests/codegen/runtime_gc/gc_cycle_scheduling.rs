//! Purpose:
//! Covers final collection of cycles whose PHP-visible owners are already retired.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - Final collection must run even when no local remains or automatic GC is disabled.
//! - The eval context remains alive until cyclic destructors finish their callbacks.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "{} {}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_gc_cycle_scheduling_shutdown_after_overwrite() {
    assert_clean(r#"<?php
class N {public mixed $self=null;function __destruct(){echo 'drop|';}}
$n=new N();$n->self=$n;$n=null;echo 'done|';
"#, "done|drop|");
}

#[test]
fn test_gc_cycle_scheduling_shutdown_after_function_return() {
    assert_clean(r#"<?php
class N {public mixed $self=null;function __destruct(){echo 'drop|';}}
function make():void{$n=new N();$n->self=$n;}
make();echo 'done|';
"#, "done|drop|");
}

#[test]
fn test_gc_cycle_scheduling_shutdown_with_automatic_gc_disabled() {
    assert_clean(r#"<?php
class N {public mixed $self=null;function __destruct(){echo 'drop|';}}
gc_disable();$n=new N();$n->self=$n;unset($n);echo 'done|';
"#, "done|drop|");
}

#[test]
fn test_gc_cycle_scheduling_acyclic_release_stays_immediate() {
    assert_clean(r#"<?php
class N {function __destruct(){echo 'drop|';}}
$n=new N();unset($n);echo 'done|';
"#, "drop|done|");
}

#[test]
fn test_gc_cycle_scheduling_explicit_collection_while_disabled() {
    assert_clean(r#"<?php
class N {public mixed $self=null;function __destruct(){echo 'drop|';}}
gc_disable();$n=new N();$n->self=$n;unset($n);echo 'before|';
gc_collect_cycles();echo 'after|';
"#, "before|drop|after|");
}

#[test]
fn test_gc_cycle_scheduling_shutdown_destructor_keeps_eval_context() {
    assert_clean(r#"<?php
class N {
 public mixed $self=null;
 function __destruct(){
  $source=getenv('PATH')===false?'echo "missing|";':'echo "drop|";';
  eval($source);
 }
}
$boot=getenv('PATH')===false?'echo "missing|";':'echo "ready|";';
eval($boot);$n=new N();$n->self=$n;$n=null;echo 'done|';
"#, "ready|done|drop|");
}

#[test]
fn test_gc_cycle_scheduling_explicit_main_return() {
    assert_clean(r#"<?php
class N {public mixed $self=null;function __destruct(){echo 'drop|';}}
$n=new N();$n->self=$n;echo 'body|';return;
"#, "body|drop|");
}
