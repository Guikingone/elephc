//! Purpose:
//! Lowers ArrayAccess and typed runtime fallback dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Lowers high-level runtime fallback casts that Phase 04 can identify by type.
pub(super) fn lower_runtime_call(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if let Some(Immediate::RuntimeCall(target)) = inst.immediate {
        return runtime_calls::lower(ctx, inst, target);
    }
    if inst.operands.len() == 3 && matches!(inst.immediate, Some(Immediate::Data(_))) {
        return lower_property_array_runtime_set(ctx, inst);
    }
    if inst.operands.len() == 1
        && matches!(inst.immediate, Some(Immediate::NominalObject { .. }))
    {
        if let Some(()) = lower_generic_object_nominal_guard(ctx, inst)? {
            return Ok(());
        }
    }
    if inst.operands.len() == 1 && matches!(inst.immediate, Some(Immediate::TypeName(_))) {
        return lower_gradual_union_param_guard(ctx, inst);
    }
    if let Some(()) = try_lower_callable_array_runtime_get(ctx, inst)? {
        return Ok(());
    }
    if let Some(()) = try_lower_array_access_runtime_call(ctx, inst)? {
        return Ok(());
    }
    if inst.operands.len() == 3 {
        if inst.result_php_type.codegen_repr() != PhpType::Void {
            return lower_mixed_array_runtime_get(ctx, inst, false);
        }
        return lower_mixed_array_runtime_set(ctx, inst);
    }
    if inst.operands.len() == 2 {
        return lower_binary_runtime_call(ctx, inst);
    }
    if inst.operands.len() != 1 {
        return Err(CodegenIrError::unsupported(format!(
            "runtime_call with {} operands returning PHP type {:?}",
            inst.operands.len(),
            inst.result_php_type
        )));
    }
    let value = expect_operand(inst, 0)?;
    let source_ty = ctx.value_php_type(value)?.codegen_repr();
    if let (PhpType::Object(class_name), PhpType::Str) =
        (&source_ty, inst.result_php_type.codegen_repr())
    {
        let normalized = class_name.trim_start_matches('\\');
        if !object_class_has_tostring(ctx, normalized) {
            emit_value_dynamic_object_to_string(ctx, value)?;
            return store_if_result(ctx, inst);
        }
        emit_object_tostring_call(ctx, value, normalized)?;
        return store_if_result(ctx, inst);
    }
    if source_ty == PhpType::Str && inst.result_php_type.codegen_repr() == PhpType::Callable {
        let result_reg = abi::int_result_reg(ctx.emitter).to_string();
        callables::emit_runtime_string_descriptor_value(
            ctx,
            value,
            &result_reg,
            "callable return",
            crate::strict_php::is_enabled(),
        )?;
        return store_if_result(ctx, inst);
    }
    if matches!(source_ty, PhpType::Array(_))
        && inst.result_php_type.codegen_repr() == PhpType::Callable
    {
        callables::emit_runtime_callable_array_descriptor_value(ctx, value, "callable return")?;
        return store_if_result(ctx, inst);
    }
    if matches!(&source_ty, PhpType::Array(elem) if elem.codegen_repr() == PhpType::Mixed)
        && matches!(
            inst.result_php_type.codegen_repr(),
            PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed
        )
    {
        ctx.load_value_to_result(value)?;
        if ctx.emitter.target.arch == Arch::X86_64 {
            ctx.emitter.instruction("mov rdi, rax");                            // pass the indexed mixed array to the hash conversion helper
        }
        abi::emit_call_label(ctx.emitter, "__rt_array_to_hash");
        return store_if_result(ctx, inst);
    }
    if matches!(source_ty, PhpType::AssocArray { .. })
        && matches!(inst.result_php_type.codegen_repr(), PhpType::Array(_))
    {
        // PHP's `array` return boundary accepts both packed and hash-backed storage. Keep the
        // associative pointer and its keys intact; array consumers dispatch on the runtime
        // storage-kind metadata when a statically indexed value is hash-backed.
        ctx.load_value_to_result(value)?;
        return store_if_result(ctx, inst);
    }
    if inst.result_php_type.codegen_repr() == PhpType::Iterable
        && matches!(
            source_ty,
            PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_) | PhpType::Iterable
        )
    {
        ctx.load_value_to_result(value)?;
        return store_if_result(ctx, inst);
    }
    if inst.result_php_type.codegen_repr() == PhpType::TaggedScalar {
        ctx.load_value_to_result(value)?;
        coerce_loaded_value_to_tagged_scalar(ctx, &source_ty)?;
        return store_if_result(ctx, inst);
    }
    if source_ty == PhpType::TaggedScalar
        && inst.result_php_type.codegen_repr() == PhpType::Int
    {
        ctx.load_value_to_result(value)?;
        let null_label = ctx.next_label("nullable_int_param_null");
        let accepted_label = ctx.next_label("nullable_int_param_accepted");
        crate::codegen::sentinels::emit_branch_if_tagged_scalar_null(
            ctx.emitter,
            &null_label,
        );
        abi::emit_jump(ctx.emitter, &accepted_label);
        ctx.emitter.label(&null_label);
        exceptions::emit_type_error(ctx, "Argument must be of type int, null given");
        ctx.emitter.label(&accepted_label);
        return store_if_result(ctx, inst);
    }
    if matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) {
        let result_ty = inst.result_php_type.codegen_repr();
        load_value_to_first_int_arg(ctx, value)?;
        match result_ty {
            PhpType::Str => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_string"),
            PhpType::Float => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_float"),
            PhpType::Int => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_int"),
            PhpType::Bool => abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_bool"),
            PhpType::Array(elem) if elem.codegen_repr() == PhpType::Mixed => {
                lower_mixed_to_mixed_indexed_array(ctx)?;
            }
            PhpType::AssocArray { value, .. } if value.codegen_repr() == PhpType::Mixed => {
                lower_mixed_to_mixed_assoc_array(ctx)?;
            }
            PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Callable
            | PhpType::Iterable
            | PhpType::Object(_) => {
                emit_unbox_mixed_to_owned_refcounted_result(ctx, &result_ty);
            }
            other => {
                return Err(CodegenIrError::unsupported(format!(
                    "runtime_call from PHP type {:?} to PHP type {:?}",
                    source_ty, other
                )))
            }
        }
        return store_if_result(ctx, inst);
    }
    Err(CodegenIrError::unsupported(format!(
        "runtime_call from PHP type {:?} to PHP type {:?}",
        source_ty, inst.result_php_type
    )))
}

/// Lowers an indexed read from a callable proven array-shaped by control flow. Callable arrays
/// use a compact descriptor at runtime, so index zero reconstructs the object/class receiver and
/// index one reconstructs the method-name string as boxed Mixed values.
fn try_lower_callable_array_runtime_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<Option<()>> {
    if inst.result.is_none() || inst.operands.len() != 3 {
        return Ok(None);
    }
    let receiver = inst.operands[0];
    if ctx.value_php_type(receiver)?.codegen_repr() != PhpType::Callable {
        return Ok(None);
    }
    let index = inst.operands[1];
    if ctx.value_php_type(index)?.codegen_repr() != PhpType::Int {
        return Err(CodegenIrError::unsupported(
            "callable array access with non-integer index",
        ));
    }

    let descriptor_reg = abi::nested_call_reg(ctx.emitter);
    let index_reg = abi::secondary_scratch_reg(ctx.emitter);
    ctx.load_value_to_result(receiver)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    ctx.load_value_to_result(index)?;
    ctx.emitter.instruction(&format!(
        "mov {}, {}",
        index_reg,
        abi::int_result_reg(ctx.emitter)
    ));
    abi::emit_pop_reg(ctx.emitter, descriptor_reg);

    let receiver_label = ctx.next_label("callable_array_receiver");
    let method_label = ctx.next_label("callable_array_method");
    let static_receiver_label = ctx.next_label("callable_array_static_receiver");
    let object_receiver_label = ctx.next_label("callable_array_object_receiver");
    let null_label = ctx.next_label("callable_array_missing_index");
    let done_label = ctx.next_label("callable_array_index_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp {}, #0", index_reg));
            ctx.emitter.instruction(&format!("b.eq {}", receiver_label));
            ctx.emitter.instruction(&format!("cmp {}, #1", index_reg));
            ctx.emitter.instruction(&format!("b.eq {}", method_label));
            ctx.emitter.instruction(&format!("b {}", null_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp {}, 0", index_reg));
            ctx.emitter.instruction(&format!("je {}", receiver_label));
            ctx.emitter.instruction(&format!("cmp {}, 1", index_reg));
            ctx.emitter.instruction(&format!("je {}", method_label));
            ctx.emitter.instruction(&format!("jmp {}", null_label));
        }
    }

    ctx.emitter.label(&receiver_label);
    let invocation_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_load_from_address(
        ctx.emitter,
        invocation_reg,
        descriptor_reg,
        callable_descriptor::CALLABLE_DESC_INVOCATION_OFFSET,
    );
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("ldr {}, [{}]", index_reg, invocation_reg));
            ctx.emitter.instruction(&format!(
                "cmp {}, #{}",
                index_reg,
                callable_descriptor::CallableDescriptorShape::InstanceMethod as u64
            ));
            ctx.emitter
                .instruction(&format!("b.eq {}", object_receiver_label));
            ctx.emitter.instruction(&format!(
                "cmp {}, #{}",
                index_reg,
                callable_descriptor::CallableDescriptorShape::StaticMethod as u64
            ));
            ctx.emitter
                .instruction(&format!("b.eq {}", static_receiver_label));
            ctx.emitter.instruction(&format!("b {}", null_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, QWORD PTR [{}]", index_reg, invocation_reg));
            ctx.emitter.instruction(&format!(
                "cmp {}, {}",
                index_reg,
                callable_descriptor::CallableDescriptorShape::InstanceMethod as u64
            ));
            ctx.emitter
                .instruction(&format!("je {}", object_receiver_label));
            ctx.emitter.instruction(&format!(
                "cmp {}, {}",
                index_reg,
                callable_descriptor::CallableDescriptorShape::StaticMethod as u64
            ));
            ctx.emitter
                .instruction(&format!("je {}", static_receiver_label));
            ctx.emitter.instruction(&format!("jmp {}", null_label));
        }
    }

    ctx.emitter.label(&object_receiver_label);
    abi::emit_load_from_address(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        descriptor_reg,
        callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET,
    );
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Object(String::new()));
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&static_receiver_label);
    emit_callable_invocation_string(ctx, invocation_reg, 8);
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&method_label);
    abi::emit_load_from_address(
        ctx.emitter,
        invocation_reg,
        descriptor_reg,
        callable_descriptor::CALLABLE_DESC_INVOCATION_OFFSET,
    );
    emit_callable_invocation_string(ctx, invocation_reg, 24);
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Str);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_result_reg(ctx.emitter),
        0x7fff_ffff_ffff_fffe,
    );
    emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Void);
    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)?;
    Ok(Some(()))
}

/// Loads one pointer/length string pair from a callable invocation metadata record.
fn emit_callable_invocation_string(
    ctx: &mut FunctionContext<'_>,
    invocation_reg: &str,
    offset: usize,
) {
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    abi::emit_load_from_address(ctx.emitter, ptr_reg, invocation_reg, offset);
    abi::emit_load_from_address(ctx.emitter, len_reg, invocation_reg, offset + 8);
}

/// Validates a boxed argument's active runtime tag against its declared union parameter.
fn lower_gradual_union_param_guard(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let Some(Immediate::TypeName(type_name)) = inst.immediate else {
        return Err(CodegenIrError::invalid_module(
            "gradual union parameter guard is missing its declared type name",
        ));
    };
    let declared_name = ctx
        .module
        .data
        .strings
        .get(type_name.as_raw() as usize)
        .cloned()
        .ok_or_else(|| CodegenIrError::missing_entry("type-name data", type_name.as_raw()))?;
    let accepted_tags = gradual_union_param_tags(&inst.result_php_type).ok_or_else(|| {
        CodegenIrError::invalid_module(format!(
            "unsupported gradual union parameter guard for PHP type {:?}",
            inst.result_php_type
        ))
    })?;
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let accepted_label = ctx.next_label("gradual_union_param_accepted");
    let wrong_tag_label = ctx.next_label("gradual_union_param_wrong_tag");
    for tag in accepted_tags {
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("cmp x0, #{}", tag));          // compare the active Mixed tag with one declared union member
                ctx.emitter.instruction(&format!("b.eq {}", accepted_label));   // accept the original boxed value when this member matches
            }
            Arch::X86_64 => {
                ctx.emitter.instruction(&format!("cmp rax, {}", tag));          // compare the active Mixed tag with one declared union member
                ctx.emitter.instruction(&format!("je {}", accepted_label));     // accept the original boxed value when this member matches
            }
        }
    }
    abi::emit_jump(ctx.emitter, &wrong_tag_label);
    super::builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_tag_label,
        &|given| format!("Argument must be of type {}, {} given", declared_name, given),
    );
    ctx.emitter.label(&accepted_label);
    ctx.load_value_to_result(value)?;                                          // keep the caller's original boxed union representation
    store_if_result(ctx, inst)
}

/// Returns the stable Mixed tags accepted by a scalar/null/array union parameter.
fn gradual_union_param_tags(ty: &PhpType) -> Option<Vec<i64>> {
    let PhpType::Union(members) = ty else {
        return None;
    };
    let mut tags = Vec::new();
    for member in members {
        let member_tags: &[i64] = match member {
            PhpType::Int => &[0],
            PhpType::Str => &[1],
            PhpType::Float => &[2],
            PhpType::Bool => &[3],
            PhpType::Array(_) | PhpType::AssocArray { .. } => &[4, 5],
            PhpType::Void => &[8],
            _ => return None,
        };
        for tag in member_tags {
            if !tags.contains(tag) {
                tags.push(*tag);
            }
        }
    }
    Some(tags)
}

/// Guards an object payload against a named class or interface boundary.
fn lower_generic_object_nominal_guard(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<Option<()>> {
    let value = expect_operand(inst, 0)?;
    let source_ty = ctx.value_php_type(value)?.codegen_repr();
    if !matches!(source_ty, PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)) {
        return Ok(None);
    }
    let Some(Immediate::NominalObject { target, boundary }) = inst.immediate else {
        return Ok(None);
    };
    let target_name = ctx
        .module
        .data
        .class_names
        .get(target.as_raw() as usize)
        .cloned()
        .ok_or_else(|| CodegenIrError::missing_entry("class data", target.as_raw()))?;
    if let PhpType::Object(source_name) = &source_ty {
        if php_symbol_key(source_name) == php_symbol_key(&target_name) {
            ctx.load_value_to_result(value)?;
            store_nominal_object_result(ctx, inst, &source_ty)?;
            return Ok(Some(()));
        }
    }
    let target_metadata = objects::classify_named_target(ctx, &target_name);
    if matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) {
        return lower_gradual_object_nominal_guard(
            ctx,
            inst,
            value,
            &target_name,
            target_metadata,
            boundary,
        )
        .map(Some);
    }
    let Some((target_id, target_kind)) = target_metadata else {
        // An unresolved nominal declaration can name an optional runtime type that is not part
        // of the compiled program. The exact-name case was accepted above; any other statically
        // known object cannot be proven compatible without inventing hierarchy metadata.
        let message = nominal_object_type_error_message(
            boundary,
            &target_name,
            false,
            "object",
        );
        exceptions::emit_type_error(ctx, &message);
        return Ok(Some(()));
    };
    let source_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    ctx.load_value_to_reg(value, source_reg)?;
    abi::emit_push_reg(ctx.emitter, source_reg);
    objects::emit_match_call(ctx, target_id, target_kind, "__rt_exception_matches");
    let accepted = ctx.next_label("generic_object_boundary_accepted");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz x0, {}", accepted));         // accept a runtime object matching the nominal return contract
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // test the nominal object matcher result
            ctx.emitter.instruction(&format!("jne {}", accepted));              // accept a runtime object matching the nominal return contract
        }
    }
    abi::emit_pop_reg(ctx.emitter, source_reg);
    let message = match boundary {
        NominalObjectBoundary::Parameter => {
            format!("Argument must be of type {}, object given", target_name)
        }
        NominalObjectBoundary::Property => {
            format!("Cannot assign object to property of type {}", target_name)
        }
        NominalObjectBoundary::Return => {
            format!("Return value must be of type {}, object returned", target_name)
        }
    };
    exceptions::emit_type_error(ctx, &message);
    ctx.emitter.label(&accepted);
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    store_nominal_object_result(ctx, inst, &source_ty)?;
    Ok(Some(()))
}

/// Boxes a raw nominal object when the boundary result uses the Mixed representation, then stores
/// the value in the instruction result slot.
fn store_nominal_object_result(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    source_ty: &PhpType,
) -> Result<()> {
    if matches!(inst.result_php_type.codegen_repr(), PhpType::Mixed)
        && matches!(source_ty.codegen_repr(), PhpType::Object(_))
    {
        emit_box_current_value_as_mixed(ctx.emitter, source_ty);
    } else if matches!(source_ty.codegen_repr(), PhpType::Object(_))
        && matches!(inst.result_php_type.codegen_repr(), PhpType::Object(_))
    {
        // The nominal guard forwards a borrowed local-object payload, while its EIR result is
        // declared owned and may be consumed by a property store, argument boundary, or return.
        // Materialize that owner before the caller transfers or releases the result.
        let result_reg = abi::int_result_reg(ctx.emitter);
        abi::emit_push_reg(ctx.emitter, result_reg);
        abi::emit_call_label(ctx.emitter, "__rt_incref");
        abi::emit_pop_reg(ctx.emitter, result_reg);
    }
    store_if_result(ctx, inst)
}

/// Validates a boxed gradual return against a nullable or non-null named object declaration.
fn lower_gradual_object_nominal_guard(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    value: ValueId,
    target_name: &str,
    target_metadata: Option<(u64, i64)>,
    boundary: NominalObjectBoundary,
) -> Result<()> {
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let object_label = ctx.next_label("gradual_object_boundary_object");
    let wrong_tag_label = ctx.next_label("gradual_object_boundary_wrong_tag");
    let accepted_label = ctx.next_label("gradual_object_boundary_accepted");
    let accepts_null = match &inst.result_php_type {
        PhpType::Union(members) => members
            .iter()
            .any(|member| matches!(member, PhpType::Void | PhpType::Never)),
        PhpType::Void | PhpType::Never => true,
        _ => false,
    };
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 = object
            ctx.emitter.instruction(&format!("b.eq {}", object_label));
            if accepts_null {
                ctx.emitter.instruction("cmp x0, #8");                          // runtime tag 8 = null
                ctx.emitter.instruction(&format!("b.eq {}", accepted_label));
            }
            ctx.emitter.instruction(&format!("b {}", wrong_tag_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 = object
            ctx.emitter.instruction(&format!("je {}", object_label));
            if accepts_null {
                ctx.emitter.instruction("cmp rax, 8");                          // runtime tag 8 = null
                ctx.emitter.instruction(&format!("je {}", accepted_label));
            }
            ctx.emitter.instruction(&format!("jmp {}", wrong_tag_label));
        }
    }
    super::builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_tag_label,
        &|given| nominal_object_type_error_message(boundary, target_name, accepts_null, given),
    );

    ctx.emitter.label(&object_label);
    if ctx.module.required_runtime_features.eval_bridge {
        let native_fallback_label = ctx.next_label("gradual_object_boundary_native_fallback");
        super::builtins::emit_eval_object_is_a_named_fallback(
            ctx,
            value,
            target_name,
            &accepted_label,
            &native_fallback_label,
        )?;
        ctx.emitter.label(&native_fallback_label);
    }
    if let Some((target_id, target_kind)) = target_metadata {
        match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),             // pass the unboxed object payload to the nominal matcher
            Arch::X86_64 => {}                                                   // the unboxed object payload is already in rdi
        }
        objects::emit_match_call(ctx, target_id, target_kind, "__rt_exception_matches");
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("cbnz x0, {}", accepted_label)); // accept an object matching the declared class or interface
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("test rax, rax");                       // test the nominal object matcher result
                ctx.emitter.instruction(&format!("jne {}", accepted_label));    // accept an object matching the declared class or interface
            }
        }
    }
    let message = nominal_object_type_error_message(
        boundary,
        target_name,
        accepts_null,
        "object",
    );
    exceptions::emit_type_error(ctx, &message);

    ctx.emitter.label(&accepted_label);
    ctx.load_value_to_result(value)?;                                          // reload the original boxed object-or-null source after matcher calls
    if matches!(inst.result_php_type.codegen_repr(), PhpType::Object(_)) {
        abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
        let payload_reg = mixed_unbox_low_payload_reg(ctx);
        let object_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
        if payload_reg != object_arg {
            abi::emit_reg_move(ctx.emitter, object_arg, payload_reg);
        }
        abi::emit_push_reg(ctx.emitter, payload_reg);
        abi::emit_call_label(ctx.emitter, "__rt_incref");
        abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    }
    store_if_result(ctx, inst)
}

/// Formats the PHP-facing type error for a nominal object parameter, property, or return guard.
fn nominal_object_type_error_message(
    boundary: NominalObjectBoundary,
    target_name: &str,
    accepts_null: bool,
    given: &str,
) -> String {
    let declared_name = if accepts_null {
        format!("{}|null", target_name)
    } else {
        target_name.to_string()
    };
    match boundary {
        NominalObjectBoundary::Parameter => {
            format!("Argument must be of type {}, {} given", declared_name, given)
        }
        NominalObjectBoundary::Property => {
            format!("Cannot assign {} to property of type {}", given, declared_name)
        }
        NominalObjectBoundary::Return => {
            format!("Return value must be of type {}, {} returned", declared_name, given)
        }
    }
}

/// Lowers generic EIR runtime calls that represent PHP `ArrayAccess` object indexing.
///
/// Subscript reads carry a trailing warn-on-missing flag that only the boxed-`Mixed`
/// runtime reader consumes, so operand count alone no longer separates a read from
/// `offsetSet`. Reads are identified structurally instead: subscript writes lower
/// through `emit_void` and carry no result value, while reads always produce one.
/// The flag operand is stripped before dispatch so `offsetGet` keeps its
/// single-argument PHP signature.
pub(super) fn try_lower_array_access_runtime_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<Option<()>> {
    let Some(receiver) = inst.operands.first().copied() else {
        return Ok(None);
    };
    let receiver_ty = ctx.raw_value_php_type(receiver)?;
    let Some(dispatch) = array_access_runtime_dispatch(ctx, &receiver_ty) else {
        return Ok(None);
    };
    // A result value marks a subscript read: `$obj[$k] = v` and `$obj[] = v` lower
    // through `emit_void`. Keying off the result PHP type instead would misread a
    // read whose declared `offsetGet` return type has no runtime representation.
    let is_read = inst.result.is_some();
    let method_name = match inst.operands.len() {
        2 if is_read => "offsetGet",
        2 => "append",
        3 if is_read => "offsetGet",
        3 => "offsetSet",
        _ => return Ok(None),
    };
    // Drop the read's warn-on-missing operand before argument materialization:
    // `offsetGet($offset)` takes one argument, and the shared method-call
    // lowerers resolve arity straight from `inst.operands`.
    let read_without_warning_flag;
    let inst = if is_read && inst.operands.len() == 3 {
        read_without_warning_flag = Instruction {
            operands: inst.operands[..2].to_vec(),
            ..inst.clone()
        };
        &read_without_warning_flag
    } else {
        inst
    };
    match dispatch {
        ArrayAccessRuntimeDispatch::Concrete(class_name) => {
            let concrete_method =
                if method_name == "append" && is_spl_doubly_linked_list_family(&class_name) {
                    "push"
                } else {
                    method_name
                };
            if let Some(intrinsic) = runtime_backed_instance_intrinsic(&class_name, concrete_method)
            {
                lower_instance_runtime_intrinsic(
                    ctx,
                    inst,
                    &class_name,
                    concrete_method,
                    intrinsic,
                )?;
            } else {
                lower_runtime_object_method_call(ctx, inst, &class_name, concrete_method)?;
            }
        }
        ArrayAccessRuntimeDispatch::Interface {
            boxed_receiver: false,
        } => {
            lower_interface_method_call(ctx, inst, "ArrayAccess", method_name)?;
        }
        ArrayAccessRuntimeDispatch::Interface {
            boxed_receiver: true,
        } => {
            lower_boxed_array_access_interface_call(ctx, inst, method_name)?;
        }
    }
    Ok(Some(()))
}

/// Returns true when a concrete class uses the SPL doubly-linked-list append helper.
pub(super) fn is_spl_doubly_linked_list_family(class_name: &str) -> bool {
    matches!(class_name, "SplDoublyLinkedList" | "SplStack" | "SplQueue")
}

/// Selects the ArrayAccess runtime dispatch strategy for a receiver type.
pub(super) fn array_access_runtime_dispatch(
    ctx: &FunctionContext<'_>,
    receiver_ty: &PhpType,
) -> Option<ArrayAccessRuntimeDispatch> {
    match receiver_ty {
        PhpType::Object(class_name) => {
            let normalized = class_name.trim_start_matches('\\');
            if interface_satisfies_interface(ctx, normalized, "ArrayAccess") {
                return Some(ArrayAccessRuntimeDispatch::Interface {
                    boxed_receiver: false,
                });
            }
            if class_implements_interface(ctx, normalized, "ArrayAccess") {
                return Some(ArrayAccessRuntimeDispatch::Concrete(normalized.to_string()));
            }
            None
        }
        PhpType::Union(members) if union_satisfies_array_access(ctx, members) => {
            Some(ArrayAccessRuntimeDispatch::Interface {
                boxed_receiver: true,
            })
        }
        _ => None,
    }
}

/// Returns true when all non-null union arms are ArrayAccess-compatible objects.
pub(super) fn union_satisfies_array_access(ctx: &FunctionContext<'_>, members: &[PhpType]) -> bool {
    let mut saw_object = false;
    for member in members {
        match member {
            PhpType::Void | PhpType::Never => {}
            PhpType::Object(class_name) => {
                if !object_name_satisfies_interface(ctx, class_name, "ArrayAccess") {
                    return false;
                }
                saw_object = true;
            }
            _ => return false,
        }
    }
    saw_object
}

/// Returns true when a class or interface name satisfies the requested interface.
pub(super) fn object_name_satisfies_interface(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    interface_name: &str,
) -> bool {
    let normalized = class_name.trim_start_matches('\\');
    interface_satisfies_interface(ctx, normalized, interface_name)
        || class_implements_interface(ctx, normalized, interface_name)
}

/// Lowers ArrayAccess on a boxed union receiver through runtime interface metadata.
pub(super) fn lower_boxed_array_access_interface_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    method_name: &str,
) -> Result<()> {
    let (interface_name, method_key, callee_sig) =
        resolve_interface_call_signature(ctx, "ArrayAccess", method_name, inst.operands.len())?;
    let receiver = expect_operand(inst, 0)?;
    let receiver_ty = PhpType::Object(interface_name.clone());
    let mut param_types = Vec::with_capacity(callee_sig.params.len() + 1);
    param_types.push(receiver_ty.clone());
    param_types.extend(callee_sig.params.iter().map(|(_, ty)| ty.codegen_repr()));
    let mut ref_params = Vec::with_capacity(callee_sig.ref_params.len() + 1);
    ref_params.push(false);
    ref_params.extend(callee_sig.ref_params.iter().copied());

    ctx.load_value_to_result(receiver)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, mixed_unbox_low_payload_reg(ctx));
    abi::emit_pop_reg(ctx.emitter, receiver_reg);
    let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx,
        receiver_reg,
        &receiver_ty,
        &inst.operands,
        &param_types,
        &ref_params,
        crate::codegen::lower_inst::RefArgCellLifetime::CallOnly,
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    let return_ty =
        iterators::emit_interface_dispatch_call(ctx, &interface_name, &method_key, None)?;
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    store_call_result(ctx, inst, &return_ty)?;
    emit_ref_arg_writebacks(ctx, &call_args)
}

/// Emits the concrete method body backing a PHP object runtime fallback.
pub(in crate::codegen) fn lower_runtime_object_method_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    class_name: &str,
    method_name: &str,
) -> Result<()> {
    let target = resolve_method_call_target(ctx, class_name, method_name, inst.operands.len())?;
    let mut param_types = Vec::with_capacity(target.params.len() + 1);
    param_types.push(PhpType::Object(class_name.to_string()));
    param_types.extend(target.params.iter().map(|param| param.codegen_repr()));
    let mut ref_params = Vec::with_capacity(target.ref_params.len() + 1);
    ref_params.push(false);
    ref_params.extend(target.ref_params.iter().copied());
    let call_args = materialize_direct_call_args_with_refs_and_options(
        ctx,
        &inst.operands,
        &param_types,
        &ref_params,
        true,
        crate::codegen::lower_inst::RefArgCellLifetime::CallOnly,
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_call_label(
        ctx.emitter,
        &method_symbol(&target.impl_class, &target.method_key),
    );
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    store_runtime_object_call_result(ctx, inst, &target.return_ty)?;
    emit_call_arg_temp_cleanups(ctx, &call_args, inst.result)?;
    emit_ref_arg_writebacks(ctx, &call_args)
}

/// Stores an object fallback call result, casting boxed Mixed values when the access type is known.
pub(super) fn store_runtime_object_call_result(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    return_ty: &PhpType,
) -> Result<()> {
    if return_ty.codegen_repr() != PhpType::Mixed {
        return store_call_result(ctx, inst, return_ty);
    }
    let Some(result) = inst.result else {
        return Ok(());
    };
    let result_ty = ctx.value_php_type(result)?.codegen_repr();
    if matches!(result_ty, PhpType::Mixed | PhpType::Union(_)) {
        ctx.store_result_value(result)?;
        return Ok(());
    }
    cast_loaded_mixed_pointer_to_result(ctx, &result_ty)?;
    ctx.store_result_value(result)
}

/// Returns true when a class implements an interface, following parent classes if needed.
pub(super) fn class_implements_interface(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    interface_name: &str,
) -> bool {
    let interface_key = php_symbol_key(interface_name.trim_start_matches('\\'));
    let mut current = Some(class_name.trim_start_matches('\\'));
    while let Some(candidate) = current {
        let Some(info) = ctx.module.class_infos.get(candidate) else {
            return false;
        };
        if info.interfaces.iter().any(|interface| {
            let interface = interface.trim_start_matches('\\');
            php_symbol_key(interface) == interface_key
                || interface_satisfies_interface(ctx, interface, interface_name)
        }) {
            return true;
        }
        current = info.parent.as_deref();
    }
    false
}

/// Returns true when an interface is or extends the requested ancestor.
pub(super) fn interface_satisfies_interface(
    ctx: &FunctionContext<'_>,
    interface_name: &str,
    ancestor_name: &str,
) -> bool {
    if php_symbol_key(interface_name.trim_start_matches('\\'))
        == php_symbol_key(ancestor_name.trim_start_matches('\\'))
    {
        return true;
    }
    let Some(interface_info) = ctx
        .module
        .interface_infos
        .get(interface_name.trim_start_matches('\\'))
    else {
        return false;
    };
    interface_info.parents.iter().any(|parent| {
        let parent = parent.trim_start_matches('\\');
        php_symbol_key(parent) == php_symbol_key(ancestor_name.trim_start_matches('\\'))
            || interface_satisfies_interface(ctx, parent, ancestor_name)
    })
}

/// Converts an untyped boxed Mixed payload into indexed-array storage with Mixed slots.
pub(super) fn lower_mixed_to_mixed_indexed_array(ctx: &mut FunctionContext<'_>) -> Result<()> {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // pass the unboxed indexed-array payload to the Mixed conversion helper
            ctx.emitter.instruction("ldr x1, [x0, #-8]");                       // load indexed-array metadata before Mixed-slot conversion
            ctx.emitter.instruction("lsr x1, x1, #8");                          // move the runtime value_type tag into the low bits
            ctx.emitter.instruction("and x1, x1, #0x7f");                       // isolate the indexed-array value_type tag
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rsi, QWORD PTR [rdi - 8]");            // load indexed-array metadata before Mixed-slot conversion
            ctx.emitter.instruction("shr rsi, 8");                              // move the runtime value_type tag into the low bits
            ctx.emitter.instruction("and rsi, 0x7f");                           // isolate the indexed-array value_type tag
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_array_to_mixed");
    abi::emit_incref_if_refcounted(ctx.emitter, &PhpType::Array(Box::new(PhpType::Mixed)));
    Ok(())
}

/// Converts an untyped boxed Mixed payload into associative-array storage with Mixed values.
pub(super) fn lower_mixed_to_mixed_assoc_array(ctx: &mut FunctionContext<'_>) -> Result<()> {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x0, x1");                              // pass the unboxed associative-array payload to the Mixed conversion helper
        }
        Arch::X86_64 => {}
    }
    abi::emit_call_label(ctx.emitter, "__rt_hash_to_mixed");
    abi::emit_incref_if_refcounted(
        ctx.emitter,
        &PhpType::AssocArray {
            key: Box::new(PhpType::Mixed),
            value: Box::new(PhpType::Mixed),
        },
    );
    Ok(())
}
