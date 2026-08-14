//! Purpose:
//! Home of the PHP `sha1` builtin: single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Arity (1–2 args) is validated by the registry's `check_arity` before the hook fires.


builtin! {
    contract: "sha1",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Sha1,
    ),
}
