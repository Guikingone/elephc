//! Purpose:
//! Covers property defaults whose elements are themselves array literals, through AOT and
//! opaque eval allocation, under `--heap-debug`. Each nested container is allocated during the
//! object's initialization and handed to the container enclosing it, so the whole tree has to be
//! owned by its root and released exactly once with the object.
//!
//! Called from:
//! - The native codegen regression harness on executable CI targets.
//!
//! Key details:
//! - Mixed defaults combine indexed arrays, hashes, empty children, and normalized duplicate keys.
//! - Independent instances and repeated destruction exercise nested owner transfer and COW.
//! - Each fixture asserts `leak summary: clean`. The two ways to get the transfer wrong land on
//!   opposite sides of that assertion: retaining the child without releasing the builder's
//!   reference leaks the whole subtree once per object, and releasing it without the retain
//!   frees a child the object still points at, which shows up as corrupted reads or a double
//!   free rather than as a leak -- so the fixtures read the values back as well.
//! - The loops allocate hundreds of objects, so a per-object leak cannot hide in heap slack.
//! - Expected stdout values are real `LC_ALL=C php` 8.5 output for the same fixtures.

use crate::support::*;

const NATIVE_DEFAULTS: &str = r#"
class NativeNestedDefaults {
    public mixed $list = [1, [2, 3], [], ["key" => [4, "value"]]];
    public mixed $hash = ["tree" => ["leaf" => [5, true, null]], "2" => ["old"], 2 => ["new"]];
}
"#;

/// Opaque eval allocates native defaults, including inherited physical slots and nested hash values.
#[test]
fn test_core_native_nested_defaults_read_from_eval() {
    let source = format!(r#"<?php
{NATIVE_DEFAULTS}
$source = 'class EvalNestedDefaults extends NativeNestedDefaults {{}}
foreach ([new NativeNestedDefaults(), new EvalNestedDefaults()] as $object) {{
    echo $object->list[0], ":", $object->list[1][1], ":", count($object->list[2]), ":";
    echo $object->list[3]["key"][1], ":", $object->hash["tree"]["leaf"][0], ":";
    echo $object->hash["tree"]["leaf"][1] ? "true:" : "bad:";
    echo $object->hash["tree"]["leaf"][2] === null ? "null:" : "bad:";
    echo $object->hash[2][0], "|";
}}' . ' // ' . $argc;
eval($source);
"#);
    assert_eq!(compile_and_run(&source), "1:3:0:value:5:true:null:new|1:3:0:value:5:true:null:new|");
}

/// Direct AOT defaults remain independent across both instance allocation and detached array writes.
#[test]
fn test_core_native_nested_defaults_preserve_cow() {
    let source = format!(r#"<?php
{NATIVE_DEFAULTS}
$first = new NativeNestedDefaults();
$second = new NativeNestedDefaults();
$copy = $first->list;
$copy[1][0] = 9;
$first->hash["tree"]["leaf"][0] = 8;
echo $copy[1][0], ":", $first->list[1][0], ":", $second->list[1][0], "|";
echo $first->hash["tree"]["leaf"][0], ":", $second->hash["tree"]["leaf"][0];
"#);
    assert_eq!(compile_and_run(&source), "9:2:2|8:5");
}

/// Mutating the original property or its detached copy leaves the other nested value unchanged.
#[test]
fn test_core_native_nested_defaults_cow_in_both_directions() {
    let source = format!(r#"<?php
{NATIVE_DEFAULTS}
$object = new NativeNestedDefaults();
$list = $object->list;
$hash = $object->hash;
$object->list[1][0] = 8;
$object->hash["tree"]["leaf"][0] = 9;
echo $list[1][0], ":", $hash["tree"]["leaf"][0], "|";
$list[1][1] = 10;
$hash["tree"]["leaf"][1] = false;
echo $object->list[1][0], ":", $object->list[1][1], ":", $list[1][1], "|";
echo $object->hash["tree"]["leaf"][0], ":";
echo $object->hash["tree"]["leaf"][1] ? "true:" : "false:";
echo $hash["tree"]["leaf"][1] ? "true" : "false";
"#);
    assert_eq!(compile_and_run(&source), "2:5|8:3:10|9:true:false");
}

/// A by-reference Mixed root publishes separated cells through its caller slot, preserving aliases.
#[test]
fn test_core_native_nested_defaults_reference_root_preserves_aliases() {
    let source = format!(r#"<?php
{NATIVE_DEFAULTS}
function mutateNestedRoot(mixed &$tree): void {{ $tree[1][0] = 9; }}
$object = new NativeNestedDefaults();
$copy = $object->list;
$alias =& $copy;
mutateNestedRoot($alias);
echo $copy[1][0], ":", $alias[1][0], ":", $object->list[1][0];
"#);
    assert_eq!(compile_and_run(&source), "9:9:2");
}

/// Static Mixed defaults retain nested indexed/hash storage and scalar runtime tags.
#[test]
fn test_core_native_static_nested_defaults_preserve_values() {
    let source = r#"<?php
class StaticNestedDefaults {
    public static mixed $list = [[1.5, 2.5], [], [true, false]];
    public static mixed $hash = ["tree" => ["leaf" => [3, "value"]]];
}
echo StaticNestedDefaults::$list[0][1], ":", count(StaticNestedDefaults::$list[1]), ":";
echo gettype(StaticNestedDefaults::$list[2][0]), ":", StaticNestedDefaults::$hash["tree"]["leaf"][1];
"#;
    assert_eq!(compile_and_run(source), "2.5:0:boolean:value");
}

/// Every nested literal allocation is released when a directly native eval object is destroyed.
#[test]
fn test_core_native_nested_default_owners_release() {
    super::core_builtins::assert_core_eval_collection_cleanup_with_native(
        NATIVE_DEFAULTS, "", "$object = new NativeNestedDefaults(); unset($object);",
    );
}

/// Inherited native defaults transfer ownership into eval subclasses without retaining hidden owners.
#[test]
fn test_core_inherited_nested_default_owners_release() {
    super::core_builtins::assert_core_eval_collection_cleanup_with_native(
        NATIVE_DEFAULTS, "class EvalNestedDefaults extends NativeNestedDefaults {}",
        "$object = new EvalNestedDefaults(); unset($object);",
    );
}

/// Repeated native nested mutations release the detached root, child zvals, and container owners.
#[test]
fn test_core_native_nested_write_owners_release() {
    let native = format!(r#"{NATIVE_DEFAULTS}
function mutate_native_nested_defaults(): void {{
    $object = new NativeNestedDefaults();
    $copy = $object->list;
    $copy[1][0] = 9;
    $object->hash["tree"]["leaf"][0] = 8;
    unset($copy, $object);
}}
"#);
    super::core_builtins::assert_core_eval_collection_cleanup_with_native(
        &native, "", "mutate_native_nested_defaults();",
    );
}


/// Asserts the program printed `expected` and left a clean heap under heap debug.
fn assert_clean(out: crate::support::ProgramOutput, expected: &str) {
    assert_eq!(out.stdout, expected, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("HEAP DEBUG: leak summary: clean"),
        "expected clean heap, got: {}",
        out.stderr
    );
}

/// An indexed default holding indexed literals releases its whole tree with the object.
#[test]
fn test_indexed_nested_property_default_releases_with_the_object() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public array $x = [[1], [2]]; }
$t = 0;
for ($i = 0; $i < 300; $i++) {
    $a = new A();
    $t += count($a->x) + $a->x[1][0];
    unset($a);
}
echo $t;
"#,
    );
    assert_clean(out, "1200");
}

/// The keyed and boxed spellings release too, including a tree that mixes both.
///
/// The hash path transfers ownership differently from the indexed one -- `__rt_hash_set` takes
/// the value it stores rather than retaining it, where the indexed path boxes into a Mixed cell
/// and releases the builder's reference -- so a single fixture cannot cover both.
#[test]
fn test_keyed_and_boxed_nested_property_defaults_release_with_the_object() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class B { public mixed $x = ["a" => [1, 2], "b" => ["c" => "d"], "e" => 3]; }
class C { public array $x = ["k" => ["j" => 7]]; }
$t = 0;
for ($i = 0; $i < 300; $i++) {
    $b = new B();
    $t += count($b->x) + $b->x["a"][1];
    unset($b);
    $c = new C();
    $t += $c->x["k"]["j"];
    unset($c);
}
echo $t;
"#,
    );
    assert_clean(out, "3600");
}

/// A deep tree with string, float and null leaves stays balanced.
///
/// Strings are the leaf that can go wrong independently: they are persisted rather than
/// refcounted like a container, so a tree carrying both has to get two ownership rules right
/// at once.
#[test]
fn test_deep_nested_property_default_with_mixed_leaves_releases_cleanly() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class D { public array $x = [[[1, "s", 2.5, null, true]]]; }
$t = 0;
for ($i = 0; $i < 300; $i++) {
    $d = new D();
    $t += count($d->x[0][0]);
    unset($d);
}
echo $t;
"#,
    );
    assert_clean(out, "1500");
}

/// A copy taken out of the default outlives the object it came from.
///
/// This is the over-release side of the contract: if the object's release freed a child the
/// copy still holds, reading the copy afterwards reads freed memory. The loop repeats it so a
/// recycled block would come back with someone else's contents.
#[test]
fn test_copy_of_a_nested_default_survives_its_object() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
class A { public array $x = [[1], [2]]; }
$t = 0;
for ($i = 0; $i < 300; $i++) {
    $a = new A();
    $inner = $a->x[0];
    unset($a);
    $t += $inner[0] + count($inner);
    unset($inner);
}
echo $t;
"#,
    );
    assert_clean(out, "600");
}
