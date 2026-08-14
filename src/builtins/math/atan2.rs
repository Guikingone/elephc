//! Purpose:
//! Home of the PHP `atan2` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `atan2` is a pure-data builtin whose return type
//!   (`Float`) is fully determined by its declaration.


builtin! {
    contract: "atan2",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Atan2,
    ),
}
