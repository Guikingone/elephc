//! Purpose:
//! Locates PHP open and close tags without confusing bytes inside lexical content for syntax.
//! This small scanner serves source-block splitting and eval-fragment validation before EvalIR
//! tokenization starts.
//!
//! Called from:
//! - `crate::interpreter::include_exec` while splitting included PHP source blocks.
//! - `crate::parser::parse_fragment()` while rejecting actual PHP opening tags in eval input.
//!
//! Key details:
//! - Quoted strings, backticks, block comments, and heredocs hide tag-shaped byte sequences.
//! - A closing tag ends a line comment, matching PHP source-mode comment semantics.

/// Returns whether an eval fragment contains an actual PHP opening tag.
pub(crate) fn contains_php_open_tag(bytes: &[u8]) -> bool {
    find_php_tag(bytes, 0, b"<?", false).is_some()
}

/// Finds the next PHP closing tag at code level after `start`.
pub(crate) fn find_php_close_tag(bytes: &[u8], start: usize) -> Option<usize> {
    find_php_tag(bytes, start, b"?>", true)
}

/// Tracks lexical regions where PHP tag-shaped bytes are ordinary source content.
enum PhpTagScanState {
    Code,
    Quoted { quote: u8, escaped: bool },
    LineComment,
    BlockComment,
    Heredoc { label: Vec<u8> },
}

/// Finds `tag` only where PHP source treats it as syntax.
fn find_php_tag(
    bytes: &[u8],
    start: usize,
    tag: &[u8],
    tag_ends_line_comment: bool,
) -> Option<usize> {
    let mut index = start;
    let mut state = PhpTagScanState::Code;
    let mut line_start = true;

    while index < bytes.len() {
        match &mut state {
            PhpTagScanState::Code => {
                if bytes[index..].starts_with(tag) {
                    return Some(index);
                }
                match bytes[index] {
                    b'\'' | b'"' | b'`' => {
                        state = PhpTagScanState::Quoted {
                            quote: bytes[index],
                            escaped: false,
                        };
                        line_start = false;
                        index += 1;
                    }
                    b'/' if bytes.get(index + 1) == Some(&b'/') => {
                        state = PhpTagScanState::LineComment;
                        line_start = false;
                        index += 2;
                    }
                    b'#' => {
                        state = PhpTagScanState::LineComment;
                        line_start = false;
                        index += 1;
                    }
                    b'/' if bytes.get(index + 1) == Some(&b'*') => {
                        state = PhpTagScanState::BlockComment;
                        line_start = false;
                        index += 2;
                    }
                    b'<' if bytes[index..].starts_with(b"<<<") => {
                        if let Some((label, body_start)) = parse_heredoc_opening(bytes, index) {
                            state = PhpTagScanState::Heredoc { label };
                            line_start = true;
                            index = body_start;
                        } else {
                            line_start = false;
                            index += 1;
                        }
                    }
                    b'\n' => {
                        line_start = true;
                        index += 1;
                    }
                    _ => {
                        line_start = false;
                        index += 1;
                    }
                }
            }
            PhpTagScanState::Quoted { quote, escaped } => {
                let current = bytes[index];
                if *escaped {
                    *escaped = false;
                } else if current == b'\\' {
                    *escaped = true;
                } else if current == *quote {
                    state = PhpTagScanState::Code;
                }
                line_start = current == b'\n';
                index += 1;
            }
            PhpTagScanState::LineComment => {
                if tag_ends_line_comment && bytes[index..].starts_with(tag) {
                    return Some(index);
                }
                if bytes[index] == b'\n' {
                    state = PhpTagScanState::Code;
                    line_start = true;
                }
                index += 1;
            }
            PhpTagScanState::BlockComment => {
                if bytes[index..].starts_with(b"*/") {
                    state = PhpTagScanState::Code;
                    line_start = false;
                    index += 2;
                } else {
                    line_start = bytes[index] == b'\n';
                    index += 1;
                }
            }
            PhpTagScanState::Heredoc { label } => {
                if line_start {
                    if let Some(after_label) = heredoc_terminator_end(bytes, index, label) {
                        state = PhpTagScanState::Code;
                        line_start = false;
                        index = after_label;
                        continue;
                    }
                }
                line_start = bytes[index] == b'\n';
                index += 1;
            }
        }
    }
    None
}

/// Parses a valid heredoc/nowdoc opener and returns its terminator label and body offset.
fn parse_heredoc_opening(bytes: &[u8], start: usize) -> Option<(Vec<u8>, usize)> {
    let mut cursor = start.checked_add(3)?;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    let quote = match bytes.get(cursor) {
        Some(b'\'' | b'"') => {
            let quote = bytes[cursor];
            cursor += 1;
            Some(quote)
        }
        _ => None,
    };
    let label_start = cursor;
    if !bytes.get(cursor).is_some_and(|byte| is_heredoc_label_start(*byte)) {
        return None;
    }
    cursor += 1;
    while bytes
        .get(cursor)
        .is_some_and(|byte| is_heredoc_label_continue(*byte))
    {
        cursor += 1;
    }
    let label = bytes[label_start..cursor].to_vec();
    if quote.is_some_and(|quote| bytes.get(cursor) != Some(&quote)) {
        return None;
    }
    if quote.is_some() {
        cursor += 1;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    if bytes.get(cursor) == Some(&b'\r') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'\n')).then_some((label, cursor + 1))
}

/// Returns whether a byte starts a PHP heredoc label.
fn is_heredoc_label_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic() || byte >= 0x80
}

/// Returns whether a byte continues a PHP heredoc label.
fn is_heredoc_label_continue(byte: u8) -> bool {
    is_heredoc_label_start(byte) || byte.is_ascii_digit()
}

/// Returns the first byte after a valid heredoc terminator line at `start`.
fn heredoc_terminator_end(bytes: &[u8], start: usize, label: &[u8]) -> Option<usize> {
    let mut cursor = start;
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    if !bytes.get(cursor..)?.starts_with(label) {
        return None;
    }
    cursor += label.len();
    if bytes.get(cursor) == Some(&b';') {
        cursor += 1;
    }
    match bytes.get(cursor) {
        None | Some(b'\n' | b'\r') => Some(cursor),
        Some(b'?') if bytes.get(cursor + 1) == Some(&b'>') => Some(cursor),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{contains_php_open_tag, find_php_close_tag};

    /// Verifies regex literals cannot truncate an included PHP code block.
    #[test]
    fn close_tag_ignores_regex_string_content() {
        let source = br#"<?php $pattern = '/\\{(?:<.*?>)?\\}/'; echo 'ok'; ?>html"#;
        let expected = source
            .windows(2)
            .rposition(|window| window == b"?>")
            .expect("fixture must contain its actual close tag");

        assert_eq!(find_php_close_tag(source, 5), Some(expected));
    }

    /// Verifies strings, block comments, backticks, and heredocs hide close-tag bytes.
    #[test]
    fn close_tag_ignores_non_code_regions() {
        let source = br#"<?php
$single = '?>';
$double = "escaped \\\"?>";
$command = `?>`;
/* ?> */
$doc = <<<'DOC'
?>
DOC;
echo 'ok'; ?>html"#;
        let expected = source
            .windows(2)
            .rposition(|window| window == b"?>")
            .expect("fixture must contain its actual close tag");

        assert_eq!(find_php_close_tag(source, 5), Some(expected));
    }

    /// Verifies a closing tag ends a line comment, as it does in PHP source mode.
    #[test]
    fn close_tag_ends_line_comment() {
        assert_eq!(find_php_close_tag(b"<?php // ?>html", 5), Some(9));
    }

    /// Verifies open-tag-looking text in lexical content remains valid eval source.
    #[test]
    fn opening_tag_ignores_non_code_regions() {
        let source = br#"echo '<?php'; /* <?php */ $doc = <<<'DOC'
<?php
DOC;
// <?php
echo "done";"#;

        assert!(!contains_php_open_tag(source));
    }

    /// Verifies an actual opening tag still remains detectable in code position.
    #[test]
    fn opening_tag_detects_code_position() {
        assert!(contains_php_open_tag(b"echo 'before'; <?php echo 'after';"));
    }
}
