//! Purpose:
//! Eval registry entry and implementation for `getenv`.
//!
//! Called from:
//! - `crate::interpreter::builtins::network_env` direct and by-value dispatch.
//!
//! Key details:
//! - Missing names return false; present names and environment arrays preserve raw bytes.
//! - A null or omitted name materializes the process environment as an associative array.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

use super::*;
eval_builtin! {
    contract: "getenv",
    area: NetworkEnv,
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
///
/// Missing names answer `false` -- php distinguishes it from a variable that IS set to the
/// empty string -- and a present name's bytes are read without a lossy UTF-8 round trip, so a
/// non-UTF-8 environment value (or lookup name) survives intact.
pub(in crate::interpreter) fn eval_getenv_result(
    name: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if values.is_null(name)? {
        return eval_getenv_all_result(values);
    }
    let name = values.string_bytes(name)?;
    match std::env::var_os(OsStr::from_bytes(&name)) {
        Some(value) => values.string_bytes_value(value.as_bytes()),
        None => values.bool_value(false),
    }
}

/// Materializes the process environment as a string-keyed associative array.
pub(in crate::interpreter) fn eval_getenv_all_result(
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let entries: Vec<_> = std::env::vars_os().collect();
    let mut result = values.assoc_new(entries.len().max(1))?;
    for (name, value) in entries {
        let key = values.string_bytes_value(name.as_bytes())?;
        let value = values.string_bytes_value(value.as_bytes())?;
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}
