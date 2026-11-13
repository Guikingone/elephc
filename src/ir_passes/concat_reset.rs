//! Purpose:
//! Removes `concat_reset` instructions that cannot change anything: the shared string scratch
//! offset is already back at the function's entry value on every path reaching them.
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - EIR lowering emits one `concat_reset` per statement so scratch strings produced by a
//!   statement are reclaimed. Most statements (integer/float arithmetic, local stores, array
//!   element reads) never touch the scratch area, so the reset before the next statement is a
//!   redundant load/store pair in every loop iteration.
//! - Calls/runtime calls and string producers other than plain loads are always dirtying,
//!   because runtime helpers may write scratch strings without declaring `alloc_concat`.
//! - A forward must-analysis tracks "clean" (offset equals the entry value): a reset makes the
//!   state clean, and any instruction that may allocate scratch strings, call out, write
//!   globals, print, warn, or throw makes it dirty. A fatal error ends the process, so it
//!   never leaves a dirty state behind for a later reset. A reset reached only by clean paths is a
//!   no-op and is neutralized. Removing a reset can never shorten the life of a scratch string.
//! - Functions with exception handlers or generator suspension points have control edges the
//!   terminator graph does not show, so they are skipped.

use std::collections::HashMap;

use crate::ir::{BlockId, DataPool, Effects, Function, Instruction, IrType, Op, Terminator};

use super::cfg::{has_exception_handlers, successors};
use super::driver::IrPass;
use super::rewrite::neutralize_to_nop;

/// Pass that removes redundant `concat_reset` instructions.
pub(crate) struct ConcatResetElim;

/// Effects after which the scratch offset may differ from the function's entry value.
fn dirty_effects(effects: Effects) -> bool {
    effects.intersects(
        Effects::ALLOC_CONCAT
            | Effects::WRITES_GLOBAL
            | Effects::OUTPUT
            | Effects::WRITES_FS
            | Effects::WRITES_PROCESS
            | Effects::MAY_WARN
            | Effects::MAY_THROW
            | Effects::MAY_DEOPT,
    )
}

/// Returns whether `inst` may leave the scratch offset away from its entry value.
///
/// Effects alone are not enough: runtime helpers and calls may write scratch strings
/// without declaring `alloc_concat`, and any non-trivial producer of a string may return
/// one that lives in the scratch area. Both are treated as dirtying.
fn dirties_scratch(function: &Function, inst: &Instruction) -> bool {
    // Element reads only warn about undefined keys / null containers; those warning helpers
    // restore the scratch offset after formatting the key, so the warning leaves it clean.
    let effects = if matches!(
        inst.op,
        Op::ArrayGet | Op::ArrayGetSilent | Op::HashGet | Op::HashGetSilent
    ) {
        inst.effects - Effects::MAY_WARN
    } else {
        inst.effects
    };
    if dirty_effects(effects) || inst.op.name().contains("call") {
        return true;
    }
    let produces_string = inst
        .result
        .and_then(|result| function.value(result))
        .is_some_and(|value| value.ir_type == IrType::Str);
    produces_string
        && !matches!(
            inst.op,
            Op::ConstStr
                | Op::LoadLocal
                | Op::LoadGlobal
                | Op::LoadStaticLocal
                | Op::LoadRefCell
                | Op::Acquire
                | Op::Move
                | Op::Borrow
        )
}

impl IrPass for ConcatResetElim {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "concat-reset-elim"
    }

    /// Neutralizes every `concat_reset` that only clean paths reach.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        if has_exception_handlers(function)
            || function.blocks.iter().any(|block| {
                matches!(block.terminator, Some(Terminator::GeneratorSuspend { .. }))
            })
        {
            return false;
        }
        let clean_in = compute_clean_in(function);
        let mut redundant = Vec::new();
        for block in &function.blocks {
            let mut clean = clean_in.get(&block.id).copied().unwrap_or(false);
            for &inst_id in &block.instructions {
                let Some(inst) = function.instruction(inst_id) else {
                    continue;
                };
                if inst.op == Op::ConcatReset {
                    if clean {
                        redundant.push(inst_id);
                    }
                    clean = true;
                } else if dirties_scratch(function, inst) {
                    clean = false;
                }
            }
        }
        if redundant.is_empty() {
            return false;
        }
        for inst_id in redundant {
            if let Some(inst) = function.instruction_mut(inst_id) {
                neutralize_to_nop(inst);
            }
        }
        true
    }
}

/// Computes, for every block, whether the scratch offset is clean on all incoming paths.
///
/// The entry block starts clean (the prologue saved the offset it will reset to); a block
/// with no predecessor other than through unknown edges starts dirty. The analysis starts
/// optimistic for reachable blocks and only ever flips them to dirty, so it terminates.
fn compute_clean_in(function: &Function) -> HashMap<BlockId, bool> {
    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for block in &function.blocks {
        if let Some(term) = &block.terminator {
            for succ in successors(term) {
                preds.entry(succ).or_default().push(block.id);
            }
        }
    }
    let mut clean_in: HashMap<BlockId, bool> = function
        .blocks
        .iter()
        .map(|block| {
            let start = block.id == function.entry || preds.contains_key(&block.id);
            (block.id, start)
        })
        .collect();
    let mut clean_out: HashMap<BlockId, bool> = HashMap::new();
    loop {
        let mut changed = false;
        for block in &function.blocks {
            let incoming = if block.id == function.entry {
                true
            } else {
                preds.get(&block.id).is_some_and(|ps| {
                    ps.iter().all(|p| clean_out.get(p).copied().unwrap_or(true))
                })
            };
            let previous = clean_in.insert(block.id, incoming);
            if previous != Some(incoming) {
                changed = true;
            }
            let mut clean = incoming;
            for &inst_id in &block.instructions {
                if let Some(inst) = function.instruction(inst_id) {
                    if inst.op == Op::ConcatReset {
                        clean = true;
                    } else if dirties_scratch(function, inst) {
                        clean = false;
                    }
                }
            }
            if clean_out.insert(block.id, clean) != Some(clean) {
                changed = true;
            }
        }
        if !changed {
            return clean_in;
        }
    }
}
