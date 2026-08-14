//! Purpose:
//! Eval registry entry and implementation for `preg_replace`.
//!
//! Called from:
//! - `crate::interpreter::builtins::hooks`.
//!
//! Key details:
//! - This file owns registry metadata, direct dispatch, by-value dispatch, and
//!   backreference replacement expansion for `preg_replace()`.

use super::super::super::*;
use super::super::spec::EvalBuiltinDefaultValue;
use super::*;

eval_builtin! {
    name: "preg_replace",
    area: Regex,
    params: [
        pattern,
        replacement,
        subject,
        limit = EvalBuiltinDefaultValue::Int(-1),
        count: by_ref = EvalBuiltinDefaultValue::Null,
    ],
    by_ref: [count],
    direct: PregReplace,
    values: PregReplace,
}

/// Evaluates PHP `preg_replace()` over eval expressions.
pub(in crate::interpreter) fn eval_builtin_preg_replace(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [pattern, replacement, subject] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let replacement = eval_expr(replacement, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            eval_preg_replace_result(pattern, replacement, subject, values)
        }
        [pattern, replacement, subject, limit] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let replacement = eval_expr(replacement, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            let limit = eval_expr(limit, context, scope, values)?;
            eval_preg_replace_result_with_count(
                pattern,
                replacement,
                subject,
                Some(limit),
                values,
            )
            .map(|(result, _)| result)
        }
        [pattern, replacement, subject, limit, count] => {
            let pattern = eval_expr(pattern, context, scope, values)?;
            let replacement = eval_expr(replacement, context, scope, values)?;
            let subject = eval_expr(subject, context, scope, values)?;
            let limit = eval_expr(limit, context, scope, values)?;
            let count_target = eval_preg_matches_target(count, context, scope, values)?;
            let (result, count) = eval_preg_replace_result_with_count(
                pattern,
                replacement,
                subject,
                Some(limit),
                values,
            )?;
            let count = values.int(count)?;
            eval_write_preg_matches_target(&count_target, count, context, values)?;
            Ok(result)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Evaluates a metadata-rich call while preserving the optional counter destination.
pub(in crate::interpreter) fn eval_builtin_preg_replace_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
    let (bound, _) = bind_evaluated_ref_builtin_args(
        &["pattern", "replacement", "subject", "limit", "count"],
        &evaluated_args,
        false,
    )?;
    let pattern = required_evaluated_ref_arg(&bound, 0)?;
    let replacement = required_evaluated_ref_arg(&bound, 1)?;
    let subject = required_evaluated_ref_arg(&bound, 2)?;
    let limit = optional_evaluated_ref_arg(&bound, 3).map(|arg| arg.value);
    let (result, replacement_count) = eval_preg_replace_result_with_count(
        pattern.value,
        replacement.value,
        subject.value,
        limit,
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

/// Replaces every regex match with a PHP-style backreference-expanded replacement.
pub(in crate::interpreter) fn eval_preg_replace_result(
    pattern: RuntimeCellHandle,
    replacement: RuntimeCellHandle,
    subject: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_preg_replace_result_with_count(pattern, replacement, subject, None, values)
        .map(|(result, _)| result)
}

/// Replaces regex matches up to the optional limit and returns the replacement count.
pub(in crate::interpreter) fn eval_preg_replace_result_with_count(
    pattern: RuntimeCellHandle,
    replacement: RuntimeCellHandle,
    subject: RuntimeCellHandle,
    limit: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<(RuntimeCellHandle, i64), EvalStatus> {
    let regex = eval_preg_regex(pattern, values)?;
    let replacement = values.string_bytes(replacement)?;
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
        eval_preg_expand_replacement(&replacement, &subject, &captures, &mut result);
        cursor = matched.end();
        replacement_count += 1;
    }
    result.extend_from_slice(&subject[cursor..]);
    Ok((values.string_bytes_value(&result)?, replacement_count))
}


/// Dispatches by-value `preg_replace()` calls after argument binding.
pub(in crate::interpreter) fn eval_preg_replace_values_result(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [pattern, replacement, subject] => {
            eval_preg_replace_result(*pattern, *replacement, *subject, values)
        }
        [pattern, replacement, subject, limit] => eval_preg_replace_result_with_count(
            *pattern,
            *replacement,
            *subject,
            Some(*limit),
            values,
        )
        .map(|(result, _)| result),
        [pattern, replacement, subject, limit, _count] => {
            values.warning(
                "preg_replace(): Argument #5 ($count) must be passed by reference, value given",
            )?;
            eval_preg_replace_result_with_count(
                *pattern,
                *replacement,
                *subject,
                Some(*limit),
                values,
            )
            .map(|(result, _)| result)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}
