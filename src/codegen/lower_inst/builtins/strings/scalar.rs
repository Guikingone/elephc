//! Purpose:
//! Lowers scalar/string conversions and `number_format` argument handling.
//!
//! Called from:
//! - The string builtin lowering facade and first-character case helpers.
//!
//! Key details:
//! - Numeric coercions and separator defaults remain PHP-compatible across targets.

use super::*;

/// Lowers `ord()` by returning the first byte of a string or zero for empty input.
pub(crate) fn lower_ord(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    load_single_string_arg(ctx, inst, "ord")?;
    let empty_label = ctx.next_label("ord_empty");
    let done_label = ctx.next_label("ord_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x2, {}", empty_label));       // return zero when ord() receives an empty string
            ctx.emitter.instruction("ldrb w0, [x1]");                           // load the first byte as an unsigned integer
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the empty-string fallback after loading the first byte
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdx, rdx");                           // return zero when ord() receives an empty string
            ctx.emitter.instruction(&format!("jz {}", empty_label));            // branch to the empty-string fallback when the length is zero
            ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");               // load the first byte as an unsigned integer
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the empty-string fallback after loading the first byte
        }
    }
    ctx.emitter.label(&empty_label);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Lowers `chr()` by converting an integer code point into a one-byte string.
pub(crate) fn lower_chr(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() != 1 {
        return Err(CodegenIrError::invalid_module(format!(
            "chr expected 1 arg, got {}",
            inst.operands.len()
        )));
    }
    let value = expect_operand(inst, 0)?;
    load_as_int(ctx, value, "chr")?;
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rdi, rax");                                // pass the character code to the x86_64 runtime helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_chr");
    store_if_result(ctx, inst)
}

/// Lowers `number_format()` by arranging its runtime helper arguments.
pub(crate) fn lower_number_format(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.is_empty() || inst.operands.len() > 4 {
        return Err(CodegenIrError::invalid_module(format!(
            "number_format expected 1 to 4 args, got {}",
            inst.operands.len()
        )));
    }

    let number = expect_operand(inst, 0)?;
    load_as_float(ctx, number, "number_format")?;
    abi::emit_push_float_reg(ctx.emitter, abi::float_result_reg(ctx.emitter));

    push_decimal_count(ctx, inst)?;
    push_separator_byte(ctx, inst, 2, 46, false, "decimal separator")?;
    push_separator_byte(ctx, inst, 3, 44, true, "thousands separator")?;
    pop_number_format_args(ctx);
    abi::emit_call_label(ctx.emitter, "__rt_number_format");
    store_if_result(ctx, inst)
}
/// Pushes the explicit or default decimal-count argument.
pub(super) fn push_decimal_count(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() >= 2 {
        let decimals = expect_operand(inst, 1)?;
        load_as_int(ctx, decimals, "number_format decimals")?;
    } else {
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    }
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    Ok(())
}

/// Pushes a one-byte separator argument, using `default_byte` when it is omitted.
pub(super) fn push_separator_byte(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operand_index: usize,
    default_byte: i64,
    empty_string_means_zero: bool,
    name: &str,
) -> Result<()> {
    if inst.operands.len() > operand_index {
        let value = expect_operand(inst, operand_index)?;
        load_separator_byte(ctx, value, empty_string_means_zero, name)?;
    } else {
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), default_byte);
    }
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    Ok(())
}

/// Loads the first byte of a separator string into the integer result register.
pub(super) fn load_separator_byte(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    empty_string_means_zero: bool,
    name: &str,
) -> Result<()> {
    if ctx.value_php_type(value)? != PhpType::Str {
        return Err(CodegenIrError::unsupported(format!(
            "number_format {} for non-string operand",
            name
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_string_value_to_regs(value, "x1", "x2")?;
            if empty_string_means_zero {
                emit_aarch64_empty_separator_guard(ctx);
            } else {
                ctx.emitter.instruction("ldrb w0, [x1]");                       // load the first byte of the separator string
            }
        }
        Arch::X86_64 => {
            ctx.load_string_value_to_regs(value, "rax", "rdx")?;
            if empty_string_means_zero {
                emit_x86_64_empty_separator_guard(ctx);
            } else {
                ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");           // load the first byte of the separator string
            }
        }
    }
    Ok(())
}

/// Emits the AArch64 empty-string fallback for the optional thousands separator.
pub(super) fn emit_aarch64_empty_separator_guard(ctx: &mut FunctionContext<'_>) {
    let use_zero = ctx.next_label("nf_sep_zero");
    let done = ctx.next_label("nf_sep_done");
    ctx.emitter.instruction(&format!("cbz x2, {}", use_zero));                  // use the no-separator sentinel when the separator string is empty
    ctx.emitter.instruction("ldrb w0, [x1]");                                   // load the first byte of the non-empty separator string
    ctx.emitter.instruction(&format!("b {}", done));                            // skip the empty-string separator fallback
    ctx.emitter.label(&use_zero);
    abi::emit_load_int_immediate(ctx.emitter, "x0", 0);
    ctx.emitter.label(&done);
}

/// Emits the x86_64 empty-string fallback for the optional thousands separator.
pub(super) fn emit_x86_64_empty_separator_guard(ctx: &mut FunctionContext<'_>) {
    let use_zero = ctx.next_label("nf_sep_zero");
    let done = ctx.next_label("nf_sep_done");
    ctx.emitter.instruction("test rdx, rdx");                                   // check whether the separator string is empty
    ctx.emitter.instruction(&format!("jz {}", use_zero));                       // use the no-separator sentinel for an empty separator
    ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");                       // load the first byte of the non-empty separator string
    ctx.emitter.instruction(&format!("jmp {}", done));                          // skip the empty-string separator fallback
    ctx.emitter.label(&use_zero);
    abi::emit_load_int_immediate(ctx.emitter, "rax", 0);
    ctx.emitter.label(&done);
}

/// Pops the staged arguments into the runtime helper's target ABI registers.
pub(super) fn pop_number_format_args(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x3");
            abi::emit_pop_reg(ctx.emitter, "x2");
            abi::emit_pop_reg(ctx.emitter, "x1");
            abi::emit_pop_float_reg(ctx.emitter, "d0");
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rdx");
            abi::emit_pop_reg(ctx.emitter, "rsi");
            abi::emit_pop_reg(ctx.emitter, "rdi");
            abi::emit_pop_float_reg(ctx.emitter, "xmm0");
        }
    }
}

/// Loads a concrete scalar value as a floating-point runtime argument.
pub(super) fn load_as_float(ctx: &mut FunctionContext<'_>, value: ValueId, name: &str) -> Result<()> {
    match ctx.load_value_to_result(value)?.codegen_repr() {
        PhpType::Float => Ok(()),
        PhpType::Int | PhpType::Bool => {
            abi::emit_int_result_to_float_result(ctx.emitter);
            Ok(())
        }
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
            abi::emit_int_result_to_float_result(ctx.emitter);
            Ok(())
        }
        PhpType::Str => {
            abi::emit_call_label(ctx.emitter, "__rt_str_to_number");
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "{} for PHP type {:?}",
            name, other
        ))),
    }
}

/// Loads a concrete scalar value as an integer runtime argument.
pub(super) fn load_as_int(ctx: &mut FunctionContext<'_>, value: ValueId, name: &str) -> Result<()> {
    match ctx.load_value_to_result(value)?.codegen_repr() {
        PhpType::Int | PhpType::Bool => Ok(()),
        PhpType::Void | PhpType::Never => {
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
            Ok(())
        }
        PhpType::Float => {
            abi::emit_float_result_to_int_result(ctx.emitter);
            Ok(())
        }
        PhpType::TaggedScalar => {
            crate::codegen::sentinels::emit_tagged_scalar_to_int_null_as_zero(ctx.emitter);
            Ok(())
        }
        PhpType::Str => {
            abi::emit_call_label(ctx.emitter, "__rt_str_to_int");
            Ok(())
        }
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, value)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int");
            Ok(())
        }
        other => Err(CodegenIrError::unsupported(format!(
            "{} for PHP type {:?}",
            name, other
        ))),
    }
}
