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
/// scope). Reuses the static-closure `$this` walker, so it stays in lockstep
/// with the constructs that walker covers. Used by EIR lowering to decide
/// whether a non-static closure defined in an instance method must implicitly
/// capture `$this`.
pub(crate) fn closure_body_uses_this(body: &[Stmt]) -> bool {
    body_must_not_use_this(body, Span::dummy()).is_err()
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
        ExprKind::FunctionCall { args, .. }
        | ExprKind::ClosureCall { args, .. }
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
// property access on a PARAMETER typed as (or a subclass of) the rebound scope is then authorized
// against that scope instead of the closure's lexically enclosing class (see `BoundScopeContext`).

use crate::parser::ast::StaticReceiver;
use crate::types::{PhpType, TypeEnv};
use std::collections::HashSet;

use super::super::super::{BoundScopeContext, Checker};

/// Resolves `Closure::bind`/`bindTo`'s `$scope` argument into a rebind class name.
///
/// Only a literal `X::class` (a `ScopedConstantAccess`-free `ClassConstant` with a `Named`
/// receiver) resolves to a rebind, and only when `X` names a REAL declared class (`X::class`
/// itself is already checker-validated elsewhere, so this repeats that lookup defensively rather
/// than trusting it). Every other shape — omitted, literal `null`, the literal string `"static"`
/// (PHP's own default, meaning "keep current scope"), `self::class`/`static::class`/`parent::class`,
/// or any non-literal expression — returns `None`: "omitted/null scope keeps the ORIGINAL scope,
/// no inference from `$newThis`" and "'static' literal = no change" (both J2-established,
/// master-verified rules this spec re-applies verbatim).
pub(crate) fn resolve_bind_scope_class(checker: &Checker, scope_arg: Option<&Expr>) -> Option<String> {
    let scope_arg = scope_arg?;
    let ExprKind::ClassConstant {
        receiver: StaticReceiver::Named(name),
    } = &scope_arg.kind
    else {
        return None;
    };
    let normalized = name.as_str().trim_start_matches('\\').to_string();
    checker.classes.contains_key(&normalized).then_some(normalized)
}

/// Checks a static/instance `Closure::bind`-family call's arguments, relaxing property-access
/// visibility inside a closure LITERAL first argument when the scope rebind is JURY-safe (see
/// the module doc comment above). Shared by `infer_static_method_call_type_with_options`
/// (`crate::types::checker::inference::objects::methods`) and the `??=` assignment-effects
/// pre-pass (`crate::types::checker::inference::expr::effects`) so both agree on the same
/// bound-scope-aware check instead of the pre-pass rejecting the closure body BEFORE the
/// specialized Closure::bind handling ever gets a chance to relax it.
///
/// `closure_arg` is the first (closure) argument; `scope_arg` is the `$scope` argument (or
/// `None` when omitted). The remaining arguments (`$newThis`, and `$scope` itself) are always
/// checked normally — only the closure literal's OWN body ever sees the rebound scope.
pub(crate) fn check_closure_bind_call_args(
    checker: &mut Checker,
    closure_arg: &Expr,
    rest: &[&Expr],
    scope_arg: Option<&Expr>,
    env: &TypeEnv,
) -> Result<(), CompileError> {
    // `env` stays `&TypeEnv` (not the `&mut TypeEnv` `infer_type_with_assignment_effects` uses)
    // so this one helper serves both callers: `infer_static_method_call_type_with_options`
    // (`crate::types::checker::inference::objects::methods`, which only has `&TypeEnv` here —
    // matching the original un-relaxed `for arg in args { self.infer_type(arg, env)?; }` loop
    // this replaces) and the `??=` assignment-effects pre-pass (`effects.rs`, which passes its
    // `&mut TypeEnv` in — Rust reborrows it as `&TypeEnv` automatically at the call site).
    if let Some(scope_class) = resolve_bind_scope_class(checker, scope_arg) {
        if let ExprKind::Closure { params, body, .. } = &closure_arg.kind {
            if closure_body_free_of_self_scope(body) {
                let eligible_params = eligible_bound_scope_params(checker, params, &scope_class);
                let previous = checker.bound_scope_context.replace(BoundScopeContext {
                    scope_class,
                    eligible_params,
                });
                let result = checker.infer_type(closure_arg, env);
                checker.bound_scope_context = previous;
                result?;
                for arg in rest {
                    checker.infer_type(arg, env)?;
                }
                if let Some(scope_arg) = scope_arg {
                    checker.infer_type(scope_arg, env)?;
                }
                return Ok(());
            }
        }
    }
    checker.infer_type(closure_arg, env)?;
    for arg in rest {
        checker.infer_type(arg, env)?;
    }
    if let Some(scope_arg) = scope_arg {
        checker.infer_type(scope_arg, env)?;
    }
    Ok(())
}

/// Returns the closure's own declared parameter names whose type is `Object(class)` where
/// `class` is `scope_class` or a subclass of it (JURY ADDENDUM #2's precise eligibility rule).
fn eligible_bound_scope_params(
    checker: &Checker,
    params: &[(String, Option<crate::parser::ast::TypeExpr>, Option<Expr>, bool)],
    scope_class: &str,
) -> HashSet<String> {
    params
        .iter()
        .filter_map(|(name, type_ann, _, _)| {
            let type_ann = type_ann.as_ref()?;
            let ty = checker.resolve_type_expr(type_ann, Span::dummy()).ok()?;
            let PhpType::Object(class_name) = ty else {
                return None;
            };
            let normalized = class_name.trim_start_matches('\\');
            (normalized == scope_class || checker.is_subclass_of(normalized, scope_class))
                .then(|| name.clone())
        })
        .collect()
}

/// Returns true when `body` is PROVABLY free of `$this`/`self::`/`static::`/`parent::` usage
/// anywhere, including inside nested closures/arrow functions — the JURY ADDENDUM #1 lexical
/// gate. A CONSERVATIVE whitelist scan: any statement or expression shape not explicitly
/// recognized as safe is treated as UNSAFE (over-reject is fine; under-reject would be
/// silent-wrong). Reuses `stmt_must_not_use_this`'s `$this`-only result as a fast reject first
/// (that walker already recognizes strictly more shapes than this conservative one does, so a
/// `$this` it finds is always a real one), then walks again for `self::`/`static::`/`parent::`.
fn closure_body_free_of_self_scope(body: &[Stmt]) -> bool {
    if closure_body_uses_this(body) {
        return false;
    }
    body.iter().all(stmt_free_of_self_scope)
}

/// Conservative (default-reject) statement scan for JURY ADDENDUM #1 — see
/// `closure_body_free_of_self_scope`.
fn stmt_free_of_self_scope(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Echo(e)
        | StmtKind::Throw(e)
        | StmtKind::ExprStmt(e)
        | StmtKind::Return(Some(e))
        | StmtKind::Assign { value: e, .. }
        | StmtKind::TypedAssign { value: e, .. }
        | StmtKind::ArrayPush { value: e, .. } => expr_free_of_self_scope(e),
        StmtKind::Return(None) => true,
        StmtKind::ArrayAssign { index, value, .. } => {
            expr_free_of_self_scope(index) && expr_free_of_self_scope(value)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_free_of_self_scope(target) && expr_free_of_self_scope(value)
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_free_of_self_scope(object) && expr_free_of_self_scope(value)
        }
        StmtKind::PropertyArrayAssign {
            object,
            index,
            value,
            ..
        } => {
            expr_free_of_self_scope(object)
                && expr_free_of_self_scope(index)
                && expr_free_of_self_scope(value)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_free_of_self_scope(condition)
                && then_body.iter().all(stmt_free_of_self_scope)
                && elseif_clauses
                    .iter()
                    .all(|(cond, body)| expr_free_of_self_scope(cond) && body.iter().all(stmt_free_of_self_scope))
                && else_body
                    .as_ref()
                    .is_none_or(|body| body.iter().all(stmt_free_of_self_scope))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_free_of_self_scope(condition) && body.iter().all(stmt_free_of_self_scope)
        }
        StmtKind::Foreach { array, body, .. } => {
            expr_free_of_self_scope(array) && body.iter().all(stmt_free_of_self_scope)
        }
        StmtKind::Switch {
            subject,
            cases,
            default,
        } => {
            expr_free_of_self_scope(subject)
                && cases.iter().all(|(patterns, body)| {
                    patterns.iter().all(expr_free_of_self_scope) && body.iter().all(stmt_free_of_self_scope)
                })
                && default
                    .as_ref()
                    .is_none_or(|body| body.iter().all(stmt_free_of_self_scope))
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            try_body.iter().all(stmt_free_of_self_scope)
                && catches.iter().all(|catch| catch.body.iter().all(stmt_free_of_self_scope))
                && finally_body
                    .as_ref()
                    .is_none_or(|body| body.iter().all(stmt_free_of_self_scope))
        }
        // The parser wraps a closure/function body in one top-level `Synthetic` node; recurse
        // into it exactly like any other nested statement list.
        StmtKind::Synthetic(body) => body.iter().all(stmt_free_of_self_scope),
        StmtKind::Global { .. } | StmtKind::StaticVar { init: _, .. } | StmtKind::Break(_) | StmtKind::Continue(_) => {
            // `static $x;`/`static $x = <const>;` initializers are compile-time constants
            // (no self::/static::/$this reachable there); global/break/continue carry no
            // sub-expressions this gate needs to inspect.
            true
        }
        // Every other shape (RefAssign, ListUnpack, includes, gotos, static-property writes,
        // dynamic-static-property writes, declarations, …) is conservatively rejected: none of
        // these appear in the shipped target shape, and none is worth the risk of an inaccurate
        // hand-written case under this gate's soundness requirement.
        _ => false,
    }
}

/// Conservative (default-reject) expression scan for JURY ADDENDUM #1 — see
/// `closure_body_free_of_self_scope`.
fn expr_free_of_self_scope(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::This => false,
        ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::Variable(_)
        | ExprKind::MagicConstant(_) => true,
        ExprKind::BinaryOp { left, right, .. } => {
            expr_free_of_self_scope(left) && expr_free_of_self_scope(right)
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Spread(inner)
        | ExprKind::Cast { expr: inner, .. } => expr_free_of_self_scope(inner),
        // Pre/post inc/dec bind a plain local variable NAME, not a sub-expression — no
        // $this/self::/static:: is reachable there.
        ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_) => true,
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            expr_free_of_self_scope(value) && expr_free_of_self_scope(default)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_free_of_self_scope(condition)
                && expr_free_of_self_scope(then_expr)
                && expr_free_of_self_scope(else_expr)
        }
        ExprKind::ArrayAccess { array, index } => {
            expr_free_of_self_scope(array) && expr_free_of_self_scope(index)
        }
        ExprKind::ArrayLiteral(items) => items.iter().all(expr_free_of_self_scope),
        ExprKind::ArrayLiteralAssoc(pairs) => pairs
            .iter()
            .all(|(k, v)| expr_free_of_self_scope(k) && expr_free_of_self_scope(v)),
        ExprKind::FunctionCall { args, .. } => args.iter().all(expr_free_of_self_scope),
        ExprKind::ExprCall { callee, args } => {
            expr_free_of_self_scope(callee) && args.iter().all(expr_free_of_self_scope)
        }
        ExprKind::MethodCall { object, args, .. } | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_free_of_self_scope(object) && args.iter().all(expr_free_of_self_scope)
        }
        ExprKind::PropertyAccess { object, .. } | ExprKind::NullsafePropertyAccess { object, .. } => {
            expr_free_of_self_scope(object)
        }
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_free_of_self_scope(object) && expr_free_of_self_scope(property)
        }
        ExprKind::NamedArg { value, .. } => expr_free_of_self_scope(value),
        ExprKind::NewObject { args, .. } => args.iter().all(expr_free_of_self_scope),
        // `self::`/`static::`/`parent::` receivers are exactly what this gate exists to reject;
        // a `Named` (or interpolated-into-a-known-class) receiver is fine.
        ExprKind::ClassConstant { receiver } | ExprKind::ScopedConstantAccess { receiver, .. } => {
            matches!(receiver, StaticReceiver::Named(_))
        }
        ExprKind::StaticMethodCall { receiver, args, .. } => {
            matches!(receiver, StaticReceiver::Named(_)) && args.iter().all(expr_free_of_self_scope)
        }
        ExprKind::StaticPropertyAccess { receiver, .. } => matches!(receiver, StaticReceiver::Named(_)),
        ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
            matches!(receiver, StaticReceiver::Named(_)) && expr_free_of_self_scope(property)
        }
        // Expression-position assignment (`$a = $b = c()`, `if ($x = f())`) desugars with a
        // `prelude` of hoisted statements plus `target`/`value`; all three must be checked.
        ExprKind::Assignment {
            target,
            value,
            prelude,
            ..
        } => {
            expr_free_of_self_scope(target)
                && expr_free_of_self_scope(value)
                && prelude.iter().all(stmt_free_of_self_scope)
        }
        // Nested closures/arrow functions recurse — a nested `$this`/`self::`/`static::` is
        // just as unsound as one at the top level (JURY ADDENDUM #1: "recursively including
        // nested closures/arrow-functions").
        ExprKind::Closure { body, is_static, .. } => *is_static || body.iter().all(stmt_free_of_self_scope),
        _ => false,
    }
}
