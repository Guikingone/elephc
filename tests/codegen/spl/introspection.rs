//! Purpose:
//! End-to-end tests for SPL and class-table introspection helpers.
//! Covers metadata arrays emitted for interfaces, parent classes, and direct trait uses.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the SPL test module.
//!
//! Key details:
//! - The helpers return associative `name => name` arrays. A literal class-name
//!   string folds to an AOT snapshot; a non-literal string or an object argument
//!   resolves through the runtime per-class relation registry instead.

use crate::support::*;

/// Verifies that class implements returns assoc interface names.
#[test]
fn test_class_implements_returns_assoc_interface_names() {
    let out = compile_and_run(
        r#"<?php
interface BaseMarker {}
interface ChildMarker extends BaseMarker {}
class ImplMarker implements ChildMarker {}

foreach (class_implements("ImplMarker") as $name => $value) {
    echo $name;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(out, "ChildMarker=ChildMarker;BaseMarker=BaseMarker;");
}

/// Verifies that class implements accepts object static type.
#[test]
fn test_class_implements_accepts_object_static_type() {
    let out = compile_and_run(
        r#"<?php
class Counter implements Countable {
    public function count(): int { return 3; }
}

$interfaces = class_implements(new Counter());
echo isset($interfaces["Countable"]) ? "yes" : "no";
"#,
    );
    assert_eq!(out, "yes");
}

/// Verifies that class implements builtin SPL class includes inherited interfaces.
///
/// Order note: elephc's `SplDoublyLinkedList` is a synthetic `FlattenedClass`
/// (`implements Iterator, Countable, ArrayAccess`, see
/// `src/types/checker/builtin_spl_classes/containers.rs`) that flows through the
/// SAME class-schema linearization as any user-declared class (`collect_interfaces`
/// in `src/types/checker/schema/classes/interfaces.rs`). Its `class_implements()`
/// order therefore follows PHP's own-declared-interfaces-then-reversed-ancestor-
/// chains rule (`php -n` verified: `class X implements A, B {}` where `A extends P`
/// reports `[A, B, P]`, not `[A, P, B]`) — own D block first, then `Traversable`
/// (Iterator's own ancestor) appended after the whole block. Real PHP's C-registered
/// `ext-spl` `SplDoublyLinkedList` interleaves `Traversable` immediately after
/// `Iterator` instead; that reflects `zend_class_implements()`'s internal C
/// registration order, a different mechanism from parsing a userland `implements`
/// clause, and isn't representative of the linearization rule this test's synthetic
/// class actually goes through.
#[test]
fn test_class_implements_builtin_spl_class_includes_inherited_interfaces() {
    let out = compile_and_run(
        r#"<?php
$interfaces = class_implements("SplDoublyLinkedList");
foreach ($interfaces as $name => $value) {
    echo $name;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(
        out,
        "Iterator=Iterator;Countable=Countable;ArrayAccess=ArrayAccess;Traversable=Traversable;"
    );
}

/// Verifies that class parents returns immediate parent then ancestors.
#[test]
fn test_class_parents_returns_immediate_parent_then_ancestors() {
    let out = compile_and_run(
        r#"<?php
class Root {}
class Middle extends Root {}
class Leaf extends Middle {}

foreach (class_parents("Leaf") as $name => $value) {
    echo $name;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(out, "Middle=Middle;Root=Root;");
}

/// Verifies that class uses returns direct class traits only.
#[test]
fn test_class_uses_returns_direct_class_traits_only() {
    let out = compile_and_run(
        r#"<?php
trait SharedTrait {}
trait LocalTrait {
    use SharedTrait;
}
class ParentWithTrait {
    use SharedTrait;
}
class ChildWithTrait extends ParentWithTrait {
    use LocalTrait;
}

foreach (class_uses("ChildWithTrait") as $name => $value) {
    echo $name;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(out, "LocalTrait=LocalTrait;");
}

/// Verifies that class uses accepts trait name.
#[test]
fn test_class_uses_accepts_trait_name() {
    let out = compile_and_run(
        r#"<?php
trait BaseTrait {}
trait CombinedTrait {
    use BaseTrait;
}

foreach (class_uses("CombinedTrait") as $name => $value) {
    echo $name;
    echo "=";
    echo $value;
    echo ";";
}
"#,
    );
    assert_eq!(out, "BaseTrait=BaseTrait;");
}

/// Verifies that class relation helpers return false for unknown literal names.
#[test]
fn test_class_relation_helpers_return_false_for_unknown_literal_names() {
    let out = compile_and_run(
        r#"<?php
var_dump(class_implements("MissingClass"));
var_dump(class_parents("MissingClass"));
var_dump(class_uses("MissingClass"));
"#,
    );
    assert_eq!(out, "bool(false)\nbool(false)\nbool(false)\n");
}

// -- non-literal class_implements()/class_parents()/class_uses() (this file's
// runtime relation registry, `_class_relation_table`/`_interface_relation_table`/
// `_trait_relation_table`) --

/// Verifies that a non-literal (runtime-selected) class name resolves the same
/// transitively-implemented interface set as the literal fast path, in PHP
/// declaration order.
#[test]
fn test_class_implements_non_literal_name_matches_literal_transitive_order() {
    let out = compile_and_run(
        r#"<?php
interface BaseMarker {}
interface ChildMarker extends BaseMarker {}
class ImplMarker implements ChildMarker {}

$name = "Impl" . "Marker";
foreach (class_implements($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "ChildMarker=ChildMarker;BaseMarker=BaseMarker;");
}

/// Verifies that a non-literal class name resolves the full ancestor chain,
/// immediate parent first.
#[test]
fn test_class_parents_non_literal_name_returns_ancestor_chain() {
    let out = compile_and_run(
        r#"<?php
class Root {}
class Middle extends Root {}
class Leaf extends Middle {}

$name = "Leaf";
foreach (class_parents($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "Middle=Middle;Root=Root;");
}

/// Verifies that a non-literal class name resolves only its own directly
/// declared trait uses, excluding traits used by its parent class.
#[test]
fn test_class_uses_non_literal_name_returns_direct_traits_only() {
    let out = compile_and_run(
        r#"<?php
trait SharedTrait {}
trait LocalTrait { use SharedTrait; }
class ParentWithTrait { use SharedTrait; }
class ChildWithTrait extends ParentWithTrait { use LocalTrait; }

$name = "ChildWithTrait";
foreach (class_uses($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "LocalTrait=LocalTrait;");
}

/// Verifies that a non-literal unknown class name returns `false` from every
/// class-relation helper, matching the literal fast path's miss behavior.
#[test]
fn test_class_relation_helpers_non_literal_miss_returns_false() {
    let out = compile_and_run(
        r#"<?php
class KnownClass {}

$name = "Missing" . "Class";
var_dump(class_implements($name));
var_dump(class_parents($name));
var_dump(class_uses($name));
"#,
    );
    assert_eq!(out, "bool(false)\nbool(false)\nbool(false)\n");
}

/// Verifies that a case-variant non-literal needle still resolves the target,
/// matching PHP's case-insensitive class/interface/trait names.
#[test]
fn test_class_implements_non_literal_name_is_case_insensitive() {
    let out = compile_and_run(
        r#"<?php
interface Marker {}
class Impl implements Marker {}

$name = "iMpL";
foreach (class_implements($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "Marker=Marker;");
}

/// Verifies non-literal `class_implements()`/`class_parents()`/`class_uses()`
/// on an interface/trait target, matching PHP's target-kind matrix: an
/// interface's `implements` is its own transitively extended parent
/// interfaces (`parents`/`uses` are empty arrays, not `false`); a trait's
/// `uses` is its own direct trait uses (`implements`/`parents` are empty
/// arrays, not `false`).
#[test]
fn test_class_relation_helpers_non_literal_interface_and_trait_targets() {
    let out = compile_and_run(
        r#"<?php
interface BaseMarker {}
interface ChildMarker extends BaseMarker {}
trait InnerTrait {}
trait OuterTrait { use InnerTrait; }

$iface = "ChildMarker";
foreach (class_implements($iface) as $k => $v) { echo $k, "=", $v, ";"; }
echo "|";
var_dump(count(class_parents($iface)) === 0);
echo "|";
var_dump(count(class_uses($iface)) === 0);
echo "|";

$trait = "OuterTrait";
foreach (class_uses($trait) as $k => $v) { echo $k, "=", $v, ";"; }
echo "|";
var_dump(count(class_implements($trait)) === 0);
echo "|";
var_dump(count(class_parents($trait)) === 0);
"#,
    );
    assert_eq!(
        out,
        "BaseMarker=BaseMarker;|bool(true)\n|bool(true)\n|InnerTrait=InnerTrait;|bool(true)\n|bool(true)\n"
    );
}

/// Verifies a `Mixed`-typed needle (a variable whose static type is not pinned
/// to `string`) still resolves through the runtime relation registry when it
/// holds a string at runtime.
#[test]
fn test_class_implements_mixed_typed_needle_resolves_at_runtime() {
    let out = compile_and_run(
        r#"<?php
interface Marker {}
class Impl implements Marker {}

function pick(bool $useClass) {
    if ($useClass) {
        return "Impl";
    }
    return 42;
}

$name = pick(true);
foreach (class_implements($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "Marker=Marker;");
}

/// Verifies an object argument always resolves through its RUNTIME class,
/// not the static declared parameter type: a `Base`-typed parameter holding a
/// `Leaf` instance must report `Leaf`'s interfaces (verified against `php -n`).
#[test]
fn test_class_implements_object_argument_uses_runtime_class_under_polymorphism() {
    let out = compile_and_run(
        r#"<?php
interface Marker {}
class Base {}
class Leaf extends Base implements Marker {}

function describe(Base $b) {
    return class_implements($b);
}

foreach (describe(new Leaf()) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "Marker=Marker;");
}

/// Verifies an anonymous-class object argument resolves its synthetic class's
/// interfaces and direct trait uses correctly.
#[test]
fn test_class_relation_helpers_accept_anonymous_class_object_argument() {
    let out = compile_and_run(
        r#"<?php
interface Marker {}
trait Greetable {}

$anon = new class implements Marker { use Greetable; };
foreach (class_implements($anon) as $k => $v) { echo $k, "=", $v, ";"; }
echo "|";
foreach (class_uses($anon) as $k => $v) { echo $k, "=", $v, ";"; }
"#,
    );
    assert_eq!(out, "Marker=Marker;|Greetable=Greetable;");
}

/// Verifies that a trait-use method alias (`use Trait { method as alias; }`)
/// does not change `class_uses()`'s reported trait identity: PHP's method
/// renaming is orthogonal to which traits a class uses (verified against
/// `php -n`).
#[test]
fn test_class_uses_ignores_trait_method_alias_adaptations() {
    let out = compile_and_run(
        r#"<?php
trait Greetable {
    public function hello() { return "hi"; }
}
class Greeter {
    use Greetable {
        hello as sayHi;
    }
}

$name = "Greeter";
foreach (class_uses($name) as $k => $v) {
    echo $k, "=", $v, ";";
}
"#,
    );
    assert_eq!(out, "Greetable=Greetable;");
}

/// Verifies the runtime-materialized relation array has the SAME per-call heap
/// footprint as the literal fast path for an equivalent relation, proving the
/// runtime hash-construction loop (`__rt_hash_from_name_list`) does not leak
/// any more than the literal path's compile-time-unrolled construction
/// already does. Neither path is fully heap-clean today: `unset()` on a
/// function-returned boxed `Mixed` array is a known, pre-existing, orthogonal
/// gap (present even for a plain array literal returned from a function —
/// unrelated to class-relation introspection) — this test pins parity between
/// the two `class_implements()` code paths rather than asserting a false
/// "clean heap" claim neither path can currently make.
#[test]
fn test_class_implements_non_literal_result_array_matches_literal_heap_footprint() {
    let live_blocks_per_iteration = |source: &str| -> u64 {
        let out = compile_and_run_with_heap_debug(source);
        assert!(out.success, "program failed: {}", out.stderr);
        assert_eq!(out.stdout, "done");
        let line = out
            .stderr
            .lines()
            .find(|line| line.starts_with("HEAP DEBUG: leak summary:"))
            .unwrap_or_else(|| panic!("missing leak summary line: {}", out.stderr));
        if line.contains("clean") {
            return 0;
        }
        line.split("live_blocks=")
            .nth(1)
            .and_then(|rest| rest.split(' ').next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| panic!("missing live_blocks count: {line}"))
            / 200
    };

    let literal_leak = live_blocks_per_iteration(
        r#"<?php
interface Marker {}
class Impl implements Marker {}

for ($i = 0; $i < 200; $i++) {
    $interfaces = class_implements("Impl");
    unset($interfaces);
}
echo "done";
"#,
    );
    let non_literal_leak = live_blocks_per_iteration(
        r#"<?php
interface Marker {}
class Impl implements Marker {}

$name = "Impl";
for ($i = 0; $i < 200; $i++) {
    $interfaces = class_implements($name);
    unset($interfaces);
}
echo "done";
"#,
    );
    assert_eq!(
        non_literal_leak, literal_leak,
        "non-literal class_implements() must not leak more per call than the literal path"
    );
}
