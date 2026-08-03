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
use crate::parser::ast::{BinOp, Expr, ExprKind, InstanceOfTarget, Stmt, StmtKind};
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

    // The tokens before the top-level `=` parse to a complete expression, but if that expression
    // is not itself an assignable target shape — e.g. `A ?: $x->p` (short-ternary) or `cond && $x->p`
    // — then this is not a statement-level assignment: the `=` binds to the adjacent lvalue *inside*
    // the expression (PHP parses `A ?: $x = B` as `A ?: ($x = B)`). Bail without consuming so the
    // caller parses the whole statement as a bare expression statement, where the Pratt parser
    // performs that adjacent-lvalue binding. A genuinely invalid target still surfaces an error there.
    if !parsed_lhs_is_assignable_target(&lhs_expr) {
        return Ok(None);
    }

    *pos = assign_pos + 1;
    // `$obj->prop = &$src` / `$arr[$k] = &$src`: reference assignment into a
    // property or array-element lvalue. The plain-variable form is handled by the
    // simple-variable assignment parser; here the LHS is a complex lvalue. The
    // append form (`$a[] = &$src`, `$a[$k][] = &$src`) routes here too and is
    // distinguished by `is_append`, with `lhs_expr` naming the CONTAINER.
    if op == AssignmentOperator::Assign
        && matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Ampersand))
    {
        return parse_postfix_ref_assign(lhs_expr, tokens, pos, is_append, span).map(Some);
    }
    let rhs = parse_assignment_value_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    if op != AssignmentOperator::Assign && !can_replay_assignment_target(&lhs_expr) {
        return lower_effectful_postfix_assignment(lhs_expr, op, rhs, span).map(Some);
    }
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
        // A postfix LHS that parsed down to a plain variable is a plain assignment. This is
        // reachable because `$GLOBALS['name']` is rewritten to a single alias variable while
        // its tokens still look like a postfix `[` target, so the postfix parser owns the
        // statement but must emit the simple-variable form.
        ExprKind::Variable(name) => StmtKind::Assign { name, value },
        _ => return Err(CompileError::new(span, "Invalid assignment target")),
    };

    Ok(Some(Stmt::new(stmt, span)))
}

/// Parses reference assignment into a complex lvalue (`$obj->prop = &$src`,
/// `$obj->$name = &$src`, `$arr[$k] = &$src`, or an append `$arr[] = &$src` /
/// `$arr[$k][] = &$src`) after the leading `&` has been detected.
///
/// `lhs_expr` is the already-parsed target lvalue. When `append` is `true`, `lhs_expr`
/// is the CONTAINER (a `Variable`, `ArrayAccess`, `PropertyAccess`, or
/// `StaticPropertyAccess`) with no sentinel index. Consumes the `&`, parses the
/// reference source, validates that the source is a legal reference source and that the
/// target has a supported lvalue shape, then returns a `RefAssignToTarget` statement.
/// Plain-variable reference assignment is handled by the simple-variable assignment
/// parser instead.
fn parse_postfix_ref_assign(
    lhs_expr: Expr,
    tokens: &[SpannedToken],
    pos: &mut usize,
    append: bool,
    span: Span,
) -> Result<Stmt, CompileError> {
    *pos += 1; // consume the `&`
    let source = parse_expr(tokens, pos)?;
    if !is_valid_reference_source(&source.kind) {
        return Err(CompileError::new(
            span,
            "Reference assignment source must be a variable, array/property element, or a by-reference call",
        ));
    }
    // A static-property source is only supported into a plain-variable target
    // (`$e = &self::$n`); aliasing it into a complex lvalue (`$obj->prop = &self::$n`)
    // is a later slice. Reject it here so it does not silently become a value copy.
    if matches!(source.kind, ExprKind::StaticPropertyAccess { .. }) {
        return Err(CompileError::new(
            span,
            "Reference assignment source must be a variable, array/property element, or a by-reference call",
        ));
    }
    // Non-append targets are a property or an explicit-key array element. Append targets are
    // instead the CONTAINER itself: a plain `Variable` (`$a[] = &…`), a nested `ArrayAccess`
    // (`$a[$k][] = &…`), or a property/static-property container (the checker then rejects the
    // property-container append forms as unsupported).
    let target_shape_ok = if append {
        matches!(
            lhs_expr.kind,
            ExprKind::Variable(_)
                | ExprKind::ArrayAccess { .. }
                | ExprKind::PropertyAccess { .. }
                | ExprKind::StaticPropertyAccess { .. }
        )
    } else {
        matches!(
            lhs_expr.kind,
            ExprKind::PropertyAccess { .. }
                | ExprKind::DynamicPropertyAccess { .. }
                | ExprKind::ArrayAccess { .. }
        )
    };
    if !target_shape_ok {
        return Err(CompileError::new(
            span,
            "Reference assignment target must be a variable, array element, or object property",
        ));
    }
    expect_semicolon(tokens, pos)?;
    Ok(Stmt::new(
        StmtKind::RefAssignToTarget {
            target: lhs_expr,
            source,
            append,
        },
        span,
    ))
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
    let temp = lowerer.next_temp_name();
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
/// assignment target. Supports the same local, property, dynamic property,
/// static property, and array target families as postfix assignment lowering.
/// Also reused by the foreach lvalue-binding desugar
/// (`crate::parser::foreach_target`) so per-iteration stores share the exact
/// statement shapes the assignment parser produces.
pub(crate) fn assignment_target_store_stmt(
    target: Expr,
    value: Expr,
    span: Span,
) -> Result<StmtKind, CompileError> {
    match target.kind {
        ExprKind::Variable(name) => Ok(StmtKind::Assign { name, value }),
        // `$obj->{$name} = $v` / `$obj->$name = $v` — dynamic property writes have no
        // dedicated StmtKind; they reuse the expression-position assignment node, the
        // same shape `try_parse_postfix_assignment` emits for a dynamic-property LHS.
        ExprKind::DynamicPropertyAccess { object, property } => Ok(StmtKind::ExprStmt(Expr::new(
            ExprKind::Assignment {
                target: Box::new(Expr::new(
                    ExprKind::DynamicPropertyAccess { object, property },
                    span,
                )),
                value: Box::new(value),
                result_target: None,
                prelude: Vec::new(),
                conditional_value_temp: None,
            },
            span,
        ))),
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

    // A top-level assignment before the trailing `++`/`--` means this is an assignment
    // statement whose right-hand side ends in a postfix increment (`$x = $o->n++;`),
    // not a discarded postfix-increment statement. Defer to the expression parser, which
    // desugars the complex-l-value postfix increment in value position.
    if let Some((assign_pos, _)) = find_top_level_assignment(tokens, start) {
        if assign_pos < incdec_pos {
            return Ok(None);
        }
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

/// Parses a discarded pre-increment/decrement on a complex l-value target
/// (`++$obj->prop;`, `--$arr[$k];`).
///
/// In a statement context the expression result is unused, so a prefix `++`/`--`
/// is observably identical to the postfix form and lowers to the same
/// read-modify-write shape as `$target += 1`. A bare local target (`++$x;`) is
/// left to the existing local-variable parser so its `PreIncrement` AST node,
/// which downstream passes already handle, is preserved.
///
/// Returns `Ok(None)` when the statement does not start with `++`/`--` followed
/// by a variable and a complex-target marker, so the caller falls back to
/// `parse_incdec_stmt`.
pub(in crate::parser::stmt) fn try_parse_prefix_incdec(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Option<Stmt>, CompileError> {
    let start = *pos;
    let is_increment = match tokens.get(start).map(|(token, _)| token) {
        Some(Token::PlusPlus) => true,
        Some(Token::MinusMinus) => false,
        _ => return Ok(None),
    };

    // A static-scope l-value target (`++self::$n;`, `--Foo::$n;`, `++\Ns\Foo::$n;`) begins
    // with a static receiver token rather than a variable. It is parsed through the general
    // expression grammar — which desugars the prefix increment to a compound assignment and
    // rejects non-l-value operands (e.g. `++FOO;`) — and emitted as an expression statement,
    // mirroring how the postfix form `self::$n++;` already parses at statement position.
    if matches!(
        tokens.get(start + 1).map(|(token, _)| token),
        Some(
            Token::Self_
                | Token::Static
                | Token::Parent
                | Token::Identifier(_)
                | Token::Backslash
        )
    ) {
        let expr = parse_expr(tokens, pos)?;
        expect_semicolon(tokens, pos)?;
        return Ok(Some(Stmt::new(StmtKind::ExprStmt(expr), span)));
    }

    // The l-value must begin with a variable or `$this`, and the token after it must
    // be a complex-target marker (`->`, `?->`, `[`). A bare `++$x;` (marker is `;`)
    // is left to `parse_incdec_stmt`, which keeps the `PreIncrement` node.
    if !matches!(
        tokens.get(start + 1).map(|(token, _)| token),
        Some(Token::Variable(_) | Token::This)
    ) {
        return Ok(None);
    }
    if !matches!(
        tokens.get(start + 2).map(|(token, _)| token),
        Some(Token::Arrow | Token::QuestionArrow | Token::LBracket)
    ) {
        return Ok(None);
    }

    let mut probe = start + 1;
    let lhs_expr = parse_expr(tokens, &mut probe)?;
    *pos = probe;
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
    // `self::$a[$dir] = &self::$a[$k]`: reference assignment into a static-property
    // array element. The leading `&` after the `=` routes to the shared reference-assign
    // parser (mirroring the property/array-element form in `try_parse_postfix_assignment`),
    // which validates the target/source shapes and emits `RefAssignToTarget`. The append form
    // (`self::$a[] = &$x`) parses here too (with `is_append`) and is rejected by the checker
    // as an unsupported static-property append target rather than mis-parsing.
    if op == AssignmentOperator::Assign
        && matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Ampersand))
    {
        return parse_postfix_ref_assign(lhs_expr, tokens, pos, is_append, span).map(Some);
    }
    let rhs = parse_assignment_value_expr(tokens, pos)?;
    expect_semicolon(tokens, pos)?;
    // `self::$a[$k][] = $v` — an append into a NESTED element of a static-property array.
    // `is_append` names the trailing `[]` and `lhs_expr` is the CONTAINER (`self::$a[$k]`), so
    // without this the `ArrayAccess` arm of the assign match below would drop the append and
    // store `$v` straight into `self::$a[$k]` — a silent one-level-too-shallow write. Route it
    // through the same read-modify-write lowering the unscoped parser uses for `$a[$k][] = $v`;
    // its write-back re-enters `assignment_target_store_stmt`, whose static-property arm emits
    // the `StaticPropertyArrayAssign` that stores the appended-to container back.
    if is_append && matches!(lhs_expr.kind, ExprKind::ArrayAccess { .. }) {
        return lower_nested_append_assignment(lhs_expr, rhs, span).map(Some);
    }
    if op != AssignmentOperator::Assign && !can_replay_assignment_target(&lhs_expr) {
        return lower_effectful_static_assignment(lhs_expr, op, rhs, span).map(Some);
    }
    let value = assignment_value(lhs_expr.clone(), op, rhs, span);

    let stmt = match lhs_expr.kind {
        ExprKind::StaticPropertyAccess { receiver, property } if is_append => {
            StmtKind::StaticPropertyArrayPush {
                receiver,
                property,
                value,
            }
        }
        // `self::${$n}[] = v` — a push through a dynamic-named static property.
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
            // `self::${$n}[$k] = v` — an array-element write through a dynamic-named static
            // property (the DebugClassLoader shape).
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
        // `self::${$n} = v` — a direct write through a dynamic-named static property.
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

/// Returns true when `expr` (the expression parsed from the tokens before a top-level `=`) is an
/// assignable target shape: a variable, array element, or object/static property access. When it
/// is anything else — a short-ternary, ternary, binary operation, or call result that merely
/// *contains* a postfix access — the leading `=` is not a statement-level assignment but binds to
/// an lvalue inside the expression, so `try_parse_postfix_assignment` bails to bare-expression
/// parsing. Mirrors the shapes accepted by `expr::is_assignment_expression_target`.
fn parsed_lhs_is_assignable_target(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Variable(_)
            | ExprKind::ArrayAccess { .. }
            | ExprKind::PropertyAccess { .. }
            | ExprKind::DynamicPropertyAccess { .. }
            | ExprKind::StaticPropertyAccess { .. }
    )
}

/// Scans tokens starting from `start` (skipping nested parentheses, brackets, and braces)
/// and returns the position and operator of the first top-level assignment at nesting depth 0.
/// Returns `None` if no assignment operator is found before a semicolon at depth 0.
///
/// Bails (`None`) as soon as a top-level ternary `?` is seen before any assignment operator:
/// once the statement is fundamentally a conditional expression (`$c ? $a[0]=7 : $a[0]=8;`), any
/// `=` reachable from here belongs to an assignment nested inside one of the ternary's branches,
/// not to a single postfix-assignment lvalue chain that starts at `start`. Deferring to the general
/// expression parser lets the Pratt loop build the ternary and let each branch's own assignment
/// bind independently — scanning past the `?` would otherwise try to parse the dangling prefix
/// before the first `=` (e.g. `$c ? $a[0]`) as a standalone expression and hard-error on the
/// incomplete ternary instead of falling back gracefully.
fn find_top_level_assignment(
    tokens: &[SpannedToken],
    start: usize,
) -> Option<(usize, AssignmentOperator)> {
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
            Token::Question if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return None;
            }
            _ if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                if let Some(op) = assignment_operator(&tokens[pos].0) {
                    return Some((pos, op));
                }
            }
            _ => {}
        }
        pos += 1;
    }

    None
}

/// Finds a top-level `++` or `--` before the statement semicolon.
///
/// Nested occurrences inside indexes or call arguments are ignored so expressions
/// such as `$items[$i++] = 1` remain assignment statements with an effectful index.
///
/// Bails (`None`) on a top-level ternary `?` before any `++`/`--` is found, for the same reason
/// `find_top_level_assignment` does: a `++`/`--` beyond the `?` belongs to one of the ternary's
/// branches, and the whole statement must fall back to general expression parsing instead of being
/// misread as a discarded postfix/prefix increment on a dangling prefix.
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
            Token::Question if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return None;
            }
            Token::PlusPlus if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some((pos, true));
            }
            Token::MinusMinus if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
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
        ExprKind::DynamicStaticPropertyAccess { property, .. } => {
            can_replay_assignment_target(property)
        }
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
fn lower_postfix_incdec_assignment(
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
}
