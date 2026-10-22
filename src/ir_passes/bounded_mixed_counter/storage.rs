//! Purpose:
//! Reuses integer-range proofs to retain scalar storage for nonoverflowing loop counters.
//!
//! Called from:
//! - Numeric storage replay after every potentially promoting local has been widened.
//!
//! Key details:
//! - Trials are discarded; successful proofs only authorize another ordinary AST lowering.
//! - By-value call positions are supplied by authoritative lowering signatures.
//! - Unknown aliases, handlers, eval, undefined reads and unproven updates fail closed.

use super::*;

/// Finds counters whose entire recurrence remains integer, independently of their observers.
pub(crate) fn proven_integer_slots(
    function: &Function,
    data: &mut DataPool,
    readonly_calls: &HashMap<InstId, HashSet<usize>>,
) -> HashSet<LocalSlotId> {
    if function.instructions.len() > 4096
        || function.flags.is_generator || function.flags.is_fiber_wrapper
        || super::super::cfg::has_exception_handlers(function)
        || super::super::local_scope::has_dynamic_eval(function)
        || function.locals.iter().any(|local| matches!(local.kind,
            LocalKind::EvalContext | LocalKind::EvalScope | LocalKind::EvalGlobalScope | LocalKind::GeneratorState))
    { return HashSet::new(); }
    let candidates: Vec<_> = function.locals.iter().filter(|local| {
        local.kind == LocalKind::PhpLocal && local.php_type == PhpType::Mixed
            && local.ir_type == IrType::Heap(IrHeapKind::Mixed)
            && local.id.as_raw() as usize >= function.params.len()
            && !function.no_epilogue_cleanup_slots.contains(&local.id)
    }).map(|local| local.id).collect();
    // Mutually observed counters cannot assume each other's tags independently.
    // A joint trial proves all updates before retaining any of their scalar slots.
    let group: HashSet<_> = candidates.iter().copied().filter(|&slot| {
        function.instructions.iter().any(|inst| {
            inst.op == Op::StoreLocal && inst.immediate == Some(Immediate::LocalSlot(slot))
                && inst.operands.first().is_some_and(|&value| increment_writer(function, value))
        })
    }).take(16).collect();
    if group.len() > 1 {
        if let Some(proof) = CounterProof::for_slots(
            function, &group, &BoundedMixedCounter::default(), data, Some(readonly_calls),
        ) {
            if trial_safe(function, &group, &proof, data) { return group; }
        }
    }
    let mut proven = HashSet::new();
    let mut trials = 0;
    for slot in candidates {
        let Some(proof) = CounterProof::with_readonly_calls(
            function, slot, &BoundedMixedCounter::default(), data, Some(readonly_calls),
        ) else { continue; };
        if trials == 16 { break; }
        trials += 1;
        if trial_safe(function, &HashSet::from([slot]), &proof, data) {
            proven.insert(slot);
        }
    }
    proven
}

/// Follows ownership wrappers only; another local's increment is not this slot's recurrence.
fn increment_writer(function: &Function, mut value: ValueId) -> bool {
    for _ in 0..function.values.len() {
        let Some(inst) = super::defining_instruction(function, value) else { return false; };
        match inst.op {
            Op::StrIncDec => return true,
            Op::Acquire | Op::MixedClone | Op::Move | Op::Borrow | Op::EnsureOwned => {
                let Some(&source) = inst.operands.first() else { return false; };
                value = source;
            }
            _ => return false,
        }
    }
    false
}

/// Proves every update on a discarded scalar SSA trial, including nested-loop invariants.
fn trial_safe(function: &Function, slots: &HashSet<LocalSlotId>, proof: &CounterProof, data: &mut DataPool) -> bool {
    let mut trial = function.clone();
    // Discovery runs before finalization. An unfinished exit is a sink, never a backedge.
    for block in &mut trial.blocks {
        if block.terminator.is_none() {
            block.terminator = Some(crate::ir::Terminator::Unreachable);
        }
    }
    proof.scalarize_slots(&mut trial, slots);
    if crate::ir::validate_function(&trial).is_err() { return false; }
    super::super::mem2reg::Mem2Reg.run(&mut trial, data);
    if trial.instructions.iter().any(|inst| {
        matches!(inst.op, Op::LoadLocal | Op::StoreLocal)
            && matches!(inst.immediate, Some(Immediate::LocalSlot(slot)) if slots.contains(&slot))
    }) { return false; }
    for _ in 0..16 {
        if !super::super::loop_optimize::LoopOptimize.run(&mut trial, data) { break; }
    }
    super::super::integer_range::IntegerRange.run(&mut trial, data);
    proof.updates.iter().all(|id| trial.instruction(*id)
        .is_some_and(|inst| matches!(inst.op, Op::IAdd | Op::ISub)))
        && crate::ir::validate_function(&trial).is_ok()
}
