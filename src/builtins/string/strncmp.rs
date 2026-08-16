//! Purpose:
//! Home of the PHP `strncmp` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `strncmp(string $string1, string $string2, int $length): int`;
//!   all three parameters are required.
//! - Like `strcmp`, the result is the raw byte difference of the first mismatching pair, not a
//!   clamped `-1/0/1`.
//! - The typed runtime target carries `MAY_THROW`: a negative `$length` raises a catchable
//!   `ValueError`, so the call must not be removable by dead-code elimination.

builtin! {
    contract: "strncmp",
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Strncmp,
    ),
}
