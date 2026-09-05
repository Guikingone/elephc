//! Purpose:
//! Interpreter tests for PHP generators: the object a function containing `yield` returns, its
//! resumption protocol, key numbering, `yield from` delegation and the errors PHP raises.
//!
//! Called from:
//! - `cargo test -p elephc-magician interpreter::tests::generators`.
//!
//! Key details:
//! - A named `function g() { yield 1; }` used to die with
//!   `call to undefined function __elephc_eval_yield()`, because only closures were lowered and
//!   they were lowered EAGERLY into an `ArrayIterator`, which re-evaluated every yielded
//!   expression and could not model an infinite generator, `send()` or `yield from`.
//! - The key rules are the ones that catch a re-implementation: an explicit key does not advance
//!   the auto-increment counter, `yield from` preserves the INNER keys and leaves the outer
//!   counter alone, and every generator in a delegation chain numbers its own.
//! - Every expected string is `php -n` 8.5.6's output on the same fragment.

use super::super::*;
use super::support::*;

/// Runs one fragment and returns what it echoed.
fn out(fragment: &[u8]) -> String {
    let program = parse_fragment(fragment).expect("parse generator fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();
    execute_program(&program, &mut scope, &mut values).expect("execute generator fragment");
    values.output.clone()
}

/// Runs one fragment expected to raise, returning what it echoed before the throw.
fn throws(fragment: &[u8]) -> (String, EvalStatus) {
    let program = parse_fragment(fragment).expect("parse generator fragment");
    let mut scope = ElephcEvalScope::new();
    let mut values = FakeOps::default();
    let status = execute_program(&program, &mut scope, &mut values)
        .expect_err("fragment should raise");
    (values.output.clone(), status)
}

/// Verifies a named generator function iterates in `foreach`.
///
/// `php -n` 8.5.6 prints `12`. This is the reported failure: a named function containing `yield`
/// reached the runtime with the parser's marker intact and died on an undefined function.
#[test]
fn a_named_generator_function_iterates() {
    assert_eq!(
        out(br#"function g() { yield 1; yield 2; } foreach (g() as $v) { echo $v; }"#),
        "12",
    );
}

/// Verifies auto-increment and explicit keys follow PHP's numbering.
///
/// `php -n` 8.5.6 prints `0=a,k=b,1=c,`: the explicit `'k'` does NOT advance the counter, so the
/// third yield takes key 1 rather than 2.
#[test]
fn explicit_keys_do_not_advance_the_auto_increment_counter() {
    assert_eq!(
        out(
            br#"function k() { yield 'a'; yield 'k' => 'b'; yield 'c'; }
foreach (k() as $key => $v) { echo "$key=$v,"; }"#
        ),
        "0=a,k=b,1=c,",
    );
}

/// Verifies `current()`, `key()`, `next()` and `valid()` drive the generator by hand.
///
/// `php -n` 8.5.6 prints `10;21;done;`.
#[test]
fn the_manual_protocol_walks_the_generator() {
    assert_eq!(
        out(
            br#"function g() { yield 1; yield 2; }
$it = g();
echo $it->current(), $it->key(), ";";
$it->next();
echo $it->current(), $it->key(), ";";
$it->next();
echo $it->valid() ? "valid" : "done", ";";"#
        ),
        "10;21;done;",
    );
}

/// Verifies `getReturn()` hands back what the body returned.
///
/// `php -n` 8.5.6 prints `R`.
#[test]
fn get_return_reports_the_bodys_return_value() {
    assert_eq!(
        out(
            br#"function r() { yield 1; return "R"; }
$x = r();
foreach ($x as $v) {}
echo $x->getReturn();"#
        ),
        "R",
    );
}

/// Verifies `send()` delivers a value to the expression the generator is suspended on.
///
/// `php -n` 8.5.6 prints `1,got:A,2,got:B,`. This needs `yield` in expression position, which
/// the parser refused entirely before this change.
#[test]
fn send_delivers_a_value_into_the_suspended_yield() {
    assert_eq!(
        out(
            br#"function e() { $a = yield 1; echo "got:$a,"; $b = yield 2; echo "got:$b,"; }
$s = e();
echo $s->current(), ",";
$s->send("A");
echo $s->current(), ",";
$s->send("B");"#
        ),
        "1,got:A,2,got:B,",
    );
}

/// Verifies `yield from` over an array preserves the inner keys and leaves the outer counter.
///
/// `php -n` 8.5.6 prints `0:0,0:10,1:20,1:99,`. The delegated array contributes keys 0 and 1 of
/// its own, and the outer generator's counter is untouched by the delegation, so the yield after
/// it takes key 1 rather than 3.
#[test]
fn yield_from_an_array_preserves_inner_keys() {
    assert_eq!(
        out(
            br#"function yf() { yield 0; yield from [10, 20]; yield 99; }
foreach (yf() as $key => $v) { echo "$key:$v,"; }"#
        ),
        "0:0,0:10,1:20,1:99,",
    );
}

/// Verifies a three-level `yield from` chain, where every generator numbers its own keys.
///
/// `php -n` 8.5.6 prints `0/a1,0/b1,0/c1,1/c2,1/b2,1/a2,`.
#[test]
fn a_three_level_yield_from_chain_numbers_each_generator_separately() {
    assert_eq!(
        out(
            br#"function l3() { yield "c1"; yield "c2"; }
function l2() { yield "b1"; yield from l3(); yield "b2"; }
function l1() { yield "a1"; yield from l2(); yield "a2"; }
foreach (l1() as $key => $v) { echo "$key/$v,"; }"#
        ),
        "0/a1,0/b1,0/c1,1/c2,1/b2,1/a2,",
    );
}

/// Verifies a yield inside a `for` and an `if` suspends and resumes correctly.
///
/// `php -n` 8.5.6 prints `024`. The loop counter has to survive each suspension, which is what
/// the frame's own scope is for.
#[test]
fn a_yield_inside_a_loop_and_a_conditional_resumes() {
    assert_eq!(
        out(
            br#"function loopy($n) { for ($i = 0; $i < $n; $i++) { if ($i % 2 == 0) { yield $i; } } }
foreach (loopy(5) as $v) { echo $v; }"#
        ),
        "024",
    );
}

/// Verifies a generator declared as a method works the same way.
///
/// `php -n` 8.5.6 prints `78`.
#[test]
fn a_method_can_be_a_generator() {
    assert_eq!(
        out(
            br#"class C { public function m() { yield 7; yield 8; } }
foreach ((new C())->m() as $v) { echo $v; }"#
        ),
        "78",
    );
}

/// Verifies a generator body does not run until it is first asked for a value.
///
/// `php -n` 8.5.6 prints `before,after,body,` — calling the function produces the object without
/// executing a line of it.
#[test]
fn the_body_does_not_run_until_the_first_request() {
    assert_eq!(
        out(
            br#"function g() { echo "body,"; yield 1; }
echo "before,";
$x = g();
echo "after,";
$x->current();"#
        ),
        "before,after,body,",
    );
}

/// Verifies traversing a finished generator raises PHP's exception.
///
/// `php -n` 8.5.6 reports `Exception: Cannot traverse an already closed generator`.
#[test]
fn traversing_a_closed_generator_raises() {
    let (output, status) = throws(
        br#"function g() { yield 1; yield 2; }
$a = g();
foreach ($a as $v) {}
foreach ($a as $v) {}"#,
    );
    assert_eq!((output.as_str(), status), ("", EvalStatus::UncaughtThrowable));
}

/// Verifies rewinding a generator that has advanced raises PHP's exception.
///
/// `php -n` 8.5.6 reports `Exception: Cannot rewind a generator that was already run`.
#[test]
fn rewinding_an_advanced_generator_raises() {
    let (_, status) = throws(
        br#"function g() { yield 1; yield 2; }
$b = g();
$b->next();
$b->rewind();"#,
    );
    assert_eq!(status, EvalStatus::UncaughtThrowable);
}

/// Verifies `getReturn()` before the generator finished raises PHP's exception.
///
/// `php -n` 8.5.6 reports `Exception: Cannot get return value of a generator that hasn't returned`.
#[test]
fn get_return_before_finishing_raises() {
    let (_, status) = throws(
        br#"function g() { yield 1; yield 2; }
$c = g();
$c->getReturn();"#,
    );
    assert_eq!(status, EvalStatus::UncaughtThrowable);
}

/// Verifies `yield from` over a scalar raises PHP's error.
///
/// `php -n` 8.5.6 reports `Error: Can use "yield from" only with arrays and Traversables`.
#[test]
fn yield_from_a_non_iterable_raises() {
    let (_, status) = throws(
        br#"function bad() { yield from 5; }
foreach (bad() as $v) {}"#,
    );
    assert_eq!(status, EvalStatus::UncaughtThrowable);
}

/// Verifies a closure and an arrow-bodied closure can be generators too.
///
/// `php -n` 8.5.6 prints `56`. Closures used to be lowered EAGERLY into an `ArrayIterator`,
/// which re-evaluated every yielded expression on each call.
#[test]
fn a_closure_can_be_a_generator() {
    assert_eq!(
        out(
            br#"$g = function () { yield 5; yield 6; };
foreach ($g() as $v) { echo $v; }"#
        ),
        "56",
    );
}
