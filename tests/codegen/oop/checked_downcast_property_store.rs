//! Purpose:
//! Integration and regression tests for the checked downcast at a PROPERTY-STORE boundary: a value
//! statically known only as an ancestor (or as an unrelated INTERFACE) may be written into a
//! narrower declared property, accepted ONLY because a runtime guard is emitted at the write —
//! passing the value through on a match and throwing PHP's own catchable `\TypeError` on mismatch.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - php-8.5.6 wording, verified against `php -n`: this position names the runtime type in the
//!   MIDDLE (`Cannot assign A to property C::$p of type B`), not at the end like the return and
//!   argument forms. The class named is the one that DECLARES the slot, not the receiver's.
//! - OWNERSHIP is the hard part here, and it is decided per VALUE rather than per position: a local
//!   source keeps its owner (releasing would double free), an owning temporary has no other owner
//!   (not releasing would leak). Both directions are pinned below by heap accounting, because
//!   neither shows up as a wrong value — the double free surfaced only as
//!   `heap debug detected bad refcount` under a 300-iteration probe.
//! - The SIDEWAYS case (an interface source into an unrelated interface slot) is not an oversight
//!   of PHP's: one concrete class routinely implements two unrelated interfaces, so the guard's
//!   `instanceof` legitimately matches and PHP, which checks nothing statically here, runs it.

use super::*;

/// The headline: an ancestor-typed value written into a narrower declared property throws PHP's
/// own catchable `TypeError`, with PHP's exact wording, and the program keeps running.
#[test]
fn test_property_store_wrong_class_throws_catchable_type_error() {
    let out = compile_and_run(
        r#"<?php
interface I {}
class A implements I {}
class B implements I { public int $n = 1; }
class Holder {
    public B $slot;
    public function set(I $v): void { $this->slot = $v; }
}
function pick(bool $b): I { return $b ? new B() : new A(); }
$h = new Holder();
try { $h->set(pick(true)); echo "ok-B\n"; } catch (\TypeError $e) { echo "unexpected\n"; }
try { $h->set(pick(false)); } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "ok-B\nCannot assign A to property Holder::$slot of type B\nalive"
    );
}

/// An INTERFACE-typed source may be written into a slot declared as an UNRELATED interface: the
/// guard's `instanceof` decides it at runtime, exactly as PHP does.
///
/// The refusal this replaces was over-strict only for an interface source. A CLASS source fixes its
/// whole ancestry, so an unrelated target could never match and the flow stays a compile error.
#[test]
fn test_property_store_sideways_interface_is_decided_at_runtime() {
    let out = compile_and_run(
        r#"<?php
interface IA {}
interface IB {}
class Both implements IA, IB { public int $n = 5; }
class OnlyA implements IA {}
class Holder { public IB $slot; }
function mk(bool $b): IA { return $b ? new Both() : new OnlyA(); }
$h = new Holder();
$x = mk(true);
try { $h->slot = $x; echo "ok-both\n"; } catch (\TypeError $e) { echo "unexpected\n"; }
$y = mk(false);
try { $h->slot = $y; } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "ok-both\nCannot assign OnlyA to property Holder::$slot of type IB\nalive"
    );
}

/// The message names the DECLARING class, not the receiver's — PHP reports where the slot was
/// declared even when the write goes through a subclass handle.
#[test]
fn test_property_store_message_names_the_declaring_class() {
    let out = compile_and_run(
        r#"<?php
interface I {}
class A implements I {}
class B implements I {}
class Base { public B $slot; }
class Child extends Base {}
function pick(): I { return new A(); }
$c = new Child();
try { $c->slot = pick(); } catch (\TypeError $e) { echo $e->getMessage(), "\n"; }
echo "alive";
"#,
    );
    assert_eq!(
        out,
        "Cannot assign A to property Base::$slot of type B\nalive"
    );
}

/// OWNERSHIP, direction 1: a LOCAL source keeps its owner across the throw. Releasing it there is a
/// double free that no type or validation check catches — it surfaced only as
/// `heap debug detected bad refcount` on the 300th iteration.
///
/// The local stays usable after the loop, and the heap is exactly balanced.
#[test]
fn test_property_store_throw_does_not_release_a_source_the_local_still_owns() {
    let out = compile_and_run(
        r#"<?php
interface IA {}
interface IB {}
class OnlyA implements IA { public int $k = 3; }
class Holder { public IB $slot; }
function mk(): IA { return new OnlyA(); }
$h = new Holder();
$x = mk();
for ($i = 0; $i < 300; $i++) {
    try { $h->slot = $x; } catch (\TypeError $e) { }
}
echo ($x instanceof IA ? "Y" : "N"), $x->k;
"#,
    );
    assert_eq!(out, "Y3");
}

/// OWNERSHIP, direction 2: an OWNING TEMPORARY has no other owner once the store is skipped, so the
/// throw must release it. Pinned by BLOCK COUNT rather than by an absolute number: the throwing loop
/// and the succeeding loop must leave the same number of live blocks, since the only difference
/// between them is which path disposed of the temporary.
///
/// The residual live blocks both paths share are the pre-existing property-reassign leak, which is
/// why this asserts equality rather than zero.
#[test]
fn test_property_store_throw_releases_an_owning_temporary_source() {
    let source = |succeeds: &str| {
        format!(
            r#"<?php
interface IA {{}}
interface IB {{}}
class Both implements IA, IB {{ public int $n = 5; }}
class OnlyA implements IA {{}}
class Holder {{ public IB $slot; }}
function mk(bool $b): IA {{ return $b ? new Both() : new OnlyA(); }}
$h = new Holder();
for ($i = 0; $i < 100; $i++) {{
    try {{ $h->slot = mk({}); }} catch (\TypeError $e) {{ }}
}}
echo "done";
"#,
            succeeds
        )
    };
    let throwing = compile_and_run_with_heap_debug(&source("false")).stderr;
    let succeeding = compile_and_run_with_heap_debug(&source("true")).stderr;
    assert_eq!(
        live_blocks(&throwing),
        live_blocks(&succeeding),
        "the throwing path must dispose of the owning temporary exactly as the storing path does;\nthrowing:\n{}\nsucceeding:\n{}",
        throwing,
        succeeding
    );
}

/// Extracts the `live_blocks=N` figure from a `--heap-debug` run's output.
fn live_blocks(output: &str) -> u64 {
    output
        .split_whitespace()
        .find_map(|token| token.strip_prefix("live_blocks="))
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("no live_blocks= in heap-debug output:\n{}", output))
}
