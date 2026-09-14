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
/// The line is the one the failing STATEMENT was written on. It used to be dropped whenever an
/// included file was executing, because a context's call site was the `include` that started the
/// file and naming the top of a file for a failure forty lines down is worse than naming nothing.
/// `EvalStmt::SourceLine` markers moved the call site to the statement being executed, so the
/// number is now the one php prints and there is nothing left to suppress.
pub(in crate::interpreter) fn note_eval_runtime_failure(
    what: impl Into<String>,
    context: &ElephcEvalContext,
) {
    let (file, _, line, _) = context.call_site();
    crate::errors::note_eval_runtime_failure(what, file, Some(line));
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

/// Creates and schedules a `FiberError` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_fiber_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("FiberError")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Rejects Fiber methods that would switch execution contexts inside a Magician handler.
pub(in crate::interpreter) fn eval_reject_fiber_switch_during_pcntl_dispatch(
    class_name: &str,
    method_name: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let is_fiber = class_name
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("Fiber");
    let switches_context = ["start", "resume", "throw", "suspend"]
        .iter()
        .any(|method| method_name.eq_ignore_ascii_case(method));
    if is_fiber
        && switches_context
        && crate::context::pcntl_runtime::fiber_dispatching()
    {
        return eval_throw_fiber_error(
            "Cannot switch fibers in current execution context",
            context,
            values,
        );
    }
    Ok(())
}

/// Rejects a switching method on an actual runtime or eval-declared Fiber receiver.
pub(in crate::interpreter) fn eval_reject_fiber_object_switch_during_pcntl_dispatch(
    object: RuntimeCellHandle,
    method_name: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    let Ok(identity) = values.object_identity(object) else {
        return Ok(());
    };
    let class_name = match context.dynamic_object_class(identity) {
        Some(class) => class.name().to_string(),
        None => runtime_object_class_name(object, values)?,
    };
    eval_reject_fiber_switch_during_pcntl_dispatch(
        &class_name,
        method_name,
        context,
        values,
    )
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

/// Creates and schedules an `ArgumentCountError` through eval's normal Throwable channel.
pub(in crate::interpreter) fn eval_throw_argument_count_error<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("ArgumentCountError")?;
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

/// Creates and schedules a `RuntimeException` through eval's normal Throwable channel.
///
/// Feature-gated: today's callers are all `crate::interpreter::builtins::curl` allocation-
/// and libcurl-failure paths (`curl_escape`/`curl_unescape`'s encode/decode failure,
/// `curl_init`'s easy-handle allocation failure, `curl_copy_handle`'s duplication
/// failure), which exist only under the `curl` feature — see that module's own doc for
/// why.
#[cfg(feature = "curl")]
pub(in crate::interpreter) fn eval_throw_runtime_exception<T>(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let exception = values.new_object("RuntimeException")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}
