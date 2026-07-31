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

/// A bare `object` return type accepts EVERY object and must take the UNGUARDED fast path.
///
/// The checker models `object` as `PhpType::Object("")` — an empty class name, not a class called
/// `""`. Before the fix, `declared_object_arms` handed that empty name to `emit_guard_chain`, which
/// emitted `Op::InstanceOf` against class `""`; no runtime class can satisfy it, so every such
/// return compiled and then died with `TypeError: mk(): Return value must be of type , A returned`.
/// Now the guard is skipped entirely and the program prints exactly what `php -n` prints.
#[test]
fn test_bare_object_return_type_is_unguarded() {
    let out = compile_and_run(
        r#"<?php
class A { public int $v = 7; }
class B { public string $s = "b"; }
function mk(): object { return new A(); }
function pick(bool $flag): object { return $flag ? new A() : new B(); }
$a = mk();
echo get_class($a), "|", $a->v, "|";
echo get_class(pick(true)), "|", get_class(pick(false));
"#,
    );
    assert_eq!(out, "A|7|A|B");
}

/// A `?object` return (the `object|null` union shape) is likewise unguarded on BOTH arms: the bare
/// `object` member accepts any object, and `null` passes through untouched.
#[test]
fn test_nullable_bare_object_return_type_is_unguarded() {
    let out = compile_and_run(
        r#"<?php
class A {}
function maybe(bool $flag): ?object { return $flag ? new A() : null; }
var_dump(get_class(maybe(true)));
var_dump(maybe(false));
"#,
    );
    assert_eq!(out, "string(1) \"A\"\nNULL\n");
}

/// ZERO-DELTA PIN for the extraction of the guard chain into
/// `crate::ir_lower::checked_downcast`: the return position's emitted shape is asserted
/// STRUCTURALLY, op by op, not just by its observable behaviour.
///
/// The guard's block names, the `instance_of` test, the `cond_br` into `return_type_guard.ok`,
/// and the RELEASING `throw_checked_return_type_error` in `return_type_guard.fail` are all part
/// of the contract the shared emitter inherited. A relocation of this chain that changed any of
/// them — a renamed block, a reordered test, a swapped throw op (the non-releasing
/// `throw_checked_type_error` here would be a DOUBLE FREE the return path cannot survive) —
/// still passes every behavioural test above, so only this one catches it.
#[test]
fn test_checked_downcast_return_guard_emits_the_pinned_op_sequence() {
    let module = emit_ir(
        r#"<?php
class A {}
class B extends A { public int $n = 1; }
function mk(): A { return new B(); }
function r(): B { $a = mk(); return $a; }
echo r()->n;
"#,
    );
    let guarded = function_ir(&module, "r()");
    let ops: Vec<&str> = guarded
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("return_type_guard")
                || line.contains("instance_of")
                || line.starts_with("cond_br")
                || line.contains("throw_checked")
                || *line == "unreachable"
        })
        .collect();
    assert_eq!(
        ops.len(),
        6,
        "unexpected guard shape in:\n{}",
        guarded
    );
    assert!(ops[0].contains("instance_of"), "guard must test instance_of first: {:?}", ops);
    assert!(ops[1].starts_with("cond_br"), "guard must branch on the test: {:?}", ops);
    assert_eq!(ops[2], "return_type_guard.ok:", "ok block name is part of the contract: {:?}", ops);
    assert_eq!(ops[3], "return_type_guard.fail:", "fail block name is part of the contract: {:?}", ops);
    assert!(
        ops[4].starts_with("throw_checked_return_type_error"),
        "the return position MUST use the releasing throw op: {:?}",
        ops
    );
    assert_eq!(ops[5], "unreachable", "the fail block must not fall through: {:?}", ops);
}

// ---------------------------------------------------------------------------------------------
// BOXED (union) sources at the return position.
//
// A union's codegen representation is a boxed `Mixed`, so these are the shapes whose throw path
// cannot use `get_class` (there is no object header to read) and cannot release the value as an
// object (the refcount word it would decrement is the cell's TAG). Both are resolved from the
// operand's own representation in `crate::codegen::lower_inst::objects::return_type_guard`.
//
// Every fixture here keeps a DECLARED union return on the source function. That is load-bearing:
// an UNDECLARED union return widens to `Mixed` and takes the gradual path instead, so a fixture
// without one passes at the baseline too and proves nothing.
// ---------------------------------------------------------------------------------------------

/// A boxed union source flowing into a CONCRETE declared class return (`BoxedToRawObject`): the
/// matching arm is unboxed and usable as the derived type, and the mismatching arm throws naming
/// its ACTUAL runtime class.
///
/// `Coll` and `Route` are given DIFFERENT LAYOUTS on purpose — a wrong-representation read of the
/// box would print visibly wrong values rather than something plausible.
#[test]
fn test_checked_downcast_return_boxed_union_into_concrete_class() {
    let out = compile_and_run(
        r#"<?php
class Coll { public int $a = 111; public int $b = 222; public int $c = 333; }
class Route { public string $name = "route-name"; public int $port = 8080; }
function pick(int $k): Coll|Route { return $k === 0 ? new Coll() : new Route(); }
function want(int $k): Route { return pick($k); }
$r = want(1);
echo $r->name, "|", $r->port, "|";
try { want(0); echo "NO THROW"; } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "route-name|8080|want(): Return value must be of type Route, Coll returned"
    );
}

/// A boxed union source flowing into a NULLABLE declared return (`BoxedToBoxed`): the null arm
/// passes through the Phase-1 tag test AS NULL, the class arm passes the `instanceof`, and the
/// unrelated arm throws against the `?Route` spelling PHP itself renders.
#[test]
fn test_checked_downcast_return_boxed_union_into_nullable_class() {
    let out = compile_and_run(
        r#"<?php
class Coll { public int $a = 111; public int $b = 222; public int $c = 333; }
class Route { public string $name = "route-name"; public int $port = 8080; }
function pick(int $k): Coll|Route|null {
    if ($k === 0) { return new Coll(); }
    if ($k === 1) { return new Route(); }
    return null;
}
function want(int $k): ?Route { return pick($k); }
echo want(1)->name, "|";
var_dump(want(2) === null);
try { want(0); echo "NO THROW"; } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "route-name|bool(true)\nwant(): Return value must be of type ?Route, Coll returned"
    );
}

/// A NULL payload reaching a NON-nullable declared object return must be named `null`, not
/// misreported through a class lookup. There is no Phase-1 null arm to catch it (the declaration
/// has none), so it falls through the `instanceof` into the throw — which is exactly where a
/// `get_class` on a boxed cell would have read a tag word as an object header.
#[test]
fn test_checked_downcast_return_boxed_null_is_named_null() {
    let out = compile_and_run(
        r#"<?php
class Coll { public int $a = 111; public int $b = 222; public int $c = 333; }
class Route { public string $name = "route-name"; public int $port = 8080; }
function pick(int $k): Coll|Route|null {
    if ($k === 0) { return new Coll(); }
    if ($k === 1) { return new Route(); }
    return null;
}
function want(int $k): Route { return pick($k); }
echo want(1)->port, "|";
try { want(2); echo "NO THROW"; } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "8080|want(): Return value must be of type Route, null returned"
    );
}

/// The `?object` source shape (a bare `object` arm plus null, boxed together): every object is a
/// legitimate `instanceof` candidate, so the guard decides it at runtime, and the null arm is
/// named `null`.
#[test]
fn test_checked_downcast_return_nullable_bare_object_source_into_concrete_class() {
    let out = compile_and_run(
        r#"<?php
class Coll { public int $a = 111; public int $b = 222; public int $c = 333; }
class Route { public string $name = "route-name"; public int $port = 8080; }
function pick(int $k): ?object {
    if ($k === 0) { return new Coll(); }
    if ($k === 1) { return new Route(); }
    return null;
}
function want(int $k): Route { return pick($k); }
echo want(1)->name, "|";
try { want(0); echo "NO THROW"; } catch (\TypeError $e) { echo $e->getMessage(); }
echo "|";
try { want(2); echo "NO THROW"; } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "route-name|want(): Return value must be of type Route, Coll returned\
         |want(): Return value must be of type Route, null returned"
    );
}

/// A caught BOXED-source return `TypeError` must leave the heap balanced. The return position
/// RELEASES the mismatched value (nothing else owns a value the caller never receives), and the
/// release helper is picked by representation: `__rt_decref_mixed` for a box. Releasing a `Mixed`
/// cell as an object would corrupt a tag word, and macOS ABSORBS the resulting double free — so
/// the exit status alone is not evidence, the allocs/frees balance is.
#[test]
fn test_checked_downcast_return_boxed_throw_path_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Coll { public int $a = 111; public int $b = 222; public int $c = 333; }
class Route { public string $name = "route-name"; public int $port = 8080; }
function pick(int $k): Coll|Route|null {
    if ($k === 0) { return new Coll(); }
    if ($k === 1) { return new Route(); }
    return null;
}
function want(int $k): Route { return pick($k); }
$caught = 0;
for ($i = 0; $i < 50; $i++) {
    try { want(0); } catch (\TypeError $e) { $caught++; }
    try { want(2); } catch (\TypeError $e) { $caught++; }
}
echo $caught;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "100");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "heap debug reported a leak on the boxed return throw path: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("live_blocks=0"),
        "boxed return throw path left live blocks behind: {}",
        out.stderr
    );
}

/// STRUCTURAL PIN for the boxed return shape: a `?D` declaration must test the NULL TAG BEFORE
/// the `instanceof`, and the fail block must still use the RELEASING return throw op.
///
/// Phase 1 ordering is invisible to the behavioural tests above whenever the null arm happens to
/// be reached by a value that is not null; only asserting the emitted order catches a reordering
/// that would send a legitimate `null` into `instanceof` (which it can never satisfy) and from
/// there into a bogus `TypeError`.
#[test]
fn test_checked_downcast_return_boxed_guard_tests_null_tag_before_instanceof() {
    let module = emit_ir(
        r#"<?php
class Coll { public int $a = 1; }
class Route { public int $n = 2; }
function pick(int $k): Coll|Route|null {
    if ($k === 0) { return new Coll(); }
    if ($k === 1) { return new Route(); }
    return null;
}
function want(int $k): ?Route { return pick($k); }
var_dump(want(2));
"#,
    );
    let guarded = function_ir(&module, "want(");
    let ops: Vec<&str> = guarded
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("return_type_guard")
                || line.contains("instance_of")
                || line.contains("is_null")
                || line.starts_with("cond_br")
                || line.contains("throw_checked")
                || *line == "unreachable"
        })
        .collect();
    let null_at = ops
        .iter()
        .position(|line| line.contains("is_null"))
        .unwrap_or_else(|| panic!("boxed guard must emit a null tag test:\n{}", guarded));
    let instanceof_at = ops
        .iter()
        .position(|line| line.contains("instance_of"))
        .unwrap_or_else(|| panic!("boxed guard must emit an instanceof:\n{}", guarded));
    assert!(
        null_at < instanceof_at,
        "the null tag test must precede the instanceof: {:?}",
        ops
    );
    assert!(
        ops.iter().any(|line| line.starts_with("throw_checked_return_type_error")),
        "the return position MUST keep the releasing throw op: {:?}",
        ops
    );
    assert!(
        !ops.iter().any(|line| line.starts_with("throw_checked_type_error")),
        "the non-releasing argument throw op would leak the return value: {:?}",
        ops
    );
}
