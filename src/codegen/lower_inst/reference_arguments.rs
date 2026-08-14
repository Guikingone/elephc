//! Purpose:
//! Materializes method and by-reference call arguments with writeback.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;

/// Loads method call arguments for lexical `self::`/`parent::` instance calls using local `this`.
pub(super) fn materialize_method_call_args_with_receiver_local_and_refs(
    ctx: &mut FunctionContext<'_>,
    receiver_slot: LocalSlotId,
    receiver_ty: &PhpType,
    operands: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
) -> Result<CallArgMaterialization> {
    if operands.len() + 1 != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "lexical instance call materialization received {} operands for {} params",
            operands.len(),
            param_types.len()
        )));
    }
    if ref_params.len() != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "lexical instance call materialization received {} ref flags for {} params",
            ref_params.len(),
            param_types.len()
        )));
    }
    let visible_param_types = &param_types[1..];
    let visible_ref_params = &ref_params[1..];
    let mut ref_writebacks =
        plan_ref_arg_writebacks(ctx, operands, visible_param_types, visible_ref_params)?;
    emit_ref_arg_temp_cells(ctx, &mut ref_writebacks)?;
    let abi_param_types = abi_param_types_for_refs(param_types, ref_params);
    let assignments =
        abi::build_outgoing_arg_assignments_for_target(ctx.emitter.target, &abi_param_types, 0);
    ctx.load_local_to_result(receiver_slot)?;
    abi::emit_push_result_value(ctx.emitter, receiver_ty);
    let mut arg_temp_bytes = call_arg_temp_slot_size(&abi_param_types[0]);
    for (index, (value, param_ty)) in operands.iter().zip(visible_param_types.iter()).enumerate() {
        if visible_ref_params[index] {
            materialize_ref_arg_address(
                ctx,
                *value,
                index,
                param_ty,
                arg_temp_bytes,
                &ref_writebacks,
                0,
            )?;
            abi::emit_push_result_value(ctx.emitter, &PhpType::Int);
        } else {
            ctx.load_value_to_result(*value)?;
            let source_ty = ctx.raw_value_php_type(*value)?;
            let push_ty = materialize_direct_call_arg_for_param(ctx, &source_ty, param_ty)?;
            abi::emit_push_result_value(ctx.emitter, &push_ty);
        }
        arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[index + 1]);
    }
    Ok(CallArgMaterialization {
        overflow_bytes: abi::materialize_outgoing_args(ctx.emitter, &assignments),
        ref_writebacks,
        cleanup_slots: Vec::new(),
        cleanup_bytes: 0,
        borrowed_stack_arg_bytes: 0,
    })
}

/// Loads method call arguments with by-reference parameter support for local operands.
pub(super) fn materialize_method_call_args_with_receiver_reg_and_refs(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    receiver_ty: &PhpType,
    operands: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
) -> Result<CallArgMaterialization> {
    if operands.len() != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "method call materialization received {} operands for {} params",
            operands.len(),
            param_types.len()
        )));
    }
    if ref_params.len() != param_types.len() {
        return Err(CodegenIrError::invalid_module(format!(
            "method call materialization received {} ref flags for {} params",
            ref_params.len(),
            param_types.len()
        )));
    }
    let ref_writebacks = plan_ref_arg_writebacks(ctx, operands, param_types, ref_params)?;
    if !ref_writebacks.is_empty() {
        return Err(CodegenIrError::unsupported(
            "receiver-register method call with scalar-to-mixed by-reference writebacks",
        ));
    }
    let abi_param_types = abi_param_types_for_refs(param_types, ref_params);
    let assignments =
        abi::build_outgoing_arg_assignments_for_target(ctx.emitter.target, &abi_param_types, 0);
    move_reg_to_int_result(ctx, receiver_reg);
    abi::emit_push_result_value(ctx.emitter, receiver_ty);
    let mut arg_temp_bytes = call_arg_temp_slot_size(&abi_param_types[0]);
    for (index, (value, param_ty)) in operands
        .iter()
        .skip(1)
        .zip(param_types.iter().skip(1))
        .enumerate()
    {
        let param_index = index + 1;
        if ref_params[param_index] {
            materialize_ref_arg_address(
                ctx,
                *value,
                param_index,
                &param_types[param_index],
                arg_temp_bytes,
                &ref_writebacks,
                0,
            )?;
            abi::emit_push_result_value(ctx.emitter, &PhpType::Int);
        } else {
            ctx.load_value_to_result(*value)?;
            let source_ty = ctx.raw_value_php_type(*value)?;
            let push_ty = materialize_direct_call_arg_for_param(ctx, &source_ty, param_ty)?;
            abi::emit_push_result_value(ctx.emitter, &push_ty);
        }
        arg_temp_bytes += call_arg_temp_slot_size(&abi_param_types[param_index]);
    }
    Ok(CallArgMaterialization {
        overflow_bytes: abi::materialize_outgoing_args(ctx.emitter, &assignments),
        ref_writebacks,
        cleanup_slots: Vec::new(),
        cleanup_bytes: 0,
        borrowed_stack_arg_bytes: 0,
    })
}

/// Converts declared parameter types to the ABI-visible shape for by-reference args.
pub(super) fn abi_param_types_for_refs(param_types: &[PhpType], ref_params: &[bool]) -> Vec<PhpType> {
    param_types
        .iter()
        .zip(ref_params.iter())
        .map(|(ty, is_ref)| {
            if *is_ref {
                PhpType::Int
            } else {
                ty.codegen_repr()
            }
        })
        .collect()
}

/// Returns the temporary stack slot size used by outgoing-argument staging.
pub(super) fn call_arg_temp_slot_size(ty: &PhpType) -> usize {
    if matches!(ty.codegen_repr(), PhpType::Void | PhpType::Never) {
        0
    } else {
        16
    }
}

/// Plans caller-side Mixed cells needed for scalar locals passed to by-reference Mixed params.
pub(super) fn plan_ref_arg_writebacks(
    ctx: &FunctionContext<'_>,
    args: &[ValueId],
    param_types: &[PhpType],
    ref_params: &[bool],
) -> Result<Vec<RefArgWriteback>> {
    let mut writebacks = Vec::new();
    for (param_index, value) in args.iter().enumerate() {
        if !ref_params[param_index] || param_types[param_index].codegen_repr() != PhpType::Mixed {
            continue;
        }
        let source_ty = ctx.raw_value_php_type(*value)?.codegen_repr();
        if matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) {
            continue;
        }
        // A non-local by-reference argument has no caller variable to update. It is materialized
        // as a throwaway ref cell below; only local sources participate in writeback planning.
        let Ok(source) = local_ref_arg_source(ctx, *value) else {
            continue;
        };
        reject_unsupported_mixed_ref_writeback_source(
            &source_ty,
            &ctx.function.name,
            source.slot,
        )?;
        writebacks.push(RefArgWriteback {
            param_index,
            source_value: *value,
            source_slot: source.slot,
            source_ty,
            cell_offset: 0,
        });
    }
    Ok(writebacks)
}

/// Rejects Mixed ref-cell writebacks whose concrete caller slot cannot consume one word safely.
pub(super) fn reject_unsupported_mixed_ref_writeback_source(
    source_ty: &PhpType,
    function_name: &str,
    source_slot: LocalSlotId,
) -> Result<()> {
    if matches!(
        source_ty.codegen_repr(),
        PhpType::Int | PhpType::Bool | PhpType::Void | PhpType::Never
    ) {
        return Ok(());
    }
    if matches!(
        source_ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_)
    ) {
        return Ok(());
    }
    Err(CodegenIrError::unsupported(format!(
        "by-reference Mixed parameter writeback to PHP type {:?} in {} for local slot {:?}",
        source_ty, function_name, source_slot
    )))
}

/// Emits persistent caller-stack Mixed cells used by scalar-to-Mixed by-reference args.
pub(super) fn emit_ref_arg_temp_cells(
    ctx: &mut FunctionContext<'_>,
    writebacks: &mut [RefArgWriteback],
) -> Result<()> {
    let total = writebacks.len();
    for (index, writeback) in writebacks.iter_mut().enumerate() {
        ctx.load_value_to_result(writeback.source_value)?;
        emit_box_current_value_as_mixed(ctx.emitter, &writeback.source_ty);
        abi::emit_push_result_value(ctx.emitter, &PhpType::Mixed);
        writeback.cell_offset = (total - index - 1) * 16;
    }
    Ok(())
}

/// Loads the address that should be passed for a by-reference argument.
pub(super) fn materialize_ref_arg_address(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    param_index: usize,
    param_ty: &PhpType,
    arg_temp_bytes: usize,
    writebacks: &[RefArgWriteback],
    ref_cell_base_offset: usize,
) -> Result<()> {
    if let Some(writeback) = writebacks
        .iter()
        .find(|writeback| writeback.param_index == param_index)
    {
        let cell_offset = arg_temp_bytes + ref_cell_base_offset + writeback.cell_offset;
        abi::emit_temporary_stack_address(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            cell_offset,
        );
        return Ok(());
    }
    if local_ref_arg_source(ctx, value).is_ok() {
        return materialize_local_ref_arg_address(ctx, value);
    }
    if value_is_array_element_address(ctx, value)? {
        ctx.load_value_to_reg(value, abi::int_result_reg(ctx.emitter))?;
        return Ok(());
    }
    materialize_temporary_ref_arg_cell(ctx, value, param_ty)
}

/// Allocates a heap ref-cell for a by-reference argument that is not a local variable.
pub(super) fn materialize_temporary_ref_arg_cell(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    param_ty: &PhpType,
) -> Result<()> {
    let source_ty = ctx.load_value_to_result(value)?;
    let target_ty = param_ty.codegen_repr();
    coerce_ref_cell_store_value(ctx, value, &source_ty, &target_ty)?;
    abi::emit_push_result_value(ctx.emitter, &target_ty);
    abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 16);
    abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
    let cell_reg = abi::symbol_scratch_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    abi::emit_pop_reg(ctx.emitter, cell_reg);
    store_pushed_value_to_ref_cell(ctx, cell_reg, &target_ty);
    move_reg_to_int_result(ctx, cell_reg);
    Ok(())
}

/// Stores the pushed argument value into a freshly allocated by-reference cell.
pub(super) fn store_pushed_value_to_ref_cell(ctx: &mut FunctionContext<'_>, cell_reg: &str, val_ty: &PhpType) {
    let temp_reg = if cell_reg == abi::temp_int_reg(ctx.emitter.target) {
        abi::symbol_scratch_reg(ctx.emitter)
    } else {
        abi::temp_int_reg(ctx.emitter.target)
    };
    match val_ty.codegen_repr() {
        PhpType::Str => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            abi::emit_pop_reg_pair(ctx.emitter, ptr_reg, len_reg);
            abi::emit_store_to_address(ctx.emitter, ptr_reg, cell_reg, 0);
            abi::emit_store_to_address(ctx.emitter, len_reg, cell_reg, 8);
        }
        PhpType::TaggedScalar => {
            let tag_reg = crate::codegen::sentinels::tagged_scalar_tag_reg(ctx.emitter);
            abi::emit_pop_reg_pair(ctx.emitter, abi::int_result_reg(ctx.emitter), tag_reg);
            abi::emit_store_to_address(ctx.emitter, abi::int_result_reg(ctx.emitter), cell_reg, 0);
            abi::emit_store_to_address(ctx.emitter, tag_reg, cell_reg, 8);
        }
        PhpType::Float => {
            abi::emit_pop_float_reg(ctx.emitter, abi::float_result_reg(ctx.emitter));
            abi::emit_store_to_address(
                ctx.emitter,
                abi::float_result_reg(ctx.emitter),
                cell_reg,
                0,
            );
        }
        _ => {
            abi::emit_pop_reg(ctx.emitter, temp_reg);
            abi::emit_store_to_address(ctx.emitter, temp_reg, cell_reg, 0);
            abi::emit_store_zero_to_address(ctx.emitter, cell_reg, 8);
        }
    }
}

/// Writes temporary Mixed by-reference cells back into the original caller locals.
pub(super) fn emit_ref_arg_writebacks(
    ctx: &mut FunctionContext<'_>,
    writebacks: &[RefArgWriteback],
) -> Result<()> {
    for writeback in writebacks {
        abi::emit_load_temporary_stack_slot(
            ctx.emitter,
            abi::int_result_reg(ctx.emitter),
            writeback.cell_offset,
        );
        if ctx.local_php_type(writeback.source_slot)?.codegen_repr() == PhpType::Mixed {
            emit_mixed_ref_writeback_to_gradual_local(ctx, writeback)?;
            continue;
        }
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
        move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
        store_current_scalar_result_to_ref_source(ctx, writeback)?;
        abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
    }
    abi::emit_release_temporary_stack(ctx.emitter, writebacks.len() * 16);
    Ok(())
}

/// Publishes a mutated by-reference Mixed cell back into gradual caller storage.
///
/// The callee may replace the cell's runtime value with a different array kind or scalar, so
/// unboxing it according to the caller's pre-call narrowed type would discard that runtime tag.
/// The local takes its own cell reference instead; the temporary argument owner is then released.
fn emit_mixed_ref_writeback_to_gradual_local(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_incref");
    abi::emit_push_reg(ctx.emitter, result_reg);
    let target_ty = ctx.local_php_type(writeback.source_slot)?.codegen_repr();
    let offset = ctx.local_offset(writeback.source_slot)?;
    super::super::frame::emit_owned_local_cleanup(
        ctx,
        writeback.source_slot,
        offset,
        &target_ty,
    );
    abi::emit_pop_reg(ctx.emitter, result_reg);
    ctx.store_current_result_to_local(writeback.source_slot)?;
    abi::emit_pop_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
    Ok(())
}

/// Returns the low payload register produced by `__rt_mixed_unbox` on the active target.
pub(super) fn mixed_unbox_low_payload_reg(ctx: &FunctionContext<'_>) -> &'static str {
    match ctx.emitter.target.arch {
        Arch::AArch64 => "x1",
        Arch::X86_64 => "rdi",
    }
}

/// Unboxes a boxed Mixed/Union payload and retains it for an owned concrete heap result.
pub(super) fn emit_unbox_mixed_to_owned_refcounted_result(ctx: &mut FunctionContext<'_>, result_ty: &PhpType) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
    abi::emit_incref_if_refcounted(ctx.emitter, result_ty);
}

/// Unboxes a guarded Mixed value into an owned concrete heap representation.
///
/// Flow-sensitive checking proves the value has the requested type before this op is emitted;
/// the runtime helper extracts its payload and this result takes its own reference so retaining
/// stores and later cleanup have a balanced ownership ledger.
pub(super) fn lower_mixed_unbox(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    load_value_to_first_int_arg(ctx, value)?;
    let result_ty = inst.result_php_type.codegen_repr();
    match (&result_ty, &inst.immediate) {
        (PhpType::Object(_), Some(Immediate::Bool(true))) => {
            emit_mixed_tag_unbox_guard(ctx, 6, &|given| {
                format!(
                    "clone(): Argument #1 ($object) must be of type object, {} given",
                    given
                )
            });
        }
        (PhpType::Object(_), Some(Immediate::Data(_))) => {
            emit_mixed_nominal_object_unbox_guard(ctx, inst)?;
        }
        (PhpType::Callable, Some(Immediate::I64(10))) => {
            emit_mixed_tag_unbox_guard(ctx, 10, &|given| {
                format!("Value must be of type callable, {} given", given)
            });
        }
        _ => abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox"),
    }
    move_reg_to_int_result(ctx, mixed_unbox_low_payload_reg(ctx));
    abi::emit_incref_if_refcounted(ctx.emitter, &result_ty);
    store_if_result(ctx, inst)
}

/// Unboxes a dynamic value and raises a catchable type error unless its tag matches.
fn emit_mixed_tag_unbox_guard(
    ctx: &mut FunctionContext<'_>,
    expected_tag: i64,
    message_for: &impl Fn(&str) -> String,
) {
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let accepted_label = ctx.next_label("mixed_unbox_tag_accepted");
    let wrong_label = ctx.next_label("mixed_unbox_wrong_tag");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cmp x0, #{}", expected_tag));     // compare the boxed value's runtime tag with the boundary contract
            ctx.emitter.instruction(&format!("b.eq {}", accepted_label));       // continue only when the runtime representation matches
            ctx.emitter.instruction(&format!("b {}", wrong_label));             // report the dynamic value's concrete runtime type
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("cmp rax, {}", expected_tag));     // compare the boxed value's runtime tag with the boundary contract
            ctx.emitter.instruction(&format!("je {}", accepted_label));         // continue only when the runtime representation matches
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));           // report the dynamic value's concrete runtime type
        }
    }
    builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_label,
        message_for,
    );
    ctx.emitter.label(&accepted_label);
}

/// Validates a boxed object against a named class or interface before unboxing it.
fn emit_mixed_nominal_object_unbox_guard(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let expected = objects::class_name_immediate(ctx, inst)?.to_string();
    if expected.is_empty() || php_symbol_key(&expected) == "object" {
        emit_mixed_tag_unbox_guard(ctx, 6, &|given| {
            format!("Value must be of type object, {} given", given)
        });
        return Ok(());
    }
    let Some((target_id, target_kind)) = objects::classify_named_target(ctx, &expected) else {
        return Err(CodegenIrError::invalid_module(format!(
            "missing runtime type metadata for gradual boundary {:?}",
            expected
        )));
    };
    let source_reg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_push_reg(ctx.emitter, source_reg);
    objects::emit_match_call(ctx, target_id, target_kind, "__rt_mixed_instanceof");
    let accepted_label = ctx.next_label("mixed_unbox_nominal_accepted");
    let wrong_label = ctx.next_label("mixed_unbox_nominal_wrong");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("cbnz x0, {}", accepted_label));   // a true matcher result satisfies the named boundary
            ctx.emitter.instruction(&format!("b {}", wrong_label));             // reject a scalar or unrelated object payload
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("test rax, rax");                           // did the boxed object satisfy the named boundary?
            ctx.emitter.instruction(&format!("jne {}", accepted_label));        // continue when class or interface matching succeeded
            ctx.emitter.instruction(&format!("jmp {}", wrong_label));           // reject a scalar or unrelated object payload
        }
    }
    ctx.emitter.label(&wrong_label);
    abi::emit_pop_reg(ctx.emitter, source_reg);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let type_error_label = ctx.next_label("mixed_unbox_nominal_type_error");
    builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &type_error_label,
        &|given| format!("Value must be of type {}, {} given", expected, given),
    );
    ctx.emitter.label(&accepted_label);
    abi::emit_pop_reg(ctx.emitter, source_reg);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    Ok(())
}

/// Stores an unboxed scalar Mixed payload back through the original by-reference source.
pub(super) fn store_current_scalar_result_to_ref_source(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    if matches!(
        writeback.source_ty.codegen_repr(),
        PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Object(_)
    ) {
        return emit_heap_ref_writeback_store(ctx, writeback);
    }
    ctx.store_current_result_to_local(writeback.source_slot)
}

/// Retains a heap payload before releasing the previous caller-slot owner, then stores it.
///
/// Retaining first is required for alias-safe replacement: the old container graph may already
/// reach the incoming payload, so releasing it first could destroy the value being assigned.
fn emit_heap_ref_writeback_store(
    ctx: &mut FunctionContext<'_>,
    writeback: &RefArgWriteback,
) -> Result<()> {
    let result_reg = abi::int_result_reg(ctx.emitter);
    let slot_ty = ctx.local_php_type(writeback.source_slot)?.codegen_repr();
    let offset = ctx.local_offset(writeback.source_slot)?;
    abi::emit_push_reg(ctx.emitter, result_reg);
    abi::emit_call_label(ctx.emitter, "__rt_incref");
    super::super::frame::emit_owned_local_cleanup(
        ctx,
        writeback.source_slot,
        offset,
        &slot_ty,
    );
    abi::emit_pop_reg(ctx.emitter, result_reg);
    ctx.store_current_result_to_local(writeback.source_slot)
}

/// Loads a local variable's address for a by-reference method-call argument.
pub(super) fn materialize_local_ref_arg_address(ctx: &mut FunctionContext<'_>, value: ValueId) -> Result<()> {
    let source = local_ref_arg_source(ctx, value)?;
    ctx.materialize_local_storage_address(source.slot, abi::int_result_reg(ctx.emitter))
}

/// Returns true when a value already holds a direct pointer to an array element slot.
pub(super) fn value_is_array_element_address(ctx: &FunctionContext<'_>, value: ValueId) -> Result<bool> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Ok(false);
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    Ok(inst_ref.op == Op::ArrayElemAddr)
}

/// Describes a local operand used as a by-reference call argument.
struct LocalRefArgSource {
    slot: LocalSlotId,
}

/// Resolves an EIR value back to a local slot and whether it already stores a ref-cell pointer.
fn local_ref_arg_source(ctx: &FunctionContext<'_>, value: ValueId) -> Result<LocalRefArgSource> {
    let Some(value_ref) = ctx.function.value(value) else {
        return Err(CodegenIrError::missing_entry("value", value.as_raw()));
    };
    let ValueDef::Instruction { inst, .. } = value_ref.def else {
        return Err(CodegenIrError::unsupported(
            "by-reference method call argument from non-local value",
        ));
    };
    let inst_ref = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    match inst_ref.op {
        Op::LoadLocal | Op::LoadRefCell => {}
        _ => {
            return Err(CodegenIrError::unsupported(format!(
                "by-reference method call argument from opcode {}",
                inst_ref.op.name()
            )))
        }
    };
    let Some(Immediate::LocalSlot(slot)) = inst_ref.immediate else {
        return Err(CodegenIrError::invalid_module(
            "by-reference load argument has no local slot",
        ));
    };
    Ok(LocalRefArgSource { slot })
}

/// Resolves an EIR value back to a `load_local` source slot for by-reference calls.
pub(super) fn local_slot_for_loaded_value(ctx: &FunctionContext<'_>, value: ValueId) -> Result<LocalSlotId> {
    local_ref_arg_source(ctx, value).map(|source| source.slot)
}

/// Returns true when a local slot stores a ref-cell pointer instead of a raw value.
pub(super) fn local_slot_stores_ref_cell_pointer(ctx: &FunctionContext<'_>, slot: LocalSlotId) -> bool {
    ctx.local_stores_ref_cell_pointer(slot)
}

/// Moves a scratch integer register into the canonical integer result register.
pub(super) fn move_reg_to_int_result(ctx: &mut FunctionContext<'_>, source_reg: &str) {
    let result_reg = abi::int_result_reg(ctx.emitter);
    if source_reg == result_reg {
        return;
    }
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("mov {}, {}", result_reg, source_reg)); // move the unboxed receiver pointer into the normal argument staging register
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov {}, {}", result_reg, source_reg)); // move the unboxed receiver pointer into the normal argument staging register
        }
    }
}
