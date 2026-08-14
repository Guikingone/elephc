//! Purpose:
//! Home of the PHP `gethostbyname` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers the hostname argument and returns `Str`.


builtin! {
    contract: "gethostbyname",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Gethostbyname,
    ),
}
