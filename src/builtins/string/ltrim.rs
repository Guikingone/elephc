//! Purpose:
//! Home of the PHP `ltrim` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `ltrim` is a pure-data builtin. The registry's arity
//!   check (1 required, 1 optional → 1 or 2 args) exactly matches the legacy check-arm
//!   constraint, so no additional validation is needed.


builtin! {
    contract: "ltrim",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Ltrim,
    ),
}
