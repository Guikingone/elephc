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
            match ctx.raw_value_php_type(flag)? {
                PhpType::Mixed | PhpType::Union(_) => validate_dynamic_flag(ctx, flag, target)?,
                PhpType::Bool | PhpType::False | PhpType::Int | PhpType::Float | PhpType::Str
                | PhpType::Void | PhpType::Never | PhpType::TaggedScalar => {}
                // EIR can refine a mixed-returning call back to its concrete container type.
                _ => emit_flag_type_error(ctx, target),
            }
            predicates::emit_value_truthiness(ctx, flag, "memory real_usage")?;
            super::super::move_int_result_to_first_arg(ctx);
        } else {
            let reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
            abi::emit_load_int_immediate(ctx.emitter, reg, 0);
        }
    }
    abi::emit_call_label(ctx.emitter, symbol);
    store_if_result(ctx, inst)
}

/// Rejects container flags before PHP truthiness can silently accept them.
fn validate_dynamic_flag(
    ctx: &mut FunctionContext<'_>,
    flag: ValueId,
    target: RuntimeFnId,
) -> Result<()> {
    ctx.load_value_to_result(flag)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let valid = ctx.next_label("memory_scalar_flag");
    // -- weak bool parameters admit scalar tags zero through three and null --
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #3");                              // ordinary scalars can be converted without a type error
            ctx.emitter.instruction(&format!("b.ls {valid}"));                  // preserve integer, string, float, and boolean inputs
            ctx.emitter.instruction("cmp x0, #8");                              // null is accepted by weak builtin boolean parameters
            ctx.emitter.instruction(&format!("b.eq {valid}"));                  // leave null conversion to the ordinary truthiness path
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 3");                              // scalar tags zero through three satisfy weak bool parameters
            ctx.emitter.instruction(&format!("jbe {valid}"));                   // reject containers before testing their truthiness
            ctx.emitter.instruction("cmp rax, 8");                              // null also coerces to false for weak builtin calls
            ctx.emitter.instruction(&format!("je {valid}"));                    // reload and coerce the original borrowed scalar below
        }
    }
    emit_flag_type_error(ctx, target);
    ctx.emitter.label(&valid);
    Ok(())
}

/// Uses the same catchable error for boxed and EIR-refined container flags.
fn emit_flag_type_error(ctx: &mut FunctionContext<'_>, target: RuntimeFnId) {
    exceptions::emit_type_error(ctx, &format!(
        "{}(): Argument #1 ($real_usage) must be of type bool", target.as_eir(),
    ));
}
