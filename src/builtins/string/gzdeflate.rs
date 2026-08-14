//! Purpose:
//! Home of the PHP `gzdeflate` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Returns a raw string; unlike the inflate variant it never fails with false.


builtin! {
    contract: "gzdeflate",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Gzdeflate,
    ),
}
