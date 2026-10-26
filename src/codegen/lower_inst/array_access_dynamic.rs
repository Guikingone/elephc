//! Purpose:
//! Dispatches boxed ArrayAccess objects before ordinary PHP array-key conversion.
//!
//! Called from:
//! - The boxed array read/write lowerers in `super::mixed_array_runtime`.
//!
//! Key details:
//! - Uses the existing boxed callback tables, including reference-return adapters.
//! - Missing native callbacks retain the original array/SPL runtime fallback.
//! - Shared argument materialization owns temporary cells across the callback.

use super::*;

/// Emits the object branch and returns its join label for the ordinary array fallback.
pub(super) fn emit_boxed_array_access_object_branch(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    method_name: &str,
) -> Result<Option<String>> {
    let receiver = expect_operand(inst, 0)?;
    if ctx.raw_value_php_type(receiver)?.is_php_array() { return Ok(None); }
    let table = match method_name {
        "offsetGet" => "_class_offsetget_ptrs",
        "offsetSet" => "_class_offsetset_ptrs",
        _ => return Err(CodegenIrError::unsupported("unknown boxed ArrayAccess callback")),
    };
    let fallback = ctx.next_label("mixed_array_callback_fallback");
    let done = ctx.next_label("mixed_array_callback_done");
    ctx.load_value_to_result(receiver)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // only object payloads have an ArrayAccess callback
            ctx.emitter.instruction(&format!("b.ne {fallback}"));               // preserve the ordinary array/scalar path
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // only object payloads have an ArrayAccess callback
            ctx.emitter.instruction(&format!("jne {fallback}"));                // preserve the ordinary array/scalar path
        }
    }
    let scratch = abi::secondary_scratch_reg(ctx.emitter);
    let class_id = abi::nested_call_reg(ctx.emitter);
    abi::emit_load_symbol_to_reg(ctx.emitter, scratch, "_class_iface_method_count", 0);
    abi::emit_load_from_address(ctx.emitter, class_id, mixed_unbox_low_payload_reg(ctx), 0);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp {class_id}, {scratch}"));     // compare the actual class id with the callback table bound
            ctx.emitter.instruction(&format!("b.hs {fallback}"));               // reject negative/synthetic and out-of-range ids
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp {class_id}, {scratch}"));     // compare the actual class id with the callback table bound
            ctx.emitter.instruction(&format!("jae {fallback}"));                // reject negative/synthetic and out-of-range ids
        }
    }
    abi::emit_symbol_address(ctx.emitter, scratch, table);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(
                &format!("ldr {scratch}, [{scratch}, {class_id}, lsl #3]")
            );                                                                  // resolve the existing boxed callback
            ctx.emitter.instruction(&format!("cbz {scratch}, {fallback}"));     // native-only SPL methods keep their established runtime adapter
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(
                &format!("mov {scratch}, QWORD PTR [{scratch} + {class_id}*8]")
            );                                                                  // resolve the existing boxed callback
            ctx.emitter.instruction(&format!("test {scratch}, {scratch}"));     // inspect the callback table entry
            ctx.emitter.instruction(&format!("jz {fallback}"));                 // native-only SPL methods keep their established runtime adapter
        }
    }
    abi::emit_push_reg(ctx.emitter, scratch);
    let call = Instruction {
        operands: inst.operands[..if method_name == "offsetSet" { 3 } else { 2 }].to_vec(),
        ..inst.clone()
    };
    emit_callback_call(ctx, &call, method_name)?;
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&fallback);
    Ok(Some(done))
}

/// Invokes the raw-object/boxed-offset table ABI without creating an interface return adapter.
fn emit_callback_call(
    ctx: &mut FunctionContext<'_>, inst: &Instruction, method_name: &str,
) -> Result<()> {
    let receiver = expect_operand(inst, 0)?;
    let receiver_ty = PhpType::Object("ArrayAccess".to_string());
    let count = inst.operands.len();
    let mut params = vec![PhpType::Mixed; count];
    params[0] = receiver_ty.clone();
    ctx.load_value_to_result(receiver)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, mixed_unbox_low_payload_reg(ctx));
    abi::emit_pop_reg(ctx.emitter, receiver_reg);
    let args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx, receiver_reg, &receiver_ty, &inst.operands, &params, &vec![false; count],
        crate::codegen::lower_inst::RefArgCellLifetime::CallOnly,
    )?;
    // Offset callbacks have register arguments; conversion cleanup records stay above the saved callee.
    debug_assert_eq!(args.overflow_bytes, 0);
    let callee = abi::nested_call_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, callee, args.cleanup_bytes);
    let pad = direct_call_stack_pad_bytes(ctx, args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, pad);
    abi::emit_call_reg(ctx.emitter, callee);
    abi::emit_release_temporary_stack(ctx.emitter, pad);
    let return_ty = if method_name == "offsetGet" { PhpType::Mixed } else { PhpType::Void };
    store_call_result(ctx, inst, &return_ty)?;
    emit_call_arg_temp_cleanups(ctx, &args, None)?;
    emit_ref_arg_writebacks(ctx, &args)?;
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    Ok(())
}
