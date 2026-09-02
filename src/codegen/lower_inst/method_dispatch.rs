//! Purpose:
//! Lowers direct and boxed-Mixed instance method dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::lower_instruction()` and sibling lowering helpers.
//!
//! Key details:
//! - Preserves EIR ownership, ABI ordering, runtime symbols, and target-aware lowering.

use super::*;
use crate::parser::ast::ExprKind;

/// Lowers a direct instance-method call on a statically known object receiver.
pub(super) fn lower_method_call(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let object = expect_operand(inst, 0)?;
    let method_name = method_name_data(ctx, inst)?.to_string();
    if let Some((class_name, true)) = objects::nullable_object_receiver_class(ctx, object)? {
        return lower_nullable_receiver_method_call(ctx, inst, object, &class_name, &method_name);
    }
    let object_ty = ctx.value_php_type(object)?.codegen_repr();
    if matches!(object_ty, PhpType::Mixed | PhpType::Union(_)) {
        if let Some(state) = fiber_state_predicate_method(&method_name) {
            return lower_mixed_fiber_state_predicate(ctx, inst, object, &method_name, state);
        }
        return lower_mixed_method_call(ctx, inst, object, &method_name);
    }
    let PhpType::Object(class_name) = object_ty else {
        return Err(CodegenIrError::unsupported(format!(
            "method call receiver for PHP type {:?}",
            object_ty
        )));
    };
    guard_static_method_receiver(ctx, object, &method_name)?;
    if builtins::has_eval_context(ctx)
        && (reflection_function_callable_metadata_method(&class_name, &method_name)
            || reflection_class_runtime_metadata_method(&class_name))
    {
        return builtins::lower_eval_method_call(ctx, inst, object, &method_name);
    }
    if let Some(state) = fiber_state_predicate(&class_name, &method_name) {
        return lower_fiber_state_predicate(ctx, inst, object, state);
    }
    if let Some(intrinsic) = generator_intrinsic(&class_name, &method_name) {
        return lower_generator_intrinsic(ctx, inst, intrinsic);
    }
    if let Some(intrinsic) = callback_filter_intrinsic(&class_name, &method_name) {
        return lower_callback_filter_accept_intrinsic(ctx, inst, intrinsic);
    }
    if is_fiber_start_call(&class_name, &method_name) {
        return lower_fiber_start(ctx, inst, object);
    }
    if is_fiber_resume_call(&class_name, &method_name) {
        return lower_fiber_resume(ctx, inst, object);
    }
    if is_fiber_throw_call(&class_name, &method_name) {
        return lower_fiber_throw(ctx, inst, object);
    }
    if is_fiber_get_return_call(&class_name, &method_name) {
        return lower_fiber_noarg_runtime_method(ctx, inst, object, "__rt_fiber_get_return");
    }
    if let Some(intrinsic) = runtime_backed_instance_intrinsic(&class_name, &method_name) {
        return lower_instance_runtime_intrinsic(ctx, inst, &class_name, &method_name, intrinsic);
    }
    if is_throwable_standard_method_call(ctx, &class_name, &method_name) {
        return lower_throwable_standard_method(ctx, inst, object, &method_name);
    }
    if ctx
        .module
        .interface_infos
        .contains_key(class_name.trim_start_matches('\\'))
    {
        return lower_interface_method_call(ctx, inst, &class_name, &method_name);
    }
    let normalized_class = class_name.trim_start_matches('\\');
    if normalized_class.is_empty() {
        return lower_narrowed_interface_method_call(ctx, inst, "", &method_name);
    }
    if !ctx.module.class_infos.contains_key(normalized_class)
        && !ctx.module.interface_infos.contains_key(normalized_class)
        && !ctx.module.extern_class_infos.contains_key(normalized_class)
        && !ctx.module.packed_class_infos.contains_key(normalized_class)
    {
        exceptions::emit_error(ctx, &format!("Class \"{}\" not found", normalized_class));
        return Ok(());
    }
    if !class_declares_method(ctx, &class_name, &method_name)
        && !narrowed_interface_candidates(ctx, normalized_class, &method_name, inst.operands.len())?
            .is_empty()
    {
        return lower_narrowed_interface_method_call(
            ctx,
            inst,
            normalized_class,
            &method_name,
        );
    }
    let owned_dynamic_done_label = if builtins::has_eval_context(ctx) {
        let native_label = ctx.next_label("typed_method_native_dispatch");
        let done_label = ctx.next_label("typed_method_dynamic_dispatch_done");
        builtins::lower_eval_owned_method_call(
            ctx,
            inst,
            object,
            &method_name,
            &native_label,
            &done_label,
        )?;
        ctx.emitter.label(&native_label);
        Some(done_label)
    } else {
        None
    };
    let target = resolve_method_call_target(ctx, &class_name, &method_name, inst.operands.len())?;
    let mut param_types = Vec::with_capacity(target.params.len() + 1);
    param_types.push(PhpType::Object(class_name));
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
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    if let Some(slot) = target.dynamic_slot {
        emit_dynamic_instance_method_call(ctx, slot);
    } else {
        abi::emit_call_label(
            ctx.emitter,
            &method_symbol(&target.impl_class, &target.method_key),
        );
    }
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    store_method_call_result(ctx, inst, &target)?;
    emit_call_arg_temp_cleanups(ctx, &call_args, inst.result)?;
    emit_ref_arg_writebacks(ctx, &call_args.ref_writebacks)?;
    if let Some(done_label) = owned_dynamic_done_label {
        abi::emit_jump(ctx.emitter, &done_label);
        ctx.emitter.label(&done_label);
    }
    Ok(())
}

/// Returns whether a ReflectionFunction method depends on retained callable metadata.
fn reflection_function_callable_metadata_method(class_name: &str, method_name: &str) -> bool {
    class_name.trim_start_matches('\\') == "ReflectionFunction"
        && matches!(
            php_symbol_key(method_name).as_str(),
            "isanonymous"
                | "isstatic"
                | "isclosure"
                | "getclosurethis"
                | "getclosurescopeclass"
                | "getclosurecalledclass"
                | "getclosureusedvariables"
        )
}

/// Returns true when runtime evaluation must own a reflection metadata method call.
fn reflection_class_runtime_metadata_method(class_name: &str) -> bool {
    matches!(
        class_name.trim_start_matches('\\'),
        "ReflectionAttribute" | "ReflectionClass" | "ReflectionObject" | "ReflectionEnum"
    )
}

/// Rejects the raw null-container representation before a static object method dispatch.
pub(super) fn guard_static_method_receiver(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    method_name: &str,
) -> Result<()> {
    let receiver_reg = abi::symbol_scratch_reg(ctx.emitter);
    let scratch_reg = abi::secondary_scratch_reg(ctx.emitter);
    let null_label = ctx.next_label("static_method_receiver_null");
    let done_label = ctx.next_label("static_method_receiver_checked");
    ctx.load_value_to_reg(object, receiver_reg)?;
    crate::codegen::sentinels::emit_branch_if_null_container(
        ctx.emitter,
        receiver_reg,
        scratch_reg,
        &null_label,
    );
    abi::emit_jump(ctx.emitter, &done_label);
    ctx.emitter.label(&null_label);
    emit_aot_raw_null_method_receiver_trace(ctx, object, method_name)?;
    emit_method_call_on_null_fatal(ctx, method_name);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Emits the raw receiver word for a statically dispatched null-method fatal.
///
/// The value is only observed on the error path and is passed after the ordinary
/// diagnostic fields, so it cannot affect ABI argument materialization or a
/// successful method call.
fn emit_aot_raw_null_method_receiver_trace(
    ctx: &mut FunctionContext<'_>,
    object: ValueId,
    method_name: &str,
) -> Result<()> {
    if !ctx.module.required_runtime_features.eval_bridge {
        return Ok(());
    }
    let (function_label, function_len) = ctx.data.add_string(ctx.function.name.as_bytes());
    let (method_label, method_len) = ctx.data.add_string(method_name.as_bytes());
    let function_ptr_arg = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_symbol_address(ctx.emitter, function_ptr_arg, &function_label);
    let function_len_arg = abi::int_arg_reg_name(ctx.emitter.target, 1);
    abi::emit_load_int_immediate(ctx.emitter, function_len_arg, function_len as i64);
    let method_ptr_arg = abi::int_arg_reg_name(ctx.emitter.target, 2);
    abi::emit_symbol_address(ctx.emitter, method_ptr_arg, &method_label);
    let method_len_arg = abi::int_arg_reg_name(ctx.emitter.target, 3);
    abi::emit_load_int_immediate(ctx.emitter, method_len_arg, method_len as i64);
    let line_arg = abi::int_arg_reg_name(ctx.emitter.target, 4);
    let line = ctx
        .current_instruction_span()
        .map(|span| span.line as i64)
        .unwrap_or_default();
    abi::emit_load_int_immediate(ctx.emitter, line_arg, line);
    let receiver_arg = abi::int_arg_reg_name(ctx.emitter.target, 5);
    ctx.load_value_to_reg(object, receiver_arg)?;
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_trace_aot_raw_null_method_receiver");
    abi::emit_call_label(ctx.emitter, &symbol);
    Ok(())
}

/// Lowers an instance-method call whose receiver is boxed as `Mixed`.
pub(super) fn lower_mixed_method_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    method_name: &str,
) -> Result<()> {
    let candidates = mixed_method_candidates(ctx, method_name, inst.operands.len())?;
    if candidates.is_empty() {
        if builtins::has_eval_context(ctx) {
            return builtins::lower_eval_method_call(ctx, inst, object, method_name);
        }
        emit_method_call_on_null_fatal(ctx, method_name);
        return Ok(());
    }
    if builtins::has_eval_context(ctx)
        && (mixed_method_call_needs_eval_callable_adapter(ctx, inst, &candidates)?
            || mixed_method_call_needs_eval_ref_adapter(ctx, inst, &candidates)?)
    {
        return builtins::lower_eval_method_call(ctx, inst, object, method_name);
    }

    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    let non_object_label = ctx.next_label("mixed_method_non_object");
    let no_match_label = ctx.next_label("mixed_method_no_match");
    let done_label = ctx.next_label("mixed_method_done");
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "mixed_method_{}",
                label_fragment(&candidate.class_name)
            ))
        })
        .collect::<Vec<_>>();

    ctx.load_value_to_result(object)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    emit_mixed_method_object_payload_or_fatal(ctx, receiver_reg, &non_object_label);
    emit_mixed_method_class_dispatch(
        ctx,
        receiver_reg,
        &candidates,
        &match_labels,
        &no_match_label,
    );

    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        lower_mixed_method_candidate_call(ctx, inst, receiver_reg, candidate, method_name)?;
        abi::emit_jump(ctx.emitter, &done_label);
    }

    ctx.emitter.label(&no_match_label);
    if builtins::has_eval_context(ctx) {
        builtins::lower_eval_method_call(ctx, inst, object, method_name)?;
        abi::emit_jump(ctx.emitter, &done_label);
    } else {
        emit_method_call_on_null_fatal(ctx, method_name);
    }

    ctx.emitter.label(&non_object_label);
    emit_method_call_on_null_fatal(ctx, method_name);

    ctx.emitter.label(&done_label);
    Ok(())
}

/// Returns whether a gradual callable argument needs the eval context carried by its object cell.
fn mixed_method_call_needs_eval_callable_adapter(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
    candidates: &[MixedMethodCandidate],
) -> Result<bool> {
    for (operand_index, operand) in inst.operands.iter().enumerate().skip(1) {
        let source_ty = ctx.raw_value_php_type(*operand)?.codegen_repr();
        if !matches!(
            source_ty,
            PhpType::Mixed | PhpType::Union(_) | PhpType::Object(_)
        ) {
            continue;
        }
        let param_index = operand_index - 1;
        if candidates.iter().any(|candidate| {
            candidate
                .target
                .params
                .get(param_index)
                .is_some_and(|param| param.codegen_repr() == PhpType::Callable)
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns whether direct dispatch would pass a promoted Mixed reference cell to a typed ref ABI.
fn mixed_method_call_needs_eval_ref_adapter(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
    candidates: &[MixedMethodCandidate],
) -> Result<bool> {
    for (operand_index, operand) in inst.operands.iter().enumerate().skip(1) {
        let Ok(slot) = local_slot_for_loaded_value(ctx, *operand) else {
            continue;
        };
        if !local_slot_stores_ref_cell_pointer(ctx, slot)
            && ctx.local_php_type(slot)?.codegen_repr() != PhpType::Mixed
        {
            continue;
        }
        let param_index = operand_index - 1;
        if candidates.iter().any(|candidate| {
            candidate
                .target
                .ref_params
                .get(param_index)
                .copied()
                .unwrap_or(false)
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Emits one concrete class branch for a `Mixed` receiver method call.
pub(super) fn lower_mixed_method_candidate_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    receiver_reg: &str,
    candidate: &MixedMethodCandidate,
    method_name: &str,
) -> Result<()> {
    // Built-in Throwables implement the standard Throwable surface through compact intrinsics,
    // not through class vtable slots — those slots stay empty for builtins. Dispatching this
    // candidate dynamically would load a null slot and branch to it, so route it to the same
    // intrinsic the direct-receiver path uses. The receiver payload is already unboxed in
    // `receiver_reg`, which is exactly what `_from_reg` expects.
    if is_throwable_standard_method_call(ctx, &candidate.class_name, method_name) {
        return lower_throwable_standard_method_from_reg(ctx, inst, receiver_reg, method_name);
    }
    let receiver_ty = PhpType::Object(candidate.class_name.clone());
    let mut param_types = Vec::with_capacity(candidate.target.params.len() + 1);
    param_types.push(receiver_ty.clone());
    param_types.extend(
        candidate
            .target
            .params
            .iter()
            .map(|param| param.codegen_repr()),
    );
    let mut ref_params = Vec::with_capacity(candidate.target.ref_params.len() + 1);
    ref_params.push(false);
    ref_params.extend(candidate.target.ref_params.iter().copied());
    guard_mixed_method_candidate_object_arguments(ctx, inst, candidate)?;
    let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx,
        receiver_reg,
        &receiver_ty,
        &inst.operands,
        &param_types,
        &ref_params,
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    if let Some(slot) = candidate.target.dynamic_slot {
        emit_dynamic_instance_method_call(ctx, slot);
    } else {
        abi::emit_call_label(
            ctx.emitter,
            &method_symbol(&candidate.target.impl_class, &candidate.target.method_key),
        );
    }
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    store_method_call_result(ctx, inst, &candidate.target)?;
    emit_call_arg_temp_cleanups(ctx, &call_args, inst.result)?;
    emit_ref_arg_writebacks(ctx, &call_args.ref_writebacks)
}

/// Validates boxed arguments against the concrete candidate's nominal object parameters.
///
/// A `Mixed` receiver has no single signature during IR lowering, so its arguments cannot be
/// narrowed before runtime class dispatch chooses a candidate. This guard supplies the missing
/// candidate-specific PHP check before ABI materialization exposes an object payload.
fn guard_mixed_method_candidate_object_arguments(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    candidate: &MixedMethodCandidate,
) -> Result<()> {
    for (param_index, value) in inst.operands.iter().skip(1).enumerate() {
        let Some(param_ty) = candidate.target.params.get(param_index) else {
            continue;
        };
        let PhpType::Object(target_name) = param_ty.codegen_repr() else {
            continue;
        };
        let source_ty = ctx.raw_value_php_type(*value)?.codegen_repr();
        if !matches!(source_ty, PhpType::Mixed | PhpType::Union(_)) {
            continue;
        }
        emit_mixed_method_candidate_object_argument_guard(ctx, *value, &target_name)?;
    }
    Ok(())
}

/// Rejects a non-matching boxed `Mixed` value before it crosses an object parameter ABI.
fn emit_mixed_method_candidate_object_argument_guard(
    ctx: &mut FunctionContext<'_>,
    value: ValueId,
    target_name: &str,
) -> Result<()> {
    let target_name = target_name.trim_start_matches('\\');
    let object_label = ctx.next_label("mixed_method_object_argument_object");
    let wrong_tag_label = ctx.next_label("mixed_method_object_argument_wrong_tag");
    let accepted_label = ctx.next_label("mixed_method_object_argument_accepted");
    ctx.load_value_to_result(value)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                            // runtime tag 6 is the only object ABI payload
            ctx.emitter.instruction(&format!("b.eq {}", object_label));
            ctx.emitter.instruction(&format!("b {}", wrong_tag_label));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                            // runtime tag 6 is the only object ABI payload
            ctx.emitter.instruction(&format!("je {}", object_label));
            ctx.emitter.instruction(&format!("jmp {}", wrong_tag_label));
        }
    }
    super::builtins::arrays::union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(
        ctx,
        &wrong_tag_label,
        &|given| format!("Argument must be of type {}, {} given", target_name, given),
    );

    ctx.emitter.label(&object_label);
    if let Some((target_id, target_kind)) = objects::classify_named_target(ctx, target_name) {
        match ctx.emitter.target.arch {
            Arch::AArch64 => ctx.emitter.instruction("mov x0, x1"),           // matcher takes the unboxed object payload in its first argument register
            Arch::X86_64 => {}                                                 // the unboxed object payload already uses the first argument register
        }
        objects::emit_match_call(ctx, target_id, target_kind, "__rt_exception_matches");
        match ctx.emitter.target.arch {
            Arch::AArch64 => {
                ctx.emitter.instruction(&format!("cbnz x0, {}", accepted_label));
            }
            Arch::X86_64 => {
                ctx.emitter.instruction("test rax, rax");
                ctx.emitter.instruction(&format!("jne {}", accepted_label));
            }
        }
    }
    exceptions::emit_type_error(
        ctx,
        &format!("Argument must be of type {}, object given", target_name),
    );
    ctx.emitter.label(&accepted_label);
    Ok(())
}

/// Lowers an interface receiver call accepted through an `instanceof` capability narrowing.
pub(super) fn lower_narrowed_interface_method_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    interface_name: &str,
    method_name: &str,
) -> Result<()> {
    lower_narrowed_interface_method_call_with_failure(
        ctx,
        inst,
        interface_name,
        method_name,
        None,
    )
}

/// Lowers an interface receiver call and raises the supplied TypeError when no implementation
/// matches the runtime class.
pub(super) fn lower_narrowed_interface_method_call_or_type_error(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    interface_name: &str,
    method_name: &str,
    type_error: &str,
) -> Result<()> {
    lower_narrowed_interface_method_call_with_failure(
        ctx,
        inst,
        interface_name,
        method_name,
        Some(type_error),
    )
}

/// Implements raw interface dispatch with either the ordinary missing-method fatal or a
/// caller-selected TypeError at the no-match boundary.
fn lower_narrowed_interface_method_call_with_failure(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    interface_name: &str,
    method_name: &str,
    type_error: Option<&str>,
) -> Result<()> {
    let candidates =
        narrowed_interface_candidates(ctx, interface_name, method_name, inst.operands.len())?;
    if candidates.is_empty() {
        if let Some(message) = type_error {
            exceptions::emit_type_error(ctx, message);
        } else {
            emit_method_call_on_null_fatal(ctx, method_name);
        }
        return Ok(());
    }
    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    let no_match_label = ctx.next_label("iface_narrowed_no_match");
    let done_label = ctx.next_label("iface_narrowed_done");
    ctx.load_value_to_result(inst.operands[0])?;
    emit_bare_object_receiver_into_reg(ctx, receiver_reg);
    emit_narrowed_interface_class_dispatch(
        ctx,
        inst,
        receiver_reg,
        &candidates,
        method_name,
        &no_match_label,
        &done_label,
    )?;

    ctx.emitter.label(&no_match_label);
    if let Some(message) = type_error {
        exceptions::emit_type_error(ctx, message);
    } else {
        emit_method_call_on_null_fatal(ctx, method_name);
    }
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Lowers the nullable form of an interface call accepted through capability narrowing.
pub(super) fn lower_narrowed_nullable_interface_method_call(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    object: ValueId,
    interface_name: &str,
    method_name: &str,
) -> Result<()> {
    let candidates =
        narrowed_interface_candidates(ctx, interface_name, method_name, inst.operands.len())?;
    if candidates.is_empty() {
        emit_method_call_on_null_fatal(ctx, method_name);
        return Ok(());
    }
    let receiver_reg = abi::nested_call_reg(ctx.emitter);
    let null_label = ctx.next_label("iface_narrowed_null");
    let no_match_label = ctx.next_label("iface_narrowed_no_match");
    let done_label = ctx.next_label("iface_narrowed_done");
    objects::emit_nullable_receiver_object_payload(ctx, object, &null_label, receiver_reg)?;
    emit_narrowed_interface_class_dispatch(
        ctx,
        inst,
        receiver_reg,
        &candidates,
        method_name,
        &no_match_label,
        &done_label,
    )?;

    ctx.emitter.label(&no_match_label);
    emit_method_call_on_null_fatal(ctx, method_name);
    ctx.emitter.label(&null_label);
    emit_method_call_on_null_fatal(ctx, method_name);
    ctx.emitter.label(&done_label);
    Ok(())
}

/// Collects concrete runtime classes compatible with the receiver type and requested method.
pub(super) fn narrowed_interface_candidates(
    ctx: &FunctionContext<'_>,
    receiver_type: &str,
    method_name: &str,
    operand_count: usize,
) -> Result<Vec<MixedMethodCandidate>> {
    mixed_method_candidates(ctx, method_name, operand_count).map(|candidates| {
        candidates
            .into_iter()
            .filter(|candidate| {
                receiver_type.is_empty()
                    || class_is_same_or_descendant(ctx, &candidate.class_name, receiver_type)
                    || class_implements_interface(ctx, &candidate.class_name, receiver_type)
            })
            .collect()
    })
}

/// Returns whether a concrete runtime class is the named class or inherits from it.
fn class_is_same_or_descendant(
    ctx: &FunctionContext<'_>,
    class_name: &str,
    ancestor_name: &str,
) -> bool {
    let ancestor_key = php_symbol_key(ancestor_name.trim_start_matches('\\'));
    let mut current = Some(class_name.trim_start_matches('\\'));
    while let Some(candidate) = current {
        if php_symbol_key(candidate) == ancestor_key {
            return true;
        }
        current = ctx
            .module
            .class_infos
            .get(candidate)
            .and_then(|class_info| class_info.parent.as_deref());
    }
    false
}

/// Emits class-id branches and concrete calls for a narrowed interface receiver.
fn emit_narrowed_interface_class_dispatch(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
    receiver_reg: &str,
    candidates: &[MixedMethodCandidate],
    method_name: &str,
    no_match_label: &str,
    done_label: &str,
) -> Result<()> {
    let match_labels = candidates
        .iter()
        .map(|candidate| {
            ctx.next_label(&format!(
                "iface_narrowed_{}",
                label_fragment(&candidate.class_name)
            ))
        })
        .collect::<Vec<_>>();
    emit_mixed_method_class_dispatch(ctx, receiver_reg, candidates, &match_labels, no_match_label);
    for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
        ctx.emitter.label(label);
        lower_mixed_method_candidate_call(ctx, inst, receiver_reg, candidate, method_name)?;
        abi::emit_jump(ctx.emitter, done_label);
    }
    Ok(())
}

/// Moves a bare object result into the register reserved for runtime class dispatch.
fn emit_bare_object_receiver_into_reg(ctx: &mut FunctionContext<'_>, receiver_reg: &str) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction(&format!("mov {}, x0", receiver_reg));      // stage the bare interface object pointer for dispatch
        }
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("mov {}, rax", receiver_reg));     // stage the bare interface object pointer for dispatch
        }
    }
}

/// Collects concrete class-method candidates for a boxed `Mixed` receiver.
pub(super) fn mixed_method_candidates(
    ctx: &FunctionContext<'_>,
    method_name: &str,
    operand_count: usize,
) -> Result<Vec<MixedMethodCandidate>> {
    let method_key = php_symbol_key(method_name);
    let mut candidates = Vec::new();
    for (class_name, class_info) in &ctx.module.class_infos {
        // Compiler-runtime classes expose synthetic checker declarations whose
        // methods are lowered by dedicated intrinsics and have no ordinary EIR
        // method entry symbol. They must never enter generic Mixed dispatch.
        if php_symbol_key(class_name.trim_start_matches('\\')) == "fiber" {
            continue;
        }
        let Some(signature) = class_info.methods.get(&method_key) else {
            continue;
        };
        let provided = operand_count.saturating_sub(1);
        if provided > signature.params.len()
            || signature.defaults[provided..]
                .iter()
                .any(|default| !matches!(default.as_ref().map(|expr| &expr.kind), Some(ExprKind::Null)))
        {
            continue;
        }
        let target = resolve_method_call_target(
            ctx,
            class_name,
            method_name,
            signature.params.len() + 1,
        )?;
        candidates.push(MixedMethodCandidate {
            class_id: class_info.class_id,
            class_name: class_name.clone(),
            target,
        });
    }
    candidates.sort_by_key(|candidate| candidate.class_id);
    Ok(candidates)
}

/// Preserves the unboxed object payload or routes non-object `Mixed` receivers to fatal.
pub(super) fn emit_mixed_method_object_payload_or_fatal(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    no_match_label: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // require an object payload before method dispatch
            ctx.emitter.instruction(&format!("b.ne {}", no_match_label));       // non-object Mixed receivers cannot call instance methods
            ctx.emitter
                .instruction(&format!("mov {}, x1", receiver_reg)); // preserve the unboxed object payload across argument lowering
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // require an object payload before method dispatch
            ctx.emitter.instruction(&format!("jne {}", no_match_label));        // non-object Mixed receivers cannot call instance methods
            ctx.emitter
                .instruction(&format!("mov {}, rdi", receiver_reg)); // preserve the unboxed object payload across argument lowering
        }
    }
}

/// Emits class-id branches for every method candidate discovered for a `Mixed` receiver.
pub(super) fn emit_mixed_method_class_dispatch(
    ctx: &mut FunctionContext<'_>,
    receiver_reg: &str,
    candidates: &[MixedMethodCandidate],
    match_labels: &[String],
    no_match_label: &str,
) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter
                .instruction(&format!("ldr x9, [{}]", receiver_reg)); // load the receiver class id for Mixed method dispatch
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "x10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp x9, x10");                         // compare the receiver class id against this method candidate
                ctx.emitter.instruction(&format!("b.eq {}", label));            // call this candidate when the runtime class id matches
            }
        }
        Arch::X86_64 => {
            ctx.emitter
                .instruction(&format!("mov r11, QWORD PTR [{}]", receiver_reg)); // load the receiver class id for Mixed method dispatch
            for (candidate, label) in candidates.iter().zip(match_labels.iter()) {
                abi::emit_load_int_immediate(ctx.emitter, "r10", candidate.class_id as i64);
                ctx.emitter.instruction("cmp r11, r10");                        // compare the receiver class id against this method candidate
                ctx.emitter.instruction(&format!("je {}", label));              // call this candidate when the runtime class id matches
            }
        }
    }
    abi::emit_jump(ctx.emitter, no_match_label);
}

/// Re-exports the shared label fragmenter so instruction lowering keeps one implementation.
///
/// `crate::names::label_fragment` is documented as deliberately NON-injective — every
/// non-alphanumeric byte collapses to `_`, so `a_b` and `aéb` collide. A second copy here
/// invited use where uniqueness matters; there is now one definition carrying that warning.
pub(super) use crate::names::label_fragment;
