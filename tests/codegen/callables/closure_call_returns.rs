//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of the type a closure whose body
//! is a single `return <call>;` reports, which lowering infers for itself and which must agree with
//! the callee's declared return type.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout.
//! - Expected output is verbatim `LC_ALL=C php` 8.5.10.

use crate::support::*;

/// Issue #1028's own repro: the callee declares `: string` and the closure reported `int`, so the
/// string was read back through an int slot and printed `0`.
#[test]
fn test_closure_returning_a_user_call_keeps_the_callee_type() {
    let out = compile_and_run(
        r#"<?php
function tag(int $n): string { return "t" . $n; }
$a = function ($v) { return tag($v); };
echo $a(1), "|";
"#,
    );
    assert_eq!(out, "t1|");
}

/// The same body in every spelling the issue lists as broken: an arrow function, a callee with no
/// return hint, no closure parameters at all, a typed closure parameter, and an immediately invoked
/// closure literal.
#[test]
fn test_every_closure_spelling_returning_a_user_call() {
    let out = compile_and_run(
        r#"<?php
function tag(int $n): string { return "t" . $n; }
function untyped($n) { return "u" . $n; }
$arrow = fn ($v) => tag($v);
$hintless = function ($v) { return untyped($v); };
$nullary = function () { return tag(3); };
$typed = function (int $v) { return tag($v); };
echo $arrow(1), $hintless(2), $nullary(), $typed(4), (function ($v) { return tag($v); })(5);
"#,
    );
    assert_eq!(out, "t1u2t3t4t5");
}

/// A callee declared AFTER the closure that returns it — the signature table is complete before
/// lowering runs, so declaration order must not matter.
#[test]
fn test_closure_returning_a_later_declared_function() {
    let out = compile_and_run(
        r#"<?php
$a = function ($v) { return later($v); };
echo $a(1), "|";
function later(int $n): string { return "l" . $n; }
"#,
    );
    assert_eq!(out, "l1|");
}

/// A `float` callee was the other silent wrong answer: `int(1)` where PHP prints `float(1.5)`.
#[test]
fn test_closure_returning_a_float_call_keeps_the_float() {
    let out = compile_and_run(
        r#"<?php
function half(int $n): float { return $n / 2; }
$a = function ($v) { return half($v); };
var_dump($a(3));
"#,
    );
    assert_eq!(out, "float(1.5)\n");
}

/// An `array` callee did not silently misprint — it refused to compile, `count for PHP type Int`.
#[test]
fn test_closure_returning_an_array_call_keeps_the_array() {
    let out = compile_and_run(
        r#"<?php
function mk(int $n): array { return [$n, $n + 1]; }
$a = function ($v) { return mk($v); };
$r = $a(1);
echo count($r), ":", $r[0], $r[1];
"#,
    );
    assert_eq!(out, "2:12");
}

/// `strlen()` on the closure's result panicked the compiler rather than answering — the checked
/// builtin was handed an `Int` operand it cannot lower.
#[test]
fn test_strlen_of_a_closure_returning_a_user_call() {
    let out = compile_and_run(
        r#"<?php
function tag(int $n): string { return "t" . $n; }
$a = function ($v) { return tag($v); };
echo strlen($a(1)), "|";
"#,
    );
    assert_eq!(out, "2|");
}

/// A callee with an UNTYPED parameter and no declared return receives that parameter under the
/// boxed-Mixed ABI, so a container it returns carries boxed elements whatever the checker
/// specialized. Its signature can still say `array<int>`.
///
/// Taking that raw stamped the closure `array<int>` over boxed storage, and the return boundary
/// cannot notice — `IrType::from_php` maps every `Array(_)` to one heap kind, and
/// `coerce_container_to_return_type` only ever widens toward `Mixed`. `echo $g(5)[0]` printed a
/// cell pointer. Going through `eir_user_function_return_type`, the helper a DIRECT call uses, is
/// what makes "the closure agrees with its body" true rather than intended.
#[test]
fn test_closure_returning_an_untyped_callees_container() {
    let out = compile_and_run(
        r#"<?php
function f($x) { return [$x]; }
function pick($v) { return [1, 2]; }
function keyed($i) { return ["k" => $i]; }
$g = function (int $v) { return f($v); };
$h = function ($x) { return pick($x); };
$k = function (int $i) { return keyed($i); };
$r = $g(5);
$p = $h(0);
$q = $k(7);
echo $r[0], "|", $p[0], $p[1], "|", $q["k"], "|", count($r), count($p), count($q);
"#,
    );
    assert_eq!(out, "5|12|7|121");
}

/// The three controls the issue lists as already correct, pinned so the new arm cannot take them:
/// a declared closure return skips the inference, a local gives lowering a typed binding to read,
/// and a body with no call never reaches the fallback.
#[test]
fn test_closure_return_shapes_that_already_worked() {
    let out = compile_and_run(
        r#"<?php
function tag(int $n): string { return "t" . $n; }
$declared = function ($v): string { return tag($v); };
$local = function ($v) { $s = tag($v); return $s; };
$nocall = function ($v) { return "d" . $v; };
$intcall = function ($v) { return strlen(tag($v)); };
echo $declared(1), $local(2), $nocall(3), $intcall(4);
"#,
    );
    assert_eq!(out, "t1t2d32");
}

/// An `int` callee is the one the old fallback answered correctly by accident. Pinned so a future
/// change to the arm cannot start disagreeing with it.
#[test]
fn test_closure_returning_an_int_call_is_unchanged() {
    let out = compile_and_run(
        r#"<?php
function twice(int $n): int { return $n * 2; }
$a = function ($v) { return twice($v); };
echo $a(3), "|";
"#,
    );
    assert_eq!(out, "6|");
}

/// A BUILTIN callee keeps answering from the map that already served it, ahead of the new
/// user-function lookup — the builtin arm must stay first.
///
/// `array_keys()` is deliberately absent: a closure returning it still refuses with
/// `array_keys associative key PHP type Mixed into result PHP type Int`, because the builtin arm
/// reads `builtin_call_types` by span and finds nothing for that call. That is the BUILTIN half of
/// the same defect, untouched here — this change adds the user/extern arm only — and reported on
/// the issue.
#[test]
fn test_closure_returning_a_builtin_call_is_unchanged() {
    let out = compile_and_run(
        r#"<?php
$upper = function (string $v) { return strtoupper($v); };
$count = function (array $v) { return count($v); };
$len = function (string $v) { return strlen($v); };
echo $upper("ab"), $count([1, 2, 3]), $len("abcd");
"#,
    );
    assert_eq!(out, "AB34");
}

/// A user function whose name shadows nothing but is reached through a closure returning it from
/// inside another function's body, so the closure is lowered as a nested one.
#[test]
fn test_nested_closure_returning_a_user_call() {
    let out = compile_and_run(
        r#"<?php
function tag(int $n): string { return "t" . $n; }
function wrap(): string {
    $a = function ($v) { return tag($v); };
    return $a(7);
}
echo wrap(), "|";
"#,
    );
    assert_eq!(out, "t7|");
}
