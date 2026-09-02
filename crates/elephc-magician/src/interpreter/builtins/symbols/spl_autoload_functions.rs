//! Purpose:
//! Eval registry entry and implementation for `spl_autoload_functions`.
//!
//! Called from:
//! - `crate::interpreter::builtins::symbols`.
//!
//! Key details:
//! - Returned callback values preserve request-global registration order.

eval_builtin! {
    contract: "spl_autoload_functions",
    area: Symbols,
    direct: Symbols,
    values: Symbols,
}

use super::super::super::*;

/// Evaluates direct `spl_autoload_functions()` calls.
pub(in crate::interpreter) fn eval_spl_autoload_functions_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_spl_autoload_functions(args, context, scope, values)
}

/// Evaluates materialized `spl_autoload_functions()` arguments.
pub(in crate::interpreter) fn eval_spl_autoload_functions_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_spl_autoload_functions_result(evaluated_args, context, values)
}

/// Evaluates `spl_autoload_functions()`.
pub(in crate::interpreter) fn eval_builtin_spl_autoload_functions(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    _scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !args.is_empty() {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_spl_autoload_functions_result(&[], context, values)
}

/// Evaluates materialized `spl_autoload_functions()`.
pub(in crate::interpreter) fn eval_spl_autoload_functions_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !evaluated_args.is_empty() {
        return Err(EvalStatus::RuntimeFatal);
    }
    let callbacks = context.autoload_callbacks();
    let mut result = values.array_new(callbacks.len())?;
    for (index, callback) in callbacks.into_iter().enumerate() {
        let index = values.int(index as i64)?;
        result = values.array_set(result, index, callback)?;
    }
    Ok(result)
}
