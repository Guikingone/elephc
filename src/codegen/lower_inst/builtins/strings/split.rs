//! Purpose:
//! Lowers explode, sscanf, str_split, and implode with temporary string cleanup.
//!
//! Called from:
//! - The string builtin lowering facade.
//!
//! Key details:
//! - Coercion temporaries are saved and released without disturbing array-result ownership.

use super::*;

/// Stack cleanup slots for split builtin string coercions that allocate owned temporaries.
pub(super) struct SplitStringTempCleanups {
    delimiter_offset: Option<usize>,
    subject_offset: Option<usize>,
    bytes: usize,
}

impl SplitStringTempCleanups {
    /// Builds a cleanup layout with one 16-byte stack slot for each owned string temporary.
    fn new(delimiter_needs_cleanup: bool, subject_needs_cleanup: bool) -> Self {
        let mut bytes = 0usize;
        let delimiter_offset = delimiter_needs_cleanup.then(|| {
            let offset = bytes;
            bytes += 16;
            offset
        });
        let subject_offset = subject_needs_cleanup.then(|| {
            let offset = bytes;
            bytes += 16;
            offset
        });
        Self {
            delimiter_offset,
            subject_offset,
            bytes,
        }
    }

    /// Returns true when no split string coercion produced an owned temporary.
    fn is_empty(&self) -> bool {
        self.bytes == 0
    }

    /// Returns the stack offsets for all saved owned string temporaries.
    fn offsets(&self) -> impl Iterator<Item = usize> + '_ {
        [self.delimiter_offset, self.subject_offset]
            .into_iter()
            .flatten()
    }
}
/// Lowers `explode(delimiter, string)` into the shared string-array splitter helper.
pub(crate) fn lower_explode(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let cleanups = plan_split_string_temp_cleanups(ctx, inst)?;
    if !cleanups.is_empty() {
        abi::emit_reserve_temporary_stack(ctx.emitter, cleanups.bytes);
    }
    load_split_pair_args(ctx, inst, "explode", &cleanups)?;
    abi::emit_call_label(ctx.emitter, "__rt_explode");
    emit_split_string_temp_cleanups(ctx, &cleanups);
    store_if_result(ctx, inst)
}

/// Lowers `sscanf(string, format)` into the shared scanner helper.
pub(crate) fn lower_sscanf(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() < 2 {
        return Err(CodegenIrError::invalid_module(format!(
            "sscanf expected at least 2 args, got {}",
            inst.operands.len()
        )));
    }
    load_input_and_pattern_args(ctx, inst, "sscanf")?;
    abi::emit_call_label(ctx.emitter, "__rt_sscanf");
    store_if_result(ctx, inst)
}

/// Lowers `str_split(string, length?)` into the fixed-width string-array splitter.
pub(crate) fn lower_str_split(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.is_empty() || inst.operands.len() > 2 {
        return Err(CodegenIrError::invalid_module(format!(
            "str_split expected 1 or 2 args, got {}",
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_str_split_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_str_split_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_split");
    store_if_result(ctx, inst)
}

/// Lowers `implode(glue, array)` by selecting the string or integer array helper.
pub(crate) fn lower_implode(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() != 2 {
        return Err(CodegenIrError::invalid_module(format!(
            "implode expected 2 args, got {}",
            inst.operands.len()
        )));
    }
    let runtime_label = implode_runtime_label(ctx, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_implode_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_implode_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, runtime_label);
    store_if_result(ctx, inst)
}
/// Materializes delimiter/payload string pairs for split-style array helpers.
pub(super) fn load_split_pair_args(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    cleanups: &SplitStringTempCleanups,
) -> Result<()> {
    if inst.operands.len() != 2 {
        return Err(CodegenIrError::invalid_module(format!(
            "{} expected 2 args, got {}",
            name,
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => load_split_pair_args_aarch64(ctx, inst, name, cleanups),
        Arch::X86_64 => load_split_pair_args_x86_64(ctx, inst, name, cleanups),
    }
}

/// Materializes AArch64 delimiter and subject strings for `explode()`.
pub(super) fn load_split_pair_args_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    cleanups: &SplitStringTempCleanups,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, name, "x1", "x2")?;
    if let Some(offset) = cleanups.delimiter_offset {
        save_split_string_temp(ctx, offset, "x1", "x2");
    }
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the delimiter string while materializing the subject string
    load_string_arg_to_regs(ctx, inst, 1, name, "x1", "x2")?;
    ctx.emitter.instruction("mov x3, x1");                                      // pass the subject string pointer as the secondary split argument
    ctx.emitter.instruction("mov x4, x2");                                      // pass the subject string length as the secondary split argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the delimiter string into primary split argument registers
    if let Some(offset) = cleanups.subject_offset {
        save_split_string_temp(ctx, offset, "x3", "x4");
    }
    Ok(())
}

/// Materializes x86_64 delimiter and subject strings for `explode()`.
pub(super) fn load_split_pair_args_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    cleanups: &SplitStringTempCleanups,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, name, "rax", "rdx")?;
    if let Some(offset) = cleanups.delimiter_offset {
        save_split_string_temp(ctx, offset, "rax", "rdx");
    }
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_string_arg_to_regs(ctx, inst, 1, name, "rax", "rdx")?;
    ctx.emitter.instruction("mov rdi, rax");                                    // pass the subject string pointer as the secondary split argument
    ctx.emitter.instruction("mov rsi, rdx");                                    // pass the subject string length as the secondary split argument
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    if let Some(offset) = cleanups.subject_offset {
        save_split_string_temp(ctx, offset, "rdi", "rsi");
    }
    Ok(())
}

/// Plans which split builtin operands produce owned temporary strings during coercion.
pub(super) fn plan_split_string_temp_cleanups(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<SplitStringTempCleanups> {
    let delimiter = expect_operand(inst, 0)?;
    let subject = expect_operand(inst, 1)?;
    Ok(SplitStringTempCleanups::new(
        value_string_coercion_needs_temp_cleanup(ctx, delimiter)?,
        value_string_coercion_needs_temp_cleanup(ctx, subject)?,
    ))
}

/// Returns true when string coercion for `value` returns a caller-owned heap string.
pub(super) fn value_string_coercion_needs_temp_cleanup(
    ctx: &FunctionContext<'_>,
    value: ValueId,
) -> Result<bool> {
    Ok(matches!(
        ctx.value_php_type(value)?.codegen_repr(),
        PhpType::Int
            | PhpType::Float
            | PhpType::Bool
            | PhpType::TaggedScalar
            | PhpType::Resource(_)
    ))
}

/// Saves a string pointer/length pair into the split builtin cleanup area.
pub(super) fn save_split_string_temp(
    ctx: &mut FunctionContext<'_>,
    offset: usize,
    ptr_reg: &str,
    len_reg: &str,
) {
    let scratch = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_temporary_stack_address(ctx.emitter, scratch, offset);
    abi::emit_store_to_address(ctx.emitter, ptr_reg, scratch, 0);
    abi::emit_store_to_address(ctx.emitter, len_reg, scratch, 8);
}

/// Releases owned split string temporaries while preserving the runtime result.
pub(super) fn emit_split_string_temp_cleanups(
    ctx: &mut FunctionContext<'_>,
    cleanups: &SplitStringTempCleanups,
) {
    if cleanups.is_empty() {
        return;
    }
    for offset in cleanups.offsets() {
        let shifted_offset = offset + 16;
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        abi::emit_load_temporary_stack_slot(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            shifted_offset,
        );
        abi::emit_call_label(ctx.emitter, "__rt_heap_free_safe");
        abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    }
    abi::emit_release_temporary_stack(ctx.emitter, cleanups.bytes);
}
/// Materializes primary input and pattern strings for scanner-style helpers.
pub(super) fn load_input_and_pattern_args(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => load_input_and_pattern_args_aarch64(ctx, inst, name),
        Arch::X86_64 => load_input_and_pattern_args_x86_64(ctx, inst, name),
    }
}

/// Materializes AArch64 input and pattern strings for `sscanf()`.
pub(super) fn load_input_and_pattern_args_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    let input = expect_string_operand(ctx, inst, 0, name)?;
    let pattern = expect_string_operand(ctx, inst, 1, name)?;
    ctx.load_string_value_to_regs(input, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the scanner input while materializing the pattern string
    ctx.load_string_value_to_regs(pattern, "x1", "x2")?;
    ctx.emitter.instruction("mov x3, x1");                                      // pass the pattern pointer as the secondary scanner argument
    ctx.emitter.instruction("mov x4, x2");                                      // pass the pattern length as the secondary scanner argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the scanner input into primary argument registers
    Ok(())
}

/// Materializes x86_64 input and pattern strings for `sscanf()`.
pub(super) fn load_input_and_pattern_args_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
) -> Result<()> {
    let input = expect_string_operand(ctx, inst, 0, name)?;
    let pattern = expect_string_operand(ctx, inst, 1, name)?;
    ctx.load_string_value_to_regs(input, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    ctx.load_string_value_to_regs(pattern, "rax", "rdx")?;
    ctx.emitter.instruction("mov rdi, rax");                                    // pass the pattern pointer as the secondary scanner argument
    ctx.emitter.instruction("mov rsi, rdx");                                    // pass the pattern length as the secondary scanner argument
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes AArch64 source string and optional chunk length for `str_split()`.
pub(super) fn lower_str_split_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let source = expect_string_operand(ctx, inst, 0, "str_split")?;
    ctx.load_string_value_to_regs(source, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the source string while materializing the chunk length
    materialize_str_split_length_aarch64(ctx, inst)?;
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the source string before calling the splitter helper
    Ok(())
}

/// Materializes x86_64 source string and optional chunk length for `str_split()`.
pub(super) fn lower_str_split_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let source = expect_string_operand(ctx, inst, 0, "str_split")?;
    ctx.load_string_value_to_regs(source, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    materialize_str_split_length_x86_64(ctx, inst)?;
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes the AArch64 optional `str_split()` chunk length.
pub(super) fn materialize_str_split_length_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 2 {
        let length = expect_operand(inst, 1)?;
        load_as_int(ctx, length, "str_split length")?;
        ctx.emitter.instruction("mov x3, x0");                                  // pass the requested chunk length to the splitter helper
    } else {
        ctx.emitter.instruction("mov x3, #1");                                  // default to one-byte chunks when length is omitted
    }
    Ok(())
}

/// Materializes the x86_64 optional `str_split()` chunk length.
pub(super) fn materialize_str_split_length_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 2 {
        let length = expect_operand(inst, 1)?;
        load_as_int(ctx, length, "str_split length")?;
        ctx.emitter.instruction("mov rdi, rax");                                // pass the requested chunk length to the splitter helper
    } else {
        ctx.emitter.instruction("mov rdi, 1");                                  // default to one-byte chunks when length is omitted
    }
    Ok(())
}

/// Returns the runtime helper label required for an `implode()` array operand.
pub(super) fn implode_runtime_label(ctx: &FunctionContext<'_>, inst: &Instruction) -> Result<&'static str> {
    let array = expect_operand(inst, 1)?;
    match ctx.value_php_type(array)? {
        PhpType::Array(elem_ty) => match elem_ty.codegen_repr() {
            // PHP stringifies bool elements as "1"/"" — NOT as the "1"/"0" that
            // `__rt_implode_int`'s `__rt_itoa` pass would produce — so bool arrays get their
            // own renderer. `PhpType::False` reaches this arm as `Bool` through `codegen_repr`.
            PhpType::Bool => Ok("__rt_implode_bool"),
            PhpType::Int => Ok("__rt_implode_int"),
            PhpType::Str | PhpType::Mixed | PhpType::Never => Ok("__rt_implode"),
            other => Err(CodegenIrError::unsupported(format!(
                "implode array element PHP type {:?}",
                other
            ))),
        },
        PhpType::Mixed | PhpType::Union(_) => Ok("__rt_implode"),
        other => Err(CodegenIrError::unsupported(format!(
            "implode array PHP type {:?}",
            other
        ))),
    }
}

/// Materializes AArch64 glue and array arguments for `implode()`.
pub(super) fn lower_implode_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let glue = expect_string_operand(ctx, inst, 0, "implode")?;
    let array = expect_operand(inst, 1)?;
    ctx.load_string_value_to_regs(glue, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the glue string while materializing the array argument
    load_implode_array_aarch64(ctx, array)?;
    ctx.emitter.instruction("mov x3, x0");                                      // pass the indexed array pointer as the third implode argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the glue string into primary implode argument registers
    Ok(())
}

/// Materializes x86_64 glue and array arguments for `implode()`.
pub(super) fn lower_implode_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let glue = expect_string_operand(ctx, inst, 0, "implode")?;
    let array = expect_operand(inst, 1)?;
    ctx.load_string_value_to_regs(glue, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_implode_array_x86_64(ctx, array)?;
    ctx.emitter.instruction("mov rdx, rax");                                    // pass the indexed array pointer as the third implode argument
    abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
    Ok(())
}

/// Loads the raw indexed-array payload consumed by `implode()` on AArch64.
pub(super) fn load_implode_array_aarch64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
) -> Result<()> {
    match ctx.value_php_type(array)?.codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_reg(array, "x0")?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("mov x0, x1");                              // pass the unboxed array payload to implode()
            Ok(())
        }
        _ => {
            ctx.load_value_to_reg(array, "x0")?;
            Ok(())
        }
    }
}

/// Loads the raw indexed-array payload consumed by `implode()` on x86_64.
pub(super) fn load_implode_array_x86_64(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
) -> Result<()> {
    match ctx.value_php_type(array)?.codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_reg(array, "rax")?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("mov rax, rdi");                            // pass the unboxed array payload to implode()
            Ok(())
        }
        _ => {
            ctx.load_value_to_reg(array, "rax")?;
            Ok(())
        }
    }
}
