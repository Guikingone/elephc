//! Purpose:
//! Lowers PHP `array_unshift()` calls for indexed arrays in the Phase 04 EIR backend.
//! Reuses the legacy scalar-slot runtime helper after copy-on-write preparation.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays::lower_array_unshift()`.
//!
//! Key details:
//! - Mutates the caller-visible array after copy-on-write splitting.
//! - Grows the payload before prepending: `__rt_array_unshift` shifts slots and bumps the
//!   length without a capacity check, so a full array would write past its allocation.
//! - Returns the new indexed-array length as PHP `int`.
//! - Supports integer and boolean indexed payloads, matching the existing 8-byte helper.

use crate::codegen::abi;
use crate::codegen::platform::Arch;
use crate::codegen::context::FunctionContext;
use crate::codegen::{CodegenIrError, Result};
use crate::ir::{Instruction, ValueId};
use crate::types::PhpType;

use super::super::super::{expect_operand, store_if_result};

/// Lowers `array_unshift()` by ensuring uniqueness, prepending one scalar value, and returning count.
pub(super) fn lower_array_unshift(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::super::ensure_arg_count(inst, "array_unshift", 2)?;
    let array = expect_operand(inst, 0)?;
    let value = expect_operand(inst, 1)?;
    let elem_ty = array_unshift_element_type(ctx.value_php_type(array)?)?;
    let value_ty = ctx.value_php_type(value)?.codegen_repr();
    require_array_unshift_value_type(&elem_ty, &value_ty)?;
    require_array_unshift_result_type(&inst.result_php_type.codegen_repr())?;
    let source_local = super::source_load_local_slot(ctx, array)?;
    ensure_unique_array_unshift_source(ctx, array)?;
    ensure_array_unshift_capacity(ctx, array)?;
    if let Some(slot) = source_local {
        ctx.store_value_to_local(slot, array)?;
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_array_unshift_aarch64(ctx, array, value)?,
        Arch::X86_64 => lower_array_unshift_x86_64(ctx, array, value)?,
    }
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

/// Verifies the prepended value matches the runtime helper's scalar slot layout.
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

/// Splits a shared indexed array before `array_unshift()` mutates its slots.
fn ensure_unique_array_unshift_source(ctx: &mut FunctionContext<'_>, array: ValueId) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_ensure_unique");
    ctx.store_result_value(array)
}

/// Guarantees the unique indexed array has room for the extra `array_unshift()` slot.
///
/// `__rt_array_unshift` shifts every live payload one slot to the right and increments the
/// logical length, but it never checks capacity. On a full array that wrote one element past
/// the payload into the neighbouring heap block and left `length > capacity`, so the next
/// copy-on-write split (`__rt_array_clone_shallow` allocates `capacity` slots and copies
/// `length` slots) both overflowed the clone and read back adjacent heap header words as PHP
/// values. `__rt_array_grow` at least doubles capacity, so one conditional growth is always
/// enough for the single prepended element. It returns a possibly-relocated pointer, which the
/// caller stores back into the array value before the local-slot write-back runs.
fn ensure_array_unshift_capacity(ctx: &mut FunctionContext<'_>, array: ValueId) -> Result<()> {
    let done_label = ctx.next_label("array_unshift_capacity_ok");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.emitter.instruction("ldr x9, [x0]");                            // load the current indexed-array logical length
            ctx.emitter.instruction("ldr x10, [x0, #8]");                       // load the current indexed-array slot capacity
            ctx.emitter.instruction("cmp x9, x10");                             // does the prepended element still fit inside the payload?
            ctx.emitter.instruction(&format!("b.lt {}", done_label));           // a spare slot means the shift stays inside the allocation
            abi::emit_call_label(ctx.emitter, "__rt_array_grow");
            ctx.store_result_value(array)?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
            ctx.emitter.instruction("mov r10, QWORD PTR [rdi]");                // load the current indexed-array logical length
            ctx.emitter.instruction("mov r11, QWORD PTR [rdi + 8]");            // load the current indexed-array slot capacity
            ctx.emitter.instruction("cmp r10, r11");                            // does the prepended element still fit inside the payload?
            ctx.emitter.instruction(&format!("jb {}", done_label));             // a spare slot means the shift stays inside the allocation
            abi::emit_call_label(ctx.emitter, "__rt_array_grow");
            ctx.store_result_value(array)?;
        }
    }
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Emits the AArch64 `array_unshift()` runtime call for scalar indexed arrays.
fn lower_array_unshift_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "x1")?;
    ctx.load_value_to_reg(array, "x0")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift");
    Ok(())
}

/// Emits the x86_64 `array_unshift()` runtime call for scalar indexed arrays.
fn lower_array_unshift_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    value: ValueId,
) -> Result<()> {
    ctx.load_value_to_reg(value, "rsi")?;
    ctx.load_value_to_reg(array, "rdi")?;
    abi::emit_call_label(ctx.emitter, "__rt_array_unshift");
    Ok(())
}
