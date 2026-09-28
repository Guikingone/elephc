//! Purpose:
//! Parses PHP delimited preg patterns and compiles them into PCRE2 regexes.
//!
//! Called from:
//! - `crate::interpreter::builtins::regex` entrypoint modules.
//!
//! Key details:
//! - A pattern php rejects -- empty, a bad delimiter, no closing delimiter, an unknown modifier,
//!   or one PCRE2 cannot compile -- produces php's warning and no regex; callers return php's
//!   false or null for it. The reasons match compiled code's (`__rt_preg_strip`) word for word,
//!   because both go through the PCRE2 shim.

use super::super::super::*;

/// Why php rejects a pattern before PCRE2 sees it; the kinds are the PCRE2 shim's
/// (`elephc_pcre2_v1_record_pattern_error`).
pub(in crate::interpreter) enum PregPatternRejection {
    Empty,
    Delimiter,
    NoEnd(u8),
    Modifier(u8),
}

impl PregPatternRejection {
    /// Records this reason in the shim, for the compile that follows to report.
    fn record(&self) {
        let (kind, byte) = match self {
            Self::Empty => (1, 0),
            Self::Delimiter => (2, 0),
            Self::NoEnd(byte) => (3, *byte),
            Self::Modifier(byte) => (4, *byte),
        };
        crate::regex_provider::record_pattern_error(kind, byte);
    }
}

/// Compiles one eval PCRE-style delimited pattern for the PHP `function`.
///
/// `None` means php rejected the pattern: its warning has been emitted and the caller returns
/// its false or null.
pub(in crate::interpreter) fn eval_preg_regex(
    pattern: RuntimeCellHandle,
    function: &str,
    values: &mut impl RuntimeValueOps,
) -> Result<Option<Regex>, EvalStatus> {
    let pattern = values.string_bytes(pattern)?;
    match eval_preg_pattern_parts(&pattern) {
        Ok((body, modifiers)) => Regex::compile_or_warn(&body, modifiers, function, values),
        Err(rejection) => {
            // The shim turns the recorded reason into this compile's failure, so the warning is
            // worded, and preg_last_error() set, exactly as for compiled code.
            rejection.record();
            Regex::compile_or_warn(b"", EvalPregModifiers::default(), function, values)
        }
    }
}

/// Splits a PHP delimited regex into body bytes and supported modifiers, as php parses it:
/// leading whitespace skipped, the closing delimiter found by a forward scan that skips escapes
/// and counts nesting for bracket delimiters.
pub(in crate::interpreter) fn eval_preg_pattern_parts(
    pattern: &[u8],
) -> Result<(Vec<u8>, EvalPregModifiers), PregPatternRejection> {
    let start = pattern
        .iter()
        .position(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .ok_or(PregPatternRejection::Empty)?;
    let pattern = &pattern[start..];
    let delimiter = pattern[0];
    if delimiter == 0 || delimiter == b'\\' || delimiter.is_ascii_alphanumeric() {
        return Err(PregPatternRejection::Delimiter);
    }
    let closing = eval_preg_closing_delimiter(delimiter);
    let close_index = eval_preg_find_closing_delimiter(pattern, delimiter, closing)
        .ok_or(PregPatternRejection::NoEnd(closing))?;
    let body = eval_preg_unescape_delimiter(&pattern[1..close_index], delimiter, closing);
    let modifiers = eval_preg_modifiers(&pattern[close_index + 1..])?;
    Ok((body, modifiers))
}

/// Returns the closing regex delimiter for PHP's paired delimiter forms.
pub(in crate::interpreter) fn eval_preg_closing_delimiter(delimiter: u8) -> u8 {
    match delimiter {
        b'(' => b')',
        b'[' => b']',
        b'{' => b'}',
        b'<' => b'>',
        _ => delimiter,
    }
}

/// Finds the closing regex delimiter: the first unescaped one, or for a bracket delimiter the
/// one that closes the outermost level.
pub(in crate::interpreter) fn eval_preg_find_closing_delimiter(
    pattern: &[u8],
    opening: u8,
    closing: u8,
) -> Option<usize> {
    let mut escaped = false;
    let mut depth = 1usize;
    for (index, byte) in pattern.iter().copied().enumerate().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if opening != closing && byte == opening {
            depth += 1;
        } else if byte == closing {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

/// Removes escapes that only protect the PHP regex delimiter from pattern stripping.
pub(in crate::interpreter) fn eval_preg_unescape_delimiter(
    body: &[u8],
    delimiter: u8,
    closing: u8,
) -> Vec<u8> {
    let mut result = Vec::with_capacity(body.len());
    let mut index = 0;
    while index < body.len() {
        if body[index] == b'\\'
            && index + 1 < body.len()
            && matches!(body[index + 1], byte if byte == delimiter || byte == closing)
        {
            result.push(body[index + 1]);
            index += 2;
        } else {
            result.push(body[index]);
            index += 1;
        }
    }
    result
}

/// Parses eval-supported PHP regex modifiers.
pub(in crate::interpreter) fn eval_preg_modifiers(
    modifiers: &[u8],
) -> Result<EvalPregModifiers, PregPatternRejection> {
    let mut parsed = EvalPregModifiers::default();
    for modifier in modifiers {
        match *modifier {
            b'i' => parsed.case_insensitive = true,
            b'm' => parsed.multi_line = true,
            b's' => parsed.dot_matches_new_line = true,
            b'U' => parsed.swap_greed = true,
            b'u' => parsed.unicode = true,
            b'A' => parsed.anchored = true,
            b'x' => parsed.extended = true,
            b'D' => parsed.dollar_endonly = true,
            b'J' => parsed.dupnames = true,
            b'n' => parsed.no_auto_capture = true,
            // `S` ("study", a no-op since PCRE2's JIT replaced PCRE1's
            // optional study pass) and `X` (PCRE2_EXTRA's stricter backslash
            // rejection, which is PCRE2's unconditional default already) are
            // both valid PHP modifiers that change nothing PCRE2 needs a
            // compile option for — accepting them without a flag reproduces
            // PHP's own behavior instead of refusing a modifier PHP allows.
            // `r` is php 8.4's caseless-restrict, accepted without a flag; php skips blanks and
            // line breaks among the modifiers.
            b'S' | b'X' | b'r' | b' ' | b'\n' | b'\r' => {}
            other => return Err(PregPatternRejection::Modifier(other)),
        }
    }
    Ok(parsed)
}
