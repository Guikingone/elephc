//! Purpose:
//! Emits physical property metadata for ordinary object equality.
//!
//! Called from:
//! - `super::emit_runtime_data_user()` while declaring class descriptor tables.
//!
//! Key details:
//! - Debug projections do not remove PHP slots; compiler-private native slots stay separate.
//! - The 48-byte row ABI matches the shared owned boxed property reader.
//! - Ancestor metadata resolves shadowed private slots by physical index.

use crate::codegen_support::runtime::compare::NativeObjectComparisonKind;
use crate::parser::ast::Visibility;
use crate::types::spl_storage_layout::{INFOS_PROPERTY, OBJECTS_PROPERTY};
use crate::types::{ClassInfo, PhpType};
use std::collections::{BTreeMap, HashMap};

/// Emits dense class-id pointers followed by each class's physical property rows.
pub(super) fn emit_comparison_data(out: &mut String, classes: &HashMap<String, ClassInfo>) {
    let by_id = classes.values().map(|info| (info.class_id, info)).collect::<BTreeMap<_, _>>();
    out.push_str("    .p2align 3\n.globl _class_cmp_prop_desc_ptrs\n_class_cmp_prop_desc_ptrs:\n");
    if let Some(max_id) = by_id.keys().next_back() {
        for id in 0..=*max_id {
            if by_id.contains_key(&id) {
                out.push_str(&format!("    .quad _class_cmp_prop_desc_{id}\n"));
            } else {
                out.push_str("    .quad _class_prop_desc_missing\n");
            }
        }
    }
    out.push_str("    .p2align 3\n.globl _class_cmp_native_descs\n_class_cmp_native_descs:\n");
    if let Some(max_id) = by_id.keys().next_back() {
        for id in 0..=*max_id {
            let (kind, objects, infos) = by_id.get(&id)
                .map(|info| native_comparison_layout(classes, info))
                .unwrap_or((NativeObjectComparisonKind::Standard, 0, 0));
            out.push_str(&format!("    .quad {}\n    .quad {objects}\n    .quad {infos}\n", kind as i64));
        }
    }
    for (id, info) in by_id {
        let properties = info.properties.iter().enumerate()
            .filter(|(_, (name, _))| !name.starts_with('\0'))
            .collect::<Vec<_>>();
        out.push_str(&format!(
            "    .p2align 3\n.globl _class_cmp_prop_desc_{id}\n_class_cmp_prop_desc_{id}:\n    .quad {}\n",
            properties.len(),
        ));
        for (index, (name, ty)) in properties {
            let metadata = physical_slot_metadata(classes, info, index, name);
            let offset = metadata.property_offsets.get(name).copied().unwrap_or(8 + index * 16);
            let tag = if matches!(ty, PhpType::Void) && !info.property_slot_is_declared(index, name) {
                7
            } else {
                super::prop_value_tag(metadata, name, ty)
            };
            // Comparison reads only byte offset and tag; renderer name words are unused.
            out.push_str(&format!("    .quad 0\n    .quad 0\n    .quad {offset}\n    .quad {tag}\n    .quad 0\n    .quad 0\n"));
        }
    }
}

/// Finds the nearest ancestor whose name-keyed metadata still addresses this slot.
fn physical_slot_metadata<'a>(
    classes: &'a HashMap<String, ClassInfo>,
    info: &'a ClassInfo,
    index: usize,
    property: &str,
) -> &'a ClassInfo {
    let mut metadata = info;
    loop {
        if metadata.visible_property_index(property) == Some(index) {
            return metadata;
        }
        let Some(parent) = metadata.parent.as_deref() else { return info; };
        let Some(parent_info) = classes.get(parent) else { return info; };
        metadata = parent_info;
    }
}

/// Separates native storage equality from standard PHP property equality.
fn native_comparison_layout(
    classes: &HashMap<String, ClassInfo>,
    info: &ClassInfo,
) -> (NativeObjectComparisonKind, usize, usize) {
    let Some(objects_index) = info.properties.iter().position(|(name, _)| name == OBJECTS_PROPERTY) else {
        return (NativeObjectComparisonKind::Standard, 0, 0);
    };
    let Some(infos_index) = info.properties.iter().position(|(name, _)| name == INFOS_PROPERTY) else {
        return (NativeObjectComparisonKind::Standard, 0, 0);
    };
    let objects_owner = physical_slot_metadata(classes, info, objects_index, OBJECTS_PROPERTY);
    let infos_owner = physical_slot_metadata(classes, info, infos_index, INFOS_PROPERTY);
    if objects_owner.property_visibilities.get(OBJECTS_PROPERTY) != Some(&Visibility::Private)
        || infos_owner.property_visibilities.get(INFOS_PROPERTY) != Some(&Visibility::Private)
    {
        return (NativeObjectComparisonKind::Standard, 0, 0);
    }
    // php-src compares only exact builtin instances; distinct subclass instances are uncomparable.
    if info.parent.is_some() {
        return (NativeObjectComparisonKind::Uncomparable, 0, 0);
    }
    (
        NativeObjectComparisonKind::ObjectStorage,
        objects_owner.property_offsets.get(OBJECTS_PROPERTY).copied().unwrap_or(8 + objects_index * 16),
        infos_owner.property_offsets.get(INFOS_PROPERTY).copied().unwrap_or(8 + infos_index * 16),
    )
}
