//! Purpose:
//! Declarative eval registry entry for `next`.
//!
//! Called from:
//! - `crate::interpreter::builtins::array`.
//!
//! Key details:
//! - Direct calls stay on the source-sensitive by-reference path so the moved
//!   internal pointer is recorded against the caller's array cell.

use super::super::super::*;

eval_builtin! {
    contract: "next",
    area: Array,
    direct: none,
    values: ArrayMutating,
}
/// Dispatches by-value callable eval calls for the `next` internal pointer builtin.
pub(in crate::interpreter) fn eval_next_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::array_pointer::eval_array_pointer_values_result("next", evaluated_args, context, values)
}
