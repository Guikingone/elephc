//! Purpose:
//! Home of the PHP `gzcompress` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Returns a raw string; unlike the decompress variants it never fails.


builtin! {
    contract: "gzcompress",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Gzcompress,
    ),
}
