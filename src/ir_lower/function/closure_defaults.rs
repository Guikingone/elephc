//! Purpose:
//! Lowers complex lazy closure defaults into ordinary native EIR functions.
//!
//! Called from:
//! - `super::lower_closure_function_with_signature()`.
//!
//! Key details:
//! - Helpers run only for omitted arguments and never require the eval bridge.
//! - Normal EIR evaluation preserves prefix effects, lazy branches, and unwind owners.
//! - Signature defaults remain intact for argument planning and reflection metadata.

use super::*;

/// Builds a zero-argument native helper for each default containing a nested lazy Error.
pub(super) fn lower_helpers(
    parent: &mut LoweringContext<'_, '_>, closure_name: &str, signature: &FunctionSig,
) {
    for (index, default) in signature.defaults.iter().enumerate() {
        let Some(default) = default else { continue; };
        if matches!(default.kind, ExprKind::Throw(_)) || !contains_deferred_error(default) {
            continue;
        }
        let name = format!("{closure_name}_default_{index}");
        let helper_signature = FunctionSig {
            params: Vec::new(), param_type_exprs: Vec::new(), param_attributes: Vec::new(),
            defaults: Vec::new(), return_type: PhpType::Mixed, declared_return: true,
            by_ref_return: false, ref_params: Vec::new(), declared_params: Vec::new(),
            variadic: None, deprecation: None, is_generator: false,
        };
        let body = [Stmt::new(StmtKind::Return(Some(default.clone())), default.span)];
        super::lower_closure_function_with_signature(
            parent, &name, &[], helper_signature, &body, &[], None, name.clone(),
            Some(default.clone()),
        );
    }
}

/// Finds normalized Error nodes in the forms permitted in constant default expressions.
pub(crate) fn contains_deferred_error(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Throw(_) => true,
        ExprKind::ArrayLiteral(values) | ExprKind::NewObject { args: values, .. } =>
            values.iter().any(contains_deferred_error),
        ExprKind::ArrayLiteralAssoc(entries) => entries.iter()
            .any(|(key, value)| contains_deferred_error(key) || contains_deferred_error(value)),
        ExprKind::ArrayLiteralMixed(entries) => entries.iter()
            .any(|entry| entry.exprs().any(contains_deferred_error)),
        ExprKind::BinaryOp { left, right, .. } =>
            contains_deferred_error(left) || contains_deferred_error(right),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } =>
            contains_deferred_error(value) || contains_deferred_error(default),
        ExprKind::Ternary { condition, then_expr, else_expr } =>
            contains_deferred_error(condition) || contains_deferred_error(then_expr)
                || contains_deferred_error(else_expr),
        ExprKind::Match { subject, arms, default } => contains_deferred_error(subject)
            || arms.iter().any(|(conditions, result)|
                conditions.iter().any(contains_deferred_error) || contains_deferred_error(result))
            || default.as_deref().is_some_and(contains_deferred_error),
        ExprKind::Negate(inner) | ExprKind::Not(inner) | ExprKind::BitNot(inner)
        | ExprKind::NamedArg { value: inner, .. } | ExprKind::Cast { expr: inner, .. } =>
            contains_deferred_error(inner),
        _ => false,
    }
}
