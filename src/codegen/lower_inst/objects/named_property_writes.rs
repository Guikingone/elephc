//! Purpose:
//! Lowers named stdClass, Mixed, dynamic-class, and nullable property writes.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Prior values are released and null writes fail at the same observable point.

use super::*;

/// Lowers a named property write to a statically known stdClass receiver.
pub(super) fn lower_stdclass_prop_set(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    value: ValueId,
    property: &str,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?.codegen_repr();
    materialize_dynamic_property_mixed_value(ctx, value, &value_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(object, "x0")?;
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
            abi::emit_pop_reg(ctx.emitter, "x3");
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(object, "rdi")?;
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
            abi::emit_pop_reg(ctx.emitter, "rcx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stdclass_set");
    Ok(())
}

/// Lowers a named property write through declared-class dispatch for a boxed Mixed receiver.
pub(super) fn lower_mixed_prop_set(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    value: ValueId,
    property: &str,
    inst: &Instruction,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?;
    let candidates = declared_mixed_property_candidates(ctx, property, inst)?
        .into_iter()
        .filter(|candidate| {
            ensure_property_value_supported(ctx, &candidate.slot, value, &value_ty, inst).is_ok()
        })
        .collect::<Vec<_>>();
    if !candidates.is_empty() {
        return lower_declared_mixed_prop_set(ctx, object, value, property, inst, candidates);
    }
    let value_ty = value_ty.codegen_repr();
    materialize_dynamic_property_mixed_value(ctx, value, &value_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(object, "x0")?;
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
            abi::emit_pop_reg(ctx.emitter, "x3");
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(object, "rdi")?;
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
            abi::emit_pop_reg(ctx.emitter, "rcx");
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_property_set");
    Ok(())
}

/// Lowers a named property write through PHP's bare `object` pseudo-type.
///
/// The receiver is an unboxed object payload whose concrete class remains runtime-only. Dispatch
/// therefore mirrors generic property reads, while candidate collection excludes declared slots
/// that cannot accept the replacement value.
pub(super) fn lower_generic_object_prop_set(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    value: ValueId,
    property: &str,
    inst: &Instruction,
) -> Result<()> {
    ctx.load_value_to_reg(object, abi::int_result_reg(ctx.emitter))?;
    lower_generic_object_prop_set_from_loaded_object(ctx, value, property, inst)
}

/// Dispatches a named property write after the raw object payload is in the result register.
fn lower_generic_object_prop_set_from_loaded_object(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    property: &str,
    inst: &Instruction,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?;
    let candidates = declared_mixed_property_candidates(ctx, property, inst)?
        .into_iter()
        .filter(|candidate| {
            ensure_property_value_supported(ctx, &candidate.slot, value, &value_ty, inst).is_ok()
        })
        .collect::<Vec<_>>();
    let miss_label = ctx.next_label("generic_object_prop_set_miss");
    let done_label = ctx.next_label("generic_object_prop_set_done");
    let stdclass_label = ctx.next_label("generic_object_prop_set_stdclass");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "generic_object_prop_set_{}",
                label_fragment(&candidate.slot.class_name)
            ))
        })
        .collect::<Vec<_>>();

    emit_mixed_property_class_dispatch(
        ctx,
        &candidates,
        &match_labels,
        &stdclass_label,
        &miss_label,
    );

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let base_reg = abi::symbol_scratch_reg(ctx.emitter);
        abi::emit_reg_move(ctx.emitter, base_reg, abi::int_result_reg(ctx.emitter));
        emit_property_store(ctx, value, &candidate.slot, base_reg)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&stdclass_label);
    emit_stdclass_set_from_loaded_object(ctx, value, property)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&miss_label);
    let message = format!(
        "Fatal error: Cannot write undeclared property ${} through an object receiver\n",
        property
    );
    emit_fatal_message(ctx, message.as_bytes());

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Dispatches a named Mixed-receiver property write across every declared owner of that slot.
fn lower_declared_mixed_prop_set(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    value: ValueId,
    property: &str,
    inst: &Instruction,
    candidates: Vec<MixedPropertyCandidate>,
) -> Result<()> {
    let null_label = ctx.next_label("mixed_prop_set_null");
    let miss_label = ctx.next_label("mixed_prop_set_miss");
    let done_label = ctx.next_label("mixed_prop_set_done");
    let stdclass_label = ctx.next_label("mixed_prop_set_stdclass");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "mixed_prop_set_{}",
                label_fragment(&candidate.slot.class_name)
            ))
        })
        .collect::<Vec<_>>();

    ctx.load_value_to_reg(object, abi::int_result_reg(ctx.emitter))?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_mixed_object_payload_or_null(ctx, &null_label);
    emit_mixed_property_class_dispatch(
        ctx,
        &candidates,
        &match_labels,
        &stdclass_label,
        &miss_label,
    );

    let value_ty = ctx.value_php_type(value)?;
    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        ensure_property_value_supported(ctx, &candidate.slot, value, &value_ty, inst)?;
        let base_reg = abi::symbol_scratch_reg(ctx.emitter);
        let result_reg = abi::int_result_reg(ctx.emitter);
        abi::emit_reg_move(ctx.emitter, base_reg, result_reg);
        emit_property_store(ctx, value, &candidate.slot, base_reg)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&stdclass_label);
    emit_stdclass_set_from_loaded_object(ctx, value, property)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&miss_label);
    let message = format!(
        "Fatal error: Cannot write undeclared property ${} through a mixed receiver\n",
        property
    );
    emit_fatal_message(ctx, message.as_bytes());

    ctx.emitter.label(&null_label);
    emit_property_assign_on_null_fatal(ctx, property);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Stores a boxed value into a stdClass whose raw object pointer is in the result register.
fn emit_stdclass_set_from_loaded_object(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    property: &str,
) -> Result<()> {
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let value_ty = ctx.value_php_type(value)?.codegen_repr();
    materialize_dynamic_property_mixed_value(ctx, value, &value_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_pop_reg(ctx.emitter, "x3");
            abi::emit_pop_reg(ctx.emitter, "x0");
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
        }
        Arch::X86_64 => {
            abi::emit_pop_reg(ctx.emitter, "rcx");
            abi::emit_pop_reg(ctx.emitter, "rdi");
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stdclass_set");
    Ok(())
}

/// Lowers a static-name write to an undeclared property on an allow-dynamic class.
pub(super) fn lower_allow_dynamic_prop_set(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    value: ValueId,
    property: &str,
    hash_offset: usize,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?.codegen_repr();
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    let boxed_reg = abi::secondary_scratch_reg(ctx.emitter);
    let (label, key_len) = ctx.data.add_string(property.as_bytes());
    ctx.load_value_to_reg(object, object_reg)?;
    abi::emit_push_reg(ctx.emitter, object_reg);
    materialize_dynamic_property_mixed_value(ctx, value, &value_ty)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov {}, x0", boxed_reg));         // preserve the boxed dynamic-property value across receiver restore
            abi::emit_pop_reg(ctx.emitter, object_reg);
            ctx.emitter
                .instruction(&format!("ldr x0, [{}, #{}]", object_reg, hash_offset)); // load the dynamic-property hash pointer from the receiver
            abi::emit_push_reg(ctx.emitter, object_reg);
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", key_len as i64);
            ctx.emitter.instruction(&format!("mov x3, {}", boxed_reg));         // pass the boxed Mixed cell as the hash value payload
            ctx.emitter.instruction("mov x4, xzr");                             // boxed Mixed hash entries do not use the high payload word
            abi::emit_load_int_immediate(
                ctx.emitter,
                "x5",
                runtime_value_tag(&PhpType::Mixed) as i64,
            );
            abi::emit_call_label(ctx.emitter, "__rt_hash_set");
            abi::emit_pop_reg(ctx.emitter, object_reg);
            abi::emit_store_to_address(ctx.emitter, "x0", object_reg, hash_offset);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, rax", boxed_reg));        // preserve the boxed dynamic-property value across receiver restore
            abi::emit_pop_reg(ctx.emitter, object_reg);
            ctx.emitter.instruction(&format!(
                "mov rdi, QWORD PTR [{} + {}]",
                object_reg, hash_offset
            ));                                                                 // load the dynamic-property hash pointer from the receiver
            abi::emit_push_reg(ctx.emitter, object_reg);
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", key_len as i64);
            ctx.emitter.instruction(&format!("mov rcx, {}", boxed_reg));        // pass the boxed Mixed cell as the hash value payload
            ctx.emitter.instruction("xor r8, r8");                              // boxed Mixed hash entries do not use the high payload word
            abi::emit_load_int_immediate(
                ctx.emitter,
                "r9",
                runtime_value_tag(&PhpType::Mixed) as i64,
            );
            abi::emit_call_label(ctx.emitter, "__rt_hash_set");
            abi::emit_pop_reg(ctx.emitter, object_reg);
            abi::emit_store_to_address(ctx.emitter, "rax", object_reg, hash_offset);
        }
    }
    Ok(())
}

/// Materializes a property value as an owned boxed `Mixed` cell in the result register.
pub(super) fn materialize_dynamic_property_mixed_value(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    value_ty: &PhpType,
) -> Result<()> {
    ctx.load_value_to_result(value)?;
    if matches!(value_ty, PhpType::Mixed | PhpType::Union(_)) {
        if !ctx.value_can_own_mixed_box_source(value)? {
            abi::emit_incref_if_refcounted(ctx.emitter, &value_ty.codegen_repr());
        }
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, value_ty);
    }
    Ok(())
}

/// Lowers a property write on a nullable receiver, fataling after RHS evaluation when null.
pub(super) fn lower_nullable_prop_set(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    value: ValueId,
    class_name: &str,
    property: &str,
) -> Result<()> {
    if class_name.trim_start_matches('\\').is_empty() {
        return lower_nullable_generic_object_prop_set(ctx, inst, object, value, property);
    }
    let slot = resolve_property_slot_for_class(ctx, class_name, property, inst)?;
    let value_ty = ctx.value_php_type(value)?;
    ensure_property_value_supported(ctx, &slot, value, &value_ty, inst)?;
    let null_label = ctx.next_label("nullable_prop_set_null");
    let done_label = ctx.next_label("nullable_prop_set_done");
    let base_reg = abi::symbol_scratch_reg(ctx.emitter);
    emit_nullable_receiver_object_payload(ctx, object, &null_label, base_reg)?;
    emit_property_store(ctx, value, &slot, base_reg)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    emit_property_assign_on_null_fatal(ctx, property);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Lowers a nullable bare-object property write through concrete runtime class dispatch.
fn lower_nullable_generic_object_prop_set(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    value: ValueId,
    property: &str,
) -> Result<()> {
    let null_label = ctx.next_label("nullable_generic_prop_set_null");
    let done_label = ctx.next_label("nullable_generic_prop_set_done");
    let object_reg = abi::int_result_reg(ctx.emitter);
    emit_nullable_receiver_object_payload(ctx, object, &null_label, object_reg)?;
    lower_generic_object_prop_set_from_loaded_object(ctx, value, property, inst)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    emit_property_assign_on_null_fatal(ctx, property);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Emits PHP's fatal diagnostic for assigning a property on null.
pub(super) fn emit_property_assign_on_null_fatal(ctx: &mut FunctionContext<'_>, property: &str) {
    let message = format!(
        "Fatal error: Attempt to assign property \"{}\" on null\n",
        property
    );
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, #2");                              // write the property-assign-on-null fatal to stderr
            ctx.emitter.adrp("x1", &message_label);
            ctx.emitter.add_lo12("x1", "x1", &message_label);
            ctx.emitter
                .instruction(&format!("mov x2, #{}", message_len)); // pass the property-assign-on-null fatal byte length
            ctx.emitter.syscall(4);
            abi::emit_exit(ctx.emitter, 1);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov edi, 2");                              // write the property-assign-on-null fatal to Linux stderr
            abi::emit_symbol_address(ctx.emitter, "rsi", &message_label);
            ctx.emitter
                .instruction(&format!("mov edx, {}", message_len)); // pass the property-assign-on-null fatal byte length
            ctx.emitter.instruction("mov eax, 1");                              // Linux x86_64 syscall 1 = write
            ctx.emitter.instruction("syscall");                                 // emit the property-assign-on-null fatal before exiting
            abi::emit_exit(ctx.emitter, 1);
        }
    }
}
