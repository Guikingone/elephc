//! Purpose:
//! Home of the PHP `stream_wrapper_unregister` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers the protocol argument and returns `Bool`.


builtin! {
    contract: "stream_wrapper_unregister",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamWrapperUnregister,
    ),
}
