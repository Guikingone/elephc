//! Purpose:
//! End-to-end regressions for issue #1123: writing a borrowed eval scope cell into a property
//! must take a reference, because the slot owns what it holds.
//!
//! Called from:
//! - `cargo test --test codegen_tests eval_property_borrow` through Rust's test harness.
//!
//! Key details:
//! - `$this` and by-value parameters are bound `ScopeCellOwnership::Borrowed`, and
//!   `EvalExpr::LoadVar` hands the stored handle back with no retain. #982 fixed the statements
//!   that keep such a value in the frame or hand it out of it; a property store is the fourth
//!   site, and it needs the third answer: unlike a scope cell, a property has no ownership
//!   label to record, so it must genuinely acquire.
//! - The symptom depends on what happens next. Overwriting the slot releases what it replaces,
//!   which destroyed the lent object outright; a SELF-reference (`$this->me = $this`) instead
//!   trips `--heap-debug` with `bad refcount` while printing the right answer without it.
//! - Instance, static and dynamic-name property stores all reach it. Property ARRAY appends and
//!   keyed sets do not: that path already retains before storing, and the fixtures below pin
//!   that this fix did not disturb it.

use crate::support::{compile_and_run, compile_and_run_with_heap_debug};

/// Verifies a borrowed value stored into an instance property survives the slot being
/// overwritten, whether it came from `$this` or from a by-value parameter.
#[test]
fn test_eval_instance_property_store_retains_a_borrowed_value() {
    let out = compile_and_run(
        r#"<?php
eval('
class EvalPropHolder { public $slot = null; }
class EvalPropBag {
    public int $k = 4;
    public function stash(EvalPropHolder $h): int { $h->slot = $this; $h->slot = 1; return $this->k; }
}

function evalPropStore(EvalPropBag $b, EvalPropHolder $h): int { $h->slot = $b; $h->slot = 1; return 9; }

$o = new EvalPropBag();
$h = new EvalPropHolder();
echo $o->stash($h), "|";
echo evalPropStore($o, $h), "|";
echo ($o instanceof EvalPropBag) ? "yes" : "no", "|", $o->k, "|";
');
"#,
    );

    assert_eq!(out, "4|9|yes|4|");
}

/// Verifies the same for a STATIC property and for a dynamic property name, which reach
/// different setters.
#[test]
fn test_eval_static_and_dynamic_property_stores_retain_a_borrowed_value() {
    let out = compile_and_run(
        r#"<?php
eval('
class EvalStaticHolder { public static $slot = null; }
class EvalDynHolder { public $slot = null; }
class EvalStoreBag { public int $k = 7; }

function evalStaticStore(EvalStoreBag $b): int { EvalStaticHolder::$slot = $b; EvalStaticHolder::$slot = 1; return 9; }
function evalDynStore(EvalStoreBag $b, EvalDynHolder $h): int { $n = "slot"; $h->$n = $b; $h->$n = 1; return 9; }

$o = new EvalStoreBag();
$h = new EvalDynHolder();
echo evalStaticStore($o), "|";
echo evalDynStore($o, $h), "|";
echo ($o instanceof EvalStoreBag) ? "yes" : "no", "|", $o->k, "|";
');
"#,
    );

    assert_eq!(out, "9|9|yes|7|");
}

/// Verifies a self-referencing property write no longer trips the heap validator.
///
/// This is the shape the issue was filed with, and the one that is silent without
/// `--heap-debug`: the program prints the right answer while the receiver's refcount is one too
/// low. The fixture asserts the program merely SURVIVES instrumentation — `$this->me = $this` is
/// a reference cycle, so the object is unreachable for a refcounting collector and a clean heap
/// is not the bar. What must not happen is `bad refcount`.
#[test]
fn test_eval_self_referencing_property_write_survives_heap_debug() {
    let out = compile_and_run_with_heap_debug(
        r#"<?php
eval('
class EvalSelfRef {
    public ?EvalSelfRef $me = null;
    public function __construct() { $this->me = $this; }
    public function k(): int { return 1; }
}
$t = 0;
for ($i = 0; $i < 200; $i++) { $o = new EvalSelfRef(); $t = $t + $o->k(); }
echo $t;
');
"#,
    );

    assert!(
        out.success,
        "a self-referencing property write tripped the heap validator: {}",
        out.stderr
    );
    assert_eq!(out.stdout, "200");
    assert!(
        !out.stderr.contains("bad refcount"),
        "expected no refcount complaint, got: {}",
        out.stderr
    );
}

/// Verifies property ARRAY stores were already correct and stay that way.
///
/// The append and keyed-set paths retain before storing, so they never had this defect. They are
/// pinned because the fix deliberately does not touch them: a second retain here would leak.
#[test]
fn test_eval_property_array_stores_are_unaffected() {
    let out = compile_and_run(
        r#"<?php
eval('
class EvalArrHolder { public array $slots = []; }
class EvalArrBag { public int $k = 4; }

function evalArrAppend(EvalArrBag $b, EvalArrHolder $h): int { $h->slots[] = $b; $h->slots = []; return 1; }
function evalArrKeyed(EvalArrBag $b, EvalArrHolder $h): int { $h->slots["x"] = $b; $h->slots = []; return 2; }
function evalArrFresh(EvalArrHolder $h): int { $h->slots[] = new EvalArrBag(); $h->slots = []; return 3; }

$o = new EvalArrBag();
$h = new EvalArrHolder();
echo evalArrAppend($o, $h), evalArrKeyed($o, $h), evalArrFresh($h), "|";
echo ($o instanceof EvalArrBag) ? "yes" : "no", "|", $o->k, "|";
');
"#,
    );

    assert_eq!(out, "123|yes|4|");
}
