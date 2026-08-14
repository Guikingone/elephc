//! Purpose:
//! Nested write-context array autovivification.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers a nested array assignment that already carries an expression target.
pub(super) fn lower_nested_array_assign(
    ctx: &mut LoweringContext<'_, '_>,
    target: &Expr,
    value: &Expr,
    span: Span,
) {
    // Lowering the FULL target as an expression routes the write through the
    // read helper (`__rt_mixed_array_get`), which returns a detached fresh box
    // whenever the slot storage is not already a boxed Mixed cell; the
    // two-operand cell replacement then mutated a temporary and the write was
    // silently lost (#529). Splitting off the innermost key writes through the
    // parent cell instead (`__rt_mixed_array_set` for Mixed parents,
    // `offsetSet` for ArrayAccess objects), which mutates the aliased
    // container for every slot representation. The parent chain itself is
    // lowered with fetch-for-write semantics so missing or null intermediate
    // elements autovivify as arrays instead of dropping the write (#555).
    if let ExprKind::ArrayAccess { array, index } = &target.kind {
        let parent = lower_nested_assign_parent(ctx, array, span);
        let key = lower_expr(ctx, index);
        // A parser-owned `ArrayReference` marker carries a promoted local reference cell into
        // the leaf slot. Use the same marker-aware path as direct array/hash writes so nested
        // targets preserve alias identity instead of attempting ordinary expression lowering.
        let value = lower_array_reference_or_value(ctx, value);
        ctx.emit_void(
            Op::RuntimeCall,
            vec![parent.value, key.value, value.value],
            None,
            effects_lookup::runtime_effects(),
            Some(span),
        );
        release_persisted_string_operand(ctx, key, span);
        release_persisted_string_operand(ctx, value, span);
        // Parent subscript reads of Mixed/refcounted elements are owning
        // temporaries (`ArrayGet`/`HashGet`/`RuntimeCall` return a +1 caller
        // reference — fresh, retained, or installed by autovivification). The
        // set helper mutates through the cell/object without consuming that
        // reference, so release it here. Non-owning parents (plain locals,
        // `$this`) are left to normal scope cleanup.
        if ctx.value_is_owning_temporary(parent) {
            crate::ir_lower::ownership::release_if_owned(ctx, parent, Some(span));
        }
        return;
    }
    let target = lower_expr(ctx, target);
    let value = lower_expr(ctx, value);
    ctx.emit_void(
        Op::RuntimeCall,
        vec![target.value, value.value],
        None,
        effects_lookup::runtime_effects(),
        Some(span),
    );
}

/// Lowers the parent chain of a nested array assignment with write-context
/// (fetch-for-write) semantics (issue #555): missing indexed elements, null
/// gap slots, boxed `Mixed(null)` elements, and missing hash keys autovivify
/// as empty arrays installed into the parent storage, and the STORED cell is
/// returned so the leaf write lands in the parent container. PHP emits no
/// undefined-key warning for these legal writes, and neither does this path.
/// Shapes without a for-write lowering fall back to the plain read used
/// before (ArrayAccess objects, non-container receivers).
pub(super) fn lower_nested_assign_parent(
    ctx: &mut LoweringContext<'_, '_>,
    expr: &Expr,
    span: Span,
) -> LoweredValue {
    let ExprKind::ArrayAccess { array, index } = &expr.kind else {
        return lower_expr(ctx, expr);
    };
    // Concrete container locals: ensure the element exists through the
    // runtime wrapper and store the possibly reallocated container back.
    if let ExprKind::Variable(name) = &array.kind {
        let name = name.clone();
        if let Some(parent) = lower_local_parent_fetch_for_write(ctx, &name, index, expr) {
            return parent;
        }
    }
    // Generic array properties may promote or relocate while fetching a child for write.
    // Publish that container back into the property before descending to the leaf.
    if let ExprKind::PropertyAccess { object, property } = &array.kind {
        if let Some(parent) =
            lower_property_parent_fetch_for_write(ctx, object, property, index, expr)
        {
            return parent;
        }
    }
    // Static array properties need the same fetch-for-write/store-back cycle as instance
    // properties. A plain expression read returns a detached Mixed child, so mutating that
    // child would never reach the symbol-backed static slot.
    if let ExprKind::StaticPropertyAccess { receiver, property } = &array.kind {
        if let Some(parent) =
            lower_static_property_parent_fetch_for_write(ctx, receiver, property, index, expr)
        {
            return parent;
        }
    }
    // Boxed Mixed receivers: chains recurse with for-write semantics; other
    // receiver shapes evaluate once as plain reads of the receiver cell.
    let receiver = if matches!(array.kind, ExprKind::ArrayAccess { .. }) {
        lower_nested_assign_parent(ctx, array, span)
    } else {
        lower_expr(ctx, array)
    };
    if ctx.builder.value_php_type(receiver.value).codegen_repr() == PhpType::Mixed {
        let key = lower_expr(ctx, index);
        let parent = ctx.emit_value(
            Op::RuntimeCall,
            vec![receiver.value, key.value],
            Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
            PhpType::Mixed,
            effects_lookup::runtime_effects(),
            Some(expr.span),
        );
        release_persisted_string_operand(ctx, key, span);
        if ctx.value_is_owning_temporary(receiver) {
            crate::ir_lower::ownership::release_if_owned(ctx, receiver, Some(span));
        }
        return parent;
    }
    // The receiver is already evaluated but not a boxed Mixed cell: finish as
    // the plain subscript read the pre-#555 lowering produced.
    lower_array_access_from_lowered_receiver(ctx, receiver, index, expr)
}

/// Ensures `$object->property[$index]` exists as the parent of a nested write.
///
/// Generic PHP `array` properties use Mixed element storage but may hold either indexed or hash
/// runtime storage. The fetch-for-write helper separates and autovivifies the requested child,
/// then `PropSet` publishes the possibly relocated container back into the property.
fn lower_property_parent_fetch_for_write(
    ctx: &mut LoweringContext<'_, '_>,
    object: &Expr,
    property: &str,
    index: &Expr,
    parent_expr: &Expr,
) -> Option<LoweredValue> {
    let span = parent_expr.span;
    let object = lower_expr(ctx, object);
    let property_ty = object_property_type(ctx, object.value, property)?.codegen_repr();
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
    let key = lower_expr(ctx, index);
    let key_ty = lowered_index_expr_key_type(ctx, index, key.value).codegen_repr();
    let (ensured_ty, read_op) = match &property_ty {
        // A PHP property declared simply as `array` may keep the empty/default element marker
        // even though its EIR storage is the generic Mixed-slot representation. Nested writes are
        // valid for every such declared array; the runtime key decides whether it stays indexed or
        // promotes to a hash.
        PhpType::Array(_) => match key_ty {
            PhpType::Int => (property_ty.clone(), Op::ArrayGetForWrite),
            PhpType::Str => (
                promoted_assoc_array_type(property_ty.clone(), PhpType::Mixed),
                Op::HashGetForWrite,
            ),
            PhpType::Mixed | PhpType::Union(_) => {
                (property_ty.clone(), Op::ArrayGetMixedKeyForWrite)
            }
            _ => return None,
        },
        PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed => {
            (property_ty.clone(), Op::HashGetForWrite)
        }
        _ => return None,
    };
    let ensured = ctx.emit_value(
        Op::RuntimeCall,
        vec![property_value.value, key.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
        ensured_ty.clone(),
        effects_lookup::runtime_effects(),
        Some(span),
    );
    ctx.emit_void(
        Op::PropSet,
        vec![object.value, ensured.value],
        Some(Immediate::Data(data)),
        Op::PropSet.default_effects(),
        Some(span),
    );
    let parent = ctx.emit_value(
        read_op,
        vec![ensured.value, key.value],
        None,
        PhpType::Mixed,
        read_op.default_effects(),
        Some(span),
    );
    release_rewritten_property_value_after_retaining_store(ctx, &ensured_ty, ensured, span);
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(span));
    } else {
        release_persisted_string_operand(ctx, key, span);
    }
    Some(parent)
}

/// Ensures `Class::$property[$index]` exists as the parent of a nested write.
///
/// The loaded static container is explicitly acquired before the consuming ensure helper. The
/// returned unique container is then read for write and moved back into the static slot, allowing
/// both autovivification and any hash relocation to remain observable through the property.
fn lower_static_property_parent_fetch_for_write(
    ctx: &mut LoweringContext<'_, '_>,
    receiver: &StaticReceiver,
    property: &str,
    index: &Expr,
    parent_expr: &Expr,
) -> Option<LoweredValue> {
    let span = parent_expr.span;
    let property_ty = static_property_type(ctx, receiver, property)?.codegen_repr();
    let key_ty = index_expr_key_type(ctx, index).codegen_repr();
    let (ensured_ty, read_op) = match &property_ty {
        PhpType::Array(_) => match key_ty {
            PhpType::Int => (property_ty.clone(), Op::ArrayGetForWrite),
            PhpType::Str => (
                promoted_assoc_array_type(property_ty.clone(), PhpType::Mixed),
                Op::HashGetForWrite,
            ),
            PhpType::Mixed | PhpType::Union(_) => {
                (property_ty.clone(), Op::ArrayGetMixedKeyForWrite)
            }
            _ => return None,
        },
        PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed => {
            (property_ty.clone(), Op::HashGetForWrite)
        }
        _ => return None,
    };
    let key = lower_expr(ctx, index);
    let property_value = load_static_property_as(ctx, receiver, property, property_ty, span);
    let property_value =
        crate::ir_lower::ownership::acquire_if_refcounted(ctx, property_value, Some(span));
    let ensured = ctx.emit_value(
        Op::RuntimeCall,
        vec![property_value.value, key.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
        ensured_ty,
        effects_lookup::runtime_effects(),
        Some(span),
    );
    let parent = ctx.emit_value(
        read_op,
        vec![ensured.value, key.value],
        None,
        PhpType::Mixed,
        read_op.default_effects(),
        Some(span),
    );
    store_static_property(ctx, receiver, property, ensured.value, span);
    if ctx.value_is_owning_temporary(key) {
        crate::ir_lower::ownership::release_if_owned(ctx, key, Some(span));
    } else {
        release_persisted_string_operand(ctx, key, span);
    }
    Some(parent)
}

/// Lowers `$local[key]` as the parent of a nested assignment when the local
/// holds a concrete container (`array<mixed>` or a Mixed-valued assoc array):
/// `__rt_array_ensure_elem_for_write` autovivifies the element in write
/// context, the possibly promoted/reallocated container is stored back into
/// the local, and the guaranteed-present element is re-read as the parent
/// cell. Returns `None` for shapes without a concrete for-write lowering
/// (typed element arrays, non-Int/Str key expressions).
pub(super) fn lower_local_parent_fetch_for_write(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    index: &Expr,
    parent_expr: &Expr,
) -> Option<LoweredValue> {
    let span = parent_expr.span;
    let local_ty = ctx.local_type(name);
    match local_ty.codegen_repr() {
        PhpType::Array(elem_ty)
            if matches!(
                elem_ty.codegen_repr(),
                PhpType::Array(_) | PhpType::AssocArray { .. }
            ) && index_expr_key_type(ctx, index) == PhpType::Int =>
        {
            let array_value = ctx.load_local(name, Some(span));
            let key = lower_expr(ctx, index);
            let key = coerce_to_int_at_span(ctx, key, Some(index.span));
            let element_ty = normalize_value_php_type(*elem_ty);
            let parent = ctx.emit_value(
                Op::ArrayGetForWrite,
                vec![array_value.value, key.value],
                None,
                element_ty,
                Op::ArrayGetForWrite.default_effects(),
                Some(span),
            );
            ctx.builder
                .set_value_ownership(parent.value, Ownership::Borrowed);
            Some(parent)
        }
        PhpType::AssocArray { value, .. }
            if matches!(
                value.codegen_repr(),
                PhpType::Array(_) | PhpType::AssocArray { .. }
            ) =>
        {
            let hash_value = ctx.load_local(name, Some(span));
            let key = lower_expr(ctx, index);
            let element_ty = normalize_value_php_type(*value);
            let parent = ctx.emit_value(
                Op::HashGetForWrite,
                vec![hash_value.value, key.value],
                None,
                element_ty,
                Op::HashGetForWrite.default_effects(),
                Some(span),
            );
            ctx.builder
                .set_value_ownership(parent.value, Ownership::Borrowed);
            Some(parent)
        }
        PhpType::Array(elem_ty)
            if elem_ty.codegen_repr() == PhpType::Mixed
                || is_empty_indexed_array_element(elem_ty.as_ref()) =>
        {
            match index_expr_key_type(ctx, index) {
                PhpType::Int => {
                    let array_value = ctx.load_local(name, Some(span));
                    let key = lower_expr(ctx, index);
                    let key = coerce_to_int_at_span(ctx, key, Some(index.span));
                    // Autovivification makes the element type effectively
                    // Mixed even when the array started empty-typed. The
                    // ensure call consumes the loaded container (in-place
                    // mutation or realloc), so the previous boxed owner of a
                    // Mixed-storage slot must be released up front and the
                    // storeback must not release again.
                    let ensured_ty = PhpType::Array(Box::new(PhpType::Mixed));
                    ctx.prepare_mutated_local_owner(name, array_value, ensured_ty.clone(), Some(span));
                    let ensured = ctx.emit_value(
                        Op::RuntimeCall,
                        vec![array_value.value, key.value],
                        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
                        ensured_ty.clone(),
                        effects_lookup::runtime_effects(),
                        Some(span),
                    );
                    ctx.store_prepared_mutated_local(name, ensured, ensured_ty, Some(span));
                    // The element now exists: the in-bounds read returns the
                    // STORED cell (retained) without an undefined-key warning.
                    let cell = ctx.emit_value(
                        Op::ArrayGetForWrite,
                        vec![ensured.value, key.value],
                        None,
                        PhpType::Mixed,
                        Op::ArrayGetForWrite.default_effects(),
                        Some(span),
                    );
                    Some(cell)
                }
                PhpType::Str => {
                    // A literal string key on an indexed local is always a
                    // hash key: promote the local to a Mixed-valued hash
                    // first (mirrors `lower_string_key_array_promotion`),
                    // then ensure the element through the hash path. The
                    // promoted hash flows straight into the ensure call and
                    // is stored back exactly once at the end.
                    let array_value = ctx.load_local(name, Some(span));
                    let assoc_ty = promoted_assoc_array_type(local_ty, PhpType::Mixed);
                    ctx.prepare_mutated_local_owner(name, array_value, assoc_ty.clone(), Some(span));
                    let hash = ctx.emit_value(
                        Op::ArrayToHash,
                        vec![array_value.value],
                        None,
                        assoc_ty.clone(),
                        Op::ArrayToHash.default_effects(),
                        Some(span),
                    );
                    Some(lower_hash_parent_fetch_for_write(ctx, name, hash, assoc_ty, index, span))
                }
                _ => None,
            }
        }
        PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed => {
            let hash_value = ctx.load_local(name, Some(span));
            let assoc_ty = ctx.local_type(name);
            ctx.prepare_mutated_local_owner(name, hash_value, assoc_ty.clone(), Some(span));
            Some(lower_hash_parent_fetch_for_write(ctx, name, hash_value, assoc_ty, index, span))
        }
        _ => None,
    }
}

/// Ensures a hash element exists for a nested write parent, stores the
/// possibly reallocated hash back into the local (the previous owner was
/// already released by `prepare_mutated_local_owner`), and re-reads the
/// stored cell (retained by `Op::HashGetForWrite`) as the parent of the leaf write.
pub(super) fn lower_hash_parent_fetch_for_write(
    ctx: &mut LoweringContext<'_, '_>,
    name: &str,
    hash_value: LoweredValue,
    assoc_ty: PhpType,
    index: &Expr,
    span: Span,
) -> LoweredValue {
    let key = lower_expr(ctx, index);
    let ensured = ctx.emit_value(
        Op::RuntimeCall,
        vec![hash_value.value, key.value],
        Some(Immediate::RuntimeCall(RuntimeCallTarget::ArrayFetchForWrite)),
        assoc_ty.clone(),
        effects_lookup::runtime_effects(),
        Some(span),
    );
    ctx.store_prepared_mutated_local(name, ensured, assoc_ty, Some(span));
    ctx.emit_value(
        Op::HashGetForWrite,
        vec![ensured.value, key.value],
        None,
        PhpType::Mixed,
        Op::HashGetForWrite.default_effects(),
        Some(span),
    )
}
