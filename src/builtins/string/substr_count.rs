//! Purpose:
//! Home of the PHP `substr_count` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Matches php-src's signature `substr_count(string $haystack, string $needle,
//!   int $offset = 0, ?int $length = null): int`, so `$length` is declared `Mixed` to
//!   carry the nullable default while `$offset` stays a plain `int`.
//! - The typed runtime target carries `MAY_THROW`: an empty `$needle` and an `$offset`
//!   or `$length` that escapes the subject each raise a catchable `ValueError`, so the
//!   call must not be removable by dead-code elimination.

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "substr_count",
    area: String,
    params: [
        haystack: Str,
        needle: Str,
        offset: Int = DefaultSpec::Int(0),
        length: Mixed = DefaultSpec::Null
    ],
    returns: Int,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::SubstrCount,
    ),
    summary: "Counts the number of non-overlapping substring occurrences.",
    php_manual: "https://www.php.net/manual/en/function.substr-count.php",
}
