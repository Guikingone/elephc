//! Purpose:
//! Home of the PHP `stream_register_wrapper` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: the return type (`Bool`) is fully determined by its declaration.
//! - `stream_register_wrapper` is an alias for `stream_wrapper_register`.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "stream_register_wrapper",
    area: Io,
    params: [protocol: Str, class: Str, flags: Int = DefaultSpec::Int(0)],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamWrapperRegister,
    ),
    summary: "Register a URL wrapper implemented as a PHP class (alias of stream_wrapper_register).",
    php_manual: "function.stream-wrapper-register",
}