//! Purpose:
//! Declarative eval registry entry for `reset`.
//!
//! Called from:
//! - `crate::interpreter::builtins::array`.
//!
//! Key details:
//! - Direct calls stay on the source-sensitive by-reference path so the rewound
//!   internal pointer is recorded against the caller's array cell.

use super::super::super::*;

eval_builtin! {
    contract: "reset",
    area: Array,
    direct: none,
    values: ArrayMutating,
}
/// Dispatches by-value callable eval calls for the `reset` internal pointer builtin.
pub(in crate::interpreter) fn eval_reset_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::array_pointer::eval_array_pointer_values_result("reset", evaluated_args, context, values)
}
