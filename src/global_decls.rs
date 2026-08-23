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
//! - The walk is deliberately SHARED rather than duplicated, so the checker can never approve a
//!   decision lowering refuses (or vice versa). Widening it is therefore a behaviour change for
//!   both sides at once: a name a closure declares `global` moves the top level's binding into the
//!   `_eir_global_*` symbol, which is what makes the closure's write visible to main.
//! - Both walks are EXHAUSTIVE on their AST enum on purpose. A `global` is a statement, but a
//!   statement can sit inside an expression (a closure body, an assignment prelude), so a new
//!   variant that carries statements must fail to compile here rather than silently become a
//!   blind spot — that is exactly how the closure and enum-method holes survived.
//! - `StmtKind::PackedClassDecl` is NOT a hole: a packed class declares typed FIELDS only, with no
//!   method bodies and no expressions, so there is nothing in it to walk.

use std::collections::HashSet;

use crate::parser::ast::{CallableTarget, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};

/// Collects the PHP variable names that any function-like body in `statements` declares `global`.
pub(crate) fn collect_global_var_names(statements: &[Stmt]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_in_body(statements, &mut names);
    names
}

/// Recursively scans statement bodies for `global` declarations.
///
/// Exhaustive on `StmtKind`: a new statement that can carry a body or an expression has to be
/// classified here rather than defaulting to "declares nothing", which is how a `global` inside an
/// enum method went uncollected.
fn collect_in_body(statements: &[Stmt], names: &mut HashSet<String>) {
    for stmt in statements {
        collect_in_stmt(stmt, names);
    }
}

/// Scans one statement and everything nested inside it.
fn collect_in_stmt(stmt: &Stmt, names: &mut HashSet<String>) {
    match &stmt.kind {
        StmtKind::Global { vars } => {
            names.extend(vars.iter().cloned());
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_in_expr(condition, names);
            collect_in_body(then_body, names);
            for (condition, body) in elseif_clauses {
                collect_in_expr(condition, names);
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
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            collect_in_expr(condition, names);
            collect_in_body(body, names);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_in_expr(array, names);
            collect_in_body(body, names);
        }
        StmtKind::FunctionDecl { params, body, .. } => {
            collect_in_param_defaults(params, names);
            collect_in_body(body, names);
        }
        StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::Synthetic(body) => {
            collect_in_body(body, names);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_in_stmt(init, names);
            }
            if let Some(condition) = condition {
                collect_in_expr(condition, names);
            }
            if let Some(update) = update {
                collect_in_stmt(update, names);
            }
            collect_in_body(body, names);
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            collect_in_expr(subject, names);
            for (patterns, body) in cases {
                collect_in_exprs(patterns, names);
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
        StmtKind::ClassDecl {
            properties,
            methods,
            constants,
            ..
        }
        | StmtKind::InterfaceDecl {
            properties,
            methods,
            constants,
            ..
        }
        | StmtKind::TraitDecl {
            properties,
            methods,
            constants,
            ..
        } => {
            for property in properties {
                if let Some(default) = &property.default {
                    collect_in_expr(default, names);
                }
            }
            for method in methods {
                collect_in_param_defaults(&method.params, names);
                collect_in_body(&method.body, names);
            }
            for constant in constants {
                collect_in_expr(&constant.value, names);
            }
        }
        // An enum's methods are checked and lowered exactly like a class's, so a `global` written
        // in one binds the same program storage. Skipping them made the checker approve a
        // top-level `unset` of a name the enum method still writes.
        StmtKind::EnumDecl {
            cases,
            methods,
            constants,
            ..
        } => {
            for case in cases {
                if let Some(value) = &case.value {
                    collect_in_expr(value, names);
                }
            }
            for method in methods {
                collect_in_param_defaults(&method.params, names);
                collect_in_body(&method.body, names);
            }
            for constant in constants {
                collect_in_expr(&constant.value, names);
            }
        }
        StmtKind::Echo(expr) | StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => {
            collect_in_expr(expr, names);
        }
        StmtKind::Assign { value, .. }
        | StmtKind::TypedAssign { value, .. }
        | StmtKind::ConstDecl { value, .. }
        | StmtKind::ListUnpack { value, .. }
        | StmtKind::ArrayPush { value, .. }
        | StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => collect_in_expr(value, names),
        StmtKind::RefAssign { source, .. } => collect_in_expr(source, names),
        StmtKind::StaticVar { init, .. } => collect_in_expr(init, names),
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            collect_in_expr(index, names);
            collect_in_expr(value, names);
        }
        StmtKind::NestedArrayAssign { target, value } => {
            collect_in_expr(target, names);
            collect_in_expr(value, names);
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            collect_in_expr(object, names);
            collect_in_expr(value, names);
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            collect_in_expr(object, names);
            collect_in_expr(index, names);
            collect_in_expr(value, names);
        }
        StmtKind::Return(value) => {
            if let Some(value) = value {
                collect_in_expr(value, names);
            }
        }
        StmtKind::Include { path, .. } => collect_in_expr(path, names),
        // A packed class declares typed FIELDS only — no defaults, no bodies, no expressions.
        StmtKind::PackedClassDecl { .. }
        // Externs are C declarations: types and names, never PHP statements.
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. }
        // Leaves: no sub-statements and no sub-expressions.
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::IncludeOnceMark { .. }
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::UseDecl { .. }
        // Variant groups/marks carry function NAMES; the bodies live in their own `FunctionDecl`s.
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. } => {}
    }
}

/// Scans one expression for the statement bodies it can carry.
///
/// Only two expression shapes hold statements — a closure literal's body and the assignment
/// prelude the parser synthesizes — but the walk has to REACH them, so every nesting arm is
/// enumerated. Exhaustive for the same reason [`collect_in_stmt`] is.
fn collect_in_expr(expr: &Expr, names: &mut HashSet<String>) {
    match &expr.kind {
        // The two statement-bearing expression shapes.
        ExprKind::Closure { params, body, .. } => {
            collect_in_param_defaults(params, names);
            collect_in_body(body, names);
        }
        ExprKind::Assignment {
            target,
            value,
            result_target,
            prelude,
            ..
        } => {
            collect_in_expr(target, names);
            collect_in_expr(value, names);
            if let Some(result_target) = result_target {
                collect_in_expr(result_target, names);
            }
            collect_in_body(prelude, names);
        }
        ExprKind::FunctionCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::ClosureCall { args, .. } => collect_in_exprs(args, names),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            collect_in_expr(object, names);
            collect_in_exprs(args, names);
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            collect_in_expr(object, names);
            collect_in_expr(method, names);
            collect_in_exprs(args, names);
        }
        ExprKind::ExprCall { callee, args } => {
            collect_in_expr(callee, names);
            collect_in_exprs(args, names);
        }
        ExprKind::NewDynamic { name_expr, args } => {
            collect_in_expr(name_expr, names);
            collect_in_exprs(args, names);
        }
        ExprKind::NewDynamicObject {
            class_name, args, ..
        } => {
            collect_in_expr(class_name, names);
            collect_in_exprs(args, names);
        }
        ExprKind::Pipe { value, callable } => {
            collect_in_expr(value, names);
            collect_in_expr(callable, names);
        }
        ExprKind::IncludeValue { path, .. } => collect_in_expr(path, names),
        ExprKind::Yield { key, value } => {
            if let Some(key) = key {
                collect_in_expr(key, names);
            }
            if let Some(value) = value {
                collect_in_expr(value, names);
            }
        }
        ExprKind::BinaryOp { left, right, .. } => {
            collect_in_expr(left, names);
            collect_in_expr(right, names);
        }
        ExprKind::InstanceOf { value, target } => {
            collect_in_expr(value, names);
            if let InstanceOfTarget::Expr(target) = target {
                collect_in_expr(target, names);
            }
        }
        ExprKind::YieldFrom(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::NamedArg { value: inner, .. }
        | ExprKind::BufferNew { len: inner, .. } => collect_in_expr(inner, names),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            collect_in_expr(value, names);
            collect_in_expr(default, names);
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_in_expr(condition, names);
            collect_in_expr(then_expr, names);
            collect_in_expr(else_expr, names);
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            collect_in_expr(subject, names);
            for (patterns, body) in arms {
                collect_in_exprs(patterns, names);
                collect_in_expr(body, names);
            }
            if let Some(default) = default {
                collect_in_expr(default, names);
            }
        }
        ExprKind::ArrayLiteral(items) => collect_in_exprs(items, names),
        ExprKind::ArrayLiteralAssoc(items) => {
            for (key, value) in items {
                collect_in_expr(key, names);
                collect_in_expr(value, names);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            collect_in_expr(array, names);
            collect_in_expr(index, names);
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. }
        | ExprKind::ObjectClassName { object } => collect_in_expr(object, names),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            collect_in_expr(object, names);
            collect_in_expr(property, names);
        }
        ExprKind::FirstClassCallable(target) => {
            if let CallableTarget::Method { object, .. } = target {
                collect_in_expr(object, names);
            }
        }
        ExprKind::Variable(_)
        | ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::ConstRef(_)
        | ExprKind::This
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::ClassConstant { .. }
        | ExprKind::ScopedConstantAccess { .. }
        | ExprKind::MagicConstant(_) => {}
    }
}

/// Scans a list of expressions.
fn collect_in_exprs(exprs: &[Expr], names: &mut HashSet<String>) {
    for expr in exprs {
        collect_in_expr(expr, names);
    }
}

/// Scans the default-value expressions of a parameter list, which can hold a closure literal.
fn collect_in_param_defaults(
    params: &[(String, Option<crate::parser::ast::TypeExpr>, Option<Expr>, bool)],
    names: &mut HashSet<String>,
) {
    for (_, _, default, _) in params {
        if let Some(default) = default {
            collect_in_expr(default, names);
        }
    }
}
