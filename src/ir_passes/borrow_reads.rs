//! Purpose:
//! Turns an owned indexed-array read of a container element (`$g[$u]` in `$g[$u][$w]`) into a
//! borrowed read when the element only feeds further reads before it is released, removing an
//! incref/decref pair per chained subscript.
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - Only `array_get` results that are heap containers (array, hash, object) qualify: their
//!   read only increments the element's refcount, so skipping it leaves a plain pointer into
//!   the parent, which stays valid while the parent is untouched.
//! - Every use must be the receiver of another container read in the same block, plus exactly
//!   one `release`; the instructions between the read and its release may only be pure reads
//!   and arithmetic, so the parent cannot be written, released or freed in the meantime.
//! - The read's result is marked `Borrowed` (codegen then skips its incref) and the release is
//!   neutralized.

use std::collections::HashMap;

use crate::ir::{DataPool, Function, InstId, IrHeapKind, IrType, Op, Ownership, ValueId};

use super::clobber::op_is_volatile_safe;
use super::driver::IrPass;
use super::rewrite::neutralize_to_nop;

/// Pass making chained container reads borrow their intermediate element.
pub(crate) struct BorrowReads;

impl IrPass for BorrowReads {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "borrow-reads"
    }

    /// Borrows every qualifying intermediate read; returns whether anything changed.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        if super::cfg::has_exception_handlers(function) {
            return false;
        }
        // Where every value is used: (instruction, operand index).
        let mut uses: HashMap<ValueId, Vec<(InstId, usize)>> = HashMap::new();
        for (index, inst) in function.instructions.iter().enumerate() {
            for (position, operand) in inst.operands.iter().enumerate() {
                uses.entry(*operand)
                    .or_default()
                    .push((InstId::from_raw(index as u32), position));
            }
        }
        let mut terminator_uses = std::collections::HashSet::new();
        for block in &function.blocks {
            if let Some(term) = &block.terminator {
                terminator_uses.extend(super::liveness::terminator_uses(term));
            }
        }
        let mut changed = false;
        for block_index in 0..function.blocks.len() {
            let order = function.blocks[block_index].instructions.clone();
            for (position, inst_id) in order.iter().enumerate() {
                let Some(inst) = function.instruction(*inst_id) else {
                    continue;
                };
                if inst.op != Op::ArrayGet
                    || inst.result_ownership != Ownership::Owned
                    || !matches!(
                        inst.result_type,
                        IrType::Heap(IrHeapKind::Array | IrHeapKind::Hash | IrHeapKind::Object)
                    )
                {
                    continue;
                }
                let Some(result) = inst.result else {
                    continue;
                };
                if terminator_uses.contains(&result) {
                    continue;
                }
                let Some(release) = sole_release_after(function, &order, position, result, &uses)
                else {
                    continue;
                };
                if let Some(inst) = function.instruction_mut(*inst_id) {
                    inst.result_ownership = Ownership::Borrowed;
                }
                if let Some(value) = function.value_mut(result) {
                    value.ownership = Ownership::Borrowed;
                }
                if let Some(inst) = function.instruction_mut(release) {
                    neutralize_to_nop(inst);
                }
                changed = true;
            }
        }
        changed
    }
}

/// Returns the single `release` of `result` when every other use is a container-read receiver
/// placed, like the release, later in the same block, with only pure instructions in between.
fn sole_release_after(
    function: &Function,
    order: &[InstId],
    position: usize,
    result: ValueId,
    uses: &HashMap<ValueId, Vec<(InstId, usize)>>,
) -> Option<InstId> {
    let all = uses.get(&result)?;
    let mut release = None;
    let mut remaining = all.len();
    for inst_id in &order[position + 1..] {
        let inst = function.instruction(*inst_id)?;
        let uses_here = inst.operands.iter().filter(|operand| **operand == result).count();
        if inst.op == Op::Release && uses_here == 1 {
            if release.is_some() {
                return None;
            }
            release = Some(*inst_id);
            remaining -= 1;
            return (remaining == 0).then_some(release?);
        }
        if uses_here > 0 {
            let receiver_only = inst.operands.first() == Some(&result) && uses_here == 1;
            if !receiver_only
                || !matches!(
                    inst.op,
                    Op::ArrayGet | Op::ArrayGetSilent | Op::HashGet | Op::HashGetSilent | Op::ArrayLen | Op::HashLen
                )
            {
                return None;
            }
            remaining -= 1;
            continue;
        }
        // Anything that could write, release or free the parent ends the window.
        let pure = op_is_volatile_safe(inst.op)
            || matches!(
                inst.op,
                Op::ArrayGet
                    | Op::ArrayGetSilent
                    | Op::HashGet
                    | Op::HashGetSilent
                    | Op::ArrayLen
                    | Op::HashLen
                    | Op::LoadLocal
                    | Op::ConstStr
            );
        if !pure {
            return None;
        }
    }
    None
}
