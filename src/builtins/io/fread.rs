//! Purpose:
//! Home of the PHP `fread` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` calls `ensure_stream_resource` on the stream argument for validation and
//!   returns `Mixed`, reflecting PHP's `string|false`: a read that FAILS answers false,
//!   while an exhausted stream answers "". `returns: Mixed` is what `fgets()` uses for the
//!   same shape — the precise union cannot be expressed through the scalar `returns:` field.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "fread",
    area: Io,
    params: [stream: Mixed, length: Int],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Fread,
    ),
    summary: "Binary-safe file read.",
    php_manual: "function.fread",
}

/// Validates the stream argument is a stream resource and returns `Mixed` for `string|false`.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    crate::types::checker::builtins::io::common::ensure_stream_resource(
        cx.checker,
        cx.name,
        &cx.args[0],
        cx.env,
    )?;
    Ok(PhpType::Mixed)
}
