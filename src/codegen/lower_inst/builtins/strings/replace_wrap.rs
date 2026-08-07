//! Purpose:
//! Lowers whole-string replacement, padding, and word wrapping builtins.
//!
//! Called from:
//! - The string builtin lowering facade.
//!
//! Key details:
//! - Optional pad, break, width, and mode arguments use target-specific ABI materialization.

use super::*;

/// Lowers `str_replace()`/`str_ireplace()` with three string operands.
pub(crate) fn lower_string_replace(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    runtime_label: &str,
) -> Result<()> {
    if inst.operands.len() != 3 {
        return Err(CodegenIrError::invalid_module(format!(
            "{} expected 3 args, got {}",
            name,
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_string_replace_aarch64(ctx, inst, name)?,
        Arch::X86_64 => lower_string_replace_x86_64(ctx, inst, name)?,
    }
    abi::emit_call_label(ctx.emitter, runtime_label);
    store_if_result(ctx, inst)
}

/// Lowers `wordwrap(string, width?, break?, cut?)` through the shared runtime helper.
pub(crate) fn lower_wordwrap(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.is_empty() || inst.operands.len() > 4 {
        return Err(CodegenIrError::invalid_module(format!(
            "wordwrap expected 1 to 4 args, got {}",
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_wordwrap_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_wordwrap_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_wordwrap");
    store_if_result(ctx, inst)
}

/// Lowers `str_pad(string, length, pad_string?, pad_type?)` through the shared runtime helper.
pub(crate) fn lower_str_pad(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() < 2 || inst.operands.len() > 4 {
        return Err(CodegenIrError::invalid_module(format!(
            "str_pad expected 2 to 4 args, got {}",
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_str_pad_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_str_pad_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_pad");
    store_if_result(ctx, inst)
}
/// Materializes AArch64 `str_replace`-family runtime arguments.
pub(super) fn lower_string_replace_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, name, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the search string while materializing replacement and subject
    load_string_arg_to_regs(ctx, inst, 1, name, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the replacement string while materializing the subject
    load_string_arg_to_regs(ctx, inst, 2, name, "x1", "x2")?;
    ctx.emitter.instruction("mov x5, x1");                                      // pass the subject string pointer as the third runtime string argument
    ctx.emitter.instruction("mov x6, x2");                                      // pass the subject string length as the third runtime string argument
    ctx.emitter.instruction("ldp x3, x4, [sp], #16");                           // restore replacement into the secondary runtime string argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore search into the primary runtime string argument
    Ok(())
}

/// Materializes x86_64 `str_replace`-family runtime arguments.
pub(super) fn lower_string_replace_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, name, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_string_arg_to_regs(ctx, inst, 1, name, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_string_arg_to_regs(ctx, inst, 2, name, "rax", "rdx")?;
    ctx.emitter.instruction("mov rcx, rax");                                    // pass the subject string pointer as the third runtime string argument
    ctx.emitter.instruction("mov r8, rdx");                                     // pass the subject string length as the third runtime string argument
    abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes AArch64 `str_pad()` runtime arguments.
pub(super) fn lower_str_pad_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let input = expect_operand(inst, 0)?;
    let target_length = expect_operand(inst, 1)?;
    load_value_as_string_to_regs(ctx, input, "str_pad", "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the input string while materializing length and pad arguments
    load_as_int(ctx, target_length, "str_pad length")?;
    abi::emit_push_reg(ctx.emitter, "x0");
    materialize_str_pad_pad_string_aarch64(ctx, inst)?;
    materialize_str_pad_type_aarch64(ctx, inst)?;
    ctx.emitter.instruction("ldp x3, x4, [sp], #16");                           // restore the pad string into secondary runtime argument registers
    abi::emit_pop_reg(ctx.emitter, "x5");
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the input string into primary runtime argument registers
    Ok(())
}

/// Materializes the AArch64 `str_pad()` pad-string argument.
pub(super) fn materialize_str_pad_pad_string_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 3 {
        let pad_string = expect_operand(inst, 2)?;
        load_value_as_string_to_regs(ctx, pad_string, "str_pad", "x1", "x2")?;
    } else {
        let (label, len) = ctx.data.add_string(b" ");
        abi::emit_symbol_address(ctx.emitter, "x1", &label);
        abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
    }
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the pad string while materializing the optional pad type
    Ok(())
}

/// Materializes the AArch64 `str_pad()` pad-type argument.
pub(super) fn materialize_str_pad_type_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 4 {
        let pad_type = expect_operand(inst, 3)?;
        load_as_int(ctx, pad_type, "str_pad pad_type")?;
        ctx.emitter.instruction("mov x7, x0");                                  // pass the requested STR_PAD mode to the runtime helper
    } else {
        ctx.emitter.instruction("mov x7, #1");                                  // default to STR_PAD_RIGHT when pad_type is omitted
    }
    Ok(())
}

/// Materializes x86_64 `str_pad()` runtime arguments.
pub(super) fn lower_str_pad_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let input = expect_operand(inst, 0)?;
    let target_length = expect_operand(inst, 1)?;
    load_value_as_string_to_regs(ctx, input, "str_pad", "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_as_int(ctx, target_length, "str_pad length")?;
    abi::emit_push_reg(ctx.emitter, "rax");
    materialize_str_pad_pad_string_x86_64(ctx, inst)?;
    materialize_str_pad_type_x86_64(ctx, inst)?;
    abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
    abi::emit_pop_reg(ctx.emitter, "rcx");
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes the x86_64 `str_pad()` pad-string argument.
pub(super) fn materialize_str_pad_pad_string_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 3 {
        let pad_string = expect_operand(inst, 2)?;
        load_value_as_string_to_regs(ctx, pad_string, "str_pad", "rax", "rdx")?;
    } else {
        let (label, len) = ctx.data.add_string(b" ");
        abi::emit_symbol_address(ctx.emitter, "rax", &label);
        abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
    }
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes the x86_64 `str_pad()` pad-type argument.
pub(super) fn materialize_str_pad_type_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 4 {
        let pad_type = expect_operand(inst, 3)?;
        load_as_int(ctx, pad_type, "str_pad pad_type")?;
        ctx.emitter.instruction("mov r8, rax");                                 // pass the requested STR_PAD mode to the runtime helper
    } else {
        ctx.emitter.instruction("mov r8, 1");                                   // default to STR_PAD_RIGHT when pad_type is omitted
    }
    Ok(())
}

/// Materializes AArch64 `wordwrap()` runtime arguments.
pub(super) fn lower_wordwrap_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let input = expect_string_operand(ctx, inst, 0, "wordwrap")?;
    ctx.load_string_value_to_regs(input, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the input string while materializing width and break arguments
    materialize_wordwrap_width_aarch64(ctx, inst)?;
    materialize_wordwrap_break_aarch64(ctx, inst)?;
    if inst.operands.len() >= 4 {
        let cut = expect_operand(inst, 3)?;
        load_as_int(ctx, cut, "wordwrap cut")?;
        ctx.emitter.instruction("mov x6, x0");                                  // pass the requested cut_long_words flag to the runtime helper
    } else {
        ctx.emitter.instruction("mov x6, #0");                                  // default cut_long_words to false when omitted
    }
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the input string into primary runtime argument registers
    Ok(())
}

/// Materializes the AArch64 wordwrap width argument.
pub(super) fn materialize_wordwrap_width_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 2 {
        let width = expect_operand(inst, 1)?;
        load_as_int(ctx, width, "wordwrap width")?;
        ctx.emitter.instruction("mov x3, x0");                                  // pass the requested wrap width to the runtime helper
    } else {
        ctx.emitter.instruction("mov x3, #75");                                 // use PHP's default wrap width when omitted
    }
    Ok(())
}

/// Materializes the AArch64 wordwrap break-string argument.
pub(super) fn materialize_wordwrap_break_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 3 {
        let break_string = expect_string_operand(ctx, inst, 2, "wordwrap")?;
        ctx.load_string_value_to_regs(break_string, "x1", "x2")?;
        ctx.emitter.instruction("mov x4, x1");                                  // pass the break-string pointer to the runtime helper
        ctx.emitter.instruction("mov x5, x2");                                  // pass the break-string length to the runtime helper
    } else {
        let (label, len) = ctx.data.add_string(b"\n");
        abi::emit_symbol_address(ctx.emitter, "x4", &label);
        abi::emit_load_int_immediate(ctx.emitter, "x5", len as i64);
    }
    Ok(())
}

/// Materializes x86_64 `wordwrap()` runtime arguments.
pub(super) fn lower_wordwrap_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let input = expect_string_operand(ctx, inst, 0, "wordwrap")?;
    ctx.load_string_value_to_regs(input, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    materialize_wordwrap_width_x86_64(ctx, inst)?;
    materialize_wordwrap_break_x86_64(ctx, inst)?;
    if inst.operands.len() >= 4 {
        let cut = expect_operand(inst, 3)?;
        load_as_int(ctx, cut, "wordwrap cut")?;
        ctx.emitter.instruction("mov r9, rax");                                 // pass the requested cut_long_words flag to the runtime helper
    } else {
        ctx.emitter.instruction("mov r9, 0");                                   // default cut_long_words to false when omitted
    }
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes the x86_64 wordwrap width argument.
pub(super) fn materialize_wordwrap_width_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 2 {
        let width = expect_operand(inst, 1)?;
        load_as_int(ctx, width, "wordwrap width")?;
        ctx.emitter.instruction("mov rdi, rax");                                // pass the requested wrap width to the runtime helper
    } else {
        ctx.emitter.instruction("mov rdi, 75");                                 // use PHP's default wrap width when omitted
    }
    Ok(())
}

/// Materializes the x86_64 wordwrap break-string argument.
pub(super) fn materialize_wordwrap_break_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 3 {
        let break_string = expect_string_operand(ctx, inst, 2, "wordwrap")?;
        ctx.load_string_value_to_regs(break_string, "rax", "rdx")?;
        ctx.emitter.instruction("mov rcx, rax");                                // pass the break-string pointer to the runtime helper
        ctx.emitter.instruction("mov r8, rdx");                                 // pass the break-string length to the runtime helper
    } else {
        let (label, len) = ctx.data.add_string(b"\n");
        abi::emit_symbol_address(ctx.emitter, "rcx", &label);
        abi::emit_load_int_immediate(ctx.emitter, "r8", len as i64);
    }
    Ok(())
}
