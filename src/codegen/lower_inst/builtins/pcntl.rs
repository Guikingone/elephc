//! Purpose:
//! Lowers recognized PCNTL calls into explicit runtime-fatal stubs for AOT targets.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::lower_builtin_call()`.
//!
//! Key details:
//! - PCNTL calls must not block compilation of runtime-dead code paths.
//! - Reaching one of these stubs terminates with a precise diagnostic instead of silently
//!   pretending that signal handlers or alarms were installed.

use crate::codegen::{abi, emit_box_current_value_as_mixed, CodegenIrError, Result};
use crate::ir::Instruction;
use crate::types::PhpType;

use super::super::super::context::FunctionContext;
use super::store_if_result;

/// Emits a runtime-fatal PCNTL stub and a structurally valid unreachable fallback result.
pub(super) fn lower_deferred_pcntl(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    validate_pcntl_operand_count(inst, name)?;
    let message = format!(
        "Fatal error: {}() is not supported by the elephc AOT runtime\n",
        name
    );
    super::super::emit_unsupported_feature_fatal(ctx, &message);

    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    if name == "pcntl_signal_get_handler" {
        emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Int);
    }
    store_if_result(ctx, inst)
}

/// Revalidates the legacy PCNTL arities at the backend boundary.
fn validate_pcntl_operand_count(inst: &Instruction, name: &str) -> Result<()> {
    let valid = match name {
        "pcntl_signal" => (2..=3).contains(&inst.operands.len()),
        "pcntl_alarm" | "pcntl_signal_get_handler" => inst.operands.len() == 1,
        "pcntl_async_signals" => inst.operands.len() <= 1,
        _ => false,
    };
    if valid {
        return Ok(());
    }
    Err(CodegenIrError::invalid_module(format!(
        "{} received {} lowered operands",
        name,
        inst.operands.len()
    )))
}
