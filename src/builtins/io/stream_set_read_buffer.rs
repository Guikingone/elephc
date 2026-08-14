//! Purpose:
//! Home of the PHP `stream_set_read_buffer` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers both arguments and returns `Int`
//!   (0 on success, matching PHP's successful no-op behaviour).


builtin! {
    contract: "stream_set_read_buffer",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSetReadBuffer,
    ),
}
