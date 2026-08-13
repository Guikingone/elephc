//! Purpose:
//! Home of the PHP `hash_hmac` builtin: single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Arity (3–4 args) is validated by the registry's `check_arity` before the hook fires.


builtin! {
    contract: "hash_hmac",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::HashHmac,
    ),
}
