//! Purpose:
//! Regressions for object-backed ArrayObject and ArrayIterator subclasses.
//!
//! Called from:
//! - The SPL codegen test module.
//!
//! Key details:
//! - Indirect runtime property mutations must reserve the backing object's boxed storage.

use crate::support::compile_and_run_with_heap_debug;

/// A derived ArrayObject removes backing properties without reading freed scalar storage.
#[test]
fn test_spl_storage_subclass_array_object_unset_backing_property() {
    let out = compile_and_run_with_heap_debug(r#"<?php
set_error_handler(function(int $severity, string $message): bool {
    if ($severity === E_WARNING) { echo $message, "\n"; }
    return true;
});
class C { public $prop = 'C::prop.orig'; }
class MyArrayObject extends ArrayObject {}
$c = new C;
$ao = new MyArrayObject($c);
foreach ($ao as $key => $value) {}
unset($ao['prop']);
var_dump($c->prop, $ao['prop']);
restore_error_handler();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "Undefined property: C::$prop\nUndefined array key \"prop\"\nNULL\nNULL\n");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "backing-property scenario leaked: {}",
        out.stderr
    );
}

/// ArrayIterator descendants share the same boxed backing-property mutation requirement.
#[test]
fn test_spl_storage_subclass_array_iterator_unset_and_replace_backing_property() {
    let out = compile_and_run_with_heap_debug(r#"<?php
set_error_handler(function(int $severity, string $message): bool {
    if ($severity === E_WARNING) { echo $message, "\n"; }
    return true;
});
class C { public $prop = 'original'; }
class MyArrayIterator extends ArrayIterator {}
$c = new C;
$iterator = new MyArrayIterator($c);
unset($iterator['prop']);
var_dump($c->prop);
$iterator['prop'] = 42;
var_dump($c->prop);
restore_error_handler();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "Undefined property: C::$prop\nNULL\nint(42)\n");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "backing-property scenario leaked: {}",
        out.stderr
    );
}

/// Replacing a descendant's array store with an object reserves its mutable property layout.
#[test]
fn test_spl_storage_subclass_exchange_array_unset_backing_property() {
    let out = compile_and_run_with_heap_debug(r#"<?php
set_error_handler(function(int $severity, string $message): bool {
    if ($severity === E_WARNING) { echo $message, "\n"; }
    return true;
});
class C { public $prop = 'original'; }
class MyArrayObject extends ArrayObject {}
$c = new C;
$ao = new MyArrayObject([]);
$ao->exchangeArray($c);
unset($ao['prop']);
var_dump($c->prop);
$ao['prop'] = 42;
var_dump($c->prop);
restore_error_handler();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "Undefined property: C::$prop\nNULL\nint(42)\n");
    assert!(
        out.stderr.contains("leak summary: clean"),
        "backing-property scenario leaked: {}",
        out.stderr
    );
}

/// Internal SPL deprecations use PHP's engine category even when reporting is suppressed.
#[test]
fn test_spl_storage_internal_deprecation_handler_severity() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
set_error_handler(function(int $severity, string $message): bool {
    echo $severity, "\n";
    return true;
}, E_DEPRECATED);
new ArrayObject(new stdClass());
new ArrayIterator(new stdClass());
$ao = new ArrayObject([]);
$ao->exchangeArray(new stdClass());
restore_error_handler();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "8192\n8192\n8192\n");
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// The reporting mask selects the engine deprecation category rather than the user category.
#[test]
fn test_spl_storage_internal_deprecation_reporting_mask() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(E_DEPRECATED);
new ArrayObject(new stdClass());
echo "done\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert!(out.stderr.contains("Deprecated: ArrayObject::__construct():"), "{}", out.stderr);
    assert!(out.stdout.ends_with("done\n"), "{}", out.stdout);
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Public trigger_error retains PHP's refusal of engine-only diagnostic levels.
#[test]
fn test_spl_storage_internal_deprecation_preserves_trigger_error_validation() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
try { trigger_error('user', E_DEPRECATED); }
catch (ValueError $error) { echo get_class($error), "\n"; }
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "ValueError\n");
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Throwing deprecation handlers retire the fresh receiver and backing argument owners.
#[test]
fn test_spl_storage_internal_deprecation_handler_throw_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(r#"<?php
error_reporting(0);
set_error_handler(function(int $severity, string $message): bool {
    throw new Exception('stop');
}, E_DEPRECATED);
try { new ArrayObject(new stdClass()); }
catch (Exception $error) { echo $error->getMessage(), "\n"; }
restore_error_handler();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "stop\n");
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
