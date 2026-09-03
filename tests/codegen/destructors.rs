//! Purpose:
//! End-to-end tests for PHP `__destruct`: the compiler invokes a class's
//! destructor when an object's refcount reaches zero, before its storage is
//! released, across scope exit, overwrite, `unset`, program end, inheritance,
//! container release, and the self-reference re-entrancy guard.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Destructors run inside the object-free path, so these fixtures assert the
//!   exact interleaving of destructor output with surrounding `echo`s. Ordering
//!   among siblings released together reflects the codegen cleanup order and is
//!   asserted as produced.

use crate::support::*;

/// A destructor fires at function scope exit, before control returns to the
/// caller's following statement.
#[test]
fn test_destruct_on_scope_exit() {
    let out = compile_and_run(
        r#"<?php
class Logger {
    public function __destruct() {
        echo "destroyed\n";
    }
}
function run() {
    $x = new Logger();
    echo "inside\n";
}
run();
echo "after\n";
"#,
    );
    assert_eq!(out, "inside\ndestroyed\nafter\n");
}

/// A top-level object's destructor runs at program end, after main's body.
#[test]
fn test_destruct_at_program_end() {
    let out = compile_and_run(
        r#"<?php
class Logger {
    public function __destruct() { echo "bye\n"; }
}
$x = new Logger();
echo "main\n";
"#,
    );
    assert_eq!(out, "main\nbye\n");
}

/// `unset($x)` releasing the last reference runs the destructor immediately.
#[test]
fn test_destruct_on_unset() {
    let out = compile_and_run(
        r#"<?php
class Logger {
    public function __destruct() { echo "gone\n"; }
}
$x = new Logger();
echo "a\n";
unset($x);
echo "b\n";
"#,
    );
    assert_eq!(out, "a\ngone\nb\n");
}

/// Overwriting a variable releases the previous object (running its destructor),
/// and the destructor can read `$this`'s properties.
#[test]
fn test_destruct_on_overwrite_reads_this() {
    let out = compile_and_run(
        r#"<?php
class Logger {
    private string $tag;
    public function __construct(string $t) { $this->tag = $t; }
    public function __destruct() { echo "drop:" . $this->tag . "\n"; }
}
$x = new Logger("first");
$x = new Logger("second");
echo "end\n";
"#,
    );
    assert_eq!(out, "drop:first\nend\ndrop:second\n");
}

/// The same overwrite still releases the previous object when a LATER store widens the
/// variable's frame slot to boxed Mixed.
///
/// `test_destruct_on_overwrite_reads_this` above passes without the trailing `$x = null`
/// because the slot stays a concrete object for the whole function. Adding that store
/// makes codegen lay the slot out as Mixed, so the overwrite's occupant load — lowered
/// earlier, when the slot still looked concrete — becomes an unbox PLUS a retain, and its
/// paired release only gave that retain back: the box owning `first` was overwritten
/// without ever being released, so `drop:first` never printed (php prints it at the second
/// store). `retype_stale_local_load_release_ops` types that release by the slot's FINAL
/// storage instead.
#[test]
fn test_destruct_on_overwrite_when_a_later_store_boxes_the_slot() {
    let out = compile_and_run(
        r#"<?php
class Logger {
    private string $tag;
    public function __construct(string $t) { $this->tag = $t; }
    public function __destruct() { echo "drop:" . $this->tag . "\n"; }
}
$x = new Logger("first");
$x = new Logger("second");
$x = null;
echo "end\n";
"#,
    );
    assert_eq!(out, "drop:first\ndrop:second\nend\n");
}

/// A fluent chain of overwrites inside a function loses EVERY superseded object when a
/// conditional null store boxes the slot — the shape Symfony's `ServiceConfigurator` uses,
/// where each `->set()` supersedes the previous configurator and the destructor is what
/// registers the service definition.
#[test]
fn test_destruct_on_repeated_overwrite_with_a_conditional_null_store() {
    let out = compile_and_run(
        r#"<?php
class Cfg {
    public string $id;
    public function __construct(string $id) { $this->id = $id; }
    public function __destruct() { echo "register {$this->id}\n"; }
}
function build(bool $reset): void {
    $c = new Cfg('a');
    $c = new Cfg('b');
    $c = new Cfg('c');
    if ($reset) { $c = null; }
    echo "built\n";
}
build(true);
echo "after\n";
"#,
    );
    assert_eq!(out, "register a\nregister b\nregister c\nbuilt\nafter\n");
}

/// The boxed-slot overwrite leaves a clean heap: before the fix the superseded object's
/// Mixed box stayed live for the whole process, so the value assertions above alone would
/// not have pinned the ownership half of the fix.
#[test]
fn test_overwrite_of_a_boxed_object_slot_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Item {
    public string $id;
    public function __construct(string $id) { $this->id = $id; }
    public function __destruct() { echo "d{$this->id}"; }
}
function run(): void {
    $x = new Item('1');
    $x = new Item('2');
    $x = null;
}
run();
echo "|end";
"#,
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
    assert_eq!(out.stdout, "d1d2|end");
}

/// A subclass with no destructor inherits its parent's `__destruct`, dispatched
/// to the implementing ancestor's method.
#[test]
fn test_destruct_inherited_from_parent() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function __destruct() { echo "base-dtor\n"; }
}
class Derived extends Base {
}
function run() { $d = new Derived(); }
run();
echo "done\n";
"#,
    );
    assert_eq!(out, "base-dtor\ndone\n");
}

/// An overriding subclass destructor runs instead of the parent's.
#[test]
fn test_destruct_override() {
    let out = compile_and_run(
        r#"<?php
class Base {
    public function __destruct() { echo "base\n"; }
}
class Derived extends Base {
    public function __destruct() { echo "derived\n"; }
}
function run() { $d = new Derived(); }
run();
echo "end\n";
"#,
    );
    assert_eq!(out, "derived\nend\n");
}

/// Objects held in an array are destructed when the array is released.
#[test]
fn test_destruct_objects_in_array() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public function __destruct() { echo "free "; }
}
function run() {
    $arr = [new Item(), new Item()];
    echo "built ";
}
run();
echo "done";
"#,
    );
    assert_eq!(out, "built free free done");
}

/// A class without `__destruct` is unaffected (no spurious calls, no crash), even
/// alongside a class that defines one.
#[test]
fn test_no_destruct_is_noop() {
    let out = compile_and_run(
        r#"<?php
class Plain {
    public int $v;
    public function __construct() { $this->v = 7; }
}
class Loud {
    public function __destruct() { echo "loud "; }
}
function run() {
    $p = new Plain();
    $l = new Loud();
    echo $p->v . " ";
}
run();
echo "end";
"#,
    );
    assert_eq!(out, "7 loud end");
}

/// The re-entrancy guard: a destructor that takes a local copy of `$this` (a
/// balanced incref then scope-exit decref) must not re-enter the free path or
/// double-run. The destructor runs exactly once.
#[test]
fn test_destruct_self_reference_guard() {
    let out = compile_and_run(
        r#"<?php
class Tricky {
    public function __destruct() {
        $tmp = $this;
        echo "x";
    }
}
function run() { $t = new Tricky(); }
run();
echo "|ok";
"#,
    );
    assert_eq!(out, "x|ok");
}

/// A destructor that releases a heap-backed property (a string) before the
/// object's own storage is freed runs cleanly.
#[test]
fn test_destruct_with_heap_property() {
    let out = compile_and_run(
        r#"<?php
class Holder {
    private array $items;
    public function __construct() { $this->items = ["a", "b", "c"]; }
    public function __destruct() { echo "count:" . count($this->items); }
}
function run() { $h = new Holder(); }
run();
echo "|fin";
"#,
    );
    assert_eq!(out, "count:3|fin");
}

/// A dynamically required temporary runs its destructor and can publish into an AOT receiver.
#[test]
fn test_dynamic_required_temporary_destructor_mutates_aot_receiver() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
class PublicationLog {
    public string $value = '';

    public function publish(string $value): void {
        $this->value = $value;
    }
}

$log = new PublicationLog();
$path = __DIR__ . '/dynamic.php';
require $path;
stage_publication($log);
echo $log->value;
"#,
            ),
            (
                "dynamic.php",
                r#"<?php
class DeferredPublication {
    private PublicationLog $log;

    public function __construct(PublicationLog $log) {
        $this->log = $log;
    }

    public function __destruct() {
        $this->log->publish('released');
    }
}

function stage_publication(PublicationLog $log): void {
    new DeferredPublication($log);
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "released");
}

/// A dynamic include releases an AOT temporary and runs its AOT destructor before returning.
#[test]
fn test_dynamic_required_aot_temporary_runs_aot_destructor() {
    let out = compile_cli_files_and_run(
        &[
            (
                "entry.php",
                r#"<?php
class AotPublicationLog {
    public string $value = '';
}

class AotDeferredPublication {
    private AotPublicationLog $log;

    public function __construct(AotPublicationLog $log) {
        $this->log = $log;
    }

    public function __destruct() {
        $this->log->value = 'released';
    }
}

$log = new AotPublicationLog();
$path = __DIR__ . '/dynamic.php';
require $path;
stage_aot_publication($log);
echo $log->value;
"#,
            ),
            (
                "dynamic.php",
                r#"<?php
function stage_aot_publication(AotPublicationLog $log): void {
    new AotDeferredPublication($log);
}
"#,
            ),
        ],
        "entry.php",
    );
    assert_eq!(out, "released");
}

/// Storing a fresh object into a NULLABLE object property still releases the producer's
/// reference, so the object's refcount can reach zero and its destructor runs.
///
/// A `?C` slot has the Mixed representation, so the assignment goes through the runtime
/// nominal-class guard. That guard borrows its operand and hands back its own reference —
/// its codegen increfs the payload precisely because its EIR result is declared owned — so
/// the producer's reference is dead once the guard has run. It was not being released, which
/// left every such object one reference above zero: `__destruct` never ran and the payload
/// leaked. A non-nullable `C` slot never took that path and was always balanced.
#[test]
fn test_destruct_after_store_into_a_nullable_object_property() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public function __construct(private string $id) {}
    public function __destruct() { echo "drop:" . $this->id . "\n"; }
}
class Nullable { public ?Item $p = null; }
class Plain { public Item $p; }

function nulled(): void {
    $h = new Nullable();
    $h->p = new Item("nulled");
    $h->p = null;
}
function replaced(): void {
    $h = new Nullable();
    $h->p = new Item("first");
    $h->p = new Item("second");
}
function plain(): void {
    $h = new Plain();
    $h->p = new Item("plain");
}
nulled();
echo "|";
replaced();
echo "|";
plain();
echo "|end";
"#,
    );
    assert_eq!(
        out,
        "drop:nulled\n|drop:first\ndrop:second\n|drop:plain\n|end"
    );
}

/// The same store leaves a clean heap: the leaked reference above kept the payload alive for
/// the whole process, so a value assertion alone would not have pinned the ownership fix.
#[test]
fn test_nullable_object_property_store_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Item { public function __destruct() { echo "d"; } }
class Nullable { public ?Item $p = null; }
function run(): void {
    $h = new Nullable();
    $h->p = new Item();
}
run();
echo "|end";
"#,
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
    assert_eq!(out.stdout, "d|end");
}
