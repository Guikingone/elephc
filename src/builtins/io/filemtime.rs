//! Purpose:
//! Home of the PHP `filemtime` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - `check` returns `Union(Int, False)` reflecting PHP behaviour where `filemtime`
//!   returns the modification time as a Unix timestamp on success or `false` on failure,
//!   matching its `fileatime` and `filectime` siblings.
//! - The registry pre-infers arguments before calling this hook.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "filemtime",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Filemtime,
    ),
}

/// Returns `Union(Int, False)` reflecting that `filemtime` can return a timestamp or `false`.
///
/// This used to declare a plain `Int` — described as "fully determined by its declaration",
/// which it was not: the declaration is what DISCARDED the failure, so a path that could not be
/// stat'ed answered with whatever the stat buffer happened to hold.
///
/// The registry pre-infers arguments before calling this hook.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    Ok(cx.checker.normalize_union_type(vec![PhpType::Int, PhpType::False]))
}
