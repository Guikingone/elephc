//! Purpose:
//! Lowers property reads from boxed Mixed and union receivers.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Runtime class dispatch produces owned Mixed results and preserves null warnings.

use super::*;

/// Lowers a property read from a raw object whose static type is polymorphic.
///
/// Interface or abstract-parent typing identifies the contract but not the concrete object layout.
/// Dispatch by runtime class id across every AOT implementation that owns the requested property,
/// then read that implementation's authoritative slot.
pub(super) fn lower_polymorphic_object_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    target_name: &str,
    property: &str,
) -> Result<()> {
    let object_reg = abi::int_result_reg(ctx.emitter);
    ctx.load_value_to_reg(object, object_reg)?;
    lower_polymorphic_object_prop_get_from_loaded_object(ctx, inst, target_name, property)
}

/// Dispatches a property read after the raw object payload has been loaded into the result register.
fn lower_polymorphic_object_prop_get_from_loaded_object(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    target_name: &str,
    property: &str,
) -> Result<()> {
    let candidates = polymorphic_property_candidates(ctx, target_name, property, inst)?;
    if candidates.is_empty() && !target_name.trim_start_matches('\\').is_empty() {
        return Err(CodegenIrError::unsupported(format!(
            "{} for polymorphic property {}::${}",
            inst.op.name(),
            target_name,
            property
        )));
    }

    let miss_label = ctx.next_label("interface_prop_miss");
    let done_label = ctx.next_label("interface_prop_done");
    let stdclass_label = target_name
        .trim_start_matches('\\')
        .is_empty()
        .then(|| ctx.next_label("generic_object_stdclass_prop"));
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "interface_prop_{}",
                label_fragment(&candidate.slot.class_name)
            ))
        })
        .collect::<Vec<_>>();

    emit_raw_object_property_class_dispatch(
        ctx,
        &candidates,
        &match_labels,
        stdclass_label.as_deref(),
        &miss_label,
    );

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let base_reg = abi::int_result_reg(ctx.emitter);
        if candidate.slot.is_declared {
            emit_uninitialized_typed_property_guard(ctx, &candidate.slot, base_reg);
        }
        emit_property_load(ctx, &candidate.slot, base_reg)?;
        materialize_loaded_property_result(ctx, inst, &candidate.slot.php_type)?;
        store_if_result(ctx, inst)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    if let Some(stdclass_label) = stdclass_label {
        ctx.emitter.label(&stdclass_label);
        emit_stdclass_get_from_loaded_object(ctx, property);
        cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
        store_if_result(ctx, inst)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&miss_label);
    emit_dynamic_property_miss_result(ctx, inst);
    store_if_result(ctx, inst)?;
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Collects concrete property slots for classes satisfying one interface or parent-class target.
fn polymorphic_property_candidates(
    ctx: &FunctionContext<'_>,
    target_name: &str,
    property: &str,
    inst: &Instruction,
) -> Result<Vec<MixedPropertyCandidate>> {
    let normalized_target = target_name.trim_start_matches('\\');
    let target_key = php_symbol_key(normalized_target);
    let target_is_any_object = normalized_target.is_empty();
    let target_is_interface = ctx
        .module
        .interface_infos
        .keys()
        .any(|candidate| php_symbol_key(candidate.trim_start_matches('\\')) == target_key);
    let mut candidates = Vec::new();
    for (class_name, class_info) in &ctx.module.class_infos {
        let satisfies_target = if target_is_any_object {
            true
        } else if target_is_interface {
            object_type_implements_interface(ctx, class_name, target_name)
        } else {
            php_symbol_key(class_name.trim_start_matches('\\')) == target_key
                || class_extends_class(ctx, class_name, target_name)
        };
        let backing = crate::types::checker::reflection_virtual_property_backing(
            class_name,
            property,
        )
        .unwrap_or(property);
        if !satisfies_target
            || !class_info
                .properties
                .iter()
                .any(|(name, _)| name == backing)
        {
            continue;
        }
        let slot = resolve_property_slot_for_class(ctx, class_name, property, inst)?;
        if target_is_any_object
            && !generic_property_candidate_matches_result(ctx, &slot.php_type, &inst.result_php_type)
        {
            continue;
        }
        candidates.push(MixedPropertyCandidate {
            class_id: class_info.class_id,
            slot,
        });
    }
    candidates.sort_by_key(|candidate| candidate.class_id);
    Ok(candidates)
}

/// Returns whether one concrete property slot can materialize the generic EIR result shape.
fn generic_property_candidate_matches_result(
    ctx: &FunctionContext<'_>,
    source: &PhpType,
    result: &PhpType,
) -> bool {
    let source = source.codegen_repr();
    let result = result.codegen_repr();
    if result == PhpType::Mixed || source == result {
        return true;
    }
    match (&source, &result) {
        (PhpType::Object(source_name), PhpType::Object(result_name)) => {
            source_name.trim_start_matches('\\').is_empty()
                || object_type_is_a(ctx, source_name, result_name)
        }
        _ => false,
    }
}

/// Emits class-id branches for a raw interface object already loaded in the result register.
fn emit_raw_object_property_class_dispatch(
    ctx: &mut FunctionContext<'_>,
    candidates: &[MixedPropertyCandidate],
    match_labels: &[String],
    stdclass_label: Option<&str>,
    miss_label: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x9, [x0]");                            // load the concrete class id behind the interface value
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "x10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp x9, x10");                         // compare against one AOT interface implementor
                ctx.emitter.instruction(&format!("b.eq {}", label));            // read that implementor's declared property slot
            }
            if let Some(stdclass_label) = stdclass_label {
                emit_branch_to_stdclass_candidate(ctx, "x9", "x10", stdclass_label);
            }
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r11, QWORD PTR [rax]");                // load the concrete class id behind the interface value
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "r10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp r11, r10");                        // compare against one AOT interface implementor
                ctx.emitter.instruction(&format!("je {}", label));              // read that implementor's declared property slot
            }
            if let Some(stdclass_label) = stdclass_label {
                emit_branch_to_stdclass_candidate(ctx, "r11", "r10", stdclass_label);
            }
        }
    }
    abi::emit_jump(ctx.emitter, miss_label);
}

/// Lowers a declared-property read from a boxed union that may hold one known object class.
pub(super) fn lower_union_object_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    class_name: &str,
    property: &str,
) -> Result<()> {
    let slot = resolve_property_slot_for_class(ctx, class_name, property, inst)?;
    let object_label = ctx.next_label("union_prop_object");
    let done_label = ctx.next_label("union_prop_done");
    ctx.load_value_to_reg(object, abi::int_result_reg(ctx.emitter))?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_branch_if_mixed_unboxed_object(ctx, &object_label);
    emit_dynamic_property_miss_result(ctx, inst);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&object_label);
    let base_reg = abi::symbol_scratch_reg(ctx.emitter);
    move_mixed_unboxed_object_payload(ctx, base_reg);
    if slot.is_declared {
        emit_uninitialized_typed_property_guard(ctx, &slot, base_reg);
    }
    emit_property_load(ctx, &slot, base_reg)?;
    materialize_loaded_property_result(ctx, inst, &slot.php_type)?;
    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Lowers `$mixed->property` through the shared stdClass-aware runtime helper.
pub(super) fn lower_mixed_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property: &str,
) -> Result<()> {
    if builtins::has_eval_context(ctx) {
        return builtins::lower_eval_property_get(ctx, inst, object, property);
    }
    let candidates = declared_mixed_property_candidates(ctx, property, inst)?;
    if !candidates.is_empty() {
        return lower_declared_mixed_prop_get(ctx, inst, object, property, candidates);
    }
    lower_runtime_mixed_prop_get(ctx, inst, object, property)
}

/// Lowers a `Mixed` receiver by dispatching known user classes before stdClass fallback.
pub(super) fn lower_declared_mixed_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property: &str,
    candidates: Vec<MixedPropertyCandidate>,
) -> Result<()> {
    let null_label = ctx.next_label("mixed_prop_null");
    let miss_label = ctx.next_label("mixed_prop_miss");
    let done_label = ctx.next_label("mixed_prop_done");
    let stdclass_label = ctx.next_label("mixed_prop_stdclass");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "mixed_prop_{}",
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

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let base_reg = abi::int_result_reg(ctx.emitter);
        if candidate.slot.is_declared {
            emit_uninitialized_typed_property_guard(ctx, &candidate.slot, base_reg);
        }
        emit_property_load(ctx, &candidate.slot, base_reg)?;
        box_mixed_property_candidate_result(ctx, &candidate.slot.php_type);
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&stdclass_label);
    emit_stdclass_get_from_loaded_object(ctx, property);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&miss_label);
    emit_undefined_property_warning_for_loaded_object(ctx, property);
    emit_boxed_null(ctx);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    emit_boxed_null(ctx);

    ctx.emitter.label(&done_label);
    cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
    store_if_result(ctx, inst)
}

/// Probes a declared typed slot through a receiver whose class is only known at run time.
///
/// `isset()`, `empty()` and `??` all ask this question before reading, precisely so they never
/// raise on an uninitialized typed property. Until this path existed the question could only be
/// asked of a receiver whose class the compiler knew, so an UNTYPED parameter — `function
/// f($container) { return isset($container->name); }`, and every generated Symfony container
/// factory is written that way — fell back to the ordinary read and died with
/// "Typed property C::$name must not be accessed before initialization" where PHP answers false.
///
/// Only the declared-slot branches answer for themselves. A stdClass receiver, a class that does
/// not declare the property at all, and every other miss answer "initialized", which sends the
/// caller to the ordinary read it has always taken — magic `__isset`, dynamic properties and the
/// undefined-property warning all keep behaving exactly as before. A receiver that is not an
/// object answers false, which is what PHP says for `isset($notAnObject->p)`.
pub(super) fn lower_mixed_prop_initialized(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property: &str,
) -> Result<()> {
    let candidates = declared_mixed_property_candidates(ctx, property, inst)?;
    let not_object_label = ctx.next_label("mixed_prop_init_not_object");
    let fallthrough_label = ctx.next_label("mixed_prop_init_fallthrough");
    let stdclass_label = ctx.next_label("mixed_prop_init_stdclass");
    let done_label = ctx.next_label("mixed_prop_init_done");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "mixed_prop_init_{}",
                label_fragment(&candidate.slot.class_name)
            ))
        })
        .collect::<Vec<_>>();

    ctx.load_value_to_reg(object, abi::int_result_reg(ctx.emitter))?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_mixed_object_payload_or_null(ctx, &not_object_label);
    emit_mixed_property_class_dispatch(
        ctx,
        &candidates,
        &match_labels,
        &stdclass_label,
        &fallthrough_label,
    );

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let base_reg = abi::int_result_reg(ctx.emitter);
        if candidate.slot.is_declared {
            emit_typed_property_initialized_bool(ctx, &candidate.slot, base_reg);
        } else {
            // An untyped slot is plain null from construction, so it is always initialized.
            abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
        }
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&stdclass_label);
    ctx.emitter.label(&fallthrough_label);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&not_object_label);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 0);

    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Lowers a `Mixed` receiver through the runtime stdClass-style property helper.
pub(super) fn lower_runtime_mixed_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property: &str,
) -> Result<()> {
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(object, "x0")?;
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(object, "rdi")?;
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_mixed_property_get");
    cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
    store_if_result(ctx, inst)
}

/// Collects declared-property candidates for a property read on an unknown `Mixed` object.
pub(super) fn declared_mixed_property_candidates(
    ctx: &FunctionContext<'_>,
    property: &str,
    inst: &Instruction,
) -> Result<Vec<MixedPropertyCandidate>> {
    let mut candidates = Vec::new();
    for (class_name, class_info) in &ctx.module.class_infos {
        if crate::types::checker::builtin_stdclass::is_stdclass(class_name) {
            continue;
        }
        let backing = crate::types::checker::reflection_virtual_property_backing(
            class_name,
            property,
        )
        .unwrap_or(property);
        if !class_info
            .properties
            .iter()
            .any(|(name, _)| name == backing)
        {
            continue;
        }
        let slot = resolve_property_slot_for_class(ctx, class_name, property, inst)?;
        candidates.push(MixedPropertyCandidate {
            class_id: class_info.class_id,
            slot,
        });
    }
    candidates.sort_by_key(|candidate| candidate.class_id);
    Ok(candidates)
}

/// Promotes an unboxed Mixed object payload into the normal result register or jumps to null.
pub(super) fn emit_mixed_object_payload_or_null(ctx: &mut FunctionContext<'_>, null_label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // check whether the Mixed receiver holds an object payload
            ctx.emitter.instruction(&format!("b.ne {}", null_label));           // non-object Mixed receivers produce a null property result
            ctx.emitter.instruction("mov x0, x1");                              // promote the unboxed object payload for class-id dispatch
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // check whether the Mixed receiver holds an object payload
            ctx.emitter.instruction(&format!("jne {}", null_label));            // non-object Mixed receivers produce a null property result
            ctx.emitter.instruction("mov rax, rdi");                            // promote the unboxed object payload for class-id dispatch
        }
    }
}

/// Emits class-id dispatch for declared property candidates, stdClass, and a real miss branch.
pub(super) fn emit_mixed_property_class_dispatch(
    ctx: &mut FunctionContext<'_>,
    candidates: &[MixedPropertyCandidate],
    match_labels: &[String],
    stdclass_label: &str,
    miss_label: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x9, [x0]");                            // load the receiver class id for Mixed property dispatch
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "x10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp x9, x10");                         // compare the receiver class id against this declared-property owner
                ctx.emitter.instruction(&format!("b.eq {}", label));            // read the declared property when the class id matches
            }
            emit_branch_to_stdclass_candidate(ctx, "x9", "x10", stdclass_label);
            abi::emit_jump(ctx.emitter, miss_label);
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r11, QWORD PTR [rax]");                // load the receiver class id for Mixed property dispatch
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "r10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp r11, r10");                        // compare the receiver class id against this declared-property owner
                ctx.emitter.instruction(&format!("je {}", label));              // read the declared property when the class id matches
            }
            emit_branch_to_stdclass_candidate(ctx, "r11", "r10", stdclass_label);
            abi::emit_jump(ctx.emitter, miss_label);
        }
    }
}

/// Branches to the stdClass fallback when the runtime module contains stdClass metadata.
pub(super) fn emit_branch_to_stdclass_candidate(
    ctx: &mut FunctionContext<'_>,
    class_id_reg: &str,
    scratch_reg: &str,
    stdclass_label: &str,
) {
    let Some(stdclass_id) = stdclass_class_id(ctx) else {
        return;
    };
    abi::emit_load_int_immediate(ctx.emitter, scratch_reg, stdclass_id as i64);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, {}", class_id_reg, scratch_reg)); // check whether the object uses stdClass dynamic storage
            ctx.emitter.instruction(&format!("b.eq {}", stdclass_label));       // route stdClass reads through the hash-backed helper
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("cmp {}, {}", class_id_reg, scratch_reg)); // check whether the object uses stdClass dynamic storage
            ctx.emitter.instruction(&format!("je {}", stdclass_label));         // route stdClass reads through the hash-backed helper
        }
    }
}

/// Returns the runtime class id assigned to stdClass in this module.
pub(super) fn stdclass_class_id(ctx: &FunctionContext<'_>) -> Option<u64> {
    ctx.module
        .class_infos
        .iter()
        .find(|(class_name, _)| crate::types::checker::builtin_stdclass::is_stdclass(class_name))
        .map(|(_, class_info)| class_info.class_id)
}

/// Boxes or retains a declared-property load so Mixed receiver paths produce owned Mixed cells.
pub(super) fn box_mixed_property_candidate_result(ctx: &mut FunctionContext<'_>, source_ty: &PhpType) {
    let source_ty = source_ty.codegen_repr();
    if source_ty == PhpType::Mixed {
        abi::emit_incref_if_refcounted(ctx.emitter, &source_ty);
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, &source_ty);
    }
}

/// Reads a static property name from an already-unboxed stdClass payload.
pub(super) fn emit_stdclass_get_from_loaded_object(ctx: &mut FunctionContext<'_>, property: &str) {
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x1", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x2", len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_stdclass_get");
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdi, rax");                            // pass the unboxed stdClass object pointer to the dynamic getter
            abi::emit_symbol_address(ctx.emitter, "rsi", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rdx", len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_stdclass_get");
        }
    }
}

/// Branches when `__rt_mixed_unbox` returned an object payload tag.
pub(super) fn emit_branch_if_mixed_unboxed_object(ctx: &mut FunctionContext<'_>, object_label: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("b.eq {}", object_label));         // read the declared property only for object payloads
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // runtime tag 6 means the boxed union holds an object payload
            ctx.emitter.instruction(&format!("je {}", object_label));           // read the declared property only for object payloads
        }
    }
}

/// Moves the low payload produced by `__rt_mixed_unbox` into the object base register.
pub(super) fn move_mixed_unboxed_object_payload(ctx: &mut FunctionContext<'_>, base_reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov {}, x1", base_reg));          // use the unboxed object pointer as the declared-property base
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, rdi", base_reg));         // use the unboxed object pointer as the declared-property base
        }
    }
}

/// Lowers `$maybeObject->property`, warning when the receiver is PHP null.
pub(super) fn lower_nullable_prop_get_with_warning(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    class_name: &str,
    property: &str,
) -> Result<()> {
    if class_name.trim_start_matches('\\').is_empty() {
        return lower_nullable_generic_object_prop_get(ctx, inst, object, property);
    }
    let slot = resolve_property_slot_for_class(ctx, class_name, property, inst)?;
    let null_label = ctx.next_label("nullable_prop_warning_null");
    let done_label = ctx.next_label("nullable_prop_warning_done");
    let base_reg = abi::symbol_scratch_reg(ctx.emitter);
    emit_nullable_receiver_object_payload(ctx, object, &null_label, base_reg)?;
    if slot.is_declared {
        emit_uninitialized_typed_property_guard(ctx, &slot, base_reg);
    }
    emit_property_load(ctx, &slot, base_reg)?;
    materialize_loaded_property_result(ctx, inst, &slot.php_type)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    if inst.op != Op::NullsafePropGet {
        emit_property_on_null_warning(ctx, property);
    }
    emit_boxed_null(ctx);

    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Lowers a nullable bare-object property read through concrete runtime class dispatch.
fn lower_nullable_generic_object_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property: &str,
) -> Result<()> {
    let null_label = ctx.next_label("nullable_generic_prop_null");
    let done_label = ctx.next_label("nullable_generic_prop_done");
    let object_reg = abi::int_result_reg(ctx.emitter);
    emit_nullable_receiver_object_payload(ctx, object, &null_label, object_reg)?;
    lower_polymorphic_object_prop_get_from_loaded_object(ctx, inst, "", property)?;
    abi::emit_jump(ctx.emitter, &done_label);

    ctx.emitter.label(&null_label);
    if inst.op != Op::NullsafePropGet {
        emit_property_on_null_warning(ctx, property);
    }
    emit_boxed_null(ctx);
    store_if_result(ctx, inst)?;

    ctx.emitter.label(&done_label);
    Ok(())
}


/// Emits PHP's warning for reading a property from null.
pub(super) fn emit_property_on_null_warning(ctx: &mut FunctionContext<'_>, property: &str) {
    let message = format!(
        "Warning: Attempt to read property \"{}\" on null\n",
        property
    );
    let (message_label, message_len) = ctx.data.add_string(message.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.adrp("x1", &message_label);
            ctx.emitter.add_lo12("x1", "x1", &message_label);
            ctx.emitter
                .instruction(&format!("mov x2, #{}", message_len)); // pass the property-on-null warning byte length
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rdi", &message_label);
            ctx.emitter
                .instruction(&format!("mov esi, {}", message_len)); // pass the property-on-null warning byte length
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_diag_warning");
}

/// Emits `Warning: Undefined property: Class::$name` for an object already in the result register.
pub(super) fn emit_undefined_property_warning_for_loaded_object(
    ctx: &mut FunctionContext<'_>,
    property: &str,
) {
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("ldr x9, [x0]");                            // load the missing-property receiver class id
            abi::emit_symbol_address(ctx.emitter, "x10", "_class_name_entries");
            ctx.emitter.instruction("lsl x11, x9, #4");                         // scale the class id to the 16-byte class-name row
            ctx.emitter.instruction("add x10, x10, x11");                       // address the receiver's class-name metadata
            ctx.emitter.instruction("ldr x1, [x10]");                           // load the receiver class-name pointer
            ctx.emitter.instruction("ldr x2, [x10, #8]");                       // load the receiver class-name byte length
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("mov r9, QWORD PTR [rax]");                 // load the missing-property receiver class id
            abi::emit_symbol_address(ctx.emitter, "r10", "_class_name_entries");
            ctx.emitter.instruction("shl r9, 4");                               // scale the class id to the 16-byte class-name row
            ctx.emitter.instruction("mov rax, QWORD PTR [r10 + r9]");           // load the receiver class-name pointer
            ctx.emitter.instruction("mov rdx, QWORD PTR [r10 + r9 + 8]");       // load the receiver class-name byte length
        }
    }
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);
    emit_property_warning_fragment(ctx, b"Warning: Undefined property: ");
    match ctx.emitter.target.arch {
        Arch::AArch64 => abi::emit_pop_reg_pair(ctx.emitter, "x1", "x2"),
        Arch::X86_64 => abi::emit_pop_reg_pair(ctx.emitter, "rdi", "rsi"),
    }
    abi::emit_call_label(ctx.emitter, "__rt_diag_warning");
    emit_property_warning_fragment(ctx, format!("::${}\n", property).as_bytes());
}
