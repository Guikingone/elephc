//! Purpose:
//! Home of the PHP `get_class` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the registry common path infers the optional argument and
//!   returns the declared `Str` type.


builtin! {
    contract: "get_class",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::GetClass,
    ),
}
