//! Purpose:
//! Home of the PHP `substr_replace` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts required `string`, `replace`, and `offset` params, plus an optional
//!   `length` param defaulting to null.


builtin! {
    contract: "substr_replace",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SubstrReplace,
    ),
}
