//! Purpose:
//! Regression coverage for issue #703: a union-typed local narrowed with `instanceof` and then
//! passed to a parameter that wants the object must arrive as the object, not as its box.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - A `Box|bool` local lives in boxed Mixed storage, because the slot's type is flow-insensitive
//!   even where the checker has narrowed it. The callee's parameter is a concrete `Heap(Object)`,
//!   so something has to unbox between the two; when nothing did, the callee read the property
//!   off the box address and printed a pointer instead of the value.
//! - The failure was SILENT -- no diagnostic, no crash, a different wrong number on every run --
//!   which is why these fixtures assert the value rather than merely that the program compiles.
//! - `test_narrowed_union_argument_is_unboxed_at_the_call_site` pins the mechanism, so the shape
//!   cannot regress into "accidentally right" if the property ever moves to offset zero.

use super::*;

/// Issue repro: the callee used to receive the Mixed box and read `num_rows` off its address.
#[test]
fn test_instanceof_narrowed_union_reaches_a_typed_object_parameter() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $num_rows = 0; }
function get(): Box|bool { $b = new Box(); $b->num_rows = 2; return $b; }
function readnum(Box $b): int { return $b->num_rows; }
$r = get();
if (!($r instanceof Box)) { exit(1); }
echo readnum($r), "|", $r->num_rows;
"#,
    );
    assert_eq!(out, "2|2");
}

/// Verifies every union flavour that boxes its local reaches the parameter as the object.
///
/// `?Box`, `Box|bool`, a three-member union and a two-class union all share one runtime
/// representation, and each one used to hand the callee a different heap address.
#[test]
fn test_every_boxed_union_flavour_reaches_the_parameter_as_an_object() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 0; }
class Other { public int $n = 0; }
function readnum(Box $b): int { return $b->n; }

function mkBool(): Box|bool { $b = new Box(); $b->n = 2; return $b; }
function mkNull(): ?Box { $b = new Box(); $b->n = 3; return $b; }
function mkWide(): Box|bool|int { $b = new Box(); $b->n = 4; return $b; }
function mkTwoClass(): Box|Other { $b = new Box(); $b->n = 5; return $b; }

foreach ([mkBool(), mkNull(), mkWide(), mkTwoClass()] as $v) {
    if ($v instanceof Box) { echo readnum($v); }
}
"#,
    );
    assert_eq!(out, "2345");
}

/// Verifies the unbox happens for every call shape, not only a plain function call.
#[test]
fn test_narrowed_union_reaches_every_call_shape() {
    let out = compile_and_run(
        r#"<?php
class Box { public int $n = 0; }
class Reader {
    public function read(Box $b): int { return $b->n; }
    public static function sread(Box $b): int { return $b->n; }
}
function readnum(Box $b): int { return $b->n; }
function readTwo(Box $a, Box $b): int { return $a->n * 10 + $b->n; }
function mk(int $n): Box|bool { $b = new Box(); $b->n = $n; return $b; }

$v = mk(2);
if (!($v instanceof Box)) { exit(1); }
$reader = new Reader();
echo readnum($v), $reader->read($v), Reader::sread($v), readTwo($v, $v);
echo (function (Box $b): int { return $b->n; })($v);
"#,
    );
    assert_eq!(out, "222222");
}

/// Verifies the call site unboxes rather than passing the Mixed cell straight through.
///
/// The EIR still types the argument `Heap(Mixed)` and the parameter `Heap(Object)`, so the
/// unbox is the backend's job. Asserting the value alone would keep passing if the property
/// ever sat at offset zero, where a box address and an object address read the same.
#[test]
fn test_narrowed_union_argument_is_unboxed_at_the_call_site() {
    let dir = make_cli_test_dir("elephc_narrowed_object_argument_unbox");
    let (user_asm, _runtime_asm, _required_libraries) = compile_source_to_asm_with_options(
        r#"<?php
class Box { public int $n = 0; }
function readnum(Box $b): int { return $b->n; }
function mk(): Box|bool { $b = new Box(); $b->n = 2; return $b; }
$v = mk();
if ($v instanceof Box) { echo readnum($v); }
"#,
        &dir,
        8_388_608,
        false,
        false,
    );

    // One spelling per target, and the leading indent matters on all of them: the directive
    // `.globl _fn_readnum` contains "bl _fn_readnum" as a substring.
    let call = ["\n    bl _fn_readnum", "\n    bl fn_readnum", "\n    call fn_readnum"]
        .iter()
        .find_map(|marker| user_asm.split_once(*marker))
        .map(|(before, _)| before)
        .expect("the fixture must call readnum");
    assert!(
        call.contains("__rt_mixed_unbox"),
        "the narrowed union argument must be unboxed before the call: {}",
        &call[call.len().saturating_sub(700)..]
    );
}
