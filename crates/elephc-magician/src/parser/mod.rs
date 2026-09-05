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
use crate::lexer::{contains_php_open_tag, find_php_close_tag, tokenize, Token, TokenKind};
use state::Parser;

/// Parses a whole PHP source file, inline HTML and every `<?php … ?>` block, into one program.
///
/// PHP compiles a file as a single token stream in which the text outside the tags is
/// `T_INLINE_HTML` and a closing tag is an implicit semicolon. Splitting the file into blocks and
/// parsing each one on its own cannot see a `{` that one block opens and a later block closes,
/// which is how every `<?php if (…): ?>…<?php endif ?>` template is written; concatenating the
/// tokens instead reproduces PHP's own view of the file, and the token lines stay file lines so a
/// diagnostic needs no offset.
pub fn parse_source_file(code: &[u8]) -> Result<EvalProgram, EvalParseDiagnostic> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut cursor = 0;
    while let Some(open) = find_php_open_tag(code, cursor) {
        push_inline_html_token(&mut tokens, code, cursor, open.tag_start);
        if open.echoes {
            tokens.push(Token::new(
                TokenKind::Ident("echo".to_string()),
                source_line(code, open.tag_start),
            ));
        }
        let close = find_php_close_tag(code, open.code_start);
        let code_end = close.unwrap_or(code.len());
        push_php_block_tokens(&mut tokens, code, open.code_start, code_end)?;
        let Some(close) = close else {
            return Parser::new(tokens, code.len()).parse_program();
        };
        // A closing tag terminates the statement it interrupts, exactly like a semicolon, and PHP
        // swallows one newline directly after it so a template does not emit a blank line per tag.
        tokens.push(Token::new(TokenKind::Semicolon, source_line(code, close)));
        cursor = skip_one_newline(code, close + 2);
    }
    push_inline_html_token(&mut tokens, code, cursor, code.len());
    Parser::new(tokens, code.len()).parse_program()
}

/// One PHP opening tag: where it starts, where its code starts, and whether it echoes.
struct PhpOpenTag {
    tag_start: usize,
    code_start: usize,
    echoes: bool,
}

/// Finds the next `<?php` or `<?=` opening tag at or after `start`.
fn find_php_open_tag(code: &[u8], start: usize) -> Option<PhpOpenTag> {
    let mut index = start;
    while index + 1 < code.len() {
        if code[index] == b'<' && code[index + 1] == b'?' {
            if code
                .get(index + 2..index + 5)
                .is_some_and(|word| word.eq_ignore_ascii_case(b"php"))
            {
                return Some(PhpOpenTag {
                    tag_start: index,
                    code_start: index + 5,
                    echoes: false,
                });
            }
            if code.get(index + 2) == Some(&b'=') {
                return Some(PhpOpenTag {
                    tag_start: index,
                    code_start: index + 3,
                    echoes: true,
                });
            }
        }
        index += 1;
    }
    None
}

/// Appends one inline-HTML token for a non-empty run of text outside the PHP tags.
fn push_inline_html_token(tokens: &mut Vec<Token>, code: &[u8], start: usize, end: usize) {
    let start = start.min(code.len());
    let end = end.min(code.len());
    if start >= end {
        return;
    }
    tokens.push(Token::new(
        TokenKind::InlineHtml(code[start..end].to_vec()),
        source_line(code, start),
    ));
}

/// Tokenizes one PHP block and appends its tokens with file-relative lines.
fn push_php_block_tokens(
    tokens: &mut Vec<Token>,
    code: &[u8],
    start: usize,
    end: usize,
) -> Result<(), EvalParseDiagnostic> {
    let block = &code[start.min(code.len())..end.min(code.len())];
    let source = normalize_binary_string_literals(block)
        .map_err(|error| EvalParseDiagnostic::at_source_end(error, code, "character"))?;
    let block_tokens = tokenize(&source)
        .map_err(|error| EvalParseDiagnostic::at_source_end(error, code, "end of file"))?;
    let offset = source_line(code, start) - 1;
    // Each block is tokenized on its own and therefore ends with the lexer's end-of-file token.
    // Concatenating those would put an end of file in the middle of the file, where every block
    // loop would stop; only the token stream as a whole has one end.
    tokens.extend(
        block_tokens
            .into_iter()
            .filter(|token| *token.kind() != TokenKind::Eof)
            .map(|token| {
                let line = token.line() + offset;
                Token::new(token.into_kind(), line)
            }),
    );
    Ok(())
}

/// Returns the one-based line the byte at `offset` sits on.
fn source_line(code: &[u8], offset: usize) -> i64 {
    1 + i64::try_from(
        code[..offset.min(code.len())]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count(),
    )
    .unwrap_or(0)
}

/// Skips the single newline PHP swallows right after a closing tag.
fn skip_one_newline(code: &[u8], start: usize) -> usize {
    match code.get(start) {
        Some(b'\n') => start + 1,
        Some(b'\r') if code.get(start + 1) == Some(&b'\n') => start + 2,
        _ => start,
    }
}

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
