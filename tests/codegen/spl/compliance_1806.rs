//! Purpose:
//! Differential regressions for SPL compliance and its runtime diagnostic prerequisites.
//!
//! Called from:
//! - `cargo test --test codegen_tests compliance_1806`.
//!
//! Key details:
//! - Inputs cover Zend's canonical integer keys, refused-call evaluation and missing returns.

use crate::support::*;

/// Canonical integer string keys preserve sign and reject malformed or overflowing magnitudes.
#[test]
fn test_compliance_1806_fixed_array_canonical_numeric_keys() {
    let out = compile_and_run(
        r#"<?php
$a = new SplFixedArray(20);
$a[0] = 99;
foreach (["12", "0", "+12", "-0", "01", "-01", "+", "-", "", "1.0", "1e1", " 12", "12 ", "9223372036854775807", "9223372036854775808", "18446744073709551616", "-9223372036854775808", "-9223372036854775809"] as $key) {
    try { $a[$key] = 7; echo "ok\n"; }
    catch (TypeError $e) { echo "type\n"; }
    catch (OutOfBoundsException $e) { echo "range\n"; }
}
var_dump($a[12], $a[0]);
"#,
    );
    assert_eq!(out, "ok\nok\ntype\ntype\ntype\ntype\ntype\ntype\ntype\ntype\ntype\ntype\ntype\nrange\ntype\ntype\nrange\ntype\nint(7)\nint(7)\n");
}

/// Oversized textual offsets cannot wrap to slot zero for read, write, or unset.
#[test]
fn test_compliance_1806_fixed_array_overflow_never_aliases_zero() {
    let out = compile_and_run(
        r#"<?php
$a = new SplFixedArray(2);
$a[0] = 42;
$key = "18446744073709551616";
try { var_dump($a[$key]); } catch (TypeError $e) { echo "read\n"; }
try { $a[$key] = 7; } catch (TypeError $e) { echo "write\n"; }
try { unset($a[$key]); } catch (TypeError $e) { echo "unset\n"; }
var_dump($a[0]);
try { var_dump(isset($a[$key])); } catch (TypeError $e) { echo "exists\n"; }
"#,
    );
    assert_eq!(out, "read\nwrite\nunset\nint(42)\nexists\n");
}

/// Refused internal calls evaluate all supplied argument side effects in source order.
#[test]
fn test_compliance_1806_internal_arity_preserves_argument_effects() {
    let out = compile_and_run(
        r#"<?php
function argument(int $n): string { echo $n; return str_repeat("x", $n); }
$a = new ArrayObject([]);
try { $a->natsort(argument(1), argument(2)); }
catch (ArgumentCountError $e) { echo ":", $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(out, "12:ArrayObject::natsort() expects exactly 0 arguments, 2 given\n");
}

/// An argument exception takes precedence over the internal callee's arity error.
#[test]
fn test_compliance_1806_argument_throw_precedes_arity_error() {
    let out = compile_and_run(
        r#"<?php
function argument(): int { throw new RuntimeException("argument"); }
$a = new ArrayObject([]);
try { $a->natsort(argument()); }
catch (RuntimeException $e) { echo $e->getMessage(); }
catch (ArgumentCountError $e) { echo "wrong"; }
"#,
    );
    assert_eq!(out, "argument");
}

/// Nullable and mixed declarations require explicit returns even when the result is discarded.
#[test]
fn test_compliance_1806_missing_nullable_and_mixed_returns_throw() {
    let out = compile_and_run(
        r#"<?php
function nullable(): ?int {}
function any(): mixed {}
function explicitNull(): ?int { return null; }
try { nullable(); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { any(); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
var_dump(explicitNull());
"#,
    );
    assert_eq!(out, "nullable(): Return value must be of type ?int, none returned\nany(): Return value must be of type mixed, none returned\nNULL\n");
}

/// Conditional fallthrough in a nullable method keeps its handler and explicit-null path.
#[test]
fn test_compliance_1806_nullable_method_conditional_fallthrough() {
    let out = compile_and_run(
        r#"<?php
class C { public function get(bool $return): ?string { if ($return) { return null; } } }
$c = new C();
var_dump($c->get(true));
try { $c->get(false); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(out, "NULL\nC::get(): Return value must be of type ?string, none returned\n");
}

/// Displayed warnings and trigger_error output are captured and discarded by output buffers.
#[test]
fn test_compliance_1806_diagnostics_follow_output_buffers() {
    let out = compile_and_run_capture(
        r#"<?php
ob_start();
trigger_error("captured", E_USER_WARNING);
$captured = ob_get_clean();
echo strpos($captured, "Warning: captured") !== false ? "captured\n" : "missing\n";
ob_start();
trigger_error("discarded", E_USER_WARNING);
ob_end_clean();
echo "done\n";
"#,
    );
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout_raw, "captured\ndone\n");
}

/// Constructor deprecations use the new expression's line after argument calls complete.
#[test]
fn test_compliance_1806_reserved_constructor_restores_callsite() {
    let out = compile_and_run_capture(
        r#"<?php
function backing(): object { return new stdClass(); }
$a = new ArrayObject(
    backing()
);
"#,
    );
    assert!(out.success, "{}", out.stderr);
    assert!(out.stdout_raw.contains("on line 3"), "{}", out.stdout_raw);
    assert!(!out.stdout_raw.contains("on line 4"), "{}", out.stdout_raw);
}

/// Uncaught exceptions retain user function frames before the unwinder removes activations.
#[test]
fn test_compliance_1806_uncaught_trace_keeps_user_frames() {
    let out = compile_and_run_capture(
        r#"<?php
function fail(): void { throw new RuntimeException("trace"); }
function caller(): void { fail(); }
caller();
"#,
    );
    assert!(!out.success);
    assert!(out.stdout_raw.contains("fail()"), "{}", out.stdout_raw);
    assert!(out.stdout_raw.contains("caller()"), "{}", out.stdout_raw);
    assert!(out.stdout_raw.contains("{main}"), "{}", out.stdout_raw);
}

/// Fresh computed string keys remain live until conversion consumes their owned boxed payload.
#[test]
fn test_compliance_1806_fixed_array_computed_string_key_lifetime() {
    let out = compile_and_run(
        r#"<?php
$a = new SplFixedArray(20);
$a[str_repeat("1", $argc + 1)] = 5;
var_dump($a[11], $a[str_repeat("1", $argc + 1)]);
unset($a[str_repeat("1", $argc + 1)]);
var_dump($a[11]);
"#,
    );
    assert_eq!(out, "int(5)\nint(5)\nNULL\n");
}

/// Error handlers receive the source call-site line for synthetic SPL constructor diagnostics.
#[test]
fn test_compliance_1806_constructor_error_handler_receives_callsite() {
    let out = compile_and_run(
        r#"<?php
set_error_handler(function ($level, $message, $file, $line) { echo $line, "\n"; return true; });
$a = new ArrayObject(
    new stdClass()
);
"#,
    );
    assert_eq!(out, "3\n");
}

/// Refused-call argument throws release fresh receivers and temporary argument results.
#[test]
fn test_compliance_1806_internal_arity_argument_throw_keeps_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function failArgument(): int { throw new RuntimeException("argument"); }
for ($i = 0; $i < 20; $i++) {
    try { (new ArrayObject([]))->natsort(str_repeat("x", $i + 1), failArgument()); }
    catch (RuntimeException $e) {}
}
echo "done\n";
"#,
    );
    assert_eq!(out.stdout, "done\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Argument temporaries survive later argument side effects before an arity error unwinds them.
#[test]
fn test_compliance_1806_arity_keeps_arguments_until_evaluation_finishes() {
    let out = compile_and_run(
        r#"<?php
class Arg { public function __destruct() { echo "D"; } }
function second(): int { echo "2"; return 0; }
$a = new ArrayObject([]);
try { $a->natsort(new Arg(), second()); } catch (ArgumentCountError $e) {}
unset($e);
echo "C";
"#,
    );
    assert_eq!(out, "2DC");
}

/// A inherited tentative-return attribute preserves native Countable dispatch.
#[test]
fn test_compliance_1806_inherited_return_type_marker_dispatches() {
    let out = compile_and_run(
        r#"<?php
class P { #[ReturnTypeWillChange] public function count() { return 7; } }
class C extends P implements Countable {}
echo count(new C()), "\n";
"#,
    );
    assert_eq!(out, "7\n");
}

/// Covariant self returns preserve nullable/scalar arms and concrete-parent interface contracts.
#[test]
fn test_compliance_1806_union_covariance_keeps_native_results() {
    let out = compile_and_run(
        r#"<?php
interface I { public function get(): I|string; }
class C implements I { public function get(): C|string { return "string"; } }
interface J { public function get(): ?J; }
class D implements J { public function get(): ?D { return null; } }
class P { public int $n = 9; }
interface K { public function get(): P; }
class E extends P implements K { public function get(): E { return $this; } }
echo (new C())->get(), "\n";
var_dump((new D())->get());
echo (new E())->get()->n, "\n";
"#,
    );
    assert_eq!(out, "string\nNULL\n9\n");
}

/// Nested array-backed wrappers write/unset their shared inner storage without mutating input arrays.
#[test]
fn test_compliance_1806_nested_wrappers_mutate_inner_entries() {
    let out = compile_and_run(
        r#"<?php
$input = ["x" => 1, "y" => 3];
$inner = new ArrayObject($input);
$middle = new ArrayIterator($inner);
$outer = new ArrayObject($middle);
$outer["x"] = 2;
unset($outer["y"]);
var_dump($input, $inner->getArrayCopy(), $outer->getArrayCopy());
"#,
    );
    assert_eq!(out, "array(2) {\n  [\"x\"]=>\n  int(1)\n  [\"y\"]=>\n  int(3)\n}\narray(1) {\n  [\"x\"]=>\n  int(2)\n}\narray(1) {\n  [\"x\"]=>\n  int(2)\n}\n");
}

/// Self-backed wrappers terminate projection and hide compiler storage keys at any nesting depth.
#[test]
fn test_compliance_1806_nested_self_backed_projection_terminates() {
    let out = compile_and_run(
        r#"<?php
pcntl_alarm(2);
class SelfStore extends ArrayObject {
    public function __construct() { parent::__construct($this); }
}
$inner = new SelfStore();
$inner->x = 5;
$outer = new ArrayObject($inner);
var_dump($outer->getArrayCopy());
$outer["x"] = 7;
var_dump($inner->x);
unset($outer["x"]);
var_dump(isset($inner->x));
pcntl_alarm(0);
"#,
    );
    assert_eq!(out, "array(1) {\n  [\"x\"]=>\n  int(5)\n}\nint(7)\nbool(false)\n");
}

/// Nested storage forwarding releases wrapper roots and copy-on-write snapshots after mutation.
#[test]
fn test_compliance_1806_nested_storage_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($i = 0; $i < 20; $i++) {
    $input = ["x" => str_repeat("a", 40), "y" => str_repeat("b", 40)];
    $inner = new ArrayObject($input);
    $middle = new ArrayIterator($inner);
    $outer = new ArrayObject($middle);
    $outer["x"] = str_repeat("c", 40);
    unset($outer["y"]);
    $snapshot = $outer->getArrayCopy();
    $snapshot["x"] = str_repeat("d", 40);
    if ($inner["x"] !== str_repeat("c", 40) || count($snapshot) !== 1 || count($input) !== 2) {
        echo "wrong";
    }
    unset($snapshot, $outer, $middle, $inner, $input);
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert!(out.stdout.ends_with("done") && !out.stdout.contains("wrong"), "unexpected output: {}", out.stdout);
    assert!(out.stderr.contains("leak summary: clean"), "nested storage leaked: {}", out.stderr);
}

/// Numeric string removal follows PHP key coercion while preserving holes and input-array COW.
#[test]
fn test_compliance_1806_nested_unset_numeric_string_preserves_holes() {
    let out = compile_and_run(
        r#"<?php
$input = ["a", "b", "c"];
$inner = new ArrayObject($input);
$outer = new ArrayIterator($inner);
unset($outer["1"]);
var_dump(array_keys($inner->getArrayCopy()), count($input));
"#,
    );
    assert_eq!(out, "array(2) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(2)\n}\nint(3)\n");
}

/// An indexed backing array is promoted safely for copies without mutating its input or owner.
#[test]
fn test_compliance_1806_indexed_storage_copy_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
for ($i = 0; $i < 20; $i++) {
    $input = [str_repeat("a", 40), str_repeat("b", 40)];
    $owner = new ArrayObject($input);
    $copy = $owner->getArrayCopy();
    $copy[0] = str_repeat("c", 40);
    if ($owner[0] !== $input[0] || count($copy) !== 2) { echo "wrong"; }
    unset($copy, $owner, $input);
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "indexed copy leaked: {}", out.stderr);
}

/// Self-backed writes release the temporary name produced by the internal Mixed-to-string cast.
#[test]
fn test_compliance_1806_self_backed_write_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
error_reporting(0);
class SelfStore extends ArrayObject {
    public function __construct() { parent::__construct($this); }
}
for ($i = 0; $i < 20; $i++) {
    $self = new SelfStore();
    $self->x = 5;
    $outer = new ArrayObject($self);
    $outer["x"] = 7;
    if ($self->x !== 7) { echo "wrong"; }
    unset($outer, $self);
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "self-backed write leaked: {}", out.stderr);
}

/// Dynamic property assignment retires its fresh receiver and computed name after the store.
#[test]
fn test_compliance_1806_dynamic_property_write_temporaries_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
error_reporting(0);
class Box { public int $x = 0; public bool $b = false; public float $f = 0.0; }
for ($i = 0; $i < 20; $i++) {
    (new Box())->{str_repeat("x", 1)} = $i;
    (new Box())->{str_repeat("b", 1)} = true;
    (new Box())->{str_repeat("f", 1)} = 1.5;
}
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "dynamic assignment leaked: {}", out.stderr);
}

/// A refused dynamic write unwinds the fresh object, computed name and boxed argument once.
#[test]
fn test_compliance_1806_dynamic_property_refusal_temporaries_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class RefusalBox { public int $x = 0; }
for ($i = 0; $i < 20; $i++) {
    try { (new RefusalBox())->{str_repeat("x", 1)} = [str_repeat("a", 40)]; }
    catch (TypeError $e) { echo "caught\n"; }
}
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "caught\n".repeat(20));
    assert!(out.stderr.contains("leak summary: clean"), "refused assignment leaked: {}", out.stderr);
}

/// A dynamic property statement evaluates receiver, name and RHS once in PHP source order.
#[test]
fn test_compliance_1806_dynamic_property_write_evaluation_order() {
    let out = compile_and_run(
        r#"<?php
class WriteOrder { public string $x = ""; }
function receiver(): WriteOrder { echo "o"; return new WriteOrder(); }
function name(): string { echo "n"; return "x"; }
function value(): string { echo "v"; return "ok"; }
receiver()->{name()} = value();
echo "done";
"#,
    );
    assert_eq!(out, "onvdone");
}

/// A named write releases an object receiver unboxed from a loop-widened local before cycle GC.
#[test]
fn test_compliance_1806_self_backed_receiver_after_loop_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
error_reporting(0);
for ($i = 0; $i < 2; $i++) {
    $inner = new ArrayObject(["x" => "a"]);
    $outer = new ArrayObject($inner);
    unset($outer, $inner);
}
class SelfStore extends ArrayObject {
    public function __construct() { parent::__construct($this); }
}
$inner = new SelfStore();
$inner->x = 5;
$outer = new ArrayObject($inner);
$outer["x"] = 7;
unset($outer["x"]);
unset($outer, $inner);
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "loop-widened receiver leaked: {}", out.stderr);
}

/// A named property statement retires a fresh receiver after the retaining store finishes.
#[test]
fn test_compliance_1806_named_property_fresh_receiver_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Named { public int $x = 0; }
for ($i = 0; $i < 20; $i++) { (new Named())->x = $i; }
echo "done";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "done");
    assert!(out.stderr.contains("leak summary: clean"), "fresh named receiver leaked: {}", out.stderr);
}
