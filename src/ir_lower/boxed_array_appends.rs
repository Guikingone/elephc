//! Purpose:
//! Normalizes append receivers after the complete function has determined local storage.
//!
//! Called from:
//! - Module lowering before effect refinement and EIR validation.
//!
//! Key details:
//! - A later reference argument can widen an earlier native array local to boxed storage.
//! - Append mutates that box directly instead of leaking an owned unbox and its replaced box.
//! - The receiver is fetched after RHS evaluation, so RHS replacement of the local is observed.

use crate::ir::{Function, Immediate, IrHeapKind, IrType, LocalKind, Module, Op, Ownership, ValueDef};
use crate::types::PhpType;

/// Repairs every function's append receivers using final local storage contracts.
pub(super) fn normalize_module(module: &mut Module) {
    for function in module.functions.iter_mut()
        .chain(module.class_methods.iter_mut())
        .chain(module.closures.iter_mut())
    {
        normalize_function(function);
    }
}

/// Replaces single-use native local reads with adjacent borrowed boxed reads for append.
fn normalize_function(function: &mut Function) {
    // Static locals cannot enter this state: the checker excludes them from by-reference
    // promotion to boxed Mixed storage, and their static slot keeps its initializer layout.
    if !function.locals.iter().any(|local| local.kind == LocalKind::PhpLocal
        && matches!(local.php_type.codegen_repr(), PhpType::Mixed | PhpType::Union(_)))
        || !function.instructions.iter().any(|inst| matches!(inst.op, Op::ArrayPush | Op::HashAppend))
    { return; }
    let uses = function.value_use_counts();
    for block in &mut function.blocks {
        let appends: Vec<_> = block.instructions.iter().copied().filter(|id| {
            matches!(function.instructions[id.as_raw() as usize].op, Op::ArrayPush | Op::HashAppend)
        }).collect();
        let mut changed = false;
        for append_id in appends {
            let append_index = append_id.as_raw() as usize;
            let receiver = function.instructions[append_index].operands[0];
            if uses.get(&receiver).copied() != Some(1) { continue; }
            let ValueDef::Instruction { block: source_block, inst: load_id, .. } =
                function.values[receiver.as_raw() as usize].def else { continue; };
            if source_block != block.id { continue; }
            let load = &function.instructions[load_id.as_raw() as usize];
            if !matches!(load.op, Op::LoadLocal | Op::LoadRefCell) { continue; }
            let Some(Immediate::LocalSlot(slot)) = load.immediate else { continue; };
            let local = &function.locals[slot.as_raw() as usize];
            if local.kind != LocalKind::PhpLocal
                || !matches!(local.php_type.codegen_repr(), PhpType::Mixed | PhpType::Union(_))
                || !matches!(load.result_php_type.codegen_repr(), PhpType::Array(_) | PhpType::AssocArray { .. })
            { continue; }

            let load = &mut function.instructions[load_id.as_raw() as usize];
            load.result_type = IrType::Heap(IrHeapKind::Mixed);
            load.result_php_type = PhpType::Mixed;
            load.result_ownership = Ownership::Borrowed;
            let value = &mut function.values[receiver.as_raw() as usize];
            value.ir_type = IrType::Heap(IrHeapKind::Mixed);
            value.php_type = PhpType::Mixed;
            value.ownership = Ownership::Borrowed;
            let append = &mut function.instructions[append_index];
            append.op = Op::MixedArrayAppend;
            append.effects = Op::MixedArrayAppend.default_effects();

            block.instructions.retain(|id| *id != load_id);
            let position = block.instructions.iter().position(|id| *id == append_id).unwrap();
            block.instructions.insert(position, load_id);
            changed = true;
        }
        if changed {
            for (index, id) in block.instructions.iter().copied().enumerate() {
                if let Some(result) = function.instructions[id.as_raw() as usize].result {
                    function.values[result.as_raw() as usize].def = ValueDef::Instruction {
                        block: block.id, index: index as u32, inst: id,
                    };
                }
            }
        }
    }
}
