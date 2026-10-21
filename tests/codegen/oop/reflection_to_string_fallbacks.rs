//! Purpose:
//! Regression coverage for resolved Reflection defaults when written AST export is unavailable.
//!
//! Called from:
//! - `codegen_tests` through the `oop` module.
//!
//! Key details:
//! - Supported constant expressions retain their values without executing constructors on dump.
//! - Operator spelling uses the documented evaluated-value fallback, unlike PHP's AST export.

use super::*;

/// Negating the minimum integer through a constant promotes to float without compiler overflow.
#[test]
fn test_reflection_dump_negated_minimum_constant() {
    let out = compile_and_run(r#"<?php
class Values { const Y = PHP_INT_MIN; const X = -self::Y; }
class Foo { public function __construct(public float $x) { echo "ctor\n"; } }
function f($a = Values::X, Foo $b = new Foo(Values::X)) {}
$a = new ReflectionParameter('f', 0);
$b = new ReflectionParameter('f', 1);
echo $a, "\n", $b, "\n";
echo $a->getDefaultValue() === 9223372036854775808.0 ? "float\n" : "wrong\n";
$o = $b->getDefaultValue();
echo $o->x === 9223372036854775808.0 ? "float\n" : "wrong\n";
"#);
    assert_eq!(out, "Parameter #0 [ <optional> $a = Values::X ]\n\
        Parameter #1 [ <optional> Foo $b = new \\Foo(Values::X) ]\nfloat\nctor\nfloat\n");
}

/// Nested array and constructor expressions retain folded defaults and omit unwritten args.
#[test]
fn test_reflection_dump_resolved_compound_default_fallbacks() {
    // PHP retains LIMIT - 1 in these dumps; this subset intentionally exports the resolved value.
    let out = compile_and_run_capture(r#"<?php
const LIMIT = 7;
class Foo {
    public function __construct(public int $x, public int $y = 9) { echo "ctor\n"; }
}


function f($a = [LIMIT - 1, [LIMIT - 2, 1 + 2]], Foo $b = new Foo(LIMIT - 1)) {}
$a = new ReflectionParameter('f', 0);
$b = new ReflectionParameter('f', 1);
echo $a, "\n", $b, "\n";
echo $a->isDefaultValueAvailable() ? "yes\n" : "no\n";
echo $b->isDefaultValueAvailable() ? "yes\n" : "no\n";
$v = $a->getDefaultValue();
echo $v[0], ':', $v[1][0], ':', $v[1][1], "\n";
$o = $b->getDefaultValue();
echo $o->x, ':', $o->y, "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "Parameter #0 [ <optional> $a = [6, [5, 3]] ]\n\
        Parameter #1 [ <optional> Foo $b = new \\Foo(6) ]\nyes\nyes\n6:5:3\nctor\n6:9\n");
}

/// Named constructor slots normalize independently of written argument order and dump spelling.
#[test]
fn test_reflection_dump_named_compound_default_fallbacks() {
    // The documented fallback evaluates LIMIT arithmetic while retaining each written name.
    let out = compile_and_run_capture(r#"<?php
const LIMIT = 7;
class Foo {
    public function __construct(public int $x = 4, public int $y = 9) { echo "ctor\n"; }
}
function f(Foo $a = new Foo(y: LIMIT - 1)) {}
function g(Foo $b = new Foo(y: LIMIT - 1, x: LIMIT - 2)) {}
function h(Foo $c = new Foo(LIMIT - 2, y: LIMIT - 1)) {}
function i(Foo $d = new Foo(x: LIMIT - 1)) {}
$a = new ReflectionParameter('f', 0);
$b = new ReflectionParameter('g', 0);
$c = new ReflectionParameter('h', 0);
$d = new ReflectionParameter('i', 0);
echo $a, "\n", $b, "\n", $c, "\n", $d, "\n";
echo $a->isDefaultValueAvailable(), $b->isDefaultValueAvailable(),
    $c->isDefaultValueAvailable(), $d->isDefaultValueAvailable(), "\n";
$first = $a->getDefaultValue(); echo $first->x, ':', $first->y, "\n";
$second = $b->getDefaultValue(); echo $second->x, ':', $second->y, "\n";
$third = $c->getDefaultValue(); echo $third->x, ':', $third->y, "\n";
$fourth = $d->getDefaultValue(); echo $fourth->x, ':', $fourth->y, "\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "Parameter #0 [ <optional> Foo $a = new \\Foo(y: 6) ]\n\
        Parameter #0 [ <optional> Foo $b = new \\Foo(y: 6, x: 5) ]\n\
        Parameter #0 [ <optional> Foo $c = new \\Foo(5, y: 6) ]\n\
        Parameter #0 [ <optional> Foo $d = new \\Foo(x: 6) ]\n\
        1111\nctor\n4:6\nctor\n5:6\nctor\n5:6\nctor\n6:9\n");
}
