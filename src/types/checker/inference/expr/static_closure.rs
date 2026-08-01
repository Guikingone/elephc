//! Purpose:
//! Infers expression static closure forms for the checker.
//! Handles type facts and diagnostics for expression shapes that need more than scalar/operator inference.
//!
//! Called from:
//! - `crate::types::checker::inference::expr`
//!
//! Key details:
//! - Expression inference shares environments with statement checking, so variable and effect updates must stay synchronized.

use crate::errors::CompileError;
use crate::parser::ast::{CallableTarget, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};
use crate::span::Span;

/// Walk a static closure body and reject any reference to `$this`. PHP forbids
/// `$this` inside `static function() {}` and `static fn() => ...` because the
/// closure isn't bound to an object instance.
pub(super) fn body_must_not_use_this(body: &[Stmt], span: Span) -> Result<(), CompileError> {
    for stmt in body {
        stmt_must_not_use_this(stmt, span)?;
    }
    Ok(())
}

/// Returns true if a closure body references `$this` anywhere, including inside
/// nested closures (which capture `$this` transitively from the enclosing
/// scope) and inside `isset($this)` probes. Unlike `body_must_not_use_this`,
/// which exempts bare `isset($this)` arguments (PHP allows the probe inside
/// static closures), this walker counts every `$this` mention so EIR lowering
/// captures `$this` for non-static closures that probe it via `isset`.
pub(crate) fn closure_body_uses_this(body: &[Stmt]) -> bool {
    body_uses_this(body)
}

/// Walks statements looking for any `$this` mention, including inside `isset()`.
fn body_uses_this(body: &[Stmt]) -> bool {
    body.iter().any(stmt_uses_this)
}

/// Returns true if the statement references `$this` anywhere.
fn stmt_uses_this(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Echo(e)
        | StmtKind::Throw(e)
        | StmtKind::ExprStmt(e)
        | StmtKind::Include { path: e, .. }
        | StmtKind::ConstDecl { value: e, .. }
        | StmtKind::StaticVar { init: e, .. }
        | StmtKind::ListUnpack { value: e, .. }
        | StmtKind::Return(Some(e))
        | StmtKind::Assign { value: e, .. }
        | StmtKind::TypedAssign { value: e, .. }
        | StmtKind::ArrayPush { value: e, .. } => expr_uses_this(e),
        StmtKind::RefAssign { .. } => false,
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_uses_this(index) || expr_uses_this(value)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_uses_this(target) || expr_uses_this(value)
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_uses_this(object) || expr_uses_this(value)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => expr_uses_this(object) || expr_uses_this(index) || expr_uses_this(value),
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_uses_this(value),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_uses_this(index) || expr_uses_this(value)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_uses_this(condition)
                || body_uses_this(then_body)
                || elseif_clauses
                    .iter()
                    .any(|(cond, body)| expr_uses_this(cond) || body_uses_this(body))
                || else_body.as_deref().is_some_and(body_uses_this)
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_uses_this(condition) || body_uses_this(body)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref().is_some_and(stmt_uses_this)
                || condition.as_ref().is_some_and(expr_uses_this)
                || update.as_deref().is_some_and(stmt_uses_this)
                || body_uses_this(body)
        }
        StmtKind::Foreach { array, body, .. } => expr_uses_this(array) || body_uses_this(body),
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_uses_this(subject)
                || cases.iter().any(|(patterns, body)| {
                    patterns.iter().any(expr_uses_this) || body_uses_this(body)
                })
                || default.as_deref().is_some_and(body_uses_this)
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            body_uses_this(try_body)
                || catches.iter().any(|catch| body_uses_this(&catch.body))
                || finally_body.as_deref().is_some_and(body_uses_this)
        }
        StmtKind::NamespaceBlock { body, .. } => body_uses_this(body),
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::InterfaceDecl { .. } => false,
        _ => false,
    }
}

/// Returns true if the expression references `$this` anywhere, including
/// inside `isset()` arguments (unlike `expr_must_not_use_this` which exempts
/// bare `$this` inside `isset`).
fn expr_uses_this(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::This => true,
        ExprKind::BinaryOp { left, right, .. } => expr_uses_this(left) || expr_uses_this(right),
        ExprKind::InstanceOf { value, target } => {
            expr_uses_this(value) || instanceof_target_uses_this(target)
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::Cast { expr: inner, .. } => expr_uses_this(inner),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            expr_uses_this(value) || expr_uses_this(default)
        }
        ExprKind::FunctionCall { name, args } => {
            name.as_str().eq_ignore_ascii_case("isset") || args.iter().any(expr_uses_this)
        }
        ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::StaticMethodCall { args, .. } => args.iter().any(expr_uses_this),
        ExprKind::ExprCall { callee, args } => expr_uses_this(callee) || args.iter().any(expr_uses_this),
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_uses_this(object) || args.iter().any(expr_uses_this)
        }
        ExprKind::ArrayLiteral(items) => items.iter().any(expr_uses_this),
        ExprKind::ArrayLiteralAssoc(pairs) => {
            pairs.iter().any(|(k, v)| expr_uses_this(k) || expr_uses_this(v))
        }
        ExprKind::ArrayAccess { array, index } => expr_uses_this(array) || expr_uses_this(index),
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => expr_uses_this(condition) || expr_uses_this(then_expr) || expr_uses_this(else_expr),
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_uses_this(subject)
                || arms.iter().any(|(patterns, value)| {
                    patterns.iter().any(expr_uses_this) || expr_uses_this(value)
                })
                || default.as_deref().is_some_and(expr_uses_this)
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_uses_this(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_uses_this(object) || expr_uses_this(property)
        }
        ExprKind::NamedArg { value, .. } => expr_uses_this(value),
        ExprKind::BufferNew { len, .. } => expr_uses_this(len),
        ExprKind::FirstClassCallable(target) => callable_target_uses_this(target),
        ExprKind::Closure { body, .. } => body_uses_this(body),
        _ => false,
    }
}

/// Returns true if a callable target references `$this`.
fn callable_target_uses_this(target: &CallableTarget) -> bool {
    match target {
        CallableTarget::Method { object, .. } => expr_uses_this(object),
        CallableTarget::Function(_) | CallableTarget::StaticMethod { .. } => false,
    }
}

/// Returns true if an instanceof target references `$this`.
fn instanceof_target_uses_this(target: &InstanceOfTarget) -> bool {
    match target {
        InstanceOfTarget::Name(_) => false,
        InstanceOfTarget::Expr(expr) => expr_uses_this(expr),
    }
}

/// Returns true when the closure body is provably free of every lexically-resolved scope
/// reference: `$this`, `self::`, `static::`, and `parent::` (recursively, including nested
/// closures/arrow functions). This is the JURY-mandated gate for `Closure::bind`/`bindTo` scope
/// rebinding: `crate::ir_lower` resolves those references against the closure's lexical class, so
/// they must be absent before the checker relaxes visibility against the rebound scope.
pub(crate) fn closure_body_free_of_self_scope(body: &[Stmt]) -> bool {
    !body_uses_this(body) && !body.iter().any(stmt_uses_relative_static)
}

/// Returns true when a closure body uses `$this` but no lexically-resolved static receiver
/// (`self::`/`static::`/`parent::`, recursively including nested closures).
///
/// This is the gate for typing a `Closure::bind($closure, $newThis, $scope)` body's `$this` from
/// `$newThis`. `crate::ir_lower` rebinds `$this` for any such body — the bind installs `$newThis`
/// as the closure's receiver and members dispatch against it at runtime — so the rebound receiver
/// is sound whatever shape the body has. The relative static receivers are still excluded: those
/// `crate::ir_lower` resolves against the closure's LEXICAL class, so authorizing them against the
/// rebound scope would approve a program whose compiled behavior diverges.
pub(crate) fn closure_body_rebinds_this_only(body: &[Stmt]) -> bool {
    body_uses_this(body) && !body.iter().any(stmt_uses_relative_static)
}

/// Returns true if the statement references `self::`/`static::`/`parent::` anywhere. `$this` is
/// covered separately by `stmt_uses_this`; this scanner only adds the relative static receivers.
fn stmt_uses_relative_static(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Echo(e)
        | StmtKind::Throw(e)
        | StmtKind::ExprStmt(e)
        | StmtKind::Include { path: e, .. }
        | StmtKind::ConstDecl { value: e, .. }
        | StmtKind::StaticVar { init: e, .. }
        | StmtKind::ListUnpack { value: e, .. }
        | StmtKind::Return(Some(e))
        | StmtKind::Assign { value: e, .. }
        | StmtKind::TypedAssign { value: e, .. }
        | StmtKind::ArrayPush { value: e, .. } => expr_uses_relative_static(e),
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_uses_relative_static(index) || expr_uses_relative_static(value)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_uses_relative_static(target) || expr_uses_relative_static(value)
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_uses_relative_static(object) || expr_uses_relative_static(value)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            expr_uses_relative_static(object)
                || expr_uses_relative_static(index)
                || expr_uses_relative_static(value)
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_uses_relative_static(value),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_uses_relative_static(index) || expr_uses_relative_static(value)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_uses_relative_static(condition)
                || then_body.iter().any(stmt_uses_relative_static)
                || elseif_clauses.iter().any(|(cond, body)| {
                    expr_uses_relative_static(cond) || body.iter().any(stmt_uses_relative_static)
                })
                || else_body
                    .as_deref()
                    .is_some_and(|b| b.iter().any(stmt_uses_relative_static))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_uses_relative_static(condition) || body.iter().any(stmt_uses_relative_static)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            init.as_deref().is_some_and(stmt_uses_relative_static)
                || condition.as_ref().is_some_and(expr_uses_relative_static)
                || update.as_deref().is_some_and(stmt_uses_relative_static)
                || body.iter().any(stmt_uses_relative_static)
        }
        StmtKind::Foreach { array, body, .. } => {
            expr_uses_relative_static(array) || body.iter().any(stmt_uses_relative_static)
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_uses_relative_static(subject)
                || cases.iter().any(|(patterns, body)| {
                    patterns.iter().any(expr_uses_relative_static)
                        || body.iter().any(stmt_uses_relative_static)
                })
                || default
                    .as_deref()
                    .is_some_and(|b| b.iter().any(stmt_uses_relative_static))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().any(stmt_uses_relative_static)
                || catches
                    .iter()
                    .any(|catch| catch.body.iter().any(stmt_uses_relative_static))
                || finally_body
                    .as_deref()
                    .is_some_and(|b| b.iter().any(stmt_uses_relative_static))
        }
        StmtKind::NamespaceBlock { body, .. } => body.iter().any(stmt_uses_relative_static),
        _ => false,
    }
}

/// Returns true if the expression references `self::`/`static::`/`parent::` anywhere, via any of
/// the seven `StaticReceiver`-bearing `ExprKind`s, recursing through every sub-expression.
fn expr_uses_relative_static(expr: &Expr) -> bool {
    let relative = |receiver: &crate::parser::ast::StaticReceiver| {
        matches!(
            receiver,
            crate::parser::ast::StaticReceiver::Self_
                | crate::parser::ast::StaticReceiver::Static
                | crate::parser::ast::StaticReceiver::Parent
        )
    };
    match &expr.kind {
        ExprKind::ClassConstant { receiver }
        | ExprKind::ScopedConstantAccess { receiver, .. }
        | ExprKind::StaticPropertyAccess { receiver, .. } => relative(receiver),
        ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
            relative(receiver) || expr_uses_relative_static(property)
        }
        ExprKind::StaticMethodCall { receiver, args, .. } => {
            relative(receiver) || args.iter().any(expr_uses_relative_static)
        }
        ExprKind::NewScopedObject { receiver, args } => {
            relative(receiver) || args.iter().any(expr_uses_relative_static)
        }
        ExprKind::BinaryOp { left, right, .. } => {
            expr_uses_relative_static(left) || expr_uses_relative_static(right)
        }
        ExprKind::InstanceOf { value, target } => {
            expr_uses_relative_static(value)
                || matches!(target, InstanceOfTarget::Expr(e) if expr_uses_relative_static(e))
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::Cast { expr: inner, .. } => expr_uses_relative_static(inner),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default } => {
            expr_uses_relative_static(value) || expr_uses_relative_static(default)
        }
        ExprKind::FunctionCall { args, .. } => args.iter().any(expr_uses_relative_static),
        ExprKind::ClosureCall { args, .. } | ExprKind::NewObject { args, .. } => {
            args.iter().any(expr_uses_relative_static)
        }
        ExprKind::ExprCall { callee, args } => {
            expr_uses_relative_static(callee) || args.iter().any(expr_uses_relative_static)
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_uses_relative_static(object) || args.iter().any(expr_uses_relative_static)
        }
        ExprKind::ArrayLiteral(items) => items.iter().any(expr_uses_relative_static),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs
            .iter()
            .any(|(k, v)| expr_uses_relative_static(k) || expr_uses_relative_static(v)),
        ExprKind::ArrayAccess { array, index } => {
            expr_uses_relative_static(array) || expr_uses_relative_static(index)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_uses_relative_static(condition)
                || expr_uses_relative_static(then_expr)
                || expr_uses_relative_static(else_expr)
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_uses_relative_static(subject)
                || arms.iter().any(|(patterns, value)| {
                    patterns.iter().any(expr_uses_relative_static)
                        || expr_uses_relative_static(value)
                })
                || default.as_deref().is_some_and(expr_uses_relative_static)
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_uses_relative_static(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_uses_relative_static(object) || expr_uses_relative_static(property)
        }
        ExprKind::NamedArg { value, .. } => expr_uses_relative_static(value),
        ExprKind::BufferNew { len, .. } => expr_uses_relative_static(len),
        ExprKind::Closure { body, .. } => body.iter().any(stmt_uses_relative_static),
        _ => false,
    }
}

/// Recursively checks a statement and its children, rejecting any `$this` usage.
/// Used to enforce the PHP rule that static closures cannot capture `$this`.
fn stmt_must_not_use_this(stmt: &Stmt, span: Span) -> Result<(), CompileError> {
    match &stmt.kind {
        StmtKind::Echo(e)
        | StmtKind::Throw(e)
        | StmtKind::ExprStmt(e)
        | StmtKind::Include { path: e, .. }
        | StmtKind::ConstDecl { value: e, .. }
        | StmtKind::StaticVar { init: e, .. }
        | StmtKind::ListUnpack { value: e, .. }
        | StmtKind::Return(Some(e))
        | StmtKind::Assign { value: e, .. }
        | StmtKind::TypedAssign { value: e, .. }
        | StmtKind::ArrayPush { value: e, .. } => expr_must_not_use_this(e, span),
        StmtKind::RefAssign { .. } => Ok(()),
        StmtKind::RefAssignToTarget { target, source, .. } => {
            expr_must_not_use_this(target, span)?;
            expr_must_not_use_this(source, span)
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_must_not_use_this(index, span)?;
            expr_must_not_use_this(value, span)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_must_not_use_this(target, span)?;
            expr_must_not_use_this(value, span)
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_must_not_use_this(object, span)?;
            expr_must_not_use_this(value, span)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            expr_must_not_use_this(object, span)?;
            expr_must_not_use_this(index, span)?;
            expr_must_not_use_this(value, span)
        }
        StmtKind::StaticPropertyAssign { value, .. }
        | StmtKind::StaticPropertyArrayPush { value, .. } => expr_must_not_use_this(value, span),
        StmtKind::StaticPropertyArrayAssign { index, value, .. } => {
            expr_must_not_use_this(index, span)?;
            expr_must_not_use_this(value, span)
        }
        StmtKind::DynamicStaticPropertyWrite { property, index, value, .. } => {
            expr_must_not_use_this(property, span)?;
            if let Some(index) = index {
                expr_must_not_use_this(index, span)?;
            }
            expr_must_not_use_this(value, span)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_must_not_use_this(condition, span)?;
            body_must_not_use_this(then_body, span)?;
            for (cond, body) in elseif_clauses {
                expr_must_not_use_this(cond, span)?;
                body_must_not_use_this(body, span)?;
            }
            if let Some(body) = else_body {
                body_must_not_use_this(body, span)?;
            }
            Ok(())
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_must_not_use_this(condition, span)?;
            body_must_not_use_this(body, span)
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(s) = init {
                stmt_must_not_use_this(s, span)?;
            }
            if let Some(c) = condition {
                expr_must_not_use_this(c, span)?;
            }
            if let Some(s) = update {
                stmt_must_not_use_this(s, span)?;
            }
            body_must_not_use_this(body, span)
        }
        StmtKind::Foreach { array, body, .. } => {
            expr_must_not_use_this(array, span)?;
            body_must_not_use_this(body, span)
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_must_not_use_this(subject, span)?;
            for (patterns, body) in cases {
                for pattern in patterns {
                    expr_must_not_use_this(pattern, span)?;
                }
                body_must_not_use_this(body, span)?;
            }
            if let Some(body) = default {
                body_must_not_use_this(body, span)?;
            }
            Ok(())
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            body_must_not_use_this(try_body, span)?;
            for catch in catches {
                body_must_not_use_this(&catch.body, span)?;
            }
            if let Some(body) = finally_body {
                body_must_not_use_this(body, span)?;
            }
            Ok(())
        }
        StmtKind::NamespaceBlock { body, .. } => body_must_not_use_this(body, span),
        StmtKind::FunctionDecl { .. }
        | StmtKind::ClassDecl { .. }
        | StmtKind::TraitDecl { .. }
        | StmtKind::InterfaceDecl { .. } => Ok(()),
        _ => Ok(()),
    }
}

/// Recursively checks an expression and its children, rejecting any `$this` usage.
/// Traverses all expression variants including nested expressions, call arguments,
/// array elements, and closure bodies. Returns an error if a `This` expression is found.
fn expr_must_not_use_this(expr: &Expr, span: Span) -> Result<(), CompileError> {
    match &expr.kind {
        ExprKind::This => Err(CompileError::new(
            span,
            "Cannot use $this inside a static closure",
        )),
        ExprKind::BinaryOp { left, right, .. } => {
            expr_must_not_use_this(left, span)?;
            expr_must_not_use_this(right, span)
        }
        ExprKind::InstanceOf { value, target } => {
            expr_must_not_use_this(value, span)?;
            instanceof_target_must_not_use_this(target, span)
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::PtrCast { expr: inner, .. }
        | ExprKind::Cast { expr: inner, .. } => expr_must_not_use_this(inner, span),
        ExprKind::NullCoalesce { value, default } => {
            expr_must_not_use_this(value, span)?;
            expr_must_not_use_this(default, span)
        }
        ExprKind::ShortTernary { value, default } => {
            expr_must_not_use_this(value, span)?;
            expr_must_not_use_this(default, span)
        }
        ExprKind::FunctionCall { name, args } => {
            if name.as_str().eq_ignore_ascii_case("isset") {
                for arg in args {
                    if matches!(&arg.kind, ExprKind::This) {
                        continue;
                    }
                    expr_must_not_use_this(arg, span)?;
                }
                Ok(())
            } else {
                for arg in args {
                    expr_must_not_use_this(arg, span)?;
                }
                Ok(())
            }
        }
        ExprKind::ClosureCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. }
        | ExprKind::StaticMethodCall { args, .. } => {
            for arg in args {
                expr_must_not_use_this(arg, span)?;
            }
            Ok(())
        }
        ExprKind::ExprCall { callee, args } => {
            expr_must_not_use_this(callee, span)?;
            for arg in args {
                expr_must_not_use_this(arg, span)?;
            }
            Ok(())
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_must_not_use_this(object, span)?;
            for arg in args {
                expr_must_not_use_this(arg, span)?;
            }
            Ok(())
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            expr_must_not_use_this(object, span)?;
            expr_must_not_use_this(method, span)?;
            for arg in args {
                expr_must_not_use_this(arg, span)?;
            }
            Ok(())
        }
        ExprKind::ArrayLiteral(items) => {
            for item in items {
                expr_must_not_use_this(item, span)?;
            }
            Ok(())
        }
        ExprKind::ArrayLiteralAssoc(pairs) => {
            for (k, v) in pairs {
                expr_must_not_use_this(k, span)?;
                expr_must_not_use_this(v, span)?;
            }
            Ok(())
        }
        ExprKind::ArrayAccess { array, index } => {
            expr_must_not_use_this(array, span)?;
            expr_must_not_use_this(index, span)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_must_not_use_this(condition, span)?;
            expr_must_not_use_this(then_expr, span)?;
            expr_must_not_use_this(else_expr, span)
        }
        ExprKind::Match {
            subject,
            arms,
            default,
        } => {
            expr_must_not_use_this(subject, span)?;
            for (patterns, value) in arms {
                for p in patterns {
                    expr_must_not_use_this(p, span)?;
                }
                expr_must_not_use_this(value, span)?;
            }
            if let Some(d) = default {
                expr_must_not_use_this(d, span)?;
            }
            Ok(())
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_must_not_use_this(object, span),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_must_not_use_this(object, span)?;
            expr_must_not_use_this(property, span)
        }
        ExprKind::NamedArg { value, .. } => expr_must_not_use_this(value, span),
        ExprKind::BufferNew { len, .. } => expr_must_not_use_this(len, span),
        ExprKind::FirstClassCallable(target) => callable_target_must_not_use_this(target, span),
        ExprKind::Closure { body, .. } => body_must_not_use_this(body, span),
        _ => Ok(()),
    }
}

/// Checks a callable target, rejecting `$this` if the target is a method call with an object expression.
/// Static method and bare function targets are always allowed since they have no `$this` binding.
fn callable_target_must_not_use_this(
    target: &CallableTarget,
    span: Span,
) -> Result<(), CompileError> {
    match target {
        CallableTarget::Method { object, .. } => expr_must_not_use_this(object, span),
        CallableTarget::Function(_) | CallableTarget::StaticMethod { .. } => Ok(()),
    }
}

/// Checks an instanceof target, rejecting `$this` if the target is a dynamic expression.
/// Name-only targets (class identifiers) are always allowed since they have no `$this` binding.
fn instanceof_target_must_not_use_this(
    target: &InstanceOfTarget,
    span: Span,
) -> Result<(), CompileError> {
    match target {
        InstanceOfTarget::Name(_) => Ok(()),
        InstanceOfTarget::Expr(expr) => expr_must_not_use_this(expr, span),
    }
}

// --- Closure::bind / bindTo scope rebind (JURY-gated) ---
//
// A `Closure::bind($closure, $newThis, $scope)` with a literal `$scope` (`X::class`) rebinds the
// closure's VISIBILITY scope to `X`. `crate::ir_lower` resolves `self::`/`static::`/`parent::`/
// `$this` LEXICALLY (against wherever the closure literal is WRITTEN in source), so relaxing the
// checker's visibility check for a closure body that uses any of those would let the checker
// accept a program whose COMPILED behavior diverges from what it approved (self::/static::
// dispatching to the closure's lexical class, not the rebound one). `closure_body_free_of_self_scope`
// is the JURY-mandated gate: PROVEN absence of `$this`/`self::`/`static::`/`parent::` anywhere in
// the body, recursively including nested closures/arrow functions, is what makes rebinding sound —
// property access on an eligible typed parameter or a non-captured closure-local variable is then
// authorized against that scope instead of the closure's lexically enclosing class (see
// `BoundScopeContext`).







