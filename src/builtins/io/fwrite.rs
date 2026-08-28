//! Purpose:
//! Home of the PHP `fwrite` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` calls `ensure_stream_resource` on the stream argument for validation and
//!   returns `Int`. Arguments are pre-inferred by the registry before the hook runs.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "fwrite",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Fwrite,
    ),
}

/// Validates the stream argument and returns PHP's `int|false` result union.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    crate::types::checker::builtins::io::common::ensure_stream_resource(
        cx.checker,
        cx.name,
        &cx.args[0],
        cx.env,
    )?;
    Ok(cx
        .checker
        .normalize_union_type(vec![PhpType::Int, PhpType::Bool]))
}
