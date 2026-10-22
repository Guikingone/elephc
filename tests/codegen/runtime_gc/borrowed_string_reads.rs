//! Purpose:
//! Exercises read-only builtin string operands and their aliasing fallbacks.
//!
//! Called from:
//! - The codegen integration suite through `runtime_gc`.
//!
//! Key details:
//! - Large subjects expose whole-string copies; heap poisoning verifies result lifetimes.

use crate::support::compile_and_run_with_heap_debug;

/// Later reference promotion retains ordinary unwind roots for a widened local read.
#[test]
fn test_borrowed_string_reads_later_boxed_local() {
    let out = compile_and_run_with_heap_debug(r#"<?php
function boxReadSubject(&$s): void { $s = $s; }
function run(): void {
    $s = str_repeat("ab", 2000);
    for ($i = 0; $i < 5; $i++) {
        try { strpos($s, "b", 5000); } catch (ValueError $e) { echo "caught|"; }
    }
    boxReadSubject($s);
    echo strlen($s), "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "caught|caught|caught|caught|caught|4000\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Local and property reads preserve bytes while only the requested slice becomes owned.
#[test]
fn test_borrowed_string_reads_local_and_property() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ReadSubject {
    public $bytes;
    public function __construct(string $s) { $this->bytes = $s; }
    public function __get(string $name): string { return "fallback"; }
    public function byteAt(int $i): string { return substr($this->bytes, $i, 1); }
    public function findAt(int $i): int { return (int)strpos($this->bytes, "b", $i); }
}
class TypedReadSubject {
    public string $bytes;
    public function __construct(string $s) { $this->bytes = $s; }
    public function byteAt(int $i): string { return substr($this->bytes, $i, 1); }
}
function run(): void {
    $s = str_repeat("ab", 200000);
    $r = new ReadSubject($s);
    $typed = new TypedReadSubject($s);
    $sum = 0;
    for ($i = 0; $i < 40; $i++) {
        $sum += ord(substr($s, $i, 1)) + ord($r->byteAt($i)) + ord($typed->byteAt($i));
    }
    $last = -1;
    echo $sum, "|", $r->findAt(2), "|", \SUBSTR(length: 1, string: $s, offset: $last), "\n";
    $byte = $r->byteAt(0);
    unset($r);
    echo $byte, "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "11700|3|b\na\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// A later mutating argument still snapshots the original string in PHP evaluation order.
#[test]
fn test_borrowed_string_reads_mutating_argument_fallback() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class MutableReadSubject {
    public string $bytes;
    public function __construct(string $s) { $this->bytes = $s; }
}
function replaceSubject(&$s): int { $s = str_repeat("z", 4000); return 1; }
function replaceProperty(MutableReadSubject $r): int { $r->bytes = str_repeat("z", 4000); return 1; }
function run(): void {
    $s = str_repeat("ab", 2000);
    $r = new MutableReadSubject($s);
    echo substr((string)$s, replaceSubject($s), 1), "|", substr($r->bytes, replaceProperty($r), 1), "\n";
    echo strpos(offset: replaceSubject($s), haystack: $s, needle: "z"), "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "b|b\n1\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// Throwing searches retire their receiver lease and preserve the caller's subject owner.
#[test]
fn test_borrowed_string_reads_throwing_search() {
    let out = compile_and_run_with_heap_debug(r#"<?php
class ThrowReadSubject {
    public string $bytes;
    public function __construct(string $s) { $this->bytes = $s; }
    public function find(int $i): void { strpos($this->bytes, "b", $i); }
}
function run(): void {
    $r = new ThrowReadSubject(str_repeat("ab", 2000));
    for ($i = 0; $i < 5; $i++) {
        try { $r->find(5000); } catch (ValueError $e) { echo "caught|"; }
    }
    echo strlen($r->bytes), "\n";
}
run();
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "caught|caught|caught|caught|caught|4000\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
