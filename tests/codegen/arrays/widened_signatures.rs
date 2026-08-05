//! Purpose:
//! Integration tests for array builtin parameters that were previously rejected at compile time
//! because elephc's signature was shorter than reference PHP's: `implode($array)`,
//! `array_unshift(&$array, ...$values)`, `array_search(..., $strict)`,
//! `array_reverse($array, $preserve_keys)`, and `range($start, $end, $step)`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected value is verbatim `LC_ALL=C php` output from PHP 8.4.20.
//! - Each new parameter is exercised BOTH positionally and as a named argument, except
//!   `array_unshift`'s variadic `values`, which reference PHP itself rejects as a named argument
//!   (`array_unshift() does not accept unknown named parameters`).
//! - The `range()` `ValueError` fixtures catch the exception in PHP so the process still exits 0;
//!   they also pin `RuntimeFnId::Range`'s `MAY_THROW` effect, without which dead-code elimination
//!   drops the diagnostic for a call whose result is unused.

use crate::support::*;

/// Verifies PHP's one-argument `implode($array)` form joins with an empty separator.
#[test]
fn test_implode_single_array_argument() {
    let out = compile_and_run("<?php echo implode([1, 2]);");
    assert_eq!(out, "12");
}

/// Verifies the one-argument `implode()` form accepts a runtime array variable.
#[test]
fn test_implode_single_array_variable() {
    let out = compile_and_run(r#"<?php $s = ["a", "b", "c"]; echo implode($s);"#);
    assert_eq!(out, "abc");
}

/// Verifies `implode([])` yields the empty string rather than an arity error.
#[test]
fn test_implode_single_empty_array() {
    let out = compile_and_run(r#"<?php echo strlen(implode([])), ":", implode([]), ";";"#);
    assert_eq!(out, "0:;");
}

/// Verifies the two-argument `implode()` form still works through named arguments.
#[test]
fn test_implode_named_arguments() {
    let out = compile_and_run(r#"<?php echo implode(separator: "-", array: [1, 2, 3]);"#);
    assert_eq!(out, "1-2-3");
}

/// Verifies the widened `implode()` signature keeps case-insensitive and namespaced lookup.
#[test]
fn test_implode_case_insensitive_and_namespaced() {
    let out = compile_and_run(r#"<?php echo IMPLODE([7, 8]), ":", \implode("|", [9, 10]);"#);
    assert_eq!(out, "78:9|10");
}

/// Verifies `array_unshift()` prepends two values in source order and returns the new count.
#[test]
fn test_array_unshift_two_values() {
    let out = compile_and_run(
        r#"<?php $a = [3, 4]; $n = array_unshift($a, 1, 2); echo $n, ":", implode(",", $a);"#,
    );
    assert_eq!(out, "4:1,2,3,4");
}

/// Verifies a four-value `array_unshift()` grows the payload before every prepend.
///
/// The source has no spare capacity, so each of the four prepends must re-run the growth check;
/// a single up-front check would write past the allocation on the second value.
#[test]
fn test_array_unshift_four_values_grow() {
    let out = compile_and_run(
        r#"<?php $a = [1, 2, 3]; $n = array_unshift($a, 10, 20, 30, 40); echo $n, ":", implode(",", $a);"#,
    );
    assert_eq!(out, "7:10,20,30,40,1,2,3");
}

/// Verifies a seven-value `array_unshift()` past several doublings stays in bounds.
#[test]
fn test_array_unshift_seven_values_grow() {
    let out = compile_and_run(
        r#"<?php $a = [1, 2, 3, 4]; $n = array_unshift($a, 7, 8, 9, 10, 11, 12, 13); echo $n, ":", implode(",", $a);"#,
    );
    assert_eq!(out, "11:7,8,9,10,11,12,13,1,2,3,4");
}

/// Verifies the value-less `array_unshift($array)` form reports the unchanged count.
#[test]
fn test_array_unshift_without_values() {
    let out = compile_and_run(
        r#"<?php $a = [5, 6]; $n = array_unshift($a); echo $n, ":", implode(",", $a);"#,
    );
    assert_eq!(out, "2:5,6");
}

/// Verifies a multi-value `array_unshift()` splits shared storage before mutating it.
#[test]
fn test_array_unshift_multi_value_copy_on_write() {
    let out = compile_and_run(
        r#"<?php $a = [1, 2]; $b = $a; $n = array_unshift($a, 0, -1); echo $n, ":", implode(",", $a), ":", implode(",", $b);"#,
    );
    assert_eq!(out, "4:0,-1,1,2:1,2");
}

/// Verifies the positional `array_search($needle, $haystack, true)` form.
#[test]
fn test_array_search_strict_positional() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; echo var_export(array_search(20, $a, true), true), ":", var_export(array_search(99, $a, true), true);"#,
    );
    assert_eq!(out, "1:false");
}

/// Verifies `array_search()`'s `strict` flag works as a named argument in both positions.
#[test]
fn test_array_search_strict_named() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; echo var_export(array_search(needle: 20, haystack: $a, strict: true), true), ":", var_export(array_search(20, $a, strict: false), true);"#,
    );
    assert_eq!(out, "1:1");
}

/// Verifies strict search rejects a cross-type match the loose form still finds.
///
/// `array_search(true, [1, 0], true)` is `false` under `===` while the loose form returns `0`,
/// which is the only case where the two modes diverge for a supported element layout.
#[test]
fn test_array_search_strict_cross_type() {
    let out = compile_and_run(
        r#"<?php $i = [1, 0]; echo var_export(array_search(true, $i, true), true), ":", var_export(array_search(true, $i, false), true);"#,
    );
    assert_eq!(out, "false:0");
}

/// Verifies strict search over an associative haystack still returns the string key.
#[test]
fn test_array_search_strict_assoc() {
    let out = compile_and_run(
        r#"<?php $m = ["a" => 1, "b" => 2]; echo var_export(array_search(2, $m, true), true), ":", var_export(array_search(2, $m, false), true);"#,
    );
    assert_eq!(out, "'b':'b'");
}

/// Verifies a runtime-unknown `strict` flag selects the mode at run time.
#[test]
fn test_array_search_strict_runtime_flag() {
    let out = compile_and_run(
        r#"<?php $a = [10, 20, 30]; $t = $argc > 0; echo var_export(array_search(20, $a, $t), true);"#,
    );
    assert_eq!(out, "1");
}

/// Verifies strict search over a string haystack keeps exact comparison.
#[test]
fn test_array_search_strict_string_haystack() {
    let out = compile_and_run(
        r#"<?php $s = ["x", "y", "z"]; echo var_export(array_search("y", $s, true), true), ":", var_export(array_search("q", $s, true), true);"#,
    );
    assert_eq!(out, "1:false");
}

/// Verifies `array_reverse($array, true)` keeps the source integer keys in reversed order.
#[test]
fn test_array_reverse_preserve_keys_positional() {
    let out = compile_and_run(
        r#"<?php $x = array_reverse([1, 2, 3], true); foreach ($x as $k => $v) { echo $k, "=>", $v, " "; }"#,
    );
    assert_eq!(out, "2=>3 1=>2 0=>1 ");
}

/// Verifies the `preserve_keys:` named argument and the resulting key-addressable lookups.
#[test]
fn test_array_reverse_preserve_keys_named() {
    let out = compile_and_run(
        r#"<?php $x = array_reverse([10, 20, 30], preserve_keys: true); foreach ($x as $k => $v) { echo $k, "=>", $v, " "; } echo "|", $x[0], ",", $x[2];"#,
    );
    assert_eq!(out, "2=>30 1=>20 0=>10 |10,30");
}

/// Verifies an explicit `false` flag keeps the renumbered indexed-array result.
#[test]
fn test_array_reverse_preserve_keys_false() {
    let out = compile_and_run(r#"<?php echo implode(",", array_reverse([1, 2, 3], false));"#);
    assert_eq!(out, "3,2,1");
}

/// Verifies the key-preserving reversal of an empty array produces an empty result.
#[test]
fn test_array_reverse_preserve_keys_empty() {
    let out = compile_and_run("<?php $x = array_reverse([], true); echo count($x);");
    assert_eq!(out, "0");
}

/// Verifies key-preserving reversal persists string payloads instead of aliasing the source.
#[test]
fn test_array_reverse_preserve_keys_strings() {
    let out = compile_and_run(
        r#"<?php $s = ["p", "q", "r"]; $x = array_reverse($s, true); foreach ($x as $k => $v) { echo $k, "=>", $v, " "; } echo "|", implode(",", $s);"#,
    );
    assert_eq!(out, "2=>r 1=>q 0=>p |p,q,r");
}

/// Verifies the positional `range($start, $end, $step)` form for ascending ranges.
#[test]
fn test_range_step_positional() {
    let out = compile_and_run(
        r#"<?php echo implode(",", range(1, 10, 2)), ":", implode(",", range(1, 10, 3));"#,
    );
    assert_eq!(out, "1,3,5,7,9:1,4,7,10");
}

/// Verifies `range()`'s `step:` named argument, alone and alongside named endpoints.
#[test]
fn test_range_step_named() {
    let out = compile_and_run(
        r#"<?php echo implode(",", range(start: 1, end: 7, step: 3)), ":", implode(",", range(1, 7, step: 2));"#,
    );
    assert_eq!(out, "1,4,7:1,3,5,7");
}

/// Verifies a descending range accepts either step sign, as reference PHP does.
#[test]
fn test_range_step_descending() {
    let out = compile_and_run(
        r#"<?php echo implode(",", range(10, 1, 2)), ":", implode(",", range(5, 1, -2)), ":", implode(",", range(10, -5, 4));"#,
    );
    assert_eq!(out, "10,8,6,4,2:5,3,1:10,6,2,-2");
}

/// Verifies the step magnitude boundaries: a step equal to the span, and degenerate ranges.
///
/// `range(1, 3, 2)` is accepted (step == span) while `range(1, 3, 3)` is a `ValueError`, and
/// `range(5, 5, 3)` yields `[5]` because an equal-endpoint range ignores the step magnitude.
#[test]
fn test_range_step_boundaries() {
    let out = compile_and_run(
        r#"<?php echo implode(",", range(1, 3, 2)), ":", implode(",", range(5, 5, 3)), ":", implode(",", range(1, 1, 1));"#,
    );
    assert_eq!(out, "1,3:5:1");
}

/// Verifies a runtime-unknown step still produces PHP's element sequence.
#[test]
fn test_range_step_runtime_value() {
    let out = compile_and_run(r#"<?php $s = 2; echo implode(",", range(1, 9, $s));"#);
    assert_eq!(out, "1,3,5,7,9");
}

/// Verifies a zero `range()` step raises PHP's catchable `ValueError`.
#[test]
fn test_range_step_zero_value_error() {
    let out = compile_and_run(
        r#"<?php $z = 0; try { range(1, 5, $z); echo "no"; } catch (ValueError $e) { echo get_class($e), ": ", $e->getMessage(); }"#,
    );
    assert_eq!(out, "ValueError: range(): Argument #3 ($step) cannot be 0");
}

/// Verifies a negative `range()` step on an increasing range raises PHP's `ValueError`.
#[test]
fn test_range_step_negative_value_error() {
    let out = compile_and_run(
        r#"<?php $n = -2; try { range(1, 5, $n); echo "no"; } catch (ValueError $e) { echo get_class($e), ": ", $e->getMessage(); }"#,
    );
    assert_eq!(
        out,
        "ValueError: range(): Argument #3 ($step) must be greater than 0 for increasing ranges"
    );
}

/// Verifies a `range()` step wider than the spanned interval raises PHP's `ValueError`.
#[test]
fn test_range_step_too_wide_value_error() {
    let out = compile_and_run(
        r#"<?php $w = 10; try { range(1, 3, $w); echo "no"; } catch (ValueError $e) { echo get_class($e), ": ", $e->getMessage(); }"#,
    );
    assert_eq!(
        out,
        "ValueError: range(): Argument #3 ($step) must be less than the range spanned by argument #1 ($start) and argument #2 ($end)"
    );
}
