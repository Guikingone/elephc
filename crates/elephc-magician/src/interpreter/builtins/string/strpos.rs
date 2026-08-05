//! Purpose:
//! Declarative eval registry entry for `strpos`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the string-position hook.
//! - The shared hook also serves `strrpos`, and both accept PHP's optional `$offset`. An
//!   offset outside the haystack is reference PHP's catchable `ValueError`; eval has no
//!   throw machinery, so it reports `EvalStatus::RuntimeFatal` the way `str_repeat` does
//!   for a negative count.

use super::super::spec::EvalBuiltinDefaultValue;

eval_builtin! {
    name: "strpos",
    area: String,
    params: [haystack, needle, offset = EvalBuiltinDefaultValue::Int(0)],
    direct: StringPosition,
    values: StringPosition,
}

use super::super::super::*;

/// Evaluates PHP `strpos(...)` over haystack, needle, and optional offset expressions.
pub(in crate::interpreter) fn eval_builtin_strpos(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_builtin_string_position_named("strpos", args, context, scope, values)
}

/// Applies PHP `strpos(...)` to evaluated haystack, needle, and optional offset values.
pub(in crate::interpreter) fn eval_strpos_result(
    haystack: RuntimeCellHandle,
    needle: RuntimeCellHandle,
    offset: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::strpos::eval_string_position_named_result("strpos", haystack, needle, offset, values)
}

/// Evaluates one named PHP byte-string position builtin.
pub(in crate::interpreter) fn eval_builtin_string_position_named(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let (haystack, needle, offset) = match args {
        [haystack, needle] => (haystack, needle, None),
        [haystack, needle, offset] => (haystack, needle, Some(offset)),
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    let haystack = eval_expr(haystack, context, scope, values)?;
    let needle = eval_expr(needle, context, scope, values)?;
    let offset = match offset {
        Some(offset) => Some(eval_expr(offset, context, scope, values)?),
        None => None,
    };
    eval_string_position_named_result(name, haystack, needle, offset, values)
}

/// Returns the first or last byte offset of a converted needle, or PHP `false`.
///
/// `offset` follows reference PHP: `strpos()` starts matching there, while a negative
/// `strrpos()` offset instead bounds where a match may end. Either spelling rejects an
/// offset outside the haystack, which PHP reports as a `ValueError`.
pub(in crate::interpreter) fn eval_string_position_named_result(
    name: &str,
    haystack: RuntimeCellHandle,
    needle: RuntimeCellHandle,
    offset: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let haystack = values.string_bytes(haystack)?;
    let needle = values.string_bytes(needle)?;
    let offset = match offset {
        Some(offset) => eval_int_value(offset, values)?,
        None => 0,
    };
    let window = string_position_window(name, &haystack, needle.len(), offset)?;
    let searched = &haystack[window.clone()];
    let position = match name {
        "strpos" if needle.is_empty() => Some(0),
        "strpos" => searched
            .windows(needle.len())
            .position(|candidate| candidate == needle),
        "strrpos" if needle.is_empty() => Some(searched.len()),
        "strrpos" => searched
            .windows(needle.len())
            .rposition(|candidate| candidate == needle),
        _ => return Err(EvalStatus::UnsupportedConstruct),
    };
    match position {
        Some(position) => {
            let position = i64::try_from(position + window.start)
                .map_err(|_| EvalStatus::RuntimeFatal)?;
            values.int(position)
        }
        None => values.bool_value(false),
    }
}

/// Resolves a `strpos()`-family `$offset` into the haystack byte range PHP actually scans.
///
/// A `strpos()` offset (and a non-negative `strrpos()` one) simply moves the start of the
/// range; a negative `strrpos()` offset instead trims the end so no match may extend past
/// `strlen($haystack) + $offset + strlen($needle)`. An offset outside the haystack is
/// reported as `EvalStatus::RuntimeFatal`, eval's stand-in for PHP's `ValueError`.
fn string_position_window(
    name: &str,
    haystack: &[u8],
    needle_len: usize,
    offset: i64,
) -> Result<std::ops::Range<usize>, EvalStatus> {
    let length = i64::try_from(haystack.len()).map_err(|_| EvalStatus::RuntimeFatal)?;
    if offset > length || offset < -length {
        return Err(EvalStatus::RuntimeFatal);
    }
    if offset >= 0 {
        let start = usize::try_from(offset).map_err(|_| EvalStatus::RuntimeFatal)?;
        return Ok(start..haystack.len());
    }
    if name == "strpos" {
        let start = usize::try_from(length + offset).map_err(|_| EvalStatus::RuntimeFatal)?;
        return Ok(start..haystack.len());
    }
    let needle_len = i64::try_from(needle_len).map_err(|_| EvalStatus::RuntimeFatal)?;
    if -offset < needle_len {
        return Ok(0..haystack.len());
    }
    let end = usize::try_from(length + offset + needle_len)
        .map_err(|_| EvalStatus::RuntimeFatal)?;
    Ok(0..end)
}
