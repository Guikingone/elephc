//! Purpose:
//! Closure lowering and eval-presence analysis.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers a closure expression into a callable descriptor backed by an EIR closure function.
pub(super) fn lower_closure(
    ctx: &mut LoweringContext<'_, '_>,
    params: &[(String, Option<TypeExpr>, Option<Expr>, bool)],
    variadic: Option<&str>,
    variadic_by_ref: bool,
    return_type: Option<&TypeExpr>,
    body: &[crate::parser::ast::Stmt],
    captures: &[String],
    capture_refs: &[String],
    expr: &Expr,
    is_static: bool,
) -> LoweredValue {
    lower_closure_with_context(
        ctx,
        params,
        variadic,
        variadic_by_ref,
        return_type,
        body,
        captures,
        capture_refs,
        expr,
        &[],
        None,
        is_static,
    )
}

/// Lowers a closure assigned to a local and specializes self by-reference captures as callable.
pub(crate) fn lower_closure_for_assignment(
    ctx: &mut LoweringContext<'_, '_>,
    assigned_name: &str,
    value: &Expr,
) -> Option<LoweredValue> {
    let ExprKind::Closure {
        params,
        variadic,
        variadic_by_ref,
        return_type,
        body,
        captures,
        capture_refs,
        is_static,
        ..
    } = &value.kind
    else {
        return None;
    };
    if !capture_refs.iter().any(|capture| capture == assigned_name) {
        return None;
    }
    Some(lower_closure_with_context(
        ctx,
        params,
        variadic.as_deref(),
        *variadic_by_ref,
        return_type.as_ref(),
        body,
        captures,
        capture_refs,
        value,
        &[],
        Some(assigned_name),
        *is_static,
    ))
}

/// Lowers a closure expression, applying contextual types to unannotated parameters.
pub(super) fn lower_closure_with_context(
    ctx: &mut LoweringContext<'_, '_>,
    params: &[(String, Option<TypeExpr>, Option<Expr>, bool)],
    variadic: Option<&str>,
    variadic_by_ref: bool,
    return_type: Option<&TypeExpr>,
    body: &[crate::parser::ast::Stmt],
    captures: &[String],
    capture_refs: &[String],
    expr: &Expr,
    contextual_arg_types: &[PhpType],
    self_ref_callable_capture: Option<&str>,
    is_static: bool,
) -> LoweredValue {
    // PHP auto-binds `$this` to non-static closures (including arrow functions)
    // defined inside an instance method, with no `use($this)` needed. The parser
    // never lists `$this` as a capture, so thread it through the existing capture
    // machinery here: load the enclosing `this` and append it to the captures so
    // the closure body gets a `this` local. Only capture when the body actually
    // references `$this` (directly or in a nested closure) — adding an unused
    // capture would push otherwise capture-free closures through capture-only
    // runtime paths. Nested closures compose: each level captures `this` from the
    // level above.
    // A method-defined closure loads the enclosing `this`; a top-level closure
    // that uses `$this` (bound later via `Closure::bind`) gets a null `this`
    // slot the bind fills, typed `Mixed` for runtime-dispatched member access.
    let with_this;
    let captures: &[String] = if !is_static
        && !captures.iter().any(|name| name == "this")
        && crate::types::checker::closure_body_uses_this(body)
    {
        with_this = captures
            .iter()
            .cloned()
            .chain(std::iter::once("this".to_string()))
            .collect::<Vec<_>>();
        &with_this
    } else {
        captures
    };
    let body_contains_eval = body_contains_eval_call(body);
    let mut captured_values = Vec::with_capacity(captures.len());
    let mut capture_params = Vec::with_capacity(captures.len());
    for capture in captures {
        let by_ref = capture_refs.iter().any(|name| name == capture);
        let (captured, php_type) = if capture == "this" && !ctx.local_slots.contains_key("this") {
            // Top-level closure: no enclosing `$this`. Start with a null receiver
            // that `Closure::bind` overwrites; `Mixed` so members dispatch at
            // runtime against the bound object's class.
            (lower_null(ctx, expr), PhpType::Mixed)
        } else {
            let php_type_override = if by_ref && self_ref_callable_capture == Some(capture.as_str()) {
                Some(PhpType::Callable)
            } else if by_ref && body_contains_eval {
                ctx.set_local_type(capture, PhpType::Mixed);
                Some(PhpType::Mixed)
            } else if by_ref
                && matches!(
                    ctx.local_type(capture).codegen_repr(),
                    PhpType::Int | PhpType::Float
                )
                && body_promotes_variable(body, capture)
            {
                // Checked arithmetic through the shared cell can promote an integer to a
                // float — `$n += 1` on `PHP_INT_MAX` — and an `Int` payload silently
                // discarded that. Widen only when the body actually performs such
                // arithmetic: an alias merely reassigned keeps its narrow, cheaper cell.
                ctx.set_local_type(capture, PhpType::Mixed);
                Some(PhpType::Mixed)
            } else {
                None
            };
            let captured = ctx.load_local(capture, Some(expr.span));
            let php_type = php_type_override
                .unwrap_or_else(|| ctx.builder.value_php_type(captured.value));
            (captured, php_type)
        };
        let immediate = by_ref.then_some(Immediate::I64(1));
        ctx.emit_void(Op::ClosureCapture, vec![captured.value], immediate, Op::ClosureCapture.default_effects(), Some(expr.span));
        if by_ref {
            ctx.mark_ref_bound_local(capture);
        }
        captured_values.push(ClosureCapture { value: captured.value });
        capture_params.push((capture.clone(), php_type, by_ref));
    }
    let name = ctx.next_closure_name();
    let loop_storage_scope =
        crate::types::nested_loop_storage_scope(&ctx.loop_storage_scope, expr.span);
    let by_ref_return = matches!(&expr.kind, ExprKind::Closure { by_ref_return: true, .. });
    let signature = if contextual_arg_types.is_empty() {
        function::lower_closure_function(
            ctx,
            &name,
            params,
            variadic,
            variadic_by_ref,
            return_type,
            body,
            &capture_params,
            self_ref_callable_capture,
            by_ref_return,
            loop_storage_scope,
        )
    } else {
        function::lower_closure_function_with_context(
            ctx,
            &name,
            params,
            variadic,
            variadic_by_ref,
            return_type,
            body,
            &capture_params,
            contextual_arg_types,
            self_ref_callable_capture,
            by_ref_return,
            loop_storage_scope,
        )
    };
    let data = ctx.intern_string(&name);
    let closure_operands = captured_values
        .iter()
        .map(|capture| capture.value)
        .collect::<Vec<_>>();
    ctx.set_pending_static_callable_result(StaticCallableBinding::Closure {
        name,
        signature,
        captures: captured_values,
    });
    let closure = ctx.emit_value(
        Op::ClosureNew,
        closure_operands,
        Some(Immediate::Data(data)),
        PhpType::Callable,
        Op::ClosureNew.default_effects(),
        Some(expr.span),
    );
    if let Some(capture) = self_ref_callable_capture {
        ctx.set_local_logical_type(capture, PhpType::Callable);
    }
    closure
}

/// Returns true when `body` performs arithmetic on `$name` that PHP can promote to float.
///
/// `+`, `-`, and `*` on integers overflow into a float, so a cell holding `$name` must be
/// able to carry either. `++`/`--` and the compound assignments both reach here, the
/// latter as an ordinary `Assign` whose value is a `BinaryOp` reading the same variable.
/// Comparisons, concatenation, and plain reassignment cannot promote and are ignored, so
/// a capture only widens when it must.
fn body_promotes_variable(body: &[Stmt], name: &str) -> bool {
    body.iter().any(|stmt| stmt_promotes_variable(stmt, name))
}

/// Returns true when one statement, or any body it owns, promotes `$name`.
fn stmt_promotes_variable(stmt: &Stmt, name: &str) -> bool {
    let promoting_value = |target: &str, value: &Expr| {
        target == name && expr_promotes_variable(value, name)
    };
    match &stmt.kind {
        StmtKind::Assign { name: target, value, .. } => promoting_value(target, value),
        StmtKind::ExprStmt(expr) | StmtKind::Echo(expr) | StmtKind::Return(Some(expr)) => {
            expr_promotes_variable(expr, name)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_promotes_variable(condition, name)
                || body_promotes_variable(then_body, name)
                || elseif_clauses.iter().any(|(condition, body)| {
                    expr_promotes_variable(condition, name) || body_promotes_variable(body, name)
                })
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_promotes_variable(body, name))
        }
        StmtKind::While { condition, body } => {
            expr_promotes_variable(condition, name) || body_promotes_variable(body, name)
        }
        StmtKind::DoWhile { body, condition } => {
            expr_promotes_variable(condition, name) || body_promotes_variable(body, name)
        }
        StmtKind::For { body, .. } | StmtKind::Foreach { body, .. } => {
            body_promotes_variable(body, name)
        }
        StmtKind::Synthetic(body) => body_promotes_variable(body, name),
        _ => false,
    }
}

/// Returns true when an expression promotes `$name` through arithmetic or a step operator.
fn expr_promotes_variable(expr: &Expr, name: &str) -> bool {
    match &expr.kind {
        ExprKind::PreIncrement(target)
        | ExprKind::PostIncrement(target)
        | ExprKind::PreDecrement(target)
        | ExprKind::PostDecrement(target) => target == name,
        ExprKind::BinaryOp { left, op, right } => {
            (matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul)
                && (reads_variable(left, name) || reads_variable(right, name)))
                || expr_promotes_variable(left, name)
                || expr_promotes_variable(right, name)
        }
        _ => false,
    }
}

/// Returns true when an expression reads `$name` directly.
fn reads_variable(expr: &Expr, name: &str) -> bool {
    matches!(&expr.kind, ExprKind::Variable(target) if target == name)
}

/// Returns true when a statement body contains an `eval(...)` call.
pub(crate) fn body_contains_eval_call(body: &[Stmt]) -> bool {
    body.iter().any(stmt_contains_eval_call)
}

/// Returns true when a statement or nested statement body contains an `eval(...)` call.
pub(super) fn stmt_contains_eval_call(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::Echo(expr)
        | StmtKind::Throw(expr)
        | StmtKind::ExprStmt(expr)
        | StmtKind::ConstDecl { value: expr, .. }
        | StmtKind::ListUnpack { value: expr, .. }
        | StmtKind::StaticVar { init: expr, .. }
        | StmtKind::Assign { value: expr, .. }
        | StmtKind::TypedAssign { value: expr, .. }
        | StmtKind::ArrayPush { value: expr, .. }
        | StmtKind::StaticPropertyAssign { value: expr, .. }
        | StmtKind::StaticPropertyArrayPush { value: expr, .. } => expr_contains_eval_call(expr),
        StmtKind::Return(expr) => expr.as_ref().is_some_and(expr_contains_eval_call),
        StmtKind::ArrayAssign { index, value, .. }
        | StmtKind::StaticPropertyArrayAssign { index, value, .. }
        | StmtKind::PropertyArrayAssign { index, value, .. } => {
            expr_contains_eval_call(index) || expr_contains_eval_call(value)
        }
        StmtKind::NestedArrayAssign { target, value } => {
            expr_contains_eval_call(target) || expr_contains_eval_call(value)
        }
        StmtKind::PropertyAssign { object, value, .. }
        | StmtKind::PropertyArrayPush { object, value, .. } => {
            expr_contains_eval_call(object) || expr_contains_eval_call(value)
        }
        StmtKind::If {
            condition,
            then_body,
            elseif_clauses,
            else_body,
        } => {
            expr_contains_eval_call(condition)
                || body_contains_eval_call(then_body)
                || elseif_clauses.iter().any(|(condition, body)| {
                    expr_contains_eval_call(condition) || body_contains_eval_call(body)
                })
                || else_body.as_ref().is_some_and(|body| body_contains_eval_call(body))
        }
        StmtKind::IfDef { then_body, else_body, .. } => {
            body_contains_eval_call(then_body)
                || else_body.as_ref().is_some_and(|body| body_contains_eval_call(body))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { condition, body } => {
            expr_contains_eval_call(condition) || body_contains_eval_call(body)
        }
        StmtKind::For { init, condition, update, body } => {
            init.as_deref().is_some_and(stmt_contains_eval_call)
                || condition.as_ref().is_some_and(expr_contains_eval_call)
                || update.as_deref().is_some_and(stmt_contains_eval_call)
                || body_contains_eval_call(body)
        }
        StmtKind::Foreach { array, body, .. } => {
            expr_contains_eval_call(array) || body_contains_eval_call(body)
        }
        StmtKind::Switch { subject, cases, default } => {
            expr_contains_eval_call(subject)
                || cases.iter().any(|(patterns, body)| {
                    patterns.iter().any(expr_contains_eval_call) || body_contains_eval_call(body)
                })
                || default.as_ref().is_some_and(|body| body_contains_eval_call(body))
        }
        StmtKind::Include { path, .. } => expr_contains_eval_call(path),
        StmtKind::Synthetic(body)
        | StmtKind::NamespaceBlock { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. } => body_contains_eval_call(body),
        StmtKind::FunctionDecl { params, body, .. } => {
            params
                .iter()
                .any(|(_, _, default, _)| default.as_ref().is_some_and(expr_contains_eval_call))
                || body_contains_eval_call(body)
        }
        StmtKind::ClassDecl { properties, methods, constants, .. }
        | StmtKind::TraitDecl { properties, methods, constants, .. }
        | StmtKind::InterfaceDecl { properties, methods, constants, .. } => {
            properties.iter().any(|property| {
                property.default.as_ref().is_some_and(expr_contains_eval_call)
            }) || constants
                .iter()
                .any(|constant| expr_contains_eval_call(&constant.value))
                || methods.iter().any(|method| {
                    method.params.iter().any(|(_, _, default, _)| {
                        default.as_ref().is_some_and(expr_contains_eval_call)
                    }) || body_contains_eval_call(&method.body)
                })
        }
        StmtKind::Try { try_body, catches, finally_body } => {
            body_contains_eval_call(try_body)
                || catches.iter().any(|catch_clause| body_contains_eval_call(&catch_clause.body))
                || finally_body.as_ref().is_some_and(|body| body_contains_eval_call(body))
        }
        StmtKind::EnumDecl { cases, .. } => cases
            .iter()
            .any(|case| case.value.as_ref().is_some_and(expr_contains_eval_call)),
        StmtKind::RefAssign { .. }
        | StmtKind::Break(_)
        | StmtKind::Continue(_)
        | StmtKind::NamespaceDecl { .. }
        | StmtKind::UseDecl { .. }
        | StmtKind::FunctionVariantGroup { .. }
        | StmtKind::FunctionVariantMark { .. }
        | StmtKind::IncludeOnceMark { .. }
        | StmtKind::Global { .. }
        | StmtKind::PackedClassDecl { .. }
        | StmtKind::ExternFunctionDecl { .. }
        | StmtKind::ExternClassDecl { .. }
        | StmtKind::ExternGlobalDecl { .. } => false,
    }
}

/// Returns true when an expression contains an `eval(...)` call.
pub(super) fn expr_contains_eval_call(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::FunctionCall { name, args } => {
            is_eval_call_name(name) || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::BinaryOp { left, right, .. } => {
            expr_contains_eval_call(left) || expr_contains_eval_call(right)
        }
        ExprKind::InstanceOf { value, target } => {
            expr_contains_eval_call(value) || instance_of_target_contains_eval_call(target)
        }
        ExprKind::Negate(expr)
        | ExprKind::Not(expr)
        | ExprKind::BitNot(expr)
        | ExprKind::Throw(expr)
        | ExprKind::Clone(expr)
        | ExprKind::ErrorSuppress(expr)
        | ExprKind::Print(expr)
        | ExprKind::Spread(expr)
        | ExprKind::Cast { expr, .. }
        | ExprKind::PtrCast { expr, .. }
        | ExprKind::BufferNew { len: expr, .. }
        | ExprKind::ObjectClassName { object: expr }
        | ExprKind::YieldFrom(expr) => expr_contains_eval_call(expr),
        ExprKind::NullCoalesce { value, default }
        | ExprKind::ShortTernary { value, default }
        | ExprKind::Pipe { value, callable: default }
        | ExprKind::ArrayAccess { array: value, index: default } => {
            expr_contains_eval_call(value) || expr_contains_eval_call(default)
        }
        ExprKind::Assignment { target, value, result_target, prelude, .. } => {
            expr_contains_eval_call(target)
                || expr_contains_eval_call(value)
                || result_target.as_ref().is_some_and(|target| expr_contains_eval_call(target))
                || body_contains_eval_call(prelude)
        }
        ExprKind::ArrayLiteral(items) => items.iter().any(expr_contains_eval_call),
        ExprKind::ArrayLiteralAssoc(entries) => entries
            .iter()
            .any(|(key, value)| expr_contains_eval_call(key) || expr_contains_eval_call(value)),
        ExprKind::Match { subject, arms, default } => {
            expr_contains_eval_call(subject)
                || arms.iter().any(|(patterns, value)| {
                    patterns.iter().any(expr_contains_eval_call) || expr_contains_eval_call(value)
                })
                || default.as_ref().is_some_and(|default| expr_contains_eval_call(default))
        }
        ExprKind::Ternary { condition, then_expr, else_expr } => {
            expr_contains_eval_call(condition)
                || expr_contains_eval_call(then_expr)
                || expr_contains_eval_call(else_expr)
        }
        ExprKind::Closure { params, body, .. } => {
            params
                .iter()
                .any(|(_, _, default, _)| default.as_ref().is_some_and(expr_contains_eval_call))
                || body_contains_eval_call(body)
        }
        ExprKind::NamedArg { value, .. } => expr_contains_eval_call(value),
        ExprKind::ClosureCall { args, .. }
        | ExprKind::StaticMethodCall { args, .. }
        | ExprKind::NewObject { args, .. }
        | ExprKind::NewScopedObject { args, .. } => args.iter().any(expr_contains_eval_call),
        ExprKind::ExprCall { callee, args } => {
            expr_contains_eval_call(callee) || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::NewDynamic { name_expr, args } => {
            expr_contains_eval_call(name_expr) || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::NewDynamicObject { class_name, args, .. } => {
            expr_contains_eval_call(class_name) || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::PropertyAccess { object, .. }
        | ExprKind::NullsafePropertyAccess { object, .. } => expr_contains_eval_call(object),
        ExprKind::DynamicPropertyAccess { object, property }
        | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
            expr_contains_eval_call(object) || expr_contains_eval_call(property)
        }
        ExprKind::MethodCall { object, args, .. }
        | ExprKind::NullsafeMethodCall { object, args, .. } => {
            expr_contains_eval_call(object) || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::NullsafeDynamicMethodCall {
            object,
            method,
            args,
        } => {
            expr_contains_eval_call(object)
                || expr_contains_eval_call(method)
                || args.iter().any(expr_contains_eval_call)
        }
        ExprKind::FirstClassCallable(target) => callable_target_contains_eval_call(target),
        ExprKind::Yield { key, value } => {
            key.as_ref().is_some_and(|key| expr_contains_eval_call(key))
                || value.as_ref().is_some_and(|value| expr_contains_eval_call(value))
        }
        ExprKind::IncludeValue { path, .. } => expr_contains_eval_call(path),
        ExprKind::StringLiteral(_)
        | ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::Variable(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::PreIncrement(_)
        | ExprKind::PostIncrement(_)
        | ExprKind::PreDecrement(_)
        | ExprKind::PostDecrement(_)
        | ExprKind::ConstRef(_)
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::This
        | ExprKind::ClassConstant { .. }
        | ExprKind::ScopedConstantAccess { .. }
        | ExprKind::MagicConstant(_) => false,
    }
}

/// Returns true when an `instanceof` target expression contains an `eval(...)` call.
pub(super) fn instance_of_target_contains_eval_call(target: &InstanceOfTarget) -> bool {
    match target {
        InstanceOfTarget::Name(_) => false,
        InstanceOfTarget::Expr(expr) => expr_contains_eval_call(expr),
    }
}

/// Returns true when a first-class callable target contains an `eval(...)` call.
pub(super) fn callable_target_contains_eval_call(target: &CallableTarget) -> bool {
    match target {
        CallableTarget::Function(_) | CallableTarget::StaticMethod { .. } => false,
        CallableTarget::Method { object, .. } => expr_contains_eval_call(object),
    }
}

/// Returns true when a function call name resolves to PHP's `eval` construct.
pub(super) fn is_eval_call_name(name: &Name) -> bool {
    php_symbol_key(name.as_str().trim_start_matches('\\')) == "eval"
}

