//! Purpose:
//! Home of the PHP `getenv` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(Str, Bool)` to reflect PHP's behaviour where `getenv`
//!   returns the value string on success or `false` if the variable is unset.
//! - The EIR result is that SAME union. It used to be overridden to a raw `Str` to match
//!   the backend helper's string registers, which made `false` indistinguishable from `""`
//!   — `getenv("ABSENT") === false` answered "present" (measured against php-src 8.5.6).
//!   `__rt_getenv` always did carry the distinction, as libc's NULL versus a pointer to
//!   `""`, so the lowering boxes on the POINTER and the checker's honest type stands.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "getenv",
    area: System,
    params: [name: Str],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Getenv,
    ),
    summary: "Gets the value of an environment variable.",
}

/// Returns `Union(Str, Bool)` reflecting that `getenv` can return a string or `false`.
///
/// Infers the argument type to trigger type-environment side effects before returning
/// the normalized union type.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
