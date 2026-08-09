//! Purpose:
//! Home of the PHP `strripos` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The declared signature is PHP's own `strripos(string $haystack, string $needle, int $offset = 0)`,
//!   the case-insensitive twin of `strrpos()`. A non-negative `$offset` starts the right-to-left
//!   search at that byte; a negative one stops the search `-$offset` bytes before the haystack
//!   end, and an out-of-haystack offset raises PHP's catchable `ValueError` from the backend
//!   lowering — the shared `lower_string_position` path both spellings go through.
//! - Case folding is ASCII-only, matching php-src's locale-independent `zend_tolower_ascii`.
//! - `check` returns `PhpType::Union([Int, False])` (position, or `false` on no match).
//!   A check hook is required because the `builtin!` macro `returns:` field only accepts
//!   a simple type identifier and cannot express a union inline. Argument types are
//!   inferred by the common registry dispatch path before the hook fires.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "strripos",
    area: String,
    params: [haystack: Str, needle: Str, offset: Int = DefaultSpec::Int(0)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Strripos,
    ),
    summary: "Finds the numeric position of the last case-insensitive occurrence of a substring.",
    php_manual: "https://www.php.net/manual/en/function.strripos.php",
}

/// Returns `PhpType::Union([Int, Bool])` for a `strripos` call (position, or `false`).
///
/// A check hook is required because the `builtin!` macro cannot express a union return
/// type inline. Argument types are inferred by the common registry dispatch path before
/// this hook fires; arity is validated by the registry from the declared parameter list.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Union(vec![PhpType::Int, PhpType::False]))
}
