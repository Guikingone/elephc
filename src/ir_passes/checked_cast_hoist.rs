//! Purpose:
//! Moves the `int` cast of a checked arithmetic result up next to the arithmetic when only
//! effect-free instructions separate them, so the backend can fuse the pair.
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - `($sum + $x) ^ ($sum & 1)` lowers to `ichecked_add`, the unrelated `ibit_and`, then the cast
//!   of the sum: the backend fuses a checked op with its cast (native add, boxed slow path only on
//!   overflow) only when they are adjacent.
//! - The cast reads only the checked result, so moving it above instructions without effects
//!   (no output, warnings, heap or local access) changes no observable order.
//! - Moved results get their recorded definition positions rewritten (`ValueDef::index`).

use crate::ir::{Function, InstId, Op, ValueDef};

use super::driver::IrPass;

/// Hoists `cast <checked> I64` next to its checked add/sub/mul or mixed numeric binop.
pub(crate) struct CheckedCastHoist;

impl IrPass for CheckedCastHoist {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "checked-cast-hoist"
    }

    /// Reorders instructions inside each block; returns true when a cast moved.
    fn run(&self, function: &mut Function, _data: &mut crate::ir::DataPool) -> bool {
        let mut changed = false;
        for block_index in 0..function.blocks.len() {
            let mut order = function.blocks[block_index].instructions.clone();
            let mut block_changed = false;
            let mut position = 0;
            while position < order.len() {
                if let Some(cast_position) = hoistable_cast(function, &order, position) {
                    let cast = order.remove(cast_position);
                    order.insert(position + 1, cast);
                    block_changed = true;
                }
                position += 1;
            }
            if block_changed {
                let block_id = function.blocks[block_index].id;
                for (index, inst_id) in order.iter().enumerate() {
                    let Some(result) = function.instruction(*inst_id).and_then(|inst| inst.result)
                    else {
                        continue;
                    };
                    if let Some(value) = function.value_mut(result) {
                        value.def = ValueDef::Instruction {
                            block: block_id,
                            index: index as u32,
                            inst: *inst_id,
                        };
                    }
                }
                function.blocks[block_index].instructions = order;
                changed = true;
            }
        }
        changed
    }
}

/// Returns the position of the cast to move right after the checked op at `position`, when every
/// instruction in between is effect-free and does not read the checked result.
fn hoistable_cast(function: &Function, order: &[InstId], position: usize) -> Option<usize> {
    let checked = function.instruction(order[position])?;
    if !matches!(
        checked.op,
        Op::ICheckedAdd | Op::ICheckedSub | Op::ICheckedMul | Op::MixedNumericBinop
    ) {
        return None;
    }
    let result = checked.result?;
    for (offset, id) in order[position + 1..].iter().enumerate() {
        let inst = function.instruction(*id)?;
        if inst.operands.contains(&result) {
            let is_int_cast = inst.op == Op::Cast
                && inst.operands.as_slice() == [result]
                && matches!(
                    inst.immediate,
                    Some(crate::ir::Immediate::CastTarget(crate::ir::IrType::I64))
                );
            // Already adjacent: nothing to do.
            return (is_int_cast && offset > 0).then_some(position + 1 + offset);
        }
        if !inst.effects.is_empty() {
            return None;
        }
    }
    None
}
