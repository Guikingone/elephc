//! Purpose:
//! Regression tests for PHP's generator auto-key counter: how explicit `yield`
//! keys interact with the implicit numbering used by keyless yields, and why
//! `yield from` is exempt from that bookkeeping.
//!
//! Called from:
//!  - `cargo test` via the integration test harness; aggregated under
//!    `tests::codegen::generators` in `tests/codegen/generators/mod.rs`.
//!
//! Key details:
//!  - PHP models the counter as `largest_used_integer_key` (initially -1): an
//!    explicit *integer* key greater than the current largest becomes the new
//!    largest, and every keyless `yield` emits `++largest`. Non-integer keys
//!    (string, float, bool, null) and integer keys at or below the largest
//!    leave it untouched, and the increment wraps at `PHP_INT_MAX`.
//!  - `yield from` forwards the delegate's keys verbatim: it neither renumbers
//!    them nor advances the outer generator's counter, so duplicate keys are
//!    expected output rather than a bug.
//!  - Every expected string in this module is real `LC_ALL=C php` 8.4 output.

use crate::support::*;

/// Verifies that an explicit integer key pushes the auto-key counter so the
/// following keyless yields continue the numbering instead of restarting at 0.
#[test]
fn test_generator_explicit_int_key_continues_auto_numbering() {
    let out = compile_and_run(
        r#"<?php
function keys() { yield 5 => "five"; yield "six"; yield "seven"; }
foreach (keys() as $k => $v) { echo "[$k:$v]"; }
"#,
    );
    assert_eq!(out, "[5:five][6:six][7:seven]");
}

/// Verifies that only keys greater than the largest integer key seen so far
/// move the counter: a lower explicit key is emitted as-is but never rewinds
/// the implicit numbering.
#[test]
fn test_generator_lower_explicit_key_does_not_rewind_counter() {
    let out = compile_and_run(
        r#"<?php
function gen() { yield 10 => "a"; yield 2 => "b"; yield "c"; yield 40 => "d"; yield "e"; }
foreach (gen() as $k => $v) { echo "[$k:$v]"; }
"#,
    );
    assert_eq!(out, "[10:a][2:b][11:c][40:d][41:e]");
}

/// Verifies that non-integer explicit keys are yielded unconverted (generators
/// do not apply array key coercion) and leave the auto-key counter alone.
#[test]
fn test_generator_non_integer_keys_leave_counter_untouched() {
    let out = compile_and_run(
        r#"<?php
function gen() { yield "s" => 1; yield 2; yield 3.5 => 3; yield 4; yield true => 5; yield 6; yield null => 7; yield 8; }
foreach (gen() as $k => $v) { echo "["; var_export($k); echo ":$v]"; }
"#,
    );
    assert_eq!(out, "['s':1][0:2][3.5:3][1:4][true:5][2:6][NULL:7][3:8]");
}

/// Verifies that negative explicit keys never move the counter, so the next
/// keyless yield still starts at 0 (PHP's largest-used key starts at -1).
#[test]
fn test_generator_negative_explicit_keys_keep_counter_at_zero() {
    let out = compile_and_run(
        r#"<?php
function gen() { yield -5 => "a"; yield "b"; yield -1 => "c"; yield "d"; }
foreach (gen() as $k => $v) { echo "[$k:$v]"; }
"#,
    );
    assert_eq!(out, "[-5:a][0:b][-1:c][1:d]");
}

/// Verifies that the counter wraps like PHP's signed 64-bit increment: a
/// `PHP_INT_MAX` key is followed by `PHP_INT_MIN`, not a float or an error.
#[test]
fn test_generator_auto_key_wraps_past_int_max() {
    let out = compile_and_run(
        r#"<?php
function gen() { yield PHP_INT_MAX => "a"; yield "b"; yield "c"; }
foreach (gen() as $k => $v) { echo "[$k:$v]"; }
"#,
    );
    assert_eq!(
        out,
        "[9223372036854775807:a][-9223372036854775808:b][-9223372036854775807:c]"
    );
}

/// Verifies that `yield from` forwards delegate keys verbatim — an inner
/// generator's own numbering and an inner array's indices both pass through
/// without renumbering and without advancing the outer counter, so the outer
/// generator's keys collide with the delegated ones exactly as in PHP.
#[test]
fn test_generator_yield_from_does_not_touch_outer_counter() {
    let out = compile_and_run(
        r#"<?php
function inner() { yield 100 => "i1"; yield "i2"; }
function outer() { yield 3 => "o1"; yield from inner(); yield "o2"; yield from [7, 8]; yield "o3"; }
foreach (outer() as $k => $v) { echo "[$k:$v]"; }
"#,
    );
    assert_eq!(out, "[3:o1][100:i1][101:i2][4:o2][0:7][1:8][5:o3]");
}

/// Verifies that the counter survives `send()`/`next()` resumptions and that a
/// generator with explicit keys still returns its `return` value, i.e. the
/// bookkeeping added to the suspend primitive does not disturb the resume path.
#[test]
fn test_generator_auto_key_survives_send_and_get_return() {
    let out = compile_and_run(
        r#"<?php
function gen() { $x = yield 20 => "p"; echo "<$x>"; yield "q"; yield 5 => "r"; yield "s"; return "done"; }
$g = gen();
echo $g->key(), ";";
$g->send("A");
echo $g->key(), ";";
$g->next();
echo $g->key(), ";";
$g->next();
echo $g->key(), ";";
$g->next();
echo $g->getReturn();
"#,
    );
    assert_eq!(out, "20;<A>21;5;22;done");
}

/// Verifies the runtime-typed key path: keys read out of an array (boxed Mixed
/// cells whose tag is only known at run time) update the counter when they hold
/// integers and are ignored when they hold strings.
#[test]
fn test_generator_runtime_typed_keys_update_counter() {
    let out = compile_and_run(
        r#"<?php
function gen($n) { $ks = [3, "s", 1, 8]; foreach ($ks as $k) { yield $k => "v"; } yield "tail"; yield $n => "z"; yield "last"; }
foreach (gen(2) as $k => $v) { echo "["; var_export($k); echo ":$v]"; }
"#,
    );
    assert_eq!(out, "[3:v]['s':v][1:v][8:v][9:tail][2:z][10:last]");
}
