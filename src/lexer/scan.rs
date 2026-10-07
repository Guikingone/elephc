//! Purpose:
//! Owns the main PHP token scanning loop and dispatches literal-specific scanners.
//! Skips whitespace/comments and emits structural, operator, keyword, and literal tokens.
//!
//! Called from:
//! - `crate::lexer::tokenize()`.
//!
//! Key details:
//! - Multi-character operators and PHP opening tags must be recognized before shorter prefixes.

use super::cursor::Cursor;
use super::literals;
use super::token::{spanned, SpannedToken, Token, TokenMetadata};
use crate::errors::CompileError;
use crate::source::SourceMode;

/// Scans the full PHP source into a stream of syntax tokens with source metadata.
///
/// Requires `<?php` as the first five characters. Dispatches to `literals` for
/// strings (which may contain interpolation), heredoc/nowdoc, numbers, variables,
/// and keywords. Returns `Token::Eof` at end-of-input.
///
/// # Errors
/// Returns `CompileError` when PHP mode lacks its opening tag, LFC contains a
/// physical PHP tag at a code boundary, or either mode contains invalid syntax.
pub fn scan_tokens(
    source: &str,
    mode: SourceMode,
) -> Result<Vec<SpannedToken>, CompileError> {
    scan_tokens_in_source(source, mode, 0)
}

/// Tokenizes one physical source while assigning a disambiguating source identity to its spans.
pub(super) fn scan_tokens_in_source(
    source: &str,
    mode: SourceMode,
    source_id: u32,
) -> Result<Vec<SpannedToken>, CompileError> {
    // A leading UTF-8 byte-order mark (U+FEFF) is ignored, matching editors that save PHP
    // files as BOM-prefixed UTF-8; stripping it keeps the `<?php` open tag at the start.
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut cursor = Cursor::new_in_source(source, source_id);
    let mut tokens = Vec::new();

    let span = cursor.span();
    if mode.requires_open_tag() {
        // `OpenTag` is emitted first because the parser expects it at index 0. Any text before
        // the first `<?php`/`<?=` is INLINE HTML and is echoed BEFORE the program's statements,
        // which is the reference order (a leading BOM was already stripped above).
        tokens.push(spanned(Token::OpenTag, span));
        if !scan_leading_html(&mut cursor, &mut tokens) {
            // No PHP open tag anywhere: the whole file is inline HTML.
            tokens.push(spanned(Token::Eof, cursor.span()));
            return Ok(tokens);
        }
    } else {
        tokens.push(spanned(Token::OpenTag, span));
    }

    loop {
        skip_whitespace_and_comments(&mut cursor);

        if cursor.is_eof() {
            tokens.push(spanned(Token::Eof, cursor.span()));
            break;
        }

        let span = cursor.span();
        if matches!(mode, SourceMode::Lfc)
            && (cursor.remaining().starts_with("<?php")
                || cursor.remaining().starts_with("?>"))
        {
            return Err(CompileError::new(
                span,
                "PHP opening and closing tags are not valid in .lfc source files",
            ));
        } else if cursor.remaining().starts_with("?>") {
            // A close tag ends PHP mode: emit `;`, swallow one newline, and echo the inline HTML
            // up to the next open tag (see `scan_close_tag`).
            scan_close_tag(&mut cursor, &mut tokens);
        } else if cursor.peek() == Some('"') {
            // Double-quoted strings may contain interpolation ($var)
            let string_tokens = literals::scan_double_string_interpolated(&mut cursor)?;
            tokens.extend(string_tokens);
        } else if cursor.remaining().starts_with("<<<") {
            // Heredoc/nowdoc — may contain interpolation ($var) for heredoc
            cursor.advance(); // consume first <
            cursor.advance(); // consume second <
            cursor.advance(); // consume third <
            let heredoc_tokens = literals::scan_heredoc(&mut cursor)?;
            tokens.extend(heredoc_tokens);
        } else {
            let starts_word = cursor.peek().is_some_and(literals::is_ident_start);
            let remaining_before = cursor.remaining();
            let token = scan_token(&mut cursor)?;
            let end = cursor.span();
            let span = crate::span::Span::with_end_from(span, end);
            let metadata = if starts_word && !matches!(token, Token::Identifier(_)) {
                let consumed_len = remaining_before.len() - cursor.remaining().len();
                let source_spelling = &remaining_before[..consumed_len];
                if token.canonical_word_spelling() == Some(source_spelling) {
                    TokenMetadata::new(span)
                } else {
                    TokenMetadata::with_source_spelling(span, source_spelling)
                }
            } else {
                TokenMetadata::new(span)
            };
            tokens.push((token, metadata));
        }
    }

    Ok(tokens)
}

/// Skips all whitespace, `//` line comments, `#` line comments (but not `#[` attribute
/// groups), and `/* */` block comments. Uses `continue` to re-check after each comment
/// type so adjacent comment forms are all skipped.
fn skip_whitespace_and_comments(cursor: &mut Cursor) {
    loop {
        while let Some(ch) = cursor.peek() {
            if ch.is_ascii_whitespace() {
                cursor.advance();
            } else {
                break;
            }
        }

        if cursor.remaining().starts_with("//") {
            while let Some(ch) = cursor.advance() {
                if ch == '\n' { break; }
            }
            continue;
        }

        if cursor.remaining().starts_with('#') && !cursor.remaining().starts_with("#[") {
            // PHP line comment introduced by `#` (but `#[` opens an attribute group).
            while let Some(ch) = cursor.advance() {
                if ch == '\n' { break; }
            }
            continue;
        }

        if cursor.remaining().starts_with("/*") {
            cursor.advance();
            cursor.advance();
            loop {
                match cursor.advance() {
                    Some('*') if cursor.peek() == Some('/') => {
                        cursor.advance();
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            continue;
        }

        break;
    }
}

/// Dispatches token scanning based on the current character.
///
/// Multi-character operators (`?->`, `??`, `??=`, `:`, `::`, `=>`, `<=>`, `->>`,
/// `<<`, `>>`, `...`, compound assignments, etc.) are recognized before returning.
/// Delegates to `literals` for single-quoted strings, double-quoted strings (in the
/// outer loop), heredoc/nowdoc (in the outer loop), numbers, variables, and keywords.
/// Returns `Token::Eof` when `cursor.peek()` is `None`.
fn scan_token(cursor: &mut Cursor) -> Result<Token, CompileError> {
    let ch = match cursor.peek() {
        Some(c) => c,
        None => return Ok(Token::Eof),
    };

    match ch {
        ';' => { cursor.advance(); Ok(Token::Semicolon) }
        ',' => { cursor.advance(); Ok(Token::Comma) }
        '\\' => { cursor.advance(); Ok(Token::Backslash) }
        '?' => {
            if cursor.remaining().starts_with("?->") {
                cursor.advance();
                cursor.advance();
                cursor.advance();
                Ok(Token::QuestionArrow)
            } else if cursor.remaining().starts_with("??") {
                cursor.advance();
                cursor.advance();
                if cursor.peek() == Some('=') {
                    cursor.advance();
                    Ok(Token::QuestionQuestionAssign)
                } else {
                    Ok(Token::QuestionQuestion)
                }
            } else {
                cursor.advance();
                Ok(Token::Question)
            }
        }
        ':' => {
            cursor.advance();
            if cursor.peek() == Some(':') { cursor.advance(); Ok(Token::DoubleColon) }
            else { Ok(Token::Colon) }
        }
        '(' => { cursor.advance(); Ok(Token::LParen) }
        ')' => { cursor.advance(); Ok(Token::RParen) }
        '{' => { cursor.advance(); Ok(Token::LBrace) }
        '}' => { cursor.advance(); Ok(Token::RBrace) }
        '[' => { cursor.advance(); Ok(Token::LBracket) }
        ']' => { cursor.advance(); Ok(Token::RBracket) }
        '=' => {
            cursor.advance();
            if cursor.peek() == Some('=') {
                cursor.advance();
                if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::EqualEqualEqual) }
                else { Ok(Token::EqualEqual) }
            }
            else if cursor.peek() == Some('>') { cursor.advance(); Ok(Token::DoubleArrow) }
            else { Ok(Token::Assign) }
        }
        '!' => {
            cursor.advance();
            if cursor.peek() == Some('=') {
                cursor.advance();
                if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::NotEqualEqual) }
                else { Ok(Token::NotEqual) }
            }
            else { Ok(Token::Bang) }
        }
        '&' => {
            cursor.advance();
            if cursor.peek() == Some('&') { cursor.advance(); Ok(Token::AndAnd) }
            else if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::AmpAssign) }
            else { Ok(Token::Ampersand) }
        }
        '|' => {
            cursor.advance();
            if cursor.peek() == Some('|') { cursor.advance(); Ok(Token::OrOr) }
            else if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::PipeAssign) }
            else if cursor.peek() == Some('>') { cursor.advance(); Ok(Token::PipeArrow) }
            else { Ok(Token::Pipe) }
        }
        '^' => {
            cursor.advance();
            if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::CaretAssign) }
            else { Ok(Token::Caret) }
        }
        '~' => { cursor.advance(); Ok(Token::Tilde) }
        '<' => {
            cursor.advance();
            if cursor.peek() == Some('<') {
                cursor.advance();
                if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::LessLessAssign) }
                else { Ok(Token::LessLess) }
            }
            else if cursor.peek() == Some('=') {
                cursor.advance();
                if cursor.peek() == Some('>') { cursor.advance(); Ok(Token::Spaceship) }
                else { Ok(Token::LessEqual) }
            }
            else if cursor.peek() == Some('>') { cursor.advance(); Ok(Token::LessGreater) }
            else { Ok(Token::Less) }
        }
        '>' => {
            cursor.advance();
            if cursor.peek() == Some('>') {
                cursor.advance();
                if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::GreaterGreaterAssign) }
                else { Ok(Token::GreaterGreater) }
            }
            else if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::GreaterEqual) }
            else { Ok(Token::Greater) }
        }
        '+' => {
            cursor.advance();
            match cursor.peek() {
                Some('+') => { cursor.advance(); Ok(Token::PlusPlus) }
                Some('=') => { cursor.advance(); Ok(Token::PlusAssign) }
                _ => Ok(Token::Plus),
            }
        }
        '-' => {
            cursor.advance();
            match cursor.peek() {
                Some('>') => { cursor.advance(); Ok(Token::Arrow) }
                Some('-') => { cursor.advance(); Ok(Token::MinusMinus) }
                Some('=') => { cursor.advance(); Ok(Token::MinusAssign) }
                _ => Ok(Token::Minus),
            }
        }
        '*' => {
            cursor.advance();
            match cursor.peek() {
                Some('*') => {
                    cursor.advance();
                    if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::StarStarAssign) }
                    else { Ok(Token::StarStar) }
                }
                Some('=') => { cursor.advance(); Ok(Token::StarAssign) }
                _ => Ok(Token::Star),
            }
        }
        '/' => {
            cursor.advance();
            if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::SlashAssign) }
            else { Ok(Token::Slash) }
        }
        '%' => {
            cursor.advance();
            if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::PercentAssign) }
            else { Ok(Token::Percent) }
        }
        '.' => {
            // Check if next char is a digit → float literal like .5
            let remaining = cursor.remaining();
            if remaining.len() > 1 && remaining.as_bytes()[1].is_ascii_digit() {
                return literals::scan_dot_float(cursor);
            }
            // Check for ... (ellipsis / spread operator)
            if remaining.starts_with("...") {
                cursor.advance(); // consume first .
                cursor.advance(); // consume second .
                cursor.advance(); // consume third .
                return Ok(Token::Ellipsis);
            }
            cursor.advance();
            if cursor.peek() == Some('=') { cursor.advance(); Ok(Token::DotAssign) }
            else { Ok(Token::Dot) }
        }
        // '"' is handled in the main loop (interpolation support)
        '\'' => literals::scan_single_string(cursor),
        '@' => { cursor.advance(); Ok(Token::At) }
        '#' => {
            if cursor.remaining().starts_with("#[") {
                cursor.advance(); // consume '#'
                cursor.advance(); // consume '['
                Ok(Token::AttrOpen)
            } else {
                // Bare '#' that wasn't consumed by skip_whitespace_and_comments
                Err(CompileError::new(cursor.span(), "Unexpected '#'"))
            }
        }
        '$' => literals::scan_variable(cursor),
        '0'..='9' => literals::scan_number(cursor),
        'a'..='z' | 'A'..='Z' | '_' => literals::scan_keyword(cursor),
        // PHP allows non-ASCII identifier characters (bytes 0x80-0xFF), so a word that
        // starts with one is scanned as an identifier rather than rejected.
        c if literals::is_ident_start(c) => literals::scan_keyword(cursor),
        _ => Err(CompileError::new(
            cursor.span(),
            &format!("Unexpected character: '{}'", ch),
        )),
    }
}

/// Scans the inline HTML before the first PHP open tag, echoing it, and consumes the open tag.
///
/// Returns `false` when the source contains no `<?php`/`<?=` at all (a pure-HTML file).
fn scan_leading_html(cursor: &mut Cursor, tokens: &mut Vec<SpannedToken>) -> bool {
    scan_html_until_open_tag(cursor, tokens)
}

/// Consumes `?>`, the single newline it swallows, and the inline HTML up to the next open tag.
///
/// The grammar the parser already has is reused, exactly as the eval lexer does: the close tag
/// is a `;`, the HTML is `echo "…";`, and `<?=` is a leading `echo`. This keeps a block that
/// spans the tags working (`if (1) { ?>IN<?php }`).
fn scan_close_tag(cursor: &mut Cursor, tokens: &mut Vec<SpannedToken>) {
    let span = cursor.span();
    cursor.advance(); // '?'
    cursor.advance(); // '>'
    // One newline right after `?>` is swallowed (`\n`, `\r\n` or `\r`), as Zend's scanner does.
    match (cursor.peek(), peek_next(cursor)) {
        (Some('\r'), Some('\n')) => {
            cursor.advance();
            cursor.advance();
        }
        (Some('\n' | '\r'), _) => {
            cursor.advance();
        }
        _ => {}
    }
    tokens.push(spanned(Token::Semicolon, span));
    scan_html_until_open_tag(cursor, tokens);
}

/// Scans inline HTML up to the next `<?php`/`<?=` (consuming it), emitting `echo "<html>";` and,
/// for `<?=`, a leading `echo`. Returns whether an open tag was found.
///
/// `<?php` opens code only when followed by an ASCII separator or end of input, case-insensitively
/// (`<?phpX` and `<?php` + U+00A0 stay HTML); the short `<?` tag is HTML (`short_open_tag` is off).
fn scan_html_until_open_tag(cursor: &mut Cursor, tokens: &mut Vec<SpannedToken>) -> bool {
    let mut html = String::new();
    let mut short_echo = false;
    let mut opened = false;
    while let Some(ch) = cursor.peek() {
        if ch == '<' && peek_next(cursor) == Some('?') {
            if at_php_open_tag(cursor) {
                for _ in 0.."<?php".len() {
                    cursor.advance();
                }
                opened = true;
                break;
            }
            if cursor.remaining().starts_with("<?=") {
                for _ in 0.."<?=".len() {
                    cursor.advance();
                }
                short_echo = true;
                opened = true;
                break;
            }
        }
        crate::string_bytes::push_literal_char(ch, &mut html);
        cursor.advance();
    }
    let span = cursor.span();
    if !html.is_empty() {
        tokens.push(spanned(Token::Echo, span));
        tokens.push(spanned(Token::StringLiteral(html), span));
        tokens.push(spanned(Token::Semicolon, span));
    }
    if short_echo {
        tokens.push(spanned(Token::Echo, span));
    }
    opened
}

/// Returns the character after the current one without advancing the cursor.
fn peek_next(cursor: &Cursor) -> Option<char> {
    cursor.remaining().chars().nth(1)
}

/// Returns whether the cursor is at `<?php` followed by an ASCII separator or end of input,
/// case-insensitively.
fn at_php_open_tag(cursor: &Cursor) -> bool {
    let bytes = cursor.remaining().as_bytes();
    if bytes.len() < 5 || !bytes[..5].eq_ignore_ascii_case(b"<?php") {
        return false;
    }
    match bytes.get(5) {
        None => true,
        Some(&b) => is_open_tag_separator(b),
    }
}

/// Returns whether `b` is one of the characters Zend allows immediately after `<?php`.
fn is_open_tag_separator(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}
