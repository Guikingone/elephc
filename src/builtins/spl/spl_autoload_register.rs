//! Purpose:
//! Home of the PHP `spl_autoload_register` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The autoload registration is an AOT stub: all three parameters are optional
//!   and any combination of 0–3 arguments is accepted. Returns `true` always.


builtin! {
    contract: "spl_autoload_register",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplAutoloadRegister,
    ),
}
