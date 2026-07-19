//! Purpose:
//! Integration or regression tests for diagnostic coverage of callables, including call user func wrong args, function exists wrong args, and call non callable variable.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

/// Verifies that error call user func wrong args.
#[test]
fn test_error_call_user_func_wrong_args() {
    // Verifies `call_user_func()` with no arguments produces a diagnostic about
    // requiring at least 1 argument.
    expect_error(
        r#"<?php call_user_func();"#,
        "call_user_func() takes at least 1 argument",
    );
}

/// php-verified: real PHP fatals at runtime with "func_get_args() cannot be called from the
/// global scope". elephc rejects this at compile time instead (a native AOT compiler has no
/// runtime "global scope" concept to fall back to), with an equivalent message.
#[test]
fn test_error_func_get_args_global_scope() {
    expect_error(
        r#"<?php func_get_args();"#,
        "Cannot call func_get_args() from the global scope",
    );
}

/// Companion to `test_error_func_get_args_global_scope` for `func_num_args()`.
#[test]
fn test_error_func_num_args_global_scope() {
    expect_error(
        r#"<?php func_num_args();"#,
        "Cannot call func_num_args() from the global scope",
    );
}

/// Gated dynamic-invoker form: first-class-callable syntax (`f(...)`) creates a generic
/// callable descriptor invoked later through the uniform-invoke ABI, which does not know
/// about an arity-hungry function's hidden trailing arity-count parameter — refused as a
/// normal compile-time diagnostic (checked in
/// `Checker::resolve_first_class_callable_sig`) rather than reaching EIR lowering.
#[test]
fn test_error_func_num_args_first_class_callable() {
    expect_error(
        r#"<?php
function f($a, $b = 2) { echo func_num_args(); }
$fn = f(...);
$fn(1, 2, 3);
"#,
        "cannot be used as a first-class callable",
    );
}

/// Gated call shape: a spread argument whose length is NOT statically known (a variable, not
/// a literal array) into an arity-hungry function's call site. A statically-sized spread
/// (`f(...[7, 8])`) IS supported — see
/// `codegen::callables::func_args_intrinsics::test_func_num_args_static_spread_call_is_counted`.
#[test]
fn test_error_func_num_args_dynamic_spread() {
    expect_error(
        r#"<?php
function f($a, $b = 2) { echo func_num_args(); }
$args = [1, 2, 3];
f(...$args);
"#,
        "cannot be called with a dynamic-length spread",
    );
}

/// Gated call shape: a literal-named callback applied PER-ELEMENT of a runtime array (e.g.
/// `array_map`) is lowered through a path that cannot append the hidden arity-count operand.
/// `call_user_func_array()` with a literal name and array IS supported (a different, static
/// argument-list lowering path) — see
/// `codegen::callables::func_args_intrinsics::test_func_num_args_through_call_user_func_array_literal_name`.
#[test]
fn test_error_func_num_args_array_map_callback_literal_name() {
    expect_error(
        r#"<?php
function dyn($a, $b = 2) { echo func_num_args(); }
array_map('dyn', [1, 2, 3]);
"#,
        "cannot be used as a callback for array_map() callback",
    );
}

/// Gated surface: a method body calling `func_num_args()`/`func_get_args()`/`func_get_arg()`
/// is rejected at compile time (methods are never marked arity-hungry — virtual dispatch
/// means a call site cannot always know which concrete override runs).
#[test]
fn test_error_func_num_args_in_method_body() {
    expect_error(
        r#"<?php
class C {
    function m($a) { return func_num_args(); }
}
(new C())->m(1);
"#,
        "this compiler does not support in methods",
    );
}

/// Verifies that error function exists wrong args.
#[test]
fn test_error_function_exists_wrong_args() {
    // Verifies `function_exists()` with no arguments produces a diagnostic about
    // requiring exactly 1 argument.
    expect_error(
        r#"<?php function_exists();"#,
        "function_exists() takes exactly 1 argument",
    );
}

// `class_exists()`/`interface_exists()`/`trait_exists()` with a non-literal name
// or a non-literal autoload flag are no longer AOT-mode errors: they compile to
// a closed-world `__rt_class_exists`/`__rt_interface_exists`/`__rt_trait_exists`
// registry lookup, cloning the `enum_exists` non-literal path. See the positive
// codegen coverage in `tests/codegen/casts_and_constants/introspection.rs`.

/// Verifies that error interface exists wrong args.
#[test]
fn test_error_interface_exists_wrong_args() {
    // Verifies `interface_exists()` with no arguments produces a diagnostic about
    // requiring 1 or 2 arguments.
    expect_error(
        r#"<?php interface_exists();"#,
        "interface_exists() takes 1 or 2 arguments",
    );
}

/// Verifies that error trait exists wrong args.
#[test]
fn test_error_trait_exists_wrong_args() {
    // Verifies `trait_exists()` with no arguments produces a diagnostic about
    // requiring 1 or 2 arguments.
    expect_error(
        r#"<?php trait_exists();"#,
        "trait_exists() takes 1 or 2 arguments",
    );
}

/// Verifies that error enum exists wrong args.
#[test]
fn test_error_enum_exists_wrong_args() {
    // Verifies `enum_exists()` with no arguments produces a diagnostic about
    // requiring 1 or 2 arguments.
    expect_error(
        r#"<?php enum_exists();"#,
        "enum_exists() takes 1 or 2 arguments",
    );
}

/// Verifies that error class implements wrong args.
#[test]
fn test_error_class_implements_wrong_args() {
    expect_error(
        r#"<?php class_implements();"#,
        "class_implements() takes 1 or 2 arguments",
    );
}

// `class_implements()`/`class_parents()`/`class_uses()` with a non-literal string
// name or an object argument are no longer AOT-mode errors: they compile to a
// closed-world `__rt_class_implements`/`__rt_class_parents`/`__rt_class_uses`
// per-class relation registry lookup, cloning the `class_exists` non-literal
// path. See the positive codegen coverage in
// `tests/codegen/spl/introspection.rs`.

/// Verifies that error class implements requires object or string.
#[test]
fn test_error_class_implements_requires_object_or_string() {
    expect_error(
        r#"<?php class_implements(42);"#,
        "class_implements() first argument must be an object or string in AOT mode",
    );
}

/// Verifies that error class parents requires literal autoload flag.
#[test]
fn test_error_class_parents_requires_literal_autoload_flag() {
    expect_error(
        r#"<?php $autoload = true; class_parents("DateTime", $autoload);"#,
        "class_parents() autoload argument must be a literal bool or int in AOT mode",
    );
}

/// Verifies that error class uses wrong args.
#[test]
fn test_error_class_uses_wrong_args() {
    expect_error(
        r#"<?php class_uses("DateTime", true, false);"#,
        "class_uses() takes 1 or 2 arguments",
    );
}

/// Verifies that error get class wrong args.
#[test]
fn test_error_get_class_wrong_args() {
    // Verifies `get_class()` with a second argument produces a diagnostic about
    // accepting at most 1 argument.
    expect_error(
        r#"<?php class Box {} $box = new Box(); get_class($box, $box);"#,
        "get_class() takes at most 1 argument",
    );
}

/// Verifies that error get parent class wrong args.
#[test]
fn test_error_get_parent_class_wrong_args() {
    // Verifies `get_parent_class()` with a second argument produces a diagnostic
    // about accepting at most 1 argument.
    expect_error(
        r#"<?php class Box {} $box = new Box(); get_parent_class($box, $box);"#,
        "get_parent_class() takes at most 1 argument",
    );
}

/// Verifies that error is subclass of wrong args.
#[test]
fn test_error_is_subclass_of_wrong_args() {
    // Verifies `is_subclass_of()` with only 1 argument produces a diagnostic
    // about requiring 2 or 3 arguments.
    expect_error(
        r#"<?php is_subclass_of("Child");"#,
        "is_subclass_of() takes 2 or 3 arguments",
    );
}

/// Verifies that error is a wrong args.
#[test]
fn test_error_is_a_wrong_args() {
    // Verifies `is_a()` with only 1 argument produces a diagnostic about
    // requiring 2 or 3 arguments.
    expect_error(
        r#"<?php is_a("Child");"#,
        "is_a() takes 2 or 3 arguments",
    );
}

/// Verifies that error get declared classes wrong args.
#[test]
fn test_error_get_declared_classes_wrong_args() {
    // Verifies `get_declared_classes()` with an extra argument produces a
    // diagnostic about accepting no arguments.
    expect_error(
        r#"<?php get_declared_classes("extra");"#,
        "get_declared_classes() takes no arguments",
    );
}

/// Verifies that error get declared interfaces wrong args.
#[test]
fn test_error_get_declared_interfaces_wrong_args() {
    // Verifies `get_declared_interfaces()` with an extra argument produces a
    // diagnostic about accepting no arguments.
    expect_error(
        r#"<?php get_declared_interfaces("extra");"#,
        "get_declared_interfaces() takes no arguments",
    );
}

/// Verifies that error get declared traits wrong args.
#[test]
fn test_error_get_declared_traits_wrong_args() {
    // Verifies `get_declared_traits()` with an extra argument produces a
    // diagnostic about accepting no arguments.
    expect_error(
        r#"<?php get_declared_traits("extra");"#,
        "get_declared_traits() takes no arguments",
    );
}

/// Verifies that error class alias rejects runtime call shape.
#[test]
fn test_error_class_alias_rejects_runtime_call_shape() {
    // Verifies `class_alias()` with a runtime variable as the second argument
    // produces a diagnostic because only top-level statements with literal
    // class names are supported in AOT mode.
    expect_error(
        r#"<?php class Original {} $alias = "Alias"; class_alias("Original", $alias);"#,
        "class_alias() is only supported as a top-level statement with literal class names",
    );
}

// --- Closure / arrow function errors ---

/// Verifies that error call non callable variable.
#[test]
fn test_error_call_non_callable_variable() {
    // Verifies invoking a non-callable variable (integer) produces a "not a callable"
    // diagnostic at runtime.
    expect_error(r#"<?php $x = 5; $x(1);"#, "not a callable");
}

/// Verifies that calling a non-callable integer expression is rejected.
///
/// Genuine-error guard for the expression-call site: a concrete `Int` receiver
/// must still produce a "not a callable" diagnostic after the gradual
/// acceptance helper is introduced.
#[test]
fn test_error_call_non_callable_int_expression() {
    // Verifies invoking an integer literal expression produces a "not a callable"
    // diagnostic at the expression-call site.
    expect_error(r#"<?php echo (5)();"#, "not a callable");
}

/// Verifies that error call user func ref param requires variable.
#[test]
fn test_error_call_user_func_ref_param_requires_variable() {
    // Verifies `call_user_func()` with a closure that has a by-reference
    // parameter and a non-variable argument produces a diagnostic requiring
    // a variable to be passed.
    expect_error(
        "<?php function bump(&$n) { $n = $n + 1; } $f = bump(...); call_user_func($f, 1);",
        "parameter $n must be passed a variable",
    );
}

/// Verifies that error call user func string literal ref param requires variable.
#[test]
fn test_error_call_user_func_string_literal_ref_param_requires_variable() {
    // Verifies `call_user_func()` with a named function string and a by-reference
    // parameter passed a non-variable argument produces a diagnostic requiring
    // a variable to be passed.
    expect_error(
        "<?php function bump(&$n) { $n = $n + 1; } call_user_func(\"bump\", 1);",
        "parameter $n must be passed a variable",
    );
}

/// Verifies that error case insensitive function string introspection keeps callback checks.
#[test]
fn test_error_case_insensitive_function_string_introspection_keeps_callback_checks() {
    // Verifies that case-insensitive function string introspection via
    // `function_exists("BUMP")` and `is_callable("BUMP")` still enforces
    // by-reference parameter semantics when `call_user_func("BUMP", ...)` is
    // subsequently invoked.
    expect_error(
        "<?php function Bump(&$n) { $n = $n + 1; } if (function_exists(\"BUMP\") && is_callable(\"BUMP\")) { call_user_func(\"BUMP\", 1); }",
        "parameter $n must be passed a variable",
    );
}

/// Verifies that error closure return type rejects mismatch.
#[test]
fn test_error_closure_return_type_rejects_mismatch() {
    // Verifies a closure with an explicit return type that returns a mismatched
    // type produces a diagnostic showing the expected and actual types.
    expect_error(
        "<?php $f = function(): string { return 1; };",
        "Closure return type expects Str, got Int",
    );
}

/// Verifies that error arrow return type rejects mismatch.
#[test]
fn test_error_arrow_return_type_rejects_mismatch() {
    // Verifies an arrow function with an explicit return type that returns a
    // mismatched type produces a diagnostic showing the expected and actual types.
    expect_error(
        "<?php $f = fn(): int => \"nope\";",
        "Closure return type expects Int, got Str",
    );
}

/// Verifies that error closure return type requires return value.
#[test]
fn test_error_closure_return_type_requires_return_value() {
    // Verifies a closure with an explicit return type and an empty body (no return)
    // produces a diagnostic about every path needing to return a value.
    expect_error(
        "<?php $f = function(): int { };",
        "Closure must return a value on every path",
    );
}

/// Verifies that error closure return type rejects partial fallthrough.
#[test]
fn test_error_closure_return_type_rejects_partial_fallthrough() {
    // Verifies a closure with an explicit return type where only some branches
    // return a value (missing return in else branch) produces a diagnostic
    // about every path needing to return a value.
    expect_error(
        "<?php $f = function(bool $ok): int { if ($ok) { return 1; } };",
        "Closure must return a value on every path",
    );
}

/// Verifies that error closure return type rejects bare return.
#[test]
fn test_error_closure_return_type_rejects_bare_return() {
    // Verifies a closure with `mixed` return type and a bare `return;` (no value)
    // produces a diagnostic about needing to return a value of the specified type.
    expect_error(
        "<?php $f = function(): mixed { return; };",
        "Closure return type must return a value of type",
    );
}

/// Verifies that error closure void return type rejects value.
#[test]
fn test_error_closure_void_return_type_rejects_value() {
    // Verifies a closure with `void` return type that returns a value produces
    // a diagnostic about not returning a value.
    expect_error(
        "<?php $f = function(): void { return 1; };",
        "Closure return type must not return a value",
    );
}

/// Verifies that error fiber callback rejects too many start args.
#[test]
fn test_error_fiber_callback_rejects_too_many_start_args() {
    // Verifies a `Fiber` with a callback accepting 8 start arguments produces
    // a diagnostic because Fibers support at most 7 start arguments.
    expect_error(
        "<?php $fiber = new Fiber(function($a, $b, $c, $d, $e, $f, $g, $h): void {});",
        "Fiber callbacks support at most 7 start arguments, got 8",
    );
}

/// Verifies that error fiber callback rejects by ref start arg.
#[test]
fn test_error_fiber_callback_rejects_by_ref_start_arg() {
    // Verifies a `Fiber` with a callback that receives a start argument
    // by reference produces a diagnostic because by-reference start args
    // are not supported.
    expect_error(
        "<?php $fiber = new Fiber(function(&$value): void {});",
        "Fiber callbacks cannot receive start arguments by reference",
    );
}

// --- PHP 8.5 pipe operator ---

/// Verifies that error pipe rhs integer not callable.
#[test]
fn test_error_pipe_rhs_int_not_callable() {
    // Verifies the pipe operator (`|>`) with a plain integer on the right-hand
    // side produces a "must be a callable" diagnostic.
    expect_error(
        "<?php $r = 5 |> 42;",
        "must be a callable",
    );
}

/// Verifies that error pipe rhs string literal not callable.
#[test]
fn test_error_pipe_rhs_string_literal_not_callable() {
    // Verifies the pipe operator (`|>`) with a bare string literal on the RHS
    // produces a "must be a callable" diagnostic because string literals are
    // treated as `Str`, not `Callable`, at compile time.
    expect_error(
        "<?php $r = 5 |> \"strlen\";",
        "must be a callable",
    );
}

/// Verifies that error pipe rejects by ref parameter.
#[test]
fn test_error_pipe_rejects_by_ref_parameter() {
    // Verifies the pipe operator (`|>`) with a function that has by-reference
    // parameters produces a diagnostic because by-reference parameters are not
    // supported with the pipe operator.
    expect_error(
        "<?php function bump(int &$n): int { return ++$n; } $r = 1 |> bump(...);",
        "by-reference parameters",
    );
}

/// Verifies that error pipe target requires more than one required arg.
#[test]
fn test_error_pipe_target_requires_more_than_one_required_arg() {
    // Verifies the pipe operator (`|>`) with a callable that requires more than
    // one argument and is called without sufficient arguments produces a diagnostic
    // showing the expected vs received argument count.
    expect_error(
        "<?php function pair(int $a, int $b): int { return $a + $b; } $r = 1 |> pair(...);",
        "expects 2 arguments, got 1",
    );
}

/// Verifies that error pipe closure literal requires two args.
#[test]
fn test_error_pipe_closure_literal_requires_two_args() {
    // Verifies the pipe operator (`|>`) with a closure literal that expects two
    // arguments but receives only one (via the pipe's left-hand side) produces
    // a diagnostic showing the expected vs received argument count.
    expect_error(
        "<?php $r = 1 |> (function(int $a, int $b): int { return $a + $b; });",
        "pipe target expects 2 arguments, got 1",
    );
}

/// Verifies that error pipe closure literal rejects by ref parameter.
#[test]
fn test_error_pipe_closure_literal_rejects_by_ref_parameter() {
    // Verifies the pipe operator (`|>`) with a closure literal containing a
    // by-reference parameter produces a diagnostic because by-reference
    // parameters are not supported with the pipe operator.
    expect_error(
        "<?php $r = 1 |> (function(&$n): int { return $n; });",
        "Pipe operator does not support by-reference parameters",
    );
}

/// Verifies that error pipe closure literal typed parameter mismatch.
#[test]
fn test_error_pipe_closure_literal_typed_parameter_mismatch() {
    // Verifies the pipe operator (`|>`) with a closure literal that has a typed
    // parameter where the piped value's type does not match produces a diagnostic
    // showing the expected vs actual parameter type.
    expect_error(
        r#"<?php $r = "nope" |> (function(int $n): int { $copy = $n; return $copy; });"#,
        "pipe target parameter $n expects Int, got Str",
    );
}

/// Verifies the `$this(...)`/`__invoke` first-class-callable relaxation stays scoped: a genuine
/// method typo used as an FCC (`$this->genuinelyMissingMethod(...)`) still produces the
/// "Undefined method for first-class callable" diagnostic. Only `__invoke` is permissively
/// accepted on a class that lacks it, so typo detection for every other method is preserved.
#[test]
fn test_error_this_fcc_missing_method_still_reported() {
    expect_error(
        r#"<?php
class Foo {
    public function real(): void {}
    public function go(): void {
        $cb = $this->genuinelyMissingMethod(...);
    }
}
$f = new Foo();
$f->go();
"#,
        "Undefined method for first-class callable",
    );
}

/// Verifies that a dynamic `$stringVar::cases()` on an unresolved class infers `array<mixed>`
/// so `array_column()` accepts it (R5). The receiver is a `string` class-name that cannot be
/// resolved to the concrete enum at compile time.
#[test]
fn test_dynamic_cases_call_infers_array_for_array_column() {
    expect_ok(
        r#"<?php
enum E: string { case A = 'a'; case B = 'b'; }
function f(string $c) { return array_column($c::cases(), 'value'); }
"#,
    );
}

/// Verifies the R5 gate infers the concrete `array<mixed>` (not the gradual `Mixed`) for a
/// dynamic `cases()`: returning it from an `int`-typed function is a type error, proving the
/// result is an array.
#[test]
fn test_dynamic_cases_call_result_is_array_not_mixed() {
    expect_error(
        r#"<?php
enum E: string { case A = 'a'; }
function f(string $c): int { return $c::cases(); }
"#,
        "got Array",
    );
}

/// Verifies the R5 gate is strictly `cases`-only: a dynamic non-`cases` static call on an
/// unresolved receiver keeps its pre-existing gradual `Mixed` inference (accepted here as an
/// `int` return), so the special case does not widen unrelated dynamic dispatch.
#[test]
fn test_dynamic_non_cases_call_stays_gradual_mixed() {
    expect_ok(
        r#"<?php
function g(string $c): int { return $c::nonexistent(); }
"#,
    );
}

/// Verifies the R5 gate does not mask genuine errors on a *resolved* receiver: calling an
/// undefined static method on a known enum still reports `Undefined method`.
#[test]
fn test_resolved_static_undefined_method_still_errors() {
    expect_error(
        r#"<?php
enum E: string { case A = 'a'; }
echo E::nonexistent();
"#,
        "Undefined method: E::nonexistent",
    );
}

/// Campaign H1 PART A: a callable-variable invocation with TOO FEW arguments stays a loud
/// compile error ("Too few arguments" is a real PHP `ArgumentCountError`, php -n verified) even
/// though the same path now tolerates EXTRA arguments (see
/// `test_callable_variable_tolerates_extra_positional_args` in
/// `tests/codegen/oop/callables/functions_and_builtins.rs`). Only the upper bound was relaxed.
#[test]
fn test_error_callable_variable_too_few_args_stays_loud() {
    expect_error(
        r#"<?php
$cb = function ($i) { return $i; };
echo $cb();
"#,
        "expects 1 arguments, got 0",
    );
}
