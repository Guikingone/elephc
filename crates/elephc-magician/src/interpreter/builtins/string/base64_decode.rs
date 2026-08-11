//! Purpose:
//! Declarative eval registry entry for `base64_decode`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the existing Base64 decode hook.
//! - The decoder is a port of php-src's `php_base64_decode_impl`, so eval and AOT agree on the
//!   awkward cases: embedded whitespace is skipped without rotating the quartet lane, unpadded
//!   input still flushes its accumulated bytes, and a stray byte is dropped by the lax mode
//!   but makes `$strict = true` return `false`.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "base64_decode",
    area: String,
    params: [string, strict = EvalBuiltinDefaultValue::Bool(false)],
    direct: Base64Decode,
    values: Base64Decode,
}

use super::super::super::*;

/// Evaluates PHP's `base64_decode(...)` over one subject expression and an optional strict flag.
pub(in crate::interpreter) fn eval_builtin_base64_decode(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match args {
        [value] => {
            let value = eval_expr(value, context, scope, values)?;
            eval_base64_decode_result(value, false, values)
        }
        [value, strict] => {
            let value = eval_expr(value, context, scope, values)?;
            let strict = eval_expr(strict, context, scope, values)?;
            let strict = values.truthy(strict)?;
            eval_base64_decode_result(value, strict, values)
        }
        _ => Err(EvalStatus::RuntimeFatal),
    }
}

/// Converts one eval value through PHP string conversion and decodes Base64 bytes.
///
/// Returns `false` instead of a string when `strict` is set and the input holds a byte outside
/// the Base64 alphabet, data after a padding character, a truncated final group, or an invalid
/// amount of padding — exactly the four `goto fail` paths in php-src's decoder.
pub(in crate::interpreter) fn eval_base64_decode_result(
    value: RuntimeCellHandle,
    strict: bool,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let input = values.string_bytes(value)?;
    let Some(output) = eval_base64_decode_bytes_mode(&input, strict) else {
        return values.bool_value(false);
    };
    values.string_bytes_value(&output)
}

/// Decodes raw Base64 text with PHP's permissive non-strict behavior.
pub(in crate::interpreter) fn eval_base64_decode_bytes(input: &[u8]) -> Vec<u8> {
    eval_base64_decode_bytes_mode(input, false).unwrap_or_default()
}

/// Decodes raw Base64 bytes, returning `None` when strict PHP validation fails.
fn eval_base64_decode_bytes_mode(input: &[u8], strict: bool) -> Option<Vec<u8>> {
    let mut output: Vec<u8> = Vec::with_capacity((input.len() / 4) * 3);
    // `accepted` is php-src's `i`: it counts only characters that entered the accumulator, so
    // a skipped byte never rotates the quartet lane. `padding` is reset by an accepted
    // character in the lax mode and rejected outright in the strict one.
    let mut accepted: usize = 0;
    let mut padding: usize = 0;
    for byte in input.iter().copied() {
        if byte == b'=' {
            padding += 1;
            continue;
        }
        let sextet = match eval_base64_decode_sextet(byte) {
            Some(sextet) => sextet,
            None => {
                if byte.is_ascii() && matches!(byte, b'\t' | b'\n' | 0x0C | b'\r' | b' ') {
                    continue;
                }
                if strict {
                    return None;
                }
                continue;
            }
        };
        if padding > 0 {
            if strict {
                return None;
            }
            padding = 0;
        }
        match accepted % 4 {
            0 => output.push(sextet << 2),
            1 => {
                let last = output.len() - 1;
                output[last] |= sextet >> 4;
                output.push((sextet & 0x0f) << 4);
            }
            2 => {
                let last = output.len() - 1;
                output[last] |= sextet >> 2;
                output.push((sextet & 0x03) << 6);
            }
            _ => {
                let last = output.len() - 1;
                output[last] |= sextet;
            }
        }
        accepted += 1;
    }
    // php-src keeps the partially assembled trailing byte out of the result: `j` only advances
    // when a byte is completed, so a group of 2 or 3 characters contributes 1 or 2 bytes.
    let complete = accepted / 4 * 3
        + match accepted % 4 {
            0 => 0,
            1 => 0,
            2 => 1,
            _ => 2,
        };
    output.truncate(complete);
    if strict {
        if accepted % 4 == 1 {
            return None;
        }
        if padding > 0 && (padding > 2 || (accepted + padding) % 4 != 0) {
            return None;
        }
    }
    Some(output)
}

/// Returns the six-bit Base64 value for one encoded byte.
pub(in crate::interpreter) fn eval_base64_decode_sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
