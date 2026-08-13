//! Purpose:
//! Home of the PHP `preg_match_all` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Pure-data builtin: return type (`Int`) is fully determined by the declaration.


builtin! {
    contract: "preg_match_all",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::PregMatchAll,
    ),
}
