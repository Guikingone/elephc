//! Purpose:
//! Integration or regression tests for end-to-end codegen coverage of object property mutations, including class array of objects property access, class property array push, and class property array assign.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Inline PHP fixtures are compiled to native binaries and assertions compare stdout or expected failures.

use super::*;

/// Verifies prefix increment on a property returns the updated value while mutating the slot.
#[test]
fn test_prefix_increment_property_expression_returns_updated_value() {
    let out = compile_and_run(
        r#"<?php
class Counter { public int $value = 0; }
$counter = new Counter();
if (++$counter->value === 1) { echo 'yes'; }
echo ':' . $counter->value;
"#,
    );
    assert_eq!(out, "yes:1");
}

/// A prefix increment in an assignment RHS is not misclassified as a discarded postfix statement.
#[test]
fn test_prefix_increment_this_property_after_concat_returns_updated_value() {
    let out = compile_and_run(
        r#"<?php
class InlineIdCounter {
    public string $currentId = 'service';
    public int $counter = 0;
    public function nextId(): string {
        $id = '.autowire_inline.'.$this->currentId.'.'.++$this->counter;
        return $id;
    }
}
echo (new InlineIdCounter())->nextId();
"#,
    );
    assert_eq!(out, ".autowire_inline.service.1");
}

/// Verifies coercive typed-property assignment invokes `__toString()` and stores
/// the resulting string rather than the source object representation.
#[test]
fn test_string_property_assignment_accepts_stringable_object() {
    let out = compile_and_run(
        r#"<?php
class PropertyText {
    public function __toString(): string {
        return "converted";
    }
}

class PropertyTextHolder {
    public string $value = "";

    public function fill(): void {
        $this->value = new PropertyText();
    }
}

$holder = new PropertyTextHolder();
$holder->fill();
echo $holder->value;
"#,
    );
    assert_eq!(out, "converted");
}

/// Verifies an interface-typed source is checked against a narrower nominal
/// property at runtime, accepting matching objects and throwing for siblings.
#[test]
fn test_nominal_property_assignment_guards_interface_source() {
    let out = compile_and_run(
        r#"<?php
interface PropertyPass {}
class ExpectedPropertyPass implements PropertyPass {}
class OtherPropertyPass implements PropertyPass {}

class NominalPropertyHolder {
    private ExpectedPropertyPass $pass;

    public function set(PropertyPass $pass): void {
        $this->pass = $pass;
    }

    public function valid(): bool {
        return $this->pass instanceof ExpectedPropertyPass;
    }
}

$holder = new NominalPropertyHolder();
$holder->set(new ExpectedPropertyPass());
echo $holder->valid() ? "ok" : "bad";
try {
    $holder->set(new OtherPropertyPass());
} catch (TypeError $error) {
    echo ":type";
}
"#,
    );
    assert_eq!(out, "ok:type");
}

/// Verifies null coalescing probes a statically undeclared property without a
/// compile error or runtime read, while still evaluating the receiver once.
#[test]
fn test_null_coalesce_accepts_missing_declared_property() {
    let out = compile_and_run(
        r#"<?php
class OptionalPropertyOwner {
    public static int $constructed = 0;

    public function __construct() {
        self::$constructed++;
    }
}

echo (new OptionalPropertyOwner())->missing ?? "fallback";
echo ":", OptionalPropertyOwner::$constructed;
"#,
    );
    assert_eq!(out, "fallback:1");
}

/// Verifies `??` and `??=` treat an uninitialized typed property as null before reading it.
#[test]
fn test_null_coalesce_initializes_uninitialized_typed_property() {
    let out = compile_and_run(
        r#"<?php
class LazyTypedProperty {
    private readonly stdClass $value;

    public function peek(): ?object {
        return $this->value ?? null;
    }

    public function resolve(): object {
        return $this->value ??= new stdClass();
    }
}

$box = new LazyTypedProperty();
var_dump($box->peek());
echo get_class($box->resolve()), ':', get_class($box->resolve());
"#,
    );
    assert_eq!(out, "NULL\nstdClass:stdClass");
}

/// Verifies `??=` keeps working when a nullable object parameter uses the gradual receiver path.
#[test]
fn test_null_coalesce_assignment_on_nullable_object_receiver() {
    let out = compile_and_run(
        r#"<?php
class GradualCoalesceBox {
    public ?string $name = null;
}

function resolveGradualName(?GradualCoalesceBox $box): string {
    return $box->name ??= 'fallback';
}

$box = new GradualCoalesceBox();
echo resolveGradualName($box), ':', resolveGradualName($box);
"#,
    );
    assert_eq!(out, "fallback:fallback");
}

/// Verifies a gradual union crossing a nullable nominal property is checked at runtime.
#[test]
fn test_guarded_union_assignment_to_nullable_nominal_property() {
    let out = compile_and_run(
        r#"<?php
class NominalBase {}
class NominalChild extends NominalBase {}
class NominalHolder {
    public ?NominalChild $value = null;
    public function set(NominalBase|string $candidate): void {
        $isChild = $candidate instanceof NominalChild;
        $this->value = $isChild ? $candidate : null;
    }
}
$holder = new NominalHolder();
$holder->set(new NominalChild());
echo get_class($holder->value);
"#,
    );
    assert_eq!(out, "NominalChild");
}

/// Verifies `foreach ($this)` iterates ordinary object properties visible in class scope.
#[test]
fn test_foreach_plain_object_properties_inside_class() {
    let out = compile_and_run(
        r#"<?php
class PropertyIterationProbe {
    public int $publicValue = 1;
    protected int $protectedValue = 2;
    private int $privateValue = 3;

    public function dump(): void {
        foreach ($this as $name => $value) {
            echo $name, '=', $value, ';';
        }
    }
}
(new PropertyIterationProbe())->dump();
"#,
    );
    assert_eq!(out, "publicValue=1;protectedValue=2;privateValue=3;");
}

/// Compiles a loop over an array of class instances, reading the `price` field
/// of each `Item` object via `$items[$i]->price` and accumulating the sum.
#[test]
fn test_class_array_of_objects_property_access() {
    let out = compile_and_run(
        r#"<?php
class Item {
    public $name;
    public $price;
    public function __construct($n, $p) { $this->name = $n; $this->price = $p; }
}
$items = [];
$items[] = new Item("Apple", 1);
$items[] = new Item("Banana", 2);
$total = 0;
for ($i = 0; $i < count($items); $i++) {
    $total = $total + $items[$i]->price;
}
echo $total;
"#,
    );
    assert_eq!(out, "3");
}

/// Exercises `$this->items[] = $value` (push operator) on a class property
/// that holds an array, verifying the pushed element is retrievable at the
/// correct index.
#[test]
fn test_class_property_array_push() {
    let out = compile_and_run(
        r#"<?php
class Bucket {
    public $items;

    public function __construct() {
        $this->items = [1, 2];
    }

    public function add($value) {
        $this->items[] = $value;
    }

    public function last(): int {
        return $this->items[2];
    }
}

$bucket = new Bucket();
$bucket->add(7);
echo $bucket->last();
"#,
    );
    assert_eq!(out, "7");
}

/// Exercises indexed write `$this->items[0] = $value` on a class property
/// that holds an array, verifying the replaced element is retrieved correctly.
#[test]
fn test_class_property_array_assign() {
    let out = compile_and_run(
        r#"<?php
class Bucket {
    public $items;

    public function __construct() {
        $this->items = [1, 2, 3];
    }

    public function replaceFirst($value) {
        $this->items[0] = $value;
    }

    public function first(): int {
        return $this->items[0];
    }
}

$bucket = new Bucket();
$bucket->replaceFirst(9);
echo $bucket->first();
"#,
    );
    assert_eq!(out, "9");
}

/// Verifies assigning an untyped function parameter into a typed object property.
#[test]
fn test_typed_int_property_accepts_untyped_function_param_assignment() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public int $n = 0;
}

function set_n(Box $box, $value): void {
    $box->n = $value;
}

$box = new Box();
set_n($box, 7);
echo $box->n;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies that a typed `public array $headers` property (initialized to `[]`)
/// accepts a string-keyed assignment (`$this->headers["Host"] = ...`) and the
/// value is retrievable via the same key.
#[test]
fn test_typed_array_property_accepts_string_key_assignment() {
    let out = compile_and_run(
        r#"<?php
class Req {
    public array $headers;

    public function __construct() {
        $this->headers = [];
        $this->headers["Host"] = "example.com";
    }
}

$r = new Req();
echo $r->headers["Host"];
"#,
    );
    assert_eq!(out, "example.com");
}

/// Verifies dynamic property writes through a `mixed` receiver can assign
/// method-computed names and values into declared mixed object properties.
#[test]
fn test_dynamic_property_set_on_mixed_receiver_from_method_values() {
    let out = compile_and_run(
        r#"<?php
class Row {
    public mixed $id;
    public mixed $name;
}

class Hydrator {
    private function value(int $i): mixed {
        if ($i == 0) {
            return 1;
        }
        return "Ada";
    }

    private function column(int $i): string {
        if ($i == 0) {
            return "id";
        }
        return "name";
    }

    public function fill(mixed $object): mixed {
        $_name = $this->column(0);
        $object->{$_name} = $this->value(0);
        $_name = $this->column(1);
        $object->{$_name} = $this->value(1);
        return $object;
    }
}

$row = (new Hydrator())->fill(new Row());
echo (($row instanceof Row) ? "Row" : "not-row") . ":" . $row->id . ":" . $row->name;
"#,
    );
    assert_eq!(out, "Row:1:Ada");
}

/// Verifies dynamic property writes through a `mixed` receiver preserve mixed
/// string values built by repeated concatenation before assignment.
#[test]
fn test_dynamic_property_set_on_mixed_receiver_with_concat_built_string() {
    let out = compile_and_run(
        r#"<?php
class Row {
    public mixed $id;
    public mixed $name;
}

class Hydrator {
    private function value(int $i): mixed {
        if ($i == 0) {
            return 1;
        }
        $_out = "";
        $_out = $_out . chr(65);
        $_out = $_out . chr(100);
        $_out = $_out . chr(97);
        return $_out;
    }

    private function column(int $i): string {
        if ($i == 0) {
            return "id";
        }
        return "name";
    }

    public function fill(mixed $object): mixed {
        $_name = $this->column(0);
        $object->{$_name} = $this->value(0);
        $_name = $this->column(1);
        $object->{$_name} = $this->value(1);
        return $object;
    }
}

$row = (new Hydrator())->fill(new Row());
echo (($row instanceof Row) ? "Row" : "not-row") . ":" . $row->id . ":" . $row->name;
"#,
    );
    assert_eq!(out, "Row:1:Ada");
}

/// Verifies dynamic property writes accept runtime-built property names and
/// runtime-built mixed string values when hydrating a declared object.
#[test]
fn test_dynamic_property_set_on_mixed_receiver_with_runtime_name_and_value() {
    let out = compile_and_run(
        r#"<?php
class Row {
    public mixed $id;
    public mixed $name;
}

class Hydrator {
    private function value(int $i): mixed {
        if ($i == 0) {
            return 1;
        }
        $_out = "";
        $_out = $_out . chr(65);
        $_out = $_out . chr(100);
        $_out = $_out . chr(97);
        return $_out;
    }

    private function column(int $i): string {
        $_name = "";
        if ($i == 0) {
            $_name = $_name . chr(105);
            $_name = $_name . chr(100);
            return $_name;
        }
        $_name = $_name . chr(110);
        $_name = $_name . chr(97);
        $_name = $_name . chr(109);
        $_name = $_name . chr(101);
        return $_name;
    }

    public function fill(mixed $object): mixed {
        $_name = $this->column(0);
        $object->{$_name} = $this->value(0);
        $_name = $this->column(1);
        $object->{$_name} = $this->value(1);
        return $object;
    }
}

$row = (new Hydrator())->fill(new Row());
echo (($row instanceof Row) ? "Row" : "not-row") . ":" . $row->id . ":" . $row->name;
"#,
    );
    assert_eq!(out, "Row:1:Ada");
}

/// Verifies a prelude-style hydrator can instantiate from a mixed class-string
/// parameter and then assign runtime dynamic property names into the object.
#[test]
fn test_dynamic_property_set_after_mixed_dynamic_instantiation() {
    let out = compile_and_run(
        r#"<?php
class Row {
    public mixed $id;
    public mixed $name;
}

class Hydrator {
    private function value(int $i): mixed {
        if ($i == 0) {
            return 1;
        }
        $_out = "";
        $_out = $_out . chr(65);
        $_out = $_out . chr(100);
        $_out = $_out . chr(97);
        return $_out;
    }

    private function column(int $i): string {
        $_name = "";
        if ($i == 0) {
            $_name = $_name . chr(105);
            $_name = $_name . chr(100);
            return $_name;
        }
        $_name = $_name . chr(110);
        $_name = $_name . chr(97);
        $_name = $_name . chr(109);
        $_name = $_name . chr(101);
        return $_name;
    }

    private function assign(mixed $object): mixed {
        $_name = $this->column(0);
        $object->{$_name} = $this->value(0);
        $_name = $this->column(1);
        $object->{$_name} = $this->value(1);
        return $object;
    }

    public function fetch(mixed $classOrObject = null): mixed {
        return $this->assign(new $classOrObject());
    }
}

$row = (new Hydrator())->fetch(Row::class);
echo (($row instanceof Row) ? "Row" : "not-row") . ":" . $row->id . ":" . $row->name;
"#,
    );
    assert_eq!(out, "Row:1:Ada");
}

/// Verifies runtime-named reads and writes dispatch across concrete classes hidden by `object`.
#[test]
fn test_dynamic_property_read_write_through_generic_object_parameter() {
    let out = compile_and_run(
        r#"<?php
class FirstDynamicPropertyTarget {
    public int $value = 1;
}

class SecondDynamicPropertyTarget {
    public int $value = 2;
}

function replaceDynamicProperty(object $object, string $name, int $value): int {
    $object->{$name} = $value;
    return $object->{$name};
}

echo replaceDynamicProperty(new FirstDynamicPropertyTarget(), "value", 3);
echo ":";
echo replaceDynamicProperty(new SecondDynamicPropertyTarget(), "value", 4);
"#,
    );
    assert_eq!(out, "3:4");
}

/// Verifies that an untyped `public $headers = []` property (array default)
/// accepts a string-keyed assignment (`$r->headers["Host"] = ...`) and the
/// value is retrievable via the same key.
#[test]
fn test_empty_array_property_default_accepts_string_key_assignment() {
    let out = compile_and_run(
        r#"<?php
class Req {
    public $headers = [];
}

$r = new Req();
$r->headers["Host"] = "example.com";
echo $r->headers["Host"];
"#,
    );
    assert_eq!(out, "example.com");
}

/// Exercises `+=` and `*=` compound assignment on a `public $value` property,
/// verifying the result is `10 + 5 = 15`, then `15 * 3 = 45`.
#[test]
fn test_class_property_compound_assign() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public $value = 10;
}

$counter = new Counter();
$counter->value += 5;
$counter->value *= 3;
echo $counter->value;
"#,
    );
    assert_eq!(out, "45");
}

/// Regression test: when the receiver of a compound property assignment is a
/// function call (`passthrough($counter)->value += 5`), the function must be
/// evaluated exactly once, not twice. Verifies output is `"r:15"` (not `"rr:15"`).
#[test]
fn test_class_property_compound_assign_evaluates_receiver_once() {
    let out = compile_and_run(
        r#"<?php
class Counter {
    public $value = 10;
}

function passthrough($counter) {
    echo "r";
    return $counter;
}

$counter = new Counter();
passthrough($counter)->value += 5;
echo ":" . $counter->value;
"#,
    );
    assert_eq!(out, "r:15");
}

/// Exercises `+=` and `>>=` compound assignment on an indexed class property
/// (`$bucket->items[1] += 6` and `$bucket->items[2] >>= 1`), verifying the
/// results are `4 + 6 = 10` and `8 >> 1 = 4`.
#[test]
fn test_class_property_array_compound_assign() {
    let out = compile_and_run(
        r#"<?php
class Bucket {
    public $items = [2, 4, 8];
}

$bucket = new Bucket();
$bucket->items[1] += 6;
$bucket->items[2] >>= 1;
echo $bucket->items[1] . "|" . $bucket->items[2];
"#,
    );
    assert_eq!(out, "10|4");
}

/// Regression test: when the receiver of an indexed compound property assignment
/// is a function call (`passthrough($bucket)->items[idx()] -= 3`), both the
/// receiver and the index expression must be evaluated exactly once each.
/// Verifies output is `"ri:5"` (not `"riri:5"` or similar).
#[test]
fn test_class_property_array_compound_assign_evaluates_receiver_and_index_once() {
    let out = compile_and_run(
        r#"<?php
class Bucket {
    public $items = [2, 4, 8];
}

function passthrough($bucket) {
    echo "r";
    return $bucket;
}

function idx() {
    echo "i";
    return 2;
}

$bucket = new Bucket();
passthrough($bucket)->items[idx()] -= 3;
echo ":" . $bucket->items[2];
"#,
    );
    assert_eq!(out, "ri:5");
}

/// Verifies that `??=` on a `readonly` property that has already been initialized
/// does not invoke the fallback expression and preserves the existing value (`7`).
#[test]
fn test_readonly_property_null_coalesce_assignment_keeps_initialized_value() {
    let out = compile_and_run(
        r#"<?php
class Box {
    public readonly int $value;

    public function __construct() {
        $this->value = 7;
    }
}

function fallback() {
    echo "fallback";
    return 9;
}

$box = new Box();
$box->value ??= fallback();
echo $box->value;
"#,
    );
    assert_eq!(out, "7");
}

/// Verifies `unset($obj->prop)` on a declared (typed) property.
///
/// PHP leaves the property UNINITIALIZED rather than nulled: `isset()` answers false,
/// `print_r` omits it, reading it raises `Error: Typed property … must not be accessed
/// before initialization`, and assigning again brings it back. `unset($a, $b)` clears
/// both targets.
#[test]
fn test_unset_declared_typed_property_leaves_it_uninitialized() {
    let out = compile_and_run(
        r#"<?php
class T { public int $n = 3; public string $s = "x"; public array $a = [1, 2]; }
$t = new T();
unset($t->n, $t->s);
var_dump(isset($t->n), isset($t->s), isset($t->a));
print_r($t);
try { echo $t->n; } catch (\Error $e) { echo "ERR:", $e->getMessage(), "\n"; }
$t->n = 9;
var_dump(isset($t->n), $t->n);
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(false)\nbool(true)\n\
         T Object\n(\n    [a] => Array\n        (\n            [0] => 1\n            [1] => 2\n        )\n\n)\n\
         ERR:Typed property T::$n must not be accessed before initialization\n\
         bool(true)\nint(9)\n"
    );
}

/// Verifies `unset()` on a property the caller cannot see still routes to `__unset`.
///
/// PHP calls `__unset` only for an INACCESSIBLE (or absent) property; a property the
/// caller can see is removed directly and `__unset` is never consulted.
#[test]
fn test_unset_inaccessible_property_calls_magic_unset() {
    let out = compile_and_run(
        r#"<?php
class Pv {
    private $secret = 1;
    public int $open = 2;
    public function __unset($k) { echo "magic:$k\n"; }
}
$p = new Pv();
unset($p->secret);
unset($p->open);
var_dump(isset($p->open));
"#,
    );
    assert_eq!(out, "magic:secret\nbool(false)\n");
}

/// Verifies `unset($std->prop)` on a `stdClass` really REMOVES the dynamic property.
///
/// Every `stdClass` property is a hash entry, so PHP's removal semantics are exact here:
/// `isset()` answers false, `json_encode()` stops listing the key, unsetting the same key
/// again and unsetting a key that was never set are both no-ops, a later write re-appends
/// the key at the END of the property order, `unset($o->b, $o->c)` removes both, and a read
/// of the removed name answers null (observed through `??`, so the fixture does not depend
/// on the undefined-property warning elephc does not yet emit for `stdClass`).
///
/// Expected output is `LC_ALL=C php 8.4.20` verbatim. The fixture deliberately avoids
/// `var_dump($o)`/`print_r($o)`: elephc renders a `stdClass` body as empty regardless of
/// `unset()`, a separate pre-existing gap.
#[test]
fn test_unset_stdclass_dynamic_property_removes_it() {
    let out = compile_and_run(
        r#"<?php
$o = new stdClass();
$o->a = 1;
$o->b = "two";
$o->c = 3;
unset($o->a);
var_dump(isset($o->a), isset($o->b));
echo json_encode($o), "\n";
unset($o->a);
unset($o->never);
echo json_encode($o), "\n";
$o->a = 9;
echo json_encode($o), "\n";
echo $o->a, "|", $o->b, "\n";
unset($o->b, $o->c);
echo json_encode($o), "\n";
var_dump($o->b ?? "gone");
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(true)\n\
         {\"b\":\"two\",\"c\":3}\n\
         {\"b\":\"two\",\"c\":3}\n\
         {\"b\":\"two\",\"c\":3,\"a\":9}\n\
         9|two\n\
         {\"a\":9}\n\
         string(4) \"gone\"\n"
    );
}

/// Verifies `unset()` of an UNDECLARED name on an `#[AllowDynamicProperties]` class removes
/// the hash entry while leaving the class's fixed slots untouched.
///
/// The receiver mixes both storage shapes: `$fixed` is a declared typed slot and `$x`/`$y`
/// are dynamic hash entries. Unsetting the dynamic names must not disturb `$fixed`, and
/// repeat/absent unsets stay no-ops. Expected output is `LC_ALL=C php 8.4.20` verbatim.
#[test]
fn test_unset_dynamic_property_on_allow_dynamic_class() {
    let out = compile_and_run(
        r#"<?php
#[AllowDynamicProperties]
class Bag { public int $fixed = 7; }
$b = new Bag();
$b->x = 1;
$b->y = "two";
unset($b->x);
var_dump(isset($b->x), isset($b->y), isset($b->fixed));
unset($b->x);
unset($b->missing);
$b->x = 5;
var_dump(isset($b->x));
echo $b->x, "|", $b->y, "|", $b->fixed, "\n";
unset($b->x, $b->y);
var_dump(isset($b->x), isset($b->y), isset($b->fixed));
echo $b->fixed, "\n";
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nbool(true)\nbool(true)\n\
         bool(true)\n\
         5|two|7\n\
         bool(false)\nbool(false)\nbool(true)\n\
         7\n"
    );
}

/// Regression: repeatedly reading the SAME dynamic property must keep answering its value.
///
/// `__rt_hash_get` only borrows the stored `Mixed` cell, but the dynamic-property read
/// hands its result to a caller that releases it, so a missing retain made every read drop
/// a reference the program never took. After enough reads the live hash entry was freed and
/// further reads answered `NULL` — a use-after-free of the property's storage.
/// Expected output is `LC_ALL=C php 8.4.20` verbatim.
#[test]
fn test_repeated_dynamic_property_reads_keep_the_value_alive() {
    let out = compile_and_run(
        r#"<?php
#[AllowDynamicProperties]
class Slot {}
$s = new Slot();
$s->v = "kept";
echo $s->v, $s->v, $s->v, "\n";
var_dump($s->v, $s->v);
var_dump($s->v);
"#,
    );
    assert_eq!(
        out,
        "keptkeptkept\nstring(4) \"kept\"\nstring(4) \"kept\"\nstring(4) \"kept\"\n"
    );
}

/// Verifies `unset()` of an UNTYPED declared property is refused with a diagnostic that
/// names that shape, instead of silently leaving a stale value behind.
///
/// PHP genuinely removes such a property: a later read warns `Undefined property` and
/// answers `null`. elephc gives each declared property a fixed, monomorphically typed slot
/// (here `Int`), which has no encoding for "removed, and reading as null" — see
/// `docs/php/classes.md`. A loud compile error beats a wrong value.
#[test]
fn test_unset_untyped_declared_property_is_rejected() {
    let error = compile_source_expect_backend_error(
        r#"<?php
class M { public $foo = 1; }
$m = new M();
unset($m->foo);
echo "ok";
"#,
    );
    assert!(
        error.contains("An UNTYPED declared property"),
        "the diagnostic must name the untyped-property shape, got: {}",
        error
    );
}

/// Verifies `unset()` of a BY-REFERENCE property is refused rather than silently skipped.
///
/// The slot holds an object-owned ref-cell pointer that the destructor still frees and that
/// a later write would write THROUGH, reviving the alias PHP's `unset()` just broke. The
/// backend used to skip the shape quietly, which left `isset()` answering `true` after an
/// `unset()` where PHP answers `false`.
#[test]
fn test_unset_by_reference_property_is_rejected() {
    let error = compile_source_expect_backend_error(
        r#"<?php
class R { public function __construct(public int &$p) {} }
$v = 3;
$r = new R($v);
unset($r->p);
"#,
    );
    assert!(
        error.contains("unset() of by-reference property R::$p"),
        "the diagnostic must name the by-reference property, got: {}",
        error
    );
}

/// Verifies `unset()` of a dynamic name on a class that declares `__unset()` is refused.
///
/// PHP consults `__unset()` only when the dynamic property is ABSENT at the unset site and
/// removes the entry silently when it is present — a choice that depends on runtime state.
/// elephc picks the lowering statically, so it declines rather than guessing one of the two
/// behaviors.
#[test]
fn test_unset_dynamic_property_with_magic_unset_is_rejected() {
    let error = compile_source_expect_backend_error(
        r#"<?php
#[AllowDynamicProperties]
class Hooked { public function __unset($n) { echo "magic:$n\n"; } }
$h = new Hooked();
$h->a = 1;
unset($h->a);
"#,
    );
    assert!(
        error.contains("unset target shape"),
        "the runtime-dependent __unset shape must be refused, got: {}",
        error
    );
}

/// Verifies `isset()` on a never-initialized typed property answers false instead of
/// raising the uninitialized-read error, matching PHP.
#[test]
fn test_isset_on_uninitialized_typed_property_is_false() {
    let out = compile_and_run(
        r#"<?php
class U { public ?int $v; public int $w = 1; }
$u = new U();
var_dump(isset($u->v), isset($u->w));
$u->v = 5;
var_dump(isset($u->v), $u->v);
"#,
    );
    assert_eq!(out, "bool(false)\nbool(true)\nbool(true)\nint(5)\n");
}

/// Verifies an inherited method sees its private typed slot as uninitialized on a child object.
#[test]
fn test_isset_on_inherited_private_uninitialized_typed_property_is_false() {
    let out = compile_and_run(
        r#"<?php
class PrivateTypedBase {
    private string $path;
    public function path(): string {
        if (!isset($this->path)) {
            $this->path = "initialized";
        }
        return $this->path;
    }
}
class PrivateTypedMiddle extends PrivateTypedBase {
    public string $middle = "middle";
}
final class PrivateTypedChild extends PrivateTypedMiddle {
    public string $child = "child";
}
echo (new PrivateTypedChild())->path();
"#,
    );
    assert_eq!(out, "initialized");
}

/// Verifies a bare `object` receiver selects the runtime class when property names collide.
#[test]
fn test_generic_object_property_write_dispatches_by_runtime_class() {
    let out = compile_and_run(
        r#"<?php
class TextPropertyOwner {
    public string $value = '';
}
class ObjectPropertyOwner {
    public object $value;
}
function assignObjectProperty(object $owner, object $value): void {
    $owner->value = $value;
}
$owner = new ObjectPropertyOwner();
assignObjectProperty($owner, new stdClass());
echo $owner->value instanceof stdClass ? 'ok' : 'bad';
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies an array write through a bare `object` receiver dispatches an anonymous class property
/// before applying ArrayAccess offset assignment semantics.
#[test]
fn test_generic_object_anonymous_property_supports_array_writes() {
    let out = compile_and_run(
        r#"<?php
class AnonymousArrayPropertyFactory {
    public function fill(object $result, ArrayObject $storage): void {
        $result->values = $storage;
        $result->values[0] = 'first';
    }
}

$storage = new ArrayObject();
$value = new class {
    public $values;
};
(new AnonymousArrayPropertyFactory())->fill($value, $storage);
echo $storage[0];
"#,
    );
    assert_eq!(out, "first");
}

/// Verifies a nullable attribute instance keeps its runtime class when a same-named property on
/// another class would otherwise make the static slot guess incompatible with the assigned value.
#[test]
fn test_nullable_attribute_property_write_falls_back_to_runtime_class() {
    let out = compile_and_run(
        r#"<?php
class RequestContextCollision {
    public string $method = '';
}

#[Attribute(Attribute::TARGET_METHOD)]
class MethodMarker {
    private ReflectionMethod $method;

    public static function from(ReflectionMethod $method): ?self {
        /** @var self|null $self */
        if (!$self = ($method->getAttributes(self::class)[0] ?? null)?->newInstance()) {
            return null;
        }
        $self->method = $method;
        return $self;
    }

    public function name(): string {
        return $this->method->getName();
    }
}

class MarkedHandler {
    #[MethodMarker]
    public function run(): void {}
}

$marker = MethodMarker::from(new ReflectionMethod(MarkedHandler::class, 'run'));
echo $marker?->name();
"#,
    );
    assert_eq!(out, "run");
}

/// Verifies append on a nullable array property mutates its boxed property cell in place and
/// autovivifies the initial null value before retaining subsequent elements.
#[test]
fn test_nullable_array_property_push_autovivifies_mixed_cell() {
    let out = compile_and_run(
        r#"<?php
class NullableRows {
    private ?array $rows = null;

    public function push(string $value): void {
        $this->rows[] = $value;
    }

    public function joined(): string {
        return implode(',', $this->rows);
    }
}

$rows = new NullableRows();
$rows->push('first');
$rows->push('second');
echo $rows->joined();
"#,
    );
    assert_eq!(out, "first,second");
}

/// Verifies that assigning `[]` to an associative instance property preserves hash storage.
#[test]
fn test_empty_array_resets_associative_instance_property() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public array $items = ['seed' => 1];

    public function reset(): void {
        $this->items = [];
    }
}
$registry = new Registry();
$registry->reset();
$registry->items['next'] = 2;
echo count($registry->items), ':', $registry->items['next'];
"#,
    );
    assert_eq!(out, "1:2");
}

/// Verifies that indexed values assigned to an associative property are promoted to hash storage.
#[test]
fn test_indexed_array_replaces_associative_instance_property() {
    let out = compile_and_run(
        r#"<?php
class Registry {
    public array $items = ['seed' => 1];

    public function replace(): void {
        $this->items = [2, 3];
    }
}
$registry = new Registry();
$registry->replace();
echo count($registry->items), ':', $registry->items[0], ':', $registry->items[1];
"#,
    );
    assert_eq!(out, "2:2:3");
}

/// Verifies an array property refined by string-key writes can later receive indexed storage;
/// numeric keys must survive the representation change alongside subsequent string keys.
#[test]
fn test_indexed_array_replaces_string_key_refined_property() {
    let out = compile_and_run(
        r#"<?php
class RefinedRegistry {
    private array $items = [];

    public function set(string $key, mixed $value): void {
        $this->items[$key] = $value;
    }

    public function replace(array $items): void {
        $this->items = $items;
    }

    public function values(): array {
        return $this->items;
    }
}
$registry = new RefinedRegistry();
$registry->set('seed', 1);
$registry->replace([2, 3]);
$registry->set('tail', 4);
$values = $registry->values();
echo count($values), ':', $values[0], ':', $values[1], ':', $values['tail'];
"#,
    );
    assert_eq!(out, "3:2:3:4");
}

/// Verifies runtime `mixed` property names use PHP string coercion for both declared-property
/// writes and reads instead of reaching the backend as an unmaterializable boxed name.
#[test]
fn test_dynamic_property_access_coerces_mixed_string_name() {
    let out = compile_and_run(
        r#"<?php
class DynamicNameRow {
    public mixed $value = null;

    public function write(mixed $name, mixed $value): void {
        $this->{$name} = $value;
    }

    public function read(mixed $name): mixed {
        return $this->{$name};
    }
}

$row = new DynamicNameRow();
$row->write('value', 'ok');
echo $row->read('value');
"#,
    );
    assert_eq!(out, "ok");
}

/// Verifies runtime-named unset dispatch for generic objects, Mixed receivers, and stdClass.
#[test]
fn test_dynamic_property_unset_dispatches_and_marks_declared_slots_uninitialized() {
    let out = compile_and_run(
        r#"<?php
class DynamicUnsetBox {
    public string $value = 'live';
}

function clearObject(object $object, string $name): void {
    unset($object->$name);
}

function clearMixed(mixed $object, string $name): void {
    unset($object->$name);
}

$first = new DynamicUnsetBox();
clearObject($first, 'value');
echo isset($first->value) ? '1' : '0';

$second = new DynamicUnsetBox();
clearMixed($second, 'value');
echo isset($second->value) ? '1' : '0';

$dynamic = new stdClass();
$dynamic->value = 'live';
clearObject($dynamic, 'value');
echo isset($dynamic->value) ? '1' : '0';
"#,
    );
    assert_eq!(out, "000");
}

/// Verifies a Mixed source is runtime-checked and widened before entering a declared array slot.
#[test]
fn test_mixed_array_values_replace_declared_instance_properties() {
    let out = compile_and_run(
        r#"<?php
class RuntimeArrayHolder {
    public array $generic = [];
    public array $associative = ['seed' => 0];

    public function replaceGeneric(mixed $value): void {
        $this->generic = $value;
    }

    public function replaceAssociative(mixed $value): void {
        $this->associative = $value;
    }
}

$holder = new RuntimeArrayHolder();
$holder->replaceGeneric([1, 'two']);
$holder->replaceAssociative(['name' => 'value', 'count' => 2]);
echo $holder->generic[0], ':', $holder->generic[1], '|';
echo $holder->associative['name'], ':', $holder->associative['count'];
try {
    $holder->replaceGeneric('not an array');
} catch (TypeError $error) {
    echo '|type';
}
"#,
    );
    assert_eq!(out, "1:two|value:2|type");
}

/// Verifies lossless numeric widening and array-to-iterable assignment for typed properties.
#[test]
fn test_declared_property_accepts_int_float_widening_and_array_iterable() {
    let out = compile_and_run(
        r#"<?php
class WidenedPropertyHolder {
    public float $ratio = 0.0;
    public iterable $values = [];

    public function fill(int $ratio, array $values): void {
        $this->ratio = $ratio;
        $this->values = $values;
    }
}

$holder = new WidenedPropertyHolder();
$holder->fill(7, ['left' => 2, 'right' => 3]);
echo $holder->ratio, ':', count($holder->values);
foreach ($holder->values as $key => $value) {
    if ($key === 'right') {
        echo ':', $value;
    }
}
"#,
    );
    assert_eq!(out, "7:2:3");
}
