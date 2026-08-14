//! Purpose:
//! Home of the PHP `stream_context_set_params` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers both arguments and returns `Bool`.


builtin! {
    contract: "stream_context_set_params",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamContextSetParams,
    ),
}
