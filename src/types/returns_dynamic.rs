//! Purpose:
//! Detects untyped functions and methods whose `return` can hand back one of their dynamic
//! (untyped, by-value) parameters unchanged.
//!
//! Called from:
//! - `crate::types::checker::functions::resolution::signature` (function return inference).
//! - `crate::ir_lower::program` (method EIR signature normalization).
//!
//! Key details:
//! - Untyped by-value parameters are boxed `mixed` at the EIR boundary, so a body returning
//!   one must return `mixed`; inferring `int` from the first call site would truncate a float
//!   that a later (e.g. recursive) call passes in.

use std::collections::HashSet;

use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};

/// Recursively scans a method body for returns that expose dynamic parameters.
pub(crate) fn body_returns_dynamic_param(body: &[Stmt], dynamic_params: &HashSet<String>) -> bool {
    body.iter()
        .any(|stmt| stmt_returns_dynamic_param(stmt, dynamic_params))
}

/// Returns true when one statement can return a dynamic parameter directly.
fn stmt_returns_dynamic_param(stmt: &Stmt, dynamic_params: &HashSet<String>) -> bool {
    match &stmt.kind {
        StmtKind::Return(Some(expr)) => expr_exposes_dynamic_param(expr, dynamic_params),
        StmtKind::Return(None) => false,
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            body_returns_dynamic_param(then_body, dynamic_params)
                || elseif_clauses
                    .iter()
                    .any(|(_, body)| body_returns_dynamic_param(body, dynamic_params))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_returns_dynamic_param(body, dynamic_params))
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            body_returns_dynamic_param(then_body, dynamic_params)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_returns_dynamic_param(body, dynamic_params))
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => body_returns_dynamic_param(body, dynamic_params),
        StmtKind::For {
            init, update, body, ..
        } => {
            init.as_ref()
                .is_some_and(|stmt| stmt_returns_dynamic_param(stmt.as_ref(), dynamic_params))
                || update
                    .as_ref()
                    .is_some_and(|stmt| stmt_returns_dynamic_param(stmt.as_ref(), dynamic_params))
                || body_returns_dynamic_param(body, dynamic_params)
        }
        StmtKind::Switch { cases, default, .. } => {
            cases
                .iter()
                .any(|(_, body)| body_returns_dynamic_param(body, dynamic_params))
                || default
                    .as_ref()
                    .is_some_and(|body| body_returns_dynamic_param(body, dynamic_params))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            body_returns_dynamic_param(try_body, dynamic_params)
                || catches
                    .iter()
                    .any(|catch| body_returns_dynamic_param(&catch.body, dynamic_params))
                || finally_body
                    .as_ref()
                    .is_some_and(|body| body_returns_dynamic_param(body, dynamic_params))
        }
        _ => false,
    }
}

/// Returns true when an expression can yield one of the dynamic parameters directly.
fn expr_exposes_dynamic_param(expr: &Expr, dynamic_params: &HashSet<String>) -> bool {
    match &expr.kind {
        ExprKind::Variable(name) => dynamic_params.contains(name),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            expr_exposes_dynamic_param(value, dynamic_params)
                || expr_exposes_dynamic_param(default, dynamic_params)
        }
        ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } => {
            expr_exposes_dynamic_param(then_expr, dynamic_params)
                || expr_exposes_dynamic_param(else_expr, dynamic_params)
        }
        ExprKind::Match { arms, default, .. } => {
            arms.iter()
                .any(|(_, arm)| expr_exposes_dynamic_param(arm, dynamic_params))
                || default
                    .as_ref()
                    .is_some_and(|expr| expr_exposes_dynamic_param(expr, dynamic_params))
        }
        ExprKind::ErrorSuppress(inner) => expr_exposes_dynamic_param(inner, dynamic_params),
        ExprKind::Assignment { value, .. } => {
            expr_exposes_dynamic_param(value, dynamic_params)
        }
        _ => false,
    }
}

