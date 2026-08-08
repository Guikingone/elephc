//! Purpose:
//! Array filter callback dispatch.
//!
//! Called from:
//! - `crate::codegen::lower_inst::builtins::arrays`.
//!
//! Key details:
//! - Preserves callback ABI, target parity, array storage, and ownership contracts.

use super::*;

/// Lowers `array_filter()` for static and first-class callbacks through the runtime helper.
pub(crate) fn lower_array_filter(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    ensure_arg_count_between(inst, "array_filter", 2, 3)?;
    let array = expect_operand(inst, 0)?;
    let callback = expect_operand(inst, 1)?;
    let mode = inst.operands.get(2).copied();
    let elem_ty = array_filter_source_element_type(ctx.value_php_type(array)?)?;
    require_array_filter_result_type(&elem_ty, &inst.result_php_type.codegen_repr())?;
    let runtime_label = if array_filter_uses_refcounted_runtime(&elem_ty) {
        "__rt_array_filter_refcounted"
    } else {
        "__rt_array_filter"
    };
    let callback_arg_types = array_filter_callback_arg_types(ctx, mode, &elem_ty)?;
    if let Some(visible_arg_types) = callback_arg_types.clone() {
        match ctx.value_php_type(callback)?.codegen_repr() {
            PhpType::Callable => {
                lower_descriptor_callback_runtime(
                    ctx,
                    callback,
                    visible_arg_types,
                    PhpType::Bool,
                    |ctx, wrapper_label, env_bytes| {
                        match ctx.emitter.target.arch {
                            Arch::AArch64 => {
                                abi::emit_symbol_address(ctx.emitter, "x0", wrapper_label);
                                ctx.load_value_to_reg(array, "x1")?;
                                load_static_callback_env_arg(ctx, "x2", env_bytes);
                                load_array_filter_mode(ctx, mode, "x3")?;
                            }
                            Arch::X86_64 => {
                                abi::emit_symbol_address(ctx.emitter, "rdi", wrapper_label);
                                ctx.load_value_to_reg(array, "rsi")?;
                                load_static_callback_env_arg(ctx, "rdx", env_bytes);
                                load_array_filter_mode(ctx, mode, "rcx")?;
                            }
                        }
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
                        match ctx.emitter.target.arch {
                            Arch::AArch64 => {
                                abi::emit_symbol_address(ctx.emitter, "x0", wrapper_label);
                                ctx.load_value_to_reg(array, "x1")?;
                                load_static_callback_env_arg(ctx, "x2", env_bytes);
                                load_array_filter_mode(ctx, mode, "x3")?;
                            }
                            Arch::X86_64 => {
                                abi::emit_symbol_address(ctx.emitter, "rdi", wrapper_label);
                                ctx.load_value_to_reg(array, "rsi")?;
                                load_static_callback_env_arg(ctx, "rdx", env_bytes);
                                load_array_filter_mode(ctx, mode, "rcx")?;
                            }
                        }
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
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            abi::emit_symbol_address(ctx.emitter, "x0", &callback_binding.label);
            ctx.load_value_to_reg(array, "x1")?;
            load_static_callback_env_arg(ctx, "x2", env_bytes);
            load_array_filter_mode(ctx, mode, "x3")?;
        }
        Arch::X86_64 => {
            abi::emit_symbol_address(ctx.emitter, "rdi", &callback_binding.label);
            ctx.load_value_to_reg(array, "rsi")?;
            load_static_callback_env_arg(ctx, "rdx", env_bytes);
            load_array_filter_mode(ctx, mode, "rcx")?;
        }
    }
    abi::emit_call_label(ctx.emitter, runtime_label);
    if env_bytes != 0 {
        abi::emit_release_temporary_stack(ctx.emitter, env_bytes);
    }
    store_if_result(ctx, inst)
}

