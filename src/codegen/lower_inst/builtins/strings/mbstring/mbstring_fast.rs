//! Purpose:
//! Lowers the hot explicit-UTF-8 mbstring cases to native ASCII helpers that fall back to
//! the shared Unicode bridge for any non-ASCII byte.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::strings::lower_mbstring()`.
//!
//! Key details:
//! - Case conversion reuses the bytewise case helpers only after proving every subject byte
//!   is ASCII; invalid or non-ASCII UTF-8 therefore keeps PHP's Unicode behavior.
//! - `mb_check_encoding()` over UTF-8 is answered directly from the shared validator.
//! - Every fallback stores its result through the same `store_if_result()` path, so the fast
//!   and bridge branches converge on one SSA destination.

use super::*;
use elephc_builtin_contract::RuntimeBuiltinId;

/// Attempts one ASCII-only fast path before the shared mbstring coordinator runs.
pub(super) fn try_lower(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    match operation {
        RuntimeBuiltinId::MbStrtoupper => {
            try_lower_case(ctx, inst, operation, "mb_strtoupper", true)
        }
        RuntimeBuiltinId::MbStrtolower => {
            try_lower_case(ctx, inst, operation, "mb_strtolower", false)
        }
        RuntimeBuiltinId::MbConvertCase => try_lower_convert_case(ctx, inst, operation),
        RuntimeBuiltinId::MbCheckEncoding => try_lower_check_encoding(ctx, inst, operation),
        RuntimeBuiltinId::MbUcfirst => {
            try_lower_ascii_string(ctx, inst, operation, "mb_ucfirst", "__rt_mb_ucfirst_utf8")
        }
        RuntimeBuiltinId::MbLcfirst => {
            try_lower_ascii_string(ctx, inst, operation, "mb_lcfirst", "__rt_mb_lcfirst_utf8")
        }
        RuntimeBuiltinId::MbStrwidth => try_lower_strwidth(ctx, inst, operation),
        RuntimeBuiltinId::MbTrim => {
            try_lower_trim(ctx, inst, operation, "mb_trim", "__rt_trim_mask")
        }
        RuntimeBuiltinId::MbLtrim => {
            try_lower_trim(ctx, inst, operation, "mb_ltrim", "__rt_ltrim_mask")
        }
        RuntimeBuiltinId::MbRtrim => {
            try_lower_trim(ctx, inst, operation, "mb_rtrim", "__rt_rtrim_mask")
        }
        RuntimeBuiltinId::MbSubstrCount => try_lower_substr_count(ctx, inst, operation),
        RuntimeBuiltinId::MbSubstr => {
            try_lower_substr_like(ctx, inst, operation, "mb_substr")
        }
        RuntimeBuiltinId::MbStrcut => {
            try_lower_substr_like(ctx, inst, operation, "mb_strcut")
        }
        RuntimeBuiltinId::MbStrPad => try_lower_str_pad(ctx, inst, operation),
        RuntimeBuiltinId::MbOrd => try_lower_ord(ctx, inst, operation),
        _ => Ok(false),
    }
}

/// Lowers `mb_strtoupper()` or `mb_strtolower()` when the encoding is explicitly UTF-8.
fn try_lower_case(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    name: &str,
    uppercase: bool,
) -> Result<bool> {
    if inst.operands.len() != 2 || !super::explicit_utf8_encoding(ctx, inst.operands[1])? {
        return Ok(false);
    }
    let helper = if uppercase { "__rt_mb_strtoupper_utf8" } else { "__rt_mb_strtolower_utf8" };
    try_lower_ascii_string(ctx, inst, operation, name, helper)
}

/// Lowers a two-argument ASCII string transform with a null-pointer fallback.
fn try_lower_ascii_string(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    name: &str,
    helper: &str,
) -> Result<bool> {
    if inst.operands.len() != 2 || !super::explicit_utf8_encoding(ctx, inst.operands[1])? {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, helper);
    let fallback = ascii_fallback_label(ctx, "mb_ascii_string_fallback");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers `mb_strwidth()` over an explicit UTF-8 string whose bytes are all ASCII.
fn try_lower_strwidth(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 2
        || !super::explicit_utf8_encoding(ctx, inst.operands[1])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_strwidth", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ctx.next_label("mb_strwidth_ascii_fallback");
    let done = ctx.next_label("mb_strwidth_ascii_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x0, {fallback}"));           // non-ASCII subjects need the width table
            ctx.emitter.instruction("mov x0, x2");                            // every ASCII codepoint has display width one
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                          // did every byte stay inside ASCII?
            ctx.emitter.instruction(&format!("jz {fallback}"));                // non-ASCII subjects need the width table
            ctx.emitter.instruction("mov rax, rdx");                          // every ASCII codepoint has display width one
        }
    }
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, operation, false)?;
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(true)
}

/// Lowers `mb_convert_case()` when the mode is literal and the encoding is explicitly UTF-8.
fn try_lower_convert_case(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 3 || !super::explicit_utf8_encoding(ctx, inst.operands[2])? {
        return Ok(false);
    }
    let Some(mode) = const_int_operand(ctx, inst.operands[1])? else {
        return Ok(false);
    };
    let helper = match mode {
        0 | 4 => "__rt_mb_strtoupper_utf8",
        1 | 3 | 5 | 7 => "__rt_mb_strtolower_utf8",
        2 | 6 => "__rt_mb_convert_case_title_utf8",
        _ => return Ok(false),
    };
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_convert_case", pointer, length)?;
    abi::emit_call_label(ctx.emitter, helper);
    let fallback = ascii_fallback_label(ctx, "mb_convert_case_ascii_fallback");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers `mb_check_encoding()` over an explicit UTF-8 string without entering the bridge.
fn try_lower_check_encoding(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    _operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 2
        || !super::explicit_utf8_encoding(ctx, inst.operands[1])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_check_encoding", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_strlen_utf8");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #-1");                            // did the validator reject the subject?
            ctx.emitter.instruction("cset x0, ne");                           // a valid count means the encoding check succeeds
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, -1");                           // did the validator reject the subject?
            ctx.emitter.instruction("setne al");                              // a valid count means the encoding check succeeds
            ctx.emitter.instruction("movzx rax, al");                         // clear the stale high bits of the boolean result
        }
    }
    store_if_result(ctx, inst)?;
    Ok(true)
}

/// Lowers `mb_trim()`/`mb_ltrim()`/`mb_rtrim()` when the subject, mask, and encoding are ASCII UTF-8.
fn try_lower_trim(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    name: &str,
    helper: &str,
) -> Result<bool> {
    if inst.operands.len() != 3
        || !super::explicit_utf8_encoding(ctx, inst.operands[2])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
        || ctx.value_php_type(inst.operands[1])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_trim_subject_fallback");
    load_string_arg_to_regs(ctx, inst, 1, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    // Both operands are proven ASCII; bytewise trimming is identical to character trimming.
    load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
    super::super::common::lower_trim_mask_arg(ctx, inst, name)?;
    abi::emit_call_label(ctx.emitter, helper);
    abi::emit_call_label(ctx.emitter, "__rt_str_dup_owned");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers `mb_substr_count()` when both strings and the encoding are ASCII UTF-8.
fn try_lower_substr_count(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 3
        || !super::explicit_utf8_encoding(ctx, inst.operands[2])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
        || ctx.value_php_type(inst.operands[1])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_substr_count", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_substr_count_subject_fallback");
    load_string_arg_to_regs(ctx, inst, 1, "mb_substr_count", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    super::super::common::load_binary_string_args(ctx, inst, "mb_substr_count")?;
    abi::emit_call_label(ctx.emitter, "__rt_substr_count");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers ASCII `mb_substr()`/`mb_strcut()` through the ordinary substring bounds and an owned copy.
fn try_lower_substr_like(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    name: &str,
) -> Result<bool> {
    if inst.operands.len() != 4
        || !super::explicit_utf8_encoding(ctx, inst.operands[3])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_substr_ascii_fallback");
    // Byte offsets equal codepoint offsets for ASCII, so reuse PHP's exact substring bounds.
    let mut synthetic = inst.clone();
    synthetic.operands.truncate(3);
    let length_kind = super::super::search::optional_null_length_kind(ctx, &synthetic, 2)?;
    let neg_done = ctx.next_label("mb_substr_neg_done");
    let len_done = ctx.next_label("mb_substr_len_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => super::super::search::lower_substr_aarch64(
            ctx,
            &synthetic,
            length_kind,
            &neg_done,
            &len_done,
        )?,
        Arch::X86_64 => super::super::search::lower_substr_x86_64(
            ctx,
            &synthetic,
            length_kind,
            &neg_done,
            &len_done,
        )?,
    }
    abi::emit_call_label(ctx.emitter, "__rt_str_dup_owned");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers ASCII `mb_str_pad()` through the ordinary byte-padding helper and an owned copy.
fn try_lower_str_pad(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 5
        || !super::explicit_utf8_encoding(ctx, inst.operands[4])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
        || ctx.value_php_type(inst.operands[2])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_str_pad", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_str_pad_subject_fallback");
    load_string_arg_to_regs(ctx, inst, 2, "mb_str_pad", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    // ASCII padding units are one byte wide, so PHP's byte padding matches UTF-8 padding.
    let mut synthetic = inst.clone();
    synthetic.operands.truncate(4);
    match ctx.emitter.target.arch {
        Arch::AArch64 => super::super::replace_wrap::lower_str_pad_aarch64(ctx, &synthetic)?,
        Arch::X86_64 => super::super::replace_wrap::lower_str_pad_x86_64(ctx, &synthetic)?,
    }
    super::super::emit_str_pad_argument_guards(ctx, true);
    abi::emit_call_label(ctx.emitter, "__rt_str_pad");
    abi::emit_call_label(ctx.emitter, "__rt_str_dup_owned");
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers `mb_ord()` over an explicit UTF-8 ASCII subject into a boxed integer or false.
fn try_lower_ord(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
) -> Result<bool> {
    if inst.operands.len() != 2
        || !super::explicit_utf8_encoding(ctx, inst.operands[1])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_ord", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_ord_ascii_fallback");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x2, {fallback}"));          // an empty subject raises the bridge's ValueError
            ctx.emitter.instruction("ldrb w1, [x1]");                          // the first ASCII byte is the codepoint
            ctx.emitter.instruction("mov x2, xzr");                            // integer payloads have no high word
            ctx.emitter.instruction("mov x0, xzr");                            // runtime tag 0 = integer
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdx, rdx");                          // is the subject empty?
            ctx.emitter.instruction(&format!("jz {fallback}"));                // an empty subject raises the bridge's ValueError
            ctx.emitter.instruction("movzx edi, BYTE PTR [rax]");              // the first ASCII byte is the codepoint
            ctx.emitter.instruction("xor esi, esi");                           // integer payloads have no high word
            ctx.emitter.instruction("xor eax, eax");                           // runtime tag 0 = integer
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
        }
    }
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Emits a branch to `fallback` when the ASCII predicate in the result register is false.
///
/// Returns the fallback label so the caller can finish the convergence block.
fn ascii_predicate_fallback_label(ctx: &mut FunctionContext<'_>, prefix: &str) -> String {
    let fallback = ctx.next_label(prefix);
    emit_predicate_branch(ctx, &fallback);
    fallback
}

/// Emits a branch to an existing fallback label when the ASCII predicate failed.
fn emit_predicate_branch(ctx: &mut FunctionContext<'_>, fallback: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x0, {fallback}"));           // a zero predicate means a non-ASCII byte
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                          // did every byte stay inside ASCII?
            ctx.emitter.instruction(&format!("jz {fallback}"));                // a zero predicate means a non-ASCII byte
        }
    }
}

/// Branches around the shared bridge only when the ASCII helper returned a null pointer.
///
/// Returns the fallback label so the caller can finish the convergence block.
fn ascii_fallback_label(ctx: &mut FunctionContext<'_>, prefix: &str) -> String {
    let fallback = ctx.next_label(prefix);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x1, {fallback}"));           // a null result pointer means the subject is not ASCII
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                          // did the ASCII converter accept the subject?
            ctx.emitter.instruction(&format!("jz {fallback}"));                // a null result pointer means the subject is not ASCII
        }
    }
    fallback
}

/// Emits the shared bridge fallback, converges both branches, and stores the result.
fn finish_fast_path(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    fallback: String,
) -> Result<bool> {
    let done = ctx.next_label("mb_ascii_fast_done");
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, operation, false)?;
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(true)
}

/// Returns a literal `i64` operand when the value was produced by `const_i64`.
fn const_int_operand(ctx: &FunctionContext<'_>, value: crate::ir::ValueId) -> Result<Option<i64>> {
    let Some(definition) = ctx.function.value(value) else { return Ok(None) };
    let crate::ir::ValueDef::Instruction { inst, .. } = definition.def else { return Ok(None) };
    let Some(instruction) = ctx.function.instruction(inst) else { return Ok(None) };
    Ok(match (instruction.op, &instruction.immediate) {
        (crate::ir::Op::ConstI64, Some(crate::ir::Immediate::I64(value))) => Some(*value),
        _ => None,
    })
}
