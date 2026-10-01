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

/// A compiled user function whose name begins with an underscore (`__()` and `_e()`, the
/// translation helpers frameworks define) is an ordinary function that interpreted code can call,
/// find with `function_exists()` and use as a callable. eval hid every compiled function whose
/// name started with `_` to keep the compiler's synthetic functions private, so a Laravel view
/// calling `__('Not Found')` died with "call to undefined function __()". Output is php 8.5's.
#[test]
fn test_eval_sees_compiled_functions_whose_names_start_with_an_underscore() {
    let out = compile_and_run(r#"<?php
function __($key = null) { return 'tr:' . $key; }
function _e($key) { return 'e:' . $key; }
if (! function_exists('_cond')) {
    function _cond($key) { return 'c:' . $key; }
}
echo __('a'), _e('a'), _cond('a'), "\n";
$code = 'echo __("b"), _e("b"), _cond("b"), "\n";
echo call_user_func("__", "c"), call_user_func("_e", "c"), call_user_func("_cond", "c"), "\n";
var_dump(function_exists("__"), function_exists("_e"), function_exists("_cond"));';
eval($code);
"#);
    assert_eq!(
        out,
        "tr:ae:ac:a\ntr:be:bc:b\ntr:ce:cc:c\nbool(true)\nbool(true)\nbool(true)\n"
    );
}

/// `get_defined_vars()` inside interpreted code answers the running scope's variables: the ones
/// the compiled caller exposes plus those the fragment bound, without an `unset` one and without
/// `$this`. It was "call to undefined function get_defined_vars()", which killed every Laravel
/// page whose Blade view uses `@include` (it compiles to
/// `array_diff_key(get_defined_vars(), ['__data' => 1, '__path' => 1])`). The ORDER php reports
/// is the lexical slot order of the enclosing function, which an interpreter cannot know, so the
/// fixture sorts the keys; and a compiled local that was never assigned is exposed to the
/// fragment as null, so the fixture keeps only non-null entries. Output is php 8.5's.
#[test]
fn test_eval_get_defined_vars_reports_the_running_scope() {
    let out = compile_and_run(r#"<?php
function render($__path, $__data) {
    extract($__data);
    $local = 5;
    $code = '$more = 1; $zap = 2; unset($zap); $vars = array_filter(array_diff_key(get_defined_vars(), ["__data" => 1, "__path" => 1, "code" => 1]), fn($v) => $v !== null); ksort($vars); echo json_encode($vars), "\n";';
    eval($code);
}
render('p', ['a' => 1, 'b' => [2, 3]]);
class T {
    public $p = 1;
    function run() { $q = 2; $code = '$names = array_keys(array_filter(get_defined_vars(), fn($v) => $v !== null)); sort($names); echo json_encode($names), "\n";'; eval($code); }
}
(new T)->run();
"#);
    assert_eq!(
        out,
        "{\"a\":1,\"b\":[2,3],\"local\":5,\"more\":1}\n[\"code\",\"q\"]\n"
    );
}

/// A constant the compiled program `define()`s is visible to interpreted code: `defined()` is
/// true and a bare name reads the value, as in php's single constant namespace. The compiled
/// `define()` kept its value in a program global the interpreter could not read, so Laravel's
/// `health-up` view (`@if (defined('LARAVEL_START'))`) silently dropped its timing line. The value
/// is the runtime one (`$n * 2`), not a folded literal, and an undefined name stays undefined.
/// Output is php 8.5's.
#[test]
fn test_eval_sees_constants_the_compiled_program_defined() {
    let out = compile_and_run(r#"<?php
$n = (int) "21";
define('LV_INT', $n * 2);
define('LV_STR', 'a' . 'b' . $n);
define('LV_FLOAT', $n / 4);
define('LV_BOOL', $n > 5);
$code = 'var_dump(defined("LV_INT"), LV_INT, LV_STR, LV_FLOAT, LV_BOOL, constant("LV_STR"), defined("LV_NOPE"));';
eval($code);
"#);
    assert_eq!(
        out,
        "bool(true)\nint(42)\nstring(4) \"ab21\"\nfloat(5.25)\nbool(true)\nstring(4) \"ab21\"\nbool(false)\n"
    );
}

/// A static call on a name an SPL autoloader only makes known as a `class_alias()` dispatches under
/// the REAL class: `static::class` is the target's name and `static::$prop` reads the target's
/// slot. The interpreter resolved the receiver before the autoloader ran, so the late-static class
/// of the call stayed the alias spelling the compiled side cannot map to a class, and Laravel's
/// facade root (`static::getFacadeRoot()`, read through that class) came back as garbage on the
/// first `Route::has()` of a Blade view. Output is php 8.5's.
#[test]
fn test_static_call_through_an_autoloaded_class_alias_uses_the_real_class() {
    let out = compile_and_run(r#"<?php
class Router { public function has($name) { return $name === "login"; } }
abstract class Facade {
    protected static $app;
    protected static $resolvedInstance;
    protected static function getFacadeAccessor() { throw new RuntimeException('no accessor'); }
    public static function setApp($app) { static::$app = $app; }
    public static function getFacadeRoot() { return static::resolveFacadeInstance(static::getFacadeAccessor()); }
    protected static function resolveFacadeInstance($name) {
        if (isset(static::$resolvedInstance[$name])) { return static::$resolvedInstance[$name]; }
        if (static::$app) { return static::$resolvedInstance[$name] = static::$app[$name]; }
    }
    public static function __callStatic($method, $args)
    {
        $instance = static::getFacadeRoot();
        echo static::class, ' ', get_debug_type($instance), "\n";
        return $instance->$method(...$args);
    }
}
class RouteFacade extends Facade { protected static function getFacadeAccessor() { return 'router'; } }
$loaderCode = 'class Loader {
    protected $aliases = ["Route" => "RouteFacade"];
    public function load($alias) {
        if (isset($this->aliases[$alias])) { return class_alias($this->aliases[$alias], $alias); }
    }
    public function register() { spl_autoload_register([$this, "load"], true, true); }
}';
eval($loaderCode);
(new Loader)->register();
Facade::setApp(['router' => new Router()]);
$code = 'var_dump(Route::has("login")); var_dump(Route::has("other"));';
eval($code);
"#);
    assert_eq!(
        out,
        "RouteFacade Router\nbool(true)\nRouteFacade Router\nbool(false)\n"
    );
}
