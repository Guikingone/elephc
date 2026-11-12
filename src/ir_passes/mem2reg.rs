//! Purpose:
//! Promotes scalar local slots (`int`/`float`/`bool` locals accessed only through
//! `load_local`/`store_local`) to SSA values, with block parameters at control-flow joins.
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - Eligibility reuses the dead-store pass rules (`PhpLocal`, no lifetime tracking, only
//!   plain loads/stores name the slot, no address escape, no eval-literal reads) and adds
//!   type agreement: every load and stored value has the slot's IR type and PHP
//!   representation, so replacing a load by the reaching value never changes typing.
//! - The entry block starts with one `load_local` per promoted slot, which supplies the
//!   slot's initial contents (a parameter or the frame's initial value) on every path; it is
//!   removed by dead-instruction elimination when no path needs it.
//! - Block parameters are placed on the iterated dominance frontier of each slot's stores
//!   (classic Cytron construction); a dominator-tree walk then replaces loads by the reaching
//!   value, drops the stores, and appends the reaching values to every edge's arguments.
//! - Functions with exception handlers or generator suspension points are skipped: their
//!   implicit control edges are not part of the terminator graph.

use std::collections::{HashMap, HashSet};

use crate::ir::{
    BlockId, DataPool, Function, Immediate, Instruction, IrType, LocalSlotId, Op, Ownership,
    Terminator, Value, ValueDef, ValueId,
};

use super::cfg::{has_exception_handlers, predecessors, successors};
use super::dominance::compute_dominance;
use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Pass promoting scalar local slots to SSA values.
pub(crate) struct Mem2Reg;

impl IrPass for Mem2Reg {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "mem2reg"
    }

    /// Promotes every eligible slot; returns whether anything changed.
    fn run(&self, function: &mut Function, data: &mut DataPool) -> bool {
        if function.flags.is_generator
            || has_exception_handlers(function)
            || function
                .blocks
                .iter()
                .any(|block| matches!(block.terminator, Some(Terminator::GeneratorSuspend { .. })))
        {
            return false;
        }
        let slots = promotable_slots(function, data);
        if slots.is_empty() {
            return false;
        }
        promote(function, &slots);
        true
    }
}

thread_local! {
    /// By-reference flags of every module function's parameters, keyed by function name.
    static CALLEE_REF_PARAMS: std::cell::RefCell<HashMap<String, Vec<bool>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Records the module's direct-call parameter by-reference flags for the next pass rounds.
pub(crate) fn set_callee_ref_params(map: HashMap<String, Vec<bool>>) {
    CALLEE_REF_PARAMS.with(|cell| *cell.borrow_mut() = map);
}

/// Returns whether `inst` reads operand `index` purely by value.
///
/// Beyond the dead-store value-only opcodes, array/hash/property data operations read their
/// scalar operands by value (only their container receivers, never scalar slots, are traced
/// back to slots), and a
/// direct `call` reads an argument by value when the resolved callee declares that parameter
/// by value. Every other consumer is treated as a potential by-reference escape.
fn reads_by_value(data: &DataPool, inst: &Instruction, index: usize) -> bool {
    use Op::*;
    if super::dead_store::op_is_value_only_consumer(inst.op) {
        return true;
    }
    match inst.op {
        ArrayGet | ArrayGetSilent | HashGet | HashGetSilent | ArrayIsset | HashIsset | ArraySet
        | HashSet | ArrayPush | HashAppend | ArrayKeyExists | PropGet | PropSet | ArrayNew
        | HashNew | ThrowCheckedReturnTypeError | ThrowCheckedTypeError => true,
        Call => {
            let Some(Immediate::Data(id)) = inst.immediate else {
                return false;
            };
            let Some(name) = data.function_names.get(id.as_raw() as usize) else {
                return false;
            };
            CALLEE_REF_PARAMS.with(|cell| {
                cell.borrow()
                    .get(&crate::names::php_symbol_key(name.trim_start_matches('\\')))
                    .and_then(|flags| flags.get(index).copied())
                    .is_some_and(|by_ref| !by_ref)
            })
        }
        _ => false,
    }
}

/// Returns the slots that can be promoted, in a stable order.
fn promotable_slots(function: &Function, data: &DataPool) -> Vec<LocalSlotId> {
    let eligible = super::dead_store::eligible_slots_with(function, data, &|inst, index| {
        reads_by_value(data, inst, index)
    });
    let mut slots: Vec<LocalSlotId> = eligible
        .into_iter()
        .filter(|slot| slot_types_agree(function, *slot))
        .collect();
    slots.sort_by_key(|slot| slot.as_raw());
    // A never-stored slot read by one entry-block load is already in SSA form.
    let entry = function.entry;
    slots.retain(|slot| {
        let mut loads = 0usize;
        let mut entry_loads = 0usize;
        for block in &function.blocks {
            for inst_id in &block.instructions {
                let Some(inst) = function.instruction(*inst_id) else {
                    continue;
                };
                if inst.immediate != Some(Immediate::LocalSlot(*slot)) {
                    continue;
                }
                if inst.op == Op::StoreLocal {
                    return true;
                }
                loads += 1;
                if block.id == entry {
                    entry_loads += 1;
                }
            }
        }
        loads > 1 || loads != entry_loads
    });
    slots
}

/// Returns whether every load and stored value of `slot` shares the slot's representation.
fn slot_types_agree(function: &Function, slot: LocalSlotId) -> bool {
    let Some(local) = function.locals.get(slot.as_raw() as usize) else {
        return false;
    };
    if !matches!(local.ir_type, IrType::I64 | IrType::F64) {
        return false;
    }
    let repr = local.php_type.codegen_repr();
    function.instructions.iter().all(|inst| {
        if inst.immediate != Some(Immediate::LocalSlot(slot)) {
            return true;
        }
        match inst.op {
            Op::LoadLocal => {
                inst.result_type == local.ir_type && inst.result_php_type.codegen_repr() == repr
            }
            Op::StoreLocal => inst.operands.first().and_then(|v| function.value(*v)).is_some_and(
                |value| value.ir_type == local.ir_type && value.php_type.codegen_repr() == repr,
            ),
            _ => false,
        }
    })
}

/// Rewrites `function` so the given slots live in SSA values.
fn promote(function: &mut Function, slots: &[LocalSlotId]) {
    let entry = function.entry;
    // -- initial values: one load per slot at the top of the entry block --
    let mut initial = HashMap::new();
    for (position, slot) in slots.iter().enumerate() {
        let local = &function.locals[slot.as_raw() as usize];
        let (ir_type, php_type) = (local.ir_type, local.php_type.clone());
        let value = ValueId::from_raw(function.values.len() as u32);
        let inst_id = crate::ir::InstId::from_raw(function.instructions.len() as u32);
        function.values.push(Value {
            ir_type,
            php_type: php_type.clone(),
            def: ValueDef::Instruction { block: entry, index: position as u32, inst: inst_id },
            ownership: Ownership::NonHeap,
        });
        function.instructions.push(Instruction {
            op: Op::LoadLocal,
            operands: Vec::new(),
            immediate: Some(Immediate::LocalSlot(*slot)),
            result: Some(value),
            result_type: ir_type,
            result_php_type: php_type,
            result_ownership: Ownership::NonHeap,
            effects: Op::LoadLocal.default_effects(),
            span: None,
            origin: None,
        });
        let entry_block = function.block_mut(entry).expect("entry block");
        entry_block.instructions.insert(position, inst_id);
        initial.insert(*slot, value);
    }
    renumber_block(function, entry);

    // -- dominance frontiers --
    let dominance = compute_dominance(function);
    let preds = predecessors(function);
    let block_count = function.blocks.len();
    let mut frontier: Vec<HashSet<BlockId>> = vec![HashSet::new(); block_count];
    for block in &function.blocks {
        let b = block.id;
        if !dominance.is_reachable(b) {
            continue;
        }
        let block_preds: Vec<BlockId> = preds[b.as_raw() as usize]
            .iter()
            .copied()
            .filter(|p| dominance.is_reachable(*p))
            .collect();
        if block_preds.len() < 2 {
            continue;
        }
        let Some(idom) = dominance.immediate_dominator(b) else {
            continue;
        };
        for p in block_preds {
            let mut runner = p;
            while runner != idom {
                frontier[runner.as_raw() as usize].insert(b);
                match dominance.immediate_dominator(runner) {
                    Some(next) => runner = next,
                    None => break,
                }
            }
        }
    }

    // -- slot liveness (pruned SSA: only joins where the slot is live-in get a parameter) --
    let slot_set: HashSet<LocalSlotId> = slots.iter().copied().collect();
    let live_in = slot_live_in(function, &slot_set);

    // -- phi (block parameter) placement --
    let mut phis: HashMap<BlockId, Vec<(LocalSlotId, ValueId)>> = HashMap::new();
    for slot in slots {
        let mut def_blocks: Vec<BlockId> = vec![entry];
        for block in &function.blocks {
            if block.instructions.iter().any(|id| {
                function.instruction(*id).is_some_and(|inst| {
                    inst.op == Op::StoreLocal && inst.immediate == Some(Immediate::LocalSlot(*slot))
                })
            }) {
                def_blocks.push(block.id);
            }
        }
        let mut has_phi: HashSet<BlockId> = HashSet::new();
        let mut worklist = def_blocks.clone();
        let mut queued: HashSet<BlockId> = def_blocks.into_iter().collect();
        while let Some(block) = worklist.pop() {
            for &join in &frontier[block.as_raw() as usize] {
                if !live_in[join.as_raw() as usize].contains(slot) {
                    continue;
                }
                if has_phi.insert(join) {
                    let local = &function.locals[slot.as_raw() as usize];
                    let (ir_type, php_type) = (local.ir_type, local.php_type.clone());
                    let value = ValueId::from_raw(function.values.len() as u32);
                    let join_block = function.block_mut(join).expect("join block");
                    let index = join_block.params.len() as u16;
                    join_block.params.push(value);
                    function.values.push(Value {
                        ir_type,
                        php_type,
                        def: ValueDef::BlockParam { block: join, index },
                        ownership: Ownership::NonHeap,
                    });
                    phis.entry(join).or_default().push((*slot, value));
                    if queued.insert(join) {
                        worklist.push(join);
                    }
                }
            }
        }
    }

    // -- renaming along the dominator tree --
    let mut replacements: HashMap<ValueId, ValueId> = HashMap::new();
    let mut current: HashMap<LocalSlotId, Vec<ValueId>> =
        slots.iter().map(|slot| (*slot, vec![initial[slot]])).collect();
    let mut stack: Vec<(BlockId, bool)> = vec![(entry, false)];
    let mut pushed: HashMap<BlockId, Vec<LocalSlotId>> = HashMap::new();
    while let Some((block_id, exiting)) = stack.pop() {
        if exiting {
            for slot in pushed.remove(&block_id).unwrap_or_default() {
                current.get_mut(&slot).expect("slot stack").pop();
            }
            continue;
        }
        stack.push((block_id, true));
        let mut defined = Vec::new();
        for (slot, value) in phis.get(&block_id).cloned().unwrap_or_default() {
            current.get_mut(&slot).expect("slot stack").push(value);
            defined.push(slot);
        }
        let instruction_ids = function.block(block_id).expect("block").instructions.clone();
        for inst_id in instruction_ids {
            let Some(inst) = function.instruction(inst_id) else {
                continue;
            };
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
                continue;
            };
            if !slot_set.contains(&slot) {
                continue;
            }
            // The entry initial loads define the slot; they are kept as real loads.
            if initial.get(&slot) == inst.result.as_ref() {
                continue;
            }
            match inst.op {
                Op::LoadLocal => {
                    let result = inst.result.expect("load result");
                    let reaching = *current[&slot].last().expect("reaching value");
                    replacements.insert(result, resolve(&replacements, reaching));
                    neutralize_to_nop(function.instruction_mut(inst_id).expect("inst"));
                }
                Op::StoreLocal => {
                    let stored = resolve(&replacements, inst.operands[0]);
                    current.get_mut(&slot).expect("slot stack").push(stored);
                    defined.push(slot);
                    neutralize_to_nop(function.instruction_mut(inst_id).expect("inst"));
                }
                _ => {}
            }
        }
        // Append the reaching values to every outgoing edge whose target has phis.
        let reaching: HashMap<LocalSlotId, ValueId> = slots
            .iter()
            .map(|slot| (*slot, resolve(&replacements, *current[slot].last().expect("value"))))
            .collect();
        if let Some(term) = function.block_mut(block_id).and_then(|b| b.terminator.as_mut()) {
            let edge = |target: BlockId, args: &mut Vec<ValueId>| {
                for (slot, _) in phis.get(&target).map(Vec::as_slice).unwrap_or(&[]) {
                    args.push(reaching[slot]);
                }
            };
            match term {
                Terminator::Br { target, args } => edge(*target, args),
                Terminator::CondBr { then_target, then_args, else_target, else_args, .. } => {
                    edge(*then_target, then_args);
                    edge(*else_target, else_args);
                }
                Terminator::Switch { cases, default, default_args, .. } => {
                    for case in cases.iter_mut() {
                        edge(case.target, &mut case.args);
                    }
                    edge(*default, default_args);
                }
                _ => {}
            }
        }
        pushed.insert(block_id, defined);
        for &child in dominance.children(block_id).iter().rev() {
            stack.push((child, false));
        }
    }
    let resolved = resolve_chains(&replacements);
    replace_all_uses(function, &resolved);
}

/// Computes, per block, the promoted slots whose current value may be read later.
///
/// A slot is live-in when some path from the block's entry loads it before storing it.
fn slot_live_in(function: &Function, slots: &HashSet<LocalSlotId>) -> Vec<HashSet<LocalSlotId>> {
    let block_count = function.blocks.len();
    let mut gen: Vec<HashSet<LocalSlotId>> = vec![HashSet::new(); block_count];
    let mut kill: Vec<HashSet<LocalSlotId>> = vec![HashSet::new(); block_count];
    let mut succs: Vec<Vec<BlockId>> = vec![Vec::new(); block_count];
    for block in &function.blocks {
        let index = block.id.as_raw() as usize;
        for inst_id in &block.instructions {
            let Some(inst) = function.instruction(*inst_id) else {
                continue;
            };
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else {
                continue;
            };
            if !slots.contains(&slot) {
                continue;
            }
            match inst.op {
                Op::LoadLocal if !kill[index].contains(&slot) => {
                    gen[index].insert(slot);
                }
                Op::StoreLocal => {
                    kill[index].insert(slot);
                }
                _ => {}
            }
        }
        if let Some(term) = &block.terminator {
            succs[index] = successors(term);
        }
    }
    let mut live_in = gen.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for index in (0..block_count).rev() {
            let mut live: HashSet<LocalSlotId> = HashSet::new();
            for succ in &succs[index] {
                live.extend(live_in[succ.as_raw() as usize].iter().copied());
            }
            live.retain(|slot| !kill[index].contains(slot));
            live.extend(gen[index].iter().copied());
            if live.len() != live_in[index].len() {
                live_in[index] = live;
                changed = true;
            }
        }
    }
    live_in
}

/// Follows a replacement chain to its final value.
fn resolve(replacements: &HashMap<ValueId, ValueId>, mut value: ValueId) -> ValueId {
    while let Some(next) = replacements.get(&value) {
        value = *next;
    }
    value
}

/// Rewrites the definition positions of every instruction result in `block`.
fn renumber_block(function: &mut Function, block: BlockId) {
    let ids = function.block(block).expect("block").instructions.clone();
    for (index, inst_id) in ids.into_iter().enumerate() {
        let Some(result) = function.instruction(inst_id).and_then(|inst| inst.result) else {
            continue;
        };
        if let Some(value) = function.value_mut(result) {
            value.def = ValueDef::Instruction { block, index: index as u32, inst: inst_id };
        }
    }
}
