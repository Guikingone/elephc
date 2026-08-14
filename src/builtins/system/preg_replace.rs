//! Purpose:
//! Home of the PHP `preg_replace` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - The optional replacement limit and by-reference replacement counter follow
//!   PHP's public five-parameter signature.
//! - The checker leaves the counter destination write-only while inferring all
//!   value inputs, allowing an undefined variable to receive the result.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "preg_replace",
    area: System,
    params: [
        pattern: Str,
        replacement: Str,
        subject: Str,
        limit: Int = DefaultSpec::Int(-1),
        ref count: Int = DefaultSpec::Null,
    ],
    returns: Str,
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::with_argument_lowering(
        crate::builtins::semantics::runtime_fn_semantics(
            crate::ir::RuntimeFnId::PregReplace,
        ),
        crate::builtins::semantics::BuiltinArgumentLowering::PositionalRegex,
    ),
    summary: "Performs a regular expression search and replace.",
}

/// Infers replacement inputs while leaving the optional counter destination write-only.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for argument in cx.args.iter().take(4) {
        cx.checker.infer_type(argument, cx.env)?;
    }
    if cx.args.len() >= 5
        && !cx
            .checker
            .is_by_ref_argument_lvalue(&cx.args[4], cx.env)?
    {
        return Err(CompileError::new(
            cx.args[4].span,
            "preg_replace() parameter $count must be passed a variable",
        ));
    }
    Ok(PhpType::Str)
}
