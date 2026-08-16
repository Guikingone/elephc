//! Purpose:
//! Home of the PHP `spl_object_hash` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - A `check` hook is required to validate that the argument is an object; returns `Str`.
//! - The hash is derived from the object's heap pointer stringified via `__rt_itoa`.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "spl_object_hash",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplObjectHash,
    ),
}

/// Validates that the argument is an object and returns `Str`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !matches!(ty, PhpType::Object(_)) {
        return Err(CompileError::new(
            cx.span,
            "spl_object_hash() argument must be an object",
        ));
    }
    Ok(PhpType::Str)
}
