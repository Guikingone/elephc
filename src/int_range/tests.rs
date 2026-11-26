//! Purpose:
//! Unit tests for the integer-range analysis: proven loop counters and masked accumulators,
//! and every aliasing/control-flow shape that must keep the checked arithmetic.
//!
//! Called from:
//! - `cargo test` (module `crate::int_range::tests`).
//!
//! Key details:
//! - Verdicts are looked up per source line, AND-ed over all sites of the requested kind.

use super::{analyze_program, ArithKind};

/// Returns the combined verdict of every `kind` site on `line`, or `None` if none was recorded.
fn verdict(src: &str, line: u32, kind: ArithKind) -> Option<bool> {
    let tokens = crate::lexer::tokenize(src).expect("tokenize should succeed");
    let program = crate::parser::parse(&tokens).expect("parse should succeed");
    let facts = analyze_program(&program, false);
    facts
        .facts
        .iter()
        .filter(|((span, site_kind), _)| span.line == line && *site_kind == kind)
        .map(|(_, proven)| *proven)
        .reduce(|a, b| a && b)
}

/// Asserts that the `kind` site(s) on `line` are proven overflow-free.
fn assert_proven(src: &str, line: u32, kind: ArithKind) {
    assert_eq!(verdict(src, line, kind), Some(true), "expected {kind:?} on line {line} to be proven:\n{src}");
}

/// Asserts that the `kind` site(s) on `line` keep the checked arithmetic.
fn assert_not_proven(src: &str, line: u32, kind: ArithKind) {
    assert_ne!(verdict(src, line, kind), Some(true), "expected {kind:?} on line {line} to stay checked:\n{src}");
}

#[test]
fn for_loop_counter_bounded_by_int_param_is_proven() {
    let src = "<?php\nfunction f(int $n) {\n  for ($i = 0; $i < $n; $i++) {\n    echo $i;\n  }\n}\n";
    assert_proven(src, 3, ArithKind::Inc);
}

#[test]
fn while_counter_with_literal_bound_is_proven() {
    let src = "<?php\n$i = 0;\nwhile ($i < 50000000) {\n  $i = $i + 1;\n}\n";
    assert_proven(src, 4, ArithKind::Add);
}

#[test]
fn countdown_is_proven() {
    let src = "<?php\n$i = 10;\nwhile ($i > 0) {\n  $i--;\n}\ndo {\n  $i -= 1;\n} while ($i > -5);\n";
    assert_proven(src, 4, ArithKind::Dec);
    assert_proven(src, 7, ArithKind::Sub);
}

#[test]
fn masked_accumulator_seeded_by_argc_is_proven() {
    let src = "<?php\n$acc = $argc;\nfor ($i = 0; $i < 50000000; $i++) {\n  $acc = ($acc * 3 + ($i & 7)) & 0x3FFFFFFF;\n}\necho $acc;\n";
    assert_proven(src, 3, ArithKind::Inc);
    assert_proven(src, 4, ArithKind::Mul);
    assert_proven(src, 4, ArithKind::Add);
}

#[test]
fn unbounded_accumulator_stays_checked() {
    let src = "<?php\n$s = 0;\nfor ($i = 0; $i < 100; $i++) {\n  $s += $i;\n}\n";
    assert_proven(src, 3, ArithKind::Inc);
    assert_not_proven(src, 4, ArithKind::Add);
}

#[test]
fn fib_argument_arithmetic_is_proven_but_sum_is_not() {
    let src = "<?php\nfunction fib(int $n): int {\n  return $n < 2 ? $n : fib($n - 1)\n    + fib($n - 2);\n}\n";
    assert_proven(src, 3, ArithKind::Sub);
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn guard_by_early_return_narrows() {
    let src = "<?php\nfunction g(int $n): int {\n  if ($n <= 0) {\n    return 0;\n  }\n  return $n - 1;\n}\n";
    assert_proven(src, 6, ArithKind::Sub);
}

#[test]
fn int_max_increment_stays_checked() {
    let src = "<?php\n$i = 9223372036854775807;\n$i++;\n$j = 9223372036854775806;\n$j++;\n";
    assert_not_proven(src, 3, ArithKind::Inc);
    assert_proven(src, 5, ArithKind::Inc);
}

#[test]
fn untyped_parameters_stay_checked() {
    let src = "<?php\nfunction h($n) {\n  return $n + 1;\n}\n";
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn by_reference_argument_pins_the_variable() {
    let by_ref = "<?php\nfunction f(&$x) { $x = 9223372036854775807; }\n$i = 0;\nf($i);\n$i++;\n";
    assert_not_proven(by_ref, 5, ArithKind::Inc);
    let by_value = "<?php\nfunction f($x) { $x = 9223372036854775807; }\n$i = 0;\nf($i);\n$i++;\n";
    assert_proven(by_value, 5, ArithKind::Inc);
}

#[test]
fn unknown_callee_pins_bare_variable_arguments() {
    let src = "<?php\n$i = 0;\n$f = 'f';\n$f($i);\n$i++;\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn global_statement_pins_top_level_variable() {
    let src = "<?php\nfunction g() { global $i; $i = 9223372036854775807; }\n$i = 0;\ng();\n$i++;\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn globals_alias_pins_top_level_variable() {
    let src = "<?php\nfunction g() { $GLOBALS['i'] = 9223372036854775807; }\n$i = 0;\ng();\n$i++;\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn reference_assignment_pins_both_sides() {
    let src = "<?php\n$i = 0;\n$r = &$i;\n$r = 9223372036854775807;\n$i++;\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn by_reference_closure_capture_pins_the_variable() {
    let src = "<?php\n$i = 0;\n$f = function () use (&$i) { $i = 9223372036854775807; };\n$f();\n$i++;\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn catch_forgets_variables_written_in_try() {
    let src = "<?php\nfunction t() { throw new Exception('x'); }\n$i = 0;\ntry {\n  $i = 9223372036854775807;\n  t();\n} catch (Exception $e) {\n  $i++;\n}\n";
    assert_not_proven(src, 8, ArithKind::Inc);
}

#[test]
fn finally_writes_are_pinned() {
    let src = "<?php\nfor ($i = 0; $i < 10; $i++) {\n  try {\n    break;\n  } finally {\n    $i = 9223372036854775807;\n  }\n}\n$i++;\n";
    assert_not_proven(src, 9, ArithKind::Inc);
}

#[test]
fn goto_makes_body_unanalyzable() {
    let src = "<?php\n$i = 0;\nstart:\n$i++;\nif ($i < 3) goto start;\n";
    assert_not_proven(src, 4, ArithKind::Inc);
}

#[test]
fn extract_makes_body_unanalyzable() {
    let src = "<?php\nfunction e(array $a) {\n  $i = 0;\n  extract($a);\n  $i++;\n}\n";
    assert_not_proven(src, 5, ArithKind::Inc);
}

#[test]
fn loop_exit_value_is_bounded_by_the_condition() {
    let src = "<?php\n$i = 0;\nwhile ($i < 10) {\n  $i++;\n}\n$j = $i * 1000;\n";
    assert_proven(src, 6, ArithKind::Mul);
}

#[test]
fn break_out_of_nested_loop_keeps_sound_exit_state() {
    let src = "<?php\n$k = 0;\nfor ($i = 0; $i < 10; $i++) {\n  for ($j = 0; $j < 10; $j++) {\n    $k = 9223372036854775807;\n    break 2;\n  }\n}\n$k++;\n";
    assert_not_proven(src, 9, ArithKind::Inc);
}

#[test]
fn operand_order_hazard_forgets_the_variable() {
    let src = "<?php\n$i = 0;\n$x = $i + ($i = 9223372036854775807);\n";
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn count_and_strlen_bounds_are_used() {
    let src = "<?php\nfunction c(array $a, string $s) {\n  return count($a) + strlen($s) + 1;\n}\n";
    assert_proven(src, 3, ArithKind::Add);
}

#[test]
fn method_bodies_and_closures_are_analyzed() {
    let src = "<?php\nclass A {\n  public function m(int $n) {\n    for ($i = 0; $i < $n; $i++) {}\n  }\n}\n$f = function (int $n) {\n  for ($i = 0; $i < $n; $i++) {}\n};\n";
    assert_proven(src, 4, ArithKind::Inc);
    assert_proven(src, 8, ArithKind::Inc);
}

#[test]
fn modulo_and_shift_ranges_are_used() {
    let src = "<?php\nfunction m(int $x) {\n  return ($x % 1000) * 1000 + ($x >> 60) + (($x & 255) << 4);\n}\n";
    assert_proven(src, 3, ArithKind::Mul);
    assert_proven(src, 3, ArithKind::Add);
}

#[test]
fn lockstep_counter_inherits_its_twin_bound() {
    let src = "<?php\n$n = $argc & 255; $j = 0;\nfor ($i = 0; $i < $n; $i++, $j++) {\n  echo $j;\n}\n";
    assert_proven(src, 3, ArithKind::Inc);
}

#[test]
fn lockstep_counter_written_in_body_stays_checked() {
    let src = "<?php\n$n = $argc & 255; $j = 0;\nfor ($i = 0; $i < $n; $i++, $j++) {\n  $j = $j * 2;\n}\n";
    assert_not_proven(src, 3, ArithKind::Inc);
}

#[test]
fn lockstep_counters_with_different_starts_stay_checked() {
    let src = "<?php\n$n = $argc & 255; $j = 1;\nfor ($i = 0; $i < $n; $i++, $j++) {\n  echo $j;\n}\n";
    assert_not_proven(src, 3, ArithKind::Inc);
}

#[test]
fn lockstep_counter_bound_by_reference_stays_checked() {
    let src = "<?php\n$n = $argc & 255; $j = 0; $r = &$j;\nfor ($i = 0; $i < $n; $i++, $j++) {\n  $r = PHP_INT_MAX;\n}\n";
    assert_not_proven(src, 3, ArithKind::Inc);
}

#[test]
fn unbounded_lockstep_counters_stay_checked() {
    let src = "<?php\nfor ($i = PHP_INT_MAX - 1, $j = PHP_INT_MAX - 1, $k = 0; $k < 4; $k++, $i++, $j++) {\n  echo $i * 1;\n}\n";
    assert_not_proven(src, 3, ArithKind::Mul);
}

#[test]
fn direct_call_arguments_seed_int_parameters() {
    let src = "<?php\nfunction a(int $i, int $j): float {\n  $n = $i + $j;\n  return 1.0 / ($n + 1);\n}\nfunction m(int $n): float {\n  $s = 0.0;\n  for ($i = 0; $i < $n; $i++) { for ($j = 0; $j < $n; $j++) { $s += a($i, $j); } }\n  return $s;\n}\necho m(500);\n";
    assert_proven(src, 3, ArithKind::Add);
}

#[test]
fn dynamic_calls_disable_parameter_seeding() {
    let src = "<?php\nfunction a(int $i, int $j): int {\n  return $i + $j;\n}\necho a(1, 2);\n$f = 'strlen';\necho $f('x');\n";
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn function_named_in_a_string_is_not_seeded() {
    let src = "<?php\nfunction a(int $i, int $j): int {\n  return $i + $j;\n}\necho a(1, 2);\necho call_user_func('a', PHP_INT_MAX, 1);\n";
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn unbounded_call_site_keeps_parameter_checked() {
    let src = "<?php\nfunction a(int $i, int $j): int {\n  return $i + $j;\n}\necho a(1, 2);\necho a(intval(\"9\"), 2);\n";
    assert_not_proven(src, 3, ArithKind::Add);
}

#[test]
fn recursive_seeding_stays_sound() {
    let src = "<?php\nfunction f(int $n): int {\n  if ($n > 1000) { return 0; }\n  return f($n * 2 + 1) + $n;\n}\necho f(1);\n";
    assert_not_proven(src, 4, ArithKind::Add);
}

#[test]
fn exported_function_is_not_seeded() {
    let src = "<?php\n#[Export]\nfunction a(int $i, int $j): int {\n  return $i + $j;\n}\necho a(1, 2);\n";
    assert_not_proven(src, 4, ArithKind::Add);
}
