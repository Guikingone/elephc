//! Purpose:
//! Computes and emits the closed-world `_class_methods_table` payload registry backing
//! `get_class_methods()` when its target is only known at runtime (a non-literal class-name
//! string, or a `Mixed`/union value).
//!
//! Called from:
//! - `crate::codegen::emit_program()` when the module's `class_methods_introspection`
//!   runtime feature is set.
//!
//! Key details:
//! - Rows are `{name_ptr, name_len, list_ptr, list_count}` — **32 bytes**, NOT the 64-byte
//!   layout the three relation tables share. That is deliberate: `__rt_sorted_name_search`
//!   takes `entry_size` as an argument, so a narrower private table costs nothing at the
//!   search site while leaving `_class_relation_table` and its fixed-size
//!   `__rt_class_relation_probe` (entry_size = 64) completely untouched.
//! - Each row carries the PUBLIC-only method names, in the same PHP declaration order the
//!   compile-time literal fold produces (`crate::codegen::lower_inst::builtins::
//!   get_class_methods::get_class_methods_names` with `self_scope = false`), so a runtime
//!   lookup and a literal call agree on content AND order for the same class. The self-scope
//!   list is NOT stored: the calling scope is always compile-time known, so the lowering bakes
//!   that list inline and only consults this table for the non-self case.
//! - Rows are sorted by lowercased name for `__rt_sorted_name_search`'s binary search, matching
//!   `_class_table`/`_class_relation_table`.
//! - Only CLASSES get rows. `get_class_methods()` on an interface name returns its method
//!   declarations in PHP, but elephc's `module.class_infos` is the only source with per-method
//!   visibility and declaration order, so an interface/trait name simply misses the table and
//!   the lowering raises PHP's `TypeError` — the same answer it gives for an undeclared name.

use crate::ir::Module;
use crate::names::php_symbol_key;
use crate::parser::ast::Visibility;
use std::collections::HashSet;

use super::instanceof::escaped_ascii;

/// One class's precomputed public method-name list (PHP declaration order).
struct MethodsRow {
    /// Lowercased ASCII sort key used for the binary search over the table.
    sort_key: String,
    methods: Vec<String>,
}

/// Emits the `_class_methods_table` payload registry computed from `module`.
pub(crate) fn emit_class_methods_registry_data(module: &Module) -> String {
    let rows = class_methods_rows(module);
    let mut out = String::new();
    out.push_str(".data\n");
    out.push_str(".p2align 3\n");
    emit_methods_table(&mut out, "_class_methods_table", "_clsmeth", &rows);
    out
}

/// Builds the name-sorted rows, one per non-synthetic class.
fn class_methods_rows(module: &Module) -> Vec<MethodsRow> {
    let mut rows: Vec<MethodsRow> = module
        .class_infos
        .keys()
        .filter(|name| !is_internal_synthetic_class_name(name))
        .map(|name| MethodsRow {
            sort_key: php_symbol_key(name.trim_start_matches('\\')),
            methods: public_method_names(module, name),
        })
        .collect();
    rows.sort_by(|a, b| a.sort_key.as_bytes().cmp(b.sort_key.as_bytes()));
    rows
}

/// Walks `class_name`'s ancestor chain and returns its PUBLIC method names in PHP declaration
/// order: the class's own declared methods first (source order), then each ancestor's, nearest
/// first, skipping any name already claimed by a more-derived level.
///
/// This is a deliberate parallel implementation of
/// `crate::codegen::lower_inst::builtins::get_class_methods::get_class_methods_names`'s
/// `self_scope = false` branch, over `&Module` instead of a `FunctionContext` — exactly the
/// arrangement `class_relation_registry` uses for the relation lists, and for the same reason:
/// it keeps the literal EIR lowering path untouched. The shared `test_dynamic_and_literal_
/// get_class_methods_agree` codegen test is what holds the two in step.
fn public_method_names(module: &Module, class_name: &str) -> Vec<String> {
    let mut claimed: HashSet<String> = HashSet::new();
    let mut names = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut current = Some(class_name.to_string());
    while let Some(level_name) = current {
        if !visited.insert(level_name.clone()) {
            break; // cycle guard against malformed metadata
        }
        let Some(info) = module.class_infos.get(&level_name) else {
            break;
        };
        for decl in &info.method_decls {
            let key = php_symbol_key(&decl.name);
            if !claimed.insert(key) {
                continue; // already claimed by a more-derived level
            }
            if decl.visibility == Visibility::Public {
                names.push(decl.name.clone());
            }
        }
        current = info.parent.clone();
    }
    names
}

/// Emits one 32-byte-per-row table plus its `<table>_count` symbol, with each non-empty method
/// list materialized as a separate flat `{name_ptr, name_len}` array.
fn emit_methods_table(out: &mut String, table: &str, prefix: &str, rows: &[MethodsRow]) {
    for (index, row) in rows.iter().enumerate() {
        emit_name_bytes(out, &format!("{}_name_{}", prefix, index), &row.sort_key);
        for (slot, name) in row.methods.iter().enumerate() {
            emit_name_bytes(out, &format!("{}_m_{}_{}", prefix, index, slot), name);
        }
        if row.methods.is_empty() {
            continue;
        }
        out.push_str(".p2align 3\n");
        out.push_str(&format!("{}_list_{}:\n", prefix, index));
        for (slot, name) in row.methods.iter().enumerate() {
            out.push_str(&format!(
                "    .quad {}_m_{}_{}\n    .quad {}\n",
                prefix,
                index,
                slot,
                name.len()
            ));
        }
    }
    out.push_str(".p2align 3\n");
    out.push_str(&format!("{}:\n", table));
    for (index, row) in rows.iter().enumerate() {
        out.push_str(&format!(
            "    .quad {}_name_{}\n    .quad {}\n",
            prefix,
            index,
            row.sort_key.len()
        ));
        if row.methods.is_empty() {
            out.push_str("    .quad 0\n    .quad 0\n");
        } else {
            out.push_str(&format!(
                "    .quad {}_list_{}\n    .quad {}\n",
                prefix,
                index,
                row.methods.len()
            ));
        }
    }
    out.push_str(".p2align 3\n");
    out.push_str(&format!("{}_count:\n    .quad {}\n", table, rows.len()));
}

/// Mirrors `crate::codegen::is_internal_synthetic_class_name` (and
/// `class_relation_registry`'s private copy of it) so this registry's row set agrees with
/// `_class_table`'s.
fn is_internal_synthetic_class_name(name: &str) -> bool {
    php_symbol_key(name).starts_with("__elephc")
}

/// Emits a label holding `value`'s raw bytes (no NUL terminator — every consumer reads the
/// paired explicit length).
fn emit_name_bytes(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("{}:\n", label));
    out.push_str(&format!("    .ascii \"{}\"\n", escaped_ascii(value)));
}
