//! Purpose:
//! Parses PHP expression precedence from keyword logical operators through
//! unary operators and scalar casts.
//!
//! Called from:
//! - `Parser::parse_expr()` and the next tighter parser precedence layer.
//!
//! Key details:
//! - Ternary, coalesce, exponentiation handoff, and PHP keyword precedence are
//!   kept explicit.

use super::*;

impl Parser {
    /// Parses an expression using PHP-like logical, comparison, concatenation, and arithmetic precedence.
    pub(in crate::parser) fn parse_expr(&mut self) -> Result<EvalExpr, EvalParseError> {
        self.parse_keyword_or()
    }

    /// Parses PHP keyword `or`, whose precedence is lower than `xor`, `and`, and ternary.
    pub(in crate::parser) fn parse_keyword_or(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_keyword_xor()?;
        while matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "or")) {
            self.advance();
            let right = self.parse_keyword_xor()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::LogicalOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses PHP keyword `xor`, whose operands are evaluated before boolean XOR.
    pub(in crate::parser) fn parse_keyword_xor(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_keyword_and()?;
        while matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "xor")) {
            self.advance();
            let right = self.parse_keyword_and()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::LogicalXor,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses PHP keyword `and`, whose precedence is lower than ternary and `&&`.
    pub(in crate::parser) fn parse_keyword_and(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_assignment()?;
        while matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "and")) {
            self.advance();
            let right = self.parse_assignment()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::LogicalAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses supported right-associative assignment expressions.
    pub(in crate::parser) fn parse_assignment(&mut self) -> Result<EvalExpr, EvalParseError> {
        let target = self.parse_ternary()?;
        let array_destructure_targets = short_array_destructure_targets(&target);
        let negated_assignment_target = negated_assignment_target(&target);
        let nested_assignment_target = nested_assignment_target(&target);
        let null_coalescing = self.consume(TokenKind::QuestionQuestionEqual);
        let assignment = if null_coalescing {
            None
        } else {
            assignment_op(self.current())
        };
        if !null_coalescing && assignment.is_none() {
            return Ok(target);
        }
        if !is_assignment_target(&target)
            && array_destructure_targets.is_none()
            && negated_assignment_target.is_none()
            && !nested_assignment_target
        {
            return Err(EvalParseError::UnexpectedToken);
        }
        if array_destructure_targets.is_some() && (null_coalescing || assignment != Some(None)) {
            return Err(EvalParseError::UnexpectedToken);
        }
        if null_coalescing {
            let default = self.parse_assignment()?;
            if nested_assignment_target {
                return nested_assignment_expr(&target, true, assignment, default)
                    .ok_or(EvalParseError::UnexpectedToken);
            }
            return Ok(EvalExpr::NullCoalesceAssign {
                target: Box::new(target),
                default: Box::new(default),
            });
        }
        if assignment == Some(None)
            && matches!(self.peek(), TokenKind::Ampersand)
            && is_property_reference_target(&target)
        {
            return Ok(target);
        }
        // `$name = &<lvalue>` where the result is used. PHP's `=` takes a `&` source wherever an
        // assignment is an expression, and `symfony/config/Resource/ClassExistenceResource.php`
        // writes `if (null !== $exists = &self::$existsCache[$this->resource])` — the file that
        // holds `throwOnRequiredClass`, the loader every Symfony `class_exists()` reaches.
        if assignment == Some(None) && matches!(self.peek(), TokenKind::Ampersand) {
            if let Some(name) = reference_bind_assign_name(&target, nested_assignment_target) {
                self.advance();
                self.advance();
                let source = self.parse_reference_source_expr()?;
                let bind = EvalExpr::ReferenceBindAssign {
                    target: name,
                    source: Box::new(source),
                };
                return if nested_assignment_target {
                    nested_assignment_replacement(&target, bind)
                        .ok_or(EvalParseError::UnexpectedToken)
                } else {
                    Ok(bind)
                };
            }
        }
        self.advance();
        let value = self.parse_assignment()?;
        if let Some(targets) = array_destructure_targets {
            return Ok(EvalExpr::ArrayDestructureAssign {
                targets,
                value: Box::new(value),
            });
        }
        if let Some(target) = negated_assignment_target {
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::LogicalNot,
                expr: Box::new(EvalExpr::Assign {
                    target: Box::new(target),
                    value: Box::new(value),
                }),
            });
        }
        if nested_assignment_target {
            return nested_assignment_expr(&target, false, assignment, value)
                .ok_or(EvalParseError::UnexpectedToken);
        }
        Ok(match assignment.expect("assignment operator was checked") {
            Some(op) => EvalExpr::CompoundAssign {
                target: Box::new(target),
                op,
                value: Box::new(value),
            },
            None => EvalExpr::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
        })
    }

    /// Parses PHP ternary expressions, including the short `expr ?: fallback` form.
    pub(in crate::parser) fn parse_ternary(&mut self) -> Result<EvalExpr, EvalParseError> {
        let condition = self.parse_null_coalesce()?;
        if !self.consume(TokenKind::Question) {
            return Ok(condition);
        }
        let then_branch = if self.consume(TokenKind::Colon) {
            None
        } else {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Colon)?;
            Some(Box::new(expr))
        };
        let else_branch = self.parse_expr()?;
        Ok(EvalExpr::Ternary {
            condition: Box::new(condition),
            then_branch,
            else_branch: Box::new(else_branch),
        })
    }

    /// Parses right-associative null coalescing below logical OR and above ternary.
    pub(in crate::parser) fn parse_null_coalesce(&mut self) -> Result<EvalExpr, EvalParseError> {
        let value = self.parse_logical_or()?;
        if !self.consume(TokenKind::QuestionQuestion) {
            return Ok(value);
        }
        let default = self.parse_null_coalesce()?;
        Ok(EvalExpr::NullCoalesce {
            value: Box::new(value),
            default: Box::new(default),
        })
    }

    /// Parses left-associative logical OR with lower precedence than logical AND.
    pub(in crate::parser) fn parse_logical_or(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_logical_and()?;
        while self.consume(TokenKind::OrOr) {
            let right = self.parse_logical_and()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::LogicalOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative logical AND with lower precedence than equality.
    pub(in crate::parser) fn parse_logical_and(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_bit_or()?;
        while self.consume(TokenKind::AndAnd) {
            let right = self.parse_bit_or()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::LogicalAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative bitwise OR with lower precedence than bitwise XOR.
    pub(in crate::parser) fn parse_bit_or(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_bit_xor()?;
        while self.consume(TokenKind::Pipe) {
            let right = self.parse_bit_xor()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::BitOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative bitwise XOR with lower precedence than bitwise AND.
    pub(in crate::parser) fn parse_bit_xor(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_bit_and()?;
        while self.consume(TokenKind::Caret) {
            let right = self.parse_bit_and()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::BitXor,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative bitwise AND with lower precedence than equality.
    pub(in crate::parser) fn parse_bit_and(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_equality()?;
        while self.consume(TokenKind::Ampersand) {
            let right = self.parse_equality()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::BitAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative equality and inequality comparisons.
    pub(in crate::parser) fn parse_equality(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_ordering()?;
        loop {
            let op = if self.consume(TokenKind::EqualEqual) {
                EvalBinOp::LooseEq
            } else if self.consume(TokenKind::NotEqual) {
                EvalBinOp::LooseNotEq
            } else if self.consume(TokenKind::EqualEqualEqual) {
                EvalBinOp::StrictEq
            } else if self.consume(TokenKind::NotEqualEqual) {
                EvalBinOp::StrictNotEq
            } else {
                break;
            };
            let right = self.parse_ordering()?;
            expr = EvalExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative ordered comparisons.
    pub(in crate::parser) fn parse_ordering(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_shift()?;
        loop {
            let op = if self.consume(TokenKind::Less) {
                EvalBinOp::Lt
            } else if self.consume(TokenKind::LessEqual) {
                EvalBinOp::LtEq
            } else if self.consume(TokenKind::Greater) {
                EvalBinOp::Gt
            } else if self.consume(TokenKind::GreaterEqual) {
                EvalBinOp::GtEq
            } else if self.consume(TokenKind::Spaceship) {
                EvalBinOp::Spaceship
            } else {
                break;
            };
            let right = self.parse_shift()?;
            expr = EvalExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative integer shift operators.
    pub(in crate::parser) fn parse_shift(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_concat()?;
        loop {
            let op = if self.consume(TokenKind::LessLess) {
                EvalBinOp::ShiftLeft
            } else if self.consume(TokenKind::GreaterGreater) {
                EvalBinOp::ShiftRight
            } else {
                break;
            };
            let right = self.parse_concat()?;
            expr = EvalExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative string concatenation.
    pub(in crate::parser) fn parse_concat(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_add()?;
        while self.consume(TokenKind::Dot) {
            let right = self.parse_add()?;
            expr = EvalExpr::Binary {
                op: EvalBinOp::Concat,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative numeric addition and subtraction.
    pub(in crate::parser) fn parse_add(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_mul()?;
        loop {
            let op = if self.consume(TokenKind::Plus) {
                EvalBinOp::Add
            } else if self.consume(TokenKind::Minus) {
                EvalBinOp::Sub
            } else {
                break;
            };
            let right = self.parse_mul()?;
            expr = EvalExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses left-associative numeric multiplication, division, and modulo.
    pub(in crate::parser) fn parse_mul(&mut self) -> Result<EvalExpr, EvalParseError> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = if self.consume(TokenKind::Star) {
                EvalBinOp::Mul
            } else if self.consume(TokenKind::Slash) {
                EvalBinOp::Div
            } else if self.consume(TokenKind::Percent) {
                EvalBinOp::Mod
            } else {
                break;
            };
            let right = self.parse_unary()?;
            expr = EvalExpr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    /// Parses right-associative unary prefix expressions.
    pub(in crate::parser) fn parse_unary(&mut self) -> Result<EvalExpr, EvalParseError> {
        if let Some(target) = self.peek_scalar_cast_type() {
            self.advance();
            self.advance();
            self.advance();
            let expr = self.parse_concat()?;
            return Ok(EvalExpr::Cast {
                target,
                expr: Box::new(expr),
            });
        }
        if matches!(self.current(), TokenKind::Ident(name) if ident_eq(name, "clone")) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Clone(Box::new(expr)));
        }
        if self.consume(TokenKind::Plus) {
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::Plus,
                expr: Box::new(expr),
            });
        }
        if self.consume(TokenKind::PlusPlus) {
            return self.parse_prefix_inc_dec_expr(true);
        }
        if self.consume(TokenKind::MinusMinus) {
            return self.parse_prefix_inc_dec_expr(false);
        }
        if self.consume(TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::Negate,
                expr: Box::new(expr),
            });
        }
        if self.consume(TokenKind::Bang) {
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::LogicalNot,
                expr: Box::new(expr),
            });
        }
        if self.consume(TokenKind::Tilde) {
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::BitNot,
                expr: Box::new(expr),
            });
        }
        if self.consume(TokenKind::At) {
            let expr = self.parse_unary()?;
            return Ok(EvalExpr::Unary {
                op: EvalUnaryOp::ErrorSuppress,
                expr: Box::new(expr),
            });
        }
        self.parse_instanceof()
    }

    /// Parses a prefix increment or decrement expression and returns its updated value.
    fn parse_prefix_inc_dec_expr(&mut self, increment: bool) -> Result<EvalExpr, EvalParseError> {
        let target = self.parse_unary()?;
        if !is_assignment_target(&target) {
            return Err(EvalParseError::UnexpectedToken);
        }
        Ok(EvalExpr::CompoundAssign {
            target: Box::new(target),
            op: if increment {
                EvalBinOp::Add
            } else {
                EvalBinOp::Sub
            },
            value: Box::new(EvalExpr::Const(EvalConst::Int(1))),
        })
    }

    /// Returns the cast target represented by the current `(type)` token window.
    pub(super) fn peek_scalar_cast_type(&self) -> Option<EvalCastType> {
        if !matches!(self.current(), TokenKind::LParen) {
            return None;
        }
        let Some(TokenKind::Ident(name)) = self.tokens.get(self.pos + 1) else {
            return None;
        };
        if !matches!(self.tokens.get(self.pos + 2), Some(TokenKind::RParen)) {
            return None;
        }
        if ident_eq(name, "int") || ident_eq(name, "integer") {
            Some(EvalCastType::Int)
        } else if ident_eq(name, "float") || ident_eq(name, "double") || ident_eq(name, "real") {
            Some(EvalCastType::Float)
        } else if ident_eq(name, "string") {
            Some(EvalCastType::String)
        } else if ident_eq(name, "bool") || ident_eq(name, "boolean") {
            Some(EvalCastType::Bool)
        } else if ident_eq(name, "array") {
            Some(EvalCastType::Array)
        } else {
            None
        }
    }

}

/// Extracts positional variable targets from a short-array destructuring assignment lhs.
fn short_array_destructure_targets(target: &EvalExpr) -> Option<Vec<Option<String>>> {
    let EvalExpr::Array(elements) = target else {
        return None;
    };
    elements
        .iter()
        .map(|element| match element {
            EvalArrayElement::Value(EvalExpr::LoadVar(name)) => Some(Some(name.clone())),
            _ => None,
        })
        .collect()
}

/// Returns whether one expression is a regular PHP assignment lvalue in the EvalIR subset.
pub(in crate::parser) fn is_assignment_target(target: &EvalExpr) -> bool {
    matches!(
        target,
        EvalExpr::LoadVar(_)
            | EvalExpr::ArrayGet { .. }
            | EvalExpr::PropertyGet { .. }
            | EvalExpr::DynamicPropertyGet { .. }
            | EvalExpr::StaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyNameGet { .. }
    )
}

/// Returns whether a lvalue is handled by the statement-level property reference binder.
fn is_property_reference_target(target: &EvalExpr) -> bool {
    matches!(
        target,
        EvalExpr::PropertyGet { .. }
            | EvalExpr::DynamicPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyGet { .. }
            | EvalExpr::DynamicStaticPropertyNameGet { .. }
            // An array element of any writable chain, `$this->data[$key] = &$rows;` above all.
            // Leaving the `= &` to the statement tail is what routes it to
            // `property_reference_bind_stmt`, which builds the `ArrayReferenceBind` that writes
            // through a whole element path; reading the `&` here instead would try to parse it as
            // the start of a value.
            | EvalExpr::ArrayGet { .. }
    )
}

/// Extracts the lvalue assigned before PHP applies a leading logical negation.
fn negated_assignment_target(target: &EvalExpr) -> Option<EvalExpr> {
    let EvalExpr::Unary {
        op: EvalUnaryOp::LogicalNot,
        expr,
    } = target
    else {
        return None;
    };
    is_assignment_target(expr).then(|| expr.as_ref().clone())
}

/// Returns the scope name a `= &` binding writes to when the assignment is used as a value.
///
/// Only a plain variable can be REBOUND as a reference: PHP aliases two scope names, which is
/// what `EvalStmt::VarReferenceBind` models and what `EvalExpr::ReferenceBindAssign` reuses. When
/// the assignment sits inside a larger expression — `null !== $exists = &…`, the shape PHP's
/// grammar produces because the left side of `=` must be a variable — the variable is the
/// rightmost assignable child, the same one `nested_assignment_target()` found.
fn reference_bind_assign_name(target: &EvalExpr, nested: bool) -> Option<String> {
    match rightmost_assignment_target(target, nested)? {
        EvalExpr::LoadVar(name) => Some(name.clone()),
        _ => None,
    }
}

/// Returns the assignable expression a nested assignment actually writes to.
fn rightmost_assignment_target(target: &EvalExpr, nested: bool) -> Option<&EvalExpr> {
    if !nested {
        return is_assignment_target(target).then_some(target);
    }
    if let EvalExpr::Unary {
        op: EvalUnaryOp::LogicalNot,
        expr,
    } = target
    {
        if is_assignment_target(expr) {
            return Some(expr.as_ref());
        }
    }
    let right = match target {
        EvalExpr::Binary { right, .. } | EvalExpr::NullCoalesce { default: right, .. } => right,
        _ => return None,
    };
    if is_assignment_target(right) {
        return Some(right.as_ref());
    }
    rightmost_assignment_target(right, true)
}

/// Rebuilds a nested-assignment target with its rightmost writable child replaced outright.
///
/// `nested_assignment_expr()` builds the replacement itself from an operator and a value; a
/// reference binding is neither, so it is handed in already built.
fn nested_assignment_replacement(target: &EvalExpr, replacement: EvalExpr) -> Option<EvalExpr> {
    if negated_assignment_target(target).is_some() {
        return Some(EvalExpr::Unary {
            op: EvalUnaryOp::LogicalNot,
            expr: Box::new(replacement),
        });
    }
    if let EvalExpr::NullCoalesce { value, default } = target {
        if is_assignment_target(default) {
            return Some(EvalExpr::NullCoalesce {
                value: Box::new(value.as_ref().clone()),
                default: Box::new(replacement),
            });
        }
        return nested_assignment_replacement(default, replacement).map(|default| {
            EvalExpr::NullCoalesce {
                value: Box::new(value.as_ref().clone()),
                default: Box::new(default),
            }
        });
    }
    let EvalExpr::Binary { op, left, right } = target else {
        return None;
    };
    if is_assignment_target(right) {
        return Some(EvalExpr::Binary {
            op: *op,
            left: Box::new(left.as_ref().clone()),
            right: Box::new(replacement),
        });
    }
    nested_assignment_replacement(right, replacement).map(|right| EvalExpr::Binary {
        op: *op,
        left: Box::new(left.as_ref().clone()),
        right: Box::new(right),
    })
}

/// Returns whether an assignment belongs to a nested lvalue on a binary expression's right edge.
///
/// PHP parses `$prefix.$suffix ??= value` as `$prefix.($suffix ??= value)`, even though the
/// completed concatenation itself is not writable. Apply the same right-edge recovery to every
/// binary operator so compound assignments retain PHP's lvalue association consistently.
///
/// `??` IS THAT SAME RIGHT EDGE, and it is not a `Binary`. PHP's assignment binds looser than
/// `??`, so `$a ?? $a = 5` could only mean `($a ?? $a) = 5`, which is not derivable because the
/// left of `=` must be a variable — bison therefore reduces `$a ?? ($a = 5)`, and php prints
/// `55`. Five Symfony files depend on it, four as `$x ?? $x = …` and Container.php as
/// `$this->factories[$id] ?? self::$make ??= self::make(...)`.
fn nested_assignment_target(target: &EvalExpr) -> bool {
    if negated_assignment_target(target).is_some() {
        return true;
    }
    let right = match target {
        EvalExpr::Binary { right, .. } | EvalExpr::NullCoalesce { default: right, .. } => right,
        _ => return false,
    };
    is_assignment_target(right) || nested_assignment_target(right)
}

/// Rebuilds an expression with its rightmost writable child replaced by an assignment.
fn nested_assignment_expr(
    target: &EvalExpr,
    null_coalescing: bool,
    assignment: Option<Option<EvalBinOp>>,
    value: EvalExpr,
) -> Option<EvalExpr> {
    if let Some(target) = negated_assignment_target(target) {
        return Some(EvalExpr::Unary {
            op: EvalUnaryOp::LogicalNot,
            expr: Box::new(nested_assignment_value(
                target,
                null_coalescing,
                assignment,
                value,
            )),
        });
    }
    if let EvalExpr::NullCoalesce {
        value: probed,
        default,
    } = target
    {
        if is_assignment_target(default) {
            return Some(EvalExpr::NullCoalesce {
                value: Box::new(probed.as_ref().clone()),
                default: Box::new(nested_assignment_value(
                    default.as_ref().clone(),
                    null_coalescing,
                    assignment,
                    value,
                )),
            });
        }
        return nested_assignment_expr(default, null_coalescing, assignment, value).map(|default| {
            EvalExpr::NullCoalesce {
                value: Box::new(probed.as_ref().clone()),
                default: Box::new(default),
            }
        });
    }
    let EvalExpr::Binary { op, left, right } = target else {
        return None;
    };
    if is_assignment_target(right) {
        return Some(EvalExpr::Binary {
            op: *op,
            left: Box::new(left.as_ref().clone()),
            right: Box::new(nested_assignment_value(
                right.as_ref().clone(),
                null_coalescing,
                assignment,
                value,
            )),
        });
    }
    nested_assignment_expr(right, null_coalescing, assignment, value).map(|right| {
        EvalExpr::Binary {
            op: *op,
            left: Box::new(left.as_ref().clone()),
            right: Box::new(right),
        }
    })
}

/// Builds the value expression stored in one recovered assignment target.
fn nested_assignment_value(
    target: EvalExpr,
    null_coalescing: bool,
    assignment: Option<Option<EvalBinOp>>,
    value: EvalExpr,
) -> EvalExpr {
    if null_coalescing {
        return EvalExpr::NullCoalesceAssign {
            target: Box::new(target),
            default: Box::new(value),
        };
    }
    match assignment.expect("assignment operator was checked") {
        Some(op) => EvalExpr::CompoundAssign {
            target: Box::new(target),
            op,
            value: Box::new(value),
        },
        None => EvalExpr::Assign {
            target: Box::new(target),
            value: Box::new(value),
        },
    }
}
