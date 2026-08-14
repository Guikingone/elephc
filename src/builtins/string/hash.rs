//! Purpose:
//! Home of the PHP `hash` builtin: single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Arity (2–3 args) is validated by the registry's `check_arity` before the hook fires.


builtin! {
    contract: "hash",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Hash,
    ),
}
