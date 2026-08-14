//! Purpose:
//! Home of the PHP `gethostname` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - No check hook: the common registry path infers no arguments and returns `Str`.


builtin! {
    contract: "gethostname",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Gethostname,
    ),
}
