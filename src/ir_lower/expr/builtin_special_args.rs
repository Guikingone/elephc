//! Purpose:
//! Special builtin argument normalization and scratch preservation.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Returns true when the call uses exactly one static empty indexed spread.
pub(super) fn is_empty_static_indexed_spread_arg(args: &[Expr]) -> bool {
    let [arg] = args else {
        return false;
    };
    let ExprKind::Spread(inner) = &arg.kind else {
        return false;
    };
    matches!(&inner.kind, ExprKind::ArrayLiteral(items) if items.is_empty())
}

/// Returns true when the callable signature accepts no visible operands.
pub(super) fn zero_arity_call_signature(name: &str, sig: Option<&FunctionSig>) -> bool {
    if let Some(sig) = sig {
        return is_zero_arity_signature(sig);
    }
    builtin_call_signature(name)
        .as_ref()
        .is_some_and(is_zero_arity_signature)
}

/// Returns true when a signature has no regular or variadic parameters.
pub(super) fn is_zero_arity_signature(sig: &FunctionSig) -> bool {
    crate::types::call_args::regular_param_count(sig) == 0 && sig.variadic.is_none()
}

/// Lowers `settype($local, "type")` and updates subsequent local type facts.
pub(super) fn lower_static_settype(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if php_symbol_key(name.trim_start_matches('\\')) != "settype" {
        return None;
    }
    let (var_arg, type_arg) = static_settype_arg_exprs(ctx, name, args)?;
    let ExprKind::Variable(local_name) = &var_arg.kind else {
        return None;
    };
    let target_ty = static_settype_target_type(&type_arg)?;
    let sig = call_signature(ctx, name, source_prefers_extension_builtin(name));
    let operands = lower_builtin_call_args(ctx, name, sig.as_ref(), args);
    let result = emit_builtin_call_value(ctx, name, operands, PhpType::Bool, expr.span, None);
    ctx.set_local_type(local_name, target_ty);
    Some(result)
}

/// Returns canonical `settype()` argument expressions for static local mutation lowering.
pub(super) fn static_settype_arg_exprs(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
    args: &[Expr],
) -> Option<(Expr, Expr)> {
    if args.len() != 2 || args.iter().any(is_spread_arg) {
        return None;
    }
    if !crate::types::call_args::has_named_args(args) {
        return Some((args[0].clone(), args[1].clone()));
    }
    let sig = call_signature(ctx, name, source_prefers_extension_builtin(name))?;
    let call_span = args
        .first()
        .map(|arg| arg.span)
        .unwrap_or_else(crate::span::Span::dummy);
    let regular_param_count = crate::types::call_args::regular_param_count(&sig);
    let plan = crate::types::call_args::plan_call_args_with_regular_param_count_and_assoc_spreads(
        &sig,
        args,
        call_span,
        regular_param_count,
        false,
        true,
        &assoc_spread_sources(ctx, args),
    )
    .ok()?;
    if plan.has_spread_args() || plan.regular_args.len() != 2 {
        return None;
    }
    let var_arg = planned_regular_arg_expr(&plan.regular_args[0])?.clone();
    let type_arg = planned_regular_arg_expr(&plan.regular_args[1])?.clone();
    Some((var_arg, type_arg))
}

/// Returns the source expression assigned to a planned regular parameter.
pub(super) fn planned_regular_arg_expr(
    arg: &crate::types::call_args::PlannedRegularArg,
) -> Option<&Expr> {
    match arg {
        crate::types::call_args::PlannedRegularArg::Source { expr, .. } => Some(expr),
        crate::types::call_args::PlannedRegularArg::Default(_)
        | crate::types::call_args::PlannedRegularArg::SpreadElement { .. } => None,
    }
}

/// Returns the PHP type named by a literal `settype()` second argument.
pub(super) fn static_settype_target_type(arg: &Expr) -> Option<PhpType> {
    let ExprKind::StringLiteral(name) = &arg.kind else {
        return None;
    };
    match php_symbol_key(name).as_str() {
        "int" | "integer" => Some(PhpType::Int),
        "float" | "double" => Some(PhpType::Float),
        "string" => Some(PhpType::Str),
        "bool" | "boolean" => Some(PhpType::Bool),
        _ => None,
    }
}

/// Lowers static function callbacks for `preg_replace_callback()`.
pub(super) fn lower_preg_replace_callback_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    if args.len() != 3 {
        return lower_args_with_signature(ctx, sig, args);
    }
    if matches!(&args[1].kind, ExprKind::Closure { .. }) {
        let pattern = lower_expr(ctx, &args[0]);
        let callback = lower_preg_replace_callback_closure(ctx, &args[1])
            .expect("preg_replace_callback closure check must match lowering");
        let subject = lower_expr(ctx, &args[2]);
        let subject = persist_call_arg_if_string(ctx, subject, args[2].span);
        return vec![pattern.value, callback.value, subject.value];
    }
    let Some(callback) = preg_replace_static_callback(ctx, &args[1]) else {
        return lower_args_with_signature(ctx, sig, args);
    };
    let pattern = lower_expr(ctx, &args[0]);
    let callback = lower_string_literal(ctx, &callback, &args[1]);
    let subject = lower_expr(ctx, &args[2]);
    let subject = persist_call_arg_if_string(ctx, subject, args[2].span);
    vec![pattern.value, callback.value, subject.value]
}

/// Lowers a `preg_replace_callback()` closure with match-array parameter context.
pub(super) fn lower_preg_replace_callback_closure(
    ctx: &mut LoweringContext<'_, '_>,
    callback: &Expr,
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
    } = &callback.kind
    else {
        return None;
    };
    Some(lower_closure_with_context(
        ctx,
        params,
        variadic.as_deref(),
        *variadic_by_ref,
        return_type.as_ref(),
        body,
        captures,
        capture_refs,
        callback,
        &[PhpType::Array(Box::new(PhpType::Str))],
        None,
        *is_static,
    ))
}

/// Returns the userland callback name accepted by the current regex runtime helper.
pub(super) fn preg_replace_static_callback(
    ctx: &LoweringContext<'_, '_>,
    callback: &Expr,
) -> Option<String> {
    match &callback.kind {
        ExprKind::FirstClassCallable(CallableTarget::Function(name)) => {
            Some(name.as_str().to_string())
        }
        ExprKind::Variable(name) => match ctx.static_callable_local(name)? {
            StaticCallableBinding::UserFunction(function_name) => Some(function_name),
            _ => None,
        },
        _ => None,
    }
}

/// Lowers simple positional `date` operands while stabilizing the format string before timestamp evaluation.
pub(super) fn lower_date_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    if args.len() != 2
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return lower_args_with_signature(ctx, sig, args);
    }
    let format = lower_expr(ctx, &args[0]);
    let format = persist_call_arg_if_string(ctx, format, args[0].span);
    vec![format.value, lower_expr(ctx, &args[1]).value]
}

/// Lowers simple positional `json_decode` operands while stabilizing string sources early.
pub(super) fn lower_json_decode_args(
    ctx: &mut LoweringContext<'_, '_>,
    sig: Option<&FunctionSig>,
    args: &[Expr],
) -> Vec<crate::ir::ValueId> {
    if args.is_empty()
        || crate::types::call_args::has_named_args(args)
        || args.iter().any(is_spread_arg)
    {
        return lower_args_with_signature(ctx, sig, args);
    }
    let source = lower_expr(ctx, &args[0]);
    let source = persist_call_arg_if_string(ctx, source, args[0].span);
    let mut operands = Vec::with_capacity(args.len());
    operands.push(source.value);
    for arg in &args[1..] {
        operands.push(lower_expr(ctx, arg).value);
    }
    operands
}

/// Emits `StrPersist` for already-string call operands before later arguments can reuse string scratch storage.
pub(super) fn persist_call_arg_if_string(
    ctx: &mut LoweringContext<'_, '_>,
    source: LoweredValue,
    span: crate::span::Span,
) -> LoweredValue {
    if source.ir_type != IrType::Str {
        return source;
    }
    ctx.emit_value(
        Op::StrPersist,
        vec![source.value],
        None,
        PhpType::Str,
        Op::StrPersist.default_effects(),
        Some(span),
    )
}
