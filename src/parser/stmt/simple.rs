//! Purpose:
//! Parses simple statement forms with minimal nested structure.
//! Handles includes, echo, expression statements, returns, throws, `$this` statements, constants,
//! and `declare(...)` directives.
//!
//! Called from:
//! - `crate::parser::stmt::parse_stmt()`.
//!
//! Key details:
//! - Include statements preserve their path expression for resolver include discovery and loading.
//! - `declare(...)` directives (`strict_types`, `ticks`, `encoding`) are parsed and discarded:
//!   elephc type-checks statically and does not support tick handlers or alternate source
//!   encodings, so the statement form lowers to a no-op and the block form keeps only its body.

use crate::errors::CompileError;
use crate::lexer::Token;
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::parser::expr::{parse_assignment_value_expr, parse_expr};
use crate::span::Span;

use super::assign::try_parse_postfix_assignment;
use super::{expect_semicolon, expect_token, parse_block};

/// Parses `include`/`require` (with optional `_once`) statements.
///
/// Consumes the keyword, optionally the parentheses, then the path expression.
/// Produces a `StmtKind::Include` with `once` and `required` flags set.
/// The path expression is preserved for resolver include discovery.
pub(super) fn parse_include(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
    once: bool,
    required: bool,
) -> Result<Stmt, CompileError> {
    *pos += 1; // consume include/require keyword

    // Support both: include 'file.php'; and include('file.php');
    let has_parens = *pos < tokens.len() && tokens[*pos].0 == Token::LParen;
    if has_parens {
        *pos += 1;
    }

    let path = parse_expr(tokens, pos)?;

    if has_parens {
        if *pos >= tokens.len() || tokens[*pos].0 != Token::RParen {
            return Err(CompileError::new(span, "Expected ')' after include path"));
        }
        *pos += 1;
    }

    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(
        StmtKind::Include {
            path,
            once,
            required,
        },
        span,
    ))
}

/// If the token at `*pos` begins an `include`/`require` (optionally `_once`), parses it as a
/// value-position include and returns an expression evaluating to the include's value, without
/// consuming a trailing semicolon. Returns `Ok(None)` when the next token is not an include
/// keyword.
///
/// PHP allows `include`/`require` in expression position (e.g. `return require X;` or
/// `$x = require X;`) and the expression evaluates to the included file's `return` value, or `1`
/// when the file has no explicit `return`. The included file runs in the *calling* scope, so it
/// can read and write the caller's variables. elephc represents this with a transient
/// `ExprKind::IncludeValue` marker that the resolver expands by inlining the included file's
/// statements into the caller's statement list (sharing scope) and capturing its top-level
/// `return` into a hidden temporary.
///
/// Visible across `crate::parser` (not just `crate::parser::stmt`) so the general prefix
/// expression parser (`crate::parser::expr::prefix`) can also parse `include`/`require` as an
/// operand in arbitrary expression position (e.g. `$x ??= require F;`, `f(require F)`).
pub(in crate::parser) fn try_parse_value_include(
    tokens: &[(Token, Span)],
    pos: &mut usize,
) -> Result<Option<Expr>, CompileError> {
    let (once, required) = match tokens.get(*pos).map(|(token, _)| token) {
        Some(Token::Include) => (false, false),
        Some(Token::IncludeOnce) => (true, false),
        Some(Token::Require) => (false, true),
        Some(Token::RequireOnce) => (true, true),
        _ => return Ok(None),
    };
    let span = tokens[*pos].1;
    *pos += 1; // consume the include/require keyword

    let has_parens = *pos < tokens.len() && tokens[*pos].0 == Token::LParen;
    if has_parens {
        *pos += 1;
    }
    let path = parse_expr(tokens, pos)?;
    if has_parens {
        expect_token(tokens, pos, &Token::RParen, "Expected ')' after include path")?;
    }

    Ok(Some(Expr::new(
        ExprKind::IncludeValue {
            path: Box::new(path),
            once,
            required,
        },
        span,
    )))
}

/// Parses `echo` statements with one or more comma-separated expressions.
///
/// Consumes `echo`, then parses expressions until a non-comma token is encountered.
/// Returns a single `Echo` statement for one expression, or a `Synthetic` wrapper
/// containing multiple `Echo` statements for PHP's multi-argument echo syntax.
pub(super) fn parse_echo(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let mut echoed = Vec::new();

    loop {
        let expr = parse_expr(tokens, pos)?;
        echoed.push(Stmt::new(StmtKind::Echo(expr), span));
        if !matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Comma)) {
            break;
        }
        *pos += 1;
    }

    expect_semicolon(tokens, pos)?;
    // Preserve the single-expression AST shape and reuse the existing
    // statement sequence wrapper for PHP's multi-argument echo syntax.
    if echoed.len() == 1 {
        Ok(echoed.remove(0))
    } else {
        Ok(Stmt::new(StmtKind::Synthetic(echoed), span))
    }
}

/// Parses a bare expression statement: an expression followed by a semicolon.
///
/// Parses the expression, consumes the trailing semicolon, and wraps it in
/// `StmtKind::ExprStmt`.
pub(super) fn parse_expr_stmt(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    let expr = parse_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::ExprStmt(expr), span))
}

/// Parses a statement prefixed with `@` error suppression.
///
/// Checks the next token to detect an include/require variant; if found,
/// delegates to `parse_include` with error-suppression semantics. Otherwise,
/// falls back to `parse_expr_stmt`.
pub(super) fn parse_error_suppressed_stmt(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    match tokens.get(*pos + 1).map(|(token, _)| token) {
        Some(Token::Include) => {
            *pos += 1;
            parse_include(tokens, pos, span, false, false)
        }
        Some(Token::IncludeOnce) => {
            *pos += 1;
            parse_include(tokens, pos, span, true, false)
        }
        Some(Token::Require) => {
            *pos += 1;
            parse_include(tokens, pos, span, false, true)
        }
        Some(Token::RequireOnce) => {
            *pos += 1;
            parse_include(tokens, pos, span, true, true)
        }
        _ => parse_expr_stmt(tokens, pos, span),
    }
}

/// Parses a `return` statement, with or without a value.
///
/// Consumes `return`. If the next token is a semicolon, returns `Return(None)`.
/// Otherwise parses an expression and consumes the trailing semicolon.
pub(super) fn parse_return(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;

    if *pos < tokens.len() && tokens[*pos].0 == Token::Semicolon {
        *pos += 1;
        return Ok(Stmt::new(StmtKind::Return(None), span));
    }

    // `return require X;` returns the included file's value (its top-level `return`, or `1`).
    if let Some(include_value) = try_parse_value_include(tokens, pos)? {
        expect_semicolon(tokens, pos)?;
        return Ok(Stmt::new(StmtKind::Return(Some(include_value)), span));
    }

    let expr = parse_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::Return(Some(expr)), span))
}

/// Parses a `throw` statement: consumes `throw`, then the exception expression.
///
/// Consumes the trailing semicolon and wraps the expression in `StmtKind::Throw`.
pub(super) fn parse_throw(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let expr = parse_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::Throw(expr), span))
}

/// Parses statements starting with `$this`.
///
/// First attempts to parse a postfix assignment via `try_parse_postfix_assignment`.
/// If that yields nothing, parses the expression. If the next token after the
/// expression is `=` and the expression is a property access, produces a
/// `StmtKind::PropertyAssign`; otherwise returns an `ExprStmt`. Consumes the
/// trailing semicolon in all cases.
pub(super) fn parse_this_stmt(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    if let Some(stmt) = try_parse_postfix_assignment(tokens, pos, span)? {
        return Ok(stmt);
    }

    // Parse as expression first
    let expr = parse_expr(tokens, pos)?;
    // Check if followed by assignment
    if *pos < tokens.len() && tokens[*pos].0 == Token::Assign {
        *pos += 1;
        let value = parse_assignment_value_expr(tokens, pos)?;
        expect_semicolon(tokens, pos)?;
        if let ExprKind::PropertyAccess { object, property } = expr.kind {
            return Ok(Stmt::new(
                StmtKind::PropertyAssign {
                    object,
                    property,
                    value,
                },
                span,
            ));
        }
        return Err(CompileError::new(
            span,
            "Invalid assignment target after $this",
        ));
    }
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::ExprStmt(expr), span))
}

/// Parses a `const` declaration: `const NAME = value;`.
///
/// Consumes `const`, expects an identifier for the name, then `=`, then the
/// value expression, and consumes the trailing semicolon. Returns a
/// `StmtKind::ConstDecl` with the name and value.
pub(super) fn parse_const_decl(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1; // consume 'const'

    let name = match tokens.get(*pos).map(|(t, _)| t) {
        Some(Token::Identifier(n)) => n.clone(),
        _ => {
            return Err(CompileError::new(
                span,
                "Expected constant name after 'const'",
            ))
        }
    };
    *pos += 1;

    expect_token(
        tokens,
        pos,
        &Token::Assign,
        "Expected '=' after constant name",
    )?;

    let value = parse_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;

    Ok(Stmt::new(StmtKind::ConstDecl { name, value }, span))
}

/// Parses a `declare(directive = value, ...)` statement, in either its statement form
/// (`declare(...);`) or block form (`declare(...) { ... }`).
///
/// PHP's `declare` construct configures compile-time directives such as `strict_types`,
/// `ticks`, and `encoding`. elephc type-checks statically already (so `strict_types` changes
/// no behavior), and does not support tick handlers or alternate source encodings, so every
/// directive is parsed for syntax only and then discarded. Directive values are literals (an
/// integer for `strict_types`/`ticks`, a string for `encoding`) — parsed directly as `ident =
/// literal` pairs rather than through the general expression parser, since a bareword directive
/// name is not a valid assignment target for `parse_expr`.
///
/// The statement form lowers to an empty `StmtKind::Synthetic` (a no-op). The block form lowers
/// to a `StmtKind::Synthetic` wrapping the block's statements, so the body runs normally while
/// the directive list itself is discarded.
pub(super) fn parse_declare(
    tokens: &[(Token, Span)],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1; // consume 'declare'
    expect_token(tokens, pos, &Token::LParen, "Expected '(' after 'declare'")?;

    loop {
        match tokens.get(*pos).map(|(t, _)| t) {
            Some(Token::Identifier(_)) => *pos += 1, // consume the directive name
            _ => {
                return Err(CompileError::new(
                    span,
                    "Expected a directive name inside 'declare(...)'",
                ))
            }
        }

        expect_token(
            tokens,
            pos,
            &Token::Assign,
            "Expected '=' after declare directive name",
        )?;

        match tokens.get(*pos).map(|(t, _)| t) {
            Some(Token::IntLiteral(_)) | Some(Token::StringLiteral(_)) => *pos += 1, // consume the literal directive value
            _ => {
                return Err(CompileError::new(
                    span,
                    "Expected an integer or string literal for declare directive value",
                ))
            }
        }

        if matches!(tokens.get(*pos).map(|(t, _)| t), Some(Token::Comma)) {
            *pos += 1; // consume ',' and parse the next directive
            continue;
        }
        break;
    }

    expect_token(
        tokens,
        pos,
        &Token::RParen,
        "Expected ')' after declare directives",
    )?;

    // Block form: `declare(...) { ... }` — keep the body, discard the directives.
    if *pos < tokens.len() && tokens[*pos].0 == Token::LBrace {
        let body = parse_block(tokens, pos)?;
        return Ok(Stmt::new(StmtKind::Synthetic(body), span));
    }

    // Statement form: `declare(...);` — a no-op.
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::Synthetic(Vec::new()), span))
}
