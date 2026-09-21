//! Purpose:
//! Static property writes and static array mutations.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Returns the value type stored by an array/hash container used for reference binding.
fn reference_element_type(container: &PhpType) -> PhpType {
    match container.codegen_repr() {
        PhpType::AssocArray { value, .. } => (*value).clone(),
        PhpType::Array(element) => (*element).clone(),
        _ => PhpType::Mixed,
    }
}

/// Lowers a static property write.
///
/// A static property outlives the enclosing scope, so it must hold its own
/// reference to a refcounted value. There are two storage disciplines, matched
/// to what the codegen store actually does:
///
/// - **Boxing store** (a Mixed/Union slot receiving a non-Mixed value, e.g.
///   `Class::$h = new C()`): codegen boxes the value with `__rt_mixed_from_value`,
///   which takes its *own* retained reference to the child. The slot therefore
///   keeps a reference independent of the source, so an owning temporary must be
///   *released* after the store (its reference is not the one the slot holds), and
///   a borrowed source must be left untouched. Acquiring here would leak the extra
///   reference on top of the box's retained one.
/// - **Moving store** (every other case: concrete-typed slot, or a Mixed→Mixed
///   move): the store consumes (moves) its value operand. An owning temporary is
///   moved in as-is, but a *borrowed* value (a parameter, local, or container read)
///   must be `Acquire`d first. Without this, storing a borrowed `Mixed`
///   (e.g. `Class::$h = $handler` where `$handler` is a `?SessionHandlerInterface`
///   parameter) leaves the property dangling once the borrow's owner releases its
///   reference, so a later read dispatches on freed memory (a fatal "on null").
///
///   A heap load from a local slot only *looks* like an owning temporary. Main
///   classifies concrete array/object local loads that way for provisional
///   unbox-release tracking, but the slot stays the owner and the frame epilogue
///   releases it, so moving from one hands the property a reference the epilogue
///   then takes back. `throw $e` reached the same conclusion first (issue #448):
///   publishing a pointer somewhere that outlives the frame is not a transfer.
///   A static property outlives it by the widest margin there is — the process —
///   so a local-slot load is acquired here like any other borrow.
pub(super) fn lower_static_property_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let mut value = lower_expr(ctx, value);
    if matches!(ctx.builder.value_php_type(value.value).codegen_repr(), PhpType::Array(_))
        && static_property_type(ctx, receiver, property)
            .is_some_and(|property_ty| matches!(property_ty.codegen_repr(), PhpType::AssocArray { .. }))
    {
        let property_ty = static_property_type(ctx, receiver, property)
            .expect("checked associative static property metadata must remain available");
        value = ctx.emit_value(
            Op::ArrayToHash,
            vec![value.value],
            None,
            property_ty,
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
    }
    if static_property_store_retains_independent_value(ctx, receiver, property, value) {
        store_static_property(ctx, receiver, property, value.value, span);
        if ctx.value_is_owning_temporary(value) {
            crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
        }
        return;
    }
    let transferable =
        ctx.value_is_owning_temporary(value) && !ctx.value_is_owned_unboxed_local_load(value.value);
    let value = if transferable {
        value
    } else {
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, value, Some(span))
    };
    store_static_property(ctx, receiver, property, value.value, span);
}

/// Returns true when codegen gives the static-property slot an independently retained value.
///
/// This covers both concrete values boxed into Mixed/Union slots and boxed Mixed values
/// unboxed into object slots. Both backend paths retain the stored child independently,
/// so borrowed sources need no `Acquire` and owning temporary sources are released after
/// the store. Unknown metadata conservatively keeps the moving-store discipline.
pub(super) fn static_property_store_retains_independent_value(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: LoweredValue,
) -> bool {
    let Some(slot_ty) = static_property_type(ctx, receiver, property) else {
        return false;
    };
    let value_ty = ctx.builder.value_php_type(value.value);
    let slot_ty = slot_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    let boxes_into_mixed = matches!(slot_ty, PhpType::Mixed | PhpType::Union(_))
        && !matches!(value_ty, PhpType::Mixed | PhpType::Union(_));
    let unboxes_into_object = matches!(slot_ty, PhpType::Object(_))
        && matches!(value_ty, PhpType::Mixed | PhpType::Union(_));
    boxes_into_mixed || unboxes_into_object
}

/// Lowers `Class::$prop[] = value`.
pub(super) fn lower_static_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    value: &Expr,
    span: Span,
) {
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::ArrayPush,
            vec![property_value.value, value.value],
            None,
            Op::ArrayPush.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let property_value = load_static_property(ctx, receiver, property, span);
    let value = lower_expr(ctx, value);
    if static_property_may_be_eval_dynamic(ctx, receiver) {
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![property_value.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `Class::$prop[index] = value`.
pub(super) fn lower_static_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_indexed_array_type)
    {
        let array_ty = property_ty.clone();
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        let value = coerce_indexed_array_set_value(ctx, &array_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    // A reference binding can widen a typed `array` static property to associative Mixed
    // storage after earlier writes have already been parsed. Lower those string/mixed-key
    // writes through the real hash mutation op and publish a relocated table back to the
    // static slot; a generic runtime-call placeholder would silently discard the write.
    if let Some(property_ty) =
        static_property_type(ctx, receiver, property).filter(is_assoc_array_type)
    {
        let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            None,
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, index, span);
        release_persisted_string_operand(ctx, value, span);
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }

    let property_value = if let Some(property_ty) = static_property_type(ctx, receiver, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        load_static_property_as(ctx, receiver, property, property_ty, span)
    } else {
        load_static_property(ctx, receiver, property, span)
    };
    // PHP reads a plain-variable index at STORE time, after the right-hand side, so
    // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
    // rule; sharing the helper is what keeps the two from answering differently for
    // the same source line.
    let (index, value) =
        crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
    if static_property_may_be_eval_dynamic(ctx, receiver) {
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        store_static_property(ctx, receiver, property, property_value.value, span);
        return;
    }
    ctx.emit_void(
        Op::RuntimeCall,
        vec![property_value.value, index.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers `self::$a[$target] = &self::$a[$source]` through one shared managed cell.
pub(super) fn lower_static_property_element_ref_assign(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    target_index: &Expr,
    source: &Expr,
    span: Span,
) {
    let ExprKind::ArrayAccess {
        index: source_index,
        ..
    } = &source.kind
    else {
        lower_expr(ctx, source);
        return;
    };

    let hash_ty = static_property_type(ctx, receiver, property).unwrap_or(PhpType::Mixed);
    let mut hash = load_static_property_as(ctx, receiver, property, hash_ty, span);
    if let IrType::Heap(crate::ir::IrHeapKind::Array) = hash.ir_type {
        let current_ty = ctx.builder.value_php_type(hash.value);
        let element_ty = reference_element_type(&current_ty);
        let assoc_ty = promoted_assoc_array_type(current_ty, element_ty);
        hash = ctx.emit_value(
            Op::ArrayToHash,
            vec![hash.value],
            None,
            assoc_ty,
            Op::ArrayToHash.default_effects(),
            Some(span),
        );
    }

    let element_ty = reference_element_type(&ctx.builder.value_php_type(hash.value));
    let source_key = lower_expr(ctx, source_index);
    let cell = ctx.emit_value(
        Op::HashRefElement,
        vec![hash.value, source_key.value],
        None,
        element_ty,
        Op::HashRefElement.default_effects(),
        Some(span),
    );
    let hash_ty_after = ctx.builder.value_php_type(hash.value);
    let target_key = lower_expr(ctx, target_index);
    let bound_hash = ctx.emit_value(
        Op::HashBindRefElement,
        vec![hash.value, target_key.value, cell.value],
        None,
        hash_ty_after,
        Op::HashBindRefElement.default_effects(),
        Some(span),
    );
    store_static_property(ctx, receiver, property, bound_hash.value, span);
}

/// Lowers a direct, indexed, or append write through a runtime static-property name.
pub(super) fn lower_dynamic_static_property_write(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &Expr,
    index: Option<&Expr>,
    append: bool,
    value: &Expr,
    span: Span,
) {
    let name = lower_expr(ctx, property);
    let name = coerce_to_string_at_span(ctx, name, Some(property.span)).value;
    let common_ty = dynamic_static_property_common_type(ctx, receiver);

    if append {
        if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
            let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
            let value = lower_expr(ctx, value);
            ctx.emit_void(
                Op::ArrayPush,
                vec![array.value, value.value],
                None,
                Op::ArrayPush.default_effects(),
                Some(span),
            );
            store_dynamic_static_property(ctx, receiver, name, array.value, span);
            return;
        }
        let array_ty = common_ty
            .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
            .unwrap_or(PhpType::Mixed);
        let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let value = lower_expr(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![array.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        return;
    }

    if let Some(index) = index {
        lower_dynamic_static_array_element_set(
            ctx, receiver, name, common_ty, index, value, span,
        );
        return;
    }

    let value = lower_expr(ctx, value);
    store_dynamic_static_property(ctx, receiver, name, value.value, span);
}

/// Lowers an indexed write through a runtime static-property name.
fn lower_dynamic_static_array_element_set(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    common_ty: Option<PhpType>,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    if let Some(array_ty) = common_ty.clone().filter(is_indexed_array_type) {
        let element_array_ty = array_ty.clone();
        let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
        let index = lower_expr(ctx, index);
        let value = lower_expr(ctx, value);
        let value = coerce_indexed_array_set_value(ctx, &element_array_ty, value, Some(span));
        ctx.emit_void(
            Op::ArraySet,
            vec![array.value, index.value, value.value],
            None,
            Op::ArraySet.default_effects(),
            Some(span),
        );
        store_dynamic_static_property(ctx, receiver, name, array.value, span);
        return;
    }

    let array_ty = common_ty
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
        .unwrap_or(PhpType::Mixed);
    let array = load_dynamic_static_property_as(ctx, receiver, name, array_ty, span);
    let index = lower_expr(ctx, index);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![array.value, index.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Loads a runtime-named static property using a precomputed string value.
fn load_dynamic_static_property_as(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    php_type: PhpType,
    span: Span,
) -> LoweredValue {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_value(
        Op::LoadDynamicStaticProperty,
        vec![name],
        Some(Immediate::Data(data)),
        php_type,
        Op::LoadDynamicStaticProperty.default_effects(),
        Some(span),
    )
}

/// Stores through a runtime static-property name using a precomputed string value.
fn store_dynamic_static_property(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    name: crate::ir::ValueId,
    value: crate::ir::ValueId,
    span: Span,
) {
    let class_name =
        static_receiver_class_name(ctx, receiver).unwrap_or_else(|| receiver_name(receiver));
    let data = ctx.intern_string(&class_name);
    ctx.emit_void(
        Op::StoreDynamicStaticProperty,
        vec![name, value],
        Some(Immediate::Data(data)),
        Op::StoreDynamicStaticProperty.default_effects(),
        Some(span),
    );
}

/// Returns the common codegen representation of a class's static properties.
fn dynamic_static_property_common_type(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> Option<PhpType> {
    let class_name = static_receiver_class_name(ctx, receiver)?;
    let class_info = ctx.classes.get(class_name.as_str())?;
    let mut types = class_info
        .static_properties
        .iter()
        .map(|(_, ty)| normalize_value_php_type(ty.codegen_repr()));
    let first = types.next()?;
    if types.all(|ty| ty == first) {
        Some(first)
    } else {
        Some(PhpType::Mixed)
    }
}

/// Returns true when a named static-property receiver may resolve through eval metadata.
pub(super) fn static_property_may_be_eval_dynamic(
    ctx: &LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
) -> bool {
    let StaticReceiver::Named(class_name) = receiver else {
        return false;
    };
    ctx.has_eval_barrier()
        && !ctx
            .classes
            .contains_key(class_name.as_str().trim_start_matches('\\'))
}
