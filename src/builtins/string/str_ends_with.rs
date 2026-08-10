//! Purpose:
//! Home of the PHP `str_ends_with` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `str_ends_with` is a pure-data builtin whose
//!   return type (`Bool`) is fully determined by its declaration. The registry
//!   derives the return type from the `returns:` field without calling a check hook.


builtin! {
    contract: "str_ends_with",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StrEndsWith,
    ),
}
