//! Purpose:
//! Home of the PHP `ksort` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The PHP signature accepts a by-reference array and an optional integer flags argument.
//!   The `ref` marker is mandatory because EIR lowering reads it from the registry signature
//!   to preserve mutation of the caller's storage.
//! - `check` accepts gradual array-compatible values and returns PHP's boolean success result.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "ksort",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Ksort,
    ),
}

/// Validates the argument type for a `ksort` call.
///
/// Requires the first argument be an indexed or associative array. The registry validates the
/// optional flags argument and the one-to-two argument arity. Returns `Ok(PhpType::Bool)` on
/// success.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let ty = cx.checker.infer_type(&cx.args[0], cx.env)?;
    if !crate::types::checker::builtins::arrays::array_arg_is_gradually_acceptable(&ty) {
        return Err(CompileError::new(cx.span, &format!("{}() argument must be array", cx.name)));
    }
    Ok(PhpType::Bool)
}
