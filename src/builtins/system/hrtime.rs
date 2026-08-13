//! Purpose:
//! Home of the PHP `hrtime` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No `check` hook is needed: `hrtime` is a pure-data builtin whose return type
//!   (`Mixed`) is fully determined by its declaration. The `as_number` parameter
//!   is optional and defaults to `false`.


builtin! {
    contract: "hrtime",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Hrtime,
    ),
}
