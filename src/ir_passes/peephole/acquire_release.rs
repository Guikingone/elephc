//! Purpose:
//! Paired acquire/release cancellation peephole: drop an `Acquire` whose result
//! is consumed only by a matching `Release`.
//!
//! Called from:
//! - `crate::ir_passes::peephole::Peephole::run` via `collect`.
//!
//! Key details:
//! - `Acquire` persists/increfs a value; `Release` frees/decrefs it. When the
//!   acquired result is used exactly once and that single use is its `Release`,
//!   the pair is refcount-neutral on every path (allocate-then-free or
//!   incref-then-decref with nothing observing the copy in between), and the
//!   acquired operand is untouched. Both instructions are neutralized.
//! - The single-use guard makes this safe regardless of how far apart the two
//!   ops are or which path the `Release` sits on: the value flows to exactly one
//!   `Release`, so removing both cannot leak or double-free.
//! - Ownership transfer: an `Acquire` of an owned heap temporary whose only other use is
//!   its `Release` in the same block just moves the reference (`v1 = acquire v0; ...;
//!   release v0`), so the acquired result is replaced by the temporary and both
//!   refcount operations are dropped. Strings are excluded because their `Acquire`
//!   also persists scratch-buffer payloads.

use std::collections::HashMap;

use crate::ir::{Function, InstId, IrType, Op, Ownership, ValueDef, ValueId};

use super::super::rewrite::count_value_uses;
use super::Rewrites;

/// Collects single-use `Acquire`/`Release` pairs, neutralizing both and
/// redirecting the (now unused) acquired result to the acquired operand.
pub(super) fn collect(function: &Function, rewrites: &mut Rewrites) {
    let uses = count_value_uses(function);
    let release_of = release_targets(function);
    let mut block_of: HashMap<InstId, crate::ir::BlockId> = HashMap::new();
    for block in &function.blocks {
        for &id in &block.instructions {
            block_of.insert(id, block.id);
        }
    }
    for (index, inst) in function.instructions.iter().enumerate() {
        if inst.op != Op::Acquire {
            continue;
        }
        let Some(acquired) = inst.result else {
            continue;
        };
        if uses.get(&acquired).copied().unwrap_or(0) != 1 {
            if let Some(&source) = inst.operands.first() {
                if transfers_owned_temporary(function, &uses, &release_of, &block_of, index, source) {
                    rewrites.rauw.insert(acquired, source);
                    rewrites.nops.push(InstId::from_raw(index as u32));
                    rewrites.nops.push(release_of[&source]);
                }
            }
            continue;
        }
        let Some(&source) = inst.operands.first() else {
            continue;
        };
        let Some(&release_index) = release_of.get(&acquired) else {
            if transfers_owned_temporary(function, &uses, &release_of, &block_of, index, source) {
                rewrites.rauw.insert(acquired, source);
                rewrites.nops.push(InstId::from_raw(index as u32));
                rewrites.nops.push(release_of[&source]);
            }
            continue;
        };
        rewrites.rauw.insert(acquired, source);
        rewrites.nops.push(InstId::from_raw(index as u32));
        rewrites.nops.push(release_index);
    }
}

/// Maps each value released by a `Release` instruction to that instruction's id.
/// A value released more than once keeps the first mapping; the single-use guard
/// in `collect` means only single-release values are ever cancelled anyway.
fn release_targets(function: &Function) -> HashMap<ValueId, InstId> {
    let mut targets = HashMap::new();
    for (index, inst) in function.instructions.iter().enumerate() {
        if inst.op != Op::Release {
            continue;
        }
        if let Some(&released) = inst.operands.first() {
            targets.entry(released).or_insert(InstId::from_raw(index as u32));
        }
    }
    targets
}

/// Returns whether the `Acquire` at `acquire_index` of `source` only moves the reference of
/// an owned heap temporary: `source` is defined by an instruction, used exactly by this
/// `Acquire` and one `Release`, and all three sit in the same block.
fn transfers_owned_temporary(
    function: &Function,
    uses: &HashMap<ValueId, usize>,
    release_of: &HashMap<ValueId, InstId>,
    block_of: &HashMap<InstId, crate::ir::BlockId>,
    acquire_index: usize,
    source: ValueId,
) -> bool {
    let Some(value) = function.value(source) else {
        return false;
    };
    if value.ownership != Ownership::Owned || !matches!(value.ir_type, IrType::Heap(_)) {
        return false;
    }
    let ValueDef::Instruction { inst: def_inst, .. } = value.def else {
        return false;
    };
    if uses.get(&source).copied().unwrap_or(0) != 2 {
        return false;
    }
    let Some(&release) = release_of.get(&source) else {
        return false;
    };
    let acquire = InstId::from_raw(acquire_index as u32);
    let Some(block) = block_of.get(&acquire) else {
        return false;
    };
    block_of.get(&def_inst) == Some(block) && block_of.get(&release) == Some(block)
}
