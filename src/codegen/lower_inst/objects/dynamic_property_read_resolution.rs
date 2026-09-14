//! Purpose:
//! Resolves runtime property names against stdClass and declared slots.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Candidate type compatibility and miss materialization remain explicit.

use super::*;

/// Lowers a runtime-name dynamic property read from a statically known `stdClass`.
pub(super) fn lower_runtime_dynamic_stdclass_prop_get(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    property_value: ValueId,
) -> Result<()> {
    ensure_runtime_dynamic_property_name(ctx, property_value, inst)?;
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.load_value_to_reg(object, "x0")?;
            ctx.load_string_value_to_regs(property_value, "x1", "x2")?;
        }
        Arch::X86_64 => {
            ctx.load_value_to_reg(object, "rdi")?;
            ctx.load_string_value_to_regs(property_value, "rsi", "rdx")?;
        }
    }
    abi::emit_call_label(ctx.emitter, "__rt_stdclass_get");
    cast_loaded_mixed_pointer_to_result(ctx, &inst.result_php_type.codegen_repr())?;
    store_if_result(ctx, inst)
}

/// Lowers a runtime string dynamic property read by dispatching across declared slots.
pub(super) fn lower_runtime_dynamic_declared_prop_get(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    property_value: ValueId,
    inst: &Instruction,
) -> Result<()> {
    let class_name = dynamic_property_object_class(ctx, object, inst)?;
    ensure_runtime_dynamic_property_name(ctx, property_value, inst)?;
    ensure_dynamic_property_miss_supported(inst)?;
    let mode = property_fetch_mode(inst);
    let arms = declared_dynamic_property_read_arms(ctx, &class_name, mode, inst)?;
    ensure_dynamic_property_arm_results_supported(&arms, inst)?;
    let match_labels = arms
        .iter()
        .map(|arm| ctx.next_label(&format!("dyn_prop_{}", label_fragment(arm.property()))))
        .collect::<Vec<_>>();
    let miss_label = ctx.next_label("dyn_prop_miss");
    let done_label = ctx.next_label("dyn_prop_done");

    let object_reg = abi::int_result_reg(ctx.emitter);
    ctx.load_value_to_reg(object, object_reg)?;
    abi::emit_push_reg(ctx.emitter, object_reg);
    let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
    ctx.load_string_value_to_regs(property_value, ptr_reg, len_reg)?;
    abi::emit_push_reg_pair(ctx.emitter, ptr_reg, len_reg);

    for (arm, label) in arms.iter().zip(match_labels.iter()) {
        emit_branch_if_dynamic_name_matches(ctx, arm.property(), label);
    }
    abi::emit_jump(ctx.emitter, &miss_label);

    for (arm, label) in arms.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        let base_reg = abi::symbol_scratch_reg(ctx.emitter);
        abi::emit_load_temporary_stack_slot(ctx.emitter, base_reg, 16);
        match arm {
            PropertyNameArm::Slot(slot) => {
                if slot.is_declared {
                    emit_uninitialized_typed_property_guard(ctx, slot, base_reg);
                }
                emit_property_load(ctx, slot, base_reg)?;
                materialize_loaded_property_result(ctx, inst, &slot.php_type)?;
                abi::emit_release_temporary_stack(ctx.emitter, 32);
            }
            // php makes the name invisible here, so it answers from the per-instance hash and the
            // ancestor's private slot keeps its value. The receiver is read out of the temporary
            // block first, then the block is released, so the arm reaches `done` with the same
            // stack pointer as every other one.
            PropertyNameArm::ScopeDynamic { property } => {
                abi::emit_release_temporary_stack(ctx.emitter, 32);
                emit_scope_dynamic_property_read(
                    ctx, inst, &class_name, property, base_reg, mode,
                )?;
            }
            // php refuses the read outright. The temporary block is released BEFORE the raise,
            // which is what keeps the unwinder's stack pointer consistent with every other arm.
            PropertyNameArm::Refuse { message, .. } => {
                abi::emit_release_temporary_stack(ctx.emitter, 32);
                super::super::exceptions::emit_error(ctx, message);
            }
            // php would answer `__get` or `__isset` here. Until a runtime name can reach the
            // accessor, the arm answers php `null`: it must not read the slot, which holds
            // private storage this scope may not see, and it must not report a diagnostic php
            // does not report either.
            PropertyNameArm::MagicDeferred { .. } => {
                abi::emit_release_temporary_stack(ctx.emitter, 32);
                emit_dynamic_property_miss_result(ctx, inst);
            }
        }
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&miss_label);
    // A class with per-instance hash storage keeps undeclared names there, so the ladder's miss
    // arm has to probe it before answering null: `$o->{$name}` must see exactly what the clone
    // override applicator, or a literal-name write, stored under the same key.
    match dynamic_property_hash_offset_for_class(ctx, &class_name, "")? {
        Some(hash_offset) => {
            lower_runtime_allow_dynamic_prop_get(ctx, inst, hash_offset, 16, 0, 32)?;
        }
        None => {
            abi::emit_release_temporary_stack(ctx.emitter, 32);
            emit_dynamic_property_miss_result(ctx, inst);
        }
    }
    ctx.emitter.label(&done_label);
    store_if_result(ctx, inst)
}

/// Returns the normalized class name for object receivers supported by dynamic property dispatch.
pub(super) fn dynamic_property_object_class(
    ctx: &FunctionContext<'_>,
    object: ValueId,
    inst: &Instruction,
) -> Result<String> {
    let object_ty = ctx.value_php_type(object)?;
    let PhpType::Object(class_name) = object_ty else {
        return Err(CodegenIrError::unsupported(format!(
            "{} for runtime dynamic receiver PHP type {:?}",
            inst.op.name(),
            object_ty
        )));
    };
    Ok(class_name.trim_start_matches('\\').to_string())
}

/// Verifies that the dynamic property name is already materialized as a string.
pub(super) fn ensure_runtime_dynamic_property_name(
    ctx: &FunctionContext<'_>,
    property_value: ValueId,
    inst: &Instruction,
) -> Result<()> {
    let property_ty = ctx.value_php_type(property_value)?;
    if property_ty == PhpType::Str {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "{} with runtime property name PHP type {:?}",
        inst.op.name(),
        property_ty
    )))
}

/// Resolves the arm every declared property name takes on a runtime-name read.
///
/// A name the LEXICAL scope resolves to a DYNAMIC property becomes a `ScopeDynamic` arm, which
/// answers from the per-instance hash instead of the physical slot. That is what php does with a
/// strict ancestor's private property: outside the class that declared it, the name is a dynamic
/// property and the private slot keeps its own value. A name php refuses becomes a `Refuse` arm
/// for a value read and is dropped for a silent probe, which then answers `null` from the miss
/// arm exactly as php's `isset()` does.
pub(super) fn declared_dynamic_property_read_arms(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    mode: PropertyFetchMode,
    inst: &Instruction,
) -> Result<Vec<PropertyNameArm>> {
    let normalized = class_name.trim_start_matches('\\');
    let property_names = {
        let class_info =
            ctx.module.class_infos.get(normalized).ok_or_else(|| {
                CodegenIrError::unsupported(format!("unknown class {}", normalized))
            })?;
        class_info
            .properties
            .iter()
            .map(|(property, _)| property.clone())
            .collect::<Vec<_>>()
    };
    let mut arms = Vec::with_capacity(property_names.len());
    for property in &property_names {
        if let Some(arm) = resolve_property_read_arm(ctx, normalized, property, mode, inst)? {
            arms.push(arm);
        }
    }
    Ok(arms)
}

/// Collects declared-property candidates readable from a boxed Mixed receiver.
///
/// A name php refuses on one of those classes keeps its arm, because the ladder dispatches on the
/// receiver's runtime class id and name, but the arm raises instead of reading. A silent probe
/// drops the arm so the name lands in the miss path and answers `null`.
pub(super) fn declared_mixed_property_get_candidates(
    ctx: &FunctionContext<'_>,
    mode: PropertyFetchMode,
    inst: &Instruction,
) -> Result<Vec<MixedPropertyReadCandidate>> {
    let mut candidates = Vec::new();
    let mut sorted_classes = ctx.module.class_infos.iter().collect::<Vec<_>>();
    sorted_classes.sort_by_key(|(_, class_info)| class_info.class_id);
    for (class_name, class_info) in sorted_classes {
        if crate::types::checker::builtin_stdclass::is_stdclass(class_name) {
            continue;
        }
        for (property, _) in &class_info.properties {
            let (slot, kind) =
                match resolve_property_read_arm(ctx, class_name, property, mode, inst) {
                    Ok(Some(PropertyNameArm::Slot(slot))) => (slot, MixedPropertyReadKind::Slot),
                    Ok(Some(PropertyNameArm::Refuse { message, .. })) => {
                        let Ok(slot) =
                            resolve_property_slot_for_class(ctx, class_name, property, inst)
                        else {
                            continue;
                        };
                        (slot, MixedPropertyReadKind::Refuse(message))
                    }
                    // The arm exists so the class id and name still dispatch here, but it answers
                    // from the per-instance hash and warns on a READ miss instead of reading the
                    // slot. Dropping it sent the name to the shared miss arm, which is silent.
                    Ok(Some(PropertyNameArm::ScopeDynamic { .. })) => {
                        let Ok(slot) =
                            resolve_property_slot_for_class(ctx, class_name, property, inst)
                        else {
                            continue;
                        };
                        (slot, MixedPropertyReadKind::ScopeDynamic)
                    }
                    Ok(Some(PropertyNameArm::MagicDeferred { .. })) => {
                        let Ok(slot) =
                            resolve_property_slot_for_class(ctx, class_name, property, inst)
                        else {
                            continue;
                        };
                        (slot, MixedPropertyReadKind::MagicDeferred)
                    }
                    _ => continue,
                };
            candidates.push(MixedPropertyReadCandidate {
                candidate: MixedPropertyCandidate {
                    class_id: class_info.class_id,
                    slot,
                },
                kind,
            });
        }
    }
    candidates.sort_by(|left, right| {
        left.candidate
            .class_id
            .cmp(&right.candidate.class_id)
            .then_with(|| {
                left.candidate
                    .slot
                    .property
                    .cmp(&right.candidate.slot.property)
            })
    });
    Ok(candidates)
}

/// Verifies that the EIR result type can receive every declared property arm.
pub(super) fn ensure_dynamic_property_arm_results_supported(
    arms: &[PropertyNameArm],
    inst: &Instruction,
) -> Result<()> {
    let slots = arms
        .iter()
        .filter_map(|arm| match arm {
            PropertyNameArm::Slot(slot) => Some(slot.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    ensure_dynamic_property_slot_results_supported(&slots, inst)
}

/// Verifies that the EIR result type can receive every declared property candidate.
pub(super) fn ensure_dynamic_property_slot_results_supported(
    slots: &[PropertySlot],
    inst: &Instruction,
) -> Result<()> {
    let result_ty = inst.result_php_type.codegen_repr();
    if result_ty == PhpType::Mixed {
        return Ok(());
    }
    for slot in slots {
        let slot_ty = slot.php_type.codegen_repr();
        let can_tag_nullable_int = result_ty == PhpType::TaggedScalar && slot_ty == PhpType::Int;
        if slot_ty != result_ty && !can_tag_nullable_int {
            return Err(CodegenIrError::unsupported(format!(
                "{} with declared property {}::${} PHP type {:?} and result PHP type {:?}",
                inst.op.name(),
                slot.class_name,
                slot.property,
                slot.php_type,
                result_ty
            )));
        }
    }
    Ok(())
}

/// Verifies that a runtime miss can be materialized in the EIR result register shape.
pub(super) fn ensure_dynamic_property_miss_supported(inst: &Instruction) -> Result<()> {
    match inst.result_php_type.codegen_repr() {
        PhpType::Mixed | PhpType::TaggedScalar | PhpType::Bool | PhpType::Int => Ok(()),
        ty => Err(CodegenIrError::unsupported(format!(
            "{} runtime miss for result PHP type {:?}",
            inst.op.name(),
            ty
        ))),
    }
}

/// Converts a just-loaded property payload into the EIR result representation.
pub(super) fn materialize_loaded_property_result(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    source_ty: &PhpType,
) -> Result<()> {
    let source_ty = source_ty.codegen_repr();
    match inst.result_php_type.codegen_repr() {
        PhpType::Mixed if source_ty == PhpType::Mixed => {
            abi::emit_incref_if_refcounted(ctx.emitter, &source_ty);
            Ok(())
        }
        PhpType::Mixed => {
            emit_box_current_value_as_mixed(ctx.emitter, &source_ty);
            Ok(())
        }
        PhpType::TaggedScalar if source_ty != PhpType::TaggedScalar => {
            super::super::coerce_loaded_value_to_tagged_scalar(ctx, &source_ty)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Emits a PHP null value for a dynamic property lookup that matched no declared slot.
pub(super) fn emit_dynamic_property_miss_result(ctx: &mut FunctionContext<'_>, inst: &Instruction) {
    match inst.result_php_type.codegen_repr() {
        PhpType::Mixed => emit_boxed_null(ctx),
        PhpType::TaggedScalar => {
            crate::codegen::sentinels::emit_tagged_scalar_null(ctx.emitter);
        }
        _ => abi::emit_load_int_immediate(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            RUNTIME_NULL_SENTINEL,
        ),
    }
}

/// Emits a runtime string comparison branch against one declared property name.
pub(super) fn emit_branch_if_dynamic_name_matches(
    ctx: &mut FunctionContext<'_>,
    property: &str,
    target_label: &str,
) {
    let (label, len) = ctx.data.add_string(property.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", 8);
            abi::emit_symbol_address(ctx.emitter, "x3", &label);
            abi::emit_load_int_immediate(ctx.emitter, "x4", len as i64);
            ctx.emitter.instruction("bl __rt_str_eq");                          // compare the runtime property name against this declared property
            ctx.emitter
                .instruction(&format!("cbnz x0, {}", target_label)); // dispatch to the declared property slot when the names match
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", 8);
            abi::emit_symbol_address(ctx.emitter, "rdx", &label);
            abi::emit_load_int_immediate(ctx.emitter, "rcx", len as i64);
            ctx.emitter.instruction("call __rt_str_eq");                        // compare the runtime property name against this declared property
            ctx.emitter.instruction("test rax, rax");                           // check whether the runtime string comparison matched
            ctx.emitter.instruction(&format!("jne {}", target_label));          // dispatch to the declared property slot when the names match
        }
    }
}

/// Converts arbitrary names into assembly-label-safe fragments.
pub(super) fn label_fragment(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}
