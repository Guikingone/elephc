//! Purpose:
//! Regression tests for the shared gradual-`int` builtin argument boundary
//! (`accepts_gradual_int` in the checker, `resolve_gradual_int_arg_to_result` in codegen) and
//! for `count()` over an `iterable` value. Both used to be compile errors on real Symfony code:
//! `exit(128 + $this->signalToKill)` and `touch($tmp, $expiresAt ?: time() + …)` were rejected
//! because elephc infers `$a + $b` as `int|float`, and `\count($messages)` was rejected for an
//! `iterable` parameter.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Each expectation was taken from `php -n` on the same source, so these lock PHP parity, not
//!   just "it compiles".
//! - The `count()` object arm is exercised in BOTH directions: a `Countable` implementor is
//!   counted through runtime interface dispatch, and a plain `Generator` raises PHP's catchable
//!   `TypeError` instead of being silently counted or silently zero.

use crate::support::*;

/// `exit()` accepts the boxed `int|float` that elephc infers for integer arithmetic.
///
/// The shared runner exposes success/failure rather than the raw status, so the status itself is
/// observed indirectly: a non-zero `exit(128 + 9)` must report failure and must stop execution.
/// The exact `rc=137` byte-for-byte match against `php -n` is covered by the campaign repro.
#[test]
fn test_exit_accepts_int_float_arithmetic_status() {
    let out = compile_and_run_capture(
        r#"<?php
class Killer {
    private int $signal = 0;
    public function arm(int $s): void { $this->signal = $s; }
    public function kill(): void { echo "before\n"; exit(128 + $this->signal); }
}
$k = new Killer();
$k->arm(9);
$k->kill();
echo "never\n";
"#,
    );
    assert_eq!(out.stdout, "before\n");
    assert!(
        !out.success,
        "exit(128 + 9) must terminate with a non-zero status, stderr={:?}",
        out.stderr
    );
}

/// A zero status through the same gradual-`int` path still exits successfully, so the boxed
/// unbox is a real value read and not a constant failure.
#[test]
fn test_exit_zero_through_gradual_int_succeeds() {
    let out = compile_and_run_capture(
        r#"<?php
$base = 0;
echo "before\n";
exit($base + 0);
"#,
    );
    assert_eq!(out.stdout, "before\n");
    assert!(out.success, "exit(0 + 0) must succeed, stderr={:?}", out.stderr);
}

/// `touch()` accepts the `int|float` timestamp produced by `$expiresAt ?: time() + N`, the
/// exact shape used by symfony/cache's `FilesystemCommonTrait::write()`.
#[test]
fn test_touch_accepts_gradual_int_timestamp() {
    let out = compile_and_run(
        r#"<?php
$p = tempnam(sys_get_temp_dir(), 'elephc_touch_reg');
$expiresAt = 1700000000;
touch($p, $expiresAt ?: time() + 31556952);
echo filemtime($p), "\n";
touch($p, $expiresAt + 5, $expiresAt + 9);
echo filemtime($p), " ", fileatime($p), "\n";
unlink($p);
"#,
    );
    assert_eq!(out, "1700000000\n1700000005 1700000009\n");
}

/// `count()` on an `iterable` parameter counts an indexed array and an associative array by
/// reading the runtime container header.
#[test]
fn test_count_iterable_parameter_counts_arrays() {
    let out = compile_and_run(
        r#"<?php
function cnt(iterable $m): int { return \count($m); }
echo cnt([1, 2, 3]), "\n";
echo cnt(['a' => 1, 'b' => 2]), "\n";
echo cnt([]), "\n";
"#,
    );
    assert_eq!(out, "3\n2\n0\n");
}

/// A `Countable` object reaching `count()` through an `iterable` slot is counted by dispatching
/// `Countable::count()` at runtime, never by traversing it.
#[test]
fn test_count_iterable_dispatches_countable_object() {
    let out = compile_and_run(
        r#"<?php
class Bag implements \IteratorAggregate, \Countable {
    private array $items;
    public function __construct(array $items) { $this->items = $items; }
    public function getIterator(): \Iterator { return new \ArrayIterator($this->items); }
    public function count(): int { return \count($this->items); }
}
function cnt(iterable $m): int { return \count($m); }
echo cnt(new Bag([1, 2, 3, 4])), "\n";
echo cnt([9]), "\n";
"#,
    );
    assert_eq!(out, "4\n1\n");
}

/// A non-`Countable` `Traversable` (a generator) must raise PHP's catchable `TypeError`, the
/// negative control that keeps the relaxed checker honest.
///
/// PHP names the concrete class in the message ("Generator given"); elephc says "Traversable"
/// because the backend has no runtime-message `TypeError` emitter. The class thrown, its
/// catchability, and the control flow are identical, so the assertion checks the stable prefix.
#[test]
fn test_count_iterable_rejects_non_countable_traversable() {
    let out = compile_and_run(
        r#"<?php
function gen(): iterable { yield 1; yield 2; }
function cnt(iterable $m): int { return \count($m); }
try {
    echo cnt(gen()), "\n";
} catch (\TypeError $e) {
    echo "caught: ", $e->getMessage(), "\n";
}
echo cnt([1, 2, 3]), "\n";
"#,
    );
    assert_eq!(
        out,
        "caught: count(): Argument #1 ($value) must be of type Countable|array, Traversable given\n3\n"
    );
}

/// `array_search()`'s `int|false` result is a usable array key: PHP casts `false` to `0`, so
/// the not-found case unsets index 0. Locks the `False` arm of `php_type_is_array_key_coercible`.
#[test]
fn test_array_search_false_result_is_a_valid_array_key() {
    let out = compile_and_run(
        r#"<?php
$c = ['aa', 'bb', 'cc'];
unset($c[array_search('bb', $c)]);
foreach ($c as $k => $v) { echo $k, '=', $v, "\n"; }
echo "---\n";
unset($c[array_search('zz', $c)]);
foreach ($c as $k => $v) { echo $k, '=', $v, "\n"; }
"#,
    );
    assert_eq!(out, "0=aa\n2=cc\n---\n2=cc\n");
}
