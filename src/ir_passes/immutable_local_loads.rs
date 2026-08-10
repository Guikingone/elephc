//! Purpose:
//! Classifies integer `load_local` instructions whose slot value cannot change.
//! Makes those reads pure so CSE and LICM can reason about local-backed arithmetic.
//!
//! Called from:
//! - The fixed-point pass driver before checked-int specialization, CSE, and LICM.
//!
//! Key details:
//! - Only concrete integer PHP locals are considered; aliases, ref cells, statics,
//!   hidden temporaries, mixed storage, and any slot named by an unknown operation fail closed.
//! - A slot is immutable when it is a read-only incoming parameter/main `$argc`, or has
//!   exactly one entry-block store that dominates every load. The entry must have no
//!   predecessor, proving that the store executes once rather than once per loop iteration.

use std::collections::HashSet;

use crate::ir::{DataPool, Effects, Function, Immediate, InstId, IrType, LocalKind, LocalSlotId, Op};
use crate::types::PhpType;

use super::cfg::predecessors;
use super::dominance::compute_dominance;
use super::driver::IrPass;

/// Marks loads from proven-immutable scalar integer locals as pure.
pub struct ImmutableLocalLoads;

impl IrPass for ImmutableLocalLoads {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "immutable_local_loads"
    }

    /// Refines effects for every load whose local slot satisfies the immutable contract.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let eligible = immutable_integer_slots(function);
        if eligible.is_empty() {
            return false;
        }

        let mut changed = false;
        for inst in &mut function.instructions {
            if inst.op != Op::LoadLocal || inst.effects.is_pure() {
                continue;
            }
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
                continue;
            };
            if eligible.contains(&slot) {
                inst.effects = Effects::PURE;
                changed = true;
            }
        }
        changed
    }
}

/// Returns integer PHP-local slots whose contents are immutable for the whole function.
fn immutable_integer_slots(function: &Function) -> HashSet<LocalSlotId> {
    let mut eligible: HashSet<LocalSlotId> = function
        .locals
        .iter()
        .filter(|local| {
            local.kind == LocalKind::PhpLocal
                && local.ir_type == IrType::I64
                && matches!(local.php_type.codegen_repr(), PhpType::Int)
        })
        .map(|local| local.id)
        .collect();
    if eligible.is_empty() {
        return eligible;
    }

    let mut loads: Vec<Vec<InstId>> = vec![Vec::new(); function.locals.len()];
    let mut stores: Vec<Vec<InstId>> = vec![Vec::new(); function.locals.len()];
    for (raw, inst) in function.instructions.iter().enumerate() {
        let inst_id = InstId::from_raw(raw as u32);
        match inst.immediate.as_ref() {
            Some(Immediate::LocalSlot(slot)) if eligible.contains(slot) => match inst.op {
                Op::LoadLocal => loads[slot.as_raw() as usize].push(inst_id),
                Op::StoreLocal => stores[slot.as_raw() as usize].push(inst_id),
                _ => {
                    eligible.remove(slot);
                }
            },
            Some(Immediate::LocalSlotPair { first, second }) => {
                eligible.remove(first);
                eligible.remove(second);
            }
            _ => {}
        }
    }

    let dominance = compute_dominance(function);
    let locations = instruction_locations(function);
    let entry_has_no_predecessor = predecessors(function)
        .get(function.entry.as_raw() as usize)
        .is_some_and(Vec::is_empty);
    eligible.retain(|slot| {
        let slot_loads = &loads[slot.as_raw() as usize];
        if slot_loads.is_empty() {
            return false;
        }
        match stores[slot.as_raw() as usize].as_slice() {
            [] => slot_is_read_only_input(function, *slot),
            [store] if entry_has_no_predecessor => {
                store_dominates_loads(function, *store, slot_loads, &locations, &dominance)
            }
            _ => false,
        }
    });
    eligible
}

/// Maps every instruction table id to its current block and in-block position.
fn instruction_locations(function: &Function) -> Vec<Option<(crate::ir::BlockId, usize)>> {
    let mut locations = vec![None; function.instructions.len()];
    for block in &function.blocks {
        for (position, inst) in block.instructions.iter().copied().enumerate() {
            locations[inst.as_raw() as usize] = Some((block.id, position));
        }
    }
    locations
}

/// Returns true for an unwritten incoming integer parameter or top-level `$argc` slot.
fn slot_is_read_only_input(function: &Function, slot: LocalSlotId) -> bool {
    let Some(name) = function.locals[slot.as_raw() as usize].name.as_deref() else {
        return false;
    };
    (function.flags.is_main && name == "argc")
        || function.params.iter().any(|param| {
            param.name == name
                && !param.by_ref
                && !param.variadic
                && param.ir_type == IrType::I64
                && matches!(param.php_type.codegen_repr(), PhpType::Int)
        })
}

/// Proves that the sole store is in the entry and occurs before every local load.
fn store_dominates_loads(
    function: &Function,
    store: InstId,
    loads: &[InstId],
    locations: &[Option<(crate::ir::BlockId, usize)>],
    dominance: &super::dominance::DominanceInfo,
) -> bool {
    let Some((store_block, store_position)) = locations[store.as_raw() as usize] else {
        return false;
    };
    if store_block != function.entry {
        return false;
    }
    loads.iter().all(|load| {
        let Some((load_block, load_position)) = locations[load.as_raw() as usize] else {
            return false;
        };
        if load_block == store_block {
            store_position < load_position
        } else {
            dominance.dominates(store_block, load_block)
        }
    })
}
