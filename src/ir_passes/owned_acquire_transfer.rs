//! Purpose:
//! Turns `copy = acquire owner; ...; release owner` on an owned heap value into a plain
//! ownership transfer, dropping one refcount increment and one decrement.
//!
//! Called from:
//! - `driver::optimize_module()` after the ordinary optimization fixed point.
//!
//! Key details:
//! - The owner must be an owned heap value whose only consuming use is the single
//!   `release` that follows the `acquire` in the same block.
//! - Every other use must be a non-consuming type test (`is_null`, `type_predicate`,
//!   `is_truthy`) placed before the `acquire` or in another block.
//! - The copy keeps its own consumer, so the reference count observed by everything else
//!   is unchanged: the object stays alive through the copy exactly as before.

use std::collections::HashMap;

use crate::ir::{Function, InstId, IrType, Op, Ownership, ValueDef, ValueId};

use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Removes redundant acquire/release pairs around a transfer of an owned heap value.
pub struct OwnedAcquireTransfer;

impl IrPass for OwnedAcquireTransfer {
    /// Names this ownership finalization pass for validation diagnostics.
    fn name(&self) -> &'static str {
        "owned-acquire-transfer"
    }

    /// Runs once the ordinary passes stopped reshaping ownership.
    fn run_after_fixed_point(&self) -> bool {
        true
    }

    /// Skips functions without any refcount copy.
    fn is_applicable(&self, function: &Function) -> bool {
        function.instructions.iter().any(|inst| inst.op == Op::Acquire)
    }

    /// Rewrites each qualifying pair into a transfer.
    fn run(&self, function: &mut Function, _data: &mut crate::ir::DataPool) -> bool {
        let users = value_users(function);
        let mut replacements = HashMap::new();
        let mut removed = Vec::new();

        for block in &function.blocks {
            for (position, &acquire_id) in block.instructions.iter().enumerate() {
                let acquire = &function.instructions[acquire_id.as_raw() as usize];
                if acquire.op != Op::Acquire || acquire.operands.len() != 1 || acquire.immediate.is_some() {
                    continue;
                }
                let owner = acquire.operands[0];
                let Some(copy) = acquire.result else { continue };
                if !is_transferable(function, owner, copy) {
                    continue;
                }
                let Some(owner_users) = users.get(&owner) else { continue };
                let mut release = None;
                let mut safe = true;
                for &(user, in_terminator) in owner_users {
                    if in_terminator {
                        safe = false;
                        break;
                    }
                    if user == acquire_id {
                        continue;
                    }
                    let inst = &function.instructions[user.as_raw() as usize];
                    let later_here = block.instructions[position + 1..].contains(&user);
                    match inst.op {
                        Op::Release if inst.operands == [owner] && inst.immediate.is_none() && later_here => {
                            if release.replace(user).is_some() {
                                safe = false;
                            }
                        }
                        Op::IsNull | Op::TypePredicate | Op::IsTruthy if !later_here => {}
                        _ => safe = false,
                    }
                    if !safe {
                        break;
                    }
                }
                let Some(release_id) = release.filter(|_| safe) else { continue };
                replacements.insert(copy, owner);
                removed.extend([acquire_id, release_id]);
            }
        }

        if replacements.is_empty() {
            return false;
        }
        // A copy of a copy (`b = acquire a; c = acquire b`) forwards both links to `a`.
        replace_all_uses(function, &resolve_chains(&replacements));
        for id in removed {
            neutralize_to_nop(&mut function.instructions[id.as_raw() as usize]);
        }
        true
    }
}

/// Returns true when `owner` is an owned, instruction-defined heap value whose copy has the
/// same representation and ownership, so the copy's uses can read the owner directly.
fn is_transferable(function: &Function, owner: ValueId, copy: ValueId) -> bool {
    let (Some(owner_value), Some(copy_value)) = (function.value(owner), function.value(copy)) else {
        return false;
    };
    matches!(owner_value.ir_type, IrType::Heap(_))
        && owner_value.ir_type == copy_value.ir_type
        && owner_value.php_type == copy_value.php_type
        && owner_value.ownership == Ownership::Owned
        && copy_value.ownership == Ownership::Owned
        && matches!(owner_value.def, ValueDef::Instruction { .. })
}

/// Maps each value to the instructions (and terminators, flagged `true`) that read it.
fn value_users(function: &Function) -> HashMap<ValueId, Vec<(InstId, bool)>> {
    let mut users: HashMap<ValueId, Vec<(InstId, bool)>> = HashMap::new();
    for block in &function.blocks {
        for &id in &block.instructions {
            for operand in &function.instructions[id.as_raw() as usize].operands {
                users.entry(*operand).or_default().push((id, false));
            }
        }
        if let Some(term) = &block.terminator {
            for value in super::terminator_uses(term) {
                users.entry(value).or_default().push((InstId::from_raw(0), true));
            }
        }
    }
    users
}
