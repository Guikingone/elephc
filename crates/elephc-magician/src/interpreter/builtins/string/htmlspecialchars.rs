//! Purpose:
//! Declarative eval registry entry for `htmlspecialchars`.
//!
//! Called from:
//! - `crate::interpreter::builtins::string`.
//!
//! Key details:
//! - Runtime dispatch is declared here and implemented through the HTML entity hook.

eval_builtin! {
    contract: "htmlspecialchars",
    area: String,
    direct: HtmlEntity,
    values: HtmlEntity,
}

use super::super::super::*;
/// Evaluates PHP `htmlspecialchars(...)` over one eval string expression.
pub(in crate::interpreter) fn eval_builtin_htmlspecialchars(
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    super::htmlspecialchars::eval_builtin_html_entity_named("htmlspecialchars", args, context, scope, values)
}

/// Evaluates a named HTML entity encode/decode builtin over one string expression.
/// All three accept the optional flags/encoding arguments; like the static
/// runtime they are evaluated but have no effect (ENT_QUOTES behaviour).
///
/// `html_entity_decode` used to be excluded here, which made the ordinary php call
/// `html_entity_decode($s, ENT_QUOTES, 'UTF-8')` an UNCATCHABLE
/// `eval() runtime failed: unsupported Call expression` inside an eval fragment while the same
/// source compiled fine once the AOT contract accepted three arguments. A builtin widened on one
/// side only is that failure shape, so the arities are kept identical on both.
pub(in crate::interpreter) fn eval_builtin_html_entity_named(
    name: &str,
    args: &[EvalExpr],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let value = match args {
        [value] | [value, _] | [value, _, _] => value,
        // `htmlspecialchars()` takes php's fourth parameter, `$double_encode`, exactly as the
        // AOT contract does: the two backends must accept the same arities.
        [value, _, _, _] if name == "htmlspecialchars" => value,
        _ => return Err(EvalStatus::RuntimeFatal),
    };
    let value = eval_expr(value, context, scope, values)?;
    let mut extras = Vec::with_capacity(args.len() - 1);
    for extra in &args[1..] {
        extras.push(eval_expr(extra, context, scope, values)?);
    }
    if let [flags, _, double_encode] = extras.as_slice() {
        if !values.truthy(*double_encode)? {
            let flags = eval_int_value(*flags, values)?;
            return eval_htmlspecialchars_keeping_entities(value, flags, values);
        }
    }
    let flags = match extras.first() {
        Some(flags) => eval_int_value(*flags, values)?,
        None => EVAL_ENT_DEFAULT_FLAGS,
    };
    eval_html_entity_named_result_with_flags(name, value, flags, values)
}

/// `htmlspecialchars($value, $flags, $encoding, false)`: encodes like
/// [`eval_htmlspecialchars_result`] but leaves every character reference the doctype considers
/// valid untouched, per `elephc_builtin_contract::html_entities::html_entity_is_kept` — the rule
/// the compiled program's prelude transcribes.
pub(in crate::interpreter) fn eval_htmlspecialchars_keeping_entities(
    value: RuntimeCellHandle,
    flags: i64,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'&' {
            if let Some(semi) = bytes[index + 1..].iter().position(|next| *next == b';') {
                let body = &bytes[index + 1..index + 1 + semi];
                if elephc_builtin_contract::html_entities::html_entity_is_kept(body, flags) {
                    output.extend_from_slice(&bytes[index..index + semi + 2]);
                    index += semi + 2;
                    continue;
                }
            }
            output.extend_from_slice(b"&amp;");
            index += 1;
            continue;
        }
        match byte {
            b'<' => output.extend_from_slice(b"&lt;"),
            b'>' => output.extend_from_slice(b"&gt;"),
            b'"' if flags & 2 != 0 => output.extend_from_slice(b"&quot;"),
            b'\'' => match eval_html_single_quote_entity(flags) {
                Some(entity) => output.extend_from_slice(entity),
                None => output.push(byte),
            },
            _ => output.push(byte),
        }
        index += 1;
    }
    values.string_bytes_value(&output)
}

/// php's default `$flags` for the HTML escapers: `ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401`.
pub(in crate::interpreter) const EVAL_ENT_DEFAULT_FLAGS: i64 = 3 | 8;

/// Applies the eval-supported HTML entity transform for one PHP string value.
pub(in crate::interpreter) fn eval_html_entity_named_result(
    name: &str,
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_html_entity_named_result_with_flags(name, value, EVAL_ENT_DEFAULT_FLAGS, values)
}

/// Applies the HTML entity transform with an explicit `$flags` bit set.
pub(in crate::interpreter) fn eval_html_entity_named_result_with_flags(
    name: &str,
    value: RuntimeCellHandle,
    flags: i64,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    match name {
        "htmlspecialchars" | "htmlentities" => eval_htmlspecialchars_with_flags(value, flags, values),
        "html_entity_decode" => eval_html_entity_decode_value_result(value, values),
        _ => Err(EvalStatus::UnsupportedConstruct),
    }
}

/// Encodes the HTML-special byte characters with php's default flags.
pub(in crate::interpreter) fn eval_htmlspecialchars_result(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    eval_htmlspecialchars_with_flags(value, EVAL_ENT_DEFAULT_FLAGS, values)
}

/// The entity spelled for a single quote, or `None` when `$flags` leaves it alone.
///
/// Bit 1 (`ENT_HTML_QUOTE_SINGLE`, set by `ENT_QUOTES`) asks for it; the doctype bits
/// (`ENT_XML1` 16, `ENT_XHTML` 32, `ENT_HTML5` 48) spell it `&apos;`, plain HTML 4.01 `&#039;`.
fn eval_html_single_quote_entity(flags: i64) -> Option<&'static [u8]> {
    if flags & 1 == 0 {
        None
    } else if flags & 48 != 0 {
        Some(b"&apos;")
    } else {
        Some(b"&#039;")
    }
}

/// Encodes the HTML-special byte characters, honouring the quote and doctype bits of `$flags`.
///
/// `ENT_COMPAT` (2) leaves `'` alone and `ENT_NOQUOTES` (0) leaves both quotes alone. Symfony's
/// error renderer escapes with `ENT_COMPAT | ENT_SUBSTITUTE`, so `I'm a teapot` must keep its
/// apostrophe.
pub(in crate::interpreter) fn eval_htmlspecialchars_with_flags(
    value: RuntimeCellHandle,
    flags: i64,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    let mut output = Vec::with_capacity(bytes.len());
    for byte in bytes {
        match byte {
            b'&' => output.extend_from_slice(b"&amp;"),
            b'<' => output.extend_from_slice(b"&lt;"),
            b'>' => output.extend_from_slice(b"&gt;"),
            b'"' if flags & 2 != 0 => output.extend_from_slice(b"&quot;"),
            b'\'' => match eval_html_single_quote_entity(flags) {
                Some(entity) => output.extend_from_slice(entity),
                None => output.push(byte),
            },
            _ => output.push(byte),
        }
    }
    values.string_bytes_value(&output)
}

/// Decodes one pass of the HTML entities emitted by the eval/static encoders.
pub(in crate::interpreter) fn eval_html_entity_decode_value_result(
    value: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let bytes = values.string_bytes(value)?;
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'&' {
            if let Some((decoded, width)) = eval_html_entity_at(&bytes[index..]) {
                output.push(decoded);
                index += width;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    values.string_bytes_value(&output)
}

/// Returns the decoded byte and consumed width for one supported HTML entity.
pub(in crate::interpreter) fn eval_html_entity_at(bytes: &[u8]) -> Option<(u8, usize)> {
    for (entity, decoded) in [
        (b"&lt;".as_slice(), b'<'),
        (b"&gt;".as_slice(), b'>'),
        (b"&quot;".as_slice(), b'"'),
        (b"&#039;".as_slice(), b'\''),
        (b"&#39;".as_slice(), b'\''),
        (b"&amp;".as_slice(), b'&'),
    ] {
        if bytes.starts_with(entity) {
            return Some((decoded, entity.len()));
        }
    }
    None
}
