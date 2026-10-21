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
    NumericInputs::new(function).query(values, false)
}

/// Retains completed numeric facts for one immutably borrowed function.
/// Provisional cycle facts stay private until the entire conjunctive query succeeds.
pub(crate) struct NumericInputs<'a> {
    function: &'a Function,
    enabled: bool,
    proven: HashSet<ValueId>,
    working: HashMap<ValueId, bool>,
}

impl<'a> NumericInputs<'a> {
    /// Binds all cached facts to an immutable function and its handler policy.
    pub(crate) fn new(function: &'a Function) -> Self {
        let enabled = !function.instructions.iter().any(|inst| matches!(inst.op,
            Op::TryPushHandler | Op::CatchBind | Op::FinallyEnter));
        Self { function, enabled, proven: HashSet::new(), working: HashMap::new() }
    }

    /// Publishes only complete successes; a failed query contributes no provisional facts.
    pub(crate) fn prove(&mut self, values: &[ValueId]) -> bool {
        self.query(values, true)
    }

    /// Avoids retaining a second value set when the caller needs only one query.
    fn query(&mut self, values: &[ValueId], retain: bool) -> bool {
        self.working.clear();
        if !self.enabled { return false; }
        let mut dominance = None;
        let mut slots = HashSet::new();
        let safe = values.iter().all(|&value| numeric(
            self.function, value, &mut slots, &mut dominance, &self.proven, &mut self.working,
        ));
        if safe && retain {
            self.proven.extend(self.working.iter().filter_map(|(&value, &safe)| safe.then_some(value)));
        }
        self.working.clear();
        safe
    }

    /// Freezes complete facts for codegen after the function's frame proof is finished.
    pub(crate) fn into_proven_values(self) -> HashSet<ValueId> {
        self.proven
    }
}

/// Shares value decisions within one conjunctive proof, avoiding exponential shared-DAG walks.
///
/// Local cycles may provisionally succeed while their writers are visited. Every writer
/// must succeed for the whole proof to pass; a failed proof discards this entire cache.
fn numeric(function: &Function, value: ValueId, slots: &mut HashSet<LocalSlotId>, dominance: &mut Option<crate::ir_passes::DominanceInfo>, proven: &HashSet<ValueId>, known: &mut HashMap<ValueId, bool>) -> bool {
    if proven.contains(&value) { return true; }
    if let Some(&safe) = known.get(&value) { return safe; }
    let safe = numeric_uncached(function, value, slots, dominance, proven, known);
    known.insert(value, safe);
    safe
}

/// Requires scalar metadata or a boxed value whose complete origin is numeric.
fn numeric_uncached(function: &Function, value: ValueId, slots: &mut HashSet<LocalSlotId>, dominance: &mut Option<crate::ir_passes::DominanceInfo>, proven: &HashSet<ValueId>, known: &mut HashMap<ValueId, bool>) -> bool {
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
            inst.operands.first().is_some_and(|&value| numeric(function, value, slots, dominance, proven, known))
        }
        Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul => {
            inst.operands.len() == 2 && inst.operands.iter().all(|&value| numeric(function, value, slots, dominance, proven, known))
        }
        Op::MixedNumericBinop if mixed(inst) => {
            inst.operands.len() == 2 && inst.operands.iter().all(|&value| numeric(function, value, slots, dominance, proven, known))
        }
        Op::StrIncDec if increment(inst) => {
            inst.operands.len() == 1 && numeric(function, inst.operands[0], slots, dominance, proven, known)
        }
        Op::LoadLocal => {
            let Some(Immediate::LocalSlot(slot)) = inst.immediate else { return false; };
            let Some(local) = function.locals.get(slot.as_raw() as usize) else { return false; };
            if local.kind != LocalKind::PhpLocal || (slot.as_raw() as usize) < function.params.len() {
                return false;
            }
            // Active slots already passed the immutable access checks. Their outer
            // visit still requires every writer and dominating initialization.
            if slots.contains(&slot) { return true; }
            #[cfg(test)]
            SLOT_VALIDATIONS.with(|checks| checks.set(checks.get() + 1));
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
            slots.insert(slot);
            let writers: Vec<_> = function.instructions.iter().filter(|inst| {
                inst.immediate == Some(Immediate::LocalSlot(slot)) && inst.op == Op::StoreLocal
            }).collect();
            let safe = !writers.is_empty() && writers.iter().all(|writer| {
                writer.operands.first().is_some_and(|&value| numeric(function, value, slots, dominance, proven, known))
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

#[cfg(test)]
std::thread_local! {
    /// Counts expensive slot eligibility checks within this test thread's proof.
    static SLOT_VALIDATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Builder, Ownership, Terminator};

    /// Builds a shared local cycle and returns its final load and scalar witnesses.
    fn shared_cycle() -> (Function, ValueId, ValueId, ValueId) {
        let mut function = Function::new("numeric_domain_work".into(), IrType::I64, PhpType::Int);
        let slot = function.add_local(
            Some("sum".into()), IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, LocalKind::PhpLocal,
        );
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let zero = builder.emit_const_i64(0);
        let one = builder.emit_const_i64(1);
        let nonnumeric = builder.emit_const_bool(false);
        builder.emit_store_local(slot, zero);
        for _ in 0..128 {
            let previous = builder.emit_load_local(slot, IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed);
            let next = builder.emit(
                Op::MixedNumericBinop, vec![previous, one],
                Some(Immediate::MixedNumericOp(MixedNumericOp::Add)),
                IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned,
            ).unwrap();
            builder.emit_store_local(slot, next);
        }
        let loaded = builder.emit_load_local(slot, IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed);
        builder.terminate(Terminator::Return { value: Some(zero) });
        (function, loaded, nonnumeric, one)
    }

    /// Shared local cycles validate accesses once while retaining all-writer checks.
    #[test]
    fn numeric_domain_reuses_prechecked_slot_eligibility() {
        let (mut function, loaded, nonnumeric, _) = shared_cycle();
        SLOT_VALIDATIONS.with(|checks| checks.set(0));
        assert!(inputs(&function, &[loaded]));
        assert_eq!(SLOT_VALIDATIONS.with(|checks| checks.get()), 1,
            "revisited loads must reuse the active slot's validated accesses");

        let mut tainted = function.clone();
        let writer = tainted.instructions.iter_mut().rev().find(|inst| inst.op == Op::StoreLocal).unwrap();
        writer.operands[0] = nonnumeric;
        assert!(!inputs(&tainted, &[loaded]), "a later nonnumeric writer must still reject the whole cycle");

        let seed = function.instructions.iter_mut().find(|inst| inst.op == Op::StoreLocal).unwrap();
        seed.op = Op::Nop;
        seed.immediate = None;
        seed.operands.clear();
        seed.effects = Op::Nop.default_effects();
        assert!(!inputs(&function, &[loaded]), "an active cycle must not initialize its own entry value");
    }

    /// Complete facts survive unrelated failures and eliminate repeated function-local queries.
    #[test]
    fn numeric_domain_reuses_only_completed_function_facts() {
        let (function, loaded, nonnumeric, _) = shared_cycle();
        let mut proof = NumericInputs::new(&function);
        SLOT_VALIDATIONS.with(|checks| checks.set(0));
        assert!(proof.prove(&[loaded]));
        let completed = proof.proven.clone();
        for inst in function.instructions.iter().filter(|inst| inst.op == Op::LoadLocal) {
            assert!(proof.prove(&[inst.result.unwrap()]));
        }
        assert!(!proof.prove(&[nonnumeric]));
        assert_eq!(proof.proven, completed);
        assert!(proof.working.is_empty());
        assert!(proof.prove(&[loaded]));
        assert_eq!(SLOT_VALIDATIONS.with(|checks| checks.get()), 1);
    }

    /// Failed cycles and conjunctions never publish provisional successes into later queries.
    #[test]
    fn numeric_domain_failed_queries_do_not_publish_cycle_facts() {
        let (mut function, loaded, nonnumeric, one) = shared_cycle();
        let first_load = function.instructions.iter().find(|inst| inst.op == Op::LoadLocal).unwrap().result.unwrap();
        let writer = function.instructions.iter_mut().rev().find(|inst| inst.op == Op::StoreLocal).unwrap();
        writer.operands[0] = nonnumeric;
        let mut proof = NumericInputs::new(&function);
        assert!(proof.prove(&[one]));
        assert!(!proof.prove(&[loaded]));
        assert_eq!(proof.proven, HashSet::from([one]));
        assert!(proof.working.is_empty());
        assert!(!proof.prove(&[first_load]));
        assert_eq!(proof.proven, HashSet::from([one]));

        let (valid, loaded, nonnumeric, _) = shared_cycle();
        let mut proof = NumericInputs::new(&valid);
        assert!(!proof.prove(&[loaded, nonnumeric]));
        assert!(proof.proven.is_empty());
        assert!(proof.working.is_empty());
        assert!(proof.prove(&[loaded]));
    }
}
