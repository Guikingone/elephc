//! Purpose:
//! Home of the PHP `ob_clean` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Truncates the top buffer without popping it.
//! - Pure-data builtin: returns `Bool` (`false` when no output buffer is active).


builtin! {
    contract: "ob_clean",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ObClean,
    ),
}
