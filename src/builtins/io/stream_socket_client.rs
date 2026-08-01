//! Purpose:
//! Home of the PHP `stream_socket_client` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates that `error_code` (arg[1]) and `error_message` (arg[2]), if provided,
//!   are plain variables (they are written by reference). Returns `Union(stream_resource, Bool)`.
//! - The out-params sit two positions earlier than `fsockopen`'s, which takes a host and a port
//!   before them; the shared `connect_error_params` helper is parameterized on those indices.
//! - `returns: Mixed` is used because the union cannot be expressed through the scalar field.
//! - `timeout`, `flags` and `context` are accepted for signature parity but not forwarded to the
//!   runtime, which always performs a blocking connect with the default flags.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "stream_socket_client",
    area: Io,
    params: [
        address: Str,
        ref error_code: Mixed = DefaultSpec::Null,
        ref error_message: Mixed = DefaultSpec::Null,
        timeout: Mixed = DefaultSpec::Null,
        flags: Mixed = DefaultSpec::Null,
        context: Mixed = DefaultSpec::Null
    ],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSocketClient,
    ),
    summary: "Open Internet or Unix domain socket connection.",
    php_manual: "function.stream-socket-client",
}

/// Validates ref output params are plain variables, then returns `Union(stream_resource, Bool)`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    super::connect_error_params::check_connect_error_params(cx, 1, 2)
}
