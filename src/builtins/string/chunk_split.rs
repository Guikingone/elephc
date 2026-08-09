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

use crate::builtins::spec::DefaultSpec;

builtin! {
    name: "chunk_split",
    area: String,
    params: [
        string: Str,
        length: Int = DefaultSpec::Int(76),
        separator: Str = DefaultSpec::Str("\r\n")
    ],
    returns: Str,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ChunkSplit,
    ),
    summary: "Splits a string into fixed-length chunks separated by a given string.",
    php_manual: "https://www.php.net/manual/en/function.chunk-split.php",
}
