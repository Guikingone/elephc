//! Purpose:
//! End-to-end tests for built-in SPL container classes.
//! Verifies Phase 4 container metadata plus runtime-backed list behavior.
//!
//! Called from:
//! - `cargo test --test codegen_tests` through the SPL test module.
//!
//! Key details:
//! - Runtime tests cover Phase 4 containers; iterator decorators and heaps remain later roadmap phases.

use crate::support::*;

// Tests that Phase 4 SPL classes appear in `spl_classes()`, `get_declared_classes()`,
// and are recognized by `class_exists()` including case-insensitive names.
/// Verifies that phase4 SPL classes are declared for introspection.
#[test]
fn test_phase4_spl_classes_are_declared_for_introspection() {
    let out = compile_and_run(
        r#"<?php
function has_name(array $names, string $target): bool {
    foreach ($names as $name) {
        if ($name === $target) {
            return true;
        }
    }
    return false;
}

$spl = spl_classes();
echo has_name($spl, "SplDoublyLinkedList");
echo has_name($spl, "SplStack");
echo has_name($spl, "SplQueue");
echo has_name($spl, "SplFixedArray");

$declared = get_declared_classes();
echo has_name($declared, "SplDoublyLinkedList");
echo has_name($declared, "SplStack");
echo has_name($declared, "SplQueue");
echo has_name($declared, "SplFixedArray");
echo has_name($declared, "InternalIterator");

var_dump(class_exists("SplDoublyLinkedList"));
var_dump(class_exists("splstack"));
var_dump(class_exists("InternalIterator"));
"#,
    );
    assert_eq!(out, "111111111bool(true)\nbool(true)\nbool(true)\n");
}

// Tests that SplDoublyLinkedList, SplStack, SplQueue, and SplFixedArray implement the
// correct interfaces (Iterator, Countable, ArrayAccess, JsonSerializable) and that
// SplStack/SplQueue inherit from SplDoublyLinkedList.
/// Verifies that phase4 SPL class interface and parent metadata.
#[test]
fn test_phase4_spl_class_interface_and_parent_metadata() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
var_dump($list instanceof Iterator);
var_dump($list instanceof Countable);
var_dump($list instanceof ArrayAccess);

$stack = new SplStack();
var_dump($stack instanceof SplDoublyLinkedList);
var_dump($stack instanceof Iterator);

$queue = new SplQueue();
var_dump($queue instanceof SplDoublyLinkedList);
var_dump($queue instanceof Countable);

$fixed = new SplFixedArray();
var_dump($fixed instanceof IteratorAggregate);
var_dump($fixed instanceof ArrayAccess);
var_dump($fixed instanceof Countable);
var_dump($fixed instanceof JsonSerializable);
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
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
            "bool(true)\n",
        )
    );
}

// Tests that SplDoublyLinkedList constants (IT_MODE_LIFO, IT_MODE_DELETE, IT_MODE_FIFO)
// are correctly inherited by SplStack and SplQueue with their expected integer values.
/// Verifies that phase4 SPL doubly linked list constants are inherited.
#[test]
fn test_phase4_spl_doubly_linked_list_constants_are_inherited() {
    let out = compile_and_run(
        r#"<?php
echo SplDoublyLinkedList::IT_MODE_LIFO;
echo ",";
echo SplStack::IT_MODE_DELETE;
echo ",";
echo SplQueue::IT_MODE_FIFO;
"#,
    );
    assert_eq!(out, "2,1,0");
}

// Tests SplDoublyLinkedList mutation methods: push, unshift, add, pop, shift, bottom, top,
// count, and isEmpty on a non-empty and empty list.
/// Verifies that phase4 SPL doubly linked list mutation methods.
#[test]
fn test_phase4_spl_doubly_linked_list_mutation_methods() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
var_dump($list->isEmpty());
$list->push("a");
$list->push(2);
$list->unshift("z");
$list->add(1, "m");
echo count($list);
echo "\n";
echo $list->bottom();
echo "|";
echo $list->top();
echo "\n";
echo $list->shift();
echo "|";
echo $list->pop();
echo "|";
echo $list->shift();
echo "|";
echo $list->pop();
echo "\n";
var_dump($list->isEmpty());
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(true)\n",
            "4\n",
            "z|2\n",
            "z|2|m|a\n",
            "bool(true)\n",
        )
    );
}

// Tests SplDoublyLinkedList iteration modes: IT_MODE_FIFO and IT_MODE_LIFO with getIteratorMode/setIteratorMode,
// verifying that foreach produces correct key:value ordering and that the mode value is preserved.
/// Verifies that phase4 SPL doubly linked list iteration modes.
#[test]
fn test_phase4_spl_doubly_linked_list_iteration_modes() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
$list->push("a");
$list->push("b");
$list->push("c");
foreach ($list as $k => $v) {
    echo $k;
    echo ":";
    echo $v;
    echo ";";
}
echo "\n";
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
foreach ($list as $k => $v) {
    echo $k;
    echo ":";
    echo $v;
    echo ";";
}
echo "\n";
echo $list->getIteratorMode();
"#,
    );
    assert_eq!(out, "0:a;1:b;2:c;\n2:c;1:b;0:a;\n2");
}

/// Verifies `prev()` before the first element exhausts the iterator (`current()` is null), like php.
#[test]
fn test_phase4_spl_doubly_linked_list_prev_before_start() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
$list->push(1);
$list->push(2);
$list->rewind();
$list->prev();
var_dump($list->current());
$list->rewind();
var_dump($list->current());
"#,
    );
    assert_eq!(out, "NULL\nint(1)\n");
}

// Tests IT_MODE_DELETE combined with IT_MODE_FIFO and IT_MODE_LIFO: verifies that foreach
// consumes elements during iteration and that count reaches zero after FIFO traversal but
// preserves order for LIFO.
/// Verifies that phase4 SPL doubly linked list delete iteration modes.
#[test]
fn test_phase4_spl_doubly_linked_list_delete_iteration_modes() {
    let out = compile_and_run(
        r#"<?php
$fifo = new SplDoublyLinkedList();
$fifo->push("a");
$fifo->push("b");
$fifo->push("c");
$fifo->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO | SplDoublyLinkedList::IT_MODE_DELETE);
foreach ($fifo as $k => $v) {
    echo $k;
    echo ":";
    echo $v;
    echo ";";
}
echo "\n";
echo count($fifo);
echo "\n";

$lifo = new SplDoublyLinkedList();
$lifo->push("a");
$lifo->push("b");
$lifo->push("c");
$lifo->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO | SplDoublyLinkedList::IT_MODE_DELETE);
foreach ($lifo as $k => $v) {
    echo $k;
    echo ":";
    echo $v;
    echo ";";
}
echo "\n";
echo count($lifo);
"#,
    );
    assert_eq!(out, "0:a;0:b;0:c;\n0\n2:c;1:b;0:a;\n0");
}

/// Verifies the checked-in SPL delete-iteration mutation stress example observes
/// PHP-compatible traversal order when the active list is mutated inside foreach.
#[test]
fn test_spl_delete_iteration_mutation_example() {
    let out = compile_and_run(include_str!(
        "../../../examples/spl-delete-iteration-mutation/main.php"
    ));
    assert_eq!(out, "0:a|0:b|0:c|0:x|\ncount=0\n");
}

// Tests SplDoublyLinkedList ArrayAccess implementation: offsetExists, offsetGet, offsetSet,
// and offsetUnset via direct bracket access on an empty list, populated list, after unset,
// and that iteration order follows LIFO mode.
/// Verifies that phase4 SPL doubly linked list array access.
#[test]
fn test_phase4_spl_doubly_linked_list_array_access() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
$list[] = "a";
$list[] = "b";
echo $list[0];
echo "|";
echo $list[1];
echo "\n";
echo isset($list[1]);
echo "\n";
unset($list[0]);
echo $list[0];
echo "\n";
$list[] = "c";
echo $list[1];
echo "\n";
$list[0] = "z";
echo $list[0];
echo "|";
echo count($list);
"#,
    );
    assert_eq!(out, "a|b\n1\nb\nc\nz|2");
}

// Tests RuntimeException from pop/shift/top on empty list and OutOfRangeException from
// offsetSet/add beyond current size, plus that iteration mode affects bracket read order.
/// Verifies that phase4 SPL doubly linked list PHP error edges.
#[test]
fn test_phase4_spl_doubly_linked_list_php_error_edges() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
try { $list->pop(); } catch (RuntimeException $e) { echo "pop"; }
try { $list->shift(); } catch (RuntimeException $e) { echo "|shift"; }
try { $list->top(); } catch (RuntimeException $e) { echo "|top"; }
$list->push("a");
try { $list[1] = "x"; } catch (OutOfRangeException $e) { echo "|set"; }
try { $list->add(2, "x"); } catch (OutOfRangeException $e) { echo "|add"; }
$list[] = "b";
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
echo "|";
echo $list[0];
echo $list[1];
"#,
    );
    assert_eq!(out, "pop|shift|top|set|add|ba");
}

// Tests SplStack push/pop/top/count and SplQueue enqueue/dequeue/bottom/top/count
// runtime methods, verifying stack LIFO and queue FIFO ordering.
/// Verifies that phase4 SPL stack and queue runtime methods.
#[test]
fn test_phase4_spl_stack_and_queue_runtime_methods() {
    let out = compile_and_run(
        r#"<?php
$stack = new SplStack();
$stack->push(1);
$stack->push(2);
echo $stack->pop();
echo "|";
echo $stack->top();
echo "|";
echo count($stack);
echo "\n";

$queue = new SplQueue();
$queue->enqueue("a");
$queue->enqueue("b");
echo $queue->dequeue();
echo "|";
echo $queue->bottom();
echo "|";
echo $queue->top();
echo "|";
echo count($queue);
"#,
    );
    assert_eq!(out, "2|1|1\na|b|b|1");
}

// SplStack array offsets are LIFO: offset 0 is the TOP of the stack, so reads, writes and
// unset() address the list in reverse storage order (issue #1659). SplQueue stays FIFO.
/// Verifies that spl stack bracket offsets follow LIFO order.
#[test]
fn test_spl_stack_bracket_offsets_are_lifo() {
    let out = compile_and_run(
        r#"<?php
$s = new SplStack();
$s->push("a"); $s->push("b"); $s->push("c");
echo $s[0], $s[1], $s[2], "\n";
echo $s->offsetGet(0), "\n";
var_dump(isset($s[2]));
$s[0] = "z";
echo $s->top(), $s->bottom(), "\n";
unset($s[0]);
echo count($s), $s->top(), "\n";
$q = new SplQueue();
$q->enqueue("a"); $q->enqueue("b"); $q->enqueue("c");
echo $q[0], $q[1], $q[2], "\n";
$q[0] = "z";
echo $q->bottom(), $q->top(), "\n";
"#,
    );
    assert_eq!(out, "cba\nc\nbool(true)\nza\n2b\nabc\nzc\n");
}

// `getIteratorMode()` reports PHP's raw mode word: `SplStack` = 6 (FIX|LIFO), `SplQueue` = 4
// (FIX), `SplDoublyLinkedList` = 0. `setIteratorMode()` freezes the LIFO/FIFO bit on a fixed
// list (RuntimeException) while allowing the DELETE bit and preserving FIX (issue #1659).
/// Verifies that spl iterator mode word and freeze.
#[test]
fn test_spl_iterator_mode_word_and_freeze() {
    let out = compile_and_run(
        r#"<?php
$s = new SplStack();
$q = new SplQueue();
$d = new SplDoublyLinkedList();
echo $s->getIteratorMode(), $q->getIteratorMode(), $d->getIteratorMode(), "|";
try { $s->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO); echo "no-throw"; }
catch (RuntimeException $e) { echo "frozen"; }
$s->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO | SplDoublyLinkedList::IT_MODE_DELETE);
echo "|", $s->getIteratorMode();
$s->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
echo "|", $s->getIteratorMode();
$q->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO | SplDoublyLinkedList::IT_MODE_DELETE);
echo "|", $q->getIteratorMode();
$d->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
echo "|", $d->getIteratorMode();
$d->setIteratorMode(4);
echo "|", $d->getIteratorMode();
$d->setIteratorMode(2);
echo "|", $d->getIteratorMode();
$s->setIteratorMode(PHP_INT_MAX);
echo "|", $s->getIteratorMode();
"#,
    );
    assert_eq!(out, "640|frozen|7|6|5|2|0|2|7");
}

// The LIFO default also drives `foreach` traversal order and `serialize()`'s flags byte; both
// are PHP-parity wins of seeding the mode and were previously unpinned (issue #1659).
/// Verifies that spl stack traversal and serialize follow lifo.
#[test]
fn test_spl_stack_traversal_and_serialize_follow_lifo() {
    let out = compile_and_run(
        r#"<?php
$s = new SplStack();
$s->push("a"); $s->push("b"); $s->push("c");
foreach ($s as $k => $v) { echo $k, ":", $v, " "; }
echo "|", serialize($s), "\n";
"#,
    );
    assert_eq!(
        out,
        "2:c 1:b 0:a |O:8:\"SplStack\":3:{i:0;i:6;i:1;a:3:{i:0;s:1:\"a\";i:1;s:1:\"b\";i:2;s:1:\"c\";}i:2;a:0:{}}\n"
    );
}

// `__debugInfo()` returns the raw mode word and the PHYSICAL storage order (first pushed first),
// not the LIFO `offsetGet()` order, and must snapshot it WITHOUT flipping the frozen mode: the
// old body called `setIteratorMode(0)`, which a fixed-mode list now rejects with a
// RuntimeException (issue #1659).
/// Verifies that spl debug info reports flags and physical order.
#[test]
fn test_spl_debug_info_reports_flags_and_physical_order() {
    let out = compile_and_run(
        r#"<?php
$s = new SplStack();
$s->push("a"); $s->push("b"); $s->push("c");
var_dump($s->__debugInfo());
$q = new SplQueue();
$q->enqueue("a"); $q->enqueue("b"); $q->enqueue("c");
var_dump($q->__debugInfo());
$d = new SplDoublyLinkedList();
$d->push("a"); $d->push("b"); $d->push("c");
var_dump($d->__debugInfo());
"#,
    );
    let entry = |flags: i64| {
        format!(
            "array(2) {{\n  [\"\0SplDoublyLinkedList\0flags\"]=>\n  int({flags})\n  [\"\0SplDoublyLinkedList\0dllist\"]=>\n  array(3) {{\n    [0]=>\n    string(1) \"a\"\n    [1]=>\n    string(1) \"b\"\n    [2]=>\n    string(1) \"c\"\n  }}\n}}\n"
        )
    };
    assert_eq!(out, format!("{}{}{}", entry(6), entry(4), entry(0)));
}

// Tests SplFixedArray getSize/setSize, direct bracket read/write, isset/unset, toArray,
// jsonSerialize, and that resizing a fixed array preserves existing elements up to the new size.
/// Verifies that phase4 SPL fixed array runtime methods.
#[test]
fn test_phase4_spl_fixed_array_runtime_methods() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(2);
echo count($fixed);
echo "|";
echo $fixed->getSize();
echo "\n";
$fixed[0] = "a";
$fixed[1] = 3;
echo $fixed[0];
echo "|";
echo $fixed[1];
echo "\n";
echo isset($fixed[0]);
unset($fixed[0]);
echo isset($fixed[0]);
echo "\n";
$fixed->setSize(3);
$fixed[2] = "c";
echo count($fixed);
echo "|";
echo $fixed[2];
echo "\n";
$array = $fixed->toArray();
echo count($array);
echo "|";
echo $array[1];
echo "|";
echo $array[2];
echo "\n";
$json = $fixed->jsonSerialize();
echo count($json);
"#,
    );
    // The post-unset `echo isset($fixed[0])` is false → "" in PHP (not "0").
    assert_eq!(out, "2|2\na|3\n1\n3|c\n3|3|c\n3");
}

// Tests that negative size to SplFixedArray constructor throws ValueError, out-of-bounds
// get/set throws OutOfBoundsException, non-integer index throws TypeError, and fromArray
// with string keys throws InvalidArgumentException.
/// Verifies that phase4 SPL fixed array PHP error edges.
#[test]
fn test_phase4_spl_fixed_array_php_error_edges() {
    let out = compile_and_run(
        r#"<?php
try { $tmp = new SplFixedArray(-1); } catch (ValueError $e) { echo "new"; }
$fixed = new SplFixedArray(1);
try { $fixed[1] = "x"; } catch (OutOfBoundsException $e) { echo "|set"; }
try { $x = $fixed[1]; } catch (OutOfBoundsException $e) { echo "|get"; }
try { $fixed["x"] = "x"; } catch (TypeError $e) { echo "|type"; }
try { $fixed->setSize(-1); } catch (ValueError $e) { echo "|resize"; }
try { SplFixedArray::fromArray(["x" => "y"]); } catch (InvalidArgumentException $e) { echo "|from"; }
"#,
    );
    assert_eq!(out, "new|set|get|type|resize|from");
}

// Tests SplDoublyLinkedList serialization helpers: __serialize, __unserialize, __debugInfo,
// serialize, and unserialize, including round-trip preservation of LIFO mode and scalar
// values (true, null).
/// Verifies that phase4 SPL doubly linked list serialization helpers.
#[test]
fn test_phase4_spl_doubly_linked_list_serialization_helpers() {
    let out = compile_and_run(
        r#"<?php
$list = new SplDoublyLinkedList();
$list->push("a");
$list->push(2);
$list->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);

$ser = $list->__serialize();
echo count($ser);
echo "|";
echo $ser[0];
echo "|";
echo count($ser[1]);
echo "|";
echo $ser[1][0];
echo "|";
echo $ser[1][1];
echo "|";
echo count($ser[2]);
echo "\n";

$debug = $list->__debugInfo();
echo count($debug);
echo "\n";

$copy = new SplDoublyLinkedList();
$copy->__unserialize($ser);
echo $copy->getIteratorMode();
echo "|";
echo $copy[0];
echo "|";
echo $copy[1];
echo "\n";

echo $list->serialize();
echo "\n";

$legacy = new SplDoublyLinkedList();
$legacy->unserialize($list->serialize());
echo $legacy->getIteratorMode();
echo "|";
echo $legacy[0];
echo "|";
echo $legacy[1];
echo "\n";

$scalars = new SplDoublyLinkedList();
$scalars->push(true);
$scalars->push(null);
$round = new SplDoublyLinkedList();
$round->unserialize($scalars->serialize());
echo $round[0] ? "true" : "false";
echo "|";
echo is_null($round[1]) ? "null" : "value";
"#,
    );
    assert_eq!(
        out,
        "3|2|2|a|2|0\n2\n2|2|a\ni:2;:s:1:\"a\";:i:2;\n2|2|a\ntrue|null"
    );
}

// Tests SplFixedArray __serialize, __unserialize, and fromArray with both preserve-keys
// (default) and non-preserving (packed) modes, verifying correct null-slot handling and
// size computation.
/// Verifies that phase4 SPL fixed array serialization and from array helpers.
#[test]
fn test_phase4_spl_fixed_array_serialization_and_from_array_helpers() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(3);
$fixed[1] = "b";
$ser = $fixed->__serialize();
echo count($ser);
echo "|";
echo is_null($ser[0]) ? "null" : $ser[0];
echo "|";
echo $ser[1];
echo "|";
echo is_null($ser[2]) ? "null" : $ser[2];
echo "\n";

$copy = new SplFixedArray();
$copy->__unserialize(["x", "y"]);
echo $copy->getSize();
echo "|";
echo $copy[0];
echo "|";
echo $copy[1];
echo "\n";

$from = SplFixedArray::fromArray([2 => "x", 5 => "y"]);
echo $from->getSize();
echo "|";
echo $from[2];
echo "|";
echo $from[5];
echo "\n";

$packed = SplFixedArray::fromArray([2 => "x", 5 => "y"], false);
echo $packed->getSize();
echo "|";
echo $packed[0];
echo "|";
echo $packed[1];
"#,
    );
    assert_eq!(out, "3|null|b|null\n2|x|y\n6|x|y\n2|x|y");
}

/// Verifies that phase4 SPL fixed array get iterator.
#[test]
fn test_phase4_spl_fixed_array_get_iterator() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(3);
$fixed[1] = "b";
$it = $fixed->getIterator();
var_dump($it instanceof InternalIterator);
var_dump($it instanceof Iterator);
var_dump($it instanceof SeekableIterator);
foreach ($it as $key => $value) {
    echo $key;
    echo "=";
    echo is_null($value) ? "null" : $value;
    echo ";";
}
echo "\n";
$it->rewind();
$fixed[0] = "a";
echo $it->current();
echo "\n";
foreach ($fixed as $key => $value) {
    echo $key;
    echo "=";
    echo is_null($value) ? "null" : $value;
    echo ";";
}
echo "\n";
$it->next();
$fixed->setSize(1);
var_dump($it->valid());
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bool(true)\n",
            "bool(true)\n",
            "bool(false)\n",
            "0=null;1=b;2=null;\n",
            "a\n",
            "0=a;1=b;2=null;\n",
            "bool(false)\n",
        )
    );
}

/// Verifies a `foreach` over an `SplFixedArray` produced by the static factory still links.
///
/// The factory (`SplFixedArray::fromArray`) is not an `ObjectNew`, and the implicit
/// `getIterator()` call the backend emits for the loop is not an EIR `MethodCall`, so the
/// reachability scan has to register the aggregate's iterator factory from `IterStart` itself;
/// otherwise the call site references a `_method_SplFixedArray_getiterator` symbol that is never
/// emitted and the program fails to link.
#[test]
fn test_spl_fixed_array_factory_nested_foreach_links_and_iterates() {
    let out = compile_and_run(
        r#"<?php
$array = SplFixedArray::fromArray([0, 1]);
foreach ($array as $value1) {
    foreach ($array as $value2) {
        echo "$value1 $value2\n";
    }
}
"#,
    );
    assert_eq!(out, "0 0\n0 1\n1 0\n1 1\n");
}

/// Verifies `$fixed[] = v` raises php's catchable `[] operator not supported for SplFixedArray`.
///
/// `$obj[] = v` is php's `offsetSet(null, v)`, and `SplFixedArray::offsetSet()` refuses a null
/// key, so the append slot has to raise the `Error` rather than call a missing `append()`.
#[test]
fn test_spl_fixed_array_append_raises_phps_error() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(10);
try {
    $fixed[] = 1;
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "[] operator not supported for SplFixedArray\n");
}

/// Verifies `SplFixedArray` offsets coerce a numeric string the way php does.
///
/// php converts `"4"` to the integer offset 4 for every ArrayAccess operation, so a string key
/// reaches the same slot as the number. A negative numeric string reaches the range check, which
/// php rejects with `OutOfBoundsException: Index invalid or out of range`.
#[test]
fn test_spl_fixed_array_numeric_string_offsets_coerce() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(5);
for ($i = 0; $i < 5; $i++) {
    $fixed[$i] = "v$i";
}
$fixed->offsetUnset("4");
var_dump($fixed[4]);
var_dump($fixed["3"]);
var_dump(isset($fixed["2"]));
try {
    $fixed["-1"];
} catch (OutOfBoundsException $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(
        out,
        "NULL\nstring(2) \"v3\"\nbool(true)\nIndex invalid or out of range\n"
    );
}

// Tests that an SplFixedArray size whose `size * 8` storage payload wraps the machine word is
// rejected by the shared `__rt_array_new` guard instead of allocating a tiny block behind a header
// that advertises 2^61 slots. PHP reports the same class of failure as
// "Possible integer overflow in memory allocation".
/// Verifies that an unrepresentable SplFixedArray size aborts instead of over-reporting capacity.
#[test]
fn test_spl_fixed_array_overflowing_size_is_fatal() {
    let err = compile_and_run_expect_failure(
        r#"<?php
$fixed = new SplFixedArray(0x2000000000000004);
echo $fixed->getSize();
"#,
    );
    assert!(
        err.contains("requested array size exceeds the maximum allowed array size"),
        "{}",
        err
    );
}

// Positive control for the SplFixedArray storage-size guard: an ordinary fixed array still
// allocates, zero-initializes, and round-trips element writes.
/// Verifies SplStack/SplQueue freeze their LIFO/FIFO iterator mode while the base list does not.
///
/// php fixes the mode bit on the two subclasses: `SplStack` refuses FIFO and `SplQueue` refuses
/// LIFO with the same `RuntimeException`, while `SplDoublyLinkedList` still switches freely.
#[test]
fn test_spl_stack_and_queue_freeze_iterator_mode() {
    let out = compile_and_run(
        r#"<?php
$stack = new SplStack();
try { $stack->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO); }
catch (RuntimeException $e) { echo "stack: ", $e->getMessage(), "\n"; }
$stack->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO);
echo "stack lifo ok\n";
$queue = new SplQueue();
try { $queue->setIteratorMode(SplDoublyLinkedList::IT_MODE_LIFO); }
catch (RuntimeException $e) { echo "queue: ", $e->getMessage(), "\n"; }
$queue->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO);
echo "queue fifo ok\n";
$dll = new SplDoublyLinkedList();
$dll->setIteratorMode(SplDoublyLinkedList::IT_MODE_FIFO);
echo "dll fifo ok\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "stack: Iterators' LIFO/FIFO modes for SplStack/SplQueue objects are frozen\n",
            "stack lifo ok\n",
            "queue: Iterators' LIFO/FIFO modes for SplStack/SplQueue objects are frozen\n",
            "queue fifo ok\n",
            "dll fifo ok\n",
        )
    );
}

/// Verifies `var_dump` renders an `SplFixedArray` as an indexed array of its backing storage.
///
/// The class stores its payload outside the declared-property layout, so without a per-class
/// var_dump adapter it rendered as `object(SplFixedArray)#1 (0) {}`. PHP prints the element count
/// in the header and one `[N]=>` line per slot, with unset slots as `NULL`.
#[test]
fn test_spl_fixed_array_var_dump_renders_elements() {
    let out = compile_and_run(
        r#"<?php
$a = new SplFixedArray(2);
$a[0] = "foo";
var_dump($a);
$b = SplFixedArray::fromArray([1, "2", false]);
var_dump($b);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "object(SplFixedArray)#1 (2) {\n",
            "  [0]=>\n",
            "  string(3) \"foo\"\n",
            "  [1]=>\n",
            "  NULL\n",
            "}\n",
            "object(SplFixedArray)#2 (3) {\n",
            "  [0]=>\n",
            "  int(1)\n",
            "  [1]=>\n",
            "  string(1) \"2\"\n",
            "  [2]=>\n",
            "  bool(false)\n",
            "}\n",
        )
    );
}

/// Verifies `clone` copies the runtime-managed payload of the SPL containers.
///
/// These classes keep their elements outside the declared-property layout, so the generic
/// shallow-clone adapter could not copy them and the operation was rejected. The per-class
/// adapters duplicate the backing storage, so mutating the clone never reaches the original.
#[test]
fn test_spl_container_clone_isolates_storage() {
    let out = compile_and_run(
        r#"<?php
$dll = new SplDoublyLinkedList();
$dll->push(1);
$dll->push(2);
$clone = clone $dll;
$clone->pop();
echo count($dll), "|", count($clone), "\n";

$fixed = new SplFixedArray(3);
$fixed[0] = "a";
$fixedClone = clone $fixed;
$fixedClone->setSize(1);
echo $fixed->getSize(), "|", $fixedClone->getSize(), "\n";

$stack = new SplStack();
$stack->push("x");
$stackClone = clone $stack;
$stackClone->pop();
echo count($stack), "|", count($stackClone), "\n";
"#,
    );
    assert_eq!(out, "2|1\n3|1\n1|0\n");
}

/// Verifies the per-class default iterator mode and traversal direction.
///
/// php seeds SplStack with LIFO (`getIteratorMode()` 6) and SplQueue with FIFO (4), while the
/// base SplDoublyLinkedList starts at 0. elephc started every list at 0, so an SplStack iterated
/// forwards and reported the wrong mode.
#[test]
fn test_spl_list_default_iterator_modes() {
    let out = compile_and_run(
        r#"<?php
$stack = new SplStack();
echo $stack->getIteratorMode(), "\n";
$queue = new SplQueue();
echo $queue->getIteratorMode(), "\n";
$dll = new SplDoublyLinkedList();
echo $dll->getIteratorMode(), "\n";
$stack->push(1);
$stack->push(2);
foreach ($stack as $v) {
    echo $v;
}
echo "\n";
$queue->enqueue(1);
$queue->enqueue(2);
foreach ($queue as $v) {
    echo $v;
}
echo "\n";
"#,
    );
    assert_eq!(out, "6\n4\n0\n21\n12\n");
}

/// Verifies `var_dump` renders the SplDoublyLinkedList family's two internal fields.
///
/// php prints the list's private `flags` (iterator mode) and `dllist` (element storage) fields;
/// without a var_dump adapter the class rendered as an empty `(0) {}` object.
#[test]
fn test_spl_doubly_linked_list_var_dump_renders_fields() {
    let out = compile_and_run(
        r#"<?php
$dll = new SplDoublyLinkedList();
$dll->push("hai");
$dll->push("thar");
var_dump($dll);
"#,
    );
    assert_eq!(
        out,
        concat!(
            "object(SplDoublyLinkedList)#1 (2) {\n",
            "  [\"flags\":\"SplDoublyLinkedList\":private]=>\n",
            "  int(0)\n",
            "  [\"dllist\":\"SplDoublyLinkedList\":private]=>\n",
            "  array(2) {\n",
            "    [0]=>\n",
            "    string(3) \"hai\"\n",
            "    [1]=>\n",
            "    string(4) \"thar\"\n",
            "  }\n",
            "}\n",
        )
    );
}

/// Verifies a by-reference `foreach` over an iterator raises php's catchable `Error`.
///
/// php refuses to bind a reference to an iterator's element, which is a copy. The loop is now
/// accepted by the checker and the diagnostic is raised at run time, matching php-src.
#[test]
fn test_by_reference_foreach_over_iterator_throws() {
    let out = compile_and_run(
        r#"<?php
$dll = new SplDoublyLinkedList();
$dll->push(1);
try {
    foreach ($dll as &$v) {
        echo "unreachable";
    }
} catch (Error $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "An iterator cannot be used with foreach by reference\n");
}

/// Verifies an uncaught by-reference `foreach` reports php's source location and stack trace.
///
/// php follows the `Fatal error:` line with ` in <file>:<line>`, `Stack trace:`, `#0 {main}` and
/// `  thrown in <file> on line <line>`. The guard now stamps the `foreach` statement's span on
/// the throwable, so both the report and `getLine()` name that statement.
#[test]
fn test_by_reference_foreach_over_iterator_uncaught_reports_location_and_trace() {
    let err = compile_and_run_expect_failure(
        r#"<?php
$it = new SplDoublyLinkedList();
$it->push(1);
foreach ($it as &$v) {
    echo "unreachable";
}
"#,
    );
    assert!(
        err.contains(
            "Fatal error: Uncaught Error: An iterator cannot be used with foreach by reference in "
        ),
        "{err}"
    );
    assert!(
        err.contains("Stack trace:\n#0 {main}\n  thrown in "),
        "{err}"
    );
}

/// Verifies `SplObjectStorage::current()` rejects a read on an exhausted iterator.
///
/// php raises `RuntimeException: Called current() on invalid iterator` rather than returning null
/// (which is what `key()` does), so a loop that reads past the end gets the same error as php-src.
#[test]
fn test_spl_object_storage_current_on_invalid_iterator_throws() {
    let out = compile_and_run(
        r#"<?php
$s = new SplObjectStorage();
var_dump($s->valid());
try {
    var_dump($s->current());
} catch (RuntimeException $e) {
    echo $e->getMessage(), "\n";
}
"#,
    );
    assert_eq!(out, "bool(false)\nCalled current() on invalid iterator\n");
}

/// Verifies `SplObjectStorage::seek()` rejects an out-of-range position.
///
/// php raises `OutOfBoundsException: Seek position N is out of range` for a negative offset or one
/// at/past the storage length, and leaves the current position untouched.
#[test]
fn test_spl_object_storage_seek_out_of_range_throws() {
    let out = compile_and_run(
        r#"<?php
$s = new SplObjectStorage();
$s[new stdClass()] = 1;
try {
    $s->seek(1);
} catch (OutOfBoundsException $e) {
    echo $e->getMessage(), "\n";
}
try {
    $s->seek(-1);
} catch (OutOfBoundsException $e) {
    echo $e->getMessage(), "\n";
}
$s->seek(0);
var_dump($s->key());
"#,
    );
    assert_eq!(
        out,
        "Seek position 1 is out of range\nSeek position -1 is out of range\nint(0)\n"
    );
}

/// Verifies that ordinary SplFixedArray allocation is unaffected by the storage-size guard.
#[test]
fn test_spl_fixed_array_normal_size_still_works() {
    let out = compile_and_run(
        r#"<?php
$fixed = new SplFixedArray(3);
$fixed[1] = 7;
echo $fixed->getSize(), ":", $fixed[1], ":", $fixed[0] === null ? "null" : "set";
"#,
    );
    assert_eq!(out, "3:7:null");
}

/// `SplFixedArray` accepts an offset the way PHP's `spl_offset_convert_to_long` does: an int, a
/// bool, a float (truncated) or a canonical integer string such as `"1"` is an index, and every
/// other offset raises a TypeError naming its type or class. A numeric string that is not a
/// canonical integer (`"01"`, `"1.0"`, `" 1"`) is rejected like any other string. Covers reads,
/// writes, `isset()`, `unset()`, the method-call form and a `mixed` cell. Regression for #1623.
#[test]
fn test_spl_fixed_array_converts_integer_string_offsets() {
    let out = compile_and_run(
        r#"<?php
class Foo {}
class Holder { public mixed $bag; }
$f = new SplFixedArray(3);
$f["1"] = "one";
echo $f["1"], "|", $f->offsetGet("1"), "|", var_export(isset($f["1"]), true), "|", $f[true], "|", $f[1.0], "\n";
unset($f["1"]);
var_dump($f[1]);
$f->offsetSet("2", "two");
$h = new Holder();
$h->bag = $f;
echo $h->bag["2"], "\n";
foreach (["x", "01", "1.0", " 1", "", null, [1], new Foo()] as $k) {
    try { $v = $f[$k]; echo "read\n"; }
    catch (TypeError $e) { echo $e->getMessage(), "\n"; }
}
try { $f["-1"] = 0; } catch (OutOfBoundsException $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(
        out,
        concat!(
            "one|one|true|one|one\n",
            "NULL\n",
            "two\n",
            "Cannot access offset of type string on SplFixedArray\n",
            "Cannot access offset of type string on SplFixedArray\n",
            "Cannot access offset of type string on SplFixedArray\n",
            "Cannot access offset of type string on SplFixedArray\n",
            "Cannot access offset of type string on SplFixedArray\n",
            "Cannot access offset of type null on SplFixedArray\n",
            "Cannot access offset of type array on SplFixedArray\n",
            "Cannot access offset of type Foo on SplFixedArray\n",
            "OutOfBoundsException: Index invalid or out of range\n",
        )
    );
}

/// `SplDoublyLinkedList` declares `int $index` and takes weak-mode coercion: any fully numeric
/// string (`" 1"`, `"1.0"`, `"+1"`) is an index, a non-numeric string is a TypeError naming
/// `string`, and null reads as index 0 (except in `offsetSet()`, where it appends). Regression
/// for #1623.
#[test]
fn test_spl_doubly_linked_list_coerces_numeric_string_offsets() {
    let out = compile_and_run(
        r#"<?php
$l = new SplDoublyLinkedList();
$l->push("a"); $l->push("b"); $l->push("c");
echo $l["1"], $l[" 1"], $l["1.0"], $l["+1"], $l->offsetGet("2"), "\n";
unset($l["0"]);
$l["1"] = "C";
$l[null] = "d";
echo count($l), $l[0], $l[1], $l[2], "\n";
foreach (["x", "1a", ""] as $k) {
    try { $v = $l[$k]; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
}
try { $l["zz"] = 1; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { unset($l["zz"]); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { $v = isset($l[[1]]); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
$q = new SplQueue();
$q->push(1); $q->push(2);
echo $q["1"], "\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "bbbbc\n",
            "3bCd\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given\n",
            "SplDoublyLinkedList::offsetSet(): Argument #1 ($index) must be of type ?int, string given\n",
            "SplDoublyLinkedList::offsetUnset(): Argument #1 ($index) must be of type int, string given\n",
            "SplDoublyLinkedList::offsetExists(): Argument #1 ($index) must be of type int, array given\n",
            "2\n",
        )
    );
}

/// Converting and rejecting SPL offsets releases every boxed offset and every thrown TypeError's
/// message, over a loop under `--heap-debug`. Regression for #1623.
#[test]
fn test_spl_offset_conversion_heap_is_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
$n = 0;
for ($i = 0; $i < 40 + ($argc > 5 ? 1 : 0); $i++) {
    $f = new SplFixedArray(3);
    $f["1"] = "one" . $i;
    $n += strlen($f["1"]);
    $n += isset($f["2"]) ? 1 : 0;
    unset($f["1"]);
    try { $f["x" . $i] = 1; } catch (TypeError $e) { $n += 1; }
    $n += $f[1.0] === null ? 1 : 0;
    $l = new SplDoublyLinkedList();
    $l->push("a"); $l->push("b" . $i);
    $n += strlen($l[" 1"]);
    unset($l["0"]);
    try { $l["zz"] = 1; } catch (TypeError $e) { $n += 1; }
    $l[null] = "c";
    $n += count($l);
}
echo $n, "\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "500\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// The linked-list family's `int $index` takes a float only when it fits an int: `INF`, `-INF`
/// and `NAN` are `TypeError(... float given)` and a numeric string spelling an out-of-range
/// value (`"1e19"`, `"18446744073709551616"`) is `... string given`, where they used to address a
/// wrapped slot. A lossy float or float-string truncates with PHP's deprecation, and an exact
/// one (`"1.0"`) converts silently. `SplFixedArray` keeps the array-key float diagnostics PHP 8.4
/// emits for it. Review follow-up for #1623.
#[test]
fn test_spl_offset_float_rules_follow_each_container() {
    let out = compile_and_run(
        r#"<?php
set_error_handler(function (int $no, string $msg) { echo "[diag] ", $msg, "\n"; return true; });
function mk(): SplDoublyLinkedList { $o = new SplDoublyLinkedList(); $o->push("zero"); $o->push("one"); $o->push("two"); return $o; }
$l = mk();
foreach ([INF, -INF, NAN, 1.5] as $k) {
    try { echo $l[$k], "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
}
foreach (["1e19", "18446744073709551616", "2.5", "1.0"] as $k) {
    try { echo $l[$k], "\n"; } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
}
try { $l->offsetSet(INF, "x"); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
try { unset($l[NAN]); } catch (TypeError $e) { echo $e->getMessage(), "\n"; }
echo $l[0], count($l), "\n";
$f = new SplFixedArray(3);
$f[0] = "zero"; $f[1] = "one";
echo $f[1.5], "|", $f[INF], "\n";
"#,
    );
    assert_eq!(
        out,
        concat!(
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, float given\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, float given\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, float given\n",
            "[diag] Implicit conversion from float 1.5 to int loses precision\n",
            "one\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given\n",
            "SplDoublyLinkedList::offsetGet(): Argument #1 ($index) must be of type int, string given\n",
            "[diag] Implicit conversion from float-string \"2.5\" to int loses precision\n",
            "two\n",
            "one\n",
            "SplDoublyLinkedList::offsetSet(): Argument #1 ($index) must be of type ?int, float given\n",
            "SplDoublyLinkedList::offsetUnset(): Argument #1 ($index) must be of type int, float given\n",
            "zero3\n",
            "[diag] Implicit conversion from float 1.5 to int loses precision\n",
            "one|[diag] The float INF is not representable as an int, cast occurred\n",
            "zero\n",
        )
    );
}

/// A float offset's diagnostic runs the user error handler while `offsetSet()` still owns the
/// value; a handler that throws must not strand it. The value is registered with the unwinder
/// across the conversion, so both containers stay heap-clean over a loop. Review follow-up for
/// #1623.
#[test]
fn test_spl_offset_set_with_throwing_float_diagnostic_is_heap_clean() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
set_error_handler(function (int $no, string $msg) { throw new RuntimeException($msg); });
$n = 0;
for ($i = 0; $i < 40 + ($argc > 5 ? 1 : 0); $i++) {
    $f = new SplFixedArray(3);
    try { $f[1.5] = "v" . $i; } catch (RuntimeException $e) { $n += 1; }
    $l = new SplDoublyLinkedList();
    $l->push("a"); $l->push("b");
    try { $l[1.5] = "w" . $i; } catch (RuntimeException $e) { $n += 1; }
    try { $l["1.5"] = "x" . $i; } catch (RuntimeException $e) { $n += 1; }
    try { $v = $l["1.5"]; } catch (RuntimeException $e) { $n += 1; }
    $n += count($l);
}
restore_error_handler();
echo $n, "\n";
"#,
    );
    assert!(out.success, "program failed: {}", out.stderr);
    assert_eq!(out.stdout, "240\n");
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}

/// An `offsetSet()` rejection names the rejected offset's type — `resource` here, where `main`
/// used to print the opaque `non-int`. Regression for #1623. (PHP 8.5 itself converts a resource
/// offset with a `Resource ID#N used as offset` warning and then raises `OutOfBoundsException`
/// for an out-of-range index; elephc's rejection of resources is the divergence recorded in
/// docs/php/types.md.) The other `ArrayAccess` operations are pinned in
/// `test_spl_fixed_array_converts_integer_string_offsets`; they depend on the offset-variable
/// handling tracked separately, so they are not asserted here.
#[test]
fn test_spl_fixed_array_resource_offset_names_the_type() {
    let out = compile_and_run(
        r#"<?php
$f = new SplFixedArray(3);
$r = fopen("php://memory", "r+");
try { $f[$r] = 1; } catch (TypeError $e) { echo "set: ", $e->getMessage(), "\n"; }
"#,
    );
    assert_eq!(
        out,
        "set: Cannot access offset of type resource on SplFixedArray\n"
    );
}
