//! Purpose:
//! Separates user property contracts from PHP zval storage on native SPL heap descendants.
//!
//! Called from:
//! - SPL metadata refinement before and after body checking, and after final global inference.
//!
//! Key details:
//! - Native object_properties_load can replace a typed property's payload without a type check.
//! - Ordinary PHP writes and Reflection retain the original declared contract by physical index.
//! - Reserved NUL-prefixed heap storage keeps its canonical native ABI and specialized types.

use super::super::Checker;
use crate::types::PhpType;

/// Boxes native-loadable user slots while preserving the first registered declaration contract.
pub(crate) fn box_heap_user_property_storage(checker: &mut Checker) {
    let classes: Vec<String> = checker.classes.keys()
        .filter(|name| inherits_native_heap(checker, name))
        .cloned().collect();
    for name in classes {
        let info = checker.classes.get_mut(&name).expect("registered heap descendant");
        // Native loading may discover property names absent from every source expression.
        // Reserve GC-visible storage without granting PHP dynamic-property permission.
        info.scope_dynamic_property_storage = true;
        for index in 0..info.properties.len() {
            let (property, ty) = &info.properties[index];
            if property.starts_with('\0') { continue; }
            // Reference fields still share the caller's typed cell ABI. Keep that shape until
            // the native loader detaches the binding with a per-instance representation marker.
            if info.property_slot_is_reference(index, property) {
                if let Some(Some(contract)) = info.native_property_contracts.remove(&index) {
                    info.properties[index].1 = contract;
                }
                continue;
            }
            let contract = info.property_slot_is_declared(index, property).then(|| ty.clone());
            info.native_property_contracts.entry(index).or_insert(contract);
            info.properties[index].1 = PhpType::Mixed;
        }
    }
}

/// Requires the compiler's internal owner, so matching user text cannot grant native authority.
fn inherits_native_heap(checker: &Checker, name: &str) -> bool {
    let mut current = Some(name);
    while let Some(name) = current {
        let Some(info) = checker.classes.get(name) else { return false; };
        if info.is_internal && matches!(name, "SplHeap" | "SplPriorityQueue") { return true; }
        current = info.parent.as_deref();
    }
    false
}
