//! Purpose:
//! Regression tests for long-standing bugs found during the Manticore-performance branch review:
//! constant propagation across switch fallthrough and catch edges, sort return values, untyped
//! numeric returns, loop-carried float promotion, function-local hash widening, `global` in
//! closures, unset in loops, shutdown cycle collection, and float exponent formatting.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was produced by the PHP interpreter.

use super::*;

/// Switch fallthrough enters the next case with the previous case's writes (constant propagation used to restart from the switch entry).
#[test]
fn test_switch_fallthrough_keeps_previous_case_writes() {
    let out = compile_and_run(r#"<?php
function sw($x) { $v = 1; switch ($x) { case 1: $v = 100; case 2: $v = $v + 1; break; default: $v = 3; } return $v; }
echo sw(1), " ", sw(2), " ", sw(3), "\n";
function sw2($x) { $v = 1; switch ($x) { case 1: $v = PHP_INT_MAX; case 2: $v = $v + 1; break; default: $v = 3; } return $v; }
echo sw2(1), " ", sw2(2), "\n";
function sw3($x) { $v = 1; switch ($x) { case 1: $v = 9223372036854775806; case 2: $v++; $v++; break; } return $v; }
echo sw3(1), " ", sw3(2), "\n";
"#);
    assert_eq!(out, "101 2 3\n9.2233720368548E+18 2\n9.2233720368548E+18 3\n");
}

/// A `catch` body sees locals written in the `try` body before the throw (constant propagation and dead-store elimination ignored the exception edge).
#[test]
fn test_catch_sees_writes_made_in_try() {
    let out = compile_and_run(r#"<?php
function exc() { $s = "e"; try { $s .= "1"; throw new Exception($s); } catch (Exception $e) { $s .= "2"; echo $e->getMessage(), $s, "\n"; } finally { $s .= "3"; } echo $s, "\n"; }
exc();
function exc2() { $s = "e"; try { $s .= "1"; throw new Exception("x"); } catch (Exception $e) { echo $s, "\n"; } }
exc2();
function exc3() { $s = "e"; $s .= "0"; try { $s .= "1"; $s .= "2"; throw new Exception("x"); } catch (Exception $e) { $s .= "3"; } echo $s, "\n"; }
exc3();
"#);
    assert_eq!(out, "e1e12\ne123\ne1\ne0123\n");
}

/// `sort()`/`ksort()` return `true` like PHP instead of NULL.
#[test]
fn test_sort_builtins_return_true() {
    let out = compile_and_run(r#"<?php
$x = [3, 1, 2]; var_dump(sort($x)); $k = [3 => 1, 1 => 2]; var_dump(ksort($k));
$h = ["b" => 1, "a" => 2]; var_dump(ksort($h)); if (ksort($h)) echo "T\n"; else echo "F\n";
$b = ksort($h); var_dump($b); echo json_encode([ksort($h)]), "\n";
"#);
    assert_eq!(out, "bool(true)\nbool(true)\nbool(true)\nT\nbool(true)\n[true]\n");
}

/// An untyped function returning its own (untyped) parameter returns `mixed`, so a recursive accumulator that overflows to float is not truncated to int.
#[test]
fn test_untyped_accumulator_recursion_keeps_float_results() {
    let out = compile_and_run(r#"<?php
function rec($n, $acc) { if ($n == 0) return $acc; return rec($n - 1, $acc * 3); }
$r = rec(60, 1);
if ($r > 1e20) echo "big\n"; else echo "small\n";
function rec2($n, $acc) { if ($n == 0) return $acc; return rec2($n - 1, $acc + $acc); }
$r = rec2(70, 1);
if ($r > 1e20) echo "big\n"; else echo "small\n";
function fact($n) { if ($n <= 1) return 1; return $n * fact($n - 1); }
echo fact(25), "\n";
function sum($n, $acc = 0) { return $n == 0 ? $acc : sum($n - 1, $acc + PHP_INT_MAX); }
echo sum(3), "\n";
"#);
    assert_eq!(out, "big\nbig\n1.5511210043331E+25\n2.7670116110564E+19\n");
}

/// A loop-carried int that overflows to float is kept as a float on later iterations and printed with PHP's exponent format.
#[test]
fn test_loop_carried_int_promoted_to_float_is_not_truncated() {
    let out = compile_and_run(r#"<?php
function lp() { $i = 0; $j = 0; while ($i < 10) { $i += 3; $j = $j + $i * 1000000000000000000; echo $j, "\n"; } return $j; }
lp();
"#);
    assert_eq!(out, "3000000000000000000\n9000000000000000000\n1.8E+19\n3.0E+19\n");
}

/// Writing an int into a string-valued hash local inside a function widens its values to mixed instead of corrupting the table.
#[test]
fn test_int_written_into_string_valued_function_local_hash() {
    let out = compile_and_run(r#"<?php
function f() {
    $m = ["b" => "x", "a" => "y"];
    $r = $m; $r2 = $r; $r2["new"] = 1; echo json_encode($r), json_encode($r2), "\n";
}
f();
"#);
    assert_eq!(out, r#"{"b":"x","a":"y"}{"b":"x","a":"y","new":1}
"#);
}

/// `global $x` inside a closure binds the top-level variable.
#[test]
fn test_global_in_closure_writes_through() {
    let out = compile_and_run(r#"<?php
$u = 5;
$cl = function() { global $u; $u = PHP_INT_MAX; };
$cl();
var_dump($u);
var_dump($u + 1);
"#);
    assert_eq!(out, "int(9223372036854775807)\nfloat(9.223372036854776E+18)\n");
}

/// `unset($q[$i - 5])` in a loop turns the indexed local into a hash before the loop, so later writes do not treat the hash as a packed array.
#[test]
fn test_sliding_window_unset_on_indexed_local_in_loop() {
    let out = compile_and_run(r#"<?php
function churn() {
    $q = [];
    for ($i = 0; $i < 20; $i++) { $q[$i] = [$i]; unset($q[$i - 5]); }
    echo count($q), json_encode($q), "\n";
}
churn();
"#);
    assert_eq!(out, r#"5{"15":[15],"16":[16],"17":[17],"18":[18],"19":[19]}
"#);
}

/// Cyclic garbage is collected at program end and its destructors run, like PHP's shutdown GC.
#[test]
fn test_cyclic_garbage_destructors_run_at_shutdown() {
    let out = compile_and_run(r#"<?php
class N { public $o; public $n; function __construct($n) { $this->n = $n; } function __destruct() { echo "d", $this->n, " "; } }
function mk() { $a = new N(1); $b = new N(2); $a->o = $b; $b->o = $a; }
mk(); echo "after-mk ";
$x = new N(3); $y = new N(4); $x->o = $y; $y->o = $x; $x = null; $y = null; echo "after-null\n";
"#);
    assert_eq!(out, "after-mk after-null\nd1 d2 d3 d4 ");
}

/// `echo` of floats in exponent form prints PHP's `1.0E+25` / `2.5E-7` forms.
#[test]
fn test_float_echo_exponent_format_matches_php() {
    let out = compile_and_run(r#"<?php echo 1e25, " ", -3e19, " ", 1.5e25, " ", 1e-10, " ", 2.5e-7, " ", 0.1, " ", 1e14, " ", 1e15, " ", INF, " ", -INF, " ", NAN, " ", 123456789012345678.0, "\n"; echo (string)1e100, "\n";"#);
    assert_eq!(out, "1.0E+25 -3.0E+19 1.5E+25 1.0E-10 2.5E-7 0.1 1.0E+14 1.0E+15 INF -INF NAN 1.2345678901235E+17\n1.0E+100\n");
}
/// A by-value argument is evaluated before a later argument reassigns the same variable, so
/// the callee receives the old value instead of a freed one.
#[test]
fn test_argument_reassigned_by_later_argument_keeps_old_value() {
    let out = compile_and_run(
        r#"<?php
function show(string $a, string $b) { echo strlen($a), ":", strlen($b), ":", $a, "\n"; }
function t() { $m = "m"; show($m, $m .= "1234567890123456789012345678901234567890"); }
t();
"#,
    );
    assert_eq!(out, "1:41:m\n");
}

/// `array_sum()` over a `mixed`-valued hash counts raw int slots (stored without boxing when the
/// arithmetic is proven), not only boxed `mixed` cells.
#[test]
fn test_array_sum_over_mixed_hash_with_raw_int_slots() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$m = [];
for ($i = 0; $i < 4; $i++) { $m["k$i"] = $i; }
for ($r = 0; $r < 2; $r++) {
  for ($i = 0; $i < 4; $i++) { $m["k$i"] = $i + $r; }
}
var_dump(array_sum($m));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "int(10)\n");
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}

/// Reference-heavy `unset()` loops stay fast: cycle collection is lazy and linear, and every
/// reference cell is still reclaimed by the end of the program.
#[test]
fn test_unset_reference_loop_collects_lazily_and_cleanly() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$m = [];
$keys = [];
for ($i = 0; $i < 200; $i++) { $m["k" . $i] = 0; $keys[] = "k" . $i; }
for ($r = 0; $r < 60; $r++) {
    foreach ($keys as $k) {
        $ref = &$m[$k];
        $ref = $ref + 1;
        unset($ref);
    }
}
$sum = 0;
foreach ($m as $v) { $sum += $v; }
echo $sum, "\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "12000\n");
    assert!(out.stderr.contains("leak summary: clean"), "{}", out.stderr);
}
