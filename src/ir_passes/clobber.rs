//! Purpose:
//! Classifies which EIR opcodes and terminators are "volatile-safe": their
//! `codegen` lowering emits no call and touches only the fixed result and
//! scratch registers, never the caller-saved registers the allocator may hand
//! out to non-call-crossing intervals.
//!
//! Called from:
//! - `crate::ir_passes::intervals` to mark each live interval as call-free.
//!
//! Key details:
//! - Safe-by-default in the correctness direction: the allowlist returns `true`
//!   only for opcodes whose lowering was audited to (a) emit no `bl`/`call`,
//!   (b) clobber a register set contained in {result, x9/x10/x11 and d0/d1 on
//!   AArch64; rax/rdx/rcx/r10/r11 and xmm0/xmm1 on x86_64}, and (c) load their
//!   SSA operands before using any scratch. Every other opcode falls through to
//!   `false`, so a forgotten opcode only loses an optimization opportunity — it
//!   can never place a live value in a register the lowering then clobbers.
//! - Because the caller-saved pools (`x12`–`x15`, `d16`–`d23`, `rsi`/`rdi`/`r8`/
//!   `r9`, `xmm2`–`xmm7`) are disjoint from every register the allowlisted
//!   lowerings touch, a value that lives only across allowlisted ops keeps its
//!   caller-saved register intact with no prologue save/restore.

use std::collections::HashSet;

use crate::ir::{Function, Immediate, Instruction, LocalKind, LocalSlotId, Op, Terminator, ValueDef};
use crate::types::PhpType;

/// Returns true when `op`'s lowering neither emits a call nor touches a
/// caller-saved register outside the fixed result/scratch set.
///
/// Used to decide whether a live interval spanning this instruction can safely
/// keep a value in a caller-saved register. The default is `false`: only the
/// audited pure-compute opcodes below are treated as volatile-safe.
pub(super) fn op_is_volatile_safe(op: Op) -> bool {
    use Op::*;
    matches!(
        op,
        // Constants materialized directly into the result register.
        ConstI64 | ConstBool | ConstNull | ConstF64
        // Integer arithmetic, bitwise, and shift: result + secondary/tertiary
        // scratch only (see `lower_inst::arithmetic`).
        | IAdd | ISub | IMul | INeg
        | IBitAnd | IBitOr | IBitXor | IBitNot
        | IShl | IShrA | ISMod | IDiv
        // Floating-point arithmetic: d0/d1 or xmm0/xmm1 only (`lower_inst::floats`).
        | FAdd | FSub | FMul | FDiv | FNeg
        // Integer/float comparisons: result + secondary scratch only.
        | ICmp | FCmp
        // Scalar int<->float conversions: inline scvtf/fcvtzs / cvtsi2sd/cvttsd2si.
        | IToF | FToI
        | Nop
    )
}

/// Returns true when `inst`'s lowering neither emits a call on its hot path nor touches a
/// caller-saved register the allocator may hand out: the volatile-safe opcodes, plus indexed
/// reads of scalar arrays (`int`/`float`/`bool`/`string`/`int|null` elements), whose only call
/// is the cold miss warning made through a register-preserving wrapper.
pub(crate) fn inst_is_volatile_safe(
    func: &Function,
    inst: &Instruction,
    ref_cell_slots: &HashSet<LocalSlotId>,
) -> bool {
    op_is_volatile_safe(inst.op)
        || is_scalar_indexed_array_get(func, inst)
        || is_plain_local_load(func, inst, ref_cell_slots)
}

/// Returns whether `inst` loads a plain local slot (never a reference cell) into a value of the
/// same representation: the lowering is one frame load into the result register(s).
fn is_plain_local_load(func: &Function, inst: &Instruction, ref_cell_slots: &HashSet<LocalSlotId>) -> bool {
    if inst.op != Op::LoadLocal {
        return false;
    }
    let Some(Immediate::LocalSlot(slot)) = inst.immediate.as_ref() else {
        return false;
    };
    if ref_cell_slots.contains(slot) {
        return false;
    }
    let Some(local) = func.locals.get(slot.as_raw() as usize) else {
        return false;
    };
    matches!(
        local.kind,
        LocalKind::PhpLocal | LocalKind::HiddenTemp | LocalKind::BorrowedTemp | LocalKind::OwnedTemp
    ) && local.php_type.codegen_repr() == inst.result_php_type.codegen_repr()
}

/// Returns every local slot that may hold a reference cell somewhere in `func`: by-reference
/// parameters and slots targeted by a ref-cell-producing instruction (mirrors
/// `crate::codegen::local_analysis`). Loads of any other slot are plain frame loads.
pub(crate) fn ref_cell_slots(func: &Function) -> HashSet<LocalSlotId> {
    let mut slots: HashSet<LocalSlotId> = func
        .params
        .iter()
        .enumerate()
        .filter(|(_, param)| param.by_ref)
        .map(|(index, _)| LocalSlotId::from_raw(index as u32))
        .collect();
    for inst in &func.instructions {
        match (inst.op, inst.immediate.as_ref()) {
            (
                Op::PromoteLocalRefCell | Op::AliasLocalRefCell | Op::AdoptRefCell | Op::LocalRefEnsure,
                Some(Immediate::LocalSlotPair { first, second }),
            ) => {
                slots.insert(*first);
                slots.insert(*second);
            }
            (_, Some(Immediate::LocalSlot(slot)))
                if !matches!(inst.op, Op::LoadLocal | Op::StoreLocal | Op::ReleaseLocalSlot) =>
            {
                // Any other slot-addressing instruction (ref binds, foreach by reference,
                // unset, ...) may change the slot's representation: stay conservative.
                slots.insert(*slot);
            }
            (_, Some(Immediate::LocalSlotPair { first, second })) => {
                slots.insert(*first);
                slots.insert(*second);
            }
            (Op::ClosureCapture, _) => {
                for operand in &inst.operands {
                    if let Some(value) = func.value(*operand) {
                        if let ValueDef::Instruction { inst: def, .. } = value.def {
                            if let Some(Immediate::LocalSlot(slot)) =
                                func.instruction(def).and_then(|def| def.immediate.as_ref())
                            {
                                slots.insert(*slot);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    slots
}

/// Returns whether `inst` reads an element of an indexed array of unboxed scalars.
pub(crate) fn is_scalar_indexed_array_get(func: &Function, inst: &Instruction) -> bool {
    let scalar = |ty: &PhpType| {
        matches!(
            ty.codegen_repr(),
            PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Str | PhpType::TaggedScalar
        )
    };
    matches!(inst.op, Op::ArrayGet | Op::ArrayGetSilent)
        && inst.operands.len() == 2
        && scalar(&inst.result_php_type)
        && func.value(inst.operands[0]).is_some_and(|array| match array.php_type.codegen_repr() {
            PhpType::Array(elem) => scalar(&elem),
            _ => false,
        })
}

/// Returns true when `term`'s lowering neither emits a call nor clobbers a
/// caller-saved register holding another live value.
///
/// Branches, multi-way switches, and returns only materialize their already
/// computed condition/scrutinee/return value through the standard chokepoints;
/// throws, generator suspends, fatals, and unreachable are conservatively unsafe.
pub(super) fn terminator_is_volatile_safe(term: &Terminator) -> bool {
    matches!(
        term,
        Terminator::Br { .. }
            | Terminator::CondBr { .. }
            | Terminator::Switch { .. }
            | Terminator::Return { .. }
    )
}
