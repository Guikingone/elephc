//! Purpose:
//! Home of the PHP `spl_autoload` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts 1 required argument (`class`) and 1 optional argument (`file_extensions`).
//! - The AOT stub evaluates arguments for side effects and returns void.


builtin! {
    contract: "spl_autoload",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplAutoload,
    ),
}
