//! Purpose:
//! Home of the PHP `preg_match` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The third param `matches` is a by-reference array output and the optional flags and offset
//!   parameters follow PHP's public five-parameter signature.
//! - `lazy_check: true` suppresses the registry's default pre-inference loop so the hook
//!   can infer args[0] and args[1] (pattern and subject) while deliberately skipping
//!   inference of args[2] (`$matches`). `$matches` is a write-only output parameter;
//!   it is not declared before the call and inferring it would produce an
//!   "Undefined variable" error.
//! - `check` validates that args[2] (when present) is a `Variable` expression; passing
//!   a non-variable to the by-ref `$matches` param is a compile error.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "preg_match",
    area: System,
    params: [
        pattern: Str,
        subject: Str,
        ref matches: ArrayMixed = DefaultSpec::Null,
        flags: Int = DefaultSpec::Int(0),
        offset: Int = DefaultSpec::Int(0),
    ],
    returns: Int,
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::with_argument_lowering(
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::PregMatch),
        crate::builtins::semantics::BuiltinArgumentLowering::PositionalRegex,
    ),
    summary: "Performs a regular expression match.",
}

/// Infers every input while leaving the by-reference capture destination write-only.
///
/// Infers args[0] (pattern) and args[1] (subject) to trigger type-environment side
/// effects, but deliberately skips inference of args[2] (`$matches`) because it is a
/// write-only output parameter that may be undefined before the call. Shared call validation
/// owns l-value enforcement for the by-reference destination.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    cx.checker.infer_type(&cx.args[1], cx.env)?;
    if cx.args.len() >= 3
        && !cx
            .checker
            .is_by_ref_argument_lvalue(&cx.args[2], cx.env)?
    {
        return Err(CompileError::new(
            cx.args[2].span,
            "preg_match() parameter $matches must be passed a variable",
        ));
    }
    if cx.args.len() >= 4 {
        cx.checker.infer_type(&cx.args[3], cx.env)?;
    }
    if cx.args.len() >= 5 {
        cx.checker.infer_type(&cx.args[4], cx.env)?;
    }
    Ok(PhpType::Int)
}
