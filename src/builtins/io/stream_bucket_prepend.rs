//! Purpose:
//! Home of the PHP `stream_bucket_prepend` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers both arguments and returns `Void`.


builtin! {
    contract: "stream_bucket_prepend",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamBucketPrepend,
    ),
}
