//! Purpose:
//! Home of the PHP `spl_autoload_unregister` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The AOT stub accepts exactly one callable argument and returns `true`.


builtin! {
    contract: "spl_autoload_unregister",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplAutoloadUnregister,
    ),
}
