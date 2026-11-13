//! Purpose:
//! Tests for the checked-cast hoist pass: an `int` cast of a checked add moves next to the add
//! across effect-free instructions, never across effects, and the moved results keep valid
//! definition positions.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Functions are built with `crate::ir::Builder`; `validate_function` checks that the
//!   rewritten blocks keep their `ValueDef` indices consistent.

use crate::ir::{
    validate_function, Builder, DataPool, Effects, Function, Immediate, IrType, Op, Ownership,
    Terminator,
};
use crate::ir_passes::checked_cast_hoist::CheckedCastHoist;
use crate::ir_passes::driver::IrPass;
use crate::types::PhpType;

/// Builds `(a + b) ^ (a & 1)` where `a + b` is a checked add whose `int` cast follows an
/// intermediate instruction with the given effects.
fn checked_add_then_cast(intermediate_effects: Effects) -> Function {
    let mut function = Function::new("hoist".to_string(), IrType::I64, PhpType::Int);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let a = builder.emit_const_i64(7);
        let b = builder.emit_const_i64(9);
        let sum = builder
            .emit(Op::ICheckedAdd, vec![a, b], None, IrType::Heap(crate::ir::IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned)
            .expect("checked add");
        let one = builder.emit_const_i64(1);
        let low = builder
            .emit_with_effects(Op::IBitAnd, vec![a, one], None, IrType::I64, PhpType::Int, Ownership::NonHeap, intermediate_effects, None)
            .expect("bit and");
        let cast = builder
            .emit(Op::Cast, vec![sum], Some(Immediate::CastTarget(IrType::I64)), IrType::I64, PhpType::Int, Ownership::NonHeap)
            .expect("cast");
        builder.emit(Op::Release, vec![sum], None, IrType::Void, PhpType::Void, Ownership::NonHeap);
        let result = builder
            .emit(Op::IBitXor, vec![cast, low], None, IrType::I64, PhpType::Int, Ownership::NonHeap)
            .expect("xor");
        builder.terminate(Terminator::Return { value: Some(result) });
    }
    function
}

/// Returns the ops of the entry block in order.
fn entry_ops(function: &Function) -> Vec<Op> {
    function
        .block(function.entry)
        .unwrap()
        .instructions
        .iter()
        .map(|id| function.instruction(*id).unwrap().op)
        .collect()
}

#[test]
fn cast_moves_next_to_checked_add_across_pure_instructions() {
    let mut function = checked_add_then_cast(Effects::empty());
    let changed = CheckedCastHoist.run(&mut function, &mut DataPool::default());
    assert!(changed);
    let ops = entry_ops(&function);
    let add = ops.iter().position(|op| *op == Op::ICheckedAdd).unwrap();
    assert_eq!(ops[add + 1], Op::Cast);
    validate_function(&function).expect("hoisted function stays valid");
    // A second run finds the pair adjacent and converges.
    assert!(!CheckedCastHoist.run(&mut function, &mut DataPool::default()));
}

#[test]
fn cast_does_not_cross_an_instruction_with_effects() {
    let mut function = checked_add_then_cast(Effects::MAY_WARN);
    let before = entry_ops(&function);
    assert!(!CheckedCastHoist.run(&mut function, &mut DataPool::default()));
    assert_eq!(entry_ops(&function), before);
}
