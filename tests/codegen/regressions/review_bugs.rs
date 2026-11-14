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

/// A bare `array` parameter called with different element types and shapes (list of ints,
/// string-keyed hash, list of floats) reads every call's values correctly instead of through
/// the first call's layout.
#[test]
fn test_generic_array_param_called_with_different_shapes() {
    let out = compile_and_run(
        r#"<?php
function first(array $a) { foreach ($a as $k => $v) { return $v; } return null; }
var_dump(first([1, 2]));
var_dump(first(["x" => "hello"]));
var_dump(first([1.5]));
"#,
    );
    assert_eq!(out, "int(1)\nstring(5) \"hello\"\nfloat(1.5)\n");
}

/// A by-value `array` parameter written in place (element write, append, `unset`, `sort`, nested
/// write, compound assignment) is separated from the caller's array by copy-on-write: the callee
/// used to mutate the caller's array (or split it and drop the caller's reference, crashing
/// later), and a nested write-back freed the stored inner array while it was still in use.
#[test]
fn test_by_value_array_param_writes_do_not_reach_the_caller() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function bump(array $a) { $a["delta"] = 9; echo $a["delta"], "\n"; }
function bumpi(array $a) { $a[0] = 9; $a[] = 4; echo $a[0], count($a), "\n"; }
function s(array $a) { sort($a); echo implode(",", $a), "\n"; }
function u(array $a) { unset($a["x"]); echo count($a), "\n"; }
function n(array $a) { $a["in"]["k"] = 5; echo $a["in"]["k"], count($a["in"]), "\n"; }
function p(array $a) { $a[] = 7; echo count($a), "\n"; return $a; }
function c(array $a) { $a["n"] += 5; $a["n"]++; return $a["n"]; }
function run(int $m) {
    $c = ["delta" => $m];
    bump($c);
    echo $c["delta"], "\n";
    $l = [$m, $m];
    $l2 = $l;
    bumpi($l);
    echo $l[0], count($l), $l2[0], "\n";
    $l = [3 * $m, 1, 2];
    s($l); echo implode(",", $l), "\n";
    $h = ["x" => $m, "y" => 2];
    u($h); echo count($h), "\n";
    $g = ["in" => ["k" => $m]];
    n($g); echo $g["in"]["k"], "\n";
    $q = [$m];
    $q2 = p($q); echo count($q), " ", count($q2), "\n";
    $q3 = p(p($q)); echo count($q3), "\n";
    $cn = ["n" => $m];
    echo c($cn), " ", $cn["n"], "\n";
    $loc = ["in" => ["k" => $m]];
    $loc["in"]["k"] = 5;
    echo count($loc["in"]), $loc["in"]["k"], "\n";
}
run($argc);
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "9\n1\n93\n121\n1,2,3\n3,1,2\n1\n2\n51\n1\n2\n1 2\n2\n3\n3\n7 1\n15\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// Assigning through a reference to an element of an int/float-typed array (`$r = &$h["a"]; $r = 5;`)
/// inside a function is read back correctly: the typed element read unboxes a `mixed` value
/// behind the reference, and a float store updates the reference's inner tag.
#[test]
fn test_reference_to_typed_array_element_reads_back_assigned_value() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function run() {
    $h = ["a" => 1, "b" => 2];
    $r = &$h["a"];
    $r = 5;
    echo $h["a"], " ", $h["b"], "\n";
    $l = [1, 2];
    $q = &$l[0];
    $q = 9;
    echo $l[0], $l[1], "\n";
    $f = ["x" => 1.5];
    $p = &$f["x"];
    $p = 2.25;
    echo $f["x"], "\n";
}
run();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "5 2\n92\n2.25\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// An `array` parameter the checker specialized to scalar elements from one call site still
/// accepts a list built from an untyped caller parameter (boxed elements): the call converts the
/// list to the scalar layout instead of handing boxes to code that reads raw ints.
#[test]
fn test_scalar_array_param_accepts_list_of_boxed_elements() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function f(array $a) { return implode(",", $a); }
function s(array $a) { sort($a); return implode(",", $a); }
function run($m) { echo f([$m, 2]), "|", f([1, 2]), "|", s([3 * $m, 1, 2]), "|", s([5, 4]), "\n"; }
run(7);
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "7,2|1,2|1,2,21|4,5\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// `$a[$i] op= $x` on a list of `mixed` elements updates a uniquely owned int box in place, and
/// falls back to the generic read/operate/store on int overflow, float/string/null elements,
/// a copy-on-write shared list, a shared element box and a missing key.
#[test]
fn test_fused_mixed_list_compound_update_matches_generic_semantics() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function upd(array $row, int $n, $m) {
    for ($j = 0; $j < $n; $j++) { $row[$j] += $m * 3; }
    return $row;
}
function ovf() {
    $r = [PHP_INT_MAX - 1, 5, 1.5, "7", null];
    for ($j = 0; $j < 5; $j++) { $r[$j] += 2 * 1; }
    var_dump($r);
    $r[1] -= PHP_INT_MAX; $r[1] -= 100; var_dump($r[1]);
    $r[0] *= 3; var_dump($r[0]);
}
function cow() {
    $a = [1, 2, 3]; $a[0] = $a[0] + 0;
    for ($i = 0; $i < 3; $i++) { $a[$i] += 0 + $i * 2; }
    $b = $a;
    for ($i = 0; $i < 3; $i++) { $b[$i] += 10 * 1; }
    $x = $b[1];
    for ($i = 0; $i < 3; $i++) { $b[$i] *= 2 * 1; }
    var_dump($a, $b, $x);
}
function miss() {
    $a = [1, 2];
    for ($i = 0; $i < 4; $i++) { $a[$i] += 5 * $i; }
    var_dump($a);
}
function refs() {
    $a = [1, 2, 3];
    $r = &$a[1];
    for ($i = 0; $i < 3; $i++) { $a[$i] += 4 * 1; }
    $r += 100;
    var_dump($a);
}
$base = [1, 2, 3];
$out = upd($base, 3, 2);
var_dump($base, $out);
ovf(); cow(); miss();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "array(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n}\narray(3) {\n  [0]=>\n  int(7)\n  [1]=>\n  int(8)\n  [2]=>\n  int(9)\n}\narray(5) {\n  [0]=>\n  float(9.223372036854776E+18)\n  [1]=>\n  int(7)\n  [2]=>\n  float(3.5)\n  [3]=>\n  int(9)\n  [4]=>\n  int(2)\n}\nfloat(-9.223372036854776E+18)\nfloat(2.7670116110564327E+19)\narray(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(4)\n  [2]=>\n  int(7)\n}\narray(3) {\n  [0]=>\n  int(22)\n  [1]=>\n  int(28)\n  [2]=>\n  int(34)\n}\nint(14)\narray(4) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(7)\n  [2]=>\n  int(10)\n  [3]=>\n  int(15)\n}\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// A function, method, static method or closure returning one of its parameters hands the caller a reference of its own: the caller's variable survives `count(f($a))` in a loop, a fresh arithmetic argument returned as is stays valid (`rec($n - 1, $acc * 3)`), and nothing leaks.
#[test]
fn test_returned_parameters_are_owned_by_the_caller() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function ga(array $x) { return $x; }
function gm($x) { return $x; }
function gt($c, $x) { return $c ? $x : 0; }
function rec($n, $acc) { if ($n == 0) return $acc; return rec($n - 1, $acc * 3); }
class C { function id(array $x) { return $x; } static function sid(array $x) { return $x; } function keep(string $s): string { return $s; } }
$a = [1, 2];
$c = new C;
for ($i = 0; $i < 3; $i++) { echo count(ga($a)), count($c->id($a)), count(C::sid($a)); }
echo count($a), "\n";
$f = function (array $x) { return $x; };
for ($i = 0; $i < 3; $i++) { echo count($f($a)); }
echo count($a), "\n";
$g = function () { return [1, 2]; }; echo count($g()), "\n";
echo gm($a[0] * 3), gt(true, $a[0] * 3), "\n";
$r = rec(60, 1); echo $r > 1e20 ? "big" : "small", "\n";
$s = "abc" . $i; $t = $c->keep($s); $s = "x"; echo $t, "\n";
var_dump($a);
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2222222222\n2222\n2\n33\nbig\nabc3\narray(2) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n}\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// PHP 8 compares a bool with anything, and `null` with an int or float, as two booleans (`null < -1` is `false < true`), in folded constants, typed operands and boxed values.
#[test]
fn test_null_and_bool_compare_as_booleans() {
    let out = compile_and_run_with_heap_debug(r#"<?php
var_dump(null < -1, null > -1, null == 0, null == 1, null < 0.5, null <=> -3, 0 > null, null >= false, true > null);
$n = null; $i = -1; var_dump($n < $i, $n == $i, $n <=> $i, $i > $n);
$big = [PHP_INT_MAX, null]; var_dump($big[1] < -1);
function m($a, $b) { return [$a < $b, $a == $b, $a <=> $b]; }
foreach ([[null, -1], [null, 0], [true, 5], [false, "a"], [true, []], [null, 2.5], [null, ""], [null, "a"], [1, true], ["0", false]] as [$a, $b]) { echo json_encode(m($a, $b)), json_encode(m($b, $a)); }
echo "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "bool(true)\nbool(false)\nbool(true)\nbool(false)\nbool(true)\nint(-1)\nbool(false)\nbool(true)\nbool(true)\nbool(true)\nbool(false)\nint(-1)\nbool(true)\nbool(true)\n[true,false,-1][false,false,1][false,true,0][false,true,0][false,true,0][false,true,0][true,false,-1][false,false,1][false,false,1][true,false,-1][true,false,-1][false,false,1][false,true,0][false,true,0][true,false,-1][false,false,1][false,true,0][false,true,0][false,true,0][false,true,0]\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// A miss-capable `int|null` read (untyped or `?int` property, int-list element) passed to a
/// `mixed` or untyped parameter is boxed for the callee instead of handing it the raw tagged word
/// (which crashed the callee).
#[test]
fn test_tagged_scalar_argument_to_mixed_parameter_is_boxed() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function v(mixed $x) { var_dump($x); }
function w($x) { return $x === null ? "null" : $x * 2; }
class Car { public $pippo; public ?int $n = null; }
$c = new Car(); v($c->pippo); $c->pippo = 5; v($c->pippo); v($c->n); $c->n = 7; v($c->n);
$a = [1, 2, 3]; v($a[1]); echo w($a[2]), w($c->n), "\n";
for ($i = 0; $i < 3; $i++) { echo w($a[$i]), " "; }
echo "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "NULL\nint(5)\nNULL\nint(7)\nint(2)\n614\n2 4 6 \n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// Regression: a method call on an element borrowed from a list is not borrowed when the method may run user code (a closure stored in a
/// property) that replaces the list; the receiver must stay alive for the whole call.
#[test]
fn test_method_receiver_from_list_survives_list_replacement_r1() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class Holder { public array $items = []; }
abstract class S { public ?Holder $h = null; public $v = 5; public $name = "nm"; public ?Closure $cb = null; abstract function area(): int; function label(): string { return $this->name . $this->v; } }
class A extends S { function area(): int { return 1; } }
class B extends S { function area(): int { if ($this->h !== null) { $this->h->items = []; } $junk = [new A(), new A(), new A()]; return $this->v + count($junk); } }
class C extends S { function area(): int { if ($this->cb !== null) { ($this->cb)(); } $junk = [1, 2, 3, 4]; return $this->v * 2; } function label(): string { global $G; $G = []; $j = ["a" . $this->v, "b"]; return $this->name . count($j); } }
function t1() {
  $h = new Holder(); $b = new B(); $b->h = $h;
  $h->items = [new A(), $b];
  $b = null;
  for ($i = 0; $i < count($h->items); $i++) { echo $h->items[$i]->area(), " "; }
  echo count($h->items), "\n";
}
function t2() {
  $shapes = [new A(), new C()];
  $c = $shapes[1];
  $c->cb = function() use (&$shapes) { $shapes = [new A()]; };
  $c = null;
  $n = count($shapes);
  for ($i = 0; $i < $n && $i < count($shapes); $i++) { echo $shapes[$i]->area(), " "; }
  echo count($shapes), "\n";
}
$G = [new A(), new C()];
function t3() { global $G; for ($i = 0; $i < 2 && $i < count($G); $i++) { echo $G[$i]->label(), " "; } echo count($G), "\n"; }
function t4() {
  $xs = [new A(), new B(), null];
  for ($i = 0; $i < 3; $i++) { if ($xs[$i] === null) { echo "null "; } else { echo $xs[$i]->area(), $xs[$i]->label(), " "; } }
  echo $xs[2]?->area() ?? "ns", "\n";
  try { echo $xs[2]->area(); } catch (Error $e) { echo get_class($e), ":", $e->getMessage(), "\n"; }
}
t1(); t2(); t3(); t4();
"#);
    assert_eq!(out.stdout, "1 8 0\n1 10 1\nnm5 nm2 0\n1nm5 8nm5 null ns\nError:Call to a member function area() on null\n");
}

/// Regression: a method call on an element of a list of sibling objects is not borrowed when the method may run user code (a closure stored in a
/// property) that replaces the list; the receiver must stay alive for the whole call.
#[test]
fn test_method_receiver_from_list_survives_list_replacement_r2() {
    let out = compile_and_run_with_heap_debug(r#"<?php
abstract class S { public $v = 5; public ?Closure $cb = null; abstract function area(): int; }
class A extends S { function area(): int { return 1; } }
class C extends S { function area(): int { if ($this->cb !== null) { ($this->cb)(); } $junk = [1, 2, 3, 4]; return $this->v * 2; } }
function t2() {
  $shapes = [new A(), new C()];
  $c = $shapes[1];
  $c->cb = function() use (&$shapes) { $shapes = [new A()]; };
  $c = null;
  $n = count($shapes);
  for ($i = 0; $i < $n && $i < count($shapes); $i++) { echo $shapes[$i]->area(), " "; }
  echo count($shapes), "\n";
}
t2();
"#);
    assert_eq!(out.stdout, "1 10 1\n");
}

/// A typed list (sibling objects, ints) assigned to a static `array` property is widened to
/// boxed slots; a borrowed local stays independent of the property afterwards.
#[test]
fn test_typed_list_assigned_to_static_array_property() {
    let out = compile_and_run(r#"<?php
abstract class S { public $v = 5; function area(): int { return $this->v; } }
class A extends S {} class B extends S { function area(): int { return 9; } }
class Reg { public static array $all = []; public static array $ints = []; }
Reg::$all = [new A(), new B()]; echo count(Reg::$all), Reg::$all[1]->area(), "\n";
$x = [new B(), new A(), new B()];
Reg::$all = $x;
$x[] = new A();
echo count($x), count(Reg::$all), Reg::$all[0]->area(), $x[3]->area(), "\n";
Reg::$all[] = new A();
echo count(Reg::$all), count($x), "\n";
Reg::$all = [new A(), new A()]; echo Reg::$all[1]->area(), "\n";
$n = [1, 2, 3]; Reg::$ints = $n; $n[0] = 7; echo Reg::$ints[0], $n[0], "\n";
function fill() { Reg::$ints = [4, 5]; return count(Reg::$ints); }
echo fill(), "\n";
"#);
    assert_eq!(out, "29\n4395\n44\n5\n17\n2\n");
}

/// An untyped identity function (`function idu($u) { return $u; }`) returning an array or hash hands back its own reference: the caller's array survives and nothing leaks.
#[test]
fn test_untyped_identity_function_keeps_caller_array() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function idu($u) { return $u; }
function f() { $a = [1,2,3]; $x = idu($a); $x = idu($x); echo count($x), "|", count($a), "\n"; }
f();
function g() { $a = [1,2,3]; for ($i = 0; $i < 3; $i++) { $x = idu($a); $x = idu($x); } $x[] = 4; echo count($x), count($a), "\n"; }
g();
function h() { $m = ['a' => 1, 'b' => [2]]; $y = idu($m); $y['c'] = 3; echo count($y), count($m), "\n"; }
h();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "3|3\n43\n32\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// A closure called directly through its variable releases owned argument temporaries (a literal array, a widened local, a new object) after the call.
#[test]
fn test_direct_closure_call_releases_argument_temporaries() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class O { public $v = 1; }
function t() {
  $g = function(array $x) { return $x; };
  echo count($g([4])), "\n";
  $a = [1, 2];
  echo count($g($a)), count($a), "\n";
  $h = fn(O $o) => $o;
  echo $h(new O())->v, "\n";
}
t();
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "1\n22\n1\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// `array_map` with a named identity callback does not leak the elements it passes.
#[test]
fn test_array_map_identity_callback_does_not_leak() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function ida(array $a) { return $a; }
$r = array_map("ida", [[1],[2]]);
echo count($r), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "2\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// A typed `bool` compared with an int or float, and bool literals in constant folding, compare as two booleans (`true <=> 2` is 0, `false < -1` is true).
#[test]
fn test_typed_bool_comparisons_use_boolean_semantics() {
    let out = compile_and_run(r#"<?php
function bi(bool $a, int $b, float $f) { var_dump($a <=> $b, $a < $b, !$a < -1, $f <=> $a, $a == $b, $a > $f); }
bi(true, 2, 1.5); bi(false, -1, 0.0); bi(true, 0, -0.5);
var_dump(true <=> 2, false < -1, 1.5 <=> true, true == 3, false >= 0);
"#);
    assert_eq!(out, "int(0)\nbool(false)\nbool(true)\nint(0)\nbool(true)\nbool(false)\nint(-1)\nbool(true)\nbool(false)\nint(0)\nbool(false)\nbool(false)\nint(1)\nbool(false)\nbool(true)\nint(0)\nbool(false)\nbool(false)\nint(0)\nbool(true)\nint(0)\nbool(true)\nbool(true)\n");
}

/// `call_user_func` with an identity closure taking `mixed` passes the argument-container
/// elements borrowed (the callee hands back its own reference) and leaks nothing.
#[test]
fn test_call_user_func_identity_closure_does_not_leak() {
    let out = compile_and_run_with_heap_debug(r#"<?php
$c = function (mixed $x) { return $x; };
echo call_user_func($c, "two"), ":", call_user_func($c, 1), "\n";
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "two:1\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "heap: {}", out.stderr);
}

/// Arguments spread into `Fiber::start()` are transferred to the fiber: a parameter suspended
/// out to the caller stays alive after the call (heap-debug poisoning exposes a freed read).
#[test]
fn test_fiber_start_spread_argument_suspended_to_caller_stays_alive() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function make(): Fiber {
    return new Fiber(function(mixed $a, mixed $b, mixed $c): void {
        echo gettype($a) . ":" . $a . "\n";
        echo gettype($b) . ":" . $b . "\n";
        $x = Fiber::suspend($c);
        echo gettype($x) . ":" . $x . "\n";
    });
}

$args = ["b" => "bee", "a" => 7, "c" => ["k" => 1]];
$f = make();
$ret = $f->start(...$args);
echo $ret["k"] . "\n";
$f->resume("done");
"#);
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "integer:7\nstring:bee\n1\nstring:done\n");
}

/// A nullable int (`?int` parameter, int-list element) ordered against an int or float compares
/// a null as a boolean (`null < -1` is true, `null <=> 0` is 0) and an int numerically.
#[test]
fn test_nullable_int_ordered_comparison_treats_null_as_bool() {
    let out = compile_and_run(r#"<?php
function ni(?int $a, int $b) { var_dump($a < $b, $a > $b, $a <=> $b, $a <= $b, $b >= $a, $b <=> $a); }
ni(null, -1); ni(null, 0); ni(null, 5); ni(3, 2); ni(-4, -4);
function nf(?int $a, float $f) { var_dump($a < $f, $a <=> $f); }
nf(null, -0.5); nf(null, 0.0); nf(2, 2.5);
$list = [5, 1, 9];
$best = 3;
$cnt = 0;
for ($i = 0; $i < 3; $i++) { if ($list[$i] < $best) { $cnt++; } }
echo $cnt, "\n";
"#);
    assert_eq!(out, "bool(true)\nbool(false)\nint(-1)\nbool(true)\nbool(true)\nint(1)\nbool(false)\nbool(false)\nint(0)\nbool(true)\nbool(true)\nint(0)\nbool(true)\nbool(false)\nint(-1)\nbool(true)\nbool(true)\nint(1)\nbool(false)\nbool(true)\nint(1)\nbool(false)\nbool(false)\nint(-1)\nbool(false)\nbool(false)\nint(0)\nbool(true)\nbool(true)\nint(0)\nbool(true)\nint(-1)\nbool(false)\nint(0)\nbool(true)\nint(-1)\n1\n");
}
