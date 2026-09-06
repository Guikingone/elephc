//! Purpose:
//! Eval-interpreter implementation of `debug_backtrace()` and `debug_print_backtrace()`.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls` for a written call.
//! - `crate::interpreter::builtins::registry::dispatch` for a dynamic-callable call.
//! - `crate::interpreter::builtins::symbols::function_exists` for the existence probe.
//!
//! Key details:
//! - A stub returning `[]` would be worse than no function at all. Symfony's
//!   `ClassExistenceResource::throwOnRequiredClass` reads the second frame and returns silently
//!   when that frame names a class probe with no `class` key; given an empty trace it throws a
//!   `ReflectionException` instead, turning a successful `class_exists()` into a fatal.
//! - Measured against `php -n` 8.5.6: the keys are `file, line, function, class, object, type,
//!   args` in that order, innermost frame first. `class`, `object` and `type` are omitted for a
//!   plain function; `object` is omitted for a static call, whose `type` is `::`.
//! - `debug_backtrace()` never appears in its own result, and `debug_print_backtrace()` prints
//!   `#N FILE(LINE): Class->method()` per frame with no trailing `{main}` line.
//! - These two are dispatched as plain interpreter handlers rather than through the PHP-visible
//!   builtin registry, exactly like the OPcache handlers next door: a shared-contract entry would
//!   also claim an AOT implementation, and a compiled binary has no interpreter frames to report.

use super::super::super::*;

/// PHP's `DEBUG_BACKTRACE_PROVIDE_OBJECT`, which is also the default `$options`.
const EVAL_BACKTRACE_PROVIDE_OBJECT: i64 = 1;
/// PHP's `DEBUG_BACKTRACE_IGNORE_ARGS`, which drops the `args` key from every frame.
const EVAL_BACKTRACE_IGNORE_ARGS: i64 = 2;

/// Returns whether `name` (already lowercased and unqualified) is one of the two backtrace
/// functions, so `function_exists()` reports it even though it is not a catalog builtin.
pub(in crate::interpreter) fn eval_debug_backtrace_function_exists(name: &str) -> bool {
    matches!(name, "debug_backtrace" | "debug_print_backtrace")
}

/// Evaluates a written `debug_backtrace(...)` or `debug_print_backtrace(...)` call.
pub(in crate::interpreter) fn eval_debug_backtrace_call(
    name: &str,
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let args = positional_call_arg_exprs(args)?;
    if args.len() > 2 {
        return Err(EvalStatus::RuntimeFatal);
    }
    let mut evaluated_args = Vec::with_capacity(args.len());
    for arg in &args {
        evaluated_args.push(eval_expr(arg, context, scope, values)?);
    }
    eval_debug_backtrace_values_result(name, &evaluated_args, context, values)
}

/// Evaluates materialized backtrace arguments for either function.
pub(in crate::interpreter) fn eval_debug_backtrace_values_result(
    name: &str,
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "debug_backtrace" => eval_debug_backtrace_result(evaluated_args, context, values),
        "debug_print_backtrace" => {
            eval_debug_print_backtrace_result(evaluated_args, context, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Builds PHP's `debug_backtrace()` array from the interpreter's live frames.
fn eval_debug_backtrace_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (options, limit) = eval_backtrace_options(evaluated_args, values)?;
    let frames = eval_backtrace_frames(context, limit);
    let mut result = values.array_new(frames.len())?;
    for (index, frame) in frames.iter().enumerate() {
        let entry = eval_backtrace_frame_array(frame, options, values)?;
        let key = values.int(index as i64)?;
        result = values.array_set(result, key, entry)?;
    }
    Ok(result)
}

/// Prints PHP's `debug_print_backtrace()` listing and returns null.
fn eval_debug_print_backtrace_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (_, limit) = eval_backtrace_options(evaluated_args, values)?;
    let frames = eval_backtrace_frames(context, limit);
    let mut output = String::new();
    for (index, frame) in frames.iter().enumerate() {
        let call = match (&frame.class, frame.kind) {
            (Some(class), EvalCallFrameKind::Method) => format!("{class}->{}", frame.function),
            (Some(class), EvalCallFrameKind::StaticMethod) => format!("{class}::{}", frame.function),
            _ => frame.function.clone(),
        };
        output.push_str(&format!(
            "#{index} {}({}): {call}()\n",
            frame.file, frame.line
        ));
    }
    let output = values.string_bytes_value(output.as_bytes())?;
    values.echo(output)?;
    values.null()
}

/// One frame copied out of the context so the borrow ends before runtime values are built.
struct EvalBacktraceFrame {
    function: String,
    class: Option<String>,
    kind: EvalCallFrameKind,
    object: Option<RuntimeCellHandle>,
    args: Option<Vec<RuntimeCellHandle>>,
    file: String,
    line: i64,
}

/// Collects the live frames innermost-first, honouring `$limit`.
fn eval_backtrace_frames(context: &ElephcEvalContext, limit: usize) -> Vec<EvalBacktraceFrame> {
    let mut frames: Vec<EvalBacktraceFrame> = context.with_call_frames(|frames| {
        frames
            .iter()
            .rev()
            .map(|frame| EvalBacktraceFrame {
                function: frame.function.clone(),
                class: frame.class.clone(),
                kind: frame.kind,
                object: frame.object,
                args: frame.args.clone(),
                file: frame.file.clone(),
                line: frame.line,
            })
            .collect()
    });
    if limit > 0 && frames.len() > limit {
        frames.truncate(limit);
    }
    frames
}

/// Reads the `$options` and `$limit` arguments PHP accepts.
fn eval_backtrace_options(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<(i64, usize), EvalStatus> {
    if evaluated_args.len() > 2 {
        return Err(EvalStatus::RuntimeFatal);
    }
    let options = match evaluated_args.first() {
        Some(options) => eval_int_value(*options, values)?,
        None => EVAL_BACKTRACE_PROVIDE_OBJECT,
    };
    let limit = match evaluated_args.get(1) {
        Some(limit) => usize::try_from(eval_int_value(*limit, values)?).unwrap_or(0),
        None => 0,
    };
    Ok((options, limit))
}

/// Builds one frame's associative array with PHP's keys, in PHP's order.
fn eval_backtrace_frame_array(
    frame: &EvalBacktraceFrame,
    options: i64,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let mut entry = values.assoc_new(7)?;
    entry = eval_backtrace_set_str(entry, "file", &frame.file, values)?;
    entry = eval_backtrace_set_int(entry, "line", frame.line, values)?;
    entry = eval_backtrace_set_str(entry, "function", &frame.function, values)?;
    if let Some(class) = &frame.class {
        entry = eval_backtrace_set_str(entry, "class", class, values)?;
        // PHP carries `object` only for an instance call, and only while the caller keeps
        // `DEBUG_BACKTRACE_PROVIDE_OBJECT` set.
        if frame.kind == EvalCallFrameKind::Method && options & EVAL_BACKTRACE_PROVIDE_OBJECT != 0 {
            if let Some(object) = frame.object {
                let object = values.retain(object)?;
                entry = eval_backtrace_set_handle(entry, "object", object, values)?;
            }
        }
        let type_name = match frame.kind {
            EvalCallFrameKind::StaticMethod => "::",
            _ => "->",
        };
        entry = eval_backtrace_set_str(entry, "type", type_name, values)?;
    }
    if options & EVAL_BACKTRACE_IGNORE_ARGS == 0 {
        if let Some(frame_args) = &frame.args {
            let mut args = values.array_new(frame_args.len())?;
            for (index, arg) in frame_args.iter().enumerate() {
                let key = values.int(index as i64)?;
                let value = values.retain(*arg)?;
                args = values.array_set(args, key, value)?;
            }
            entry = eval_backtrace_set_handle(entry, "args", args, values)?;
        }
    }
    Ok(entry)
}

/// Stores one string-keyed string entry in a frame array.
fn eval_backtrace_set_str(
    entry: RuntimeCellHandle,
    key: &str,
    value: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let value = values.string(value)?;
    eval_backtrace_set_handle(entry, key, value, values)
}

/// Stores one string-keyed integer entry in a frame array.
fn eval_backtrace_set_int(
    entry: RuntimeCellHandle,
    key: &str,
    value: i64,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let value = values.int(value)?;
    eval_backtrace_set_handle(entry, key, value, values)
}

/// Stores one string-keyed runtime handle in a frame array.
fn eval_backtrace_set_handle(
    entry: RuntimeCellHandle,
    key: &str,
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let key = values.string(key)?;
    values.array_set(entry, key, value)
}
