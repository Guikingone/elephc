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
//! - A boxed `mixed` element read may also be borrowed when it only feeds mixed `+`/`-`/`*`
//!   (`$dist[$u] + $w`), which reads the box and allocates its own result, a null test, or the
//!   receiver of a method call whose targets provably write and release nothing.
//! - The read's result is marked `Borrowed` (codegen then skips its incref) and the release is
//!   neutralized.

use std::collections::HashMap;

use crate::ir::{DataPool, Effects, Function, InstId, IrHeapKind, IrType, Op, Ownership, ValueId};

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
                let mixed_element = inst.result_type == IrType::Heap(IrHeapKind::Mixed);
                if inst.op != Op::ArrayGet
                    || inst.result_ownership != Ownership::Owned
                    || !(mixed_element
                        || matches!(
                            inst.result_type,
                            IrType::Heap(IrHeapKind::Array | IrHeapKind::Hash | IrHeapKind::Object)
                        ))
                {
                    continue;
                }
                let Some(result) = inst.result else {
                    continue;
                };
                if terminator_uses.contains(&result) {
                    continue;
                }
                let mut window: Vec<InstId> = order[position + 1..].to_vec();
                if mixed_element {
                    if let Some(next) = non_throwing_successor(function, block_index) {
                        window.extend(function.blocks[next].instructions.iter().copied());
                    }
                }
                let Some(release) = sole_release_after(function, &window, result, &uses, mixed_element)
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

/// Returns the block a `cond_br` continues into when its other target only throws (the
/// `method.null.fatal` arm of a method call on a nullable receiver) and the continuation is
/// entered from this block alone, so the borrow window can span the null check.
fn non_throwing_successor(function: &Function, block_index: usize) -> Option<usize> {
    let Some(crate::ir::Terminator::CondBr {
        then_target,
        then_args,
        else_target,
        else_args,
        ..
    }) = &function.blocks[block_index].terminator
    else {
        return None;
    };
    if !then_args.is_empty() || !else_args.is_empty() {
        return None;
    }
    let index_of = |id: crate::ir::BlockId| function.blocks.iter().position(|block| block.id == id);
    let throws_only = |index: usize| {
        let block = &function.blocks[index];
        matches!(block.terminator, Some(crate::ir::Terminator::Unreachable))
            && block.instructions.iter().all(|id| {
                function
                    .instruction(*id)
                    .is_some_and(|inst| matches!(inst.op, Op::ThrowError | Op::Nop))
            })
    };
    let (then_index, else_index) = (index_of(*then_target)?, index_of(*else_target)?);
    let next = if throws_only(then_index) {
        else_index
    } else if throws_only(else_index) {
        then_index
    } else {
        return None;
    };
    let target = function.blocks[next].id;
    let predecessors = function
        .blocks
        .iter()
        .filter_map(|block| block.terminator.as_ref())
        .filter(|term| super::cfg::successors(term).contains(&target))
        .count();
    (predecessors == 1).then_some(next)
}

/// Returns the single `release` of `result` when every other use is a container-read receiver
/// placed, like the release, later in the same window, with only pure instructions in between.
fn sole_release_after(
    function: &Function,
    window: &[InstId],
    result: ValueId,
    uses: &HashMap<ValueId, Vec<(InstId, usize)>>,
    mixed_element: bool,
) -> Option<InstId> {
    let all = uses.get(&result)?;
    let mut release = None;
    let mut remaining = all.len();
    for inst_id in window {
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
        if uses_here > 0 && mixed_element {
            // A boxed element may only feed mixed arithmetic, which reads the box and
            // allocates a fresh result without running user code or retaining the operand,
            // a null test, or the receiver slot of a method call whose possible targets can
            // neither write anything nor release a reference (`$shapes[$i]->area()`).
            let allowed = match inst.op {
                Op::MixedNumericBinop | Op::IsNull => true,
                Op::MethodCall => {
                    uses_here == 1
                        && inst.operands.first() == Some(&result)
                        && !inst.effects.intersects(
                            Effects::WRITES_LOCAL
                                | Effects::WRITES_HEAP
                                | Effects::WRITES_GLOBAL
                                | Effects::REFCOUNT_OP
                                | Effects::MAY_THROW,
                        )
                }
                _ => false,
            };
            if !allowed {
                return None;
            }
            remaining -= uses_here;
            continue;
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
        // Anything that could write, release or free the parent ends the window. Checked and
        // mixed arithmetic only allocate their result, and scalar casts only read registers.
        let scalar_cast = inst.op == Op::Cast
            && inst.operands.iter().all(|operand| {
                function.value(*operand).is_some_and(|value| {
                    matches!(value.ir_type, IrType::I64 | IrType::F64 | IrType::TaggedScalar)
                })
            });
        let pure = op_is_volatile_safe(inst.op)
            || scalar_cast
            || matches!(
                inst.op,
                Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul | Op::MixedNumericBinop
            )
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
