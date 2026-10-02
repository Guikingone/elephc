//! Purpose:
//! Eval registry entry and implementation for `array_change_key_case`.
//!
//! Called from:
//! - `crate::interpreter::builtins::array` dispatch tables.
//!
//! Key details:
//! - String keys are ASCII-lowercased (`CASE_LOWER`, the default) or uppercased; any non-zero
//!   `$case` means upper, as in php. Integer keys pass through untouched, and on a collision the
//!   later entry wins while the first position is kept.
//! - The compiled program gets this function from `elephc::array_edge_prelude`, which interpreted
//!   code cannot reach (Symfony's `AcceptHeader` and `RedirectResponse` call it while a request is
//!   interpreted).

use super::super::super::*;

eval_builtin! {
    contract: "array_change_key_case",
    area: Array,
    direct: Array,
    values: Array,
}

/// Dispatches direct eval calls for `array_change_key_case()`.
pub(in crate::interpreter) fn eval_array_change_key_case_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [array] => {
            let array = eval_expr(array, context, scope, values)?;
            eval_array_change_key_case_result(array, None, values)
        }
        [array, case] => {
            let array = eval_expr(array, context, scope, values)?;
            let case = eval_expr(case, context, scope, values)?;
            eval_array_change_key_case_result(array, Some(case), values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Dispatches evaluated-argument eval calls for `array_change_key_case()`.
pub(in crate::interpreter) fn eval_array_change_key_case_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    _context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match evaluated_args {
        [array] => eval_array_change_key_case_result(*array, None, values),
        [array, case] => eval_array_change_key_case_result(*array, Some(*case), values),
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Rebuilds the array with every string key case-mapped.
fn eval_array_change_key_case_result(
    array: RuntimeCellHandle,
    case: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let upper = match case {
        Some(case) => eval_int_value(case, values)? != 0,
        None => false,
    };
    let len = values.array_len(array)?;
    let mut result = values.assoc_new(len)?;
    for position in 0..len {
        let key = values.array_iter_key(array, position)?;
        let value = values.array_get(array, key)?;
        let key = if values.type_tag(key)? == EVAL_TAG_STRING {
            let mut bytes = values.string_bytes(key)?;
            if upper {
                bytes.make_ascii_uppercase();
            } else {
                bytes.make_ascii_lowercase();
            }
            values.string_bytes_value(&bytes)?
        } else {
            key
        };
        result = values.array_set(result, key, value)?;
    }
    Ok(result)
}
