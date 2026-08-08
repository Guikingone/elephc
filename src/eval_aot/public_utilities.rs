//! Purpose:
//! Exposes constant evaluation and static user-function eligibility helpers.
//!
//! Called from:
//! - The eval AOT facade and sibling analysis modules.
//!
//! Key details:
//! - Argument normalization reuses shared call planning and scalar type checks.

use super::*;

/// Evaluates integer-only literal expressions recognized by eval AOT analysis.
pub(crate) fn const_int_expr(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::Negate(inner) => const_int_expr(inner)?.checked_neg(),
        _ => None,
    }
}

/// Evaluates finite numeric literal expressions recognized by eval AOT analysis.
pub(super) fn const_finite_numeric_expr(expr: &Expr) -> Option<f64> {
    const MAX_EXACT_F64_INT: i64 = 9_007_199_254_740_992;
    let value = match &expr.kind {
        ExprKind::IntLiteral(value) if (-MAX_EXACT_F64_INT..=MAX_EXACT_F64_INT).contains(value) => {
            *value as f64
        }
        ExprKind::FloatLiteral(value) => *value,
        ExprKind::Negate(inner) => -const_finite_numeric_expr(inner)?,
        _ => return None,
    };
    value.is_finite().then_some(value)
}

/// Checks a user-function signature against the native-only eval call subset.
pub(crate) fn static_function_signature_supported(signature: &FunctionSig, args: &[Expr]) -> bool {
    if !signature.declared_return
        || signature.declared_params.iter().any(|declared| !declared)
        || signature.ref_params.len() != signature.params.len()
        || signature.variadic.is_some()
        || !static_function_return_type_supported(&signature.return_type)
    {
        return false;
    }
    let Some(args) = normalize_static_function_args(signature, args) else {
        return false;
    };
    signature.params.len() == args.len()
        && signature
            .params
            .iter()
            .zip(signature.ref_params.iter().copied())
            .zip(args.iter())
            .all(|((param, by_ref), arg)| !by_ref && static_function_arg_supported(&param.1, arg))
}

/// Normalizes user-function arguments for eval AOT eligibility checks.
///
/// Static spread arrays are expanded through the shared call planner; dynamic
/// spreads that remain after planning stay on the eval bridge fallback.
pub(super) fn normalize_static_function_args(signature: &FunctionSig, args: &[Expr]) -> Option<Vec<Expr>> {
    if !crate::types::call_args::has_named_args(args)
        && !args
            .iter()
            .any(|arg| matches!(arg.kind, ExprKind::Spread(_)))
    {
        return normalize_positional_static_function_args(signature, args);
    }
    let call_span = args.first().map(|arg| arg.span).unwrap_or_else(Span::dummy);
    let plan = plan_call_args(signature, args, call_span, false, false).ok()?;
    if plan.has_spread_args() {
        return None;
    }
    Some(plan.normalized_args())
}

/// Appends scalar default values for positional static user-function calls.
pub(super) fn normalize_positional_static_function_args(
    signature: &FunctionSig,
    args: &[Expr],
) -> Option<Vec<Expr>> {
    if args.len() > signature.params.len() {
        return None;
    }
    let mut normalized = args.to_vec();
    for idx in args.len()..signature.params.len() {
        let default = signature.defaults.get(idx)?.clone()?;
        normalized.push(default);
    }
    Some(normalized)
}

/// Returns true when a user function return can be boxed by eval EIR AOT.
pub(super) fn static_function_return_type_supported(ty: &PhpType) -> bool {
    matches!(
        ty.codegen_repr(),
        PhpType::Int | PhpType::Bool | PhpType::Float | PhpType::Str
    )
}

/// Returns true when a literal argument matches the supported scalar parameter type.
pub(super) fn static_function_arg_supported(param_ty: &PhpType, arg: &Expr) -> bool {
    matches!(
        (param_ty.codegen_repr(), &arg.kind),
        (PhpType::Int, ExprKind::IntLiteral(_))
            | (PhpType::Bool, ExprKind::BoolLiteral(_))
            | (PhpType::Float, ExprKind::FloatLiteral(_))
            | (PhpType::Str, ExprKind::StringLiteral(_))
    )
}
