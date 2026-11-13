//! Purpose:
//! Removes closures that are built only to be stored in a local nothing ever reads: every
//! call through such a local was already lowered to a direct call of the closure body with
//! the captured values (`$add = fn($x) => $x + $i; $sum += $add($i);`), so allocating the
//! closure object, storing it and releasing the previous one is dead work.
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - Eligible slots are plain `PhpLocal` slots touched only by `store_local` and
//!   `release_local_slot`: no `load_local`, reference binding, capture or any other
//!   slot-addressing instruction. Global-storage variables never use local slots, and
//!   functions with eval scope state are skipped.
//! - Every store into the slot must store a single-use `closure_new` whose captured values
//!   are scalars or strings, so dropping the closure early cannot run a destructor or free
//!   anything observable sooner than PHP would.
//! - The stores, the `closure_new` instructions and the slot releases become `nop`; the slot
//!   then only ever holds its initial null, which the epilogue cleanup tolerates.

use std::collections::{HashMap, HashSet};

use crate::ir::{DataPool, Function, Immediate, InstId, IrType, LocalKind, LocalSlotId, Op, ValueDef};

use super::driver::IrPass;
use super::rewrite::{count_value_uses, neutralize_to_nop};

/// Pass removing closures stored only into never-read locals.
pub(crate) struct DeadClosureLocal;

impl IrPass for DeadClosureLocal {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "dead-closure-local"
    }

    /// Neutralizes dead closure constructions and their slot traffic.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        if function.locals.iter().any(|local| {
            matches!(
                local.kind,
                LocalKind::EvalContext | LocalKind::EvalScope | LocalKind::EvalGlobalScope
            )
        }) {
            return false;
        }
        let uses = count_value_uses(function);
        // Per slot: the stores and releases, or `None` once the slot is disqualified.
        let mut slots: HashMap<LocalSlotId, Option<(Vec<InstId>, Vec<InstId>)>> = HashMap::new();
        for (index, inst) in function.instructions.iter().enumerate() {
            let id = InstId::from_raw(index as u32);
            let mut touched: Vec<LocalSlotId> = Vec::new();
            match inst.immediate.as_ref() {
                Some(Immediate::LocalSlot(slot)) => touched.push(*slot),
                Some(Immediate::LocalSlotPair { first, second }) => {
                    touched.push(*first);
                    touched.push(*second);
                }
                _ => {}
            }
            for slot in touched {
                let entry = slots.entry(slot).or_insert_with(|| Some((Vec::new(), Vec::new())));
                let Some((stores, releases)) = entry.as_mut() else {
                    continue;
                };
                match inst.op {
                    Op::StoreLocal if stores_dead_scalar_closure(function, &uses, inst) => {
                        stores.push(id)
                    }
                    Op::ReleaseLocalSlot => releases.push(id),
                    _ => *entry = None,
                }
            }
        }
        let param_count = function.params.len();
        let mut changed = false;
        for (slot, plan) in slots {
            let Some((stores, releases)) = plan else {
                continue;
            };
            let index = slot.as_raw() as usize;
            if index < param_count || stores.is_empty() {
                continue;
            }
            if function.locals.get(index).is_none_or(|local| local.kind != LocalKind::PhpLocal) {
                continue;
            }
            let mut closures: HashSet<InstId> = HashSet::new();
            for store in &stores {
                let Some(def) = function
                    .instruction(*store)
                    .and_then(|inst| inst.operands.first().copied())
                    .and_then(|value| function.value(value))
                    .and_then(|value| match value.def {
                        ValueDef::Instruction { inst, .. } => Some(inst),
                        ValueDef::BlockParam { .. } => None,
                    })
                else {
                    continue;
                };
                closures.insert(def);
            }
            for inst in stores.iter().chain(releases.iter()).chain(closures.iter()) {
                if let Some(inst) = function.instruction_mut(*inst) {
                    neutralize_to_nop(inst);
                }
            }
            changed = true;
        }
        changed
    }
}

/// Returns whether `store` stores a single-use `closure_new` that only captures scalars.
fn stores_dead_scalar_closure(
    function: &Function,
    uses: &HashMap<crate::ir::ValueId, usize>,
    store: &crate::ir::Instruction,
) -> bool {
    let Some(&value) = store.operands.first() else {
        return false;
    };
    if uses.get(&value).copied() != Some(1) {
        return false;
    }
    let Some(ValueDef::Instruction { inst, .. }) = function.value(value).map(|value| value.def) else {
        return false;
    };
    let Some(closure) = function.instruction(inst) else {
        return false;
    };
    closure.op == Op::ClosureNew
        && closure.operands.iter().all(|capture| {
            function.value(*capture).is_some_and(|capture| {
                matches!(capture.ir_type, IrType::I64 | IrType::F64 | IrType::Str | IrType::TaggedScalar)
            })
        })
}
