//! Purpose:
//! Home of the PHP `chunk_split` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts a required `string` param plus optional `length` and `separator` params
//!   with PHP's `76` / `"\r\n"` defaults.
//! - `$length < 1` raises php-src's `ValueError`, so the runtime function is declared
//!   `MAY_THROW` rather than pure and cannot be eliminated as dead code.


builtin! {
    contract: "chunk_split",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ChunkSplit,
    ),
}
