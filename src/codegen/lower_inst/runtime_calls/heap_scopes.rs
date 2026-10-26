//! Purpose:
//! Materializes compiler-private heap mutation scopes through the standard target ABI.
//!
//! Called from:
//! - The typed runtime-operation dispatcher.
//!
//! Key details:
//! - Scalar operands occupy one integer word; native restore stages its string pair explicitly.
//! - Receivers and arrays are typed EIR operands, never PHP-provided raw addresses.

use crate::codegen::{context::FunctionContext, CodegenIrError, Result};
use crate::codegen_support::abi;
use crate::ir::{Instruction, RuntimeCallTarget};

/// Calls the scope helper after staging all operands away from volatile argument registers.
pub(super) fn lower(ctx: &mut FunctionContext<'_>, inst: &Instruction, target: RuntimeCallTarget) -> Result<()> {
    if target == RuntimeCallTarget::NativePropertyRestore { return lower_native_restore(ctx, inst); }
    let entry = match target {
        RuntimeCallTarget::NativePropertyReplace => "__rt_native_property_replace",
        RuntimeCallTarget::HeapScopeBegin => "__rt_heap_scope_begin",
        RuntimeCallTarget::HeapScopeRead => "__rt_heap_scope_read",
        RuntimeCallTarget::HeapScopeRootWritten => "__rt_heap_scope_root_written",
        RuntimeCallTarget::HeapScopeEnd => "__rt_heap_scope_end",
        RuntimeCallTarget::HeapScopeReleaseRoot => "__rt_heap_scope_release_root",
        _ => return Err(CodegenIrError::invalid_module("invalid heap-scope target")),
    };
    for value in &inst.operands {
        ctx.load_value_to_result(*value)?;
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    }
    for index in 0..inst.operands.len() {
        abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, index),
            (inst.operands.len() - index - 1) * 16);
    }
    abi::emit_release_temporary_stack(ctx.emitter, inst.operands.len() * 16);
    abi::emit_call_label(ctx.emitter, entry);
    if let Some(result) = inst.result { ctx.store_result_value(result)?; }
    Ok(())
}

/// Expands the native property name string pair into four C ABI integer arguments.
fn lower_native_restore(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ctx.load_value_to_result(inst.operands[0])?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    ctx.load_value_to_result(inst.operands[1])?;
    let (pointer, length) = abi::string_result_regs(ctx.emitter);
    abi::emit_push_reg_pair(ctx.emitter, pointer, length);
    ctx.load_value_to_result(inst.operands[2])?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    for (index, offset) in [32, 16, 24, 0].into_iter().enumerate() {
        abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, index), offset);
    }
    abi::emit_release_temporary_stack(ctx.emitter, 48);
    abi::emit_call_label(ctx.emitter, "__rt_native_property_restore");
    if let Some(result) = inst.result { ctx.store_result_value(result)?; }
    Ok(())
}
