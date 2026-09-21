//! Purpose:
//! Direct unset lowering for locals, arrays, properties, and magic methods.
//!
//! Called from:
//! - `crate::ir_lower::expr`.
//!
//! Key details:
//! - Preserves source-order evaluation, EIR typing, effects, and ownership contracts.

use super::*;

/// Lowers supported `unset(...)` targets without evaluating them as ordinary call args.
pub(super) fn lower_unset_locals(
    ctx: &mut LoweringContext<'_, '_>,
    args: &[Expr],
    expr: &Expr,
) -> Option<LoweredValue> {
    if !args.iter().all(|arg| unset_target_supported(ctx, arg)) {
        return None;
    }
    let null = lower_null(ctx, expr);
    for arg in args {
        match &arg.kind {
            ExprKind::Variable(name) => {
                ctx.unset_local(name, null, Some(arg.span));
            }
            ExprKind::ArrayAccess { array, index } => {
                lower_unset_array_access(ctx, array, index, arg);
            }
            ExprKind::PropertyAccess { object, property }
            | ExprKind::NullsafePropertyAccess { object, property } => {
                lower_unset_property_access(ctx, object, property, arg);
            }
            ExprKind::DynamicPropertyAccess { object, property }
            | ExprKind::NullsafeDynamicPropertyAccess { object, property } => {
                lower_unset_dynamic_property_access(ctx, object, property, arg);
            }
            _ => {}
        }
    }
    crate::ir_lower::ownership::collect_cycles(ctx, Some(expr.span));
    Some(null)
}

/// Returns true when an `unset(...)` target has direct EIR lowering.
pub(super) fn unset_target_supported(ctx: &LoweringContext<'_, '_>, arg: &Expr) -> bool {
    match &arg.kind {
        ExprKind::Variable(_) => true,
        ExprKind::ArrayAccess { array, .. } => {
            unset_array_access_has_object_receiver(ctx, array)
                || unset_array_access_has_local_array_receiver(ctx, array)
                || unset_array_access_has_property_array_receiver(ctx, array)
                || unset_array_access_has_generic_object_property_receiver(ctx, array)
                || unset_array_access_has_static_property_receiver(ctx, array)
                || crate::ir_lower::stmt::nested_local_hash_unset_supported(ctx, arg)
                || crate::ir_lower::stmt::nested_property_hash_unset_supported(ctx, arg)
                || crate::ir_lower::stmt::nested_static_property_hash_unset_supported(ctx, arg)
        }
        ExprKind::PropertyAccess { object, property }
        | ExprKind::NullsafePropertyAccess { object, property } => {
            unset_property_access_has_direct_lowering(ctx, object, property)
        }
        ExprKind::DynamicPropertyAccess { .. }
        | ExprKind::NullsafeDynamicPropertyAccess { .. } => true,
        _ => false,
    }
}

/// Lowers `unset($object->$property)` after preserving PHP's receiver/name evaluation order.
fn lower_unset_dynamic_property_access(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &Expr,
    expr: &Expr,
) {
    let object = lower_expr(ctx, object);
    let property = lower_expr(ctx, property);
    let property = coerce_to_string_at_span(ctx, property, Some(expr.span));
    ctx.emit_void(
        Op::DynamicPropUnset,
        vec![object.value, property.value],
        None,
        Op::DynamicPropUnset.default_effects(),
        Some(expr.span),
    );
    if ctx.value_is_owning_temporary(property) {
        crate::ir_lower::ownership::release_if_owned(ctx, property, Some(expr.span));
    }
}

/// Returns true when an array-access unset receiver is a declared static property whose storage
/// can be converted to a sparse hash and written back through the static-property slot.
fn unset_array_access_has_static_property_receiver(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ExprKind::StaticPropertyAccess { receiver, property } = &array.kind else {
        return false;
    };
    matches!(
        static_property_result_type(ctx, receiver, property, array).codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed | PhpType::Union(_)
    )
}

/// Returns true when an array-access unset receiver is a declared instance property with concrete
/// array storage that can be loaded, sparsified, mutated, and written back directly in EIR.
fn unset_array_access_has_property_array_receiver(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ExprKind::PropertyAccess { object, property } = &array.kind else {
        return false;
    };
    matches!(
        property_access_expr_type_for_ir(ctx, object, property)
            .map(|ty| ty.codegen_repr()),
        Some(PhpType::Array(_) | PhpType::AssocArray { .. })
    )
}

/// Returns true when the receiver is a property of a GENERIC object — one whose class the compiler
/// cannot name, so there is no declared slot to walk, only a boxed cell to read and write back.
///
/// Symfony's generated container is written this way throughout: `getXService($container, …)`
/// takes an UNTYPED parameter and mutates `$container->services[$id]`. The element WRITE already
/// has this path (`lower_generic_object_mixed_property_array_write`); this is what lets the UNSET
/// reach its counterpart instead of the fallback whose only job is to refuse.
///
/// Restricted to a plain variable receiver, which is the shape that occurs and the shape the
/// lowering handles; anything else keeps its existing treatment.
fn unset_array_access_has_generic_object_property_receiver(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ExprKind::PropertyAccess { object, property } = &array.kind else {
        return false;
    };
    if matches!(
        property_access_expr_type_for_ir(ctx, object, property)
            .map(|ty| ty.codegen_repr()),
        Some(PhpType::Mixed | PhpType::Union(_))
    ) {
        return true;
    }
    let ExprKind::Variable(name) = &object.kind else {
        return false;
    };
    let ty = ctx.local_type(name);
    matches!(ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
        || matches!(&ty, PhpType::Object(class_name) if class_name.trim_start_matches('\\').is_empty())
}

/// Returns true when an array-access unset receiver is a plain array/hash local whose element the
/// EIR backend can remove.
///
/// Associative arrays remove the element directly; packed indexed arrays are converted to a hash at
/// the unset site (PHP `unset()` leaves a sparse array). By-reference locals are excluded: their
/// storage is aliased to a caller whose static type would no longer match after a representation
/// change.
pub(super) fn unset_array_access_has_local_array_receiver(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ExprKind::Variable(name) = &array.kind else {
        return false;
    };
    matches!(
        ctx.local_type(name).codegen_repr(),
        PhpType::AssocArray { .. } | PhpType::Array(_) | PhpType::Mixed | PhpType::Union(_)
    )
}

/// Returns true when an array-access unset receiver is a static ArrayAccess object.
pub(super) fn unset_array_access_has_object_receiver(
    ctx: &LoweringContext<'_, '_>,
    array: &Expr,
) -> bool {
    let ty = match &array.kind {
        ExprKind::Variable(name) => ctx
            .local_types
            .get(name)
            .cloned()
            .unwrap_or_else(|| infer_expr_type_syntactic(array)),
        ExprKind::StaticPropertyAccess { receiver, property } => {
            static_property_result_type(ctx, receiver, property, array)
        }
        ExprKind::PropertyAccess { object, property } => {
            property_access_expr_type_for_ir(ctx, object, property)
                .unwrap_or_else(|| infer_expr_type_syntactic(array))
        }
        _ => infer_expr_type_syntactic(array),
    };
    type_satisfies_array_access_for_ir(ctx, &ty)
}

/// Lowers `unset($array[$key])`, dispatching on the receiver kind.
///
/// An associative-array local removes the element in place through `Op::HashUnset`. A packed
/// indexed-array local is first converted to a hash (PHP keeps the surviving keys without
/// renumbering) and then removed. An `ArrayAccess` object dispatches to its `offsetUnset($key)`
/// method like before. By-reference array locals fall through to the object path.
pub(super) fn lower_unset_array_access(
    ctx: &mut LoweringContext<'_, '_>,
    array: &Expr,
    index: &Expr,
    expr: &Expr,
) {
    if crate::ir_lower::stmt::lower_nested_local_hash_unset(ctx, expr) {
        return;
    }
    if crate::ir_lower::stmt::lower_nested_property_hash_unset(ctx, expr) {
        return;
    }
    if crate::ir_lower::stmt::lower_nested_static_property_hash_unset(ctx, expr) {
        return;
    }
    if let ExprKind::StaticPropertyAccess { receiver, property } = &array.kind {
        if lower_unset_static_property_array_element(ctx, receiver, property, index, expr) {
            return;
        }
    }
    if let ExprKind::PropertyAccess { object, property } = &array.kind {
        if lower_unset_property_array_element(ctx, object, property, index, expr) {
            return;
        }
    }
    if let ExprKind::Variable(name) = &array.kind {
        match ctx.local_type(name).codegen_repr() {
            PhpType::AssocArray { .. } => {
                lower_unset_hash_element(ctx, name, array.span, index, expr);
                return;
            }
            PhpType::Array(elem_ty) => {
                let elem_ty = if *elem_ty == PhpType::Never {
                    PhpType::Mixed
                } else {
                    *elem_ty
                };
                lower_unset_indexed_element(ctx, name, elem_ty, array.span, index, expr);
                return;
            }
            PhpType::Mixed | PhpType::Union(_) => {
                lower_unset_mixed_array_element(ctx, name, array.span, index, expr);
                return;
            }
            _ => {}
        }
    }
    let synthetic = Expr::new(
        ExprKind::MethodCall {
            object: Box::new(array.clone()),
            method: "offsetUnset".to_string(),
            args: vec![index.clone()],
        },
        expr.span,
    );
    lower_expr(ctx, &synthetic);
}

/// Lowers `unset(Class::$property[$key])` by promoting the static array to sparse hash storage,
/// deleting the key, and publishing the possibly relocated container back to the static slot.
fn lower_unset_static_property_array_element(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    expr: &Expr,
) -> bool {
    let property_expr = Expr::new(
        ExprKind::StaticPropertyAccess {
            receiver: receiver.clone(),
            property: property.to_string(),
        },
        expr.span,
    );
    let property_ty = static_property_result_type(ctx, receiver, property, &property_expr)
        .codegen_repr();
    if !matches!(
        property_ty,
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Mixed | PhpType::Union(_)
    ) {
        return false;
    }
    let property_value = lower_static_property_get(ctx, receiver, property, &property_expr);
    let hash_ty = match &property_ty {
        PhpType::Array(element) => PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(element.codegen_repr()),
        },
        PhpType::AssocArray { .. } => property_ty.clone(),
        PhpType::Mixed | PhpType::Union(_) => PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(PhpType::Mixed),
        },
        _ => return false,
    };
    let hash = match property_ty {
        PhpType::Array(_) => ctx.emit_value(
            Op::ArrayToHash,
            vec![property_value.value],
            None,
            hash_ty,
            Op::ArrayToHash.default_effects(),
            Some(expr.span),
        ),
        PhpType::Mixed | PhpType::Union(_) => ctx.emit_value(
            Op::MixedToHash,
            vec![property_value.value],
            None,
            hash_ty,
            Op::MixedToHash.default_effects(),
            Some(expr.span),
        ),
        PhpType::AssocArray { .. } => property_value,
        _ => return false,
    };
    let key = lower_expr(ctx, index);
    ctx.emit_void(
        Op::HashUnset,
        vec![hash.value, key.value],
        None,
        Op::HashUnset.default_effects(),
        Some(expr.span),
    );
    let stored = if matches!(property_ty, PhpType::Mixed | PhpType::Union(_)) {
        ctx.box_value_as_mixed(hash, PhpType::Mixed, Some(expr.span))
    } else {
        hash
    };
    let name = format!("{}::{}", receiver_name(receiver), property);
    let data = ctx.intern_string(&name);
    ctx.emit_void(
        Op::StoreStaticProperty,
        vec![stored.value],
        Some(Immediate::Data(data)),
        Op::StoreStaticProperty.default_effects(),
        Some(expr.span),
    );
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(expr.span));
    } else {
        crate::ir_lower::stmt::release_persisted_string_operand(ctx, key, expr.span);
    }
    true
}

/// Lowers `unset($object->property[$key])` by separating the declared array property, promoting
/// indexed storage to a sparse hash when needed, deleting the key, and publishing the possibly
/// relocated container back through the ordinary retaining property store.
fn lower_unset_property_array_element(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    expr: &Expr,
) -> bool {
    let object = lower_expr(ctx, object);
    let declared_array_property =
        crate::ir_lower::stmt::object_property_type(ctx, object.value, property)
            .map(|ty| ty.codegen_repr())
            .filter(|ty| matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. }));
    let Some(property_ty) = declared_array_property else {
        // No declared ARRAY slot to walk, but the property may still be one boxed cell — either
        // because the receiver's class is unknown (Symfony's generated container) or because the
        // property is declared without a type (Twig's `Node::$attributes`). Both read, modify and
        // write that cell back; anything else keeps falling through to `offsetUnset()`.
        return lower_unset_gradual_property_element(ctx, &object, property, index, expr);
    };
    let data = ctx.intern_string(property);
    // PHP evaluates the dimension even when the property is uninitialized and the unset itself
    // becomes a no-op. Lower it before the initialization branch, then release it once at the
    // merge shared by the present and absent paths.
    let key = lower_expr(ctx, index);
    // The CONTAINER of an `unset` is fetched QUIETLY, like the container of an `isset`/`empty`
    // operand: measured with `php -n` 8.5.6, `unset($o->p['k'])` on an uninitialized typed
    // property neither raises nor initializes it. Over an EVAL-OWNED receiver this fetch leaves
    // compiled code through the bridge, which raises unless the mode is on.
    //
    // This is the ONE-level path. The two- and three-level chains are lowered by
    // `lower_nested_property_hash_unset`, whose gate is `(2..=3).contains(&chain.len())`, so
    // wrapping only that one left exactly this shape uncovered -- and it is the shape Symfony
    // writes: `unset($this->checkedLazyNodes[$id])` in `CheckCircularReferencesPass`.
    // An UNINITIALIZED typed property makes the whole `unset` a no-op, so the slot is probed
    // BEFORE the fetch. Doing it after is not enough: the fetch itself is what raises on the
    // native path, through the compiled typed-property read guard, which no interpreter-side
    // mode can reach. Measured under `php -n` 8.5.6, `unset($o->p['k'])` on `public array $p;`
    // left unassigned neither raises nor initializes it -- `isset($o->p)` is still `false`
    // afterwards -- while the same unset on a written property really does delete the element
    // and leaves the (now empty) array initialized.
    //
    // The probe reads the object's own initialization marker, which is the right store on both
    // routes: an AOT class keeps its property values in native slots even when the INSTANCE is
    // eval-owned. An eval-DECLARED class never reaches this lowering, because
    // `object_property_type` above would not have resolved a declared array type for it.
    let absent_block = ctx
        .builder
        .create_named_block("unset.property_element.absent", Vec::new());
    let fetch_block = ctx
        .builder
        .create_named_block("unset.property_element.fetch", Vec::new());
    let delete_block = ctx
        .builder
        .create_named_block("unset.property_element.delete", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("unset.property_element.merge", Vec::new());
    let initialized = ctx.emit_value(
        Op::PropInitialized,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: fetch_block,
        then_args: Vec::new(),
        else_target: absent_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(absent_block);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(fetch_block);
    crate::ir_lower::expr::emit_quiet_property_fetch(ctx, true, expr.span);
    let property_value = ctx.emit_value(
        Op::PropGet,
        vec![object.value],
        Some(Immediate::Data(data)),
        property_ty.clone(),
        Op::PropGet.default_effects(),
        Some(expr.span),
    );
    crate::ir_lower::expr::emit_quiet_property_fetch(ctx, false, expr.span);
    // A container that still comes back absent is skipped as well. `ArrayToHash` of a null
    // container is what segfaulted the Symfony worker once the fetch stopped raising, so the
    // second guard stays even though the probe above should already have branched away.
    let is_absent = ctx.emit_value(
        Op::IsNull,
        vec![property_value.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_absent.value,
        then_target: absent_block,
        then_args: Vec::new(),
        else_target: delete_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(delete_block);
    let property_value =
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(expr.span));
    let hash_ty = match &property_ty {
        PhpType::Array(element) => PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(element.codegen_repr()),
        },
        PhpType::AssocArray { .. } => property_ty.clone(),
        _ => unreachable!("array property type checked above"),
    };
    let hash = if matches!(property_ty, PhpType::Array(_)) {
        ctx.emit_value(
            Op::ArrayToHash,
            vec![property_value.value],
            None,
            hash_ty.clone(),
            Op::ArrayToHash.default_effects(),
            Some(expr.span),
        )
    } else {
        property_value
    };
    ctx.emit_void(
        Op::HashUnset,
        vec![hash.value, key.value],
        None,
        Op::HashUnset.default_effects(),
        Some(expr.span),
    );
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, hash.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(expr.span),
    );
    crate::ir_lower::stmt::release_rewritten_property_value_after_retaining_store(
        ctx,
        &hash_ty,
        hash,
        expr.span,
    );
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(expr.span));
    } else {
        crate::ir_lower::stmt::release_persisted_string_operand(ctx, key, expr.span);
    }
    true
}

/// Lowers `unset($obj->prop[$key])` when the receiver is a GENERIC object, not a known class.
///
/// The element WRITE through the same shape already exists
/// (`lower_generic_object_mixed_property_array_write`) and exists because Symfony's generated
/// container passes itself as an UNTYPED parameter — `getXService($container, …)` mutating
/// `$container->services[$id]` — so the declared-property ladder has no class to walk. The unset
/// in the container's own `catch` block is the same shape and used to be refused outright.
///
/// Read, modify, write back: the property is one boxed cell, `MixedToHash` makes an independently
/// owned sparse hash of it (so removing a key cannot mutate an alias of the original boxed array,
/// and PHP's non-renumbering `unset()` holds for an indexed source too), the key goes, and the
/// result is boxed back into the same property.
///
/// `PropInitialized` guards the absent property: `unset($o->missing['k'])` is a silent no-op in
/// PHP, while `MixedToHash` of a null raises the gradual array `TypeError`.
///
/// The new box is NOT released after the store: `property_store_keeps_independent_ref(Mixed,
/// Mixed)` is false, so the `PropSet` takes the reference. That is the one place this differs from
/// the typed path above, which stores back the very pointer it retained and therefore releases it.
fn lower_unset_gradual_property_element(
    ctx: &mut LoweringContext<'_, '_>,
    object: &LoweredValue,
    property: &str,
    index: &Expr,
    expr: &Expr,
) -> bool {
    // KNOWN LIMIT. A gradual property could hold an ArrayAccess OBJECT instead of an array, and
    // PHP decides that at run time. The shape this replaces did not decide it either — it emitted
    // `offsetUnset()` unconditionally, which faults on the array case that actually occurs
    // (Symfony's container, Twig's `Node::$attributes`). Getting both right needs a runtime tag
    // branch here; until then this takes the array reading, which is the one these frameworks use.
    let receiver_is_generic =
        crate::ir_lower::stmt::property_array_writes::is_generic_object_receiver(
            ctx,
            object.value,
            property,
        );
    let property_is_gradual = matches!(
        crate::ir_lower::stmt::object_property_type(ctx, object.value, property)
            .map(|ty| ty.codegen_repr()),
        Some(PhpType::Mixed | PhpType::Union(_))
    );
    if !receiver_is_generic && !property_is_gradual {
        return false;
    }
    let data = ctx.intern_string(property);
    // PHP evaluates the dimension even when the unset turns out to be a no-op.
    let key = lower_expr(ctx, index);
    let absent_block = ctx
        .builder
        .create_named_block("unset.generic_property_element.absent", Vec::new());
    let delete_block = ctx
        .builder
        .create_named_block("unset.generic_property_element.delete", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("unset.generic_property_element.merge", Vec::new());
    let initialized = ctx.emit_value(
        Op::PropInitialized,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::Bool,
        Op::PropInitialized.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: initialized.value,
        then_target: delete_block,
        then_args: Vec::new(),
        else_target: absent_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(absent_block);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(delete_block);
    let property_value = ctx.emit_value(
        Op::PropGet,
        vec![object.value],
        Some(Immediate::Data(data)),
        PhpType::Mixed,
        Op::PropGet.default_effects(),
        Some(expr.span),
    );
    let assoc_ty = PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    };
    let hash = ctx.emit_value(
        Op::MixedToHash,
        vec![property_value.value],
        None,
        assoc_ty,
        Op::MixedToHash.default_effects(),
        Some(expr.span),
    );
    ctx.emit_void(
        Op::HashUnset,
        vec![hash.value, key.value],
        None,
        Op::HashUnset.default_effects(),
        Some(expr.span),
    );
    let boxed = ctx.box_value_as_mixed(hash, PhpType::Mixed, Some(expr.span));
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, boxed.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(expr.span),
    );
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(expr.span));
    } else {
        crate::ir_lower::stmt::release_persisted_string_operand(ctx, key, expr.span);
    }
    true
}

/// Lowers `unset($mixed[$key])` for a gradual local that holds an array at runtime.
///
/// `MixedToHash` enforces the runtime array boundary and returns an independently owned sparse
/// hash, so deleting a key cannot mutate aliases of the original boxed array. The mutated hash is
/// boxed back into the local's unchanged `Mixed` storage after the previous boxed owner is
/// released. This is the gradual counterpart of `lower_unset_indexed_element` and preserves PHP's
/// non-renumbering `unset()` semantics for both indexed and associative runtime arrays.
fn lower_unset_mixed_array_element(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    array_span: Span,
    index: &Expr,
    expr: &Expr,
) {
    let source = ctx.load_local(name, Some(array_span));
    let assoc_ty = PhpType::AssocArray {
        key: Box::new(PhpType::Mixed),
        value: Box::new(PhpType::Mixed),
    };
    let hash = ctx.emit_value(
        Op::MixedToHash,
        vec![source.value],
        None,
        assoc_ty,
        Op::MixedToHash.default_effects(),
        Some(array_span),
    );
    let key = lower_expr(ctx, index);
    ctx.emit_void(
        Op::HashUnset,
        vec![hash.value, key.value],
        None,
        Op::HashUnset.default_effects(),
        Some(expr.span),
    );
    crate::ir_lower::stmt::release_persisted_string_operand(ctx, key, expr.span);
    let boxed = ctx.box_value_as_mixed(hash, PhpType::Mixed, Some(expr.span));
    let slot = ctx.declare_local(name, PhpType::Mixed);
    ctx.release_stored_local_value(name, slot, Some(expr.span));
    ctx.store_prepared_mutated_local(name, boxed, PhpType::Mixed, Some(expr.span));
    ctx.set_local_type(name, PhpType::Mixed);
}

/// Lowers `unset($hash[$key])` for an associative-array local as a `HashUnset` instruction.
///
/// Loads the array local, lowers the key, and emits the removal. The backend (`lower_hash_unset`)
/// copy-on-write splits the table, releases the removed key/value payloads, and stores the unique
/// table pointer back into the local slot, so no explicit store-back is needed here.
pub(super) fn lower_unset_hash_element(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    array_span: Span,
    index: &Expr,
    expr: &Expr,
) {
    let array_value = ctx.load_local(name, Some(array_span));
    let index_value = lower_expr(ctx, index);
    ctx.emit_void(
        Op::HashUnset,
        vec![array_value.value, index_value.value],
        None,
        Op::HashUnset.default_effects(),
        Some(expr.span),
    );
}

/// Lowers `unset($arr[$key])` for a packed indexed-array local.
///
/// PHP's `unset()` removes a key without renumbering, so the array can no longer be a contiguous
/// packed list (e.g. `unset([1,2,3][1])` leaves keys `0` and `2`). The local is converted to a hash
/// (`Op::ArrayToHash`) and retyped as `AssocArray<Int, T>`, after which the element is removed
/// through `HashUnset`. Subsequent uses of the local therefore see the associative representation.
pub(super) fn lower_unset_indexed_element(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    elem_ty: PhpType,
    array_span: Span,
    index: &Expr,
    expr: &Expr,
) {
    let array_value = ctx.load_local(name, Some(array_span));
    let assoc_ty = PhpType::AssocArray {
        key: Box::new(PhpType::Int),
        value: Box::new(elem_ty),
    };
    let hash = ctx.emit_value(
        Op::ArrayToHash,
        vec![array_value.value],
        None,
        assoc_ty.clone(),
        Op::ArrayToHash.default_effects(),
        Some(array_span),
    );
    ctx.store_mutated_local(name, hash, assoc_ty, Some(array_span));
    lower_unset_hash_element(ctx, name, array_span, index, expr);
}

/// Returns true when a property unset target can be lowered without normal property storage support.
pub(super) fn unset_property_access_has_direct_lowering(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
) -> bool {
    matches!(
        property_unset_action(ctx, object, property),
        Some(
            UnsetPropertyAction::Magic
                | UnsetPropertyAction::Noop
                | UnsetPropertyAction::ClearTyped
                | UnsetPropertyAction::RemoveDynamic
        )
    )
}

/// Lowers `unset($object->property)` for magic and no-op property targets.
/// Lowers `unset($object->property)` for magic, no-op, fixed-slot and dynamic property targets.
pub(super) fn lower_unset_property_access(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    expr: &Expr,
) {
    match property_unset_action(ctx, object, property) {
        Some(UnsetPropertyAction::Magic) => {
            let object = lower_expr(ctx, object);
            lower_magic_property_unset(ctx, object, property, expr);
        }
        Some(UnsetPropertyAction::Noop) => {
            lower_expr(ctx, object);
        }
        // Both storage shapes share `Op::PropUnset`: the backend already resolves the
        // receiver's property storage, so it picks the fixed-slot marker or the
        // dynamic-hash removal from the same instruction.
        Some(UnsetPropertyAction::ClearTyped | UnsetPropertyAction::RemoveDynamic) => {
            let object = lower_expr(ctx, object);
            let data = ctx.intern_string(property);
            ctx.emit_void(
                Op::PropUnset,
                vec![object.value],
                Some(Immediate::Data(data)),
                Op::PropUnset.default_effects(),
                Some(expr.span),
            );
        }
        Some(UnsetPropertyAction::Fallback) | None => {}
    }
}

/// Describes how `unset($object->property)` should be lowered for a known receiver class.
pub(super) enum UnsetPropertyAction {
    Fallback,
    Magic,
    Noop,
    /// The property has a DECLARED type, so PHP's `unset()` leaves it uninitialized —
    /// a state elephc's fixed property slots represent exactly.
    ClearTyped,
    /// The property lives in the receiver's dynamic-property hash (`stdClass`, or an
    /// undeclared name on an `#[AllowDynamicProperties]` class), where PHP's `unset()`
    /// really is a key removal.
    RemoveDynamic,
}

/// Selects the PHP-visible `unset()` behavior for a statically known object property operand.
pub(super) fn property_unset_action(
    ctx: &LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
) -> Option<UnsetPropertyAction> {
    let (class_name, _) = isset_object_expr_class(ctx, object)?;
    // Every `stdClass` property is a hash entry, so `unset()` is a plain key removal and
    // `stdClass` declares no magic methods that could intercept it.
    if is_builtin_stdclass_name(&class_name) {
        return Some(UnsetPropertyAction::RemoveDynamic);
    }
    let class_info = ctx.classes.get(class_name.as_str())?;
    if class_info.allow_dynamic_properties && class_info.visible_property(property).is_none() {
        return Some(dynamic_property_unset_action(ctx, &class_name));
    }
    if property_is_accessible_for_ir(ctx, &class_name, class_info, property) {
        // PHP does NOT consult `__unset` for a property it can see: it removes the
        // property itself. A DECLARED (typed) property becomes uninitialized, which
        // elephc's fixed slots can represent exactly.
        if class_info.visible_property_is_declared(property) {
            return Some(UnsetPropertyAction::ClearTyped);
        }
        // Untyped and typed fixed slots share the marker-backed absent state. This keeps
        // `isset()` false and permits a later assignment to initialize the slot again.
        return Some(UnsetPropertyAction::ClearTyped);
    }
    if class_method_signature(ctx, &class_name, &php_symbol_key("__unset")).is_some() {
        Some(UnsetPropertyAction::Magic)
    } else {
        Some(UnsetPropertyAction::Noop)
    }
}

/// Lowers a magic `__unset($name)` call, guarding nullable receivers as a no-op.
pub(super) fn lower_magic_property_unset(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    expr: &Expr,
) {
    if value_is_nullable(ctx, object.value) {
        lower_nullable_magic_property_unset(ctx, object, property, expr);
        return;
    }
    let args = vec![Expr::new(
        ExprKind::StringLiteral(property.to_string()),
        expr.span,
    )];
    lower_method_call_with_receiver(ctx, object, "__unset", &args, Op::MethodCall, expr);
}

/// Lowers `__unset` for nullable receivers, doing nothing when the receiver is null.
pub(super) fn lower_nullable_magic_property_unset(
    ctx: &mut LoweringContext<'_, '_>,
    object: LoweredValue,
    property: &str,
    expr: &Expr,
) {
    let null_block = ctx
        .builder
        .create_named_block("unset.property.null", Vec::new());
    let call_block = ctx
        .builder
        .create_named_block("unset.property.call", Vec::new());
    let merge = ctx
        .builder
        .create_named_block("unset.property.merge", Vec::new());
    let is_null = ctx.emit_value(
        Op::IsNull,
        vec![object.value],
        None,
        PhpType::Bool,
        Op::IsNull.default_effects(),
        Some(expr.span),
    );
    ctx.builder.terminate(Terminator::CondBr {
        cond: is_null.value,
        then_target: null_block,
        then_args: Vec::new(),
        else_target: call_block,
        else_args: Vec::new(),
    });

    ctx.builder.position_at_end(null_block);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(call_block);
    let args = vec![Expr::new(
        ExprKind::StringLiteral(property.to_string()),
        expr.span,
    )];
    lower_method_call_with_receiver(ctx, object, "__unset", &args, Op::MethodCall, expr);
    branch_to(ctx, merge);

    ctx.builder.position_at_end(merge);
}
