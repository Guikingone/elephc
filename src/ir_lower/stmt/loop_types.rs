//! Purpose:
//! Pre-scans a loop's body and header expressions before lowering it and widens the
//! flow-sensitive local-type facts of any local whose in-loop reassignment would push
//! its frame storage cross-category to `Mixed`.
//!
//! Called from:
//! - `crate::ir_lower::stmt::lower_while`, `lower_do_while`, `lower_for`, and `lower_foreach`.
//!
//! Key details:
//! - `local_types` types every `load_local`. A loop back-edge makes a slot that is widened
//!   to `Mixed` inside the body live on later iterations at reads placed textually before
//!   the reassignment, so those reads must already be typed `Mixed` or they coerce a `Mixed`
//!   cell to the stale narrow type (the same wrong-cast class fixed for `if`/`switch` in the
//!   branch-scoping change). Widening to `Mixed` is the only retroactively-lossless target:
//!   a `Mixed` cell can hold either iteration's value and reads coerce correctly.
//! - Only locals that already have an entry type are considered; a local first assigned
//!   inside the loop has no stale entry type to leak. Same-category widenings (int↔bool,
//!   array↔array) are intentionally left alone: they are a separate concern and forcing them
//!   to `Mixed` would both over-box and misrepresent the entry value.

use std::collections::HashMap;

use crate::ir::widened_local_storage_type;
use crate::ir_lower::context::LoweringContext;
use crate::parser::ast::{BinOp, CastType, Expr, ExprKind, Stmt, StmtKind};
use crate::types::checker::infer_expr_type_syntactic;
use crate::types::PhpType;

/// Widens loop-carried locals to `Mixed` before the loop body/header is lowered.
///
/// Scans `bodies` (loop body statement lists) and `single_stmts` (e.g. a `for` update
/// statement) for reassignments, plus `exprs` (e.g. a loop condition) for expression-position
/// reassignments, then, for each local whose known entry type would be widened to `Mixed` by an
/// in-loop assignment, sets its logical type to `Mixed` for the whole loop scope. Locals whose
/// in-loop assignments stay within their storage category, and locals without an entry type, are
/// left untouched so ordinary counters/accumulators are not boxed.
pub(super) fn prewiden_loop_carried_locals(
    ctx: &mut LoweringContext<'_, '_>,
    bodies: &[&[Stmt]],
    single_stmts: &[&Stmt],
    exprs: &[&Expr],
) {
    let mut assigned: HashMap<String, Vec<Option<PhpType>>> = HashMap::new();
    for body in bodies {
        for stmt in *body {
            collect_stmt(ctx, stmt, &mut assigned);
        }
    }
    for stmt in single_stmts {
        collect_stmt(ctx, stmt, &mut assigned);
    }
    for expr in exprs {
        collect_expr(ctx, expr, &mut assigned);
    }

    let mut to_widen: Vec<String> = Vec::new();
    for (name, estimates) in &assigned {
        let Some(entry_type) = ctx.local_types.get(name).cloned() else {
            continue;
        };
        let entry_repr = entry_type.codegen_repr();
        if matches!(entry_repr, PhpType::Mixed) {
            continue;
        }
        if reassignment_widens_to_mixed(&entry_type, &entry_repr, estimates) {
            to_widen.push(name.clone());
        }
    }
    for name in to_widen {
        ctx.set_local_type(&name, PhpType::Mixed);
    }
}

/// Returns true when any collected in-loop assignment would widen `entry_type`'s storage to `Mixed`.
///
/// A known assigned type triggers widening only when it is genuinely cross-category with the entry
/// type (matching `widened_local_storage_type`). An unresolved assignment (`None`) triggers widening
/// only for scalar entry types, where boxing is cheap and the wrong-cast miscompile actually occurs;
/// for non-scalar entries an unresolved assignment is assumed type-preserving to avoid boxing object
/// or array locals (which would route through the slower `Mixed` dispatch paths).
///
/// A `null` entry is a scalar entry here, and any assignment of a non-null type widens it.
/// `widened_local_storage_type` deliberately folds `Void`/`Never` into the `Int`/`Bool` category —
/// they share the single-machine-word storage class — and so answers `incoming` rather than `Mixed`
/// for `null` reassigned to an `int`. That answer is right about *storage* and wrong about *type*:
/// PHP's `null` is not an integer, and a read placed textually before the reassignment stays typed
/// `null` while the slot holds the previous iteration's int. That is precisely the stale-type leak
/// this module exists to prevent, so `null` entries are tested against the storage category
/// directly instead of through the storage-widening lattice.
fn reassignment_widens_to_mixed(
    entry_type: &PhpType,
    entry_repr: &PhpType,
    estimates: &[Option<PhpType>],
) -> bool {
    let null_entry = is_null_category(entry_repr);
    let scalar_entry = null_entry
        || matches!(
            entry_repr,
            PhpType::Int | PhpType::Float | PhpType::Str | PhpType::Bool | PhpType::TaggedScalar
        );
    for estimate in estimates {
        match estimate {
            Some(assigned) => {
                let assigned_repr = assigned.codegen_repr();
                // An indexed array literal reassigned to an assoc-array local (or vice versa) is
                // reconciled to the local's array storage by `contextualize_array_assignment`, so
                // it never widens the slot to Mixed even though `widened_local_storage_type` reports
                // Mixed for the raw `Array`/`AssocArray` mismatch. Treat both as one array category.
                if is_array_category(entry_repr) && is_array_category(&assigned_repr) {
                    continue;
                }
                if null_entry && !is_null_category(&assigned_repr) {
                    return true;
                }
                if matches!(widened_local_storage_type(entry_type, assigned), PhpType::Mixed) {
                    return true;
                }
            }
            None => {
                if scalar_entry {
                    return true;
                }
            }
        }
    }
    false
}

/// Returns true for the storage categories that carry no value of their own (PHP `null`).
fn is_null_category(ty: &PhpType) -> bool {
    matches!(ty, PhpType::Void | PhpType::Never)
}

/// Returns true for the array-shaped storage categories (`array<...>` and keyed `array<k, v>`).
fn is_array_category(ty: &PhpType) -> bool {
    matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. })
}

/// Records reassignments found in one statement, recursing into every nested statement body.
///
/// Declaration bodies (functions, closures, classes) open a new variable scope, so their
/// assignments do not touch the enclosing loop's locals and are not descended into.
fn collect_stmt(
    ctx: &LoweringContext<'_, '_>,
    stmt: &Stmt,
    assigned: &mut HashMap<String, Vec<Option<PhpType>>>,
) {
    match &stmt.kind {
        StmtKind::Assign { name, value } => {
            record_assignment(ctx, name, value, assigned);
            collect_expr(ctx, value, assigned);
        }
        StmtKind::TypedAssign { type_expr, name, value } => {
            let declared = ctx.type_expr_to_php_type_for_value(type_expr);
            assigned.entry(name.clone()).or_default().push(Some(declared));
            collect_expr(ctx, value, assigned);
        }
        StmtKind::ArrayAssign { index, value, .. } => {
            collect_expr(ctx, index, assigned);
            collect_expr(ctx, value, assigned);
        }
        StmtKind::ArrayPush { value, .. } => collect_expr(ctx, value, assigned),
        StmtKind::NestedArrayAssign { target, value } => {
            collect_expr(ctx, target, assigned);
            collect_expr(ctx, value, assigned);
        }
        StmtKind::Echo(expr) | StmtKind::Throw(expr) | StmtKind::ExprStmt(expr) => {
            collect_expr(ctx, expr, assigned);
        }
        StmtKind::Return(Some(expr)) => collect_expr(ctx, expr, assigned),
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            collect_expr(ctx, condition, assigned);
            collect_body(ctx, then_body, assigned);
            for (clause_condition, clause_body) in elseif_clauses {
                collect_expr(ctx, clause_condition, assigned);
                collect_body(ctx, clause_body, assigned);
            }
            if let Some(else_body) = else_body {
                collect_body(ctx, else_body, assigned);
            }
        }
        StmtKind::While { condition, body } => {
            collect_expr(ctx, condition, assigned);
            collect_body(ctx, body, assigned);
        }
        StmtKind::DoWhile { body, condition } => {
            collect_body(ctx, body, assigned);
            collect_expr(ctx, condition, assigned);
        }
        StmtKind::For {
            init,
            condition,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_stmt(ctx, init, assigned);
            }
            if let Some(condition) = condition {
                collect_expr(ctx, condition, assigned);
            }
            if let Some(update) = update {
                collect_stmt(ctx, update, assigned);
            }
            collect_body(ctx, body, assigned);
        }
        StmtKind::Foreach { array, body, .. } => {
            collect_expr(ctx, array, assigned);
            collect_body(ctx, body, assigned);
        }
        StmtKind::Switch { subject, cases, default } => {
            collect_expr(ctx, subject, assigned);
            for (patterns, body) in cases {
                for pattern in patterns {
                    collect_expr(ctx, pattern, assigned);
                }
                collect_body(ctx, body, assigned);
            }
            if let Some(default) = default {
                collect_body(ctx, default, assigned);
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_body(ctx, try_body, assigned);
            for catch in catches {
                if let Some(variable) = &catch.variable {
                    let caught = if catch.exception_types.len() == 1 {
                        PhpType::Object(
                            catch.exception_types[0]
                                .as_str()
                                .trim_start_matches('\\')
                                .to_string(),
                        )
                    } else {
                        PhpType::Object("Throwable".to_string())
                    };
                    assigned
                        .entry(variable.clone())
                        .or_default()
                        .push(Some(caught));
                }
                collect_body(ctx, &catch.body, assigned);
            }
            if let Some(finally_body) = finally_body {
                collect_body(ctx, finally_body, assigned);
            }
        }
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => collect_body(ctx, body, assigned),
        _ => {}
    }
}

/// Recurses over a statement list, collecting reassignments for the pre-scan.
fn collect_body(
    ctx: &LoweringContext<'_, '_>,
    body: &[Stmt],
    assigned: &mut HashMap<String, Vec<Option<PhpType>>>,
) {
    for stmt in body {
        collect_stmt(ctx, stmt, assigned);
    }
}

/// Records a `$name = value` reassignment estimate for the pre-scan.
fn record_assignment(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    value: &Expr,
    assigned: &mut HashMap<String, Vec<Option<PhpType>>>,
) {
    let estimate = estimate_assigned_type(ctx, value);
    assigned.entry(name.to_string()).or_default().push(estimate);
}

/// Walks an expression for expression-position `$var = ...` reassignments and nested sub-expressions.
///
/// Only simple-variable assignment targets contribute to the pre-scan (array/property targets do not
/// change a local's own type). The walk descends structurally so an assignment buried inside a call
/// argument or condition is still seen.
fn collect_expr(
    ctx: &LoweringContext<'_, '_>,
    expr: &Expr,
    assigned: &mut HashMap<String, Vec<Option<PhpType>>>,
) {
    if let ExprKind::Assignment { target, value, .. } = &expr.kind {
        if let ExprKind::Variable(name) = &target.kind {
            record_assignment(ctx, name, value, assigned);
        }
    }
    for child in child_expressions(expr) {
        collect_expr(ctx, child, assigned);
    }
}

/// Estimates the PHP type an expression would produce, using flow-sensitive local facts.
///
/// Returns `None` when the type cannot be determined cheaply (for example a method call or a
/// property access), which the caller treats as "possibly widening" for scalar entry types.
/// Unlike the purely syntactic inferer, variables resolve to their current flow type so that
/// counters and accumulators (`$i = $i + 1`, `$s = $s . "x"`) are recognised as type-preserving.
fn estimate_assigned_type(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> Option<PhpType> {
    match &expr.kind {
        ExprKind::IntLiteral(_) => Some(PhpType::Int),
        ExprKind::FloatLiteral(_) => Some(PhpType::Float),
        ExprKind::StringLiteral(_) => Some(PhpType::Str),
        ExprKind::BoolLiteral(_) => Some(PhpType::Bool),
        ExprKind::Null => Some(PhpType::Void),
        ExprKind::Variable(name) => Some(ctx.local_type(name)),
        ExprKind::PreIncrement(name)
        | ExprKind::PostIncrement(name)
        | ExprKind::PreDecrement(name)
        | ExprKind::PostDecrement(name) => Some(ctx.local_type(name)),
        ExprKind::Cast { target, .. } => Some(cast_target_type(target)),
        ExprKind::Not(_) | ExprKind::InstanceOf { .. } => Some(PhpType::Bool),
        ExprKind::BitNot(_) => Some(PhpType::Int),
        ExprKind::Negate(inner) => Some(numeric_result(ctx, inner)),
        ExprKind::BinaryOp { left, op, right } => estimate_binary(ctx, left, op, right),
        ExprKind::Ternary { then_expr, else_expr, .. } => {
            join_estimates(estimate_assigned_type(ctx, then_expr), estimate_assigned_type(ctx, else_expr))
        }
        ExprKind::ShortTernary { value, default } | ExprKind::NullCoalesce { value, default } => {
            join_estimates(estimate_assigned_type(ctx, value), estimate_assigned_type(ctx, default))
        }
        ExprKind::ArrayLiteral(_) => Some(PhpType::Array(Box::new(PhpType::Mixed))),
        ExprKind::ArrayLiteralAssoc(_) => Some(PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(PhpType::Mixed),
        }),
        ExprKind::NewObject { class_name, .. } => Some(PhpType::Object(class_name.as_str().to_string())),
        ExprKind::Closure { .. } | ExprKind::FirstClassCallable(_) => Some(PhpType::Callable),
        ExprKind::ErrorSuppress(inner) => estimate_assigned_type(ctx, inner),
        ExprKind::FunctionCall { name, .. } => Some(
            ctx.functions
                .get(name.as_str())
                .map(|sig| sig.return_type.clone())
                .unwrap_or_else(|| infer_expr_type_syntactic(expr)),
        ),
        _ => None,
    }
}

/// Estimates the result type of a binary operator from its operands' flow types.
fn estimate_binary(ctx: &LoweringContext<'_, '_>, left: &Expr, op: &BinOp, right: &Expr) -> Option<PhpType> {
    match op {
        BinOp::Concat => Some(PhpType::Str),
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Mod => {
            // `+` over two arrays is the array-union operator; every other numeric case yields
            // int unless a float operand promotes it. Being conservative here only affects
            // whether a numeric local is considered type-stable, never correctness.
            if matches!(op, BinOp::Add)
                && matches!(estimate_assigned_type(ctx, left), Some(PhpType::Array(_)) | Some(PhpType::AssocArray { .. }))
            {
                return None;
            }
            Some(numeric_binary_result(ctx, left, right))
        }
        BinOp::Div | BinOp::Pow => Some(PhpType::Float),
        BinOp::Eq
        | BinOp::NotEq
        | BinOp::StrictEq
        | BinOp::StrictNotEq
        | BinOp::Lt
        | BinOp::Gt
        | BinOp::LtEq
        | BinOp::GtEq
        | BinOp::And
        | BinOp::Or
        | BinOp::Xor => Some(PhpType::Bool),
        BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::ShiftLeft | BinOp::ShiftRight => {
            Some(PhpType::Int)
        }
        BinOp::Spaceship => Some(PhpType::Int),
        BinOp::NullCoalesce => join_estimates(
            estimate_assigned_type(ctx, left),
            estimate_assigned_type(ctx, right),
        ),
    }
}

/// Returns `Float` when either operand is float-typed, else `Int`, for numeric operators.
fn numeric_binary_result(ctx: &LoweringContext<'_, '_>, left: &Expr, right: &Expr) -> PhpType {
    if numeric_result(ctx, left) == PhpType::Float || numeric_result(ctx, right) == PhpType::Float {
        PhpType::Float
    } else {
        PhpType::Int
    }
}

/// Returns the numeric category (`Float`/`Int`) of an operand, defaulting to `Int` when unknown.
fn numeric_result(ctx: &LoweringContext<'_, '_>, expr: &Expr) -> PhpType {
    match estimate_assigned_type(ctx, expr) {
        Some(PhpType::Float) => PhpType::Float,
        _ => PhpType::Int,
    }
}

/// Combines two branch estimates: equal types are kept, any disagreement or unknown yields `None`.
fn join_estimates(left: Option<PhpType>, right: Option<PhpType>) -> Option<PhpType> {
    match (left, right) {
        (Some(left), Some(right)) if left.codegen_repr() == right.codegen_repr() => Some(left),
        _ => None,
    }
}

/// Maps a cast target to the PHP type it produces.
fn cast_target_type(target: &CastType) -> PhpType {
    match target {
        CastType::Int => PhpType::Int,
        CastType::Float => PhpType::Float,
        CastType::String => PhpType::Str,
        CastType::Bool => PhpType::Bool,
        CastType::Array => PhpType::Array(Box::new(PhpType::Mixed)),
        CastType::Object => PhpType::Object(String::new()),
    }
}

/// Returns the direct child expressions of `expr` for the structural assignment walk.
fn child_expressions(expr: &Expr) -> Vec<&Expr> {
    let mut children: Vec<&Expr> = Vec::new();
    match &expr.kind {
        ExprKind::BinaryOp { left, right, .. } => {
            children.push(left);
            children.push(right);
        }
        ExprKind::Negate(inner)
        | ExprKind::Not(inner)
        | ExprKind::BitNot(inner)
        | ExprKind::Throw(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Print(inner)
        | ExprKind::Clone(inner)
        | ExprKind::Spread(inner) => children.push(inner),
        ExprKind::InstanceOf { value, .. } => children.push(value),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            children.push(value);
            children.push(default);
        }
        ExprKind::Pipe { value, callable } => {
            children.push(value);
            children.push(callable);
        }
        ExprKind::Assignment { target, value, .. } => {
            children.push(target);
            children.push(value);
        }
        ExprKind::ListUnpack { value, .. } => children.push(value),
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            children.push(condition);
            children.push(then_expr);
            children.push(else_expr);
        }
        ExprKind::Cast { expr, .. } => children.push(expr),
        ExprKind::FunctionCall { args, .. } => children.extend(args.iter()),
        ExprKind::ArrayLiteral(elems) => children.extend(elems.iter()),
        ExprKind::ArrayLiteralAssoc(entries) => {
            for (key, value) in entries {
                children.push(key);
                children.push(value);
            }
        }
        ExprKind::Match { subject, arms, default } => {
            children.push(subject);
            for (patterns, arm) in arms {
                children.extend(patterns.iter());
                children.push(arm);
            }
            if let Some(default) = default {
                children.push(default);
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            children.push(array);
            children.push(index);
        }
        ExprKind::NewObject { args, .. }
        | ExprKind::NewDynamic { args, .. } => children.extend(args.iter()),
        ExprKind::NewDynamicObject { class_name, args, .. } => {
            children.push(class_name);
            children.extend(args.iter());
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => children.push(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            children.push(object);
            children.push(property);
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            children.push(object);
            children.extend(args.iter());
        }
        ExprKind::StaticMethodCall { args, .. } => children.extend(args.iter()),
        ExprKind::ClosureCall { args, .. } => children.extend(args.iter()),
        ExprKind::ExprCall { callee, args } => {
            children.push(callee);
            children.extend(args.iter());
        }
        ExprKind::NamedArg { value, .. } => children.push(value),
        _ => {}
    }
    children
}
