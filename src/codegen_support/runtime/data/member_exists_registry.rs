//! Purpose:
//! Emits the closed-world class/member registry backing dynamic `method_exists()` and
//! `property_exists()` probes.
//!
//! Called from:
//! - `crate::codegen::emit_program()` when class-method introspection is required.
//!
//! Key details:
//! - Rows are 64 bytes: class name plus class-string method, object method, and property lists.
//! - Method lists are ASCII-lowercased because PHP method names are case-insensitive.
//! - Object method lists include inherited private methods; class-string lists hide them.

use std::collections::{BTreeSet, HashSet};

use crate::ir::Module;
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use crate::types::ClassInfo;

use super::instanceof::escaped_ascii;

/// One class row and its three PHP-observable member sets.
struct MemberRow {
    sort_key: String,
    class_methods: Vec<String>,
    object_methods: Vec<String>,
    properties: Vec<String>,
}

/// Emits `_member_exists_table` and its count symbol for the checked module.
pub(crate) fn emit_member_exists_registry_data(module: &Module) -> String {
    let rows = member_rows(module);
    let mut out = String::new();
    out.push_str(".data\n.p2align 3\n");
    emit_member_lists(&mut out, &rows);
    emit_member_table(&mut out, &rows);
    out
}

/// Builds deterministic registry rows for every non-synthetic declared class.
fn member_rows(module: &Module) -> Vec<MemberRow> {
    let method_candidates = all_method_candidates(module);
    let property_candidates = all_property_candidates(module);
    let mut rows = module
        .class_infos
        .iter()
        .filter(|(name, _)| !is_internal_synthetic_class_name(name))
        .map(|(class_name, class_info)| MemberRow {
            sort_key: php_symbol_key(class_name.trim_start_matches('\\')),
            class_methods: method_candidates
                .iter()
                .filter(|method| {
                    method_exists_for_class(module, class_name, class_info, method, false)
                })
                .cloned()
                .collect(),
            object_methods: method_candidates
                .iter()
                .filter(|method| {
                    method_exists_for_class(module, class_name, class_info, method, true)
                })
                .cloned()
                .collect(),
            properties: property_candidates
                .iter()
                .filter(|property| property_exists_for_class(class_name, class_info, property))
                .cloned()
                .collect(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.sort_key.as_bytes().cmp(right.sort_key.as_bytes()));
    rows
}

/// Collects every method key that can appear in a class's flattened metadata.
fn all_method_candidates(module: &Module) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for class_info in module.class_infos.values() {
        names.extend(class_info.methods.keys().map(|name| php_symbol_key(name)));
        names.extend(class_info.static_methods.keys().map(|name| php_symbol_key(name)));
    }
    names
}

/// Collects every case-sensitive instance/static property name in the module.
fn all_property_candidates(module: &Module) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for class_info in module.class_infos.values() {
        names.extend(class_info.property_visibilities.keys().cloned());
        names.extend(class_info.static_property_visibilities.keys().cloned());
    }
    names
}

/// Returns PHP's method-existence answer for one class, target form, and method key.
fn method_exists_for_class(
    module: &Module,
    class_name: &str,
    class_info: &ClassInfo,
    method_key: &str,
    target_is_object: bool,
) -> bool {
    if class_info.methods.contains_key(method_key)
        && (target_is_object
            || method_visible_from_class_string(
                class_name,
                method_key,
                &class_info.method_visibilities,
                &class_info.method_declaring_classes,
            ))
    {
        return true;
    }
    if class_info.static_methods.contains_key(method_key)
        && (target_is_object
            || method_visible_from_class_string(
                class_name,
                method_key,
                &class_info.static_method_visibilities,
                &class_info.static_method_declaring_classes,
            ))
    {
        return true;
    }
    target_is_object && parent_chain_declares_method(module, class_info, method_key)
}

/// Returns whether a method remains visible through a class-string target.
fn method_visible_from_class_string(
    class_name: &str,
    method_key: &str,
    visibilities: &std::collections::HashMap<String, Visibility>,
    declaring_classes: &std::collections::HashMap<String, String>,
) -> bool {
    visibilities.get(method_key) != Some(&Visibility::Private)
        || declaring_classes
            .get(method_key)
            .is_none_or(|declaring_class| php_symbol_key(declaring_class) == php_symbol_key(class_name))
}

/// Returns whether any ancestor declares a method, including private methods visible on objects.
fn parent_chain_declares_method(
    module: &Module,
    class_info: &ClassInfo,
    method_key: &str,
) -> bool {
    let mut visited = HashSet::new();
    let mut parent_name = class_info.parent.as_deref();
    while let Some(candidate) = parent_name {
        let key = php_symbol_key(candidate.trim_start_matches('\\'));
        if !visited.insert(key.clone()) {
            return false;
        }
        let Some(parent_info) = lookup_class_info(module, &key) else {
            return false;
        };
        if parent_info.methods.contains_key(method_key)
            || parent_info.static_methods.contains_key(method_key)
        {
            return true;
        }
        parent_name = parent_info.parent.as_deref();
    }
    false
}

/// Looks up class metadata using PHP's case-insensitive class-name rules.
fn lookup_class_info<'a>(module: &'a Module, class_key: &str) -> Option<&'a ClassInfo> {
    module
        .class_infos
        .iter()
        .find(|(candidate, _)| php_symbol_key(candidate.trim_start_matches('\\')) == class_key)
        .map(|(_, class_info)| class_info)
}

/// Returns PHP's property-existence answer for one flattened class schema.
fn property_exists_for_class(
    class_name: &str,
    class_info: &ClassInfo,
    property_name: &str,
) -> bool {
    property_visible_from_class_string(
        class_name,
        property_name,
        &class_info.property_visibilities,
        &class_info.property_declaring_classes,
    ) || property_visible_from_class_string(
        class_name,
        property_name,
        &class_info.static_property_visibilities,
        &class_info.static_property_declaring_classes,
    )
}

/// Returns whether a property exists without exposing an inherited private declaration.
fn property_visible_from_class_string(
    class_name: &str,
    property_name: &str,
    visibilities: &std::collections::HashMap<String, Visibility>,
    declaring_classes: &std::collections::HashMap<String, String>,
) -> bool {
    let Some(visibility) = visibilities.get(property_name) else {
        return false;
    };
    visibility != &Visibility::Private
        || declaring_classes
            .get(property_name)
            .is_none_or(|declaring_class| php_symbol_key(declaring_class) == php_symbol_key(class_name))
}

/// Emits all non-empty `{name_ptr, name_len}` member lists referenced by registry rows.
fn emit_member_lists(out: &mut String, rows: &[MemberRow]) {
    for (row_index, row) in rows.iter().enumerate() {
        emit_name_bytes(out, &format!("_memex_name_{}", row_index), &row.sort_key);
        emit_one_member_list(out, row_index, "cm", &row.class_methods);
        emit_one_member_list(out, row_index, "om", &row.object_methods);
        emit_one_member_list(out, row_index, "pr", &row.properties);
    }
}

/// Emits one row-owned member list, omitting storage for an empty list.
fn emit_one_member_list(out: &mut String, row_index: usize, kind: &str, names: &[String]) {
    for (slot, name) in names.iter().enumerate() {
        emit_name_bytes(
            out,
            &format!("_memex_{}_{}_{}", kind, row_index, slot),
            name,
        );
    }
    if names.is_empty() {
        return;
    }
    out.push_str(".p2align 3\n");
    out.push_str(&format!("_memex_{}_list_{}:\n", kind, row_index));
    for (slot, name) in names.iter().enumerate() {
        out.push_str(&format!(
            "    .quad _memex_{}_{}_{}\n    .quad {}\n",
            kind,
            row_index,
            slot,
            name.len()
        ));
    }
}

/// Emits the fixed-width registry table and its row-count symbol.
fn emit_member_table(out: &mut String, rows: &[MemberRow]) {
    out.push_str(".p2align 3\n.globl _member_exists_table\n_member_exists_table:\n");
    for (index, row) in rows.iter().enumerate() {
        out.push_str(&format!(
            "    .quad _memex_name_{}\n    .quad {}\n",
            index,
            row.sort_key.len()
        ));
        emit_list_ref(out, index, "cm", row.class_methods.len());
        emit_list_ref(out, index, "om", row.object_methods.len());
        emit_list_ref(out, index, "pr", row.properties.len());
    }
    out.push_str(&format!(
        ".p2align 3\n.globl _member_exists_table_count\n_member_exists_table_count:\n    .quad {}\n",
        rows.len()
    ));
}

/// Emits one list pointer/count pair inside a registry row.
fn emit_list_ref(out: &mut String, row_index: usize, kind: &str, count: usize) {
    if count == 0 {
        out.push_str("    .quad 0\n    .quad 0\n");
    } else {
        out.push_str(&format!(
            "    .quad _memex_{}_list_{}\n    .quad {}\n",
            kind, row_index, count
        ));
    }
}

/// Emits one byte-string label consumed with an explicit length word.
fn emit_name_bytes(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("{}:\n", label));
    out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii(value)));
}

/// Returns true for compiler-internal classes excluded from PHP introspection.
fn is_internal_synthetic_class_name(name: &str) -> bool {
    php_symbol_key(name).starts_with("__elephc")
}

