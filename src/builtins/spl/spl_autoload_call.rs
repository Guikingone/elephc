//! Purpose:
//! Home of the PHP `spl_autoload_call` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The AOT stub accepts exactly one class-name argument and returns void.


builtin! {
    contract: "spl_autoload_call",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SplAutoloadCall,
    ),
}
