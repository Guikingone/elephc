//! Purpose:
//! Home of the PHP `getcwd` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook: `getcwd` is a pure-data builtin whose `Str` return type is
//!   fully determined by its declaration. The registry common path enforces its
//!   0-argument arity before falling back to `returns`.


builtin! {
    contract: "getcwd",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Getcwd,
    ),
}
