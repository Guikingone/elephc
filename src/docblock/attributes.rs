//! Purpose:
//! Locates declarations after attribute groups so their generic docblocks stay attached.
//! Uses the source lexer to distinguish structural brackets from string and comment contents.
//!
//! Called from:
//! - `crate::docblock::collect()` when an annotated declaration has attributes.
//!
//! Key details:
//! - Line numbers are zero-based here, matching the source-line collector.
//! - Tagged PHP and tagless LFC sources share the same attribute tokens.
//! - Consecutive groups are skipped together, including groups sharing a declaration's line.

use std::collections::HashMap;

use crate::lexer::{tokenize, tokenize_with_mode, Token};
use crate::source::SourceMode;

/// Maps each attribute group's starting line to the next declaration token's line.
pub(super) fn declaration_lines(source: &str) -> HashMap<usize, usize> {
    let mut lines = HashMap::new();
    let Ok(tokens) = tokenize(source).or_else(|_| tokenize_with_mode(source, SourceMode::Lfc))
    else {
        return lines;
    };
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].0 != Token::AttrOpen {
            index += 1;
            continue;
        }
        let mut starts = Vec::new();
        while index < tokens.len() && tokens[index].0 == Token::AttrOpen {
            starts.push(tokens[index].1.span.line as usize - 1);
            index += 1;
            let mut depth = 1;
            while index < tokens.len() && depth > 0 {
                match tokens[index].0 {
                    Token::AttrOpen | Token::LBracket => depth += 1,
                    Token::RBracket => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
            if depth > 0 {
                return lines;
            }
        }
        if let Some((token, metadata)) = tokens.get(index) {
            if *token != Token::Eof {
                let target = metadata.span.line as usize - 1;
                for start in starts {
                    lines.insert(start, target);
                }
            }
        }
    }
    lines
}
