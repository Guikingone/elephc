//! Purpose:
//! Lowers PHP `array_unshift(array &$array, mixed ...$values)` calls for
//! indexed scalar (`Int`/`Bool`) arrays in the EIR backend.
//!
//! Called from:
//! - `crate::codegen_ir::lower_inst::builtins::arrays::lower_array_unshift()`.
//!
//! Key details:
//! - Real variadic support (H5): each of the `N` prepended values is applied
//!   via its own capacity-aware `__rt_array_unshift_grow` call, in REVERSE
//!   source order, so the first-listed value ends up first — php-verified
//!   `array_unshift($a, 1, 2)` → `[1, 2, ...old]`.
//! - COW and capacity growth are handled entirely inside
//!   `__rt_array_unshift_grow` (mirrors `__rt_array_push_int`); this file only
//!   needs to write the (possibly reallocated) array pointer that each call
//!   returns back into the by-ref source local BETWEEN every value, so a
//!   realloc on any iteration keeps aliases (`$b =& $a`) in sync.
//! - Mutates the caller-visible array after copy-on-write splitting.
//! - Returns the new indexed-array length as PHP `int`.
//! - Supports integer and boolean indexed payloads, matching the existing 8-byte helper.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen_ir::context::FunctionContext;
use crate::codegen_ir::{CodegenIrError, Result};
use crate::ir::{Instruction, ValueId};
use crate::types::PhpType;

use super::super::super::{expect_operand, store_if_result};

/// Lowers `array_unshift()` by ensuring uniqueness/capacity, prepending every
/// variadic value (source order preserved), and returning the new count.
pub(super) fn lower_array_unshift(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count_between(inst, "array_unshift", 2, usize::MAX)?;
    let array = expect_operand(inst, 0)?;
    let elem_ty = array_unshift_element_type(ctx.value_php_type(array)?)?;
    let mut values = Vec::with_capacity(inst.operands.len() - 1);
    for i in 1..inst.operands.len() {
        let value = expect_operand(inst, i)?;
        let value_ty = ctx.value_php_type(value)?.codegen_repr();
        require_array_unshift_value_type(&elem_ty, &value_ty)?;
        values.push(value);
    }
    require_array_unshift_result_type(&inst.result_php_type.codegen_repr())?;

    let source_local = super::source_load_local_slot(ctx, array)?;

    // Prepend in reverse source order: the LAST call places the FIRST-listed
    // value at index 0 (each prepend pushes everything already placed one
    // slot to the right), matching PHP's left-to-right variadic order.
    for &value in values.iter().rev() {
        match ctx.emitter.target.arch {
            Arch::AArch64 => lower_array_unshift_one_aarch64(ctx, array, value)?,
            Arch::X86_64 => lower_array_unshift_one_x86_64(ctx, array, value)?,
        }
        // `__rt_array_unshift_grow` returns the (possibly COW-split and/or
        // reallocated) array pointer in the integer result register — write
        // it back into the SSA value AND the by-ref source local immediately,
        // mirroring the identical pattern `crate::codegen_ir::lower_inst::arrays::lower_array_push`
        // uses for its own (single-value) write-back.
        ctx.store_result_value(array)?;
        if let Some(slot) = source_local {
            ctx.store_value_to_local(slot, array)?;
        }
    }

    emit_load_array_length_as_result(ctx, array)?;
    store_if_result(ctx, inst)
}

/// Returns the supported element payload type for an indexed-array `array_unshift()`.
fn array_unshift_element_type(ty: PhpType) -> Result<PhpType> {
    match ty.codegen_repr() {
        PhpType::Array(elem) => {
            let elem = elem.codegen_repr();
            if matches!(elem, PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never) {
                return Ok(elem);
            }
            Err(CodegenIrError::unsupported(format!(
                "array_unshift indexed-array element PHP type {:?}",
                elem
            )))
        }
        other => Err(CodegenIrError::unsupported(format!(
            "array_unshift for PHP type {:?}",
            other
        ))),
    }
}

/// Verifies a prepended value matches the runtime helper's scalar slot layout.
fn require_array_unshift_value_type(elem_ty: &PhpType, value_ty: &PhpType) -> Result<()> {
    if matches!(value_ty, PhpType::Int | PhpType::Bool)
        && (elem_ty == value_ty || matches!(elem_ty, PhpType::Void | PhpType::Never))
    {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "array_unshift value PHP type {:?} for indexed-array element PHP type {:?}",
        value_ty,
        elem_ty
    )))
}

/// Verifies the lowered `array_unshift()` result carries PHP's integer count metadata.
fn require_array_unshift_result_type(result_ty: &PhpType) -> Result<()> {
    if result_ty == &PhpType::Int {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "array_unshift result PHP type {:?}",
        result_ty
    )))
}

/// Emits the AArch64 `__rt_array_unshift_grow()` runtime call for one scalar value.
fn lower_array_unshift_one_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "x1")?;
    ctx.load_value_to_reg(array, "x0")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift_grow");
    Ok(())
}

/// Emits the x86_64 `__rt_array_unshift_grow()` runtime call for one scalar value.
fn lower_array_unshift_one_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "rsi")?;
    ctx.load_value_to_reg(array, "rdi")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift_grow");
    Ok(())
}

/// Loads the array's current length (header offset 0) into the integer result
/// register — `array_unshift()`'s PHP-visible return value.
fn emit_load_array_length_as_result(ctx: &mut FunctionContext<'_>, array: ValueId) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.emitter.instruction("ldr x0, [x0]");                            // length = the array header's first field
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rax")?;
            ctx.emitter.instruction("mov rax, QWORD PTR [rax]");                // length = the array header's first field
        }
    }
    Ok(())
}
