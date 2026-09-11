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
    if let Some(default) = property_null_coalesce_assignment_default(value, object, property, span)
    {
        lower_statement_property_null_coalesce_assignment(ctx, object, property, default, span);
        return;
    }
    // A statically-decided readonly-property write outside the declaring
    // constructor raises a catchable `Error` in PHP rather than a compile-time
    // error, but the object and RHS expressions must still be evaluated first.
    let throw_access_message = ctx
        .throw_access_sites
        .get(&(ctx.loop_storage_scope.clone(), span))
        .and_then(|info| {
        if let ThrowAccessKind::ReadonlyProperty { class_name, property } = &info.kind {
            Some(format!("Cannot modify readonly property {}::${}", class_name, property))
        } else {
            None
        }
    });
    let object = lower_expr(ctx, object);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let value_expr = value;
    let lowered_value = lower_expr(ctx, value_expr);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
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
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, value.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    if let Some(property_ty) = object_property_type(ctx, object.value, property) {
        release_property_assignment_source_after_retaining_store(ctx, &property_ty, value, span);
    }
}

/// Returns the lazy default when this statement originated from `property ??= default`.
///
/// Statement parsing lowers compound assignments into ordinary expressions, so the shared source
/// span is the stable marker that distinguishes `??=` from a user-written
/// `property = property ?? default`. The latter must keep its ordinary assignment semantics.
fn property_null_coalesce_assignment_default<'a>(
    value: &'a Expr,
    object: &Expr,
    property: &str,
    span: Span,
) -> Option<&'a Expr> {
    if value.span != span {
        return None;
    }
    let ExprKind::NullCoalesce {
        value: current,
        default,
    } = &value.kind
    else {
        return None;
    };
    let ExprKind::PropertyAccess {
        object: current_object,
        property: current_property,
    } = &current.kind
    else {
        return None;
    };
    (current_property == property && current_object.as_ref() == object).then_some(default)
}

/// Lowers a statement-level property `??=` while evaluating its receiver exactly once.
///
/// The statement AST historically represents `??=` as a `PropertyAssign` containing a regular
/// `NullCoalesce` expression. Reusing the expression-level conditional lowering keeps the RHS
/// lazy and gives the merge the property's concrete storage type; the synthetic local prevents
/// the read branch and default write from re-evaluating an effectful receiver.
fn lower_statement_property_null_coalesce_assignment(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    default: &Expr,
    span: Span,
) {
    let object_value = lower_expr(ctx, object);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let object_type = ctx.builder.value_php_type(object_value.value).clone();
    let object_temp = ctx.declare_synthetic_php_local(object_type.clone());
    ctx.store_local(&object_temp, object_value, object_type, Some(span));
    let object_expr = Expr::new(ExprKind::Variable(object_temp), span);
    let target = Expr::new(
        ExprKind::PropertyAccess {
            object: Box::new(object_expr),
            property: property.to_string(),
        },
        span,
    );
    let value = Expr::new(
        ExprKind::NullCoalesce {
            value: Box::new(target.clone()),
            default: Box::new(default.clone()),
        },
        span,
    );
    let _ = crate::ir_lower::expr::lower_conditional_non_local_null_coalesce_assignment(
        ctx,
        None,
        &target,
        &value,
        None,
        &value,
    );
}

/// Lowers a direct backing-slot write without invoking magic methods or property set hooks.
pub(in crate::ir_lower) fn lower_raw_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let lowered_value = lower_expr(ctx, value);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let lowered_value = contextualize_property_array_assignment(
        ctx,
        object.value,
        property,
        lowered_value,
        value,
        span,
    );
    let property_ty = object_property_type(ctx, object.value, property);
    let lowered_value = match property_ty {
        Some(ref ty) => coerce_typed_assign_value(ctx, lowered_value, ty, span),
        None => lowered_value,
    };
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, lowered_value.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    if let Some(property_ty) = property_ty {
        release_property_assignment_source_after_retaining_store(
            ctx,
            &property_ty,
            lowered_value,
            span,
        );
    }
}

/// Lowers a declared-property reference bind while preserving the source cell identity.
pub(super) fn lower_property_ref_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    source: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let ExprKind::PropertyAccess {
        object: source_object,
        property: source_property,
    } = &source.kind
    else {
        return;
    };
    let value_type = property_access_expr_type_for_ir(ctx, source_object, source_property)
        .unwrap_or(PhpType::Mixed);
    let source_object = lower_expr(ctx, source_object);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let source_data = ctx.intern_string(source_property);
    let cell_ptr = ctx.emit_value(
        Op::LoadPropRefCell,
        vec![source_object.value],
        Some(Immediate::Data(source_data)),
        value_type.clone(),
        Op::LoadPropRefCell.default_effects(),
        Some(span),
    );
    let target_data = ctx.intern_string(property);
    ctx.emit_void(
        Op::BindPropRefCell,
        vec![object.value, cell_ptr.value],
        Some(Immediate::Data(target_data)),
        Op::BindPropRefCell.default_effects(),
        Some(span),
    );
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
    _value_expr: &Expr,
    span: Span,
) -> LoweredValue {
    let php_type = ctx.builder.value_php_type(lowered.value);
    if !matches!(php_type.codegen_repr(), PhpType::Array(_)) {
        return lowered;
    }
    let Some(contextual_ty) = object_property_type(ctx, object, property) else {
        return lowered;
    };
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
