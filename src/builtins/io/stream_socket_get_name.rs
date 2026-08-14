//! Purpose:
//! Home of the PHP `stream_socket_get_name` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates arg[0] is a stream resource, then returns `Union(Str, Bool)`.
//! - Arguments are pre-inferred by the registry before the hook runs.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "stream_socket_get_name",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSocketGetName,
    ),
}

/// Validates arg[0] is a stream resource, then returns `Union(Str, Bool)`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    crate::types::checker::builtins::io::common::ensure_stream_resource(cx.checker, cx.name, &cx.args[0], cx.env)?;
    Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
