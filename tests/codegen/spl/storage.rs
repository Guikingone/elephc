//! Purpose:
//! End-to-end tests for SPL storage iterator classes.
//! Covers EmptyIterator, ArrayIterator, and ArrayObject as Phase 5 storage foundations.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the SPL test module.
//!
//! Key details:
//! - ArrayIterator and ArrayObject preserve insertion-order keys through Mixed keys/values storage.

use crate::support::*;

/// Verifies that storage classes are declared and implement contracts.
#[test]
fn test_storage_classes_are_declared_and_implement_contracts() {
    let out = compile_and_run(
        r#"<?php
var_dump(class_exists("EmptyIterator"));
var_dump(class_exists("ArrayIterator"));
var_dump(class_exists("ArrayObject"));
var_dump(new EmptyIterator() instanceof Iterator);
var_dump(new ArrayIterator([]) instanceof SeekableIterator);
var_dump(new ArrayIterator([]) instanceof ArrayAccess);
var_dump(new ArrayObject([]) instanceof IteratorAggregate);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
}

/// Verifies that empty iterator foreach has no values.
#[test]
fn test_empty_iterator_foreach_has_no_values() {
    let out = compile_and_run(
        r#"<?php
echo "start:";
foreach (new EmptyIterator() as $k => $v) {
    echo "bad";
}
echo "end";
"#,
    );
    assert_eq!(out, "start:end");
}

/// Verifies `EmptyIterator::key()`/`current()` raise php's `BadMethodCallException`.
///
/// `EmptyIterator` never has a current element, and php refuses both readers with its own wording
/// rather than answering null.
#[test]
fn test_empty_iterator_readers_raise_bad_method_call() {
    let out = compile_and_run(
        r#"<?php
$it = new EmptyIterator();
var_dump($it->valid());
try {
    $it->key();
} catch (BadMethodCallException $e) {
    echo $e->getMessage(), "\n";
}
try {
    $it->current();
} catch (BadMethodCallException $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(
        out,
        "bool(false)\nAccessing the key of an EmptyIterator\nAccessing the value of an EmptyIterator\n"
    );
}

/// Verifies dynamic property creation on a builtin SPL container behaves like php's.
///
/// php accepts `$ao->x = 7` on an `ArrayObject` the way it does on every class: the write creates
/// the property and reports php 8.5's `Creation of dynamic property ArrayObject::$x is deprecated`
/// (stripped from the captured stdout by this harness), a read answers the stored value, `isset()`
/// probes the hash, and `unset()` removes it.
#[test]
fn test_array_object_dynamic_properties_report_phps_deprecation() {
    let out = compile_and_run(
        r#"<?php
$ao = new ArrayObject([1]);
$ao->x = 7;
echo $ao->x, "\n";
var_dump(isset($ao->x));
unset($ao->x);
var_dump(isset($ao->x));

$ai = new ArrayIterator([2]);
$ai->y = 8;
echo $ai->y, "\n";
"#,
    );
    assert_eq!(out, "7\nbool(true)\nbool(false)\n8\n");
}

/// Verifies `disable_functions` makes calling a disabled internal container method a catchable Error.
///
/// php consults the ini at the call and only for internal functions and methods, so a disabled
/// `asort` raises `Error: Cannot call method asort when function asort is disabled` while a user
/// method of the same name is unaffected.
#[test]
fn test_disable_functions_rejects_internal_container_method() {
    elephc::codegen_support::set_ini_overrides(vec![(
        "disable_functions".to_string(),
        "asort".to_string(),
    )]);
    let out = compile_and_run(
        r#"<?php
$ao = new ArrayObject([3, 1, 2]);
try {
    $ao->asort();
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
class Mine {
    public function asort() { return "user method unaffected"; }
}
echo (new Mine())->asort(), "\n";
"#,
    );
    elephc::codegen_support::set_ini_overrides(Vec::new());
    assert_eq!(
        out,
        "Cannot call method asort when function asort is disabled\nuser method unaffected\n"
    );
}

/// Verifies that array iterator iterates associative keys and values.
#[test]
fn test_array_iterator_iterates_associative_keys_and_values() {
    let out = compile_and_run(
        r#"<?php
$it = new ArrayIterator(["a" => 10, "b" => 20]);
foreach ($it as $k => $v) {
    echo $k;
    echo "=";
    echo $v;
    echo ";";
}
"#,
    );
    assert_eq!(out, "a=10;b=20;");
}

/// Verifies that array iterator count seek and current.
#[test]
fn test_array_iterator_count_seek_and_current() {
    let out = compile_and_run(
        r#"<?php
$it = new ArrayIterator(["x" => "first", "y" => "second"]);
echo count($it);
echo ":";
$it->seek(1);
echo $it->key();
echo "=";
echo $it->current();
"#,
    );
    assert_eq!(out, "2:y=second");
}

/// Verifies that array iterator array access and mutation.
#[test]
fn test_array_iterator_array_access_and_mutation() {
    let out = compile_and_run(
        r#"<?php
$it = new ArrayIterator(["a" => 1]);
echo $it["a"];
echo ":";
var_dump($it->offsetExists("b"));
$it["b"] = 2;
$it[] = 3;
foreach ($it as $k => $v) {
    echo $k;
    echo "=";
    echo $v;
    echo ";";
}
"#,
    );
    assert_eq!(out, "1:bool(false)\na=1;b=2;0=3;");
}

/// Verifies that array object returns array iterator.
#[test]
fn test_array_object_returns_array_iterator() {
    let out = compile_and_run(
        r#"<?php
$obj = new ArrayObject(["left" => 4, "right" => 5]);
echo count($obj);
echo ":";
foreach ($obj as $k => $v) {
    echo $k;
    echo "=";
    echo $v;
    echo ";";
}
"#,
    );
    assert_eq!(out, "2:left=4;right=5;");
}

/// Verifies `ArrayObject`/`ArrayIterator` accept an object as backing storage.
///
/// PHP declares the constructor parameter as `array|object`; an object contributes its PUBLIC
/// properties as the backing entries. The synthetic constructor stores one boxed value and
/// normalizes an object through `get_object_vars`, so both forms iterate identically. This also
/// guards the runtime result types of `get_object_vars`/`array_values` on a boxed argument: when
/// they fell back to `mixed`, the backend emitted a `mixed_clone`/unbox on a raw array pointer and
/// crashed.
#[test]
fn test_array_object_and_iterator_accept_object_backing() {
    let out = compile_and_run(
        r#"<?php
class P { public $a = 1; public $b = 2; }
$ao = new ArrayObject(new P());
foreach ($ao as $k => $v) { echo $k; echo "="; echo $v; echo ";"; }
echo ":";
$ai = new ArrayIterator((object)["x" => 7, "y" => 8]);
foreach ($ai as $k => $v) { echo $k; echo "="; echo $v; echo ";"; }
echo ":";
echo count(new ArrayObject());
"#,
    );
    assert_eq!(out, "a=1;b=2;:x=7;y=8;:0");
}

/// Verifies the `ArrayObject`/`ArrayIterator` flags, iterator-class, and exchange-array surface.
#[test]
fn test_array_storage_flags_iterator_class_and_exchange() {
    let out = compile_and_run(
        r#"<?php
$o = new ArrayObject(["a" => 1, "b" => 2], ArrayObject::ARRAY_AS_PROPS);
echo $o->getFlags(), ":";
$o->setFlags(0);
echo $o->getFlags(), ":";
echo $o->getIteratorClass(), ":";
$o->setIteratorClass("ArrayIterator");
echo $o->getIteratorClass(), ":";
$old = $o->exchangeArray(["x" => 9]);
foreach ($old as $k => $v) { echo $k; echo "="; echo $v; echo ";"; }
echo ":";
foreach ($o as $k => $v) { echo $k; echo "="; echo $v; echo ";"; }
echo ":";
$it = new ArrayIterator([1, 2]);
echo $it->getFlags(), ":";
$it->setFlags(2);
echo $it->getFlags();
"#,
    );
    assert_eq!(out, "2:0:ArrayIterator:ArrayIterator:a=1;b=2;:x=9;:0:2");
}

/// Verifies `var_dump` renders the single private `storage` property, like PHP.
///
/// `ArrayObject`/`ArrayIterator` keep their bookkeeping in `__elephc`-prefixed slots that the
/// `var_dump` descriptor hides, so exactly one `"storage":<class>:private` row is printed in place
/// of the internal fields.
#[test]
fn test_array_storage_var_dump_shows_php_storage_property() {
    let out = compile_and_run(
        r#"<?php
var_dump(new ArrayObject(['a' => 1]));
$it = new ArrayIterator([2 => 'x']);
var_dump($it);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "object(ArrayObject)#1 (1) {\n",
            "  [\"storage\":\"ArrayObject\":private]=>\n",
            "  array(1) {\n",
            "    [\"a\"]=>\n",
            "    int(1)\n",
            "  }\n",
            "}\n",
            "object(ArrayIterator)#1 (1) {\n",
            "  [\"storage\":\"ArrayIterator\":private]=>\n",
            "  array(1) {\n",
            "    [2]=>\n",
            "    string(1) \"x\"\n",
            "  }\n",
            "}\n",
        )
    );
}

/// Verifies `asort`/`ksort`/`uasort`/`uksort` preserve key association like PHP.
#[test]
fn test_array_storage_sorts_preserve_keys() {
    let out = compile_and_run(
        r#"<?php
$ao = new ArrayObject([4, 2, 3]);
echo $ao->asort() ? "t" : "f";
foreach ($ao as $k => $v) { echo $k; echo ":"; echo $v; echo ";"; }
echo "|";
$b = new ArrayObject(['b' => 2, 'a' => 1]);
$b->ksort();
foreach ($b as $k => $v) { echo $k; echo ":"; echo $v; echo ";"; }
echo "|";
$c = new ArrayObject([2, 3, 1]);
$c->uasort(function($x, $y) { return $y <=> $x; });
foreach ($c as $k => $v) { echo $k; echo ":"; echo $v; echo ";"; }
"#,
    );
    assert_eq!(out, "t1:2;2:3;0:4;|a:1;b:2;|1:3;0:2;2:1;");
}

/// Verifies that array iterator get array copy preserves keys.
#[test]
fn test_array_iterator_get_array_copy_preserves_keys() {
    let out = compile_and_run(
        r#"<?php
$it = new ArrayIterator(["a" => 1]);
$it["b"] = 2;
$copy = $it->getArrayCopy();
foreach ($copy as $k => $v) {
    echo $k;
    echo "=";
    echo $v;
    echo ";";
}
"#,
    );
    assert_eq!(out, "a=1;b=2;");
}

/// Verifies `??` over an `ArrayAccess` receiver probes `offsetExists()` before `offsetGet()`.
///
/// PHP's null-coalescing operator is a silent probe: a missing key must yield the default, not
/// the `UnexpectedValueException` a direct `offsetGet()` raises. A regression here routed the
/// coalesce straight into `offsetGet()`, so `$map[$missing] ?? null` threw instead of printing
/// `NULL`, and `SplObjectStorage::offsetGet()` could not raise for a genuinely absent object.
#[test]
fn test_array_access_coalesce_probes_offset_exists() {
    let out = compile_and_run(
        r#"<?php
$a = new stdClass();
$b = new stdClass();
$map = new SplObjectStorage();
$map[$a] = 'foo';
var_dump($map[$b] ?? null);
var_dump($map[$a] ?? null);
try {
    $map->offsetGet($b);
} catch (UnexpectedValueException $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "NULL\nstring(3) \"foo\"\nObject not found\n");
}

/// Verifies the SPL gate opens for `unserialize`, whose class name lives in the DATA.
///
/// A valid serialized `SplFixedArray` came back as `__PHP_Incomplete_Class` because no static walk
/// can read the name inside the payload — and adding an unrelated `class_exists("SplFixedArray")`
/// to the program made the same payload work, which is the signature of a gate closing on a
/// reference it cannot see. The date/time gate already treated `unserialize` this way.
#[test]
fn test_spl_registers_for_unserialize() {
    let out = compile_and_run(
        r#"<?php
$o = unserialize('O:13:"SplFixedArray":2:{i:0;i:7;i:1;i:9;}');
echo get_class($o), ":", $o[0], $o[1];
"#,
    );
    assert_eq!(out, "SplFixedArray:79");
}

/// Verifies that an object-backed `ArrayObject` keeps the wrapped object live.
///
/// php keeps the object as the backing store: iteration reads its properties, a keyed write
/// creates a dynamic property ON the wrapped object (observable through it afterwards), and a
/// keyed removal drops that property.
#[test]
fn test_array_object_object_backing_writes_reach_the_wrapped_object() {
    let out = compile_and_run(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
class C { public $b = 2; public $a = 1; }
$o = new C();
$ao = new ArrayObject($o);
foreach ($ao as $k => $v) { echo "$k=$v "; } echo "\n";
$ao['zz'] = 5;
foreach ($ao as $k => $v) { echo "$k=$v "; } echo "\n";
var_dump(count($ao));
var_dump(array_key_exists('zz', get_object_vars($o)));
$ao->offsetUnset('a');
foreach ($ao as $k => $v) { echo "$k=$v "; } echo "\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "b=2 a=1 \n",
            "b=2 a=1 zz=5 \n",
            "int(3)\n",
            "bool(true)\n",
            "b=2 zz=5 \n",
        )
    );
}

/// Verifies that `exchangeArray()` answers the OLD store's contents as an array.
///
/// For an object store the old contents are the wrapped object's properties; for an
/// `ArrayObject`-shaped store they are the array that object follows.
#[test]
fn test_array_object_exchange_array_returns_object_store_contents() {
    let out = compile_and_run(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
class C { public $b = 2; }
$o = new C();
$ao = new ArrayObject($o);
var_dump($ao->exchangeArray(['x' => 9]));
var_dump($ao->getArrayCopy());
$inner = new ArrayObject(['x' => 1]);
$outer = new ArrayObject($inner);
var_dump($outer->getArrayCopy());
$nest = new ArrayIterator(new ArrayIterator(['y' => 2]));
var_dump($nest->getArrayCopy());
"#,
    );
    assert_eq!(
        out,
        concat!(
            "array(1) {\n",
            "  [\"b\"]=>\n",
            "  int(2)\n",
            "}\n",
            "array(1) {\n",
            "  [\"x\"]=>\n",
            "  int(9)\n",
            "}\n",
            "array(1) {\n",
            "  [\"x\"]=>\n",
            "  int(1)\n",
            "}\n",
            "array(1) {\n",
            "  [\"y\"]=>\n",
            "  int(2)\n",
            "}\n",
        )
    );
}

/// Verifies php's bug #41691 shape: a SELF-backed `ArrayObject` subclass.
///
/// `parent::__construct($this, ...)` keeps the object itself as the store; `exchangeArray()`
/// answers the property table with uninitialized declared properties as NULL and without the
/// internal backing key php never declared.
#[test]
fn test_array_object_self_backed_exchange_array_answers_property_table() {
    let (raw, _dir) = compile_and_run_in_dir(
        r#"<?php
class A extends ArrayObject {
    public function __construct($dummy, $flags) {
        parent::__construct($this, $flags);
    }
    public $a;
    public $b;
    public $c;
}
$a = new A(null, ArrayObject::ARRAY_AS_PROPS);
var_dump($a->exchangeArray(array('a' => 1, 'b' => 1, 'c' => 1)));
echo "Done\n";
"#,
    );
    // php's deprecation reaches stdout with the run's own file path, so strip the display line
    // and assert the remainder exactly (the corpus runner asserts the full line separately).
    assert_eq!(
        raw.matches(
            "Deprecated: ArrayObject::__construct(): Using an object as a backing array for ArrayObject is deprecated, as it allows violating class constraints and invariants in "
        )
        .count(),
        1,
        "expected exactly one object-backing deprecation: {raw}"
    );
    let out = strip_php_display_lines(&raw);
    assert_eq!(
        out,
        concat!(
            "array(3) {\n",
            "  [\"a\"]=>\n",
            "  NULL\n",
            "  [\"b\"]=>\n",
            "  NULL\n",
            "  [\"c\"]=>\n",
            "  NULL\n",
            "}\n",
            "Done\n",
        )
    );
}

/// Verifies that a keyed `unset()` through a `mixed` receiver reaches the wrapped object.
///
/// php defers the receiver's shape to run time: `unset($m['k'])` removes the entry from an array
/// or calls `ArrayAccess::offsetUnset()` on an object. elephc used to refuse to compile that
/// shape; the runtime container dispatch now answers for both, and the refused shapes raise
/// php's own fatal instead of dropping the write.
#[test]
fn test_array_object_mixed_receiver_unset_reaches_the_backing_object() {
    let out = compile_and_run(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
class C { public $b = 2; public $a = 1; }
function drop($m, $key) { unset($m[$key]); return $m; }
$o = new C();
$ao = new ArrayObject($o);
drop($ao, 'a');
var_dump(array_key_exists('a', get_object_vars($o)));
var_dump(count($ao));
var_dump(drop(['k' => 1, 'j' => 2], 'k'));
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(false)\n",
            "int(1)\n",
            "array(1) {\n",
            "  [\"j\"]=>\n",
            "  int(2)\n",
            "}\n",
        )
    );
}

/// Verifies `natsort()` orders values in php's natural order, keeping keys, and answers true.
#[test]
fn test_array_object_natsort_orders_values_naturally() {
    let out = compile_and_run(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
$ao = new ArrayObject(['boo10', 'boo1', 'boo2', 'boo22', 'BOO5']);
var_dump($ao->natsort());
foreach ($ao as $k => $v) { echo "$k=$v "; } echo "\n";
$a2 = new ArrayObject(['a' => 'img12', 'b' => 'img2', 'c' => 'img1x']);
$a2->natsort();
foreach ($a2 as $k => $v) { echo "$k=$v "; } echo "\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(true)\n",
            "4=BOO5 1=boo1 2=boo2 0=boo10 3=boo22 \n",
            "c=img1x b=img2 a=img12 \n",
        )
    );
}

/// Verifies `natcasesort()` folds case before comparing, so mixed-case keys sort naturally.
#[test]
fn test_array_object_natcasesort_folds_case() {
    let out = compile_and_run(
        r#"<?php
error_reporting(E_ALL & ~E_DEPRECATED);
$ao = new ArrayObject(['a' => 'boo10', 'b' => 'boo1', 'c' => 'boo2', 'd' => 'boo22', 'e' => 'BOO5']);
var_dump($ao->natcasesort());
foreach ($ao as $k => $v) { echo "$k=$v "; } echo "\n";
"#,
    );
    assert_eq!(out, "bool(true)\nb=boo1 c=boo2 e=BOO5 a=boo10 d=boo22 \n");
}

/// Verifies that a wrong argument count on an internal class's method raises php's catchable
/// `ArgumentCountError` at run time with php's own wording, instead of a compile-time refusal.
#[test]
fn test_array_object_internal_method_arity_is_a_runtime_argument_count_error() {
    let out = compile_and_run(
        r#"<?php
$ao = new ArrayObject([1]);
try {
    $ao->natsort('blah');
} catch (ArgumentCountError $e) {
    echo $e->getMessage(), "\n";
}
var_dump(count($ao));
"#,
    );
    assert_eq!(
        out,
        concat!(
            "ArrayObject::natsort() expects exactly 0 arguments, 1 given\n",
            "int(1)\n",
        )
    );
}
