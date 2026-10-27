//! Purpose:
//! Turns reads of constants the checker proved undefined into php's run-time `Error`, and sends
//! method calls the receiver's declared type cannot answer to run-time dispatch.
//!
//! Called from:
//! - `crate::generics::monomorphize` once the checker has settled, so the CLI pipeline and every
//!   harness that checks through it see the same program.
//!
//! Key details:
//! - php compiles `y;` and throws `Error: Undefined constant "y"` only when the read executes, so
//!   a program with an unreached undefined constant still runs.
//! - The checker types such a read `mixed` and records its span; this pass replaces exactly those
//!   spans with `CompilerCall::UndefinedConstant`, lowered as the throw.
//! - A deferred method call keeps its shape; only its receiver is wrapped in
//!   `CompilerCall::MixedReceiver`, so lowering dispatches on the object the call meets.

use std::collections::HashSet;

use crate::names::{CompilerCall, Name};
use crate::parser::ast::{ClassMethod, Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;
use crate::undefined_reads::for_each_child_expr_mut;

/// The checker's post-check sites, by span.
struct Sites<'a> {
    constants: &'a HashSet<Span>,
    method_calls: &'a HashSet<Span>,
}

/// Rewrites every recorded undefined constant read and deferred method call in the program.
pub fn rewrite(program: &mut [Stmt], constants: &HashSet<Span>, method_calls: &HashSet<Span>) {
    if constants.is_empty() && method_calls.is_empty() {
        return;
    }
    rewrite_stmts(program, &Sites { constants, method_calls });
}

/// Rewrites a statement list.
fn rewrite_stmts(stmts: &mut [Stmt], sites: &Sites<'_>) {
    for stmt in stmts.iter_mut() {
        rewrite_stmt(stmt, sites);
    }
}

/// Rewrites the expressions and nested bodies of one statement.
fn rewrite_stmt(stmt: &mut Stmt, sites: &Sites<'_>) {
    match &mut stmt.kind {
        StmtKind::FunctionDecl { body, .. } => rewrite_stmts(body, sites),
        StmtKind::ClassDecl { methods, .. }
        | StmtKind::TraitDecl { methods, .. }
        | StmtKind::InterfaceDecl { methods, .. }
        | StmtKind::EnumDecl { methods, .. } => rewrite_methods(methods, sites),
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ArrayPush { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::StaticVar { init: value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::Echo(value)
        | StmtKind::Throw(value)
        | StmtKind::ExprStmt(value)
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. }
        | StmtKind::NestedArrayAssign { value, .. }
        | StmtKind::Return(Some(value)) => rewrite_expr(value, sites),
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            rewrite_expr(index, sites);
            rewrite_expr(value, sites);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            rewrite_expr(object, sites);
            rewrite_expr(value, sites);
        }
        StmtKind::PropertyArrayAssign { object, index, value, .. } => {
            rewrite_expr(object, sites);
            rewrite_expr(index, sites);
            rewrite_expr(value, sites);
        }
        StmtKind::Foreach { array, body, .. } => {
            rewrite_expr(array, sites);
            rewrite_stmts(body, sites);
        }
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            rewrite_expr(condition, sites);
            rewrite_stmts(then_body, sites);
            for (condition, body) in elseif_clauses.iter_mut() {
                rewrite_expr(condition, sites);
                rewrite_stmts(body, sites);
            }
            if let Some(body) = else_body {
                rewrite_stmts(body, sites);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            rewrite_expr(condition, sites);
            rewrite_stmts(body, sites);
        }
        StmtKind::For { init, condition, update, body } => {
            if let Some(init) = init {
                rewrite_stmt(init, sites);
            }
            if let Some(condition) = condition {
                rewrite_expr(condition, sites);
            }
            if let Some(update) = update {
                rewrite_stmt(update, sites);
            }
            rewrite_stmts(body, sites);
        }
        StmtKind::Switch { subject, cases, default } => {
            rewrite_expr(subject, sites);
            for (conditions, body) in cases.iter_mut() {
                for condition in conditions.iter_mut() {
                    rewrite_expr(condition, sites);
                }
                rewrite_stmts(body, sites);
            }
            if let Some(body) = default {
                rewrite_stmts(body, sites);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            rewrite_stmts(try_body, sites);
            for catch in catches.iter_mut() {
                rewrite_stmts(&mut catch.body, sites);
            }
            if let Some(body) = finally_body {
                rewrite_stmts(body, sites);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            rewrite_stmts(then_body, sites);
            if let Some(body) = else_body {
                rewrite_stmts(body, sites);
            }
        }
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => rewrite_stmts(body, sites),
        _ => {}
    }
}

/// Rewrites every method body.
fn rewrite_methods(methods: &mut [ClassMethod], sites: &Sites<'_>) {
    for method in methods.iter_mut() {
        rewrite_stmts(&mut method.body, sites);
    }
}

/// Rewrites one expression tree.
fn rewrite_expr(expr: &mut Expr, sites: &Sites<'_>) {
    match &mut expr.kind {
        ExprKind::ConstRef(name) if sites.constants.contains(&expr.span) => {
            let span = expr.span;
            let name = name.as_str().trim_start_matches('\\').to_string();
            *expr = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::UndefinedConstant),
                    args: vec![Expr::new(ExprKind::StringLiteral(name), span)],
                },
                span,
            );
        }
        ExprKind::NullCoalesce { value, default } => {
            rewrite_expr(value, sites);
            rewrite_expr(default, sites);
        }
        ExprKind::ErrorSuppress(inner) => rewrite_expr(inner, sites),
        ExprKind::Closure { body, .. } => rewrite_stmts(body, sites),
        ExprKind::Assignment { target, value, .. } => {
            rewrite_expr(target, sites);
            rewrite_expr(value, sites);
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewGeneric { args, .. }
        | ExprKind::ClosureCall { args, .. } => rewrite_exprs(args, sites),
        ExprKind::MethodCall { object, args, .. } | ExprKind::NullsafeMethodCall { object, args, .. } => {
            rewrite_expr(object, sites);
            rewrite_exprs(args, sites);
            if sites.method_calls.contains(&expr.span) {
                let span = object.span;
                let receiver = std::mem::replace(object.as_mut(), Expr::new(ExprKind::Null, span));
                **object = Expr::new(
                    ExprKind::FunctionCall {
                        name: Name::compiler_call(CompilerCall::MixedReceiver),
                        args: vec![receiver],
                    },
                    span,
                );
            }
        }
        ExprKind::ExprCall { callee, args } => {
            rewrite_expr(callee, sites);
            rewrite_exprs(args, sites);
        }
        _ => for_each_child_expr_mut(expr, &mut |child| rewrite_expr(child, sites)),
    }
}

/// Rewrites a list of expressions.
fn rewrite_exprs(exprs: &mut [Expr], sites: &Sites<'_>) {
    for expr in exprs.iter_mut() {
        rewrite_expr(expr, sites);
    }
}
