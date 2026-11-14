//! Purpose:
//! Lowers `$local = $a op $b` on a plain `mixed` local (mixed `+`/`-`/`*`, or a checked int one
//! boxing its `int|float` result → release of the local's previous value → store) as an
//! in-place update of the local's own box.
//!
//! Called from:
//! - `crate::codegen::block_emit` for the instruction groups `collect_fused_local_updates`
//!   recorded in `FunctionContext::fused_local_updates`.
//!
//! Key details:
//! - Fast path: both operands hold ints, the native operation does not overflow, and the box
//!   the local holds is a heap box owned by the local alone (refcount 1) whose payload is a
//!   scalar (int, float, bool or null, so no heap payload needs releasing): the int result is
//!   written into that box. This is the net effect of the unfused sequence, which would free
//!   that box and store a fresh one holding the same int.
//! - Any other case runs the original instructions (operation, release, store) on a slow
//!   path; caller-saved registers live across the group are preserved there only. The fast
//!   path touches result and scratch registers only.

use crate::codegen::context::{FunctionContext, FusedLocalUpdate};
use crate::codegen::platform::Arch;
use crate::codegen::{abi, CodegenIrError, Result};
use crate::ir::{Immediate, InstId, Instruction, Op};

use super::fused_array_update::{instruction, merged_call_saves};

/// Mixed tags whose payload owns no heap storage: int (0), float (2), bool (3) and null (8).
const SCALAR_TAG_MASK: u32 = (1 << 0) | (1 << 2) | (1 << 3) | (1 << 8);

/// Lowers one fused `$local = $a op $b` group at the position of its mixed operation.
pub(crate) fn lower_fused_local_update(
    ctx: &mut FunctionContext<'_>,
    binop_id: InstId,
    update: FusedLocalUpdate,
) -> Result<()> {
    let binop = instruction(ctx, binop_id)?;
    let store = instruction(ctx, update.store)?;
    let slot = match store.immediate {
        Some(Immediate::LocalSlot(slot)) => slot,
        _ => return Err(CodegenIrError::invalid_module("fused local update without a slot")),
    };
    let offset = ctx.local_offset(slot)?;
    let slow = ctx.next_label("local_update_slow");
    let done = ctx.next_label("local_update_done");
    if binop.op == Op::MixedNumericBinop {
        super::arithmetic::emit_mixed_numeric_int_fast_path(ctx, &binop, &slow)?;
    } else {
        emit_checked_int_fast_path(ctx, &binop, &slow)?;
    }
    match ctx.emitter.target.arch {
        Arch::X86_64 => {
            // The int result is in rax.
            abi::load_at_offset(ctx.emitter, "r11", offset);                   // the box the local holds
            crate::codegen_support::abi::emit_symbol_address(ctx.emitter, "r10", "_heap_buf");
            ctx.emitter.instruction("cmp r11, r10");                            // below the heap (null, sentinels, static data): generic path
            ctx.emitter.instruction(&format!("jb {}", slow));
            crate::codegen_support::abi::emit_symbol_address(ctx.emitter, "rcx", "_heap_off");
            ctx.emitter.instruction("add r10, QWORD PTR [rcx]");                // the live heap end
            ctx.emitter.instruction("cmp r11, r10");                            // beyond the live heap: generic path
            ctx.emitter.instruction(&format!("jae {}", slow));
            ctx.emitter.instruction("cmp DWORD PTR [r11 - 12], 1");             // a shared box must not change in place
            ctx.emitter.instruction(&format!("jne {}", slow));
            ctx.emitter.instruction("mov r10, QWORD PTR [r11]");                // the box tag
            ctx.emitter.instruction("cmp r10, 8");                              // tags above null hold heap payloads
            ctx.emitter.instruction(&format!("ja {}", slow));
            ctx.emitter.instruction(&format!("mov ecx, {}", SCALAR_TAG_MASK));  // tags without a heap payload
            ctx.emitter.instruction("bt ecx, r10d");                            // does the box hold a scalar?
            ctx.emitter.instruction(&format!("jnc {}", slow));
            ctx.emitter.instruction("mov QWORD PTR [r11], 0");                  // the box now holds an int
            ctx.emitter.instruction("mov QWORD PTR [r11 + 8], rax");            // the new int payload
            ctx.emitter.instruction("mov QWORD PTR [r11 + 16], 0");             // clear the high payload word
            ctx.emitter.instruction(&format!("jmp {}", done));
        }
        Arch::AArch64 => {
            // The int result is in x9.
            // x17 computes far slot addresses: the default scratch x9 holds the result.
            abi::load_at_offset_scratch(ctx.emitter, "x11", offset, "x17");    // the box the local holds
            crate::codegen_support::abi::emit_symbol_address(ctx.emitter, "x10", "_heap_buf");
            ctx.emitter.instruction("cmp x11, x10");                            // below the heap (null, sentinels, static data): generic path
            ctx.emitter.instruction(&format!("b.lo {}", slow));
            crate::codegen_support::abi::emit_symbol_address(ctx.emitter, "x16", "_heap_off");
            ctx.emitter.instruction("ldr x16, [x16]");                          // the live heap extent
            ctx.emitter.instruction("add x10, x10, x16");                       // the live heap end
            ctx.emitter.instruction("cmp x11, x10");                            // beyond the live heap: generic path
            ctx.emitter.instruction(&format!("b.hs {}", slow));
            ctx.emitter.instruction("ldur w16, [x11, #-12]");                   // box refcount
            ctx.emitter.instruction("cmp w16, #1");                             // a shared box must not change in place
            ctx.emitter.instruction(&format!("b.ne {}", slow));
            ctx.emitter.instruction("ldr x16, [x11]");                          // the box tag
            ctx.emitter.instruction("cmp x16, #8");                             // tags above null hold heap payloads
            ctx.emitter.instruction(&format!("b.hi {}", slow));
            ctx.emitter.instruction(&format!("mov x17, #{}", SCALAR_TAG_MASK)); // tags without a heap payload
            ctx.emitter.instruction("lsr x17, x17, x16");                       // bring the tag's bit to bit 0
            ctx.emitter.instruction(&format!("tbz x17, #0, {}", slow));         // not a scalar: generic path
            ctx.emitter.instruction("str xzr, [x11]");                          // the box now holds an int
            ctx.emitter.instruction("str x9, [x11, #8]");                       // the new int payload
            ctx.emitter.instruction("str xzr, [x11, #16]");                     // clear the high payload word
            ctx.emitter.instruction(&format!("b {}", done));
        }
    }

    ctx.emitter.label(&slow);
    let group = [binop_id, update.release, update.store];
    let saves = merged_call_saves(ctx, &group);
    for save in &saves {
        if save.store_before {
            let offset = ctx.value_frame_offset(save.value)?;
            abi::store_at_offset(ctx.emitter, save.reg, offset); // preserve a caller-saved register value across the generic update
        }
        ctx.allocation.suspend(save.value);
    }
    for id in &group {
        super::lower_instruction(ctx, *id)?;
    }
    for save in &saves {
        ctx.allocation.resume(save.value, save.reg);
        if save.reload_after {
            let offset = ctx.value_frame_offset(save.value)?;
            abi::load_at_offset(ctx.emitter, save.reg, offset); // reload the preserved caller-saved register value
        }
    }
    ctx.emitter.label(&done);
    Ok(())
}

/// Emits the native int operation of a checked `+`/`-`/`*`, branching to `slow` on overflow
/// (the checked helper then produces the float result). On fall-through the int result is in
/// x9 (AArch64) / rax (x86_64); only result and scratch registers are touched.
fn emit_checked_int_fast_path(ctx: &mut FunctionContext<'_>, inst: &Instruction, slow: &str) -> Result<()> {
    let (lhs, rhs) = match inst.operands.as_slice() {
        [lhs, rhs] => (*lhs, *rhs),
        _ => return Err(CodegenIrError::invalid_module("checked int operation without two operands")),
    };
    match ctx.emitter.target.arch {
        Arch::X86_64 => {
            ctx.load_value_to_reg(lhs, "rax")?;
            ctx.load_value_to_reg(rhs, "rcx")?;
            let mnemonic = match inst.op {
                Op::ICheckedAdd => "add",
                Op::ICheckedSub => "sub",
                _ => "imul",
            };
            ctx.emitter.instruction(&format!("{} rax, rcx", mnemonic));          // native int operation
            ctx.emitter.instruction(&format!("jo {}", slow));                   // overflow: the checked helper yields a float
        }
        Arch::AArch64 => {
            ctx.load_value_to_reg(lhs, "x0")?;
            ctx.load_value_to_reg(rhs, "x10")?;
            match inst.op {
                Op::ICheckedAdd => {
                    ctx.emitter.instruction("adds x9, x0, x10");                // add and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // overflow: the checked helper yields a float
                }
                Op::ICheckedSub => {
                    ctx.emitter.instruction("subs x9, x0, x10");                // subtract and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // overflow: the checked helper yields a float
                }
                _ => {
                    ctx.emitter.instruction("mul x9, x0, x10");                 // low 64 bits of the product
                    ctx.emitter.instruction("smulh x11, x0, x10");              // high 64 bits of the product
                    ctx.emitter.instruction("cmp x11, x9, asr #63");            // does the high half only extend the sign?
                    ctx.emitter.instruction(&format!("b.ne {}", slow));         // overflow: the checked helper yields a float
                }
            }
        }
    }
    Ok(())
}
