//! Purpose:
//! Callable and eval-bridge gaps closed together: calling a first-class callable where it is
//! made, every `Closure::fromCallable()` form, request-wide eval statics, and `$v = &call()`
//! over a callee that returns a value.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through Rust's test harness.
//!
//! Key details:
//! - Expected stdout is `php -n` 8.5's own; php's notices go to stderr here, as elephc's do.

use crate::support::*;

/// `$o->m(...)()` and `f(...)(x)` call the Closure they just built; the parser stopped at the
/// second `(`. `$o->me()->m(...)` over an untyped `return $this` also named the class "".
#[test]
fn test_a_first_class_callable_is_called_where_it_is_made() {
    let out = compile_and_run(r#"<?php
class K {
    public $n = 3;
    function m($x = 1) { return "m" . ($x + $this->n); }
    static function s($x) { return "s" . $x; }
    function me() { return $this; }
}
$o = new K;
echo $o->m(...)(), "\n";
echo $o->m(...)(5), "\n";
echo K::s(...)("a"), "\n";
echo strtoupper(...)("abc"), "\n";
echo $o->me()->m(...)(2), "\n";
echo $o->m(...)(...)(7), "\n";
"#);
    assert_eq!(out, r#"m4
m8
sa
ABC
m5
m10
"#);
}

/// Only a function-name string was rewritten to a first-class callable; every other literal
/// reached the eval bridge, which leaked its argument and refused a compiled Closure outright.
#[test]
fn test_closure_from_callable_accepts_every_literal_callable() {
    let out = compile_and_run(r#"<?php
class K {
    public $n = 3;
    function m($x = 1) { return "m" . ($x + $this->n); }
    static function s($x = "d") { return "s" . $x; }
}
$o = new K;
echo Closure::fromCallable([$o, 'm'])(2), "\n";
echo Closure::fromCallable('K::s')("a"), "\n";
echo Closure::fromCallable(['K', 's'])(), "\n";
echo Closure::fromCallable('strtoupper')("x"), "\n";
$c = Closure::fromCallable(function ($v) { return $v * 2; });
echo $c(21), "\n";
echo get_class(Closure::fromCallable([new K, 'm'])), "\n";
$f = Closure::fromCallable(fn($v) => $v + 1);
var_dump($f instanceof Closure, $f(1));
$g = function () { return "same"; };
var_dump(Closure::fromCallable($g) === $g);
echo Closure::fromCallable($g)(), "\n";
$h = $g;
var_dump($g === $h, $g === $c, $g !== $c);
"#);
    assert_eq!(out, r#"m5
sa
sd
X
42
Closure
bool(true)
int(2)
bool(true)
same
bool(true)
bool(false)
bool(true)
"#);
}

/// A `mixed` callback still goes through the bridge, which answered a compiled Closure with
/// `function "" not found`: Twig's `ReflectionCallable` does exactly this for a closure filter.
#[test]
fn test_closure_from_callable_over_a_runtime_value() {
    let out = compile_and_run(r#"<?php
class K {
    public $n = 3;
    function m($x = 1) { return "m" . ($x + $this->n); }
    static function s($x = "d") { return "s" . $x; }
}
function mk(mixed $callable): Closure {
    try {
        $closure = \Closure::fromCallable($callable);
    } catch (\TypeError $e) {
        throw new \LogicException("not callable", 0, $e);
    }
    return $closure;
}
$g = fn($v) => $v * 2;
echo mk($g)(21), "\n";
var_dump(mk($g) === $g);
echo mk('strtoupper')("x"), "\n";
echo mk([new K, 'm'])(2), "\n";
echo mk('K::s')("a"), "\n";
echo mk(['K', 's'])(), "\n";
try { mk('nope_fn'); } catch (LogicException $e) { echo get_class($e->getPrevious()), "\n"; }
"#);
    assert_eq!(out, r#"42
bool(true)
X
m5
sa
sd
TypeError
"#);
}

/// Each compiled activation builds its own eval context, and statics lived in the context:
/// `static $c` and `self::$shared` restarted every time the calling function returned.
#[test]
fn test_eval_class_statics_outlive_the_calling_function() {
    let out = compile_and_run(r#"<?php
class EvalPrelude {}
eval('
class Z {
    static $shared = [];
    function counter() { static $c = 0; $c++; return $c; }
    static function remember($v) { self::$shared[] = $v; return count(self::$shared); }
}
');
function run() {
    $z = new Z;
    echo $z->counter(), $z->counter(), " ", Z::remember(1), Z::remember(2), "\n";
}
run();
run();
"#);
    assert_eq!(out, r#"12 12
34 34
"#);
}

/// `$v = &f()` over a by-value callee bound the returned VALUE as a reference cell and
/// faulted on the next access. php assigns it (with a notice); an eval-declared `&method` too.
#[test]
fn test_reference_bind_of_a_by_value_call_takes_the_value() {
    let out = compile_and_run(r#"<?php
class N { public $items = ["a"]; function &ref() { return $this->items; } function val() { return $this->items; } }
function g() { return [1]; }
class EvalPrelude {}
eval('class Z { public $items = ["a"]; function &ref() { return $this->items; } }');
function run() {
    $n = new N;
    $r = &$n->ref();
    $r[] = "b";
    echo implode(",", $n->items), "\n";
    $v = &$n->val();
    $v[] = "c";
    echo implode(",", $n->items), " ", implode(",", $v), "\n";
    $w = &g();
    $w[] = 2;
    echo implode(",", $w), "\n";
    $z = new Z;
    $e = &$z->ref();
    $e[] = "b";
    echo implode(",", $z->items), "\n";
}
run();
"#);
    assert_eq!(out, r#"a,b
a,b a,b,c
1,2
a,b
"#);
}

/// A constant-true guard folds to a plain block before checking, and the return fold skipped
/// such blocks: the guarded `return false` vanished from the inferred type, which became the
/// other exit's alone. An array exit failed codegen; a string exit silently answered `""`.
/// (`opcache_get_status()` has exactly this shape when the build disables the cache.)
#[test]
fn test_a_return_behind_a_constant_guard_counts_toward_the_return_type() {
    let out = compile_and_run(r#"<?php
function gate_false() { if (false === false) { return false; } $status = ["x" => 1]; return $status; }
function gate_true() { if (true) { return 5; } return ["x" => 1]; }
function gate_string() { if (false === false) { return false; } $s = "x"; return $s; }
var_dump(gate_false(), gate_true(), gate_string());
"#);
    assert_eq!(out, r#"bool(false)
int(5)
bool(false)
"#);
}

/// A dynamic method call whose only argument is a spread binds the spread's string keys to the
/// parameters NAMED by them, as php does: `$this->{$m}(...['data' => [1], 'view' => 'v'])` reaches
/// `named('v', [1])`. The call used to be desugared to `call_user_func`, which passes every entry
/// positionally. Laravel's `ViewController::callAction` is exactly this shape. Output is php 8.5's.
#[test]
fn test_dynamic_method_call_with_a_lone_spread_binds_string_keys_by_name() {
    let out = compile_and_run(r#"<?php
class K {
    public function collect(...$args) { return json_encode($args); }
    public function named($view, $data = []) { return json_encode([$view, $data]); }
    public function viaName($m, $p) { return $this->{$m}(...$p); }
    public function viaPositional($m, $a, $b) { return $this->{$m}($a, $b); }
}
$k = new K();
echo $k->viaName('named', ['data' => [1], 'view' => 'v']), "\n";
echo $k->viaName('named', ['v', [2]]), "\n";
echo $k->viaPositional('named', 'w', [3]), "\n";
echo $k->viaName('collect', ['a', 'b']), "\n";
"#);
    assert_eq!(out, "[\"v\",[1]]\n[\"v\",[2]]\n[\"w\",[3]]\n[\"a\",\"b\"]\n");
}
