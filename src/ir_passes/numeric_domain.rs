//! Purpose:
//! Proves that an EIR value graph can contain only PHP integers or floats.
//!
//! Called from:
//! - `codegen::stack_numeric` before frame selection and scalar retirement.
//! - `ir_passes::concat_reset` before treating numeric updates as scratch-inert.
//!
//! Key details:
//! - Every local writer participates in the proof, including seeds and loop backedges.
//! - Calls, parameters with unknown tags, references and nonnumeric values fail closed.

use std::collections::{HashMap, HashSet};
use crate::ir::{Function, Immediate, IrHeapKind, IrType, LocalKind, LocalSlotId, Op, ValueId};
use crate::types::PhpType;
use crate::ir::MixedNumericOp;

/// Accepts only the three arithmetic operations with exact integer overflow-to-float semantics.
pub(crate) fn mixed(inst: &crate::ir::Instruction) -> bool {
    inst.op == Op::MixedNumericBinop && matches!(inst.immediate,
        Some(Immediate::MixedNumericOp(MixedNumericOp::Add | MixedNumericOp::Sub | MixedNumericOp::Mul)))
}

/// Recognizes only the two unary steps whose numeric semantics are checked addition.
pub(crate) fn increment(inst: &crate::ir::Instruction) -> bool {
    inst.op == Op::StrIncDec && matches!(inst.immediate, Some(Immediate::I64(-1 | 1)))
}

/// Recognizes ordered numeric predicates whose unordered floating-point result is false.
pub(crate) fn comparison(inst: &crate::ir::Instruction) -> bool {
    use crate::ir::CmpPredicate::{Slt, Sle, Sgt, Sge};
    inst.op == Op::PhpRelCmp && matches!(inst.immediate, Some(Immediate::CmpPredicate(Slt | Sle | Sgt | Sge)))
}

/// Checks a closed numeric graph, constructing dominance only for otherwise numeric private slots.
/// Local cycles are accepted inductively only after every writer and dominating seed passes.
pub(crate) fn inputs(function: &Function, values: &[ValueId]) -> bool {
    if function.instructions.iter().any(|inst| matches!(inst.op,
        Op::TryPushHandler | Op::CatchBind | Op::FinallyEnter))
    {
        return false;
    }
    let mut dominance = None;
    let mut slots = HashSet::new();
    let mut known = HashMap::new();
    values.iter().all(|&value| numeric(function, value, &mut slots, &mut dominance, &mut known))
}

/// Shares value decisions within one conjunctive proof, avoiding exponential shared-DAG walks.
///
/// Local cycles may provisionally succeed while their writers are visited. Every writer
/// must succeed for the whole proof to pass; a failed proof discards this entire cache.
fn numeric(function: &Function, value: ValueId, slots: &mut HashSet<LocalSlotId>, dominance: &mut Option<crate::ir_passes::DominanceInfo>, known: &mut HashMap<ValueId, bool>) -> bool {
    if let Some(&safe) = known.get(&value) { return safe; }
    let safe = numeric_uncached(function, value, slots, dominance, known);
    known.insert(value, safe);
    safe
}

/// Requires scalar metadata or a boxed value whose complete origin is numeric.
fn numeric_uncached(function: &Function, value: ValueId, slots: &mut HashSet<LocalSlotId>, dominance: &mut Option<crate::ir_passes::DominanceInfo>, known: &mut HashMap<ValueId, bool>) -> bool {
    let Some(metadata) = function.value(value) else { return false; };
    if matches!((&metadata.ir_type, &metadata.php_type),
        (IrType::I64, PhpType::Int) | (IrType::F64, PhpType::Float))
    {
        return true;
    }
    if metadata.ir_type != IrType::Heap(IrHeapKind::Mixed)
        || metadata.php_type.codegen_repr() != PhpType::Mixed
    {
        return false;
    }
    let Some(inst) = super::rewrite::defining_instruction(function, value) else { return false; };
    match inst.op {
        Op::Acquire | Op::MixedClone | Op::Move | Op::Borrow | Op::EnsureOwned | Op::MixedBox => {
            inst.operands.first().is_some_and(|&value| numeric(function, value, slots, dominance, known))
        }
        Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul => {
            inst.operands.len() == 2 && inst.operands.iter().all(|&value| numeric(function, value, slots, dominance, known))
        }
        Op::MixedNumericBinop if mixed(inst) => {
            inst.operands.len() == 2 && inst.operands.iter().all(|&value| numeric(function, value, slots, dominance, known))
        }
        Op::StrIncDec if increment(inst) => {
            inst.operands.len() == 1 && numeric(function, inst.operands[0], slots, dominance, known)
        }
        Op::LoadLocal => {
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else { return false; };
            let Some(local) = function.locals.get(slot.as_raw() as usize) else { return false; };
            if local.kind != LocalKind::PhpLocal || (slot.as_raw() as usize) < function.params.len() {
                return false;
            }
            if crate::ir_passes::address_escaping_slots(function).contains(&slot) {
                return false;
            }
            if function.instructions.iter().any(|inst| {
                inst.immediate == Some(Immediate::LocalSlot(slot))
                    && !matches!(inst.op, Op::LoadLocal | Op::StoreLocal | Op::ReleaseLocalSlot)
                    || matches!(inst.immediate, Some(Immediate::LocalSlotPair { first, second })
                        if first == slot || second == slot)
            }) {
                return false;
            }
            if !slots.insert(slot) { return true; }
            let writers: Vec<_> = function.instructions.iter().filter(|inst| {
                inst.immediate == Some(Immediate::LocalSlot(slot)) && inst.op == Op::StoreLocal
            }).collect();
            let safe = !writers.is_empty() && writers.iter().all(|writer| {
                writer.operands.first().is_some_and(|&value| numeric(function, value, slots, dominance, known))
            }) && initialized(function, slot, dominance.get_or_insert_with(|| {
                crate::ir_passes::compute_dominance(function)
            }));
            slots.remove(&slot);
            safe
        }
        _ => false,
    }
}

/// Requires a dominating store before every read, including reads feeding another slot's writer.
/// The enclosing origin proof checks every stored value's numeric domain separately, so a
/// computed numeric temporary can initialize its slot without requiring a literal seed.
fn initialized(function: &Function, slot: LocalSlotId, dominance: &crate::ir_passes::DominanceInfo) -> bool {
    let mut seeds = Vec::new();
    let mut loads = Vec::new();
    for block in &function.blocks {
        for (position, &id) in block.instructions.iter().enumerate() {
            let inst = &function.instructions[id.as_raw() as usize];
            if inst.immediate != Some(Immediate::LocalSlot(slot)) { continue; }
            if inst.op == Op::LoadLocal { loads.push((block.id, position)); }
            if inst.op == Op::StoreLocal {
                seeds.push((block.id, position));
            }
        }
    }
    loads.iter().all(|&(block, position)| seeds.iter().any(|&(seed_block, seed_position)| {
        if seed_block == block { seed_position < position }
        else { dominance.dominates(seed_block, block) }
    }))
}
