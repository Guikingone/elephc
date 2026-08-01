//! Purpose:
//! Integration tests for `func_num_args()`, `func_get_args()`, and `func_get_arg()` — the
//! hidden arity-count/variadic-tail ABI extension threaded through direct free-function
//! calls and closed-world-unambiguous methods (see `crate::types::checker::func_args_scan`).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every case here was php-verified with `php -n` before being encoded (see the campaign
//!   notes in `src/types/checker/func_args_scan/mod.rs`); assertions match PHP's actual
//!   `func_num_args()`/`func_get_args()` counting rules, including named-argument mid-gap
//!   default fills and trailing-optional exclusion.
//! - Functions that are NOT arity-hungry must stay completely unaffected (zero-cost rule);
//!   `test_untouched_function_unaffected_by_feature` is a direct regression guard for that.

use crate::support::*;

/// Verifies `func_num_args()` counts exactly the arguments passed, not the declared count.
#[test]
fn test_func_num_args_basic_counts() {
    let out = compile_and_run(
        r#"<?php
function f($a, $b = 2) {
    echo func_num_args(), ",";
}
f(1);
f(1, 2);
f(1, 2, 3);
"#,
    );
    assert_eq!(out, "1,2,3,");
}

/// Verifies `func_get_args()` returns the passed values, in declaration order, including
/// values beyond the declared parameter count via the (synthesized) variadic tail.
#[test]
fn test_func_get_args_returns_passed_values_in_order() {
    let out = compile_and_run(
        r#"<?php
function f($a, $b = 2) {
    echo implode(",", func_get_args());
}
f(1, 2, 3);
"#,
    );
    assert_eq!(out, "1,2,3");
}

/// php-verified: named arguments that fill a MID-gap (a later param named explicitly forces
/// an earlier optional param's default to be materialized and counted) are included, but a
/// TRAILING unpassed optional parameter is excluded.
#[test]
fn test_func_num_args_named_arg_mid_gap_fill() {
    let out = compile_and_run(
        r#"<?php
function m($a, $b = 10, $c = 20) {
    echo func_num_args(), ":", implode(",", func_get_args()), "\n";
}
m(a: 1, c: 99);
m(a: 1);
"#,
    );
    assert_eq!(out, "3:1,10,99\n1:1\n");
}

/// Verifies a real (user-declared) variadic parameter combined with `func_get_args()` still
/// reports every passed argument, reusing the existing variadic tail rather than synthesizing
/// a second one.
#[test]
fn test_func_get_args_with_real_variadic_param() {
    let out = compile_and_run(
        r#"<?php
function g($a, ...$rest) {
    echo func_num_args(), ":", implode(",", func_get_args());
}
g(1, 2, 3);
"#,
    );
    assert_eq!(out, "3:1,2,3");
}

/// Verifies `func_get_args()` returns a fresh COPY: mutating the returned array must not
/// affect the original parameter's value (php-verified).
#[test]
fn test_func_get_args_returns_copy_not_alias() {
    let out = compile_and_run(
        r#"<?php
function h($a) {
    $args = func_get_args();
    $args[0] = "mutated";
    echo $a, ":", $args[0];
}
h("original");
"#,
    );
    assert_eq!(out, "original:mutated");
}

/// Verifies `func_get_arg($i)` reads back an in-range argument.
#[test]
fn test_func_get_arg_in_range() {
    let out = compile_and_run(
        r#"<?php
function k($a, $b) {
    echo func_get_arg(0), ",", func_get_arg(1);
}
k(10, 20);
"#,
    );
    assert_eq!(out, "10,20");
}

/// php-verified: `func_get_arg()` past the number of arguments actually passed throws a
/// `ValueError` with PHP's exact static message, catchable like any other exception.
#[test]
fn test_func_get_arg_out_of_range_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
function k($a, $b) {
    try {
        func_get_arg(5);
    } catch (\ValueError $e) {
        echo get_class($e), ":", $e->getMessage();
    }
}
k(1, 2);
"#,
    );
    assert_eq!(
        out,
        "ValueError:func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function"
    );
}

/// php-verified: a negative `func_get_arg()` position throws a different static `ValueError`
/// message than the out-of-range case.
#[test]
fn test_func_get_arg_negative_position_throws_value_error() {
    let out = compile_and_run(
        r#"<?php
function k($a) {
    try {
        func_get_arg(-1);
    } catch (\ValueError $e) {
        echo $e->getMessage();
    }
}
k(1);
"#,
    );
    assert_eq!(
        out,
        "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0"
    );
}

/// Verifies zero-declared-parameter functions accept and count unlimited trailing arguments
/// once they call `func_num_args()` (an edge case that surfaced a checker forward-reference
/// arg-count bug during development — kept as a dedicated regression test).
#[test]
fn test_func_num_args_zero_declared_params() {
    let out = compile_and_run(
        r#"<?php
function z() { echo func_num_args(); }
z(1, 2);
"#,
    );
    assert_eq!(out, "2");
}

/// Verifies a statically-named `call_user_func_array()` call into an arity-hungry function
/// still reports the correct count — this dynamic-but-statically-resolved form is supported
/// (not gated), since it lowers through the same named/positional argument materialization as
/// a direct call.
#[test]
fn test_func_num_args_through_call_user_func_array_literal_name() {
    let out = compile_and_run(
        r#"<?php
function dyn($a, $b = 2) { echo func_num_args(); }
call_user_func_array('dyn', [1, 2, 3]);
"#,
    );
    assert_eq!(out, "3");
}

/// Supported spread shape: a STATICALLY-sized spread (a literal array, or anything the
/// existing static-spread expansion machinery can flatten) is expanded before counting, so
/// it behaves exactly like the equivalent explicit positional arguments — php-verified
/// (`fx(...[7, 8])` matches `php -n`'s `2:7,8`).
#[test]
fn test_func_num_args_static_spread_call_is_counted() {
    let out = compile_and_run(
        r#"<?php
function fx($a, $b = 2) { return func_num_args() . ":" . implode(",", func_get_args()); }
echo fx(...[7, 8]);
"#,
    );
    assert_eq!(out, "2:7,8");
}

/// Verifies constructors and non-overridden instance methods preserve surplus legacy
/// arguments, including the Symfony-compatible zero-declared-parameter method shape.
#[test]
fn test_func_args_intrinsics_in_unambiguous_methods() {
    let out = compile_and_run(
        r#"<?php
class LegacyOptions {
    private array $constructorArgs;

    public function __construct($modern = true) {
        $this->constructorArgs = func_get_args();
    }

    public function constructorCount(): int {
        return count($this->constructorArgs);
    }

    public function legacyFlag(/* bool $enabled = true */): bool {
        return !func_num_args() || func_get_arg(0);
    }
}

$options = new LegacyOptions(true, "deprecated", 7);
echo $options->constructorCount(), ":";
echo $options->legacyFlag(false) ? "on" : "off";
"#,
    );
    assert_eq!(out, "3:off");
}

/// Regression guard: an arity-hungry method reached through a boxed `Mixed` receiver (here
/// an untyped parameter) must still receive its hidden `__fga_argc` operand. The boxed
/// dynamic-dispatch path (`lower_mixed_method_call` → `mixed_method_candidates`) filters
/// candidate classes by ABI arity; that filter reads the EMITTED method ABI (which carries
/// the hidden argc), not the checker-visible signature, so the single closed-world
/// implementation is matched instead of being dropped — a dropped candidate previously
/// mis-emitted a spurious "Call to a member function on null" fatal at runtime.
#[test]
fn test_func_args_intrinsic_in_method_via_mixed_receiver() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public function tally() {
        return func_num_args() . ":" . implode(",", func_get_args());
    }
}
function dispatch($receiver) {
    return $receiver->tally(10, 20, 30);
}
echo dispatch(new Counter());
"#,
    );
    assert_eq!(out, "3:10,20,30");
}

// Gated dynamic-invoker forms (first-class-callable syntax, dynamic-length spread, and a
// per-element dynamic-callback path) are now checker-level `CompileError`s — see
// `error_tests::callables::test_error_func_num_args_first_class_callable`,
// `test_error_func_num_args_dynamic_spread`, and
// `test_error_func_num_args_array_map_callback_literal_name`. Not asserted here (not a
// `#[should_panic]` codegen test) because a checker rejection is not a compiler panic;
// asserting it via `compile_and_run` would conflate "correctly refused invalid input" with
// "the compiler crashed".

/// Zero-cost regression guard: a function that does NOT call any `func_*` arity intrinsic
/// must be completely unaffected by this feature — no hidden parameter, and its ordinary
/// arg-count checking is unchanged (calling it with too many arguments is still a compile
/// error, unlike a real arity-hungry function).
#[test]
fn test_untouched_function_unaffected_by_feature() {
    let out = compile_and_run(
        r#"<?php
function plain($a, $b = 2) {
    return $a + $b;
}
echo plain(1, 2);
"#,
    );
    assert_eq!(out, "3");
}

/// An interface declaration inherited through an ABSTRACT class is not an implementation.
///
/// `ClassInfo::methods` carries the bodyless declaration that `abstract class B implements I`
/// inherits from `I`, with no `method_impl_classes` entry. Counting that declaration as a
/// second "implementation" used to make `func_args_scan`'s closed-world gate reject the
/// whole shape, even though `C::start` is the only body dispatch can reach. php-verified
/// with `php -n`: both call forms print the same as below.
#[test]
fn test_func_args_intrinsic_in_method_declared_by_interface_via_abstract_class() {
    let out = compile_and_run(
        r#"<?php
interface StyleContract {
    public function startBar(int $max = 0): void;
}
abstract class BaseStyle implements StyleContract {
    public function tag(): string { return 'base'; }
}
class RealStyle extends BaseStyle {
    public function startBar(int $max = 0): void
    {
        $format = \func_num_args() > 1 ? func_get_arg(1) : 'default';
        echo $max, "/", $format, ";";
    }
}
$style = new RealStyle();
$style->startBar(5);
$style->startBar(5, 'custom');
"#,
    );
    assert_eq!(out, "5/default;5/custom;");
}

/// Same shape for a STATIC method: an abstract sibling class carrying an unrelated static
/// method must not perturb the closed-world gate for a uniquely implemented one.
#[test]
fn test_func_args_intrinsic_in_static_method_with_abstract_sibling() {
    let out = compile_and_run(
        r#"<?php
abstract class MakerBase {
    public static function describe(): string { return 'base'; }
}
class Maker extends MakerBase {
    public static function build(int $n = 0): string
    {
        return 'n=' . \func_num_args();
    }
}
echo Maker::build(1, 2);
"#,
    );
    assert_eq!(out, "n=2");
}
