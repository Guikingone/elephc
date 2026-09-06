//! Purpose:
//! Parses for clauses, variable stores, property/static assignments, and increment/decrement forms.
//!
//! Called from:
//! - Statement dispatch and for-loop clause parsing.
//!
//! Key details:
//! - Compound assignment and property targets lower directly into explicit EvalIR statement variants.

use super::*;
use crate::parser::expressions::precedence::is_assignment_target;
use std::sync::atomic::{AtomicUsize, Ordering};

impl Parser {
    /// Parses short array destructuring assignment into ordered optional variable targets.
    pub(in crate::parser) fn parse_array_destructure_stmt(
        &mut self,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let pattern = self.parse_destructure_pattern()?;
        self.expect(TokenKind::Equal)?;
        let value = self.parse_expr()?;
        self.expect_semicolon()?;
        if let Some(targets) = plain_variable_destructure_targets(&pattern) {
            return Ok(vec![EvalStmt::ArrayDestructure { targets, value }]);
        }
        let subject = next_destructure_subject_name();
        let mut statements = vec![EvalStmt::StoreVar {
            name: subject.clone(),
            value,
        }];
        self.push_destructure_element_writes(&pattern, &subject, &mut statements)?;
        Ok(statements)
    }

    /// Parses one PHP list-assignment pattern, holes and keys and nesting included.
    ///
    /// It cannot go through `parse_array_literal()`: a pattern admits a HOLE, `[$a, , $b] = $v`,
    /// which is not a value literal, and its elements are assignment targets rather than values.
    pub(super) fn parse_destructure_pattern(
        &mut self,
    ) -> Result<Vec<EvalDestructureElement>, EvalParseError> {
        self.expect(TokenKind::LBracket)?;
        let mut elements = Vec::new();
        loop {
            if self.consume(TokenKind::RBracket) {
                break;
            }
            if self.consume(TokenKind::Comma) {
                elements.push(EvalDestructureElement::Skip);
                continue;
            }
            elements.push(self.parse_destructure_element()?);
            if self.consume(TokenKind::RBracket) {
                break;
            }
            self.expect(TokenKind::Comma)?;
        }
        if elements.is_empty() {
            return Err(self.fail(EvalParseError::UnexpectedToken));
        }
        Ok(elements)
    }

    /// Parses one element of a destructuring pattern: an optional key, then a target or a nesting.
    fn parse_destructure_element(&mut self) -> Result<EvalDestructureElement, EvalParseError> {
        if matches!(self.current(), TokenKind::LBracket) {
            let elements = self.parse_destructure_pattern()?;
            return Ok(EvalDestructureElement::Nested { key: None, elements });
        }
        let first = self.parse_expr()?;
        if !self.consume(TokenKind::FatArrow) {
            return Ok(EvalDestructureElement::Target {
                key: None,
                target: first,
            });
        }
        if matches!(self.current(), TokenKind::LBracket) {
            let elements = self.parse_destructure_pattern()?;
            return Ok(EvalDestructureElement::Nested {
                key: Some(first),
                elements,
            });
        }
        let target = self.parse_expr()?;
        Ok(EvalDestructureElement::Target {
            key: Some(first),
            target,
        })
    }

    /// Appends one assignment per element of a destructuring pattern, recursing into nested ones.
    ///
    /// `EvalStmt::ArrayDestructure` names its targets by SCOPE NAME, which is every target the
    /// vendor tree used until `[$this->keys, $this->values] = $values;` in
    /// `symfony/cache/Adapter/PhpArrayAdapter.php`. PHP's list assignment takes any assignable
    /// expression, an explicit key and a nested pattern; lowering to one read of the subject plus
    /// one ordinary assignment per element reproduces all three. The subject is evaluated ONCE,
    /// into a name whose leading NUL no PHP variable can carry, which is what PHP guarantees and
    /// the convention the `foreach` destructuring target already used; each element becomes
    /// `EvalExpr::Assign`, which writes through any lvalue the general location machinery accepts.
    ///
    /// The positional index counts EVERY keyless element, holes included, because that is what
    /// `[$first, , $third] = [1, 2, 3]` means.
    pub(super) fn push_destructure_element_writes(
        &mut self,
        elements: &[EvalDestructureElement],
        subject: &str,
        statements: &mut Vec<EvalStmt>,
    ) -> Result<(), EvalParseError> {
        let mut position = 0i64;
        for element in elements {
            let key = match element {
                EvalDestructureElement::Skip => {
                    position += 1;
                    continue;
                }
                EvalDestructureElement::Target { key, .. }
                | EvalDestructureElement::Nested { key, .. } => match key {
                    Some(key) => key.clone(),
                    None => {
                        let key = EvalExpr::Const(EvalConst::Int(position));
                        position += 1;
                        key
                    }
                },
            };
            let read = EvalExpr::ArrayGet {
                array: Box::new(EvalExpr::LoadVar(subject.to_string())),
                index: Box::new(key),
            };
            match element {
                EvalDestructureElement::Skip => unreachable!("a hole continues above"),
                EvalDestructureElement::Nested { elements, .. } => {
                    let nested = next_destructure_subject_name();
                    statements.push(EvalStmt::StoreVar {
                        name: nested.clone(),
                        value: read,
                    });
                    self.push_destructure_element_writes(elements, &nested, statements)?;
                }
                EvalDestructureElement::Target { target, .. } => {
                    if !is_assignment_target(target) {
                        return Err(self.fail(EvalParseError::ExpectedVariable));
                    }
                    statements.push(EvalStmt::Expr(EvalExpr::Assign {
                        target: Box::new(target.clone()),
                        value: Box::new(read),
                    }));
                }
            }
        }
        Ok(())
    }

    /// Parses the optional first clause of a `for` loop.
    pub(in crate::parser) fn parse_for_init_clause(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        if matches!(self.current(), TokenKind::Semicolon) {
            return Ok(Vec::new());
        }
        self.parse_for_clause_stmt_list()
    }

    /// Parses the optional update clause of a `for` loop.
    pub(in crate::parser) fn parse_for_update_clause(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        if self.consume(TokenKind::RParen) {
            return Ok(Vec::new());
        }
        let statements = self.parse_for_clause_stmt_list()?;
        self.expect(TokenKind::RParen)?;
        Ok(statements)
    }

    /// Parses one comma-separated `for` clause list.
    ///
    /// PHP's grammar makes each of the three `for` clauses a `for_exprs` list, so
    /// `for ($i = 0, $n = 3; $i < $n; ++$i, --$n)` runs both elements of the first clause once and
    /// both elements of the third clause on every iteration, in source order. Each element lowers
    /// to its own statement, which is exactly the vector the `For` statement's init and update
    /// fields already hold.
    fn parse_for_clause_stmt_list(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        let mut statements = self.parse_for_clause_stmt()?;
        while self.consume(TokenKind::Comma) {
            statements.extend(self.parse_for_clause_stmt()?);
        }
        Ok(statements)
    }

    /// Parses one statement-like `for` clause without consuming a delimiter.
    pub(in crate::parser) fn parse_for_clause_stmt(&mut self) -> Result<Vec<EvalStmt>, EvalParseError> {
        match self.current() {
            TokenKind::PlusPlus | TokenKind::MinusMinus
                if self.current_starts_prefixed_static_property_inc_dec() =>
            {
                self.parse_static_property_inc_dec_stmt(true, false)
            }
            TokenKind::PlusPlus | TokenKind::MinusMinus
                if self.current_starts_prefixed_dynamic_static_property_inc_dec() =>
            {
                self.parse_dynamic_static_property_inc_dec_stmt(true, false)
            }
            TokenKind::PlusPlus | TokenKind::MinusMinus
                if self.current_starts_prefixed_property_inc_dec() =>
            {
                self.parse_prefixed_property_inc_dec_stmt(false)
            }
            TokenKind::PlusPlus | TokenKind::MinusMinus => {
                self.parse_prefix_inc_dec_stmt(false)
            }
            TokenKind::Ident(_) | TokenKind::Backslash
                if self.current_starts_static_property_postfix_inc_dec() =>
            {
                self.parse_static_property_inc_dec_stmt(false, false)
            }
            TokenKind::DollarIdent(name) if matches!(self.peek(), TokenKind::LBracket) => {
                self.parse_array_set_clause(name.clone())
            }
            TokenKind::DollarIdent(_)
                if self.current_starts_dynamic_static_property_postfix_inc_dec() =>
            {
                self.parse_dynamic_static_property_inc_dec_stmt(false, false)
            }
            TokenKind::DollarIdent(_) if matches!(self.peek(), TokenKind::Arrow) => {
                self.parse_property_stmt(false)
            }
            TokenKind::DollarIdent(name)
                if matches!(self.peek(), TokenKind::PlusPlus | TokenKind::MinusMinus) =>
            {
                self.parse_postfix_inc_dec_stmt(name.clone(), false)
            }
            TokenKind::DollarIdent(name) if assignment_op(self.peek()).is_some() => {
                let name = name.clone();
                self.parse_var_store_stmt(name, false)
            }
            _ => {
                let expr = self.parse_expr()?;
                self.parse_property_like_stmt_tail(expr, false)
            }
        }
    }

    /// Parses `$name[index] = expr`, `$name[] = expr`, and nested `??=` in a `for` clause.
    pub(in crate::parser) fn parse_array_set_clause(
        &mut self,
        name: String,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        self.expect(TokenKind::LBracket)?;
        if self.consume(TokenKind::RBracket) {
            self.expect(TokenKind::Equal)?;
            if self.consume(TokenKind::Ampersand) {
                let source = self.parse_reference_source_expr()?;
                return Ok(vec![EvalStmt::ArrayAppendReferenceBind { name, source }]);
            }
            let value = self.parse_expr()?;
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
            let source = self.parse_reference_source_expr()?;
            return Ok(vec![EvalStmt::ArrayReferenceBind { target, source }]);
        }
        let Some(op) = assignment_op(self.current()) else {
            return Err(EvalParseError::UnexpectedToken);
        };
        self.advance();
        if op.is_none() && self.consume(TokenKind::Ampersand) {
            let source = self.parse_reference_source_expr()?;
            return Ok(vec![EvalStmt::ArrayReferenceBind { target, source }]);
        }
        let value = self.parse_expr()?;
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

    /// Parses `$name = expr` and simple variable compound assignments.
    pub(in crate::parser) fn parse_var_store_stmt(
        &mut self,
        name: String,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        let Some(op) = assignment_op(self.current()) else {
            return Err(EvalParseError::UnexpectedToken);
        };
        self.advance();
        if op.is_none() && matches!(self.current(), TokenKind::Ampersand) {
            self.advance();
            let source = self.parse_reference_source_name()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return Ok(vec![match source {
                ReferenceSource::Variable(source) => EvalStmt::ReferenceAssign {
                    target: name,
                    source,
                },
                ReferenceSource::Lvalue(source) => EvalStmt::VarReferenceBind {
                    target: name,
                    source,
                },
            }]);
        }
        let value = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        let value = assignment_value(&name, op, value);
        Ok(vec![EvalStmt::StoreVar { name, value }])
    }

    /// Parses `Class::$property = expr` and simple static-property compound assignments.
    pub(in crate::parser) fn parse_static_property_set_stmt(
        &mut self,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let class_name = self.parse_qualified_name()?;
        let class_name = self.resolve_static_class_name(class_name);
        self.expect(TokenKind::DoubleColon)?;
        let TokenKind::DollarIdent(property) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let property = property.clone();
        self.advance();
        if self.consume(TokenKind::LBracket) {
            if self.consume(TokenKind::RBracket) {
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                if require_semicolon {
                    self.expect_semicolon()?;
                }
                return Ok(vec![EvalStmt::StaticPropertyArrayAppend {
                    class_name,
                    property,
                    value,
                }]);
            }
            let index = self.parse_expr()?;
            self.expect(TokenKind::RBracket)?;
            let Some(op) = assignment_op(self.current()) else {
                return Err(EvalParseError::UnexpectedToken);
            };
            self.advance();
            let value = self.parse_expr()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return Ok(vec![EvalStmt::StaticPropertyArraySet {
                class_name,
                property,
                index,
                op,
                value,
            }]);
        }
        let Some(op) = assignment_op(self.current()) else {
            return Err(EvalParseError::UnexpectedToken);
        };
        self.advance();
        if op.is_none() && self.consume(TokenKind::Ampersand) {
            let (mut stmts, source) = self.parse_reference_source_via_alias()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            stmts.push(EvalStmt::StaticPropertyReferenceBind {
                class_name,
                property,
                source,
            });
            return Ok(stmts);
        }
        let value = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        let value = match op {
            Some(op) => EvalExpr::Binary {
                op,
                left: Box::new(EvalExpr::StaticPropertyGet {
                    class_name: class_name.clone(),
                    property: property.clone(),
                }),
                right: Box::new(value),
            },
            None => value,
        };
        Ok(vec![EvalStmt::StaticPropertySet {
            class_name,
            property,
            value,
        }])
    }

    /// Parses `$class::$property = expr` and compound assignments with a dynamic receiver.
    pub(in crate::parser) fn parse_dynamic_static_property_set_stmt(
        &mut self,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let TokenKind::DollarIdent(class_name) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let class_name = EvalExpr::LoadVar(class_name.clone());
        self.advance();
        self.expect(TokenKind::DoubleColon)?;
        let TokenKind::DollarIdent(property) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let property = property.clone();
        self.advance();
        if self.consume(TokenKind::LBracket) {
            if self.consume(TokenKind::RBracket) {
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                if require_semicolon {
                    self.expect_semicolon()?;
                }
                return Ok(vec![EvalStmt::DynamicStaticPropertyArrayAppend {
                    class_name,
                    property,
                    value,
                }]);
            }
            let index = self.parse_expr()?;
            self.expect(TokenKind::RBracket)?;
            let Some(op) = assignment_op(self.current()) else {
                return Err(EvalParseError::UnexpectedToken);
            };
            self.advance();
            let value = self.parse_expr()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return Ok(vec![EvalStmt::DynamicStaticPropertyArraySet {
                class_name,
                property,
                index,
                op,
                value,
            }]);
        }
        let Some(op) = assignment_op(self.current()) else {
            return Err(EvalParseError::UnexpectedToken);
        };
        self.advance();
        if op.is_none() && self.consume(TokenKind::Ampersand) {
            let (mut stmts, source) = self.parse_reference_source_via_alias()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            stmts.push(EvalStmt::DynamicStaticPropertyReferenceBind {
                class_name,
                property,
                source,
            });
            return Ok(stmts);
        }
        let value = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        let value = match op {
            Some(op) => EvalExpr::Binary {
                op,
                left: Box::new(EvalExpr::DynamicStaticPropertyGet {
                    class_name: Box::new(class_name.clone()),
                    property: property.clone(),
                }),
                right: Box::new(value),
            },
            None => value,
        };
        Ok(vec![EvalStmt::DynamicStaticPropertySet {
            class_name,
            property,
            value,
        }])
    }

    /// Parses static property increment/decrement as read-modify-write.
    pub(in crate::parser) fn parse_static_property_inc_dec_stmt(
        &mut self,
        prefixed: bool,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let prefix_increment = if prefixed {
            let increment = matches!(self.current(), TokenKind::PlusPlus);
            self.advance();
            Some(increment)
        } else {
            None
        };
        let class_name = self.parse_qualified_name()?;
        let class_name = self.resolve_static_class_name(class_name);
        self.expect(TokenKind::DoubleColon)?;
        let TokenKind::DollarIdent(property) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let property = property.clone();
        self.advance();
        let increment = if let Some(increment) = prefix_increment {
            increment
        } else {
            let increment = matches!(self.current(), TokenKind::PlusPlus);
            self.advance();
            increment
        };
        if require_semicolon {
            self.expect_semicolon()?;
        }
        Ok(vec![EvalStmt::StaticPropertyIncDec {
            class_name,
            property,
            increment,
        }])
    }

    /// Parses dynamic static property increment/decrement as read-modify-write.
    pub(in crate::parser) fn parse_dynamic_static_property_inc_dec_stmt(
        &mut self,
        prefixed: bool,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let prefix_increment = if prefixed {
            let increment = matches!(self.current(), TokenKind::PlusPlus);
            self.advance();
            Some(increment)
        } else {
            None
        };
        let TokenKind::DollarIdent(class_name) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let class_name = EvalExpr::LoadVar(class_name.clone());
        self.advance();
        self.expect(TokenKind::DoubleColon)?;
        let TokenKind::DollarIdent(property) = self.current() else {
            return Err(EvalParseError::ExpectedVariable);
        };
        let property = property.clone();
        self.advance();
        let increment = if let Some(increment) = prefix_increment {
            increment
        } else {
            let increment = matches!(self.current(), TokenKind::PlusPlus);
            self.advance();
            increment
        };
        if require_semicolon {
            self.expect_semicolon()?;
        }
        Ok(vec![EvalStmt::DynamicStaticPropertyIncDec {
            class_name,
            property,
            increment,
        }])
    }

    /// Parses prefix `++$name` / `--$name` and supported property-like prefix mutations.
    pub(in crate::parser) fn parse_prefix_inc_dec_stmt(
        &mut self,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let increment = matches!(self.current(), TokenKind::PlusPlus);
        self.advance();
        if let TokenKind::DollarIdent(name) = self.current() {
            if !matches!(
                self.peek(),
                TokenKind::DoubleColon
                    | TokenKind::Arrow
                    | TokenKind::QuestionArrow
                    | TokenKind::LBracket
            ) {
                let name = name.clone();
                self.advance();
                if require_semicolon {
                    self.expect_semicolon()?;
                }
                return Ok(vec![inc_dec_store(name, increment)]);
            }
        }
        let target = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        self.inc_dec_stmt_for_target(target, increment)
    }

    /// Lowers one parsed increment/decrement target to a statement.
    ///
    /// A PROPERTY target keeps its dedicated statement. Anything else -- `++$a["k"]["c"];` and
    /// `++$this->m["k"];` write through an array ELEMENT, not through a property -- had no
    /// statement at all and was refused. The semicolon had already been consumed by then, so the
    /// diagnostic named the NEXT statement's first token, and because the parser records only
    /// its FIRST failure position across backtracking the reported line could belong to another
    /// statement entirely. An element target becomes the same read-modify-write expression
    /// `parse_prefix_inc_dec_expr` builds, so the statement and expression spellings agree by
    /// construction rather than by coincidence.
    fn inc_dec_stmt_for_target(
        &mut self,
        target: EvalExpr,
        increment: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        match property_inc_dec_stmt(target.clone(), increment) {
            Ok(stmt) => Ok(vec![stmt]),
            Err(_) if is_assignment_target(&target) => {
                Ok(vec![EvalStmt::Expr(EvalExpr::CompoundAssign {
                    target: Box::new(target),
                    op: if increment {
                        EvalBinOp::Add
                    } else {
                        EvalBinOp::Sub
                    },
                    value: Box::new(EvalExpr::Const(EvalConst::Int(1))),
                })])
            }
            Err(error) => Err(error),
        }
    }

    /// Parses postfix `$name++` and `$name--` as simple statement effects.
    pub(in crate::parser) fn parse_postfix_inc_dec_stmt(
        &mut self,
        name: String,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        self.advance();
        let increment = matches!(self.current(), TokenKind::PlusPlus);
        self.advance();
        if require_semicolon {
            self.expect_semicolon()?;
        }
        Ok(vec![inc_dec_store(name, increment)])
    }

    /// Parses prefix property increment/decrement as read-modify-write.
    pub(in crate::parser) fn parse_prefixed_property_inc_dec_stmt(
        &mut self,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let increment = matches!(self.current(), TokenKind::PlusPlus);
        self.advance();
        let target = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        self.inc_dec_stmt_for_target(target, increment)
    }

    /// Parses `$object->property` as either an expression statement or property write.
    pub(in crate::parser) fn parse_property_stmt(
        &mut self,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        let target = self.parse_expr()?;
        self.parse_property_like_stmt_tail(target, require_semicolon)
    }

    /// Parses assignment, array-write, or inc/dec tails after a parsed property-like target.
    pub(super) fn parse_property_like_stmt_tail(
        &mut self,
        target: EvalExpr,
        require_semicolon: bool,
    ) -> Result<Vec<EvalStmt>, EvalParseError> {
        // The expression parser now recognises `TARGET[] = value`, so a WHOLE-STATEMENT append
        // arrives here already folded into one expression instead of leaving the `[` for the
        // loop below. A statement append keeps its dedicated statement: `PropertyArrayAppend`
        // and its dynamic and static siblings run through the property-aware read/write helpers
        // (`eval_property_get_result` / `eval_property_set_result`), which enforce visibility
        // and consult the dynamic-property overlay. Unfolding it back is what keeps that path,
        // and every expectation that names those statements, exactly as it was.
        if let EvalExpr::ArrayAppendAssign {
            target: append_target,
            value,
        } = target
        {
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return match property_array_append_stmt((*append_target).clone(), (*value).clone()) {
                Ok(stmt) => Ok(vec![stmt]),
                // A target with no dedicated statement -- `$this->rows["k"][] = 1;` writes
                // through an ARRAY ELEMENT, not through a property -- keeps the expression.
                Err(_) => Ok(vec![EvalStmt::Expr(EvalExpr::ArrayAppendAssign {
                    target: append_target,
                    value,
                })]),
            };
        }
        if matches!(self.current(), TokenKind::PlusPlus | TokenKind::MinusMinus) {
            let increment = matches!(self.current(), TokenKind::PlusPlus);
            self.advance();
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return property_inc_dec_stmt(target, increment).map(|stmt| vec![stmt]);
        }
        if self.consume(TokenKind::LBracket) {
            if self.consume(TokenKind::RBracket) {
                self.expect(TokenKind::Equal)?;
                let value = self.parse_expr()?;
                if require_semicolon {
                    self.expect_semicolon()?;
                }
                return property_array_append_stmt(target, value).map(|stmt| vec![stmt]);
            }
            let index = self.parse_expr()?;
            self.expect(TokenKind::RBracket)?;
            // `$this->errorCount[$key] ??= 0;` — `??=` is not in `assignment_op()`, which lists the
            // operators that lower to a binary op, and it lowers to its own expression instead.
            if self.consume(TokenKind::QuestionQuestionEqual) {
                let default = self.parse_expr()?;
                if require_semicolon {
                    self.expect_semicolon()?;
                }
                return Ok(vec![EvalStmt::Expr(EvalExpr::NullCoalesceAssign {
                    target: Box::new(EvalExpr::ArrayGet {
                        array: Box::new(target),
                        index: Box::new(index),
                    }),
                    default: Box::new(default),
                })]);
            }
            let Some(op) = assignment_op(self.current()) else {
                return Err(EvalParseError::UnexpectedToken);
            };
            self.advance();
            let value = self.parse_expr()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return property_array_set_stmt(target, index, op, value).map(|stmt| vec![stmt]);
        }
        let Some(op) = assignment_op(self.current()) else {
            if require_semicolon {
                self.expect_semicolon()?;
            }
            return Ok(vec![EvalStmt::Expr(target)]);
        };
        // A target that turns out not to be assignable is only discovered after the whole
        // right-hand side has been read, so remember the operator PHP names in that diagnostic.
        let operator_pos = self.pos;
        self.advance();
        if op.is_none() && self.consume(TokenKind::Ampersand) {
            let (mut stmts, source) = self.parse_reference_source_via_alias()?;
            if require_semicolon {
                self.expect_semicolon()?;
            }
            stmts.push(property_reference_bind_stmt(target, source)?);
            return Ok(stmts);
        }
        let value = self.parse_expr()?;
        if require_semicolon {
            self.expect_semicolon()?;
        }
        match (target, op) {
            (EvalExpr::ArrayGet { array, index }, op) => {
                property_array_set_stmt(*array, *index, op, value).map(|stmt| vec![stmt])
            }
            (EvalExpr::PropertyGet { object, property }, None) => Ok(vec![EvalStmt::PropertySet {
                object: *object,
                property,
                value,
            }]),
            (EvalExpr::PropertyGet { object, property }, Some(op)) => {
                Ok(vec![EvalStmt::PropertyCompoundAssign {
                    object: *object,
                    property,
                    op,
                    value,
                }])
            }
            (EvalExpr::DynamicPropertyGet { object, property }, None) => {
                Ok(vec![EvalStmt::DynamicPropertySet {
                    object: *object,
                    property: *property,
                    value,
                }])
            }
            (EvalExpr::DynamicPropertyGet { object, property }, Some(op)) => {
                Ok(vec![EvalStmt::DynamicPropertyCompoundAssign {
                    object: *object,
                    property: *property,
                    op,
                    value,
                }])
            }
            (
                EvalExpr::DynamicStaticPropertyGet {
                    class_name,
                    property,
                },
                op,
            ) => {
                let class_name = *class_name;
                let value = match op {
                    Some(op) => EvalExpr::Binary {
                        op,
                        left: Box::new(EvalExpr::DynamicStaticPropertyGet {
                            class_name: Box::new(class_name.clone()),
                            property: property.clone(),
                        }),
                        right: Box::new(value),
                    },
                    None => value,
                };
                Ok(vec![EvalStmt::DynamicStaticPropertySet {
                    class_name,
                    property,
                    value,
                }])
            }
            (
                EvalExpr::DynamicStaticPropertyNameGet {
                    class_name,
                    property,
                },
                op,
            ) => {
                let class_name = *class_name;
                let property = *property;
                let value = match op {
                    Some(op) => EvalExpr::Binary {
                        op,
                        left: Box::new(EvalExpr::DynamicStaticPropertyNameGet {
                            class_name: Box::new(class_name.clone()),
                            property: Box::new(property.clone()),
                        }),
                        right: Box::new(value),
                    },
                    None => value,
                };
                Ok(vec![EvalStmt::DynamicStaticPropertyNameSet {
                    class_name,
                    property,
                    value,
                }])
            }
            _ => Err(self.fail_at(operator_pos, EvalParseError::UnexpectedToken)),
        }
    }

    /// Parses the source of a `= &` reference binding.
    ///
    /// PHP accepts any assignable expression after `=&`, not just a plain variable. The
    /// interpreter's reference machinery can alias every storage shape `eval_call_arg_value()`
    /// resolves to an `EvalReferenceTarget`: a variable, an array element (including a nested
    /// one), an object property, a static property, and their dynamic-name forms. Anything else
    /// — a call, a literal, an operator expression — names no storage, so it is reported as an
    /// unsupported construct instead of being lowered into a binding that would silently degrade
    /// to a by-value assignment.
    pub(in crate::parser) fn parse_reference_source_expr(
        &mut self,
    ) -> Result<EvalExpr, EvalParseError> {
        let source = self.parse_expr()?;
        if eval_expr_binds_a_reference(&source) {
            Ok(source)
        } else {
            Err(EvalParseError::UnsupportedConstruct)
        }
    }

    /// Parses a `= &` source and splits it into the plain-variable and general-lvalue cases.
    ///
    /// The variable-to-variable case stays on `EvalStmt::ReferenceAssign` because PHP aliases
    /// two scope names symmetrically there — writing either name updates both — which the
    /// scope's named-alias table models and a one-way `EvalReferenceTarget` write-back does not.
    fn parse_reference_source_name(&mut self) -> Result<ReferenceSource, EvalParseError> {
        match self.parse_reference_source_expr()? {
            EvalExpr::LoadVar(name) => Ok(ReferenceSource::Variable(name)),
            source => Ok(ReferenceSource::Lvalue(source)),
        }
    }

    /// Parses a `= &` source and lowers a non-variable one through a hidden binding variable.
    ///
    /// `PropertyReferenceBind` and its static-property siblings identify their source by scope
    /// name, and each resolves it through `scope.reference_target()` first. Binding the general
    /// lvalue to a hidden name first therefore hands those statements the exact reference target
    /// they need without teaching each one to evaluate an expression.
    fn parse_reference_source_via_alias(
        &mut self,
    ) -> Result<(Vec<EvalStmt>, String), EvalParseError> {
        match self.parse_reference_source_name()? {
            ReferenceSource::Variable(name) => Ok((Vec::new(), name)),
            ReferenceSource::Lvalue(source) => {
                let target = next_reference_binding_name();
                Ok((
                    vec![EvalStmt::VarReferenceBind {
                        target: target.clone(),
                        source,
                    }],
                    target,
                ))
            }
        }
    }
}

/// The two shapes a `= &` source lowers to.
enum ReferenceSource {
    /// A plain variable, aliased by name in the scope.
    Variable(String),
    /// Any other assignable expression, aliased through its reference target.
    Lvalue(EvalExpr),
}

/// Returns whether an expression names storage PHP can bind a reference to.
fn eval_expr_binds_a_reference(expr: &EvalExpr) -> bool {
    matches!(
        expr,
        EvalExpr::LoadVar(_)
            | EvalExpr::ArrayGet { .. }
            | EvalExpr::PropertyGet { .. }
            | EvalExpr::DynamicPropertyGet { .. }
            | EvalExpr::StaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyNameGet { .. }
    )
}

/// One element of a PHP list-assignment pattern, as written.
pub(super) enum EvalDestructureElement {
    /// A hole, `[$a, , $b] = $v`, which consumes a position and assigns nothing.
    Skip,
    /// An assignable expression, with the explicit key that selects its element when there is one.
    Target {
        key: Option<EvalExpr>,
        target: EvalExpr,
    },
    /// A pattern of its own, `[[$a, $b], $c] = $v`.
    Nested {
        key: Option<EvalExpr>,
        elements: Vec<EvalDestructureElement>,
    },
}

/// Returns the plain scope names of a destructuring pattern, or None when it holds anything else.
///
/// `EvalStmt::ArrayDestructure` names its targets by scope name and is kept for the shape it can
/// carry — `[$a, , $b] = $v` — because that statement evaluates the subject without a hidden
/// variable. Everything richer goes through the lowering above.
pub(super) fn plain_variable_destructure_targets(
    elements: &[EvalDestructureElement],
) -> Option<Vec<Option<String>>> {
    if elements.is_empty() {
        return None;
    }
    elements
        .iter()
        .map(|element| match element {
            EvalDestructureElement::Skip => Some(None),
            EvalDestructureElement::Target {
                key: None,
                target: EvalExpr::LoadVar(name),
            } => Some(Some(name.clone())),
            _ => None,
        })
        .collect()
}

/// Returns a scope name for one hidden destructuring subject.
///
/// The leading NUL byte cannot appear in a PHP variable name, so the subject is invisible to user
/// code, the same convention `next_reference_binding_name()` and the `foreach` destructuring
/// target already use.
fn next_destructure_subject_name() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("\0elephc_destructure_subject:{id}")
}

/// Returns a scope name for one hidden reference binding.
///
/// The leading NUL byte cannot appear in a PHP variable name, so the binding is invisible to
/// user code the way `eval_property_reference_alias_name()` already keeps property aliases out
/// of the visible scope.
fn next_reference_binding_name() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("\0elephc_reference_source:{id}")
}
