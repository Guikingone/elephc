//! Purpose:
//! Regression tests for reading back an array whose runtime storage was promoted
//! to a hash at write time while its static type stayed indexed (`Array(Mixed)`)
//! or `Mixed`. Covers a boxed-Mixed key store, a by-reference callee promotion,
//! and a string-key store into a `Mixed`-typed local (the symfony/yaml parser
//! mapping-accumulation shape).
//!
//! Called from:
//! - `cargo test` through the `codegen_tests` harness via `crate::support`.
//!
//! Key details:
//! - A `$arr[$k] = $v` write with a runtime-tagged key (or a string key into an
//!   empty indexed array) promotes the array to a hash. The static type does not
//!   track that promotion, so every read path — `$arr["k"]`, `isset($arr["k"])`,
//!   a boxed-Mixed `$mixed["k"]`, `is_array`, `count` — must dispatch on the live
//!   runtime heap kind rather than the checker's original indexed guess.
//! - Before the fix: the boxed-Mixed key store crashed (indexed `isset` read the
//!   promoted-hash layout as packed slots), and a string-key store into a
//!   `Mixed`-typed local was silently dropped (`__rt_mixed_array_set` rejected
//!   string keys on an indexed payload instead of promoting it).
//! - Outputs are cross-checked against `php -r`.

use crate::support::*;

/// Stores a boxed-Mixed string key into an empty local, then reads it back
/// through `isset`, the `??` reader, and `count`. Previously the `isset` inlined
/// a packed-indexed probe over the promoted hash layout and crashed (SIGSEGV).
#[test]
fn test_mixed_key_store_isset_and_read() {
    let out = compile_and_run(
        r#"<?php
function mkey(): mixed { return "name"; }
$d = [];
$k = mkey();
$d[$k] = "elephc";
echo isset($d["name"]) ? "Y" : "N", "|", $d["name"] ?? "MISS", "|", count($d), "\n";
"#,
    );
    assert_eq!(out, "Y|elephc|1\n");
}

/// Builds a hash through a by-reference `array &$d` parameter with a Mixed value,
/// then returns it via `empty()`/ternary and reads a string key in the caller.
/// Previously the caller read the promoted hash through the indexed path and saw
/// an empty array.
#[test]
fn test_byref_hash_build_return_readback() {
    let out = compile_and_run(
        r#"<?php
function g(array &$d, string $k, mixed $v): void { $d[$k] = $v; }
function h(): mixed { $d = []; g($d, "name", "elephc"); return empty($d) ? null : $d; }
$r = h();
echo is_array($r) ? $r["name"] : "NULL", "\n";
"#,
    );
    assert_eq!(out, "elephc\n");
}

/// Stores a string key into a `Mixed`-typed local that wraps an empty indexed
/// array, then returns it via the short-ternary and reads it back. This is the
/// symfony/yaml `$data[$key] = $value` mapping shape; previously the write was
/// silently dropped and the value came back as a non-array with count 0.
#[test]
fn test_string_key_store_into_mixed_local() {
    let out = compile_and_run(
        r#"<?php
function emptyArr(): mixed { return []; }
function build(string $key): mixed {
    $data = emptyArr();
    $data[$key] = "elephc";
    return $data ?: null;
}
$r = build("name");
echo (is_array($r) ? "arr" : "non"), "|", count(is_array($r) ? $r : []), "|", ($r["name"] ?? "MISS"), "\n";
"#,
    );
    assert_eq!(out, "arr|1|elephc\n");
}

/// Accumulates several boxed-Mixed string-key entries into one array, then reads
/// each back and probes `isset` for a present and a missing key. Confirms the
/// promoted hash keeps every entry and that a missing-key `isset` stays false
/// instead of misreading a packed slot.
#[test]
fn test_multi_entry_mixed_key_readback() {
    let out = compile_and_run(
        r#"<?php
function k(string $s): mixed { return $s; }
$d = [];
$d[k("a")] = "1";
$d[k("b")] = "2";
echo $d["a"], $d["b"], "|", (isset($d["a"]) ? "Y" : "N"), (isset($d["z"]) ? "Y" : "N"), "\n";
"#,
    );
    assert_eq!(out, "12|YN\n");
}
