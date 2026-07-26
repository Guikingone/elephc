//! Purpose:
//! Implements Pratt parsing for PHP infix, postfix, access, call, and assignment expressions.
//! Encodes operator precedence and associativity into binding-power tables.
//!
//! Called from:
//! - `crate::parser::expr::parse_expr()` and recursive expression parsing paths.
//!
//! Key details:
//! - Binding powers must match PHP precedence exactly because downstream passes trust the AST shape.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::names::Name;
use crate::parser::ast::{BinOp, CallableTarget, Expr, ExprKind, InstanceOfTarget};
use crate::parser::stmt::parse_name;
use crate::span::Span;

use super::assignment_targets::{
    AssignmentExpressionLowerer, build_append_assignment_expression,
    is_assignment_expression_target, is_non_local_assignment_target,
    simple_positional_list_vars,
};
use super::calls::parse_first_class_callable_parens;
use super::parse_args;
use super::prefix::parse_prefix;
use super::prefix_complex::build_dynamic_method_first_class_callable;
use super::parse_expr;

/// Parses an expression using Pratt parsing, starting with a prefix expression and
/// extending it via infix, postfix, access, call, and assignment operators.
///
/// `min_bp` is the minimum binding power required to continue parsing. Lower
/// binding powers indicate looser binding (e.g., lower precedence). The function
/// returns when it encounters a token whose binding power is below `min_bp` or
/// when no more infix operators apply.
///
/// # Inputs
/// - `tokens`: the token stream
/// - `pos`: current position (updated to the token after the parsed expression)
/// - `min_bp`: minimum binding power threshold
///
/// # Returns
/// The parsed left-hand side expression with all applicable operators applied.
///
/// # Algorithm
/// 1. Parse an initial prefix expression (`parse_prefix`)
/// 2. In the first loop, consume postfix operators: array access (`[]`),
///    object access (`->` / `?->`), and callable invocation (`()`)
/// 3. In the second loop, consume ternary (`? :`), instanceof, assignments
///    (`=`, `+=`, `??=`, etc.), pipe (`|>`), and remaining binary operators
pub(super) fn parse_expr_bp(
    tokens: &[SpannedToken],
    pos: &mut usize,
    min_bp: u8,
) -> Result<Expr, CompileError> {
    // Every expression-recursion cycle passes through here, so this single
    // guard bounds the whole expression grammar: when less than 128 KiB of
    // stack remains, the recursion continues on a fresh 4 MiB segment instead
    // of overflowing. Deeply nested expressions (builtin preludes, generated
    // code) otherwise abort 2 MiB test-thread stacks. The red zone must exceed
    // one full expression-recursion cycle in a debug build
    // (parse_expr_bp_inner + parse_prefix + parse_named_expr is ~80 KiB), so
    // 64 KiB is no longer enough headroom.
    stacker::maybe_grow(128 * 1024, 4 * 1024 * 1024, || {
        parse_expr_bp_inner(tokens, pos, min_bp)
    })
}

/// The actual Pratt loop behind the stack-growth guard of [`parse_expr_bp`].
fn parse_expr_bp_inner(
    tokens: &[SpannedToken],
    pos: &mut usize,
    min_bp: u8,
) -> Result<Expr, CompileError> {
    let mut lhs = parse_prefix(tokens, pos)?;

    loop {
        if *pos >= tokens.len() {
            break;
        }

        match &tokens[*pos].0 {
            Token::LBracket => {
                let span = tokens[*pos].1.span;
                *pos += 1;
                // Empty index `$a[]` — PHP's array-append marker. It has no read form (`echo
                // $a[];` is a hard error, matching PHP's "Cannot use [] for reading"), so it is
                // only legal here when immediately followed by a plain `=`, making the whole
                // `lhs[] = rhs` an assignment-expression target. Desugar it right away: there is
                // no `index` expression to build an `ArrayAccess` node from.
                if *pos < tokens.len() && tokens[*pos].0 == Token::RBracket {
                    *pos += 1; // consume ']'
                    if *pos >= tokens.len() || tokens[*pos].0 != Token::Assign {
                        return Err(CompileError::new(span, "Cannot use [] for reading"));
                    }
                    *pos += 1; // consume '='
                    // Right binding power 6 matches `assignment_bp`'s `(l_bp, r_bp) = (7, 6)` for
                    // `=`, keeping append's RHS precedence identical to ordinary assignment.
                    let rhs = parse_expr_bp(tokens, pos, 6)?;
                    lhs = build_append_assignment_expression(lhs, rhs, span)?;
                    continue;
                }
                let index = parse_expr(tokens, pos)?;
                if *pos >= tokens.len() || tokens[*pos].0 != Token::RBracket {
                    return Err(CompileError::new(span, "Expected ']'"));
                }
                *pos += 1;
                lhs = Expr::new(
                    ExprKind::ArrayAccess {
                        array: Box::new(lhs),
                        index: Box::new(index),
                    },
                    span,
                );
            }
            Token::DoubleColon => {
                // A `::` reaching the postfix loop has a dynamic (variable/expression) class
                // receiver — named-class static calls are fully parsed in the prefix parser.
                // `$cls::method(args)` / `$cls::$method(args)` desugars to
                // `call_user_func([$cls, $method], ...args)`, reusing the runtime dispatch path.
                let span = tokens[*pos].1.span;
                *pos += 1; // consume '::'
                if matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Class)) {
                    *pos += 1;
                    let span = crate::parser::expr::span_through_prev_token(tokens, *pos, span);
                    lhs = Expr::new(
                        ExprKind::ObjectClassName {
                            object: Box::new(lhs),
                        },
                        span,
                    );
                    continue;
                }
                // Track the bare-identifier member name so a `::CONST` (not followed by
                // `(`) can lower to a dynamic class-constant access instead of a call.
                let mut identifier_member: Option<String> = None;
                let member = match tokens
                    .get(*pos)
                    .map(|(token, metadata)| (token.clone(), metadata.span))
                {
                    Some((Token::Identifier(name), name_span)) => {
                        *pos += 1;
                        identifier_member = Some(name.clone());
                        Expr::new(ExprKind::StringLiteral(name), name_span)
                    }
                    Some((Token::Variable(name), var_span)) => {
                        *pos += 1;
                        Expr::new(ExprKind::Variable(name), var_span)
                    }
                    Some((Token::LBrace, _)) => {
                        *pos += 1;
                        let member = parse_expr(tokens, pos)?;
                        if *pos >= tokens.len() || tokens[*pos].0 != Token::RBrace {
                            return Err(CompileError::new(span, "Expected '}'"));
                        }
                        *pos += 1;
                        member
                    }
                    _ => {
                        return Err(CompileError::new(span, "Expected method name after '::'"));
                    }
                };
                if *pos < tokens.len() && tokens[*pos].0 == Token::LParen {
                    *pos += 1; // consume '('
                    if parse_first_class_callable_parens(tokens, pos)? {
                        // `$obj::$method(...)` / `$cls::method(...)` — a first-class callable bound
                        // to a runtime-named static/scoped method on a dynamic receiver. Reuse the
                        // same `[receiver, method]` callable as the call form, wrapped in a Closure.
                        lhs = build_dynamic_method_first_class_callable(lhs, member, span);
                        continue;
                    }
                    let dynamic_args = crate::parser::expr::parse_args(tokens, pos, span)?;
                    let span = crate::parser::expr::span_through_prev_token(tokens, *pos, span);
                    reject_named_args_in_dynamic_call(&dynamic_args, span)?;
                    let mut call_args =
                        vec![Expr::new(ExprKind::ArrayLiteral(vec![lhs, member]), span)];
                    call_args.extend(dynamic_args);
                    lhs = Expr::new(
                        ExprKind::FunctionCall {
                            name: crate::names::Name::unqualified("call_user_func"),
                            args: call_args,
                        },
                        span,
                    );
                } else if let Some(const_name) = identifier_member {
                    // `$obj::CONST` — a class constant accessed through an object/variable.
                    // The class is resolved from `object`'s static type in the checker.
                    lhs = Expr::new(
                        ExprKind::DynamicClassConstantAccess {
                            object: Box::new(lhs),
                            name: const_name,
                        },
                        span,
                    );
                } else {
                    return Err(CompileError::new(
                        span,
                        "Dynamic static property access (`$class::$prop`) is not supported yet",
                    ));
                }
            }
            Token::Arrow | Token::QuestionArrow => {
                let arrow_span = tokens[*pos].1.span;
                let nullsafe = tokens[*pos].0 == Token::QuestionArrow;
                *pos += 1;
                let member = match parse_object_member(tokens, pos, arrow_span, nullsafe)? {
                    ObjectMember::Named(member_name) => member_name,
                    ObjectMember::Dynamic(property) => {
                        if *pos < tokens.len() && tokens[*pos].0 == Token::LParen {
                            *pos += 1; // consume '('
                            if parse_first_class_callable_parens(tokens, pos)? {
                                // `$obj->$method(...)` — a first-class callable bound to a
                                // runtime-named instance method. Reuse the same `[$obj, $method]`
                                // callable as the call form, wrapped in a Closure. PHP forbids
                                // combining the nullsafe operator with Closure creation.
                                if nullsafe {
                                    return Err(CompileError::new(
                                        arrow_span,
                                        "Cannot combine nullsafe operator with Closure creation",
                                    ));
                                }
                                lhs = build_dynamic_method_first_class_callable(
                                    lhs, property, arrow_span,
                                );
                                continue;
                            }
                            let dynamic_args =
                                crate::parser::expr::parse_args(tokens, pos, arrow_span)?;
                            let arrow_span = crate::parser::expr::span_through_prev_token(tokens, *pos, arrow_span);
                            reject_named_args_in_dynamic_call(&dynamic_args, arrow_span)?;
                            if nullsafe {
                                lhs = Expr::new(
                                    ExprKind::NullsafeDynamicMethodCall {
                                        object: Box::new(lhs),
                                        method: Box::new(property),
                                        args: dynamic_args,
                                    },
                                    arrow_span,
                                );
                                continue;
                            }
                            // `$obj->$method(args)` reuses the runtime dynamic-dispatch path by
                            // desugaring to `call_user_func([$obj, $method], ...args)`.
                            let mut call_args = vec![Expr::new(
                                ExprKind::ArrayLiteral(vec![lhs, property]),
                                arrow_span,
                            )];
                            call_args.extend(dynamic_args);
                            lhs = Expr::new(
                                ExprKind::FunctionCall {
                                    name: crate::names::Name::unqualified("call_user_func"),
                                    args: call_args,
                                },
                                arrow_span,
                            );
                            continue;
                        }
                        lhs = Expr::new(
                            if nullsafe {
                                ExprKind::NullsafeDynamicPropertyAccess {
                                    object: Box::new(lhs),
                                    property: Box::new(property),
                                }
                            } else {
                                ExprKind::DynamicPropertyAccess {
                                    object: Box::new(lhs),
                                    property: Box::new(property),
                                }
                            },
                            arrow_span,
                        );
                        continue;
                    }
                };
                let member_name = member;
                if *pos < tokens.len() && tokens[*pos].0 == Token::LParen {
                    *pos += 1;
                    if parse_first_class_callable_parens(tokens, pos)? {
                        if nullsafe {
                            return Err(CompileError::new(
                                arrow_span,
                                "Cannot combine nullsafe operator with Closure creation",
                            ));
                        }
                        lhs = Expr::new(
                            ExprKind::FirstClassCallable(CallableTarget::Method {
                                object: Box::new(lhs),
                                method: member_name,
                            }),
                            arrow_span,
                        );
                    } else {
                        let args = parse_args(tokens, pos, arrow_span)?;
                        let arrow_span = crate::parser::expr::span_through_prev_token(tokens, *pos, arrow_span);
                        lhs = Expr::new(
                            if nullsafe {
                                ExprKind::NullsafeMethodCall {
                                    object: Box::new(lhs),
                                    method: member_name,
                                    args,
                                }
                            } else {
                                ExprKind::MethodCall {
                                    object: Box::new(lhs),
                                    method: member_name,
                                    args,
                                }
                            },
                            arrow_span,
                        );
                    }
                } else {
                    lhs = Expr::new(
                        if nullsafe {
                            ExprKind::NullsafePropertyAccess {
                                object: Box::new(lhs),
                                property: member_name,
                            }
                        } else {
                            ExprKind::PropertyAccess {
                                object: Box::new(lhs),
                                property: member_name,
                            }
                        },
                        arrow_span,
                    );
                }
            }
            Token::LParen => {
                if matches!(
                    lhs.kind,
                    ExprKind::ArrayAccess { .. }
                        | ExprKind::ExprCall { .. }
                        | ExprKind::ClosureCall { .. }
                        | ExprKind::FunctionCall { .. }
                        | ExprKind::MethodCall { .. }
                        | ExprKind::NullsafeMethodCall { .. }
                        | ExprKind::StaticMethodCall { .. }
                ) {
                    let call_span = tokens[*pos].1.span;
                    *pos += 1;
                    let args = parse_args(tokens, pos, call_span)?;
                    let call_span = crate::parser::expr::span_through_prev_token(tokens, *pos, call_span);
                    lhs = Expr::new(
                        ExprKind::ExprCall {
                            callee: Box::new(lhs),
                            args,
                        },
                        call_span,
                    );
                } else {
                    break;
                }
            }
            // Postfix `++`/`--` on a complex l-value (`$obj->p++`, `$a[$i]--`,
            // `$this->n++`). Bare-variable postfix is consumed earlier in the prefix
            // parser; a non-local target reaching here cannot be incremented by a
            // simple node, so desugar to the value-preserving `(L += 1) - 1` form,
            // which yields the OLD value like PHP while the compound assignment stores.
            Token::PlusPlus | Token::MinusMinus if is_non_local_assignment_target(&lhs) => {
                let increment = tokens[*pos].0 == Token::PlusPlus;
                let span = tokens[*pos].1.span;
                *pos += 1;
                lhs = build_incdec_value(lhs, increment, false, span);
            }
            _ => break,
        }
    }

    loop {
        if *pos >= tokens.len() {
            break;
        }

        if tokens[*pos].0 == Token::Question {
            let ternary_bp = 7;
            if ternary_bp < min_bp {
                break;
            }

            let span = tokens[*pos].1.span;
            *pos += 1;
            if *pos < tokens.len() && tokens[*pos].0 == Token::Colon {
                *pos += 1;
                let default = parse_expr_bp(tokens, pos, ternary_bp)?;
                lhs = Expr::new(
                    ExprKind::ShortTernary {
                        value: Box::new(lhs),
                        default: Box::new(default),
                    },
                    span,
                );
                continue;
            }

            let then_expr = parse_expr(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos].0 != Token::Colon {
                return Err(CompileError::new(span, "Expected ':' in ternary operator"));
            }
            *pos += 1;
            let else_expr = parse_expr_bp(tokens, pos, ternary_bp)?;
            lhs = Expr::new(
                ExprKind::Ternary {
                    condition: Box::new(lhs),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                },
                span,
            );
            continue;
        }

        if tokens[*pos].0 == Token::InstanceOf {
            let instanceof_bp = 35;
            if instanceof_bp < min_bp {
                break;
            }

            let span = tokens[*pos].1.span;
            *pos += 1;
            let target = parse_instanceof_target(tokens, pos, span)?;
            lhs = Expr::new(
                ExprKind::InstanceOf {
                    value: Box::new(lhs),
                    target,
                },
                span,
            );
            continue;
        }

        if let Some((op, l_bp, r_bp)) = assignment_bp(&tokens[*pos].0) {
            // `[$a, $b] = EXPR` in expression position: a simple positional list-destructuring
            // assignment (e.g. `if ([$a, $b] = $pairs ?? null)`). Only plain `=` with an
            // all-simple-`$variable` array-literal target qualifies; keyed/nested/non-variable
            // targets fall through to ordinary handling and its "Invalid assignment target" error.
            // A list target is a valid target, so — like an lvalue — it binds even below `min_bp`.
            if matches!(op, AssignmentOperator::Assign) {
                if let Some(vars) = simple_positional_list_vars(&lhs) {
                    let span = tokens[*pos].1.span;
                    *pos += 1;
                    let rhs = parse_expr_bp(tokens, pos, r_bp)?;
                    lhs = Expr::new(
                        ExprKind::ListUnpack {
                            vars,
                            value: Box::new(rhs),
                        },
                        span,
                    );
                    continue;
                }
            }

            // PHP binds `=` to the immediately-preceding lvalue even when the assignment sits in
            // the right operand of a higher-precedence operator: `false !== $x = f()` parses as
            // `false !== ($x = f())`, `1 + $b = 5` as `1 + ($b = 5)`, and `!$b = 7` as
            // `!($b = 7)`. So a below-`min_bp` assignment is still consumed when `lhs` is a valid
            // target; it only yields to the enclosing context (breaks) when `lhs` is not an
            // lvalue, which then surfaces as the invalid-target error just below.
            let lhs_is_target = is_assignment_expression_target(&lhs);
            if l_bp < min_bp && !lhs_is_target {
                break;
            }

            if !lhs_is_target {
                return Err(CompileError::new(lhs.span, "Invalid assignment target"));
            }

            let span = tokens[*pos].1.span;
            *pos += 1;
            let rhs = parse_expr_bp(tokens, pos, r_bp)?;
            // Widen only the END so the span covers through the value expression;
            // the start stays on the operator token, keeping diagnostics anchored.
            let span = span.merge(rhs.span);
            if is_non_local_assignment_target(&lhs) {
                let null_coalesce_assign = matches!(op, AssignmentOperator::NullCoalesce);

                let mut lowerer = AssignmentExpressionLowerer::new(span);
                let target = lowerer.stabilize_non_local_target(lhs, &rhs);
                let conditional_value_temp =
                    null_coalesce_assign.then(|| lowerer.reserve_value_temp());
                let rhs = if null_coalesce_assign {
                    rhs
                } else {
                    lowerer.bind_value(&target, rhs)
                };
                let (value, result_target) = match op {
                    AssignmentOperator::Assign => (rhs.clone(), rhs),
                    AssignmentOperator::NullCoalesce => {
                        let value = assignment_value(
                            target.clone(),
                            AssignmentOperator::NullCoalesce,
                            rhs,
                            span,
                        );
                        (value, target.clone())
                    }
                    AssignmentOperator::Compound(op) => {
                        let value = assignment_value(
                            target.clone(),
                            AssignmentOperator::Compound(op),
                            rhs,
                            span,
                        );
                        let result_value = lowerer.bind_result_value(value);
                        (result_value.clone(), result_value)
                    }
                };
                let prelude = lowerer.finish();
                lhs = Expr::new(
                    ExprKind::Assignment {
                        target: Box::new(target.clone()),
                        value: Box::new(value),
                        result_target: Some(Box::new(result_target)),
                        prelude,
                        conditional_value_temp,
                    },
                    span,
                );
            } else {
                let value = match op {
                    AssignmentOperator::Assign => rhs,
                    op => assignment_value(lhs.clone(), op, rhs, span),
                };
                lhs = Expr::new(
                    ExprKind::Assignment {
                        target: Box::new(lhs),
                        value: Box::new(value),
                        result_target: None,
                        prelude: Vec::new(),
                        conditional_value_temp: None,
                    },
                    span,
                );
            }
            continue;
        }

        // PHP 8.5 pipe operator `|>`: left-associative, BP (24, 25) — sits between
        // comparisons (23, 24) and shifts (25, 26), matching php-src/RFC precedence
        // (lower than `.`, shifts, `+`/`-`, higher than comparisons, `??`, ternary,
        // logical, and assignment). Built as a dedicated `ExprKind::Pipe` node, not a
        // `BinOp`, so that LHS-first evaluation order and pipe-specific diagnostics are
        // preserved through later passes.
        if matches!(tokens[*pos].0, Token::PipeArrow) {
            let (l_bp, r_bp) = (24u8, 25u8);
            if l_bp < min_bp {
                break;
            }
            let span = tokens[*pos].1.span;
            *pos += 1;
            if starts_unparenthesized_arrow_function(tokens, *pos) {
                return Err(CompileError::new(
                    tokens[*pos].1.span,
                    "Arrow functions used as pipe targets must be parenthesized",
                ));
            }
            let rhs = parse_expr_bp(tokens, pos, r_bp)?;
            lhs = Expr::new(
                ExprKind::Pipe {
                    value: Box::new(lhs),
                    callable: Box::new(rhs),
                },
                span,
            );
            continue;
        }

        let (op, l_bp, r_bp) = match infix_bp(&tokens[*pos].0) {
            Some(binding) => binding,
            None => break,
        };

        if l_bp < min_bp {
            break;
        }

        let span = tokens[*pos].1.span;
        *pos += 1;
        let rhs = parse_expr_bp(tokens, pos, r_bp)?;
        // Widen only the END through the right operand; the start stays on the
        // operator token, keeping diagnostics anchored.
        let span = span.merge(rhs.span);
        if op == BinOp::NullCoalesce {
            lhs = Expr::new(
                ExprKind::NullCoalesce {
                    value: Box::new(lhs),
                    default: Box::new(rhs),
                },
                span,
            );
        } else {
            lhs = Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(lhs),
                    op,
                    right: Box::new(rhs),
                },
                span,
            );
        }
    }

    Ok(lhs)
}

/// Returns `true` if the token at `pos` is an arrow function (`fn`) or a static
/// arrow function (`static fn`) that is not wrapped in parentheses.
///
/// Used by pipe operator (`|>`) parsing to reject unparenthesized arrow functions
/// as pipe targets, since PHP requires them to be parenthesized.
fn starts_unparenthesized_arrow_function(tokens: &[SpannedToken], pos: usize) -> bool {
    matches!(tokens.get(pos).map(|(token, _)| token), Some(Token::Fn))
        || (matches!(tokens.get(pos).map(|(token, _)| token), Some(Token::Static))
            && matches!(tokens.get(pos + 1).map(|(token, _)| token), Some(Token::Fn)))
}

/// Represents the member accessed after an object operator (`->` or `?->`).
///
/// `Named` holds a static identifier string. `Dynamic` holds an expression
/// inside braces (`{$expr}`) used for computed property/method names.
pub(super) enum ObjectMember {
    Named(String),
    Dynamic(Expr),
}

/// Parses the member or property name following an object operator (`->` or `?->`).
///
/// Handles three forms:
/// - Static identifier: `->foo`
/// - Dynamic expression in braces: `->{$expr}`
/// - PHP 8 semi-reserved keywords as member names: any keyword (e.g. `->self`, `->parent`,
///   `->static`, `->class`, `->list`, `->print`) is accepted via the shared bareword mapper.
///
/// Returns `ObjectMember::Named` for identifiers and keywords, or
/// `ObjectMember::Dynamic` for brace-enclosed expressions.
///
/// # Inputs
/// - `arrow_span`: span of the `->` or `?->` token, used for error reporting
/// - `nullsafe`: whether the operator was `?->` (changes error messages)
pub(super) fn parse_object_member(
    tokens: &[SpannedToken],
    pos: &mut usize,
    arrow_span: Span,
    nullsafe: bool,
) -> Result<ObjectMember, CompileError> {
    if let Some((Token::LBrace, _)) = tokens.get(*pos) {
        *pos += 1;
        let property = parse_expr(tokens, pos)?;
        if *pos >= tokens.len() || tokens[*pos].0 != Token::RBrace {
            return Err(CompileError::new(arrow_span, "Expected '}'"));
        }
        *pos += 1;
        return Ok(ObjectMember::Dynamic(property));
    }
    // `->$var` selects a member whose name is the runtime value of `$var`.
    if let Some((Token::Variable(name), var_span)) = tokens
        .get(*pos)
        .map(|(token, metadata)| (token.clone(), metadata.span))
    {
        *pos += 1;
        return Ok(ObjectMember::Dynamic(Expr::new(
            ExprKind::Variable(name),
            var_span,
        )));
    }
    // PHP 8 allows identifiers and any semi-reserved keyword as a member name after `->`/`?->`.
    if let Some(name) = tokens
        .get(*pos)
        .and_then(|(token, metadata)| {
            crate::parser::keyword_name::bareword_name_from_token(token, metadata)
        })
    {
        *pos += 1;
        return Ok(ObjectMember::Named(name));
    }
    Err(CompileError::new(
        arrow_span,
        if nullsafe {
            "Expected property or method name after '?->'"
        } else {
            "Expected property or method name after '->'"
        },
    ))
}

/// Represents the specific assignment operator encountered during parsing.
#[derive(Debug, Clone, PartialEq)]
enum AssignmentOperator {
    Assign,
    Compound(BinOp),
    NullCoalesce,
}

/// Rejects named arguments in a dynamic call (`$obj->$m(...)`, `$cls::$m(...)`, `C::$m(...)`).
///
/// The callee method name is a runtime value, so its parameter names are unknown at compile time
/// and named arguments cannot be matched to positions. Emits a clear diagnostic instead of letting
/// the internal `call_user_func` desugaring surface a confusing error.
pub(in crate::parser) fn reject_named_args_in_dynamic_call(
    args: &[Expr],
    span: Span,
) -> Result<(), CompileError> {
    if args
        .iter()
        .any(|arg| matches!(arg.kind, ExprKind::NamedArg { .. }))
    {
        return Err(CompileError::new(
            span,
            "Named arguments are not supported in dynamic calls; the target method is not known at compile time",
        ));
    }
    Ok(())
}

/// Looks up assignment operator binding power.
///
/// Returns `None` for non-assignment tokens. For recognized tokens, returns
/// `(AssignmentOperator, left_bp, right_bp)` with binding powers `(7, 6)`,
/// enforcing right-associativity (rhs binds tighter than lhs).
fn assignment_bp(token: &Token) -> Option<(AssignmentOperator, u8, u8)> {
    let op = match token {
        Token::Assign => AssignmentOperator::Assign,
        Token::PlusAssign => AssignmentOperator::Compound(BinOp::Add),
        Token::MinusAssign => AssignmentOperator::Compound(BinOp::Sub),
        Token::StarAssign => AssignmentOperator::Compound(BinOp::Mul),
        Token::StarStarAssign => AssignmentOperator::Compound(BinOp::Pow),
        Token::SlashAssign => AssignmentOperator::Compound(BinOp::Div),
        Token::PercentAssign => AssignmentOperator::Compound(BinOp::Mod),
        Token::DotAssign => AssignmentOperator::Compound(BinOp::Concat),
        Token::AmpAssign => AssignmentOperator::Compound(BinOp::BitAnd),
        Token::PipeAssign => AssignmentOperator::Compound(BinOp::BitOr),
        Token::CaretAssign => AssignmentOperator::Compound(BinOp::BitXor),
        Token::LessLessAssign => AssignmentOperator::Compound(BinOp::ShiftLeft),
        Token::GreaterGreaterAssign => AssignmentOperator::Compound(BinOp::ShiftRight),
        Token::QuestionQuestionAssign => AssignmentOperator::NullCoalesce,
        _ => return None,
    };
    Some((op, 7, 6))
}

/// Computes the value expression for an assignment operator applied to `target`
/// and `rhs`.
///
/// - `Assign`: returns `rhs`
/// - `Compound(op)`: returns `BinaryOp { left: target, op, right: rhs }`
/// - `NullCoalesce`: returns `NullCoalesce { value: target, default: rhs }`
///
/// The `target` is consumed and placed on the left side of the derived expression.
fn assignment_value(target: Expr, op: AssignmentOperator, rhs: Expr, span: Span) -> Expr {
    match op {
        AssignmentOperator::Assign => rhs,
        AssignmentOperator::Compound(op) => Expr::new(
            ExprKind::BinaryOp {
                left: Box::new(target),
                op,
                right: Box::new(rhs),
            },
            span,
        ),
        AssignmentOperator::NullCoalesce => Expr::new(
            ExprKind::NullCoalesce {
                value: Box::new(target),
                default: Box::new(rhs),
            },
            span,
        ),
    }
}

/// Builds an `ExprKind::Assignment` for `target <op> rhs`.
///
/// Non-local targets (property/array/static access) are stabilized so any
/// side-effecting receiver or index is evaluated exactly once, and the result
/// value is bound so the assignment expression yields the PHP-correct value.
/// Local (bare-variable) targets take the simple path with no prelude. Shared by
/// the assignment-operator branch of `parse_expr_bp` and the increment/decrement
/// desugaring in `build_incdec_value`.
fn build_assignment_expression(
    target: Expr,
    op: AssignmentOperator,
    rhs: Expr,
    span: Span,
) -> Expr {
    if is_non_local_assignment_target(&target) {
        let null_coalesce_assign = matches!(op, AssignmentOperator::NullCoalesce);

        let mut lowerer = AssignmentExpressionLowerer::new(span);
        let target = lowerer.stabilize_non_local_target(target, &rhs);
        let conditional_value_temp = null_coalesce_assign.then(|| lowerer.reserve_value_temp());
        let rhs = if null_coalesce_assign {
            rhs
        } else {
            lowerer.bind_value(&target, rhs)
        };
        let (value, result_target) = match op {
            AssignmentOperator::Assign => (rhs.clone(), rhs),
            AssignmentOperator::NullCoalesce => {
                let value =
                    assignment_value(target.clone(), AssignmentOperator::NullCoalesce, rhs, span);
                (value, target.clone())
            }
            AssignmentOperator::Compound(op) => {
                let value =
                    assignment_value(target.clone(), AssignmentOperator::Compound(op), rhs, span);
                let result_value = lowerer.bind_result_value(value);
                (result_value.clone(), result_value)
            }
        };
        let prelude = lowerer.finish();
        Expr::new(
            ExprKind::Assignment {
                target: Box::new(target.clone()),
                value: Box::new(value),
                result_target: Some(Box::new(result_target)),
                prelude,
                conditional_value_temp,
            },
            span,
        )
    } else {
        let value = match op {
            AssignmentOperator::Assign => rhs,
            op => assignment_value(target.clone(), op, rhs, span),
        };
        Expr::new(
            ExprKind::Assignment {
                target: Box::new(target),
                value: Box::new(value),
                result_target: None,
                prelude: Vec::new(),
                conditional_value_temp: None,
            },
            span,
        )
    }
}

/// Desugars an increment/decrement of a complex l-value (`$obj->p`, `$a[$i]`,
/// `$this->n`, static property) into compound-assignment form, since elephc has no
/// in-place increment node for non-local targets.
///
/// `++L` / `--L` become `(L += 1)` / `(L -= 1)`, which yield the NEW value. The
/// postfix forms `L++` / `L--` become `(L += 1) - 1` / `(L -= 1) + 1`, which yield
/// the OLD value while still performing the store. The compound assignment built by
/// [`build_assignment_expression`] stabilizes the target, so the receiver/index is
/// evaluated exactly once, matching PHP. The result is numeric, which covers every
/// supported use (PHP's string-increment magic applies only to bare variables, which
/// keep their dedicated `PreIncrement`/`PostIncrement` nodes).
pub(super) fn build_incdec_value(target: Expr, increment: bool, prefix: bool, span: Span) -> Expr {
    let bin_op = if increment { BinOp::Add } else { BinOp::Sub };
    let one = Expr::new(ExprKind::IntLiteral(1), span);
    let assignment =
        build_assignment_expression(target, AssignmentOperator::Compound(bin_op), one, span);
    if prefix {
        return assignment;
    }
    // Postfix yields the old value: undo the in-expression increment numerically.
    let undo_op = if increment { BinOp::Sub } else { BinOp::Add };
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(assignment),
            op: undo_op,
            right: Box::new(Expr::new(ExprKind::IntLiteral(1), span)),
        },
        span,
    )
}

/// Parses the target of an `instanceof` operator.
///
/// Handles the PHP 8.0 class-name forms and the dynamic expression forms:
/// - bare `self`, `parent`, `static` keyword → `InstanceOfTarget::Name` (the checker's
///   instanceof narrowing depends on the Name form, so these must stay Name)
/// - static-property RHS (`D::$proto`, `self::$p`, `static::$p`, `parent::$p`,
///   `\App\D::$p`, incl. dynamic `::${…}`), detected by the non-consuming lookahead
///   [`instanceof_static_property_lookahead`] → parsed as `Expr`
/// - `$this`, a variable, or a parenthesized expression → parsed as `Expr`
/// - class/interface name → resolved via `parse_name` into a qualified `Name`
///
/// The dynamic forms are parsed with `min_bp = 36` (above comparisons) so the postfix
/// loop still folds `->prop` / `[idx]` chains into the target, matching PHP's
/// `new_variable` grammar for the instanceof RHS.
fn parse_instanceof_target(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<InstanceOfTarget, CompileError> {
    // A static-property RHS starts like a class name (or `self`/`static`/`parent`), so
    // it must be routed to the expression parser BEFORE the Name/keyword arms below.
    // The lookahead never consumes tokens, so bare-name forms are left untouched.
    if instanceof_static_property_lookahead(tokens, *pos) {
        let target = parse_expr_bp(tokens, pos, 36)?;
        return Ok(InstanceOfTarget::Expr(Box::new(target)));
    }
    match tokens.get(*pos).map(|(token, _)| token) {
        Some(Token::Self_) => {
            *pos += 1;
            Ok(InstanceOfTarget::Name(Name::unqualified("self")))
        }
        Some(Token::Parent) => {
            *pos += 1;
            Ok(InstanceOfTarget::Name(Name::unqualified("parent")))
        }
        Some(Token::Static) => {
            *pos += 1;
            Ok(InstanceOfTarget::Name(Name::unqualified("static")))
        }
        Some(Token::This) | Some(Token::Variable(_)) | Some(Token::LParen) => {
            let target = parse_expr_bp(tokens, pos, 36)?;
            Ok(InstanceOfTarget::Expr(Box::new(target)))
        }
        _ => parse_name(
            tokens,
            pos,
            span,
            "Expected class or interface name after 'instanceof'",
        )
        .map(InstanceOfTarget::Name),
    }
}

/// Bounded, non-consuming lookahead deciding whether the `instanceof` RHS at `pos` is a
/// static-property access (`D::$proto`, `\App\D::$p`, `self::$p`, `static::$p`,
/// `parent::$p`, or the dynamic-name forms `D::${…}` / `D::$$v`).
///
/// Returns `true` only when a `self`/`static`/`parent` keyword or a (possibly
/// backslash-qualified) identifier name is immediately followed by `::` and then a
/// `$prop` variable or a bare `$` (the dynamic static-property marker). `Foo::CONST`
/// (identifier member) stays `false` so it keeps producing a loud parse error, matching
/// PHP where a class constant is not a valid `instanceof` RHS. Never mutates `pos`.
fn instanceof_static_property_lookahead(tokens: &[SpannedToken], pos: usize) -> bool {
    let mut i = pos;
    match tokens.get(i).map(|(token, _)| token) {
        Some(Token::Self_) | Some(Token::Static) | Some(Token::Parent) => {
            i += 1;
        }
        Some(Token::Identifier(_)) | Some(Token::Backslash) => {
            // Skip a possibly qualified name: `\`? Identifier (`\` Identifier)*.
            if matches!(tokens.get(i).map(|(token, _)| token), Some(Token::Backslash)) {
                i += 1;
            }
            loop {
                match tokens.get(i).map(|(token, _)| token) {
                    Some(Token::Identifier(_)) => i += 1,
                    _ => return false,
                }
                if matches!(tokens.get(i).map(|(token, _)| token), Some(Token::Backslash)) {
                    i += 1;
                    continue;
                }
                break;
            }
        }
        _ => return false,
    }
    if !matches!(
        tokens.get(i).map(|(token, _)| token),
        Some(Token::DoubleColon)
    ) {
        return false;
    }
    i += 1;
    matches!(
        tokens.get(i).map(|(token, _)| token),
        Some(Token::Variable(_)) | Some(Token::Dollar)
    )
}

/// Looks up binary operator binding power for Pratt parsing.
///
/// Returns `None` for tokens that are not binary operators. For recognized tokens,
/// returns `(BinOp, left_bp, right_bp)` where the left binding power is used
/// when parsing the left operand and the right binding power when parsing the
/// right operand. The asymmetric pair encodes associativity: left-associative
/// operators have `l_bp < r_bp` so the right operand is parsed with a tighter
/// bp threshold, while right-associative operators (e.g., `**`) have
/// `l_bp > r_bp` so the left side binds tighter.
///
/// Precedence order (lowest to highest): `or` (1) < `xor` (3) < `and` (5)
/// < `??` (9) < `||` (11) < `&&` (13) < `|` (15) < `^` (17) < `&` (19)
/// < `==`/`!=`/`===`/`!==` (21) < `<`/`>`/`<=`/`>=`/`<=>` (23)
/// < `<<`/`>>` (25) < `.` (27) < `+`/`-` (29) < `*`/`/`/`%` (31)
/// < `**` (37 right-assoc)
fn infix_bp(token: &Token) -> Option<(BinOp, u8, u8)> {
    match token {
        Token::Or => Some((BinOp::Or, 1, 2)),
        Token::Xor => Some((BinOp::Xor, 3, 4)),
        Token::And => Some((BinOp::And, 5, 6)),
        Token::QuestionQuestion => Some((BinOp::NullCoalesce, 9, 8)),
        Token::OrOr => Some((BinOp::Or, 11, 12)),
        Token::AndAnd => Some((BinOp::And, 13, 14)),
        Token::Pipe => Some((BinOp::BitOr, 15, 16)),
        Token::Caret => Some((BinOp::BitXor, 17, 18)),
        Token::Ampersand => Some((BinOp::BitAnd, 19, 20)),
        Token::EqualEqual => Some((BinOp::Eq, 21, 22)),
        Token::NotEqual => Some((BinOp::NotEq, 21, 22)),
        Token::EqualEqualEqual => Some((BinOp::StrictEq, 21, 22)),
        Token::NotEqualEqual => Some((BinOp::StrictNotEq, 21, 22)),
        Token::Less => Some((BinOp::Lt, 23, 24)),
        Token::Greater => Some((BinOp::Gt, 23, 24)),
        Token::LessEqual => Some((BinOp::LtEq, 23, 24)),
        Token::GreaterEqual => Some((BinOp::GtEq, 23, 24)),
        Token::Spaceship => Some((BinOp::Spaceship, 23, 24)),
        Token::LessLess => Some((BinOp::ShiftLeft, 25, 26)),
        Token::GreaterGreater => Some((BinOp::ShiftRight, 25, 26)),
        Token::Dot => Some((BinOp::Concat, 27, 28)),
        Token::Plus => Some((BinOp::Add, 29, 30)),
        Token::Minus => Some((BinOp::Sub, 29, 30)),
        Token::Star => Some((BinOp::Mul, 31, 32)),
        Token::Slash => Some((BinOp::Div, 31, 32)),
        Token::Percent => Some((BinOp::Mod, 31, 32)),
        Token::StarStar => Some((BinOp::Pow, 37, 36)),
        _ => None,
    }
}
