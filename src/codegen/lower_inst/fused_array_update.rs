//! Purpose:
//! Lowers `$a[$i] op= $x` on a list of `mixed` elements (`array_get` → mixed `+`/`-`/`*` →
//! `array_set` of the same element → release) as an in-place update of the element's box.
//!
//! Called from:
//! - `crate::codegen::block_emit` for the instruction groups `collect_fused_array_updates`
//!   recorded in `FunctionContext::fused_array_updates`.
//!
//! Key details:
//! - Fast path: the list and the element box are both uniquely owned (refcount 1), the index
//!   is in bounds, the box holds an int and the native operation does not overflow: the new
//!   int is written into the box payload. Nothing is allocated, retained or released, which is
//!   exactly the net effect of the unfused sequence (the old box would be released and a fresh
//!   one with the same refcount stored in the same slot).
//! - Any other case runs the original instructions (checked operand, read, operation,
//!   releases, store) on a slow path; caller-saved registers live across the group are
//!   preserved there only, so the fast path stays free of spills.
//! - The right operand is a native int, or the fused checked int operation producing it (its
//!   overflow also takes the slow path, which recomputes it).

use crate::codegen::context::{FunctionContext, FusedArrayUpdate};
use crate::codegen::platform::Arch;
use crate::codegen::{abi, CodegenIrError, Result};
use crate::ir::{Immediate, InstId, Instruction, MixedNumericOp, Op};

/// The null-array sentinel a missing receiver is represented by.
const NULL_ARRAY_SENTINEL: i64 = 0x7FFF_FFFF_FFFF_FFFE;

/// Lowers one fused `$a[$i] op= $x` group at the position of its mixed operation.
pub(crate) fn lower_fused_array_update(
    ctx: &mut FunctionContext<'_>,
    binop_id: InstId,
    update: &FusedArrayUpdate,
) -> Result<()> {
    let binop = instruction(ctx, binop_id)?;
    let get = instruction(ctx, update.get)?;
    let checked = update.checked.map(|id| instruction(ctx, id)).transpose()?;
    let op = match binop.immediate {
        Some(Immediate::MixedNumericOp(op)) => op,
        _ => return Err(CodegenIrError::invalid_module("fused array update without an operation")),
    };
    let slow = ctx.next_label("array_update_slow");
    let done = ctx.next_label("array_update_done");
    let array = get.operands[0];
    let index = get.operands[1];
    match ctx.emitter.target.arch {
        Arch::X86_64 => {
            // The right int operand into rcx.
            match &checked {
                Some(checked) => {
                    ctx.load_value_to_reg(checked.operands[0], "rax")?;
                    ctx.load_value_to_reg(checked.operands[1], "rcx")?;
                    emit_x86_int_op(ctx, checked_op(checked.op), "rax", "rcx", &slow);
                    ctx.emitter.instruction("mov rcx, rax");                    // the checked operand as a native int
                }
                None => {
                    ctx.load_value_to_reg(binop.operands[1], "rcx")?;
                }
            }
            ctx.load_value_to_reg(array, "r11")?;
            ctx.load_value_to_reg(index, "rdx")?;
            ctx.emitter.instruction("test r11, r11");                           // a missing list takes the generic path
            ctx.emitter.instruction(&format!("jz {}", slow));
            abi::emit_load_int_immediate(ctx.emitter, "r10", NULL_ARRAY_SENTINEL);
            ctx.emitter.instruction("cmp r11, r10");                            // the null-array sentinel takes the generic path
            ctx.emitter.instruction(&format!("je {}", slow));
            ctx.emitter.instruction("cmp DWORD PTR [r11 - 12], 1");             // a shared list must be separated first
            ctx.emitter.instruction(&format!("jne {}", slow));
            ctx.emitter.instruction("cmp rdx, QWORD PTR [r11]");                // index within the length (unsigned: negative is out)
            ctx.emitter.instruction(&format!("jae {}", slow));
            ctx.emitter.instruction("mov r11, QWORD PTR [r11 + rdx * 8 + 24]"); // the element box
            ctx.emitter.instruction("test r11, r11");                           // no box: generic path
            ctx.emitter.instruction(&format!("jz {}", slow));
            ctx.emitter.instruction("cmp QWORD PTR [r11], 0");                  // the box must hold an int
            ctx.emitter.instruction(&format!("jne {}", slow));
            ctx.emitter.instruction("cmp DWORD PTR [r11 - 12], 1");             // a shared box must not change in place
            ctx.emitter.instruction(&format!("jne {}", slow));
            ctx.emitter.instruction("mov rax, QWORD PTR [r11 + 8]");            // the element int
            emit_x86_int_op(ctx, op, "rax", "rcx", &slow);
            ctx.emitter.instruction("mov QWORD PTR [r11 + 8], rax");            // write the new int into the box
            ctx.emitter.instruction(&format!("jmp {}", done));
        }
        Arch::AArch64 => {
            // The right int operand into x10.
            match &checked {
                Some(checked) => {
                    ctx.load_value_to_reg(checked.operands[0], "x0")?;
                    ctx.load_value_to_reg(checked.operands[1], "x10")?;
                    emit_aarch64_int_op(ctx, checked_op(checked.op), "x0", "x10", &slow);
                    ctx.emitter.instruction("mov x10, x0");                     // the checked operand as a native int
                }
                None => {
                    ctx.load_value_to_reg(binop.operands[1], "x10")?;
                }
            }
            ctx.load_value_to_reg(array, "x11")?;
            ctx.load_value_to_reg(index, "x9")?;
            ctx.emitter.instruction(&format!("cbz x11, {}", slow));             // a missing list takes the generic path
            abi::emit_load_int_immediate(ctx.emitter, "x16", NULL_ARRAY_SENTINEL);
            ctx.emitter.instruction("cmp x11, x16");                            // the null-array sentinel takes the generic path
            ctx.emitter.instruction(&format!("b.eq {}", slow));
            ctx.emitter.instruction("ldur w16, [x11, #-12]");                   // list refcount
            ctx.emitter.instruction("cmp w16, #1");                             // a shared list must be separated first
            ctx.emitter.instruction(&format!("b.ne {}", slow));
            ctx.emitter.instruction("ldr x16, [x11]");                          // list length
            ctx.emitter.instruction("cmp x9, x16");                             // index within the length (unsigned: negative is out)
            ctx.emitter.instruction(&format!("b.hs {}", slow));
            ctx.emitter.instruction("add x16, x11, #24");                       // payload base
            ctx.emitter.instruction("ldr x11, [x16, x9, lsl #3]");              // the element box
            ctx.emitter.instruction(&format!("cbz x11, {}", slow));             // no box: generic path
            ctx.emitter.instruction("ldr x16, [x11]");                          // the box tag
            ctx.emitter.instruction(&format!("cbnz x16, {}", slow));            // the box must hold an int
            ctx.emitter.instruction("ldur w16, [x11, #-12]");                   // box refcount
            ctx.emitter.instruction("cmp w16, #1");                             // a shared box must not change in place
            ctx.emitter.instruction(&format!("b.ne {}", slow));
            ctx.emitter.instruction("ldr x0, [x11, #8]");                       // the element int
            emit_aarch64_int_op(ctx, op, "x0", "x10", &slow);
            ctx.emitter.instruction("str x0, [x11, #8]");                       // write the new int into the box
            ctx.emitter.instruction(&format!("b {}", done));
        }
    }

    ctx.emitter.label(&slow);
    let mut group: Vec<InstId> = Vec::new();
    group.extend(update.checked);
    group.push(update.get);
    group.push(binop_id);
    group.extend(update.after.iter().copied());
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

/// Returns a cloned instruction of the current function.
pub(super) fn instruction(ctx: &FunctionContext<'_>, id: InstId) -> Result<Instruction> {
    ctx.function
        .instruction(id)
        .cloned()
        .ok_or_else(|| CodegenIrError::missing_entry("instruction", id.as_raw()))
}

/// Maps a checked int opcode to the matching native operation.
fn checked_op(op: Op) -> MixedNumericOp {
    match op {
        Op::ICheckedAdd => MixedNumericOp::Add,
        Op::ICheckedSub => MixedNumericOp::Sub,
        _ => MixedNumericOp::Mul,
    }
}

/// Emits `lhs = lhs op rhs` on x86_64, branching to `overflow` when the result does not fit.
fn emit_x86_int_op(ctx: &mut FunctionContext<'_>, op: MixedNumericOp, lhs: &str, rhs: &str, overflow: &str) {
    let mnemonic = match op {
        MixedNumericOp::Add => "add",
        MixedNumericOp::Sub => "sub",
        _ => "imul",
    };
    ctx.emitter.instruction(&format!("{} {}, {}", mnemonic, lhs, rhs));        // native int operation
    ctx.emitter.instruction(&format!("jo {}", overflow));                       // overflow: generic path (float result)
}

/// Emits `lhs = lhs op rhs` on AArch64, branching to `overflow` when the result does not fit.
fn emit_aarch64_int_op(ctx: &mut FunctionContext<'_>, op: MixedNumericOp, lhs: &str, rhs: &str, overflow: &str) {
    match op {
        MixedNumericOp::Add => {
            ctx.emitter.instruction(&format!("adds {}, {}, {}", lhs, lhs, rhs)); // add and set the overflow flag
            ctx.emitter.instruction(&format!("b.vs {}", overflow));             // overflow: generic path (float result)
        }
        MixedNumericOp::Sub => {
            ctx.emitter.instruction(&format!("subs {}, {}, {}", lhs, lhs, rhs)); // subtract and set the overflow flag
            ctx.emitter.instruction(&format!("b.vs {}", overflow));             // overflow: generic path (float result)
        }
        _ => {
            ctx.emitter.instruction(&format!("mul x17, {}, {}", lhs, rhs));     // low 64 bits of the product
            ctx.emitter.instruction(&format!("smulh x16, {}, {}", lhs, rhs));   // high 64 bits of the product
            ctx.emitter.instruction("cmp x16, x17, asr #63");                   // does the high half only extend the sign?
            ctx.emitter.instruction(&format!("b.ne {}", overflow));             // overflow: generic path (float result)
            ctx.emitter.instruction(&format!("mov {}, x17", lhs));              // the product
        }
    }
}

/// Merges the caller-saved register saves of every instruction of the group (one per value).
pub(super) fn merged_call_saves(ctx: &FunctionContext<'_>, group: &[InstId]) -> Vec<crate::ir_passes::CallSave> {
    let mut merged: Vec<crate::ir_passes::CallSave> = Vec::new();
    for id in group {
        for save in ctx.allocation.call_saves(*id) {
            match merged.iter_mut().find(|known| known.value == save.value) {
                Some(known) => {
                    known.store_before |= save.store_before;
                    known.reload_after |= save.reload_after;
                }
                None => merged.push(*save),
            }
        }
    }
    merged
}
