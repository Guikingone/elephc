//! Purpose:
//! Declarative eval registry entry and implementation for `chunk_split`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Mirrors php-src exactly, including the back-compat branch that appends the separator
//!   after the trailing partial chunk and returns a lone separator for an empty subject.
//! - A `$length` below 1 is php-src's `ValueError`, reported here as a runtime fatal.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "chunk_split",
    area: String,
    params: [
        string,
        length = EvalBuiltinDefaultValue::Int(76),
        separator = EvalBuiltinDefaultValue::String("\r\n"),
    ],
    direct: ChunkSplit,
    values: ChunkSplit,
}

use super::super::super::*;

/// Evaluates PHP `chunk_split(...)` over one subject and its optional length/separator.
pub(in crate::interpreter) fn eval_builtin_chunk_split(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [subject] => {
            let subject = eval_expr(subject, context, scope, values)?;
            eval_chunk_split_result(subject, None, None, values)
        }
        [subject, length] => {
            let subject = eval_expr(subject, context, scope, values)?;
            let length = eval_expr(length, context, scope, values)?;
            eval_chunk_split_result(subject, Some(length), None, values)
        }
        [subject, length, separator] => {
            let subject = eval_expr(subject, context, scope, values)?;
            let length = eval_expr(length, context, scope, values)?;
            let separator = eval_expr(separator, context, scope, values)?;
            eval_chunk_split_result(subject, Some(length), Some(separator), values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Splits an already evaluated subject into fixed-size chunks joined by the separator.
pub(in crate::interpreter) fn eval_chunk_split_result(
    subject: RuntimeCellHandle,
    length: Option<RuntimeCellHandle>,
    separator: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(subject)?;
    let length = match length {
        Some(length) => eval_int_value(length, values)?,
        None => 76,
    };
    if length < 1 {
        return Err(EvalStatus::RuntimeFatal);
    }
    let separator = match separator {
        Some(separator) => values.string_bytes(separator)?,
        None => b"\r\n".to_vec(),
    };
    let output = eval_chunk_split_bytes(&bytes, length as usize, &separator);
    values.string_bytes_value(&output)
}

/// Applies the php-src chunking rule over already converted byte slices.
///
/// One pass always runs, so an empty subject yields exactly one separator — the observable
/// effect of php-src's `chunklen > srclen` back-compat branch.
pub(in crate::interpreter) fn eval_chunk_split_bytes(
    bytes: &[u8],
    length: usize,
    separator: &[u8],
) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + separator.len());
    let mut offset = 0usize;
    loop {
        let take = length.min(bytes.len() - offset);
        output.extend_from_slice(&bytes[offset..offset + take]);
        output.extend_from_slice(separator);
        offset += take;
        if offset >= bytes.len() {
            break;
        }
    }
    output
}
