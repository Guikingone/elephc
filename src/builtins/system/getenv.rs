//! Purpose:
//! Home of the PHP `getenv` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(Str, False)` to reflect PHP's behaviour where `getenv`
//!   returns the value string on success or `false` if the variable is unset.
//! - The EIR result carries that union too. It used to be overridden to plain
//!   `Str` "for present and missing variables alike", which is where the two
//!   answers were collapsed: an unset variable came back as `""`, so
//!   `getenv($x) !== false` — the idiom for "is this set" — was true for every
//!   name, silently.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "getenv",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Getenv,
    ),
}

/// Returns `Union(Str, False)` reflecting that `getenv` can return a string or `false`.
///
/// Infers the argument type to trigger type-environment side effects before returning
/// the normalized union type.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
