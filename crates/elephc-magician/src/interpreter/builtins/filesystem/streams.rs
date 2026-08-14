//! Purpose:
//! Shared stream argument coercions for eval filesystem stream builtins.
//!
//! Called from:
//! - Leaf filesystem stream builtin files that need stream-resource coercions.
//!
//! Key details:
//! - Runtime resource payloads are zero-based keys into `EvalStreamResources`.
//!   They are NOT recoverable from the PHP-visible resource id: that id comes
//!   from the runtime's own mint-on-demand registry and is deliberately
//!   independent of the payload. Always go through `eval_resource_payload()`.

use super::super::super::*;
/// Converts a runtime resource cell into eval's zero-based stream id.
pub(in crate::interpreter) fn eval_stream_resource_id(
    stream: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<i64, EvalStatus> {
    eval_resource_payload(stream, values)
}

/// Converts a stream length argument into a non-negative `usize`.
pub(in crate::interpreter) fn eval_nonnegative_usize(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<usize, EvalStatus> {
    let value = eval_int_value(value, values)?;
    usize::try_from(value).map_err(|_| EvalStatus::RuntimeFatal)
}

/// Converts an optional stream length where null and -1 mean "read all".
pub(in crate::interpreter) fn eval_optional_stream_length(
    value: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<usize>, EvalStatus> {
    let Some(value) = value else {
        return Ok(None);
    };
    if values.type_tag(value)? == EVAL_TAG_NULL {
        return Ok(None);
    }
    let value = eval_int_value(value, values)?;
    if value == -1 {
        return Ok(None);
    }
    Ok(Some(
        usize::try_from(value).map_err(|_| EvalStatus::RuntimeFatal)?,
    ))
}

/// Converts an optional absolute stream offset where null and -1 mean no seek.
pub(in crate::interpreter) fn eval_optional_stream_offset(
    value: Option<RuntimeCellHandle>,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<i64>, EvalStatus> {
    let Some(value) = value else {
        return Ok(None);
    };
    if values.type_tag(value)? == EVAL_TAG_NULL {
        return Ok(None);
    }
    let value = eval_int_value(value, values)?;
    if value < 0 {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Converts one runtime cell to a UTF-8 string for stream mode arguments.
pub(in crate::interpreter) fn eval_stream_string(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<String, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// One CSV control argument, as php-src names it in the `ValueError` an ill-sized value raises.
///
/// The position is that function's OWN `Argument #N`: `fgetcsv()` and `fputcsv()` count a
/// `$stream` and (for the reader) a `$length` first, so their separator is `#3` while
/// `str_getcsv()`'s is `#2`.
pub(in crate::interpreter) struct CsvControlArgument {
    /// The php function name the message opens with.
    pub function: &'static str,
    /// php-src's `Argument #N` position for this control.
    pub position: usize,
    /// The php parameter name, without its `$`.
    pub parameter: &'static str,
    /// Whether an EMPTY string is accepted — true only for `$escape`.
    pub empty_allowed: bool,
}

/// Converts an optional one-byte CSV control argument, applying php-src's own size rule.
///
/// php validates the CONTROL before it reads a record: a separator or enclosure has to be
/// exactly one character, an escape has to be empty or one character, and anything else is a
/// catchable `ValueError`. Taking the first byte and dropping the rest — what this used to do —
/// let `str_getcsv($s, "::")` parse on `:` in silence, and made an EMPTY `$escape` select the
/// `"\\"` default rather than the RFC 4180 doubling mode php reaches through it.
pub(in crate::interpreter) fn eval_csv_control_byte(
    value: Option<RuntimeCellHandle>,
    default: u8,
    control: CsvControlArgument,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<u8, EvalStatus> {
    let Some(value) = value else {
        return Ok(default);
    };
    if values.type_tag(value)? == EVAL_TAG_NULL {
        return Ok(default);
    }
    let bytes = values.string_bytes(value)?;
    match bytes.len() {
        1 => Ok(bytes[0]),
        // The parsers spell doubling mode as a ZERO escape byte, which is exactly what an empty
        // `$escape` asks for; a zero separator or enclosure never reaches them.
        0 if control.empty_allowed => Ok(0),
        _ => eval_csv_control_error(&control, context, values),
    }
}

/// Raises PHP's catchable `ValueError` for a CSV control that is not a single character.
fn eval_csv_control_error<T>(
    control: &CsvControlArgument,
    context: &mut ElephcEvalContext,
    values: &mut impl RuntimeValueOps,
) -> Result<T, EvalStatus> {
    let text = if control.empty_allowed {
        format!(
            "{}(): Argument #{} (${}) must be empty or a single character",
            control.function, control.position, control.parameter
        )
    } else {
        format!(
            "{}(): Argument #{} (${}) must be a single character",
            control.function, control.position, control.parameter
        )
    };
    let exception = values.new_object("ValueError")?;
    let message = values.string(&text)?;
    let code = values.int(0)?;
    values.construct_object(exception, vec![message, code])?;
    context.set_pending_throw(exception);
    Err(EvalStatus::UncaughtThrowable)
}
