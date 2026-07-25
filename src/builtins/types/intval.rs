//! Purpose:
//! Home of the PHP `intval` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - The optional `$base` parameter routes through the typed `Intval` runtime target so
//!   non-decimal string parsing remains target-aware.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    name: "intval",
    area: Types,
    params: [value: Mixed, base: Int = DefaultSpec::Int(10)],
    returns: Int,
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Intval,
    ),
    summary: "Returns the integer value of a variable.",
    php_manual: "function.intval",
}

/// Validates PHP's one-or-two-argument `intval()` surface and the integer base.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    cx.checker.infer_type(&cx.args[0], cx.env)?;
    if let Some(base) = cx.args.get(1) {
        let base_ty = cx.checker.infer_type(base, cx.env)?;
        if base_ty != PhpType::Int {
            return Err(CompileError::new(
                base.span,
                "intval() base argument must be int",
            ));
        }
    }
    Ok(PhpType::Int)
}
