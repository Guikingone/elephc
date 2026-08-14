//! Purpose:
//! Home of the PHP `fdiv` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `fdiv` is a pure-data builtin whose return type
//!   (`Float`) is fully determined by its declaration.


builtin! {
    contract: "fdiv",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Fdiv,
    ),
}
