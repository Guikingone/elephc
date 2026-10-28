//! Purpose:
//! Parses a reference stored INTO an element or a property — `$a[] =& $x;`, `$a['k'] = &$x;`,
//! `$o->p =& $x;` — as the reference forms elephc already supports.
//!
//! Called from:
//! - `crate::parser::stmt::parse_stmt_dispatch()`, before the ordinary variable statement.
//!
//! Key details:
//! - php makes the target slot and `$x` one reference. The statement becomes
//!   `<target> = $x; $x =& <target>;` (for a push the second line names the pushed key,
//!   `intval(array_key_last(...))`): the slot receives `$x`'s value, then `$x` is re-bound to
//!   the slot, so later writes through either name are shared.
//! - An undefined `$x` still warns once on the copy, where php creates it silently; carrying
//!   the value through a reference-bound temporary instead stored the cell itself in a typed
//!   element inside functions.
//! - The source must be a plain variable and the target an element or property chain rooted at
//!   a variable or `$this`; static properties and expression-position forms keep their errors.
//! - A variable that was already a reference to a third name leaves that set (php would join
//!   the slot to it).

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{Stmt, StmtKind};
use crate::span::Span;

/// Returns the index of the `=` of a `<element or property> = & $var ;` statement at `start`.
pub(in crate::parser::stmt) fn reference_store_assign_index(
    tokens: &[SpannedToken],
    start: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut has_access = false;
    let mut index = start;
    while let Some((token, _)) = tokens.get(index) {
        match token {
            Token::LParen | Token::LBracket | Token::LBrace => {
                if depth == 0 && *token == Token::LBracket {
                    has_access = true;
                }
                depth += 1;
            }
            Token::RParen | Token::RBracket | Token::RBrace => depth = depth.checked_sub(1)?,
            Token::Arrow if depth == 0 => has_access = true,
            Token::DoubleColon if depth == 0 => return None,
            Token::Semicolon | Token::Eof if depth == 0 => return None,
            Token::Assign if depth == 0 => {
                let shape = (
                    tokens.get(index + 1).map(|(token, _)| token),
                    tokens.get(index + 2).map(|(token, _)| token),
                    tokens.get(index + 3).map(|(token, _)| token),
                );
                return match shape {
                    (Some(Token::Ampersand), Some(Token::Variable(_)), Some(Token::Semicolon))
                        if has_access && index > start + 1 =>
                    {
                        Some(index)
                    }
                    _ => None,
                };
            }
            _ => {}
        }
        index += 1;
    }
    None
}

/// Parses the reference store found by `reference_store_assign_index`.
pub(in crate::parser::stmt) fn parse_reference_store(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    assign: usize,
) -> Result<Stmt, CompileError> {
    let meta = tokens[*pos].1.clone();
    let token = |kind: Token| (kind, meta.clone());
    let target = &tokens[*pos..assign];
    let source = tokens[assign + 2].clone();
    let push = target.len() > 2
        && target[target.len() - 2].0 == Token::LBracket
        && target[target.len() - 1].0 == Token::RBracket;

    let mut store = target.to_vec();
    store.extend([token(Token::Assign), source.clone()]);
    let mut rebind = vec![source.clone(), token(Token::Assign), token(Token::Ampersand)];
    if push {
        let base = &target[..target.len() - 2];
        rebind.extend_from_slice(base);
        // A push always lands on an integer key: `intval` states it for the element reference.
        // The base was null, false or undefined before the push and is an array after it; the
        // `(array)` cast tells the checker so.
        rebind.extend([
            token(Token::LBracket),
            token(Token::Identifier("intval".to_string())),
            token(Token::LParen),
            token(Token::Identifier("array_key_last".to_string())),
            token(Token::LParen),
            token(Token::LParen),
            token(Token::Identifier("array".to_string())),
            token(Token::RParen),
        ]);
        rebind.extend_from_slice(base);
        rebind.extend([token(Token::RParen), token(Token::RParen), token(Token::RBracket)]);
    } else {
        rebind.extend_from_slice(target);
    }

    let mut body = Vec::new();
    // Every node lowering keys by span must be distinct: the target appears in both
    // statements, so each token gets its own column past any real one (same line).
    let mut column = 50_000;
    for mut stream in [store, rebind] {
        stream.extend([token(Token::Semicolon), token(Token::Eof)]);
        for (_, meta) in stream.iter_mut() {
            let span = meta.span;
            *meta = crate::lexer::TokenMetadata::new(Span::new_in_source(span.line, column, span.source_id()));
            column += 1;
        }
        let mut cursor = 0;
        body.push(crate::parser::stmt::parse_stmt(&stream, &mut cursor)?);
    }
    *pos = assign + 4;
    Ok(Stmt::new(StmtKind::Synthetic(body), span))
}
