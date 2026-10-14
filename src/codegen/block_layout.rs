//! Purpose:
//! Chooses the machine emission order of one function's EIR blocks and finds integer
//! compares that can be fused into their block's conditional branch.
//!
//! Called from:
//! - `crate::codegen::block_emit::emit_blocks()`.
//!
//! Key details:
//! - The first table block stays first: the prologue falls through into it.
//! - Blocks that can only end the activation abnormally (`unreachable`, `fatal`, `throw`)
//!   are cold. They move after every other block, in their original relative order, so a
//!   loop body's hot path is laid out contiguously and branches over no cold code.
//! - Every terminator lowers to explicit jumps except a jump to the block laid out next,
//!   which is what makes any order semantically safe.

use crate::ir::{BasicBlock, Function, InstId, Op, Terminator};
use crate::ir_passes::terminator_uses;

/// Returns block-table indices in emission order: hot blocks first, cold blocks last.
pub(super) fn emission_order(function: &Function) -> Vec<usize> {
    let mut hot = Vec::with_capacity(function.blocks.len());
    let mut cold = Vec::new();
    for (index, block) in function.blocks.iter().enumerate() {
        if index > 0 && is_cold(block) {
            cold.push(index);
        } else {
            hot.push(index);
        }
    }
    hot.extend(cold);
    hot
}

/// Returns true for a block whose terminator never continues normal control flow.
fn is_cold(block: &BasicBlock) -> bool {
    matches!(
        block.terminator,
        Some(Terminator::Unreachable | Terminator::Fatal { .. } | Terminator::Throw { .. })
    )
}

/// Returns the integer compare that `block`'s `cond_br` can consume as flags, if any.
///
/// The compare must be the block's last emitting instruction (only `nop`s may follow), and its
/// boolean must have no use besides the branch condition, so skipping its materialization is
/// unobservable and no instruction runs between the flag-setting compare and the branch.
pub(super) fn fusible_compare(function: &Function, block: &BasicBlock) -> Option<InstId> {
    let Some(Terminator::CondBr { cond, .. }) = &block.terminator else {
        return None;
    };
    let compare = block
        .instructions
        .iter()
        .rev()
        .copied()
        .find(|id| function.instruction(*id).is_some_and(|inst| inst.op != Op::Nop))?;
    let inst = function.instruction(compare)?;
    if inst.op != Op::ICmp || inst.result != Some(*cond) {
        return None;
    }
    let operand_uses = function
        .instructions
        .iter()
        .flat_map(|inst| inst.operands.iter())
        .filter(|operand| **operand == *cond)
        .count();
    let terminator_use_count: usize = function
        .blocks
        .iter()
        .filter_map(|block| block.terminator.as_ref())
        .map(|term| terminator_uses(term).iter().filter(|value| **value == *cond).count())
        .sum();
    (operand_uses == 0 && terminator_use_count == 1).then_some(compare)
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Unit tests for machine block ordering and compare-branch fusion eligibility.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Functions are built directly with the EIR builder so CFG shapes are exact.

    use super::{emission_order, fusible_compare};
    use crate::ir::{
        Builder, CmpPredicate, Function, Immediate, IrType, Op, Ownership, Terminator,
    };
    use crate::types::PhpType;

    /// Builds `entry -> cold(unreachable) | hot -> exit` with an icmp feeding the branch.
    fn diamond(extra_use: bool) -> Function {
        let mut function = Function::new("f".to_string(), IrType::I64, PhpType::Int);
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", Vec::new());
        let cold = builder.create_named_block("cold", Vec::new());
        let hot = builder.create_named_block("hot", Vec::new());
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let a = builder.emit_const_i64(1);
        let b = builder.emit_const_i64(2);
        let cond = builder
            .emit(
                Op::ICmp,
                vec![a, b],
                Some(Immediate::CmpPredicate(CmpPredicate::Slt)),
                IrType::I64,
                PhpType::Bool,
                Ownership::NonHeap,
            )
            .expect("icmp result");
        builder.terminate(Terminator::CondBr {
            cond,
            then_target: hot,
            then_args: Vec::new(),
            else_target: cold,
            else_args: Vec::new(),
        });
        builder.position_at_end(cold);
        builder.terminate(Terminator::Unreachable);
        builder.position_at_end(hot);
        let returned = if extra_use { cond } else { a };
        builder.terminate(Terminator::Return { value: Some(returned) });
        function
    }

    /// Verifies cold blocks move after hot ones while the first block stays first.
    #[test]
    fn cold_blocks_are_laid_out_last() {
        let function = diamond(false);
        assert_eq!(emission_order(&function), vec![0, 2, 1]);
    }

    /// Verifies a single-use icmp feeding the branch is fusible, and a reused one is not.
    #[test]
    fn only_single_use_branch_compares_fuse() {
        let function = diamond(false);
        assert!(fusible_compare(&function, &function.blocks[0]).is_some());
        let function = diamond(true);
        assert!(fusible_compare(&function, &function.blocks[0]).is_none());
    }
}
