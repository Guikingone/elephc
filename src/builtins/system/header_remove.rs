//! Purpose:
//! Home of PHP's `header_remove` builtin and its web response-state semantics.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - An omitted name clears all pending response headers; a supplied name removes matching headers
//!   case-insensitively without changing the response status.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "header_remove",
    area: System,
    params: [name: Str = DefaultSpec::Null],
    returns: Void,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HeaderRemove,
    ),
    summary: "Removes one or all pending HTTP response headers.",
    php_manual: "function.header-remove",
}
