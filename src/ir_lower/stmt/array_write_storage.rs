//! Purpose:
//! Array push and indexed-array local storage normalization.
//!
//! Called from:
//! - `crate::ir_lower::stmt`.
//!
//! Key details:
//! - Preserves statement ordering, CFG shape, EIR effects, and ownership contracts.

use super::*;

/// Lowers `$array[] = value`.
pub(super) fn lower_array_push(ctx: &mut LoweringContext<'_, '_>, array: &str, value: &Expr, span: Span) {
    // Value first, container second: see `lower_array_assign` for the stale-pointer failure.
    let value = lower_array_reference_or_value(ctx, value);
    if ctx.builder.insertion_block_is_terminated() {
        return;
    }
    let array_value = ctx.load_local(array, Some(span));
    let op = if array_value.ir_type == IrType::Heap(crate::ir::IrHeapKind::Array) {
        Op::ArrayPush
    } else if array_value.ir_type == IrType::Heap(crate::ir::IrHeapKind::Mixed) {
        Op::MixedArrayAppend
    } else {
        Op::RuntimeCall
    };
    if op == Op::ArrayPush {
        let (array_value, updated_ty, needs_storeback) =
            if ref_bound_mixed_indexed_array_write(ctx, array, value) {
                (array_value, Some(ctx.local_type(array)), true)
            } else {
                prepare_indexed_array_local_write(ctx, array_value, value, span)
            };
        ctx.emit_void(
            op,
            vec![array_value.value, value.value],
            None,
            op.default_effects(),
            Some(span),
        );
        let elem_ty = indexed_array_write_element_type(ctx, array_value, updated_ty.as_ref());
        finish_indexed_array_local_write(
            ctx,
            array,
            array_value,
            updated_ty,
            needs_storeback,
            span,
        );
        release_indexed_array_write_operand(ctx, elem_ty.as_ref(), value, span);
        return;
    }
    let operands = if op == Op::RuntimeCall {
        let receiver_ty = ctx.local_type(array);
        object_push_operands(ctx, &receiver_ty, array_value, value, span)
    } else {
        vec![array_value.value, value.value]
    };
    ctx.emit_void(op, operands, None, op.default_effects(), Some(span));
    release_persisted_string_operand(ctx, value, span);
}

/// Operands of the runtime call that appends `value` to an object receiver (`$o[] = $v`).
///
/// php runs `$obj[] = $v` as `$obj->offsetSet(null, $v)`. The two-operand form is the SPL
/// containers' native `append()`, which a user `ArrayAccess` class does not have: the backend
/// refused the push with "method call to unknown method Bag::append" (Doctrine's
/// `ArrayCollection`, league/commonmark's block parsers).
pub(super) fn object_push_operands(
    ctx: &mut LoweringContext<'_, '_>,
    receiver_ty: &PhpType,
    receiver: LoweredValue,
    value: LoweredValue,
    span: Span,
) -> Vec<crate::ir::ValueId> {
    if !push_goes_through_offset_set(ctx, receiver_ty) {
        return vec![receiver.value, value.value];
    }
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
    vec![receiver.value, null_key, value.value]
}

/// Returns whether `$value[] = $v` on a value of this static type is `offsetSet(null, $v)`: an
/// `ArrayAccess` object other than the SPL containers the runtime backs directly, whose push
/// lowers to their native `append()`. Anything else keeps its existing lowering — an array or a
/// hash is written in place, and an object that is NOT `ArrayAccess` must still raise php's
/// "Cannot use object of type … as array".
fn push_goes_through_offset_set(ctx: &LoweringContext<'_, '_>, ty: &PhpType) -> bool {
    let PhpType::Object(class_name) = ty.codegen_repr() else {
        return false;
    };
    if !type_satisfies_array_access_for_ir(ctx, &ty.codegen_repr()) {
        return false;
    }
    !matches!(
        crate::names::php_symbol_key(class_name.trim_start_matches('\\')).as_str(),
        "arrayobject"
            | "arrayiterator"
            | "recursivearrayiterator"
            | "spldoublylinkedlist"
            | "splqueue"
            | "splstack"
    )
}

/// Prepares an indexed-array local for an offset assignment.
pub(super) fn prepare_indexed_array_local_set(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<PhpType>, bool) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    if indexed_array_refcounted_set_needs_mixed_conversion(&current_ty, &value_ty) {
        let updated_ty = PhpType::Array(Box::new(PhpType::Mixed));
        let converted = ctx.emit_value(
            Op::ArrayToMixed,
            vec![array_value.value],
            None,
            updated_ty.clone(),
            Op::ArrayToMixed.default_effects(),
            Some(span),
        );
        return (converted, Some(updated_ty), true);
    }
    prepare_indexed_array_local_write(ctx, array_value, value, span)
}

/// Coerces miss-capable scalar reads before writing them into a concrete indexed-array slot.
pub(super) fn coerce_indexed_array_set_value(
    ctx: &mut LoweringContext<'_, '_>,
    array_ty: &PhpType,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match array_ty.codegen_repr() {
        PhpType::Array(elem_ty)
            if elem_ty.codegen_repr() == PhpType::Int
                && matches!(
                    ctx.builder.value_php_type(value.value).codegen_repr(),
                    PhpType::Mixed | PhpType::TaggedScalar | PhpType::Union(_)
                ) =>
        {
            coerce_owned_box_to_int(ctx, value, span)
        }
        _ => value,
    }
}

/// Coerces a boxed or nullable value to `int` and releases the source when it is a fresh
/// temporary (`$a[0] + 1` over an `int|null` element boxes its sum): the int is all the write
/// keeps, and the box leaked one cell per write. A value still owned elsewhere is left alone.
fn coerce_owned_box_to_int(
    ctx: &mut LoweringContext<'_, '_>,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    let is_fresh = ctx.value_is_owning_temporary(value);
    let coerced = coerce_to_int(ctx, value, span);
    if is_fresh && coerced.value != value.value {
        crate::ir_lower::ownership::release_if_owned(ctx, value, span);
    }
    coerced
}

/// Returns true when a refcounted indexed-array assignment should use Mixed slots.
pub(super) fn indexed_array_refcounted_set_needs_mixed_conversion(
    current_ty: &PhpType,
    value_ty: &PhpType,
) -> bool {
    let PhpType::Array(elem_ty) = current_ty.codegen_repr() else {
        return false;
    };
    let elem_ty = elem_ty.codegen_repr();
    let value_ty = value_ty.codegen_repr();
    elem_ty != value_ty
        && elem_ty != PhpType::Mixed
        && elem_ty.is_refcounted()
        && value_ty.is_refcounted()
}

/// Converts typed indexed arrays to Mixed when a local write would make them heterogeneous.
pub(in crate::ir_lower) fn prepare_indexed_array_local_write(
    ctx: &mut LoweringContext<'_, '_>,
    array_value: LoweredValue,
    value: LoweredValue,
    span: Span,
) -> (LoweredValue, Option<PhpType>, bool) {
    let current_ty = ctx.builder.value_php_type(array_value.value);
    let value_ty = ctx.builder.value_php_type(value.value);
    let Some(updated_ty) = indexed_array_write_updated_type(current_ty.clone(), value_ty) else {
        return (array_value, None, false);
    };
    if !indexed_array_write_needs_mixed_conversion(&current_ty, &updated_ty) {
        return (array_value, Some(updated_ty), false);
    }
    let converted = ctx.emit_value(
        Op::ArrayToMixed,
        vec![array_value.value],
        None,
        updated_ty.clone(),
        Op::ArrayToMixed.default_effects(),
        Some(span),
    );
    (converted, Some(updated_ty), true)
}

/// Updates local type facts and emits explicit storeback for converted array writes.
pub(in crate::ir_lower) fn finish_indexed_array_local_write(
    ctx: &mut LoweringContext<'_, '_>,
    array: &str,
    array_value: LoweredValue,
    updated_ty: Option<PhpType>,
    needs_storeback: bool,
    span: Span,
) {
    let Some(updated_ty) = updated_ty else {
        return;
    };
    if needs_storeback {
        ctx.store_mutated_local(array, array_value, updated_ty, Some(span));
    } else {
        ctx.set_local_type(array, updated_ty);
    }
}

/// Returns true when a ref-bound indexed array should keep its caller-visible element type.
pub(in crate::ir_lower) fn ref_bound_mixed_indexed_array_write(
    ctx: &LoweringContext<'_, '_>,
    array: &str,
    value: LoweredValue,
) -> bool {
    ctx.is_ref_bound_local(array)
        && matches!(
            ctx.builder.value_php_type(value.value).codegen_repr(),
            PhpType::Mixed | PhpType::Union(_)
        )
}

/// Returns the refined array type after writing a value into an indexed array.
pub(super) fn indexed_array_write_updated_type(current_ty: PhpType, value_ty: PhpType) -> Option<PhpType> {
    match current_ty.codegen_repr() {
        PhpType::Array(elem_ty) if is_empty_indexed_array_element(elem_ty.as_ref()) => Some(
            PhpType::Array(Box::new(normalize_empty_array_write_element_type(value_ty))),
        ),
        PhpType::Array(elem_ty) if elem_ty.codegen_repr() == PhpType::Mixed => None,
        PhpType::Array(elem_ty) => {
            let elem_ty = elem_ty.codegen_repr();
            if elem_ty == value_ty.codegen_repr() {
                return None;
            }
            let value_ty = normalize_array_write_element_type(value_ty.codegen_repr());
            if elem_ty == value_ty {
                None
            } else {
                Some(PhpType::Array(Box::new(PhpType::Mixed)))
            }
        }
        _ => None,
    }
}

/// Returns true when an indexed-array write needs runtime conversion to Mixed slots.
pub(super) fn indexed_array_write_needs_mixed_conversion(current_ty: &PhpType, updated_ty: &PhpType) -> bool {
    let PhpType::Array(current_elem) = current_ty.codegen_repr() else {
        return false;
    };
    let PhpType::Array(updated_elem) = updated_ty.codegen_repr() else {
        return false;
    };
    updated_elem.codegen_repr() == PhpType::Mixed && current_elem.codegen_repr() != PhpType::Mixed
}

/// Returns true for the placeholder element type used by empty indexed arrays.
pub(super) fn is_empty_indexed_array_element(elem_ty: &PhpType) -> bool {
    matches!(elem_ty.codegen_repr(), PhpType::Never | PhpType::Void)
}

/// Preserves the first concrete value type written into an empty indexed array.
pub(super) fn normalize_empty_array_write_element_type(item_type: PhpType) -> PhpType {
    normalize_materialized_element_type(item_type)
}

/// Coerces a boxed or nullable-int value written into a `static` hash whose elements are `int`
/// to that element type, so the write keeps the slot's program-wide storage type.
pub(super) fn coerce_static_hash_set_value(
    ctx: &mut LoweringContext<'_, '_>,
    hash_ty: &PhpType,
    value: LoweredValue,
    span: Option<Span>,
) -> LoweredValue {
    match hash_ty.codegen_repr() {
        PhpType::AssocArray { value: elem_ty, .. }
            if elem_ty.codegen_repr() == PhpType::Int
                && matches!(
                    ctx.builder.value_php_type(value.value).codegen_repr(),
                    PhpType::Mixed | PhpType::TaggedScalar | PhpType::Union(_)
                ) =>
        {
            coerce_owned_box_to_int(ctx, value, span)
        }
        _ => value,
    }
}
