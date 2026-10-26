//! Purpose:
//! Emits PHP property-projection policy separately from serialization and GC layout.
//!
//! Called from:
//! - `crate::codegen_support::runtime::data::user::emit_runtime_data_user()`.
//!
//! Key details:
//! - SPL array-wrapper backing slots are compiler storage, not PHP declarations.
//! - User properties retain their visibility through their actual declaring class.

use std::collections::HashMap;

use crate::types::ClassInfo;

/// Emits dense class and slot tables deciding whether PHP property enumeration sees a slot.
pub(super) fn emit_projection_visibility(
    out: &mut String,
    max_class_id: Option<u64>,
    classes_by_id: &HashMap<u64, &ClassInfo>,
    names_by_id: &HashMap<u64, &String>,
    classes: &HashMap<String, ClassInfo>,
) {
    out.push_str(".p2align 3\n.globl _class_property_projection_ptrs\n_class_property_projection_ptrs:\n");
    if let Some(max_id) = max_class_id {
        for id in 0..=max_id {
            if classes_by_id.contains_key(&id) {
                out.push_str(&format!("    .quad _class_property_projection_{id}\n"));
            } else {
                out.push_str("    .quad _class_property_projection_missing\n");
            }
        }
        for id in 0..=max_id {
            let Some(info) = classes_by_id.get(&id) else { continue; };
            let name = names_by_id[&id];
            out.push_str(&format!("_class_property_projection_{id}:\n"));
            let php_properties = info.properties.iter()
                .filter(|(property, _)| !property.starts_with('\0'));
            if php_properties.clone().next().is_none() {
                out.push_str("    .quad 0\n");
            }
            for (property, _) in php_properties {
                let declaring = info.property_declaring_classes.get(property)
                    .map(String::as_str).unwrap_or(name.as_str());
                let compiler_storage = is_internal_array_wrapper(declaring, classes);
                out.push_str(if compiler_storage { "    .quad 0\n" } else { "    .quad 1\n" });
            }
        }
    }
    out.push_str("_class_property_projection_missing:\n    .quad 0\n");
}

/// Emits nonvirtual builtin copy callbacks and authoritative base flags offsets for array casts.
pub(super) fn emit_array_cast_hooks(
    out: &mut String,
    max_class_id: Option<u64>,
    names_by_id: &HashMap<u64, &String>,
    classes: &HashMap<String, ClassInfo>,
) {
    let mut hooks = Vec::new();
    if let Some(max_id) = max_class_id {
        for id in 0..=max_id {
            let hook = names_by_id.get(&id).and_then(|name| array_cast_hook(name, classes));
            hooks.push(hook);
        }
    }
    out.push_str(".p2align 3\n.globl _class_spl_array_cast_ptrs\n_class_spl_array_cast_ptrs:\n");
    for (id, hook) in hooks.iter().enumerate() {
        out.push_str(&match hook {
            Some(_) => format!("    .quad _class_spl_array_cast_{id}\n"),
            None => "    .quad 0\n".into(),
        });
    }
    for (id, hook) in hooks.into_iter().enumerate() {
        if let Some((copy, flags_offset)) = hook {
            // Kind 1 calls a nonvirtual builtin copy operation and reads its scalar flags slot.
            out.push_str(&format!("_class_spl_array_cast_{id}:\n    .quad 1\n    .quad {flags_offset}\n    .quad {copy}\n"));
        }
    }
}

/// Resolves the closest internal provider, ignoring PHP user overrides of getArrayCopy.
fn array_cast_hook(name: &str, classes: &HashMap<String, ClassInfo>) -> Option<(String, usize)> {
    let mut current = Some(name);
    let mut provider = None;
    for _ in 0..classes.len() {
        let name = current?;
        let info = classes.get(name)?;
        if info.is_internal && provider.is_none() {
            if let Some(implementation) = info.method_impl_classes.get("getarraycopy") {
                if is_internal_array_wrapper(implementation, classes) {
                    provider = Some(crate::names::method_symbol(implementation, "getarraycopy"));
                }
            }
        }
        if info.is_internal && matches!(name, "ArrayObject" | "ArrayIterator") {
            let flag_name = if info.properties.iter().any(|(name, _)| name == "__elephc_flags") {
                "__elephc_flags"
            } else {
                "flags"
            };
            let slot = info.properties.iter().position(|(property, _)| property == flag_name)?;
            let flags = info.property_offsets.get(flag_name).copied().unwrap_or(8 + slot * 16);
            return Some((provider?, flags));
        }
        current = info.parent.as_deref();
    }
    None
}

/// Compiler-owned descendants can implement a live view while declaring no real PHP fields.
fn is_internal_array_wrapper(name: &str, classes: &HashMap<String, ClassInfo>) -> bool {
    if !classes.get(name).is_some_and(|info| info.is_internal) {
        return false;
    }
    let mut current = Some(name);
    for _ in 0..classes.len() {
        let Some(name) = current else { break; };
        let Some(info) = classes.get(name) else { break; };
        if info.is_internal && matches!(name, "ArrayObject" | "ArrayIterator") {
            return true;
        }
        current = info.parent.as_deref();
    }
    false
}
