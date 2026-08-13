//! Purpose:
//! Declarative eval registry entry and implementation for `quoted_printable_encode`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Ports php-src's `php_quot_print_encode` byte-for-byte, including the pre-charged column
//!   counter and the UTF-8 lookahead allowance that keeps a multi-byte character off a soft
//!   line break, so eval and compiled output stay identical for binary input.

eval_builtin! {
    name: "quoted_printable_encode",
    area: String,
    params: [string],
    direct: QuotedPrintableEncode,
    values: QuotedPrintableEncode,
}

use super::super::super::*;

/// Maximum output column php-src allows before folding with a soft line break.
const QUOTED_PRINTABLE_MAX_LINE: usize = 75;

/// Uppercase hex digits php-src writes after the `=` escape introducer.
const QUOTED_PRINTABLE_HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Evaluates PHP's `quoted_printable_encode(...)` over one eval expression.
pub(in crate::interpreter) fn eval_builtin_quoted_printable_encode(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let [value] = args else {
        return Err(EvalStatus::RuntimeFatal);
    };
    let value = eval_expr(value, context, scope, values)?;
    eval_quoted_printable_encode_result(value, values)
}

/// Converts one eval value through PHP string conversion and quoted-printable encodes it.
pub(in crate::interpreter) fn eval_quoted_printable_encode_result(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    values.string_bytes_value(&eval_quoted_printable_encode_bytes(&bytes))
}

/// Encodes `bytes` with the MIME quoted-printable transfer encoding php-src implements.
///
/// An embedded `CRLF` is copied through and resets the column counter. A control byte, `0x7F`,
/// any high-bit byte, `=`, or a space directly before a `CR` becomes `=XX`; everything else is
/// literal. Lines are folded at column 75 with a trailing `=`, and a UTF-8 lead byte reserves
/// room for its continuation bytes so a character is never split across the fold.
pub(in crate::interpreter) fn eval_quoted_printable_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut column: usize = 0;
    let mut index: usize = 0;
    while index < bytes.len() {
        let current = bytes[index];
        index += 1;
        // php-src reads one byte past the current one and relies on the string's NUL
        // terminator, so the lookahead past the final byte is zero rather than absent.
        let lookahead = bytes.get(index).copied().unwrap_or(0);
        if current == b'\r' && lookahead == b'\n' && index < bytes.len() {
            output.push(b'\r');
            output.push(b'\n');
            index += 1;
            column = 0;
            continue;
        }
        let escaped = current < 0x20
            || current == 0x7F
            || current & 0x80 != 0
            || current == b'='
            || (current == b' ' && lookahead == b'\r');
        if !escaped {
            column += 1;
            if column > QUOTED_PRINTABLE_MAX_LINE {
                output.extend_from_slice(b"=\r\n");
                column = 1;
            }
            output.push(current);
            continue;
        }
        column += 3;
        let folds = match current {
            0x00..=0x7F => column > QUOTED_PRINTABLE_MAX_LINE,
            0x80..=0xDF => column + 3 > QUOTED_PRINTABLE_MAX_LINE,
            0xE0..=0xEF => column + 6 > QUOTED_PRINTABLE_MAX_LINE,
            0xF0..=0xF4 => column + 9 > QUOTED_PRINTABLE_MAX_LINE,
            // Above 0xF4 no UTF-8 lead byte exists, and php-src's condition chain simply
            // falls through without folding. Reproduced rather than "fixed".
            _ => false,
        };
        if folds {
            output.extend_from_slice(b"=\r\n");
            column = 3;
        }
        output.push(b'=');
        output.push(QUOTED_PRINTABLE_HEX[usize::from(current >> 4)]);
        output.push(QUOTED_PRINTABLE_HEX[usize::from(current & 0x0F)]);
    }
    output
}
