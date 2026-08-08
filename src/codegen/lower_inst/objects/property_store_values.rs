//! Purpose:
//! Materializes property-store values and emits compact packed-field stores.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Coercions and result restoration preserve the declared slot representation.

use super::*;

/// Loads an SSA value in the shape required by a typed object property store.
pub(super) fn load_property_store_value_to_result(
    ctx: &mut FunctionContext<'_>,
    value: crate::ir::ValueId,
    slot_ty: &PhpType,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?;
    if can_box_value_for_mixed_property(&value_ty, slot_ty) {
        let loaded_ty = ctx.load_value_to_result(value)?.codegen_repr();
        // Property stores do not consume the SSA source; explicit release ops still
        // own temporary cleanup after `prop_set`.
        emit_box_current_value_as_mixed(ctx.emitter, &loaded_ty);
        return Ok(());
    }
    if can_store_boxed_value_for_mixed_property(&value_ty, slot_ty) {
        ctx.load_value_to_result(value)?;
        // Transfer an unreleased owning box into the property; retain borrowed values and
        // temporaries whose explicit EIR cleanup still owns the source reference.
        if !ctx.value_can_own_mixed_box_source(value)? {
            abi::emit_incref_if_refcounted(ctx.emitter, &value_ty);
        }
        return Ok(());
    }
    if can_convert_indexed_array_to_mixed_property(&value_ty, slot_ty) {
        let loaded_ty = ctx.load_value_to_result(value)?.codegen_repr();
        let PhpType::Array(source_elem) = &loaded_ty else {
            return Err(CodegenIrError::unsupported(format!(
                "property array widening from PHP type {:?}",
                value_ty
            )));
        };
        // Give the conversion helper an owned candidate. Its COW split consumes that retain
        // while leaving the SSA source untouched, and the returned unique array transfers
        // directly into the property slot.
        abi::emit_incref_if_refcounted(ctx.emitter, &loaded_ty);
        emit_loaded_indexed_array_to_mixed(ctx, &source_elem.codegen_repr());
        return Ok(());
    }
    if can_store_assoc_array_as_mixed_property(&value_ty, slot_ty) {
        let loaded_ty = ctx.load_value_to_result(value)?.codegen_repr();
        let PhpType::AssocArray {
            value: source_value,
            ..
        } = &loaded_ty
        else {
            return Err(CodegenIrError::unsupported(format!(
                "property associative-array widening from PHP type {:?}",
                value_ty
            )));
        };
        // Retain before a possible COW conversion so `PropSet` never consumes the SSA source.
        // The retained value itself is the property owner when the hash already stores Mixed
        // entries.
        abi::emit_incref_if_refcounted(ctx.emitter, &loaded_ty);
        if source_value.codegen_repr() != PhpType::Mixed {
            emit_loaded_assoc_array_to_mixed(ctx);
        }
        return Ok(());
    }
    if can_store_value_as_tagged_scalar_property(&value_ty, slot_ty) {
        match value_ty.codegen_repr() {
            PhpType::Void | PhpType::Never => {
                crate::codegen::sentinels::emit_tagged_scalar_null(ctx.emitter);
            }
            _ => {
                ctx.load_value_to_result(value)?;
                coerce_loaded_value_to_tagged_scalar(ctx, &value_ty)?;
            }
        }
        return Ok(());
    }
    if can_coerce_tagged_scalar_to_int_property(&value_ty, slot_ty) {
        ctx.load_value_to_result(value)?;
        crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
        return Ok(());
    }
    if matches!(value_ty.codegen_repr(), PhpType::Mixed | PhpType::Union(_)) {
        load_value_to_first_int_arg(ctx, value)?;
        match slot_ty.codegen_repr() {
            PhpType::Str => emit_mixed_string_for_persistent_store(ctx),
            PhpType::Int => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int"),
            PhpType::Bool => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_bool"),
            PhpType::Float => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_float"),
            PhpType::Object(_) => property_values::emit_mixed_object_for_property_store(ctx),
            _ => {}
        }
        return Ok(());
    }
    let loaded_ty = ctx.load_value_to_result(value)?;
    if matches!(slot_ty.codegen_repr(), PhpType::Str) {
        abi::emit_call_label(ctx.emitter, "__rt_str_persist");
        return Ok(());
    }
    if matches!(slot_ty.codegen_repr(), PhpType::Callable) {
        callable_descriptor::emit_retain_current_descriptor(ctx.emitter);
    } else if slot_ty.codegen_repr().is_refcounted() {
        abi::emit_incref_if_refcounted(ctx.emitter, &loaded_ty.codegen_repr());
    }
    Ok(())
}

/// Emits a compact packed-field store without writing object-property metadata words.
pub(super) fn emit_packed_field_store(
    ctx: &mut FunctionContext<'_>,
    value: crate::ir::ValueId,
    slot: &PropertySlot,
    base_reg: &str,
) -> Result<()> {
    match &slot.php_type {
        PhpType::Float => {
            let float_reg = abi::float_result_reg(ctx.emitter);
            abi::emit_push_reg(ctx.emitter, base_reg);
            ctx.load_value_to_reg(value, float_reg)?;
            abi::emit_pop_reg(ctx.emitter, base_reg);
            abi::emit_store_to_address(ctx.emitter, float_reg, base_reg, slot.offset);
        }
        PhpType::Bool
        | PhpType::False
        | PhpType::Int
        | PhpType::Void
        | PhpType::Never
        | PhpType::Pointer(_)
        | PhpType::Resource(_) => {
            let int_reg = abi::int_result_reg(ctx.emitter);
            abi::emit_push_reg(ctx.emitter, base_reg);
            ctx.load_value_to_reg(value, int_reg)?;
            abi::emit_pop_reg(ctx.emitter, base_reg);
            abi::emit_store_to_address(ctx.emitter, int_reg, base_reg, slot.offset);
        }
        _ => {
            return Err(CodegenIrError::unsupported(format!(
                "packed field store for PHP type {:?}",
                slot.php_type
            )))
        }
    }
    Ok(())
}

/// Returns true for property values represented as a single pointer-sized word.
pub(super) fn is_pointer_sized_property_type(php_type: &PhpType) -> bool {
    matches!(
        php_type.codegen_repr(),
        PhpType::Iterable
            | PhpType::Mixed
            | PhpType::Union(_)
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Buffer(_)
            | PhpType::Callable
            | PhpType::Object(_)
            | PhpType::Packed(_)
            | PhpType::Pointer(_)
            | PhpType::Resource(_)
    )
}
