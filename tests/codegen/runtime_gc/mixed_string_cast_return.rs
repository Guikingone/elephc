//! Purpose:
//! Regression coverage for issue #700: returning the result of a cast that COPIES must hand
//! the caller an owned string it releases, not a borrowed one it leaves behind.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - The leak was in the return-alias summary, not the runtime: `return (string)$v` was read
//!   as "returns parameter 0's storage", so the caller treated the fresh copy as borrowed and
//!   never released it -- one block per call, unbounded in a loop.
//! - EIR lowering elides exactly one cast, `(string)` over a value that is already a `Str`.
//!   That one still aliases its parameter, so the negative guards below matter as much as the
//!   leak fixtures: reporting it as fresh would make the caller release storage it borrowed.
//! - Every fixture asserts both the output and `HEAP DEBUG: leak summary: clean`.

use crate::support::*;

/// Verifies the issue's own reproducer is heap-clean.
#[test]
fn test_mixed_string_cast_return_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function frob($v) { return (string)$v; }
function mk(int $n): string { return str_repeat("a", $n); }
$x = mk(3);
$y = frob($x);
echo $y;
"#,
    );
    assert_eq!(out.stdout, "aaa", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "returning a mixed-to-string cast must not leak the copy: {}",
        out.stderr
    );
}

/// Verifies the same leak through a local, which is how the cast usually reaches a return.
#[test]
fn test_mixed_string_cast_return_through_local_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function frob($v) { $s = (string)$v; return $s; }
$x = str_repeat("a", 3);
echo frob($x);
"#,
    );
    assert_eq!(out.stdout, "aaa", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "a local holding the cast must not leak it either: {}",
        out.stderr
    );
}

/// Verifies the leak does not accumulate, which is what made it more than a nuisance.
#[test]
fn test_mixed_string_cast_return_does_not_accumulate_in_a_loop() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function frob($v) { return (string)$v; }
$x = str_repeat("a", 3);
$n = 0;
for ($i = 0; $i < 50; $i++) { $t = frob($x); $n += strlen($t); }
echo $n;
"#,
    );
    assert_eq!(out.stdout, "150", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "50 calls must not leave 50 blocks behind: {}",
        out.stderr
    );
}

/// Verifies the parameter shapes that are NOT a plain `string`, which the cast also copies.
///
/// `?string` and `string|int` both reach the callee as a boxed `mixed`, so `(string)` over
/// either one allocates exactly as it does for an untyped parameter.
#[test]
fn test_string_cast_return_is_heap_clean_for_every_boxed_parameter_shape() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function fromNullable(?string $v): string { return (string)$v; }
function fromUnion(string|int $v): string { return (string)$v; }
$x = str_repeat("a", 3);
echo fromNullable($x), fromUnion($x);
"#,
    );
    assert_eq!(out.stdout, "aaaaaa", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "a boxed parameter shape must not leak its string copy: {}",
        out.stderr
    );
}

/// Guard: the one cast EIR lowering elides still hands back the caller's own storage.
///
/// `(string)` over a `string` parameter compiles to nothing at all, so the returned pointer IS
/// the argument's. Treating it as fresh would make the caller release a string it only
/// borrowed, and the value would be read after free here.
#[test]
fn test_string_cast_of_a_string_parameter_still_passes_storage_through() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function ident(string $s): string { return (string)$s; }
$x = str_repeat("a", 3);
$y = ident($x);
echo $x, $y, strlen($x), strlen($y);
"#,
    );
    assert_eq!(out.stdout, "aaaaaa33", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "an elided cast must stay a borrow: {}",
        out.stderr
    );
}

/// Guard: the sibling paths that were already clean stay clean.
///
/// Concatenating, echoing or discarding the same cast all release the temporary inside the
/// callee. They are the shapes that proved the runtime was right and the summary was wrong.
#[test]
fn test_sibling_mixed_string_cast_paths_stay_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
function viaConcat($v) { return $v . "!"; }
function viaEcho($v): void { echo (string)$v; }
function viaDiscard($v): void { (string)$v; }
$x = str_repeat("a", 3);
echo viaConcat($x);
viaEcho($x);
viaDiscard($x);
echo "done";
"#,
    );
    assert_eq!(out.stdout, "aaa!aaadone", "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("leak summary: clean"),
        "the already-clean sibling paths must stay clean: {}",
        out.stderr
    );
}
