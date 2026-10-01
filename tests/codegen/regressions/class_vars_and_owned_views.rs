//! Purpose:
//! Regression tests for the native `get_class_vars()` and `compact()`, and for the ownership
//! defects found while building them: a null first store into an empty hash, owned string
//! temporaries viewed by the trim family and `substr`, a branch-promoted hash local returned under
//! `: array`, and fresh results presumed to alias their arguments.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected output was captured from php 8.5 running the same source.

use crate::support::*;

/// `get_class_vars()` reports DEFAULT values (statics included, never their current value),
/// instance properties before static ones, own declarations before inherited ones, and filters
/// non-public properties by the calling scope; an unknown name is a catchable `TypeError`.
#[test]
fn test_get_class_vars_matches_php() {
    let out = compile_and_run(
        r#"<?php
const K = 7;
class P { public $a = 1; protected $b = [2]; private $c = "c"; public static $s = 5; private static $ps = 6; public ?int $n = null; public int $u; public $k = K + 1;
  static function inside() { return get_class_vars(static::class); }
  static function self() { return get_class_vars("P"); } }
class Q extends P { public $q = "q"; public $a = 10; protected $b2 = 3; static function own() { return get_class_vars("Q"); } }
P::$s = 99;
echo json_encode(get_class_vars("Q")), "\n";
echo json_encode(P::inside()), "\n";
echo json_encode(Q::inside()), "\n";
echo json_encode(P::self()), "\n";
echo json_encode(Q::own()), "\n";
$n = "q";
echo json_encode(get_class_vars(strtoupper($n))), "\n";
try { get_class_vars("Nope"); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(out, "{\"q\":\"q\",\"a\":10,\"n\":null,\"u\":null,\"k\":8,\"s\":5}\n{\"a\":1,\"b\":[2],\"c\":\"c\",\"n\":null,\"u\":null,\"k\":8,\"s\":5,\"ps\":6}\n{\"q\":\"q\",\"a\":10,\"b2\":3,\"b\":[2],\"c\":\"c\",\"n\":null,\"u\":null,\"k\":8,\"s\":5,\"ps\":6}\n{\"a\":1,\"b\":[2],\"c\":\"c\",\"n\":null,\"u\":null,\"k\":8,\"s\":5,\"ps\":6}\n{\"q\":\"q\",\"a\":10,\"b2\":3,\"b\":[2],\"n\":null,\"u\":null,\"k\":8,\"s\":5}\n{\"q\":\"q\",\"a\":10,\"n\":null,\"u\":null,\"k\":8,\"s\":5}\nget_class_vars(): Argument #1 ($class) must be a valid class name, Nope given\n");
}

/// Eloquent's shape: a bare call inside a namespace on `static::class`, read by key; trait
/// properties count as the using class's own, and a trait answers for its own declarations.
#[test]
fn test_get_class_vars_in_a_namespace_through_static_class() {
    let out = compile_and_run(
        r#"<?php
namespace App\Models;
trait HasStamps { protected $stampFormat = 'Y-m-d'; public static $booted = false; }
abstract class Model {
    use HasStamps;
    public $timestamps = true;
    protected $table = null;
    protected $casts = ['id' => 'int', 'meta' => self::class];
    const KIND = 'model';
    protected $kind = self::KIND . '!';
    public static function usesTimestamps(): bool {
        $class = static::class;
        return get_class_vars($class)['timestamps'];
    }
    public function vars() { return get_class_vars(static::class); }
}
class Post extends Model { public $timestamps = false; protected $table = 'posts'; }
class Tag extends Model {}
var_dump(Post::usesTimestamps(), Tag::usesTimestamps());
echo json_encode(get_class_vars(Post::class)), "\n";
echo json_encode((new Tag())->vars()), "\n";
echo json_encode(get_class_vars('\App\Models\HasStamps')), "\n";
try { get_class_vars("Nope"); } catch (\TypeError $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
$total = 0;
for ($i = 0; $i < 50; $i++) { $total += count(get_class_vars(Post::class)); }
var_dump($total);
"#,
    );
    assert_eq!(out, "bool(false)\nbool(true)\n{\"timestamps\":false,\"booted\":false}\n{\"timestamps\":true,\"table\":null,\"casts\":{\"id\":\"int\",\"meta\":\"App\\\\Models\\\\Model\"},\"kind\":\"model!\",\"stampFormat\":\"Y-m-d\",\"booted\":false}\n{\"booted\":false}\nTypeError: get_class_vars(): Argument #1 ($class) must be a valid class name, Nope given\nint(100)\n");
}

/// The declared `get_class_vars` serves callable strings, which answer for the global scope.
#[test]
fn test_get_class_vars_as_a_callable_string() {
    let out = compile_and_run(
        r#"<?php
class P { public $a = 1; protected $b = 2; public static $s = 3; }
echo json_encode(call_user_func('get_class_vars', 'P')), "\n";
echo json_encode(array_map('get_class_vars', ['P'])), "\n";
$f = 'get_class_vars';
echo json_encode($f('P')), "\n";
echo function_exists('get_class_vars') ? "yes" : "no", "\n";
"#,
    );
    assert_eq!(out, "{\"a\":1,\"s\":3}\n[{\"a\":1,\"s\":3}]\n{\"a\":1,\"s\":3}\nyes\n");
}

/// `compact()` with literal names (nested arrays, duplicates, a null value, `\compact`, inside a
/// namespace and a method) builds php's array from the caller's variables.
#[test]
fn test_compact_with_literal_names() {
    let out = compile_and_run(
        r#"<?php
namespace App\Http;

function f() { $a = "x"; $b = [1]; $c = compact("a", "b"); return count($c) . json_encode($c); }
echo f(), "\n";
function g() { $a = "x"; $b = [1]; $n = null; return json_encode(compact(["a", ["b"]], "n", "a")); }
echo g(), "\n";
class Ctl { public function show(int $id) { $title = "T" . $id; $items = [1, 2]; return \compact('title', 'items', 'id'); } }
echo json_encode((new Ctl())->show(7)), "\n";
$top = "t";
echo json_encode(compact('top')), "\n";
"#,
    );
    assert_eq!(out, "2{\"a\":\"x\",\"b\":[1]}\n{\"a\":\"x\",\"b\":[1],\"n\":null}\n{\"title\":\"T7\",\"items\":[1,2],\"id\":7}\n{\"top\":\"t\"}\n");
}

/// `$a[1][] = 7` on `[[1, 2], [3]]` vivifies in one arm of a lowering-time branch; the
/// container's conversion to `array<mixed>` happened only there, so the other arm read raw
/// inner-array pointers as boxed cells and `$a[0]` printed `float(2.0E-323)`.
#[test]
fn test_nested_append_into_an_int_list_keeps_its_siblings() {
    let out = compile_and_run(
        r#"<?php
$a = [[1, 2], [3]];
$a[1][] = 7;
var_dump($a);
function f() { $b = [[1, 2], [3]]; $b[1][] = 7; $b[0][] = 8; return json_encode($b); }
echo f(), "\n";
$c = [["x"], []];
$c[1][] = "y";
$c[0][1] = "z";
echo json_encode($c), "\n";
"#,
    );
    assert_eq!(out, "array(2) {\n  [0]=>\n  array(2) {\n    [0]=>\n    int(1)\n    [1]=>\n    int(2)\n  }\n  [1]=>\n  array(2) {\n    [0]=>\n    int(3)\n    [1]=>\n    int(7)\n  }\n}\n[[1,2,8],[3,7]]\n[[\"x\",\"z\"],[\"y\"]]\n");
}

/// `$v = []; $v['k'] = null;` types the hash's values as null, which the backend refused to
/// store (`hash_set value PHP type Void`).
#[test]
fn test_first_null_store_into_an_empty_hash() {
    let out = compile_and_run(
        r#"<?php
function f(bool $s, string $k) {
    $vars = [];
    if ($s) { $vars[$k] = null; $vars['b'] = true; $vars['c'] = "x"; }
    else { $vars['z'] = [1]; }
    return $vars;
}
var_dump(f(true, "a"), f(false, "a"));
function g() { $v = []; $v['n'] = null; return $v; }
var_dump(g());
"#,
    );
    assert_eq!(out, "array(3) {\n  [\"a\"]=>\n  NULL\n  [\"b\"]=>\n  bool(true)\n  [\"c\"]=>\n  string(1) \"x\"\n}\narray(1) {\n  [\"z\"]=>\n  array(1) {\n    [0]=>\n    int(1)\n  }\n}\narray(1) {\n  [\"n\"]=>\n  NULL\n}\n");
}

/// Owned string temporaries viewed by `ltrim`/`trim`/`rtrim`/`substr` are released once the view
/// is copied, a returned acquired copy is not persisted twice, a local promoted to a hash in
/// one branch and returned under `: array` releases its boxed slot, and the owned arguments of
/// `str_pad`/`str_repeat`/`date`, whose results never alias them, are released.
#[test]
fn test_owned_string_views_and_branch_promoted_hash_returns_are_leak_free() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function f(bool $b): array { $v = []; if ($b) { $v["a"] = 1; } return $v; }
function g(mixed $c): string { return ltrim($c, "x"); }
function h(mixed $c): string { return substr($c, 1); }
function k($c) { return trim($c) . rtrim($c, "x"); }
function p($c) { return str_pad($c, 6, "-") . str_repeat($c, 2) . date("Y", 86400 * 400); }
function run(): void {
    $n = 0;
    $s = "";
    for ($i = 0; $i < 5; $i++) {
        $n += count(f($i % 2 == 0));
        $s = g("xxP") . h("ab") . k(" xPx ") . p("ab");
    }
    echo $n, "|", $s, "\n";
}
run();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "3|PbxPx xPx ab----abab1971\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// `__toString` results inside a concatenation, a loop that rewrites its foreach key in a branch
/// before keying a hash returned under `: array` (`array_combine`'s prelude shape), and the
/// owned arguments of `ucwords`/`wordwrap` are all released.
#[test]
fn test_tostring_concat_branchy_loop_hash_and_word_helpers_are_leak_free() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class T { public function __construct(private string $s) {} public function __toString(): string { return "<" . $this->s . ">"; } }
function keyed(array $keys): array {
    $result = [];
    foreach ($keys as $key) {
        if (!is_int($key) && !is_string($key)) { $key = (string) $key; }
        $result[$key] = strlen($key);
    }
    return $result;
}
function words($s) { return ucwords($s) . "|" . wordwrap($s, 3, "\n", true); }
function run(): void {
    $out = "";
    for ($i = 0; $i < 5; $i++) {
        $t = new T("v" . $i);
        $out = "x" . $t . "|" . count(array_combine(["a", "b"], ["x", "y"])) . "|" . json_encode(keyed(["k", "kk"])) . "|" . words("ab cd");
    }
    echo $out, "\n";
}
run();
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "x<v4>|2|{\"k\":1,\"kk\":2}|Ab Cd|ab\ncd\n");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}
