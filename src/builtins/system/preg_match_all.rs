//! Purpose:
//! Home of the PHP `preg_match_all` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The optional by-reference capture array, flags, and offset follow PHP's public signature.
//! - The checker leaves the capture destination write-only while inferring all value inputs.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "preg_match_all",
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
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::PregMatchAll),
        crate::builtins::semantics::BuiltinArgumentLowering::PositionalRegex,
    ),
    summary: "Performs a global regular expression match and returns the number of matches.",
}

/// Infers every input while leaving the by-reference capture destination write-only.
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
            "preg_match_all() parameter $matches must be passed a variable",
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
