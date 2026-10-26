//! Purpose:
//! Lowers every print_r call through the same owned capture boundary.
//! Keeps result representation independent from globals changed by callbacks.
//!
//! Called from:
//! - `super::lower_print_r`, re-exported for the typed PrintR runtime target.
//!
//! Key details:
//! - The runtime consumes one Mixed-cell owner.
//! - Existing null-container sentinels become canonical null before boxing.
//! - Named and positional argument evaluation was already planned in EIR.

use super::{abi, CodegenIrError, FunctionContext, Instruction, PhpType, Result};
use super::{expect_operand, loaded_php_semantic_type, store_if_result};

/// Boxes the source once and lets the runtime own its capture and argument lifetime.
pub(crate) fn lower_print_r(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    if inst.operands.is_empty() || inst.operands.len() > 2 {
        return Err(CodegenIrError::unsupported("print_r() takes 1 or 2 arguments"));
    }
    let value = expect_operand(inst, 0)?;
    let ty = loaded_php_semantic_type(ctx, value)?;
    if ty == PhpType::Callable {
        return Err(CodegenIrError::unsupported("print_r for PHP type Callable"));
    }
    box_owned_input(ctx, &ty);
    let result = abi::int_result_reg(ctx.emitter);
    abi::emit_push_reg(ctx.emitter, result);
    let kind = match inst.result_php_type.codegen_repr() {
        PhpType::Bool => 0,
        PhpType::Str => 1,
        _ => 2,
    };
    if kind == 2 {
        if let Some(flag) = inst.operands.get(1).copied() {
            let flag_ty = ctx.load_value_to_reg(flag, result)?;
            if !matches!(flag_ty.codegen_repr(), PhpType::Bool | PhpType::Int) {
                return Err(CodegenIrError::unsupported(format!(
                    "print_r $return flag for PHP type {flag_ty:?}",
                )));
            }
        } else {
            abi::emit_load_int_immediate(ctx.emitter, result, 0);
        }
    } else {
        abi::emit_load_int_immediate(ctx.emitter, result, i64::from(kind == 1));
    }
    let flag = abi::int_arg_reg_name(ctx.emitter.target, 1);
    abi::emit_reg_move(ctx.emitter, flag, result);
    let input = abi::int_arg_reg_name(ctx.emitter.target, 0);
    abi::emit_pop_reg(ctx.emitter, input);
    let representation = abi::int_arg_reg_name(ctx.emitter.target, 2);
    abi::emit_load_int_immediate(ctx.emitter, representation, kind);
    let format = i64::from(matches!(&ty, PhpType::Pointer(_) | PhpType::Buffer(_) | PhpType::Packed(_)));
    abi::emit_load_int_immediate(ctx.emitter, abi::int_arg_reg_name(ctx.emitter.target, 3), format);
    abi::emit_call_label(ctx.emitter, "__rt_print_r_scoped");
    store_if_result(ctx, inst)
}

/// Gives the runtime one cell owner without changing the caller's original storage.
fn box_owned_input(ctx: &mut FunctionContext<'_>, ty: &PhpType) {
    if matches!(ty, PhpType::Mixed | PhpType::Union(_)) {
        abi::emit_call_label(ctx.emitter, "__rt_incref");
        return;
    }
    if matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. } | PhpType::Iterable | PhpType::Object(_)) {
        let null = ctx.next_label("print_r_input_null");
        let ready = ctx.next_label("print_r_input_boxed");
        let result = abi::int_result_reg(ctx.emitter);
        let scratch = abi::secondary_scratch_reg(ctx.emitter);
        crate::codegen::sentinels::emit_branch_if_null_container(ctx.emitter, result, scratch, &null);
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, ty);
        abi::emit_jump(ctx.emitter, &ready);
        ctx.emitter.label(&null);
        abi::emit_load_int_immediate(ctx.emitter, result, 0);
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, &PhpType::Void);
        ctx.emitter.label(&ready);
    } else {
        crate::codegen::emit_box_current_value_as_mixed(ctx.emitter, ty);
    }
}
