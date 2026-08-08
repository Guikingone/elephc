//! Purpose:
//! Lowers string search, substring, repetition, replacement-slice, and `strstr` builtins.
//!
//! Called from:
//! - The string builtin lowering facade.
//!
//! Key details:
//! - Search sentinels and optional bounds are materialized consistently for both targets.

use super::*;

/// Lowers `str_contains()` through `strpos()` and converts found positions to bool.
pub(crate) fn lower_str_contains(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    load_binary_string_args(ctx, inst, "str_contains")?;
    abi::emit_call_label(ctx.emitter, "__rt_strpos");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // check whether strpos() found the needle at any non-negative position
            ctx.emitter.instruction("cset x0, ge");                             // normalize the signed strpos() result into a PHP boolean
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 0");                              // check whether strpos() found the needle at any non-negative position
            ctx.emitter.instruction("setge al");                                // normalize the signed strpos() result into the low boolean byte
            ctx.emitter.instruction("movzx eax, al");                           // widen the normalized boolean byte into the integer result register
        }
    }
    store_if_result(ctx, inst)
}

/// Lowers `strpos()`/`strrpos()` and boxes position-or-false results as Mixed.
pub(crate) fn lower_string_position(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    name: &str,
    runtime_label: &str,
) -> Result<()> {
    load_binary_string_args(ctx, inst, name)?;
    abi::emit_call_label(ctx.emitter, runtime_label);
    box_search_result(ctx, name);
    store_if_result(ctx, inst)
}

/// Lowers `substr(string, offset, length?)` with target-local pointer arithmetic.
pub(crate) fn lower_substr(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() < 2 || inst.operands.len() > 3 {
        return Err(CodegenIrError::invalid_module(format!(
            "substr expected 2 or 3 args, got {}",
            inst.operands.len()
        )));
    }
    let neg_done = ctx.next_label("substr_neg_done");
    let len_done = ctx.next_label("substr_len_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_substr_aarch64(ctx, inst, &neg_done, &len_done)?,
        Arch::X86_64 => lower_substr_x86_64(ctx, inst, &neg_done, &len_done)?,
    }
    store_if_result(ctx, inst)
}

/// Lowers `substr_replace(string, replacement, start, length?)`.
pub(crate) fn lower_substr_replace(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() < 3 || inst.operands.len() > 4 {
        return Err(CodegenIrError::invalid_module(format!(
            "substr_replace expected 3 or 4 args, got {}",
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_substr_replace_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_substr_replace_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_substr_replace");
    store_if_result(ctx, inst)
}

/// Lowers `str_repeat(string, times)` through the shared runtime helper.
pub(crate) fn lower_str_repeat(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() != 2 {
        return Err(CodegenIrError::invalid_module(format!(
            "str_repeat expected 2 args, got {}",
            inst.operands.len()
        )));
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_str_repeat_aarch64(ctx, inst)?,
        Arch::X86_64 => lower_str_repeat_x86_64(ctx, inst)?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_repeat");
    store_if_result(ctx, inst)
}

/// Lowers `strstr(haystack, needle)` by searching and returning the matching suffix.
pub(crate) fn lower_strstr(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.len() < 2 || inst.operands.len() > 3 {
        return Err(CodegenIrError::invalid_module(format!(
            "strstr expected 2 or 3 args, got {}",
            inst.operands.len()
        )));
    }
    if inst.result.is_some() && inst.result_php_type.codegen_repr() != PhpType::Mixed {
        // `crate::builtins::string::strstr::check` types EVERY call `string|false`, whose
        // representation is `Mixed`, and the arms below leave a BOXED cell in the integer
        // result register. A `Str` result type here would make `store_if_result` copy the
        // string-pair registers instead, which no longer hold the answer — fail loudly rather
        // than emit that silently wrong store.
        return Err(CodegenIrError::invalid_module(format!(
            "strstr result must be Mixed (string|false), got {:?}",
            inst.result_php_type
        )));
    }
    let labels = StrstrLabels {
        prefix: ctx.next_label("strstr_prefix"),
        miss: ctx.next_label("strstr_miss"),
        box_match: ctx.next_label("strstr_box_match"),
        end: ctx.next_label("strstr_end"),
    };
    // The `$before_needle` flag is materialized FIRST and parked on the temporary stack: every
    // register that could hold it (including the caller-saved scratch the truthiness helpers
    // use) is clobbered by the haystack/needle materialization and the `__rt_strpos` call.
    materialize_truthy_flag(ctx, inst, 2, "strstr")?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_strstr_aarch64(ctx, inst, &labels)?,
        Arch::X86_64 => lower_strstr_x86_64(ctx, inst, &labels)?,
    }
    ctx.emitter.label(&labels.end);
    store_if_result(ctx, inst)
}

/// The four branch targets `lower_strstr` threads through its per-architecture emitters.
///
/// `prefix` selects the `$before_needle` substring, `miss` boxes PHP's `false`, `box_match` is
/// where both hit arms converge to box the selected substring as a string, and `end` is the
/// common continuation where the boxed `Mixed` cell is stored.
pub(super) struct StrstrLabels {
    prefix: String,
    miss: String,
    box_match: String,
    end: String,
}

/// Emits the AArch64 inline substring pointer/length calculation.
pub(super) fn lower_substr_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    neg_done: &str,
    len_done: &str,
) -> Result<()> {
    load_substr_string_and_offset_aarch64(ctx, inst)?;
    if inst.operands.len() >= 3 {
        let length = expect_operand(inst, 2)?;
        load_as_int(ctx, length, "substr length")?;
        ctx.emitter.instruction("mov x3, x0");                                  // move the explicit substring length into the clamp register
    } else {
        ctx.emitter.instruction("mov x3, #-1");                                 // use -1 as the sentinel for an omitted substring length
    }
    ctx.emitter.instruction("ldr x0, [sp], #16");                               // restore the substring offset after optional length materialization
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the source string pointer and length
    ctx.emitter.instruction("cmp x0, #0");                                      // check whether the requested offset is negative
    ctx.emitter.instruction(&format!("b.ge {}", neg_done));                     // skip tail-relative offset adjustment for non-negative offsets
    ctx.emitter.instruction("add x0, x2, x0");                                  // convert the negative offset into a tail-relative byte index
    ctx.emitter.instruction("cmp x0, #0");                                      // check whether the tail-relative offset still points before the string
    ctx.emitter.instruction("csel x0, xzr, x0, lt");                            // clamp underflowing offsets back to the start of the string
    ctx.emitter.label(neg_done);
    ctx.emitter.instruction("cmp x0, x2");                                      // compare the final offset against the full source-string length
    ctx.emitter.instruction("csel x0, x2, x0, gt");                             // clamp offsets past the end to the source-string length
    ctx.emitter.instruction("add x1, x1, x0");                                  // advance the result pointer to the selected substring start
    ctx.emitter.instruction("sub x2, x2, x0");                                  // compute the remaining byte length after the selected offset
    ctx.emitter.instruction("cmn x3, #1");                                      // check whether the optional length argument was omitted
    ctx.emitter.instruction(&format!("b.eq {}", len_done));                     // keep the full remaining tail when no explicit length was provided
    ctx.emitter.instruction("cmp x3, #0");                                      // check whether the requested substring length is negative
    ctx.emitter.instruction("csel x3, xzr, x3, lt");                            // clamp negative requested lengths to zero
    ctx.emitter.instruction("cmp x3, x2");                                      // compare requested length against the remaining tail length
    ctx.emitter.instruction("csel x2, x3, x2, lt");                             // shrink the result length when the requested length is shorter
    ctx.emitter.label(len_done);
    Ok(())
}

/// Loads the source string and offset for AArch64 `substr()` lowering.
pub(super) fn load_substr_string_and_offset_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let offset = expect_operand(inst, 1)?;
    load_string_arg_to_regs(ctx, inst, 0, "substr", "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the source string while materializing numeric arguments
    load_as_int(ctx, offset, "substr offset")?;
    ctx.emitter.instruction("str x0, [sp, #-16]!");                             // preserve the substring offset while materializing the optional length
    Ok(())
}

/// Emits the x86_64 inline substring pointer/length calculation.
pub(super) fn lower_substr_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    neg_done: &str,
    len_done: &str,
) -> Result<()> {
    load_substr_string_and_offset_x86_64(ctx, inst)?;
    if inst.operands.len() >= 3 {
        let length = expect_operand(inst, 2)?;
        load_as_int(ctx, length, "substr length")?;
        ctx.emitter.instruction("mov rcx, rax");                                // move the explicit substring length into the clamp register
    } else {
        abi::emit_load_int_immediate(ctx.emitter, "rcx", -1);
    }
    abi::emit_pop_reg(ctx.emitter, "rax");
    abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
    ctx.emitter.instruction("cmp rax, 0");                                      // check whether the requested offset is negative
    ctx.emitter.instruction(&format!("jge {}", neg_done));                      // skip tail-relative offset adjustment for non-negative offsets
    ctx.emitter.instruction("add rax, rsi");                                    // convert the negative offset into a tail-relative byte index
    ctx.emitter.instruction("cmp rax, 0");                                      // check whether the tail-relative offset still points before the string
    ctx.emitter.instruction("mov r8, 0");                                       // materialize zero for offset and length clamping
    ctx.emitter.instruction("cmovl rax, r8");                                   // clamp underflowing offsets back to the start of the string
    ctx.emitter.label(neg_done);
    ctx.emitter.instruction("cmp rax, rsi");                                    // compare the final offset against the full source-string length
    ctx.emitter.instruction("cmovg rax, rsi");                                  // clamp offsets past the end to the source-string length
    ctx.emitter.instruction("add rdi, rax");                                    // advance the result pointer to the selected substring start
    ctx.emitter.instruction("sub rsi, rax");                                    // compute the remaining byte length after the selected offset
    ctx.emitter.instruction("cmp rcx, -1");                                     // check whether the optional length argument was omitted
    ctx.emitter.instruction(&format!("je {}", len_done));                       // keep the full remaining tail when no explicit length was provided
    ctx.emitter.instruction("cmp rcx, 0");                                      // check whether the requested substring length is negative
    ctx.emitter.instruction("mov r8, 0");                                       // materialize zero for negative length clamping
    ctx.emitter.instruction("cmovl rcx, r8");                                   // clamp negative requested lengths to zero
    ctx.emitter.instruction("cmp rcx, rsi");                                    // compare requested length against the remaining tail length
    ctx.emitter.instruction("cmovl rsi, rcx");                                  // shrink the result length when the requested length is shorter
    ctx.emitter.label(len_done);
    ctx.emitter.instruction("mov rax, rdi");                                    // return the selected substring pointer in the string result register
    ctx.emitter.instruction("mov rdx, rsi");                                    // return the selected substring length in the string result register
    Ok(())
}

/// Loads the source string and offset for x86_64 `substr()` lowering.
pub(super) fn load_substr_string_and_offset_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let offset = expect_operand(inst, 1)?;
    load_string_arg_to_regs(ctx, inst, 0, "substr", "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_as_int(ctx, offset, "substr offset")?;
    abi::emit_push_reg(ctx.emitter, "rax");
    Ok(())
}

/// Materializes AArch64 `str_repeat()` runtime arguments.
pub(super) fn lower_str_repeat_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let source = expect_string_operand(ctx, inst, 0, "str_repeat")?;
    let times = expect_operand(inst, 1)?;
    ctx.load_string_value_to_regs(source, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the source string while materializing the repeat count
    load_as_int(ctx, times, "str_repeat times")?;
    ctx.emitter.instruction("mov x3, x0");                                      // pass the repeat count as the third string-helper argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the source string into runtime argument registers
    Ok(())
}

/// Materializes x86_64 `str_repeat()` runtime arguments.
pub(super) fn lower_str_repeat_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let source = expect_string_operand(ctx, inst, 0, "str_repeat")?;
    let times = expect_operand(inst, 1)?;
    ctx.load_string_value_to_regs(source, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_as_int(ctx, times, "str_repeat times")?;
    ctx.emitter.instruction("mov rdi, rax");                                    // pass the repeat count as the extra x86_64 runtime argument
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Emits AArch64 `strstr()` search and suffix reconstruction.
pub(super) fn lower_strstr_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    labels: &StrstrLabels,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, "strstr", "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the haystack while materializing the needle string
    load_string_arg_to_regs(ctx, inst, 1, "strstr", "x1", "x2")?;
    ctx.emitter.instruction("mov x3, x1");                                      // pass the needle pointer as the secondary string argument
    ctx.emitter.instruction("mov x4, x2");                                      // pass the needle length as the secondary string argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the haystack into primary string argument registers
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the haystack while strpos() returns the match offset
    abi::emit_call_label(ctx.emitter, "__rt_strpos");
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore the haystack for substring reconstruction
    ctx.emitter.instruction("ldr x9, [sp], #16");                               // reload the parked $before_needle flag now that every call is done
    ctx.emitter.instruction("cmp x0, #0");                                      // check whether strpos() returned a valid match offset
    ctx.emitter.instruction(&format!("b.lt {}", labels.miss));                  // PHP returns false, not "", when the needle is absent
    ctx.emitter.instruction("cmp x9, #0");                                      // was $before_needle truthy?
    ctx.emitter.instruction(&format!("b.ne {}", labels.prefix));                // a truthy flag selects the part BEFORE the needle
    ctx.emitter.instruction("add x1, x1, x0");                                  // advance the haystack pointer to the matching suffix
    ctx.emitter.instruction("sub x2, x2, x0");                                  // shrink the haystack length to the matching suffix length
    ctx.emitter.instruction(&format!("b {}", labels.box_match));                // both hit arms box the selected substring identically
    ctx.emitter.label(&labels.prefix);
    ctx.emitter.instruction("mov x2, x0");                                      // keep the haystack pointer and cut the length at the match offset
    ctx.emitter.label(&labels.box_match);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
    ctx.emitter.instruction(&format!("b {}", labels.end));                      // skip the miss arm once the substring is boxed
    ctx.emitter.label(&labels.miss);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
    Ok(())
}

/// Emits x86_64 `strstr()` search and suffix reconstruction.
pub(super) fn lower_strstr_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    labels: &StrstrLabels,
) -> Result<()> {
    load_string_arg_to_regs(ctx, inst, 0, "strstr", "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_string_arg_to_regs(ctx, inst, 1, "strstr", "rax", "rdx")?;
    ctx.emitter.instruction("mov r8, rax");                                     // preserve the needle pointer while restoring the haystack
    ctx.emitter.instruction("mov r9, rdx");                                     // preserve the needle length while restoring the haystack
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    ctx.emitter.instruction("mov rdi, rax");                                    // pass the haystack pointer as the first SysV string argument
    ctx.emitter.instruction("mov rsi, rdx");                                    // pass the haystack length as the second SysV string argument
    ctx.emitter.instruction("mov rdx, r8");                                     // pass the needle pointer as the third SysV string argument
    ctx.emitter.instruction("mov rcx, r9");                                     // pass the needle length as the fourth SysV string argument
    abi::emit_call_label(ctx.emitter, "__rt_strpos");
    ctx.emitter.instruction("mov r8, rax");                                     // preserve the signed match offset while restoring the haystack
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    abi::emit_pop_reg(ctx.emitter, "r9");                                       // reload the parked $before_needle flag now that every call is done
    ctx.emitter.instruction("cmp r8, 0");                                       // check whether strpos() returned a valid match offset
    ctx.emitter.instruction(&format!("jl {}", labels.miss));                    // PHP returns false, not "", when the needle is absent
    ctx.emitter.instruction("cmp r9, 0");                                       // was $before_needle truthy?
    ctx.emitter.instruction(&format!("jne {}", labels.prefix));                 // a truthy flag selects the part BEFORE the needle
    ctx.emitter.instruction("add rax, r8");                                     // advance the haystack pointer to the matching suffix
    ctx.emitter.instruction("sub rdx, r8");                                     // shrink the haystack length to the matching suffix length
    ctx.emitter.instruction(&format!("jmp {}", labels.box_match));              // both hit arms box the selected substring identically
    ctx.emitter.label(&labels.prefix);
    ctx.emitter.instruction("mov rdx, r8");                                     // keep the haystack pointer and cut the length at the match offset
    ctx.emitter.label(&labels.box_match);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
    ctx.emitter.instruction(&format!("jmp {}", labels.end));                    // skip the miss arm once the substring is boxed
    ctx.emitter.label(&labels.miss);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);
    crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Bool);
    Ok(())
}
/// Materializes AArch64 `substr_replace()` runtime arguments.
pub(super) fn lower_substr_replace_aarch64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let subject = expect_string_operand(ctx, inst, 0, "substr_replace")?;
    let replacement = expect_string_operand(ctx, inst, 1, "substr_replace")?;
    let start = expect_operand(inst, 2)?;
    ctx.load_string_value_to_regs(subject, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the subject string while materializing replacement and slice bounds
    ctx.load_string_value_to_regs(replacement, "x1", "x2")?;
    ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                         // preserve the replacement string while materializing slice bounds
    load_as_int(ctx, start, "substr_replace start")?;
    abi::emit_push_reg(ctx.emitter, "x0");
    materialize_substr_replace_length_aarch64(ctx, inst)?;
    abi::emit_pop_reg(ctx.emitter, "x0");
    ctx.emitter.instruction("ldp x3, x4, [sp], #16");                           // restore replacement into the secondary runtime string argument
    ctx.emitter.instruction("ldp x1, x2, [sp], #16");                           // restore subject into the primary runtime string argument
    Ok(())
}

/// Materializes x86_64 `substr_replace()` runtime arguments.
pub(super) fn lower_substr_replace_x86_64(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let subject = expect_string_operand(ctx, inst, 0, "substr_replace")?;
    let replacement = expect_string_operand(ctx, inst, 1, "substr_replace")?;
    let start = expect_operand(inst, 2)?;
    ctx.load_string_value_to_regs(subject, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    ctx.load_string_value_to_regs(replacement, "rax", "rdx")?;
    abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");
    load_as_int(ctx, start, "substr_replace start")?;
    abi::emit_push_reg(ctx.emitter, "rax");
    materialize_substr_replace_length_x86_64(ctx, inst)?;
    abi::emit_pop_reg(ctx.emitter, "rcx");
    abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");
    abi::emit_pop_reg_pair(ctx.emitter, "rax", "rdx");
    Ok(())
}

/// Materializes the AArch64 optional `substr_replace()` length argument.
pub(super) fn materialize_substr_replace_length_aarch64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 4 {
        let length = expect_operand(inst, 3)?;
        load_as_int(ctx, length, "substr_replace length")?;
        ctx.emitter.instruction("mov x7, x0");                                  // pass the explicit replacement length to the runtime helper
    } else {
        ctx.emitter.instruction("mov x7, #-1");                                 // use -1 sentinel so replacement runs through the subject end
    }
    Ok(())
}

/// Materializes the x86_64 optional `substr_replace()` length argument.
pub(super) fn materialize_substr_replace_length_x86_64(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() >= 4 {
        let length = expect_operand(inst, 3)?;
        load_as_int(ctx, length, "substr_replace length")?;
        ctx.emitter.instruction("mov r8, rax");                                 // pass the explicit replacement length to the runtime helper
    } else {
        abi::emit_load_int_immediate(ctx.emitter, "r8", -1);
    }
    Ok(())
}
/// Boxes a raw string-search position result into the Mixed pointer representation.
pub(super) fn box_search_result(ctx: &mut FunctionContext<'_>, label_prefix: &str) {
    let found_label = ctx.next_label(&format!("{}_found", label_prefix));
    let end_label = ctx.next_label(&format!("{}_done", label_prefix));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // distinguish a valid non-negative match offset from the not-found sentinel
            ctx.emitter.instruction(&format!("b.ge {}", found_label));          // box a found offset as an integer result
            ctx.emitter.instruction("mov x1, #0");                              // use zero as the false payload for the mixed bool box
            ctx.emitter.instruction("mov x2, #0");                              // clear the unused high payload word for bool mixed boxes
            ctx.emitter.instruction("mov x0, #3");                              // select runtime tag 3 for a boolean false mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("b {}", end_label));               // skip integer boxing after producing the false result
            ctx.emitter.label(&found_label);
            ctx.emitter.instruction("mov x1, x0");                              // move the found offset into the mixed helper payload register
            ctx.emitter.instruction("mov x2, #0");                              // clear the unused high payload word for integer mixed boxes
            ctx.emitter.instruction("mov x0, #0");                              // select runtime tag 0 for an integer mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.label(&end_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 0");                              // distinguish a valid non-negative match offset from the not-found sentinel
            ctx.emitter.instruction(&format!("jge {}", found_label));           // box a found offset as an integer result
            ctx.emitter.instruction("xor edi, edi");                            // use zero as the false payload for the mixed bool box
            ctx.emitter.instruction("xor esi, esi");                            // clear the unused high payload word for bool mixed boxes
            ctx.emitter.instruction("mov eax, 3");                              // select runtime tag 3 for a boolean false mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("jmp {}", end_label));             // skip integer boxing after producing the false result
            ctx.emitter.label(&found_label);
            ctx.emitter.instruction("mov rdi, rax");                            // move the found offset into the mixed helper payload register
            ctx.emitter.instruction("xor esi, esi");                            // clear the unused high payload word for integer mixed boxes
            ctx.emitter.instruction("xor eax, eax");                            // select runtime tag 0 for an integer mixed value
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.label(&end_label);
        }
    }
}
