//! Purpose:
//! Regression tests for branch merges and array literals whose arms or elements are static
//! properties or calls, typed from their declarations rather than their syntax (issue #1501).
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every fixture reads `$argc` so the ternary, `?:` and `match` survive AST constant folding.
//! - The syntactic fallback answered `Int` for a static property or a call, which sized the
//!   merge temp as an integer: an array arm became its element count, a string or object arm
//!   became `0`, and a literal element was stamped as an integer. Expected output is from php.

use super::*;

/// Verifies a ternary over two static array properties yields the chosen array, at top level
/// and inside a function, rather than the array's element count (the shape of issue #1501).
#[test]
fn test_ternary_over_static_array_properties_keeps_the_array() {
    let out = compile_and_run(
        r#"<?php
class C { public static array $strs = ["a", "b"]; public static array $more = ["c"]; }
$x = $argc > 0 ? C::$strs : C::$more;
var_dump($x);
function g(int $n) { $y = $n > 0 ? C::$strs : C::$more; var_dump($y); }
g($argc);
g(0);
"#,
    );
    assert_eq!(
        out,
        "array(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n}\n\
         array(2) {\n  [0]=>\n  string(1) \"a\"\n  [1]=>\n  string(1) \"b\"\n}\n\
         array(1) {\n  [0]=>\n  string(1) \"c\"\n}\n"
    );
}

/// Verifies ternaries over static string, object, int/float and untyped properties keep each
/// arm's value: strings and objects were read back as `0`, and a float arm was truncated.
#[test]
fn test_ternary_over_static_scalar_object_and_untyped_properties() {
    let out = compile_and_run(
        r#"<?php
class O { public int $v = 5; }
class C {
    public static string $s1 = "hello";
    public static string $s2 = "world";
    public static ?O $o1 = null;
    public static ?O $o2 = null;
    public static int $i = 7;
    public static float $f = 1.5;
    public static $untyped = ["u"];
}
C::$o1 = new O();
$t = new O();
$t->v = 9;
C::$o2 = $t;
echo $argc > 0 ? C::$s1 : C::$s2, "\n";
echo strlen($argc > 5 ? C::$s1 : C::$s2), "\n";
echo ($argc > 0 ? C::$o1 : C::$o2)->v, "\n";
echo ($argc > 5 ? C::$o1 : C::$o2)->v, "\n";
var_dump($argc > 0 ? C::$i : C::$f);
var_dump($argc > 5 ? C::$i : C::$f);
var_dump($argc > 0 ? C::$untyped : C::$s1);
"#,
    );
    assert_eq!(
        out,
        "hello\n5\n5\n9\nint(7)\nfloat(1.5)\narray(1) {\n  [0]=>\n  string(1) \"u\"\n}\n"
    );
}

/// Verifies `?:`, `match`, `self::`/`static::` arms, a `foreach` source and a copied result
/// over static array properties, and that writing the copy leaves the property untouched.
#[test]
fn test_short_ternary_match_and_foreach_over_static_array_properties() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static array $strs = ["a", "b"];
    public static array $more = ["c"];
    public static string $s = "hello";
    public static function pick(int $n): array { return $n > 0 ? self::$strs : static::$more; }
}
echo implode(",", C::$strs ?: C::$more), "\n";
echo implode(",", match ($argc) { 1 => C::$strs, default => C::$more }), "\n";
echo match ($argc) { 1 => C::$s, default => "other" }, "\n";
echo implode(",", C::pick($argc)), "|", implode(",", C::pick(0)), "\n";
foreach ($argc > 0 ? C::$strs : C::$more as $v) {
    echo $v, ";";
}
echo "\n";
$y = $argc > 0 ? C::$strs : C::$more;
$y[0] = "changed";
echo count($y), " ", C::$strs[0], " ", $y[0], "\n";
"#,
    );
    assert_eq!(out, "a,b\na,b\nhello\na,b|c\na;b;\n2 a changed\n");
}

/// Verifies indexed and associative array literals type a static-property element from its
/// declaration: an array element was stamped `int` and printed as its count, and the
/// associative form failed to compile with an unsupported `hash_set` value type.
#[test]
fn test_array_literals_with_static_property_elements() {
    let out = compile_and_run(
        r#"<?php
class C {
    public static array $strs = ["a", "b"];
    public static string $s = "hello";
    public static float $f = 1.5;
    public static $untyped = ["u"];
}
var_dump([C::$strs, C::$s, C::$f]);
var_dump(["k" => C::$strs, "s" => C::$s, "f" => C::$f, "u" => C::$untyped]);
"#,
    );
    assert_eq!(
        out,
        "array(3) {\n  [0]=>\n  array(2) {\n    [0]=>\n    string(1) \"a\"\n    [1]=>\n    string(1) \"b\"\n  }\n  [1]=>\n  string(5) \"hello\"\n  [2]=>\n  float(1.5)\n}\n\
         array(4) {\n  [\"k\"]=>\n  array(2) {\n    [0]=>\n    string(1) \"a\"\n    [1]=>\n    string(1) \"b\"\n  }\n  [\"s\"]=>\n  string(5) \"hello\"\n  [\"f\"]=>\n  float(1.5)\n  [\"u\"]=>\n  array(1) {\n    [0]=>\n    string(1) \"u\"\n  }\n}\n"
    );
}

/// Verifies ternary arms that are function, method, static-method and builtin calls keep the
/// declared or inferred array result instead of being cast to `int(1)`.
#[test]
fn test_ternary_over_call_arms_keeps_array_results() {
    let out = compile_and_run(
        r#"<?php
function f() { return ["f1", "f2"]; }
class D {
    public static function a(): array { return ["da"]; }
    public function m() { return ["mm"]; }
}
$d = new D();
echo implode(",", $argc > 0 ? f() : f()), "\n";
echo implode(",", $argc > 0 ? $d->m() : $d->m()), "\n";
echo implode(",", $argc > 0 ? D::a() : f()), "\n";
echo implode(",", $argc > 5 ? D::a() : $d->m()), "\n";
echo implode(",", $argc > 0 ? explode(",", "x,y") : []), "\n";
"#,
    );
    assert_eq!(out, "f1,f2\nmm\nda\nmm\nx,y\n");
}

/// Verifies static-property and call arms in a loop release every merged value: the heap is
/// clean at exit, so no arm leaks the reference its merge temp acquired.
#[test]
fn test_static_property_and_call_arms_in_a_loop_are_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class O { public $v = 5; }
class C {
    public static array $strs = ["a", "b"];
    public static array $more = ["c"];
    public static string $s1 = "hello";
    public static string $s2 = "world";
    public static ?O $o1 = null;
    public static ?O $o2 = null;
}
function f() { return ["f1", "f2"]; }
class D { public function m() { return ["mm"]; } }
C::$o1 = new O();
C::$o2 = new O();
$d = new D();
$total = 0;
for ($i = 0; $i < 20 + $argc; $i++) {
    $a = $i % 2 ? C::$strs : C::$more;
    $b = $i % 3 ? C::$s1 : C::$s2;
    $o = $i % 2 ? C::$o1 : C::$o2;
    $c = C::$strs ?: C::$more;
    $m = match ($i % 3) { 0 => C::$strs, 1 => C::$more, default => ["lit"] };
    $l = [C::$strs, C::$s1, "k" => C::$o1];
    $g = $i % 2 ? f() : $d->m();
    $total += count($a) + strlen($b) + $o->v + count($c) + count($m) + count($l) + count($g);
}
echo $total;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "405");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}
