//! Purpose:
//! Eval registry entry and implementation for `buffer_ptr`.
//!
//! Called from:
//! - `crate::interpreter::builtins::hooks`.
//!
//! Key details:
//! - Eval buffers are the AOT-shaped header (length word, stride word) followed by the
//!   payload, so the payload address is the header address plus the header words.

use std::mem;

use super::super::super::*;


eval_builtin! {
    contract: "buffer_ptr",
    area: RawMemory,
    direct: BufferPtr,
    values: BufferPtr,
}

/// Evaluates PHP `buffer_ptr()` over one eval expression.
pub(in crate::interpreter) fn eval_builtin_buffer_ptr(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [buffer] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let buffer = eval_expr(buffer, context, scope, values)?;
    eval_buffer_ptr_result(buffer, values)
}

/// Dispatches by-value `buffer_ptr()` calls after argument binding.
pub(in crate::interpreter) fn eval_buffer_ptr_values_result(
    evaluated_args: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [buffer] = evaluated_args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    eval_buffer_ptr_result(*buffer, values)
}

/// Answers the payload address that follows an AOT-shaped buffer header.
fn eval_buffer_ptr_result(
    buffer: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let header = super::ptr::eval_non_null_pointer(buffer, values)? as usize;
    let payload = header
        .checked_add(super::buffer_new::BUFFER_HEADER_WORDS * mem::size_of::<usize>())
        .ok_or(EvalStatus::RuntimeFatal)?;
    super::ptr::eval_address_value(payload, values)
}
