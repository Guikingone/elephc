//! Purpose:
//! Home of the PHP `intdiv` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `intdiv` is a pure-data builtin whose return type
//!   (`Int`) is fully determined by its declaration.


builtin! {
    contract: "intdiv",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Intdiv,
    ),
}
