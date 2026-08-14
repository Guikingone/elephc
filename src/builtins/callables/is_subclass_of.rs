//! Purpose:
//! Home of the PHP `is_subclass_of` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the registry common path infers all arguments and returns
//!   the declared `Bool` type.
//! - `allow_string` defaults to `true` (PHP's default for `is_subclass_of`).


builtin! {
    contract: "is_subclass_of",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::IsSubclassOf,
    ),
}
