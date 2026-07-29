//! Purpose:
//! Home of the PHP `set_socket_blocking` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `set_socket_blocking` is an alias for `stream_set_blocking`.

builtin! {
    name: "set_socket_blocking",
    area: Io,
    params: [stream: Mixed, enable: Bool],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSetBlocking,
    ),
    summary: "Set blocking mode on a socket stream (alias of stream_set_blocking).",
    php_manual: "function.stream-set-blocking",
}