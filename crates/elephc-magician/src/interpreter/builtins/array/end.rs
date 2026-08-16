//! Purpose:
//! Declarative eval registry entry for `end`.
//!
//! Called from:
//! - `crate::interpreter::builtins::array`.
//!
//! Key details:
//! - Direct calls stay on the source-sensitive by-reference path so the advanced
//!   internal pointer is recorded against the caller's array cell.

use super::super::super::*;

eval_builtin! {
    contract: "end",
    area: Array,
    direct: none,
    values: ArrayMutating,
}
/// Dispatches by-value callable eval calls for the `end` internal pointer builtin.
pub(in crate::interpreter) fn eval_end_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::array_pointer::eval_array_pointer_values_result("end", evaluated_args, context, values)
}
