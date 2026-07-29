//! Purpose:
//! Home of the PHP `socket_set_timeout` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: the return type (`Bool`) is fully determined by its declaration.
//! - `socket_set_timeout` is an alias for `stream_set_timeout`.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "socket_set_timeout",
    area: Io,
    params: [stream: Mixed, seconds: Int, microseconds: Int = DefaultSpec::Int(0)],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSetTimeout,
    ),
    summary: "Set timeout period on a socket stream (alias of stream_set_timeout).",
    php_manual: "function.stream-set-timeout",
}