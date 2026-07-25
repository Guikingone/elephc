//! Purpose:
//! Home of the PHP `preg_replace` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - Accepts PHP's optional `$limit` and by-reference `$count` parameters.
//! - The lazy check skips eager reads of the write-only `$count` output variable.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "preg_replace",
    area: System,
    params: [pattern: Str, replacement: Str, subject: Str, limit: Int = DefaultSpec::Int(-1), ref count: Mixed = DefaultSpec::Null],
    returns: Str,
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::PregReplace,
    ),
    summary: "Performs a regular expression search and replace.",
}

/// Validates the optional by-reference `$count` output and infers only input operands.
///
/// `$count` is deliberately not inferred before the call because PHP defines it through
/// the builtin. When supplied it must be a variable so the EIR lowering can write the
/// replacement count back into caller storage.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    for arg in cx.args.iter().take(4) {
        cx.checker.infer_type(arg, cx.env)?;
    }
    if cx.args.len() == 5 && !matches!(cx.args[4].kind, ExprKind::Variable(_)) {
        return Err(CompileError::new(
            cx.args[4].span,
            "preg_replace() parameter $count must be passed a variable",
        ));
    }
    Ok(PhpType::Str)
}
