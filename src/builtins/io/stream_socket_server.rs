//! Purpose:
//! Home of the PHP `stream_socket_server` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(stream_resource, Bool)` reflecting PHP's false-on-failure return.
//! - `returns: Mixed` is used because the union cannot be expressed through the scalar field.
//! - The `error_code` and `error_message` parameters are by-reference: the caller passes
//!   plain variables that the runtime writes on failure.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "stream_socket_server",
    area: Io,
    params: [
        address: Str,
        ref(Int) error_code: Mixed = DefaultSpec::Null,
        ref(Str) error_message: Mixed = DefaultSpec::Null,
        flags: Int = DefaultSpec::Int(12),
        context: Mixed = DefaultSpec::Null
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSocketServer,
    ),
    summary: "Create an Internet or Unix domain server socket.",
    php_manual: "function.stream-socket-server",
}

/// Returns PHP's `resource|false` result. The by-reference outputs need no check here: their
/// `ref(T)` declarations carry the rule.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx.checker.normalize_union_type(vec![PhpType::stream_resource(), PhpType::False]))
}