//! Purpose:
//! Implements PHP's `version_compare()` for the eval interpreter, as the fallback it reaches when
//! no function of that name was declared.
//!
//! Called from:
//! - `crate::interpreter::expressions::calls::eval_call`, after every declared and native function
//!   lookup, so a userland definition (a polyfill) wins.
//!
//! Key details:
//! - A transcription of php-src's `php_version_compare()`: both strings are canonicalized (`-`,
//!   `_` and `+` become `.`, and a `.` is inserted wherever a digit run meets a non-digit run),
//!   then compared part by part. Numeric parts compare as integers; the special forms order as
//!   `dev < alpha = a < beta = b < RC = rc < # < pl = p`, and any unknown form sorts below `dev`.
//! - Not in the shared builtin catalog: there is no compiled counterpart, so a COMPILED call is
//!   still refused at compile time. Interpreted library code (Laravel's database layer) calls it
//!   to gate behaviour on a driver version.

use super::super::super::*;

/// Returns whether `name` is the function this fallback answers.
pub(in crate::interpreter) fn eval_is_version_compare_fallback(name: &str) -> bool {
    name.trim_start_matches('\\').eq_ignore_ascii_case("version_compare")
}

fn is_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}

/// A character that is neither a digit nor a `.`.
fn is_non_digit(byte: u8) -> bool {
    !is_digit(byte) && byte != b'.'
}

fn is_special_separator(byte: u8) -> bool {
    matches!(byte, b'-' | b'_' | b'+')
}

/// php-src's `php_canonicalize_version()`.
fn canonicalize(version: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(version.len() + 4);
    let Some((&first, rest)) = version.split_first() else {
        return out;
    };
    out.push(first);
    let mut last_source = first;
    for &byte in rest {
        let last_out = *out.last().unwrap_or(&b'.');
        if is_special_separator(byte) {
            if last_out != b'.' {
                out.push(b'.');
            }
        } else if (is_non_digit(last_source) && is_digit(byte))
            || (is_digit(last_source) && is_non_digit(byte))
        {
            if last_out != b'.' {
                out.push(b'.');
            }
            out.push(byte);
        } else if !byte.is_ascii_alphanumeric() {
            if last_out != b'.' {
                out.push(b'.');
            }
        } else {
            out.push(byte);
        }
        last_source = byte;
    }
    out
}

/// The rank of a special form, or -7 for one php does not know.
fn special_form_rank(form: &[u8]) -> i32 {
    const FORMS: &[(&[u8], i32)] = &[
        (b"dev", 0),
        (b"alpha", 1),
        (b"a", 1),
        (b"beta", 2),
        (b"b", 2),
        (b"RC", 3),
        (b"rc", 3),
        (b"#", 4),
        (b"pl", 5),
        (b"p", 5),
    ];
    FORMS
        .iter()
        .find(|(name, _)| form.starts_with(name))
        .map_or(-7, |(_, rank)| *rank)
}

fn compare_special_forms(left: &[u8], right: &[u8]) -> i32 {
    (special_form_rank(left) - special_form_rank(right)).signum()
}

/// Leading decimal digits of a part, as php's `strtol` reads them.
fn leading_number(part: &[u8]) -> i64 {
    let mut value: i64 = 0;
    for &byte in part.iter().take_while(|byte| is_digit(**byte)) {
        value = value.saturating_mul(10).saturating_add(i64::from(byte - b'0'));
    }
    value
}

fn first_is_digit(part: &[u8]) -> bool {
    part.first().is_some_and(|byte| is_digit(*byte))
}

/// php-src's `php_version_compare()` over two raw version strings: -1, 0 or 1.
pub(in crate::interpreter) fn compare_versions(left: &[u8], right: &[u8]) -> i32 {
    if left.is_empty() || right.is_empty() {
        if left.is_empty() && right.is_empty() {
            return 0;
        }
        return if left.is_empty() { -1 } else { 1 };
    }
    compare_canonical(&canonicalize(left), &canonicalize(right))
}

fn compare_canonical(left: &[u8], right: &[u8]) -> i32 {
    let mut left_parts = left.split(|byte| *byte == b'.');
    let mut right_parts = right.split(|byte| *byte == b'.');
    let mut compare = 0;
    let mut left_part = left_parts.next();
    let mut right_part = right_parts.next();
    while let (Some(l), Some(r)) = (left_part, right_part) {
        compare = if first_is_digit(l) && first_is_digit(r) {
            (leading_number(l) - leading_number(r)).signum() as i32
        } else if !first_is_digit(l) && !first_is_digit(r) {
            compare_special_forms(l, r)
        } else if first_is_digit(l) {
            compare_special_forms(b"#N#", r)
        } else {
            compare_special_forms(l, b"#N#")
        };
        if compare != 0 {
            return compare;
        }
        left_part = left_parts.next();
        right_part = right_parts.next();
    }
    match (left_part, right_part) {
        (Some(l), None) => {
            if first_is_digit(l) {
                1
            } else {
                // The rest of `left` against "#N#": compare the remainder as a version.
                let mut remainder = l.to_vec();
                for part in left_parts {
                    remainder.push(b'.');
                    remainder.extend_from_slice(part);
                }
                compare_versions(&remainder, b"#N#")
            }
        }
        (None, Some(r)) => {
            if first_is_digit(r) {
                -1
            } else {
                let mut remainder = r.to_vec();
                for part in right_parts {
                    remainder.push(b'.');
                    remainder.extend_from_slice(part);
                }
                compare_versions(b"#N#", &remainder)
            }
        }
        _ => compare,
    }
}

/// Evaluates `version_compare($version1, $version2, $operator = null)`.
pub(in crate::interpreter) fn eval_version_compare_fallback_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let args = positional_call_arg_exprs(args)?;
    if !(2..=3).contains(&args.len()) {
        return Err(EvalStatus::RuntimeFatal);
    }
    let left = eval_expr(&args[0], context, scope, values)?;
    let right = eval_expr(&args[1], context, scope, values)?;
    let operator = match args.get(2) {
        Some(operator) => Some(eval_expr(operator, context, scope, values)?),
        None => None,
    };
    let left_bytes = values.string_bytes(left)?;
    let right_bytes = values.string_bytes(right)?;
    let compare = compare_versions(&left_bytes, &right_bytes);
    let Some(operator) = operator else {
        return values.int(i64::from(compare));
    };
    if values.is_null(operator)? {
        return values.int(i64::from(compare));
    }
    let operator_bytes = values.string_bytes(operator)?;
    let result = match operator_bytes.as_slice() {
        b"<" | b"lt" => compare < 0,
        b"<=" | b"le" => compare <= 0,
        b">" | b"gt" => compare > 0,
        b">=" | b"ge" => compare >= 0,
        b"==" | b"eq" => compare == 0,
        b"!=" | b"<>" | b"ne" => compare != 0,
        _ => {
            let exception = values.new_object("ValueError")?;
            let message = values.string(
                "version_compare(): Argument #3 ($operator) must be a valid comparison operator",
            )?;
            let code = values.int(0)?;
            values.construct_object(exception, vec![message, code])?;
            context.set_pending_throw(exception);
            return Err(EvalStatus::UncaughtThrowable);
        }
    };
    values.bool_value(result)
}
