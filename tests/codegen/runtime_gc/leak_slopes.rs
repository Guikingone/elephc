//! Purpose:
//! Ownership regressions found while bringing the `preg_*` array forms up, each measured as a
//! heap SLOPE: the live blocks after one iteration must equal those after forty.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A fixture's baseline is not always zero -- a class instance or a cached value can outlive
//!   the loop -- so each asserts that what stays live does not GROW with the iteration count.
//! - Expected stdout is `php -n` 8.5's own output at both iteration counts.

use crate::support::{
    compile_and_run_with_heap_debug, compile_and_run_with_heap_debug_and_regex, ProgramOutput,
};

/// Live heap blocks the heap-debug summary reports at exit; `clean` is zero.
fn live_blocks(out: &ProgramOutput) -> usize {
    if out.stderr.contains("HEAP DEBUG: leak summary: clean") {
        return 0;
    }
    let marker = "HEAP DEBUG: leak summary: live_blocks=";
    let start = out
        .stderr
        .find(marker)
        .unwrap_or_else(|| panic!("no leak summary in: {}", out.stderr))
        + marker.len();
    out.stderr[start..]
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .and_then(|digits| digits.parse().ok())
        .unwrap_or_else(|| panic!("unreadable leak summary in: {}", out.stderr))
}

/// Runs `source` with `__N__` at 1 and at 40 and asserts the extra iterations left nothing live.
fn assert_no_leak_slope(source: &str, once_expected: &str, many_expected: &str, regex: bool) {
    let run = |n: &str| {
        let source = source.replace("__N__", n);
        if regex {
            compile_and_run_with_heap_debug_and_regex(&source)
        } else {
            compile_and_run_with_heap_debug(&source)
        }
    };
    let once = run("1");
    let many = run("40");
    assert_eq!(once.stdout, once_expected, "stderr: {}", once.stderr);
    assert_eq!(many.stdout, many_expected, "stderr: {}", many.stderr);
    assert_eq!(
        live_blocks(&once),
        live_blocks(&many),
        "live blocks grew with the iteration count\nonce: {}\nforty times: {}",
        once.stderr,
        many.stderr
    );
}

/// `return (string) $x;` over a `mixed` or untyped parameter. The alias summary sees the cast as
/// returning `$x`, but a boxed value becomes a string only as a COPY, so the caller kept the box it
/// made at the call boundary -- and, for a variable argument, the result -- alive forever.
#[test]
fn test_string_cast_of_a_mixed_parameter_does_not_leak() {
    assert_no_leak_slope(
        r#"<?php
function asMixed(mixed $x) { return (string) $x; }
function asTyped(mixed $x): string { return (string) $x; }
function untyped($x) { return (string) $x; }
$s = str_repeat('s', 100);
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += strlen(asMixed('abc')) + strlen(asTyped('abc')) + strlen(asMixed(12));
    $total += strlen(untyped('abc')) + strlen(untyped($s));
}
echo $total, "\n";
"#,
        r#"111
"#,
        r#"4440
"#,
        false,
    );
}

/// `$out = []; $out['a'] = 1;` promotes the empty list to a hash, and the promotion must CONSUME
/// the slot's reference; a slot load was not provably owned, so the old array leaked every call.
#[test]
fn test_first_string_key_write_does_not_leak_the_empty_array() {
    assert_no_leak_slope(
        r#"<?php
function build() { $out = []; $out['a'] = 1; $out['b'] = 'x'; return $out; }
function buildString() { $out = []; $out['a'] = 'x'; return $out; }
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += count(build()) + count(buildString());
}
echo $total, "\n";
"#,
        r#"3
"#,
        r#"120
"#,
        false,
    );
}

/// A local holding a string before a loop that stores something else into it is boxed at loop
/// entry; the entry conversion loaded the string as an unboxing COPY and boxed it again, and the
/// first copy leaked every call.
#[test]
fn test_loop_widened_string_local_does_not_leak() {
    assert_no_leak_slope(
        r#"<?php
function mk(int $i) { return $i % 2 ? null : 'z'; }
function wg(string $s, array $ps) { $r = $s; foreach ($ps as $p) { $r = $p; } return 1; }
function wc(array $ps) { $r = 'lit'; foreach ($ps as $p) { $r = mk($p); } return 1; }
function wd(string $s, array $ps) { $r = $s; foreach ($ps as $p) { $r = mk($p); } return $r; }
$s = str_repeat('s', 100);
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += wg($s, [1, 2]) + wc([1, 2]) + strlen((string) wd($s, [1, 2]));
}
echo $total, "\n";
"#,
        r#"3
"#,
        r#"120
"#,
        false,
    );
}

/// `preg_match($p, $s, $m)` into a fresh local of a function boxes the matches array into a
/// `mixed` slot, and boxing RETAINS an array: the call's own reference leaked the whole matches
/// array on every call -- ten blocks for `preg_match_all()`.
#[test]
fn test_matches_into_a_fresh_function_local_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
function all(string $pattern, string $subject) { preg_match_all($pattern, $subject, $plain); return count($plain); }
function one(string $pattern, string $subject) { preg_match($pattern, $subject, $m); return count($m); }
function flagged(string $subject) { preg_match_all('/(a)(x)?/', $subject, $sets, PREG_SET_ORDER | PREG_OFFSET_CAPTURE); return count($sets); }
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += all('/(a)(x)?/', 'za') + one('/(a)(x)?/', 'za') + flagged('zaza');
}
echo $total, "\n";
"#,
        r#"7
"#,
        r#"280
"#,
        true,
    );
}

/// `preg_replace()` returned its subject ITSELF when the pattern failed, so its result was declared
/// to alias its arguments and none of them was ever released -- a `(string)` pattern copy leaked
/// on every call, failing or not.
#[test]
fn test_replacement_with_a_variable_pattern_releases_its_arguments() {
    assert_no_leak_slope(
        r#"<?php
function viaMixed(mixed $p) { return preg_replace((string) $p, 'x', 'bcd'); }
function viaTyped(string $p, string $s) { $r = preg_replace($p, 'x', $s); return $r; }
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += strlen(viaMixed('/a/')) + strlen(viaTyped('/b/', 'abc')) + (int) is_null(@viaTyped('/(/', 'abc'));
}
echo $total, "\n";
"#,
        r#"7
"#,
        r#"280
"#,
        true,
    );
}

/// `$out[$key] = (string) $value` stores a string by copying it, locally or into a property hash,
/// so the cast's own copy is the write's to drop; neither path dropped it.
#[test]
fn test_cast_values_written_under_mixed_keys_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class Holder { public array $a = []; public function put($k, $v) { $this->a[$k] = (string) $v; } }
function copyAll(mixed $subject) { $out = []; foreach ($subject as $key => $value) { $out[$key] = (string) $value; } return $out; }
$h = new Holder;
$subjects = ['a' => 'apple', 'b' => 'berry'];
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += count(copyAll($subjects));
    $h->put($i % 3 ? 'k' : 1, 'v' . $i);
    $total += count($h->a);
}
echo $total, "\n";
"#,
        r#"3
"#,
        r#"159
"#,
        false,
    );
}

/// A string a CALL returns is taken over by the Mixed box it is written into -- only a copied
/// string (the cast above) is the write's to release. Releasing the call's string as well freed
/// what the container now held: `$out[$i] = label($raw, $i)` read back as garbage bytes, and
/// PDO's `FETCH_NAMED` rows came back with corrupted column names. `label` recurses so the
/// inliner leaves it a real call.
#[test]
fn test_call_results_written_under_mixed_keys_stay_owned_by_the_array() {
    assert_no_leak_slope(
        r#"<?php
class Holder { public array $a = []; public function name(int $i): string { return strtoupper(chr(97 + $i) . "x"); } public function put(int $n) { for ($i = 0; $i < $n; $i++) { $this->a[$i] = $this->name($i); } } }
function label(array $raw, int $i): string { if ($i < 0) { return label($raw, 0); } return strtoupper($raw[$i]); }
function names(array $raw) { $out = []; for ($i = 0; $i < count($raw); $i++) { $out[$i] = label($raw, $i); } return $out; }
$h = new Holder;
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $row = names(['ab', 'cd', 'ef']);
    $total += strlen(implode('', $row));
    $h->put(3);
}
echo $total, ' ', implode(',', names(['x', 'y'])), ' ', implode(',', $h->a), "\n";
"#,
        r#"6 X,Y AX,BX,CX
"#,
        r#"240 X,Y AX,BX,CX
"#,
        false,
    );
}

/// References taken inside a loop body are lowered once and run every iteration. Each rebinding
/// must drop the share the previous iteration took (`$r = &$v`, the by-reference `foreach` that
/// shares its elements), and a local already promoted to a cell must not be re-boxed or have its
/// cell's value overwritten without a release (`$a[$i] = &$v`, `[&$v]`). Every one of these leaked
/// one block per extra iteration.
#[test]
fn test_references_taken_in_loops_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
function aliases() { $sum = 0; for ($i = 0; $i < 3; $i++) { $v = $i; $r = &$v; $r = $r + 1; $sum += $v; } return $sum; }
function elements() { $a = [0, 0, 0]; for ($i = 0; $i < 3; $i++) { $v = $i; $a[$i] = &$v; } return array_sum($a); }
function literals() { $pool = []; foreach ([1, 2, 3] as $k) { $v = $k; $pool[] = [&$v]; } return count($pool) + $pool[0][0]; }
function shared() { $s = []; $vars = ['x' => 1, 'y' => 2]; foreach ($vars as $name => &$value) { $s[$name] = &$value; } unset($value); return $vars['x'] + $vars['y'] + count($s); }
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += aliases() + elements() + literals() + shared();
}
echo $total, "\n";
"#,
        r#"23
"#,
        r#"920
"#,
        false,
    );
}

/// The eval bridge's owner lookup reads an array callable's first element and never released the
/// key or the element, so every `is_callable([$object, 'method'])` on a `mixed` value leaked two
/// blocks in a program that links the bridge.
#[test]
fn test_is_callable_on_an_array_callable_does_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class K { public function low($m) { return 1; } public static function st($m) { return 1; } }
function check(mixed $cb) { return is_callable($cb) ? 1 : 0; }
$k = new K;
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += check([$k, 'low']) + check([$k, 'nope']) + check(['K', 'st']) + check([1, 2]);
}
eval('$unused = 1;');
echo $total, "\n";
"#,
        r#"2
"#,
        r#"80
"#,
        false,
    );
}

/// Every array and flag form, run in a loop, keeps its heap flat.
#[test]
fn test_preg_array_forms_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class Lower { public function low($m) { return strtolower($m[0]); } }
function viaPattern($cb, $p) { return preg_replace_callback($p, $cb, 'APPLE pie'); }
$subjects = ['a' => 'apple', 'b' => 'berry', 'c' => 'cherry'];
$k = new Lower;
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += count(preg_replace('/e/', 'E', $subjects));
    $total += strlen(preg_replace(['/a/', '/e/'], ['A', 'E'], 'apple pie'));
    $total += count(preg_replace_callback('/r+/', fn($m) => strtoupper($m[0]), $subjects, -1, $n)) + $n;
    $total += strlen(preg_replace_callback('/(a)(x)?/', fn($m) => json_encode($m), 'za', -1, $c, PREG_OFFSET_CAPTURE));
    $total += strlen(preg_replace_callback('/(a)(x)?/', fn($m) => json_encode($m), 'zaza', -1, $c, PREG_UNMATCHED_AS_NULL));
    $total += count(preg_filter(['/a/', '/b/'], 'x', ['aa', 'cc', 'bb', 'k' => 'ab'], 1, $c)) + $c;
    $total += strlen((string) preg_filter('/a/', 'x', 'bcd'));
    $total += count(preg_replace_callback_array(['/a/' => fn($m) => 'A', '/b/' => [$k, 'low']], ['x' => 'ab', 'y' => 'cc'], 1, $c));
    $total += strlen(viaPattern([$k, 'low'], '/P/')) + strlen(viaPattern(fn($m) => '#', '/e/'));
    $total += count(@preg_replace(['/a/', '/(/'], 'x', ['k' => 'ab', 'j' => 'cd'], -1, $c)) + $c;
}
echo $total, "\n";
"#,
        r#"93
"#,
        r#"3720
"#,
        true,
    );
}

/// An element write into a local whose frame slot is BOXED -- because another branch assigns it a
/// `mixed` value -- borrows an unboxed array with its own reference and stores a new box. The old
/// box was never dropped, so `$a = []; $a[0] = 'x';` leaked two blocks every call.
#[test]
fn test_element_write_into_a_boxed_local_does_not_leak() {
    assert_no_leak_slope(
        r#"<?php
function build(mixed $row, int $flags) { if ($flags & 256) { $a = $row; } else { $a = []; $a[0] = 'x'; $a[1] = 'y'; } return count($a); }
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += build([1], 512) + build([1, 2, 3], 256);
}
echo $total, "\n";
"#,
        r#"5
"#,
        r#"200
"#,
        false,
    );
}

/// Every way of invoking a callable that the compiler builds a temporary for. The invoker only
/// borrows its callback, and each shape dropped something: the `[$class, 'm']` array behind
/// `$class::m()` (and the original of the Mixed copy a typed callable array is converted to), an
/// immediately invoked closure, the static-method descriptor behind `$a()` on a local, the Mixed
/// cell a scalar-typed invocation is cast from, and -- in the eval bridge -- the comparison
/// result and the answer of `spl_autoload_register()`. Measured before: 62 blocks per call.
#[test]
fn test_callable_invocation_shapes_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class K { static function n() { return 4; } function m() { return "y" . mt_rand(1, 1); } static function s() { return "x" . mt_rand(1, 1); } static function load($c) { } }
function shapes() {
    $c = 'K';
    $a = ['K', 's'];
    $l = ['K', 'load'];
    spl_autoload_register($l);
    spl_autoload_register($l);
    spl_autoload_unregister($l);
    return $c::n() . $c::s() . call_user_func([$c, 'n']) . [new K, 'm']() . $a() . call_user_func($a)
        . (function () { return "z" . mt_rand(1, 1); })() . implode(array_map(['K', 's'], [1]));
}
$out = '';
for ($i = 0; $i < __N__; $i++) { $out = shapes(); }
echo $out, "\n";
"#,
        r#"4x14y1x1x1z1x1
"#,
        r#"4x14y1x1x1z1x1
"#,
        false,
    );
}

/// A reflector over a compiled class, and its eval-dispatched getters. Construction boxed the
/// member objects and attribute arrays it had just created with a retaining box, the bridge
/// never released the key and element of each name it read back, and a scalar getter's result
/// cell was cast and dropped. Measured before: 619 blocks per iteration.
#[test]
fn test_reflectors_over_a_compiled_class_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class Base {}
interface Marker {}
final class Shape extends Base implements Marker {
    const A = 1.5;
    public $p = 1;
    public $q;
    function m(int $a = 2, $b = 'x') {}
    static function s() {}
}
function reflect() {
    $class = new ReflectionClass('Shape');
    $object = new ReflectionObject(new Shape());
    $method = new ReflectionMethod('Shape', 'm');
    $property = new ReflectionProperty('Shape', 'p');
    return $class->getName() . $class->getShortName() . ($class->isFinal() ? 'F' : '-')
        . ($class->hasMethod('m') ? 'M' : '-') . $class->getConstant('A')
        . $object->getName() . $method->getNumberOfParameters() . $property->getName();
}
$out = '';
for ($i = 0; $i < __N__; $i++) { $out = reflect(); }
echo $out, "\n";
"#,
        r#"ShapeShapeFM1.5Shape2p
"#,
        r#"ShapeShapeFM1.5Shape2p
"#,
        false,
    );
}

/// Calls from compiled code into eval-declared code. The bridge boxes each argument with a
/// reference of its own, so the compiled temporary (`[1, 2, $i]`) was never dropped; and a
/// METHOD activation was torn down without releasing its scope (a function's was), so every
/// method local leaked, and a by-value array parameter's separated copy kept the caller's array
/// alive. Measured before: 8 blocks per iteration.
#[test]
fn test_calls_into_eval_declared_code_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class EvalPrelude {}
eval('class Z { static function f($a) { return count($a); } static function h() { $t = 5; $u = [1, 2]; return 1; } } function zf($a) { return count($a); }');
$total = 0;
for ($i = 0; $i < __N__; $i++) {
    $total += Z::f([1, 2, $i]) + zf([1, $i]) + Z::h();
}
echo $total, "\n";
"#,
        r#"6
"#,
        r#"240
"#,
        false,
    );
}

/// Temporaries inside eval-declared code, and the literal forms of `Closure::fromCallable()`.
/// The interpreter never released the literal arguments a builtin evaluated for itself, nor a
/// binary or unary operator's temporary operands (and the zero `-1` is computed against); the
/// bridge's `.` dropped both operands' string casts; and a literal `fromCallable()` reached the
/// bridge, which kept its argument. Measured before: 27 blocks per iteration.
#[test]
fn test_eval_expression_temporaries_do_not_leak() {
    assert_no_leak_slope(
        r#"<?php
class EvalPrelude {}
eval('class Z { static function t($x) { $a = strlen("ab") . strtoupper($x); $b = -1; "a" . "y"; return $a . $b; } }');
class K { function m($v) { return "m$v"; } }
$out = '';
for ($i = 0; $i < __N__; $i++) {
    $out = Z::t("q") . Closure::fromCallable([new K, 'm'])(1) . Closure::fromCallable('strtoupper')("z");
}
echo $out, "\n";
"#,
        r#"2Q-1m1Z
"#,
        r#"2Q-1m1Z
"#,
        false,
    );
}
