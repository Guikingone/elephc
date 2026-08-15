//! Purpose:
//! Home of the PHP `sscanf` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts required `string` and `format` params plus a variadic `vars` list.
//! - `check` returns `PhpType::Array(Box::new(PhpType::Str))` because the macro
//!   `returns:` field cannot express a parameterized array type inline.
//! - The by-ref `$vars` output form is REFUSED rather than mis-executed. php assigns each
//!   field through the reference and returns the field COUNT; this backend has no way to
//!   express that yet — `ParamSpec` carries `by_ref`, but `variadic: Some("vars")` is a bare
//!   NAME with no by-ref marker, so the checker never introduces the variables and the
//!   lowering never writes them. Left accepted, the call compiled and silently did the wrong
//!   thing on both counts at once: measured with `php -n` (8.5.6),
//!   `$name = "unset"; $age = -1; $n = sscanf("alice 30", "%s %d", $name, $age);` gives php
//!   `int(2)` / `"alice"` / `int(30)`, while this backend returned the ARRAY
//!   `["alice", "30"]` and left BOTH variables untouched.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "sscanf",
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Sscanf,
    ),
}

/// Returns `PhpType::Array(Box::new(PhpType::Str))` for a `sscanf` call.
///
/// A check hook is required because the `builtin!` macro cannot express a
/// parameterized array return type inline. The hook also refuses the by-ref `$vars`
/// output form, which this backend cannot express and previously mis-executed in silence.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    reject_by_ref_vars(cx.name, cx.args.len(), cx.span)?;
    Ok(PhpType::Array(Box::new(PhpType::Str)))
}

/// Refuses the `scanf` by-ref `$vars` output form for `sscanf()`/`fscanf()`.
///
/// Shared by both builtins so the two stay on one wording. `arg_count` is the full argument
/// count including the two leading required parameters, so anything past 2 is a `$vars` entry.
pub(crate) fn reject_by_ref_vars(
    name: &str,
    arg_count: usize,
    span: crate::span::Span,
) -> Result<(), CompileError> {
    if arg_count <= 2 {
        return Ok(());
    }
    Err(CompileError::new(
        span,
        &format!(
            "{}(): the by-ref $vars output form is not supported; \
             call it with only the format and read the returned array",
            name
        ),
    ))
}
