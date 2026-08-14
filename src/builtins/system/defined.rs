//! Purpose:
//! Home of the PHP `defined` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Constant names may be computed at runtime and are probed against the emitted registry.

builtin! {
    contract: "defined",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Defined,
    ),
}
