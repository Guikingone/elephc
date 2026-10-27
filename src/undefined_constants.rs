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

/// Rewrites every recorded undefined constant read and deferred method call in the program.
pub fn rewrite(program: &mut [Stmt], constants: &HashSet<Span>, method_calls: &HashSet<Span>) {
    if constants.is_empty() && method_calls.is_empty() {
        return;
    }
    walk_program_exprs_mut(program, &mut |expr| rewrite_site(expr, constants, method_calls));
}

/// Applies the same rewrite to method bodies kept outside the program: lowering emits a class's
/// methods from the checker's `method_decls` copies, not from the declaration statement.
pub fn rewrite_methods(
    methods: &mut [ClassMethod],
    constants: &HashSet<Span>,
    method_calls: &HashSet<Span>,
) {
    if constants.is_empty() && method_calls.is_empty() {
        return;
    }
    walk_methods(methods, &mut |expr| rewrite_site(expr, constants, method_calls), true);
}

/// Applies the checker's decision to one recorded node.
fn rewrite_site(expr: &mut Expr, constants: &HashSet<Span>, method_calls: &HashSet<Span>) {
    let span = expr.span;
    match &mut expr.kind {
        ExprKind::ConstRef(name) if constants.contains(&span) => {
            let name = name.as_str().trim_start_matches('\\').to_string();
            *expr = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::UndefinedConstant),
                    args: vec![Expr::new(ExprKind::StringLiteral(name), span)],
                },
                span,
            );
        }
        ExprKind::MethodCall { object, .. } | ExprKind::NullsafeMethodCall { object, .. }
            if method_calls.contains(&span) =>
        {
            let receiver_span = object.span;
            let receiver = std::mem::replace(object.as_mut(), Expr::new(ExprKind::Null, receiver_span));
            **object = Expr::new(
                ExprKind::FunctionCall {
                    name: Name::compiler_call(CompilerCall::MixedReceiver),
                    args: vec![receiver],
                },
                receiver_span,
            );
        }
        _ => {}
    }
}

/// Calls `visit` on every expression of the program, pre-order, through statement bodies,
/// methods and closures. `visit` may replace the node it is given; the replacement's children
/// are walked.
pub(crate) fn walk_program_exprs_mut(program: &mut [Stmt], visit: &mut dyn FnMut(&mut Expr)) {
    walk_stmts(program, visit, true);
}

/// Like `walk_program_exprs_mut`, but stays in one variable scope: function and class bodies and
/// non-arrow closures are not entered (an arrow function reads its enclosing scope).
pub(crate) fn walk_scope_exprs_mut(stmts: &mut [Stmt], visit: &mut dyn FnMut(&mut Expr)) {
    walk_stmts(stmts, visit, false);
}

/// Visits a statement list.
fn walk_stmts(stmts: &mut [Stmt], visit: &mut dyn FnMut(&mut Expr), nested: bool) {
    for stmt in stmts.iter_mut() {
        walk_stmt(stmt, visit, nested);
    }
}

/// Visits the expressions and nested bodies of one statement.
fn walk_stmt(stmt: &mut Stmt, visit: &mut dyn FnMut(&mut Expr), nested: bool) {
    match &mut stmt.kind {
        StmtKind::FunctionDecl { body, .. } => {
            if nested {
                walk_stmts(body, visit, nested)
            }
        }
        StmtKind::ClassDecl { methods, .. }
        | StmtKind::TraitDecl { methods, .. }
        | StmtKind::InterfaceDecl { methods, .. }
        | StmtKind::EnumDecl { methods, .. } => {
            if nested {
                walk_methods(methods, visit, nested)
            }
        }
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
        | StmtKind::Return(Some(value)) => walk_expr(value, visit, nested),
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            walk_expr(index, visit, nested);
            walk_expr(value, visit, nested);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            walk_expr(object, visit, nested);
            walk_expr(value, visit, nested);
        }
        StmtKind::PropertyArrayAssign { object, index, value, .. } => {
            walk_expr(object, visit, nested);
            walk_expr(index, visit, nested);
            walk_expr(value, visit, nested);
        }
        StmtKind::Foreach { array, body, .. } => {
            walk_expr(array, visit, nested);
            walk_stmts(body, visit, nested);
        }
        StmtKind::If { condition, then_body, elseif_clauses, else_body } => {
            walk_expr(condition, visit, nested);
            walk_stmts(then_body, visit, nested);
            for (condition, body) in elseif_clauses.iter_mut() {
                walk_expr(condition, visit, nested);
                walk_stmts(body, visit, nested);
            }
            if let Some(body) = else_body {
                walk_stmts(body, visit, nested);
            }
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            walk_expr(condition, visit, nested);
            walk_stmts(body, visit, nested);
        }
        StmtKind::For { init, condition, update, body } => {
            if let Some(init) = init {
                walk_stmt(init, visit, nested);
            }
            if let Some(condition) = condition {
                walk_expr(condition, visit, nested);
            }
            if let Some(update) = update {
                walk_stmt(update, visit, nested);
            }
            walk_stmts(body, visit, nested);
        }
        StmtKind::Switch { subject, cases, default } => {
            walk_expr(subject, visit, nested);
            for (conditions, body) in cases.iter_mut() {
                for condition in conditions.iter_mut() {
                    walk_expr(condition, visit, nested);
                }
                walk_stmts(body, visit, nested);
            }
            if let Some(body) = default {
                walk_stmts(body, visit, nested);
            }
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            walk_stmts(try_body, visit, nested);
            for catch in catches.iter_mut() {
                walk_stmts(&mut catch.body, visit, nested);
            }
            if let Some(body) = finally_body {
                walk_stmts(body, visit, nested);
            }
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            walk_stmts(then_body, visit, nested);
            if let Some(body) = else_body {
                walk_stmts(body, visit, nested);
            }
        }
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => walk_stmts(body, visit, nested),
        _ => {}
    }
}

/// Visits every method body.
fn walk_methods(methods: &mut [ClassMethod], visit: &mut dyn FnMut(&mut Expr), nested: bool) {
    for method in methods.iter_mut() {
        walk_stmts(&mut method.body, visit, nested);
    }
}

/// Visits one expression tree: the node first (it may replace itself), then its children.
fn walk_expr(expr: &mut Expr, visit: &mut dyn FnMut(&mut Expr), nested: bool) {
    visit(expr);
    match &mut expr.kind {
        ExprKind::NullCoalesce { value, default } => {
            walk_expr(value, visit, nested);
            walk_expr(default, visit, nested);
        }
        ExprKind::ErrorSuppress(inner) => walk_expr(inner, visit, nested),
        ExprKind::Closure { body, is_arrow, .. } => {
            if nested || *is_arrow {
                walk_stmts(body, visit, nested)
            }
        }
        ExprKind::Assignment { target, value, .. } => {
            walk_expr(target, visit, nested);
            walk_expr(value, visit, nested);
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewGeneric { args, .. }
        | ExprKind::ClosureCall { args, .. } => walk_exprs(args, visit, nested),
        ExprKind::MethodCall { object, args, .. } | ExprKind::NullsafeMethodCall { object, args, .. } => {
            walk_expr(object, visit, nested);
            walk_exprs(args, visit, nested);
        }
        ExprKind::ExprCall { callee, args } => {
            walk_expr(callee, visit, nested);
            walk_exprs(args, visit, nested);
        }
        _ => for_each_child_expr_mut(expr, &mut |child| walk_expr(child, visit, nested)),
    }
}

/// Visits a list of expressions.
fn walk_exprs(exprs: &mut [Expr], visit: &mut dyn FnMut(&mut Expr), nested: bool) {
    for expr in exprs.iter_mut() {
        walk_expr(expr, visit, nested);
    }
}
