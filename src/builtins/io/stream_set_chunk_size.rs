//! Purpose:
//! Home of the PHP `stream_set_chunk_size` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers both arguments and returns `Int`
//!   (the previous chunk size, or the PHP default of 8192 on failure).


builtin! {
    contract: "stream_set_chunk_size",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamSetChunkSize,
    ),
}
