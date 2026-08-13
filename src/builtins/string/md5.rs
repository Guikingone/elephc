//! Purpose:
//! Home of the PHP `md5` builtin: single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Arity (1–2 args) is validated by the registry's `check_arity` before the hook fires.


builtin! {
    contract: "md5",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Md5,
    ),
}
