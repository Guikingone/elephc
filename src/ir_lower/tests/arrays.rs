//! Purpose:
//! Regression tests for AST-to-EIR lowering of indexed array expressions.
//!
//! Called from:
//! - `crate::ir_lower::tests`.
//!
//! Key details:
//! - Array access result metadata must come from the lowered array value, not
//!   from syntactic fallback inference that lacks local type facts.

use crate::ir::print_module;

/// Verifies indexed array access preserves string and float element metadata.
/// The indices are runtime-unknown (`$argc`) so the accesses survive AST-level
/// array-fact propagation, which folds constant-index reads of literal-backed
/// locals before lowering.
#[test]
fn indexed_array_access_uses_array_element_type() {
    let module = super::lower_source(
        r#"<?php
$strings = ["a", "b"];
echo $strings[$argc];
$floats = [1.5, 2.5];
echo $floats[$argc];
"#,
    );
    let text = print_module(&module);
    assert!(
        text.contains(": Str php=string own=maybe_owned = array_get"),
        "missing string array_get metadata in {text}"
    );
    assert!(
        text.contains(": F64 php=float = array_get"),
        "missing float array_get metadata in {text}"
    );
}

/// An array local RE-BOUND to an incompatible type inside an array-representation fixed-point
/// region keeps its old slot at the old representation, and the new value gets a fresh one.
///
/// The ternary makes this statement a conversion-HIDING region, so `lower_region_at_type_fixpoint`
/// runs its speculative discovery pass over it while `$a` is still a convertible `array<int>`
/// candidate. Two things have to come out of that:
///
/// - nothing is canonicalized at the region entry. Canonicalization emits `Op::ArrayToMixed` /
///   `Op::ArrayToHash` against the binding live at region ENTRY, and this region ENDS that
///   binding; a conversion belonging to the fresh slot must not be hoisted onto the old one.
/// - the old slot is not widened. The rebind releases and nulls it through the ordinary overwrite
///   path, and that path widens the slot to whatever type it stores — a whole-frame property, so
///   nulling at `Void` would re-type every load of that slot the body already lowered.
#[test]
fn retype_in_a_conversion_region_rebinds_without_widening_the_old_slot() {
    let module = super::lower_source(
        r#"<?php
$a = [1, $argc];
$a = $argc > 0 ? "s" . $argc : "t";
echo $a;
"#,
    );
    let text = print_module(&module);
    assert!(
        !text.contains("array_to_mixed") && !text.contains("array_to_hash"),
        "the fixed point canonicalized a binding the region re-binds: {text}"
    );
    assert!(
        text.contains("php=array<int> own=owned = load_local slot[1]"),
        "the abandoned slot lost its concrete array<int> storage type: {text}"
    );
    assert!(
        !text.contains("php=string own=maybe_owned = load_local slot[1]"),
        "the re-bound string was stored through the OLD slot instead of a fresh one: {text}"
    );
}

/// A local the checker marked as branch-divergently assigned gets a boxed `mixed` slot before its
/// FIRST store and keeps it through the array-representation fixed point.
///
/// The `if` here is a conversion-hiding region (`$b[0] = "x"` promotes `$b` to `array<mixed>`), so
/// `lower_region_at_type_fixpoint` lowers the whole statement SPECULATIVELY, reads the discovered
/// conversions, rolls everything back and lowers it again. `$a`'s first store — and therefore the
/// `declare_local(.., Mixed)` that precedes it — happens inside that region, so this is the shape
/// that pins the marked slot's stability under the pass:
///
/// - the marked name is never a canonicalization CANDIDATE: `convertible_array_locals` only picks
///   names whose `local_types` entry is an `Array(_)`, and a marked name's is `Mixed` from its
///   first store onwards, so no `Op::ArrayToMixed`/`Op::ArrayToHash` is ever hoisted onto it;
/// - the rollback (`LoweringContext::restore`) puts back `local_slots`, `local_types` and the whole
///   function, so the discarded pass leaves no slot behind and the real pass re-declares the same
///   `Mixed` one from the same recorded span;
/// - and no later store can narrow it: `widened_local_storage_type` answers `Mixed` for every
///   `(Mixed, other)` pair, so the `int` arm's store widens nothing.
///
/// Both arms therefore read slot 2 as `mixed`, which is what makes the two dynamic outcomes share
/// one binding.
#[test]
fn mixed_storage_local_keeps_its_boxed_slot_through_the_fixed_point() {
    let module = super::lower_source(
        r#"<?php
$b = [1, $argc];
if ($argc > 1) { $a = 0; $b[0] = "x"; } else { $a = "ciao"; }
echo $a, "|", $b[0];
"#,
    );
    let text = print_module(&module);
    assert_eq!(
        text.matches("php=mixed own=maybe_owned = load_local slot[2]")
            .count(),
        2,
        "both arms must read the marked local out of one boxed slot: {text}"
    );
    assert!(
        !text.contains("php=int = load_local slot[2]")
            && !text.contains("php=string own=maybe_owned = load_local slot[2]"),
        "the marked local's slot was narrowed to a concrete representation: {text}"
    );
    assert_eq!(
        text.matches("array_to_mixed").count(),
        1,
        "exactly one conversion belongs here — `$b`'s, hoisted to the region entry; a second one \
         would mean the marked local was canonicalized as if it were a convertible array: {text}"
    );
}
