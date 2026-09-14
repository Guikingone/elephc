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

/// Lowers a named property write through the runtime Mixed object-property setter.
pub(super) fn lower_mixed_prop_set(
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
    abi::emit_call_label(ctx.emitter, "__rt_mixed_property_set");
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

/// Lowers a RUNTIME-name write to an undeclared property on an allow-dynamic class.
///
/// The static-name sibling interns the key in the data pool. A name only known at run time has
/// no such label, so the pointer/length pair is taken from the caller's temporary stack frame,
/// the same frame `lower_runtime_object_prop_set` already staged the receiver and the name in.
///
/// Without this the declared-slot ladder's MISS arm fell off the end and the write vanished:
/// `$dyn->{$name} = $v` on an `#[\AllowDynamicProperties]` class stored nothing at all, which is
/// also what `clone($dyn, [$name => $v])` would have done for every undeclared key.
///
/// `frame_bytes` is the caller's reserved block: the receiver sits at `receiver_offset`, the
/// name pointer at `name_offset` and its length at `name_offset + 8`. The block is released here.
pub(super) fn lower_runtime_allow_dynamic_prop_set(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    hash_offset: usize,
    receiver_offset: usize,
    name_offset: usize,
    frame_bytes: usize,
) -> Result<()> {
    let value_ty = ctx.value_php_type(value)?.codegen_repr();
    let boxed_reg = abi::secondary_scratch_reg(ctx.emitter).to_string();
    let object_reg = abi::symbol_scratch_reg(ctx.emitter).to_string();
    let target = ctx.emitter.target;
    materialize_dynamic_property_mixed_value(ctx, value, &value_ty)?;
    abi::emit_reg_move(ctx.emitter, &boxed_reg, abi::int_result_reg(ctx.emitter));
    abi::emit_load_temporary_stack_slot(ctx.emitter, &object_reg, receiver_offset);
    abi::emit_load_from_address(
        ctx.emitter,
        abi::int_arg_reg_name(target, 0),
        &object_reg,
        hash_offset,
    );
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_arg_reg_name(target, 1), name_offset);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_arg_reg_name(target, 2),
        name_offset + 8,
    );
    abi::emit_reg_move(ctx.emitter, abi::int_arg_reg_name(target, 3), &boxed_reg);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_arg_reg_name(target, 4), 0);
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(target, 5),
        runtime_value_tag(&PhpType::Mixed) as i64,
    );
    abi::emit_call_label(ctx.emitter, "__rt_hash_set");
    // The helper can reallocate, so the receiver is reloaded and the fresh table stored back.
    abi::emit_load_temporary_stack_slot(ctx.emitter, &object_reg, receiver_offset);
    abi::emit_store_to_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        &object_reg,
        hash_offset,
    );
    abi::emit_release_temporary_stack(ctx.emitter, frame_bytes);
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

/// Emits php 8.5's dynamic-property deprecation before a runtime-name hash write CREATES a key.
///
/// The class reached this path through the property hash
/// `crate::types::checker::clone_override_storage` reserved for `clone($object, [...])`, which is
/// storage only: php still reports `Creation of dynamic property C::$n is deprecated` for a class
/// that carries neither `#[\AllowDynamicProperties]` nor stdClass's engine exemption. The level is
/// not passed explicitly; `__rt_diag_warning` derives `E_DEPRECATED` from the `Deprecated: `
/// prefix, strips it for a user error handler, and gates the default line on `error_reporting`.
///
/// Only a CREATION is reported, so the key is probed first: re-cloning an object that already
/// carries the name overwrites it, and php 8.5.10 stays silent for that second write.
///
/// Nothing owned exists yet at this point. The override value is boxed by
/// `lower_runtime_allow_dynamic_prop_set` afterwards, so an error handler that THROWS out of the
/// deprecation unwinds with no boxed cell and no half-written hash entry to leak, and the clone
/// itself is released by the applicator's ordinary throwing path.
pub(super) fn emit_dynamic_property_creation_deprecation(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    hash_offset: usize,
    receiver_offset: usize,
    name_offset: usize,
) -> Result<()> {
    let deprecated = ctx
        .module
        .class_infos
        .get(class_name)
        .is_some_and(|info| info.dynamic_property_creation_is_deprecated());
    if !deprecated {
        return Ok(());
    }
    let target = ctx.emitter.target;
    let skip_label = ctx.next_label("dyn_prop_create_deprecation_skip");
    let object_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, receiver_offset);
    abi::emit_load_from_address(
        ctx.emitter,
        abi::int_arg_reg_name(target, 0),
        object_reg,
        hash_offset,
    );
    abi::emit_load_temporary_stack_slot(ctx.emitter, abi::int_arg_reg_name(target, 1), name_offset);
    abi::emit_load_temporary_stack_slot(
        ctx.emitter,
        abi::int_arg_reg_name(target, 2),
        name_offset + 8,
    );
    abi::emit_call_label(ctx.emitter, "__rt_hash_get");
    abi::emit_branch_if_int_result_nonzero(ctx.emitter, &skip_label);
    emit_property_warning_fragment(
        ctx,
        format!("Deprecated: Creation of dynamic property {}::$", class_name).as_bytes(),
        false,
    );
    let (name_ptr_reg, name_len_reg) = match target.arch {
        Arch::AArch64 => ("x1", "x2"),
        Arch::X86_64 => ("rdi", "rsi"),
    };
    abi::emit_load_temporary_stack_slot(ctx.emitter, name_ptr_reg, name_offset);
    abi::emit_load_temporary_stack_slot(ctx.emitter, name_len_reg, name_offset + 8);
    abi::emit_call_label(ctx.emitter, "__rt_diag_warning_fragment");
    emit_property_warning_fragment(ctx, b" is deprecated\n", true);
    ctx.emitter.label(&skip_label);
    Ok(())
}
