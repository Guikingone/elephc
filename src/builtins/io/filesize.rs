//! Purpose:
//! Home of the PHP `filesize` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(Int, False)` reflecting PHP behaviour where `filesize`
//!   returns the size in bytes on success or `false` on failure, matching its
//!   `fileatime` and `filectime` siblings.
//! - The registry pre-infers arguments before calling this hook.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "filesize",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Filesize,
    ),
}

/// Returns `Union(Int, False)` reflecting that `filesize` can return a byte count or `false`.
///
/// This used to declare a plain `Int` — described as "a pure-data builtin whose return type is
/// fully determined by its declaration", which it was not: the declaration is what DISCARDED the
/// failure, so an unstatable path answered `0`, a legitimate size for an empty file.
///
/// The registry pre-infers arguments before calling this hook.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx.checker.normalize_union_type(vec![PhpType::Int, PhpType::False]))
}
