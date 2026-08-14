//! Purpose:
//! Home of the PHP `mkdir` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The declaration exposes PHP's optional permissions, recursive, and stream
//!   context arguments while the runtime semantic target remains shared by
//!   direct calls and first-class callable consumers.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "mkdir",
    area: Io,
    params: [
        directory: Str,
        permissions: Int = DefaultSpec::Int(0o777),
        recursive: Bool = DefaultSpec::Bool(false),
        context: Mixed = DefaultSpec::Null
    ],
    returns: Bool,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Mkdir,
    ),
    summary: "Makes a directory.",
    php_manual: "function.mkdir",
}
