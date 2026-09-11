//! Purpose:
//! Lowers named and dynamic instanceof entry points.
//!
//! Called from:
//! - The object lowering facade and sibling object support modules.
//!
//! Key details:
//! - Eval-aware and runtime metadata paths keep their existing precedence.

use super::*;

/// Lowers named `instanceof` using runtime class/interface metadata.
pub(in crate::codegen::lower_inst) fn lower_instanceof(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let value_ty = ctx.value_php_type(value)?;
    let class_name = class_name_immediate(ctx, inst)?.to_string();
    if !matches!(
        value_ty,
        PhpType::Callable | PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_)
    ) {
        emit_false(ctx);
        return store_if_result(ctx, inst);
    }
    if builtins::has_eval_context(ctx) {
        return builtins::lower_eval_object_is_a(ctx, inst, value, &class_name, false);
    }
    // An AOT method can receive an object that was created by a request-global eval context.
    // Its native storage header is deliberately the generic object wrapper, so the ordinary
    // matcher would only see `stdClass`. Ask the bridge-owned identity table first, then retain
    // the native matcher as the fallback for ordinary AOT objects and callable descriptors.
    let eval_fallback_done = if ctx.module.required_runtime_features.eval_bridge
        && matches!(value_ty, PhpType::Object(_) | PhpType::Mixed | PhpType::Union(_))
    {
        let eval_matched = ctx.next_label("instanceof_eval_owner_matched");
        let native_fallback = ctx.next_label("instanceof_eval_owner_native_fallback");
        let done = ctx.next_label("instanceof_eval_owner_done");
        builtins::emit_eval_object_is_a_named_fallback(
            ctx,
            value,
            &class_name,
            &eval_matched,
            &native_fallback,
        )?;
        ctx.emitter.label(&eval_matched);
        abi::emit_load_int_immediate(ctx.emitter, abi::int_result_reg(ctx.emitter), 1);
        abi::emit_jump(ctx.emitter, &done);
        ctx.emitter.label(&native_fallback);
        Some(done)
    } else {
        None
    };
    if class_name
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("Closure")
    {
        emit_closure_instanceof(ctx, value, &value_ty)?;
        if let Some(done) = &eval_fallback_done {
            abi::emit_jump(ctx.emitter, done);
            ctx.emitter.label(done);
        }
        return store_if_result(ctx, inst);
    }
    let Some((target_id, target_kind)) = classify_named_target(ctx, &class_name) else {
        emit_false(ctx);
        if let Some(done) = &eval_fallback_done {
            abi::emit_jump(ctx.emitter, done);
            ctx.emitter.label(done);
        }
        return store_if_result(ctx, inst);
    };
    match value_ty {
        PhpType::Callable => {
            emit_callable_object_capture_or_null(ctx, value)?;
            emit_match_call(ctx, target_id, target_kind, "__rt_exception_matches");
        }
        PhpType::Object(_) => {
            ctx.load_value_to_reg(value, abi::int_arg_reg_name(ctx.emitter.target, 0))?;
            emit_match_call(ctx, target_id, target_kind, "__rt_exception_matches");
        }
        PhpType::Mixed | PhpType::Union(_) => {
            ctx.load_value_to_reg(value, abi::int_arg_reg_name(ctx.emitter.target, 0))?;
            emit_match_call(ctx, target_id, target_kind, "__rt_mixed_instanceof");
        }
        _ => emit_false(ctx),
    }
    if let Some(done) = &eval_fallback_done {
        abi::emit_jump(ctx.emitter, done);
        ctx.emitter.label(done);
    }
    store_if_result(ctx, inst)
}

/// Lowers dynamic `instanceof` where the target is resolved from a runtime string or object.
pub(in crate::codegen::lower_inst) fn lower_instanceof_dynamic(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let value = expect_operand(inst, 0)?;
    let target = expect_operand(inst, 1)?;
    if builtins::has_eval_context(ctx) {
        return builtins::lower_eval_object_is_a_dynamic(ctx, inst, value, target, false);
    }
    let value_ty = ctx.value_php_type(value)?;
    let target_ty = ctx.value_php_type(target)?;
    let target_false = ctx.next_label("instanceof_dynamic_target_false");
    let done = ctx.next_label("instanceof_dynamic_done");
    emit_normalized_dynamic_instanceof_value(ctx, value, &value_ty)?;
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    emit_dynamic_target_metadata(ctx, target, &target_ty, &target_false)?;
    emit_dynamic_match_call(ctx);
    abi::emit_jump(ctx.emitter, &done);
    ctx.emitter.label(&target_false);
    abi::emit_pop_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    emit_false(ctx);
    ctx.emitter.label(&done);
    store_if_result(ctx, inst)
}
