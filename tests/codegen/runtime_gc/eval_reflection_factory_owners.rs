//! Purpose:
//! Checks native producer ownership for Reflection objects created by eval.
//!
//! Called from:
//! - The runtime GC codegen integration suite.
//!
//! Key details:
//! - The result cell adopts the factory allocation while metadata arguments stay borrowed.
//! - Nested method, parameter and type owners must remain usable and retire together.

use crate::support::compile_and_run_with_heap_debug;

fn assert_clean(source: &str, expected: &str) {
    let out = compile_and_run_with_heap_debug(source);
    assert!(out.success, "stdout={:?}\nstderr={}", out.stdout, out.stderr);
    assert_eq!(out.stdout, expected, "{}", out.stderr);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

#[test]
fn test_eval_reflection_factory_owners_class() {
    assert_clean(r#"<?php
eval('class EvalFactoryClass {function ping(int $argument):int{return $argument;}} $r = new ReflectionClass(EvalFactoryClass::class); return $r;');
echo $r->getName();unset($r);
"#, "EvalFactoryClass");
}

#[test]
fn test_eval_reflection_factory_owners_method() {
    assert_clean(r#"<?php
eval('class EvalFactoryMethod {function execute(int $argument):int{return $argument;}} $r = new ReflectionMethod(EvalFactoryMethod::class, "execute"); return $r;');
echo $r->getName();unset($r);
"#, "execute");
}
