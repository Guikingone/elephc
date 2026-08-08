//! Purpose:
//! Declarative eval registry entry and implementation for `quotemeta`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Escapes php-src's `quotemeta` character set verbatim, byte-for-byte, so eval and
//!   compiled output stay identical for binary input.

eval_builtin! {
    name: "quotemeta",
    area: String,
    params: [string],
    direct: QuoteMeta,
    values: QuoteMeta,
}

use super::super::super::*;

/// Bytes PHP's `quotemeta` prefixes with a backslash.
const QUOTEMETA_ESCAPED: &[u8] = b".\\+*?[^]$()";

/// Evaluates PHP's `quotemeta(...)` over one eval expression.
pub(in crate::interpreter) fn eval_builtin_quotemeta(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [value] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let value = eval_expr(value, context, scope, values)?;
    eval_quotemeta_result(value, values)
}

/// Converts one eval value through PHP string conversion and escapes its metacharacters.
pub(in crate::interpreter) fn eval_quotemeta_result(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    values.string_bytes_value(&eval_quotemeta_bytes(&bytes))
}

/// Prefixes every regular-expression metacharacter in `bytes` with a single backslash.
pub(in crate::interpreter) fn eval_quotemeta_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    for byte in bytes {
        if QUOTEMETA_ESCAPED.contains(byte) {
            output.push(b'\\');
        }
        output.push(*byte);
    }
    output
}
