//! Purpose:
//! Lowers typed native heap metric calls to allocation-free runtime helpers.
//!
//! Called from:
//! - `runtime_functions::group_11` for the five memory operations.
//!
//! Key details:
//! - Optional boolean arguments have already passed shared call normalization.

use super::*;
use crate::ir::RuntimeFnId;

/// Materializes the optional arena flag and calls the matching native heap helper.
pub(crate) fn lower(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    target: RuntimeFnId,
) -> Result<()> {
    let (symbol, has_flag) = match target {
        RuntimeFnId::MemoryGetUsage => ("__rt_memory_get_usage", true),
        RuntimeFnId::MemoryGetPeakUsage => ("__rt_memory_get_peak_usage", true),
        RuntimeFnId::MemoryResetPeakUsage => ("__rt_memory_reset_peak_usage", false),
        RuntimeFnId::MemoryGetAllocations => ("__rt_memory_get_allocations", false),
        RuntimeFnId::MemoryGetFrees => ("__rt_memory_get_frees", false),
        _ => return Err(CodegenIrError::invalid_module("expected a memory runtime operation")),
    };
    shared::ensure_arg_count_between(inst, symbol, 0, usize::from(has_flag))?;
    if has_flag {
        if let Some(&flag) = inst.operands.first() {
            predicates::emit_value_truthiness(ctx, flag, "memory real_usage")?;
            super::super::move_int_result_to_first_arg(ctx);
        } else {
            let reg = if ctx.emitter.target.arch == Arch::AArch64 { "x0" } else { "rdi" };
            abi::emit_load_int_immediate(ctx.emitter, reg, 0);
        }
    }
    abi::emit_call_label(ctx.emitter, symbol);
    store_if_result(ctx, inst)
}
