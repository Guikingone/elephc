//! Purpose:
//! Integration tests for integer keys that skip slots of a non-empty packed local array.
//!
//! Called from:
//! - `cargo test` through Rust's test harness, via `arrays/mod.rs`.
//!
//! Key details:
//! - PHP promotes a packed array to a hash the moment an integer key skips a slot or is
//!   negative; packed storage must never zero-fill the gap. Keys the checker can bound (a
//!   literal `0`, the unconditional counter of a `for` loop starting at `0`) stay packed.

use crate::support::*;

/// Literal and variable keys past the end of a typed packed local keep PHP's hash shape.
#[test]
fn test_nonempty_local_skipping_key_promotes_to_hash() {
    let out = compile_and_run(r#"<?php
function lit(): array { $a = [1, 2]; $a[5] = 3; return $a; }
function var_key(int $k, int $n): array { $a = [1]; $a[$k] = $n; return $a; }
function neg(int $k): array { $a = [1, 2]; $a[$k] = 9; return $a; }
function str_then_int(int $k, int $n): array { $a = []; $a[0] = "s"; $a[$k] = $n; return $a; }
function in_range(int $k): array { $a = [1, 2, 3]; $a[$k] = 7; return $a; }
echo json_encode(lit()), "\n";
echo json_encode(var_key(3, 6)), " ", count(var_key(3, 6)), "\n";
echo json_encode(neg(-1)), "\n";
echo json_encode(str_then_int(3, 6)), "\n";
echo json_encode(in_range(1)), " ", json_encode(in_range(3)), "\n";
$a = [10, 20];
$a[4] = 50;
$a[] = 60;
echo json_encode($a), " ", $a[0] + $a[4], "\n";
foreach ($a as $k => $v) { echo $k, "=", $v, " "; }
echo "\n";
"#);
    assert_eq!(
        out,
        "{\"0\":1,\"1\":2,\"5\":3}\n{\"0\":1,\"3\":6} 2\n{\"0\":1,\"1\":2,\"-1\":9}\n{\"0\":\"s\",\"3\":6}\n[1,7,3] [1,2,3,7]\n{\"0\":10,\"1\":20,\"4\":50,\"5\":60} 60\n0=10 1=20 4=50 5=60 \n"
    );
}

/// Skipping writes inside loops and branches promote the local for the code after them.
#[test]
fn test_nonempty_local_skipping_key_in_loops_and_branches() {
    let out = compile_and_run(r#"<?php
function loop_ids(array $ids): array {
    $rows = [0];
    foreach ($ids as $id) { $rows[$id] = $id * 2; }
    return $rows;
}
function branch(int $k): array {
    $a = [1, 2];
    if ($k > 0) { $a[$k] = 5; }
    $a[] = 9;
    return $a;
}
function counter_from(int $start): array {
    $a = [7];
    for ($i = $start; $i < $start + 2; $i++) { $a[$i] = $i; }
    return $a;
}
function int_keys(int $n): array {
    $a = [9];
    for ($i = 0; $i < $n; $i++) { $k = intdiv($i, 1) * 3 + 2; $a[$k] = $k; }
    return $a;
}
function dense(int $n): array {
    $a = [5];
    for ($i = 0; $i < $n; $i++) { $a[$i] = $i * 10; }
    return $a;
}
echo json_encode(loop_ids([3, 7])), "\n";
echo json_encode(branch(4)), " ", json_encode(branch(0)), "\n";
echo json_encode(counter_from(5)), " ", json_encode(counter_from(0)), "\n";
echo json_encode(dense(3)), " ", json_encode(int_keys(3)), "\n";
"#);
    assert_eq!(
        out,
        "{\"0\":0,\"3\":6,\"7\":14}\n{\"0\":1,\"1\":2,\"4\":5,\"5\":9} [1,2,9]\n{\"0\":7,\"5\":5,\"6\":6} [0,1]\n[0,10,20] {\"0\":9,\"2\":2,\"5\":5,\"8\":8}\n"
    );
}

/// A `??=` insert past the end of a non-empty packed local creates the key like PHP.
#[test]
fn test_nonempty_local_skipping_key_null_coalesce_insert() {
    let out = compile_and_run(r#"<?php
function upd(int $k): array {
    $a = [1, 2];
    $a[$k] ??= 4;
    return $a;
}
echo json_encode(upd(6)), " ", json_encode(upd(1)), "\n";
"#);
    assert_eq!(out, "{\"0\":1,\"1\":2,\"6\":4} [1,2]\n");
}

/// Count-bounded sorting loops keep producing PHP's results on their proven packed writes.
#[test]
fn test_count_bounded_loop_writes_sort_correctly() {
    let out = compile_and_run(r#"<?php
function bubble($arr) {
    $n = count($arr);
    for ($i = 0; $i < $n - 1; $i++) {
        for ($j = 0; $j < $n - $i - 1; $j++) {
            if ($arr[$j] > $arr[$j + 1]) { $tmp = $arr[$j]; $arr[$j] = $arr[$j + 1]; $arr[$j + 1] = $tmp; }
        }
    }
    return $arr;
}
function select($a) {
    $n = count($a);
    for ($i = 0; $i < $n; $i++) {
        for ($j = $i + 1; $j < $n; $j++) {
            if ($a[$j] < $a[$i]) { $t = $a[$i]; $a[$i] = $a[$j]; $a[$j] = $t; }
        }
    }
    return $a;
}
function shift_right($a) {
    for ($i = 0; $i < count($a) - 1; $i++) { $a[$i + 1] = $a[$i]; }
    return $a;
}
echo json_encode(bubble([5, 3, 8, 1])), json_encode(select([4, 2, 9, 0])), json_encode(shift_right([7, 1, 2])), "\n";
"#);
    assert_eq!(out, "[1,3,5,8][0,2,4,9][7,7,7]\n");
}

/// Counter-indexed sorts keep typed slots and still sort exactly as PHP does.
#[test]
fn test_promotable_counter_sort_matches_php() {
    let out = compile_and_run(r#"<?php
function insertion(int $n) {
    $arr = [];
    $seed = 7;
    for ($i = 0; $i < $n; $i++) { $seed = ($seed * 31 + 5) % 1000; $arr[] = $seed; }
    for ($i = 1; $i < $n; $i++) {
        $key = $arr[$i];
        $j = $i - 1;
        while ($j >= 0 && $arr[$j] > $key) { $arr[$j + 1] = $arr[$j]; $j--; }
        $arr[$j + 1] = $key;
    }
    return $arr;
}
echo json_encode(insertion(8)), "\n";
"#);
    assert_eq!(out, "[47,222,327,462,502,567,582,887]\n");
}

/// A boxed key that skips a slot promotes a typed local at run time; every later operation sees
/// PHP's hash: reads, `isset`, appends, `count`, `foreach`, whole-value consumers and returns.
#[test]
fn test_promotable_local_promotes_on_a_skipping_boxed_key() {
    let out = compile_and_run(r#"<?php
function churn(int $n) {
    $a = [10, 20, 30];
    for ($i = 0; $i < 3; $i++) { $j = $i + 1; $a[$j] = $a[$i]; }
    $k = $n + 5;
    $a[$k] = 99;
    $a[] = 7;
    $e = isset($a[6]) ? "y" : "n";
    $sum = 0;
    foreach ($a as $v) { $sum += $v; }
    return [json_encode($a), count($a), $a[6], $e, $sum, implode(",", $a)];
}
function negative(int $n) {
    $a = [5, 6];
    $a[$n - 2] = 7;
    return $a;
}
echo implode(" | ", churn(1)), "\n";
echo json_encode(negative(1)), "\n";
"#);
    assert_eq!(
        out,
        "{\"0\":10,\"1\":10,\"2\":10,\"3\":10,\"6\":99,\"7\":7} | 6 | 99 | y | 146 | 10,10,10,10,99,7\n{\"0\":5,\"1\":6,\"-1\":7}\n"
    );
}

/// A missed boxed-key read written back into a promotable local stores PHP's `null`, not `0`.
#[test]
fn test_promotable_local_keeps_null_from_a_missed_read() {
    let out = compile_and_run(r#"<?php
function holes(int $n) {
    $a = [1, 2, 3];
    $j = $n + 9;
    $a[$n] = @$a[$j];
    return [json_encode($a), isset($a[1]) ? "set" : "unset", count($a), $a[1] === null ? "null" : "int"];
}
echo implode(" ", holes(1)), "\n";
"#);
    assert_eq!(out, "[1,null,3] unset 3 null\n");
}

/// Promotable writes, promotions, null entries and whole-value views release every owner.
#[test]
fn test_promotable_local_operations_do_not_leak() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function churn(int $n) {
    $arr = [5, 3, 8, 1, 9, 2];
    for ($i = 1; $i < count($arr); $i++) {
        $key = $arr[$i];
        $j = $i - 1;
        while ($j >= 0 && $arr[$j] > $key) { $arr[$j + 1] = $arr[$j]; $j--; }
        $arr[$j + 1] = $key;
    }
    $sorted = json_encode($arr);
    $k = $n + 20;
    $arr[$k] = @$arr[$k + 1];
    $arr[] = 4;
    return $sorted . " " . json_encode($arr) . " " . count($arr);
}
for ($r = 0; $r < 3; $r++) { echo churn($r), "\n"; }
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        "[1,2,3,5,8,9] {\"0\":1,\"1\":2,\"2\":3,\"3\":5,\"4\":8,\"5\":9,\"20\":null,\"21\":4} 8\n[1,2,3,5,8,9] {\"0\":1,\"1\":2,\"2\":3,\"3\":5,\"4\":8,\"5\":9,\"21\":null,\"22\":4} 8\n[1,2,3,5,8,9] {\"0\":1,\"1\":2,\"2\":3,\"3\":5,\"4\":8,\"5\":9,\"22\":null,\"23\":4} 8\n",
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A promotable `array<int>` crosses a declared `array<int>` parameter and return raw: the callee
/// keeps typed slots, promotes on its own skipping write, and callers see PHP's arrays.
#[test]
fn test_promotable_array_crosses_declared_array_int_boundaries() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function shift_into(array<int> $p, int $k): array<int> {
    $n = count($p);
    for ($i = 0; $i < $n; $i++) { $j = $i + $k; $p[$j] = $p[$i]; }
    return $p;
}
function total(array<int> $p): int {
    $s = 0;
    foreach ($p as $v) { $s += $v; }
    return $s;
}
function run(int $n) {
    $a = [1, 2, 3];
    for ($i = 0; $i < 2; $i++) { $j = $i + 1; $a[$j] = $a[$i]; }
    $b = shift_into($a, $n);
    $c = shift_into($b, 5);
    return json_encode($a) . " " . json_encode($b) . " " . json_encode($c) . " " . count($c)
        . " " . total($c);
}
echo run(1), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        "[1,1,1] [1,1,1,1] {\"0\":1,\"1\":1,\"2\":1,\"3\":1,\"5\":1,\"6\":1,\"7\":1,\"8\":1} 8 8\n",
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A promotable `array<int>` passed to an undeclared parameter reaches a typed clone raw, while
/// plain and dynamic callers keep the original; PHP sees one function under its own name.
#[test]
fn test_promotable_array_reaches_typed_clone_of_untyped_param() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function shift_into($p, $k) {
    $n = count($p);
    for ($i = 0; $i < $n; $i++) { $j = $i + $k; $p[$j] = $p[$i]; }
    return $p;
}
function where($p) {
    $p[] = 1;
    return __FUNCTION__ . ":" . debug_backtrace()[0]["function"] . ":" . count($p);
}
function run(int $n) {
    $a = [1, 2, 3];
    for ($i = 0; $i < 2; $i++) { $j = $i + 1; $a[$j] = $a[$i]; }
    $b = shift_into($a, $n);
    $c = shift_into([7, 8], 3);
    $f = 'shift_into';
    $d = $f($b, 5);
    return json_encode($a) . " " . json_encode($b) . " " . json_encode($c) . " "
        . json_encode($d) . " " . where($a);
}
echo run(1), "\n";
var_dump(function_exists('shift_into'), function_exists('shift_into#__promotable_0'));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        "[1,1,1] [1,1,1,1] {\"0\":7,\"1\":8,\"3\":7,\"4\":8} \
         {\"0\":1,\"1\":1,\"2\":1,\"3\":1,\"5\":1,\"6\":1,\"7\":1,\"8\":1} where:where:4\n\
         bool(true)\nbool(false)\n",
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A boxed untyped parameter that a skipping key promoted to a hash is returned as that hash,
/// not walked as packed slots (which crashed).
#[test]
fn test_untyped_param_skipping_key_returns_its_hash() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function f($p) { $p[3] = 7; return $p; }
function g($p, $k) {
    $n = count($p);
    for ($i = 0; $i < $n; $i++) { $j = $i + $k; $p[$j] = $p[$i]; }
    return $p;
}
echo json_encode(f([7, 8])), " ", json_encode(g([7, 8], 3)), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        "{\"0\":7,\"1\":8,\"3\":7} {\"0\":7,\"1\":8,\"3\":7,\"4\":8}\n",
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Counts PHP's precision deprecations for float keys in a program's stderr.
fn float_key_deprecations(stderr: &str) -> usize {
    stderr.matches("Implicit conversion from float").count()
}

/// A float variable key past the end, or negative, promotes a non-promotable packed local
/// (float and string elements) to a hash and reports its precision loss once.
#[test]
fn test_nonempty_local_skipping_float_variable_key_promotes_to_hash() {
    let out = compile_and_run_capture(r#"<?php
function past(float $k): string { $a = [1.0, 2.0]; $a[$k] = 9.0; return json_encode($a) . " " . count($a); }
function names(float $k): string { $a = ["x", "y"]; $a[$k] = "z"; return json_encode($a) . " " . count($a); }
echo past($argc + 4.5), "\n", past(-1.5), "\n", names($argc + 3.25), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(
        out.stdout,
        "{\"0\":1,\"1\":2,\"5\":9} 3\n{\"0\":1,\"1\":2,\"-1\":9} 3\n{\"0\":\"x\",\"1\":\"y\",\"4\":\"z\"} 3\n"
    );
    assert_eq!(float_key_deprecations(&out.stderr), 3, "{}", out.stderr);
}

/// A boxed fractional key on a promotable local reports its precision loss once per write,
/// both when it promotes the packed array and when it writes the already-promoted hash.
#[test]
fn test_promotable_local_boxed_float_key_warns_once_per_write() {
    let out = compile_and_run_capture(r#"<?php
function put(mixed $k, mixed $m): int {
    $a = [0];
    $a[$k] = 10;
    $a[$m] = 20;
    return $a[0] + count($a);
}
echo put(2.5, 3.5), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "3\n");
    assert_eq!(float_key_deprecations(&out.stderr), 2, "{}", out.stderr);
}

/// A packed `array<int>` local with no write that can promote it (none, an append, a write at
/// literal key `0`) reads as `mixed` under a counter or boxed key, as without promotable locals:
/// reassigning an `int` local from it and indexing a buffer with it compile.
#[test]
fn test_unpromotable_int_array_counter_reads_keep_mixed_view() {
    let out = compile_and_run(r#"<?php
function pick(int $n): string {
    $cells = [2, 0, 1];
    $y = 0;
    if ($n > 0) {
        $y = $cells[$n + 1];
    }
    $out = "";
    for ($i = $n - 1; $i < 3; $i++) {
        $out .= $cells[$i] . ",";
    }
    return $out . ($y ?? -1);
}
function appended(int $n): string {
    $cells = [2, 0, 1];
    $cells[] = 4;
    $y = 0;
    if ($n > 0) {
        $y = $cells[$n + 1];
    }
    return (string)($y ?? -1);
}
function fill(): int {
    $cellOf = [1, 0, 1];
    $cellOf[0] = 1;
    $slots = buffer_new<int>(2);
    $item = 1;
    $slots[$cellOf[$item + 1]] = 7;
    return $slots[1];
}
$cellOf = [1, 0, 1];
$slots = buffer_new<int>(2);
for ($item = 2; $item >= 0; $item--) {
    $slots[$cellOf[$item]] += $item + 1;
}
echo pick($argc), "|", appended($argc), "|", fill(), "|", $slots[0], ",", $slots[1], "\n";
"#);
    assert_eq!(out, "2,0,1,1|1|7|2,4\n");
}

/// A later write that cannot promote (a string cast or concatenation key, a key still bound by an earlier
/// `foreach`, a value that is never an integer) leaves a counter-key read checked before it on
/// the `mixed` view, so assigning that read to an `int` local inside a branch compiles.
#[test]
fn test_unpromotable_later_writes_keep_mixed_view_for_earlier_reads() {
    let out = compile_and_run(r#"<?php
function after_cast(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[(string)$n] = 1; return (string)($y ?? -1); }
function after_concat(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells["k" . $n] = 1; return (string)($y ?? -1); }
function after_foreach(int $n): string { $cells = [2, 0, 1]; foreach ([10] as $k => $v) {} $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$k] = 1; return (string)($y ?? -1); }
function after_string_value(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = "x"; return (string)($y ?? -1); }
function after_float_value(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = 1.5; return (string)($y ?? -1); }
echo after_cast($argc), after_concat($argc), after_foreach($argc), after_string_value($argc), after_float_value($argc), "\n";
"#);
    assert_eq!(out, "11111\n");
}

/// A later write the checker does not promote, whatever its syntax (arithmetic typed `mixed`, a
/// division, a comparison, a negation, a `::class` or suppressed string key, a bool, `null`, array
/// or concatenated value), denies the view its walk gave: the body is walked again and the read
/// checked before that write keeps the `mixed` view.
#[test]
fn test_unpromoted_writes_deny_the_view_and_rewalk_the_body() {
    let out = compile_and_run(r#"<?php
class Box {}
function plus(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = $n + 1; return (string)($y ?? -1); }
function half(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = $n / 2; return (string)($y ?? -1); }
function cmp(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = $n > 0; return (string)($y ?? -1); }
function neg(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = -$n; return (string)($y ?? -1); }
function klass(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[Box::class] = 1; return (string)($y ?? -1); }
function quiet(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[@((string)$n)] = 1; return (string)($y ?? -1); }
function flag(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = false; return (string)($y ?? -1); }
function nothing(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = null; return (string)($y ?? -1); }
function nested(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = []; return (string)($y ?? -1); }
function joined(int $n): string { $cells = [2, 0, 1]; $y = 0; if ($n > 0) { $y = $cells[$n + 1]; } $cells[$n] = "a" . "b"; return (string)($y ?? -1); }
echo plus($argc), half($argc), cmp($argc), neg($argc), klass($argc), quiet($argc), flag($argc), nothing($argc), nested($argc), joined($argc), "\n";
"#);
    assert_eq!(out, "1111111111\n");
}

/// A promotion an earlier walk made does not outlive the walk that drops it: a literal-key read of
/// a local only a denied read fed keeps `int`. A chain of four method locals, each promoted only
/// through the next one's view, converges although every pass denies one more view.
#[test]
fn test_view_denials_chain_through_locals_and_method_passes() {
    let out = compile_and_run(r#"<?php
function chained(int $n): string {
    $cells = [2, 0, 1];
    $other = [9, 8, 7];
    $saved = 0;
    $rd = $cells[$n + 1];
    if ($n > 0) {
        $saved = $other[0];
    }
    $other[$n] = $rd;
    $cells[$n] = $n + 1;
    return (string)($saved ?? -1);
}
class C {
    function pick(int $n): string {
        $a = [2, 0, 1];
        $b = [2, 0, 1];
        $c = [2, 0, 1];
        $d = [2, 0, 1];
        $saved = 0;
        $rd = $d[$n + 1];
        $rc = $c[$n + 1];
        $rb = $b[$n + 1];
        $ra = $a[$n + 1];
        if ($n > 0) {
            $saved = $ra;
        }
        $c[$n] = $rd;
        $b[$n] = $rc;
        $a[$n] = $rb;
        $d[$n] = $n + 1;
        return (string)($saved ?? -1);
    }
}
echo chained($argc), "|", (new C)->pick($argc), "\n";
"#);
    assert_eq!(out, "9|1\n");
}

/// A user function named `main` does not share the top-level body's walk state, so calling it
/// after a top-level read does not keep that read's denied `?int` view.
#[test]
fn test_user_function_named_main_keeps_top_level_views_apart() {
    let out = compile_and_run(r#"<?php
function main(int $n): int {
    return $n;
}
$cells = [2, 0, 1];
$y = 0;
if ($argc > 0) {
    $y = $cells[$argc + 1];
}
$cells[$argc] = $argc + 1;
main(1);
echo (string)($y ?? -1), "\n";
"#);
    assert_eq!(out, "1\n");
}

/// A proven `array<float>` (and a typed clone's own float-list result) passed to an untyped
/// `array` parameter reaches a float-typed clone: chained calls stay typed, a callee write with
/// a negative or skipping key still promotes the copy to a hash, and a dynamic call with a mixed
/// list keeps the original.
#[test]
fn test_float_list_reaches_typed_clone_of_untyped_param() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function A($i, $j) { return 1.0 / ((($i + $j) * ($i + $j + 1) >> 1) + $i + 1); }
function Av($n, $v) {
    $out = [];
    for ($i = 0; $i < $n; $i++) { $s = 0.0; for ($j = 0; $j < $n; $j++) { $s += A($i, $j) * $v[$j]; } $out[$i] = $s; }
    return $out;
}
function scale($v, $k) { $v[$k] = -1.5; return $v; }
function run(int $n) {
    $u = [];
    for ($i = 0; $i < $n; $i++) { $u[$i] = 1.0; }
    $v = Av($n, $u);
    $w = Av($n, $v);
    $x = scale($w, -3);
    $y = scale($w, $n + 2);
    $f = 'Av';
    $z = $f($n, [1, 2.5, "3"]);
    return json_encode($u) . " " . json_encode($w) . " " . json_encode($x) . " " . json_encode($y) . " " . json_encode($z);
}
echo run(3), "\n";
var_dump(function_exists('Av'));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "[1,1,1] [2.1678418803418804,0.7593376068376068,0.39209949594564975] {\"0\":2.1678418803418804,\"1\":0.7593376068376068,\"2\":0.39209949594564975,\"-3\":-1.5} {\"0\":2.1678418803418804,\"1\":0.7593376068376068,\"2\":0.39209949594564975,\"5\":-1.5} [3,1.2083333333333333,0.6752136752136753]\nbool(true)\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A `for` counter passed by value to a user function keeps its packed array, while a counter a
/// by-reference closure capture, a by-reference parameter or a `global` binding advances still
/// produces PHP's sparse hash.
#[test]
fn test_counter_aliasing_callees_keep_sparse_keys() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function fill($a, $n) { for ($k = 0; $k < $n; $k++) { $a[$k] = $k * 0.5; } return $a; }
function bump(&$i) { $i += 3; }
function viaglobal() { global $i; $i += 3; }
function run(int $n) {
    $o = [];
    for ($i = 0; $i < $n; $i++) { $o[$i] = 1.0; $c = function () use (&$i) { $i += 2; }; $c(); }
    $p = [];
    for ($i = 0; $i < $n; $i++) { $p[$i] = 2.0; bump($i); }
    $q = [];
    for ($i = 0; $i < $n; $i++) { $q[$i] = fill([], 2)[1]; }
    return json_encode($o) . " " . json_encode($p) . " " . json_encode($q);
}
echo run(9), "\n";
$r = [];
for ($i = 0; $i < 9; $i++) { $r[$i] = 0.25; viaglobal(); }
echo json_encode($r), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "{\"0\":1,\"3\":1,\"6\":1} {\"0\":2,\"4\":2,\"8\":2} [0.5,0.5,0.5,0.5,0.5,0.5,0.5,0.5,0.5]\n{\"0\":0.25,\"4\":0.25,\"8\":0.25}\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a loop body that later writes a string key into the argument.
#[test]
fn test_float_list_clone_guard_c1() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $x) { $s += $x; } return $s; }
function run(int $n) {
    $u = [1.5, 2.5];
    for ($i = 0; $i < $n; $i++) { echo sum($u), "\n"; $u["k$i"] = 1.5; }
}
run(2);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "4\n5.5\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a reference alias writing a string key into the argument.
#[test]
fn test_float_list_clone_guard_c2() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $x) { $s += $x; } return $s; }
function run() {
    $u = [1.5, 2.5];
    $r = &$u; $r["k"] = 4.0;
    echo sum($u), "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "8\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a by-reference closure capture writing a string key into the argument.
#[test]
fn test_float_list_clone_guard_c3() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $x) { $s += $x; } return $s; }
function run() {
    $u = [1.5, 2.5];
    $f = function() use (&$u) { $u["a"] = 4.0; }; $f();
    echo sum($u), "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "8\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a callee whose returned local a by-reference capture turned into a hash.
#[test]
fn test_float_list_clone_guard_d5() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $k => $x) { $s += $x; echo $k, ","; } return $s; }
function b3($v) { $o = [1.5]; $f = function() use (&$o) { $o["q"] = 2.0; }; $f(); return $o; }
function run(int $n) { $u = [1.5, 2.5]; $r = b3($u); echo sum($r), "\n"; var_dump($r); echo sum(b3($u)), "\n"; }
run(3);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0,q,3.5\narray(2) {\n  [0]=>\n  float(1.5)\n  [\"q\"]=>\n  float(2)\n}\n0,q,3.5\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when an argument rebound in the loop to a call result that later gets a string key.
#[test]
fn test_float_list_clone_guard_d9() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $k => $x) { $s += $x; echo $k, ","; } return $s; }
function b7($v) { $o = []; foreach ($v as $x) { $o[] = $x; } return $o; }
function run(int $n) { $u = [1.5, 2.5]; for ($i = 0; $i < $n; $i++) { echo sum($u), "\n"; $u = b7($u); $u["x$i"] = 1.0; } }
run(3);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0,1,4\n0,1,x0,5\n0,1,2,x1,6\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a chained call whose argument later gets a string key in the loop.
#[test]
fn test_float_list_clone_guard_d10() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function sum($v) { $s = 0.0; foreach ($v as $k => $x) { $s += $x; echo $k, ","; } return $s; }
function b8($v) { return $v; }
function run(int $n) { $u = [1.5, 2.5]; for ($i = 0; $i < $n; $i++) { echo sum(b8($u)), "\n"; $u["x$i"] = 1.0; } }
run(3);
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "0,1,4\n0,1,x0,5\n0,1,x0,x1,6\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a callee with a by-reference capture returning a hash.
#[test]
fn test_float_list_clone_guard_m4() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function b3($v) { $o = [1.5]; $f = function() use (&$o) { $o["q"] = 2.0; }; $f(); return $o; }
function run() { $u = [1.5, 2.5]; var_dump(b3($u)); }
run();
var_dump(b3([1, "a"]));
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "array(2) {\n  [0]=>\n  float(1.5)\n  [\"q\"]=>\n  float(2)\n}\narray(2) {\n  [0]=>\n  float(1.5)\n  [\"q\"]=>\n  float(2)\n}\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A float list stays off the typed float-list clone when a bare `array` return of an int list that the caller extends with sparse and string keys.
#[test]
fn test_float_list_clone_guard_e5() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function c9(array $v): array { return [1, 2, 3]; }
function run() {
    $u = [1.5, 2.5];
    $r = c9($u);
    $r[10] = 5;
    $r["s"] = "x";
    var_dump($r);
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "array(5) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n  [10]=>\n  int(5)\n  [\"s\"]=>\n  string(1) \"x\"\n}\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A `for` counter write keeps packed storage only when every path to the next iteration writes
/// the slot (both arms of an `if`, a `continue` after the write); a `continue` before it, a
/// one-armed branch, a `continue 2` out of a nested loop or `switch`, and a `catch` that
/// continues still produce PHP's sparse hash.
#[test]
fn test_packed_counter_writes_on_every_iteration_path() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function f(int $n) {
  $a = []; for ($i = 0; $i < $n; $i++) { if ($i == 2) continue; $a[$i] = $i; }
  $b = []; for ($i = 0; $i < $n; $i++) { if ($i % 2) { $b[$i] = 1; } }
  $c = []; for ($i = 0; $i < $n; $i++) { $c[$i] = 0; if ($i == 1) continue; $c[$i] = 2; }
  $d = []; for ($i = 0; $i < $n; $i++) { if ($i == 1) { $d[$i] = 5; continue; } elseif ($i == 3) { $d[$i] = 6; } else { $d[$i] = 7; } }
  $e = []; for ($i = 0; $i < $n; $i++) { if ($i == 1) { $e[$i] = 5; } elseif ($i == 3) { $e[$i] = 6; } }
  $g = []; for ($i = 0; $i < $n; $i++) { foreach ([1] as $x) { if ($i == 2) continue 2; } $g[$i] = $x; }
  $h = []; for ($i = 0; $i < $n; $i++) { switch ($i) { case 2: continue 2; default: $h[$i] = 1; } }
  $k = []; for ($i = 0; $i < $n; $i++) { if ($i == 3) break; if ($i == 1) { $k[$i] = 1; } else { $k[$i] = 2; } }
  $m = []; for ($i = 0; $i < $n; $i++) { try { if ($i == 2) throw new Exception(); $m[$i] = 1; } catch (Exception $ex) { continue; } }
  $p = []; for ($i = 0; $i < $n; $i++) { while (true) { if ($i == 2) { continue 2; } break; } $p[$i] = 1; }
  $q = []; for ($i = 0; $i < $n; $i++) { if ($i == 2) { $q[$i + 1] = 9; } else { $q[$i] = 1; } }
  return json_encode([$a, $b, $c, $d, $e, $g, $h, $k, $m, $p, $q]);
}
echo f(5), "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "[{\"0\":0,\"1\":1,\"3\":3,\"4\":4},{\"1\":1,\"3\":1},[2,0,2,2,2],[7,5,7,6,7],{\"1\":5,\"3\":6},{\"0\":1,\"1\":1,\"3\":1,\"4\":1},{\"0\":1,\"1\":1,\"3\":1,\"4\":1},[2,1,2],{\"0\":1,\"1\":1,\"3\":1,\"4\":1},{\"0\":1,\"1\":1,\"3\":1,\"4\":1},{\"0\":1,\"1\":1,\"3\":1,\"4\":1}]\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}


/// A `bool` list written under an unbounded key stays typed and promotes to a hash only when
/// the key really skips a slot.
#[test]
fn test_promotable_bool_list_keeps_slots_through_sparse_writes() {
    let out = compile_and_run(
        r#"<?php
function f(int $n, int $u) { $d = []; for ($i = 0; $i < $n; $i++) { $d[$i] = false; }
  for ($k = 0; $k < 3; $k++) { $d[$u + $k] = true; }
  return $d; }
echo json_encode(f(4, 1)), "\n"; echo json_encode(f(4, 3)), "\n"; echo json_encode(f(2, 7)), "\n"; echo json_encode(f(3, -2)), "\n";
function g(int $n) { $d = []; for ($i = 0; $i < $n; $i++) { $d[$i] = false; }
  $u = $n * 2; for ($k = 0; $k < 2; $k++) { $d[$u] = true; $u++; }
  $c = 0; for ($i = 0; $i < $n * 3; $i++) { if (isset($d[$i])) { $c++; } if (!empty($d[$i])) $c += 10; }
  var_dump(count($d), $c, $d[$n*2] ?? "dflt", $d[1] ?? "dflt");
  foreach ($d as $k => $v) echo $k, "=", var_export($v, true), " "; echo "\n";
  $copy = $d; $copy[100] = true; echo count($d), count($copy), "\n";
  var_dump(array_keys($d), in_array(true, $d, true));
  return $d; }
var_dump(g(3));
function h(int $n) { $seen = []; for ($i = 0; $i < $n; $i++) { $seen[$i] = false; }
  $x = 5; $seen[$x] = true; $seen["k"] = true; var_dump($seen); }
h(2);
"#,
    );
    assert_eq!(out, "[false,true,true,true]\n[false,false,false,true,true,true]\n{\"0\":false,\"1\":false,\"7\":true,\"8\":true,\"9\":true}\n{\"0\":true,\"1\":false,\"2\":false,\"-2\":true,\"-1\":true}\nint(5)\nint(25)\nbool(true)\nbool(false)\n0=false 1=false 2=false 6=true 7=true \n56\narray(5) {\n  [0]=>\n  int(0)\n  [1]=>\n  int(1)\n  [2]=>\n  int(2)\n  [3]=>\n  int(6)\n  [4]=>\n  int(7)\n}\nbool(true)\narray(5) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [2]=>\n  bool(false)\n  [6]=>\n  bool(true)\n  [7]=>\n  bool(true)\n}\narray(4) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [5]=>\n  bool(true)\n  [\"k\"]=>\n  bool(true)\n}\n");
}

/// A warned read of a missing `bool` list element is PHP's `null`, in value position and in
/// truth tests alike (the slot's null sentinel word read as `true` before).
#[test]
fn test_missing_bool_list_element_reads_null() {
    let out = compile_and_run(
        r#"<?php
$i = 7; $b = [true];
if ($b[$i]) echo "T\n"; else echo "F\n";
echo $b[$i] ? "t" : "f", "\n";
var_dump(!$b[$i], $b[$i] == false, $b[$i] === true, $b[$i] ?? "dflt", isset($b[$i]), empty($b[$i]));
function g(array $x, int $j) { return $x[$j]; }
var_dump(g([true, true], 5));
echo "[", $b[$i], "]\n";
"#,
    );
    assert_eq!(out, "F\nf\nbool(true)\nbool(true)\nbool(false)\nstring(4) \"dflt\"\nbool(false)\nbool(true)\nNULL\n[]\n");
}

/// A missed `bool` list read written into another slot stores PHP's `null`, not the sentinel.
#[test]
fn test_bool_list_missing_read_written_back_stays_null() {
    let out = compile_and_run(
        r#"<?php
function f(int $n, int $u) { $d = []; for ($i = 0; $i < $n; $i++) { $d[$i] = false; }
  $d[$u] = $d[$u + 20]; var_dump($d); $d[$u+1] = !$d[$u + 30]; var_dump($d); }
f(2, 2); f(2, 6);
"#,
    );
    assert_eq!(out, "array(3) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [2]=>\n  NULL\n}\narray(4) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [2]=>\n  NULL\n  [3]=>\n  bool(true)\n}\narray(3) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [6]=>\n  NULL\n}\narray(4) {\n  [0]=>\n  bool(false)\n  [1]=>\n  bool(false)\n  [6]=>\n  NULL\n  [7]=>\n  bool(true)\n}\n");
}

/// Visited-set and sieve loops over a promotable `bool` list read missing slots as `null`.
#[test]
fn test_promotable_bool_list_visited_set_walks() {
    let out = compile_and_run(
        r#"<?php
function f(int $n) { $seen = []; for ($i = 0; $i < $n; $i++) { $seen[$i] = false; }
  $x = 1; $cnt = 0;
  while (!$seen[$x]) { $seen[$x] = true; $x = ($x * 3 + 1) % $n; $cnt++; }
  return $cnt . ":" . json_encode($seen); }
echo f(10), "\n", f(7), "\n", f(1), "\n";
function sieve(int $n) { $c = []; for ($i = 0; $i <= $n; $i++) $c[$i] = false; $r = [];
  for ($i = 2; $i <= $n; $i++) { if (!$c[$i]) { $r[] = $i; for ($j = $i * $i; $j <= $n; $j += $i) $c[$j] = true; } } return $r; }
echo implode(",", sieve(60)), "\n";
function sieve2(int $n) { $c = []; $r = [];
  for ($i = 2; $i <= $n; $i++) { if (!isset($c[$i])) { $r[] = $i; for ($j = $i * $i; $j <= $n * 2; $j += $i) $c[$j] = true; } } echo count($c), "\n"; return $r; }
echo implode(",", sieve2(60)), "\n";
function sieve3(int $n) { $c = [false, false]; $r = [];
  for ($i = 2; $i <= $n; $i++) { if (empty($c[$i])) { $r[] = $i; for ($j = $i * $i; $j <= $n * 2; $j += $i) $c[$j] = true; } } echo count($c), json_encode($c), "\n"; return $r; }
echo implode(",", sieve3(30)), "\n";
"#,
    );
    assert_eq!(out, "4:[true,true,false,true,true,false,false,false,false,false]\n6:[true,true,true,false,true,true,true]\n2:[true,true]\n2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59\n89\n2,3,5,7,11,13,17,19,23,29,31,37,41,43,47,53,59\n44{\"0\":false,\"1\":false,\"4\":true,\"6\":true,\"8\":true,\"10\":true,\"12\":true,\"14\":true,\"16\":true,\"18\":true,\"20\":true,\"22\":true,\"24\":true,\"26\":true,\"28\":true,\"30\":true,\"32\":true,\"34\":true,\"36\":true,\"38\":true,\"40\":true,\"42\":true,\"44\":true,\"46\":true,\"48\":true,\"50\":true,\"52\":true,\"54\":true,\"56\":true,\"58\":true,\"60\":true,\"9\":true,\"15\":true,\"21\":true,\"27\":true,\"33\":true,\"39\":true,\"45\":true,\"51\":true,\"57\":true,\"25\":true,\"35\":true,\"55\":true,\"49\":true}\n2,3,5,7,11,13,17,19,23,29\n");
}

/// An append or string-key write after a sparse write of a promotable `bool` list keeps PHP's
/// keys, and copies of the promoted storage stay intact.
#[test]
fn test_promotable_bool_list_append_after_sparse_write() {
    let out = compile_and_run(
        r#"<?php
function f(int $u){ $h=[false]; $h[$u]=true; $h[]=false; var_dump($h); foreach ($h as $k => $v) echo $k, ","; echo "\n"; }
f(2); f(7);
function g(int $u){ $h=[false]; $h[$u]=true; $h["k"]=false; var_dump($h); }
g(2);
function l(int $u,int $n){ $h=[]; for($i=0;$i<$n;$i++){$h[$i]=false;} $h[$u]=true; $h[]=false; foreach($h as $k=>$v){echo $k,",";} $x=$h; $x[0]=true; var_dump(count($x)); echo json_encode($x),"\n"; }
l(50,3); l(1,3);
"#,
    );
    assert_eq!(out, "array(3) {\n  [0]=>\n  bool(false)\n  [2]=>\n  bool(true)\n  [3]=>\n  bool(false)\n}\n0,2,3,\narray(3) {\n  [0]=>\n  bool(false)\n  [7]=>\n  bool(true)\n  [8]=>\n  bool(false)\n}\n0,7,8,\narray(3) {\n  [0]=>\n  bool(false)\n  [2]=>\n  bool(true)\n  [\"k\"]=>\n  bool(false)\n}\n0,1,2,50,51,int(5)\n{\"0\":true,\"1\":false,\"2\":false,\"50\":true,\"51\":false}\n0,1,2,3,int(4)\n[true,true,false,false]\n");
}
