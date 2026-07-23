//! Purpose:
//! Integration or regression tests for diagnostic coverage of array builtins, including array mixed type checks, array union operand checks, and indexed array union compatible element types.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Invalid PHP snippets are checked through shared diagnostic helpers for messages, spans, and recovery behavior.

use super::*;

// Verifies that a heterogeneous associative array with string and integer values widens to `mixed` without error.
/// Verifies that assoc array mixed type checks.
#[test]
fn test_assoc_array_mixed_type_checks() {
    assert!(
        check_source(r#"<?php $a = ["name" => "Alice", "age" => 30];"#).is_ok(),
        "heterogeneous associative-array values should widen to mixed",
    );
}

// Regression test: array union with a non-array right operand produces a type error.
/// Verifies that error array union requires array operands.
#[test]
fn test_error_array_union_requires_array_operands() {
    expect_error(
        r#"<?php $result = [1, 2] + 3;"#,
        "Array union requires both operands to be arrays",
    );
}

// Regression test: indexed array union with mismatched element types (int vs string) produces a type error.
/// Verifies that error indexed array union requires compatible element types.
#[test]
fn test_error_indexed_array_union_requires_compatible_element_types() {
    expect_error(
        r#"<?php $result = [1] + ["right", "side"];"#,
        "Array union requires compatible indexed array element types",
    );
}

// --- v0.6: array function argument errors ---

/// Verifies that error array reverse wrong args.
#[test]
fn test_error_array_reverse_wrong_args() {
    expect_error(
        "<?php array_reverse();",
        "array_reverse() takes 1 or 2 arguments",
    );
}

/// Verifies array_merge() rejects a concretely non-array argument. array_merge is
/// variadic in PHP 8 (`array_merge(array ...$arrays)`), so arity is no longer fixed;
/// a plain scalar argument is still a concrete type error. Input: `array_merge(5)`.
#[test]
fn test_error_array_merge_non_array_arg() {
    expect_error(
        "<?php array_merge(5);",
        "array_merge() argument #1 must be array",
    );
}

// --- Gradual-typing acceptance for array-taking builtins (Mixed / union) ---
//
// These type-check cleanly (the argument is accepted under the gradual boundary).
// Full end-to-end codegen for the `Mixed` element case is a separate downstream EIR
// concern, so acceptance is asserted at the type-checker level via `expect_ok`.

/// Verifies count() accepts a genuinely `Mixed` argument (a `mixed` parameter) under
/// the gradual-typing boundary instead of reporting a concrete type error.
#[test]
fn test_count_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): int { return count($x); }");
}

/// Verifies ksort() (a by-reference sort) accepts a `Mixed` argument gradually.
#[test]
fn test_ksort_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { ksort($x); }");
}

/// Verifies array_values() accepts a `Mixed` argument gradually.
#[test]
fn test_array_values_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { $y = array_values($x); }");
}

/// Verifies array_filter() accepts a `Mixed` first argument gradually.
#[test]
fn test_array_filter_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { $y = array_filter($x, fn ($v) => $v); }");
}

/// Verifies array_map() accepts a `Mixed` array argument gradually.
#[test]
fn test_array_map_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { $y = array_map(fn ($v) => $v, $x); }");
}

/// Verifies array_reverse() accepts a `Mixed` argument gradually (R4), instead of the previous
/// strict concrete-array-only check.
#[test]
fn test_array_reverse_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { $y = array_reverse($x); }");
}

/// Verifies array_reverse() accepts an `array|false` union operand (R4): `grapheme_str_split`
/// returns `array|false`, which `count` already accepts and `array_reverse` now does too.
#[test]
fn test_array_reverse_union_operand_type_checks() {
    expect_ok("<?php $u = grapheme_str_split('abc'); $r = array_reverse($u);");
}

/// Verifies array_column() accepts a `Mixed` first argument gradually (R4).
#[test]
fn test_array_column_mixed_arg_type_checks() {
    expect_ok("<?php function tc(mixed $x): void { $y = array_column($x, 'k'); }");
}

/// Verifies array_column() accepts an `array|false` union operand gradually (R4).
#[test]
fn test_array_column_union_operand_type_checks() {
    expect_ok("<?php $u = grapheme_str_split('abc'); $r = array_column($u, 'k');");
}

/// Verifies array_reverse() still rejects a concretely non-array argument (a plain int),
/// preserving the disjoint-type error after the gradual relaxation.
#[test]
fn test_error_array_reverse_non_array_arg() {
    expect_error(
        "<?php $a = 5; array_reverse($a);",
        "array_reverse() argument must be array",
    );
}

/// Verifies array_column() still rejects a concretely non-array first argument.
#[test]
fn test_error_array_column_non_array_arg() {
    expect_error(
        "<?php array_column(5, 'k');",
        "array_column() first argument must be array",
    );
}

/// Verifies array_merge() type-checks with three array arguments (it is variadic).
#[test]
fn test_array_merge_three_args_type_checks() {
    expect_ok("<?php $r = array_merge([1], [2], [3]);");
}

/// Verifies array_merge() type-checks with zero arguments (returns an empty array).
#[test]
fn test_array_merge_zero_args_type_checks() {
    expect_ok("<?php $r = array_merge();");
}

/// Verifies ksort() still rejects a concretely non-array argument (a plain int),
/// preserving the disjoint-type error. Input: `$a = 5; ksort($a);`.
#[test]
fn test_error_ksort_non_array_arg() {
    expect_error("<?php $a = 5; ksort($a);", "ksort() argument must be array");
}

/// Verifies array_values() still rejects a concretely non-array argument.
#[test]
fn test_error_array_values_non_array_arg() {
    expect_error(
        "<?php $a = 5; array_values($a);",
        "array_values() argument must be array",
    );
}

/// Verifies array_map() still rejects a concretely non-array data argument.
#[test]
fn test_error_array_map_non_array_arg() {
    expect_error(
        "<?php array_map(fn ($v) => $v, 5);",
        "array_map() second argument must be array",
    );
}

/// Verifies array_filter() still rejects a concretely non-array first argument.
#[test]
fn test_error_array_filter_non_array_arg() {
    expect_error(
        "<?php array_filter(5, fn ($v) => $v);",
        "array_filter() first argument must be array",
    );
}

/// Verifies count() still rejects a concretely non-array, non-Countable argument
/// (a plain int), preserving the disjoint-type error. Input: `count(5)`.
#[test]
fn test_error_count_non_array_arg() {
    expect_error(
        "<?php count(5);",
        "count() argument must be array or Countable object",
    );
}

/// Verifies count() still rejects a concretely non-array string argument, confirming
/// the by-ref OUT-only overwrite change did not blanket-accept non-array types through
/// the gradual boundary. A plain `string` local is not aliased by any out-only by-ref
/// call, so it stays `string` and `count()` on it remains a compile error. Input:
/// `count("hello")`.
#[test]
fn test_error_count_string_arg() {
    expect_error(
        r#"<?php count("hello");"#,
        "count() argument must be array or Countable object",
    );
}

/// Core correctness requirement: an UNGUARDED `count()` on an `iterable`-typed value still errors.
/// An `iterable` can be a non-`Countable` `Traversable`, so `count()` on it is a static error in
/// PHP 8 unless an `is_countable()` guard proves the value is an array or `Countable`. The guard
/// narrowing must NOT blanket-accept `Iterable`. Input: `count($x)` with `iterable $x`.
#[test]
fn test_error_count_iterable_unguarded() {
    expect_error(
        "<?php function f(iterable $x): int { return count($x); }",
        "count() argument must be array or Countable object",
    );
}

/// Verifies the Symfony ProgressBar shape type-checks: an `is_countable()` guard narrows an
/// `iterable`-typed variable to `array|Countable` inside the TERNARY true-branch, so the guarded
/// `count($x)` is accepted. Checker-only (`is_countable` itself is not lowered to codegen).
#[test]
fn test_is_countable_ternary_guard_narrows_iterable_count() {
    expect_ok(
        "<?php function f(iterable $x): ?int { return is_countable($x) ? count($x) : null; }",
    );
}

/// Verifies the `if`-form counterpart of the ProgressBar shape: an `is_countable()` guard narrows
/// an `iterable`-typed variable to `array|Countable` inside the `if` true-branch, so the guarded
/// `count($x)` is accepted. Checker-only.
#[test]
fn test_is_countable_if_guard_narrows_iterable_count() {
    expect_ok(
        "<?php function f(iterable $x): ?int { if (is_countable($x)) { return count($x); } return null; }",
    );
}

/// Verifies that error array sum wrong args.
#[test]
fn test_error_array_sum_wrong_args() {
    expect_error("<?php array_sum();", "array_sum() takes exactly 1 argument");
}

/// Verifies that error array search wrong args.
#[test]
fn test_error_array_search_wrong_args() {
    expect_error(
        "<?php $a = [1]; array_search($a);",
        "array_search() takes 2 or 3 arguments",
    );
}

/// Verifies that `array_search()` with four arguments produces the correct arity error.
#[test]
fn test_error_array_search_too_many_args() {
    expect_error(
        "<?php $a = [1]; array_search(1, $a, true, 1);",
        "array_search() takes 2 or 3 arguments",
    );
}

/// Verifies that error array key exists wrong args.
#[test]
fn test_error_array_key_exists_wrong_args() {
    expect_error(
        "<?php array_key_exists(1);",
        "array_key_exists() takes exactly 2 arguments",
    );
}

/// Verifies that error array slice wrong args.
#[test]
fn test_error_array_slice_wrong_args() {
    expect_error(
        "<?php $a = [1]; array_slice($a);",
        "array_slice() takes 2 to 4 arguments",
    );
}

/// Verifies that error array combine wrong args.
#[test]
fn test_error_array_combine_wrong_args() {
    expect_error(
        "<?php $a = [1]; array_combine($a);",
        "array_combine() takes exactly 2 arguments",
    );
}

/// Verifies that error range wrong args.
#[test]
fn test_error_range_wrong_args() {
    expect_error("<?php range(1);", "range() takes exactly 2 arguments");
}

/// Verifies that error shuffle wrong args.
#[test]
fn test_error_shuffle_wrong_args() {
    expect_error("<?php shuffle();", "shuffle() takes exactly 1 argument");
}

/// Verifies that error array fill wrong args.
#[test]
fn test_error_array_fill_wrong_args() {
    expect_error(
        "<?php array_fill(0, 5);",
        "array_fill() takes exactly 3 arguments",
    );
}

/// Verifies that error array push wrong args.
#[test]
fn test_error_array_push_wrong_args() {
    expect_error(
        "<?php array_push();",
        "array_push() takes exactly 2 arguments",
    );
}

/// Verifies that error array pop wrong args.
#[test]
fn test_error_array_pop_wrong_args() {
    expect_error("<?php array_pop();", "array_pop() takes exactly 1 argument");
}

/// Verifies that error in array wrong args (too few).
#[test]
fn test_error_in_array_wrong_args() {
    expect_error("<?php in_array(1);", "in_array() takes 2 or 3 arguments");
}

/// Verifies that `in_array()` rejects more than the three supported arguments.
#[test]
fn test_error_in_array_too_many_args() {
    expect_error(
        "<?php in_array(1, [1], true, 2);",
        "in_array() takes 2 or 3 arguments",
    );
}

/// Verifies that error array keys wrong args.
#[test]
fn test_error_array_keys_wrong_args() {
    expect_error(
        "<?php array_keys();",
        "array_keys() takes exactly 1 argument",
    );
}

/// Verifies that error array values wrong args.
#[test]
fn test_error_array_values_wrong_args() {
    expect_error(
        "<?php array_values();",
        "array_values() takes exactly 1 argument",
    );
}

/// Verifies that error sort wrong args.
#[test]
fn test_error_sort_wrong_args() {
    expect_error("<?php sort();", "sort() takes 1 or 2 arguments");
}

/// Verifies that error rsort wrong args.
#[test]
fn test_error_rsort_wrong_args() {
    expect_error("<?php rsort();", "rsort() takes 1 or 2 arguments");
}

/// Verifies that error isset wrong args.
#[test]
fn test_error_isset_wrong_args() {
    expect_error("<?php isset();", "isset() takes at least 1 argument");
}

/// Verifies that error array unique wrong args.
#[test]
fn test_error_array_unique_wrong_args() {
    expect_error(
        "<?php array_unique();",
        "array_unique() takes 1 or 2 arguments",
    );
}

/// Verifies that error array product wrong args.
#[test]
fn test_error_array_product_wrong_args() {
    expect_error(
        "<?php array_product();",
        "array_product() takes exactly 1 argument",
    );
}

/// Verifies that error array shift wrong args.
#[test]
fn test_error_array_shift_wrong_args() {
    expect_error(
        "<?php array_shift();",
        "array_shift() takes exactly 1 argument",
    );
}

/// Verifies that error array unshift wrong args.
#[test]
fn test_error_array_unshift_wrong_args() {
    // array_unshift() is variadic since 3a2bb667a (array + 1+ values); zero
    // args is still an arity violation, just with a "at least" message now.
    expect_error(
        "<?php array_unshift();",
        "array_unshift() takes at least 2 arguments",
    );
}

/// Verifies that error array splice wrong args.
#[test]
fn test_error_array_splice_wrong_args() {
    expect_error(
        "<?php array_splice();",
        "array_splice() takes 2 to 4 arguments",
    );
}

/// Verifies that error array flip wrong args.
#[test]
fn test_error_array_flip_wrong_args() {
    expect_error(
        "<?php array_flip();",
        "array_flip() takes exactly 1 argument",
    );
}

/// Verifies that error array chunk wrong args.
#[test]
fn test_error_array_chunk_wrong_args() {
    expect_error(
        "<?php array_chunk();",
        "array_chunk() takes exactly 2 arguments",
    );
}

/// Verifies that error array pad wrong args.
#[test]
fn test_error_array_pad_wrong_args() {
    expect_error(
        "<?php array_pad();",
        "array_pad() takes exactly 3 arguments",
    );
}

/// Verifies that error array fill keys wrong args.
#[test]
fn test_error_array_fill_keys_wrong_args() {
    expect_error(
        "<?php array_fill_keys();",
        "array_fill_keys() takes exactly 2 arguments",
    );
}

/// Verifies that error count wrong args.
#[test]
fn test_error_count_wrong_args() {
    expect_error("<?php count();", "count() takes exactly 1 argument");
}

/// Verifies that error array diff wrong args.
#[test]
fn test_error_array_diff_wrong_args() {
    expect_error(
        "<?php array_diff();",
        "array_diff() takes exactly 2 arguments",
    );
}

/// Verifies that error array intersect wrong args.
#[test]
fn test_error_array_intersect_wrong_args() {
    expect_error(
        "<?php array_intersect();",
        "array_intersect() takes exactly 2 arguments",
    );
}

/// Verifies that error array diff key wrong args.
#[test]
fn test_error_array_diff_key_wrong_args() {
    expect_error(
        "<?php array_diff_key();",
        "array_diff_key() takes exactly 2 arguments",
    );
}

/// Verifies that error array intersect key wrong args.
#[test]
fn test_error_array_intersect_key_wrong_args() {
    expect_error(
        "<?php array_intersect_key();",
        "array_intersect_key() takes exactly 2 arguments",
    );
}

/// Verifies that error array rand wrong args.
#[test]
fn test_error_array_rand_wrong_args() {
    expect_error(
        "<?php array_rand();",
        "array_rand() takes exactly 1 argument",
    );
}

/// Verifies that error asort wrong args.
#[test]
fn test_error_asort_wrong_args() {
    expect_error("<?php asort();", "asort() takes 1 or 2 arguments");
}

/// Verifies that error arsort wrong args.
#[test]
fn test_error_arsort_wrong_args() {
    expect_error("<?php arsort();", "arsort() takes 1 or 2 arguments");
}

/// Verifies that error ksort wrong args.
#[test]
fn test_error_ksort_wrong_args() {
    expect_error("<?php ksort();", "ksort() takes 1 or 2 arguments");
}

/// Verifies that error krsort wrong args.
#[test]
fn test_error_krsort_wrong_args() {
    expect_error("<?php krsort();", "krsort() takes 1 or 2 arguments");
}

/// Verifies that error natsort wrong args.
#[test]
fn test_error_natsort_wrong_args() {
    expect_error("<?php natsort();", "natsort() takes exactly 1 argument");
}

/// Verifies that error natcasesort wrong args.
#[test]
fn test_error_natcasesort_wrong_args() {
    expect_error(
        "<?php natcasesort();",
        "natcasesort() takes exactly 1 argument",
    );
}

/// Verifies that error array column wrong args.
#[test]
fn test_error_array_column_wrong_args() {
    expect_error(
        r#"<?php array_column([]);"#,
        "array_column() takes exactly 2 arguments",
    );
}

/// Verifies that error array map wrong args.
#[test]
fn test_error_array_map_wrong_args() {
    expect_error(
        r#"<?php array_map("fn");"#,
        "array_map() takes exactly 2 arguments",
    );
}

/// Verifies that a string-literal callback naming a non-existent, non-builtin
/// function is still reported as "Undefined function" after the
/// builtin-callable resolution path was added (genuine-undefined guard).
#[test]
fn test_error_array_map_string_literal_undefined_function() {
    expect_error(
        r#"<?php array_map('no_such_function_anywhere', [1, 2]);"#,
        "Undefined function: no_such_function_anywhere",
    );
}

/// Verifies that array_filter() rejects too few arguments (0 args; PHP allows 1–3).
#[test]
fn test_error_array_filter_wrong_args() {
    expect_error(
        r#"<?php array_filter();"#,
        "array_filter() takes 1 to 3 arguments",
    );
}

/// Verifies that array_filter() rejects too many arguments (4 args; PHP allows 1–3).
#[test]
fn test_error_array_filter_too_many_args() {
    expect_error(
        r#"<?php array_filter([1], fn($v) => $v, 0, 1);"#,
        "array_filter() takes 1 to 3 arguments",
    );
}

/// Verifies that array_reduce() rejects too few arguments (1 arg; PHP allows 2–3).
#[test]
fn test_error_array_reduce_wrong_args() {
    expect_error(
        r#"<?php array_reduce([]);"#,
        "array_reduce() takes 2 or 3 arguments",
    );
}

/// Verifies that array_reduce() rejects too many arguments (4 args; PHP allows 2–3).
#[test]
fn test_error_array_reduce_too_many_args() {
    expect_error(
        r#"<?php array_reduce([], fn($c, $v) => $c, 0, 1);"#,
        "array_reduce() takes 2 or 3 arguments",
    );
}

/// Verifies that error array walk wrong args.
#[test]
fn test_error_array_walk_wrong_args() {
    expect_error(
        r#"<?php array_walk([]);"#,
        "array_walk() takes exactly 2 arguments",
    );
}

/// Verifies that error usort wrong args.
#[test]
fn test_error_usort_wrong_args() {
    expect_error(r#"<?php usort([]);"#, "usort() takes exactly 2 arguments");
}

/// Verifies that error uksort wrong args.
#[test]
fn test_error_uksort_wrong_args() {
    expect_error(r#"<?php uksort([]);"#, "uksort() takes exactly 2 arguments");
}

/// Verifies that error uasort wrong args.
#[test]
fn test_error_uasort_wrong_args() {
    expect_error(r#"<?php uasort([]);"#, "uasort() takes exactly 2 arguments");
}

/// Verifies that error usort first class callable wrong arity.
#[test]
fn test_error_usort_first_class_callable_wrong_arity() {
    expect_error(
        r#"<?php
class BadComparator {
    public function cmp($a) {
        return 0;
    }
}

$bad = new BadComparator();
$values = [2, 1];
usort($values, $bad->cmp(...));
"#,
        "Method BadComparator::cmp expects 1 arguments, got 2",
    );
}

/// Verifies that error list unpack non array.
#[test]
fn test_error_list_unpack_non_array() {
    expect_error("<?php [$a, $b] = 42;", "List unpacking requires an array");
}

/// Verifies list unpacking rejects a nullable array when no guard removes the null member.
#[test]
fn test_error_list_unpack_nullable_array_without_guard() {
    expect_error(
        "<?php function row(): ?array { return null; } $entry = row(); [$a, $b] = $entry;",
        "List unpacking requires an array",
    );
}

// --- call_user_func_array errors ---

/// Verifies that error call user func array wrong args.
#[test]
fn test_error_call_user_func_array_wrong_args() {
    expect_error(
        "<?php call_user_func_array(\"foo\");",
        "call_user_func_array() takes exactly 2 arguments",
    );
}

// --- v0.8 system function errors ---

/// Verifies that error spread non array.
#[test]
fn test_error_spread_non_array() {
    expect_error(
        "<?php $x = 5; $y = [...$x];",
        "Spread operator requires an array",
    );
}

/// Verifies that error static property array push requires array.
#[test]
fn test_error_static_property_array_push_requires_array() {
    expect_error(
        "<?php class Box { public static int $items = 1; } Box::$items[] = 2;",
        "Array push requires an array static property, got int",
    );
}

/// Verifies that indexed array unrelated object values widen to mixed.
#[test]
fn test_indexed_array_unrelated_object_values_widen_to_mixed() {
    assert!(
        check_source("<?php class Dog {} class Car {} $items = [new Dog(), new Car()];").is_ok(),
        "heterogeneous indexed-array values should widen to mixed",
    );
}

/// Verifies `array_map()` rejects object elements until its callback runtime supports them.
#[test]
fn test_error_array_map_rejects_object_elements() {
    expect_error(
        "<?php final class Box {} $items = [new Box()]; array_map(static fn(Box $box): Box => $box, $items);",
        "array_map() does not yet support object array elements",
    );
}

/// Verifies contextual callback checking still rejects declarations incompatible with known elements.
#[test]
fn test_error_array_callback_rejects_known_element_mismatch() {
    expect_error(
        "<?php array_map(static fn(string $value): string => $value, [1, 2]);",
        "array_map() callback parameter $value expects Str, got Int",
    );
}

/// Verifies that error call user func array ref callback param requires variable.
#[test]
fn test_error_call_user_func_array_ref_callback_param_requires_variable() {
    expect_error(
        "<?php function bump(&$n) { $n = $n + 1; } call_user_func_array(\"bump\", [1]);",
        "parameter $n must be passed a variable",
    );
}

// -- Recognition-layer coverage for newly registered array builtins --
// These builtins are recognized at type-check time (catalog + signature +
// checker return type + first-class-callable sig); their EIR/runtime lowering
// is deferred, so only type-check recognition is asserted here (no
// compile_and_run, which would fail at the deferred codegen stage).

/// Verifies that the internal-pointer family `reset()` (by-ref), `current()`,
/// and `key()` type-check on an array argument.
#[test]
fn test_reset_current_key_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = [1, 2, 3];
$r = reset($a);
$c = current($a);
$k = key($a);
echo $r;
"#
        )
        .is_ok(),
        "reset()/current()/key() should be recognized on an array argument",
    );
}

/// Verifies that `array_key_first()` type-checks (returns string|int|null).
#[test]
fn test_array_key_first_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = ["x" => 1, "y" => 2];
$first = array_key_first($a);
echo $first;
"#
        )
        .is_ok(),
        "array_key_first() should be recognized and return string|int|null",
    );
}

/// Verifies that `array_replace_recursive()` type-checks in its variadic form.
#[test]
fn test_array_replace_recursive_recognized() {
    assert!(
        check_source(
            r#"<?php
$merged = array_replace_recursive(["a" => 1], ["a" => 2], ["b" => 3]);
echo count($merged);
"#
        )
        .is_ok(),
        "array_replace_recursive() should be recognized as a variadic array merge",
    );
}

/// Verifies that `array_walk_recursive()` type-checks with a by-ref-free
/// callback (its by-ref &$value callback modeling matches array_walk exactly).
#[test]
fn test_array_walk_recursive_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = [1, 2, 3];
array_walk_recursive($a, function ($v) { echo $v; });
"#
        )
        .is_ok(),
        "array_walk_recursive() should be recognized with a callback",
    );
}

/// Verifies that `is_countable()` type-checks and returns bool for any value,
/// including through first-class-callable syntax.
#[test]
fn test_is_countable_recognized() {
    assert!(
        check_source(
            r#"<?php
$a = [1, 2, 3];
$b = is_countable($a);
$f = is_countable(...);
echo is_callable($f);
"#
        )
        .is_ok(),
        "is_countable() should be recognized and return bool",
    );
}

expect_builtin_arity_error!(
    test_error_reset_wrong_args,
    "<?php reset();",
    "reset() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_array_key_first_wrong_args,
    "<?php array_key_first([], 1);",
    "array_key_first() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_array_walk_recursive_wrong_args,
    "<?php array_walk_recursive([1]);",
    "array_walk_recursive() takes 2 or 3 arguments"
);

expect_builtin_arity_error!(
    test_error_is_countable_wrong_args,
    "<?php is_countable();",
    "is_countable() takes exactly 1 argument"
);

/// Verifies that `current()` rejects a concretely non-array argument.
#[test]
fn test_error_current_non_array() {
    expect_error(
        "<?php current(5);",
        "current() argument must be array",
    );
}

expect_builtin_arity_error!(
    test_error_array_is_list_wrong_args,
    "<?php array_is_list([1, 2], 3);",
    "array_is_list() takes exactly 1 argument"
);

expect_builtin_arity_error!(
    test_error_array_replace_no_args,
    "<?php array_replace();",
    "array_replace() requires at least 1 argument"
);

/// Verifies that array_is_list() rejects a non-array argument.
#[test]
fn test_error_array_is_list_non_array() {
    expect_error(
        "<?php array_is_list(5);",
        "array_is_list() argument must be array",
    );
}


/// Verifies that array_key_last() rejects a non-array argument.
#[test]
fn test_error_array_key_last_non_array() {
    expect_error(
        "<?php array_key_last(\"x\");",
        "array_key_last() argument must be array",
    );
}

/// Verifies that array_replace() accepts a single argument (PHP:
/// `array_replace(array $array, array ...$replacements)` — the replacements are optional).
#[test]
fn test_error_array_replace_wrong_args() {
    expect_ok("<?php $a = [\"k\" => 1]; $r = array_replace($a); print_r($r);");
}

/// Verifies that array_replace() rejects string-element indexed arrays (scalar indexed inputs
/// are supported; string/heap element indexed inputs are a follow-up).
#[test]
fn test_error_array_replace_string_indexed_unsupported() {
    expect_error(
        "<?php array_replace([\"a\", \"b\"], [\"c\"]);",
        "array_replace() arguments must be associative arrays or indexed arrays of scalars",
    );
}

/// Verifies that array_replace_recursive() accepts a single argument (PHP:
/// the replacement arrays are an optional variadic tail).
#[test]
fn test_error_array_replace_recursive_wrong_args() {
    expect_ok("<?php $a = [\"k\" => 1]; $r = array_replace_recursive($a); print_r($r);");
}

/// Verifies that array_replace_recursive() rejects string-element indexed arrays (scalar indexed
/// inputs are supported; string/heap element indexed inputs are a follow-up).
#[test]
fn test_error_array_replace_recursive_string_indexed_unsupported() {
    expect_error(
        "<?php array_replace_recursive([\"a\"], [\"b\"]);",
        "array_replace_recursive() arguments must be associative arrays or indexed arrays of scalars",
    );
}

/// Verifies that array_diff_assoc() with a single argument reports an arity error.
#[test]
fn test_error_array_diff_assoc_wrong_args() {
    expect_error(
        "<?php $a = [\"k\" => 1]; array_diff_assoc($a);",
        "array_diff_assoc() takes exactly 2 arguments",
    );
}

/// Verifies that array_intersect_assoc() rejects string-element indexed arrays (scalar indexed
/// inputs are supported; string/heap element indexed inputs are a follow-up).
#[test]
fn test_error_array_intersect_assoc_string_indexed_unsupported() {
    expect_error(
        "<?php array_intersect_assoc([\"a\", \"b\"], [\"a\"]);",
        "array_intersect_assoc() arguments must be associative arrays or indexed arrays of scalars",
    );
}

/// Verifies that array_merge_recursive() with a single argument reports an arity error.
#[test]
fn test_error_array_merge_recursive_wrong_args() {
    expect_error(
        "<?php $a = [\"k\" => 1]; array_merge_recursive($a);",
        "array_merge_recursive() takes exactly 2 arguments",
    );
}

/// Verifies that array_merge_recursive() rejects string-element indexed arrays (scalar indexed
/// inputs are supported; string/heap element indexed inputs are a follow-up).
#[test]
fn test_error_array_merge_recursive_string_indexed_unsupported() {
    expect_error(
        "<?php array_merge_recursive([\"a\"], [\"b\"]);",
        "array_merge_recursive() arguments must be associative arrays or indexed arrays of scalars",
    );
}

/// Verifies that array_find() with a single argument reports an arity error.
#[test]
fn test_error_array_find_wrong_args() {
    expect_error(
        "<?php function f($x) { return true; } array_find([1, 2]);",
        "array_find() takes exactly 2 arguments",
    );
}

/// Verifies that array_any() with a single argument reports an arity error.
#[test]
fn test_error_array_any_wrong_args() {
    expect_error(
        "<?php function f($x) { return true; } array_any([1, 2]);",
        "array_any() takes exactly 2 arguments",
    );
}

/// Verifies that array_all() rejects a non-array first argument.
#[test]
fn test_error_array_all_non_array() {
    expect_error(
        "<?php function f($x) { return true; } array_all(5, \"f\");",
        "array_all() first argument must be array",
    );
}


/// Verifies that array_udiff() with two arguments reports an arity error.
#[test]
fn test_error_array_udiff_wrong_args() {
    expect_error(
        "<?php function c($a, $b) { return 0; } array_udiff([1], [2]);",
        "array_udiff() takes exactly 3 arguments",
    );
}

/// Verifies that array_uintersect() rejects a non-array first argument.
#[test]
fn test_error_array_uintersect_non_array() {
    expect_error(
        "<?php function c($a, $b) { return 0; } array_uintersect(5, [2], \"c\");",
        "array_uintersect() first argument must be array",
    );
}

/// Verifies that array_multisort() with a single argument reports an arity error.
#[test]
fn test_error_array_multisort_wrong_args() {
    expect_error(
        "<?php $a = [1, 2]; array_multisort($a);",
        "array_multisort() takes exactly 2 arguments",
    );
}

/// Verifies that array_multisort() rejects non-indexed-array arguments.
#[test]
fn test_error_array_multisort_non_array() {
    expect_error(
        "<?php $a = [1, 2]; array_multisort($a, 5);",
        "array_multisort() arguments must be indexed arrays",
    );
}
