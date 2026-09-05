//! Purpose:
//! Defines stable integer status codes returned by the eval bridge ABI.
//! Keeps Rust error shapes internal while exposing C-compatible outcomes.
//!
//! Called from:
//! - `crate::__elephc_eval_execute()`
//!
//! Key details:
//! - Numeric values are part of the ABI contract and must remain stable.

/// Stable eval bridge status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalStatus {
    Ok,
    ParseError,
    RuntimeFatal,
    UncaughtThrowable,
    UnsupportedConstruct,
    AbiMismatch,
}

impl EvalStatus {
    /// Returns the C ABI integer code for this status.
    pub const fn code(self) -> i32 {
        match self {
            Self::Ok => 0,
            Self::ParseError => 1,
            Self::RuntimeFatal => 2,
            Self::UncaughtThrowable => 3,
            Self::UnsupportedConstruct => 4,
            Self::AbiMismatch => 5,
        }
    }
}

/// Parse failures detected before lowering a runtime eval fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalParseError {
    PhpOpenTag,
    InvalidUtf8,
    UnsupportedConstruct,
    UnexpectedToken,
    UnexpectedEof,
    InvalidNumber,
    UnterminatedString,
    UnterminatedComment,
    ExpectedVariable,
    ExpectedSemicolon,
}

impl EvalParseError {
    /// Returns the ABI status that should be reported for this parse failure.
    pub const fn status(self) -> EvalStatus {
        match self {
            Self::UnsupportedConstruct => EvalStatus::UnsupportedConstruct,
            Self::PhpOpenTag
            | Self::InvalidUtf8
            | Self::UnexpectedToken
            | Self::UnexpectedEof
            | Self::InvalidNumber
            | Self::UnterminatedString
            | Self::UnterminatedComment
            | Self::ExpectedVariable
            | Self::ExpectedSemicolon => EvalStatus::ParseError,
        }
    }
}

/// A parse failure plus the position and token PHP names in its diagnostic.
///
/// `EvalParseError` alone tells the ABI which status to report; PHP's own message also names
/// the failing token, the line it sits on, and the file that holds it. The parser knows all
/// three at the moment it gives up, so it attaches them here instead of letting the caller
/// print a constant that names none of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalParseDiagnostic {
    error: EvalParseError,
    line: i64,
    token: String,
}

impl EvalParseDiagnostic {
    /// Builds a diagnostic for a failure at one fragment line and token.
    pub fn new(error: EvalParseError, line: i64, token: impl Into<String>) -> Self {
        Self {
            error,
            line,
            token: token.into(),
        }
    }

    /// Builds a diagnostic for a failure detected before or during tokenization.
    ///
    /// Nothing has a token position yet, so the failure is reported at the fragment's last
    /// line, which is where PHP reports an unterminated literal or comment too.
    pub fn at_source_end(error: EvalParseError, code: &[u8], token: impl Into<String>) -> Self {
        let line = 1 + i64::try_from(code.iter().filter(|byte| **byte == b'\n').count())
            .unwrap_or(i64::MAX - 1);
        Self::new(error, line, token)
    }

    /// Returns the underlying parse failure.
    pub const fn error(&self) -> EvalParseError {
        self.error
    }

    /// Returns the ABI status that should be reported for this failure.
    pub const fn status(&self) -> EvalStatus {
        self.error.status()
    }

    /// Returns the one-based fragment line the failing token starts on.
    pub const fn line(&self) -> i64 {
        self.line
    }

    /// Returns PHP's description of the failing token, such as `identifier "bar"`.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Renders the reason clause PHP puts after `Parse error: `.
    ///
    /// PHP appends `, expecting "…"` from its LALR item set when the grammar admits a small
    /// set of continuations. A recursive-descent parser has no such set, and emitting a
    /// different one would be worse than emitting none, so the clause is left out.
    pub fn reason(&self) -> String {
        format!("syntax error, unexpected {}", self.token)
    }

    /// Returns the same diagnostic with its line shifted by a whole-file offset.
    ///
    /// Parsing runs on the bytes between one `<?php` and the next `?>`, so a fragment line is
    /// the file line only when the tag opens the file. The offset restores the file line PHP
    /// reports for a fragment that follows inline HTML or an earlier code block.
    pub fn with_line_offset(&self, offset: i64) -> Self {
        Self {
            error: self.error,
            line: self.line.saturating_add(offset),
            token: self.token.clone(),
        }
    }

    /// Renders the diagnostic PHP prints for a parse failure inside an included file.
    pub fn include_message(&self, file: &str) -> String {
        format!(
            "\nParse error: {} in {file} on line {}\n",
            self.reason(),
            self.line
        )
    }

    /// Renders the diagnostic PHP prints for a parse failure inside an `eval()` fragment.
    ///
    /// PHP names the file and line of the `eval()` call itself, then the line inside the
    /// evaluated fragment.
    pub fn eval_message(&self, caller_file: &str, caller_line: i64) -> String {
        format!(
            "\nParse error: {} in {caller_file}({caller_line}) : eval()'d code on line {}\n",
            self.reason(),
            self.line
        )
    }
}

/// Writes one fatal PHP diagnostic where PHP itself writes it.
///
/// Measured against `php -n` 8.5.6: a parse error is printed on standard output, not standard
/// error, and `@` never suppresses it, so it must not travel through the runtime's warning
/// helper. The write is flushed immediately because the generated runtime echoes with
/// unbuffered writes and the diagnostic has to keep its place among them.
pub fn report_fatal_diagnostic(message: &str) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(message.as_bytes());
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies a fragment line is shifted onto the file line the fragment starts at.
    #[test]
    fn line_offset_moves_a_fragment_line_onto_its_file_line() {
        let diagnostic = EvalParseDiagnostic::new(EvalParseError::UnexpectedToken, 2, "token \";\"");

        assert_eq!(diagnostic.with_line_offset(4).line(), 6);
        assert_eq!(diagnostic.with_line_offset(0).line(), 2);
    }

    /// Verifies the include diagnostic reproduces PHP's file-and-line wording.
    #[test]
    fn include_diagnostic_names_the_file_the_line_and_the_token() {
        let diagnostic =
            EvalParseDiagnostic::new(EvalParseError::UnexpectedToken, 290, "identifier \"bar\"");

        assert_eq!(
            diagnostic.include_message("/app/x.php"),
            "\nParse error: syntax error, unexpected identifier \"bar\" in /app/x.php on line 290\n"
        );
    }

    /// Verifies the eval diagnostic reproduces PHP's `eval()'d code` wording.
    #[test]
    fn eval_diagnostic_names_the_call_site_and_the_fragment_line() {
        let diagnostic =
            EvalParseDiagnostic::new(EvalParseError::UnexpectedToken, 1, "variable \"$b\"");

        assert_eq!(
            diagnostic.eval_message("/app/x.php", 3),
            "\nParse error: syntax error, unexpected variable \"$b\" in /app/x.php(3) : eval()'d code on line 1\n"
        );
    }

    /// Verifies a pre-tokenization failure is reported at the fragment's last line.
    #[test]
    fn source_end_diagnostic_counts_fragment_lines() {
        let diagnostic = EvalParseDiagnostic::at_source_end(
            EvalParseError::UnterminatedString,
            b"$a = 1;\n$b = 2;\n$c = \"x",
            "end of file",
        );

        assert_eq!(diagnostic.line(), 3);
        assert_eq!(diagnostic.status(), EvalStatus::ParseError);
    }

    /// Verifies only known unsupported syntax maps to the unsupported ABI status.
    #[test]
    fn parse_error_status_distinguishes_unsupported_constructs() {
        assert_eq!(
            EvalParseError::UnsupportedConstruct.status(),
            EvalStatus::UnsupportedConstruct
        );
        assert_eq!(
            EvalParseError::UnexpectedToken.status(),
            EvalStatus::ParseError
        );
    }
}
