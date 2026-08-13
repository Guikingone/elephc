//! Purpose:
//! Home of the PHP `getdate` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `getdate` is a pure-data builtin whose return type
//!   (`Mixed`) is fully determined by its declaration. The `timestamp` parameter
//!   is optional and defaults to `null` (current time).


builtin! {
    contract: "getdate",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Getdate,
    ),
}
