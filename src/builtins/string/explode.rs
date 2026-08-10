//! Purpose:
//! Home of the PHP `explode` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The declared signature carries the full golden param list (`separator`, `string`,
//!   `limit`); `$limit` defaults to `PHP_INT_MAX`, which is how php-src spells "no limit",
//!   so `RuntimeFnId::Explode` sees one uniform three-argument contract.
//! - `check` returns `PhpType::Array(Box::new(PhpType::Str))`. A check hook is required
//!   because the `builtin!` macro `returns:` field cannot express an array type inline.
//!   Argument types are inferred by the common registry dispatch path before the hook
//!   fires.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "explode",
    area: String,
    params: [separator: Str, string: Str, limit: Int = DefaultSpec::IntMax],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Explode,
    ),
    summary: "Splits a string by a separator into an array of substrings.",
    php_manual: "https://www.php.net/manual/en/function.explode.php",
}

/// Returns `PhpType::Array(Box::new(PhpType::Str))` for an `explode` call.
///
/// A check hook is required because the `builtin!` macro cannot express array return
/// types inline. Argument types are inferred by the common registry dispatch path before
/// this hook fires; arity is validated by the registry from the declared parameter list.
fn check(_cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
