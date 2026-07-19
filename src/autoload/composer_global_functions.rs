//! Purpose:
//! Pre-scans Composer `autoload.files` entries for globally declared (non-namespaced) free
//! function names, including ones nested inside `if (!function_exists('X')) { function X() {}
//! }`-style guards, before any name resolution runs.
//!
//! Called from:
//! - `crate::pipeline::compile()`, which installs the result via
//!   `name_resolver::with_known_composer_global_functions` for the span covering the main
//!   name-resolution pass and `autoload::run`.
//!
//! Key details:
//! - Deliberately scoped to `autoload.files` only, not the full PSR-4-referenced class tree.
//!   `Registry::always_included_files()` is known upfront (before any resolution begins), so this
//!   scan can run before it; a PSR-4-referenced class file's own global-function declarations are
//!   only discovered as the reference graph is walked, too late for a pre-pass to see, so calls to
//!   those stay out of scope (the fully general cross-file case is a separate, harder problem).
//! - Uses lex+parse only (no magic-constants/conditional/resolver/name-resolver): only free-function
//!   NAMES are needed, not their bodies. A read or parse failure here is silently skipped — such a
//!   file is either genuinely absent (an edge case in test fixtures) or will surface its own error
//!   later through `autoload::run`'s own (tolerant or strict) loading, so this pre-scan must not be
//!   a second place that can hard-fail compilation.
//! - Recurses into `if`/`while`/`do-while`/`for`/`foreach`/`switch`/`try`/synthetic/include-once-guard
//!   bodies (the same shapes `name_resolver::symbols::collect_symbols` now recurses through) but
//!   never into function/closure/method bodies, matching PHP scoping: a function declared inside
//!   another function is a runtime-local declaration, not a free global one.

use std::collections::HashMap;

use crate::names::{php_symbol_key, Name};
use crate::parser::ast::{Stmt, StmtKind};

use super::Registry;

/// Scans every `autoload.files` entry in `registry` and returns every globally-declared
/// (non-namespaced) free-function name found, keyed by `php_symbol_key` with the original
/// declared-case name as the value — mirrors `name_resolver::symbols`'s folded-symbol convention
/// so callers can format a resolved name the same way.
pub fn scan_composer_global_functions(registry: &Registry) -> HashMap<String, String> {
    let mut names = HashMap::new();
    for path in registry.always_included_files() {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(tokens) = crate::lexer::tokenize(&content) else {
            continue;
        };
        let Ok(program) = crate::parser::parse(&tokens) else {
            continue;
        };
        collect_global_function_decls(&program, None, &mut names);
    }
    names
}

/// Recursively collects top-level (namespace-empty) `FunctionDecl` names from `stmts` into `out`,
/// tracking the innermost enclosing namespace so a declaration inside `namespace X { ... }` is
/// correctly excluded (only the GLOBAL fallback target matters here; a namespaced declaration is
/// already visible to `name_resolver::symbols::collect_symbols`'s own namespace-local lookup).
fn collect_global_function_decls(
    stmts: &[Stmt],
    current_namespace: Option<&str>,
    out: &mut HashMap<String, String>,
) {
    let mut namespace = current_namespace.map(str::to_string);
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::NamespaceDecl { name } => {
                namespace = Some(namespace_name(name));
            }
            StmtKind::NamespaceBlock { name, body } => {
                collect_global_function_decls(body, Some(&namespace_name(name)), out);
            }
            StmtKind::FunctionDecl { name, .. } => {
                if namespace.as_deref().is_none_or(str::is_empty) {
                    out.entry(php_symbol_key(name)).or_insert_with(|| name.clone());
                }
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_global_function_decls(then_body, namespace.as_deref(), out);
                for (_, body) in elseif_clauses {
                    collect_global_function_decls(body, namespace.as_deref(), out);
                }
                if let Some(body) = else_body {
                    collect_global_function_decls(body, namespace.as_deref(), out);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::For { body, .. }
            | StmtKind::Foreach { body, .. } => {
                collect_global_function_decls(body, namespace.as_deref(), out);
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_global_function_decls(body, namespace.as_deref(), out);
                }
                if let Some(body) = default {
                    collect_global_function_decls(body, namespace.as_deref(), out);
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_global_function_decls(try_body, namespace.as_deref(), out);
                for catch in catches {
                    collect_global_function_decls(&catch.body, namespace.as_deref(), out);
                }
                if let Some(body) = finally_body {
                    collect_global_function_decls(body, namespace.as_deref(), out);
                }
            }
            StmtKind::Synthetic(body) | StmtKind::IncludeOnceGuard { body, .. } => {
                collect_global_function_decls(body, namespace.as_deref(), out);
            }
            _ => {}
        }
    }
}

/// Extracts the namespace name as a canonical string from an optional `Name`, matching
/// `name_resolver`'s own `namespace_name` helper (kept as a small local copy since that one is
/// private to `name_resolver`).
fn namespace_name(name: &Option<Name>) -> String {
    name.as_ref().map(Name::as_canonical).unwrap_or_default()
}
