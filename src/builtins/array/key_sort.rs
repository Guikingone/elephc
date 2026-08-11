//! Purpose:
//! Shared type-checking contract for PHP key-ordering builtins.
//!
//! Called from:
//! - `crate::builtins::array::ksort` and `crate::builtins::array::krsort`.
//!
//! Key details:
//! - Concrete arrays are accepted directly; integer-indexed cells of `array<mixed>` defer their
//!   runtime tag validation to the shared nested key-sort lowering path.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::PhpType;

/// Validates the common array receiver contract for `ksort()` and `krsort()`.
///
/// Concrete indexed and associative arrays are accepted statically. An integer-addressed cell of
/// a packed `array<mixed>` is also accepted because the nested lowering path checks its runtime tag
/// before mutation and raises the builtin-specific PHP `TypeError` for scalar or missing cells.
pub(super) fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    let accepts_mixed_nested_element = ty == PhpType::Mixed
        && is_packed_mixed_array_element_lvalue(cx.checker, cx.env, &cx.args[0])?;
    if !matches!(ty, PhpType::Array(_) | PhpType::AssocArray { .. })
        && !accepts_mixed_nested_element
    {
        return Err(CompileError::new(cx.span, &format!("{}() argument must be array", cx.name)));
    }
    Ok(PhpType::Bool)
}

/// Reports whether `arg` is an integer-addressed lvalue in a packed `array<mixed>` local.
///
/// This is deliberately narrower than accepting arbitrary `Mixed`: the EIR nested-place path can
/// recover one stable boxed cell only from this receiver shape and guards every non-array payload.
fn is_packed_mixed_array_element_lvalue(
    checker: &mut crate::types::checker::Checker,
    env: &crate::types::TypeEnv,
    arg: &Expr,
) -> Result<bool, CompileError> {
    let arg = match &arg.kind {
        ExprKind::NamedArg { value, .. } => value.as_ref(),
        _ => arg,
    };
    let ExprKind::ArrayAccess { array, index } = &arg.kind else {
        return Ok(false);
    };
    if !matches!(array.kind, ExprKind::Variable(_)) {
        return Ok(false);
    }
    let parent_ty = checker.infer_type(array, env)?;
    let index_ty = checker.infer_type(index, env)?;
    Ok(
        matches!(
            parent_ty.codegen_repr(),
            PhpType::Array(element_ty) if element_ty.codegen_repr() == PhpType::Mixed
        )
            && crate::types::normalized_array_key_type(index, index_ty) == PhpType::Int,
    )
}
