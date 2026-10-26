//! Purpose:
//! Shares boolean-parameter validation between native heap usage queries.
//!
//! Called from:
//! - The `memory_get_usage` and `memory_get_peak_usage` builtin homes.
//!
//! Key details:
//! - Scalar coercions match PHP's weak boolean parameter boundary.
//! - Dynamic values retain runtime truthiness conversion; known containers are rejected.

use crate::builtins::semantics::{
    runtime_fn_semantics, BuiltinSemanticInput, BuiltinSemantics, BuiltinValidation,
};
use crate::errors::CompileError;
use crate::ir::RuntimeFnId;
use crate::types::PhpType;

/// Adds shared scalar flag validation to an otherwise ordinary metric runtime call.
pub(super) const fn query_semantics(target: RuntimeFnId) -> BuiltinSemantics {
    let mut semantics = runtime_fn_semantics(target);
    semantics.validation = BuiltinValidation::Shared(validate_flag);
    semantics
}

/// Rejects values that cannot be passed to PHP's optional boolean real_usage parameter.
fn validate_flag(input: &BuiltinSemanticInput<'_>) -> Result<(), CompileError> {
    if input.arg_types.first().is_some_and(|ty| !accepts_flag(ty)) {
        return Err(CompileError::new(
            input.args.first().map_or(input.span, |arg| arg.span),
            &format!("{}() argument $real_usage must be bool-compatible", input.name),
        ));
    }
    Ok(())
}

/// Accepts weak scalar coercions and runtime-typed flags, excluding known container types.
fn accepts_flag(ty: &PhpType) -> bool {
    match ty {
        PhpType::Bool | PhpType::False | PhpType::Int | PhpType::Float | PhpType::Str
        | PhpType::Void | PhpType::Never | PhpType::Mixed | PhpType::TaggedScalar => true,
        PhpType::Union(members) => members.iter().all(accepts_flag),
        _ => false,
    }
}
