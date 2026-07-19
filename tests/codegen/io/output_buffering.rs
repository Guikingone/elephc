//! Purpose:
//! Integration tests for output buffering (`ob_start()`/`ob_get_clean()`/
//! `ob_end_flush()`/`ob_end_clean()`/`ob_get_contents()`/`ob_get_level()`/
//! `ob_get_status()`), `headers_sent()`, `flush()`, `header_remove()`, and
//! `get_class_methods()`.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Assertions are php-identical (php -n verified against PHP 8.5): nesting,
//!   discard-vs-flush semantics, empty-stack notices, and buffered output
//!   being swallowed by a later discard are all exercised.
//! - `get_class_methods()` covers both calling-scope forms this branch
//!   supports: public-only from outside a class, and public+protected+own-
//!   private from inside a matching method.
//! - `printf`/`vprintf`, and the array/hash/Mixed-value portions of
//!   `var_dump`/`print_r`, write via raw syscalls that bypass the `ob_start()`
//!   buffer (php -n verified: real PHP DOES capture all of this — elephc's
//!   choke point is `__rt_stdout_write`, which those paths never call). This
//!   is a documented supported-subset divergence, enforced LOUD: a runtime
//!   fatal, never a silent write outside the active buffer.

use super::*;

/// Verifies the basic `ob_start()`/`ob_get_clean()` round trip: captured
/// output never reaches real stdout, and the returned string is exact.
#[test]
fn test_ob_start_get_clean_basic() {
    let out = compile_and_run(
        "<?php echo \"before\\n\"; ob_start(); echo \"captured\"; $c = ob_get_clean(); var_dump($c); echo \"after\\n\";",
    );
    assert_eq!(out, "before\nstring(8) \"captured\"\nafter\n");
}

/// Verifies nested `ob_start()` write-through: `ob_end_flush()` on the inner
/// buffer lands in the OUTER buffer (not real stdout), matching PHP's nested
/// buffering semantics exactly.
#[test]
fn test_ob_nesting_end_flush_writes_through() {
    let out = compile_and_run(
        "<?php ob_start(); echo \"outer-\"; ob_start(); echo \"inner\"; ob_end_flush(); echo \"-tail\"; $outer = ob_get_clean(); var_dump($outer);",
    );
    assert_eq!(out, "string(16) \"outer-inner-tail\"\n");
}

/// Verifies `ob_end_clean()` discards buffered content without ever emitting
/// it, and returns `true`.
#[test]
fn test_ob_end_clean_discards() {
    let out = compile_and_run(
        "<?php ob_start(); echo \"discard-me\"; var_dump(ob_end_clean()); echo \"after\\n\";",
    );
    assert_eq!(out, "bool(true)\nafter\n");
}

/// Verifies `ob_get_contents()` peeks without popping: the buffer is still
/// active (and its content still capturable) after the call.
#[test]
fn test_ob_get_contents_does_not_pop() {
    let out = compile_and_run(
        "<?php ob_start(); echo \"x\"; $peek = ob_get_contents(); $clean = ob_get_clean(); var_dump($peek); var_dump($clean);",
    );
    assert_eq!(out, "string(1) \"x\"\nstring(1) \"x\"\n");
}

/// Verifies `ob_get_level()` reflects the current nesting depth, and that
/// output produced WHILE a buffer is active is swallowed if that buffer is
/// later discarded (php -n verified: this is exactly PHP's own behavior, not
/// an elephc quirk — a buffered `var_dump()` call is never a real echo until
/// its buffer is flushed).
#[test]
fn test_ob_get_level_and_buffered_output_swallowed_by_discard() {
    let out = compile_and_run(
        "<?php var_dump(ob_get_level()); ob_start(); var_dump(ob_get_level()); ob_end_clean(); var_dump(ob_get_level());",
    );
    assert_eq!(out, "int(0)\nint(0)\n");
}

/// Verifies `ob_end_clean()`/`ob_end_flush()` on an empty stack both return
/// `false` (php -n verified; the stderr notice text is asserted separately
/// against the runtime message constants, not stdout here).
#[test]
fn test_ob_end_on_empty_stack_returns_false() {
    let out = compile_and_run(
        "<?php var_dump(ob_end_clean()); var_dump(ob_end_flush()); var_dump(ob_get_contents()); var_dump(ob_get_clean());",
    );
    assert_eq!(out, "bool(false)\nbool(false)\nbool(false)\nbool(false)\n");
}

/// Verifies `ob_get_status()` on an empty stack returns an empty array, and a
/// non-empty stack reports the expected fields (php -n verified shape for a
/// plain, callback-free buffer: name/type/flags/level/chunk_size/buffer_used;
/// `buffer_size` is elephc's own real fixed capacity, a disclosed difference
/// from PHP's internal growable-chunk default — see the runtime doc comment).
#[test]
fn test_ob_get_status_shape() {
    let out = compile_and_run(
        r#"<?php
$empty = ob_get_status();
var_dump(count($empty));
ob_start();
echo "hello";
$s = ob_get_status();
$name = $s['name']; $type = $s['type']; $flags = $s['flags'];
$level = $s['level']; $chunk_size = $s['chunk_size']; $buffer_used = $s['buffer_used'];
ob_end_clean();
var_dump($name);
var_dump($type);
var_dump($flags);
var_dump($level);
var_dump($chunk_size);
var_dump($buffer_used);
"#,
    );
    assert_eq!(
        out,
        "int(0)\nstring(22) \"default output handler\"\nint(0)\nint(112)\nint(0)\nint(0)\nint(5)\n"
    );
}

/// Verifies `ob_start()` case-insensitively (PHP builtins are case-insensitive).
#[test]
fn test_ob_start_case_insensitive() {
    let out = compile_and_run("<?php OB_START(); echo \"x\"; var_dump(Ob_Get_Clean());");
    assert_eq!(out, "string(1) \"x\"\n");
}

/// Verifies `headers_sent()` reflects REAL (non-buffered) output — false
/// before any output occurs, with `$file`/`$line` overwritten to `""`/`0`
/// even on the `false` branch (php -n verified: PHP always writes these
/// out-params, contrary to a naive "untouched unless true" assumption) —
/// and stays true once real output has occurred, even while a LATER
/// `ob_start()` is active (headers_sent() reports whether output has EVER
/// left the buffer stack, not the current buffering state).
#[test]
fn test_headers_sent_reflects_real_output_and_writes_by_ref_args() {
    let out = compile_and_run(
        r#"<?php
$file = "SENTINEL"; $line = 424242;
var_dump(headers_sent($file, $line));
var_dump($file);
var_dump($line);
ob_start();
echo "buffered, not real yet";
$sent_while_buffered = headers_sent();
ob_end_clean();
var_dump($sent_while_buffered);
echo "real output";
var_dump(headers_sent());
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nstring(0) \"\"\nint(0)\nbool(true)\nreal outputbool(true)\n"
    );
}

/// Verifies `flush()` is a sound void no-op: it changes nothing observable.
#[test]
fn test_flush_is_void_noop() {
    let out = compile_and_run("<?php echo \"a\"; var_dump(flush()); echo \"b\";");
    assert_eq!(out, "aNULL\nb");
}

/// Verifies `get_class_methods()` from OUTSIDE any class: public-only,
/// declaration order (own class first, then inherited, skipping an
/// already-listed override), matching php -n exactly.
#[test]
fn test_get_class_methods_outside_public_only_decl_order() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function pubBase() {}
    protected function protBase() {}
    private function privBase() {}
    public static function statBase() {}
}
class Child extends Base {
    public function pubChild() {}
    private function privChild() {}
    public function pubBase() {}
}
$methods = get_class_methods(new Child());
foreach ($methods as $m) { echo $m . "\n"; }
echo "---\n";
$methods2 = get_class_methods('Child');
foreach ($methods2 as $m) { echo $m . "\n"; }
"#,
    );
    assert_eq!(
        out,
        "pubChild\npubBase\nstatBase\n---\npubChild\npubBase\nstatBase\n"
    );
}

/// Verifies `ob_get_status(true)` (full output-buffer stack status) is kept
/// loud rather than silently returning only the current level: a disclosed
/// residual, not an accept-and-ignore. Rejected during EIR lowering (not the
/// checker), so the assertion is on the backend's own diagnostic text.
#[test]
#[should_panic(expected = "ob_get_status(true): full output-buffer stack status is not supported")]
fn test_ob_get_status_true_stays_loud() {
    compile_and_run("<?php var_dump(ob_get_status(true));");
}

/// Verifies `get_class_methods()` from INSIDE a method of the SAME class:
/// public + protected + own-private are visible, but an ANCESTOR's private
/// method stays excluded — matching php -n exactly.
#[test]
fn test_get_class_methods_self_scope_visibility() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function pubBase() {}
    protected function protBase() {}
    private function privBase() {}
}
class Inside extends Base {
    public function pubIn() {}
    private function privIn() {}
    function test() {
        $r = get_class_methods($this);
        foreach ($r as $m) { echo $m . "\n"; }
    }
}
(new Inside())->test();
"#,
    );
    assert_eq!(out, "pubIn\nprivIn\ntest\npubBase\nprotBase\n");
}

/// Verifies `var_dump()` of an array INSIDE an active `ob_start()` buffer is
/// a loud runtime fatal, not a silent write outside the buffer: the array/hash
/// walkers (`__rt_var_dump_array_*`/`__rt_var_dump_hash`) write via raw
/// syscalls that bypass elephc's `ob_start()` choke point (`__rt_stdout_write`)
/// — real PHP DOES buffer this (php -n verified: `ob_start(); var_dump([1]);`
/// captures `array(1) { [0]=> int(1) }` into the buffer with no error), so
/// this is a disclosed supported-subset divergence, never a silent gap.
#[test]
fn test_var_dump_array_inside_ob_start_is_loud() {
    let err = compile_and_run_expect_failure("<?php ob_start(); var_dump([1, 2, 3]);");
    assert!(
        err.contains("Fatal error: var_dump(): array/hash contents inside an active output buffer are not supported"),
        "unexpected error: {err}"
    );
}

/// Verifies `printf()` INSIDE an active `ob_start()` buffer is a loud runtime
/// fatal (the formatted write is a raw syscall that bypasses the buffer —
/// real PHP DOES buffer `printf()` output; see the file-level doc comment).
#[test]
fn test_printf_inside_ob_start_is_loud() {
    let err = compile_and_run_expect_failure(r#"<?php ob_start(); printf("hi %d", 5);"#);
    assert!(
        err.contains("Fatal error: printf() inside an active output buffer is not supported"),
        "unexpected error: {err}"
    );
}

/// Verifies `var_dump()` of an array OUTSIDE any output buffer is completely
/// unaffected by the guard added for the buffered case above (the guard's
/// "no buffer active" path is a plain fall-through, zero-cost, and must never
/// change observable output).
#[test]
fn test_var_dump_array_outside_ob_start_unaffected() {
    let out = compile_and_run("<?php var_dump([1, 2, 3]);");
    assert_eq!(
        out,
        "array(3) {\n  [0]=>\n  int(1)\n  [1]=>\n  int(2)\n  [2]=>\n  int(3)\n}\n"
    );
}
