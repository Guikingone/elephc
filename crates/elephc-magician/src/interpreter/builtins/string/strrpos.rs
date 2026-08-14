//! Purpose:
//! Declarative eval registry entry for `strrpos`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the string-position hook.

eval_builtin! {
    contract: "strrpos",
    area: String,
    direct: StringPosition,
    values: StringPosition,
}

use super::super::super::*;

/// Evaluates PHP `strrpos(...)` over haystack, needle, and optional offset expressions.
pub(in crate::interpreter) fn eval_builtin_strrpos(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_builtin_string_position_named("strrpos", args, context, scope, values)
}

/// Applies PHP `strrpos(...)` to evaluated haystack, needle, and optional offset values.
pub(in crate::interpreter) fn eval_strrpos_result(
    haystack: RuntimeCellHandle,
    needle: RuntimeCellHandle,
    offset: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_string_position_named_result("strrpos", haystack, needle, offset, values)
}
