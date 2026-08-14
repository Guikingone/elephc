//! Purpose:
//! Home of the PHP `putenv` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Pure-data builtin: return type (`Bool`) is fully determined by the declaration.


builtin! {
    contract: "putenv",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Putenv,
    ),
}
