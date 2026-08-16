//! Purpose:
//! Home of the PHP `dechex` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `dechex(int $num): string`.
//! - The value is rendered as an UNSIGNED 64-bit quantity, so `dechex(-1)` is
//!   `"ffffffffffffffff"`; the shared `__rt_dec_to_base` renderer owns that contract.
//! - Digits above 9 use lowercase letters, exactly like reference PHP.

builtin! {
    contract: "dechex",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Dechex,
    ),
}
