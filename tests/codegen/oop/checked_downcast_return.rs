//! Purpose:
//! Integration and regression tests for checked-downcast-on-return: a function/method may
//! declare a return type narrower than a value it's statically only known to be a SUPERTYPE
//! of (base→derived), accepted ONLY because a runtime `instanceof` guard is emitted at the
//! return boundary — matching a match on the fly and throwing a catchable `\TypeError` (naming
//! the ACTUAL runtime class) on mismatch, exactly like PHP's own return-type enforcement.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Covers interface-declared, union-declared, and nullable declared return shapes, a
//!   dynamically-dispatched (`new $class(...)`) factory chain, and `return $this` (which must
//!   emit NO guard — proven-safe by construction).
//! - Proven-subtype and `return $this` no-guard claims are verified textually via `--emit-ir`
//!   (`emit_ir`/`function_ir` helpers below), not just by running the program, since a spurious
//!   guard would still produce correct output but violate the zero-cost-when-proven design goal.

use super::*;
use std::fs;
use std::process::Command;

/// Emits textual EIR for a source snippet through the CLI (`--emit-ir`, optimizer on).
fn emit_ir(source: &str) -> String {
    let dir = make_cli_test_dir("elephc_checked_downcast_return_emit_ir");
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

/// Extracts the textual EIR for one function from a printed module.
fn function_ir<'a>(module: &'a str, signature_prefix: &str) -> &'a str {
    let marker = format!("  function {}", signature_prefix);
    let start = module
        .find(&marker)
        .unwrap_or_else(|| panic!("function `{}` not found in emitted IR:\n{}", signature_prefix, module));
    let rest = &module[start..];
    let end = rest.find("\n  }").map(|idx| idx + 4).unwrap_or(rest.len());
    &rest[..end]
}

/// (a) Factory shape end-to-end: the value IS the declared subtype at runtime. Methods only
/// declared on the derived class are callable on the guarded return, proving the checker
/// threads the precise (derived) type through to the caller.
#[test]
fn test_checked_downcast_return_factory_matches_declared_subtype_runs() {
    let out = compile_and_run(
        r#"<?php
class NodeDefinition {}
class ScalarNodeDefinition extends NodeDefinition {
    public function scalarOnly(): string { return "scalar-only"; }
}
class NodeBuilder {
    public function node(string $type): NodeDefinition {
        if ($type === 'scalar') {
            return new ScalarNodeDefinition();
        }
        return new NodeDefinition();
    }
    public function scalarNode(): ScalarNodeDefinition {
        return $this->node('scalar');
    }
}
$b = new NodeBuilder();
$n = $b->scalarNode();
echo get_class($n), " ", $n->scalarOnly();
"#,
    );
    assert_eq!(out, "ScalarNodeDefinition scalar-only");
}

/// (b) NEGATIVE: the value is a base-but-not-derived instance. The guard must throw a
/// catchable `\TypeError`, never silently accept or crash, with PHP's exact message shape
/// (php -n verified: `"f(): Return value must be of type D, B returned"`).
#[test]
fn test_checked_downcast_return_mismatch_throws_catchable_type_error() {
    let out = compile_and_run(
        r#"<?php
class B {}
class D extends B {}
function makeD(bool $wantD): D {
    if ($wantD) {
        return new D();
    }
    return new B();
}
try {
    makeD(false);
    echo "no-throw";
} catch (\TypeError $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(out, "makeD(): Return value must be of type D, B returned");
}

/// The matching branch of the SAME guarded function must still run PHP-identically (both
/// branches of one guarded function, not just two separate functions).
#[test]
fn test_checked_downcast_return_matching_branch_of_guarded_function_runs() {
    let out = compile_and_run(
        r#"<?php
class B {}
class D extends B {}
function makeD(bool $wantD): D {
    if ($wantD) {
        return new D();
    }
    return new B();
}
echo get_class(makeD(true));
"#,
    );
    assert_eq!(out, "D");
}

/// (c) Interface-declared return: the declared type is an interface the derived class
/// implements (through its base class), not a concrete class.
#[test]
fn test_checked_downcast_return_interface_declared_matches() {
    let out = compile_and_run(
        r#"<?php
interface NodeParentInterface {}
class NodeDefinition implements NodeParentInterface {}
class ScalarNodeDefinition extends NodeDefinition {}
class NodeBuilder {
    public function node(string $type): NodeDefinition {
        if ($type === 'scalar') {
            return new ScalarNodeDefinition();
        }
        return new NodeDefinition();
    }
    // Declared as the INTERFACE, not NodeDefinition: node()'s static return type
    // (NodeDefinition) is a proper ancestor of ScalarNodeDefinition, so this needs the
    // same base->derived guard even though the declared type is an interface.
    public function asParent(bool $wantScalar): NodeParentInterface {
        return $this->node($wantScalar ? 'scalar' : 'plain');
    }
}
$b = new NodeBuilder();
echo get_class($b->asParent(true)), " ", get_class($b->asParent(false));
"#,
    );
    assert_eq!(out, "ScalarNodeDefinition NodeDefinition");
}

/// (d) Chain NodeBuilder-like probe: a dynamically string-dispatched `new $class(...)` inside
/// one method feeds a base-declared factory, and TWO sibling methods each declare a more
/// specific derived return type over that same factory — exactly the Symfony
/// `NodeBuilder::scalarNode()`/`booleanNode()` shape.
#[test]
fn test_checked_downcast_return_dynamic_new_chain_runs() {
    let out = compile_and_run(
        r#"<?php
class NodeDefinition {
    public function tag(): string { return "base"; }
}
class ScalarNodeDefinition extends NodeDefinition {
    public function tag(): string { return "scalar"; }
}
class BooleanNodeDefinition extends NodeDefinition {
    public function tag(): string { return "boolean"; }
}
class NodeBuilder {
    public function node(string $type): NodeDefinition {
        $class = $type === 'boolean' ? BooleanNodeDefinition::class : ScalarNodeDefinition::class;
        $instance = new $class();
        return $instance;
    }
    public function scalarNode(): ScalarNodeDefinition {
        return $this->node('scalar');
    }
    public function booleanNode(): BooleanNodeDefinition {
        return $this->node('boolean');
    }
}
$b = new NodeBuilder();
echo get_class($b->scalarNode()), " ", $b->scalarNode()->tag(), " ";
echo get_class($b->booleanNode()), " ", $b->booleanNode()->tag();
"#,
    );
    assert_eq!(out, "ScalarNodeDefinition scalar BooleanNodeDefinition boolean");
}

/// Union-declared return (multi-arm guard): the object arm is checked with the same guard;
/// the scalar arm of the union is untouched (no object type at all reaches the guard).
#[test]
fn test_checked_downcast_return_union_declared_object_and_scalar_arms() {
    let out = compile_and_run(
        r#"<?php
class B {}
class D extends B {}
function make(bool $wantObject): D|int {
    if (!$wantObject) {
        return 42;
    }
    return new D();
}
echo get_class(make(true)), " ";
var_dump(make(false));
"#,
    );
    assert_eq!(out, "D int(42)\n");
}

/// Union-declared return where the object arm mismatches: the guard still fires and throws.
#[test]
fn test_checked_downcast_return_union_declared_object_arm_mismatch_throws() {
    let out = compile_and_run(
        r#"<?php
class B {}
class D extends B {}
function make(bool $wantMismatch): D|int {
    if ($wantMismatch) {
        return new B();
    }
    return 7;
}
try {
    make(true);
    echo "no-throw";
} catch (\TypeError $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(out, "make(): Return value must be of type D|int, B returned");
}

/// Nullable declared return (`?D`): the object arm is guarded; the `null` arm passes through
/// the existing nullable-return handling untouched, matching PHP's own `?D` message rendering
/// (php -n verified: `"f(): Return value must be of type ?D, B returned"`).
#[test]
fn test_checked_downcast_return_nullable_object_and_null_arms() {
    let out = compile_and_run(
        r#"<?php
class B {}
class D extends B {}
function make(int $mode): ?D {
    if ($mode === 0) {
        return null;
    }
    if ($mode === 1) {
        return new D();
    }
    return new B();
}
var_dump(make(0));
echo get_class(make(1)), " ";
try {
    make(2);
    echo "no-throw";
} catch (\TypeError $e) {
    echo $e->getMessage();
}
"#,
    );
    assert_eq!(
        out,
        "NULL\nD make(): Return value must be of type ?D, B returned"
    );
}

/// `return $this` shapes never need a guard: inside a method, `$this`'s static type IS the
/// declaring class, so a `self`/`static`-declared (or any ancestor-declared) return is always
/// already proven safe. Functional proof; the zero-guard claim is verified separately via
/// `--emit-ir` below.
#[test]
fn test_checked_downcast_return_this_shape_runs() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function make(): static {
        return $this;
    }
}
class Derived extends Base {}
$d = new Derived();
echo get_class($d->make());
"#,
    );
    assert_eq!(out, "Derived");
}

/// Verifies via `--emit-ir` that `return $this` emits NO `instance_of`/
/// `throw_checked_return_type_error` ops — the guard must be entirely absent (zero cost), not
/// just harmless, for a return that's already proven safe by construction.
#[test]
fn test_checked_downcast_return_this_shape_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class Base {
    public function make(): static {
        return $this;
    }
}
$b = new Base();
echo get_class($b->make());
"#,
    );
    let function = function_ir(&module, "Base::make(");
    assert!(
        !function.contains("instance_of") && !function.contains("throw_checked_return_type_error"),
        "return $this must never emit a downcast guard:\n{}",
        function
    );
}

/// Verifies via `--emit-ir` that an ordinary already-provably-safe covariant return (the
/// value's static type already IS a declared arm) emits NO guard — the base→derived guard
/// is strictly additive over existing covariant returns, never re-checking what's already sound.
#[test]
fn test_checked_downcast_return_proven_subtype_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class NodeDefinition {}
class ScalarNodeDefinition extends NodeDefinition {}
class NodeBuilder {
    public function node(string $type): NodeDefinition {
        if ($type === 'scalar') {
            return new ScalarNodeDefinition();
        }
        return new NodeDefinition();
    }
}
$b = new NodeBuilder();
echo get_class($b->node('scalar'));
"#,
    );
    let function = function_ir(&module, "NodeBuilder::node(");
    assert!(
        !function.contains("instance_of") && !function.contains("throw_checked_return_type_error"),
        "a value that's already a proven subtype of the declared return must emit no guard:\n{}",
        function
    );
}

/// Verifies via `--emit-ir` that the base→derived relaxation DOES emit a guard chain (sanity
/// check paired with the two "no guard" tests above, so an accidental always-skip regression
/// in the "proven safe" fast path would be caught here).
#[test]
fn test_checked_downcast_return_relaxed_case_emits_guard() {
    let module = emit_ir(
        r#"<?php
class B {}
class D extends B {}
function makeD(bool $wantD): D {
    if ($wantD) {
        return new D();
    }
    return new B();
}
echo get_class(makeD(true));
"#,
    );
    let function = function_ir(&module, "makeD(");
    assert!(function.contains("instance_of"), "expected an instance_of guard:\n{}", function);
    assert!(
        function.contains("throw_checked_return_type_error"),
        "expected a throw_checked_return_type_error fail path:\n{}",
        function
    );
}

/// Heap-safety: the matching guard branch must be entirely leak-free — the mismatched-but-
/// proven-safe value flows straight through to the caller with normal ownership.
#[test]
fn test_checked_downcast_return_match_branch_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class B {}
class D extends B {}
function makeD(bool $wantD): D {
    if ($wantD) {
        return new D();
    }
    return new B();
}
echo get_class(makeD(true));
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "{}",
        out.stderr
    );
}

/// Heap-safety: the guard's throw (fail) branch must release the mismatched object it
/// inspected and discarded — it is never returned to the caller, so the guard is its only
/// owner. Compares the leaked BLOCK COUNT against an equivalent plain `throw new
/// \TypeError($dynamicallyBuiltMessage)` + `catch`-and-never-read control (the message must
/// be dynamically built in the control too — an apples-to-apples comparison, since a
/// compile-time-literal message needs no heap allocation at all and would under-count).
///
/// The control throws its `TypeError` from a single inline expression
/// (`throw new \TypeError("..." . (...))`), matching the guard's own message synthesis,
/// which is likewise built and passed to the thrown `TypeError` in one step with no
/// PHP-visible intermediate local. An EARLIER version of this control instead assigned the
/// concatenation to a `$msg` local before throwing it (`$msg = "..." . (...); throw new
/// \TypeError($msg);`); `php -n`-equivalent semantics are identical, but under
/// `--heap-debug` that extra local added ONE MORE leaked block than the guard could ever
/// produce: elephc has a PRE-EXISTING, orthogonal gap where a live local's heap-allocated
/// value is not released when a `throw` unwinds past its scope before the local's normal
/// release point runs (see the `Throwable message ownership` / unwind-leak project notes) —
/// unrelated to this guard, since the guard's mismatch branch never materializes an
/// intermediate local for its message. Comparing against that local-carrying control made
/// the assertion require the guard to ALSO leak a block it structurally cannot leak, which
/// is why this test previously failed: the guard was (correctly) fully clean while the
/// control (incidentally) was not. With the control's intermediate local removed, both
/// scenarios legitimately reach `live_blocks=0`; what this test guards against is the guard
/// leaking the DISCARDED MISMATCHED OBJECT (`new B()`) on top of whatever the control leaks,
/// so the two must still match if a future change reintroduces a leak on either side.
#[test]
fn test_checked_downcast_return_mismatch_branch_leaks_no_more_blocks_than_baseline_throw() {
    let guard_mismatch = compile_and_run_with_heap_debug(
        r#"<?php
class B {}
class D extends B {}
function makeD(bool $wantD): D {
    if ($wantD) {
        return new D();
    }
    return new B();
}
try {
    makeD(false);
} catch (\TypeError $e) {
    echo "caught";
}
"#,
    );
    let baseline_throw = compile_and_run_with_heap_debug(
        r#"<?php
function f(bool $x): void {
    throw new \TypeError("some message here " . ($x ? "yes" : "no"));
}
try {
    f(true);
} catch (\TypeError $e) {
    echo "caught";
}
"#,
    );
    assert!(guard_mismatch.success, "program failed: {}", guard_mismatch.stderr);
    assert!(baseline_throw.success, "program failed: {}", baseline_throw.stderr);
    assert_eq!(
        live_blocks(&guard_mismatch.stderr),
        live_blocks(&baseline_throw.stderr),
        "guard-mismatch leaked block count ({}) must match the pre-existing caught-exception \
         baseline ({}) — the mismatched object must be released, not added on top",
        guard_mismatch.stderr,
        baseline_throw.stderr
    );
}

/// Parses the `live_blocks=N` figure out of a `--heap-debug` leak-summary line; a `clean`
/// summary (no `live_blocks=` field at all) counts as zero.
fn live_blocks(stderr: &str) -> u64 {
    let marker = "leak summary:";
    let start = stderr.find(marker).unwrap_or_else(|| panic!("no leak summary in: {}", stderr));
    let tail = &stderr[start..];
    let key = "live_blocks=";
    let Some(key_start) = tail.find(key) else {
        return 0;
    };
    let digits_start = key_start + key.len();
    let digits: String = tail[digits_start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().unwrap_or_else(|_| panic!("bad live_blocks digits in: {}", tail))
}
