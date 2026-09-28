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
