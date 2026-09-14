//! Purpose:
//! Instance property writes, magic setters, and hook dispatch.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers an object property write.
pub(super) fn lower_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    // A statically-decided readonly-property write outside the declaring
    // constructor raises a catchable `Error` in PHP rather than a compile-time
    // error, but the object and RHS expressions must still be evaluated first.
    let throw_access_message = ctx.throw_access_sites.get(&span).and_then(|info| {
        if let ThrowAccessKind::ReadonlyProperty { class_name, property } = &info.kind {
            Some(format!("Cannot modify readonly property {}::${}", class_name, property))
        } else {
            None
        }
    });
    let object = lower_expr(ctx, object);
    let value_expr = value;
    let lowered_value = lower_expr(ctx, value_expr);
    if let Some(message) = throw_access_message {
        if ctx.value_is_owning_temporary(object) {
            crate::ir_lower::ownership::release_if_owned(ctx, object, Some(span));
        }
        if ctx.value_is_owning_temporary(lowered_value) {
            crate::ir_lower::ownership::release_if_owned(ctx, lowered_value, Some(span));
        }
        lower_throw_access_error(ctx, &message, span);
        return;
    }
    let value = contextualize_property_array_assignment(
        ctx,
        object.value,
        property,
        lowered_value,
        value_expr,
        span,
    );
    // Property slots use their declared/inferred storage representation. In particular, an
    // untyped property widened to Mixed needs a boxed cell even when this assignment is scalar.
    let property_ty = object_property_type(ctx, object.value, property);
    let value = match property_ty {
        // A runtime-shaped value assigned to a class-typed property stays BOXED here. Unboxing
        // it now would promote whatever the cell holds to an object pointer sight unseen, and
        // a null or an unrelated class would be stored instead of raising PHP's `TypeError`.
        // The backend's weak-mode property guard checks the runtime class and then unboxes.
        Some(ty)
            if declared_property_type(ctx, object.value, property)
                && boxed_value_for_class_typed_property(ctx, &ty, value) =>
        {
            value
        }
        Some(ty) => coerce_typed_assign_value(ctx, value, &ty, span),
        None => value,
    };
    // A packed `int` field accepts a boxed Mixed value only through a strict runtime
    // narrowing (int tag → raw payload, anything else → TypeError). Without it the packed
    // store would write the box POINTER into fixed field storage; with a coercion it would
    // silently truncate the overflow promotion the box exists to carry.
    let value = narrow_mixed_value_for_packed_int_field(ctx, object.value, property, value, span);
    if magic_set_receiver_has_method(ctx, object.value, property) {
        lower_magic_property_set(ctx, object.value, property, value, span);
        return;
    }
    // Route a write to a set-hooked property to its `__propset_<p>($value)` accessor, except inside
    // that property's own accessor where `$this->prop = v` must write the raw backing slot.
    if set_hook_receiver_has_accessor(ctx, object.value, property)
        && !ctx.in_own_property_accessor(property)
    {
        lower_property_hook_set(ctx, object.value, property, value, span);
        return;
    }
    let data = ctx.intern_string(property);
    // A declared property store carries MAY_THROW: the weak typed-property guard raises PHP's
    // catchable TypeError, and a Stringable receiver can throw out of its own __toString. Both
    // jump to __rt_throw_current with the assigned temporary still pure SSA, so pin it in the
    // unwind chain for the store and retire the pin without releasing once the store returns.
    //
    // Only a store that keeps an INDEPENDENT owner is pinned, which is exactly the condition
    // `release_property_assignment_source_after_retaining_store` releases under. A store that
    // instead transfers the temporary into the slot owes no release afterwards, and pinning it
    // would leave one reference nothing retires.
    let stored_property_ty = object_property_type(ctx, object.value, property).unwrap_or(PhpType::Mixed);
    let pins = if property_store_keeps_independent_ref(
        &stored_property_ty,
        &ctx.builder.value_php_type(value.value),
    ) {
        crate::ir_lower::expr::pin_in_flight_owners(ctx, &[value.value], span)
    } else {
        Vec::new()
    };
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, value.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    crate::ir_lower::expr::unpin_in_flight_owners(ctx, pins, span);
    // Undeclared dynamic properties store boxed Mixed values. Boxing retains a
    // concrete temporary payload just like a declared property store does.
    let property_ty = object_property_type(ctx, object.value, property).unwrap_or(PhpType::Mixed);
    release_property_assignment_source_after_retaining_store(ctx, &property_ty, value, span);
}

/// Narrows a boxed Mixed value assigned to a packed `int` field into its raw `I64` payload.
///
/// Emits `Op::PackedFieldMixedToInt` (int tag passes, every other runtime tag throws a
/// catchable `TypeError` naming the runtime type) and releases the source box right after:
/// the payload is a raw copy, so the box's lifetime ends at the narrowing, not at the store.
/// Non-packed receivers, non-Mixed values, and non-int fields pass through untouched.
fn narrow_mixed_value_for_packed_int_field(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    value: LoweredValue,
    span: Span,
) -> LoweredValue {
    let PhpType::Packed(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return value;
    };
    if !matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed
    ) {
        return value;
    }
    let normalized = class_name.trim_start_matches('\\');
    let Some(field_ty) = ctx
        .packed_classes
        .get(normalized)
        .and_then(|info| info.fields.iter().find(|field| field.name == property))
        .map(|field| field.php_type.codegen_repr())
    else {
        return value;
    };
    if field_ty != PhpType::Int {
        return value;
    }
    let message = format!(
        "Packed field {}::${} must be of type int, ",
        normalized, property
    );
    let data = ctx.intern_string(&message);
    let narrowed = ctx.emit_value(
        Op::PackedFieldMixedToInt,
        vec![value.value],
        Some(Immediate::Data(data)),
        PhpType::Int,
        Op::PackedFieldMixedToInt.default_effects(),
        Some(span),
    );
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
    narrowed
}

/// Returns true when a property write should dispatch to `__set`.
pub(super) fn magic_set_receiver_has_method(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return false;
    };
    let normalized = class_name.trim_start_matches('\\');
    let Some(class_info) = ctx.classes.get(normalized) else {
        return false;
    };
    if class_info
        .properties
        .iter()
        .any(|(name, _)| name == property)
    {
        return false;
    }
    class_info.methods.contains_key(&php_symbol_key("__set"))
}

/// Lowers an undeclared property write to a normal `__set` instance-method call.
pub(super) fn lower_magic_property_set(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    value: LoweredValue,
    span: Span,
) {
    let property_data = ctx.intern_string(property);
    let property_name = ctx.emit_value(
        Op::ConstStr,
        Vec::new(),
        Some(Immediate::Data(property_data)),
        PhpType::Str,
        Op::ConstStr.default_effects(),
        Some(span),
    );
    let method_data = ctx.intern_string("__set");
    ctx.emit_void(
        Op::MethodCall,
        vec![object, property_name.value, value.value],
        Some(Immediate::Data(method_data)),
        Op::MethodCall.default_effects(),
        Some(span),
    );
    release_magic_set_value_after_call(ctx, value, span);
}

/// Releases an owning RHS temporary after the `__set` call has consumed it.
pub(super) fn release_magic_set_value_after_call(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Span,
) {
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Returns true when the runtime class of `object` declares a `__propset_<property>` set-hook
/// accessor, meaning a write to `property` should be routed through it.
pub(super) fn set_hook_receiver_has_accessor(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return false;
    };
    let normalized = class_name.trim_start_matches('\\');
    ctx.classes.get(normalized).is_some_and(|info| {
        info.methods
            .contains_key(&php_symbol_key(&property_hook_set_method(property)))
    })
}

/// Lowers a write to a set-hooked property as a call to its `__propset_<p>($value)` accessor,
/// passing the assigned value as the single argument and releasing it if it was an owning temporary.
pub(super) fn lower_property_hook_set(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    value: LoweredValue,
    span: Span,
) {
    let method_data = ctx.intern_string(&property_hook_set_method(property));
    ctx.emit_void(
        Op::MethodCall,
        vec![object, value.value],
        Some(Immediate::Data(method_data)),
        Op::MethodCall.default_effects(),
        Some(span),
    );
    release_magic_set_value_after_call(ctx, value, span);
}

/// Converts array literals to hash storage when a declared object property requires assoc storage.
pub(super) fn contextualize_property_array_assignment(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    lowered: LoweredValue,
    value_expr: &Expr,
    span: Span,
) -> LoweredValue {
    let Some(contextual_ty) = object_property_type(ctx, object, property) else {
        return lowered;
    };
    contextualize_property_array_value(ctx, lowered, value_expr, &contextual_ty, span)
}

/// Consumes a fresh indexed literal when the physical property requires associative storage.
pub(in crate::ir_lower) fn contextualize_property_array_value(
    ctx: &mut LoweringContext<'_, '_>,
    lowered: LoweredValue,
    value_expr: &Expr,
    contextual_ty: &PhpType,
    span: Span,
) -> LoweredValue {
    let php_type = ctx.builder.value_php_type(lowered.value);
    if !matches!(value_expr.kind, ExprKind::ArrayLiteral(_)) {
        return lowered;
    }
    if !matches!(php_type.codegen_repr(), PhpType::Array(_)) {
        return lowered;
    }
    let contextual_ty = contextual_ty.codegen_repr();
    if !matches!(contextual_ty, PhpType::AssocArray { .. }) {
        return lowered;
    }
    ctx.emit_value(
        Op::ArrayToHash,
        vec![lowered.value],
        None,
        contextual_ty,
        Op::ArrayToHash.default_effects(),
        Some(span),
    )
}

/// Returns true when a boxed value is assigned to a class-typed property slot.
fn boxed_value_for_class_typed_property(
    ctx: &LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
) -> bool {
    matches!(property_ty.codegen_repr(), PhpType::Object(_))
        && ctx.builder.value_php_type(value.value).codegen_repr() == PhpType::Mixed
}

/// Returns true when the receiver's class declares a PHP type for this property.
///
/// An untyped property has no weak-mode property typing to enforce, so its writes keep the
/// previous unboxing lowering.
fn declared_property_type(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object).codegen_repr() else {
        return false;
    };
    ctx.classes
        .get(class_name.trim_start_matches('\\'))
        .is_some_and(|class_info| class_info.visible_property_is_declared(property))
}
