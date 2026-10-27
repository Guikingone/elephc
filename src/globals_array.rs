//! Purpose:
//! Resolves `$GLOBALS['name']` with a literal key to the global variable itself, so reads,
//! writes and method calls through it compile like `global $name;` does.
//!
//! Called from:
//! - `crate::undefined_reads::rewrite()`, first, so the undefined-read passes see plain
//!   variables instead of a `$GLOBALS` mention that would make the scope dynamic.
//!
//! Key details:
//! - In the main scope the global *is* the local, so `$GLOBALS['x']` becomes `$x`.
//! - In a function, method or closure it becomes `$x` plus a `global $x;` binding at the top of
//!   the body, unless the scope already uses `$x` for something else (a parameter or a local):
//!   then the access is left alone, since one name cannot be both.
//! - Only string-literal keys are resolved; `$GLOBALS[$k]`, `unset($GLOBALS['x'])` and
//!   `$GLOBALS` as a whole keep their existing handling.

use std::collections::{BTreeSet, HashSet};

use crate::parser::ast::{ClassMethod, Expr, ExprKind, Stmt, StmtKind};
use crate::undefined_constants::walk_scope_exprs_mut;

/// Rewrites literal-key `$GLOBALS` accesses in every scope of the program.
pub fn rewrite(program: &mut Vec<Stmt>) {
    rewrite_nested_scopes(program);
    rewrite_scope(program, &HashSet::new(), true);
}

/// Rewrites the function, method and closure scopes nested in `stmts`.
fn rewrite_nested_scopes(stmts: &mut [Stmt]) {
    for stmt in stmts.iter_mut() {
        match &mut stmt.kind {
            StmtKind::FunctionDecl { params, variadic, body, .. } => {
                let names = params.iter().map(|(name, ..)| name.clone()).chain(variadic.clone()).collect();
                rewrite_function_body(body, &names);
            }
            StmtKind::ClassDecl { methods, .. }
            | StmtKind::TraitDecl { methods, .. }
            | StmtKind::InterfaceDecl { methods, .. }
            | StmtKind::EnumDecl { methods, .. } => rewrite_methods(methods),
            _ => crate::undefined_reads::for_each_child_body_mut_pub(stmt, &mut |body| {
                rewrite_nested_scopes(body)
            }),
        }
    }
    walk_scope_exprs_mut(stmts, &mut |expr| {
        if let ExprKind::Closure { params, variadic, body, is_arrow: false, captures, capture_refs, .. } =
            &mut expr.kind
        {
            let names = params
                .iter()
                .map(|(name, ..)| name.clone())
                .chain(variadic.clone())
                .chain(captures.iter().cloned())
                .chain(capture_refs.iter().cloned())
                .collect();
            rewrite_function_body(body, &names);
        }
    });
}

/// Rewrites every method body as its own scope.
fn rewrite_methods(methods: &mut [ClassMethod]) {
    for method in methods.iter_mut() {
        let names = method
            .params
            .iter()
            .map(|(name, ..)| name.clone())
            .chain(method.variadic.clone())
            .collect();
        rewrite_function_body(&mut method.body, &names);
    }
}

/// Rewrites one function-like body, then binds the globals it now names.
fn rewrite_function_body(body: &mut Vec<Stmt>, params: &HashSet<String>) {
    rewrite_nested_scopes(body);
    let mut used = params.clone();
    collect_scope_variables(body, &mut used);
    let bound = rewrite_scope(body, &used, false);
    if !bound.is_empty() {
        let span = body.first().map(|stmt| stmt.span).unwrap_or_else(crate::span::Span::dummy);
        body.insert(0, Stmt::new(StmtKind::Global { vars: bound.into_iter().collect() }, span));
    }
}

/// Replaces the literal-key accesses of one scope whose name is free, returning the names used.
fn rewrite_scope(stmts: &mut Vec<Stmt>, taken: &HashSet<String>, main: bool) -> BTreeSet<String> {
    let mut bound = BTreeSet::new();
    let free = |name: &str| main || !taken.contains(name);
    rewrite_assign_statements(stmts, &free, &mut bound);
    walk_scope_exprs_mut(stmts, &mut |expr| {
        if let Some(name) = literal_global(expr).filter(|name| free(name)) {
            bound.insert(name.clone());
            expr.kind = ExprKind::Variable(name);
        }
    });
    bound
}

/// Turns statement-level `$GLOBALS['x'] = v;` into `$x = v;` throughout one scope.
fn rewrite_assign_statements(stmts: &mut [Stmt], free: &dyn Fn(&str) -> bool, bound: &mut BTreeSet<String>) {
    for stmt in stmts.iter_mut() {
        if let StmtKind::ArrayAssign { array, index, value } = &mut stmt.kind {
            if array == "GLOBALS" {
                if let ExprKind::StringLiteral(name) = &index.kind {
                    if free(name) {
                        let name = name.clone();
                        let value = std::mem::replace(value, Expr::new(ExprKind::Null, stmt.span));
                        bound.insert(name.clone());
                        stmt.kind = StmtKind::Assign { name, value };
                        continue;
                    }
                }
            }
        }
        if !matches!(
            stmt.kind,
            StmtKind::FunctionDecl { .. }
                | StmtKind::ClassDecl { .. }
                | StmtKind::TraitDecl { .. }
                | StmtKind::InterfaceDecl { .. }
                | StmtKind::EnumDecl { .. }
        ) {
            crate::undefined_reads::for_each_child_body_mut_pub(stmt, &mut |body| {
                rewrite_assign_statements(body, free, bound)
            });
        }
    }
}

/// Returns the key of `$GLOBALS['name']`.
fn literal_global(expr: &Expr) -> Option<String> {
    let ExprKind::ArrayAccess { array, index } = &expr.kind else {
        return None;
    };
    match (&array.kind, &index.kind) {
        (ExprKind::Variable(base), ExprKind::StringLiteral(name)) if base == "GLOBALS" && !name.is_empty() => {
            Some(name.clone())
        }
        _ => None,
    }
}

/// Collects every variable name one scope mentions (reads, writes, captures).
fn collect_scope_variables(stmts: &mut [Stmt], names: &mut HashSet<String>) {
    crate::undefined_reads::collect_scope_written_names(stmts, names);
    walk_scope_exprs_mut(stmts, &mut |expr| match &expr.kind {
        ExprKind::Variable(name) if name != "GLOBALS" => {
            names.insert(name.clone());
        }
        ExprKind::Closure { captures, capture_refs, .. } => {
            names.extend(captures.iter().cloned());
            names.extend(capture_refs.iter().cloned());
        }
        _ => {}
    });
}
