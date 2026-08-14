//! Purpose:
//! Home of the PHP `decbin` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `decbin(int $num): string`.
//! - The value is rendered as an UNSIGNED 64-bit quantity, so `decbin(-1)` is 64 `1` digits;
//!   the shared `__rt_dec_to_base` renderer owns that contract.

builtin! {
    contract: "decbin",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Decbin,
    ),
}
