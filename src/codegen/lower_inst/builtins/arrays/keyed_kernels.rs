//! Purpose:
//! Lowers the fused packed-int map+sum and filter-count kernels selected by AST fusion.
//!
//! Called from:
//! - `crate::codegen::lower_inst::runtime_functions::group_01`.
//!
//! Key details:
//! - Both kernels take the array pointer and one literal integer parameter.
//! - The map+sum kernel returns a fresh boxed Mixed cell; the filter-count kernel
//!   returns a raw PHP int.

use crate::codegen::context::FunctionContext;
use crate::codegen::platform::Arch;
use crate::codegen::{abi, Result};
use crate::ir::Instruction;

use super::{expect_operand, store_if_result};

/// Lowers `array_map_sum_add(array, offset)` into the fused runtime kernel.
pub(crate) fn lower_array_map_sum_add(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_two_operand_kernel(ctx, inst, "__rt_array_map_sum_add")
}

/// Lowers the raw-int observation of the fused map+sum kernel.
pub(crate) fn lower_array_map_sum_add_int(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_two_operand_kernel(ctx, inst, "__rt_array_map_sum_add_int")
}

/// Lowers the declared-int map+sum overflow preflight.
pub(crate) fn lower_array_map_sum_add_safe(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_two_operand_kernel(ctx, inst, "__rt_array_map_sum_add_safe")
}

/// Lowers `array_filter_count_mod(array, divisor)` into the fused runtime kernel.
pub(crate) fn lower_array_filter_count_mod(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    lower_two_operand_kernel(ctx, inst, "__rt_array_filter_count_mod")
}

/// Loads the array pointer and literal parameter into the target argument registers.
fn lower_two_operand_kernel(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    symbol: &str,
) -> Result<()> {
    let array = expect_operand(inst, 0)?;
    let parameter = expect_operand(inst, 1)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(array, "x0")?;
            ctx.load_value_to_reg(parameter, "x1")?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(array, "rdi")?;
            ctx.load_value_to_reg(parameter, "rsi")?;
        }
    }
    abi::emit_call_label(ctx.emitter, symbol);
    store_if_result(ctx, inst)
}
