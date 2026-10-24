//! Purpose:
//! Executes method splices with nested calls, property writes and string scratch lifetimes.
//!
//! Called from:
//! - The optimizer method-inlining codegen suite.
//!
//! Key details:
//! - Reuses strict heap validation and compares IR optimization on and off.

use super::*;

/// Inherited setters keep their declaring scope while nested static/free calls preserve evaluation.
#[test]
fn test_method_effects_inherited_setters_and_nested_calls() {
    let source = r#"<?php
function outside(int $n): int { for ($i = 0; $i < 2; $i++) { $n = $n + 1; } return $n; }
final class Entity { public static function index(int $id): int { return $id & 255; } }
class Balance {
    private int $value = 0;
    public function set(int $id): void { $this->value = Entity::index($id); }
    public function add(int $amount): void { $this->value = $this->value + $amount; }
    public function value(): int { return $this->value; }
}
final class Counter extends Balance {
    private int $value = 99;
    public function own(): int { return $this->value; }
}
function exercise(Counter $counter, int $id): int {
    $counter->set($id);
    $counter->add(outside($id));
    return $counter->value() * 1000 + $counter->own();
}
echo exercise(new Counter(), $argc), "\n";
"#;
    assert_both_modes(source, true, "4099\n");
}

/// Spliced string writes and concatenation never reset the caller's in-flight string scratch.
#[test]
fn test_method_effects_string_building_keeps_caller_scratch() {
    let source = r#"<?php
final class Label {
    private string $prefix = '<';
    public function prefix(string $prefix): void { $this->prefix = $prefix; }
    public function wrap(string $name): string { return $this->prefix . $name . ']'; }
}
function decorate(Label $label, string $name): string {
    $label->prefix('[');
    return 'head:' . $label->wrap($name) . ':tail';
}
echo decorate(new Label(), 'item'), "\n";
"#;
    assert_both_modes(source, true, "head:[item]:tail\n");
}

/// A nested real call still creates an exception at its original PHP source line after splicing.
#[test]
fn test_method_effects_nested_call_preserves_exception_location() {
    let source = r#"<?php
function nested_failure(int $n): int {
    for ($i = 0; $i < $n; $i++) {}
    throw new RuntimeException('nested');
}
final class Invoke { public function call(int $n): int { return nested_failure($n); } }
function exercise(Invoke $invoke, int $n): int { return $invoke->call($n); }
try { exercise(new Invoke(), $argc); }
catch (RuntimeException $error) { echo $error->getMessage(), ':', $error->getLine(), "\n"; }
"#;
    let line = source.lines().position(|line| line.contains("throw new RuntimeException")).unwrap() + 1;
    assert_both_modes(source, true, &format!("nested:{line}\n"));
}
