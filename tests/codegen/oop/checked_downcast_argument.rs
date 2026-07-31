//! Purpose:
//! Integration and regression tests for the checked downcast at a CALL-ARGUMENT boundary: a value
//! statically known only as an ancestor (or as the bare `object` pseudo-type, or as a boxed
//! `mixed`) may be passed to a parameter declared narrower, accepted ONLY because a runtime guard
//! is emitted at the boundary — passing the value through on a match and throwing a catchable
//! `\TypeError` naming the ACTUAL runtime type on mismatch, exactly like PHP's own argument
//! enforcement.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Every expected message here is php-8.5.6 verified. PHP additionally appends
//!   `, called in <file> on line <n>` to the argument form for a userland callee; elephc does not
//!   reproduce that tail (it names the call site's file and line, which an AOT binary would have
//!   to bake in from its compile-time path), so the assertions stop at ` given`.
//! - The BOXED cases are the ones that were silently miscompiled before this guard existed: a
//!   `mixed` argument reaching a concrete object parameter was handed to the callee as a raw
//!   Mixed BOX pointer, which then read a garbage integer out of it. They are pinned here by
//!   asserting the thrown message, not merely that the program survives.
//! - Zero-cost claims (bare `object` parameter, proven-subtype argument) are verified textually
//!   via `--emit-ir`, since a spurious guard would still produce correct output.

use super::*;
use std::fs;
use std::process::Command;

/// Emits textual EIR for a source snippet through the CLI (`--emit-ir`, optimizer on).
fn emit_ir(source: &str) -> String {
    let dir = make_cli_test_dir("elephc_checked_downcast_argument_emit_ir");
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

/// The headline control: a wrong class at an argument boundary throws a CATCHABLE `TypeError`,
/// the program keeps running, and the exit status stays 0.
///
/// `B` and `C` deliberately have DIFFERENT first-slot layouts (`int` vs `string`). With matching
/// layouts an unguarded build happily bit-reads the wrong object and prints a plausible integer,
/// so a same-layout version of this test is green whether or not the guard exists.
#[test]
fn test_checked_downcast_argument_wrong_class_throws_catchable_type_error() {
    let out = compile_and_run(
        r#"<?php
class A {}
class B extends A { public int $n = 7; }
class C extends A { public string $s = 'x'; }
function mk(): A { return new C(); }
function f(B $b): int { return $b->n; }
$a = mk();
try { echo f($a), "\n"; } catch (\TypeError $e) { echo get_class($e), ': ', $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "TypeError: f(): Argument #1 ($b) must be of type B, C given\nalive"
    );
}

/// The matching arm of the same shape passes through untouched and reaches the callee as a usable
/// object of the declared class.
#[test]
fn test_checked_downcast_argument_matching_class_passes_through() {
    let out = compile_and_run(
        r#"<?php
class A {}
class B extends A { public int $n = 7; }
class C extends A { public string $s = 'x'; }
function mk(bool $flag): A { return $flag ? new B() : new C(); }
function f(B $b): int { return $b->n; }
echo f(mk(true));
"#,
    );
    assert_eq!(out, "7");
}

/// Interface hierarchies take the same path: a base-interface value into a sub-interface
/// parameter, which is the `PrototypedArrayNode::setPrototype` shape.
#[test]
fn test_checked_downcast_argument_interface_hierarchy() {
    let out = compile_and_run(
        r#"<?php
interface NodeInterface {}
interface PrototypeNodeInterface extends NodeInterface { public function tag(): string; }
class ProtoNode implements PrototypeNodeInterface { public function tag(): string { return 'proto'; } }
class PlainNode implements NodeInterface {}
function node(bool $ok): NodeInterface { return $ok ? new ProtoNode() : new PlainNode(); }
function setPrototype(PrototypeNodeInterface $node): string { return $node->tag(); }
echo setPrototype(node(true)), "|";
try { echo setPrototype(node(false)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "proto|setPrototype(): Argument #1 ($node) must be of type PrototypeNodeInterface, PlainNode given"
    );
}

/// A bare `object` SOURCE (a raw object pointer of statically unknown class) is guardable: every
/// declared arm is a legitimate `instanceof` target for it. This is the
/// `PhpArrayAdapter::doGet` shape.
#[test]
fn test_checked_downcast_argument_bare_object_source() {
    let out = compile_and_run(
        r#"<?php
interface AdapterInterface { public function name(): string; }
class ArrayAdapter implements AdapterInterface { public function name(): string { return 'array'; } }
class NotAnAdapter { public int $x = 1; }
function pool(bool $ok): object { return $ok ? new ArrayAdapter() : new NotAnAdapter(); }
function doGet(AdapterInterface $pool): string { return $pool->name(); }
echo doGet(pool(true)), "|";
try { echo doGet(pool(false)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "array|doGet(): Argument #1 ($pool) must be of type AdapterInterface, NotAnAdapter given"
    );
}

/// A `mixed` argument carrying the WRONG OBJECT into a concrete object parameter. Before the
/// guard this handed the callee a raw `Mixed` box pointer, which read a garbage integer out of
/// the box header and printed it with exit 0.
#[test]
fn test_checked_downcast_argument_boxed_wrong_object_throws() {
    let out = compile_and_run(
        r#"<?php
class B { public int $n = 7; }
class D2 { public string $s = 'zzz'; }
function g(): mixed { return new D2(); }
function f(B $b): int { return $b->n; }
$m = g();
try { echo f($m), "\n"; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "f(): Argument #1 ($b) must be of type B, D2 given\nalive"
    );
}

/// The same shape with a SCALAR payload, which exercises the boxed fail path's runtime tag table
/// rather than its `get_class` route.
#[test]
fn test_checked_downcast_argument_boxed_scalar_names_its_php_type() {
    let out = compile_and_run(
        r#"<?php
class B { public int $n = 7; }
function g(int $mode): mixed {
    if ($mode === 0) { return 'hello'; }
    if ($mode === 1) { return 12; }
    if ($mode === 2) { return 1.5; }
    if ($mode === 3) { return true; }
    if ($mode === 4) { return false; }
    if ($mode === 5) { return [1, 2]; }
    return null;
}
function f(B $b): int { return $b->n; }
for ($i = 0; $i <= 6; $i++) {
    $m = g($i);
    try { echo f($m); } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
}
"#,
    );
    assert_eq!(
        out,
        "f(): Argument #1 ($b) must be of type B, string given\n\
         f(): Argument #1 ($b) must be of type B, int given\n\
         f(): Argument #1 ($b) must be of type B, float given\n\
         f(): Argument #1 ($b) must be of type B, true given\n\
         f(): Argument #1 ($b) must be of type B, false given\n\
         f(): Argument #1 ($b) must be of type B, array given\n\
         f(): Argument #1 ($b) must be of type B, null given\n"
    );
}

/// A `mixed` argument carrying the RIGHT object reaches the callee as a usable object: the guard's
/// ok-edge unboxes it, so the callee's by-offset property access reads the real instance.
#[test]
fn test_checked_downcast_argument_boxed_right_object_is_unboxed() {
    let out = compile_and_run(
        r#"<?php
class B { public int $n = 7; public function twice(): int { return $this->n * 2; } }
function g(): mixed { return new B(); }
function f(B $b): int { return $b->twice() + $b->n; }
$m = g();
echo f($m);
"#,
    );
    assert_eq!(out, "21");
}

/// PHP names the DECLARING class of a method, not the receiver's: `Sub::m()` inherited from `K`
/// reports `K::m()`. Constructors likewise report `C::__construct()`.
#[test]
fn test_checked_downcast_argument_callee_is_the_declaring_class() {
    let out = compile_and_run(
        r#"<?php
class Wrong { public string $s = 'w'; }
class Right { public int $n = 1; }
function any(bool $ok): object { return $ok ? new Right() : new Wrong(); }
class K { public function m(Right $r): int { return $r->n; } }
class Sub extends K {}
class Ctor { public function __construct(Right $r) {} }
$s = new Sub();
try { echo $s->m(any(false)); } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
try { new Ctor(any(false)); } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(
        out,
        "K::m(): Argument #1 ($r) must be of type Right, Wrong given\n\
         Ctor::__construct(): Argument #1 ($r) must be of type Right, Wrong given\n"
    );
}

/// A caught argument `TypeError` must leave the heap balanced: the throw does NOT release the
/// mismatched value (its caller-side local still owns it), and the caller's own cleanup does.
/// Releasing it in the throw would be a double free — which macOS ABSORBS, so the exit status
/// alone is not evidence; the allocs/frees balance under `--heap-debug` is.
#[test]
fn test_checked_downcast_argument_throw_path_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A {}
class B extends A { public int $n = 7; }
class C extends A { public string $s = 'zz'; }
function mk(): A { return new C(); }
function f(B $b): int { return $b->n; }
$caught = 0;
for ($i = 0; $i < 50; $i++) {
    $a = mk();
    try { echo f($a); } catch (\TypeError $e) { $caught++; }
    unset($a);
}
echo $caught;
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "50");
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "heap debug reported a leak on the argument throw path: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains("live_blocks=0"),
        "argument throw path left live blocks behind: {}",
        out.stderr
    );
}

/// A bare `object` PARAMETER accepts every object, so no guard may be emitted for it:
/// `Op::InstanceOf` against the empty class name can never match.
#[test]
fn test_checked_downcast_argument_bare_object_param_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class C { public string $s = 'x'; }
function f(object $o): string { return get_class($o); }
echo f(new C());
"#,
    );
    assert!(
        !module.contains("arg_type_guard"),
        "a bare `object` parameter must emit no guard:\n{}",
        module
    );
}

/// A proven-subtype argument is free: the value already IS an instance of the declared parameter
/// type, so the guard would only tax an already-correct call site.
#[test]
fn test_checked_downcast_argument_proven_subtype_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class A {}
class B extends A { public int $n = 3; }
function f(A $a): string { return get_class($a); }
echo f(new B());
"#,
    );
    assert!(
        !module.contains("arg_type_guard"),
        "a proven-subtype argument must emit no guard:\n{}",
        module
    );
}

/// A legitimate `null` into a nullable object parameter passes through untouched, and emits no
/// guard: a declared `?B` is a boxed slot, which this boundary does not guard.
#[test]
fn test_checked_downcast_argument_nullable_param_passes_null_through() {
    let module = emit_ir(
        r#"<?php
class A {}
class B extends A { public int $n = 3; }
function f(?B $b): string { return $b === null ? 'null' : 'obj'; }
echo f(null), f(new B());
"#,
    );
    assert!(
        !module.contains("arg_type_guard"),
        "a nullable object parameter must emit no guard yet:\n{}",
        module
    );
    let out = compile_and_run(
        r#"<?php
class A {}
class B extends A { public int $n = 3; }
function f(?B $b): string { return $b === null ? 'null' : 'obj'; }
echo f(null), "|", f(new B());
"#,
    );
    assert_eq!(out, "null|obj");
}

/// OWNERSHIP PIN: the argument position must use the NON-releasing throw op. The return position
/// keeps the releasing one. Swapping them compiles, passes every behavioural test above, and is a
/// double free the caller's own cleanup then walks into — only an IR-level assertion catches it.
#[test]
fn test_checked_downcast_argument_uses_the_non_releasing_throw_op() {
    let module = emit_ir(
        r#"<?php
class A {}
class B extends A { public int $n = 7; }
class C extends A { public string $s = 'x'; }
function mk(): A { return new C(); }
function f(B $b): int { return $b->n; }
function r(): B { $a = mk(); return $a; }
$a = mk();
try { echo f($a); } catch (\TypeError $e) {}
try { echo r()->n; } catch (\TypeError $e) {}
"#,
    );
    let argument_fail = module
        .split("arg_type_guard.fail:")
        .nth(1)
        .expect("argument guard fail block not emitted");
    let argument_throw = argument_fail
        .lines()
        .find(|line| line.contains("throw_checked"))
        .expect("argument fail block emits no throw");
    assert!(
        argument_throw.contains("throw_checked_type_error"),
        "the argument position must NOT use the releasing return op: {}",
        argument_throw
    );

    let return_fail = module
        .split("return_type_guard.fail:")
        .nth(1)
        .expect("return guard fail block not emitted");
    let return_throw = return_fail
        .lines()
        .find(|line| line.contains("throw_checked"))
        .expect("return fail block emits no throw");
    assert!(
        return_throw.contains("throw_checked_return_type_error"),
        "the return position must keep the releasing op: {}",
        return_throw
    );
}

// --- UNION SOURCES. A union's `codegen_repr()` is a boxed `Mixed`, so these are the boxed shapes:
//     a union into a concrete object parameter must UNBOX on the ok-edge, and a union into a
//     declared `?D` copies the box pointer through. Both need Phase-1 tag tests, because a union
//     source can carry a legitimate `null` (or array, or string) payload that must not be sent
//     down the `instanceof` branch. ---

/// The Symfony `RouteTrait::$route` shape: a declared `RouteCollection|Route` reaching
/// `RouteCollection::addCollection`'s concrete `RouteCollection` parameter. The matching member
/// passes through as a usable object; the sibling member throws PHP's own `TypeError`.
///
/// `RC` and `RT` deliberately have DIFFERENT first-slot layouts (`int` vs `string`): with matching
/// layouts an unguarded build bit-reads the wrong object and prints a plausible integer, so a
/// same-layout version of this test is green whether or not the guard exists.
#[test]
fn test_checked_downcast_argument_object_union_into_concrete_class() {
    let out = compile_and_run(
        r#"<?php
class RC { public int $n = 7; }
class RT { public string $s = 'zzzzzzzz'; }
function src(int $k): RC|RT { return $k === 0 ? new RC() : new RT(); }
function add(RC $c): int { return $c->n; }
echo add(src(0)), "|";
try { echo add(src(1)); } catch (\TypeError $e) { echo $e->getMessage(); }
echo "|alive";
"#,
    );
    assert_eq!(
        out,
        "7|add(): Argument #1 ($c) must be of type RC, RT given|alive"
    );
}

/// A union source carrying the `false` SENTINEL of a `LoaderInterface|false` into a concrete
/// class parameter (the `RoutingConfigurator::__construct` shape). The fail path names the payload
/// `false`, not `object` — its own tag table, not the array builtins' generic bucket.
#[test]
fn test_checked_downcast_argument_union_false_sentinel_names_false() {
    let out = compile_and_run(
        r#"<?php
interface LoaderInterface { public function tag(): string; }
class PhpFileLoader implements LoaderInterface { public int $n = 11; public function tag(): string { return 'php'; } }
class YamlLoader implements LoaderInterface { public string $s = 'yamlyaml'; public function tag(): string { return 'yaml'; } }
function resolve(int $k): LoaderInterface|false {
    if ($k === 0) { return new PhpFileLoader(); }
    if ($k === 1) { return new YamlLoader(); }
    return false;
}
function cfg(PhpFileLoader $loader): int { return $loader->n; }
echo cfg(resolve(0)), "|";
try { echo cfg(resolve(1)); } catch (\TypeError $e) { echo $e->getMessage(); }
echo "|";
try { echo cfg(resolve(2)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "11|cfg(): Argument #1 ($loader) must be of type PhpFileLoader, YamlLoader given\
         |cfg(): Argument #1 ($loader) must be of type PhpFileLoader, false given"
    );
}

/// A union source carrying an ARRAY member into an INTERFACE parameter — the
/// `ContainerBuilder::addResource` / `GlobFileLoader` shape. The array payload must be named
/// `array`, and it must never be bit-read as an object.
#[test]
fn test_checked_downcast_argument_union_array_member_into_interface() {
    let out = compile_and_run(
        r#"<?php
interface ResourceInterface { public function key(): string; }
class GlobResource implements ResourceInterface { public int $n = 5; public function key(): string { return 'glob'; } }
function globbed(int $k): array|GlobResource|null {
    if ($k === 0) { return new GlobResource(); }
    if ($k === 1) { return ['a', 'b']; }
    return null;
}
function addResource(ResourceInterface $r): string { return $r->key(); }
echo addResource(globbed(0)), "|";
try { echo addResource(globbed(1)); } catch (\TypeError $e) { echo $e->getMessage(); }
echo "|";
try { echo addResource(globbed(2)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "glob|addResource(): Argument #1 ($r) must be of type ResourceInterface, array given\
         |addResource(): Argument #1 ($r) must be of type ResourceInterface, null given"
    );
}

/// A union source into a declared UNION parameter (`?NodeInterface`) — the boxed-into-boxed shape,
/// and the one that makes Phase 1 load-bearing: a legitimate `null` member must reach the callee
/// AS NULL rather than fall into the `instanceof` branch. The `BooleanNode`/`EnumNode`
/// `$parent` shape.
#[test]
fn test_checked_downcast_argument_union_into_nullable_object_param() {
    let out = compile_and_run(
        r#"<?php
interface NodeInterface { public function name(): string; }
interface NodeParentInterface { public function label(): string; }
class ArrayNode implements NodeInterface { public int $n = 3; public function name(): string { return 'arr'; } }
class Builder implements NodeParentInterface { public string $s = 'builderx'; public function label(): string { return 'b'; } }
function parent_of(int $k): NodeParentInterface|NodeInterface|null {
    if ($k === 0) { return new ArrayNode(); }
    if ($k === 1) { return new Builder(); }
    return null;
}
function boolean_node(?NodeInterface $parent): string { return $parent === null ? '<null>' : $parent->name(); }
echo boolean_node(parent_of(0)), "|", boolean_node(parent_of(2)), "|";
try { echo boolean_node(parent_of(1)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "arr|<null>|boolean_node(): Argument #1 ($parent) must be of type ?NodeInterface, Builder given"
    );
}

/// Phase 1 BEFORE Phase 2, checked in the emitted EIR rather than only at runtime: a run that
/// passes a real `null` would also succeed with the tests in the wrong order (the null would just
/// take the fail path of an `instanceof` it can never satisfy and… still throw), so the ORDER is
/// what has to be pinned, and only an IR inspection can see it.
///
/// The fixture deliberately carries NO `try`/`catch`: a catch clause lowers to its own
/// `instance_of`, and an earlier version of this test was satisfied by that one while the guard
/// itself was not emitted at all. Every assertion below therefore names a guard BLOCK first.
#[test]
fn test_checked_downcast_argument_tag_tests_precede_instanceof() {
    let module = emit_ir(
        r#"<?php
interface NodeInterface { public function name(): string; }
class ArrayNode implements NodeInterface { public int $n = 3; public function name(): string { return 'arr'; } }
class Builder { public string $s = 'builderx'; }
function parent_of(int $k): Builder|NodeInterface|null {
    if ($k === 0) { return new ArrayNode(); }
    if ($k === 1) { return new Builder(); }
    return null;
}
function boolean_node(?NodeInterface $parent): string { return $parent === null ? '<null>' : $parent->name(); }
echo boolean_node(parent_of(0));
"#,
    );
    let ok_at = module
        .find("arg_type_guard.ok:")
        .unwrap_or_else(|| panic!("the argument guard was not emitted at all:\n{}", module));
    let head = &module[..ok_at];
    assert!(
        head.contains(" = is_null "),
        "Phase 1 emitted no null tag test ahead of the guard's ok block:\n{}",
        head
    );
    assert!(
        !head.contains(" = instance_of "),
        "Phase 2 ran before Phase 1's tag tests; a legitimate null would be routed into the \
         object branch:\n{}",
        head
    );
    let check_at = module
        .find("arg_type_guard.check:")
        .unwrap_or_else(|| panic!("the Phase-2 block was not emitted:\n{}", module));
    assert!(
        module[check_at..].contains(" = instance_of "),
        "the Phase-2 block emits no instanceof:\n{}",
        &module[check_at..]
    );
}

/// A union whose EVERY member is already covered by a declared arm costs nothing: `D|null` into
/// `?D` can fail no runtime test, so the fast path must emit no guard at all. Without this a
/// guard would be taxed onto every already-correct nullable-object call site in the program.
#[test]
fn test_checked_downcast_argument_covered_union_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class D { public int $q = 3; }
function pick(int $k): D|null { return $k === 0 ? new D() : null; }
function g(?D $d): int { return $d === null ? -1 : $d->q; }
echo g(pick(0)), g(pick(1));
"#,
    );
    assert!(
        !module.contains("arg_type_guard"),
        "a fully covered union source must emit no guard blocks:\n{}",
        module
    );
}

/// A declared union PHP CONVERTS INTO must never reach the chain: a `Stringable` object arriving
/// at a `string|D` slot is weak-coerced by PHP, and an arm chain would throw over it. The gate is
/// `declared_union_guard_is_php_faithful`; this pins that no guard is emitted for such a slot.
#[test]
fn test_checked_downcast_argument_conversion_bearing_union_emits_no_guard() {
    let module = emit_ir(
        r#"<?php
class D { public int $n = 1; }
class S { public function __toString(): string { return 'from-S'; } }
function f(string|D $v): string { return is_string($v) ? 'str' : 'obj'; }
echo f(new S());
"#,
    );
    assert!(
        !module.contains("arg_type_guard"),
        "a declared union with a scalar arm must emit no guard blocks:\n{}",
        module
    );
}

/// The declared ARRAY arm's pass-through — the one Phase-1 arm besides `null` that a guarded
/// declaration can carry. A real array must reach the callee AS A USABLE ARRAY (`count()` sees
/// its elements), a real null must reach it as null, and a value matching neither arm must throw.
///
/// The message also pins PHP's CANONICAL union rendering: the source declares
/// `array|D|null` and php-8.5.6 renders it `D|array|null` (class arms first in declared order,
/// then the built-ins in PHP's own fixed order).
#[test]
fn test_checked_downcast_argument_declared_array_arm_passes_through() {
    let out = compile_and_run(
        r#"<?php
class D { public int $n = 4; }
class Other { public string $s = 'otherotr'; }
function src(int $k): D|Other|array|null {
    if ($k === 0) { return new D(); }
    if ($k === 1) { return ['a', 'b', 'c']; }
    if ($k === 2) { return null; }
    return new Other();
}
function take(array|D|null $v): string {
    if ($v === null) { return '<null>'; }
    if (is_array($v)) { return 'array:' . count($v); }
    return 'obj:' . $v->n;
}
echo take(src(0)), "|", take(src(1)), "|", take(src(2)), "|";
try { echo take(src(3)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "obj:4|array:3|<null>|take(): Argument #1 ($v) must be of type D|array|null, Other given"
    );
}

/// PHP does not echo a union declaration back in its declared order, and a guard message that did
/// would diverge from `php -n` on every multi-arm union. Class arms come first in DECLARED order,
/// then the built-ins in PHP's own fixed order — php-8.5.6 renders the source's `array|E|D|null`
/// as `E|D|array|null`.
#[test]
fn test_checked_downcast_argument_union_message_uses_php_canonical_order() {
    let out = compile_and_run(
        r#"<?php
class D { public int $n = 1; }
class E { public int $m = 2; }
class Other { public string $s = 'otherotr'; }
function src(int $k): D|E|Other|array|null { return $k === 0 ? new D() : new Other(); }
function take(array|E|D|null $v): string { return is_object($v) ? get_class($v) : 'x'; }
echo take(src(0)), "|";
try { echo take(src(1)); } catch (\TypeError $e) { echo $e->getMessage(); }
"#,
    );
    assert_eq!(
        out,
        "D|take(): Argument #1 ($v) must be of type E|D|array|null, Other given"
    );
}
