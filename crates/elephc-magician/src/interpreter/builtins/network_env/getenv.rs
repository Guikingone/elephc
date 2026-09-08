//! Purpose:
//! Eval registry entry and implementation for `getenv`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Accepts 0–2 arguments, matching the shared catalogue. An omitted or null
//!   name answers the whole live environment as an associative array.
//! - `local_only` is evaluated for side effects and ignored: eval has no
//!   environment separate from the process's, same as AOT CLI.
//! - Unset variables return an empty string to match current eval semantics.

use super::*;

eval_builtin! {
    contract: "getenv",
    area: NetworkEnv,
    direct: NetworkEnv,
    values: NetworkEnv,
}

/// Evaluates PHP `getenv()`, `getenv($name)`, and `getenv($name, $local_only)`.
pub(in crate::interpreter) fn eval_builtin_getenv(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [] => eval_getenv_all_result(values),
        [name] => {
            let name = eval_expr(name, context, scope, values)?;
            eval_getenv_name_result(name, values)
        }
        [name, local_only] => {
            let name = eval_expr(name, context, scope, values)?;
            let _local_only = eval_expr(local_only, context, scope, values)?;
            eval_getenv_name_result(name, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Reads one environment variable, or the whole environment when `$name` is null.
pub(in crate::interpreter) fn eval_getenv_name_result(
    name: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.is_null(name)? {
        return eval_getenv_all_result(values);
    }
    eval_getenv_result(name, values)
}

/// Reads one environment variable and returns an empty string when it is unset.
pub(in crate::interpreter) fn eval_getenv_result(
    name: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let name = values.string_bytes(name)?;
    let name = String::from_utf8_lossy(&name);
    let value = std::env::var_os(name.as_ref())
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    values.string(&value)
}

/// Builds the live process environment as a string-keyed associative array.
pub(in crate::interpreter) fn eval_getenv_all_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let entries: Vec<(String, String)> = std::env::vars().collect();
    let mut result = values.assoc_new(entries.len())?;
    for (key, value) in entries {
        let key = values.string(&key)?;
        let value = values.string(&value)?;
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}
