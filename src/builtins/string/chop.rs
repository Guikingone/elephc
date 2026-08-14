//! Purpose:
//! Home of the PHP `chop` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `chop` is a PHP alias for `rtrim`. Both share the same signature, runtime
//!   helpers, and parameter defaults.
//! - No `check` hook is needed: `chop` is a pure-data builtin. The registry's arity
//!   check (1 required, 1 optional → 1 or 2 args) exactly matches the legacy check-arm
//!   constraint, so no additional validation is needed.


builtin! {
    contract: "chop",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Chop,
    ),
}
