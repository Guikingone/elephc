//! Purpose:
//! Home of the PHP `stream_context_get_params` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `AssocArray{Str, Mixed}` which is not scalar-expressible, so
//!   `returns: Mixed` is used and the hook overrides the return type.
//! - Arguments are pre-inferred by the registry before the hook runs; the hook does NOT
//!   re-infer them.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "stream_context_get_params",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamContextGetParams,
    ),
}

/// Returns `AssocArray{Str, Mixed}` reflecting the context parameters map structure.
///
/// Arguments are pre-inferred by the registry; this hook only refines the return type
/// beyond what the scalar `returns: Mixed` field can express.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::AssocArray {
        key: Box::new(PhpType::Str),
        value: Box::new(PhpType::Mixed),
    })
}
