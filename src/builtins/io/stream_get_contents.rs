//! Purpose:
//! Home of the PHP `stream_get_contents` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - `check` validates the stream resource, then validates `length` (int|null) and `offset`
//!   (int) via `stream_support` helpers. Returns `Union(Str, Bool)`.
//! - `length` and `offset` are optional with defaults `null` and `-1` respectively.
//! - `returns: Mixed` is used because the union cannot be expressed through the scalar field.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::builtins::io::stream_support;
use crate::errors::CompileError;
use crate::types::PhpType;
use crate::types::checker::builtins::io::common;

builtin! {
    contract: "stream_get_contents",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::StreamGetContents,
    ),
}

/// Validates the stream resource, optional length (int|null), and optional offset (int).
/// Returns `Union(Str, Bool)` reflecting PHP's false-on-failure return.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    common::ensure_stream_resource(cx.checker, cx.name, &cx.args[0], cx.env)?;
    if let Some(length) = cx.args.get(1) {
        stream_support::ensure_optional_int(cx.checker, cx.name, "length", length, cx.env)?;
    }
    if let Some(offset) = cx.args.get(2) {
        stream_support::ensure_int(cx.checker, cx.name, "offset", offset, cx.env)?;
    }
    Ok(cx.checker.normalize_union_type(vec![PhpType::Str, PhpType::False]))
}
