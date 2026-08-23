//! Purpose:
//! Collects the PHP variable names that any function-like body in a program declares with
//! `global`, so the checker and EIR lowering answer "does program storage back this name?" from
//! ONE walk instead of two that can drift.
//!
//! Called from:
//! - `crate::types::checker::driver::check_types_impl` (once per check, before the first walk)
//! - `crate::ir_lower::function::lower_main` and the per-body lowering entry points
//!
//! Key details:
//! - `global $x;` inside a function/method/closure binds `$x` to the program-global cell the TOP
//!   LEVEL also writes through its own local slot. The checker must therefore not end a top-level
//!   binding of such a name (`unset` would leave the name unbound while another body still reaches
//!   the storage by name), and lowering must not abandon its slot. Both sides read this set.
//! - The walk is deliberately shared rather than duplicated: its known blind spots — a `global`
//!   written inside a CLOSURE body, an `enum` method, or a `packed class` — then stay identical on
//!   both sides, so the checker never approves a decision lowering refuses (or vice versa). A
//!   `global` in one of those places is under-collected on both sides at once; widening the walk
//!   is a behaviour change for lowering as much as for the checker and is left to its own task.

use std::collections::HashSet;

use crate::parser::ast::{Stmt, StmtKind};

/// Collects the PHP variable names that any function-like body in `statements` declares `global`.
pub(crate) fn collect_global_var_names(statements: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_in_body(statements, &mut names);
    names
}

/// Recursively scans statement bodies for `global` declarations.
fn collect_in_body(statements: &[Stmt], names: &mut HashSet<String>) {
    for stmt in statements {
        match &stmt.kind {
            StmtKind::Global { vars } => {
                names.extend(vars.iter().cloned());
            }
            StmtKind::If {
                then_body,
                elseif_clauses,
                else_body,
                ..
            } => {
                collect_in_body(then_body, names);
                for (_, body) in elseif_clauses {
                    collect_in_body(body, names);
                }
                if let Some(body) = else_body {
                    collect_in_body(body, names);
                }
            }
            StmtKind::IfDef {
                then_body,
                else_body,
                ..
            } => {
                collect_in_body(then_body, names);
                if let Some(body) = else_body {
                    collect_in_body(body, names);
                }
            }
            StmtKind::While { body, .. }
            | StmtKind::DoWhile { body, .. }
            | StmtKind::Foreach { body, .. }
            | StmtKind::FunctionDecl { body, .. }
            | StmtKind::NamespaceBlock { body, .. }
            | StmtKind::IncludeOnceGuard { body, .. }
            | StmtKind::Synthetic(body) => {
                collect_in_body(body, names);
            }
            StmtKind::For {
                init, update, body, ..
            } => {
                if let Some(init) = init {
                    collect_in_body(std::slice::from_ref(init.as_ref()), names);
                }
                if let Some(update) = update {
                    collect_in_body(std::slice::from_ref(update.as_ref()), names);
                }
                collect_in_body(body, names);
            }
            StmtKind::Switch { cases, default, .. } => {
                for (_, body) in cases {
                    collect_in_body(body, names);
                }
                if let Some(body) = default {
                    collect_in_body(body, names);
                }
            }
            StmtKind::Try {
                try_body,
                catches,
                finally_body,
            } => {
                collect_in_body(try_body, names);
                for catch in catches {
                    collect_in_body(&catch.body, names);
                }
                if let Some(body) = finally_body {
                    collect_in_body(body, names);
                }
            }
            StmtKind::ClassDecl { methods, .. }
            | StmtKind::InterfaceDecl { methods, .. }
            | StmtKind::TraitDecl { methods, .. } => {
                for method in methods {
                    collect_in_body(&method.body, names);
                }
            }
            _ => {}
        }
    }
}
