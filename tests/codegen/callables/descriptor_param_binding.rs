//! Purpose:
//! Regression tests for runtime-selected callable parameter validation before marshalling.
//!
//! Called from:
//! - The codegen integration suite's `callables` module.
//!
//! Key details:
//! - Mixed relay parameters keep runtime values unknown to static argument validation.
//! - Fixed and named/positional variadic descriptor routes must reject invalid types catchably.

use crate::support::*;

#[test]
fn descriptor_param_binding_rejects_invalid_positional_variadic_values() {
    let out = compile_and_run(r#"<?php
function collect_ints(int ...$values): int { return $values[0]; }
function relay(callable $callback, mixed $value): mixed { return $callback($value); }
foreach (["abc", [], new stdClass()] as $value) {
    try { relay(collect_ints(...), $value); echo "accepted|"; }
    catch (TypeError $error) { echo "TypeError|"; }
}
echo relay(collect_ints(...), "42");
"#);
    assert_eq!(out, "TypeError|TypeError|TypeError|42");
}

#[test]
fn descriptor_param_binding_rejects_invalid_named_variadic_values() {
    let out = compile_and_run(r#"<?php
function collect_ints(int ...$values): int { return $values["value"]; }
function relay(callable $callback, mixed $value): mixed {
    return call_user_func_array($callback, ["value" => $value]);
}
try { relay(collect_ints(...), "abc"); echo "accepted|"; }
catch (TypeError $error) { echo "TypeError|"; }
echo relay(collect_ints(...), "42");
"#);
    assert_eq!(out, "TypeError|42");
}

#[test]
fn descriptor_param_binding_rejects_invalid_fixed_values() {
    let out = compile_and_run(r#"<?php
function take_int(int $value): int { return $value; }
function relay(callable $callback, mixed $value): mixed { return $callback($value); }
foreach (["abc", [], new stdClass()] as $value) {
    try { relay(take_int(...), $value); echo "accepted|"; }
    catch (TypeError $error) { echo "TypeError|"; }
}
echo relay(take_int(...), "42");
"#);
    assert_eq!(out, "TypeError|TypeError|TypeError|42");
}

#[test]
fn descriptor_param_binding_rejects_array_and_object_to_string() {
    let out = compile_and_run(r#"<?php
function take_strings(string ...$values): string { return $values[0]; }
function relay(callable $callback, mixed $value): mixed { return $callback($value); }
foreach ([[], new stdClass()] as $value) {
    try { relay(take_strings(...), $value); echo "accepted|"; }
    catch (TypeError $error) { echo "TypeError|"; }
}
echo relay(take_strings(...), 42);
"#);
    assert_eq!(out, "TypeError|TypeError|42");
}

#[test]
fn descriptor_param_binding_preserves_stringable_weak_coercion() {
    let out = compile_and_run(r#"<?php
class Text {
    public int $calls = 0;
    public function __toString(): string { $this->calls += 1; return "text"; }
}
function take_strings(string ...$values): string { return $values[0]; }
function relay(callable $callback, mixed $value): mixed { return $callback($value); }
$text = new Text();
echo relay(take_strings(...), $text), "|", $text->calls;
"#);
    assert_eq!(out, "text|1");
}

#[test]
fn descriptor_param_binding_numeric_string_bounds() {
    let out = compile_and_run(r#"<?php
function take_ints(int ...$values): int { return $values[0]; }
function relay(callable $callback, mixed $value): mixed { return $callback($value); }
foreach (["  +0012  ", "1e2", "9223372036854775807", "-9223372036854775808", "-9223372036854775809"] as $value) {
    echo relay(take_ints(...), $value), "|";
}
foreach (["9223372036854775808", "-9223372036854777856", "1e100", "12abc"] as $value) {
    try { relay(take_ints(...), $value); echo "accepted|"; }
    catch (TypeError $error) { echo "TypeError|"; }
}
"#);
    assert_eq!(out, "12|100|9223372036854775807|-9223372036854775808|-9223372036854775808|TypeError|TypeError|TypeError|TypeError|");
}

#[test]
fn descriptor_param_binding_releases_owners_when_stringable_throws() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class BrokenText {
    public function __toString(): string { throw new RuntimeException("broken"); }
}
function take_strings(string $first, string ...$values): string { return $first; }
function relay(callable $callback, mixed $value): mixed {
    return $callback("first", "earlier", $value);
}
try { relay(take_strings(...), new BrokenText()); echo "accepted"; }
catch (RuntimeException $error) { echo $error->getMessage(); unset($error); }
"#);
    assert_eq!(out.stdout, "broken", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_forwards_named_and_positional_source_arguments_once() {
    let out = compile_and_run(r#"<?php
function relay(callable $callback, mixed $value): mixed {
    return $callback(10, 20, tail: $value);
}
echo relay(function (int $head, int ...$values): string {
    return $head . ":" . json_encode($values);
}, "42");
"#);
    assert_eq!(out, "10:{\"0\":20,\"tail\":42}");
}

#[test]
fn descriptor_param_binding_named_array_prefix_survives_stringable_throw() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class BrokenText {
    public function __toString(): string { throw new RuntimeException("broken"); }
}
function consume(array $prefix, string ...$values): string { return "unused"; }
function relay(callable $callback, mixed $prefix, mixed $value): mixed {
    return call_user_func_array($callback, ["prefix" => $prefix, "tail" => $value]);
}
$prefix = [11, 22];
try { relay(consume(...), $prefix, new BrokenText()); echo "accepted"; }
catch (RuntimeException $error) { echo $error->getMessage(), "|", json_encode($prefix); unset($error); }
unset($prefix);
"#);
    assert_eq!(out.stdout, "broken|[11,22]", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_named_array_prefix_owns_normal_parameter_and_keeps_cow() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function consume(array $prefix): string { $prefix[0] = 99; return json_encode($prefix); }
function relay(callable $callback, mixed $prefix): mixed {
    return call_user_func_array($callback, ["prefix" => $prefix]);
}
$prefix = [11, 22];
echo relay(consume(...), $prefix), "|", json_encode($prefix);
unset($prefix);
"#);
    assert_eq!(out.stdout, "[99,22]|[11,22]", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_array_default_transfers_exactly_one_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function consume($values = []): int { return count($values); }
function relay(callable $callback): mixed { return $callback(); }
echo relay(consume(...));
"#);
    assert_eq!(out.stdout, "0", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_untyped_variadic_contract_does_not_follow_inferred_storage() {
    let out = compile_and_run(r#"<?php
function collect(...$values): string { return json_encode($values); }
function relay(callable $callback, mixed $value): mixed { return $callback($value, named: $value); }
collect(1);
echo relay(collect(...), "abc"), "|";
$object = new stdClass();
$object->answer = 42;
echo relay(collect(...), $object), "|";
echo relay(collect(...), [1, "mixed"]);
"#);
    assert_eq!(out, "{\"0\":\"abc\",\"named\":\"abc\"}|{\"0\":{\"answer\":42},\"named\":{\"answer\":42}}|{\"0\":[1,\"mixed\"],\"named\":[1,\"mixed\"]}");
}

#[test]
fn descriptor_param_binding_borrowed_mixed_return_preserves_one_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function identity(mixed $value): mixed { return $value; }
$source = [11, 22];
$result = call_user_func_array("identity", [$source]);
echo json_encode($result), "|", json_encode($source);
unset($result);
unset($source);
"#);
    assert_eq!(out.stdout, "[11,22]|[11,22]", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_borrowed_string_return_transfers_parameter_owner() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function identity(string $value): string { return $value; }
$source = str_repeat("x", 24);
$result = call_user_func_array("identity", [$source]);
echo strlen($result), "|", strlen($source);
unset($result);
unset($source);
"#);
    assert_eq!(out.stdout, "24|24", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_returned_callable_box_keeps_its_payload_lease() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function answer(): int { return 42; }
function return_callback(callable $callback): mixed { return $callback; }
$callback = return_callback(answer(...));
echo call_user_func_array($callback, []);
unset($callback);
"#);
    assert_eq!(out.stdout, "42", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

#[test]
fn descriptor_param_binding_object_lookup_uses_runtime_string_abi() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class RequiredGroup {}
class WrongGroup {}
function require_group(RequiredGroup $group): int { return 42; }
function require_group_alternate(RequiredGroup $group): int { return 42; }
$callback = strtolower($argc > 0 ? "REQUIRE_GROUP" : "REQUIRE_GROUP_ALTERNATE");
echo call_user_func_array($callback, [new RequiredGroup()]), "|";
try { call_user_func_array($callback, [new WrongGroup()]); echo "accepted"; }
catch (TypeError $error) { echo "TypeError"; unset($error); }
"#);
    assert_eq!(out.stdout, "42|TypeError", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
