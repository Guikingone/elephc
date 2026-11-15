//! Purpose:
//! Removes a `load_local` of a plain local, or a scalar `prop_get`, that repeats an earlier
//! identical read in the same block with nothing in between able to change what it reads
//! (`$this->r * $this->r` reads `$this` and `r` once).
//!
//! Called from:
//! - `crate::ir_passes::driver::default_passes()`.
//!
//! Key details:
//! - Heap reads whose value any release, store, refcount operation, call or terminator
//!   consumes are never shared (a `maybe_owned` load may carry a reference).
//! - Only non-owning reads with the same IR and PHP type of `PhpLocal`/`HiddenTemp` slots that are never addressed by reference (no by-reference
//!   parameter, no reference-cell local in the function) are tracked, and only `int`/`float`/
//!   `bool` property reads (no ownership to balance).
//! - Any instruction with an effect beyond reading memory, allocating, or possibly
//!   failing/warning (a store, heap write, refcount change, call, output, ...) ends every
//!   tracked read; a `store_local`/`unset_local` of a slot also does.
//! - Functions with exception handlers or generator suspension are skipped.

use std::collections::HashMap;

use crate::ir::{DataPool, Effects, Function, Immediate, IrType, LocalKind, Op, ValueId};

use super::cfg::has_exception_handlers;
use super::driver::IrPass;
use super::rewrite::{neutralize_to_nop, replace_all_uses, resolve_chains};

/// Pass removing repeated local and scalar property reads within a block.
pub(crate) struct RedundantLoads;

impl IrPass for RedundantLoads {
    /// Returns the stable pass name used in driver diagnostics.
    fn name(&self) -> &'static str {
        "redundant-loads"
    }

    /// Forwards every repeated read to its first occurrence; returns whether anything changed.
    fn run(&self, function: &mut Function, _data: &mut DataPool) -> bool {
        if function.flags.is_generator
            || has_exception_handlers(function)
            || function.params.iter().any(|param| param.by_ref)
            || function.locals.iter().any(|local| local.kind == LocalKind::RefCell)
        {
            return false;
        }
        let harmless = Effects::READS_LOCAL
            | Effects::READS_HEAP
            | Effects::READS_GLOBAL
            | Effects::ALLOC_HEAP
            | Effects::ALLOC_CONCAT
            | Effects::MAY_FATAL
            | Effects::MAY_WARN
            | Effects::MAY_THROW;
        // Heap values whose reference a consumer may take or drop (a release, a store, any
        // refcount-changing operation, a terminator): their loads are never shared.
        let mut consumed: std::collections::HashSet<ValueId> = std::collections::HashSet::new();
        for inst in &function.instructions {
            if inst.effects.contains(Effects::REFCOUNT_OP)
                || matches!(inst.op, Op::Release | Op::StoreLocal | Op::Acquire)
                || inst.op.name().contains("call")
            {
                consumed.extend(inst.operands.iter().copied());
            }
        }
        for block in &function.blocks {
            if let Some(term) = &block.terminator {
                consumed.extend(super::liveness::terminator_uses(term));
            }
        }
        let mut replace: HashMap<ValueId, ValueId> = HashMap::new();
        let mut dead = Vec::new();
        for block in &function.blocks {
            let mut locals: HashMap<u32, (ValueId, (IrType, crate::types::PhpType))> = HashMap::new();
            let mut props: HashMap<(ValueId, u32), ValueId> = HashMap::new();
            for inst_id in &block.instructions {
                let Some(inst) = function.instruction(*inst_id) else {
                    continue;
                };
                match (inst.op, inst.immediate.clone(), inst.result) {
                    (Op::LoadLocal, Some(Immediate::LocalSlot(slot)), Some(result)) => {
                        let plain = function.locals.get(slot.as_raw() as usize).is_some_and(|local| {
                            matches!(local.kind, LocalKind::PhpLocal | LocalKind::HiddenTemp)
                        }) && inst.result_ownership != crate::ir::Ownership::Owned
                            && (!matches!(inst.result_type, IrType::Heap(_)) || !consumed.contains(&result));
                        if plain {
                            // Reads of one slot may view it differently (a list as `mixed`): only
                            // an identical view is shared.
                            let view = (inst.result_type, inst.result_php_type.clone());
                            match locals.get(&slot.as_raw()) {
                                Some((first, first_view))
                                    if *first_view == view
                                        && (!matches!(inst.result_type, IrType::Heap(_)) || !consumed.contains(first)) =>
                                {
                                    replace.insert(result, *first);
                                    dead.push(*inst_id);
                                }
                                Some(_) => {}
                                None => {
                                    locals.insert(slot.as_raw(), (result, view));
                                }
                            }
                        }
                        continue;
                    }
                    (Op::StoreLocal | Op::UnsetLocal, Some(Immediate::LocalSlot(slot)), _) => {
                        locals.remove(&slot.as_raw());
                        props.clear();
                        continue;
                    }
                    (Op::PropGet, Some(Immediate::Data(data)), Some(result))
                        if matches!(inst.result_type, IrType::I64 | IrType::F64)
                            && inst.operands.len() == 1
                            && (inst.effects - harmless).is_empty() =>
                    {
                        let key = (inst.operands[0], data.as_raw());
                        match props.get(&key) {
                            Some(first) => {
                                replace.insert(result, *first);
                                dead.push(*inst_id);
                            }
                            None => {
                                props.insert(key, result);
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
                if inst.op == Op::Nop || inst.op == Op::ConcatReset {
                    continue;
                }
                if !(inst.effects - harmless).is_empty() || inst.op.name().contains("call") {
                    locals.clear();
                    props.clear();
                }
            }
        }
        if replace.is_empty() {
            return false;
        }
        let resolved = resolve_chains(&replace);
        replace_all_uses(function, &resolved);
        for id in dead {
            if let Some(inst) = function.instruction_mut(id) {
                neutralize_to_nop(inst);
            }
        }
        true
    }
}
