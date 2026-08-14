//! Purpose:
//! Lowers throw expressions and exception-handler stack instructions.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Lowers expression-form `throw` through the same runtime path as throw terminators.
pub(super) fn lower_throw_exception(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    super::super::lower_term::lower_throw_value(ctx, value)
}

/// Validates a boxed gradual value as a Throwable object and transfers it to the unwinder.
pub(in crate::codegen) fn lower_mixed_throw_value(ctx: &mut FunctionContext<'_>) -> Result<()> {
    let throwable_interface_id = ctx
        .module
        .interface_infos
        .iter()
        .find(|(name, _)| php_symbol_key(name) == php_symbol_key("Throwable"))
        .map(|(_, info)| info.interface_id)
        .ok_or_else(|| CodegenIrError::unsupported("missing Throwable interface metadata"))?;
    let object_label = ctx.next_label("mixed_throw_object");
    let throwable_label = ctx.next_label("mixed_throw_throwable");
    let result_reg = abi::int_result_reg(ctx.emitter);

    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");
            ctx.emitter.instruction(&format!("b.eq {}", object_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");
            ctx.emitter.instruction(&format!("je {}", object_label));
        }
    }
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 0);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    exceptions::emit_error(ctx, "Can only throw objects");

    ctx.emitter.label(&object_label);
    move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 0),
        0,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 1),
        throwable_interface_id as i64,
    );
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 2),
        1,
    );
    abi::emit_call_label(ctx.emitter, "__rt_exception_matches");
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &throwable_label);

    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 16);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    exceptions::emit_error(
        ctx,
        "Cannot throw objects that do not implement Throwable",
    );

    ctx.emitter.label(&throwable_label);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 0);
    abi::emit_incref_if_refcounted(
        ctx.emitter,
        &PhpType::Object("Throwable".to_string()),
    );
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 16);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    abi::emit_store_reg_to_symbol(ctx.emitter, result_reg, "_exc_value", 0);
    abi::emit_call_label(ctx.emitter, "__rt_throw_current");
    Ok(())
}

/// Lowers a static-message catchable PHP `Error` without evaluating later operands.
pub(super) fn lower_throw_error(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if !inst.operands.is_empty() {
        return Err(CodegenIrError::invalid_module(format!(
            "{} expects no operands",
            inst.op.name()
        )));
    }
    let data = expect_data(inst)?;
    let message = ctx
        .module
        .data
        .strings
        .get(data.as_raw() as usize)
        .ok_or_else(|| CodegenIrError::missing_entry("data string", data.as_raw()))?
        .clone();
    exceptions::emit_error(ctx, &message);
    Ok(())
}

/// Lowers a runtime-string catchable PHP `Error` without evaluating later operands.
pub(super) fn lower_throw_error_value(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let message = expect_operand(inst, 0)?;
    exceptions::emit_error_value(ctx, message)
}

/// Pushes an EIR exception handler and branches to the handler block after `longjmp`.
pub(super) fn lower_try_push_handler(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let token = expect_i64(inst)?;
    let handler_offset = ctx.try_handler_offset(token)?;
    let handler_block = BlockId::from_raw(token as u32);
    let handler_label = ctx.block_label_for_id(handler_block)?;
    let scratch = abi::temp_int_reg(ctx.emitter.target);

    ctx.emitter.comment("push EIR exception handler");
    abi::emit_load_symbol_to_reg(ctx.emitter, scratch, "_exc_handler_top", 0);
    abi::store_at_offset(ctx.emitter, scratch, handler_offset);
    abi::emit_load_int_immediate(ctx.emitter, scratch, 0);
    abi::store_at_offset(ctx.emitter, scratch, handler_offset - 8);
    abi::emit_load_symbol_to_reg(ctx.emitter, scratch, "_rt_diag_suppression", 0);
    abi::store_at_offset(
        ctx.emitter,
        scratch,
        handler_offset - TRY_HANDLER_DIAG_DEPTH_OFFSET,
    );
    abi::emit_frame_slot_address(ctx.emitter, scratch, handler_offset);
    abi::emit_store_reg_to_symbol(ctx.emitter, scratch, "_exc_handler_top", 0);
    abi::emit_frame_slot_address(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 0),
        handler_offset - TRY_HANDLER_JMP_BUF_OFFSET,
    );
    ctx.emitter.bl_c("setjmp");
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &handler_label);
    Ok(())
}

/// Pops an EIR exception handler and restores the saved diagnostic-suppression depth.
pub(super) fn lower_try_pop_handler(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let token = expect_i64(inst)?;
    let handler_offset = ctx.try_handler_offset(token)?;
    let scratch = abi::temp_int_reg(ctx.emitter.target);
    ctx.emitter.comment("pop EIR exception handler");
    abi::load_at_offset(ctx.emitter, scratch, handler_offset);
    abi::emit_store_reg_to_symbol(ctx.emitter, scratch, "_exc_handler_top", 0);
    abi::load_at_offset(
        ctx.emitter,
        scratch,
        handler_offset - TRY_HANDLER_DIAG_DEPTH_OFFSET,
    );
    abi::emit_store_reg_to_symbol(ctx.emitter, scratch, "_rt_diag_suppression", 0);
    Ok(())
}

/// Loads the currently active exception object from the runtime exception slot.
pub(super) fn lower_catch_current(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    abi::emit_load_symbol_to_reg(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        "_exc_value",
        0,
    );
    store_if_result(ctx, inst)
}

/// Takes the active exception into an owned SSA result and clears the runtime slot.
pub(super) fn lower_catch_bind(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let result = inst
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("catch_bind missing owned result"))?;
    let result_ty = ctx.value_php_type(result)?;
    abi::emit_load_symbol_to_result(ctx.emitter, "_exc_value", &result_ty);
    ctx.store_result_value(result)?;
    abi::emit_store_zero_to_symbol(ctx.emitter, "_exc_value", 0);
    Ok(())
}
