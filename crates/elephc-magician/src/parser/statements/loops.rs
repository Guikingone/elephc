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
use std::sync::atomic::{AtomicUsize, Ordering};

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
                return Ok(vec![EvalStmt::ArrayAppendReferenceBind {
                    target: EvalExpr::LoadVar(name),
                    source,
                }]);
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
                // `$loops[$k][] = &$path;` BINDS a newly appended element rather than storing a
                // value in it. The `&` was never looked for here, so the append branch went
                // straight to `parse_expr`, which cannot start with an ampersand.
                if self.consume(TokenKind::Ampersand) {
                    let source = self.parse_reference_source_expr()?;
                    self.expect_semicolon()?;
                    return Ok(vec![EvalStmt::ArrayAppendReferenceBind { target, source }]);
                }
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
        let first = self.parse_foreach_value_target()?;
        let (key, value, value_by_ref) = if matches!(self.current(), TokenKind::FatArrow) {
            self.advance();
            if !matches!(first.slot, EvalForeachSlot::Name(_) | EvalForeachSlot::Lvalue(_))
                || first.by_ref
            {
                return Err(EvalParseError::ExpectedVariable);
            }
            let value = self.parse_foreach_value_target()?;
            (Some(first), value.slot, value.by_ref)
        } else {
            (None, first.slot, first.by_ref)
        };
        self.expect(TokenKind::RParen)?;
        // A foreach binds its key and value to NAMES. Anything else -- a pattern, a property, an
        // element -- is bound to a hidden name and copied into place by a statement prepended to
        // the body, which is what `[$a, $b]` already did and what `$this->k =>` now does too.
        let (value_name, value_prologue) = self.foreach_target_binding(value);
        let (key_name, key_prologue) = match key {
            Some(key) => {
                let (name, prologue) = self.foreach_target_binding(key.slot);
                (Some(name), prologue)
            }
            None => (None, None),
        };
        let mut body = self.parse_statement_body_or_alternative("endforeach")?;
        if let Some(prologue) = value_prologue {
            body.insert(0, prologue);
        }
        if let Some(prologue) = key_prologue {
            body.insert(0, prologue);
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

    /// Parses one foreach key or value target: a variable, a pattern, or any other lvalue.
    fn parse_foreach_value_target(&mut self) -> Result<EvalForeachTarget, EvalParseError> {
        let by_ref = self.consume(TokenKind::Ampersand);
        if let TokenKind::DollarIdent(value_name) = self.current() {
            if !matches!(
                self.peek(),
                TokenKind::Arrow | TokenKind::LBracket | TokenKind::DoubleColon
            ) {
                let value_name = value_name.clone();
                self.advance();
                return Ok(EvalForeachTarget {
                    slot: EvalForeachSlot::Name(value_name),
                    by_ref,
                });
            }
        }
        if by_ref {
            return Err(EvalParseError::ExpectedVariable);
        }
        if matches!(self.current(), TokenKind::LBracket) {
            return Ok(EvalForeachTarget {
                slot: EvalForeachSlot::Pattern(self.parse_destructure_pattern()?),
                by_ref,
            });
        }
        let target = self.parse_ternary()?;
        if !is_assignment_target(&target) {
            return Err(EvalParseError::ExpectedVariable);
        }
        Ok(EvalForeachTarget {
            slot: EvalForeachSlot::Lvalue(target),
            by_ref,
        })
    }

    /// Returns the name a foreach slot binds to, plus the statement that moves it into place.
    fn foreach_target_binding(&mut self, slot: EvalForeachSlot) -> (String, Option<EvalStmt>) {
        match slot {
            EvalForeachSlot::Name(name) => (name, None),
            EvalForeachSlot::Pattern(targets) => {
                let name = next_foreach_binding_name();
                (
                    name.clone(),
                    Some(EvalStmt::ArrayDestructure {
                        targets,
                        value: EvalExpr::LoadVar(name),
                    }),
                )
            }
            EvalForeachSlot::Lvalue(target) => {
                let name = next_foreach_binding_name();
                (
                    name.clone(),
                    Some(EvalStmt::Expr(EvalExpr::Assign {
                        target: Box::new(target),
                        value: Box::new(EvalExpr::LoadVar(name)),
                    })),
                )
            }
        }
    }
}

/// One parsed foreach key or value target.
pub(in crate::parser) struct EvalForeachTarget {
    slot: EvalForeachSlot,
    by_ref: bool,
}

/// What a foreach key or value target names.
pub(in crate::parser) enum EvalForeachSlot {
    /// A plain `$name`, which the loop can bind directly.
    Name(String),
    /// A `[...]` destructuring pattern.
    Pattern(Vec<Option<EvalDestructureTarget>>),
    /// Any other writable lvalue, such as `$this->k`.
    Lvalue(EvalExpr),
}

/// Returns a scope name for one hidden foreach binding.
///
/// The leading NUL byte cannot appear in a PHP variable name, so the binding is invisible to
/// user code.
fn next_foreach_binding_name() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("\0elephc_foreach_binding:{id}")
}
