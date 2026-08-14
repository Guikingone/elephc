//! Purpose:
//! Array filter callback dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays`.
//!
//! Key details:
//! - Preserves callback ABI, target parity, array storage, and ownership contracts.

use super::*;

/// Lowers `array_filter()` with either PHP truthiness or a static/first-class callback.
pub(crate) fn lower_array_filter(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "array_filter", 1, 3)?;
    let array = expect_operand(inst, 0)?;
    let callback = inst.operands.get(1).copied();
    let mode = inst.operands.get(2).copied();
    let source_ty = ctx.value_php_type(array)?.codegen_repr();
    let gradual = matches!(source_ty, PhpType::Mixed | PhpType::Union(_));
    let elem_ty = if gradual {
        require_mixed_array_filter_result_type(&inst.result_php_type.codegen_repr())?;
        emit_array_filter_gradual_source_guard(ctx, array)?;
        PhpType::Mixed
    } else {
        let elem_ty = array_filter_source_element_type(source_ty)?;
        require_array_filter_result_type(&elem_ty, &inst.result_php_type.codegen_repr())?;
        elem_ty
    };
    let runtime_label = if gradual {
        "__rt_array_filter_mixed"
    } else if array_filter_uses_refcounted_runtime(&elem_ty) {
        "__rt_array_filter_refcounted"
    } else {
        "__rt_array_filter"
    };
    if array_filter_uses_default_callback(ctx, callback)? {
        let wrapper_label = emit_array_filter_truthiness_wrapper(ctx, &elem_ty, gradual)?;
        load_array_filter_runtime_args(ctx, array, None, &wrapper_label, 0)?;
        abi::emit_call_label(ctx.emitter, runtime_label);
        return store_if_result(ctx, inst);
    }

    let callback = callback.ok_or_else(|| {
        CodegenIrError::invalid_module("array_filter callback unexpectedly missing")
    })?;
    let callback_arg_types = array_filter_callback_arg_types(ctx, mode, &elem_ty, gradual)?;
    if let Some(visible_arg_types) = callback_arg_types.clone() {
        match ctx.value_php_type(callback)?.codegen_repr() {
            PhpType::Callable => {
                lower_descriptor_callback_runtime(
                    ctx,
                    callback,
                    visible_arg_types,
                    PhpType::Bool,
                    |ctx, wrapper_label, env_bytes| {
                        load_array_filter_runtime_args(
                            ctx,
                            array,
                            mode,
                            wrapper_label,
                            env_bytes,
                        )?;
                        abi::emit_call_label(ctx.emitter, runtime_label);
                        Ok(())
                    },
                )?;
                store_if_result(ctx, inst)?;
                return Ok(());
            }
            PhpType::Str => {
                lower_runtime_string_descriptor_callback(
                    ctx,
                    callback,
                    Some(&PhpType::Array(Box::new(elem_ty.clone()))),
                    visible_arg_types,
                    PhpType::Bool,
                    super::super::super::instruction_strict_php_profile(inst),
                    "array_filter",
                    |ctx, wrapper_label, env_bytes| {
                        load_array_filter_runtime_args(
                            ctx,
                            array,
                            mode,
                            wrapper_label,
                            env_bytes,
                        )?;
                        abi::emit_call_label(ctx.emitter, runtime_label);
                        Ok(())
                    },
                )?;
                store_if_result(ctx, inst)?;
                return Ok(());
            }
            _ => {}
        }
    }
    let callback_binding = static_sort_callback_binding(
        ctx,
        callback,
        "array_filter callback",
        callback_arg_types.as_deref(),
    )?;
    let env_bytes = reserve_static_callback_env(ctx, callback_binding.env_source)?;
    load_array_filter_runtime_args(
        ctx,
        array,
        mode,
        &callback_binding.label,
        env_bytes,
    )?;
    abi::emit_call_label(ctx.emitter, runtime_label);
    if env_bytes != 0 {
        abi::emit_release_temporary_stack(ctx.emitter, env_bytes);
    }
    store_if_result(ctx, inst)
}

/// Returns whether an omitted or literal-null callback requests PHP truthiness filtering.
fn array_filter_uses_default_callback(
    ctx: &FunctionContext<'_>,
    callback: Option<ValueId>,
) -> Result<bool> {
    let Some(callback) = callback else {
        return Ok(true);
    };
    let value = ctx
        .function
        .value(callback)
        .ok_or_else(|| CodegenIrError::missing_entry("value", callback.as_raw()))?;
    let ValueDef::Instruction { inst, .. } = value.def else {
        return Ok(false);
    };
    let defining = ctx
        .function
        .instruction(inst)
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", inst.as_raw()))?;
    Ok(defining.op == Op::ConstNull)
}

/// Loads the callback, source, environment, and mode expected by all filter runtimes.
fn load_array_filter_runtime_args(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
    mode: Option<ValueId>,
    callback_label: &str,
    env_bytes: usize,
) -> Result<()> {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x0", callback_label);
            ctx.load_value_to_reg(array, "x1")?;
            load_static_callback_env_arg(ctx, "x2", env_bytes);
            load_array_filter_mode(ctx, mode, "x3")
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rdi", callback_label);
            ctx.load_value_to_reg(array, "rsi")?;
            load_static_callback_env_arg(ctx, "rdx", env_bytes);
            load_array_filter_mode(ctx, mode, "rcx")
        }
    }
}

/// Checks that a gradual source currently contains either indexed or associative array storage.
fn emit_array_filter_gradual_source_guard(
    ctx: &mut FunctionContext<'_>,
    array: ValueId,
) -> Result<()> {
    ctx.load_value_to_result(array)?;
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let valid = ctx.next_label("array_filter_gradual_array");
    let invalid = ctx.next_label("array_filter_gradual_wrong_type");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("b.eq {valid}"));
            ctx.emitter.instruction("cmp x0, #5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("b.eq {valid}"));
            ctx.emitter.instruction(&format!("b {invalid}"));
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 4");                              // runtime tag 4 = indexed array
            ctx.emitter.instruction(&format!("je {valid}"));
            ctx.emitter.instruction("cmp rax, 5");                              // runtime tag 5 = associative array
            ctx.emitter.instruction(&format!("je {valid}"));
            ctx.emitter.instruction(&format!("jmp {invalid}"));
        }
    }
    union_type_guard::emit_mixed_wrong_tag_type_error_dispatch(ctx, &invalid, &|given| {
        format!(
            "array_filter(): Argument #1 ($array) must be of type array, {} given",
            given
        )
    });
    ctx.emitter.label(&valid);
    Ok(())
}

/// Emits a target-aware callback that returns PHP truthiness for one array element.
fn emit_array_filter_truthiness_wrapper(
    ctx: &mut FunctionContext<'_>,
    elem_ty: &PhpType,
    boxed_input: bool,
) -> Result<String> {
    let wrapper = ctx.next_label("array_filter_truthiness");
    let continuation = ctx.next_label("array_filter_after_truthiness");
    abi::emit_jump(ctx.emitter, &continuation);
    ctx.emitter.label(&wrapper);
    if boxed_input || elem_ty.codegen_repr() == PhpType::Mixed {
        emit_boxed_array_filter_truthiness(ctx);
    } else {
        emit_typed_array_filter_truthiness(ctx, &elem_ty.codegen_repr())?;
    }
    ctx.emitter.label(&continuation);
    Ok(wrapper)
}

/// Emits a callback that delegates boxed values to the shared PHP truthiness helper.
fn emit_boxed_array_filter_truthiness(ctx: &mut FunctionContext<'_>) {
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("sub sp, sp, #16");                         // preserve the callback return address across the runtime call
            ctx.emitter.instruction("str x30, [sp]");                           // save the filter loop continuation
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_bool");
            ctx.emitter.instruction("ldr x30, [sp]");                           // restore the filter loop continuation
            ctx.emitter.instruction("add sp, sp, #16");                         // release callback spill storage
            ctx.emitter.instruction("ret");                                     // return normalized truthiness in x0
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("push rbp");                                // align the nested runtime call and preserve the frame pointer
            ctx.emitter.instruction("mov rbp, rsp");                            // establish the callback frame
            ctx.emitter.instruction("mov rax, rdi");                            // mixed_cast_bool consumes the boxed cell in rax
            abi::emit_call_label(ctx.emitter, "__rt_mixed_cast_bool");
            ctx.emitter.instruction("pop rbp");                                 // restore the filter runtime frame
            ctx.emitter.instruction("ret");                                     // return normalized truthiness in rax
        }
    }
}

/// Emits PHP truthiness for a callback argument carried in its native element ABI.
fn emit_typed_array_filter_truthiness(
    ctx: &mut FunctionContext<'_>,
    elem_ty: &PhpType,
) -> Result<()> {
    let false_label = ctx.next_label("array_filter_truthiness_false");
    let true_label = ctx.next_label("array_filter_truthiness_true");
    let return_label = ctx.next_label("array_filter_truthiness_return");
    match (ctx.emitter.target.arch, elem_ty) {
        (Arch::AArch64, PhpType::Int | PhpType::Bool | PhpType::False) => {
            ctx.emitter.instruction("cmp x0, #0");                              // integer-like values are truthy when non-zero
            ctx.emitter.instruction("cset x0, ne");                             // normalize the predicate result
        }
        (Arch::X86_64, PhpType::Int | PhpType::Bool | PhpType::False) => {
            ctx.emitter.instruction("test rdi, rdi");                           // integer-like values are truthy when non-zero
            ctx.emitter.instruction("setne al");                                // normalize the predicate result byte
            ctx.emitter.instruction("movzx eax, al");                           // clear unused result bits
        }
        (Arch::AArch64, PhpType::Str) => {
            ctx.emitter.instruction(&format!("cbz x1, {false_label}"));         // empty strings are false
            ctx.emitter.instruction("cmp x1, #1");                              // only the single-byte string `0` is additionally false
            ctx.emitter.instruction(&format!("b.ne {true_label}"));
            ctx.emitter.instruction("ldrb w9, [x0]");                           // read the only byte
            ctx.emitter.instruction("cmp w9, #48");                             // ASCII `0`
            ctx.emitter.instruction("cset x0, ne");                             // every other byte is true
            ctx.emitter.instruction(&format!("b {return_label}"));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov x0, #0");                              // return false
            ctx.emitter.instruction(&format!("b {return_label}"));
            ctx.emitter.label(&true_label);
            ctx.emitter.instruction("mov x0, #1");                              // return true
            ctx.emitter.label(&return_label);
        }
        (Arch::X86_64, PhpType::Str) => {
            ctx.emitter.instruction("test rsi, rsi");                           // empty strings are false
            ctx.emitter.instruction(&format!("jz {false_label}"));
            ctx.emitter.instruction("cmp rsi, 1");                              // only the single-byte string `0` is additionally false
            ctx.emitter.instruction(&format!("jne {true_label}"));
            ctx.emitter.instruction("cmp BYTE PTR [rdi], 48");                  // ASCII `0`
            ctx.emitter.instruction("setne al");                                // every other byte is true
            ctx.emitter.instruction("movzx eax, al");                           // clear unused result bits
            ctx.emitter.instruction(&format!("jmp {return_label}"));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("xor eax, eax");                            // return false
            ctx.emitter.instruction(&format!("jmp {return_label}"));
            ctx.emitter.label(&true_label);
            ctx.emitter.instruction("mov eax, 1");                              // return true
            ctx.emitter.label(&return_label);
        }
        (Arch::AArch64, PhpType::Array(_) | PhpType::AssocArray { .. }) => {
            ctx.emitter.instruction(&format!("cbz x0, {false_label}"));         // null containers are false
            ctx.emitter.instruction("ldr x0, [x0]");                            // load container length
            ctx.emitter.instruction("cmp x0, #0");                              // non-empty arrays are true
            ctx.emitter.instruction("cset x0, ne");                             // normalize the result
            ctx.emitter.instruction(&format!("b {return_label}"));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("mov x0, #0");                              // return false
            ctx.emitter.label(&return_label);
        }
        (Arch::X86_64, PhpType::Array(_) | PhpType::AssocArray { .. }) => {
            ctx.emitter.instruction("test rdi, rdi");                           // null containers are false
            ctx.emitter.instruction(&format!("jz {false_label}"));
            ctx.emitter.instruction("cmp QWORD PTR [rdi], 0");                  // non-empty arrays are true
            ctx.emitter.instruction("setne al");                                // normalize the result
            ctx.emitter.instruction("movzx eax, al");                           // clear unused result bits
            ctx.emitter.instruction(&format!("jmp {return_label}"));
            ctx.emitter.label(&false_label);
            ctx.emitter.instruction("xor eax, eax");                            // return false
            ctx.emitter.label(&return_label);
        }
        (Arch::AArch64, PhpType::Void | PhpType::Never) => {
            ctx.emitter.instruction("mov x0, #0");                              // null-like elements are false
        }
        (Arch::X86_64, PhpType::Void | PhpType::Never) => {
            ctx.emitter.instruction("xor eax, eax");                            // null-like elements are false
        }
        (Arch::AArch64, ty) if ty.is_refcounted() => {
            ctx.emitter.instruction("mov x0, #1");                              // concrete objects are true
        }
        (Arch::X86_64, ty) if ty.is_refcounted() => {
            ctx.emitter.instruction("mov eax, 1");                              // concrete objects are true
        }
        (_, ty) => {
            return Err(CodegenIrError::unsupported(format!(
                "array_filter truthiness for PHP type {:?}",
                ty
            )))
        }
    }
    ctx.emitter.instruction("ret");                                             // return predicate truthiness to the filter loop
    Ok(())
}
