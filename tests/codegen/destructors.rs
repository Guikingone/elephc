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

/// A temporary passed to a NULLABLE object parameter is destroyed when the call returns, the
/// same as one passed to a non-nullable parameter.
///
/// `?C` has the Mixed ABI, so the argument is boxed and then run through the nullable nominal
/// guard, which FORWARDS the same cell. The forwarded value is borrowed, so the post-call
/// cleanup — which asks only about the argument itself — released nothing and the box kept the
/// object alive forever. The non-nullable and untyped arms are the controls: neither emits the
/// guard, and both were already correct.
#[test]
fn test_destruct_of_a_temporary_passed_to_a_nullable_object_parameter() {
    let out = compile_and_run(
        r#"<?php
class C {
    public string $id;
    public function __construct(string $id) { $this->id = $id; }
    public function __destruct() { echo "destruct {$this->id}\n"; }
}
function nn(C $c): void { echo "nn\n"; }
function nu(?C $c): void { echo "nu\n"; }
function un($c): void { echo "un\n"; }
nn(new C('nn'));
nu(new C('nu'));
un(new C('un'));
echo "after\n";
"#,
    );
    assert_eq!(
        out,
        "nn\ndestruct nn\nnu\ndestruct nu\nun\ndestruct un\nafter\n"
    );
}

/// The nullable-parameter call leaves a clean heap: the leaked box kept the object live for the
/// whole process, so the ordering assertion alone would not pin the ownership half.
#[test]
fn test_nullable_object_parameter_call_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class C { public function __destruct() { echo "d"; } }
function nu(?C $c): void { echo "n"; }
nu(new C());
echo "|end";
"#,
    );
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected a clean heap, got: {}",
        out.stderr
    );
    assert_eq!(out.stdout, "nd|end");
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

/// A discarded fluent result destructs at the end of its statement even when the method's
/// declared return type forces the receiver to be boxed on the way out.
///
/// `return $this` hands the caller a borrowed receiver, so the return lowering acquires it to
/// balance the caller's release. A declared return type that cannot carry a bare object pointer
/// — a nullable object, `mixed`, or an object union — makes the return coercion box the receiver
/// into a freshly allocated Mixed cell first. That cell already owns its reference and is itself
/// handed over as owned, so the extra acquire landed on the box: the caller's single release
/// could not free it, the object never reached zero, and `__destruct` never ran. Only the
/// object-only return type (`: Cfg`) was ever balanced, which is why a fluent DSL lost exactly
/// the trailing call of a chain — the one whose result nothing else consumes.
#[test]
fn test_destruct_of_a_discarded_fluent_result_boxed_by_its_return_type() {
    let out = compile_and_run(
        r#"<?php
class Other {}
class Cfg {
    public function __construct(private string $id) {}
    public function plain(): Cfg { return $this; }
    public function nullable(): ?Cfg { return $this; }
    public function anything(): mixed { return $this; }
    public function united(): Cfg|Other { return $this; }
    public function __destruct() { echo "drop:" . $this->id . "\n"; }
}
(new Cfg("plain"))->plain();
echo "|";
(new Cfg("nullable"))->nullable();
echo "|";
(new Cfg("anything"))->anything();
echo "|";
(new Cfg("united"))->united();
echo "|end";
"#,
    );
    assert_eq!(
        out,
        "drop:plain\n|drop:nullable\n|drop:anything\n|drop:united\n|end"
    );
}

/// A boxed `return $this` still keeps the receiver alive for the binding that owns it: the box
/// holds its own reference, so dropping the extra acquire must not destruct a live object.
#[test]
fn test_boxed_returned_this_keeps_a_live_receiver_alive() {
    let out = compile_and_run(
        r#"<?php
class Cfg {
    public function __construct(private string $id) {}
    public function nullable(): ?Cfg { return $this; }
    public function anything(): mixed { return $this; }
    public function id(): string { return $this->id; }
    public function __destruct() { echo "drop:" . $this->id . "\n"; }
}
$c = new Cfg("kept");
$c->nullable();
echo "after-nullable:" . $c->id() . "|";
$c->anything();
echo "after-anything:" . $c->id() . "|";
$held = $c->nullable();
unset($c);
echo "held:" . $held->id() . "|";
unset($held);
echo "end";
"#,
    );
    assert_eq!(
        out,
        "after-nullable:kept|after-anything:kept|held:kept|drop:kept\nend"
    );
}

/// A boxed fluent chain destructs in statement order: every link is the same receiver, so the
/// object drops once, when the statement's trailing result is released.
#[test]
fn test_destruct_order_of_a_boxed_fluent_chain() {
    let out = compile_and_run(
        r#"<?php
class Cfg {
    public function __construct(private string $id) {}
    public function step(): ?Cfg { return $this; }
    public function __destruct() { echo "drop:" . $this->id . "\n"; }
}
function make(string $id): Cfg { return new Cfg($id); }
make("a")->step()->step();
echo "|";
make("b")->step();
echo "|end";
"#,
    );
    assert_eq!(out, "drop:a\n|drop:b\n|end");
}

/// The discarded boxed result leaves a clean heap: before the fix the object stayed one
/// reference above zero for the life of the process, so a value assertion alone would not
/// have pinned the ownership balance.
#[test]
fn test_discarded_boxed_fluent_result_leaves_a_clean_heap() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class Other {}
class Cfg {
    public function nullable(): ?Cfg { return $this; }
    public function anything(): mixed { return $this; }
    public function united(): Cfg|Other { return $this; }
    public function __destruct() { echo "d"; }
}
function run(): void {
    (new Cfg())->nullable();
    (new Cfg())->anything();
    (new Cfg())->united();
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
    assert_eq!(out.stdout, "ddd|end");
}

/// An eval context in the enclosing function must not change when a discarded chain is destroyed.
///
/// A function that owns an eval context has every statically typed method call preceded by an
/// eval-bridge probe, and the probe boxes its receiver into a Mixed cell. The bridge only borrows
/// that cell, so the probe still owns it on both exits; when the miss path dropped it the receiver
/// stayed one reference above zero and the destructor never ran at all. `php -n` prints `destruct`
/// before `after chain`, so the ordering — not merely the presence of the line — is the assertion.
#[test]
fn test_destruct_of_discarded_chain_under_an_eval_context() {
    let out = compile_and_run(
        r#"<?php
class Cfg {
    public array $data = [];
    public function __destruct() { echo "destruct\n"; }
    public function args(array $a): static { $this->data = $a; return $this; }
    public function tag(string $t): static { $this->data[] = $t; return $this; }
}
function run(): void {
    eval('$seed = 1;');
    (new Cfg())->args([1, 2])->tag('x');
    echo "after chain\n";
}
run();
echo "end\n";
"#,
    );
    assert_eq!(out, "destruct\nafter chain\nend\n");
}

/// Releasing probe-boxed operands must not touch a cell the bridge handed back as the result.
///
/// An interpreted body may return the very cell it was passed, in which case that cell's
/// reference moved to the result and is no longer the caller's to drop. Releasing it anyway
/// would free `$local` while `$back` still names it, so this fixture pins the surviving object:
/// `$local` is read after the call and its destructor runs once, at scope exit.
#[test]
fn test_eval_call_returning_its_argument_keeps_the_returned_cell() {
    let out = compile_and_run(
        r#"<?php
class T {
    public function __construct(public string $n) {}
    public function __destruct() { echo "destruct {$this->n}\n"; }
}
function run(): void {
    eval('function echo_back($v) { return $v; }');
    $local = new T('local');
    $back = echo_back($local);
    echo "back is " . $back->n . "\n";
    echo "scope end\n";
}
run();
echo "end\n";
"#,
    );
    assert_eq!(out, "back is local\nscope end\ndestruct local\nend\n");
}

/// A `static::` call inside an eval-context function releases the operands its override probe boxed.
///
/// The late-bound static path runs the same borrow-and-drop probe as instance dispatch, so a
/// receiver reaching it through a discarded chain has the same destructor to lose.
#[test]
fn test_destruct_of_discarded_chain_calling_a_late_bound_static() {
    let out = compile_and_run(
        r#"<?php
abstract class Base {
    public static function processValue(mixed $v, bool $flag = false): mixed { return $v; }
}
class Cfg extends Base {
    public mixed $data = null;
    public function __destruct() { echo "destruct\n"; }
    public function args(array $a): static {
        $this->data = static::processValue($a, true);
        return $this;
    }
    public function tag(string $t): static {
        $this->data = static::processValue($t, true);
        return $this;
    }
}
function run(): void {
    eval('$seed = 1;');
    (new Cfg())->args([1, 2])->tag('x');
    echo "after chain\n";
}
run();
echo "end\n";
"#,
    );
    assert_eq!(out, "destruct\nafter chain\nend\n");
}
