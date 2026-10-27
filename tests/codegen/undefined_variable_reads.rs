//! Purpose:
//! Regressions for php's run-time handling of variables and constants nothing defines.
//!
//! Called from:
//! - `cargo test --test codegen_tests undefined_variable_reads`.
//!
//! Key details:
//! - php compiles such reads, warns `Undefined variable $x` at run time and yields `null`.
//! - `isset`, `empty`, the left side of `??` and `@` stay silent; an echo operand warns.
//! - An undefined constant compiles and throws `Error: Undefined constant "X"` when read.

use crate::support::*;

/// Undefined reads warn at run time and behave as `null` in every scope kind.
#[test]
fn test_undefined_variable_reads_warn_and_yield_null() {
    let run = compile_and_run_capture(r#"<?php
var_dump((bool) (true ? $x : $y));
foreach ($missing as $v) { echo "never"; }
function f() { return $undefined + 1; }
var_dump(f());
$fn = function () { return $inside; };
var_dump($fn());
$s = "[" . $len . "]";
echo $s, "\n";
echo "done\n";
"#);
    let (out, err) = (run.stdout, run.stderr);
    for name in ["x", "missing", "undefined", "inside", "len"] {
        assert!(err.contains(&format!("Warning: Undefined variable ${name}")), "{name}: {err}");
    }
    assert!(!err.contains("Undefined variable $y"), "the untaken ternary arm must not warn: {err}");
    assert!(out.contains("bool(false)\n"), "{out}");
    for value in ["int(1)\n", "NULL\n", "[]\n"] {
        assert!(out.contains(value), "{value}: {out}");
    }
    assert!(!out.contains("never"), "{out}");
    assert!(out.ends_with("done\n"), "{out}");
}

/// Probes, null coalescing and `@` read undefined variables silently.
#[test]
fn test_undefined_variable_probes_stay_silent() {
    let run = compile_and_run_capture(r#"<?php
var_dump(isset($nope), empty($nope), $nope ?? "dflt", @$quiet);
"#);
    assert!(!run.stderr.contains("Warning"), "{}", run.stderr);
    assert_eq!(run.stdout, "bool(false)\nbool(true)\nstring(4) \"dflt\"\nNULL\n");
}

/// A variable written anywhere in its scope keeps the ordinary compile-time handling.
#[test]
fn test_variable_written_later_is_not_rewritten() {
    let out = compile_and_run(r#"<?php
function g(int $n): int { $total = 0; for ($i = 0; $i < $n; $i++) { $total += $i; } return $total; }
echo g(4), "\n";
"#);
    assert_eq!(out, "6\n");
}

/// Method and enum-method bodies are their own scopes: a top-level local is not visible there,
/// and a builtin argument read of a never-defined name warns before the call runs.
#[test]
fn test_undefined_variable_reads_in_methods_and_builtin_args() {
    let run = compile_and_run_capture(r#"<?php
$value = 5;
class C { public function f() { return $value; } }
enum E { case A; public function g() { return $missing; } }
var_dump((new C())->f());
var_dump(E::A->g());
var_dump(ord($undeclared . "A"));
"#);
    for name in ["value", "missing", "undeclared"] {
        assert!(run.stderr.contains(&format!("Warning: Undefined variable ${name}")), "{name}: {}", run.stderr);
    }
    assert_eq!(run.stdout, "NULL\nNULL\nint(65)\n");
}

/// An undefined constant is php's catchable run-time `Error`, raised only when the read runs;
/// constants are case-sensitive, and a declaration behind a branch is still read back by value.
#[test]
fn test_undefined_constant_reads_throw_when_reached() {
    let out = compile_and_run(r#"<?php
const MyConst = 1;
try { echo myconst; } catch (Error $e) { echo $e->getMessage(), "\n"; }
function getA() { return A; }
try { getA(); } catch (Error $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
try { while (y) { echo "never"; } } catch (Error $e) { echo $e->getMessage(), "\n"; }
if ($argc > 5) { echo NEVER_DEFINED; }
echo $argc > 5 ? NOT_REACHED : "ok", "\n";
const B = 2;
echo B, "\n";
"#);
    assert_eq!(
        out,
        "Undefined constant \"myconst\"\nError: Undefined constant \"A\"\nUndefined constant \"y\"\nok\n2\n",
    );
}

/// Inside a namespace, `namespace\PHP_EOL` names that namespace's constant, never the global one.
#[test]
fn test_relative_constant_has_no_global_fallback() {
    let out = compile_and_run(
        "<?php namespace App; try { echo namespace\\PHP_EOL; } catch (\\Error $e) { echo $e->getMessage(); }",
    );
    assert_eq!(out, "Undefined constant \"App\\PHP_EOL\"");
}

/// An uncaught undefined-constant read ends the program with php's fatal error.
#[test]
fn test_undefined_constant_uncaught_is_fatal() {
    let run = compile_and_run_capture("<?php echo \"a\\n\"; foo; echo \"b\\n\";");
    assert!(!run.success);
    assert!(
        run.stdout.starts_with("a\n\nFatal error: Uncaught Error: Undefined constant \"foo\" in "),
        "{}",
        run.stdout
    );
    assert!(!run.stdout.contains("b\n"), "{}", run.stdout);
}

/// An untyped closure parameter holds whatever the caller passes: an error handler's message
/// (its second argument, a string) reaches `new Exception($m)`, which an `int` view refused.
#[test]
fn test_untyped_closure_parameters_are_mixed() {
    let out = compile_and_run(r#"<?php
$f = function($a, $b) { throw new Exception($b); };
try { $f(1, "two"); } catch (Exception $e) { echo $e->getMessage(), "\n"; }
set_error_handler(function($_, $m) { throw new Exception($m); });
try { echo $undefined; } catch (Exception $e) { echo $e->getMessage(), "\n"; }
"#);
    assert_eq!(out, "two\nUndefined variable $undefined\n");
}

/// A variable the scope also assigns as a whole keeps the checker's own handling: hoisting
/// `$a = []` before the loop typed it as an array and refused `$a = false` (assign_dim_017).
#[test]
fn test_vivification_skips_a_variable_also_assigned_whole() {
    let out = compile_and_run(r#"<?php
function test() { for ($i = 0; $i < 3; $i++) { $a[] = 0; $a = false; } }
@test();
echo "DONE\n";
"#);
    // The function's own return value is left out: a string return after this loop reads back
    // wrong on its own (bead elephc-4ml.23).
    assert_eq!(out, "DONE\n");
}

/// An element write to an undefined variable creates the array, at top level and inside a
/// function (where the result used to be garbage); a loop keeps appending, and a write in an
/// untaken branch leaves the variable undefined.
#[test]
fn test_element_write_vivifies_an_undefined_variable() {
    let run = compile_and_run_with_heap_debug(r#"<?php
$my_var[] = 5;
echo json_encode($my_var), "\n";
function f() { $a['k'] = 1; $b[] = 2; $c[1][2] = 3; return [$a, $b, $c]; }
echo json_encode(f()), "\n";
for ($i = 0; $i < 3; $i++) { $acc[] = $i; }
echo json_encode($acc), "\n";
function g($flag) {
    if ($flag) { $x['on'] = 1; }
    return isset($x) ? json_encode($x) : "unset";
}
echo g(true), " ", g(false), "\n";
"#);
    assert!(run.success, "stdout: {}\nstderr: {}", run.stdout, run.stderr);
    assert_eq!(
        run.stdout,
        "[5]\n[{\"k\":1},[2],{\"1\":{\"2\":3}}]\n[0,1,2]\n{\"on\":1} unset\n"
    );
    assert!(run.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", run.stderr);
}

/// A read no write on any path can precede warns even when the scope writes the name later,
/// while a write on some branch, an earlier loop iteration or the condition keeps it defined.
#[test]
fn test_read_before_any_write_warns() {
    let run = compile_and_run_capture(r#"<?php
function f() {
    if (isset($abc)) { echo "no\n"; }
    var_dump($abc);
    $abc = 1;
    var_dump($abc);
}
f();
function g($c) {
    if ($c) { $x = 1; }
    var_dump($x);
    for ($i = 0; $i < 2; $i++) { if ($i == 1) { echo $y, "\n"; } $y = "y"; }
    if ($m = strlen("ab")) { echo $m, "\n"; }
}
g(true);
echo $top, "|\n";
$top = 1;
echo $top, "\n";
"#);
    let (out, err) = (run.stdout, run.stderr);
    assert!(err.contains("Undefined variable $abc"), "{err}");
    assert!(err.contains("Undefined variable $top"), "{err}");
    for name in ["$x", "$y", "$m"] {
        assert!(!err.contains(&format!("Undefined variable {name}")), "{name}: {err}");
    }
    assert_eq!(out, "NULL\nint(1)\nint(1)\ny\n2\n|\n1\n");
}

/// Reads after `unset`, a self-read in an assignment, a loop's first iteration, a reference to
/// an undefined source, a property write on one, an index under `isset`, and literal-key
/// `$GLOBALS` all behave like php (output measured with php 8.5).
#[test]
fn test_undefined_reads_in_ordered_and_loop_positions() {
    let run = compile_and_run_capture(r#"<?php
function a() { $v = 1; unset($v); var_dump($v); }
function b() { $old = isset($old) ? (array)$old : []; var_dump($old); }
function c() { $n = 0; do { $n++; $total = ($total ?? 0) + $n; $hits++; } while ($n < 3); var_dump($total, $hits); }
function d() { $c =& $a; var_dump($a); }
function e() { try { $obj->a = 3; } catch (Error $e) { echo $e->getMessage(), "\n"; } }
function f() { var_dump(isset($arr[$k])); }
a(); b(); c(); d(); e(); f();
class Box { public $n = 1; }
$box = new Box;
function g() { $GLOBALS['box']->n++; $GLOBALS['made'] = 'yes'; return $GLOBALS['box']->n; }
var_dump(g(), $made);
"#);
    let (out, err) = (run.stdout, run.stderr);
    for name in ["$v", "$hits", "$k"] {
        assert!(err.contains(&format!("Undefined variable {name}")), "{name}: {err}");
    }
    for name in ["$old", "$total", "$a", "$obj"] {
        assert!(!err.contains(&format!("Undefined variable {name}")), "{name}: {err}");
    }
    assert_eq!(
        out,
        "NULL\narray(0) {\n}\nint(6)\nint(3)\nNULL\nAttempt to assign property \"a\" on null\n\
         bool(false)\nint(2)\nstring(3) \"yes\"\n"
    );
}
