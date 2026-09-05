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
use super::assignments::{plain_variable_destructure_targets, EvalDestructureElement};
use crate::parser::expressions::precedence::is_assignment_target;

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
        let (first_target, first_by_ref) = self.parse_foreach_target()?;
        let (key_target, value_target, value_by_ref) =
            if matches!(self.current(), TokenKind::FatArrow) {
                self.advance();
                if first_by_ref || matches!(first_target, EvalForeachTarget::Pattern(_)) {
                    return Err(self.fail(EvalParseError::ExpectedVariable));
                }
                let (value_target, value_by_ref) = self.parse_foreach_target()?;
                (Some(first_target), value_target, value_by_ref)
            } else {
                (None, first_target, first_by_ref)
            };
        self.expect(TokenKind::RParen)?;
        let mut body = self.parse_statement_body_or_alternative("endforeach")?;
        // Only a plain scope variable can be the name `EvalStmt::Foreach` binds. Every other
        // target — a property on the key side, a destructuring pattern on the value side — binds a
        // HIDDEN name instead and is copied into place by statements at the head of the body. That
        // is the shape foreach destructuring has always used here; this widens it to the key and
        // to element targets PHP accepts, `foreach ($map as $this->currentId => $definition)` in
        // `dependency-injection/Compiler/ResolveInvalidReferencesPass.php` above all.
        let value_name = match &value_target {
            EvalForeachTarget::Variable(name) => name.clone(),
            _ => FOREACH_VALUE_BINDING_NAME.to_string(),
        };
        let value_writes = self.foreach_target_writes(&value_target, &value_name)?;
        for statement in value_writes.into_iter().rev() {
            body.insert(0, statement);
        }
        let key_name = match key_target {
            None => None,
            Some(EvalForeachTarget::Variable(name)) => Some(name),
            Some(target) => {
                let hidden = FOREACH_KEY_BINDING_NAME.to_string();
                let writes = self.foreach_target_writes(&target, &hidden)?;
                for statement in writes.into_iter().rev() {
                    body.insert(0, statement);
                }
                Some(hidden)
            }
        };
        Ok(vec![EvalStmt::Foreach {
            array,
            key_name,
            value_name,
            value_by_ref,
            body,
        }])
    }

    /// Parses one `foreach` target: a scope variable, another lvalue, or a destructuring pattern.
    fn parse_foreach_target(&mut self) -> Result<(EvalForeachTarget, bool), EvalParseError> {
        let by_ref = self.consume(TokenKind::Ampersand);
        if let TokenKind::DollarIdent(name) = self.current() {
            if !matches!(
                self.peek(),
                TokenKind::Arrow | TokenKind::LBracket | TokenKind::DoubleColon
            ) {
                let name = name.clone();
                self.advance();
                return Ok((EvalForeachTarget::Variable(name), by_ref));
            }
        }
        if by_ref {
            return Err(EvalParseError::ExpectedVariable);
        }
        if matches!(self.current(), TokenKind::LBracket) {
            return Ok((
                EvalForeachTarget::Pattern(self.parse_destructure_pattern()?),
                false,
            ));
        }
        let target = self.parse_expr()?;
        if !is_assignment_target(&target) {
            return Err(self.fail(EvalParseError::ExpectedVariable));
        }
        Ok((EvalForeachTarget::Lvalue(target), false))
    }

    /// Returns the statements that copy one bound `foreach` name into its real target.
    fn foreach_target_writes(
        &mut self,
        target: &EvalForeachTarget,
        bound_name: &str,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        match target {
            EvalForeachTarget::Variable(_) => Ok(Vec::new()),
            EvalForeachTarget::Lvalue(target) => Ok(vec![EvalStmt::Expr(EvalExpr::Assign {
                target: Box::new(target.clone()),
                value: Box::new(EvalExpr::LoadVar(bound_name.to_string())),
            })]),
            EvalForeachTarget::Pattern(elements) => {
                if let Some(targets) = plain_variable_destructure_targets(elements) {
                    return Ok(vec![EvalStmt::ArrayDestructure {
                        targets,
                        value: EvalExpr::LoadVar(bound_name.to_string()),
                    }]);
                }
                let mut statements = Vec::new();
                self.push_destructure_element_writes(elements, bound_name, &mut statements)?;
                Ok(statements)
            }
        }
    }
}

/// The three shapes a `foreach` target takes.
enum EvalForeachTarget {
    /// A plain scope variable, which `EvalStmt::Foreach` binds by name.
    Variable(String),
    /// Any other assignable expression, copied out of a hidden name at the head of the body.
    Lvalue(EvalExpr),
    /// A destructuring pattern, lowered at the head of the body.
    Pattern(Vec<EvalDestructureElement>),
}

/// The scope name a `foreach` value target that is not a plain variable is bound to.
///
/// The leading NUL byte cannot appear in a PHP variable name, which is what keeps the binding out
/// of the visible scope. One fixed name is enough for any nesting: the statements that copy it
/// into the real target run at the HEAD of the body, before any inner loop can rebind it, and
/// nothing reads it afterwards.
const FOREACH_VALUE_BINDING_NAME: &str = "\0elephc_foreach_destructure";

/// The scope name a `foreach` key target that is not a plain variable is bound to.
const FOREACH_KEY_BINDING_NAME: &str = "\0elephc_foreach_key";
