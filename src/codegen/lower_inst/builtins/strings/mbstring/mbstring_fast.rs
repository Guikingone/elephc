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
        RuntimeBuiltinId::MbStrpos => try_lower_position(
            ctx,
            inst,
            operation,
            "mb_strpos",
            "__rt_strpos",
            super::super::StringPositionDirection::Forward,
        ),
        RuntimeBuiltinId::MbStripos => try_lower_position(
            ctx,
            inst,
            operation,
            "mb_stripos",
            "__rt_stripos",
            super::super::StringPositionDirection::Forward,
        ),
        RuntimeBuiltinId::MbStrrpos => try_lower_position(
            ctx,
            inst,
            operation,
            "mb_strrpos",
            "__rt_strrpos",
            super::super::StringPositionDirection::Reverse,
        ),
        RuntimeBuiltinId::MbStrripos => try_lower_position(
            ctx,
            inst,
            operation,
            "mb_strripos",
            "__rt_strripos",
            super::super::StringPositionDirection::Reverse,
        ),
        RuntimeBuiltinId::MbStrstr => try_lower_strstr(ctx, inst, operation, "__rt_strpos"),
        RuntimeBuiltinId::MbStristr => try_lower_strstr(ctx, inst, operation, "__rt_stripos"),
        RuntimeBuiltinId::MbStrrchr => try_lower_strstr(ctx, inst, operation, "__rt_strrpos"),
        RuntimeBuiltinId::MbStrrichr => try_lower_strstr(ctx, inst, operation, "__rt_strripos"),
        RuntimeBuiltinId::MbScrub => try_lower_scrub(ctx, inst, operation),
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

/// Lowers ASCII `mb_strpos()`-family searches through the ordinary position helpers.
fn try_lower_position(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    name: &str,
    runtime_label: &str,
    direction: super::super::StringPositionDirection,
) -> Result<bool> {
    if inst.operands.len() != 4
        || !super::explicit_utf8_encoding(ctx, inst.operands[3])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
        || ctx.value_php_type(inst.operands[1])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_position_haystack_fallback");
    load_string_arg_to_regs(ctx, inst, 1, name, pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    // ASCII byte offsets equal UTF-8 codepoint offsets, so reuse PHP's exact search bounds.
    let mut synthetic = inst.clone();
    synthetic.operands.truncate(3);
    super::super::search::lower_string_position_no_store(
        ctx,
        &synthetic,
        name,
        runtime_label,
        direction,
    )?;
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers a raw integer `(int)mb_strpos()`-family result selected by the cast sink.
pub(super) fn lower_position_int(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    source: RuntimeBuiltinId,
    name: &str,
    runtime_label: &str,
    direction: super::super::StringPositionDirection,
) -> Result<()> {
    let fast = inst.operands.len() == 4
        && super::explicit_utf8_encoding(ctx, inst.operands[3])?
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str
        && ctx.value_php_type(inst.operands[1])?.codegen_repr() == PhpType::Str;
    let (fallback, done) = if fast {
        let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
        load_string_arg_to_regs(ctx, inst, 0, name, pointer, length)?;
        abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
        let fallback = ascii_predicate_fallback_label(ctx, "mb_position_int_haystack_fallback");
        load_string_arg_to_regs(ctx, inst, 1, name, pointer, length)?;
        abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
        emit_predicate_branch(ctx, &fallback);
        let mut synthetic = inst.clone();
        synthetic.operands.truncate(3);
        super::super::search::lower_string_position_int_no_store(
            ctx,
            &synthetic,
            name,
            runtime_label,
            direction,
        )?;
        let done = ctx.next_label("mb_position_int_done");
        abi::emit_jump(ctx.emitter, &done);
        ctx.emitter.label(&fallback);
        (true, Some(done))
    } else {
        (false, None)
    };
    super::lower_mbstring_impl(ctx, inst, source, false)?;
    emit_unbox_boxed_int(ctx);
    if let Some(done) = done {
        ctx.emitter.label(&done);
    }
    emit_negative_to_zero(ctx);
    store_if_result(ctx, inst)?;
    let _ = fallback;
    Ok(())
}

/// Lowers a raw integer `(int)mb_ord()` result selected by the cast sink.
pub(super) fn lower_ord_int(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let fast = inst.operands.len() == 2
        && super::explicit_utf8_encoding(ctx, inst.operands[1])?
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbOrd, false)?;
        emit_unbox_boxed_int(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_ord", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_ord_int_ascii_fallback");
    let done = ctx.next_label("mb_ord_int_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbz x2, {fallback}"));           // an empty subject raises the bridge's ValueError
            ctx.emitter.instruction("ldrb w0, [x1]");                          // the first ASCII byte is the codepoint
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rdx, rdx");                          // is the subject empty?
            ctx.emitter.instruction(&format!("jz {fallback}"));                // an empty subject raises the bridge's ValueError
            ctx.emitter.instruction("movzx eax, BYTE PTR [rax]");              // the first ASCII byte is the codepoint
        }
    }
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbOrd, false)?;
    emit_unbox_boxed_int(ctx);
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Lowers `mb_scrub()` over an already-valid explicit UTF-8 subject to an owned copy.
fn try_lower_scrub(
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
    load_string_arg_to_regs(ctx, inst, 0, "mb_scrub", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_strlen_utf8");
    let fallback = ctx.next_label("mb_scrub_invalid_fallback");
    let done = ctx.next_label("mb_scrub_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #-1");                            // did the validator reject malformed UTF-8?
            ctx.emitter.instruction(&format!("b.eq {fallback}"));             // malformed input needs the replacement bridge
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, -1");                           // did the validator reject malformed UTF-8?
            ctx.emitter.instruction(&format!("je {fallback}"));               // malformed input needs the replacement bridge
        }
    }
    // Valid UTF-8 scrubbing is a byte-for-byte copy, so reuse the owned-slice helper.
    load_string_arg_to_regs(ctx, inst, 0, "mb_scrub", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_str_dup_owned");
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, operation, false)?;
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(true)
}

/// Lowers `strlen(mb_detect_encoding(...))` for a single explicit UTF-8 encoding list.
pub(super) fn lower_detect_encoding_len(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let fast = inst.operands.len() == 2
        && super::explicit_utf8_encoding(ctx, inst.operands[1])?
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbDetectEncoding, false)?;
        emit_boxed_string_length(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_detect_encoding", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_strlen_utf8");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #-1");                            // did the validator reject malformed UTF-8?
            ctx.emitter.instruction("mov x9, #5");                             // detected encoding name "UTF-8" has five bytes
            ctx.emitter.instruction("csel x0, xzr, x9, eq");                   // invalid input detects false, whose length is zero
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, -1");                           // did the validator reject malformed UTF-8?
            ctx.emitter.instruction("mov r10, 5");                            // detected encoding name "UTF-8" has five bytes
            ctx.emitter.instruction("mov r11, 0");                            // invalid input detects false, whose length is zero
            ctx.emitter.instruction("cmove r10, r11");                        // select zero for malformed input
            ctx.emitter.instruction("mov rax, r10");                          // carry the selected name length
        }
    }
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Lowers `strlen(mb_chr(...))` to the UTF-8 byte length of one explicit-UTF-8 codepoint.
pub(super) fn lower_chr_len(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let fast = inst.operands.len() == 2
        && super::explicit_utf8_encoding(ctx, inst.operands[1])?;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbChr, false)?;
        emit_boxed_string_length(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    load_as_int(ctx, inst.operands[0], "mb_chr codepoint")?;
    let zero = ctx.next_label("mb_chr_len_zero");
    let one = ctx.next_label("mb_chr_len_one");
    let two = ctx.next_label("mb_chr_len_two");
    let three = ctx.next_label("mb_chr_len_three");
    let four = ctx.next_label("mb_chr_len_four");
    let done = ctx.next_label("mb_chr_len_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                             // is the codepoint negative?
            ctx.emitter.instruction(&format!("b.lt {zero}"));                  // invalid codepoints encode as false
            ctx.emitter.instruction("cmp x0, #0x80");                          // does the codepoint fit one UTF-8 byte?
            ctx.emitter.instruction(&format!("b.lo {one}"));                   // a one-byte sequence
            ctx.emitter.instruction("cmp x0, #0x800");                         // does the codepoint fit two UTF-8 bytes?
            ctx.emitter.instruction(&format!("b.lo {two}"));                   // a two-byte sequence
            ctx.emitter.instruction("mov w9, #0xD800");                        // materialize the first surrogate codepoint
            ctx.emitter.instruction("cmp x0, x9");                             // is the codepoint below the surrogate range?
            ctx.emitter.instruction(&format!("b.lo {three}"));                 // a three-byte sequence
            ctx.emitter.instruction("mov w9, #0xE000");                        // materialize the first post-surrogate codepoint
            ctx.emitter.instruction("cmp x0, x9");                             // is the codepoint inside the surrogate range?
            ctx.emitter.instruction(&format!("b.lo {zero}"));                  // surrogates encode as false
            ctx.emitter.instruction("mov w9, #0x10000");                       // materialize the first four-byte codepoint
            ctx.emitter.instruction("cmp x0, x9");                             // does the codepoint fit three UTF-8 bytes?
            ctx.emitter.instruction(&format!("b.lo {three}"));                 // a three-byte sequence
            ctx.emitter.instruction("mov w9, #0x110000");                      // materialize one past the last Unicode codepoint
            ctx.emitter.instruction("cmp x0, x9");                             // is the codepoint beyond Unicode?
            ctx.emitter.instruction(&format!("b.lo {four}"));                  // a four-byte sequence
            abi::emit_jump(ctx.emitter, &zero);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                         // is the codepoint negative?
            ctx.emitter.instruction(&format!("js {zero}"));                    // invalid codepoints encode as false
            ctx.emitter.instruction("cmp rax, 0x80");                         // does the codepoint fit one UTF-8 byte?
            ctx.emitter.instruction(&format!("jb {one}"));                     // a one-byte sequence
            ctx.emitter.instruction("cmp rax, 0x800");                        // does the codepoint fit two UTF-8 bytes?
            ctx.emitter.instruction(&format!("jb {two}"));                     // a two-byte sequence
            ctx.emitter.instruction("cmp rax, 0xD800");                       // is the codepoint below the surrogate range?
            ctx.emitter.instruction(&format!("jb {three}"));                   // a three-byte sequence
            ctx.emitter.instruction("cmp rax, 0xE000");                       // is the codepoint inside the surrogate range?
            ctx.emitter.instruction(&format!("jb {zero}"));                    // surrogates encode as false
            ctx.emitter.instruction("cmp rax, 0x10000");                      // does the codepoint fit three UTF-8 bytes?
            ctx.emitter.instruction(&format!("jb {three}"));                   // a three-byte sequence
            ctx.emitter.instruction("cmp rax, 0x110000");                     // is the codepoint beyond Unicode?
            ctx.emitter.instruction(&format!("jb {four}"));                    // a four-byte sequence
            abi::emit_jump(ctx.emitter, &zero);
        }
    }
    for (label, length) in [(&one, 1), (&two, 2), (&three, 3), (&four, 4), (&zero, 0)] {
        ctx.emitter.label(label);
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), length);
        abi::emit_jump(ctx.emitter, &done);
    }
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Lowers `strlen(mb_strimwidth(...))` for a zero start and explicit UTF-8 ASCII strings.
pub(super) fn lower_strimwidth_len(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let fast = inst.operands.len() == 5
        && super::explicit_utf8_encoding(ctx, inst.operands[4])?
        && const_int_operand(ctx, inst.operands[1])? == Some(0)
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str
        && ctx.value_php_type(inst.operands[3])?.codegen_repr() == PhpType::Str;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbStrimwidth, false)?;
        move_string_length_to_result(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_strimwidth", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_strimwidth_subject_fallback");
    load_string_arg_to_regs(ctx, inst, 3, "mb_strimwidth", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    let done = ctx.next_label("mb_strimwidth_len_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            load_string_arg_to_regs(ctx, inst, 0, "mb_strimwidth", "x1", "x2")?;
            ctx.emitter.instruction("str x2, [sp, #-16]!");                    // preserve the subject byte length
            load_string_arg_to_regs(ctx, inst, 3, "mb_strimwidth", "x1", "x2")?;
            ctx.emitter.instruction("str x2, [sp, #-16]!");                    // preserve the trim marker byte length
            load_as_int(ctx, inst.operands[2], "mb_strimwidth width")?;
            ctx.emitter.instruction("cmp x0, #0");                             // is the requested width negative?
            let cleanup = ctx.next_label("mb_strimwidth_negative_cleanup");
            ctx.emitter.instruction(&format!("b.lt {cleanup}"));               // negative widths keep the deprecated bridge path
            ctx.emitter.instruction("mov x10, x0");                            // preserve the requested width
            ctx.emitter.instruction("ldr x11, [sp], #16");                     // reload the trim marker byte length
            ctx.emitter.instruction("ldr x2, [sp], #16");                      // reload the subject byte length
            ctx.emitter.instruction("cmp x2, x10");                            // does the subject exceed the requested width?
            let full = ctx.next_label("mb_strimwidth_full_length");
            ctx.emitter.instruction(&format!("b.le {full}"));                  // no marker is appended when the subject fits
            ctx.emitter.instruction("cmp x10, x11");                           // is the width no longer than the marker itself?
            ctx.emitter.instruction("csel x0, x11, x10, le");                  // a short width yields the whole marker
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&full);
            ctx.emitter.instruction("mov x0, x2");                             // the whole subject is returned unchanged
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&cleanup);
            ctx.emitter.instruction("add sp, sp, #32");                        // drop both saved lengths before the bridge
            abi::emit_jump(ctx.emitter, &fallback);
        }
        Arch::X86_64 => {
            load_string_arg_to_regs(ctx, inst, 0, "mb_strimwidth", "rax", "rdx")?;
            ctx.emitter.instruction("push rdx");                               // preserve the subject byte length
            load_string_arg_to_regs(ctx, inst, 3, "mb_strimwidth", "rax", "rdx")?;
            ctx.emitter.instruction("push rdx");                               // preserve the trim marker byte length
            load_as_int(ctx, inst.operands[2], "mb_strimwidth width")?;
            ctx.emitter.instruction("test rax, rax");                          // is the requested width negative?
            let cleanup = ctx.next_label("mb_strimwidth_negative_cleanup_x86");
            ctx.emitter.instruction(&format!("js {cleanup}"));                 // negative widths keep the deprecated bridge path
            ctx.emitter.instruction("mov rcx, rax");                           // preserve the requested width
            ctx.emitter.instruction("pop r10");                                // reload the trim marker byte length
            ctx.emitter.instruction("pop rsi");                                // reload the subject byte length
            ctx.emitter.instruction("cmp rsi, rcx");                           // does the subject exceed the requested width?
            let full = ctx.next_label("mb_strimwidth_full_length_x86");
            ctx.emitter.instruction(&format!("jle {full}"));                   // no marker is appended when the subject fits
            ctx.emitter.instruction("mov rax, rcx");                           // a truncated result keeps the requested width
            ctx.emitter.instruction("cmp rcx, r10");                           // is the width no longer than the marker itself?
            ctx.emitter.instruction("cmovle rax, r10");                        // a short width yields the whole marker
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&full);
            ctx.emitter.instruction("mov rax, rsi");                           // the whole subject is returned unchanged
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&cleanup);
            ctx.emitter.instruction("add rsp, 16");                            // drop both saved lengths before the bridge
            abi::emit_jump(ctx.emitter, &fallback);
        }
    }
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbStrimwidth, false)?;
    move_string_length_to_result(ctx);
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Moves a native string pair's length into the integer result register.
fn move_string_length_to_result(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x2");                             // return the produced string's byte length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rax, rdx");                          // return the produced string's byte length
        }
    }
}

/// Unboxes a boxed `int|false` result and releases its temporary Mixed cell.
fn emit_unbox_boxed_int(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #32");                        // reserve the cell and payload across unboxing
            ctx.emitter.instruction("str x0, [sp, #0]");                       // save the boxed cell before unboxing
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("str x1, [sp, #8]");                       // save the integer payload before releasing the cell
            ctx.emitter.instruction("ldr x0, [sp, #0]");                       // reload the cell for its final release
            abi::emit_call_label(ctx.emitter, "__rt_decref_any");
            ctx.emitter.instruction("ldr x0, [sp, #8]");                       // return the unboxed integer payload
            ctx.emitter.instruction("add sp, sp, #32");                        // release the unboxing spill slots
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 32");                            // reserve the cell and payload across unboxing
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");               // save the boxed cell before unboxing
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rdi");           // save the integer payload before releasing the cell
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");               // reload the cell for its final release
            abi::emit_call_label(ctx.emitter, "__rt_decref_any");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");           // return the unboxed integer payload
            ctx.emitter.instruction("add rsp, 32");                            // release the unboxing spill slots
        }
    }
}

/// Normalizes a not-found signed position to PHP's `(int)false` zero.
fn emit_negative_to_zero(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                             // did the search report its negative not-found sentinel?
            ctx.emitter.instruction("csel x0, xzr, x0, lt");                    // map not-found to zero for the integer observer
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                          // did the search report its negative not-found sentinel?
            ctx.emitter.instruction("mov r10, 0");                             // materialize the integer zero replacement
            ctx.emitter.instruction("cmovs rax, r10");                         // map not-found to zero for the integer observer
        }
    }
}

/// Lowers ASCII `mb_strstr()`-family searches through the ordinary substring helpers.
fn try_lower_strstr(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operation: RuntimeBuiltinId,
    runtime_label: &str,
) -> Result<bool> {
    if inst.operands.len() != 4
        || !super::explicit_utf8_encoding(ctx, inst.operands[3])?
        || ctx.value_php_type(inst.operands[0])?.codegen_repr() != PhpType::Str
        || ctx.value_php_type(inst.operands[1])?.codegen_repr() != PhpType::Str
    {
        return Ok(false);
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_strstr", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_strstr_haystack_fallback");
    load_string_arg_to_regs(ctx, inst, 1, "mb_strstr", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    emit_predicate_branch(ctx, &fallback);
    // ASCII byte matches equal UTF-8 character matches, so reuse PHP's substring helpers.
    let mut synthetic = inst.clone();
    synthetic.operands.truncate(3);
    super::super::search::lower_strstr_no_store(ctx, &synthetic, runtime_label)?;
    finish_fast_path(ctx, inst, operation, fallback)
}

/// Lowers `strlen(mb_strstr(...))` through the allocation-free ASCII length helper.
pub(super) fn lower_strstr_len(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    source: RuntimeBuiltinId,
    mode: i64,
) -> Result<()> {
    let fast = inst.operands.len() == 4
        && super::explicit_utf8_encoding(ctx, inst.operands[3])?
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str
        && ctx.value_php_type(inst.operands[1])?.codegen_repr() == PhpType::Str;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, source, false)?;
        emit_boxed_string_length(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    super::super::common::materialize_truthy_flag(ctx, inst, 2, "mb_strstr")?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_push_reg(ctx.emitter, "x0");                             // park the before-needle flag
            load_string_arg_to_regs(ctx, inst, 0, "mb_strstr", "x1", "x2")?;
            ctx.emitter.instruction("stp x1, x2, [sp, #-16]!");                 // preserve the haystack while loading the needle
            load_string_arg_to_regs(ctx, inst, 1, "mb_strstr", "x1", "x2")?;
            ctx.emitter.instruction("mov x3, x1");                             // pass the needle pointer as the secondary string argument
            ctx.emitter.instruction("mov x4, x2");                             // pass the needle length as the secondary string argument
            ctx.emitter.instruction("ldp x1, x2, [sp], #16");                   // restore the haystack into the primary string argument
            abi::emit_pop_reg(ctx.emitter, "x5");                              // restore the before-needle flag
            abi::emit_load_int_immediate(ctx.emitter, "x6", mode);             // select the forward/reverse case mode
        }
        Arch::X86_64 => {
            abi::emit_push_reg(ctx.emitter, "rax");                            // park the before-needle flag
            load_string_arg_to_regs(ctx, inst, 0, "mb_strstr", "rax", "rdx")?;
            abi::emit_push_reg_pair(ctx.emitter, "rax", "rdx");                // preserve the haystack while loading the needle
            load_string_arg_to_regs(ctx, inst, 1, "mb_strstr", "rax", "rdx")?;
            ctx.emitter.instruction("mov r10, rax");                           // preserve the needle pointer
            ctx.emitter.instruction("mov r11, rdx");                           // preserve the needle length
            abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi");                 // restore the haystack into the primary SysV string arguments
            abi::emit_pop_reg(ctx.emitter, "r8");                              // restore the before-needle flag
            ctx.emitter.instruction("mov rdx, r10");                           // pass the needle pointer as the third SysV string argument
            ctx.emitter.instruction("mov rcx, r11");                           // pass the needle length as the fourth SysV string argument
            abi::emit_load_int_immediate(ctx.emitter, "r9", mode);             // select the forward/reverse case mode
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mb_strstr_len_utf8");
    let fallback = ctx.next_label("mb_strstr_len_fallback");
    let done = ctx.next_label("mb_strstr_len_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #-1");                             // did the ASCII helper decline the operands?
            ctx.emitter.instruction(&format!("b.eq {fallback}"));              // non-ASCII operands need the Unicode bridge
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, -1");                            // did the ASCII helper decline the operands?
            ctx.emitter.instruction(&format!("je {fallback}"));                // non-ASCII operands need the Unicode bridge
        }
    }
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, source, false)?;
    emit_boxed_string_length(ctx);
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Unboxes a `string|false` cell and returns the byte length of a string, or zero for false.
fn emit_boxed_string_length(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #32");                        // reserve the cell, tag, and length
            ctx.emitter.instruction("str x0, [sp, #0]");                       // save the boxed cell before unboxing
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("str x0, [sp, #8]");                       // save the runtime tag
            ctx.emitter.instruction("str x2, [sp, #16]");                      // save the string byte length
            ctx.emitter.instruction("ldr x0, [sp, #0]");                       // reload the cell for its final release
            abi::emit_call_label(ctx.emitter, "__rt_decref_any");
            ctx.emitter.instruction("ldr x0, [sp, #16]");                      // start with the string byte length
            ctx.emitter.instruction("ldr x9, [sp, #8]");                       // reload the runtime tag
            ctx.emitter.instruction("cmp x9, #1");                             // is this a boxed string?
            ctx.emitter.instruction("csel x0, xzr, x0, ne");                   // false and null have length zero
            ctx.emitter.instruction("add sp, sp, #32");                        // release the unboxing spill slots
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 32");                            // reserve the cell, tag, and length
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");               // save the boxed cell before unboxing
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], rax");           // save the runtime tag
            ctx.emitter.instruction("mov QWORD PTR [rsp + 16], rsi");          // save the string byte length
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp]");               // reload the cell for its final release
            abi::emit_call_label(ctx.emitter, "__rt_decref_any");
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 16]");          // start with the string byte length
            ctx.emitter.instruction("mov r10, QWORD PTR [rsp + 8]");           // reload the runtime tag
            ctx.emitter.instruction("cmp r10, 1");                             // is this a boxed string?
            ctx.emitter.instruction("mov r11, 0");                             // materialize the zero length for false and null
            ctx.emitter.instruction("cmovne rax, r11");                        // false and null have length zero
            ctx.emitter.instruction("add rsp, 32");                            // release the unboxing spill slots
        }
    }
}

/// Lowers `count(mb_str_split(...))` through the allocation-free ASCII chunk count.
pub(super) fn lower_str_split_count(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let fast = inst.operands.len() == 3
        && super::explicit_utf8_encoding(ctx, inst.operands[2])?
        && ctx.value_php_type(inst.operands[0])?.codegen_repr() == PhpType::Str;
    if !fast {
        super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbStrSplit, false)?;
        emit_array_length_and_release(ctx);
        store_if_result(ctx, inst)?;
        return Ok(());
    }
    let (pointer, length) = (string_ptr_reg(ctx), string_len_reg(ctx));
    load_string_arg_to_regs(ctx, inst, 0, "mb_str_split", pointer, length)?;
    abi::emit_call_label(ctx.emitter, "__rt_mb_is_ascii");
    let fallback = ascii_predicate_fallback_label(ctx, "mb_str_split_count_ascii_fallback");
    let done = ctx.next_label("mb_str_split_count_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("str x2, [sp, #-16]!");                    // preserve the subject length across the chunk load
            load_as_int(ctx, inst.operands[1], "mb_str_split length")?;
            ctx.emitter.instruction("cmp x0, #0");                             // is the chunk length positive?
            let cleanup = ctx.next_label("mb_str_split_count_cleanup");
            ctx.emitter.instruction(&format!("b.le {cleanup}"));               // a non-positive chunk lets the bridge raise its ValueError
            ctx.emitter.instruction("mov x10, x0");                            // preserve the chunk length
            ctx.emitter.instruction("ldr x0, [sp], #16");                      // reload the subject byte length
            ctx.emitter.instruction("add x0, x0, x10");                        // add one chunk so the ceiling divide is exact
            ctx.emitter.instruction("sub x0, x0, #1");                         // complete the ceiling numerator
            ctx.emitter.instruction("udiv x0, x0, x10");                       // ceil(subject bytes / chunk bytes)
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&cleanup);
            ctx.emitter.instruction("add sp, sp, #16");                        // drop the saved subject length before the bridge
            abi::emit_jump(ctx.emitter, &fallback);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rdx");                              // preserve the subject length across the chunk load
            load_as_int(ctx, inst.operands[1], "mb_str_split length")?;
            ctx.emitter.instruction("test rax, rax");                         // is the chunk length positive?
            let cleanup = ctx.next_label("mb_str_split_count_cleanup_x86");
            ctx.emitter.instruction(&format!("jle {cleanup}"));                // a non-positive chunk lets the bridge raise its ValueError
            ctx.emitter.instruction("mov rcx, rax");                          // preserve the chunk length
            ctx.emitter.instruction("pop rax");                               // reload the subject byte length
            ctx.emitter.instruction("lea rax, [rax + rcx - 1]");              // add one chunk so the ceiling divide is exact
            ctx.emitter.instruction("xor edx, edx");                          // clear the dividend high half
            ctx.emitter.instruction("div rcx");                               // ceil(subject bytes / chunk bytes)
            abi::emit_jump(ctx.emitter, &done);
            ctx.emitter.label(&cleanup);
            ctx.emitter.instruction("pop rdx");                               // drop the saved subject length before the bridge
            abi::emit_jump(ctx.emitter, &fallback);
        }
    }
    ctx.emitter.label(&fallback);
    super::lower_mbstring_impl(ctx, inst, RuntimeBuiltinId::MbStrSplit, false)?;
    emit_array_length_and_release(ctx);
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)?;
    Ok(())
}

/// Reads an indexed array's length header and releases the temporary array.
fn emit_array_length_and_release(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #32");                        // reserve the array pointer and its length
            ctx.emitter.instruction("str x0, [sp, #0]");                       // save the array before reading its length
            ctx.emitter.instruction("ldr x9, [x0]");                           // load the indexed-array length header
            ctx.emitter.instruction("str x9, [sp, #8]");                       // save the length across the array release
            ctx.emitter.instruction("bl __rt_decref_any");                    // release the temporary split array
            ctx.emitter.instruction("ldr x0, [sp, #8]");                       // return the array length
            ctx.emitter.instruction("add sp, sp, #32");                        // release the length spill slots
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("sub rsp, 32");                            // reserve the array pointer and its length
            ctx.emitter.instruction("mov QWORD PTR [rsp], rax");               // save the array before reading its length
            ctx.emitter.instruction("mov r10, QWORD PTR [rax]");               // load the indexed-array length header
            ctx.emitter.instruction("mov QWORD PTR [rsp + 8], r10");           // save the length across the array release
            ctx.emitter.instruction("call __rt_decref_any");                   // release the temporary split array
            ctx.emitter.instruction("mov rax, QWORD PTR [rsp + 8]");           // return the array length
            ctx.emitter.instruction("add rsp, 32");                            // release the length spill slots
        }
    }
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
