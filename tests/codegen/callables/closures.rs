//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of callables closures, including closure basic, closure multiple params, and arrow function basic.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use crate::support::*;

// --- Anonymous functions (closures) and arrow functions ---

/// Verifies a declared `Closure` return accepts the descriptor-equivalent type produced by
/// `instanceof Closure` narrowing while the other branch converts a generic callable.
#[test]
fn test_closure_return_accepts_instanceof_narrowed_callable() {
    let out = compile_and_run(
        r#"<?php
function normalize_closure_return(callable $code): Closure {
    if (!$code instanceof Closure) {
        return $code(...);
    }
    return $code;
}

function closure_return_target(int $value): int {
    return $value + 1;
}

$literal = normalize_closure_return(fn (int $value): int => $value + 2);
$firstClass = normalize_closure_return(closure_return_target(...));
echo $literal(3) . ":" . $firstClass(4);
"#,
    );
    assert_eq!(out, "5:5");
}

/// Verifies a nullable object rebound to a callable converges to `Closure` after a negated
/// `instanceof Closure` branch normalizes the possible non-closure object with first-class syntax.
#[test]
fn test_negated_closure_guard_converges_nullable_object_to_callable() {
    let out = compile_and_run(
        r#"<?php
class CallableApplication {
    public function __invoke(): int {
        return 7;
    }
}

function normalize_application(?object $application): Closure {
    $application ??= static fn (): int => 5;

    if (!$application instanceof Closure) {
        if (!is_callable($application)) {
            throw new LogicException("not callable");
        }

        $application = $application(...);
    }

    return $application;
}

$default = normalize_application(null);
$object = normalize_application(new CallableApplication());
echo $default() . ":" . $object();
"#,
    );
    assert_eq!(out, "5:7");
}

/// Verifies a method's early object return is checked with its branch-local type even when the
/// same parameter is normalized to a callable later in the body.
#[test]
fn test_method_early_object_return_keeps_flow_type_before_callable_reassignment() {
    let out = compile_and_run(
        r#"<?php
interface RunnerContract {
    public function run(): int;
}

class ConcreteRunner implements RunnerContract {
    public function run(): int {
        return 9;
    }
}

class InvokableApplication {
    public function __invoke(): int {
        return 7;
    }
}

class ClosureRunnerAdapter implements RunnerContract {
    public function __construct(Closure $closure) {}

    public function run(): int {
        return 1;
    }
}

class RunnerFactory {
    public function getRunner(?object $application): RunnerContract {
        $application ??= static fn (): int => 5;

        if ($application instanceof RunnerContract) {
            return $application;
        }

        if (!$application instanceof Closure) {
            if (!is_callable($application)) {
                throw new LogicException("not callable");
            }
            $application = $application(...);
        }

        return new ClosureRunnerAdapter($application);
    }
}

$factory = new RunnerFactory();
echo $factory->getRunner(new ConcreteRunner())->run();
echo ":";
echo $factory->getRunner(new InvokableApplication())->run();
"#,
    );
    assert_eq!(out, "9:1");
}

/// Verifies basic anonymous function creation, assignment to variable, and invocation with one argument.
#[test]
fn test_closure_basic() {
    let out = compile_and_run(
        r#"<?php
$double = function($x) { return $x * 2; };
echo $double(5);
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies anonymous function with multiple parameters and a simple arithmetic body.
#[test]
fn test_closure_multiple_params() {
    let out = compile_and_run(
        r#"<?php
$add = function($a, $b) { return $a + $b; };
echo $add(3, 7);
"#,
    );
    assert_eq!(out, "10");
}

/// Verifies basic arrow function (`fn`) syntax with one parameter and multiplication body.
#[test]
fn test_arrow_function_basic() {
    let out = compile_and_run(
        r#"<?php
$triple = fn($x) => $x * 3;
echo $triple(4);
"#,
    );
    assert_eq!(out, "12");
}

/// Verifies arrow function with a compound expression body (`$x * $x + 1`).
#[test]
fn test_arrow_function_expression() {
    let out = compile_and_run(
        r#"<?php
$calc = fn($x) => $x * $x + 1;
echo $calc(5);
"#,
    );
    assert_eq!(out, "26");
}

/// Verifies a variable ASSIGNED inside an arrow-function body is treated as a body-local, not a
/// mis-detected outer capture: `fn($x) => ($y = $x + 1) + $y` assigns `$y` then reads it, and must
/// compile (not report "Undefined variable in use(): $y"). `f(2)`: `$y = 3`, `3 + 3` = 6.
#[test]
fn test_arrow_function_body_local_assignment_is_not_captured() {
    let out = compile_and_run(
        r#"<?php
$f = fn($x) => ($y = $x + 1) + $y;
echo $f(2);
"#,
    );
    assert_eq!(out, "6");
}

/// Verifies the assignment-target seeding does not swallow a genuine outer capture: `$outer` is
/// only ever read inside the arrow body, so it is still captured by value at definition time.
#[test]
fn test_arrow_function_genuine_capture_still_captured_with_body_assignment() {
    let out = compile_and_run(
        r#"<?php
$outer = 10;
$g = fn($x) => ($y = $x + 1) + $y + $outer;
echo $g(2);
"#,
    );
    assert_eq!(out, "16");
}

/// Regression for #300: arrow functions capture outer locals by value at definition time.
#[test]
fn test_arrow_function_captures_outer_local_by_value() {
    let out = compile_and_run(
        r#"<?php
$x = 1;
$f = fn() => $x;
$x = 2;
echo $f();
"#,
    );
    assert_eq!(out, "1");
}

/// Verifies closure with typed parameter, return type annotation, and `use` clause capturing a string variable.
#[test]
fn test_closure_return_type_annotation() {
    let out = compile_and_run(
        r#"<?php
$prefix = "id:";
$format = function(int $value) use ($prefix): string {
    return $prefix . $value;
};
echo $format(7);
"#,
    );
    assert_eq!(out, "id:7");
}

/// Verifies closure parameter and return type are both `string`, with passthrough returning the same value.
#[test]
fn test_closure_return_type_annotation_uses_typed_param() {
    let out = compile_and_run(
        r#"<?php
$identity = function(string $value): string {
    return $value;
};
echo $identity("ok");
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies arrow function with typed `int` parameter and return type annotation.
#[test]
fn test_arrow_return_type_annotation() {
    let out = compile_and_run(
        r#"<?php
$double = fn(int $value): int => $value * 2;
echo $double(9);
"#,
    );
    assert_eq!(out, "18");
}

/// Verifies immediately-invoked arrow function (IIFE) with return type annotation and no parameters.
#[test]
fn test_iife_arrow_return_type_annotation() {
    let out = compile_and_run(
        r#"<?php
echo (fn(): string => "ready")();
"#,
    );
    assert_eq!(out, "ready");
}

/// Verifies `array_map` with an anonymous closure using `array_map(function($x) { ... }, [...])` syntax.
#[test]
fn test_closure_array_map() {
    let out = compile_and_run(
        r#"<?php
$result = array_map(function($x) { return $x * 10; }, [1, 2, 3]);
echo $result[0];
echo $result[1];
echo $result[2];
"#,
    );
    assert_eq!(out, "102030");
}

/// Verifies `array_map` with a typed arrow function `fn(int $x): int => ...` passed as callable.
#[test]
fn test_arrow_function_array_map() {
    let out = compile_and_run(
        r#"<?php
$result = array_map(fn(int $x): int => $x + 100, [1, 2, 3]);
echo $result[0];
echo $result[1];
echo $result[2];
"#,
    );
    assert_eq!(out, "101102103");
}

/// Verifies `array_map` with a closure that captures a variable via `use ($factor)`.
#[test]
fn test_captured_closure_array_map() {
    let out = compile_and_run(
        r#"<?php
$factor = 7;
$result = array_map(function($x) use ($factor) { return $x * $factor; }, [1, 2, 3]);
echo $result[0];
echo $result[1];
echo $result[2];
"#,
    );
    assert_eq!(out, "71421");
}

/// Verifies `array_map` where the callable closure is assigned to a variable before passing.
#[test]
fn test_captured_closure_variable_array_map() {
    let out = compile_and_run(
        r#"<?php
$offset = 5;
$add = function($x) use ($offset) { return $x + $offset; };
$result = array_map($add, [10, 20]);
echo $result[0];
echo $result[1];
"#,
    );
    assert_eq!(out, "1525");
}

/// Verifies callback runtimes read by-value closure captures from descriptor storage
/// instead of rereading the current source variable after reassignment.
#[test]
fn test_captured_closure_variable_array_map_uses_descriptor_capture_after_reassign() {
    let out = compile_and_run(
        r#"<?php
$offset = 5;
$add = function(int $x) use ($offset): int {
    return $x + $offset;
};
$offset = 100;
$result = array_map($add, [1, 2]);
echo $result[0];
echo ":";
echo $result[1];
"#,
    );
    assert_eq!(out, "6:7");
}

/// Verifies string capture via `use ($prefix)` in a typed closure passed to `array_map`, producing string-concatenated output.
#[test]
fn test_captured_closure_variable_array_map_string_capture() {
    let out = compile_and_run(
        r#"<?php
$prefix = "id:";
$format = function(int $value) use ($prefix): string {
    return $prefix . $value;
};
$result = array_map($format, [7, 8]);
echo $result[0];
echo ",";
echo $result[1];
"#,
    );
    assert_eq!(out, "id:7,id:8");
}

/// Verifies `str_starts_with` inside a captured closure passed to `array_map` with string array input.
#[test]
fn test_captured_closure_variable_array_map_string_values() {
    let out = compile_and_run(
        r#"<?php
$prefix = "a";
$starts = function(string $value) use ($prefix): int {
    return str_starts_with($value, $prefix) ? 1 : 0;
};
$result = array_map($starts, ["aa", "bb", "ab"]);
echo $result[0];
echo $result[1];
echo $result[2];
"#,
    );
    assert_eq!(out, "101");
}

/// Verifies `array_filter` with an anonymous closure returning even numbers.
#[test]
fn test_closure_array_filter() {
    let out = compile_and_run(
        r#"<?php
$evens = array_filter([1, 2, 3, 4, 5, 6], function($x) { return $x % 2 == 0; });
echo count($evens);
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies `array_filter` with a captured `use ($limit)` closure comparing against a threshold.
#[test]
fn test_captured_closure_array_filter() {
    let out = compile_and_run(
        r#"<?php
$limit = 4;
$filtered = array_filter([1, 4, 5, 9], function($x) use ($limit) { return $x > $limit; });
echo count($filtered);
foreach ($filtered as $value) { echo $value; }
"#,
    );
    assert_eq!(out, "259");
}

/// Verifies `str_starts_with` inside a captured closure passed to `array_filter` with string array input.
#[test]
fn test_captured_closure_variable_array_filter_string_values() {
    let out = compile_and_run(
        r#"<?php
$prefix = "a";
$starts = function(string $value) use ($prefix) {
    return str_starts_with($value, $prefix);
};
$filtered = array_filter(["aa", "bb", "ab"], $starts);
echo count($filtered);
foreach ($filtered as $value) { echo $value; }
"#,
    );
    assert_eq!(out, "2aaab");
}

/// Verifies `call_user_func` with a closure that captures a base value via `use ($base)`.
#[test]
fn test_captured_closure_call_user_func() {
    let out = compile_and_run(
        r#"<?php
$base = 30;
$fn = function($x) use ($base) { return $base + $x; };
echo call_user_func($fn, 12);
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies `call_user_func` with an inline immediately-created captured closure without intermediate variable assignment.
#[test]
fn test_inline_captured_closure_call_user_func() {
    let out = compile_and_run(
        r#"<?php
$base = 9;
echo call_user_func(function($x) use ($base) { return $x * $base; }, 6);
"#,
    );
    assert_eq!(out, "54");
}

/// Verifies inline closure `call_user_func()` dispatch goes through a descriptor invoker.
#[test]
fn test_inline_closure_call_user_func_uses_descriptor_invoker() {
    let source = r#"<?php
$base = 9;
echo call_user_func(function(int $x) use ($base): int { return $x * $base; }, 6);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "54");

    let dir = make_cli_test_dir("elephc_inline_closure_call_user_func_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "inline call_user_func closure dispatch should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies branch-selected captured callables route through `call_user_func()` descriptor invokers.
#[test]
fn test_call_user_func_complex_captured_callable_expr_uses_descriptor_invoker() {
    let source = r#"<?php
class Counter {
    public int $base = 0;

    public function add(int $n = 4): int {
        return $n + $this->base;
    }
}

$left = new Counter();
$left->base = 3;
$right = new Counter();
$right->base = 7;
$use_left = false;
echo call_user_func($use_left ? $left->add(...) : $right->add(...), 5);
echo ",";
echo call_user_func($use_left ? $left->add(...) : $right->add(...));
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12,11");

    let dir = make_cli_test_dir("elephc_call_user_func_complex_callable_expr_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "call_user_func branch-selected captured callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies `call_user_func()` descriptor invokers preserve by-reference args for branch callables.
#[test]
fn test_call_user_func_complex_captured_callable_expr_preserves_by_ref_arg() {
    let source = r#"<?php
class Counter {
    public int $step = 0;

    public function bump(int &$n): void {
        $n = $n + $this->step;
    }
}

$left = new Counter();
$left->step = 3;
$right = new Counter();
$right->step = 7;
$use_left = false;
$value = 5;
call_user_func($use_left ? $left->bump(...) : $right->bump(...), $value);
echo $value;
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12");
}

/// Verifies branch-selected captured first-class callables use descriptor invokers.
#[test]
fn test_direct_complex_captured_callable_expr_uses_descriptor_invoker() {
    let source = r#"<?php
class Counter {
    public int $base = 0;

    public function add(int $n): int {
        return $n + $this->base;
    }
}

$left = new Counter();
$left->base = 3;
$right = new Counter();
$right->base = 7;
$use_left = false;
echo ($use_left ? $left->add(...) : $right->add(...))(5);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12");

    let dir = make_cli_test_dir("elephc_direct_complex_callable_expr_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "direct branch-selected captured callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies direct descriptor calls with one spread source pass the source container through.
#[test]
fn test_direct_complex_captured_callable_expr_single_spread_uses_descriptor_invoker() {
    let source = r#"<?php
class Prefixer {
    public string $prefix = "";

    public function wrap(string $name, string $suffix = "!"): string {
        return $this->prefix . $name . $suffix;
    }
}

$left = new Prefixer();
$left->prefix = "L:";
$right = new Prefixer();
$right->prefix = "R:";
$use_left = false;
$args = ["Ada"];
echo ($use_left ? $left->wrap(...) : $right->wrap(...))(...$args);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "R:Ada!");

    let dir = make_cli_test_dir("elephc_direct_complex_callable_expr_single_spread_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "direct branch-selected single-spread callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies direct descriptor calls with positional+spread args build invoker containers.
#[test]
fn test_direct_complex_captured_callable_expr_positional_spread_uses_descriptor_invoker() {
    let source = r#"<?php
class Prefixer {
    public string $prefix = "";

    public function wrap(string $name, string $suffix): string {
        return $this->prefix . $name . $suffix;
    }
}

$left = new Prefixer();
$left->prefix = "L:";
$right = new Prefixer();
$right->prefix = "R:";
$use_left = false;
$args = ["?"];
echo ($use_left ? $left->wrap(...) : $right->wrap(...))("Ada", ...$args);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "R:Ada?");

    let dir = make_cli_test_dir("elephc_direct_complex_callable_expr_positional_spread_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "direct branch-selected positional+spread callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies branch-selected descriptor invokers preserve named arguments and defaults.
#[test]
fn test_direct_complex_captured_callable_expr_named_args_use_descriptor_invoker() {
    let source = r#"<?php
class Counter {
    public int $base = 0;

    public function add(int $n = 4): int {
        return $n + $this->base;
    }
}

$left = new Counter();
$left->base = 3;
$right = new Counter();
$right->base = 7;
$use_left = false;
echo ($use_left ? $left->add(...) : $right->add(...))(n: 5);
echo ",";
echo ($use_left ? $left->add(...) : $right->add(...))();
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12,11");

    let dir = make_cli_test_dir("elephc_direct_complex_callable_expr_named_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "direct branch-selected named callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies branch-selected descriptor invokers accept spread prefixes followed by named args.
#[test]
fn test_direct_complex_captured_callable_expr_named_spread_args_use_descriptor_invoker() {
    let source = r#"<?php
class Counter {
    public int $base = 0;

    public function add(int $n = 4, int $scale = 1): int {
        return ($n * $scale) + $this->base;
    }
}

$left = new Counter();
$left->base = 3;
$right = new Counter();
$right->base = 7;
$use_left = false;
$args = [2];
echo ($use_left ? $left->add(...) : $right->add(...))(...$args, scale: 5);
echo ",";
$empty = [];
echo ($use_left ? $left->add(...) : $right->add(...))(...$empty, n: 6);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "17,13");

    let dir = make_cli_test_dir("elephc_direct_complex_callable_expr_named_spread_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "direct branch-selected named+spread callable calls should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies stored branch-selected captured callables invoke through descriptor metadata.
#[test]
fn test_stored_branch_selected_captured_callable_variable_uses_descriptor_invoker() {
    let source = r#"<?php
class Prefixer {
    public string $prefix = "";

    public function wrap(string $name, string $suffix = "!"): string {
        return $this->prefix . $name . $suffix;
    }
}

$left = new Prefixer();
$left->prefix = "L:";
$right = new Prefixer();
$right->prefix = "R:";
$use_left = false;
$cb = $use_left ? $left->wrap(...) : $right->wrap(...);
echo $cb(name: "Ada");
echo ",";
echo $cb("Eve", suffix: "?");
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "R:Ada!,R:Eve?");

    let dir = make_cli_test_dir("elephc_stored_branch_callable_variable_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "stored branch-selected callable variables should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies stored descriptor calls preserve by-reference args through runtime signature metadata.
#[test]
fn test_stored_branch_selected_captured_callable_variable_preserves_by_ref_arg() {
    let source = r#"<?php
class Counter {
    public int $step = 0;

    public function bump(int &$n): void {
        $n = $n + $this->step;
    }
}

$left = new Counter();
$left->step = 3;
$right = new Counter();
$right->step = 7;
$use_left = false;
$cb = $use_left ? $left->bump(...) : $right->bump(...);
$value = 5;
$cb($value);
echo $value;
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12");

    let dir = make_cli_test_dir("elephc_stored_branch_callable_variable_by_ref_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "stored branch-selected callable variables with by-ref args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies stored untyped branch-selected callables use descriptor metadata for named args.
#[test]
fn test_stored_branch_selected_untyped_callable_variable_named_args_uses_descriptor_invoker() {
    let source = r#"<?php
class Calculator {
    public $base;

    public function __construct($base) {
        $this->base = $base;
    }

    public function scale($value = 1, $factor = 1) {
        return $this->base + ($value * $factor);
    }
}

$left = new Calculator(10);
$right = new Calculator(100);
$use_left = false;
$cb = $use_left ? $left->scale(...) : $right->scale(...);
echo $cb(value: 2, factor: 4);
$args = [2];
echo ",";
echo $cb(...$args, factor: 4);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "108,108");

    let dir = make_cli_test_dir("elephc_stored_untyped_branch_callable_named_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "stored untyped branch-selected callable variables with named args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies callable params with unknown signatures dereference named variable markers for by-value params.
#[test]
fn test_callable_param_unknown_signature_named_variable_arg_uses_descriptor_invoker() {
    let source = r#"<?php
function run(callable $cb): void {
    $name = "Ada";
    echo $cb(name: $name);
    echo ":";
    echo $name;
}

$cb = function(string $name): string {
    return "hi " . $name;
};

run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "hi Ada:Ada");

    let dir = make_cli_test_dir("elephc_callable_param_unknown_named_value_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "callable params with unknown named variable args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies callable params with unknown signatures preserve named by-reference variables.
#[test]
fn test_callable_param_unknown_signature_named_by_ref_arg_uses_descriptor_invoker() {
    let source = r#"<?php
function run(callable $cb): void {
    $value = 5;
    $cb(value: $value);
    echo $value;
}

$cb = function(int &$value): void {
    $value = $value + 7;
};

run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12");

    let dir = make_cli_test_dir("elephc_callable_param_unknown_named_ref_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "callable params with unknown named by-ref args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies unknown callable params preserve named by-reference variables after a spread prefix.
#[test]
fn test_callable_param_unknown_signature_named_spread_by_ref_arg_uses_descriptor_invoker() {
    let source = r#"<?php
function run(callable $cb): void {
    $value = 5;
    $args = [];
    $cb(...$args, value: $value);
    echo $value;
}

$cb = function(int &$value): void {
    $value = $value + 11;
};

run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "16");
}

/// Verifies unknown callable params preserve positional by-reference variables before a spread tail.
#[test]
fn test_callable_param_unknown_signature_positional_spread_by_ref_arg_uses_descriptor_invoker() {
    let source = r#"<?php
function run(callable $cb): void {
    $value = 5;
    $args = [];
    $cb($value, ...$args);
    echo $value;
}

$cb = function(int &$value): void {
    $value = $value + 13;
};

run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "18");

    let dir = make_cli_test_dir("elephc_callable_param_unknown_positional_spread_ref_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "callable params with positional+spread by-ref args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies unknown callable params preserve positional by-reference variables before named suffixes.
#[test]
fn test_callable_param_unknown_signature_named_spread_prefix_by_ref_arg_uses_descriptor_invoker() {
    let source = r#"<?php
function run(callable $cb): void {
    $value = 5;
    $args = [];
    $cb($value, ...$args, label: "done");
    echo ":" . $value;
}

$cb = function(int &$value, string $label): void {
    echo $label;
    $value = $value + 9;
};

run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "done:14");

    let dir = make_cli_test_dir("elephc_callable_param_unknown_named_spread_prefix_ref_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "callable params with named+spread prefix by-ref args should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies receiver-bound callable params preserve named by-reference variables.
#[test]
fn test_callable_param_unknown_signature_method_named_by_ref_arg_uses_descriptor_invoker() {
    let source = r#"<?php
class Bumper {
    public $step;

    public function __construct($step) {
        $this->step = $step;
    }

    public function bump(&$value) {
        $value = $value + $this->step;
    }
}

function run(callable $cb): void {
    $value = 5;
    $cb(value: $value);
    echo $value;
}

$left = new Bumper(3);
$right = new Bumper(7);
$use_left = false;
$cb = $use_left ? $left->bump(...) : $right->bump(...);
run($cb);
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "12");
}

/// Verifies callable descriptors loaded from array elements invoke through runtime metadata.
#[test]
fn test_array_loaded_branch_selected_captured_callable_uses_descriptor_invoker() {
    let source = r#"<?php
class Prefixer {
    public string $prefix = "";

    public function wrap(string $name, string $suffix = "!"): string {
        return $this->prefix . $name . $suffix;
    }
}

$left = new Prefixer();
$left->prefix = "L:";
$right = new Prefixer();
$right->prefix = "R:";
$use_left = false;
$callbacks = [$use_left ? $left->wrap(...) : $right->wrap(...)];
echo $callbacks[0]("Ada");
"#;
    let out = compile_and_run(source);
    assert_eq!(out, "R:Ada!");

    let dir = make_cli_test_dir("elephc_array_loaded_branch_callable_invoker");
    let (user_asm, _runtime_asm, _required_libraries) =
        compile_source_to_asm_with_options(source, &dir, 8_388_608, false, false);
    assert!(
        user_asm.contains("callable_invoker"),
        "array-loaded branch-selected callable descriptors should route through descriptor invokers:\n{}",
        user_asm
    );
    let _ = fs::remove_dir_all(dir);
}

/// Verifies `array_filter` with an arrow function predicate filtering values greater than 8.
#[test]
fn test_arrow_function_array_filter() {
    let out = compile_and_run(
        r#"<?php
$big = array_filter([1, 5, 10, 15, 20], fn($x) => $x > 8);
echo count($big);
"#,
    );
    assert_eq!(out, "3");
}

/// Verifies closure stored in a variable and called multiple times, confirming each invocation is independent.
#[test]
fn test_closure_as_variable_then_call() {
    let out = compile_and_run(
        r#"<?php
$fn = function($x) { return $x + 1; };
$a = $fn(10);
$b = $fn(20);
echo $a;
echo $b;
"#,
    );
    assert_eq!(out, "1121");
}

/// Verifies anonymous closure with no parameters that returns a constant integer.
#[test]
fn test_closure_no_params() {
    let out = compile_and_run(
        r#"<?php
$hello = function() { return 42; };
echo $hello();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies arrow function with no parameters that returns a constant integer.
#[test]
fn test_arrow_no_params() {
    let out = compile_and_run(
        r#"<?php
$val = fn() => 99;
echo $val();
"#,
    );
    assert_eq!(out, "99");
}

/// Verifies `array_reduce` with an anonymous closure summing a numeric array, using an initial carry value of 0.
#[test]
fn test_closure_array_reduce() {
    let out = compile_and_run(
        r#"<?php
$sum = array_reduce([1, 2, 3, 4], function($carry, $item) { return $carry + $item; }, 0);
echo $sum;
"#,
    );
    assert_eq!(out, "10");
}

// --- IIFE (Immediately Invoked Function Expression) ---

/// Verifies immediately-invoked anonymous function expression (IIFE) returning a constant.
#[test]
fn test_iife_basic() {
    let out = compile_and_run(
        r#"<?php
echo (function() { return 42; })();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies immediately-invoked anonymous function expression (IIFE) with one argument passed at call site.
#[test]
fn test_iife_with_args() {
    let out = compile_and_run(
        r#"<?php
echo (function($x) { return $x * 3; })(7);
"#,
    );
    assert_eq!(out, "21");
}

/// Verifies immediately-invoked arrow function (IIFE) with one argument passed at call site.
#[test]
fn test_iife_arrow() {
    let out = compile_and_run(
        r#"<?php
echo (fn($x) => $x + 100)(5);
"#,
    );
    assert_eq!(out, "105");
}

/// Verifies immediately-invoked closures can use named arguments and defaults.
#[test]
fn test_iife_named_args_and_defaults() {
    let out = compile_and_run(
        r#"<?php
echo (function(int $n = 4): int { return $n + 1; })(n: 5);
echo ",";
echo (function(int $n = 4): int { return $n + 1; })();
"#,
    );
    assert_eq!(out, "6,5");
}

// --- Calling closures from array access ---

/// Verifies closure stored in an array and invoked via array-access syntax `$fns[0](5)`.
#[test]
fn test_closure_from_array_call() {
    let out = compile_and_run(
        r#"<?php
$fns = [function($x) { return $x * 10; }];
echo $fns[0](5);
"#,
    );
    assert_eq!(out, "50");
}

/// Verifies parameterless closure stored in an array and invoked via array-access syntax `$fns[0]()`.
#[test]
fn test_closure_from_array_no_args() {
    let out = compile_and_run(
        r#"<?php
$fns = [function() { return 99; }];
echo $fns[0]();
"#,
    );
    assert_eq!(out, "99");
}

// --- Closure returning closure ---

/// Verifies a closure that returns another closure, which is then invoked, confirming proper closure-of-closure codegen.
#[test]
fn test_closure_returning_closure() {
    let out = compile_and_run(
        r#"<?php
$f = function() { return function() { return 99; }; };
$g = $f();
echo $g();
"#,
    );
    assert_eq!(out, "99");
}

/// Verifies a closure factory that returns a closure accepting one argument, which is then called with a value.
#[test]
fn test_closure_returning_closure_with_args() {
    let out = compile_and_run(
        r#"<?php
$maker = function() { return function($x) { return $x * 3; }; };
$fn = $maker();
echo $fn(7);
"#,
    );
    assert_eq!(out, "21");
}

// --- Closures auto-bind $this when defined in an instance method ---

/// Verifies a non-static closure defined in a method auto-captures `$this`, so
/// reading a property through `$this->prop` works without an explicit `use`.
#[test]
fn test_closure_in_method_auto_captures_this_property() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v = 5;
    public function make() {
        return function() { return $this->v + 1; };
    }
}
$c = new C();
$f = $c->make();
echo $f();
"#,
    );
    assert_eq!(out, "6");
}

/// Verifies an arrow function defined in a method auto-captures `$this`.
#[test]
fn test_arrow_in_method_auto_captures_this() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v = 5;
    public function make() {
        return fn() => $this->v * 10;
    }
}
$c = new C();
$f = $c->make();
echo $f();
"#,
    );
    assert_eq!(out, "50");
}

/// Verifies a captured `$this` can call instance methods inside the closure body.
#[test]
fn test_closure_in_method_calls_this_method() {
    let out = compile_and_run(
        r#"<?php
class C {
    public function greet() { return "hi"; }
    public function make() {
        return function() { return $this->greet() . "!"; };
    }
}
$c = new C();
$f = $c->make();
echo $f();
"#,
    );
    assert_eq!(out, "hi!");
}

/// Verifies a closure captures `$this` alongside an explicit `use($var)` capture.
#[test]
fn test_closure_captures_this_and_use_variable() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $base = 100;
    public function make($add) {
        return function() use ($add) { return $this->base + $add; };
    }
}
$c = new C();
$f = $c->make(7);
echo $f();
"#,
    );
    assert_eq!(out, "107");
}

/// Verifies the captured `$this` is the live object: mutations through the
/// closure persist across calls and are visible on the object.
#[test]
fn test_closure_mutates_this_property() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $n = 0;
    public function bump() {
        return function() { $this->n = $this->n + 1; return $this->n; };
    }
}
$c = new C();
$f = $c->bump();
echo $f(), $f(), $f();
"#,
    );
    assert_eq!(out, "123");
}

/// Verifies `$this` flows transitively into a nested closure defined inside an
/// outer closure: each level captures `$this` from the level above.
#[test]
fn test_nested_closures_share_this() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v = 3;
    public function make() {
        return function() {
            $inner = fn() => $this->v * 2;
            return $inner();
        };
    }
}
$c = new C();
$f = $c->make();
echo $f();
"#,
    );
    assert_eq!(out, "6");
}

/// Verifies a closure that returns the captured `$this`'s state keeps the object
/// alive and readable after the defining method has returned.
#[test]
fn test_closure_reads_this_after_method_returns() {
    let out = compile_and_run(
        r#"<?php
class C {
    public string $name = "Ada";
    public function greeter() {
        return function() { return "Hi " . $this->name; };
    }
}
$c = new C();
$g = $c->greeter();
echo $g();
"#,
    );
    assert_eq!(out, "Hi Ada");
}

// --- Closure::bind / ->bindTo() rebind a closure's captured $this ---

/// Verifies `$closure->bindTo($newThis)` returns a closure whose `$this` is the
/// new receiver, leaving the original closure unchanged.
#[test]
fn test_closure_bindto_rebinds_this() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function getter() {
        return function() { return $this->v; };
    }
}
$c1 = new C(7);
$c2 = new C(99);
$f = $c1->getter();
$bound = $f->bindTo($c2);
echo $f(), $bound(), $f();
"#,
    );
    assert_eq!(out, "7997");
}

/// Verifies the static `Closure::bind($closure, $newThis)` form rebinds `$this`.
#[test]
fn test_closure_bind_static_form() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function getter() {
        return function() { return $this->v; };
    }
}
$c1 = new C(7);
$c2 = new C(50);
$f = $c1->getter();
$bound = Closure::bind($f, $c2);
echo $bound();
"#,
    );
    assert_eq!(out, "50");
}

/// Verifies a non-static closure declared inside a static method may reference `$this` and receive
/// its runtime receiver later through `Closure::bind`, while the static method itself has no
/// implicit receiver.
#[test]
fn test_closure_from_static_method_can_bind_this_later() {
    let out = compile_and_run(
        r#"<?php
class StaticClosureFactory {
    public static function make(): Closure {
        return function (): string { return $this->value; };
    }
}
class StaticClosureTarget {
    public string $value = "bound";
}

$closure = Closure::bind(
    StaticClosureFactory::make(),
    new StaticClosureTarget(),
    StaticClosureTarget::class,
);
echo $closure();
"#,
    );
    assert_eq!(out, "bound");
}

/// Verifies the optional `$scope` argument is accepted by both bind spellings.
#[test]
fn test_closure_bind_accepts_scope_argument() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function getter() {
        return function() { return $this->v; };
    }
}
$c1 = new C(1);
$c2 = new C(42);
$f = $c1->getter();
$a = Closure::bind($f, $c2, C::class);
$b = $f->bindTo($c2, C::class);
echo $a(), " ", $b();
"#,
    );
    assert_eq!(out, "42 42");
}

/// Verifies `$closure->call($newThis, ...$args)` binds `$this` and invokes the
/// closure in one step, passing through the trailing arguments.
#[test]
fn test_closure_call_binds_and_invokes() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function adder() {
        return function(int $n) { return $this->v + $n; };
    }
}
$c1 = new C(7);
$c2 = new C(100);
$f = $c1->adder();
echo $f->call($c2, 5);   // 105 — bound to $c2
echo " ";
echo $f->call($c1, 1);   // 8   — bound to $c1
"#,
    );
    assert_eq!(out, "105 8");
}

// --- A closure defined outside a class may reference $this and be bound later ---

/// Verifies a top-level closure that references `$this` can be bound to an object
/// via `Closure::bind`, dispatching member access against the bound object.
#[test]
fn test_top_level_closure_bind_reads_property() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $x = 42;
}
$reader = function() { return $this->x; };
$bound = Closure::bind($reader, new C());
echo $bound();
"#,
    );
    assert_eq!(out, "42");
}

/// Verifies `$this` is rebound from `$newThis` (argument two) and NOT from `$scope` (argument
/// three), which governs visibility only. `A` and the unrelated scope class `D` declare
/// different properties, and the enclosing class `K` declares neither, so reading the bound
/// object's property proves all three candidate classes apart.
#[test]
fn test_closure_bind_this_comes_from_receiver_not_scope() {
    let out = compile_and_run(
        r#"<?php
class A { public string $tag = 'A'; }
class D { public string $other = 'D'; }
class K {
    public function run(): void {
        $a = new A();
        echo \Closure::bind(fn () => $this->tag, $a, D::class)();
    }
}
(new K())->run();
"#,
    );
    assert_eq!(out, "A");
}

/// Verifies the two-argument `Closure::bind($closure, $newThis)` form rebinds `$this` from
/// inside a method. PHP keeps the closure's existing scope when `$scope` is omitted, so a
/// public property on the new receiver stays readable.
#[test]
fn test_closure_bind_two_argument_form_rebinds_this_in_method() {
    let out = compile_and_run(
        r#"<?php
class Loader { public string $tag = 'LOADER'; }
class Kernel {
    public function run(): void {
        $x = new Loader();
        echo \Closure::bind(fn () => $this->tag, $x)();
    }
}
(new Kernel())->run();
"#,
    );
    assert_eq!(out, "LOADER");
}

/// Verifies an early-exit `instanceof` guard proves the rebind target's class for the code that
/// follows it. Both classes declare `$tag`, so an unnarrowed receiver does not fail loudly — it
/// silently reads the enclosing `Kernel`'s value. The differing values make that legible.
#[test]
fn test_closure_bind_receiver_narrowed_by_early_exit_guard() {
    let out = compile_and_run(
        r#"<?php
class Loader { public string $tag = 'LOADER'; }
class Kernel {
    public string $tag = 'KERNEL';
    public function run(object $x): void {
        if (!$x instanceof Loader) {
            throw new \LogicException('nope');
        }
        echo \Closure::bind(fn () => $this->tag, $x, $x)();
    }
}
(new Kernel())->run(new Loader());
"#,
    );
    assert_eq!(out, "LOADER");
}

/// Verifies the canonical scope-stealing pattern: a standalone closure bound to
/// an object reads a private property (visibility is permissive once bound).
#[test]
fn test_top_level_closure_bind_reads_private_property() {
    let out = compile_and_run(
        r#"<?php
class Account {
    private int $balance = 250;
}
$peek = function() { return $this->balance; };
$read = Closure::bind($peek, new Account(), Account::class);
echo $read();
"#,
    );
    assert_eq!(out, "250");
}

/// Verifies a top-level closure that calls a method on `$this` and takes an
/// argument, bound via both `bindTo` and `call`.
#[test]
fn test_top_level_closure_bind_method_and_call() {
    let out = compile_and_run(
        r#"<?php
class Greeter {
    public string $name = "Ada";
    public function hi(): string { return "Hi " . $this->name; }
}
$f = function(string $suffix) { return $this->hi() . $suffix; };
$bound = $f->bindTo(new Greeter());
echo $bound("!");
echo "|";
echo $f->call(new Greeter(), "?");
"#,
    );
    assert_eq!(out, "Hi Ada!|Hi Ada?");
}

// --- __rt_closure_bind generalization: captureless, N captures, static-closure divergence ---

/// Regression test for the `__rt_closure_bind` generalization
/// (`crate::codegen::runtime::callables::closure_bind`): binding a CAPTURELESS closure (no
/// `use(...)`, no implicit `$this`) must succeed and return a working copy, not fatal. The
/// pre-generalization runtime hard-required exactly one `$this` capture.
#[test]
fn test_closure_bind_captureless_closure() {
    let out = compile_and_run(
        r#"<?php
$f = function () { return 42; };
$bound = \Closure::bind($f, null);
echo $bound();
"#,
    );
    assert_eq!(out, "42");
}

/// Regression test: binding a captureless `static` closure with a `null` new `$this` must
/// succeed (php-verified: PHP only rejects a NON-null `$this` on a static closure).
#[test]
fn test_closure_bind_captureless_static_closure_null_this_succeeds() {
    let out = compile_and_run(
        r#"<?php
$f = \Closure::bind(static function () { return 7; }, null);
echo $f();
"#,
    );
    assert_eq!(out, "7");
}

/// Regression test: binding a closure with several by-value captures (int, string) must copy
/// every capture into the new descriptor, not just a single `$this` slot. The pre-generalization
/// runtime fataled on any capture shape other than exactly one `$this`.
#[test]
fn test_closure_bind_multiple_by_value_captures() {
    let out = compile_and_run(
        r#"<?php
function make($a, $b, $c) {
    return function () use ($a, $b, $c) { return "$a-$b-$c"; };
}
$f = make(1, "hi", "z");
$bound = \Closure::bind($f, null);
echo $f(), "|", $bound();
"#,
    );
    assert_eq!(out, "1-hi-z|1-hi-z");
}

/// Regression test: rebinding `$this` on a closure that ALSO has other by-value captures must
/// only touch the `$this` slot, leaving the other captures intact on the bound copy.
#[test]
fn test_closure_bind_this_capture_alongside_other_captures() {
    let out = compile_and_run(
        r#"<?php
class C {
    public int $v;
    public function __construct(int $v) { $this->v = $v; }
    public function adder($n) {
        return function () use ($n) { return $this->v + $n; };
    }
}
$c1 = new C(1);
$c2 = new C(100);
$f = $c1->adder(5);
$bound = \Closure::bind($f, $c2);
echo $f(), " ", $bound();
"#,
    );
    assert_eq!(out, "6 105");
}

/// Regression test (JURY ADDENDUM #4): a by-value capture must NOT alias mutably between the
/// source and bound closures — binding a closure creates an independent copy whose captured
/// (by-value) locals are its OWN, not a pointer shared with the source. Proven with TWO
/// independently-created closures (each holding a different captured string): binding one
/// must not disturb the other's capture. Each closure is invoked exactly once — a pre-existing,
/// unrelated gap where calling the SAME closure a second time to read a captured string returns
/// an empty string instead of the capture (reproduced without any `Closure::bind` involvement:
/// `$f = make("orig"); echo $f(); echo $f();` — the second call already returns "" on `main`)
/// means a second-call assertion on either closure would fail for a reason unrelated to bind.
#[test]
fn test_closure_bind_by_value_capture_independent_across_bound_copies() {
    let out = compile_and_run(
        r#"<?php
function make($label) {
    return function () use ($label) { return $label; };
}
$f = make("orig");
$other = make("other");
$bound = \Closure::bind($f, null);
echo $bound();
echo $other();
"#,
    );
    assert_eq!(out, "origother");
}

/// Regression test (JURY ADDENDUM #4): a by-reference capture (`use (&$x)`) SHARES the same
/// storage between the source and bound closures — mutating through either is visible through
/// both (php-verified: interleaved calls each advance the SAME shared counter).
#[test]
fn test_closure_bind_by_ref_capture_shares_storage() {
    let out = compile_and_run(
        r#"<?php
function counter() {
    $n = 0;
    $inc = function () use (&$n) { return ++$n; };
    $bound = \Closure::bind($inc, null);
    return $inc() . " " . $bound() . " " . $inc() . " " . $bound();
}
echo counter();
"#,
    );
    assert_eq!(out, "1 2 3 4");
}

/// Regression test (JURY ADDENDUM #5): binding a NON-null `$this` onto a `static` closure must
/// not fatal or crash the process; PHP itself only warns and returns `?Closure`'s `null` arm.
/// elephc's `__rt_closure_bind` returns a null descriptor for this case (documented divergence:
/// the result stays statically typed `Callable`, so `=== null` cannot observe it at the language
/// level yet — see the runtime-return-value assertion below, which observes the divergence the
/// way that IS currently reachable: the process does not crash and execution continues normally
/// past the rejected bind).
#[test]
fn test_closure_bind_static_closure_non_null_this_does_not_crash() {
    let out = compile_and_run(
        r#"<?php
class C { public int $v = 5; }
$sc = \Closure::bind(static function () { return 1; }, new C());
echo "reached";
"#,
    );
    assert_eq!(out, "reached");
}

/// Regression test: omitting `$scope` (or passing `null`) keeps the closure's original scope —
/// a private property the closure's OWN declaring class can already see remains readable after
/// a `$this`-only rebind with no scope argument.
#[test]
fn test_closure_bind_omitted_scope_keeps_original_scope() {
    let out = compile_and_run(
        r#"<?php
class Account {
    private int $balance;
    public function __construct(int $balance) { $this->balance = $balance; }
    public function peeker() {
        return function () { return $this->balance; };
    }
}
$a1 = new Account(10);
$a2 = new Account(20);
$f = $a1->peeker();
$bound = \Closure::bind($f, $a2);
echo $bound();
"#,
    );
    assert_eq!(out, "20");
}

// --- Closure::bind scope rebind (checker relax, JURY-gated) ---

/// Regression test for the checker scope-rebind relaxation
/// (`crate::types::checker::inference::expr::static_closure::check_closure_bind_call_args`):
/// `Closure::bind($closure, null, Scope::class)` with a literal `$scope` lets a STATIC closure
/// literal read/write a PROTECTED property through a PARAMETER typed as (or a subclass of) the
/// rebound scope — the `ContractsTrait::doGet()` idiom this campaign targets. Before this
/// relaxation, `$item->secret` here was rejected: `$item`'s protected property is inaccessible
/// from the closure's lexically enclosing (top-level) scope.
#[test]
fn test_closure_bind_scope_rebind_allows_protected_property_access_on_param() {
    let out = compile_and_run(
        r#"<?php
class Box {
    protected int $secret;
    public function __construct(int $secret) { $this->secret = $secret; }
}
$reader = \Closure::bind(
    static function (Box $item) {
        return $item->secret;
    },
    null,
    Box::class
);
echo $reader(new Box(42));
"#,
    );
    assert_eq!(out, "42");
}

/// Regression test: the scope rebind also authorizes WRITES to a protected property through an
/// eligible parameter, not just reads.
#[test]
fn test_closure_bind_scope_rebind_allows_protected_property_write_on_param() {
    let out = compile_and_run(
        r#"<?php
class Box {
    protected int $secret = 0;
}
$writer = \Closure::bind(
    static function (Box $item, int $value) {
        $item->secret = $value;
    },
    null,
    Box::class
);
$box = new Box();
$writer($box, 99);
$reader = \Closure::bind(static function (Box $item) { return $item->secret; }, null, Box::class);
echo $reader($box);
"#,
    );
    assert_eq!(out, "99");
}

/// Regression test for Symfony Cache's bound factory closures: a local constructed inside a
/// closure rebound to the constructed class may write that class's protected properties.
#[test]
fn test_closure_bind_scope_rebind_allows_protected_property_write_on_constructed_local() {
    let out = compile_and_run(
        r#"<?php
class BoundCacheItem {
    protected string $key = "";
    public function getKey(): string { return $this->key; }
}
$factory = \Closure::bind(
    static function (string $key): BoundCacheItem {
        $item = new BoundCacheItem();
        $item->key = $key;
        return $item;
    },
    null,
    BoundCacheItem::class
);
echo $factory("cache-key")->getKey();
"#,
    );
    assert_eq!(out, "cache-key");
}

/// Regression test for Symfony Cache's proxy factory: an untyped closure parameter narrowed
/// with `instanceof` to the rebound scope may expose a protected property to a local item.
#[test]
fn test_closure_bind_scope_rebind_allows_protected_property_read_on_narrowed_param() {
    let out = compile_and_run(
        r#"<?php
class BoundProxyItem {
    protected string $secret;
    public function __construct(string $secret) { $this->secret = $secret; }
    public function getSecret(): string { return $this->secret; }
}
$copy = \Closure::bind(
    static function ($source): BoundProxyItem {
        $item = new BoundProxyItem("");
        if ($source instanceof BoundProxyItem) {
            $item->secret = $source->secret;
        }
        return $item;
    },
    null,
    BoundProxyItem::class
);
echo $copy(new BoundProxyItem("metadata"))->getSecret();
"#,
    );
    assert_eq!(out, "metadata");
}

/// Regression test: an OMITTED `$scope` argument keeps the closure's ORIGINAL (lexical) scope —
/// no inference from `$newThis`, matching J2's established rule. A top-level closure without a
/// scope rebind still cannot read a protected property through a typed parameter.
#[test]
fn test_closure_bind_omitted_scope_does_not_relax_protected_access() {
    let out = compile_expect_check_error(
        r#"<?php
class Box {
    protected int $secret = 5;
}
$reader = \Closure::bind(static function (Box $item) {
    return $item->secret;
}, null);
echo $reader(new Box());
"#,
    );
    assert!(
        out.contains("Cannot access protected property"),
        "expected a protected-access error without a scope rebind, got: {}",
        out
    );
}

/// Regression test (JURY ADDENDUM #1): a closure body that references `$this` ANYWHERE stays
/// loud even with a literal `$scope` argument — the lexical gate rejects the rebind because
/// `self::`/`static::`/`$this` resolve LEXICALLY at codegen time, not against the rebound scope.
/// This closure is a NON-static closure (so it captures `$this` automatically) whose body reads
/// `$this->outer` — the gate must reject the relaxation and the checker must still reject the
/// otherwise-inaccessible property read.
#[test]
fn test_closure_bind_this_usage_keeps_gate_loud() {
    let out = compile_expect_check_error(
        r#"<?php
class Outer {
    public int $outer = 1;
    public function make() {
        return function (Box $item) {
            return $this->outer + $item->secret;
        };
    }
}
class Box {
    protected int $secret = 5;
}
$outer = new Outer();
$reader = \Closure::bind($outer->make(), $outer, Box::class);
echo $reader(new Box());
"#,
    );
    assert!(
        out.contains("Cannot access protected property"),
        "expected the gate to keep the protected-access error loud when the body uses $this, got: {}",
        out
    );
}

/// Regression test (JURY ADDENDUM #2): the relaxation applies ONLY to the closure's OWN declared
/// PARAMETERS typed as the rebound scope — a captured (`use`) variable of the SAME class does NOT
/// become eligible just because the scope was rebound to its class.
#[test]
fn test_closure_bind_scope_rebind_does_not_extend_to_captured_variables() {
    let out = compile_expect_check_error(
        r#"<?php
class Box {
    protected int $secret = 5;
}
$captured = new Box();
$reader = \Closure::bind(static function () use ($captured) {
    return $captured->secret;
}, null, Box::class);
echo $reader();
"#,
    );
    assert!(
        out.contains("Cannot access protected property"),
        "expected a captured variable to stay outside the relaxed eligibility set, got: {}",
        out
    );
}

/// Regression test (Symfony `MicroKernelTrait::getBuildDir`): the single-`return $this->prop`
/// shape `Closure::bind(fn () => $this->warmupDir, $this, Kernel::class)()` reads a PRIVATE
/// property authorized by the rebound scope. `crate::ir_lower` lowers exactly this shape by
/// boxing `$newThis` as the closure's `$this`; the checker authorizes the `$this` receiver
/// against the literal scope only for this form. php-verified: `/warm` then `/build`.
#[test]
fn test_closure_bind_this_single_property_read_private() {
    let out = compile_and_run(
        r#"<?php
class Kernel {
    private ?string $warmupDir = null;
    public function __construct(?string $w) { $this->warmupDir = $w; }
    public function getBuildDir(): string { return "/build"; }
}
class AppKernel extends Kernel {
    public function probe(): string {
        return \Closure::bind(fn () => $this->warmupDir, $this, Kernel::class)() ?? $this->getBuildDir();
    }
}
echo (new AppKernel("/warm"))->probe(), (new AppKernel(null))->probe();
"#,
    );
    assert_eq!(out, "/warm/build");
}

/// A `$this`-using `Closure::bind` body that is not the single-`return $this->prop` shape — here
/// an ARRAY of private property reads, Symfony's `TraceableCommand` idiom — rebinds `$this` to
/// `$newThis` and reads the receiver's own values.
///
/// `$this` is `$newThis` (the `Command` argument), not the lexically enclosing `TraceableCommand`,
/// so the private `$code` read yields the argument's `"x"` and not the enclosing object's `null`.
/// The explicit `Command::class` scope authorizes the private access even though the enclosing
/// class is a subclass. Matches `php -n`.
#[test]
fn test_closure_bind_this_array_body_reads_bound_receiver() {
    let out = compile_and_run(
        r#"<?php
class Command {
    private ?string $code = null;
    public function __construct(?string $c) { $this->code = $c; }
}
class TraceableCommand extends Command {
    public function probe(Command $command): void {
        [$code] = \Closure::bind(fn () => [$this->code], $command, Command::class)();
        echo $code ?? "null";
    }
}
(new TraceableCommand(null))->probe(new Command("x"));
"#,
    );
    assert_eq!(out, "x");
}

/// The rebound `$this` of a non-single-property bind body reaches a receiver whose class the
/// enclosing scope is unrelated to: `Reader` declares neither `$a` nor `$b`, so resolving them
/// against the enclosing class would not compile at all. Guards the `$newThis`-class typing
/// (`set_bound_closure_this_class` on the generic bind path) that makes them resolve.
#[test]
fn test_closure_bind_this_array_body_from_unrelated_class() {
    let out = compile_and_run(
        r#"<?php
class Holder { private $a = 'A1'; private $b = 'B2'; }
class Reader {
    public function read(Holder $h): string {
        [$x, $y] = \Closure::bind(fn () => [$this->a, $this->b], $h, Holder::class)();
        return $x . $y;
    }
}
echo (new Reader())->read(new Holder());
"#,
    );
    assert_eq!(out, "A1B2");
}

/// Regression test (keep-loud boundary): the `$this` single-property bind authorizes the receiver
/// ONLY against the literal `$scope`. A wrong scope (unrelated to the property's declaring class)
/// must stay loud, so the relaxation never blanket-authorizes a private access.
#[test]
fn test_closure_bind_this_single_property_wrong_scope_stays_loud() {
    let out = compile_expect_check_error(
        r#"<?php
class A { private int $secret = 7; public function __construct(int $x) { $this->secret = $x; } }
class Unrelated {}
class Probe extends A {
    public function go(): int {
        return \Closure::bind(fn () => $this->secret, $this, Unrelated::class)();
    }
}
echo (new Probe(9))->go();
"#,
    );
    assert!(
        out.contains("Cannot access private property"),
        "expected a wrong bind scope to keep the private access loud, got: {}",
        out
    );
}

/// Regression test (F3 — Symfony `ViewEvent`/`TraceableCommand` bound-closure shape): the
/// immediate-invoke `Closure::bind(fn () => $this->prop, $newThis, Scope::class)()` shape where
/// `$newThis` is an instance of a class OTHER than the closure's lexically-enclosing method's
/// class. The rebound `$this` must resolve `$this->prop`'s offset against `$newThis`'s layout —
/// here a subclass that inherits a PRIVATE property declared on the rebind scope — not against the
/// enclosing class, which does not declare the property at all. `crate::ir_lower` types the
/// closure body's `$this` as the bound receiver's class (see
/// `crate::ir_lower::expr::build_bound_closure_binding`) and the checker redirects the property
/// resolution to the rebind scope (see
/// `crate::types::checker::inference::objects::access::Checker::infer_property_on_class_type`),
/// keeping the two in lock-step. Mirrors `ViewEvent:41` (private prop inherited by the bound
/// receiver). php-verified oracle: 13.
#[test]
fn test_closure_bind_scope_rebind_reads_inherited_private_on_foreign_receiver() {
    let out = compile_and_run(
        r#"<?php
class Base { public function __construct(private int $hidden = 0) {} }
class Derived extends Base {}
class Outer {
    public function peek(Derived $d): int {
        return \Closure::bind(fn () => $this->hidden, $d, Base::class)();
    }
}
echo (new Outer())->peek(new Derived(13));
"#,
    );
    assert_eq!(out, "13");
}

// --- Untyped closure/arrow-fn parameter is Mixed inside the body (PHP semantics) ---

/// An untyped arrow-function parameter with no contextual hint is `Mixed` inside the body, so
/// indexing it (`$x[0]`) type-checks through the gradual Mixed-index path instead of being
/// rejected as a non-array `Int`. The indexed values flow through a string concat at runtime.
#[test]
fn test_untyped_arrow_param_index_is_mixed() {
    let out = compile_and_run(
        r#"<?php
$idx = fn($x) => $x[0] . $x[1];
echo $idx(['a', 'b', 'c']);
"#,
    );
    assert_eq!(out, "ab");
}

/// The Symfony YAML Unescaper pattern: an untyped arrow-fn callback indexing `$m[0]` is accepted
/// by `preg_replace_callback` (its callback parameter is `Mixed`), and the match group flows into
/// a string builtin. Previously the untyped parameter defaulted to `Int`, so `$m[0]` was rejected
/// and the callback had no statically known signature.
#[test]
fn test_untyped_callback_param_index_preg_replace_callback() {
    let out = compile_and_run(
        r#"<?php
$cb = fn($m) => strtoupper($m[0]);
echo preg_replace_callback('/[a-z]/', $cb, "abc");
"#,
    );
    assert_eq!(out, "ABC");
}

/// Verifies `preg_replace_callback()` accepts the optional fourth `$limit` argument
/// (PHP 3–6 arity). The subject has a single match so the observable output is
/// identical whether or not the runtime honors `$limit` (limit semantics deferred).
/// Cross-check: `php -r 'echo preg_replace_callback('/\d/', fn($m)=>$m[0]*2, "a1b", 1);'`
/// yields "a2b".
#[test]
fn test_preg_replace_callback_four_args_with_limit() {
    let out = compile_and_run(
        r#"<?php
echo preg_replace_callback('/\d/', fn($m) => $m[0] * 2, "a1b", 1);
"#,
    );
    assert_eq!(out, "a2b");
}

/// An untyped-parameter arithmetic closure passed to `array_map` keeps working: `array_map`
/// exposes the element type so the parameter is typed precisely (the contextual-hint path is
/// unaffected by the no-hint Mixed fallback), and the multiplication lowers correctly.
#[test]
fn test_untyped_arrow_param_arithmetic_array_map_unaffected() {
    let out = compile_and_run(
        r#"<?php
$r = array_map(fn($n) => $n * 2, [1, 2, 3]);
echo $r[0], $r[1], $r[2];
"#,
    );
    assert_eq!(out, "246");
}

// -- callable-sig registry cross-contamination fix (cycle-4 I3 / PART B2) --
//
// Regression coverage for the confirmed collision: `Checker::callable_sigs`/
// `closure_return_types`/`callable_param_names` (keyed only by LOCAL VARIABLE NAME, e.g.
// `$callback`) previously leaked across every function/method body check — methods had NO
// scoping at all (unlike free functions, which only scoped their OWN declared callable params),
// so a closure assigned to a same-named local in one method/function silently contaminated the
// next one checked. See `Checker::enter_callable_var_scope`/`exit_callable_var_scope` in
// `src/types/checker/mod.rs`.

/// Verifies the exact reproduced collision shape: two UNRELATED classes each declare a
/// `run(callable $callback)` method (same method name, same parameter name), invoked with
/// DIFFERENTLY-shaped closures. Before the fix, checking `Beta::run` (or a later, unrelated
/// function/method reusing the name `$callback`) could silently validate its closure invocation
/// against `Alpha::run`'s closure signature instead of its own, either rejecting a valid call or
/// accepting/miscompiling an invalid one.
#[test]
fn test_callable_param_sig_no_cross_class_same_method_name_collision() {
    let out = compile_and_run(
        r#"<?php
class Alpha {
    public function run(callable $callback): string {
        return $callback("hello");
    }
}
class Beta {
    public function run(callable $callback): int {
        return $callback(5);
    }
}
$a = new Alpha();
echo $a->run(function (string $s): string { return strtoupper($s); });
echo "|";
$b = new Beta();
echo $b->run(function (int $n): int { return $n * 2; });
"#,
    );
    assert_eq!(out, "HELLO|10");
}

/// Same collision shape as `test_callable_param_sig_no_cross_class_same_method_name_collision`,
/// but through a TRAIT method flattened into two unrelated classes (`class.name` — the
/// declaring/flattened-owner class — is the cross-call cache key qualifier for a trait-flattened
/// method, per the JURY ADDENDUM). Each class's flattened copy of `run` must specialize its OWN
/// `$callback` parameter independently.
#[test]
fn test_callable_param_sig_no_cross_trait_flattened_method_collision() {
    let out = compile_and_run(
        r#"<?php
trait RunnerTrait {
    public function run(callable $callback): int {
        return $callback(3);
    }
}
class First {
    use RunnerTrait;
}
class Second {
    use RunnerTrait;
}
$f = new First();
echo $f->run(function (int $n): int { return $n + 100; });
echo "|";
$s = new Second();
echo $s->run(function (int $n): int { return $n * 10; });
"#,
    );
    assert_eq!(out, "103|30");
}

/// Verifies a closure assigned to an ORDINARY (non-parameter) local variable inside one method
/// does not leak into a different method that happens to declare a local of the same name —
/// the actual mechanism behind the confirmed `--web` collision (a `Symfony\Component\Yaml\
/// Unescaper`-shaped closure-in-a-local leaking into an unrelated `PhpFileLoader`-shaped
/// `callable $callback` parameter). `Gamma::pick` assigns `$callback` to a `string`-returning
/// closure with an untyped param; `Delta::pick` independently assigns `$callback` to an
/// `int`-returning closure. Checking them in sequence must not let either specialization bleed
/// into the other.
#[test]
fn test_callable_local_variable_sig_no_cross_method_collision() {
    let out = compile_and_run(
        r#"<?php
class Gamma {
    public function pick(): string {
        $callback = function ($match) { return strtoupper($match); };
        return $callback("hi");
    }
}
class Delta {
    public function pick(): int {
        $callback = function ($match) { return $match + 1; };
        return $callback(41);
    }
}
$g = new Gamma();
echo $g->pick();
echo "|";
$d = new Delta();
echo $d->pick();
"#,
    );
    assert_eq!(out, "HI|42");
}

/// Verifies that `isset($this)` inside a `static` closure evaluates to `false`
/// because static closures have no `$this` binding (issue #359).
#[test]
fn test_static_closure_isset_this_returns_false() {
    let out = compile_and_run(
        r#"<?php
$f = static function(): bool { return isset($this); };
echo $f() ? "true" : "false";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies that `isset($this)` inside a static closure defined in an instance
/// method still evaluates to `false` (issue #359).
#[test]
fn test_static_closure_isset_this_false_even_in_method() {
    let out = compile_and_run(
        r#"<?php
class C {
    public function m(): void {
        $f = static function(): bool { return isset($this); };
        echo $f() ? "true" : "false";
    }
}
(new C())->m();
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies that `isset($this, $x)` in a static closure returns `false` because
/// `$this` is unset, making the AND of all isset arguments false (issue #359).
#[test]
fn test_static_closure_isset_this_with_other_set_var() {
    let out = compile_and_run(
        r#"<?php
$x = 1;
$f = static function(): bool { return isset($this, $x); };
echo $f() ? "true" : "false";
"#,
    );
    assert_eq!(out, "false");
}

/// Verifies that `isset($this)` in a non-static closure inside an instance
/// method evaluates to `true` because non-static closures auto-bind `$this`
/// (issue #359).
#[test]
fn test_non_static_closure_isset_this_true_in_method() {
    let out = compile_and_run(
        r#"<?php
class C {
    public function m(): bool {
        $f = function(): bool { return isset($this); };
        return $f();
    }
}
echo (new C())->m() ? "true" : "false";
"#,
    );
    assert_eq!(out, "true");
}

/// Verifies that a recursive closure capturing itself by reference with
/// `use(&$f)` compiles and runs correctly (issue #382).
#[test]
fn test_recursive_closure_factorial() {
    let out = compile_and_run(
        r#"<?php
$f = function (int $n) use (&$f): int {
    return $n <= 1 ? 1 : $n * $f($n - 1);
};
echo $f(5);
"#,
    );
    assert_eq!(out, "120");
}

/// Verifies a recursive closure computing Fibonacci numbers (issue #382).
#[test]
fn test_recursive_closure_fibonacci() {
    let out = compile_and_run(
        r#"<?php
$fib = function (int $n) use (&$fib): int {
    return $n < 2 ? $n : $fib($n - 1) + $fib($n - 2);
};
echo $fib(10);
"#,
    );
    assert_eq!(out, "55");
}

/// Verifies an inferred closure array return remains available through a local callable invocation.
#[test]
fn test_local_closure_infers_array_return_after_loop_mutation() {
    let out = compile_and_run(
        r#"<?php
function segmentCount(string $path): int {
    $splitPath = static function ($path) {
        $result = [];

        foreach (explode("/", trim($path, "/")) as $segment) {
            if (".." === $segment) {
                array_pop($result);
            } elseif ("." !== $segment && "" !== $segment) {
                $result[] = $segment;
            }
        }

        return $result;
    };

    $segments = $splitPath($path);
    return count($segments);
}

echo segmentCount("/a/b/../c");
"#,
    );
    assert_eq!(out, "2");
}

/// Verifies the dynamic-method first-class-callable forms `$obj->$name(...)` (instance),
/// `Class::$name(...)` (named-class static) and `$cls::$name(...)` (runtime class-name static) —
/// where the method name is a runtime value — each build a genuine `Closure` that dispatches by the
/// runtime-resolved method name. Exercises two static-method closures in one scope that share the
/// method-name capture with distinct receivers (a class-constant one and a runtime-string one): the
/// shapes that regress when the desugar collapses its synthesized nodes onto one span key.
#[test]
fn test_dynamic_method_first_class_callable() {
    let out = compile_and_run(
        r#"<?php
class Greeter {
    public function greet(string $x): string { return "hi $x"; }
    public static function shout(string $x): string { return "HI $x"; }
}
class Other {
    public static function shout(string $x): string { return "YO $x"; }
}

$obj = new Greeter();
$m = "greet";
$instance = $obj->$m(...);

$sm = "shout";
$viaClassConst = Greeter::$sm(...);
$cls = "Other";
$viaVar = $cls::$sm(...);

echo ($instance instanceof Closure ? "C" : "?"), ":",
     $instance("bob"), ":",
     $viaClassConst("a"), ":",
     $viaVar("b");
"#,
    );
    assert_eq!(out, "C:hi bob:HI a:YO b");
}
