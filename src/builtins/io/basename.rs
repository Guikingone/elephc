//! Purpose:
//! Home of the PHP `basename` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `basename` is a pure-data builtin whose return type
//!   (`Str`) is fully determined by its declaration. The registry common path
//!   infers arguments and enforces arity before falling back to `returns`.


builtin! {
    contract: "basename",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Basename,
    ),
}
