//! Purpose:
//! Eval registry entry and implementation for `date_default_timezone_set`.
//!
//! Called from:
//! - `crate::interpreter::builtins::time` direct and by-value dispatch.
//!
//! Key details:
//! - Valid identifiers update the shared AOT request timezone when the runtime bridge is present.

use super::super::super::*;

eval_builtin! {
    contract: "date_default_timezone_set",
    area: Time,
    direct: Time,
    values: Time,
}

/// Evaluates PHP `date_default_timezone_set($timezoneId)`.
pub(in crate::interpreter) fn eval_builtin_date_default_timezone_set(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [timezone] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let timezone = eval_expr(timezone, context, scope, values)?;
    eval_date_default_timezone_set_result(timezone, context, values)
}

/// Validates and stores one eval-local default timezone identifier.
pub(in crate::interpreter) fn eval_date_default_timezone_set_result(
    timezone: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let timezone_handle = timezone;
    let timezone = values.string_bytes(timezone_handle)?;
    let timezone = String::from_utf8_lossy(&timezone).into_owned();
    if !elephc_tz::timezone_identifier_valid(&timezone) {
        values.warning(&format!(
            "date_default_timezone_set(): Timezone ID '{timezone}' is invalid"
        ))?;
        return values.bool_value(false);
    }
    if let Some(result) = values.runtime_builtin_call(
        elephc_builtin_contract::RuntimeBuiltinId::DateDefaultTimezoneSet,
        &[timezone_handle],
    )? {
        context.set_default_timezone(timezone);
        return Ok(result);
    }
    context.set_default_timezone(timezone);
    values.bool_value(true)
}
