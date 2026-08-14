//! Purpose:
//! Eval registry entry and implementation for `preg_replace_callback`.
//!
//! Called from:
//! - `crate::interpreter::builtins::hooks`.
//!
//! Key details:
//! - This file owns registry metadata, direct dispatch, by-value dispatch, and
//!   callback invocation for `preg_replace_callback()`.

use super::super::super::*;
use super::super::spec::EvalBuiltinDefaultValue;
use super::super::*;
use super::*;

eval_builtin! {
    name: "preg_replace_callback",
    area: Regex,
    params: [
        pattern,
        callback,
        subject,
        limit = EvalBuiltinDefaultValue::Int(-1),
        count: by_ref = EvalBuiltinDefaultValue::Null,
    ],
    by_ref: [count],
    direct: PregReplaceCallback,
    values: PregReplaceCallback,
}

/// Evaluates PHP `preg_replace_callback()` over eval expressions.
pub(in crate::interpreter) fn eval_builtin_preg_replace_callback(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [pattern, callback, subject] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let callback = eval_expr(callback, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            eval_preg_replace_callback_result_from_scope(
                pattern,
                callback,
                subject,
                Some(scope),
                context,
                values,
            )
        }
        [pattern, callback, subject, limit] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let callback = eval_expr(callback, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            let limit = eval_expr(limit, context, scope, values)?;
            eval_preg_replace_callback_result_with_count_from_scope(
                pattern,
                callback,
                subject,
                Some(limit),
                Some(scope),
                context,
                values,
            )
            .map(|(result, _)| result)
        }
        [pattern, callback, subject, limit, count] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let callback = eval_expr(callback, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            let limit = eval_expr(limit, context, scope, values)?;
            let count_target = eval_preg_matches_target(count, context, scope, values)?;
            let (result, count) = eval_preg_replace_callback_result_with_count_from_scope(
                pattern,
                callback,
                subject,
                Some(limit),
                Some(scope),
                context,
                values,
            )?;
            let count = values.int(count)?;
            eval_write_preg_matches_target(&count_target, count, context, values)?;
            Ok(result)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Evaluates a metadata-rich callback replacement call with optional counter writeback.
pub(in crate::interpreter) fn eval_builtin_preg_replace_callback_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
    let (bound, _) = bind_evaluated_ref_builtin_args(
        &["pattern", "callback", "subject", "limit", "count"],
        &evaluated_args,
        false,
    )?;
    let pattern = required_evaluated_ref_arg(&bound, 0)?;
    let callback = required_evaluated_ref_arg(&bound, 1)?;
    let subject = required_evaluated_ref_arg(&bound, 2)?;
    let limit = optional_evaluated_ref_arg(&bound, 3).map(|arg| arg.value);
    let (result, replacement_count) =
        eval_preg_replace_callback_result_with_count_from_scope(
            pattern.value,
            callback.value,
            subject.value,
            limit,
            Some(scope),
            context,
            values,
        )?;
    let Some(count) = optional_evaluated_ref_arg(&bound, 4) else {
        return Ok(result);
    };
    let target = count
        .ref_target
        .clone()
        .ok_or(EvalStatus::RuntimeFatal)?;
    let replacement_count = values.int(replacement_count)?;
    eval_write_preg_matches_target(&target, replacement_count, context, values)?;
    Ok(result)
}

/// Replaces every regex match by invoking an eval-supported callback with `$matches`.
pub(in crate::interpreter) fn eval_preg_replace_callback_result(
    pattern: RuntimeCellHandle,
    callback: RuntimeCellHandle,
    subject: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_preg_replace_callback_result_from_scope(pattern, callback, subject, None, context, values)
}

/// Replaces regex matches with optional lexical scope for callback names.
fn eval_preg_replace_callback_result_from_scope(
    pattern: RuntimeCellHandle,
    callback: RuntimeCellHandle,
    subject: RuntimeCellHandle,
    lexical_scope: Option<&ElephcEvalScope>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_preg_replace_callback_result_with_count_from_scope(
        pattern,
        callback,
        subject,
        None,
        lexical_scope,
        context,
        values,
    )
    .map(|(result, _)| result)
}

/// Replaces callback matches up to the optional limit and returns the completed count.
pub(in crate::interpreter) fn eval_preg_replace_callback_result_with_count_from_scope(
    pattern: RuntimeCellHandle,
    callback: RuntimeCellHandle,
    subject: RuntimeCellHandle,
    limit: Option<RuntimeCellHandle>,
    lexical_scope: Option<&ElephcEvalScope>,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(RuntimeCellHandle, i64), EvalStatus> {
    let regex = eval_preg_regex(pattern, values)?;
    let callback = eval_callable_with_optional_scope(callback, context, lexical_scope, values)?;
    let subject = values.string_bytes(subject)?;
    let limit = limit
        .map(|limit| eval_int_value(limit, values))
        .transpose()?
        .unwrap_or(-1);
    let mut result = Vec::with_capacity(subject.len());
    let mut cursor = 0;
    let mut replacement_count = 0i64;
    for captures in regex.captures_iter(&subject) {
        if limit >= 0 && replacement_count >= limit {
            break;
        }
        let Some(matched) = captures.get(0) else {
            continue;
        };
        result.extend_from_slice(&subject[cursor..matched.start()]);
        let matches = eval_preg_capture_array(&subject, Some(&captures), false, false, values)?;
        let callback_result =
            eval_evaluated_callable_with_values(&callback, vec![matches], context, values)?;
        let callback_result = values.cast_string(callback_result)?;
        let callback_bytes = values.string_bytes(callback_result)?;
        result.extend_from_slice(&callback_bytes);
        cursor = matched.end();
        replacement_count += 1;
    }
    result.extend_from_slice(&subject[cursor..]);
    Ok((values.string_bytes_value(&result)?, replacement_count))
}


/// Dispatches by-value `preg_replace_callback()` calls after argument binding.
pub(in crate::interpreter) fn eval_preg_replace_callback_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [pattern, callback, subject] => {
            eval_preg_replace_callback_result(*pattern, *callback, *subject, context, values)
        }
        [pattern, callback, subject, limit] => {
            eval_preg_replace_callback_result_with_count_from_scope(
                *pattern,
                *callback,
                *subject,
                Some(*limit),
                None,
                context,
                values,
            )
            .map(|(result, _)| result)
        }
        [pattern, callback, subject, limit, _count] => {
            values.warning(
                "preg_replace_callback(): Argument #5 ($count) must be passed by reference, value given",
            )?;
            eval_preg_replace_callback_result_with_count_from_scope(
                *pattern,
                *callback,
                *subject,
                Some(*limit),
                None,
                context,
                values,
            )
            .map(|(result, _)| result)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}
