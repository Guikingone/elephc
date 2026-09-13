//! Purpose:
//! Function return storage coercion and container widening.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Coerces a value to the current function return storage type when needed.
pub(super) fn coerce_to_return_type(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    let return_php_type = ctx.return_php_type.clone();
    let value = guard_generic_object_nominal_return(ctx, value, &return_php_type, span);
    let source_declared_type = ctx.builder.value_php_type(value.value).clone();
    let value = match crate::types::param_binding::scalar_param_cast(
        &return_php_type,
        &source_declared_type,
    ) {
        Some(cast) => crate::ir_lower::expr::apply_scalar_param_cast(
            ctx,
            cast,
            value,
            &return_php_type,
            span,
        ),
        None => value,
    };
    let value = crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
        ctx,
        value,
        &return_php_type,
        span,
    );
    if matches!(return_php_type.codegen_repr(), PhpType::Callable)
        && matches!(
            ctx.builder.value_php_type(value.value).codegen_repr(),
            PhpType::Str | PhpType::Array(_)
        )
    {
        return coerce_to_callable_return(ctx, value, span);
    }
    if let Some(value) = coerce_container_to_return_type(ctx, value, span) {
        return value;
    }
    if value.ir_type == ctx.return_type {
        return value;
    }
    match ctx.return_type {
        IrType::I64 => {
            if let Some(verified) = declared_int_return_boundary(ctx, value, span) {
                return verified;
            }
            coerce_return_scalar_source(ctx, value, span, coerce_to_int)
        }
        IrType::F64 => coerce_return_scalar_source(ctx, value, span, coerce_to_float),
        IrType::Str => coerce_return_scalar_source(ctx, value, span, coerce_to_string),
        IrType::TaggedScalar => {
            coerce_return_scalar_source(ctx, value, span, coerce_to_tagged_scalar)
        }
        IrType::Heap(_) if ctx.return_php_type.codegen_repr() == PhpType::Mixed => {
            ctx.box_value_as_mixed(value, ctx.return_php_type.clone(), span)
        }
        IrType::Heap(_) => ctx.emit_value(
            Op::RuntimeCall,
            vec![value.value],
            None,
            ctx.return_php_type.clone(),
            effects_lookup::runtime_effects(),
            span,
        ),
        IrType::Void => value,
    }
}

/// Resolves a runtime string or receiver/method pair to callable descriptor storage.
fn coerce_to_callable_return(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    let coerced = ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        None,
        PhpType::Callable,
        effects_lookup::runtime_effects(),
        span,
    );
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    coerced
}

/// Verifies a dynamically-typed value against a DECLARED `int` return boundary.
///
/// A declared boundary carries PHP's coercive-mode verification: an in-range float
/// truncates, but a float outside the int range, a non-numeric string, null, or any
/// container throws `TypeError` — where the plain `coerce_to_int` path silently wrapped
/// the value. Applies only when the source type still needs a runtime decision (boxed
/// Mixed, or a raw F64 left by constant folding); statically-int sources keep the direct
/// path, and INFERRED int returns are untouched — an undeclared function must never throw
/// on its own return value.
fn declared_int_return_boundary(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> Option<LoweredValue> {
    if !ctx.return_type_is_declared || ctx.return_php_type != PhpType::Int {
        return None;
    }
    if !matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Float
    ) {
        return None;
    }
    let prefix = format!("{}(): Return value must be of type int, ", ctx.owner_name());
    let data = ctx.intern_string(&prefix);
    let verified = ctx.emit_value(
        Op::ReturnBoundaryMixedToInt,
        vec![value.value],
        Some(Immediate::Data(data)),
        PhpType::Int,
        Op::ReturnBoundaryMixedToInt.default_effects(),
        span,
    );
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    Some(verified)
}

/// Inserts the runtime class guard required when an object crosses a narrower nominal boundary.
fn guard_generic_object_nominal_return(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    return_type: &PhpType,
    span: Option<Span>,
) -> LoweredValue {
    let source_type = ctx.builder.value_php_type(value.value).codegen_repr();
    let needs_object_guard = matches!(source_type, PhpType::Object(_))
        || crate::types::param_binding::gradual_object_requires_runtime_nominal_guard(
            return_type,
            &source_type,
        );
    if !needs_object_guard {
        return value;
    }
    let Some(target_name) =
        crate::types::param_binding::nominal_object_boundary_target(return_type)
    else {
        return value;
    };
    if matches!(source_type, PhpType::Object(_))
        && crate::ir_lower::expr::param_accepts_object_without_string_coercion(
        ctx,
        return_type,
        &source_type,
    ) {
        return value;
    }
    let data = ctx.intern_class_name(&target_name);
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        Some(Immediate::NominalObject {
            target: data,
            boundary: crate::ir::NominalObjectBoundary::Return,
        }),
        return_type.clone(),
        effects_lookup::runtime_effects(),
        span,
    )
}


/// Coerces a return value and releases the old owning temporary when replaced.
pub(super) fn coerce_return_scalar_source(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
    coerce: fn(&mut LoweringContext<'_, '_>, LoweredValue, Option<Span>) -> LoweredValue,
) -> LoweredValue {
    let coerced = coerce(ctx, value, span);
    if coerced.value != value.value && ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    coerced
}

/// Coerces an integer-or-null value into the two-word tagged-scalar return shape.
pub(super) fn coerce_to_tagged_scalar(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    if value.ir_type == IrType::TaggedScalar {
        return value;
    }
    if matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Void
    ) {
        return ctx.emit_value(
            Op::ConstNull,
            Vec::new(),
            None,
            PhpType::TaggedScalar,
            Op::ConstNull.default_effects(),
            span,
        );
    }
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        None,
        PhpType::TaggedScalar,
        effects_lookup::runtime_effects(),
        span,
    )
}

/// Widens returned container payload storage to the current function return contract.
pub(super) fn coerce_container_to_return_type(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> Option<LoweredValue> {
    let source_ty = ctx.builder.value_php_type(value.value).codegen_repr();
    let return_ty = ctx.return_php_type.codegen_repr();
    if let (
        PhpType::AssocArray {
            key,
            value: source_value,
        },
        PhpType::Array(return_element),
    ) = (&source_ty, &return_ty)
    {
        if return_element.codegen_repr() == PhpType::Mixed {
            // PHP's bare `array` return contract accepts both indexed and associative layouts.
            // Widen concrete hash slots first, then only retype the same runtime container as
            // the generic array contract; converting the hash itself to indexed storage would
            // discard string keys and change iteration order.
            let source = if source_value.codegen_repr() == PhpType::Mixed {
                value
            } else {
                ctx.emit_value(
                    Op::HashToMixed,
                    vec![value.value],
                    None,
                    PhpType::AssocArray {
                        key: key.clone(),
                        value: Box::new(PhpType::Mixed),
                    },
                    Op::HashToMixed.default_effects(),
                    span,
                )
            };
            return Some(ctx.emit_value(
                Op::Move,
                vec![source.value],
                None,
                return_ty,
                Op::Move.default_effects(),
                span,
            ));
        }
    }
    // A bare `array` contract resolves to the HASH whenever some return path yields one
    // (`Checker::generic_array_return_contract`), so an INDEXED return on another path must be
    // CONVERTED here rather than restamped. `Op::ArrayToHash` carries the element type across
    // unchanged, so a packed vector of raw slots is boxed first when the contract stores Mixed.
    if let (
        PhpType::Array(source_elem),
        PhpType::AssocArray {
            value: return_value,
            ..
        },
    ) = (&source_ty, &return_ty)
    {
        let source_elem = source_elem.codegen_repr();
        if source_elem != PhpType::Never
            && source_elem != return_value.codegen_repr()
            && return_value.codegen_repr() == PhpType::Mixed
        {
            let boxed = ctx.emit_value(
                Op::ArrayToMixed,
                vec![value.value],
                None,
                PhpType::Array(Box::new(PhpType::Mixed)),
                Op::ArrayToMixed.default_effects(),
                span,
            );
            return Some(ctx.emit_value(
                Op::ArrayToHash,
                vec![boxed.value],
                None,
                return_ty,
                Op::ArrayToHash.default_effects(),
                span,
            ));
        }
    }
    let op = match (source_ty, return_ty.clone()) {
        (PhpType::Array(source_elem), PhpType::Array(return_elem))
            if source_elem.codegen_repr() != PhpType::Mixed
                && return_elem.codegen_repr() == PhpType::Mixed =>
        {
            Op::ArrayToMixed
        }
        (
            PhpType::AssocArray {
                value: source_value,
                ..
            },
            PhpType::AssocArray {
                value: return_value,
                ..
            },
        ) if source_value.codegen_repr() != PhpType::Mixed
            && return_value.codegen_repr() == PhpType::Mixed =>
        {
            Op::HashToMixed
        }
        (
            PhpType::Array(source_elem),
            PhpType::AssocArray {
                value: return_value,
                ..
            },
        ) if source_elem.as_ref() == &PhpType::Never
            || source_elem.codegen_repr() == return_value.codegen_repr() =>
        {
            Op::ArrayToHash
        }
        _ => return None,
    };
    Some(ctx.emit_value(
        op,
        vec![value.value],
        None,
        return_ty,
        op.default_effects(),
        span,
    ))
}

/// Coerces a value to integer storage.
pub(super) fn coerce_to_int(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::I64 => value,
        IrType::F64 => ctx.emit_value(
            Op::FToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::FToI.default_effects(),
            span,
        ),
        IrType::Str => ctx.emit_value(
            Op::StrToI,
            vec![value.value],
            None,
            PhpType::Int,
            Op::StrToI.default_effects(),
            span,
        ),
        _ => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::I64)),
            PhpType::Int,
            Op::Cast.default_effects(),
            span,
        ),
    }
}

/// Coerces a value to float storage.
pub(super) fn coerce_to_float(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match value.ir_type {
        IrType::F64 => value,
        IrType::I64 => ctx.emit_value(
            Op::IToF,
            vec![value.value],
            None,
            PhpType::Float,
            Op::IToF.default_effects(),
            span,
        ),
        IrType::Str => ctx.emit_value(
            Op::StrToF,
            vec![value.value],
            None,
            PhpType::Float,
            Op::StrToF.default_effects(),
            span,
        ),
        _ => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::F64)),
            PhpType::Float,
            Op::Cast.default_effects(),
            span,
        ),
    }
}

/// Coerces a value to string storage.
pub(super) fn coerce_to_string(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    // Mixed uses one integer-sized boxed-cell register regardless of the PHP payload. Preserve
    // the semantic representation at a declared string return boundary instead of rendering the
    // cell pointer as an integer string.
    if matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    ) {
        return ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::Str)),
            PhpType::Str,
            Op::Cast.default_effects(),
            span,
        );
    }
    match value.ir_type {
        IrType::Str => value,
        IrType::I64 | IrType::TaggedScalar => ctx.emit_value(
            Op::IToStr,
            vec![value.value],
            None,
            PhpType::Str,
            Op::IToStr.default_effects(),
            span,
        ),
        IrType::F64 => ctx.emit_value(
            Op::FToStr,
            vec![value.value],
            None,
            PhpType::Str,
            Op::FToStr.default_effects(),
            span,
        ),
        _ => ctx.emit_value(
            Op::Cast,
            vec![value.value],
            Some(Immediate::CastTarget(IrType::Str)),
            PhpType::Str,
            Op::Cast.default_effects(),
            span,
        ),
    }
}
