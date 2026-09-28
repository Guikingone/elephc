//! Purpose:
//! Parses qualified and unqualified PHP names used by statement-level syntax.
//!
//! Called from:
//! - Declaration, namespace, FFI, and object-oriented statement parsers.
//!
//! Key details:
//! - `enum` remains a soft keyword in class-like name positions.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token, TokenMetadata};
use crate::names::{Name, NameKind};
use crate::span::Span;

/// Converts a token accepted as an ordinary PHP name segment to its source spelling.
pub(crate) fn name_part_from_token(
    token: &Token,
    metadata: &TokenMetadata,
) -> Option<String> {
    match token {
        Token::Identifier(name) => Some(name.clone()),
        Token::Enum => crate::parser::keyword_name::bareword_name_from_token(token, metadata),
        _ => None,
    }
}

/// Returns the segment spelled by the token at `pos` inside a qualified name.
///
/// PHP 8 lexes a qualified name as one token, so a reserved word is an ordinary segment
/// there (`Demo\Namespace`, `Vendor\Default\Theme`). A keyword counts only when the name
/// already has a separator before it (`after_separator`) or when a separator and a further
/// segment follow it with no space between (see [`keyword_starts_qualified_name`]); alone, it
/// is still the keyword.
pub(crate) fn qualified_segment_at(
    tokens: &[SpannedToken],
    pos: usize,
    after_separator: bool,
) -> Option<String> {
    let (token, metadata) = tokens.get(pos)?;
    if let Some(part) = name_part_from_token(token, metadata) {
        return Some(part);
    }
    if after_separator || keyword_starts_qualified_name(tokens, pos) {
        return crate::parser::keyword_name::bareword_name_from_token(token, metadata);
    }
    None
}

/// Returns whether a word at `pos` is glued to a following `\` and segment, as in
/// `Default\Palette`, which PHP 8 lexes as one qualified-name token.
///
/// The three tokens must touch in the source: `new \Foo` and `echo \strlen($s)` are a keyword
/// followed by a fully qualified name, while `new\Foo` is the qualified name `new\Foo`.
/// `namespace` never qualifies: a leading `namespace\` is PHP's relative-name prefix
/// (`namespace\Foo` is `Foo` in the current namespace), not a segment spelled `namespace`.
pub(crate) fn word_starts_qualified_name(tokens: &[SpannedToken], pos: usize) -> bool {
    let is_word = |index: usize| {
        tokens.get(index).is_some_and(|(token, metadata)| {
            crate::parser::keyword_name::bareword_name_from_token(token, metadata).is_some()
        })
    };
    !matches!(tokens.get(pos), Some((Token::Namespace, _)))
        && is_word(pos)
        && matches!(tokens.get(pos + 1), Some((Token::Backslash, _)))
        && tokens_touch(tokens, pos)
        && is_word(pos + 2)
        && tokens_touch(tokens, pos + 1)
}

/// Returns whether the token at `pos` is a reserved word that starts a qualified name, as
/// `Default` does in `Default\Palette::accent()`. Such a token would otherwise be dispatched
/// as its keyword, so statement, expression and type parsers check this before their keyword
/// arms. An ordinary identifier is not reported: it already starts a name everywhere.
pub(crate) fn keyword_starts_qualified_name(tokens: &[SpannedToken], pos: usize) -> bool {
    tokens
        .get(pos)
        .is_some_and(|(token, metadata)| name_part_from_token(token, metadata).is_none())
        && word_starts_qualified_name(tokens, pos)
}

/// Returns whether the token at `left` ends exactly where the token after it starts, with no
/// whitespace or comment between them. Tokens without a source extent never touch.
fn tokens_touch(tokens: &[SpannedToken], left: usize) -> bool {
    let (Some((_, first)), Some((_, second))) = (tokens.get(left), tokens.get(left + 1)) else {
        return false;
    };
    let (first, second) = (first.span, second.span);
    first.has_extent()
        && first.source_id() == second.source_id()
        && first.end_line == second.line
        && first.end_column() == second.col
}

/// Returns whether the token at `pos` starts a PHP class-like name.
pub(crate) fn name_starts_at(tokens: &[SpannedToken], pos: usize) -> bool {
    match tokens.get(pos) {
        Some((Token::Backslash, _)) => true,
        Some((token, metadata)) => {
            name_part_from_token(token, metadata).is_some()
                || keyword_starts_qualified_name(tokens, pos)
        }
        None => false,
    }
}

/// Parses one unqualified class-like declaration name.
pub(crate) fn parse_unqualified_name(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    error: &str,
) -> Result<String, CompileError> {
    let Some((token, metadata)) = tokens.get(*pos) else {
        return Err(CompileError::new(span, error));
    };
    let name = name_part_from_token(token, metadata)
        .ok_or_else(|| CompileError::new(span, error))?;
    *pos += 1;
    Ok(name)
}

/// Parses a PHP qualified or unqualified name from the token stream.
pub(crate) fn parse_name(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    first_error: &str,
) -> Result<Name, CompileError> {
    let mut kind = NameKind::Unqualified;
    if *pos < tokens.len() && tokens[*pos].0 == Token::Backslash {
        kind = NameKind::FullyQualified;
        *pos += 1;
    }

    let mut parts = Vec::new();
    loop {
        let after_separator = kind == NameKind::FullyQualified || !parts.is_empty();
        match qualified_segment_at(tokens, *pos, after_separator) {
            Some(part) => {
                parts.push(part);
                *pos += 1;
            }
            _ if parts.is_empty() => return Err(CompileError::new(span, first_error)),
            _ => {
                return Err(CompileError::new(
                    span,
                    "Expected identifier after '\\' in qualified name",
                ))
            }
        }

        if *pos < tokens.len() && tokens[*pos].0 == Token::Backslash {
            if kind != NameKind::FullyQualified {
                kind = NameKind::Qualified;
            }
            *pos += 1;
            continue;
        }
        break;
    }

    Ok(Name::from_parts(kind, parts))
}
