//! Purpose:
//! Home of PHP's `error_log` builtin and its stderr-oriented AOT runtime contract.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Optional delivery arguments are accepted for PHP signature compatibility; the AOT runtime
//!   currently writes every message to stderr and reports success.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "error_log",
    area: System,
    params: [message: Str, message_type: Int = DefaultSpec::Int(0), destination: Str = DefaultSpec::Str(""), additional_headers: Str = DefaultSpec::Str("")],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ErrorLog,
    ),
    summary: "Writes a message to the configured error log destination.",
    php_manual: "function.error-log",
}
