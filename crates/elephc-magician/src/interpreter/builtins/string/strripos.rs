//! Purpose:
//! Declarative eval registry entry for `strripos`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the shared string-position hook,
//!   which folds both operands with php-src's ASCII-only rule before the ordinary byte search.
//! - `$offset` follows `strrpos()`: a negative value bounds where a match may END rather than
//!   where the scan starts, and an offset outside the haystack is reference PHP's catchable
//!   `ValueError`, reported here as `EvalStatus::RuntimeFatal`.

eval_builtin! {
    contract: "strripos",
    area: String,
    direct: StringPosition,
    values: StringPosition,
}

use super::super::super::*;

/// Evaluates PHP `strripos(...)` over haystack, needle, and optional offset expressions.
pub(in crate::interpreter) fn eval_builtin_strripos(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_builtin_string_position_named("strripos", args, context, scope, values)
}

/// Applies PHP `strripos(...)` to evaluated haystack, needle, and optional offset values.
pub(in crate::interpreter) fn eval_strripos_result(
    haystack: RuntimeCellHandle,
    needle: RuntimeCellHandle,
    offset: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_string_position_named_result("strripos", haystack, needle, offset, values)
}
