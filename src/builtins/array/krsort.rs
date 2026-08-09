//! Purpose:
//! Home of the PHP `krsort` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The golden signature is `first_param_ref(fixed(["array"]))`: exactly 1 argument,
//!   the `array` param is by-reference. The `ref` marker is mandatory — it is what makes
//!   by-reference mutation lower correctly (ir_lower reads `ref_params` from the registry sig).
//! - `check` accepts concrete arrays and integer-indexed cells of `array<mixed>`;
//!   the latter are checked by the nested lowering/runtime path before mutation.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::PhpType;

builtin! {
    name: "krsort",
    area: Array,
    params: [ref array: Mixed],
    returns: Bool,
    check: check,
    semantics: crate::builtins::semantics::with_argument_lowering(
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::Krsort),
        crate::builtins::semantics::BuiltinArgumentLowering::ReverseKeySort,
    ),
    summary: "Sorts an array by key in descending order.",
    php_manual: "https://www.php.net/manual/en/function.krsort.php",
}

/// Validates the argument type for a `krsort` call.
///
/// Requires a concrete array, or an integer-addressed cell of a packed `array<mixed>`.
///
/// The dynamic cell exception is limited to the nested lvalue path whose runtime promotion
/// guards non-array cells with PHP's `TypeError`; it does not relax direct mixed arguments.
/// Arity (exactly 1) is pre-validated by the registry. Returns `Ok(PhpType::Bool)` on success.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
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
/// The EIR `ArrayGet` path exposes one stable boxed cell for this shape. Its runtime promotion
/// validates the child tag before mutating it, so scalar or missing cells still fail as a PHP
/// `TypeError` instead of being treated as concrete arrays by the type checker.
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
