//! Purpose:
//! Parses and desugars PHP `goto` statements whose labels introduce terminal fallback tails.
//! Keeps the emitted AST structured by replacing each supported jump with a cloned tail.
//!
//! Called from:
//! - `crate::parser::stmt::parse_stmt_dispatch()` and executable-scope parsing entry points.
//!
//! Key details:
//! - Only label tails guaranteed to return or throw are supported; arbitrary CFG jumps remain errors.
//! - Parser-only markers use NUL-prefixed strings, which cannot collide with PHP source strings.

use std::collections::HashMap;

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::span::Span;

const GOTO_MARKER_PREFIX: &str = "\0elephc-terminal-goto:";
const LABEL_MARKER_PREFIX: &str = "\0elephc-terminal-label:";

/// Returns true when the token at `pos` starts a PHP label (`name:`) at statement position.
pub(super) fn starts_label(tokens: &[SpannedToken], pos: usize) -> bool {
    matches!(tokens.get(pos).map(|(token, _)| token), Some(Token::Identifier(_)))
        && matches!(tokens.get(pos + 1).map(|(token, _)| token), Some(Token::Colon))
}

/// Parses `goto label;` into an internal marker removed before the AST leaves the parser.
pub(super) fn parse_goto(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let name = match tokens.get(*pos).map(|(token, _)| token) {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(CompileError::new(span, "Expected label name after 'goto'")),
    };
    *pos += 1;
    super::stmt::expect_semicolon(tokens, pos)?;
    Ok(marker_stmt(GOTO_MARKER_PREFIX, &name, span))
}

/// Parses `label:` into an internal marker removed before the AST leaves the parser.
pub(super) fn parse_label(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    let name = match tokens.get(*pos).map(|(token, _)| token) {
        Some(Token::Identifier(name)) => name.clone(),
        _ => return Err(CompileError::new(span, "Expected goto label name")),
    };
    *pos += 2;
    Ok(marker_stmt(LABEL_MARKER_PREFIX, &name, span))
}

/// Rewrites supported gotos within one PHP executable scope without crossing function boundaries.
pub(super) fn desugar_scope(statements: &mut Vec<Stmt>) -> Result<(), CompileError> {
    let mut labels = HashMap::new();
    collect_labels(statements, &mut labels)?;
    rewrite_gotos(statements, &labels)
}

/// Creates a parser-only marker statement for a goto or label.
fn marker_stmt(prefix: &str, name: &str, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Synthetic(vec![Stmt::new(
            StmtKind::ExprStmt(Expr::new(
                ExprKind::StringLiteral(format!("{}{}", prefix, name)),
                span,
            )),
            span,
        )]),
        span,
    )
}

/// Extracts a parser-only marker name when `statement` is a goto or label marker.
fn marker_name<'a>(statement: &'a Stmt, prefix: &str) -> Option<&'a str> {
    let StmtKind::Synthetic(body) = &statement.kind else {
        return None;
    };
    let [inner] = body.as_slice() else {
        return None;
    };
    let StmtKind::ExprStmt(expr) = &inner.kind else {
        return None;
    };
    let ExprKind::StringLiteral(value) = &expr.kind else {
        return None;
    };
    value.strip_prefix(prefix)
}

/// Collects each supported label and the terminal statement tail that begins after it.
fn collect_labels(
    statements: &[Stmt],
    labels: &mut HashMap<String, Vec<Stmt>>,
) -> Result<(), CompileError> {
    for (index, statement) in statements.iter().enumerate() {
        if let Some(name) = marker_name(statement, LABEL_MARKER_PREFIX) {
            let tail = statements[index + 1..].to_vec();
            if tail.is_empty() || !statements_terminate(&tail) || contains_marker(&tail) {
                return Err(unsupported_goto_error(
                    statement.span,
                    name,
                    "its target does not begin a self-contained terminal tail",
                ));
            }
            if labels.insert(name.to_string(), tail).is_some() {
                return Err(CompileError::new(
                    statement.span,
                    &format!("Label '{}' has already been defined in this scope", name),
                ));
            }
        }
        collect_nested_labels(statement, labels)?;
    }
    Ok(())
}

/// Descends through structured statement bodies while staying inside the current function scope.
fn collect_nested_labels(
    statement: &Stmt,
    labels: &mut HashMap<String, Vec<Stmt>>,
) -> Result<(), CompileError> {
    match &statement.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            collect_labels(then_body, labels)?;
            for (_, body) in elseif_clauses {
                collect_labels(body, labels)?;
            }
            if let Some(body) = else_body {
                collect_labels(body, labels)?;
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            collect_labels(then_body, labels)?;
            if let Some(body) = else_body {
                collect_labels(body, labels)?;
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. } => collect_labels(body, labels)?,
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                collect_labels(body, labels)?;
            }
            if let Some(body) = default {
                collect_labels(body, labels)?;
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            collect_labels(try_body, labels)?;
            for catch in catches {
                collect_labels(&catch.body, labels)?;
            }
            if let Some(body) = finally_body {
                collect_labels(body, labels)?;
            }
        }
        StmtKind::Synthetic(body) if marker_name(statement, LABEL_MARKER_PREFIX).is_none() => {
            collect_labels(body, labels)?;
        }
        _ => {}
    }
    Ok(())
}

/// Replaces goto markers with their cloned terminal tails and removes label markers.
fn rewrite_gotos(
    statements: &mut Vec<Stmt>,
    labels: &HashMap<String, Vec<Stmt>>,
) -> Result<(), CompileError> {
    let mut rewritten = Vec::with_capacity(statements.len());
    for mut statement in std::mem::take(statements) {
        if marker_name(&statement, LABEL_MARKER_PREFIX).is_some() {
            continue;
        }
        if let Some(name) = marker_name(&statement, GOTO_MARKER_PREFIX) {
            let Some(tail) = labels.get(name) else {
                return Err(unsupported_goto_error(
                    statement.span,
                    name,
                    "no matching terminal label exists in this scope",
                ));
            };
            statement.kind = StmtKind::Synthetic(tail.clone());
            rewritten.push(statement);
            continue;
        }
        rewrite_nested_gotos(&mut statement, labels)?;
        rewritten.push(statement);
    }
    *statements = rewritten;
    Ok(())
}

/// Descends through structured statement bodies to rewrite gotos in the current scope.
fn rewrite_nested_gotos(
    statement: &mut Stmt,
    labels: &HashMap<String, Vec<Stmt>>,
) -> Result<(), CompileError> {
    match &mut statement.kind {
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body,
            ..
        } => {
            rewrite_gotos(then_body, labels)?;
            for (_, body) in elseif_clauses {
                rewrite_gotos(body, labels)?;
            }
            if let Some(body) = else_body {
                rewrite_gotos(body, labels)?;
            }
        }
        StmtKind::IfDef {
            then_body,
            else_body,
            ..
        } => {
            rewrite_gotos(then_body, labels)?;
            if let Some(body) = else_body {
                rewrite_gotos(body, labels)?;
            }
        }
        StmtKind::While { body, .. }
        | StmtKind::DoWhile { body, .. }
        | StmtKind::For { body, .. }
        | StmtKind::Foreach { body, .. }
        | StmtKind::IncludeOnceGuard { body, .. }
        | StmtKind::NamespaceBlock { body, .. } => rewrite_gotos(body, labels)?,
        StmtKind::Switch { cases, default, .. } => {
            for (_, body) in cases {
                rewrite_gotos(body, labels)?;
            }
            if let Some(body) = default {
                rewrite_gotos(body, labels)?;
            }
        }
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            rewrite_gotos(try_body, labels)?;
            for catch in catches {
                rewrite_gotos(&mut catch.body, labels)?;
            }
            if let Some(body) = finally_body {
                rewrite_gotos(body, labels)?;
            }
        }
        StmtKind::Synthetic(body) => rewrite_gotos(body, labels)?,
        _ => {}
    }
    Ok(())
}

/// Returns whether a statement list cannot fall through its final statement.
fn statements_terminate(statements: &[Stmt]) -> bool {
    statements.last().is_some_and(statement_terminates)
}

/// Returns whether one structured statement guarantees a return or throw on every path.
fn statement_terminates(statement: &Stmt) -> bool {
    match &statement.kind {
        StmtKind::Return(_) | StmtKind::Throw(_) => true,
        StmtKind::Synthetic(body) => statements_terminate(body),
        StmtKind::If {
            then_body,
            elseif_clauses,
            else_body: Some(else_body),
            ..
        } => {
            statements_terminate(then_body)
                && elseif_clauses
                    .iter()
                    .all(|(_, body)| statements_terminate(body))
                && statements_terminate(else_body)
        }
        StmtKind::IfDef {
            then_body,
            else_body: Some(else_body),
            ..
        } => statements_terminate(then_body) && statements_terminate(else_body),
        StmtKind::Try {
            try_body,
            catches,
            finally_body,
        } => {
            finally_body.as_ref().is_some_and(|body| statements_terminate(body))
                || (statements_terminate(try_body)
                    && !catches.is_empty()
                    && catches.iter().all(|catch| statements_terminate(&catch.body)))
        }
        _ => false,
    }
}

/// Returns whether a prospective terminal tail still contains a goto or label marker.
fn contains_marker(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| {
        marker_name(statement, GOTO_MARKER_PREFIX).is_some()
            || marker_name(statement, LABEL_MARKER_PREFIX).is_some()
    })
}

/// Builds the diagnostic for a goto outside the deliberately supported terminal-tail subset.
fn unsupported_goto_error(span: Span, label: &str, reason: &str) -> CompileError {
    CompileError::new(
        span,
        &format!(
            "`goto` is not supported for label `{}`: {}; elephc currently supports only jumps to a label whose remaining block always returns or throws",
            label, reason
        ),
    )
}
