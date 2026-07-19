//! Purpose:
//! Top-level `func_num_args()`/`func_get_args()`/`func_get_arg()` discovery for the type
//! checker. Walks a function/method body looking for a call to any of the three
//! arity-introspection builtins at the body's OWN scope, returning `true` as soon as one is
//! found. Closures form a fresh arity scope and are skipped, mirroring `yield_validation`.
//!
//! Called from:
//! - `crate::types::checker::func_args_scan::mark_func_args_functions` — decides which
//!   user functions/methods need the hidden arity-count/variadic-tail ABI extension.
//! - `crate::types::checker::func_args_scan::validate` — reused to flag illegal uses
//!   (global scope, closures) once the "own scope" callers are known.
//!
//! Key details:
//! - The walker visits every statement and expression shape that can contain a call in
//!   v1's grammar. Anything inside a `Closure` expression belongs to a different arity
//!   scope, so we deliberately do not peek through it (same rule as generator detection).

use crate::names::php_symbol_key;
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};

/// Returns `true` if `name` is the case-insensitive, optionally-backslash-qualified name of
/// `func_num_args`, `func_get_args`, or `func_get_arg`.
pub(crate) fn is_func_args_intrinsic_name(name: &str) -> bool {
    matches!(
        php_symbol_key(name.trim_start_matches('\\')).as_str(),
        "func_num_args" | "func_get_args" | "func_get_arg"
    )
}

/// Scans the top-level statements of a function/method body for a call to
/// `func_num_args()`, `func_get_args()`, or `func_get_arg()` at the body's own scope.
/// Returns `true` on the first match found. Closures are skipped entirely — a call
/// inside a closure belongs to that closure's own arity scope, not the enclosing one.
pub(crate) fn body_calls_func_args_intrinsic(body: &[Stmt]) -> bool {
    body.iter().any(stmt_calls_func_args_intrinsic)
}

/// Recursively checks each statement variant for a `func_num_args`/`func_get_args`/
/// `func_get_arg` call at the same scope. Skips nested `FunctionDecl`, `ClassDecl`,
/// `TraitDecl`, and `InterfaceDecl` boundaries — those introduce their own scope.
fn stmt_calls_func_args_intrinsic(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::FunctionDecl { .. } | StmtKind::ClassDecl { .. } | StmtKind::TraitDecl { .. } => false,
        StmtKind::InterfaceDecl { .. } => false,
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().any(stmt_calls_func_args_intrinsic)
                || catches.iter().any(|c| c.body.iter().any(stmt_calls_func_args_intrinsic))
                || finally_body
                    .as_ref()
                    .map(|f| f.iter().any(stmt_calls_func_args_intrinsic))
                    .unwrap_or(false)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_calls_func_args_intrinsic(condition)
                || then_body.iter().any(stmt_calls_func_args_intrinsic)
                || elseif_clauses
                    .iter()
                    .any(|(c, b)| expr_calls_func_args_intrinsic(c) || b.iter().any(stmt_calls_func_args_intrinsic))
                || else_body
                    .as_ref()
                    .map(|b| b.iter().any(stmt_calls_func_args_intrinsic))
                    .unwrap_or(false)
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            then_body.iter().any(stmt_calls_func_args_intrinsic)
                || else_body
                    .as_ref()
                    .map(|b| b.iter().any(stmt_calls_func_args_intrinsic))
                    .unwrap_or(false)
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_calls_func_args_intrinsic(condition) || body.iter().any(stmt_calls_func_args_intrinsic)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref().map(stmt_calls_func_args_intrinsic).unwrap_or(false)
                || condition.as_ref().map(expr_calls_func_args_intrinsic).unwrap_or(false)
                || update.as_deref().map(stmt_calls_func_args_intrinsic).unwrap_or(false)
                || body.iter().any(stmt_calls_func_args_intrinsic)
        }
        StmtKind::Foreach { array, body, .. } => {
            expr_calls_func_args_intrinsic(array) || body.iter().any(stmt_calls_func_args_intrinsic)
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_calls_func_args_intrinsic(subject)
                || cases.iter().any(|(vals, body)| {
                    vals.iter().any(expr_calls_func_args_intrinsic) || body.iter().any(stmt_calls_func_args_intrinsic)
                })
                || default
                    .as_ref()
                    .map(|d| d.iter().any(stmt_calls_func_args_intrinsic))
                    .unwrap_or(false)
        }
        StmtKind::Synthetic(stmts) | StmtKind::NamespaceBlock { body: stmts, .. } => {
            stmts.iter().any(stmt_calls_func_args_intrinsic)
        }
        StmtKind::Echo(e) | StmtKind::ExprStmt(e) | StmtKind::Throw(e) => expr_calls_func_args_intrinsic(e),
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticVar { init: value, .. } => expr_calls_func_args_intrinsic(value),
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_calls_func_args_intrinsic(index) || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_calls_func_args_intrinsic(target) || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::ArrayPush { value, .. } => expr_calls_func_args_intrinsic(value),
        StmtKind::Return(opt) => opt.as_ref().map(expr_calls_func_args_intrinsic).unwrap_or(false),
        StmtKind::Include { path, .. } => expr_calls_func_args_intrinsic(path),
        StmtKind::PropertyAssign { object, value, .. } => {
            expr_calls_func_args_intrinsic(object) || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_calls_func_args_intrinsic(object) || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::PropertyArrayAssign { object, index, value, .. } => {
            expr_calls_func_args_intrinsic(object)
                || expr_calls_func_args_intrinsic(index)
                || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_calls_func_args_intrinsic(value),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_calls_func_args_intrinsic(index) || expr_calls_func_args_intrinsic(value)
        }
        StmtKind::DynamicStaticPropertyWrite { property, index, value, .. } => {
            expr_calls_func_args_intrinsic(property)
                || index.as_ref().is_some_and(expr_calls_func_args_intrinsic)
                || expr_calls_func_args_intrinsic(value)
        }
        _ => false,
    }
}

/// Recursively checks each expression variant for a `func_num_args`/`func_get_args`/
/// `func_get_arg` call. Skips `Closure` expressions — a call inside a closure belongs to
/// that closure's own arity scope. Handles the same expression shapes as
/// `yield_validation::detect::expr_contains_yield`.
pub(crate) fn expr_calls_func_args_intrinsic(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::FunctionCall { name, args } => {
            is_func_args_intrinsic_name(name.as_str()) || args.iter().any(expr_calls_func_args_intrinsic)
        }
        // Don't peek into closures — a call inside one belongs to a different arity scope.
        ExprKind::Closure { .. } => false,
        ExprKind::BinaryOp { left, right, .. } => {
            expr_calls_func_args_intrinsic(left) || expr_calls_func_args_intrinsic(right)
        }
        ExprKind::InstanceOf { value, .. } => expr_calls_func_args_intrinsic(value),
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Spread(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. } => expr_calls_func_args_intrinsic(inner),
        ExprKind::NullCoalesce { value, default } => {
            expr_calls_func_args_intrinsic(value) || expr_calls_func_args_intrinsic(default)
        }
        ExprKind::Pipe { value, callable } => {
            expr_calls_func_args_intrinsic(value) || expr_calls_func_args_intrinsic(callable)
        }
        ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::StaticMethodCall { args, .. } => args.iter().any(expr_calls_func_args_intrinsic),
        ExprKind::ExprCall { callee, args } => {
            expr_calls_func_args_intrinsic(callee) || args.iter().any(expr_calls_func_args_intrinsic)
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_calls_func_args_intrinsic(object) || args.iter().any(expr_calls_func_args_intrinsic)
        }
        ExprKind::ArrayLiteral(items) => items.iter().any(expr_calls_func_args_intrinsic),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs
            .iter()
            .any(|(k, v)| expr_calls_func_args_intrinsic(k) || expr_calls_func_args_intrinsic(v)),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_calls_func_args_intrinsic(subject)
                || arms.iter().any(|(patterns, value)| {
                    patterns.iter().any(expr_calls_func_args_intrinsic) || expr_calls_func_args_intrinsic(value)
                })
                || default.as_ref().map(|d| expr_calls_func_args_intrinsic(d)).unwrap_or(false)
        }
        ExprKind::ArrayAccess { array, index } => {
            expr_calls_func_args_intrinsic(array) || expr_calls_func_args_intrinsic(index)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_calls_func_args_intrinsic(condition)
                || expr_calls_func_args_intrinsic(then_expr)
                || expr_calls_func_args_intrinsic(else_expr)
        }
        ExprKind::ShortTernary { value, default } => {
            expr_calls_func_args_intrinsic(value) || expr_calls_func_args_intrinsic(default)
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_calls_func_args_intrinsic(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_calls_func_args_intrinsic(object) || expr_calls_func_args_intrinsic(property)
        }
        ExprKind::NamedArg { value, .. } => expr_calls_func_args_intrinsic(value),
        ExprKind::BufferNew { len, .. } => expr_calls_func_args_intrinsic(len),
        _ => false,
    }
}
