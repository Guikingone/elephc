//! Purpose:
//! Materializes statically typed native objects through the shared runtime construction bridge.
//!
//! Called from:
//! - Reflection owner lowering when constructor metadata depends on runtime values.
//!
//! Key details:
//! - Positional operands cross the bridge as boxed Mixed cells.
//! - The returned Mixed wrapper is released after its object payload receives an owned reference.

use super::*;

/// Constructs a statically typed object through runtime metadata and stores its raw object payload.
pub(in crate::codegen::lower_inst) fn lower_eval_native_object_new(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let (name_label, name_len) = ctx.intern_class_name_data(expect_data(inst)?)?;
    let args_offset = EVAL_STACK_BYTES;
    let stack_bytes = eval_function_call_stack_bytes(inst.operands.len());
    abi::emit_reserve_temporary_stack(ctx.emitter, stack_bytes);
    store_eval_function_call_args(ctx, inst, args_offset)?;
    ensure_eval_context(ctx)?;
    load_eval_context_to_arg(ctx, 0);
    let name_arg = abi::int_arg_reg_name(ctx.emitter.target, 1);
    abi::emit_symbol_address(ctx.emitter, name_arg, &name_label);
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 2),
        name_len as i64,
    );
    let args_arg = abi::int_arg_reg_name(ctx.emitter.target, 3);
    if inst.operands.is_empty() {
        abi::emit_load_int_immediate(ctx.emitter, args_arg, 0);
    } else {
        abi::emit_temporary_stack_address(ctx.emitter, args_arg, args_offset);
    }
    abi::emit_load_int_immediate(
        ctx.emitter,
        abi::int_arg_reg_name(ctx.emitter.target, 4),
        inst.operands.len() as i64,
    );
    let out_arg = abi::int_arg_reg_name(ctx.emitter.target, 5);
    abi::emit_temporary_stack_address(ctx.emitter, out_arg, 0);
    let symbol = ctx
        .emitter
        .target
        .extern_symbol("__elephc_eval_reflection_new_object");
    abi::emit_call_label(ctx.emitter, &symbol);
    emit_eval_status_check(ctx);

    let result_reg = abi::int_result_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_RESULT_VALUE_CELL_OFFSET);
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_TEMP_CELL_OFFSET);
    crate::codegen::lower_inst::reference_arguments::emit_unbox_mixed_to_owned_refcounted_result(
        ctx,
        &inst.result_php_type.codegen_repr(),
    );
    abi::emit_store_to_sp(ctx.emitter, result_reg, EVAL_CODE_PTR_OFFSET);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_TEMP_CELL_OFFSET);
    abi::emit_decref_if_refcounted(ctx.emitter, &PhpType::Mixed);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result_reg, EVAL_CODE_PTR_OFFSET);
    abi::emit_release_temporary_stack(ctx.emitter, stack_bytes);
    store_if_result(ctx, inst)
}
