//! Purpose:
//! Home of the PHP `decoct` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `decoct(int $num): string`.
//! - The value is rendered as an UNSIGNED 64-bit quantity, so `decoct(-1)` is
//!   `"1777777777777777777777"`; the shared `__rt_dec_to_base` renderer owns that contract.

builtin! {
    contract: "decoct",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Decoct,
    ),
}
