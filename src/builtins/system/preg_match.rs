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
//! - A literal pattern with the `u` modifier returns `int|false`: an invalid UTF-8 subject fails.

use crate::builtins::spec::BuiltinCheckCtx;
use crate::errors::CompileError;
use crate::types::PhpType;

builtin! {
    contract: "preg_match",
    check: check,
    lazy_check: true,
    semantics: crate::builtins::semantics::with_argument_lowering(
        crate::builtins::semantics::runtime_fn_semantics(crate::ir::RuntimeFnId::PregMatch),
        crate::builtins::semantics::BuiltinArgumentLowering::PositionalRegex,
    ),
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
    if literal_pattern_is_utf8(&cx.args[0]) {
        return Ok(PhpType::Union(vec![PhpType::Int, PhpType::False]));
    }
    Ok(PhpType::Int)
}

/// Returns whether a preg pattern is a string literal carrying the `u` modifier.
///
/// Under `u` a subject that is not valid UTF-8 makes the call fail, and php reports the failure
/// in the RESULT: `preg_match` returns false and `preg_replace` null. Symfony's console
/// `Helper::width()` tells a binary string apart by exactly that null. Only a literal pattern is
/// widened, so a call whose pattern cannot fail this way keeps its unboxed result.
pub(crate) fn literal_pattern_is_utf8(pattern: &crate::parser::ast::Expr) -> bool {
    let crate::parser::ast::ExprKind::StringLiteral(pattern) = &pattern.kind else {
        return false;
    };
    let pattern = pattern.trim_start();
    let Some(open) = pattern.chars().next() else {
        return false;
    };
    let close = match open {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        '<' => '>',
        other => other,
    };
    pattern
        .rfind(close)
        .filter(|&end| end > 0)
        .is_some_and(|end| pattern[end + close.len_utf8()..].contains('u'))
}
