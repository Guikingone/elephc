//! Purpose:
//! Implements PHP's `mb_strimwidth()` for the eval interpreter, as the fallback it reaches when
//! no function of that name was declared.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call`, after every declared and native function
//!   lookup, so a polyfill that defines `mb_strimwidth()` wins.
//!
//! Key details:
//! - Laravel's `Str::limit()` is `rtrim(mb_strimwidth($value, 0, $limit, '', 'UTF-8')).$end`, and
//!   symfony/polyfill-mbstring does not provide it, so every `limit()` died with "call to
//!   undefined function mb_strimwidth()" on a runtime without ext-mbstring.
//! - The width of a character is 2 for the East Asian Wide/Fullwidth blocks and common emoji,
//!   0 for combining marks of the Mn blocks listed below, and 1 otherwise. The marker counts
//!   toward the width, exactly as php's `mbfl_strimwidth` does.
//! - Not in the shared builtin catalog: there is no compiled counterpart, so a COMPILED call is
//!   still refused at compile time. That asymmetry is the known gap.

use super::super::super::*;

/// Returns whether `name` is the function this fallback answers.
pub(in crate::interpreter) fn eval_is_mb_strimwidth_fallback(name: &str) -> bool {
    name.trim_start_matches('\\').eq_ignore_ascii_case("mb_strimwidth")
}

/// The display width of one code point, as `mb_strwidth()` counts it.
fn code_point_width(code: u32) -> usize {
    const WIDE: &[(u32, u32)] = &[
        (0x1100, 0x115F),
        (0x231A, 0x231B),
        (0x2329, 0x232A),
        (0x23E9, 0x23EC),
        (0x25FD, 0x25FE),
        (0x2614, 0x2615),
        (0x2648, 0x2653),
        (0x26AA, 0x26AB),
        (0x26BD, 0x26BE),
        (0x26C4, 0x26C5),
        (0x2705, 0x2705),
        (0x270A, 0x270B),
        (0x2728, 0x2728),
        (0x274C, 0x274C),
        (0x2753, 0x2755),
        (0x2795, 0x2797),
        (0x2B1B, 0x2B1C),
        (0x2B50, 0x2B50),
        (0x2B55, 0x2B55),
        (0x2E80, 0x303E),
        (0x3041, 0x33FF),
        (0x3400, 0x4DBF),
        (0x4E00, 0x9FFF),
        (0xA000, 0xA4CF),
        (0xA960, 0xA97F),
        (0xAC00, 0xD7A3),
        (0xF900, 0xFAFF),
        (0xFE10, 0xFE19),
        (0xFE30, 0xFE6F),
        (0xFF01, 0xFF60),
        (0xFFE0, 0xFFE6),
        (0x1F004, 0x1F004),
        (0x1F0CF, 0x1F0CF),
        (0x1F18E, 0x1F18E),
        (0x1F191, 0x1F19A),
        (0x1F200, 0x1F320),
        (0x1F32D, 0x1F335),
        (0x1F337, 0x1F37C),
        (0x1F37E, 0x1F393),
        (0x1F3A0, 0x1F3CA),
        (0x1F3CF, 0x1F3D3),
        (0x1F3E0, 0x1F3F0),
        (0x1F3F4, 0x1F3F4),
        (0x1F3F8, 0x1F43E),
        (0x1F440, 0x1F440),
        (0x1F442, 0x1F4FC),
        (0x1F4FF, 0x1F53D),
        (0x1F54B, 0x1F54E),
        (0x1F550, 0x1F567),
        (0x1F57A, 0x1F57A),
        (0x1F595, 0x1F596),
        (0x1F5A4, 0x1F5A4),
        (0x1F5FB, 0x1F64F),
        (0x1F680, 0x1F6C5),
        (0x1F6CC, 0x1F6CC),
        (0x1F6D0, 0x1F6D2),
        (0x1F6D5, 0x1F6D7),
        (0x1F6EB, 0x1F6EC),
        (0x1F6F4, 0x1F6FC),
        (0x1F7E0, 0x1F7EB),
        (0x1F90C, 0x1F93A),
        (0x1F93C, 0x1F945),
        (0x1F947, 0x1F9FF),
        (0x1FA70, 0x1FAFF),
        (0x20000, 0x2FFFD),
        (0x30000, 0x3FFFD),
    ];
    const ZERO: &[(u32, u32)] = &[
        (0x0300, 0x036F),
        (0x0483, 0x0489),
        (0x0591, 0x05BD),
        (0x200B, 0x200F),
        (0x20D0, 0x20FF),
        (0xFE00, 0xFE0F),
        (0xFE20, 0xFE2F),
    ];
    let within = |table: &[(u32, u32)]| table.iter().any(|(low, high)| (*low..=*high).contains(&code));
    if within(WIDE) {
        2
    } else if within(ZERO) {
        0
    } else {
        1
    }
}

/// Splits UTF-8 text into `(byte_start, byte_end, width)` characters; an invalid byte is one
/// width-1 character, which keeps every byte of the input accounted for.
fn characters(bytes: &[u8]) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let lead = bytes[index];
        let length = match lead {
            0x00..=0x7F => 1,
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => 1,
        };
        let end = index + length;
        let decoded = (end <= bytes.len() && length > 1)
            .then(|| std::str::from_utf8(&bytes[index..end]).ok())
            .flatten()
            .and_then(|text| text.chars().next());
        match decoded {
            Some(character) => {
                out.push((index, end, code_point_width(character as u32)));
                index = end;
            }
            None => {
                out.push((index, index + 1, 1));
                index += 1;
            }
        }
    }
    out
}

/// Total display width of a byte string.
fn string_width(bytes: &[u8]) -> usize {
    characters(bytes).iter().map(|(_, _, width)| width).sum()
}

/// Throws a catchable `ValueError`, which is what php raises for an out-of-range argument.
fn throw_value_error(
    message: &str,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let exception = values.new_object("ValueError")?;
    let message = values.string(message)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}

/// Evaluates `mb_strimwidth($string, $start, $width, $trim_marker = "", $encoding = null)`.
pub(in crate::interpreter) fn eval_mb_strimwidth_fallback_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let args = positional_call_arg_exprs(args)?;
    if !(3..=5).contains(&args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let string = eval_expr(&args[0], context, scope, values)?;
    let start = eval_expr(&args[1], context, scope, values)?;
    let width = eval_expr(&args[2], context, scope, values)?;
    let marker = match args.get(3) {
        Some(marker) => Some(eval_expr(marker, context, scope, values)?),
        None => None,
    };
    if let Some(encoding) = args.get(4) {
        // The encoding is evaluated for its side effects; text is always treated as UTF-8.
        eval_expr(encoding, context, scope, values)?;
    }
    let bytes = values.string_bytes(string)?;
    let marker_bytes = match marker {
        Some(marker) => values.string_bytes(marker)?,
        None => Vec::new(),
    };
    let start = eval_int_value(start, values)?;
    let width = eval_int_value(width, values)?;

    let chars = characters(&bytes);
    let length = chars.len() as i64;
    let start = if start < 0 { start + length } else { start };
    if start < 0 || start > length {
        return throw_value_error(
            "mb_strimwidth(): Argument #2 ($start) is out of range",
            context,
            values,
        );
    }
    let rest = &chars[start as usize..];
    let rest_width: usize = rest.iter().map(|(_, _, width)| width).sum();
    let width = if width < 0 {
        let adjusted = rest_width as i64 + width;
        if adjusted < 0 {
            return throw_value_error(
                "mb_strimwidth(): Argument #3 ($width) is out of range",
                context,
                values,
            );
        }
        adjusted as usize
    } else {
        width as usize
    };
    let first_byte = rest.first().map_or(bytes.len(), |(begin, _, _)| *begin);
    if rest_width <= width {
        return values.string_bytes_value(&bytes[first_byte..]);
    }
    let room = width.saturating_sub(string_width(&marker_bytes));
    let mut used = 0;
    let mut end = first_byte;
    for (_, character_end, character_width) in rest {
        if used + character_width > room {
            break;
        }
        used += character_width;
        end = *character_end;
    }
    let mut output = bytes[first_byte..end].to_vec();
    output.extend_from_slice(&marker_bytes);
    values.string_bytes_value(&output)
}
