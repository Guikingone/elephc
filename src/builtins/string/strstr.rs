//! Purpose:
//! Home of the PHP `strstr` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The result type is `string|false`, NOT `string`. PHP returns `false` when the needle is
//!   absent (verified on reference 8.5.6: `strstr("hello", "zzz")` is `bool(false)`), which is
//!   what makes the idiomatic `if (strstr($h, $n) !== false)` meaningful. elephc used to declare
//!   `Str` and lower a miss to the EMPTY STRING, so that comparison was always true and the miss
//!   was indistinguishable from a hit on an empty suffix.
//! - `Union(Str, False)` has backend representation `Mixed`, so `strstr` now yields a boxed cell
//!   — the same shape `phpversion($ext)` and `opcache_get_status()` use for their `string|false`
//!   and `array|false` results. `strings::lower_strstr` boxes both arms accordingly.
//! - The third parameter (`$before_needle`) is accepted, matching PHP: a truthy flag returns the
//!   part of the haystack BEFORE the needle instead of from the needle onwards. It used to be
//!   declared but capped away by `max_args: 2`; the cap is gone, so `check_arity` now derives
//!   "strstr() takes 2 or 3 arguments" from the params like `substr()` does.
//! - No `$before_needle` type check is needed: PHP takes any truthy value there, and the lowering
//!   materializes it through the shared truthiness helper.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "strstr",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Strstr,
    ),
}

/// Returns `string|false` for every `strstr()` call.
///
/// ONE type for every arity and every argument shape, deliberately. Whether the needle occurs in
/// the haystack is a run-time fact even when both are literals as far as this hook is concerned,
/// and `$before_needle` only selects WHICH substring is returned, never whether one is returned
/// at all — so no argument pattern can narrow this to plain `Str` without contradicting the value
/// `lower_strstr` produces. Declaring the union unconditionally is what keeps the checker's answer
/// and the backend's boxed `Mixed` layout in agreement (`Union(Str, False).codegen_repr()` is
/// `Mixed`); the `phpversion` builtin documents the same reasoning and the miscompile that
/// disagreement causes.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
