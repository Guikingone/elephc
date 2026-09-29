//! Purpose:
//! Promotes eligible scalar PHP local slots into EIR SSA values.
//!
//! Called from:
//! - The EIR fixed-point pass driver before scalar simplification and register allocation.
//!
//! Key details:
//! - Only fully defined, non-aliased integer, boolean, and float PHP locals qualify.
//! - Values live into CFG joins and loop headers use EIR block parameters.
//! - Exceptional control flow and implicit eval scope access retain memory slots.

use std::collections::{HashMap, HashSet};

use crate::ir::{
    BlockId, DataPool, Function, Immediate, InstId, IrType, LocalKind, LocalSlotId, Op,
    Ownership, Terminator, Value, ValueDef, ValueId,
};
use crate::types::PhpType;

use super::cfg::{has_exception_handlers, predecessors, reverse_postorder};
use super::dominance::{compute_dominance, DominanceInfo};
use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Promotes eligible local slots across the whole explicit CFG.
pub struct Mem2Reg;

impl IrPass for Mem2Reg {
    /// Returns the pass name used by validation diagnostics.
    fn name(&self) -> &'static str {
        "mem2reg"
    }

    /// Skips functions with no local loads at all.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(|inst| inst.op == Op::LoadLocal)
    }

    /// Promotes each independently eligible slot and reports whether IR changed.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        if function.flags.is_generator
            || has_exception_handlers(function)
            || function.instructions.iter().any(|inst| inst.op.name().starts_with("eval_"))
        {
            return false;
        }

        let slots = eligible_slots(function);
        if slots.is_empty() {
            return false;
        }
        let preds = predecessors(function);
        if !preds[function.entry.as_raw() as usize].is_empty() {
            return false;
        }
        let order = reverse_postorder(function);
        let reachable: HashSet<BlockId> = order.iter().copied().collect();
        let dominance = compute_dominance(function);
        let mut changed = false;
        for slot in slots {
            let facts = slot_facts(function, slot);
            let live_in = live_in(function, &facts, &order);
            let defined_in = defined_in(&facts, &preds, &order);
            if !loads_are_defined(function, slot, &defined_in, &reachable) {
                continue;
            }
            let mut phi_blocks = vec![false; function.blocks.len()];
            for &block in &order {
                let raw = block.as_raw() as usize;
                if live_in[raw] && preds[raw].len() > 1 {
                    if preds[raw].iter().any(|pred| !reachable.contains(pred))
                        || function.blocks[raw].params.len() >= u16::MAX as usize
                    {
                        phi_blocks.clear();
                        break;
                    }
                    phi_blocks[raw] = true;
                }
            }
            if phi_blocks.is_empty() {
                continue;
            }
            changed |= promote_slot(function, slot, &phi_blocks, &order, &dominance);
        }
        changed
    }
}

/// Returns scalar PHP locals whose entire observable use is plain load/store traffic.
fn eligible_slots(function: &Function) -> Vec<LocalSlotId> {
    let escaping = super::by_ref_alias::address_escaping_slots(function);
    let mut eligible: HashSet<LocalSlotId> = function
        .locals
        .iter()
        .filter(|local| local.kind == LocalKind::PhpLocal)
        .filter(|local| {
            matches!(
                (local.ir_type, local.php_type.codegen_repr()),
                (IrType::I64, PhpType::Int | PhpType::Bool)
                    | (IrType::F64, PhpType::Float)
            )
        })
        .filter(|local| !escaping.contains(&local.id))
        .filter(|local| {
            local.name.as_deref().is_some_and(|name| {
                !function.params.iter().any(|param| param.name == name)
                    && !(function.flags.is_main && matches!(name, "argc" | "argv"))
            })
        })
        .map(|local| local.id)
        .collect();

    let mut loads = HashSet::new();
    let mut stores = HashSet::new();
    let mut loaded_values = HashMap::new();
    for inst in &function.instructions {
        match inst.immediate.as_ref() {
            Some(Immediate::LocalSlot(slot)) if eligible.contains(slot) => {
                let local = &function.locals[slot.as_raw() as usize];
                match inst.op {
                    Op::LoadLocal if inst.result.is_some_and(|value| {
                        value_matches_local(function, value, local.ir_type, &local.php_type)
                    }) => {
                        loads.insert(*slot);
                        loaded_values.insert(inst.result.expect("checked result"), *slot);
                    }
                    Op::StoreLocal if inst.operands.first().is_some_and(|value| {
                        value_matches_local(function, *value, local.ir_type, &local.php_type)
                    }) => {
                        stores.insert(*slot);
                    }
                    _ => {
                        eligible.remove(slot);
                    }
                }
            }
            Some(Immediate::LocalSlotPair { first, second }) => {
                eligible.remove(first);
                eligible.remove(second);
            }
            Some(Immediate::IterStart(metadata)) => {
                for slot in metadata.local_slots() {
                    eligible.remove(&slot);
                }
            }
            _ => {}
        }
    }
    for block in &function.blocks {
        if let Some(term) = &block.terminator {
            for value in branch_arguments(term) {
                if let Some(slot) = loaded_values.get(&value) {
                    eligible.remove(slot);
                }
            }
        }
    }
    let mut result: Vec<_> = eligible
        .into_iter()
        .filter(|slot| loads.contains(slot) && stores.contains(slot))
        .collect();
    result.sort_unstable();
    result
}

/// Checks that forwarding a stored value preserves the local load representation.
fn value_matches_local(function: &Function, value: ValueId, ir_type: IrType, php_type: &PhpType) -> bool {
    function.value(value).is_some_and(|value| {
        value.ir_type == ir_type
            && value.php_type.codegen_repr() == php_type.codegen_repr()
            && value.ownership == Ownership::NonHeap
    })
}

/// Per-block facts for one slot's ordinary loads and stores.
struct SlotFacts {
    reads_before_store: Vec<bool>,
    stores: Vec<bool>,
}

/// Records whether each block reads the slot before writing it, and whether it writes it.
fn slot_facts(function: &Function, slot: LocalSlotId) -> SlotFacts {
    let mut reads_before_store = vec![false; function.blocks.len()];
    let mut stores = vec![false; function.blocks.len()];
    for block in &function.blocks {
        let raw = block.id.as_raw() as usize;
        for &inst_id in &block.instructions {
            let inst = &function.instructions[inst_id.as_raw() as usize];
            if inst.immediate != Some(Immediate::LocalSlot(slot)) {
                continue;
            }
            match inst.op {
                Op::LoadLocal if !stores[raw] => reads_before_store[raw] = true,
                Op::StoreLocal => stores[raw] = true,
                _ => {}
            }
        }
    }
    SlotFacts { reads_before_store, stores }
}

/// Computes where the slot's previous value may be needed across a block boundary.
fn live_in(function: &Function, facts: &SlotFacts, order: &[BlockId]) -> Vec<bool> {
    let mut live = vec![false; function.blocks.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for &block_id in order.iter().rev() {
            let raw = block_id.as_raw() as usize;
            let successors = function.blocks[raw]
                .terminator
                .as_ref()
                .map(super::cfg::successors)
                .unwrap_or_default();
            let next = facts.reads_before_store[raw]
                || (!facts.stores[raw]
                    && successors.iter().any(|succ| live[succ.as_raw() as usize]));
            if next != live[raw] {
                live[raw] = next;
                changed = true;
            }
        }
    }
    live
}

/// Computes which block entries are definitely preceded by a store to this slot.
fn defined_in(facts: &SlotFacts, preds: &[Vec<BlockId>], order: &[BlockId]) -> Vec<bool> {
    let mut incoming = vec![false; facts.stores.len()];
    let mut outgoing = facts.stores.clone();
    let entry = order[0];
    let mut changed = true;
    while changed {
        changed = false;
        for &block in order.iter().skip(1) {
            let raw = block.as_raw() as usize;
            let next_in = !preds[raw].is_empty()
                && preds[raw].iter().all(|pred| outgoing[pred.as_raw() as usize]);
            let next_out = next_in || facts.stores[raw];
            if incoming[raw] != next_in || outgoing[raw] != next_out {
                incoming[raw] = next_in;
                outgoing[raw] = next_out;
                changed = true;
            }
        }
    }
    incoming[entry.as_raw() as usize] = false;
    incoming
}

/// Rejects a slot when a reachable load can still observe its memory initialization.
fn loads_are_defined(
    function: &Function,
    slot: LocalSlotId,
    defined_in: &[bool],
    reachable: &HashSet<BlockId>,
) -> bool {
    for block in &function.blocks {
        if !reachable.contains(&block.id) {
            continue;
        }
        let mut defined = defined_in[block.id.as_raw() as usize];
        for &inst_id in &block.instructions {
            let inst = &function.instructions[inst_id.as_raw() as usize];
            if inst.immediate != Some(Immediate::LocalSlot(slot)) {
                continue;
            }
            match inst.op {
                Op::LoadLocal if !defined => return false,
                Op::StoreLocal => defined = true,
                _ => {}
            }
        }
    }
    true
}

/// Adds join parameters, renames loads and stores, and supplies incoming edge values.
fn promote_slot(
    function: &mut Function,
    slot: LocalSlotId,
    phi_blocks: &[bool],
    order: &[BlockId],
    dominance: &DominanceInfo,
) -> bool {
    let local = &function.locals[slot.as_raw() as usize];
    let (ir_type, php_type) = (local.ir_type, local.php_type.clone());
    let mut phi_values = vec![None; function.blocks.len()];
    for &block in order {
        let raw = block.as_raw() as usize;
        if !phi_blocks[raw] {
            continue;
        }
        let index = function.blocks[raw].params.len();
        let value = ValueId::from_raw(function.values.len() as u32);
        function.values.push(Value {
            ir_type,
            php_type: php_type.clone(),
            def: ValueDef::BlockParam { block, index: index as u16 },
            ownership: Ownership::NonHeap,
        });
        function.blocks[raw].params.push(value);
        phi_values[raw] = Some(value);
    }

    let mut end_values = vec![None; function.blocks.len()];
    let mut replacements = HashMap::new();
    let mut neutralized = Vec::new();
    rename_block(
        function,
        function.entry,
        slot,
        None,
        &phi_values,
        dominance,
        &mut end_values,
        &mut replacements,
        &mut neutralized,
    );
    for &block in order {
        let raw = block.as_raw() as usize;
        if let Some(term) = function.blocks[raw].terminator.as_mut() {
            append_edge_values(term, &phi_values, end_values[raw]);
        }
    }
    replace_all_uses(function, &resolve_chains(&replacements));
    for inst_id in &neutralized {
        neutralize_to_nop(&mut function.instructions[inst_id.as_raw() as usize]);
    }
    !neutralized.is_empty()
}

/// Walks the dominator tree while carrying the current SSA definition of one local.
#[allow(clippy::too_many_arguments)]
fn rename_block(
    function: &Function,
    block: BlockId,
    slot: LocalSlotId,
    incoming: Option<ValueId>,
    phi_values: &[Option<ValueId>],
    dominance: &DominanceInfo,
    end_values: &mut [Option<ValueId>],
    replacements: &mut HashMap<ValueId, ValueId>,
    neutralized: &mut Vec<InstId>,
) {
    let raw = block.as_raw() as usize;
    let mut current = phi_values[raw].or(incoming);
    for &inst_id in &function.blocks[raw].instructions {
        let inst = &function.instructions[inst_id.as_raw() as usize];
        if inst.immediate != Some(Immediate::LocalSlot(slot)) {
            continue;
        }
        match inst.op {
            Op::LoadLocal => {
                let value = inst.result.expect("eligible load has a result");
                replacements.insert(value, current.expect("eligible load is defined"));
                neutralized.push(inst_id);
            }
            Op::StoreLocal => {
                let mut value = inst.operands[0];
                while let Some(&next) = replacements.get(&value) {
                    value = next;
                }
                current = Some(value);
                neutralized.push(inst_id);
            }
            _ => {}
        }
    }
    end_values[raw] = current;
    for &child in dominance.children(block) {
        rename_block(
            function,
            child,
            slot,
            current,
            phi_values,
            dominance,
            end_values,
            replacements,
            neutralized,
        );
    }
}

/// Adds a predecessor's current value to every edge targeting a new join parameter.
fn append_edge_values(term: &mut Terminator, phi_values: &[Option<ValueId>], value: Option<ValueId>) {
    let add = |target: BlockId, args: &mut Vec<ValueId>| {
        if phi_values[target.as_raw() as usize].is_some() {
            args.push(value.expect("join predecessor has a definition"));
        }
    };
    match term {
        Terminator::Br { target, args } => add(*target, args),
        Terminator::CondBr { then_target, then_args, else_target, else_args, .. } => {
            add(*then_target, then_args);
            add(*else_target, else_args);
        }
        Terminator::Switch { cases, default, default_args, .. } => {
            for case in cases {
                add(case.target, &mut case.args);
            }
            add(*default, default_args);
        }
        Terminator::GeneratorSuspend { resume, resume_args, .. } => add(*resume, resume_args),
        Terminator::Return { .. }
        | Terminator::Throw { .. }
        | Terminator::Fatal { .. }
        | Terminator::Unreachable => {}
    }
}

/// Lists values carried on CFG edges, excluding ordinary terminator operands.
fn branch_arguments(term: &Terminator) -> Vec<ValueId> {
    match term {
        Terminator::Br { args, .. } => args.clone(),
        Terminator::CondBr { then_args, else_args, .. } => {
            then_args.iter().chain(else_args).copied().collect()
        }
        Terminator::Switch { cases, default_args, .. } => cases
            .iter()
            .flat_map(|case| case.args.iter().copied())
            .chain(default_args.iter().copied())
            .collect(),
        Terminator::GeneratorSuspend { resume_args, .. } => resume_args.clone(),
        _ => Vec::new(),
    }
}
