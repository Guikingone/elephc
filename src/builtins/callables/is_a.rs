//! Purpose:
//! Home of the PHP `is_a` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the registry common path infers all arguments and returns
//!   the declared `Bool` type.
//! - `allow_string` defaults to `false` (PHP's default for `is_a`).


builtin! {
    contract: "is_a",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::IsA,
    ),
}
