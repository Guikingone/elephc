//! Purpose:
//! Resolves include file paths and parses included PHP source files.
//! Runs lexer, parser, and magic-constant substitution for included files.
//!
//! Called from:
//! - `crate::resolver::engine_includes` and include discovery.
//!
//! Key details:
//! - Included-file diagnostics are tagged with the target file and original include span.

use std::path::{Path, PathBuf};

use crate::errors::CompileError;
use crate::lexer;
use crate::parser;
use crate::parser::ast::Stmt;
use crate::span::Span;

/// Resolves a relative include path against a base directory.
///
/// Returns the path unchanged if already absolute, otherwise joins it
/// with `base_dir`. The path string is not validated for existence.
pub(super) fn resolve_path(path: &str, base_dir: &Path) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    }
}

/// Parses an included PHP source file, returning its AST.
///
/// Reads the file contents from disk, tokenizes, and parses to a `Vec<Stmt>`.
/// Errors include the original `include_span` for diagnostics tracing.
///
/// The parsed statements are normalized so an `else`-less `if` whose branches all return adopts the
/// statements following it as its `else` body (see `include_returns::normalize_early_returns`).
/// Both declaration discovery and include expansion parse through here, so both observe the same
/// mutually exclusive shape for a file that dispatches with an early `return`.
pub(super) fn parse_file(path: &Path, include_span: Span) -> Result<Vec<Stmt>, CompileError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        CompileError::new(
            include_span,
            &format!("Cannot read '{}': {}", path.display(), e),
        )
    })?;

    let file = path.display().to_string();

    let tokens = lexer::tokenize(&source).map_err(|e| e.with_file(file.clone()))?;

    let stmts = parser::parse(&tokens).map_err(|e| e.with_file(file))?;

    Ok(super::include_returns::normalize_early_returns(stmts))
}
