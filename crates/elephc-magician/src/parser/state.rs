//! Purpose:
//! Parses tokenized eval fragments into EvalIR.
//! The parser state owns PHP statement and expression grammar for the runtime
//! eval subset after tokenization has completed.
//!
//! Called from:
//! - `crate::parser::parse_fragment()`.
//!
//! Key details:
//! - Namespace imports are tracked as parser state and restored across blocks.
//! - Unsupported PHP constructs fail with explicit parse statuses instead of
//!   partially lowering ambiguous syntax.

use super::cursor::split_first_name_segment;
use crate::errors::{EvalParseDiagnostic, EvalParseError};
use crate::eval_ir::EvalProgram;
use crate::lexer::{Token, TokenKind};
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) const EVAL_YIELD_INTRINSIC: &str = "__elephc_eval_yield";

/// Marker call the parser emits for `yield from EXPR`.
///
/// Delegation needs its own marker rather than a flag on the plain one, because the two differ
/// in what they do to the generator's key counter: `php -n` 8.5.6 preserves the inner keys of a
/// delegated array or generator and leaves the OUTER auto-increment counter untouched, so
/// `yield 0; yield from [10, 20]; yield 99;` produces the keys 0, 0, 1, 1.
pub(crate) const EVAL_YIELD_FROM_INTRINSIC: &str = "__elephc_eval_yield_from";

static ANONYMOUS_CLASS_COUNTER: AtomicUsize = AtomicUsize::new(0);
static CLOSURE_FUNCTION_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Parses tokenized eval fragments into EvalIR.
pub(super) struct Parser {
    pub(super) tokens: Vec<TokenKind>,
    pub(super) token_lines: Vec<i64>,
    pub(super) pos: usize,
    pub(super) source_len: usize,
    pub(super) namespace: String,
    pub(super) imports: NamespaceImports,
    pub(super) allow_use_imports: bool,
    /// How many class-like bodies enclose the rule currently running.
    ///
    /// PHP resolves `self` and `parent` against the enclosing class, and a closure or arrow
    /// function written inside a class body inherits that scope. Its parameters are parsed in the
    /// plain-function type position, so the position alone cannot tell a legal `self` from the
    /// top-level one PHP rejects; this depth can.
    pub(super) class_scope_depth: usize,
    /// Where the failing token sits, when the cursor has already moved past it.
    ///
    /// A recursive-descent rule can consume several tokens before it discovers that the shape
    /// it just read is not assignable, so the cursor at the point the error surfaces is not
    /// always the token PHP names. This parser never backtracks, so exactly one error is ever
    /// created per fragment and the first stamp is the one that describes it.
    pub(super) error_pos: Option<usize>,
    /// Whether to emit `EvalStmt::SourceLine` markers before each parsed statement.
    ///
    /// Set only by `parse_source_file`. See the variant's own docblock for why a fragment does
    /// not carry them.
    pub(super) track_source_lines: bool,
    /// Line of the last marker emitted, so an unchanged line does not emit a second one.
    pub(super) last_source_line: i64,
}

/// A parsed PHP name plus whether it used a leading global namespace separator.
pub(super) struct ParsedQualifiedName {
    pub(super) name: String,
    pub(super) absolute: bool,
}

/// Import alias tables active for the current namespace declaration region.
#[derive(Default)]
pub(super) struct NamespaceImports {
    classes: HashMap<String, String>,
    functions: HashMap<String, String>,
    constants: HashMap<String, String>,
}

/// The `use` declaration namespace being imported.
#[derive(Copy, Clone, Eq, PartialEq)]
pub(super) enum UseImportKind {
    Class,
    Function,
    Const,
}

/// Removes every doc-comment token the grammar never reads, keeping the token lines aligned.
///
/// PHP's scanner hands `T_DOC_COMMENT` to `zendlex()`, which skips it exactly like whitespace and
/// parks the text in `CG(doc_comment)` for the next declaration; no grammar rule ever sees the
/// token. This lexer instead keeps the token in the stream so `ReflectionClass::getDocComment()`
/// can report it, and every rule that does not expect one — a parameter list above all, but also
/// an argument list, an array literal or a `match` arm — refused a file `php -n` parses. Dropping
/// the doc comments that no rule reads reproduces PHP's transparency while leaving the one place
/// the grammar does read them, a doc comment in front of a class declaration, untouched.
fn drop_unread_doc_comments(
    tokens: Vec<TokenKind>,
    lines: Vec<i64>,
) -> (Vec<TokenKind>, Vec<i64>) {
    if !tokens
        .iter()
        .any(|token| matches!(token, TokenKind::DocComment(_)))
    {
        return (tokens, lines);
    }
    let read: Vec<bool> = tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            !matches!(token, TokenKind::DocComment(_))
                || tokens[index + 1..]
                    .iter()
                    .find(|next| !matches!(next, TokenKind::DocComment(_)))
                    .is_some_and(super::cursor::starts_doc_commented_declaration)
        })
        .collect();
    let kept_tokens = tokens
        .into_iter()
        .zip(read.iter())
        .filter_map(|(token, keep)| keep.then_some(token))
        .collect();
    let kept_lines = lines
        .into_iter()
        .zip(read.iter().chain(std::iter::repeat(&true)))
        .filter_map(|(line, keep)| keep.then_some(line))
        .collect();
    (kept_tokens, kept_lines)
}

/// Returns a parser-global synthetic class name for one eval anonymous class expression.
pub(super) fn next_anonymous_class_name() -> String {
    let id = ANONYMOUS_CLASS_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("class@anonymous#eval{id}")
}

/// Returns a parser-global synthetic function name for one eval closure expression.
pub(super) fn next_closure_function_name() -> String {
    let id = CLOSURE_FUNCTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{{closure:eval:function:{id}}}")
}

impl NamespaceImports {
    /// Stores one class import under PHP's case-insensitive class alias key.
    pub(super) fn insert_class(&mut self, alias: String, name: String) {
        self.classes.insert(alias.to_ascii_lowercase(), name);
    }

    /// Stores one function import under PHP's case-insensitive function alias key.
    pub(super) fn insert_function(&mut self, alias: String, name: String) {
        self.functions.insert(alias.to_ascii_lowercase(), name);
    }

    /// Stores one constant import under PHP's case-sensitive constant alias key.
    pub(super) fn insert_constant(&mut self, alias: String, name: String) {
        self.constants.insert(alias, name);
    }

    /// Resolves a class import, including aliases used as the first segment of a class name.
    pub(super) fn resolve_class(&self, name: &str) -> Option<String> {
        let (first, tail) = split_first_name_segment(name);
        let imported = self.classes.get(&first.to_ascii_lowercase())?;
        Some(match tail {
            Some(tail) => format!("{imported}\\{tail}"),
            None => imported.clone(),
        })
    }

    /// Resolves an unqualified function alias.
    pub(super) fn resolve_function(&self, name: &str) -> Option<&str> {
        self.functions
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// Resolves a case-sensitive unqualified constant alias.
    pub(super) fn resolve_constant(&self, name: &str) -> Option<&str> {
        self.constants.get(name).map(String::as_str)
    }
}

impl Parser {
    /// Creates a parser over tokens produced from a source fragment.
    pub(super) fn new(tokens: Vec<Token>, source_len: usize) -> Self {
        let token_lines = tokens.iter().map(Token::line).collect();
        let tokens = tokens.into_iter().map(Token::into_kind).collect();
        let (tokens, token_lines) = drop_unread_doc_comments(tokens, token_lines);
        Self {
            tokens,
            token_lines,
            pos: 0,
            source_len,
            namespace: String::new(),
            imports: NamespaceImports::default(),
            allow_use_imports: true,
            class_scope_depth: 0,
            error_pos: None,
            track_source_lines: false,
            last_source_line: 0,
        }
    }

    /// Turns on `EvalStmt::SourceLine` markers for a whole-file parse.
    pub(super) const fn tracking_source_lines(mut self) -> Self {
        self.track_source_lines = true;
        self
    }

    /// Runs one class-like body parser with `self` and `parent` legal in every nested type.
    pub(super) fn in_class_scope<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> Result<T, EvalParseError>,
    ) -> Result<T, EvalParseError> {
        self.class_scope_depth += 1;
        let parsed = body(self);
        self.class_scope_depth -= 1;
        parsed
    }

    /// Records the current token as the one a failure should name, then returns that failure.
    pub(super) fn fail(&mut self, error: EvalParseError) -> EvalParseError {
        self.fail_at(self.pos, error)
    }

    /// Records a specific token as the one a failure should name, then returns that failure.
    pub(super) fn fail_at(&mut self, pos: usize, error: EvalParseError) -> EvalParseError {
        if self.error_pos.is_none() {
            self.error_pos = Some(pos);
        }
        error
    }

    /// Returns the token index a diagnostic should name for the failure just reported.
    fn failing_token_pos(&self) -> usize {
        self.error_pos.unwrap_or(self.pos)
    }

    /// Parses a complete eval fragment until EOF.
    ///
    /// The cursor stops on the token the grammar could not accept, so the failure is turned
    /// into a positioned diagnostic here rather than at any of the hundreds of sites that
    /// return a bare `EvalParseError`.
    pub(super) fn parse_program(mut self) -> Result<EvalProgram, EvalParseDiagnostic> {
        let mut statements = Vec::new();
        while !matches!(self.current(), TokenKind::Eof) {
            match self.parse_stmt() {
                Ok(parsed) => statements.extend(parsed),
                Err(error) => {
                    self.trace_parse_error(&error);
                    let pos = self.failing_token_pos();
                    return Err(EvalParseDiagnostic::new(
                        error,
                        self.token_lines.get(pos).copied().unwrap_or(1),
                        self.tokens
                            .get(pos)
                            .unwrap_or(&TokenKind::Eof)
                            .php_description(),
                    ));
                }
            }
        }
        Ok(EvalProgram::new(self.source_len, statements))
    }

    /// Emits the failing token position when opt-in eval tracing is enabled.
    fn trace_parse_error(&self, error: &EvalParseError) {
        if !crate::eval_trace::enabled() {
            return;
        }
        let _ = catch_unwind(AssertUnwindSafe(|| {
            eprintln!(
                "[elephc-eval-trace] kind=parser phase=parse_error error={error:?} pos={} line={:?} previous={:?} current={:?} next={:?}",
                self.pos,
                self.token_lines.get(self.pos),
                self.pos.checked_sub(1).and_then(|pos| self.tokens.get(pos)),
                self.tokens.get(self.pos),
                self.tokens.get(self.pos + 1),
            );
        }));
    }
}
