//! Purpose:
//! Integration and regression tests for the REFUSAL half of a weak-mode scalar argument boundary:
//! a boxed `mixed` value carrying a payload PHP rejects at a declared `string`/`int`/`float`/`bool`
//! parameter must raise the same catchable `\TypeError` PHP raises, instead of being fed to the
//! boundary's silent coercion.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - These are the cases that were SILENTLY MISCOMPILED before the guard existed, measured against
//!   `php -n` on php-8.5.6: an array reaching a declared `string` printed `""`, an array reaching a
//!   declared `int` printed its ELEMENT COUNT, and a `null` reaching any non-nullable scalar
//!   printed that scalar's zero value — all with exit status 0, where PHP throws.
//! - The complement matters just as much and is pinned here too: every payload PHP CONVERTS
//!   (`int`, `float`, `bool`, numeric string into any of the four; `null` into a `?T`) must still
//!   convert. A guard that threw on those would be an over-refusal, which is the exact failure mode
//!   `crate::types::checked_downcast::guard_is_php_faithful` keeps the arm chain away from.
//! - The catch-reachability test is not decoration: the AST effect model prunes catch clauses whose
//!   try body "cannot throw", and a declared type boundary is a throw site no statement of the
//!   callee's BODY contains. Without `optimize::declared_type_boundary_may_throw` the handler is
//!   deleted and the guard's `TypeError` escapes as an uncaught fatal.
//! - Expected messages are php-8.5.6 verbatim minus PHP's `, called in <file> on line <n>` tail,
//!   which an AOT binary would have to bake in from its compile-time path.

use super::*;

/// An ARRAY payload reaching a declared `string` parameter throws PHP's own catchable `TypeError`.
///
/// Before the refusal guard this printed `[]` — the array fell straight into the boundary's
/// `IToStr` coercion, which yields the empty string — and the program continued with exit 0.
#[test]
fn test_boxed_array_into_declared_string_parameter_throws_catchable_type_error() {
    let out = compile_and_run(
        r#"<?php
function g(string $s): string { return "[" . $s . "]"; }
function mk(): mixed { return [1, 2]; }
try { echo g(mk()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "g(): Argument #1 ($s) must be of type string, array given\nalive"
    );
}

/// An ARRAY payload reaching a declared `int` parameter throws rather than silently becoming its
/// element COUNT — the most damaging member of this family, because the wrong value is plausible.
#[test]
fn test_boxed_array_into_declared_int_parameter_throws_instead_of_element_count() {
    let out = compile_and_run(
        r#"<?php
function f(int $v): string { return "i:" . $v; }
function mk(): mixed { return [1, 2]; }
try { echo f(mk()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "f(): Argument #1 ($v) must be of type int, array given\nalive"
    );
}

/// A `null` payload reaching a NON-nullable declared scalar throws; PHP does not coerce null into
/// a declared scalar for a userland callee.
#[test]
fn test_boxed_null_into_non_nullable_declared_scalar_throws() {
    let out = compile_and_run(
        r#"<?php
function g(string $s): string { return "[" . $s . "]"; }
function f(int $v): string { return "i:" . $v; }
function mk(): mixed { return null; }
try { echo g(mk()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
try { echo f(mk()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "g(): Argument #1 ($s) must be of type string, null given\n\
         f(): Argument #1 ($v) must be of type int, null given\n\
         alive"
    );
}

/// A NULLABLE declaration renders as `?string` in the message and still refuses an array, while
/// letting a real `null` through — the two halves of the same declaration.
#[test]
fn test_nullable_scalar_declaration_refuses_array_but_admits_null() {
    let out = compile_and_run(
        r#"<?php
function h(?string $s): string { return "<" . ($s === null ? "NULL" : $s) . ">"; }
function arr(): mixed { return [1, 2]; }
function nil(): mixed { return null; }
try { echo h(arr()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo h(nil()), "\n";
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "h(): Argument #1 ($s) must be of type ?string, array given\n<NULL>\nalive"
    );
}

/// The complement: every payload PHP CONVERTS at these boundaries still converts. An over-refusing
/// guard would throw here, and the silent-coercion family this fixes would simply have moved.
#[test]
fn test_coercible_payloads_still_convert_at_a_declared_scalar_parameter() {
    let out = compile_and_run(
        r#"<?php
function fs(string $v): string { return "s:" . $v; }
function fi(int $v): string { return "i:" . $v; }
function ff(float $v): string { return "f:" . $v; }
function fb(bool $v): string { return "b:" . ($v ? "T" : "F"); }
function mi(): mixed { return 42; }
function mf(): mixed { return 1.5; }
function mb(): mixed { return true; }
function ms(): mixed { return "7"; }
echo fs(mi()), " ", fs(mf()), " ", fs(mb()), " ", fs(ms()), "\n";
echo fi(mi()), " ", fi(mb()), " ", fi(ms()), "\n";
echo ff(mi()), " ", ff(mf()), " ", ff(ms()), "\n";
echo fb(mi()), " ", fb(mf()), " ", fb(ms()), "\n";
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "s:42 s:1.5 s:1 s:7\ni:42 i:1 i:7\nf:42 f:1.5 f:7\nb:T b:T b:T\nalive"
    );
}

/// The guard's `TypeError` is reachable by a `catch` around a call whose callee's BODY contains no
/// throw at all — the case the AST effect model used to prune the handler for.
///
/// `f`'s body is a bare `return`; every throw here comes from its declared parameter type. With the
/// handler pruned, the same program dies with an uncaught fatal and a non-zero exit status.
#[test]
fn test_declared_parameter_boundary_throw_is_catchable_when_the_callee_body_cannot_throw() {
    let out = compile_and_run(
        r#"<?php
function f(int $v): int { return $v; }
function mk(): mixed { return [1, 2]; }
try {
    echo f(mk()), "\n";
} catch (\Throwable $e) {
    echo "caught ", get_class($e), "\n";
}
echo "alive";
"#,
    );
    assert_eq!(out, "caught TypeError\nalive");
}

/// An OBJECT payload at a declared `string` parameter is decided by `instanceof Stringable` —
/// PHP's own rule, not an approximation: a `__toString`-bearing class converts, a plain one throws.
///
/// Before this, the plain-object case died with an UNCATCHABLE `Fatal error: Object could not be
/// converted to string`, so a `catch (\Throwable)` around it never ran and the process exited 1.
#[test]
fn test_object_payload_at_a_string_parameter_is_decided_by_stringable() {
    let out = compile_and_run(
        r#"<?php
class Plain {}
class Strish { public function __toString(): string { return "S"; } }
function fs(string $v): string { return "s:" . $v; }
function plain(): mixed { return new Plain(); }
function strish(): mixed { return new Strish(); }
try { echo fs(strish()), "\n"; } catch (\Throwable $e) { echo "unexpected\n"; }
try { echo fs(plain()), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "s:S\nfs(): Argument #1 ($v) must be of type string, Plain given\nalive"
    );
}

/// The whole point of the family, in Symfony's own shape: a parameter-bag-style
/// `array|bool|string|int|float|<interface>|null` return handed to a `?string` and a `?array`
/// parameter. PHP RUNS this — converting the scalars, passing string/null through, and throwing a
/// catchable `TypeError` for the payloads the declaration refuses — so refusing it at COMPILE time
/// rejected a program PHP accepts.
#[test]
fn test_wide_union_argument_into_a_narrow_declared_parameter_matches_php() {
    let out = compile_and_run(
        r#"<?php
interface Marker {}
class Enumish implements Marker {}
class Bag {
    private array $vals = [];
    public function set(string $k, mixed $v): void { $this->vals[$k] = $v; }
    public function get(string $k): array|bool|string|int|float|Marker|null {
        return $this->vals[$k] ?? null;
    }
}
function useEnv(?string $env): string { return "e:" . ($env ?? 'NULL'); }
function useArr(?array $a): string { return "a:" . ($a === null ? 'NULL' : count($a)); }
$b = new Bag();
$b->set('s', 'prod');
$b->set('i', 7);
$b->set('arr', [1, 2, 3]);
$b->set('obj', new Enumish());
// Direct calls on purpose: a `$fn(...)` variable-callable routes through the uniform invoker,
// which is a DIFFERENT boundary with no signature and therefore no guard.
foreach (['s', 'i', 'arr', 'obj', 'missing'] as $k) {
    try { echo useEnv($b->get($k)), "|"; } catch (\TypeError $e) { echo "T|"; }
    try { echo useArr($b->get($k)), "|"; } catch (\TypeError $e) { echo "T|"; }
}
echo "\nalive";
"#,
    );
    // Per key: (?string, ?array). string→e:prod/T, int→e:7/T, array→T/a:3, object→T/T, null→both NULL.
    assert_eq!(out, "e:prod|T|e:7|T|T|a:3|T|T|e:NULL|a:NULL|\nalive");
}

/// A statically-proven-good argument pays nothing: a `mixed` whose declared parameter is not a
/// scalar conversion boundary, and a concrete scalar source, both emit ZERO refusal blocks.
#[test]
fn test_scalar_coercion_refusal_guard_is_not_emitted_where_it_cannot_fire() {
    let ir = emit_ir_for_scalar_refusal(
        r#"<?php
function keep(mixed $v): mixed { return $v; }
function concrete(string $s): string { return $s; }
function mk(): mixed { return "x"; }
echo concrete("literal"), "\n";
echo keep(mk()) === null ? "n" : "y", "\n";
"#,
    );
    assert!(
        !ir.contains("arg_coercion_guard"),
        "no refusal guard may be emitted for a `mixed` parameter or a concrete string argument; EIR:\n{}",
        ir
    );
}

/// Emits textual EIR for a source snippet through the CLI (`--emit-ir`, optimizer on).
fn emit_ir_for_scalar_refusal(source: &str) -> String {
    use std::fs;
    use std::process::Command;

    let dir = make_cli_test_dir("elephc_scalar_coercion_refusal_emit_ir");
    let php_path = dir.join("main.php");
    fs::write(&php_path, source).expect("failed to write PHP fixture");

    let mut command: Command = elephc_cli_command(&dir);
    command.arg("--emit-ir");
    let output = command
        .arg(&php_path)
        .output()
        .expect("failed to run elephc --emit-ir");
    assert!(
        output.status.success(),
        "elephc --emit-ir failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("EIR output should be UTF-8");
    let _ = fs::remove_dir_all(&dir);
    text
}
