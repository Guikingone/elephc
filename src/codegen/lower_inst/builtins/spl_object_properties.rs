//! Purpose:
//! Lowers raw numeric insertion into SPL backing-object property tables.
//!
//! Called from:
//! - The typed runtime-function dispatcher for ElephcSplObjectAppend.
//!
//! Key details:
//! - The receiver remains borrowed from EIR's operand owner.
//! - One independently owned Mixed value is transferred to the runtime append helper.

use crate::codegen::{abi, emit_box_current_value_as_mixed, CodegenIrError, Result};
use crate::codegen::platform::Arch;
use crate::ir::Instruction;
use crate::types::PhpType;
use super::super::super::context::FunctionContext;
use super::{expect_operand, store_if_result};

/// Transfers a boxed value to the integer property-bucket insertion helper.
pub(crate) fn lower_append(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    super::ensure_arg_count(inst, "__elephc_spl_object_append", 2)?;
    let object = expect_operand(inst, 0)?;
    let value = expect_operand(inst, 1)?;
    let object_ty = ctx.raw_value_php_type(object)?.codegen_repr();
    ctx.load_value_to_result(object)?;
    match object_ty {
        PhpType::Object(_) => {}
        PhpType::Mixed | PhpType::Union(_) => {
            abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
            let valid = ctx.next_label("spl_object_append_object");
            match ctx.emitter.target.arch {
                Arch::AArch64 => {
                    ctx.emitter.instruction("cmp x0, #6");                      // require an object tag before extracting its payload
                    ctx.emitter.instruction(&format!("b.eq {valid}"));          // preserve dynamically typed object receivers
                }
                Arch::X86_64 => {
                    ctx.emitter.instruction("cmp rax, 6");                      // require an object tag before extracting its payload
                    ctx.emitter.instruction(&format!("je {valid}"));            // preserve dynamically typed object receivers
                }
            }
            super::super::exceptions::emit_type_error(ctx, "SPL backing property append requires an object");
            ctx.emitter.label(&valid);
            let payload = if ctx.emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
            abi::emit_reg_move(ctx.emitter, abi::int_result_reg(ctx.emitter), payload);
        }
        other => return Err(CodegenIrError::unsupported(format!("SPL property append from {other:?}"))),
    }
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let value_ty = ctx.raw_value_php_type(value)?.codegen_repr();
    ctx.load_value_to_result(value)?;
    if matches!(value_ty, PhpType::Mixed | PhpType::Union(_)) {
        abi::emit_incref_if_refcounted(ctx.emitter, &value_ty);
    } else {
        emit_box_current_value_as_mixed(ctx.emitter, &value_ty);
    }
    abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    let arg0 = abi::int_arg_reg_name(ctx.emitter.target, 0);
    let arg1 = abi::int_arg_reg_name(ctx.emitter.target, 1);
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg0, 16);
    abi::emit_load_temporary_stack_slot(ctx.emitter, arg1, 0);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    abi::emit_call_label(ctx.emitter, "__rt_spl_object_property_append");
    store_if_result(ctx, inst)
}
