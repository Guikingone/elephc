//! Purpose:
//! Declarative eval registry entry for `mkdir`.
//!
//! Called from:
//! - `crate::interpreter::builtins::filesystem`.
//!
//! Key details:
//! - Runtime dispatch preserves optional permissions and recursive creation for
//!   local paths and passes the matching mode/options pair to userspace wrappers.

eval_builtin! {
    contract: "mkdir",
    area: Filesystem,
    direct: Filesystem,
    values: Filesystem,
}

use super::super::super::*;
use crate::stream_wrappers;
use std::os::unix::fs::DirBuilderExt;

/// Dispatches direct eval calls for the `mkdir` filesystem builtin through the area dispatcher.
pub(in crate::interpreter) fn eval_mkdir_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [directory] => {
            let directory = eval_expr(directory, context, scope, values)?;
            eval_mkdir_result(directory, None, None, context, values)
        }
        [directory, permissions] => {
            let directory = eval_expr(directory, context, scope, values)?;
            let permissions = eval_expr(permissions, context, scope, values)?;
            eval_mkdir_result(directory, Some(permissions), None, context, values)
        }
        [directory, permissions, recursive] | [directory, permissions, recursive, _] => {
            let directory = eval_expr(directory, context, scope, values)?;
            let permissions = eval_expr(permissions, context, scope, values)?;
            let recursive = eval_expr(recursive, context, scope, values)?;
            if let [_, _, _, stream_context] = args {
                let _ = eval_expr(stream_context, context, scope, values)?;
            }
            eval_mkdir_result(
                directory,
                Some(permissions),
                Some(recursive),
                context,
                values,
            )
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Dispatches evaluated-argument calls for the `mkdir` filesystem builtin through the area dispatcher.
pub(in crate::interpreter) fn eval_mkdir_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [directory] => eval_mkdir_result(*directory, None, None, context, values),
        [directory, permissions] => {
            eval_mkdir_result(*directory, Some(*permissions), None, context, values)
        }
        [directory, permissions, recursive] | [directory, permissions, recursive, _] => {
            eval_mkdir_result(
                *directory,
                Some(*permissions),
                Some(*recursive),
                context,
                values,
            )
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Creates one local directory with PHP-compatible mode and recursive semantics.
pub(in crate::interpreter) fn eval_mkdir_result(
    directory: RuntimeCellHandle,
    permissions: Option<RuntimeCellHandle>,
    recursive: Option<RuntimeCellHandle>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let path = eval_path_string(directory, values)?;
    let permissions = permissions
        .map(|value| eval_int_value(value, values))
        .transpose()?
        .unwrap_or(0o777) as u32;
    let recursive = recursive
        .map(|value| values.truthy(value))
        .transpose()?
        .unwrap_or(false);
    let options = i64::from(recursive);
    if let Some(result) = super::user_wrapper_path_ops::eval_user_wrapper_mkdir_result(
        &path,
        i64::from(permissions),
        options,
        context,
        values,
    )? {
        return Ok(result);
    }
    let Some(path) = stream_wrappers::local_filesystem_path(&path) else {
        return values.bool_value(false);
    };
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(permissions).recursive(recursive);
    values.bool_value(builder.create(path).is_ok())
}
