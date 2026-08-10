//! Purpose:
//! Home of the PHP `random_int` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `random_int` is a pure-data builtin returning `Int`.


builtin! {
    contract: "random_int",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::RandomInt,
    ),
}
