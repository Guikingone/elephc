//! Purpose:
//! Instance property array mutations and retaining-store cleanup.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers `$object->prop[] = value`.
pub(super) fn lower_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    let initialize_uninitialized = is_concrete_object_receiver(ctx, object.value);
    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property) {
        initialize_uninitialized_array_property_for_write(
            ctx,
            object.value,
            property,
            &property_ty,
            initialize_uninitialized,
            span,
        );
    }
    let generic_receiver = is_generic_object_receiver(ctx, object.value);
    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::ArrayPush,
            vec![property_value.value, value.value],
            None,
            Op::ArrayPush.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(
            ctx,
            &property_ty,
            property_value,
            span,
        );
        return;
    }

    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_assoc_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(
            ctx,
            &property_ty,
            property_value,
            span,
        );
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| property_type_uses_mixed_array_storage(ty))
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![property_value.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
        if generic_receiver {
            ctx.emit_void(
                Op::PropSet,
                vec![object.value, property_value.value],
                Some(Immediate::Data(data)),
                Op::PropSet.default_effects(),
                Some(span),
            );
            release_rewritten_property_value_after_retaining_store(
                ctx,
                &property_ty,
                property_value,
                span,
            );
        }
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty,
            Op::PropGet.default_effects(),
            Some(span),
        );
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if generic_receiver {
        lower_generic_object_mixed_property_array_write(
            ctx, &object, property, None, value, span,
        );
        return;
    }

    let value = lower_expr(ctx, value);
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![object.value, value.value],
        Some(Immediate::Data(data)),
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Returns whether a declared array-like property is represented by one boxed Mixed cell.
fn property_type_uses_mixed_array_storage(property_ty: &PhpType) -> bool {
    match property_ty.codegen_repr() {
        PhpType::Mixed => true,
        PhpType::Union(members) => {
            let mut saw_array = false;
            for member in members {
                match member.codegen_repr() {
                    PhpType::Array(_) | PhpType::AssocArray { .. } => saw_array = true,
                    PhpType::Void | PhpType::Never => {}
                    _ => return false,
                }
            }
            saw_array
        }
        _ => false,
    }
}

/// Lowers `$object->prop[index] = value`.
pub(super) fn lower_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    let initialize_uninitialized = is_concrete_object_receiver(ctx, object.value);
    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property) {
        initialize_uninitialized_array_property_for_write(
            ctx,
            object.value,
            property,
            &property_ty,
            initialize_uninitialized,
            span,
        );
    }
    let generic_receiver = is_generic_object_receiver(ctx, object.value);
    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        let value = coerce_indexed_array_set_value(ctx, &property_ty, value, Some(span));
        if index.ir_type == IrType::Str {
            let assoc_ty = promoted_assoc_array_type(
                property_ty.clone(),
                ctx.builder.value_php_type(value.value),
            );
            let hash = ctx.emit_value(
                Op::ArrayToHash,
                vec![property_value.value],
                None,
                assoc_ty.clone(),
                Op::ArrayToHash.default_effects(),
                Some(span),
            );
            ctx.emit_void(
                Op::HashSet,
                vec![hash.value, index.value, value.value],
                None,
                Op::HashSet.default_effects(),
                Some(span),
            );
            release_persisted_string_operand(ctx, index, span);
            release_persisted_string_operand(ctx, value, span);
            ctx.emit_void(
                Op::PropSet,
                vec![object.value, hash.value],
                Some(Immediate::Data(data)),
                Op::PropSet.default_effects(),
                Some(span),
            );
            release_rewritten_property_value_after_retaining_store(
                ctx, &assoc_ty, hash, span,
            );
            return;
        }
        if property_ty.codegen_repr() == PhpType::Array(Box::new(PhpType::Mixed))
            && index_is_boxed_mixed_key(index.ir_type)
        {
            let rewritten = ctx.emit_value(
                Op::ArraySetMixedKey,
                vec![property_value.value, index.value, value.value],
                None,
                property_ty.clone(),
                Op::ArraySetMixedKey.default_effects(),
                Some(span),
            );
            ctx.emit_void(
                Op::PropSet,
                vec![object.value, rewritten.value],
                Some(Immediate::Data(data)),
                Op::PropSet.default_effects(),
                Some(span),
            );
            release_rewritten_property_value_after_retaining_store(
                ctx,
                &property_ty,
                rewritten,
                span,
            );
            return;
        }
        ctx.emit_void(
            Op::ArraySet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(
            ctx,
            &property_ty,
            property_value,
            span,
        );
        return;
    }
    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_assoc_array_type)
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(
            ctx,
            &property_ty,
            property_value,
            span,
        );
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty,
            Op::PropGet.default_effects(),
            Some(span),
        );
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| property_type_uses_mixed_array_storage(ty))
    {
        let data = ctx.intern_string(property);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        let property_value =
            crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
        // Same store-time index rule as every other element write: share the helper so this
        // path cannot answer differently for the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        ctx.emit_void(
            Op::PropSet,
            vec![object.value, property_value.value],
            Some(Immediate::Data(data)),
            Op::PropSet.default_effects(),
            Some(span),
        );
        release_rewritten_property_value_after_retaining_store(
            ctx,
            &property_ty,
            property_value,
            span,
        );
        return;
    }

    if generic_receiver {
        lower_generic_object_mixed_property_array_write(
            ctx,
            &object,
            property,
            Some(index),
            value,
            span,
        );
        return;
    }

    // PHP reads a plain-variable index at STORE time, after the right-hand side, so
    // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
    // rule; sharing the helper is what keeps the two from answering differently for
    // the same source line.
    let (index, value) =
        crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
    let data = ctx.intern_string(property);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![object.value, index.value, value.value],
        Some(Immediate::Data(data)),
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Mutates one runtime-typed property through generic-object class-id dispatch and republishes it.
fn lower_generic_object_mixed_property_array_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: &LoweredValue,
    property: &str,
    index: Option<&Expr>,
    value: &Expr,
    span: Span,
) {
    let data = ctx.intern_string(property);
    let property_ty = PhpType::Mixed;
    let property_value = ctx.emit_value(
        Op::PropGet,
        vec![object.value],
        Some(Immediate::Data(data)),
        property_ty.clone(),
        Op::PropGet.default_effects(),
        Some(span),
    );
    let property_value =
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
    let index = index.map(|index| lower_expr(ctx, index));
    let value = lower_expr(ctx, value);
    if let Some(index) = index {
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
    } else {
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![property_value.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
    }
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, property_value.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    release_rewritten_property_value_after_retaining_store(
        ctx,
        &property_ty,
        property_value,
        span,
    );
}

/// Autovivifies an uninitialized declared array property before mutating one of its elements.
pub(super) fn initialize_uninitialized_array_property_for_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    property_ty: &PhpType,
    initialize_uninitialized: bool,
    span: Span,
) {
    if !initialize_uninitialized {
        return;
    }
    // The slot's REPRESENTATION decides how the empty container is stored, and a nullable
    // array property does not represent as an array: `?array` is a union, and every union
    // reaches codegen as `Mixed`. Keying only on the representation therefore skipped
    // auto-initialization entirely for `?array $p`, and `$o->p['k'] = 1` raised where PHP
    // initializes — measured against `php -n` 8.5.6, which treats `array` and `?array`
    // identically here. The DECLARED type says which container to make; the representation
    // says whether it has to be boxed on the way into the slot.
    let Some(container_ty) = auto_initialized_container_type(property_ty) else {
        return;
    };
    let op = match container_ty.codegen_repr() {
        PhpType::Array(_) => Op::ArrayNew,
        PhpType::AssocArray { .. } => Op::HashNew,
        _ => return,
    };
    let slot_is_boxed = matches!(property_ty.codegen_repr(), PhpType::Mixed);
    let data = ctx.intern_string(property);
    let initialized = ctx.emit_value(
        Op::PropInitialized,
        vec![object],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(span),
    );
    let initialize = ctx
        .builder
        .create_named_block("property_array.initialize", Vec::new());
    let ready = ctx
        .builder
        .create_named_block("property_array.ready", Vec::new());
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: ready,
        then_args: Vec::new(),
        else_target: initialize,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(initialize);
    let empty = ctx.emit_value(
        op,
        Vec::new(),
        Some(Immediate::Capacity(0)),
        container_ty.clone(),
        op.default_effects(),
        Some(span),
    );
    // A raw array word stored into a `Mixed` slot is the representation mismatch this codebase
    // keeps rediscovering, so a boxed slot gets a boxed value.
    let (empty, stored_ty) = if slot_is_boxed {
        (
            ctx.box_value_as_mixed(empty, PhpType::Mixed, Some(span)),
            PhpType::Mixed,
        )
    } else {
        (empty, container_ty.clone())
    };
    ctx.emit_void(
        Op::PropSet,
        vec![object, empty.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    release_property_assignment_source_after_retaining_store(ctx, &stored_ty, empty, span);
    branch_to(ctx, ready);

    ctx.builder.position_at_end(ready);
}

/// Returns the container type PHP auto-initializes an indexed property write into.
///
/// An `array`/`array<T>` slot makes that array. A UNION containing one — `?array`,
/// `array|null`, `array|string` — makes the array member, because PHP auto-initializes on the
/// array side of the union rather than refusing. A union with no array member, and every
/// non-array type, has no auto-initialization: PHP raises
/// `Cannot auto-initialize an array inside property C::$p of type int` there instead.
fn auto_initialized_container_type(property_ty: &PhpType) -> Option<PhpType> {
    match property_ty {
        PhpType::Array(_) | PhpType::AssocArray { .. } => Some(property_ty.clone()),
        PhpType::Union(members) => members
            .iter()
            .find(|member| matches!(member, PhpType::Array(_) | PhpType::AssocArray { .. }))
            .cloned(),
        _ => match property_ty.codegen_repr() {
            PhpType::Array(_) | PhpType::AssocArray { .. } => Some(property_ty.codegen_repr()),
            _ => None,
        },
    }
}

/// Returns whether an EIR receiver carries PHP's bare `object` pseudo-type.
fn is_generic_object_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
) -> bool {
    matches!(
        ctx.builder.value_php_type(object),
        PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty()
    )
}

/// Returns whether the receiver names one concrete class with native property slots.
pub(super) fn is_concrete_object_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
) -> bool {
    matches!(
        ctx.builder.value_php_type(object).codegen_repr(),
        PhpType::Object(class_name) if !class_name.trim_start_matches('\\').is_empty()
    )
}

/// Releases a temporary assigned into an object property after `PropSet` retains or boxes it.
pub(super) fn release_property_assignment_source_after_retaining_store(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
    span: Span,
) {
    if !ctx.value_is_owning_temporary(value) {
        return;
    }
    if !property_store_keeps_independent_ref(property_ty, &ctx.builder.value_php_type(value.value))
    {
        return;
    }
    crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
}

/// Releases an element temporary after a property-array write retains it for storage.
pub(super) fn release_property_array_insert_value_after_retain(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
    span: Span,
) {
    let Some(elem_ty) = indexed_property_array_element_type(property_ty) else {
        return;
    };
    if matches!(elem_ty.codegen_repr(), PhpType::Mixed | PhpType::Callable) {
        return;
    }
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases the loaded property value after rewriting it through a retaining `PropSet`.
pub(in crate::ir_lower) fn release_rewritten_property_value_after_retaining_store(
    ctx: &mut LoweringContext<'_, '_>,
    property_ty: &PhpType,
    property_value: LoweredValue,
    span: Span,
) {
    if property_ty.codegen_repr().is_refcounted() {
        crate::ir_lower::ownership::release_if_owned(ctx, property_value, Some(span));
    }
}

/// Returns whether a property store creates a distinct retained/boxed owner for the value.
pub(super) fn property_store_keeps_independent_ref(property_ty: &PhpType, value_ty: &PhpType) -> bool {
    let property_ty = property_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    if matches!((&property_ty, &value_ty), (PhpType::Mixed, PhpType::Mixed)) {
        return false;
    }
    if matches!(value_ty, PhpType::Mixed | PhpType::Union(_))
        && matches!(property_ty, PhpType::Int | PhpType::Bool | PhpType::Float)
    {
        return true;
    }
    if matches!(property_ty, PhpType::Str) {
        return true;
    }
    property_ty.is_refcounted()
}

/// Returns the element type for property arrays that use retaining indexed/hash helpers.
pub(super) fn indexed_property_array_element_type(property_ty: &PhpType) -> Option<PhpType> {
    match property_ty.codegen_repr() {
        PhpType::Array(elem_ty) => Some(elem_ty.codegen_repr()),
        PhpType::AssocArray { value, .. } => Some(value.codegen_repr()),
        _ => None,
    }
}
