//! Purpose:
//! Tests the ownership transfer of `copy = acquire owner; release owner` pairs.
//!
//! Called from:
//! - The library test harness through `ir_passes::tests`.
//!
//! Key details:
//! - Chained copies forward to the first owner, so no use names a removed acquire.
//! - Borrowed owners and owners read again after the copy keep their refcount pair.

use crate::ir::{
    validate_function, Builder, DataPool, Function, IrHeapKind, IrType, Op, Ownership, Terminator, ValueId,
};
use crate::ir_passes::driver::IrPass;
use crate::ir_passes::owned_acquire_transfer::OwnedAcquireTransfer;
use crate::types::PhpType;

/// Emits one boxed value with explicit ownership.
fn boxed(builder: &mut Builder<'_>, op: Op, operands: Vec<ValueId>, owner: Ownership) -> ValueId {
    builder.emit(op, operands, None, IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, owner)
        .expect("boxed result")
}

/// Emits a release of `value`.
fn release(builder: &mut Builder<'_>, value: ValueId) {
    builder.emit(Op::Release, vec![value], None, IrType::Void, PhpType::Void, Ownership::NonHeap);
}

/// Builds `owner -> copy (-> copy2)` with the releases following the copies.
fn fixture(owner_ownership: Ownership, chained: bool, reread: bool) -> Function {
    let mut function = Function::new("transfer".to_string(), IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed);
    let mut builder = Builder::new(&mut function);
    let entry = builder.create_named_block("entry", vec![]);
    builder.set_entry(entry);
    builder.position_at_end(entry);
    let lhs = builder.emit_const_i64(7);
    let rhs = builder.emit_const_i64(3);
    let owner = boxed(&mut builder, Op::ICheckedAdd, vec![lhs, rhs], owner_ownership);
    let copy = boxed(&mut builder, Op::Acquire, vec![owner], Ownership::Owned);
    let result = if chained {
        let second = boxed(&mut builder, Op::Acquire, vec![copy], Ownership::Owned);
        release(&mut builder, copy);
        second
    } else {
        copy
    };
    if reread {
        let observer = boxed(&mut builder, Op::Acquire, vec![owner], Ownership::Owned);
        release(&mut builder, observer);
    }
    release(&mut builder, owner);
    builder.terminate(Terminator::Return { value: Some(result) });
    function
}

/// Runs the pass alone.
fn transfer(function: &mut Function) -> bool {
    OwnedAcquireTransfer.run(function, &mut DataPool::default())
}

/// Returns the value the function returns.
fn returned(function: &Function) -> ValueId {
    match function.blocks[0].terminator {
        Some(Terminator::Return { value: Some(value) }) => value,
        _ => panic!("missing return"),
    }
}

/// A single copy of an owned value becomes the owner itself.
#[test]
fn owned_acquire_transfer_forwards_a_single_copy() {
    let mut function = fixture(Ownership::Owned, false, false);
    let owner = function.instructions[2].result.unwrap();
    assert!(transfer(&mut function));
    assert_eq!(returned(&function), owner);
    assert!(!function.instructions.iter().any(|inst| matches!(inst.op, Op::Acquire | Op::Release)));
    assert!(validate_function(&function).is_ok());
}

/// A copy of a copy forwards to the first owner instead of a removed acquire.
#[test]
fn owned_acquire_transfer_resolves_chained_copies() {
    let mut function = fixture(Ownership::Owned, true, false);
    let owner = function.instructions[2].result.unwrap();
    assert!(transfer(&mut function));
    assert_eq!(returned(&function), owner);
    assert!(!function.instructions.iter().any(|inst| matches!(inst.op, Op::Acquire | Op::Release)));
    assert!(validate_function(&function).is_ok());
}

/// Borrowed owners and owners read after the copy keep their acquire/release pair.
#[test]
fn owned_acquire_transfer_keeps_borrowed_or_reread_owners() {
    for (ownership, reread) in [(Ownership::Borrowed, false), (Ownership::Owned, true)] {
        let mut function = fixture(ownership, false, reread);
        let copy = function.instructions[3].result.unwrap();
        assert!(!transfer(&mut function));
        assert_eq!(returned(&function), copy);
    }
}
