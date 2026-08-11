//! Purpose:
//! Declarative eval registry entry for `str_getcsv`, and the CSV record parser behind it.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Applies the same rule as the compiled `__rt_str_getcsv` helper, derived from `php -n`
//!   8.5.6 over 22 inputs: strip ONE trailing line terminator; if nothing is left the answer
//!   is a single empty field; strip ONE more; then parse with a newline as ordinary data.
//! - A newline is only structural at the very end. `"a\nb"` is one field containing a
//!   newline, and `"\n\n"` yields one empty field while `"\n"` yields the single-element
//!   answer from the early exit.

eval_builtin! {
    name: "str_getcsv",
    area: String,
    params: [
        string,
        separator = EvalBuiltinDefaultValue::String(","),
        enclosure = EvalBuiltinDefaultValue::String("\""),
        escape = EvalBuiltinDefaultValue::String("\\")
    ],
    direct: StrGetcsv,
    values: StrGetcsv,
}

use super::super::spec::EvalBuiltinDefaultValue;
use super::super::super::*;

/// Evaluates PHP `str_getcsv()` over its subject and optional control characters.
pub(in crate::interpreter) fn eval_builtin_str_getcsv(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    if args.is_empty() || args.len() > 4 {
        return Err(EvalStatus::RuntimeFatal);
    }
    let subject = eval_expr(&args[0], context, scope, values)?;
    let mut controls = Vec::new();
    for arg in &args[1..] {
        controls.push(eval_expr(arg, context, scope, values)?);
    }
    eval_str_getcsv_result(subject, &controls, values)
}

/// Parses one CSV record out of an evaluated string.
pub(in crate::interpreter) fn eval_str_getcsv_result(
    subject: RuntimeCellHandle,
    controls: &[RuntimeCellHandle],
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let control_byte = |values: &mut dyn FnMut(usize) -> Result<Option<u8>, EvalStatus>,
                        index: usize|
     -> Result<Option<u8>, EvalStatus> { values(index) };
    let mut read_control = |index: usize| -> Result<Option<u8>, EvalStatus> {
        match controls.get(index) {
            None => Ok(None),
            Some(handle) => {
                let bytes = values.string_bytes(*handle)?;
                Ok(bytes.first().copied())
            }
        }
    };
    let separator = control_byte(&mut read_control, 0)?.unwrap_or(b',');
    let enclosure = control_byte(&mut read_control, 1)?.unwrap_or(b'"');
    let escape = control_byte(&mut read_control, 2)?.unwrap_or(b'\\');

    let subject = values.string_bytes(subject)?;
    let trimmed = strip_one_terminator(&subject);
    let mut result = values.array_new(0)?;
    if trimmed.is_empty() {
        // php-src puts `null` here; an eval string array holds strings, so this matches the
        // compiled backend's `""` rather than inventing a different answer for `eval()`.
        result = values.string_array_push(result, "")?;
        return Ok(result);
    }
    let record = strip_one_terminator(trimmed);
    for field in parse_csv_record(record, separator, enclosure, escape) {
        // CSV fields are arbitrary bytes; the eval array stores them as PHP strings.
        let field = String::from_utf8_lossy(&field).into_owned();
        result = values.string_array_push(result, &field)?;
    }
    Ok(result)
}

/// Removes one trailing `\r\n`, `\n` or `\r`, treating `\r\n` as a single terminator.
fn strip_one_terminator(bytes: &[u8]) -> &[u8] {
    match bytes.last() {
        Some(b'\n') => {
            let rest = &bytes[..bytes.len() - 1];
            match rest.last() {
                Some(b'\r') => &rest[..rest.len() - 1],
                _ => rest,
            }
        }
        Some(b'\r') => &bytes[..bytes.len() - 1],
        _ => bytes,
    }
}

/// Splits one CSV record, unescaping enclosures the way php-src does.
///
/// A newline is ordinary data here: the caller has already removed the structural ones.
fn parse_csv_record(bytes: &[u8], separator: u8, enclosure: u8, escape: u8) -> Vec<Vec<u8>> {
    #[derive(Clone, Copy)]
    enum State {
        Outside,
        Field,
        Quoted,
        AfterEscape,
        AfterCloseQuote,
    }
    let mut fields = Vec::new();
    let mut field = Vec::new();
    let mut state = State::Outside;
    for &byte in bytes {
        match state {
            State::Outside => {
                if byte == separator {
                    fields.push(std::mem::take(&mut field));
                } else if byte == enclosure {
                    state = State::Quoted;
                } else {
                    field.push(byte);
                    state = State::Field;
                }
            }
            State::Field => {
                if byte == separator {
                    fields.push(std::mem::take(&mut field));
                    state = State::Outside;
                } else {
                    field.push(byte);
                }
            }
            State::Quoted => {
                if escape != 0 && byte == escape {
                    state = State::AfterEscape;
                } else if byte == enclosure {
                    state = State::AfterCloseQuote;
                } else {
                    field.push(byte);
                }
            }
            State::AfterEscape => {
                if byte != enclosure {
                    field.push(escape);
                }
                field.push(byte);
                state = State::Quoted;
            }
            State::AfterCloseQuote => {
                if byte == enclosure {
                    field.push(enclosure);
                    state = State::Quoted;
                } else if byte == separator {
                    fields.push(std::mem::take(&mut field));
                    state = State::Outside;
                } else {
                    field.push(enclosure);
                    field.push(byte);
                    state = State::Field;
                }
            }
        }
    }
    fields.push(field);
    fields
}
