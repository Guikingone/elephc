//! Purpose:
//! Identifies integer/bool constants that the backend rematerializes as immediates instead
//! of keeping them in a register or frame slot.
//!
//! Called from:
//! - `crate::codegen::block_emit` (skips emitting the constants, reads them as immediates).
//! - `crate::ir_passes::regalloc` (never spends a register on a skipped constant).
//!
//! Key details:
//! - Every `const_i64`/`const_bool` with an `int`/`bool` representation is an immediate:
//!   loads of it into general-purpose registers emit the constant directly.
//! - A constant is skipped entirely only when every use reads its operands through the
//!   backend's value-load chokepoints (integer arithmetic, compares, `store_local`) or is a
//!   block-argument edge copy; any other use keeps the constant materialized in its home.

use std::collections::{HashMap, HashSet};

use crate::ir::{Function, Immediate, InstId, IrType, Op, Terminator, ValueId};
use crate::types::PhpType;

/// Rematerialization facts for one function.
#[derive(Debug, Default)]
pub struct RematerializedConsts {
    /// Immediate of every integer/bool constant value.
    pub immediates: HashMap<ValueId, i64>,
    /// Constant instructions that need no emission at all.
    pub skipped: HashSet<InstId>,
}

/// Computes the rematerialized constants of `function`.
pub fn rematerialized_consts(function: &Function) -> RematerializedConsts {
    let mut facts = RematerializedConsts::default();
    let mut const_insts: HashMap<ValueId, InstId> = HashMap::new();
    for block in &function.blocks {
        for inst_id in &block.instructions {
            let Some(inst) = function.instruction(*inst_id) else {
                continue;
            };
            let imm = match (inst.op, inst.immediate.as_ref()) {
                (Op::ConstI64, Some(Immediate::I64(value))) => *value,
                (Op::ConstBool, Some(Immediate::Bool(value))) => i64::from(*value),
                _ => continue,
            };
            let Some(result) = inst.result else {
                continue;
            };
            if inst.result_type != IrType::I64
                || !matches!(inst.result_php_type.codegen_repr(), PhpType::Int | PhpType::Bool)
            {
                continue;
            }
            facts.immediates.insert(result, imm);
            const_insts.insert(result, *inst_id);
        }
    }
    if const_insts.is_empty() {
        return facts;
    }
    let mut needs_home: HashSet<ValueId> = HashSet::new();
    for inst in &function.instructions {
        let reads_through_loads = matches!(
            inst.op,
            Op::IAdd
                | Op::ISub
                | Op::IMul
                | Op::IBitAnd
                | Op::IBitOr
                | Op::IBitXor
                | Op::ICmp
                | Op::StoreLocal
                | Op::Nop
        );
        if !reads_through_loads {
            needs_home.extend(inst.operands.iter().copied());
        }
    }
    for block in &function.blocks {
        let Some(term) = block.terminator.as_ref() else {
            continue;
        };
        let branch_args: HashSet<ValueId> = match term {
            Terminator::Br { args, .. } => args.iter().copied().collect(),
            Terminator::CondBr { then_args, else_args, .. } => {
                then_args.iter().chain(else_args).copied().collect()
            }
            Terminator::Switch { cases, default_args, .. } => cases
                .iter()
                .flat_map(|case| case.args.iter().copied())
                .chain(default_args.iter().copied())
                .collect(),
            _ => HashSet::new(),
        };
        for value in super::liveness::terminator_uses(term) {
            if !branch_args.contains(&value) {
                needs_home.insert(value);
            }
        }
    }
    for (value, inst_id) in const_insts {
        if !needs_home.contains(&value) {
            facts.skipped.insert(inst_id);
        }
    }
    facts
}
