//! Purpose:
//! Home of the PHP `fmod` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `fmod` is a pure-data builtin whose return type
//!   (`Float`) is fully determined by its declaration.


builtin! {
    contract: "fmod",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Fmod,
    ),
}
