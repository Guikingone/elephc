//! Purpose:
//! Chains a captured boxed exception behind a new native Throwable.
//!
//! Called from:
//! - Typed compiler calls while completing SPL data/priority destructor cleanup.
//!
//! Key details:
//! - The existing exception-chain helper consumes one old-object owner.
//! - Acquire that owner independently of the borrowed or temporary input box.

use crate::codegen::{context::FunctionContext, Result};
use crate::codegen_support::{abi, mixed_unbox_payload_reg, platform::Arch};
use crate::ir::Instruction;

/// Preserves both operands across unboxing and transfers exactly one predecessor owner.
pub(super) fn lower(ctx: &mut FunctionContext<'_>, inst: &Instruction) -> Result<()> {
    for operand in &inst.operands {
        ctx.load_value_to_result(*operand)?;
        abi::emit_push_reg(ctx.emitter, abi::int_result_reg(ctx.emitter));
    }
    let result = abi::int_result_reg(ctx.emitter);
    abi::emit_load_temporary_stack_slot(ctx.emitter, result, 0);
    abi::emit_call_label(ctx.emitter, "__rt_mixed_unbox");
    let done = ctx.next_label("exception_chain_mixed_done");
    match ctx.emitter.target.arch {
        Arch::AArch64 => {
            ctx.emitter.instruction("cmp x0, #6");                              // only an actual Throwable object can become an exception predecessor
            ctx.emitter.instruction(&format!("b.ne {done}"));                   // ignore absent or malformed compiler-private pending state
        }
        Arch::X86_64 => {
            ctx.emitter.instruction("cmp rax, 6");                              // only an actual Throwable object can become an exception predecessor
            ctx.emitter.instruction(&format!("jne {done}"));                    // ignore absent or malformed compiler-private pending state
        }
    }
    abi::emit_reg_move(ctx.emitter, result, mixed_unbox_payload_reg(ctx.emitter.target));
    abi::emit_store_to_sp(ctx.emitter, result, 0);
    abi::emit_call_label(ctx.emitter, "__rt_incref");
    abi::emit_load_temporary_stack_slot(ctx.emitter, result, 16);
    let predecessor = if ctx.emitter.target.arch == Arch::AArch64 { "x1" } else { "rdi" };
    abi::emit_load_temporary_stack_slot(ctx.emitter, predecessor, 0);
    abi::emit_call_label(ctx.emitter, "__rt_exception_chain");
    ctx.emitter.label(&done);
    abi::emit_release_temporary_stack(ctx.emitter, 32);
    Ok(())
}
