//! Lowers explicit compiled interface activation events into request-local cells.

use super::*;

pub(super) fn lower_classlike_activate(
    ctx: &mut FunctionContext<'_>,
    inst: &Instruction,
) -> Result<()> {
    let Some(Immediate::ClassLikeActivation { kind, name, .. }) = &inst.immediate else {
        return Err(CodegenIrError::invalid_module(
            "class_like_activate requires typed declaration identity",
        ));
    };
    if *kind != crate::parser::ast::ClassLikeKind::Interface {
        return Err(CodegenIrError::unsupported(format!(
            "class-like activation for {:?}", kind
        )));
    }
    let name = ctx
        .module
        .data
        .strings
        .get(name.as_raw() as usize)
        .ok_or_else(|| CodegenIrError::missing_entry("data string", name.as_raw()))?;
    let cell = crate::names::classlike_activation_symbol(*kind, name);
    ctx.data.add_comm(cell.clone(), 8);

    let active = ctx.next_label("classlike_activate_already_active");
    let done = ctx.next_label("classlike_activate_done");
    let value = abi::temp_int_reg(ctx.emitter.target);
    abi::emit_load_symbol_to_reg(ctx.emitter, value, &cell, 0);
    match ctx.emitter.target.arch {
        Arch::AArch64 => ctx.emitter.instruction(&format!("cbnz {}, {}", value, active)),
        Arch::X86_64 => {
            ctx.emitter.instruction(&format!("test {}, {}", value, value));
            ctx.emitter.instruction(&format!("jne {}", active));
        }
    }
    abi::emit_load_int_immediate(ctx.emitter, value, 1);
    abi::emit_store_reg_to_symbol(ctx.emitter, value, &cell, 0);
    abi::emit_jump(ctx.emitter, &done);

    ctx.emitter.label(&active);
    exceptions::emit_error(ctx, &format!("Cannot redeclare interface {}", name));
    ctx.emitter.label(&done);
    Ok(())
}
