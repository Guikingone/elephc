//! Purpose:
//! Builds PHP Throwable objects for interpreter paths that need catchable runtime errors.
//!
//! Called from:
//! - `crate::interpreter::statements` and dynamic dispatch helpers.
//!
//! Key details:
//! - Helpers schedule the object in `ElephcEvalContext` and return `UncaughtThrowable`
//!   so surrounding try/catch execution can consume it.

use super::*;

/// Records why the interpreter is about to fail, with the PHP location that asked for it.
///
/// The bridge status the caller returns carries a number and nothing else, so the description
/// travels beside it and `__elephc_eval_report_runtime_fatal` prints the pair. Only failures
/// that are NOT Throwables need this: a Throwable already names itself.
///
/// The line is dropped while an included file is executing. A context's call site is the
/// `eval()` or `include` that started the fragment, so for an `eval()` it is exactly the line
/// PHP names, and for an include it is the top of the included file rather than the statement
/// that failed inside it.
pub(in crate::interpreter) fn note_eval_runtime_failure(
    what: impl Into<String>,
    context: &ElephcEvalContext,
) {
    let (file, _, line, _) = context.call_site();
    let line = (!context.executing_include()).then_some(line);
    crate::errors::note_eval_runtime_failure(what, file, line);
}

/// Creates and schedules an `Error` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("Error")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Creates and schedules a `TypeError` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_type_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("TypeError")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Creates and schedules a `ValueError` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_builtin_value_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("ValueError")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Creates and schedules a `DivisionByZeroError` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_builtin_division_by_zero_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("DivisionByZeroError")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}
