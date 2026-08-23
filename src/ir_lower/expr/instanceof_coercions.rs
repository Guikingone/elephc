//! Purpose:
//! Instanceof and scalar coercion helpers.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers `instanceof`.
pub(super) fn lower_instanceof(
    ctx: &mut LoweringContext<'_, '_>,
    value: &Expr,
    target: &InstanceOfTarget,
    expr: &Expr,
) -> LoweredValue {
    let value = lower_expr(ctx, value);
    if statically_known_instanceof_result(ctx, expr) == Some(false) {
        if ctx.value_is_owning_temporary(value) {
            crate::ir_lower::ownership::release_if_owned(ctx, value, Some(expr.span));
        }
        return lower_bool_literal(ctx, false, expr);
    }
    let mut operands = vec![value.value];
    let immediate = match target {
        InstanceOfTarget::Name(name) => {
            if name.as_str().trim_start_matches('\\') == "static" && ctx.local_slots.contains_key("this") {
                operands.push(ctx.load_local("this", Some(expr.span)).value);
                None
            } else {
                Some(Immediate::Data(ctx.intern_class_name(&instanceof_target_name(ctx, name.as_str()))))
            }
        }
        InstanceOfTarget::Expr(expr) => {
            operands.push(lower_expr(ctx, expr).value);
            None
        }
    };
    let op = if immediate.is_some() { Op::InstanceOf } else { Op::InstanceOfDynamic };
    ctx.emit_value(op, operands, immediate, PhpType::Bool, op.default_effects(), Some(expr.span))
}

/// Returns the closed-world result of a named `instanceof` whose target has no emitted metadata.
///
/// The left-hand value must still be evaluated by the caller. An earlier `eval()` disables the
/// proof because runtime code may have declared the target class or interface. Negation is folded
/// recursively so branch lowering can omit the unreachable arm without relying on IR passes.
pub(in crate::ir_lower) fn statically_known_instanceof_result(
    ctx: &LoweringContext<'_, '_>,
    expr: &Expr,
) -> Option<bool> {
    if let ExprKind::Not(inner) = &expr.kind {
        return statically_known_instanceof_result(ctx, inner).map(|value| !value);
    }
    let ExprKind::InstanceOf {
        target: InstanceOfTarget::Name(name),
        ..
    } = &expr.kind
    else {
        return None;
    };
    if ctx.eval_executed() {
        return None;
    }
    let target = instanceof_target_name(ctx, name.as_str());
    if target
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("Closure")
    {
        // `Closure` is a PHP intrinsic represented by a callable descriptor rather than a
        // normal entry in the AOT class metadata registry. The backend has a dedicated tag-10
        // check, so it must see this expression even when no type declaration referenced
        // `Closure` while building the registry.
        return None;
    }
    let known = ctx.classes.contains_key(&target)
        || ctx.interfaces.contains_key(&target)
        || ctx.enums.contains_key(&target);
    (!known).then_some(false)
}

/// Resolves lexical `instanceof` target keywords to concrete class names when possible.
fn instanceof_target_name(
    ctx: &LoweringContext<'_, '_>,
    name: &str,
) -> String {
    match name.trim_start_matches('\\') {
        "self" => ctx.current_class.clone().unwrap_or_else(|| name.to_string()),
        "parent" => ctx
            .current_class
            .as_deref()
            .and_then(|class_name| ctx.classes.get(class_name))
            .and_then(|class_info| class_info.parent.clone())
            .unwrap_or_else(|| name.to_string()),
        _ => name.to_string(),
    }
}

/// Returns the positive nominal local-type fact proven by one static `instanceof` branch.
pub(in crate::ir_lower) fn instanceof_branch_local_type(
    ctx: &LoweringContext<'_, '_>,
    condition: &Expr,
    branch_matches: bool,
) -> Option<(String, PhpType)> {
    let (name, target) = positive_instanceof_local(condition, branch_matches)?;
    let class_name = instanceof_target_name(ctx, target);
    if class_name.trim_start_matches('\\') == "static" {
        return None;
    }
    Some((name.to_string(), PhpType::Object(class_name)))
}

/// Extracts a local and named target when the selected branch proves an `instanceof` condition.
fn positive_instanceof_local(condition: &Expr, branch_matches: bool) -> Option<(&str, &str)> {
    if let ExprKind::Not(inner) = &condition.kind {
        return positive_instanceof_local(inner, !branch_matches);
    }
    if !branch_matches {
        return None;
    }
    let ExprKind::InstanceOf {
        value,
        target: InstanceOfTarget::Name(target),
    } = &condition.kind
    else {
        return None;
    };
    let name = match &value.kind {
        ExprKind::Variable(name) => name.as_str(),
        ExprKind::This => "this",
        _ => return None,
    };
    Some((name, target.as_str()))
}

/// Coerces a value to integer storage before integer-only operations.
pub(super) fn coerce_to_int(ctx: &mut LoweringContext<'_, '_>, value: LoweredValue, expr: &Expr) -> LoweredValue {
    coerce_to_int_at_span(ctx, value, Some(expr.span))
}

/// Coerces a value to integer storage using an explicit source span.
pub(crate) fn coerce_to_int_at_span(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::I64
            if matches!(
                ctx.builder.value_php_type(value.value).codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            ) =>
        {
            ctx.emit_value(
                Op::Move,
                vec![value.value],
                None,
                PhpType::Int,
                Op::Move.default_effects(),
                span,
            )
        }
        IrType::I64 => value,
        IrType::F64 => ctx.emit_value(Op::FToI, vec![value.value], None, PhpType::Int, Op::FToI.default_effects(), span),
        IrType::Str => ctx.emit_value(Op::StrToI, vec![value.value], None, PhpType::Int, Op::StrToI.default_effects(), span),
        _ => {
            let result = ctx.emit_value(
                Op::Cast,
                vec![value.value],
                Some(Immediate::CastTarget(IrType::I64)),
                PhpType::Int,
                Op::Cast.default_effects(),
                span,
            );
            // The cast lowers to `__rt_mixed_cast_int`, which returns a raw
            // scalar that never aliases the source box. Dropping the owning
            // reference here leaked one checked-arithmetic Mixed cell per
            // evaluation for `%`, bitops, comparisons, and coerced array
            // indexes with a compound operand (issue #500).
            release_coerced_source_if_owned(ctx, value, span);
            result
        }
    }
}

/// Coerces a value to float when the storage type allows a direct conversion.
pub(super) fn coerce_to_float(ctx: &mut LoweringContext<'_, '_>, value: LoweredValue, expr: &Expr) -> LoweredValue {
    coerce_to_float_at_span(ctx, value, Some(expr.span))
}

/// Coerces a value to float storage using an explicit source span.
pub(super) fn coerce_to_float_at_span(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::F64 => value,
        IrType::I64 => ctx.emit_value(Op::IToF, vec![value.value], None, PhpType::Float, Op::IToF.default_effects(), span),
        _ => {
            let result = ctx.emit_value(
                Op::Cast,
                vec![value.value],
                Some(Immediate::CastTarget(IrType::F64)),
                PhpType::Float,
                Op::Cast.default_effects(),
                span,
            );
            // Mirror of the int coercion above: `__rt_mixed_cast_float`
            // returns a raw scalar, so the owning source box (e.g. a checked
            // `pow` operand, issue #500) must be released here.
            release_coerced_source_if_owned(ctx, value, span);
            result
        }
    }
}

/// Coerces a value to string when possible.
pub(super) fn coerce_to_string(ctx: &mut LoweringContext<'_, '_>, value: LoweredValue, expr: &Expr) -> LoweredValue {
    coerce_to_string_at_span(ctx, value, Some(expr.span))
}

/// Coerces a value to string storage using an explicit source span.
pub(in crate::ir_lower) fn coerce_to_string_at_span(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<crate::span::Span>,
) -> LoweredValue {
    if matches!(ctx.builder.value_php_type(value.value), PhpType::Resource(_)) {
        return ctx.emit_value(
            Op::ResourceToStr,
            vec![value.value],
            None,
            PhpType::Str,
            Op::ResourceToStr.default_effects(),
            span,
        );
    }
    match value.ir_type {
        IrType::Str => value,
        IrType::I64 | IrType::TaggedScalar => ctx.emit_value(Op::IToStr, vec![value.value], None, PhpType::Str, Op::IToStr.default_effects(), span),
        IrType::F64 => ctx.emit_value(Op::FToStr, vec![value.value], None, PhpType::Str, Op::FToStr.default_effects(), span),
        _ => {
            let result = ctx.emit_value(
                Op::Cast,
                vec![value.value],
                Some(Immediate::CastTarget(IrType::Str)),
                PhpType::Str,
                Op::Cast.default_effects(),
                span,
            );
            release_coerced_source_if_owned(ctx, value, span);
            result
        }
    }
}
