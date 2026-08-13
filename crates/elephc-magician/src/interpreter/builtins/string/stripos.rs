//! Purpose:
//! Declarative eval registry entry for `stripos`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the shared string-position hook,
//!   which folds both operands with php-src's ASCII-only rule before the ordinary byte search.
//! - `$offset` follows `strpos()`: a negative value is resolved against the haystack length and
//!   an offset outside the haystack is reference PHP's catchable `ValueError`, reported here as
//!   `EvalStatus::RuntimeFatal` because eval has no throw machinery.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "stripos",
    area: String,
    params: [haystack, needle, offset = EvalBuiltinDefaultValue::Int(0)],
    direct: StringPosition,
    values: StringPosition,
}

use super::super::super::*;

/// Evaluates PHP `stripos(...)` over haystack, needle, and optional offset expressions.
pub(in crate::interpreter) fn eval_builtin_stripos(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_builtin_string_position_named("stripos", args, context, scope, values)
}

/// Applies PHP `stripos(...)` to evaluated haystack, needle, and optional offset values.
pub(in crate::interpreter) fn eval_stripos_result(
    haystack: RuntimeCellHandle,
    needle: RuntimeCellHandle,
    offset: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_string_position_named_result("stripos", haystack, needle, offset, values)
}
