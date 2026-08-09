//! Purpose:
//! Home of the PHP `strncasecmp` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature
//!   `strncasecmp(string $string1, string $string2, int $length): int`; all three parameters
//!   are required.
//! - Case folding is ASCII-only, exactly like `strcasecmp`, and the result is the raw folded
//!   byte difference rather than a clamped `-1/0/1`.
//! - The typed runtime target carries `MAY_THROW`: a negative `$length` raises a catchable
//!   `ValueError`, so the call must not be removable by dead-code elimination.

builtin! {
    name: "strncasecmp",
    area: String,
    params: [string1: Str, string2: Str, length: Int],
    returns: Int,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Strncasecmp,
    ),
    summary: "Compares the first n bytes of two strings, ignoring ASCII case.",
    php_manual: "https://www.php.net/manual/en/function.strncasecmp.php",
}
