//! Purpose:
//! Detects and lowers postfix assignments for complex expression targets.
//! Replays parseable l-values and creates effect-preserving lowerings for property/static assignments.
//!
//! Called from:
//! - `crate::parser::stmt::simple::parse_expr_stmt()` and assignment statement dispatch.
//!
//! Key details:
//! - Complex target lowering must not duplicate side effects while preserving PHP source evaluation order.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{
    BinOp, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind, NESTED_APPEND_TEMP_PREFIX,
};
use crate::parser::expr::{parse_assignment_value_expr, parse_expr};
use crate::span::Span;

use super::super::expect_semicolon;
use super::compound::{
    assignment_operator, assignment_value, is_valid_reference_source, AssignmentOperator,
};

/// Parses a postfix assignment where the target involves property access, array access,
/// or other complex expressions. Detects `+=` append style via `[]` in the target.
/// Returns the lowered `StmtKind` directly for simple targets, or synthesizes a
/// temporary-variable sequence for effectful (compound operator) targets that cannot
/// be replayed safely.
/// Returns `Ok(None)` if the token range does not contain a postfix assignment pattern.
pub(in crate::parser::stmt) fn try_parse_postfix_assignment(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Option<Stmt>, CompileError> {
    let start = *pos;
    let Some((assign_pos, op)) = find_top_level_assignment(tokens, start) else {
        return Ok(None);
    };
    if assign_pos < start + 3 {
        return Ok(None);
    }

    let lhs = &tokens[start..assign_pos];
    let is_append = lhs.len() >= 3
        && lhs[lhs.len() - 2].0 == Token::LBracket
        && lhs[lhs.len() - 1].0 == Token::RBracket;
    if is_append && op != AssignmentOperator::Assign {
        return Err(CompileError::new(span, "Invalid assignment target"));
    }
    let contains_postfix = lhs
        .iter()
        .skip(1)
        .any(|(token, _)| matches!(token, Token::Arrow | Token::QuestionArrow | Token::LBracket));
    if !contains_postfix {
        return Ok(None);
    }

    let mut lhs_pos = 0;
    let lhs_expr_tokens = if is_append {
        &lhs[..lhs.len() - 2]
    } else {
        lhs
    };
    let lhs_expr = parse_expr(lhs_expr_tokens, &mut lhs_pos)?;
    if lhs_pos != lhs_expr_tokens.len() {
        return Err(CompileError::new(span, "Invalid assignment target"));
    }

    *pos = assign_pos + 1;
    if op == AssignmentOperator::Assign
        && matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Ampersand))
    {
        return parse_complex_ref_assignment(lhs_expr, is_append, tokens, pos, span).map(Some);
    }
    let rhs = parse_assignment_value_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    if op != AssignmentOperator::Assign && !can_replay_assignment_target(&lhs_expr) {
        return lower_effectful_postfix_assignment(lhs_expr, op, rhs, span).map(Some);
    }
    // A compound write reads its element at store time too, so a right-hand side that
    // reassigns a plain-variable index decides which element is READ as well as which is
    // written: `$c = [10, 20]; $k = 0; $c[$k] += ($k = 1);` leaves `[10, 21]` in PHP. The
    // desugar below embeds that read INSIDE the right-hand side, where it would still see
    // the old index, so the right-hand side is settled into a temporary first.
    //
    // `??=` is deliberately excluded although it is also "not `Assign`": its right-hand
    // side is LAZY, evaluated only when the key is absent. Hoisting it runs it every time
    // AND before the presence check, so `$a[$slot] ??= ($slot = 0)` would test the slot the
    // right-hand side just selected instead of the one written in the source.
    let mut hoisted = EffectfulTargetLowerer::new(span);
    let rhs = if matches!(op, AssignmentOperator::Compound(_))
        && compound_rhs_can_disturb_index(&lhs_expr, &rhs)
    {
        hoisted.stabilize_unconditionally(rhs)
    } else {
        rhs
    };
    let lhs_span = lhs_expr.span;
    if is_append {
        let stmt = match lhs_expr.kind {
            ExprKind::Variable(array) => StmtKind::ArrayPush { array, value: rhs },
            ExprKind::PropertyAccess { object, property } => StmtKind::PropertyArrayPush {
                object,
                property,
                value: rhs,
            },
            ExprKind::ArrayAccess { array, index } => {
                return lower_nested_append_assignment(
                    Expr::new(ExprKind::ArrayAccess { array, index }, lhs_span),
                    rhs,
                    span,
                )
                .map(Some);
            }
            _ => return Err(CompileError::new(span, "Invalid assignment target")),
        };
        return Ok(Some(Stmt::new(stmt, span)));
    }

    let value = assignment_value(lhs_expr.clone(), op, rhs, span);

    let stmt = match lhs_expr.kind {
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::Variable(array) => StmtKind::ArrayAssign {
                array,
                index: *index,
                value,
            },
            ExprKind::PropertyAccess { object, property } => StmtKind::PropertyArrayAssign {
                object,
                property,
                index: *index,
                value,
            },
            _ => StmtKind::NestedArrayAssign {
                target: Expr::new(ExprKind::ArrayAccess { array, index }, span),
                value,
            },
        },
        ExprKind::PropertyAccess { object, property } => StmtKind::PropertyAssign {
            object,
            property,
            value,
        },
        ExprKind::DynamicPropertyAccess { object, property } => StmtKind::ExprStmt(Expr::new(
            ExprKind::Assignment {
                target: Box::new(Expr::new(
                    ExprKind::DynamicPropertyAccess { object, property },
                    lhs_span,
                )),
                value: Box::new(value),
                result_target: None,
                prelude: Vec::new(),
                conditional_value_temp: None,
            },
            span,
        )),
        // `$GLOBALS['name']` tokens look postfix-shaped, but the expression parser rewrites the
        // complete target to one alias variable, so this is an ordinary variable assignment.
        ExprKind::Variable(name) => StmtKind::Assign { name, value },
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };

    Ok(Some(hoisted.finish_if_used(stmt, span)))
}

/// True when `target` writes an element at a plain-VARIABLE index and `rhs` can reassign
/// that variable before the write lands.
///
/// A replay-safe right-hand side has no effects at all, so it can disturb nothing — which is
/// what keeps `$counts[$k]++` and `$sums[$k] += $n` on the existing path, emitting no temporary.
/// Every other right-hand side is settled first.
///
/// The gate used to also require the right-hand side to MENTION the index by name, which was
/// unsound: `$c[$k] += (function() use (&$k) { $k = 1; return 1; })();` answered `10,11` where
/// PHP answers `10,21`, because the mention sits in a closure the call expression does not scan.
/// Widening it costs a temporary on shapes like `$sums[$k] += f()` and costs no correctness:
/// PHP evaluates a compound assignment's right-hand side BEFORE reading the element, so settling
/// it early is what the language does anyway.
fn compound_rhs_can_disturb_index(target: &Expr, rhs: &Expr) -> bool {
    let ExprKind::ArrayAccess { index, .. } = &target.kind else {
        return false;
    };
    if !matches!(index.kind, ExprKind::Variable(_)) {
        return false;
    }
    !can_replay_assignment_target(rhs)
}

/// Lowers a complex target reference bind into ordinary statements.
///
/// For `$obj->prop =& $source`, PHP first writes the source value into the property-owned cell and
/// then aliases the source local to that cell. A local source therefore lowers to a synthetic
/// property assignment plus the existing local reference assignment. A declared-property source
/// retains its own cell identity in `PropertyRefAssign`, allowing both object slots to own the same
/// cell instead of rebinding a temporary and losing the original alias.
/// Array-element targets such as `$values[$key] =& $source` and nested appends such as
/// `$loops[$key][] =& $source` wrap the source in the owning array-reference marker before
/// reusing the ordinary set or nested append/write-back lowering.
fn parse_complex_ref_assignment(
    target: Expr,
    is_append: bool,
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1;
    let source = parse_expr(tokens, pos)?;
    if !is_valid_reference_source(&source.kind) {
        return Err(CompileError::new(
            span,
            "Reference assignment source must be a variable, array/property element, or a by-reference call",
        ));
    }
    expect_semicolon(tokens, pos)?;
    if is_append {
        if !matches!(&source.kind, ExprKind::Variable(_)) {
            return Err(CompileError::new(
                span,
                "Nested array reference appends currently require a variable source",
            ));
        }
        let reference = Expr::new(ExprKind::ArrayReference(Box::new(source)), span);
        return lower_nested_append_assignment(target, reference, span);
    }
    let source_name = match &source.kind {
        ExprKind::Variable(source_name) => Some(source_name),
        ExprKind::PropertyAccess { .. } => None,
        _ => {
            return Err(CompileError::new(
                span,
                "Complex reference targets currently require a variable or declared-property source",
            ));
        }
    };
    if matches!(&target.kind, ExprKind::ArrayAccess { .. }) {
        if source_name.is_none() {
            return Err(CompileError::new(
                span,
                "Array reference targets currently require a variable source",
            ));
        }
        let reference = Expr::new(ExprKind::ArrayReference(Box::new(source)), span);
        let kind = assignment_target_store_stmt(target, reference, span)?;
        return Ok(Stmt::new(kind, span));
    }
    let ExprKind::PropertyAccess { object, property } = &target.kind else {
        return Err(CompileError::new(
            span,
            "Complex reference targets currently require a declared property",
        ));
    };
    if source_name.is_none() {
        return Ok(Stmt::new(
            StmtKind::PropertyRefAssign {
                object: object.clone(),
                property: property.clone(),
                source,
            },
            span,
        ));
    }
    let assign = Stmt::new(
        StmtKind::PropertyAssign {
            object: object.clone(),
            property: property.clone(),
            value: source.clone(),
        },
        span,
    );
    let bind = Stmt::new(
        StmtKind::RefAssign {
            target: source_name.expect("variable reference source checked above").clone(),
            source: target,
        },
        span,
    );
    Ok(Stmt::new(StmtKind::Synthetic(vec![assign, bind]), span))
}

/// Lowers an append through a nested array target (`$a[0][] = $value`) into a
/// synthetic read/append/write-back sequence. The temporary append triggers the
/// existing copy-on-write split, and the final assignment stores the detached
/// nested array back into the original slot.
fn lower_nested_append_assignment(
    target: Expr,
    value: Expr,
    span: Span,
) -> Result<Stmt, CompileError> {
    let mut lowerer = EffectfulTargetLowerer::new(span);
    let target = lowerer.stabilize_array_target(target);
    let temp = lowerer.next_nested_append_temp_name();
    lowerer.stmts.push(Stmt::new(
        StmtKind::Assign {
            name: temp.clone(),
            value: target.clone(),
        },
        span,
    ));
    lowerer.stmts.push(Stmt::new(
        StmtKind::ArrayPush {
            array: temp.clone(),
            value,
        },
        span,
    ));
    let write_back =
        assignment_target_store_stmt(target, Expr::new(ExprKind::Variable(temp), span), span)?;
    Ok(lowerer.finish(write_back))
}

/// Builds the statement that writes `value` back into an already-stabilized
/// assignment target. Supports the same local, property, static property, and
/// array target families as postfix assignment lowering.
pub(crate) fn assignment_target_store_stmt(
    target: Expr,
    value: Expr,
    span: Span,
) -> Result<StmtKind, CompileError> {
    match target.kind {
        ExprKind::Variable(name) => Ok(StmtKind::Assign { name, value }),
        ExprKind::PropertyAccess { object, property } => Ok(StmtKind::PropertyAssign {
            object,
            property,
            value,
        }),
        ExprKind::StaticPropertyAccess { receiver, property } => {
            Ok(StmtKind::StaticPropertyAssign {
                receiver,
                property,
                value,
            })
        }
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::Variable(array) => Ok(StmtKind::ArrayAssign {
                array,
                index: *index,
                value,
            }),
            ExprKind::PropertyAccess { object, property } => Ok(StmtKind::PropertyArrayAssign {
                object,
                property,
                index: *index,
                value,
            }),
            ExprKind::StaticPropertyAccess { receiver, property } => {
                Ok(StmtKind::StaticPropertyArrayAssign {
                    receiver,
                    property,
                    index: *index,
                    value,
                })
            }
            _ => Ok(StmtKind::NestedArrayAssign {
                target: Expr::new(ExprKind::ArrayAccess { array, index }, span),
                value,
            }),
        },
        _ => Err(CompileError::new(span, "Invalid assignment target")),
    }
}

/// Builds the statement that appends `value` through an assignment-expression target.
///
/// Bare locals and property/static-property receivers map to their ordinary push statements;
/// nested array targets reuse the established read/push/write-back desugaring so growth is
/// republished all the way to the original container.
pub(crate) fn assignment_target_append_stmt(
    target: Expr,
    value: Expr,
    span: Span,
) -> Result<Stmt, CompileError> {
    let kind = match target.kind {
        ExprKind::Variable(array) => StmtKind::ArrayPush { array, value },
        ExprKind::PropertyAccess { object, property } => StmtKind::PropertyArrayPush {
            object,
            property,
            value,
        },
        ExprKind::StaticPropertyAccess { receiver, property } => {
            StmtKind::StaticPropertyArrayPush {
                receiver,
                property,
                value,
            }
        }
        ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
            StmtKind::DynamicStaticPropertyWrite {
                receiver,
                property,
                index: None,
                append: true,
                value,
            }
        }
        ExprKind::ArrayAccess { array, index } => {
            return lower_nested_append_assignment(
                Expr::new(ExprKind::ArrayAccess { array, index }, span),
                value,
                span,
            );
        }
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };
    Ok(Stmt::new(kind, span))
}

/// Parses discarded post-increment/decrement on a scoped (static class member) l-value target.
/// Handles `A::$x++`, `static::$x--`, `parent::$y++`, etc. Returns `Ok(None)` when no
/// postfix `++`/`--` is found at the top level of the statement.
pub(in crate::parser::stmt) fn try_parse_scoped_postfix_incdec(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Option<Stmt>, CompileError> {
    let start = *pos;
    let Some((incdec_pos, is_increment)) = find_top_level_postfix_incdec(tokens, start) else {
        return Ok(None);
    };
    if incdec_pos < start + 3 {
        return Ok(None);
    }

    let lhs = &tokens[start..incdec_pos];
    let mut lhs_pos = 0;
    let lhs_expr = parse_expr(lhs, &mut lhs_pos)?;
    if lhs_pos != lhs.len() {
        return Err(CompileError::new(span, "Invalid increment target"));
    }

    if !matches!(lhs_expr.kind, ExprKind::StaticPropertyAccess { .. }) {
        return Ok(None);
    }

    *pos = incdec_pos + 1;
    expect_semicolon(tokens, pos)?;

    lower_postfix_incdec_assignment(lhs_expr, is_increment, span).map(Some)
}

/// Parses discarded post-increment/decrement on a complex l-value target.
///
/// For statement contexts the original expression result is unused, so `$a[0]++`
/// can be lowered to the same read-modify-write shape as `$a[0] += 1`.
/// Simple local `$x++` is left to the existing local-variable parser.
pub(in crate::parser::stmt) fn try_parse_postfix_incdec(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Option<Stmt>, CompileError> {
    let start = *pos;
    let Some((incdec_pos, is_increment)) = find_top_level_postfix_incdec(tokens, start) else {
        return Ok(None);
    };
    if incdec_pos < start + 3 {
        return Ok(None);
    }

    let lhs = &tokens[start..incdec_pos];
    let contains_complex_target = lhs
        .iter()
        .skip(1)
        .any(|(token, _)| matches!(token, Token::Arrow | Token::QuestionArrow | Token::LBracket));
    if !contains_complex_target {
        return Ok(None);
    }

    let mut lhs_pos = 0;
    let lhs_expr = parse_expr(lhs, &mut lhs_pos)?;
    if lhs_pos != lhs.len() {
        return Err(CompileError::new(span, "Invalid increment target"));
    }

    *pos = incdec_pos + 1;
    expect_semicolon(tokens, pos)?;

    lower_postfix_incdec_assignment(lhs_expr, is_increment, span).map(Some)
}

/// Parses a scoped (static class member) postfix assignment, handling targets like
/// `$obj::prop`, `$obj::$prop`, and `$obj::prop[]`. Detects `+=` append style via `[]`.
/// For compound operators on static properties that cannot be replayed safely, lowers
/// to a temporary-variable sequence via `lower_effectful_static_assignment`.
/// Returns `Ok(None)` when no scoped assignment pattern is found.
pub(in crate::parser::stmt) fn try_parse_scoped_property_assignment(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Option<Stmt>, CompileError> {
    let start = *pos;
    let Some((assign_pos, op)) = find_top_level_assignment(tokens, start) else {
        return Ok(None);
    };
    if assign_pos < start + 3 {
        return Ok(None);
    }

    let lhs = &tokens[start..assign_pos];
    let is_append = lhs.len() >= 3
        && lhs[lhs.len() - 2].0 == Token::LBracket
        && lhs[lhs.len() - 1].0 == Token::RBracket;
    if is_append && op != AssignmentOperator::Assign {
        return Err(CompileError::new(span, "Invalid assignment target"));
    }
    let mut lhs_pos = 0;
    let lhs_expr_tokens = if is_append {
        &lhs[..lhs.len() - 2]
    } else {
        lhs
    };
    let lhs_expr = parse_expr(lhs_expr_tokens, &mut lhs_pos)?;
    if lhs_pos != lhs_expr_tokens.len() {
        return Err(CompileError::new(span, "Invalid assignment target"));
    }

    *pos = assign_pos + 1;
    if op == AssignmentOperator::Assign
        && matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Ampersand))
    {
        *pos += 1;
        let source = parse_expr(tokens, pos)?;
        expect_semicolon(tokens, pos)?;
        let ExprKind::ArrayAccess { array, index } = lhs_expr.kind else {
            return Err(CompileError::new(
                span,
                "Reference assignment target must be a static-property array element",
            ));
        };
        let ExprKind::StaticPropertyAccess { receiver, property } = array.kind else {
            return Err(CompileError::new(
                span,
                "Reference assignment target must be a static-property array element",
            ));
        };
        return Ok(Some(Stmt::new(
            StmtKind::StaticPropertyElementRefAssign {
                receiver,
                property,
                index: *index,
                source,
            },
            span,
        )));
    }
    let rhs = parse_assignment_value_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    if op != AssignmentOperator::Assign && !can_replay_assignment_target(&lhs_expr) {
        return lower_effectful_static_assignment(lhs_expr, op, rhs, span).map(Some);
    }
    let value = assignment_value(lhs_expr.clone(), op, rhs, span);

    // `self::$b[$k][] = $v` (and its `static::` / `parent::` / `Named::` siblings) is an append
    // through a *nested* target. The trailing `[]` was stripped from the LHS tokens above, so
    // `lhs_expr` is an `ExprKind::ArrayAccess` and the `if is_append` guard on the bare
    // `StaticPropertyAccess` arm below — which only ever handles `self::$b[] = $v` — cannot
    // match it. Left alone it falls into the plain `ArrayAccess` arm, which ignores `is_append`
    // entirely and emits `StaticPropertyArrayAssign`: the append is silently dropped and the
    // bucket is OVERWRITTEN with the single value. Route it through the same read/append/
    // write-back desugar every other nested-append target already uses; the write-back builder
    // (`assignment_target_store_stmt`) already supports the static-property family.
    if is_append && matches!(lhs_expr.kind, ExprKind::ArrayAccess { .. }) {
        return lower_nested_append_assignment(lhs_expr, value, span).map(Some);
    }

    let stmt = match lhs_expr.kind {
        ExprKind::StaticPropertyAccess { receiver, property } if is_append => {
            StmtKind::StaticPropertyArrayPush {
                receiver,
                property,
                value,
            }
        }
        ExprKind::DynamicStaticPropertyAccess { receiver, property } if is_append => {
            StmtKind::DynamicStaticPropertyWrite {
                receiver,
                property,
                index: None,
                append: true,
                value,
            }
        }
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::StaticPropertyAccess { receiver, property } => {
                StmtKind::StaticPropertyArrayAssign {
                    receiver,
                    property,
                    index: *index,
                    value,
                }
            }
            ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
                StmtKind::DynamicStaticPropertyWrite {
                    receiver,
                    property,
                    index: Some(*index),
                    append: false,
                    value,
                }
            }
            _ => StmtKind::NestedArrayAssign {
                target: Expr::new(ExprKind::ArrayAccess { array, index }, span),
                value,
            },
        },
        ExprKind::StaticPropertyAccess { receiver, property } => StmtKind::StaticPropertyAssign {
            receiver,
            property,
            value,
        },
        ExprKind::DynamicStaticPropertyAccess { receiver, property } => {
            StmtKind::DynamicStaticPropertyWrite {
                receiver,
                property,
                index: None,
                append: false,
                value,
            }
        }
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };

    Ok(Some(Stmt::new(stmt, span)))
}

/// Scans tokens starting from `start` (skipping nested parentheses, brackets, and braces)
/// and returns the first statement-level assignment before a top-level conditional begins.
/// Assignments in ternary or null-coalescing branches remain expression assignments for Pratt.
/// Returns `None` if no eligible operator is found before a semicolon at depth 0.
fn find_top_level_assignment(
    tokens: &[SpannedToken],
    start: usize,
) -> Option<(usize, AssignmentOperator)> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut saw_top_level_conditional = false;
    let mut pos = start;

    while pos < tokens.len() {
        match tokens[pos].0 {
            Token::LParen => paren_depth += 1,
            Token::RParen => paren_depth = paren_depth.saturating_sub(1),
            Token::LBracket => bracket_depth += 1,
            Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
            Token::LBrace => brace_depth += 1,
            Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
            Token::Semicolon if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return None;
            }
            Token::Question if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                // An assignment after a top-level ternary marker belongs to a branch, not to
                // the whole expression as a postfix statement target (`cond ?: $obj->p = v`).
                // Let the Pratt parser preserve PHP's assignment precedence in that branch.
                saw_top_level_conditional = true;
            }
            Token::QuestionQuestion
                if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 =>
            {
                // `??` is right-associative and its fallback may be an assignment:
                // `$a ?? $b ?? $k = $name`. Stealing that `=` would treat the complete
                // coalescing expression as an lvalue and reject valid PHP.
                saw_top_level_conditional = true;
            }
            _ if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                if !saw_top_level_conditional {
                    if let Some(op) = assignment_operator(&tokens[pos].0) {
                        return Some((pos, op));
                    }
                }
            }
            _ => {}
        }
        pos += 1;
    }

    None
}

/// Finds a top-level postfix `++` or `--` immediately before the statement semicolon.
///
/// Nested occurrences inside indexes or call arguments are ignored so expressions
/// such as `$items[$i++] = 1` remain assignment statements with an effectful index. A
/// prefix operator inside an assignment RHS (`$id = 'x'.++$obj->counter`) is not a
/// postfix statement target and must remain with the ordinary assignment parser.
fn find_top_level_postfix_incdec(tokens: &[SpannedToken], start: usize) -> Option<(usize, bool)> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut pos = start;

    while pos < tokens.len() {
        match tokens[pos].0 {
            Token::LParen => paren_depth += 1,
            Token::RParen => paren_depth = paren_depth.saturating_sub(1),
            Token::LBracket => bracket_depth += 1,
            Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
            Token::LBrace => brace_depth += 1,
            Token::RBrace => brace_depth = brace_depth.saturating_sub(1),
            Token::Semicolon if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return None;
            }
            // `$k = $c->i++;` is an ASSIGNMENT whose right-hand side ends in `++`, not an
            // increment statement targeting `$k = $c->i`. Stopping at a top-level `=` hands
            // it to the expression parser, which desugars the increment in place; without
            // this the whole line was claimed here and rejected as an invalid target.
            Token::Assign if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return None;
            }
            Token::PlusPlus
                if paren_depth == 0
                    && bracket_depth == 0
                    && brace_depth == 0
                    && matches!(tokens.get(pos + 1), Some((Token::Semicolon, _))) =>
            {
                return Some((pos, true));
            }
            Token::MinusMinus
                if paren_depth == 0
                    && bracket_depth == 0
                    && brace_depth == 0
                    && matches!(tokens.get(pos + 1), Some((Token::Semicolon, _))) =>
            {
                return Some((pos, false));
            }
            _ => {}
        }
        pos += 1;
    }

    None
}

/// Returns `true` if the expression is safe to replay as an l-value in a compound assignment,
/// meaning its value can be read multiple times without observable side effects.
/// Replayable expressions include variables, literals, property/static-property access on
/// replayable bases, and recursively their sub-expressions.
/// Function calls, new[], and most other `ExprKind` variants return `false`.
pub(crate) fn can_replay_assignment_target(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Variable(_) | ExprKind::This | ExprKind::StaticPropertyAccess { .. } => true,
        ExprKind::ArrayAccess { array, index } => {
            can_replay_assignment_target(array) && can_replay_assignment_target(index)
        }
        ExprKind::PropertyAccess { object, .. } => can_replay_assignment_target(object),
        ExprKind::DynamicPropertyAccess { object, property } => {
            can_replay_assignment_target(object) && can_replay_assignment_target(property)
        }
        ExprKind::BinaryOp { left, right, .. } => {
            can_replay_assignment_target(left) && can_replay_assignment_target(right)
        }
        ExprKind::InstanceOf { value, target } => {
            can_replay_assignment_target(value) && can_replay_instanceof_target(target)
        }
        ExprKind::Negate(value)
        | ExprKind::Not(value)
        | ExprKind::BitNot(value)
        | ExprKind::Cast { expr: value, .. }
        | ExprKind::PtrCast { expr: value, .. }
        | ExprKind::NamedArg { value, .. }
        | ExprKind::Spread(value) => can_replay_assignment_target(value),
        ExprKind::NullCoalesce { value, default } | ExprKind::ShortTernary { value, default } => {
            can_replay_assignment_target(value) && can_replay_assignment_target(default)
        }
        ExprKind::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            can_replay_assignment_target(condition)
                && can_replay_assignment_target(then_expr)
                && can_replay_assignment_target(else_expr)
        }
        ExprKind::IntLiteral(_)
        | ExprKind::FloatLiteral(_)
        | ExprKind::StringLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Null
        | ExprKind::ConstRef(_)
        | ExprKind::ClassConstant { .. }
        | ExprKind::ScopedConstantAccess { .. }
        | ExprKind::MagicConstant(_) => true,
        _ => false,
    }
}

/// Recursively checks whether `target` can be used as an r-value in a replay-safe assignment.
fn can_replay_instanceof_target(target: &InstanceOfTarget) -> bool {
    match target {
        InstanceOfTarget::Name(_) => true,
        InstanceOfTarget::Expr(expr) => can_replay_assignment_target(expr),
    }
}

/// Lowers a compound postfix assignment (e.g., `+=`, `-=`) on a non-replayable l-value
/// by extracting sub-expressions into temporary variables so each is evaluated exactly once.
/// Builds a `Synthetic` statement containing the temporaries followed by the final assignment.
fn lower_effectful_postfix_assignment(
    lhs_expr: Expr,
    op: AssignmentOperator,
    rhs: Expr,
    span: Span,
) -> Result<Stmt, CompileError> {
    let mut lowerer = EffectfulTargetLowerer::new(span);
    let lowered = match lhs_expr.kind {
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::Variable(array) => {
                let index = lowerer.stabilize(*index);
                let target = Expr::new(
                    ExprKind::ArrayAccess {
                        array: Box::new(Expr::new(ExprKind::Variable(array.clone()), span)),
                        index: Box::new(index.clone()),
                    },
                    span,
                );
                let value = assignment_value(target, op, rhs, span);
                StmtKind::ArrayAssign {
                    array,
                    index,
                    value,
                }
            }
            ExprKind::PropertyAccess { object, property } => {
                let object = Box::new(lowerer.stabilize(*object));
                let index = lowerer.stabilize(*index);
                let target = Expr::new(
                    ExprKind::ArrayAccess {
                        array: Box::new(Expr::new(
                            ExprKind::PropertyAccess {
                                object: object.clone(),
                                property: property.clone(),
                            },
                            span,
                        )),
                        index: Box::new(index.clone()),
                    },
                    span,
                );
                let value = assignment_value(target, op, rhs, span);
                StmtKind::PropertyArrayAssign {
                    object,
                    property,
                    index,
                    value,
                }
            }
            _ => {
                let target = lowerer.stabilize_array_target(Expr::new(
                    ExprKind::ArrayAccess { array, index },
                    span,
                ));
                let value = assignment_value(target.clone(), op, rhs, span);
                StmtKind::NestedArrayAssign { target, value }
            }
        },
        ExprKind::PropertyAccess { object, property } => {
            let object = Box::new(lowerer.stabilize(*object));
            let target = Expr::new(
                ExprKind::PropertyAccess {
                    object: object.clone(),
                    property: property.clone(),
                },
                span,
            );
            let value = assignment_value(target, op, rhs, span);
            StmtKind::PropertyAssign {
                object,
                property,
                value,
            }
        }
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };
    Ok(lowerer.finish(lowered))
}

/// Lowers discarded post-increment/decrement to the existing assignment statement forms.
///
/// Statement position discards the operator's value, so prefix `++$obj->n;` lowers through
/// here too: with the result unused, `++X` and `X++` are both `X += 1`.
pub(crate) fn lower_postfix_incdec_assignment(
    lhs_expr: Expr,
    is_increment: bool,
    span: Span,
) -> Result<Stmt, CompileError> {
    let op = if is_increment {
        AssignmentOperator::Compound(BinOp::Add)
    } else {
        AssignmentOperator::Compound(BinOp::Sub)
    };
    let one = Expr::new(ExprKind::IntLiteral(1), span);

    if !can_replay_assignment_target(&lhs_expr) {
        return lower_effectful_postfix_assignment(lhs_expr, op, one, span);
    }

    let value = assignment_value(lhs_expr.clone(), op, one, span);
    let kind = match lhs_expr.kind {
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::Variable(array) => StmtKind::ArrayAssign {
                array,
                index: *index,
                value,
            },
            ExprKind::PropertyAccess { object, property } => StmtKind::PropertyArrayAssign {
                object,
                property,
                index: *index,
                value,
            },
            _ => StmtKind::NestedArrayAssign {
                target: Expr::new(ExprKind::ArrayAccess { array, index }, span),
                value,
            },
        },
        ExprKind::PropertyAccess { object, property } => StmtKind::PropertyAssign {
            object,
            property,
            value,
        },
        ExprKind::StaticPropertyAccess { receiver, property } => {
            StmtKind::StaticPropertyAssign {
                receiver,
                property,
                value,
            }
        }
        _ => return Err(CompileError::new(span, "Invalid increment target")),
    };

    Ok(Stmt::new(kind, span))
}

/// Lowers a compound static property assignment where the target cannot be replayed safely.
/// Temporaries are created for any sub-expression that could produce observable side effects
/// (e.g., method calls on the object or array accesses). Returns a `Synthetic` statement.
fn lower_effectful_static_assignment(
    lhs_expr: Expr,
    op: AssignmentOperator,
    rhs: Expr,
    span: Span,
) -> Result<Stmt, CompileError> {
    let mut lowerer = EffectfulTargetLowerer::new(span);
    let lowered = match lhs_expr.kind {
        ExprKind::ArrayAccess { array, index } => match array.kind {
            ExprKind::StaticPropertyAccess { receiver, property } => {
                let index = lowerer.stabilize(*index);
                let target = Expr::new(
                    ExprKind::ArrayAccess {
                        array: Box::new(Expr::new(
                            ExprKind::StaticPropertyAccess {
                                receiver: receiver.clone(),
                                property: property.clone(),
                            },
                            span,
                        )),
                        index: Box::new(index.clone()),
                    },
                    span,
                );
                let value = assignment_value(target, op, rhs, span);
                StmtKind::StaticPropertyArrayAssign {
                    receiver,
                    property,
                    index,
                    value,
                }
            }
            _ => {
                let target = lowerer.stabilize_array_target(Expr::new(
                    ExprKind::ArrayAccess { array, index },
                    span,
                ));
                let value = assignment_value(target.clone(), op, rhs, span);
                StmtKind::NestedArrayAssign { target, value }
            }
        },
        ExprKind::StaticPropertyAccess { receiver, property } => {
            let target = Expr::new(
                ExprKind::StaticPropertyAccess {
                    receiver: receiver.clone(),
                    property: property.clone(),
                },
                span,
            );
            let value = assignment_value(target, op, rhs, span);
            StmtKind::StaticPropertyAssign {
                receiver,
                property,
                value,
            }
        }
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };
    Ok(lowerer.finish(lowered))
}

/// Helper that rewrites complex l-value targets into sequences of temporary-variable
/// assignments so that source evaluation order is preserved and side effects are not duplicated.
struct EffectfulTargetLowerer {
    span: Span,
    next_temp: usize,
    stmts: Vec<Stmt>,
}

impl EffectfulTargetLowerer {
    /// Initializes the lowerer with the source span used for all synthesized statements
    /// and temporary variable names.
    fn new(span: Span) -> Self {
        Self {
            span,
            next_temp: 0,
            stmts: Vec::new(),
        }
    }

    /// If `expr` is replay-safe, returns it unchanged. Otherwise, emits an `Assign`
    /// statement to a uniquely-named temporary and returns a `Variable` reference to it.
    /// Increments `next_temp` to keep temporary names unique across the same statement.
    fn stabilize(&mut self, expr: Expr) -> Expr {
        if can_replay_assignment_target(&expr) {
            return expr;
        }
        let name = self.next_temp_name();
        self.stmts.push(Stmt::new(
            StmtKind::Assign {
                name: name.clone(),
                value: expr,
            },
            self.span,
        ));
        Expr::new(ExprKind::Variable(name), self.span)
    }

    /// Returns a unique synthetic temporary name for this lowered statement.
    fn next_temp_name(&mut self) -> String {
        let name = format!(
            "__elephc_compound_{}_{}_{}",
            self.span.line, self.span.col, self.next_temp
        );
        self.next_temp += 1;
        name
    }

    /// Mints the temporary that holds the bucket of a nested append, under its own reserved
    /// prefix.
    ///
    /// The prefix is what lets IR lowering recognize a nested-append `Synthetic` group and
    /// lower it as a fused, in-place append instead of the read/copy/write-back it desugars
    /// to here. `next_temp_name`'s prefix is shared with the `.=` / `+=` desugars, which emit
    /// the same statement shapes, so it cannot serve as that signal. PHP source cannot forge
    /// either lock: the name is not a legal PHP identifier and `StmtKind::Synthetic` has no
    /// surface syntax.
    fn next_nested_append_temp_name(&mut self) -> String {
        let name = format!(
            "{}{}_{}_{}",
            NESTED_APPEND_TEMP_PREFIX, self.span.line, self.span.col, self.next_temp
        );
        self.next_temp += 1;
        name
    }

    /// Stabilizes an array-access target, recursively stabilizing both the array base
    /// and the index. For simple array bases (Variable, This, StaticPropertyAccess),
    /// the array base is kept as-is; deeper bases are stabilized via `stabilize_array_base`.
    fn stabilize_array_target(&mut self, expr: Expr) -> Expr {
        let span = expr.span;
        match expr.kind {
            ExprKind::ArrayAccess { array, index } => Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(self.stabilize_array_base(*array)),
                    index: Box::new(self.stabilize(*index)),
                },
                span,
            ),
            _ => self.stabilize(expr),
        }
    }

    /// Stabilizes the base of a nested array access chain. Recursively processes
    /// `ArrayAccess` and `PropertyAccess` chains; returns `Variable`, `This`,
    /// and `StaticPropertyAccess` directly; calls `stabilize` for all other expressions.
    fn stabilize_array_base(&mut self, expr: Expr) -> Expr {
        let span = expr.span;
        match expr.kind {
            ExprKind::ArrayAccess { array, index } => Expr::new(
                ExprKind::ArrayAccess {
                    array: Box::new(self.stabilize_array_base(*array)),
                    index: Box::new(self.stabilize(*index)),
                },
                span,
            ),
            ExprKind::PropertyAccess { object, property } => Expr::new(
                ExprKind::PropertyAccess {
                    object: Box::new(self.stabilize_array_base(*object)),
                    property,
                },
                span,
            ),
            ExprKind::Variable(_) | ExprKind::This | ExprKind::StaticPropertyAccess { .. } => expr,
            _ => self.stabilize(expr),
        }
    }

    /// Appends `final_stmt` as the last statement and wraps the entire sequence
    /// in a `Synthetic` statement node returned as a single `Stmt`.
    fn finish(mut self, final_stmt: StmtKind) -> Stmt {
        self.stmts.push(Stmt::new(final_stmt, self.span));
        Stmt::new(StmtKind::Synthetic(self.stmts), self.span)
    }

    /// Returns `final_stmt` unwrapped when nothing was hoisted, so a caller that only
    /// SOMETIMES needs a temporary does not wrap every other statement in a `Synthetic`
    /// group it has no use for.
    fn finish_if_used(self, final_stmt: StmtKind, span: Span) -> Stmt {
        if self.stmts.is_empty() {
            return Stmt::new(final_stmt, span);
        }
        self.finish(final_stmt)
    }

    /// Emits `expr` into a fresh temporary and returns a reference to it, even when the
    /// expression is replay-safe. `stabilize` exists to make an effectful expression
    /// evaluate ONCE; this exists to make it evaluate EARLY.
    fn stabilize_unconditionally(&mut self, expr: Expr) -> Expr {
        let name = self.next_temp_name();
        self.stmts.push(Stmt::new(
            StmtKind::Assign {
                name: name.clone(),
                value: expr,
            },
            self.span,
        ));
        Expr::new(ExprKind::Variable(name), self.span)
    }
}
