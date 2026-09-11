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
    validate_function, Builder, Function, IrType, Terminator, ValidationError,
};
use crate::types::PhpType;

fn source_marker_module(immediate: crate::ir::Immediate) -> crate::ir::Module {
    use crate::ir::{Module, Op, Ownership, SourceCatalog};
    let catalog = SourceCatalog::from_units([crate::resolver::SourceUnit {
        canonical_path: "/fixture.php".into(), mode: crate::source::SourceMode::Php,
        source: "<?php".into(),
    }]).unwrap();
    let mut module = Module::with_source_catalog(crate::codegen::platform::Target::detect_host(), catalog);
    let mut function = Function::new("marker".into(), IrType::Void, PhpType::Void);
    let mut builder = Builder::new(&mut function);
    let entry = builder.create_named_block("entry", vec![]);
    builder.set_entry(entry);
    builder.position_at_end(entry);
    builder.emit(Op::IncludeOnceMark, vec![], Some(immediate), IrType::Void, PhpType::Void, Ownership::NonHeap);
    builder.terminate(Terminator::Return { value: None });
    module.add_function(function);
    module
}

#[test]
fn source_marker_validation_rejects_unknown_ids_and_unresolved_paths() {
    use crate::ir::{validate_module, Immediate, SourceId};
    assert!(validate_module(&source_marker_module(Immediate::Source(SourceId::from_raw(0)))).is_ok());
    assert_eq!(validate_module(&source_marker_module(Immediate::Source(SourceId::from_raw(1)))),
        Err(ValidationError::UnknownSource(SourceId::from_raw(1))));
    let path = std::path::PathBuf::from("/missing.php");
    assert_eq!(validate_module(&source_marker_module(Immediate::UnresolvedSource(path.clone()))),
        Err(ValidationError::UnresolvedSource(path)));
    assert!(matches!(validate_module(&source_marker_module(Immediate::Data(crate::ir::DataId::from_raw(0)))),
        Err(ValidationError::MissingImmediate { expected: "source id", .. })));
}

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
    assert!(validate_function(&function).is_ok());
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
