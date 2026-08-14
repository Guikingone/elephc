//! Purpose:
//! Home of the PHP `http_response_code` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Pure-data builtin: return type (`Int`) is fully determined by the declaration.
//! - `arity_error` overrides the default "takes at most 1 argument" message to match
//!   the legacy phrasing "takes 0 or 1 arguments".


builtin! {
    contract: "http_response_code",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HttpResponseCode,
    ),
}
