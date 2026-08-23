//! Purpose:
//! Lowers boxed Mixed array reads, writes, and fetch-for-write operations.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Lowers binary runtime fallbacks that Phase 04 can identify by operand type.
pub(super) fn lower_binary_runtime_call(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let receiver = expect_operand(inst, 0)?;
    let receiver_ty = ctx.value_php_type(receiver)?.codegen_repr();
    let result_ty = inst.result_php_type.codegen_repr();
    match (receiver_ty, &result_ty) {
        (PhpType::Mixed | PhpType::Union(_), PhpType::Void) => {
            lower_mixed_cell_runtime_assign(ctx, inst)
        }
        (PhpType::Mixed | PhpType::Union(_), _) => {
            lower_mixed_array_runtime_get(ctx, inst, false)
        }
        (PhpType::AssocArray { .. }, PhpType::Void) => hashes::lower_hash_append(ctx, inst),
        (other, _) => Err(CodegenIrError::unsupported(format!(
            "runtime_call with receiver PHP type {:?} returning PHP type {:?}",
            other, inst.result_php_type
        ))),
    }
}

/// Lowers `$mixed[$key]` through the shared boxed Mixed array/hash/stdClass reader.
pub(super) fn lower_mixed_array_runtime_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    for_write: bool,
) -> Result<()> {
    let receiver = expect_operand(inst, 0)?;
    let key = expect_operand(inst, 1)?;
    let warn_on_missing = expect_operand(inst, 2)?;
    if !for_write && value_is_const_int(ctx, key, 0)? {
        return lower_mixed_callable_receiver_or_array_get(
            ctx,
            inst,
            receiver,
            key,
            warn_on_missing,
        );
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            hashes::materialize_hash_key_aarch64(ctx, key)?;
            ctx.load_value_to_reg(warn_on_missing, "x3")?;
            ctx.load_value_to_reg(receiver, "x0")?;
        }
        Arch::X86_64 => {
            hashes::materialize_hash_key_x86_64(ctx, key)?;
            ctx.load_value_to_reg(warn_on_missing, "rcx")?;
            ctx.load_value_to_reg(receiver, "rdi")?;
        }
    }
    abi::emit_call_label(
        ctx.emitter,
        if for_write {
            "__rt_mixed_array_get_for_write"
        } else {
            "__rt_mixed_array_get"
        },
    );
    cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
    store_if_result(ctx, inst)
}

/// Reads index zero from a boxed callable-array descriptor, otherwise delegates to the ordinary
/// boxed array/hash reader. This preserves object identity for APIs returning callable arrays.
fn lower_mixed_callable_receiver_or_array_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    receiver: ValueId,
    key: ValueId,
    warn_on_missing: ValueId,
) -> Result<()> {
    let callable_label = ctx.next_label("mixed_callable_array_receiver");
    let native_label = ctx.next_label("mixed_callable_array_native");
    let done_label = ctx.next_label("mixed_callable_array_done");
    let descriptor_reg = abi::nested_call_reg(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(receiver, "x0")?;
            ctx.emitter.instruction(&format!("cbz x0, {}", native_label));
            ctx.emitter.instruction("ldr x9, [x0]");                            // inspect the boxed runtime tag
            ctx.emitter.instruction("cmp x9, #10");                            // callable descriptor tag
            ctx.emitter.instruction(&format!("b.ne {}", native_label));
            ctx.emitter.instruction("ldr x9, [x0, #8]");                       // load the callable descriptor payload
            ctx.emitter.instruction("ldr x10, [x9]");                          // inspect the descriptor source-shape kind
            ctx.emitter.instruction(&format!(
                "cmp x10, #{}",
                callable_descriptor::CALLABLE_DESC_KIND_ARRAY
            ));
            ctx.emitter.instruction(&format!("b.eq {}", callable_label));
            ctx.emitter.instruction(&format!("b {}", native_label));
            ctx.emitter.label(&callable_label);
            ctx.emitter
                .instruction(&format!("mov {}, x9", descriptor_reg));
            abi::emit_load_from_address(
                ctx.emitter,
                "x0",
                descriptor_reg,
                callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET,
            );
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(receiver, "rax")?;
            ctx.emitter.instruction("test rax, rax");                          // null Mixed cells use the ordinary reader
            ctx.emitter.instruction(&format!("je {}", native_label));
            ctx.emitter.instruction("cmp QWORD PTR [rax], 10");                // callable descriptor tag
            ctx.emitter.instruction(&format!("jne {}", native_label));
            ctx.emitter.instruction("mov r10, QWORD PTR [rax + 8]");            // load the callable descriptor payload
            ctx.emitter.instruction(&format!(
                "cmp QWORD PTR [r10], {}",
                callable_descriptor::CALLABLE_DESC_KIND_ARRAY
            ));
            ctx.emitter.instruction(&format!("je {}", callable_label));
            ctx.emitter.instruction(&format!("jmp {}", native_label));
            ctx.emitter.label(&callable_label);
            ctx.emitter
                .instruction(&format!("mov {}, r10", descriptor_reg));
            abi::emit_load_from_address(
                ctx.emitter,
                "rax",
                descriptor_reg,
                callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET,
            );
        }
    }
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Object(String::new()));
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&native_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            hashes::materialize_hash_key_aarch64(ctx, key)?;
            ctx.load_value_to_reg(warn_on_missing, "x3")?;
            ctx.load_value_to_reg(receiver, "x0")?;
        }
        Arch::X86_64 => {
            hashes::materialize_hash_key_x86_64(ctx, key)?;
            ctx.load_value_to_reg(warn_on_missing, "rcx")?;
            ctx.load_value_to_reg(receiver, "rdi")?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_array_get");
    ctx.emitter.label(&done_label);
    cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
    store_if_result(ctx, inst)
}

/// Returns whether one EIR operand is the requested integer literal.
fn value_is_const_int(ctx: &FunctionContext<'_>, value: ValueId, expected: i64) -> Result<bool> {
    let value = ctx
        .function
        .value(value)
        .ok_or_else(|| CodegenIrError::missing_entry("value", value.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value.def else {
        return Ok(false);
    };
    let inst = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    Ok(inst.op == Op::ConstI64 && inst.immediate == Some(Immediate::I64(expected)))
}

/// Lowers typed fetch-for-write parent reads of nested array writes (issue #555).
///
/// Two receiver shapes share the `ArrayFetchForWrite` runtime target:
/// - boxed `Mixed` receiver → `__rt_mixed_array_get_for_write(cell, key)`
///   autovivifies missing/null elements inside the receiver cell and returns
///   an owned boxed cell (the STORED one whenever storage is boxed);
/// - concrete `Array`/`AssocArray` receiver → `__rt_array_ensure_elem_for_write
///   (container, tag, key)` autovivifies the element and returns the possibly
///   promoted/reallocated container pointer for the local storeback.
pub(super) fn lower_array_fetch_for_write_runtime_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let receiver = expect_operand(inst, 0)?;
    let key = expect_operand(inst, 1)?;
    let receiver_ty = ctx.value_php_type(receiver)?.codegen_repr();
    match receiver_ty {
        PhpType::Mixed | PhpType::Union(_) => {
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    hashes::materialize_hash_key_aarch64(ctx, key)?;
                    ctx.load_value_to_reg(receiver, "x0")?;
                }
                Arch::X86_64 => {
                    hashes::materialize_hash_key_x86_64(ctx, key)?;
                    ctx.load_value_to_reg(receiver, "rdi")?;
                }
            }
            abi::emit_call_label(ctx.emitter, "__rt_mixed_array_get_for_write");
            cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
            store_if_result(ctx, inst)
        }
        PhpType::Array(_) | PhpType::AssocArray { .. } => {
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    hashes::materialize_hash_key_aarch64(ctx, key)?;
                    abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
                    ctx.load_value_to_reg(receiver, "x0")?;
                    abi::emit_push_reg(ctx.emitter, "x0");
                    abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
                    ctx.emitter.instruction("cmp x0, #3");                      // detect hash storage even when the static PHP type remains generic array
                    abi::emit_load_int_immediate(ctx.emitter, "x1", 4);
                    abi::emit_load_int_immediate(ctx.emitter, "x9", 5);
                    ctx.emitter.instruction("csel x1, x9, x1, eq");             // pass runtime payload tag 5 for hashes and 4 for indexed arrays
                    abi::emit_pop_reg(ctx.emitter, "x0");
                    abi::emit_pop_reg_pair(ctx.emitter, "x2", "x3");
                }
                Arch::X86_64 => {
                    hashes::materialize_hash_key_x86_64(ctx, key)?;
                    abi::emit_push_reg_pair(ctx.emitter, "rsi", "rdx");
                    ctx.load_value_to_reg(receiver, "rdi")?;
                    abi::emit_push_reg(ctx.emitter, "rdi");
                    ctx.emitter.instruction("mov rax, rdi");                    // classify the receiver through the runtime helper's result-register ABI
                    abi::emit_call_label(ctx.emitter, "__rt_heap_kind");
                    ctx.emitter.instruction("cmp rax, 3");                      // detect hash storage even when the static PHP type remains generic array
                    abi::emit_load_int_immediate(ctx.emitter, "rsi", 4);
                    abi::emit_load_int_immediate(ctx.emitter, "r8", 5);
                    ctx.emitter.instruction("cmove rsi, r8");                   // pass runtime payload tag 5 for hashes and 4 for indexed arrays
                    abi::emit_pop_reg(ctx.emitter, "rdi");
                    abi::emit_pop_reg_pair(ctx.emitter, "rdx", "rcx");
                }
            }
            abi::emit_call_label(ctx.emitter, "__rt_array_ensure_elem_for_write");
            store_if_result(ctx, inst)
        }
        other => Err(CodegenIrError::unsupported(format!(
            "fetch-for-write runtime_call with receiver PHP type {:?}",
            other
        ))),
    }
}

/// Lowers `$mixed[$key] = $value` through the shared boxed Mixed array/hash/stdClass writer.
pub(super) fn lower_mixed_array_runtime_set(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let receiver = expect_operand(inst, 0)?;
    let key = expect_operand(inst, 1)?;
    let value = expect_operand(inst, 2)?;
    match ctx.value_php_type(receiver)?.codegen_repr() {
        PhpType::Mixed | PhpType::Union(_) => {}
        other => {
            return Err(CodegenIrError::unsupported(format!(
                "runtime_call array set with receiver PHP type {:?}",
                other
            )))
        }
    }
    let object_label = ctx.next_label("mixed_array_set_array_access");
    let native_label = ctx.next_label("mixed_array_set_native");
    let done_label = ctx.next_label("mixed_array_set_done");
    emit_mixed_array_set_object_guard(ctx, receiver, &object_label, &native_label)?;

    ctx.emitter.label(&native_label);
    match ctx.emitter.target.arch {
        Arch::AArch64 => lower_mixed_array_runtime_set_aarch64(ctx, receiver, key, value)?,
        Arch::X86_64 => lower_mixed_array_runtime_set_x86_64(ctx, receiver, key, value)?,
    }
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&object_label);
    array_access_runtime::lower_boxed_array_access_interface_call(ctx, inst, "offsetSet")?;
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Branches boxed object payloads to module-aware ArrayAccess dispatch before the shared helper.
fn emit_mixed_array_set_object_guard(
    ctx: &mut FunctionContext<'_>,
    receiver: ValueId,
    object_label: &str,
    native_label: &str,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(receiver, "x0")?;
            ctx.emitter.instruction(&format!("cbz x0, {}", native_label));      // absent Mixed cells stay on the null-safe runtime path
            ctx.emitter.instruction("ldr x9, [x0]");                            // inspect the boxed runtime payload tag
            ctx.emitter.instruction("cmp x9, #6");                              // object payloads require module-aware interface dispatch
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // call the concrete ArrayAccess::offsetSet implementation
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(receiver, "rax")?;
            ctx.emitter.instruction("test rax, rax");                           // absent Mixed cells stay on the null-safe runtime path
            ctx.emitter.instruction(&format!("je {}", native_label));
            ctx.emitter.instruction("cmp QWORD PTR [rax], 6");                  // object payloads require module-aware interface dispatch
            ctx.emitter.instruction(&format!("je {}", object_label));           // call the concrete ArrayAccess::offsetSet implementation
        }
    }
    abi::emit_jump(ctx.emitter, native_label);
    Ok(())
}

/// Materializes AArch64 operands for the boxed Mixed array/hash writer.
pub(super) fn lower_mixed_array_runtime_set_aarch64(
    ctx: &mut FunctionContext<'_>,
    receiver: ValueId,
    key: ValueId,
    value: ValueId,
) -> Result<()> {
    let value_ty = ctx.load_value_to_result(value)?.codegen_repr();
    if matches!(value_ty, PhpType::Mixed | PhpType::Union(_)) {
        if !ctx.value_can_transfer_ownership_to_consumer(value)? {
            abi::emit_incref_if_refcounted(ctx.emitter, &value_ty);
        }
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, &value_ty);
    }
    abi::emit_push_reg(ctx.emitter, "x0");
    hashes::materialize_hash_key_aarch64(ctx, key)?;
    abi::emit_push_reg_pair(ctx.emitter, "x1", "x2");
    ctx.load_value_to_reg(receiver, "x0")?;
    abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2");
    abi::emit_pop_reg(ctx.emitter, "x3");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_array_set");
    Ok(())
}

/// Materializes x86_64 operands for the boxed Mixed array/hash writer.
pub(super) fn lower_mixed_array_runtime_set_x86_64(
    ctx: &mut FunctionContext<'_>,
    receiver: ValueId,
    key: ValueId,
    value: ValueId,
) -> Result<()> {
    let value_ty = ctx.load_value_to_result(value)?.codegen_repr();
    if matches!(value_ty, PhpType::Mixed | PhpType::Union(_)) {
        if !ctx.value_can_transfer_ownership_to_consumer(value)? {
            abi::emit_incref_if_refcounted(ctx.emitter, &value_ty);
        }
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, &value_ty);
    }
    abi::emit_push_reg(ctx.emitter, "rax");
    hashes::materialize_hash_key_x86_64(ctx, key)?;
    abi::emit_push_reg_pair(ctx.emitter, "rsi", "rdx");
    ctx.load_value_to_reg(receiver, "rdi")?;
    abi::emit_pop_reg_pair(ctx.emitter, "rsi", "rdx");
    abi::emit_pop_reg(ctx.emitter, "rcx");
    abi::emit_call_label(ctx.emitter, "__rt_mixed_array_set");
    Ok(())
}
