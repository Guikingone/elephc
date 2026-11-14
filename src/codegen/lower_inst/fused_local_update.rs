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
    if let Some((checked_id, _)) = update.checked {
        let checked = instruction(ctx, checked_id)?;
        emit_checked_into_mixed_fast_path(ctx, &checked, &binop, &slow)?;
    } else if binop.op == Op::MixedNumericBinop {
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
    let mut group: Vec<InstId> = Vec::new();
    if let Some((checked_id, _)) = update.checked {
        group.push(checked_id);
    }
    group.push(binop_id);
    if let Some((_, checked_release)) = update.checked {
        group.push(checked_release);
    }
    group.extend([update.release, update.store]);
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

/// Emits the native int path of a mixed `+`/`-`/`*` whose operand is a fused checked int
/// operation (`$sum += $a * $b`): the checked operation runs natively, the other operand must be
/// a native int or a box holding an int, and neither operation may overflow; otherwise branches
/// to `slow`. On fall-through the int result is in x9 (AArch64) / rax (x86_64).
fn emit_checked_into_mixed_fast_path(
    ctx: &mut FunctionContext<'_>,
    checked: &Instruction,
    binop: &Instruction,
    slow: &str,
) -> Result<()> {
    let checked_result = checked
        .result
        .ok_or_else(|| CodegenIrError::invalid_module("checked int operation without a result"))?;
    let (first, second) = match binop.operands.as_slice() {
        [first, second] => (*first, *second),
        _ => return Err(CodegenIrError::invalid_module("mixed operation without two operands")),
    };
    let checked_is_left = first == checked_result;
    let other = if checked_is_left { second } else { first };
    let other_is_box = ctx.value_php_type(other)?.codegen_repr() == crate::types::PhpType::Mixed;
    let op = match binop.immediate {
        Some(Immediate::MixedNumericOp(op)) => op,
        _ => return Err(CodegenIrError::invalid_module("mixed operation without an operator")),
    };
    // The checked operation itself, into x9 / rdx.
    emit_checked_int_fast_path(ctx, checked, slow)?;
    match ctx.emitter.target.arch {
        Arch::X86_64 => {
            ctx.emitter.instruction("mov rdx, rax");                            // the checked int result
            if other_is_box {
                ctx.load_value_to_reg(other, "r11")?;
                ctx.emitter.instruction("test r11, r11");                       // no box: generic path
                ctx.emitter.instruction(&format!("jz {}", slow));
                ctx.emitter.instruction("cmp QWORD PTR [r11], 0");              // the box must hold an int
                ctx.emitter.instruction(&format!("jne {}", slow));
                ctx.emitter.instruction("mov rax, QWORD PTR [r11 + 8]");        // the boxed int
            } else {
                ctx.load_value_to_reg(other, "rax")?;
            }
            if checked_is_left {
                ctx.emitter.instruction("mov rcx, rax");                        // the other operand on the right
                ctx.emitter.instruction("mov rax, rdx");                        // the checked result on the left
            } else {
                ctx.emitter.instruction("mov rcx, rdx");                        // the checked result on the right
            }
            let mnemonic = match op {
                crate::ir::MixedNumericOp::Add => "add",
                crate::ir::MixedNumericOp::Sub => "sub",
                _ => "imul",
            };
            ctx.emitter.instruction(&format!("{} rax, rcx", mnemonic));         // native int operation
            ctx.emitter.instruction(&format!("jo {}", slow));                   // overflow: generic path (float result)
        }
        Arch::AArch64 => {
            ctx.emitter.instruction("mov x11, x9");                             // the checked int result
            if other_is_box {
                ctx.load_value_to_reg(other, "x16")?;
                ctx.emitter.instruction(&format!("cbz x16, {}", slow));         // no box: generic path
                ctx.emitter.instruction("ldr x17, [x16]");                      // the box tag
                ctx.emitter.instruction(&format!("cbnz x17, {}", slow));        // the box must hold an int
                ctx.emitter.instruction("ldr x10, [x16, #8]");                  // the boxed int
            } else {
                ctx.load_value_to_reg(other, "x10")?;
            }
            let (lhs, rhs) = if checked_is_left { ("x11", "x10") } else { ("x10", "x11") };
            match op {
                crate::ir::MixedNumericOp::Add => {
                    ctx.emitter.instruction(&format!("adds x9, {}, {}", lhs, rhs)); // add and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // overflow: generic path (float result)
                }
                crate::ir::MixedNumericOp::Sub => {
                    ctx.emitter.instruction(&format!("subs x9, {}, {}", lhs, rhs)); // subtract and set the overflow flag
                    ctx.emitter.instruction(&format!("b.vs {}", slow));         // overflow: generic path (float result)
                }
                _ => {
                    ctx.emitter.instruction(&format!("mul x9, {}, {}", lhs, rhs)); // low 64 bits of the product
                    ctx.emitter.instruction(&format!("smulh x17, {}, {}", lhs, rhs)); // high 64 bits of the product
                    ctx.emitter.instruction("cmp x17, x9, asr #63");            // does the high half only extend the sign?
                    ctx.emitter.instruction(&format!("b.ne {}", slow));         // overflow: generic path (float result)
                }
            }
        }
    }
    Ok(())
}
