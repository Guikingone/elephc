//! Purpose:
//! Home of the PHP `stream_get_filters` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Array(Str)`, which is not scalar-expressible, so `returns: Mixed` is
//!   used and the hook overrides the return type. The hook takes no arguments.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "stream_get_filters",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamGetFilters,
    ),
}

/// Returns `Array(Str)` as the precise return type for `stream_get_filters`.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
