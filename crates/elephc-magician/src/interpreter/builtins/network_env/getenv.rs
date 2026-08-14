//! Purpose:
//! Eval registry entry and implementation for `getenv`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Unset variables return an empty string to match current eval semantics.
//! - A null or omitted name materializes the process environment as an associative array.

use super::*;
use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "getenv",
    area: NetworkEnv,
    params: [
        name = EvalBuiltinDefaultValue::Null,
        local_only = EvalBuiltinDefaultValue::Bool(false)
    ],
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates the zero-to-two-argument environment lookup forms over eval expressions.
pub(in crate::interpreter) fn eval_builtin_getenv(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [] => eval_getenv_all_result(values),
        [name] | [name, _] => {
            let name = eval_expr(name, context, scope, values)?;
            if let Some(local_only) = args.get(1) {
                let local_only = eval_expr(local_only, context, scope, values)?;
                let _ = values.truthy(local_only)?;
            }
            eval_getenv_result(name, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Reads one variable, or enumerates all variables when the name is null.
pub(in crate::interpreter) fn eval_getenv_result(
    name: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.is_null(name)? {
        return eval_getenv_all_result(values);
    }
    let name = values.string_bytes(name)?;
    let name = String::from_utf8_lossy(&name);
    let value = std::env::var_os(name.as_ref())
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    values.string(&value)
}

/// Materializes the process environment as a string-keyed associative array.
pub(in crate::interpreter) fn eval_getenv_all_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let entries: Vec<_> = std::env::vars_os().collect();
    let mut result = values.assoc_new(entries.len().max(1))?;
    for (name, value) in entries {
        let key = values.string(&name.to_string_lossy())?;
        let value = values.string(&value.to_string_lossy())?;
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}
