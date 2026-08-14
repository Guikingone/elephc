//! Purpose:
//! Declarative eval registry entry for `class_parents`.
//!
//! Called from:
//! - `crate::interpreter::builtins::symbols`.
//!
//! Key details:
//! - Shared class-relation logic lives in `class_implements`.

eval_builtin! {
    contract: "class_parents",
    area: Symbols,
    direct: Symbols,
    values: Symbols,
}

use super::super::super::*;

/// Dispatches direct eval calls for the `class_parents` symbol builtin.
pub(in crate::interpreter) fn eval_class_parents_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::class_implements::eval_builtin_class_relation("class_parents", args, context, scope, values)
}

/// Dispatches evaluated-argument calls for the `class_parents` symbol builtin.
pub(in crate::interpreter) fn eval_class_parents_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::class_implements::eval_class_relation_result("class_parents", evaluated_args, context, values)
}
