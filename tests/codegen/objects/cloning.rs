//! Purpose:
//! Integration or regression tests for PHP object cloning codegen.
//! Covers shallow object copies, declared property slots, stdClass dynamic properties, and `__clone` hooks.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures compile to native binaries and compare stdout against PHP clone semantics.

use super::*;

/// Verifies cloning declared scalar/string properties creates an independent object slot copy.
#[test]
fn test_clone_copies_declared_properties_independently() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public int $n = 1;
    public string $label = "one";
}
$a = new Item();
$b = clone $a;
$b->n = 2;
$b->label = "two";
echo $a->n . ":" . $a->label . "|" . $b->n . ":" . $b->label;
"#,
    );
    assert_eq!(out, "1:one|2:two");
}

/// Verifies `__clone()` is invoked after the shallow copy and mutates the clone, not the source.
#[test]
fn test_clone_invokes_magic_clone_on_the_copy() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public int $n = 1;
    public function __clone(): void {
        echo "hook;";
        $this->n = $this->n + 10;
    }
}
$a = new Counter();
$b = clone $a;
echo $a->n . "|" . $b->n;
"#,
    );
    assert_eq!(out, "hook;1|11");
}

/// Verifies `__clone()` can replace a string property without corrupting the source object.
#[test]
fn test_clone_persists_string_property_before_magic_clone_mutation() {
    let out = compile_and_run(
        r#"<?php
class LabelBox {
    public string $label = "A";
    public function __clone(): void {
        $this->label = $this->label . ":copy";
    }
}
$a = new LabelBox();
$b = clone $a;
echo $a->label . "|" . $b->label;
"#,
    );
    assert_eq!(out, "A|A:copy");
}

/// Verifies object-valued properties are shallow-copied, so nested object mutations remain shared.
#[test]
fn test_clone_keeps_nested_objects_shared() {
    let out = compile_and_run(
        r#"<?php
class Child {
    public int $x = 1;
}
class Boxed {
    public Child $child;
    public function __construct() {
        $this->child = new Child();
    }
}
$a = new Boxed();
$b = clone $a;
$b->child->x = 7;
echo $a->child->x . "|" . $b->child->x;
"#,
    );
    assert_eq!(out, "7|7");
}

/// Verifies cloning preserves a declared object union for a following fluent method call.
#[test]
fn test_clone_preserves_object_union_for_fluent_dispatch() {
    let out = compile_and_run(
        r#"<?php
final class CloneRoute {
    public string $path = '';

    public function setPath(string $path): self {
        $this->path = $path;
        return $this;
    }
}

final class CloneCollection {}

final class CloneHolder {
    private CloneRoute|CloneCollection $route;

    public function __construct() {
        $this->route = new CloneRoute();
    }

    public function route(string $path): CloneRoute {
        return (clone $this->route)->setPath($path);
    }
}

$route = (new CloneHolder())->route('/ready');
echo $route->path;
"#,
    );
    assert_eq!(out, "/ready");
}

/// Verifies stdClass dynamic properties are copied into a separate hash table during cloning.
#[test]
fn test_clone_copies_stdclass_dynamic_properties_independently() {
    let out = compile_and_run(
        r#"<?php
$a = new stdClass();
$a->name = "source";
$b = clone $a;
$b->name = "copy";
$b->extra = "new";
echo $a->name . "|" . $b->name . "|" . (isset($a->extra) ? "Y" : "N");
"#,
    );
    assert_eq!(out, "source|copy|N");
}

/// Verifies cloning through an interface uses runtime class metadata rather than treating the
/// interface name as a concrete object layout.
#[test]
fn test_clone_interface_typed_receiver_dispatches_runtime_class() {
    let out = compile_and_run(
        r#"<?php
interface CloneView {
    public function value(): string;
}

class CloneViewImpl implements CloneView {
    public string $name = 'source';

    public function value(): string {
        return $this->name;
    }

    public function __clone(): void {
        $this->name = 'clone';
    }
}

function duplicate(CloneView $value): CloneView {
    return clone $value;
}

$source = new CloneViewImpl();
$clone = duplicate($source);
echo $source->value() . '|' . $clone->value();
"#,
    );
    assert_eq!(out, "source|clone");
}

/// Verifies cloning an untyped parameter checks and dispatches on its runtime object class.
#[test]
fn test_clone_mixed_receiver_dispatches_runtime_class() {
    let out = compile_and_run(
        r#"<?php
class DynamicCloneValue {
    public string $name = 'source';

    public function __clone(): void {
        $this->name = 'clone';
    }
}

function duplicate_dynamic($value) {
    return clone $value;
}

$source = new DynamicCloneValue();
$clone = duplicate_dynamic($source);
echo $source->name . '|' . $clone->name;
"#,
    );
    assert_eq!(out, "source|clone");
}

/// Verifies a clone whose source is untyped remains gradual when passed to a concrete object parameter.
#[test]
fn test_clone_mixed_result_can_flow_to_concrete_object_parameter() {
    let out = compile_and_run(
        r#"<?php
class GradualCloneValue {
    public string $name = 'ready';
}

function duplicate_gradual($value) {
    return clone $value;
}

function read_gradual_clone(GradualCloneValue $value): string {
    return $value->name;
}

echo read_gradual_clone(duplicate_gradual(new GradualCloneValue()));
"#,
    );
    assert_eq!(out, "ready");
}

/// Verifies name resolution reaches a static method call nested under `clone`.
#[test]
fn test_clone_resolves_nested_static_method_receiver() {
    let out = compile_and_run(
        r#"<?php
namespace CloneResolution;

class CloneStyle {
    public string $name = 'ready';
}

class CloneFactory {
    public static function make(): CloneStyle {
        return new CloneStyle();
    }
}

class CloneConsumer {
    public static function read(): string {
        $copy = clone CloneFactory::make();
        return $copy->name;
    }
}

echo CloneConsumer::read();
"#,
    );
    assert_eq!(out, "ready");
}

/// Verifies a cloned `string` property survives the SOURCE object's next assignment.
///
/// A string is the one owned property payload that carries no refcount: a `string` slot owns an
/// independent `__rt_str_persist` block, every store into one persists, and every release frees
/// outright. The clone called `__rt_incref` on the copied pointer, which retained NOTHING, so the
/// clone and its source shared one block that the source's next assignment freed. This loop —
/// the shape `Symfony\Component\String\ByteString::split()` is written in — printed `delta` for
/// every element and 2-byte garbage once the freed block had been reused. The `array` property is
/// in the fixture on purpose: arrays ARE refcounted, their incref was real, and their surviving
/// while the strings did not is what made the defect look string-specific.
#[test]
fn test_clone_gives_each_copy_its_own_string_property_block() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Chunk {
    public string $string = '';
    public array $tags = [];
}

$chunk = new Chunk();
$out = [];
foreach (['alpha', 'bravo', 'charlie', 'delta'] as $piece) {
    $chunk->string = $piece;
    $chunk->tags = [$piece];
    $out[] = clone $chunk;
}
foreach ($out as $copy) {
    echo $copy->string, "|", $copy->tags[0], "\n";
}
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "alpha|alpha\nbravo|bravo\ncharlie|charlie\ndelta|delta\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies inherited, promoted, and nullable string properties each get their own block too.
///
/// The nullable one is a union, so it keeps the refcounted `Mixed` retain; the inherited and
/// promoted ones are plain `string` slots reached through a different declaration path. Under
/// `--heap-debug` the pre-fix program stopped at "bad refcount" — the incref writing a refcount
/// word a string block does not have — which is the other half of the same defect.
#[test]
fn test_clone_gives_inherited_and_promoted_string_properties_their_own_blocks() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Base {
    public string $tag = 'base';
}
class Node extends Base {
    public array $tags = [];
    public ?string $note = null;

    public function __construct(public string $name = '') {}
}

$n = new Node();
$out = [];
foreach (['alpha', 'bravo', 'charlie', 'delta'] as $chunk) {
    $n->name = $chunk . '!';
    $n->tag = strtoupper($chunk);
    $n->tags = [$chunk];
    $n->note = $chunk === 'bravo' ? null : $chunk . '?';
    $out[] = clone $n;
}
foreach ($out as $o) {
    echo $o->name, "|", $o->tag, "|", $o->tags[0], "|", $o->note ?? 'NULL', "\n";
}
$a = new Node('x');
$b = clone $a;
$a->name = 'changed';
echo $b->name, "|", $a->name, "\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(
        out.stdout,
        "alpha!|ALPHA|alpha|alpha?\nbravo!|BRAVO|bravo|NULL\ncharlie!|CHARLIE|charlie|charlie?\n\
         delta!|DELTA|delta|delta?\nx|changed\n"
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
}

/// Verifies `clone $this` in an inherited method copies the RUNTIME class, not the declaring one.
///
/// This is Symfony's `ServiceLocator::withContext()` shape: the base declares the cloning method,
/// the subclass overrides behaviour, and the clone must keep the subclass -- otherwise the
/// override is silently lost and every property the subclass declares goes with it.
#[test]
fn test_clone_of_this_in_inherited_method_keeps_runtime_subclass() {
    let out = compile_and_run(
        r#"<?php
class BaseBox {
    public string $tag = 'base';

    public function copy(): static {
        return clone $this;
    }
}

class ChildBox extends BaseBox {
    public string $extra = 'x';

    public function label(): string {
        return 'child:' . $this->extra;
    }
}

$source = new ChildBox();
$source->extra = 'kept';
$copy = $source->copy();
echo get_class($copy) . '|' . $copy->label() . '|' . $copy->tag;
"#,
    );
    assert_eq!(out, "ChildBox|child:kept|base");
}

/// Verifies a subclass-only `__clone()` runs exactly once for a clone taken through the base.
#[test]
fn test_clone_through_base_runs_subclass_only_magic_clone_once() {
    let out = compile_and_run(
        r#"<?php
class HookBase {
    public int $n = 1;

    public function copy(): static {
        return clone $this;
    }
}

class HookChild extends HookBase {
    public function __clone(): void {
        echo 'child-hook;';
        $this->n += 10;
    }
}

$source = new HookChild();
$copy = $source->copy();
echo get_class($copy) . '|' . $source->n . '|' . $copy->n;
"#,
    );
    assert_eq!(out, "child-hook;HookChild|1|11");
}

/// Verifies an inherited `__clone()` runs exactly ONCE when the clone dispatches on the subclass.
///
/// The hook has two possible emitters -- the IR lowering's ordinary method call and the runtime
/// clone dispatch's per-candidate call -- and running both would apply it twice.
#[test]
fn test_clone_through_base_runs_inherited_magic_clone_once() {
    let out = compile_and_run(
        r#"<?php
class InhBase {
    public int $n = 1;

    public function __clone(): void {
        echo 'base-hook;';
        $this->n += 100;
    }

    public function copy(): static {
        return clone $this;
    }
}

class InhChild extends InhBase {}

$source = new InhChild();
$copy = $source->copy();
echo get_class($copy) . '|' . $source->n . '|' . $copy->n;
"#,
    );
    assert_eq!(out, "base-hook;InhChild|1|101");
}

/// Verifies an overridden `__clone()` runs the SUBCLASS hook, once, for a clone taken in the base.
#[test]
fn test_clone_through_base_runs_overridden_magic_clone_once() {
    let out = compile_and_run(
        r#"<?php
class OvrBase {
    public int $n = 1;

    public function __clone(): void {
        echo 'ovr-base;';
        $this->n += 1;
    }

    public function copy(): static {
        return clone $this;
    }
}

class OvrChild extends OvrBase {
    public function __clone(): void {
        echo 'ovr-child;';
        $this->n += 2;
    }
}

$child = new OvrChild();
$child_copy = $child->copy();
$base = new OvrBase();
$base_copy = $base->copy();
echo get_class($child_copy) . '|' . $child_copy->n . '|'
    . get_class($base_copy) . '|' . $base_copy->n;
"#,
    );
    assert_eq!(out, "ovr-child;ovr-base;OvrChild|3|OvrBase|2");
}
