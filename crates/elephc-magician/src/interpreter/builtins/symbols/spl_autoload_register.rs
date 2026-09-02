//! Purpose:
//! Eval registry entry and implementation for `spl_autoload_register`.
//!
//! Called from:
//! - `crate::interpreter::builtins::symbols`.
//!
//! Key details:
//! - Callbacks retain their PHP values until unregistered or the web request resets.

eval_builtin! {
    contract: "spl_autoload_register",
    area: Symbols,
    direct: Symbols,
    values: Symbols,
}

use super::super::super::*;

/// Evaluates direct `spl_autoload_register(...)` calls and preserves source-order arguments.
pub(in crate::interpreter) fn eval_spl_autoload_register_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (callback, throw, prepend) = match args {
        [callback] => (callback, None, None),
        [callback, throw] => (callback, Some(throw), None),
        [callback, throw, prepend] => (callback, Some(throw), Some(prepend)),
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    let callback = eval_expr(callback, context, scope, values)?;
    if let Some(throw) = throw {
        let _ = eval_expr(throw, context, scope, values)?;
    }
    let prepend = prepend
        .map(|prepend| eval_expr(prepend, context, scope, values))
        .transpose()?
        .map(|prepend| values.truthy(prepend))
        .transpose()?
        .unwrap_or(false);
    register_spl_autoload_callback(callback, prepend, context, values)
}

/// Evaluates materialized `spl_autoload_register(...)` arguments.
pub(in crate::interpreter) fn eval_spl_autoload_register_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (callback, throw, prepend) = match evaluated_args {
        [callback] => (*callback, None, None),
        [callback, throw] => (*callback, Some(*throw), None),
        [callback, throw, prepend] => (*callback, Some(*throw), Some(*prepend)),
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    let _ = throw.map(|throw| values.truthy(throw)).transpose()?;
    let prepend = prepend
        .map(|prepend| values.truthy(prepend))
        .transpose()?
        .unwrap_or(false);
    register_spl_autoload_callback(callback, prepend, context, values)
}

/// Evaluates direct `spl_autoload_unregister(...)` while preserving its callback value.
pub(in crate::interpreter) fn eval_builtin_spl_autoload_bool(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "spl_autoload_register" => eval_spl_autoload_register_declared_call(args, context, scope, values),
        "spl_autoload_unregister" => {
            let [callback] = args else {
                return Err(EvalStatus::RuntimeFatal);
            };
            let callback = eval_expr(callback, context, scope, values)?;
            eval_spl_autoload_unregister_result(callback, context, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Evaluates materialized SPL autoload registration calls.
pub(in crate::interpreter) fn eval_spl_autoload_bool_result(
    name: &str,
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "spl_autoload_register" => {
            eval_spl_autoload_register_declared_values_result(evaluated_args, context, values)
        }
        "spl_autoload_unregister" => match evaluated_args {
            [callback] => eval_spl_autoload_unregister_result(*callback, context, values),
            _ => Err(EvalStatus::RuntimeFatal),
        },
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Validates and retains one callback in the request-global SPL autoload table.
pub(crate) fn register_spl_autoload_callback(
    callback: RuntimeCellHandle,
    prepend: bool,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let _ = eval_callable(callback, context, values)?;
    register_spl_autoload_callback_unchecked(callback, prepend, context, values)
}

/// Retains an AOT callback whose native descriptor is validated when the class is requested.
pub(crate) fn register_spl_autoload_callback_unchecked(
    callback: RuntimeCellHandle,
    prepend: bool,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if context.has_autoload_callback(callback) {
        return values.bool_value(true);
    }
    let first_callback = context.has_no_autoload_callbacks();
    if first_callback
        && !crate::context::register_global_eval_autoload_context(
            context as *mut ElephcEvalContext,
            prepend,
        )
    {
        return Err(EvalStatus::RuntimeFatal);
    }
    let callback = values.retain(callback)?;
    context.register_autoload_callback(callback, prepend);
    values.bool_value(true)
}

/// Removes one retained callback and reports PHP's boolean unregister result.
fn eval_spl_autoload_unregister_result(
    callback: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let Some(callback) = context.unregister_autoload_callback(callback) else {
        return values.bool_value(false);
    };
    eval_release_value(context, values, callback)?;
    if context.has_no_autoload_callbacks() {
        crate::context::unregister_global_eval_autoload_context(context as *mut ElephcEvalContext);
    }
    values.bool_value(true)
}
