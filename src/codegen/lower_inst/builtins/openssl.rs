//! Purpose:
//! Lowers PHP OpenSSL cipher builtins from typed EIR into target-aware runtime calls.
//!
//! Called from:
//! - Typed runtime-function dispatch for the four OpenSSL cipher builtins.
//!
//! Key details:
//! - Encrypt/decrypt arguments are staged in target-neutral field blocks before large C ABI calls.
//! - Runtime helpers own base64 handling and returned storage; the bridge always receives raw bytes.
//! - Phase 2 leaves GCM tag input/output empty until the dedicated AEAD lowering is added.

use crate::codegen::platform::Arch;
use crate::codegen::{abi, CodegenIrError, Result};
use crate::ir::Instruction;

use super::super::super::context::FunctionContext;
use super::strings::{
    load_as_int, load_single_string_arg, load_string_arg_to_regs, materialize_truthy_flag,
};
use super::store_if_result;

/// Lowers `openssl_encrypt()` through a stack field block consumed by the shared runtime helper.
pub(crate) fn lower_openssl_encrypt(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() < 3 || inst.operands.len() > 8 {
        return Err(CodegenIrError::invalid_module(format!(
            "openssl_encrypt expected 3 to 8 args, got {}",
            inst.operands.len()
        )));
    }
    const FRAME_SIZE: usize = 96;
    abi::emit_reserve_temporary_stack(ctx.emitter, FRAME_SIZE);
    stage_openssl_string_field(ctx, inst, 0, 0, "openssl_encrypt data")?;
    stage_openssl_string_field(ctx, inst, 1, 16, "openssl_encrypt cipher_algo")?;
    stage_openssl_string_field(ctx, inst, 2, 32, "openssl_encrypt passphrase")?;
    stage_openssl_int_field(ctx, inst, 3, 48, 0, "openssl_encrypt options")?;
    stage_openssl_optional_string_field(ctx, inst, 4, 56, "openssl_encrypt iv")?;
    stage_openssl_optional_string_field(ctx, inst, 6, 72, "openssl_encrypt aad")?;
    stage_openssl_int_field(ctx, inst, 7, 88, 16, "openssl_encrypt tag_length")?;
    crate::codegen::hash_crypto::publish_elephc_cipher_function_pointers(ctx.emitter);
    stage_openssl_frame_pointer(ctx);
    abi::emit_call_label(ctx.emitter, "__rt_openssl_encrypt");
    abi::emit_release_temporary_stack(ctx.emitter, FRAME_SIZE);
    super::io::box_owned_string_or_false_result(ctx, "openssl_encrypt");
    store_if_result(ctx, inst)
}

/// Lowers `openssl_decrypt()` through a stack field block consumed by the shared runtime helper.
pub(crate) fn lower_openssl_decrypt(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() < 3 || inst.operands.len() > 7 {
        return Err(CodegenIrError::invalid_module(format!(
            "openssl_decrypt expected 3 to 7 args, got {}",
            inst.operands.len()
        )));
    }
    const FRAME_SIZE: usize = 112;
    abi::emit_reserve_temporary_stack(ctx.emitter, FRAME_SIZE);
    stage_openssl_string_field(ctx, inst, 0, 0, "openssl_decrypt data")?;
    stage_openssl_string_field(ctx, inst, 1, 16, "openssl_decrypt cipher_algo")?;
    stage_openssl_string_field(ctx, inst, 2, 32, "openssl_decrypt passphrase")?;
    stage_openssl_int_field(ctx, inst, 3, 48, 0, "openssl_decrypt options")?;
    stage_openssl_optional_string_field(ctx, inst, 4, 56, "openssl_decrypt iv")?;
    stage_openssl_optional_string_field(ctx, inst, 6, 72, "openssl_decrypt aad")?;
    stage_openssl_empty_string_field(ctx, 88);
    crate::codegen::hash_crypto::publish_elephc_cipher_function_pointers(ctx.emitter);
    stage_openssl_frame_pointer(ctx);
    abi::emit_call_label(ctx.emitter, "__rt_openssl_decrypt");
    abi::emit_release_temporary_stack(ctx.emitter, FRAME_SIZE);
    super::io::box_owned_string_or_false_result(ctx, "openssl_decrypt");
    store_if_result(ctx, inst)
}

/// Lowers `openssl_cipher_iv_length()` and boxes unknown ciphers as PHP false.
pub(crate) fn lower_openssl_cipher_iv_length(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    load_single_string_arg(ctx, inst, "openssl_cipher_iv_length")?;
    crate::codegen::hash_crypto::publish_elephc_cipher_function_pointers(ctx.emitter);
    abi::emit_call_label(ctx.emitter, "__rt_openssl_cipher_iv_length");
    box_openssl_int_or_false_result(ctx, "openssl_iv_length");
    store_if_result(ctx, inst)
}

/// Lowers `openssl_get_cipher_methods()` through the bridge-backed packed inventory helper.
pub(crate) fn lower_openssl_get_cipher_methods(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    if inst.operands.len() > 1 {
        return Err(CodegenIrError::invalid_module(format!(
            "openssl_get_cipher_methods expected 0 or 1 args, got {}",
            inst.operands.len()
        )));
    }
    materialize_truthy_flag(ctx, inst, 0, "openssl_get_cipher_methods")?;
    crate::codegen::hash_crypto::publish_elephc_cipher_function_pointers(ctx.emitter);
    if ctx.emitter.target.arch == Arch::X86_64 {
        ctx.emitter.instruction("mov rdi, rax");                                // pass aliases flag to the method-list helper
    }
    abi::emit_call_label(ctx.emitter, "__rt_openssl_get_cipher_methods");
    store_if_result(ctx, inst)
}

/// Stages one required string operand into the OpenSSL runtime field block.
fn stage_openssl_string_field(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operand_index: usize,
    field_offset: usize,
    name: &str,
) -> Result<()> {
    let (ptr_reg, len_reg) = match ctx.emitter.target.arch {
        Arch::AArch64 => ("x1", "x2"),
        Arch::X86_64 => ("rax", "rdx"),
    };
    load_string_arg_to_regs(ctx, inst, operand_index, name, ptr_reg, len_reg)?;
    store_openssl_field_pair(ctx, field_offset, ptr_reg, len_reg);
    Ok(())
}

/// Stages an optional string operand, using a null pointer with zero length when omitted.
fn stage_openssl_optional_string_field(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operand_index: usize,
    field_offset: usize,
    name: &str,
) -> Result<()> {
    if inst.operands.get(operand_index).is_none() {
        stage_openssl_empty_string_field(ctx, field_offset);
        return Ok(());
    }
    stage_openssl_string_field(ctx, inst, operand_index, field_offset, name)
}

/// Writes a null/zero empty-string pair into an OpenSSL runtime field block.
fn stage_openssl_empty_string_field(ctx: &mut FunctionContext<'_>, field_offset: usize) {
    let scratch = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_temporary_stack_address(ctx.emitter, scratch, field_offset);
    abi::emit_store_zero_to_address(ctx.emitter, scratch, 0);
    abi::emit_store_zero_to_address(ctx.emitter, scratch, 8);
}

/// Stages one integer operand or a literal default into the OpenSSL runtime field block.
fn stage_openssl_int_field(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    operand_index: usize,
    field_offset: usize,
    default: i64,
    name: &str,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    if let Some(value) = inst.operands.get(operand_index).copied() {
        load_as_int(ctx, value, name)?;
    } else {
        abi::emit_load_int_immediate(ctx.emitter, result_reg, default);
    }
    let scratch = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_temporary_stack_address(ctx.emitter, scratch, field_offset);
    abi::emit_store_to_address(ctx.emitter, result_reg, scratch, 0);
    Ok(())
}

/// Stores a pointer/length pair into the current OpenSSL runtime field block.
fn store_openssl_field_pair(
    ctx: &mut FunctionContext<'_>,
    field_offset: usize,
    ptr_reg: &str,
    len_reg: &str,
) {
    let scratch = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_temporary_stack_address(ctx.emitter, scratch, field_offset);
    abi::emit_store_to_address(ctx.emitter, ptr_reg, scratch, 0);
    abi::emit_store_to_address(ctx.emitter, len_reg, scratch, 8);
}

/// Places the current temporary OpenSSL field-block address in the helper's first argument.
fn stage_openssl_frame_pointer(ctx: &mut FunctionContext<'_>) {
    let arg_reg = match ctx.emitter.target.arch {
        Arch::AArch64 => "x0",
        Arch::X86_64 => "rdi",
    };
    abi::emit_temporary_stack_address(ctx.emitter, arg_reg, 0);
}

/// Boxes a nonnegative integer result or any negative bridge status as PHP false.
fn box_openssl_int_or_false_result(ctx: &mut FunctionContext<'_>, label_prefix: &str) {
    let false_label = ctx.next_label(&format!("{}_false", label_prefix));
    let done_label = ctx.next_label(&format!("{}_done", label_prefix));
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #0");                              // negative IV-length status means unknown cipher
            ctx.emitter.instruction(&format!("b.lt {}", false_label));
            ctx.emitter.instruction("mov x1, x0");                              // integer Mixed payload
            ctx.emitter.instruction("mov x2, #0");
            ctx.emitter.instruction("mov x0, #0");                              // runtime tag 0 = integer
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("b {}", done_label));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov x1, #0");                              // false payload
            ctx.emitter.instruction("mov x2, #0");
            ctx.emitter.instruction("mov x0, #3");                              // runtime tag 3 = boolean
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.label(&done_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // negative IV-length status means unknown cipher
            ctx.emitter.instruction(&format!("js {}", false_label));
            ctx.emitter.instruction("mov rdi, rax");                            // integer Mixed payload
            ctx.emitter.instruction("xor esi, esi");
            ctx.emitter.instruction("xor eax, eax");                            // runtime tag 0 = integer
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.instruction(&format!("jmp {}", done_label));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("xor edi, edi");                            // false payload
            ctx.emitter.instruction("xor esi, esi");
            ctx.emitter.instruction("mov eax, 3");                              // runtime tag 3 = boolean
            abi::emit_call_label(ctx.emitter, "__rt_mixed_from_value");
            ctx.emitter.label(&done_label);
        }
    }
}
