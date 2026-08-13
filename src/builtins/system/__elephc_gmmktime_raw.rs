//! Purpose:
//! Home of the internal `__elephc_gmmktime_raw` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - This is an internal builtin (`internal: true`) not exposed as a PHP-visible function.
//!   It is used by the synthetic DateTime body as a raw gmmktime alias.
//! - The typed runtime target delegates to the same emitter as `gmmktime`.


builtin! {
    contract: "__elephc_gmmktime_raw",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ElephcGmmktimeRaw,
    ),
}
