//! Purpose:
//! Parses statements that begin with a PHP variable token.
//! Routes variable syntax to compound assignment, postfix assignment, or expression-statement parsing.
//!
//! Called from:
//! - `crate::parser::stmt::parse_stmt()`.
//!
//! Key details:
//! - Variable-leading statements are ambiguous, so dispatch order protects assignment-specific syntax first.

use super::{compound, postfix};
use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{Expr, ExprKind, Stmt, StmtKind};
use crate::parser::expr::{parse_assignment_value_expr, parse_expr};
use crate::span::Span;

use super::super::expect_semicolon;

/// Parses statements that begin with a PHP variable token (`$name`).
///
/// Dispatches to postfix assignment, post-increment/decrement, property access with
/// compound assignment, closure calls, or regular/compound assignment based on the
/// token that follows the variable. When the variable is not immediately followed by
/// an assignment operator, it is treated as an operand of a larger expression (e.g. a
/// comparison, ternary, logical, or `instanceof` expression) and parsed as a bare
/// expression statement.
///
/// # Arguments
/// - `tokens` — the token stream
/// - `pos` — current position (mutated by parsing)
/// - `span` — source span of the statement
///
/// # Returns
/// `Stmt` with `StmtKind::PostIncrement`, `PostDecrement`, `PropertyAssign`,
/// `ExprStmt`, or compound/regular assignment variants.
///
/// # Panics
/// Unreachable if the first token is not `Token::Variable`.
pub(in crate::parser::stmt) fn parse_variable_stmt(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    let name = match &tokens[*pos].0 {
        Token::Variable(n) => n.clone(),
        _ => unreachable!(),
    };

    // A statement beginning with `$GLOBALS` never reaches the expression parser's own refusal,
    // so the same rule is applied here. Without it `$GLOBALS = [...]` compiled and kept running
    // past the point where PHP raises its fatal — a silent divergence, and the single shape this
    // module must never allow. `$GLOBALS['name'] = ...` is the supported form and is left alone.
    if name == "GLOBALS" {
        let next = tokens.get(*pos + 1).map(|(t, _)| t);
        if let Some(message) = crate::globals_array::unsupported_use_message(
            matches!(next, Some(Token::LBracket)),
            matches!(next, Some(Token::Assign)),
        ) {
            return Err(CompileError::new(span, message));
        }
    }

    if let Some(stmt) = postfix::try_parse_postfix_assignment(tokens, pos, span)? {
        return Ok(stmt);
    }
    if let Some(stmt) = postfix::try_parse_postfix_incdec(tokens, pos, span)? {
        return Ok(stmt);
    }

    // Post-increment/decrement
    if *pos + 1 < tokens.len() {
        match &tokens[*pos + 1].0 {
            Token::PlusPlus => {
                *pos += 2;
                expect_semicolon(tokens, pos)?;
                let expr = Expr::new(ExprKind::PostIncrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            Token::MinusMinus => {
                *pos += 2;
                expect_semicolon(tokens, pos)?;
                let expr = Expr::new(ExprKind::PostDecrement(name), span);
                return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
            }
            _ => {}
        }
    }

    if *pos + 1 < tokens.len()
        && matches!(
            tokens[*pos + 1].0,
            Token::Arrow | Token::QuestionArrow | Token::LBracket | Token::DoubleColon
        )
    {
        let expr = parse_expr(tokens, pos)?;
        if let Some(op) = tokens
            .get(*pos)
            .and_then(|(token, _)| compound::assignment_operator(token))
        {
            *pos += 1;
            let rhs = parse_assignment_value_expr(tokens, pos)?;
            expect_semicolon(tokens, pos)?;
            if let ExprKind::PropertyAccess { object, property } = expr.kind {
                let target = Expr::new(
                    ExprKind::PropertyAccess {
                        object: object.clone(),
                        property: property.clone(),
                    },
                    span,
                );
                let value = compound::assignment_value(target, op, rhs, span);
                return Ok(Stmt::new(
                    StmtKind::PropertyAssign {
                        object,
                        property,
                        value,
                    },
                    span,
                ));
            }
            return Err(CompileError::new(span, "Invalid assignment target"));
        }
        expect_semicolon(tokens, pos)?;
        return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
    }

    // Closure call: $fn(args);
    if *pos + 1 < tokens.len() && tokens[*pos + 1].0 == Token::LParen {
        let expr = parse_expr(tokens, pos)?;
        expect_semicolon(tokens, pos)?;
        return Ok(Stmt::new(StmtKind::ExprStmt(expr), span));
    }

    // Regular or compound assignment, only when an assignment operator directly
    // follows the variable; otherwise the variable is an operand of a larger
    // expression statement (comparison, ternary, logical, `instanceof`, bare use, …).
    if *pos + 1 < tokens.len()
        && compound::assignment_operator(&tokens[*pos + 1].0).is_some()
    {
        return compound::parse_assign(tokens, pos, span);
    }

    // A literal or variable directly after the variable (`$x "hi";`, `$x 5;`, `$x $y;`)
    // can never continue the `$x` expression: the only legal statement shape there is an
    // assignment whose `=` is missing, so report that instead of a misleading
    // "Expected ';'" from the generic expression fallback. Only prefix-exclusive tokens
    // are matched — operators like `-`, `(`, or `[` legitimately continue `$x` as infix
    // or postfix syntax and must keep the expression-statement path.
    if *pos + 1 < tokens.len()
        && matches!(
            tokens[*pos + 1].0,
            Token::StringLiteral(_)
                | Token::IntLiteral(_)
                | Token::FloatLiteral(_)
                | Token::Variable(_)
        )
    {
        return Err(CompileError::new(span, "Expected '=' after variable name"));
    }

    let expr = parse_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(StmtKind::ExprStmt(expr), span))
}
