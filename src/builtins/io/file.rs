//! Purpose:
//! Home of the PHP `file` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - PHP's signature is `file(string $filename, int $flags = 0, $context = null)`; elephc declares
//!   `filename` and `flags`. The stream-context parameter is not modelled, so it is left out
//!   rather than accepted and ignored.
//! - `flags` is an ordinary run-time integer bitmask (`FILE_USE_INCLUDE_PATH`,
//!   `FILE_IGNORE_NEW_LINES`, `FILE_SKIP_EMPTY_LINES`), NOT a shape-changing literal: the result
//!   is `Array<Str>` for every flag combination, so it does not need to be known at compile time.
//! - `check` returns `Array<Str>` (the file's lines). A check hook is required
//!   because the array return type cannot be expressed through the scalar `returns:`
//!   field.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "file",
    area: Io,
    params: [filename: Str, flags: Int = DefaultSpec::Int(0)],
    returns: Mixed,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::File,
    ),
    summary: "Reads an entire file into an array.",
    php_manual: "function.file",
}

/// Returns `Array<Str>` reflecting that `file` yields the file's lines as strings.
///
/// The `$flags` bitmask only changes the CONTENT of the lines (trailing newline removal and
/// empty-line skipping), never the container shape, so the result type is flag-independent.
/// Arity (1 or 2) is pre-validated by the registry, and the registry already inferred every
/// argument once for side effects.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}
