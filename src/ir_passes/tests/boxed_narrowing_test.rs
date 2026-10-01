//! Purpose:
//! Tests representation-sensitive consumers of proven or constant boxed arithmetic.
//!
//! Called from:
//! - The EIR pass unit-test harness.
//!
//! Key details:
//! - Range proofs and constant folding must both preserve unsupported boxed consumers.
//! - Integer and float constants have different typed-store conversion contracts.

use crate::ir::{validate_function, Builder, CmpPredicate, DataPool, Function, Immediate, IrHeapKind, IrType,
    LocalKind, Op, Ownership, Terminator};
use crate::ir_passes::{const_fold::ConstFold, driver::IrPass, integer_range::IntegerRange};
use crate::types::PhpType;

/// Builds a bounded checked sum followed by a cast requiring a boxed input cell.
fn array_cast_fixture() -> Function {
    let mut function = Function::new("array_cast".to_string(), IrType::Void, PhpType::Void);
    let mut builder = Builder::new(&mut function);
    let entry = builder.create_named_block("entry", vec![]);
    builder.set_entry(entry);
    builder.position_at_end(entry);
    let one = builder.emit_const_i64(1);
    let sum = builder.emit(
        Op::ICheckedAdd, vec![one, one], None,
        IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned,
    ).unwrap();
    builder.emit(
        Op::Cast, vec![sum], Some(Immediate::CastTarget(IrType::Heap(IrHeapKind::Mixed))),
        IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned,
    );
    builder.terminate(Terminator::Return { value: None });
    function
}

/// Neither the range pass nor subsequent folding may unbox an array-cast input.
#[test]
fn array_cast_requires_boxed_arithmetic() {
    for pass in [&IntegerRange as &dyn IrPass, &ConstFold as &dyn IrPass] {
        let mut function = array_cast_fixture();
        assert!(validate_function(&function).is_ok());
        assert!(!pass.run(&mut function, &mut DataPool::default()), "{}", pass.name());
        assert_eq!(function.instructions[1].op, Op::ICheckedAdd);
        assert!(validate_function(&function).is_ok());
    }
}

/// Typed ref-cell stores cannot silently lose their Mixed-to-bool/float/string coercion.
#[test]
fn typed_ref_cell_keeps_required_conversion() {
    for target in [PhpType::Bool, PhpType::Float, PhpType::Str] {
        for pass in [&IntegerRange as &dyn IrPass, &ConstFold as &dyn IrPass] {
            let mut function = Function::new("ref_store".to_string(), IrType::Void, PhpType::Void);
            {
                let mut builder = Builder::new(&mut function);
                let entry = builder.create_named_block("entry", vec![]);
                builder.set_entry(entry);
                builder.position_at_end(entry);
                let slot = builder.add_local(Some("target".to_string()), IrType::I64,
                    target.clone(), LocalKind::RefCell);
                let one = builder.emit_const_i64(1);
                let sum = builder.emit(Op::ICheckedAdd, vec![one, one], None,
                    IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned).unwrap();
                builder.emit(Op::StoreRefCell, vec![sum], Some(Immediate::LocalSlot(slot)),
                    IrType::Void, target.clone(), Ownership::NonHeap);
                builder.terminate(Terminator::Return { value: None });
            }
            assert!(validate_function(&function).is_ok());
            assert!(!pass.run(&mut function, &mut DataPool::default()), "{target:?}, {}", pass.name());
        }
    }
}

/// Overflow folded to a float must not bypass an integer slot's explicit numeric conversion.
#[test]
fn folded_float_keeps_integer_store_conversion() {
    let mut function = Function::new("float_store".to_string(), IrType::Void, PhpType::Void);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let slot = builder.add_local(Some("target".to_string()), IrType::I64,
            PhpType::Int, LocalKind::PhpLocal);
        let max = builder.emit_const_i64(i64::MAX);
        let sum = builder.emit(Op::ICheckedAdd, vec![max, max], None,
            IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned).unwrap();
        builder.emit_store_local(slot, sum);
        builder.terminate(Terminator::Return { value: None });
    }
    assert!(validate_function(&function).is_ok());
    assert!(!ConstFold.run(&mut function, &mut DataPool::default()));
}

/// Runtime ordering must retain tagged inputs and bool/null spaceship coercion rules.
#[test]
fn ordering_consumers_preserve_required_boxed_operands() {
    for op in [Op::PhpRelCmp, Op::Spaceship] {
        for null_rhs in [false, true] {
            for overflow in [false, true] {
                for pass in [&IntegerRange as &dyn IrPass, &ConstFold as &dyn IrPass] {
                    let mut function = Function::new("ordering".to_string(), IrType::Void, PhpType::Void);
                    let mut builder = Builder::new(&mut function);
                    let entry = builder.create_named_block("entry", vec![]);
                    builder.set_entry(entry);
                    builder.position_at_end(entry);
                    let lhs = builder.emit_const_i64(if overflow { i64::MAX } else { 1 });
                    let sum = builder.emit(Op::ICheckedAdd, vec![lhs, lhs], None,
                        IrType::Heap(IrHeapKind::Mixed), PhpType::Mixed, Ownership::Owned).unwrap();
                    let rhs = if null_rhs { builder.emit_const_null() } else { builder.emit_const_bool(true) };
                    let (immediate, ty) = if op == Op::PhpRelCmp {
                        (Some(Immediate::CmpPredicate(CmpPredicate::Slt)), PhpType::Bool)
                    } else { (None, PhpType::Int) };
                    builder.emit(op, vec![sum, rhs], immediate, IrType::I64, ty, Ownership::NonHeap);
                    builder.terminate(Terminator::Return { value: None });
                    assert!(validate_function(&function).is_ok());
                    assert!(!pass.run(&mut function, &mut DataPool::default()), "{op:?}, {}", pass.name());
                }
            }
        }
    }
}
