//! Purpose:
//! Implements TOKEN_PARSE syntax validation and parser feedback for reserved identifiers.
//!
//! Called from:
//! - `crate::tokenize` only when TOKEN_PARSE is set.
//!
//! Key details:
//! - Parsing uses PHP's grammar rather than elephc's supported source subset.
//! - Feedback visits genuine identifier nodes, leaving magic constants and ::class intact.

use std::collections::HashSet;
use mago_allocator::LocalArena;
use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_syntax::{cst::Node, parser::parse_file_content_with_settings, settings::{LexerSettings, ParserSettings}};
use crate::{id, newline_count, ParseError, Token};

/// Validates PHP source and changes semi-reserved identifier tokens to T_STRING.
pub(super) fn validate(source: &[u8], tokens: &mut [Token]) -> Result<(), ParseError> {
    let input = super::lex::normalize_tags(source);
    let arena = LocalArena::new();
    let program = parse_file_content_with_settings(&arena, FileId::zero(), &input, ParserSettings {
        lexer: LexerSettings { enable_short_tags: false }, ..ParserSettings::default()
    });
    if let Some(error) = program.errors.first() {
        let at = (error.span().start.offset as usize).min(source.len());
        let message = if at == source.len() { "syntax error, unexpected end of file".to_owned() }
        else {
            let token = tokens.iter().find(|t| t.pos <= at && at < t.pos + t.len);
            match token {
                Some(t) if t.id == id("T_STRING") => format!("syntax error, unexpected identifier \"{}\"", String::from_utf8_lossy(&source[t.pos..t.pos + t.len])),
                Some(t) if t.len <= 3 => format!("syntax error, unexpected token \"{}\"", String::from_utf8_lossy(&source[t.pos..t.pos + t.len])),
                Some(t) => format!("syntax error, unexpected token \"{}\"", crate::token_name(t.id).unwrap_or("UNKNOWN")),
                None => "syntax error, unexpected end of file".to_owned(),
            }
        };
        return Err(ParseError { message, line: 1 + newline_count(&source[..at]) });
    }
    let mut names = HashSet::new();
    collect_names(Node::Program(program), &mut names);
    for token in tokens {
        if token.id >= 256 && names.contains(&token.pos) { token.id = id("T_STRING"); }
    }
    Ok(())
}

/// Collects local identifier positions, skipping AST nodes with dedicated PHP token kinds.
fn collect_names(node: Node<'_, '_>, names: &mut HashSet<usize>) {
    if matches!(node, Node::MagicConstant(_)) { return; }
    // `::class` remains T_CLASS, whereas a method named class receives feedback.
    if let Node::ClassLikeConstantSelector(mago_syntax::cst::ClassLikeConstantSelector::Identifier(identifier)) = node {
        if identifier.value.eq_ignore_ascii_case(b"class") { return; }
    }
    if let Node::LocalIdentifier(identifier) = node {
        names.insert(identifier.span.start.offset as usize);
        return;
    }
    node.visit_children(|child| collect_names(child, names));
}
