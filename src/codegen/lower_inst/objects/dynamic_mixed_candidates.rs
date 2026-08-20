//! Purpose:
//! Dispatches Mixed class names across statically known dynamic-new candidates.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Candidate matching, constructor calls, and runtime fallback preserve scratch state.

use super::*;

/// Materializes the dynamic class name as a string result pair, branching for non-string Mixed.
pub(super) fn emit_generic_dynamic_new_class_string(
    ctx: &mut FunctionContext<'_>,
    class_name_value: ValueId,
    non_string_label: &str,
) -> Result<bool> {
    let class_ty = ctx.value_php_type(class_name_value)?.codegen_repr();
    match class_ty {
        PhpType::Str => {
            let (ptr_reg, len_reg) = abi::string_result_regs(ctx.emitter);
            ctx.load_string_value_to_regs(class_name_value, ptr_reg, len_reg)?;
            Ok(true)
        }
        PhpType::Mixed | PhpType::Union(_) => {
            load_value_to_first_int_arg(ctx, class_name_value)?;
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction("cmp x0, #1");                      // require a boxed string class name for dynamic construction
                    ctx.emitter
                        .instruction(&format!("b.ne {}", non_string_label)); // non-string class names produce the runtime null fallback
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("cmp rax, 1");                      // require a boxed string class name for dynamic construction
                    ctx.emitter
                        .instruction(&format!("jne {}", non_string_label)); // non-string class names produce the runtime null fallback
                    ctx.emitter.instruction("mov rax, rdi");                    // move the unboxed string pointer into the string result register
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Returns AOT dynamic-new candidates in stable class-id order.
pub(super) fn dynamic_new_mixed_candidates(
    ctx: &FunctionContext<'_>,
    arg_count: Option<usize>,
    inst: &Instruction,
) -> Result<Vec<DynamicNewCandidate>> {
    let mut candidates = Vec::new();
    let mut sorted_classes = ctx.module.class_infos.iter().collect::<Vec<_>>();
    sorted_classes.sort_by_key(|(_, class_info)| class_info.class_id);
    for (class_name, class_info) in sorted_classes {
        if !is_dynamic_new_mixed_aot_candidate(class_name) {
            continue;
        }
        if let Some(candidate) =
            dynamic_new_candidate(ctx, class_name, class_info, arg_count, inst)?
        {
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}

/// Returns AOT candidates that can be allocated without constructor dispatch.
pub(super) fn dynamic_new_without_constructor_mixed_candidates(
    ctx: &FunctionContext<'_>,
    inst: &Instruction,
) -> Result<Vec<DynamicNewCandidate>> {
    let mut candidates = Vec::new();
    let mut sorted_classes = ctx.module.class_infos.iter().collect::<Vec<_>>();
    sorted_classes.sort_by_key(|(_, class_info)| class_info.class_id);
    for (class_name, class_info) in sorted_classes {
        if !is_dynamic_new_mixed_aot_candidate(class_name) {
            continue;
        }
        if let Some(candidate) =
            dynamic_new_without_constructor_candidate(ctx, class_name, class_info, inst)?
        {
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}

/// Returns true when a class can safely use the static allocation path for `new $name`.
pub(super) fn is_dynamic_new_mixed_aot_candidate(class_name: &str) -> bool {
    if class_name.starts_with("__Elephc") {
        return false;
    }
    if supported_dynamic_new_builtin_class_names().contains(&class_name) {
        return true;
    }
    !known_dynamic_new_builtin_class_names().contains(&class_name)
}

/// Builtin class names the MIXED dynamic-`new` path can allocate ahead of time.
///
/// NOT the same question as `codegen_support::dynamic_new::supported_dynamic_new_builtin_class_names`,
/// despite the near-identical name: that list is what a `new $c` can construct at all, and this
/// one is the subset whose allocation this path can emit statically. Widening this to the other
/// list makes `new $c("stdClass")` with `ReflectionClass` fail to compile —
/// `unsupported EIR backend feature: dynamic_object_new_mixed for default value of property
/// $__constants with PHP type Mixed` — so the difference is load-bearing, not drift.
pub(super) fn supported_dynamic_new_builtin_class_names() -> &'static [&'static str] {
    &[
        "ArgumentCountError",
        "ArrayIterator",
        "ArrayObject",
        "AssertionError",
        "BadFunctionCallException",
        "BadMethodCallException",
        "CallbackFilterIterator",
        "DivisionByZeroError",
        "DomainException",
        "Error",
        "ArithmeticError",
        "Exception",
        "Fiber",
        "FiberError",
        "InvalidArgumentException",
        "IteratorIterator",
        "JsonException",
        "LengthException",
        "LogicException",
        "OutOfBoundsException",
        "OutOfRangeException",
        "OverflowException",
        "RangeException",
        "RecursiveCallbackFilterIterator",
        "ReflectionException",
        "RuntimeException",
        "SplDoublyLinkedList",
        "SplFixedArray",
        "SplQueue",
        "SplStack",
        "TypeError",
        "UnderflowException",
        "UnexpectedValueException",
        "ValueError",
        "ArithmeticError",
        "stdClass",
    ]
}

/// Builtin class names that must not be mistaken for user-instantiable classes.
pub(super) fn known_dynamic_new_builtin_class_names() -> &'static [&'static str] {
    &[
        "AppendIterator",
        "ArgumentCountError",
        "ArrayIterator",
        "ArrayObject",
        "AssertionError",
        "BadFunctionCallException",
        "BadMethodCallException",
        "CachingIterator",
        "CallbackFilterIterator",
        "DirectoryIterator",
        "DivisionByZeroError",
        "DomainException",
        "EmptyIterator",
        "Error",
        "Exception",
        "Fiber",
        "FiberError",
        "FilesystemIterator",
        "FilterIterator",
        "Generator",
        "GlobIterator",
        "InfiniteIterator",
        "InternalIterator",
        "InvalidArgumentException",
        "IteratorIterator",
        "JsonException",
        "LengthException",
        "LimitIterator",
        "LogicException",
        "MultipleIterator",
        "NoRewindIterator",
        "OutOfBoundsException",
        "OutOfRangeException",
        "OverflowException",
        "ParentIterator",
        "Phar",
        "PharData",
        "RangeException",
        "RecursiveArrayIterator",
        "RecursiveCachingIterator",
        "RecursiveCallbackFilterIterator",
        "RecursiveDirectoryIterator",
        "RecursiveFilterIterator",
        "RecursiveIteratorIterator",
        "RecursiveRegexIterator",
        "ReflectionAttribute",
        "ReflectionClass",
        "ReflectionObject",
        "ReflectionEnum",
        "ReflectionClassConstant",
        "ReflectionEnumBackedCase",
        "ReflectionEnumUnitCase",
        "ReflectionException",
        "ReflectionFunction",
        "ReflectionMethod",
        "ReflectionNamedType",
        "ReflectionParameter",
        "ReflectionProperty",
        "ReflectionUnionType",
        "ReflectionIntersectionType",
        "RegexIterator",
        "RuntimeException",
        "SplDoublyLinkedList",
        "SplFileInfo",
        "SplFileObject",
        "SplFixedArray",
        "SplHeap",
        "SplMaxHeap",
        "SplMinHeap",
        "SplObjectStorage",
        "SplPriorityQueue",
        "SplQueue",
        "SplStack",
        "SplTempFileObject",
        "TypeError",
        "UnderflowException",
        "UnexpectedValueException",
        "ValueError",
        "ArithmeticError",
        "stdClass",
    ]
}

/// Branches when the saved dynamic class-string matches one AOT candidate class.
pub(super) fn emit_branch_if_dynamic_new_mixed_class_name_matches(
    ctx: &mut FunctionContext<'_>,
    class_name: &str,
    matched_label: &str,
) {
    let (candidate_label, candidate_len) = ctx.data.add_string(class_name.as_bytes());
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", 8);
            abi::emit_symbol_address(ctx.emitter, "x3", &candidate_label);
            abi::emit_load_int_immediate(ctx.emitter, "x4", candidate_len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_strcasecmp");
            ctx.emitter.instruction("cmp x0, #0");                              // check whether the dynamic class-string matches this AOT class
            ctx.emitter.instruction(&format!("b.eq {}", matched_label));        // select this AOT allocation path on a class-name match
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdi", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rsi", 8);
            abi::emit_symbol_address(ctx.emitter, "rdx", &candidate_label);
            abi::emit_load_int_immediate(ctx.emitter, "rcx", candidate_len as i64);
            abi::emit_call_label(ctx.emitter, "__rt_strcasecmp");
            ctx.emitter.instruction("test rax, rax");                           // check whether the dynamic class-string matches this AOT class
            ctx.emitter.instruction(&format!("je {}", matched_label));          // select this AOT allocation path on a class-name match
        }
    }
}

/// Allocates one generic dynamic-new candidate, runs defaults/constructor, and boxes it as Mixed.
pub(super) fn emit_dynamic_new_mixed_candidate(
    ctx: &mut FunctionContext<'_>,
    candidate: &DynamicNewCandidate,
    constructor_args: &[ValueId],
    constructor_arg_container: Option<ValueId>,
    dummy_receiver_operand: ValueId,
    result: ValueId,
) -> Result<()> {
    if candidate.class_name == "SplFixedArray" && constructor_arg_container.is_none() {
        return emit_dynamic_new_mixed_spl_fixed_array_candidate(
            ctx,
            candidate.class_id,
            constructor_args,
            result,
        );
    }
    if is_spl_doubly_linked_list_family(&candidate.class_name)
        && constructor_arg_container.is_none()
    {
        return emit_dynamic_new_mixed_spl_dll_candidate(ctx, candidate.class_id, result);
    }
    emit_object_allocation(
        ctx,
        candidate.class_id,
        candidate.property_count,
        candidate.allow_dynamic_properties,
        &candidate.uninitialized_marker_offsets,
        &candidate.owned_reference_property_offsets,
    )?;
    let object_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, object_reg);
    let object_base_reg = abi::secondary_scratch_reg(ctx.emitter);
    for default in &candidate.property_defaults {
        abi::emit_load_temporary_stack_slot(ctx.emitter, object_base_reg, 0);
        emit_property_default(ctx, object_base_reg, default)?;
    }
    if let Some(constructor) = &candidate.constructor_impl {
        if let Some(arg_container) = constructor_arg_container {
            emit_dynamic_new_mixed_constructor_container_call(
                ctx,
                candidate,
                constructor,
                arg_container,
            )?;
        } else {
            emit_dynamic_new_mixed_constructor_call(
                ctx,
                candidate,
                constructor,
                constructor_args,
                dummy_receiver_operand,
            )?;
        }
    }
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    emit_box_current_owned_value_as_mixed(
        ctx.emitter,
        &PhpType::Object(candidate.class_name.clone()),
    );
    ctx.store_result_value(result)
}

/// Allocates one constructorless dynamic-new candidate and boxes it as Mixed.
pub(super) fn emit_dynamic_new_without_constructor_mixed_candidate(
    ctx: &mut FunctionContext<'_>,
    candidate: &DynamicNewCandidate,
    result: ValueId,
) -> Result<()> {
    emit_object_allocation(
        ctx,
        candidate.class_id,
        candidate.property_count,
        candidate.allow_dynamic_properties,
        &candidate.uninitialized_marker_offsets,
        &candidate.owned_reference_property_offsets,
    )?;
    let object_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, object_reg);
    let object_base_reg = abi::secondary_scratch_reg(ctx.emitter);
    for default in &candidate.property_defaults {
        abi::emit_load_temporary_stack_slot(ctx.emitter, object_base_reg, 0);
        emit_property_default(ctx, object_base_reg, default)?;
    }
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 16);
    emit_box_current_owned_value_as_mixed(
        ctx.emitter,
        &PhpType::Object(candidate.class_name.clone()),
    );
    ctx.store_result_value(result)
}

/// Allocates a dynamic `SplFixedArray` candidate through its runtime storage constructor.
pub(super) fn emit_dynamic_new_mixed_spl_fixed_array_candidate(
    ctx: &mut FunctionContext<'_>,
    class_id: u64,
    constructor_args: &[ValueId],
    result: ValueId,
) -> Result<()> {
    if constructor_args.len() > 1 {
        return Err(CodegenIrError::unsupported(format!(
            "dynamic SplFixedArray constructor with {} EIR operands",
            constructor_args.len()
        )));
    }
    if let Some(size) = constructor_args.first().copied() {
        ctx.load_value_to_result(size)?;
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
        abi::emit_load_int_immediate(
            ctx.emitter,
            abi::int_arg_reg_name(ctx.emitter.target, 0),
            class_id as i64,
        );
        abi::emit_pop_reg(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, 1));
    } else {
        abi::emit_load_int_immediate(
            ctx.emitter,
            abi::int_arg_reg_name(ctx.emitter.target, 0),
            class_id as i64,
        );
        abi::emit_load_int_immediate(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, 1), 0);
    }
    abi::emit_call_label(ctx.emitter, "__rt_spl_fixed_new");
    emit_box_current_owned_value_as_mixed(
        ctx.emitter,
        &PhpType::Object("SplFixedArray".to_string()),
    );
    ctx.store_result_value(result)
}

/// Allocates a dynamic SPL doubly-linked-list-family candidate through runtime storage.
pub(super) fn emit_dynamic_new_mixed_spl_dll_candidate(
    ctx: &mut FunctionContext<'_>,
    class_id: u64,
    result: ValueId,
) -> Result<()> {
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 0),
        class_id as i64,
    );
    abi::emit_call_label(ctx.emitter, "__rt_spl_dll_new");
    emit_box_current_owned_value_as_mixed(ctx.emitter, &PhpType::Object(String::new()));
    ctx.store_result_value(result)
}

/// Calls the selected candidate constructor while the new object is parked on the temp stack.
pub(super) fn emit_dynamic_new_mixed_constructor_call(
    ctx: &mut FunctionContext<'_>,
    candidate: &DynamicNewCandidate,
    constructor: &ConstructorCallTarget,
    constructor_args: &[ValueId],
    dummy_receiver_operand: ValueId,
) -> Result<()> {
    let object_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, object_reg, 0);
    let mut operands = Vec::with_capacity(constructor_args.len() + 1);
    operands.push(dummy_receiver_operand);
    operands.extend(constructor_args.iter().copied());
    let object_ty = PhpType::Object(candidate.class_name.clone());
    let mut param_types = Vec::with_capacity(constructor.param_types.len() + 1);
    param_types.push(object_ty.clone());
    param_types.extend_from_slice(&constructor.param_types);
    let mut ref_params = Vec::with_capacity(constructor.ref_params.len() + 1);
    ref_params.push(false);
    ref_params.extend_from_slice(&constructor.ref_params);
    let call_args = materialize_method_call_args_with_receiver_reg_and_refs(
        ctx,
        object_reg,
        &object_ty,
        &operands,
        &param_types,
        &ref_params,
    )?;
    let caller_stack_pad_bytes = direct_call_stack_pad_bytes(ctx, call_args.overflow_bytes);
    abi::emit_reserve_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_call_label(
        ctx.emitter,
        &method_symbol(&constructor.impl_class, &php_symbol_key("__construct")),
    );
    abi::emit_release_temporary_stack(ctx.emitter, caller_stack_pad_bytes);
    abi::emit_release_temporary_stack(ctx.emitter, call_args.overflow_bytes);
    emit_ref_arg_writebacks(ctx, &call_args.ref_writebacks)
}

/// Invokes a selected dynamic constructor through its uniform descriptor invoker when PHP
/// supplied named arguments or one or more spread arrays.
pub(super) fn emit_dynamic_new_mixed_constructor_container_call(
    ctx: &mut FunctionContext<'_>,
    candidate: &DynamicNewCandidate,
    constructor: &ConstructorCallTarget,
    arg_container: ValueId,
) -> Result<()> {
    let receiver_ty = PhpType::Object(candidate.class_name.clone());
    let captures = vec![("receiver".to_string(), receiver_ty.clone(), false)];
    let constructor_key = php_symbol_key("__construct");
    let entry_label = emit_instance_method_descriptor_entry_wrapper(
        ctx,
        &constructor.impl_class,
        &constructor_key,
        &constructor.sig,
    )?;
    let invoker_label = emit_runtime_callable_invoker_inline(ctx, &constructor.sig, &captures);
    let php_name = format!("{}::__construct", candidate.class_name);
    let descriptor_label = callable_descriptor::static_descriptor_with_optional_invoker_meta(
        ctx.data,
        &entry_label,
        Some(&php_name),
        callable_descriptor::CALLABLE_DESC_KIND_FIRST_CLASS,
        Some(&constructor.sig),
        &captures,
        &[],
        callable_descriptor::CallableDescriptorInvocation::method(
            callable_descriptor::CallableDescriptorShape::InstanceMethod,
            Some(candidate.class_name.clone()),
            "__construct",
        ),
        Some(&invoker_label),
    );

    let result_reg = abi::int_result_reg(ctx.emitter).to_string();
    let descriptor_reg = abi::nested_call_reg(ctx.emitter).to_string();
    let total_bytes = callable_descriptor::CALLABLE_DESC_RUNTIME_CAPTURE_OFFSET + 16;
    abi::emit_load_temporary_stack_slot(ctx.emitter, &result_reg, 0);
    abi::emit_incref_if_refcounted(ctx.emitter, &receiver_ty);
    abi::emit_push_reg(ctx.emitter, &result_reg);
    abi::emit_load_int_immediate(ctx.emitter, &result_reg, total_bytes as i64);
    abi::emit_call_label(ctx.emitter, "__rt_heap_alloc");
    ctx.emitter
        .instruction(&format!("mov {}, {}", descriptor_reg, result_reg)); // preserve the runtime constructor descriptor while copying its static header
    callable_descriptor::emit_copy_static_descriptor_to_runtime(
        ctx.emitter,
        &descriptor_reg,
        &descriptor_label,
    );
    abi::emit_pop_reg(ctx.emitter, &result_reg);
    callable_descriptor::emit_store_current_result_to_runtime_capture(
        ctx.emitter,
        &descriptor_reg,
        0,
        &receiver_ty,
    );
    callables::emit_descriptor_reg_invoker_mixed_result_with_arg_container(
        ctx,
        &descriptor_reg,
        arg_container,
        "dynamic_constructor",
        true,
    )?;
    abi::emit_call_label(ctx.emitter, "__rt_decref_mixed");
    Ok(())
}

/// Invokes the runtime class-name registry fallback and boxes a matched object as Mixed.
pub(super) fn emit_dynamic_new_mixed_fallback(ctx: &mut FunctionContext<'_>) {
    let miss_label = ctx.next_label("dynamic_new_mixed_missing_class");
    let done_label = ctx.next_label("dynamic_new_mixed_fallback_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x1", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "x2", 8);
            abi::emit_call_label(ctx.emitter, "__rt_new_by_name");
            ctx.emitter.instruction(&format!("cbz x0, {}", miss_label));        // registry miss is PHP's class-not-found fatal for source-level dynamic construction
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Object(String::new()));
            ctx.emitter.instruction(&format!("b {}", done_label));              // skip the fatal path after a registry allocation
            ctx.emitter.label(&miss_label);
            emit_dynamic_new_class_not_found_fatal(ctx);
            ctx.emitter.label(&done_label);
        }
        Arch::X86_64 => {
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rax", 0);
            abi::emit_load_temporary_stack_slot(ctx.emitter, "rdx", 8);
            abi::emit_call_label(ctx.emitter, "__rt_new_by_name");
            ctx.emitter.instruction("test rax, rax");                           // did the runtime class registry produce an object?
            ctx.emitter.instruction(&format!("jz {}", miss_label));             // registry miss is PHP's class-not-found fatal
            abi::emit_release_temporary_stack(ctx.emitter, 16);
            emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Object(String::new()));
            ctx.emitter.instruction(&format!("jmp {}", done_label));            // skip the fatal path after a registry allocation
            ctx.emitter.label(&miss_label);
            emit_dynamic_new_class_not_found_fatal(ctx);
            ctx.emitter.label(&done_label);
        }
    }
}
