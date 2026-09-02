//! Purpose:
//! Parses do-while, for, foreach, and array-set loop-adjacent statements.
//!
//! Called from:
//! - `Parser::parse_stmt()` and for-clause parsing.
//!
//! Key details:
//! - Foreach key/value targets and statement bodies retain EvalIR source order.
//! - `for` and `foreach` accept PHP's alternative `:` … `endfor;`/`endforeach;` bodies, which
//!   lower to the same EvalIR as the brace form.

use super::*;

impl Parser {
    /// Parses `do { ... } while (expr);`.
    pub(in crate::parser) fn parse_do_while_stmt(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        let body = self.parse_statement_body()?;
        if !matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "while")) {
            return Err(EvalParseError::UnexpectedToken);
        }
        self.advance();
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        self.expect_semicolon()?;
        Ok(vec![EvalStmt::DoWhile { body, condition }])
    }

    /// Parses `$name[index] = expr;`, `$name[] = expr;`, and nested `??=` writes.
    pub(in crate::parser) fn parse_array_set_stmt(
        &mut self,
        name: String,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        self.expect(TokenKind::LBracket)?;
        if self.consume(TokenKind::RBracket) {
            self.expect(TokenKind::Equal)?;
            if self.consume(TokenKind::Ampersand) {
                let source = self.parse_expr()?;
                self.expect_semicolon()?;
                return Ok(vec![EvalStmt::ArrayAppendReferenceBind { name, source }]);
            }
            let value = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(vec![EvalStmt::ArrayAppendVar { name, value }]);
        }
        let index = self.parse_expr()?;
        self.expect(TokenKind::RBracket)?;
        let mut target = EvalExpr::ArrayGet {
            array: Box::new(EvalExpr::LoadVar(name.clone())),
            index: Box::new(index.clone()),
        };
        let mut nested = false;
        while self.consume(TokenKind::LBracket) {
            if self.consume(TokenKind::RBracket) {
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                self.expect_semicolon()?;
                return Ok(vec![EvalStmt::ArrayAppend { target, value }]);
            }
            nested = true;
            let nested_index = self.parse_expr()?;
            self.expect(TokenKind::RBracket)?;
            target = EvalExpr::ArrayGet {
                array: Box::new(target),
                index: Box::new(nested_index),
            };
        }
        if self.consume(TokenKind::QuestionQuestionEqual) {
            let default = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(vec![EvalStmt::Expr(EvalExpr::NullCoalesceAssign {
                target: Box::new(target),
                default: Box::new(default),
            })]);
        }
        if matches!(self.current(), TokenKind::Equal)
            && matches!(self.peek(), TokenKind::Ampersand)
        {
            self.advance();
            self.advance();
            let source = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(vec![EvalStmt::ArrayReferenceBind { target, source }]);
        }
        let Some(op) = assignment_op(self.current()) else {
            return Err(EvalParseError::UnexpectedToken);
        };
        self.advance();
        if op.is_none() && self.consume(TokenKind::Ampersand) {
            let source = self.parse_expr()?;
            self.expect_semicolon()?;
            return Ok(vec![EvalStmt::ArrayReferenceBind { target, source }]);
        }
        let value = self.parse_expr()?;
        self.expect_semicolon()?;
        if nested || op.is_some() {
            let expression = match op {
                Some(op) => EvalExpr::CompoundAssign {
                    target: Box::new(target),
                    op,
                    value: Box::new(value),
                },
                None => EvalExpr::Assign {
                    target: Box::new(target),
                    value: Box::new(value),
                },
            };
            return Ok(vec![EvalStmt::Expr(expression)]);
        }
        Ok(vec![EvalStmt::ArraySetVar { name, index, value }])
    }

    /// Parses `for (init; condition; update) { ... }`.
    pub(in crate::parser) fn parse_for_stmt(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        self.expect(TokenKind::LParen)?;
        let init = self.parse_for_init_clause()?;
        self.expect_semicolon()?;
        let condition = if matches!(self.current(), TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect_semicolon()?;
        let update = self.parse_for_update_clause()?;
        let body = self.parse_statement_body_or_alternative("endfor")?;
        Ok(vec![EvalStmt::For {
            init,
            condition,
            update,
            body,
        }])
    }

    /// Parses value, key-value, and array-destructuring `foreach` targets.
    pub(in crate::parser) fn parse_foreach_stmt(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        self.expect(TokenKind::LParen)?;
        let array = self.parse_expr()?;
        if !matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "as")) {
            return Err(EvalParseError::UnexpectedToken);
        }
        self.advance();
        let (first_value_name, first_targets, first_by_ref) = self.parse_foreach_value_target()?;
        let (key_name, value_name, targets, value_by_ref) = if matches!(self.current(), TokenKind::FatArrow) {
            self.advance();
            if first_targets.is_some() || first_by_ref {
                return Err(EvalParseError::ExpectedVariable);
            }
            let (value_name, targets, value_by_ref) = self.parse_foreach_value_target()?;
            (Some(first_value_name), value_name, targets, value_by_ref)
        } else {
            (None, first_value_name, first_targets, first_by_ref)
        };
        self.expect(TokenKind::RParen)?;
        let mut body = self.parse_statement_body_or_alternative("endforeach")?;
        if let Some(targets) = targets {
            body.insert(
                0,
                EvalStmt::ArrayDestructure {
                    targets,
                    value: EvalExpr::LoadVar(value_name.clone()),
                },
            );
        }
        Ok(vec![EvalStmt::Foreach {
            array,
            key_name,
            value_name,
            value_by_ref,
            body,
        }])
    }

    /// Parses a simple variable or a short-array destructuring foreach value target.
    fn parse_foreach_value_target(
        &mut self,
    ) -> Result<(String, Option<Vec<Option<String>>>, bool), EvalParseError> {
        let by_ref = self.consume(TokenKind::Ampersand);
        if let TokenKind::DollarIdent(value_name) = self.current() {
            let value_name = value_name.clone();
            self.advance();
            return Ok((value_name, None, by_ref));
        }
        if by_ref {
            return Err(EvalParseError::ExpectedVariable);
        }
        self.expect(TokenKind::LBracket)?;
        let mut targets = Vec::new();
        while !self.consume(TokenKind::RBracket) {
            if self.consume(TokenKind::Comma) {
                targets.push(None);
                continue;
            }
            let TokenKind::DollarIdent(name) = self.current() else {
                return Err(EvalParseError::ExpectedVariable);
            };
            targets.push(Some(name.clone()));
            self.advance();
            if self.consume(TokenKind::RBracket) {
                break;
            }
            self.expect(TokenKind::Comma)?;
        }
        if targets.is_empty() {
            return Err(EvalParseError::UnexpectedToken);
        }
        Ok(("\0elephc_foreach_destructure".to_string(), Some(targets), false))
    }
}
