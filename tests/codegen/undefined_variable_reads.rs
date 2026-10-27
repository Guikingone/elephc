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
