//! Purpose:
//! Instance property array mutations and retaining-store cleanup.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Returns the one owner of a property container the write below mutates and stores back.
///
/// A Mixed property read is already an owning temporary (codegen retains the box it returns),
/// and acquiring it again gave the write TWO owners while the cleanup after the store releases
/// one: `$this->data[$k] = $v` on an untyped property left the box one reference high per write,
/// so the whole array outlived its object. A borrowed read (a typed container slot) is acquired.
fn own_property_container(
    ctx: &mut LoweringContext<'_, '_>,
    property_value: LoweredValue,
    span: Span,
) -> LoweredValue {
    if ctx.value_is_owning_temporary(property_value) {
        return property_value;
    }
    crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span))
}

/// Releases the receiver of a property element write once the write is done.
///
/// A chained receiver (`$ast->selectClause->selectExpressions[$k] = $v`) is a property read the
/// lowering acquired, and none of the write paths gave that reference back: the intermediate
/// object -- and everything it held -- outlived its owner by one per write. A borrowed receiver
/// (`$this`, a local) is not an owning temporary and is left alone.
fn release_property_write_receiver(ctx: &mut LoweringContext<'_, '_>, object: LoweredValue, span: Span) {
    if ctx.value_is_owning_temporary(object) {
        crate::ir_lower::ownership::release_if_owned(ctx, object, Some(span));
    }
}

/// Lowers `$object->prop[] = value`.
pub(super) fn lower_property_array_push(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    value: &Expr,
    span: Span,
) {
    let object = lower_expr(ctx, object);
    let object = interface_receiver_as_generic_object(ctx, object, span);
    lower_property_array_push_on(ctx, object, property, value, span);
    release_property_write_receiver(ctx, object, span);
}

/// Retypes a receiver known only by an interface as a plain object: an interface declares no
/// property, so a write through it (`$metadata->table['indexes'] = []` over Doctrine's
/// `Persistence\Mapping\ClassMetadata`) takes the run-time resolved path an untyped object takes.
fn interface_receiver_as_generic_object(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    span: Span,
) -> LoweredValue {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object.value) else {
        return object;
    };
    let name = class_name.trim_start_matches('\\');
    if name.is_empty() || ctx.classes.contains_key(name) || !ctx.interfaces.contains_key(name) {
        return object;
    }
    ctx.emit_value(
        Op::Move,
        vec![object.value],
        None,
        PhpType::Object(String::new()),
        Op::Move.default_effects(),
        Some(span),
    )
}

/// Lowers `$object->prop[] = value` on an already lowered receiver.
fn lower_property_array_push_on(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    value: &Expr,
    span: Span,
) {
    if object_property_type(ctx, object.value, property).is_some_and(|ty| ty.is_php_array()) {
        lower_php_array_property_write(ctx, object, property, None, value, span, false);
        return;
    }
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
    let generic_receiver = is_generic_object_receiver(ctx, object.value, property);
    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        // The value is evaluated BEFORE the container is fetched (PHP's delayed dim fetch), so
        // a right-hand side that writes this same property is not lost to a stale copy.
        let value = lower_expr(ctx, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
        let property_value =
            own_property_container(ctx, property_value, span);
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
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        // The value is evaluated BEFORE the container is fetched (PHP's delayed dim fetch), so
        // a right-hand side that writes this same property is not lost to a stale copy.
        let value = lower_expr(ctx, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
        let property_value =
            own_property_container(ctx, property_value, span);
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
        .filter(|ty| {
            // A nullable ArrayAccess object (`?ArrayObject`, `?WeakMap`) boxes like a Mixed
            // property, but appending to it is `offsetSet(null, $v)` on the OBJECT; the Mixed
            // array append wrote into a detached copy and the push vanished.
            property_type_uses_mixed_array_storage(ty) && !type_satisfies_array_access_for_ir(ctx, ty)
        })
    {
        let data = ctx.intern_string(property);
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        let value = lower_expr(ctx, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
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
        } else if ctx.value_is_owning_temporary(property_value) {
            // The append mutated the property's own box through this read, which owns one
            // reference to it (a Mixed property read retains); without the release every
            // `$this->items[] = $v` left the array one reference high and it outlived its object.
            crate::ir_lower::ownership::release_if_owned(ctx, property_value, Some(span));
        }
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        let data = ctx.intern_string(property);
        let value = lower_expr(ctx, value);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty,
            Op::PropGet.default_effects(),
            Some(span),
        );
        // A nullable receiver is a boxed `?Class`; the push dispatch keys on the class itself.
        let receiver_ty = ctx.builder.value_php_type(property_value.value);
        let operands = if matches!(receiver_ty, PhpType::Union(_) | PhpType::Mixed) {
            // The class is only known at run time, so the native `append()` shortcut the typed
            // form takes is unavailable: dispatch the generic `offsetSet(null, $v)`.
            let null_key = ctx
                .builder
                .emit_with_effects(
                    Op::ConstNull,
                    Vec::new(),
                    None,
                    IrType::I64,
                    PhpType::Void,
                    Ownership::NonHeap,
                    Op::ConstNull.default_effects(),
                    Some(span),
                )
                .expect("const_null produces a value");
            vec![property_value.value, null_key, value.value]
        } else {
            super::array_write_storage::object_push_operands(
                ctx,
                &receiver_ty,
                property_value,
                value,
                span,
            )
        };
        ctx.emit_void(
            Op::RuntimeCall,
            operands,
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        // `offsetSet()`/`append()` borrow the receiver and the value, as in the indexed write.
        if ctx.value_is_owning_temporary(property_value) {
            crate::ir_lower::ownership::release_if_owned(ctx, property_value, Some(span));
        }
        super::array_write_core::release_object_offset_set_operand(ctx, value, span);
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
///
/// A desugared `??=` becomes a probe plus a conditional insert, and a desugared compound update
/// writes with the key its own read already converted (see `ElementUpdate`).
pub(super) fn lower_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let update = desugared_element_update(value, span, |read| {
        reads_property_array_element(read, object, property, index)
    });
    if let Some(ElementUpdate::NullCoalesce { read, default }) = update {
        crate::ir_lower::expr::lower_null_coalesce_update_stmt(ctx, read, default, span);
        return;
    }
    if lower_string_property_offset_write(ctx, object, property, index, value, span) {
        return;
    }
    lower_property_array_assign_with_diagnosed_key(
        ctx,
        object,
        property,
        index,
        value,
        span,
        update.is_some(),
    );
}

/// Lowers `$this->types[$i] = 'i'` on a `string` property as php's byte-offset write: the string
/// is read into a hidden local, written through the local string-offset path, and stored back.
///
/// Only a receiver that is evaluated without effects (`$this`, a variable) is replayed; returns
/// `false`, leaving the generic lowering, for any other shape or property type.
fn lower_string_property_offset_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
) -> bool {
    if !matches!(object.kind, ExprKind::This | ExprKind::Variable(_)) {
        return false;
    }
    let property_ty =
        crate::ir_lower::expr::property_access_expr_type_for_ir(ctx, object, property);
    if property_ty.map(|ty| ty.codegen_repr()) != Some(PhpType::Str) {
        return false;
    }
    let temp = ctx.declare_synthetic_php_local(PhpType::Str);
    let property_read = Expr::new(
        ExprKind::PropertyAccess { object: Box::new(object.clone()), property: property.to_string() },
        span,
    );
    let statements = [
        StmtKind::Assign { name: temp.clone(), value: property_read },
        StmtKind::ArrayAssign { array: temp.clone(), index: index.clone(), value: value.clone() },
        StmtKind::PropertyAssign {
            object: Box::new(object.clone()),
            property: property.to_string(),
            value: Expr::new(ExprKind::Variable(temp), span),
        },
    ];
    for kind in statements {
        crate::ir_lower::stmt::lower_stmt(ctx, &crate::parser::ast::Stmt::new(kind, span));
        if ctx.builder.insertion_block_is_terminated() {
            break;
        }
    }
    true
}

/// Returns whether `read` is the element `$object->property[index]` a statement writes.
fn reads_property_array_element(read: &Expr, object: &Expr, property: &str, index: &Expr) -> bool {
    matches!(
        &read.kind,
        ExprKind::ArrayAccess { array, index: read_index }
            if read_index.as_ref() == index
                && matches!(
                    &array.kind,
                    ExprKind::PropertyAccess { object: read_object, property: read_property }
                        if read_object.as_ref() == object && read_property == property
                )
    )
}

/// Lowers `$object->prop[index] = value`, optionally reusing a float-key diagnosis.
///
/// `key_already_diagnosed` marks the write half of a compound update: its read of the same
/// element already reported the float key's conversion, as PHP does once per update. An
/// `ArrayAccess` property receives the key unconverted, and a runtime-typed receiver keeps its
/// own conversion, so neither consults the flag.
pub(crate) fn lower_property_array_assign_with_diagnosed_key(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    let object = lower_expr(ctx, object);
    let object = interface_receiver_as_generic_object(ctx, object, span);
    if lower_nullable_receiver_property_array_assign(
        ctx,
        object,
        property,
        index,
        value,
        span,
        key_already_diagnosed,
    ) {
        // That path releases the boxed receiver on both of its edges.
        return;
    }
    lower_property_array_assign_on(ctx, object, property, index, value, span, key_already_diagnosed);
    release_property_write_receiver(ctx, object, span);
}

/// Lowers `$object->prop[index] = value` on an already lowered, non-nullable receiver.
fn lower_property_array_assign_on(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    if object_property_type(ctx, object.value, property).is_some_and(|ty| ty.is_php_array()) {
        lower_php_array_property_write(
            ctx,
            object,
            property,
            Some(index),
            value,
            span,
            key_already_diagnosed,
        );
        return;
    }
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
    let generic_receiver = is_generic_object_receiver(ctx, object.value, property);
    if let Some(property_ty) =
        generic_object_array_property_type(ctx, object.value, property).filter(is_indexed_array_type)
    {
        let data = ctx.intern_string(property);
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        // Key and value are evaluated BEFORE the container is fetched (PHP's delayed dim
        // fetch), so a right-hand side that writes this same property keeps its writes.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
        let property_value =
            own_property_container(ctx, property_value, span);
        let index =
            coerce_array_key_to_int_at_span(ctx, index, Some(span), key_already_diagnosed);
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
            // The helper stores its own reference to the value, exactly as on the local path
            // (`array_write_core::lower_mixed_key_array_set`): a boxed temporary and a string
            // the box copied are still this write's to drop.
            if matches!(
                ctx.builder.value_php_type(value.value).codegen_repr(),
                PhpType::Mixed | PhpType::Union(_)
            ) && ctx.value_is_owning_temporary(value)
            {
                crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
            }
            release_string_copied_into_box(ctx, value, span);
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
        if property_array_write_mutates_in_place(ctx, object.value, property, generic_receiver) {
            lower_in_place_property_hash_set(
                ctx,
                object.value,
                property,
                &property_ty,
                index,
                value,
                span,
            );
            return;
        }
        let data = ctx.intern_string(property);
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        // Key and value are evaluated BEFORE the container is fetched (PHP's delayed dim
        // fetch), so a right-hand side that writes this same property keeps its writes.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
        let property_value =
            own_property_container(ctx, property_value, span);
        ctx.emit_void(
            Op::HashSet,
            vec![property_value.value, index.value, value.value],
            key_already_diagnosed.then_some(Immediate::Bool(true)),
            Op::HashSet.default_effects(),
            Some(span),
        );
        release_property_array_insert_value_after_retain(ctx, &property_ty, value, span);
        // That helper releases only a value stored into CONCRETE element slots. A hash stores a
        // string by copying it, as on the promotion path above, so a fresh string key or value
        // is still this write's to drop: `$this->a[$k] = (string) $v` leaked the cast's copy on
        // every call.
        let helper_released_value = indexed_property_array_element_type(&property_ty)
            .is_some_and(|element| {
                !matches!(element.codegen_repr(), PhpType::Mixed | PhpType::Callable)
            });
        release_persisted_string_operand(ctx, index, span);
        if !helper_released_value {
            release_persisted_string_operand(ctx, value, span);
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
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| type_satisfies_array_access_for_ir(ctx, ty))
    {
        let data = ctx.intern_string(property);
        // PHP reads a plain-variable index at STORE time, after the right-hand side, so
        // `$o->a[$i] = ($i = 1)` writes index 1. The bare-local write already used this
        // rule; sharing the helper is what keeps the two from answering differently for
        // the same source line.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty,
            Op::PropGet.default_effects(),
            Some(span),
        );
        ctx.emit_void(
            Op::RuntimeCall,
            vec![property_value.value, index.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        // `offsetSet()` borrows every operand: the receiver read (an OWNED unbox for a nullable
        // or union-typed property such as `?WeakMap`), a non-string key and the value are all
        // this write's to drop. Only the declared-object receiver was a plain borrow, so the
        // others leaked the WeakMap's cell on every `$this->map[$k] = v`.
        if ctx.value_is_owning_temporary(property_value) {
            crate::ir_lower::ownership::release_if_owned(ctx, property_value, Some(span));
        }
        if matches!(ctx.builder.value_php_type(index.value).codegen_repr(), PhpType::Str) {
            release_persisted_string_operand(ctx, index, span);
        } else {
            super::array_write_core::release_object_offset_set_operand(ctx, index, span);
        }
        super::array_write_core::release_object_offset_set_operand(ctx, value, span);
        return;
    }

    if let Some(property_ty) = generic_object_array_property_type(ctx, object.value, property)
        .filter(|ty| property_type_uses_mixed_array_storage(ty))
    {
        let data = ctx.intern_string(property);
        // The CONTAINER of an indexed write or unset is fetched quietly, the same as the
        // container of an `isset`/`empty` operand. PHP does not raise for an uninitialized
        // typed property here either: an indexed write auto-initializes it and an `unset` of
        // one of its elements is a no-op that leaves it uninitialized. Over an EVAL-OWNED
        // receiver this fetch leaves compiled code through the bridge, which raises unless the
        // mode is on -- `unset($this->p[$id])` and `$this->p[$id] = true` on the same property
        // as the `empty()` above are what still stopped the Symfony request once the operand
        // form was fixed.
        // Same store-time index rule as every other element write: share the helper so this
        // path cannot answer differently for the same source line.
        // Key and value are evaluated BEFORE the container is fetched (PHP's delayed dim
        // fetch), so a right-hand side that writes this same property keeps its writes.
        let (index, value) =
            crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
        let property_value = ctx.emit_value(
            Op::PropGet,
            vec![object.value],
            Some(Immediate::Data(data)),
            property_ty.clone(),
            Op::PropGet.default_effects(),
            Some(span),
        );
        crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
        let property_value =
            own_property_container(ctx, property_value, span);
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
/// Lowers `$o->p[$k] = $v` over a `?C` receiver: a null one throws php's `Attempt to modify
/// property "p" on null`, any other is unboxed to `C` and written through the typed path.
///
/// doctrine/orm's `AbstractHydrator` writes `$this->rsm->newObject[$i]` through its
/// `ResultSetMapping|null $rsm`. The boxed receiver otherwise took the generic Mixed write, which
/// releases neither the receiver read nor the key and value (26 blocks per three writes).
/// Returns false, emitting nothing, for any other receiver type.
fn lower_nullable_receiver_property_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    index: &Expr,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) -> bool {
    let object_ty = ctx.builder.value_php_type(object.value);
    let PhpType::Union(members) = &object_ty else {
        return false;
    };
    let class_name = match members.as_slice() {
        [PhpType::Object(name), PhpType::Void] | [PhpType::Void, PhpType::Object(name)] => {
            name.clone()
        }
        _ => return false,
    };
    let narrowed_ty = PhpType::Object(class_name);
    let null_block = ctx.builder.create_named_block("propwrite.null", Vec::new());
    let write_block = ctx.builder.create_named_block("propwrite.non_null", Vec::new());
    let merge = ctx.builder.create_named_block("propwrite.merge", Vec::new());
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![object.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: null_block,
        then_args: Vec::new(),
        else_target: write_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(null_block);
    crate::ir_lower::ownership::release_if_owned(ctx, object, Some(span));
    crate::ir_lower::stmt::lower_throw_access_error_expr(
        ctx,
        &format!("Attempt to modify property \"{}\" on null", property),
        span,
    );
    branch_to(ctx, merge);

    ctx.builder.position_at_end(write_block);
    // The unbox takes its own reference; parking it in a PHP local hands that reference to the
    // slot, and the write below then sees an ordinary typed local receiver.
    let narrowed = ctx.emit_value(
        Op::MixedUnbox,
        vec![object.value],
        None,
        narrowed_ty.clone(),
        Op::MixedUnbox.default_effects(),
        Some(span),
    );
    crate::ir_lower::ownership::release_if_owned(ctx, object, Some(span));
    let receiver_temp = ctx.declare_synthetic_php_local(narrowed_ty.clone());
    ctx.store_local(&receiver_temp, narrowed, narrowed_ty.clone(), Some(span));
    let receiver = Expr::new(ExprKind::Variable(receiver_temp.clone()), span);
    lower_property_array_assign_with_diagnosed_key(
        ctx,
        &receiver,
        property,
        index,
        value,
        span,
        key_already_diagnosed,
    );
    let slot = ctx.declare_local(&receiver_temp, narrowed_ty);
    ctx.release_stored_local_value(&receiver_temp, slot, Some(span));
    ctx.emit_void(
        Op::UnsetLocal,
        Vec::new(),
        Some(Immediate::LocalSlot(slot)),
        Op::UnsetLocal.default_effects(),
        Some(span),
    );
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    true
}

fn lower_generic_object_mixed_property_array_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: &LoweredValue,
    property: &str,
    index: Option<&Expr>,
    value: &Expr,
    span: Span,
) {
    // Key and value first, then the container: PHP's dim fetch for write is delayed past the
    // right-hand side, so a call there that writes this same property keeps its writes.
    let index = index.map(|index| lower_expr(ctx, index));
    let value = lower_expr(ctx, value);
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
        own_property_container(ctx, property_value, span);
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

/// Whether `$object->property[$k] = $v` may write into the property's container in place.
///
/// The retaining shape -- read, retain, write, store back -- gives the write two owners, so the
/// runtime copies the whole table on every element written: a loop filling a property became
/// quadratic, and Symfony's `ErrorHandler::setLoggers` spent 5% of a request copying a
/// fifteen-entry array fifteen times. Writing through the property's own reference needs a
/// receiver whose slot codegen can name: a class the closed world declares, the property
/// declared in a plain slot (not by reference, not hooked, not readonly -- each of those keeps
/// the path that already implements its rule), and no receiver the generic path owns.
fn property_array_write_mutates_in_place(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    generic_receiver: bool,
) -> bool {
    if generic_receiver {
        return false;
    }
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object) else {
        return false;
    };
    let Some(info) = ctx.classes.get(class_name.trim_start_matches('\\')) else {
        return false;
    };
    info.property_offsets.contains_key(property)
        && info.declared_properties.contains(property)
        && !info.reference_properties.contains(property)
        && !info.backed_hooked_properties.contains(property)
        && !info.readonly_properties.contains(property)
}

/// Lowers `$object->property[$k] = $v` for an associative property as one in-place `HashSet`.
///
/// The key and value are evaluated BEFORE the container is read, which is PHP's order -- the
/// dim fetch for write is a delayed opline emitted after the right-hand side -- and what makes
/// the borrowed read safe: `$this->services[$id] = $this->create($id)` may itself write
/// `$this->services`, relocating the table, and a container read first would be left dangling.
/// The `HashSet` carries `Immediate::InPlacePropertyWrite`: its receiver is the property, the property's
/// own reference is the one the write consumes, and codegen publishes the result into the slot
/// (`hashes::lower_hash_set`). No `PropSet` follows.
fn lower_in_place_property_hash_set(
    ctx: &mut LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
    property_ty: &PhpType,
    index: &Expr,
    value: &Expr,
    span: Span,
) {
    let (index, value) =
        crate::ir_lower::stmt::array_write_core::lower_write_key_and_value(ctx, index, value);
    let data = ctx.intern_string(property);
    // Quiet for the same reason as the retaining path: over an eval-owned receiver the read
    // leaves compiled code, and PHP does not raise for the container of an indexed write.
    crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, span);
    let property_value = ctx.emit_value(
        Op::PropGet,
        vec![object],
        Some(Immediate::Data(data)),
        property_ty.clone(),
        Op::PropGet.default_effects(),
        Some(span),
    );
    crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, span);
    ctx.emit_void(
        Op::HashSet,
        vec![property_value.value, index.value, value.value],
        Some(Immediate::InPlacePropertyWrite),
        Op::HashSet.default_effects(),
        Some(span),
    );
    release_property_array_insert_value_after_retain(ctx, property_ty, value, span);
    let helper_released_value = indexed_property_array_element_type(property_ty)
        .is_some_and(|element| !matches!(element.codegen_repr(), PhpType::Mixed | PhpType::Callable));
    release_persisted_string_operand(ctx, index, span);
    if !helper_released_value {
        release_persisted_string_operand(ctx, value, span);
    }
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
    // The lowering type of `?array` is already `Mixed`; the declared union still names the
    // array member to create.
    let Some(container_ty) = auto_initialized_container_type(property_ty).or_else(|| {
        declared_object_property_type(ctx, object, property)
            .and_then(|declared| auto_initialized_container_type(&declared))
    }) else {
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

/// Returns whether an EIR receiver needs the read/modify/write-back element write.
///
/// PHP's bare `object` pseudo-type is the original case, and a BOXED GRADUAL receiver — an
/// untyped parameter holding an object — needs exactly the same treatment: the compiler cannot
/// name the class, so it cannot address the property's slot and mutate the array in place.
///
/// Falling through instead reached the final `RuntimeCall` fallback, whose
/// `__rt_mixed_property_get` only understands stdClass and answers null for every other class.
/// `__rt_mixed_array_set` then wrote into that null and the element was SILENTLY DISCARDED, while
/// a scalar write to the same property through the same receiver worked — that one already went
/// through the declared-property class-id ladder. Symfony's generated container is written
/// entirely against this shape (`getXService($container, $lazyLoad = true)` storing into
/// `$container->services[$id]`), so every service it built was rebuilt on the next `get()`.
pub(in crate::ir_lower) fn is_generic_object_receiver(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    match ctx.builder.value_php_type(object) {
        PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty() => {
            return true
        }
        PhpType::Mixed | PhpType::Union(_) => return true,
        _ => {}
    }
    receiver_stores_property_dynamically(ctx, object, property)
}

/// Returns whether this receiver keeps `property` in its dynamic side table rather than a slot.
///
/// A stdClass has no declared properties at all — `(object) ['vars' => []]` gives one whose keys
/// the compiler never sees — and a `#[\AllowDynamicProperties]` class keeps anything it did not
/// declare the same way. Both read and write that storage exactly as PHP's bare `object` type
/// does, so an element write through one takes the same Mixed read/modify/write the generic
/// receiver takes. Without this the lowering fell through to a `RuntimeCall` the backend has no
/// arm for, and `$state->vars[] = $m[1]` in Symfony's `CompiledUrlMatcherDumper` reached it.
fn receiver_stores_property_dynamically(
    ctx: &LoweringContext<'_, '_>,
    object: crate::ir::ValueId,
    property: &str,
) -> bool {
    let PhpType::Object(class_name) = ctx.builder.value_php_type(object) else {
        return false;
    };
    let class_name = class_name.trim_start_matches('\\');
    if crate::types::checker::builtin_stdclass::is_stdclass(class_name) {
        return true;
    }
    ctx.classes.get(class_name).is_some_and(|class_info| {
        class_info.allow_dynamic_properties && class_info.visible_property(property).is_none()
    })
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

/// Separates a declared PHP array property before mutating its packed-or-hash boxed payload.
/// `PropGetForWrite` publishes the detached cell and returns a borrow owned by the property.
/// `key_already_diagnosed` lets the boxed writer rebuild a float key its read already reported.
fn lower_php_array_property_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    index: Option<&Expr>,
    value: &Expr,
    span: Span,
    key_already_diagnosed: bool,
) {
    // php evaluates the key and the assigned value BEFORE it fetches the property for write
    // (`zend_delayed_compile_dim` delays the container fetch past the right-hand side). A
    // right-hand side that writes the same property -- `$this->types[$n] = $this->resolve($n)`
    // where `resolve()` recursively fills `$this->types`, as Symfony's
    // `FormRegistry::getType()` does -- must see its writes kept: fetching the container first
    // wrote the outer element into a stale, separated copy and lost the inner ones.
    let (key, value) = match index {
        Some(index) => {
            let (index, value) = array_write_core::lower_write_key_and_value(ctx, index, value);
            (Some(index), value)
        }
        None => (None, lower_expr(ctx, value)),
    };
    // php auto-initializes an UNINITIALIZED typed `array` property on an element write or
    // append (`public array $a; $o->a['k'] = 1;`, `$this->items[$k] ??= ...`). Fetching it for
    // write first raised "must not be accessed before initialization" instead.
    if is_concrete_object_receiver(ctx, object.value) {
        if let Some(property_ty) = object_property_type(ctx, object.value, property) {
            initialize_uninitialized_array_property_for_write(
                ctx,
                object.value,
                property,
                &property_ty,
                true,
                span,
            );
        }
    }
    let data = ctx.intern_string(property);
    let array = ctx.emit_value(
        Op::PropGetForWrite,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::php_array(),
        Op::PropGetForWrite.default_effects(),
        Some(span),
    );
    ctx.builder.set_value_ownership(array.value, Ownership::Borrowed);
    let value = if let Some(index) = key {
        ctx.emit_void(
            Op::RuntimeCall,
            vec![array.value, index.value, value.value],
            key_already_diagnosed.then_some(Immediate::Bool(true)),
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, index, span);
        value
    } else {
        ctx.emit_void(
            Op::MixedArrayAppend,
            vec![array.value, value.value],
            None,
            Op::MixedArrayAppend.default_effects(),
            Some(span),
        );
        // The append boxes a call's string by TAKING it over (`box_value_for_mixed_container`),
        // so releasing it here freed what the array now holds: `$this->paths[] =
        // rtrim($path, '/\\')` in Twig's `FilesystemLoader::addPath()` stored freed bytes, and
        // no template was ever found. Only a string the box copied is still this write's.
        if ctx.builder.value_php_type(value.value).codegen_repr() == PhpType::Str {
            release_string_copied_into_box(ctx, value, span);
            return;
        }
        value
    };
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Releases a temporary assigned into an object property after `PropSet` retains or boxes it.
pub(in crate::ir_lower) fn release_property_assignment_source_after_retaining_store(
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
    if !property_array_insert_retains_value(ctx, property_ty, value) {
        return;
    }
    if ctx.value_is_owning_temporary(value) {
        crate::ir_lower::ownership::release_if_owned(ctx, value, Some(span));
    }
}

/// Returns true when the backend insert helper takes its own reference to the element value.
///
/// Three disciplines meet here, and only the first needs a release:
///
/// - **Indexed storage** (`__rt_array_set_*`, `__rt_array_push_*`): the value is incref'd into
///   the slot unconditionally, so an owning temporary keeps a reference nobody consumes.
/// - **Mixed-element indexed storage from a concrete value**: codegen boxes the value first,
///   and the boxing path releases the box it made, so releasing here would double-free.
/// - **Assoc storage** (`__rt_hash_set`): the value is only incref'd when it cannot hand over
///   its own reference (`value_can_own_mixed_box_source`), so an owning temporary is moved in
///   and the store consumes it.
///
/// A Mixed-element *indexed* array fed an already-boxed Mixed value therefore falls in the
/// first group, not the second: nothing boxes, the incref still fires, and without the release
/// every `$o->items[0] += 1` / `$o->items[] = f()` leaks one Mixed cell (issue #1041).
fn property_array_insert_retains_value(
    ctx: &LoweringContext<'_, '_>,
    property_ty: &PhpType,
    value: LoweredValue,
) -> bool {
    let Some(elem_ty) = indexed_property_array_element_type(property_ty) else {
        return false;
    };
    if !matches!(elem_ty.codegen_repr(), PhpType::Mixed | PhpType::Callable) {
        return true;
    }
    if !matches!(property_ty.codegen_repr(), PhpType::Array(_)) {
        return false;
    }
    matches!(
        ctx.builder.value_php_type(value.value).codegen_repr(),
        PhpType::Mixed | PhpType::Union(_)
    )
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
    // Callable slots retain through the descriptor ABI, outside is_refcounted().
    if matches!(property_ty, PhpType::Str | PhpType::Callable) {
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
