//! Purpose:
//! Eval registry entry and implementation for `date` plus shared date-format helpers.
//!
//! Called from:
//! - `crate::interpreter::builtins::time` direct and by-value dispatch.
//!
//! Key details:
//! - `gmdate` calls this file for shared formatting and UTC/local timestamp conversion.

use std::os::unix::ffi::OsStrExt;
use std::sync::Mutex;

use super::super::*;
use super::*;

eval_builtin! {
    contract: "date",
    area: Time,
    direct: Time,
    values: Time,
}

static EVAL_TZ_MUTEX: Mutex<()> = Mutex::new(());

unsafe extern "C" {
    /// Re-reads libc's process-global timezone environment.
    fn tzset();
}

/// Evaluates PHP `date($format, $timestamp = time())` for the eval subset.
pub(in crate::interpreter) fn eval_builtin_date(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_builtin_date_like("date", args, context, scope, values)
}

/// Evaluates PHP `date($format, $timestamp = time())` for the eval subset.
pub(in crate::interpreter) fn eval_builtin_date_like(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [format] => {
            let format = eval_expr(format, context, scope, values)?;
            eval_date_result(name, format, None, context, values)
        }
        [format, timestamp] => {
            let format = eval_expr(format, context, scope, values)?;
            let timestamp = eval_expr(timestamp, context, scope, values)?;
            eval_date_result(name, format, Some(timestamp), context, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Formats one Unix timestamp through PHP `date()` token rules supported by elephc.
pub(in crate::interpreter) fn eval_date_result(
    name: &str,
    format: RuntimeCellHandle,
    timestamp: Option<RuntimeCellHandle>,
    context: &ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let format = values.string_bytes(format)?;
    let timestamp = match timestamp {
        Some(timestamp) if !values.is_null(timestamp)? => eval_int_value(timestamp, values)?,
        None => eval_current_unix_timestamp()?,
        Some(_) => eval_current_unix_timestamp()?,
    };
    let (timezone, localtime) = match name {
        "date" => (context.default_timezone(), true),
        "gmdate" => ("UTC", false),
        _ => return Err(EvalStatus::UnsupportedConstruct),
    };
    let output = elephc_tz::format_timestamp_php(timestamp, timezone, &format, localtime)
        .ok_or(EvalStatus::RuntimeFatal)?;
    values.string_bytes_value(&output)
}

/// Converts one Unix timestamp to eval-timezone broken-down time through libc.
pub(in crate::interpreter) fn eval_context_localtime(
    timestamp: i64,
    context: &ElephcEvalContext,
) -> Result<libc::tm, EvalStatus> {
    eval_with_timezone(context.default_timezone(), || eval_localtime(timestamp))
}

/// Converts one Unix timestamp to process-local broken-down time through libc.
pub(in crate::interpreter) fn eval_localtime(timestamp: i64) -> Result<libc::tm, EvalStatus> {
    let raw = timestamp.try_into().map_err(|_| EvalStatus::RuntimeFatal)?;
    let mut tm = MaybeUninit::<libc::tm>::uninit();
    let result = unsafe { libc::localtime_r(&raw, tm.as_mut_ptr()) };
    if result.is_null() {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(unsafe { tm.assume_init() })
}

/// Runs one libc timezone-sensitive operation under the eval context timezone.
pub(in crate::interpreter) fn eval_with_timezone<T>(
    timezone: &str,
    operation: impl FnOnce() -> Result<T, EvalStatus>,
) -> Result<T, EvalStatus> {
    let _guard = EVAL_TZ_MUTEX
        .lock()
        .map_err(|_| EvalStatus::RuntimeFatal)?;
    let previous = std::env::var_os("TZ")
        .map(|value| CString::new(value.as_bytes()).map_err(|_| EvalStatus::RuntimeFatal))
        .transpose()?;
    eval_apply_process_timezone(timezone)?;
    let result = operation();
    eval_restore_process_timezone(previous.as_ref())?;
    result
}

/// Applies one timezone identifier to libc's process-global timezone state.
fn eval_apply_process_timezone(timezone: &str) -> Result<(), EvalStatus> {
    let key = CString::new("TZ").map_err(|_| EvalStatus::RuntimeFatal)?;
    let value = CString::new(timezone).map_err(|_| EvalStatus::RuntimeFatal)?;
    let status = unsafe { libc::setenv(key.as_ptr(), value.as_ptr(), 1) };
    if status != 0 {
        return Err(EvalStatus::RuntimeFatal);
    }
    unsafe { tzset() };
    Ok(())
}

/// Restores the process timezone that was active before an eval-local conversion.
fn eval_restore_process_timezone(previous: Option<&CString>) -> Result<(), EvalStatus> {
    let key = CString::new("TZ").map_err(|_| EvalStatus::RuntimeFatal)?;
    let status = if let Some(value) = previous {
        unsafe { libc::setenv(key.as_ptr(), value.as_ptr(), 1) }
    } else {
        unsafe { libc::unsetenv(key.as_ptr()) }
    };
    if status != 0 {
        return Err(EvalStatus::RuntimeFatal);
    }
    unsafe { tzset() };
    Ok(())
}

/// Returns a checked month index for PHP `date()` name tables.
pub(in crate::interpreter) fn eval_tm_month_index(tm: &libc::tm) -> Result<usize, EvalStatus> {
    let index = usize::try_from(tm.tm_mon).map_err(|_| EvalStatus::RuntimeFatal)?;
    if index >= EVAL_MONTH_NAMES.len() {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(index)
}

/// Returns a checked weekday index for PHP `date()` name tables.
pub(in crate::interpreter) fn eval_tm_weekday_index(tm: &libc::tm) -> Result<usize, EvalStatus> {
    let index = usize::try_from(tm.tm_wday).map_err(|_| EvalStatus::RuntimeFatal)?;
    if index >= EVAL_WEEKDAY_NAMES.len() {
        return Err(EvalStatus::RuntimeFatal);
    }
    Ok(index)
}
