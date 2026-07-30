//! Purpose:
//! Verifies structural, type, branch, and dominance checks in the EIR validator.
//!
//! Called from:
//! - `crate::ir::tests`.
//!
//! Key details:
//! - Negative cases prove validator failures are based on current function state,
//!   not on assumptions from the builder.

use crate::ir::{
    validate_function, Builder, Function, Immediate, IrHeapKind, IrType, Op, Ownership,
    Terminator, ValidationError,
};
use crate::types::PhpType;

/// An empty function has no valid entry block and fails validation.
#[test]
fn empty_function_fails_validation() {
    let function = Function::new("empty".to_string(), IrType::Void, PhpType::Void);
    assert_eq!(validate_function(&function), Err(ValidationError::NoBlocks));
}

/// A one-block return function passes validation.
#[test]
fn well_formed_function_passes() {
    let mut function = Function::new("ok".to_string(), IrType::I64, PhpType::Int);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let value = builder.emit_const_i64(7);
        builder.terminate(Terminator::Return { value: Some(value) });
    }
    let result = validate_function(&function);
    assert!(result.is_ok(), "{result:?}");
}

/// Both implicit and explicit casts require exactly one EIR operand.
#[test]
fn cast_variants_reject_invalid_operand_counts() {
    for immediate in [
        Immediate::CastTarget(IrType::I64),
        Immediate::ExplicitCastTarget(IrType::I64),
    ] {
        let mut function =
            Function::new("bad_cast_arity".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", vec![]);
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let _ = builder.emit(
                Op::Cast,
                Vec::new(),
                Some(immediate),
                IrType::I64,
                PhpType::Int,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        assert!(matches!(
            validate_function(&function),
            Err(ValidationError::OperandCountMismatch {
                expected: "1",
                actual: 0,
                ..
            })
        ));
    }
}

/// Both implicit and explicit cast targets must equal the instruction result type.
#[test]
fn cast_variants_reject_target_result_mismatches() {
    for immediate in [
        Immediate::CastTarget(IrType::I64),
        Immediate::ExplicitCastTarget(IrType::I64),
    ] {
        let mut function =
            Function::new("bad_cast_target".to_string(), IrType::Void, PhpType::Void);
        {
            let mut builder = Builder::new(&mut function);
            let entry = builder.create_named_block("entry", vec![]);
            builder.set_entry(entry);
            builder.position_at_end(entry);
            let source = builder.emit_const_i64(1);
            let _ = builder.emit(
                Op::Cast,
                vec![source],
                Some(immediate),
                IrType::F64,
                PhpType::Float,
                Ownership::NonHeap,
            );
            builder.terminate(Terminator::Return { value: None });
        }
        assert!(matches!(
            validate_function(&function),
            Err(ValidationError::CastTargetTypeMismatch {
                target: IrType::I64,
                result: IrType::F64,
                ..
            })
        ));
    }
}

/// Exact array pointer storage accepts only the corresponding two-member
/// `array|null` metadata used by nullable container reads.
#[test]
fn exact_nullable_container_storage_passes() {
    let mut function =
        Function::new("nullable_array".to_string(), IrType::Void, PhpType::Void);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let array_type = PhpType::Array(Box::new(PhpType::Int));
        let _ = builder.emit(
            Op::ArrayNew,
            Vec::new(),
            Some(Immediate::Capacity(0)),
            IrType::Heap(IrHeapKind::Array),
            PhpType::Union(vec![array_type, PhpType::Void]),
            Ownership::MaybeOwned,
        );
        builder.terminate(Terminator::Return { value: None });
    }
    let result = validate_function(&function);
    assert!(result.is_ok(), "{result:?}");
}

/// Exact container storage rejects unions with more than one concrete member
/// because a single pointer representation cannot encode both alternatives.
#[test]
fn ambiguous_nullable_container_storage_fails() {
    let mut function =
        Function::new("ambiguous_array".to_string(), IrType::Void, PhpType::Void);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let _ = builder.emit(
            Op::ArrayNew,
            Vec::new(),
            Some(Immediate::Capacity(0)),
            IrType::Heap(IrHeapKind::Array),
            PhpType::Union(vec![
                PhpType::Array(Box::new(PhpType::Int)),
                PhpType::Str,
                PhpType::Void,
            ]),
            Ownership::MaybeOwned,
        );
        builder.terminate(Terminator::Return { value: None });
    }
    assert!(matches!(
        validate_function(&function),
        Err(ValidationError::PhpTypeMismatch(_))
    ));
}

/// A returned value must match the function's EIR return type.
#[test]
fn return_type_mismatch_fails() {
    let mut function = Function::new("bad".to_string(), IrType::F64, PhpType::Float);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let value = builder.emit_const_i64(1);
        builder.terminate(Terminator::Return { value: Some(value) });
    }
    assert!(matches!(
        validate_function(&function),
        Err(ValidationError::ReturnTypeMismatch { .. })
    ));
}

/// Branch argument types must match destination block parameter types.
#[test]
fn branch_argument_type_mismatch_fails() {
    let mut function = Function::new("branch_bad".to_string(), IrType::Void, PhpType::Void);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        let target = builder.create_named_block("target", vec![(IrType::F64, PhpType::Float)]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        let value = builder.emit_const_i64(1);
        builder.terminate(Terminator::Br {
            target,
            args: vec![value],
        });
        builder.position_at_end(target);
        builder.terminate(Terminator::Return { value: None });
    }
    assert!(matches!(
        validate_function(&function),
        Err(ValidationError::BranchArgTypeMismatch { .. })
    ));
}

/// A value defined in a non-dominating block cannot be returned elsewhere.
#[test]
fn use_not_dominated_fails() {
    let mut function = Function::new("dom_bad".to_string(), IrType::I64, PhpType::Int);
    {
        let mut builder = Builder::new(&mut function);
        let entry = builder.create_named_block("entry", vec![]);
        let other = builder.create_named_block("other", vec![]);
        let exit = builder.create_named_block("exit", vec![]);
        builder.set_entry(entry);
        builder.position_at_end(entry);
        builder.terminate(Terminator::Br {
            target: exit,
            args: vec![],
        });
        builder.position_at_end(other);
        let hidden = builder.emit_const_i64(9);
        builder.terminate(Terminator::Unreachable);
        builder.position_at_end(exit);
        builder.terminate(Terminator::Return {
            value: Some(hidden),
        });
    }
    assert!(matches!(
        validate_function(&function),
        Err(ValidationError::UseNotDominated { .. })
    ));
}
