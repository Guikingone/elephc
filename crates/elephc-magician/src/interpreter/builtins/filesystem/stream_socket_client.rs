//! Purpose:
//! Declarative eval registry entry and implementation for `stream_socket_client`.
//!
//! Called from:
//! - `crate::interpreter::builtins::filesystem`.
//!
//! Key details:
//! - Opened sockets enter eval's normal stream table.
//! - Direct calls keep their source-sensitive by-reference error-output path, mirroring
//!   `fsockopen`; the out-params sit at args 1 and 2 rather than 2 and 3 because there is a
//!   single `address` argument in front of them instead of a host/port pair.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "stream_socket_client",
    area: Filesystem,
    params: [
        address,
        error_code: by_ref = EvalBuiltinDefaultValue::Null,
        error_message: by_ref = EvalBuiltinDefaultValue::Null,
        timeout = EvalBuiltinDefaultValue::Null,
        flags = EvalBuiltinDefaultValue::Null,
        context = EvalBuiltinDefaultValue::Null
    ],
    by_ref: [error_code, error_message],
    direct: none,
    values: Filesystem,
}

use super::super::super::*;
use super::*;

/// Evaluates a positional `stream_socket_client()` call without writable error outputs.
pub(in crate::interpreter) fn eval_stream_socket_client_declared_call(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !(1..=6).contains(&args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let address = eval_expr(&args[0], context, scope, values)?;
    for arg in &args[1..] {
        eval_expr(arg, context, scope, values)?;
    }
    eval_stream_socket_client_by_value_ref_warnings(args.len(), values)?;
    eval_stream_socket_client_result(address, context, values)
}

/// Evaluates a by-value `stream_socket_client()` call from already evaluated arguments.
pub(in crate::interpreter) fn eval_stream_socket_client_declared_values_result(
    evaluated_args: &[RuntimeCellHandle],
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if !(1..=6).contains(&evaluated_args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    eval_stream_socket_client_by_value_ref_warnings(evaluated_args.len(), values)?;
    eval_stream_socket_client_result(evaluated_args[0], context, values)
}

/// Evaluates a `stream_socket_client()` call that can write its by-reference error outputs.
pub(in crate::interpreter) fn eval_builtin_stream_socket_client_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let evaluated_args = eval_call_arg_values(args, context, scope, values)?;
    let (bound, _) = bind_evaluated_ref_builtin_args(
        &[
            "address",
            "error_code",
            "error_message",
            "timeout",
            "flags",
            "context",
        ],
        &evaluated_args,
        false,
    )?;
    let address = required_evaluated_ref_arg(&bound, 0)?;
    let error_code_target = optional_evaluated_ref_arg(&bound, 1)
        .map(|arg| arg.ref_target.clone().ok_or(EvalStatus::RuntimeFatal))
        .transpose()?;
    let error_message_target = optional_evaluated_ref_arg(&bound, 2)
        .map(|arg| arg.ref_target.clone().ok_or(EvalStatus::RuntimeFatal))
        .transpose()?;
    let (result, error_code, error_message) =
        eval_stream_socket_client_with_error_result(address.value, context, values)?;
    super::fsockopen::eval_write_socket_int_output_ref_target(
        error_code_target.as_ref(),
        error_code,
        context,
        values,
    )?;
    super::fsockopen::eval_write_socket_output_ref_target(
        error_message_target.as_ref(),
        Some(error_message),
        context,
        values,
    )?;
    Ok(result)
}

/// Opens an address-addressed TCP stream and returns PHP `stream_socket_client()` error outputs.
pub(in crate::interpreter) fn eval_stream_socket_client_with_error_result(
    address: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<(RuntimeCellHandle, i64, String), EvalStatus> {
    let address = eval_path_string(address, values)?;
    match context.stream_resources_mut().open_tcp_stream_result(&address) {
        Ok(id) => Ok((values.resource(id)?, 0, String::new())),
        Err(error) => {
            let error_code = i64::from(error.raw_os_error().unwrap_or(0));
            Ok((values.bool_value(false)?, error_code, error.to_string()))
        }
    }
}

/// Emits PHP by-reference warnings for by-value socket error outputs.
fn eval_stream_socket_client_by_value_ref_warnings(
    supplied_count: usize,
    values: &mut impl RuntimeValueOps,
) -> Result<(), EvalStatus> {
    if supplied_count >= 2 {
        values.warning(
            "stream_socket_client(): Argument #2 ($error_code) must be passed by reference, value given",
        )?;
    }
    if supplied_count >= 3 {
        values.warning(
            "stream_socket_client(): Argument #3 ($error_message) must be passed by reference, value given",
        )?;
    }
    Ok(())
}

/// Opens a connected TCP stream resource.
pub(in crate::interpreter) fn eval_stream_socket_client_result(
    address: RuntimeCellHandle,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let address = eval_path_string(address, values)?;
    match context.stream_resources_mut().open_tcp_stream(&address) {
        Some(id) => values.resource(id),
        None => values.bool_value(false),
    }
}
