//! Purpose:
//! Evaluates PHP user-level diagnostics raised by `trigger_error()` in runtime fragments.
//!
//! Called from:
//! - `crate::interpreter::eval_call()` for direct positional `trigger_error()` calls.
//!
//! Key details:
//! - The PHP error-suppression operator controls visible warning-like diagnostics but does not
//!   make a user fatal recoverable.

use super::super::super::*;

const E_USER_ERROR: i64 = 256;
const E_USER_WARNING: i64 = 512;
const E_USER_NOTICE: i64 = 1024;
const E_USER_DEPRECATED: i64 = 16384;

/// Evaluates PHP's `trigger_error(message, error_level = E_USER_NOTICE)` builtin.
pub(in crate::interpreter) fn eval_builtin_trigger_error(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (message, level) = match args {
        [message] => (eval_expr(message, context, scope, values)?, E_USER_NOTICE),
        [message, level] => {
            let message = eval_expr(message, context, scope, values)?;
            let level = eval_expr(level, context, scope, values)?;
            (message, eval_int_value(level, values)?)
        }
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    if !matches!(
        level,
        E_USER_ERROR | E_USER_WARNING | E_USER_NOTICE | E_USER_DEPRECATED
    ) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let message = String::from_utf8(values.string_bytes(message)?)
        .map_err(|_| EvalStatus::RuntimeFatal)?;
    let prefix = match level {
        E_USER_ERROR => "Fatal error",
        E_USER_WARNING => "Warning",
        E_USER_NOTICE => "Notice",
        E_USER_DEPRECATED => "Deprecated",
        _ => unreachable!("validated user error level"),
    };
    if !context.errors_suppressed() {
        values.warning(&format!("{prefix}: {message}\n"))?;
    }
    if level == E_USER_ERROR {
        return Err(EvalStatus::RuntimeFatal);
    }
    values.bool_value(true)
}
