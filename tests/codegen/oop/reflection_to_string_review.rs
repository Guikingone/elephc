//! Purpose:
//! Regression coverage for PHP Reflection dump details and eval-created reflector casts.
//!
//! Called from:
//! - `codegen_tests` through the `oop` module.
//!
//! Key details:
//! - Expected strings were checked with PHP 8.5.11, excluding the source location line.
//! - Direct objects and member listings must share the same formatting.

use super::*;

/// Overrides report the overwritten declaration and the root prototype independently.
#[test]
fn test_reflection_dump_override_origins() {
    let out = compile_and_run(r#"<?php
class A { public function run() {} private function hidden() {} }
class B extends A { public function run() {} public function hidden() {} }
class C extends B { public function run() {} }
class D extends B {}
interface I { public function run(); }
class E extends A implements I { public function run() {} }
echo new ReflectionMethod('B', 'run');
echo new ReflectionMethod('C', 'run');
echo new ReflectionMethod('D', 'run');
echo new ReflectionMethod('B', 'hidden');
echo new ReflectionMethod('E', 'run');
$r = new ReflectionClass('C');
foreach ($r->getMethods() as $m) { if ($m->getName() === 'run') echo $m; }
"#);
    assert_eq!(out, "Method [ <user, overwrites A, prototype A> public method run ] {\n}\n\
        Method [ <user, overwrites B, prototype A> public method run ] {\n}\n\
        Method [ <user, inherits B, prototype A> public method run ] {\n}\n\
        Method [ <user> public method hidden ] {\n}\n\
        Method [ <user, overwrites A, prototype I> public method run ] {\n}\n\
        Method [ <user, overwrites B, prototype A> public method run ] {\n}\n");
}

/// Float defaults round to 14 digits before choosing the decimal or exponential form.
#[test]
fn test_reflection_dump_float_precision() {
    let out = compile_and_run(r#"<?php
function f(float $a = 0.30000000000000004, float $b = 1e14,
    float $c = 1e13, float $d = 99999999999999.9, float $e = -0.0) {}
foreach ((new ReflectionFunction('f'))->getParameters() as $p) echo $p, "\n";
"#);
    assert_eq!(out, "Parameter #0 [ <optional> float $a = 0.3 ]\n\
        Parameter #1 [ <optional> float $b = 1.0E+14 ]\n\
        Parameter #2 [ <optional> float $c = 10000000000000.0 ]\n\
        Parameter #3 [ <optional> float $d = 1.0E+14 ]\n\
        Parameter #4 [ <optional> float $e = -0.0 ]\n");
}

/// Strings escape control bytes, backslashes and UTF-8 bytes without escaping quote marks.
#[test]
fn test_reflection_dump_string_escapes() {
    let out = compile_and_run(r#"<?php
function f(string $s = "\n\r\t\v\f\e\x00\x7F\xFF\\'é", array $a = ["a\nb" => "c\td"]) {}
foreach ((new ReflectionFunction('f'))->getParameters() as $p) echo $p, "\n";
"#);
    assert_eq!(out, "Parameter #0 [ <optional> string $s = '\\n\\r\\t\\v\\f\\e\\x00\\x7F\\xFF\\\\'\\xC3\\xA9' ]\n\
        Parameter #1 [ <optional> array $a = ['a\\nb' => 'c\\td'] ]\n");
}

/// Iterable aliases expand in nullable parameters and return types as PHP unions.
#[test]
fn test_reflection_dump_iterable_alias() {
    let out = compile_and_run(r#"<?php
function f(iterable $a = [], ?iterable $b = null): iterable { return []; }
echo new ReflectionFunction('f');
"#);
    assert_eq!(out, "Function [ <user> function f ] {\n\n  - Parameters [2] {\n\
        \x20   Parameter #0 [ <optional> Traversable|array $a = [] ]\n\
        \x20   Parameter #1 [ <optional> Traversable|array|null $b = NULL ]\n  }\n\
        \x20 - Return [ Traversable|array ]\n}\n");
}

/// Native constant defaults retain both their PHP-visible names and their resolved values.
#[test]
fn test_reflection_dump_native_constant_defaults() {
    let out = compile_and_run(r#"<?php
function f(int $flags = ENT_QUOTES) {}
$p = new ReflectionParameter('f', 'flags');
echo $p, "\n", (int)$p->isDefaultValueConstant(), " ",
    (int)$p->isDefaultValueAvailable(), " ", $p->getDefaultValueConstantName(), " ", $p->getDefaultValue();
"#);
    assert_eq!(out, "Parameter #0 [ <optional> int $flags = ENT_QUOTES ]\n1 1 ENT_QUOTES 3");
}

/// Deprecated attributes and tentative internal return types get distinct PHP labels.
#[test]
fn test_reflection_dump_deprecated_and_tentative_labels() {
    let out = compile_and_run(r#"<?php
#[Deprecated] function oldFunction() {}
class D { #[Deprecated] public function oldMethod() {} }
echo new ReflectionFunction('oldFunction');
echo new ReflectionMethod('D', 'oldMethod');
echo new ReflectionMethod('Iterator', 'current');
echo new ReflectionMethod('Countable', 'count');
echo new ReflectionMethod('ReflectionFunction', 'getName');
"#);
    assert_eq!(out, "Function [ <user, deprecated> function oldFunction ] {\n}\n\
        Method [ <user, deprecated> public method oldMethod ] {\n}\n\
        Method [ <internal:Core> abstract public method current ] {\n\n  - Parameters [0] {\n  }\n  - Tentative return [ mixed ]\n}\n\
        Method [ <internal:Core> abstract public method count ] {\n\n  - Parameters [0] {\n  }\n  - Tentative return [ int ]\n}\n\
        Method [ <internal:Reflection, inherits ReflectionFunctionAbstract> public method getName ] {\n\n  - Parameters [0] {\n  }\n  - Tentative return [ string ]\n}\n");
}

/// Eval-created reflectors keep their dumps for direct casts and casts after eval returns.
#[test]
fn test_reflection_dump_eval_created_casts() {
    let out = compile_and_run(r#"<?php
$source = $argc > 0 ? 'function dumpEval(int $a): void {} return new ReflectionFunction("dumpEval");' : '';
$r = eval($source);
echo $r;
$source = $argc > 0 ? 'class DumpEval { public function run(int $a): void {} }
    $m = new ReflectionMethod("DumpEval", "run"); echo $m;
    $p = new ReflectionParameter(["DumpEval", "run"], 0); echo $p, "\n";' : '';
eval($source);
"#);
    assert_eq!(out, "Function [ <user> function dumpEval ] {\n\n  - Parameters [1] {\n\
        \x20   Parameter #0 [ <required> int $a ]\n  }\n  - Return [ void ]\n}\n\
        Method [ <user> public method run ] {\n\n  - Parameters [1] {\n\
        \x20   Parameter #0 [ <required> int $a ]\n  }\n  - Return [ void ]\n}\n\
        Parameter #0 [ <required> int $a ]\n");
}

/// Constructors distinguish ordinary replacement from an abstract constructor prototype.
#[test]
fn test_reflection_dump_constructor_origins() {
    let out = compile_and_run(r#"<?php
class A { public function __construct() {} }
class B extends A { public function __construct() {} }
abstract class P { abstract public function __construct(); }
class Q extends P { public function __construct() {} }
echo new ReflectionMethod('B', '__construct');
echo new ReflectionMethod('Q', '__construct');
"#);
    assert_eq!(out, "Method [ <user, overwrites A, ctor> public method __construct ] {\n}\n\
        Method [ <user, overwrites P, prototype P, ctor> public method __construct ] {\n}\n");
}

/// AST-exported object arguments use double-quoted escapes for control characters.
#[test]
fn test_reflection_dump_object_string_escapes() {
    let out = compile_and_run(r#"<?php
class Box { public function __construct(public string $s) {} }
function f(Box $b = new Box("\n\t\\'é")) {}
echo new ReflectionParameter('f', 0);
"#);
    assert_eq!(out, "Parameter #0 [ <optional> Box $b = new \\Box(\"\\n\\t\\\\'é\") ]");
}

/// Lazy defaults keep their written dump without running constructors.
#[test]
fn test_reflection_dump_deferred_array_defaults() {
    let out = compile_and_run(r#"<?php
class Box { public function __construct() { echo "constructed"; } }
function f(array $items = [new Box()]) {}
echo new ReflectionParameter('f', 0);
"#);
    assert_eq!(out, "Parameter #0 [ <optional> array $items = [new \\Box()] ]");
}

