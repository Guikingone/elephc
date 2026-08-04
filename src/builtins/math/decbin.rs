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
    name: "decbin",
    area: Math,
    params: [num: Int],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Decbin,
    ),
    summary: "Converts an integer to its binary string representation.",
    php_manual: "https://www.php.net/manual/en/function.decbin.php",
}
