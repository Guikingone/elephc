//! Purpose:
//! Specializes boxed checked integer add/sub/mul when every observation narrows to `int`.
//! Removes the transient Mixed allocation without changing PHP overflow semantics.
//!
//! Called from:
//! - The fixed-point pass driver after peephole and immutable-local classification.
//!
//! Key details:
//! - The accepted use graph contains only integer casts/stores plus ordinary acquire/release
//!   scaffolding. Returns, output, calls, comparisons, Mixed stores, terminators, lifetime pins,
//!   and unknown/future opcodes reject the candidate.
//! - Rewritten producers yield non-owning I64 values through `IChecked*ToInt`; removable casts
//!   and ownership instructions are neutralized only after every use has been proven safe.

use std::collections::{HashMap, HashSet};

use crate::ir::{
    DataPool, Function, Immediate, InstId, IrType, LocalSlotId, Op, Ownership, ValueId,
};
use crate::types::PhpType;

use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Rewrites checked arithmetic whose complete use graph observes only a PHP integer.
pub struct CheckedIntSink;

impl IrPass for CheckedIntSink {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "checked_int_sink"
    }

    /// Specializes every independently proven checked-arithmetic producer in the function.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        let uses = instruction_uses(function);
        let terminator_uses = terminator_used_values(function);
        let candidates: Vec<InstId> = function
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(raw, inst)| checked_to_int_op(inst.op).map(|_| InstId::from_raw(raw as u32)))
            .collect();
        let mut changed = false;

        for candidate in candidates {
            let Some(inst) = function.instruction(candidate) else {
                continue;
            };
            let Some(result) = inst.result else {
                continue;
            };
            let mut rewrite = SinkRewrite::default();
            if !analyze_sink_graph(
                function,
                result,
                &uses,
                &terminator_uses,
                &mut HashSet::new(),
                &mut rewrite,
            ) || !rewrite.saw_integer_sink
            {
                continue;
            }

            apply_specialization(function, candidate, result, rewrite);
            changed = true;
        }
        changed
    }
}

/// Deferred mutations collected while proving one candidate's entire use graph.
#[derive(Default)]
struct SinkRewrite {
    replacements: HashMap<ValueId, ValueId>,
    neutralize: HashSet<InstId>,
    saw_integer_sink: bool,
}

/// Builds the instruction-use adjacency list for every SSA value.
fn instruction_uses(function: &Function) -> HashMap<ValueId, Vec<InstId>> {
    let mut uses: HashMap<ValueId, Vec<InstId>> = HashMap::new();
    for (raw, inst) in function.instructions.iter().enumerate() {
        let inst_id = InstId::from_raw(raw as u32);
        for operand in &inst.operands {
            uses.entry(*operand).or_default().push(inst_id);
        }
    }
    uses
}

/// Returns every SSA value read by a terminator, which is never an integer-only sink here.
fn terminator_used_values(function: &Function) -> HashSet<ValueId> {
    function
        .blocks
        .iter()
        .filter_map(|block| block.terminator.as_ref())
        .flat_map(super::liveness::terminator_uses)
        .collect()
}

/// Proves that every use of `value` is an accepted integer sink or removable scaffold.
fn analyze_sink_graph(
    function: &Function,
    value: ValueId,
    uses: &HashMap<ValueId, Vec<InstId>>,
    terminator_uses: &HashSet<ValueId>,
    visited: &mut HashSet<ValueId>,
    rewrite: &mut SinkRewrite,
) -> bool {
    if !visited.insert(value) {
        return true;
    }
    if terminator_uses.contains(&value) {
        return false;
    }
    let Some(value_uses) = uses.get(&value) else {
        return false;
    };
    for use_id in value_uses {
        let Some(user) = function.instruction(*use_id) else {
            return false;
        };
        match user.op {
            Op::Cast if matches!(user.immediate, Some(Immediate::CastTarget(IrType::I64))) => {
                let Some(cast_result) = user.result else {
                    return false;
                };
                rewrite.replacements.insert(cast_result, value);
                rewrite.neutralize.insert(*use_id);
                rewrite.saw_integer_sink = true;
            }
            Op::StoreLocal | Op::StoreStaticLocal | Op::InitStaticLocal
                if local_store_is_integer(function, user.immediate.as_ref()) =>
            {
                rewrite.saw_integer_sink = true;
            }
            Op::StoreRefCell
                if matches!(user.result_php_type.codegen_repr(), PhpType::Int) =>
            {
                rewrite.saw_integer_sink = true;
            }
            Op::Acquire if user.immediate.is_none() => {
                let Some(acquired) = user.result else {
                    return false;
                };
                if !analyze_sink_graph(
                    function,
                    acquired,
                    uses,
                    terminator_uses,
                    visited,
                    rewrite,
                ) {
                    return false;
                }
                rewrite.replacements.insert(acquired, value);
                rewrite.neutralize.insert(*use_id);
            }
            Op::Release => {
                rewrite.neutralize.insert(*use_id);
            }
            _ => return false,
        }
    }
    true
}

/// Returns true when a local-slot immediate names concrete integer storage.
fn local_store_is_integer(function: &Function, immediate: Option<&Immediate>) -> bool {
    let Some(Immediate::LocalSlot(slot)) = immediate else {
        return false;
    };
    integer_local(function, *slot)
}

/// Checks that a local slot uses the concrete I64/PHP-int frame representation.
fn integer_local(function: &Function, slot: LocalSlotId) -> bool {
    function
        .locals
        .get(slot.as_raw() as usize)
        .is_some_and(|local| {
            local.ir_type == IrType::I64
                && matches!(local.php_type.codegen_repr(), PhpType::Int)
        })
}

/// Maps one boxed checked opcode to its allocation-free int-observing counterpart.
fn checked_to_int_op(op: Op) -> Option<Op> {
    match op {
        Op::ICheckedAdd => Some(Op::ICheckedAddToInt),
        Op::ICheckedSub => Some(Op::ICheckedSubToInt),
        Op::ICheckedMul => Some(Op::ICheckedMulToInt),
        _ => None,
    }
}

/// Commits a proven specialization, redirects scalar uses, and erases old scaffolding.
fn apply_specialization(
    function: &mut Function,
    candidate: InstId,
    result: ValueId,
    rewrite: SinkRewrite,
) {
    let replacements = resolve_chains(&rewrite.replacements);
    replace_all_uses(function, &replacements);
    for inst_id in rewrite.neutralize {
        if let Some(inst) = function.instruction_mut(inst_id) {
            neutralize_to_nop(inst);
        }
    }

    let replacement_op = function
        .instruction(candidate)
        .and_then(|inst| checked_to_int_op(inst.op))
        .expect("checked-int sink candidate retained its opcode until commit");
    if let Some(inst) = function.instruction_mut(candidate) {
        inst.op = replacement_op;
        inst.result_type = IrType::I64;
        inst.result_php_type = PhpType::Int;
        inst.result_ownership = Ownership::NonHeap;
        inst.effects = replacement_op.default_effects();
    }
    if let Some(value) = function.value_mut(result) {
        value.ir_type = IrType::I64;
        value.php_type = PhpType::Int;
        value.ownership = Ownership::NonHeap;
    }
}
