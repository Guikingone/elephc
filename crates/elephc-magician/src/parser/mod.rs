//! Purpose:
//! Parses runtime PHP eval fragments into EvalIR statement form.
//! The module entry point validates fragment boundaries, delegates tokenization,
//! and hands tokens to focused parser state.
//!
//! Called from:
//! - `crate::ffi::execute::__elephc_eval_execute()`
//! - `crate::interpreter` tests and nested eval execution paths.
//!
//! Key details:
//! - PHP eval fragments are statement fragments and must not include opening
//!   `<?` / `<?php` tags.
//! - File and directory metadata are supplied by the eval context at execution time.

mod cursor;
mod expressions;
mod state;
mod statements;

#[cfg(test)]
mod tests;

use crate::errors::{EvalParseDiagnostic, EvalParseError};
use crate::eval_ir::EvalProgram;
use crate::lexer::{contains_php_open_tag, tokenize};
use state::Parser;

/// Parses an eval fragment into by-name EvalIR statements.
///
/// Failures carry the fragment line and the failing token so callers can name them the way PHP
/// does; the three failures detected before the parser sees a token are reported at the
/// fragment's last line, which is where PHP reports an unterminated literal too.
pub fn parse_fragment(code: &[u8]) -> Result<EvalProgram, EvalParseDiagnostic> {
    if contains_php_open_tag(code) {
        return Err(EvalParseDiagnostic::at_source_end(
            EvalParseError::PhpOpenTag,
            code,
            "token \"<?php\"",
        ));
    }
    let source = normalize_binary_string_literals(code)
        .map_err(|error| EvalParseDiagnostic::at_source_end(error, code, "character"))?;
    let tokens = tokenize(&source)
        .map_err(|error| EvalParseDiagnostic::at_source_end(error, code, "end of file"))?;
    Parser::new(tokens, code.len()).parse_program()
}

/// Rewrites non-UTF-8 bytes inside quoted PHP literals to private markers before lexing.
///
/// PHP strings are byte sequences, while the lexer consumes UTF-8 Rust text. Markers preserve
/// the exact original byte so `Parser::parse_primary()` can emit `EvalConst::Bytes`; invalid bytes
/// outside a literal remain a source error.
fn normalize_binary_string_literals(code: &[u8]) -> Result<String, EvalParseError> {
    if let Ok(source) = std::str::from_utf8(code) {
        return Ok(source.to_owned());
    }
    let mut output = String::with_capacity(code.len());
    let mut quote = None;
    let mut escaped = false;
    for &byte in code {
        if byte >= 0x80 {
            let Some(_) = quote else {
                return Err(EvalParseError::InvalidUtf8);
            };
            output.push(char::from_u32(0xF0000 + byte as u32).expect("binary marker is valid"));
            escaped = false;
            continue;
        }
        let ch = byte as char;
        output.push(ch);
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
        } else if matches!(ch, '\'' | '"') {
            quote = Some(ch);
        }
    }
    Ok(output)
}
