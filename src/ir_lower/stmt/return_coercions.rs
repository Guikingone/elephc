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
    let value = crate::ir_lower::gradual_coercions::coerce_gradual_value_to_boundary(
        ctx,
        value,
        &return_php_type,
        span,
    );
    if let Some(value) = coerce_container_to_return_type(ctx, value, span) {
        return value;
    }
    if value.ir_type == ctx.return_type {
        return value;
    }
    match ctx.return_type {
        IrType::I64 => coerce_return_scalar_source(ctx, value, span, coerce_to_int),
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

/// Inserts the runtime class guard required when bare `object` crosses a nominal return boundary.
fn guard_generic_object_nominal_return(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    return_type: &PhpType,
    span: Option<Span>,
) -> LoweredValue {
    let PhpType::Object(source_name) = ctx.builder.value_php_type(value.value).codegen_repr() else {
        return value;
    };
    if !source_name.trim_start_matches('\\').is_empty() {
        return value;
    }
    let Some(target_name) = nominal_object_return_target(return_type) else {
        return value;
    };
    let data = ctx.intern_string(&target_name);
    ctx.emit_value(
        Op::RuntimeCall,
        vec![value.value],
        Some(Immediate::Data(data)),
        PhpType::Object(target_name),
        effects_lookup::runtime_effects(),
        span,
    )
}

/// Extracts the single named object accepted by an object or nullable-object return type.
fn nominal_object_return_target(return_type: &PhpType) -> Option<String> {
    match return_type {
        PhpType::Object(name) if !name.trim_start_matches('\\').is_empty() => Some(name.clone()),
        PhpType::Union(members) => {
            let mut target = None;
            for member in members {
                match member {
                    PhpType::Void | PhpType::Never => {}
                    PhpType::Object(name) if !name.trim_start_matches('\\').is_empty() => {
                        if target.as_ref().is_some_and(|existing| existing != name) {
                            return None;
                        }
                        target = Some(name.clone());
                    }
                    _ => return None,
                }
            }
            target
        }
        _ => None,
    }
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
        (PhpType::Array(source_elem), PhpType::AssocArray { .. })
            if source_elem.as_ref() == &PhpType::Never =>
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
