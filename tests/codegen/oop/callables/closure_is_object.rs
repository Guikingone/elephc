//! Purpose:
//! Integration and regression tests for `is_object()` and `is_callable()` answering a CLOSURE the
//! way PHP does — including a closure that reached the predicate inside a boxed `Mixed` cell,
//! which is how one arrives from an untyped/`mixed` return.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A `Closure` IS an object in PHP: `is_object(fn () => 1)` is `true`, and `Closure` is a real
//!   class. Both predicates used to answer `false` for the boxed form, because the Mixed tag a
//!   closure descriptor carries (10) was not in either dispatcher's accepted set — only the plain
//!   object tag (6) was.
//! - Tag 10 holds a closure or a first-class-callable descriptor and nothing else. A function-name
//!   callable is tagged as a string and the `[obj, 'method']` forms as arrays, so admitting it lets
//!   no non-object and no non-callable through.
//! - This is the pair Symfony's `PhpFileLoader::load()` turns on:
//!   `\is_object($result) && \is_callable($result)` decides whether the closure a config file
//!   returned is applied at all. Both answering `false` skipped every configurator silently.

use super::*;

/// The headline pair, on a closure that reached the predicates through a boxed `Mixed` cell.
#[test]
fn test_is_object_and_is_callable_accept_a_boxed_closure() {
    let out = compile_and_run(
        r#"<?php
function mk(string $p): mixed {
    if ($p === "c") { return function (string $x): string { return "cfg:" . $x; }; }
    return ["a"];
}
$m = mk("c");
echo (is_callable($m) ? "IC1" : "IC0"), "\n";
echo (is_object($m) ? "IO1" : "IO0"), "\n";
"#,
    );
    assert_eq!(out, "IC1\nIO1\n");
}

/// The `&&` chain Symfony uses, with the non-closure arms kept so the guard is a real decision
/// rather than a constant.
#[test]
fn test_is_object_and_is_callable_chain_selects_the_closure_branch() {
    let out = compile_and_run(
        r#"<?php
function load(string $p): mixed {
    if ($p === "closure") { return function (): string { return "cfg"; }; }
    if ($p === "array") { return ["imports" => []]; }
    return 42;
}
foreach (["closure", "array", "int"] as $p) {
    $r = load($p);
    if (is_object($r) && is_callable($r)) {
        echo "callable\n";
    } elseif (is_array($r)) {
        echo "array\n";
    } else {
        echo "scalar\n";
    }
}
"#,
    );
    assert_eq!(out, "callable\narray\nscalar\n");
}

/// A directly-typed closure, and the negative controls: admitting tag 10 must not make a string,
/// int, or array answer `is_object()` true, and a plain object must stay true.
#[test]
fn test_is_object_negative_controls_are_unchanged() {
    let out = compile_and_run(
        r#"<?php
class Plain { public int $n = 1; }
echo (is_object(function () { return 1; }) ? "1" : "0");
echo (is_object("str") ? "1" : "0");
echo (is_object(42) ? "1" : "0");
echo (is_object([1, 2]) ? "1" : "0");
echo (is_object(new Plain()) ? "1" : "0");
"#,
    );
    assert_eq!(out, "10001");
}

/// `is_callable()` must keep refusing a boxed value that is not callable — the tag-10 arm is an
/// addition to the dispatcher, not a relaxation of it.
#[test]
fn test_is_callable_still_refuses_a_boxed_non_callable() {
    let out = compile_and_run(
        r#"<?php
function mk(int $which): mixed {
    if ($which === 0) { return 42; }
    if ($which === 1) { return "no_such_function_anywhere"; }
    return ["not", "a", "callable", "pair"];
}
echo (is_callable(mk(0)) ? "1" : "0");
echo (is_callable(mk(1)) ? "1" : "0");
echo (is_callable(mk(2)) ? "1" : "0");
"#,
    );
    assert_eq!(out, "000");
}
