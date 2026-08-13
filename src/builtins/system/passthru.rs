//! Purpose:
//! Home of the PHP `passthru` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Pure-data builtin: return type (`Void`) is fully determined by the declaration.


builtin! {
    contract: "passthru",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Passthru,
    ),
}
