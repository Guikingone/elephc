//! Purpose:
//! Proves receiver stability and the fatal null paths of buffer-property accesses.
//!
//! Called from:
//! - The buffer property reuse and loop hoisting passes.
//!
//! Key details:
//! - A foreign receiver may warn on null even when its static object type is non-nullable.
//! - An immediate buffer access rejects the null read's handle zero before execution can resume.
//! - Borrowed inliner bindings can be forwarded only across a unique dominating store.

use std::collections::HashMap;

use crate::ir::{Effects, Function, Immediate, InstId, LocalKind, Op, ValueDef, ValueId};
use crate::types::PhpType;

use super::buffer_access::{buffer_handle_value, slots_named_as_storage};
use super::by_ref_alias::address_escaping_slots;
use super::dominance::compute_dominance;
use super::loops::NaturalLoop;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Requires a raw access of this read's handle before any observable operation or branch.
/// A null receiver warns, but returns handle zero; that access fatals before a warning handler's
/// buffer mutations could invalidate any subsequently used cached descriptor.
pub(super) fn traps_on_null(function: &Function, read: &crate::ir::Instruction) -> bool {
    null_trapping_access(function, read).is_some()
}

/// Returns the checked access ending a property read's non-observable straight-line prefix.
pub(super) fn null_trapping_access(function: &Function, read: &crate::ir::Instruction) -> Option<InstId> {
    let handle = read.result?;
    let ValueDef::Instruction { block, index, .. } = function.value(handle)?.def else { return None; };
    for &id in function.block(block)?.instructions.get(index as usize + 1..)? {
        let inst = &function.instructions[id.as_raw() as usize];
        if matches!(inst.op, Op::BufferGet | Op::BufferSet)
            && inst.operands.first().is_some_and(|&value| buffer_handle_value(function, value) == handle)
        { return Some(id); }
        if inst.op == Op::Acquire
            && inst.operands.first().is_some_and(|&value| buffer_handle_value(function, value) == handle)
        { continue; }
        if !super::buffer_property_loops::can_duplicate(function, id) { return None; }
    }
    None
}

/// Accepts an available SSA receiver or an exact local load of storage the loop cannot change.
pub(super) fn invariant_receiver(function: &Function, loop_ref: &NaturalLoop, receiver: ValueId) -> bool {
    let Some(value) = function.value(receiver) else { return false; };
    if !matches!(value.php_type, PhpType::Object(_)) { return false; }
    match value.def {
        ValueDef::BlockParam { block, .. } => !loop_ref.contains(block),
        ValueDef::Instruction { block, inst, .. } if loop_ref.contains(block) => {
            let load = &function.instructions[inst.as_raw() as usize];
            let Some(Immediate::LocalSlot(slot)) = load.immediate else { return false; };
            let written = slots_named_as_storage(loop_ref.blocks.iter().flat_map(|&block| {
                function.blocks[block.as_raw() as usize].instructions.iter()
                    .map(|id| &function.instructions[id.as_raw() as usize])
            }));
            load.op == Op::LoadLocal
                && can_load_receiver(function, inst)
                && !written.contains(&slot)
                && !address_escaping_slots(function).contains(&slot)
                && !function.params.get(slot.as_raw() as usize).is_some_and(|param| param.by_ref)
        }
        ValueDef::Instruction { .. } => true,
    }
}

/// Allows cloning a borrowed object word, never acquiring or releasing its lifetime.
pub(super) fn can_load_receiver(function: &Function, id: InstId) -> bool {
    let inst = &function.instructions[id.as_raw() as usize];
    let Some(Immediate::LocalSlot(slot)) = inst.immediate else { return false; };
    let Some(local) = function.locals.get(slot.as_raw() as usize) else { return false; };
    inst.op == Op::LoadLocal
        && local.kind == LocalKind::PhpLocal
        && matches!(local.php_type, PhpType::Object(_))
        && local.ir_type == inst.result_type && local.php_type == inst.result_php_type
        && inst.effects.difference(Effects::READS_LOCAL).is_empty()
        && matches!(inst.result_ownership, crate::ir::Ownership::Borrowed | crate::ir::Ownership::MaybeOwned)
}

/// Removes inliner-only borrowed parameter copies whose unique store dominates every load.
/// Slot-address consumers and ownership/type conversions are excluded before SSA forwarding.
pub(super) fn forward_borrowed_bindings(function: &mut Function) -> bool {
    let dominance = compute_dominance(function);
    let escaping = address_escaping_slots(function);
    let mut replacements = HashMap::new();
    let mut removed = Vec::new();
    for local in &function.locals {
        if local.kind != LocalKind::BorrowedTemp || escaping.contains(&local.id) { continue; }
        if !matches!(local.php_type, PhpType::Object(_) | PhpType::Int | PhpType::Float | PhpType::Bool | PhpType::Buffer(_)) {
            continue;
        }
        let mut stores = Vec::new();
        let mut loads = Vec::new();
        let mut invalid = false;
        for block in &function.blocks {
            for (index, &id) in block.instructions.iter().enumerate() {
                let inst = &function.instructions[id.as_raw() as usize];
                if inst.immediate == Some(Immediate::LocalSlot(local.id)) {
                    match inst.op {
                        Op::LoadLocal => loads.push((block.id, index, id)),
                        Op::StoreLocal => stores.push((block.id, index, id)),
                        _ => invalid = true,
                    }
                } else if slots_named_as_storage(std::iter::once(inst)).contains(&local.id) {
                    invalid = true;
                }
            }
        }
        if invalid || stores.len() != 1 || loads.is_empty() { continue; }
        let (block, index, store) = stores[0];
        let Some(&source) = function.instructions[store.as_raw() as usize].operands.first() else { continue; };
        let value = &function.values[source.as_raw() as usize];
        if value.ir_type != local.ir_type || value.php_type != local.php_type { continue; }
        if !loads.iter().all(|&(load_block, load_index, id)| {
            let load = &function.instructions[id.as_raw() as usize];
            load.result_type == value.ir_type && load.result_php_type == value.php_type
                && load.result_ownership == value.ownership
                && load.effects.difference(Effects::READS_LOCAL).is_empty()
                && if block == load_block { index < load_index } else { dominance.dominates(block, load_block) }
        }) { continue; }
        removed.push(store);
        for (_, _, id) in loads {
            replacements.insert(function.instructions[id.as_raw() as usize].result.unwrap(), source);
            removed.push(id);
        }
    }
    if removed.is_empty() { return false; }
    replace_all_uses(function, &resolve_chains(&replacements));
    for id in removed { neutralize_to_nop(&mut function.instructions[id.as_raw() as usize]); }
    true
}
