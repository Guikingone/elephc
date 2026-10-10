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
