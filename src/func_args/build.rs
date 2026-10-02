//! Purpose:
//! Builds the plain-PHP expressions that replace `func_num_args()`, `func_get_args()` and
//! `func_get_arg($position)` inside a function scope that received the hidden variadic
//! parameter `mixed ...$__elephc_func_args` or the source-declared variadic.
//!
//! Called from:
//! - `crate::func_args::walk::Rewriter`.
//!
//! Key details:
//! - The reconstruction reads regular parameter variables at the call point and appends
//!   the variadic tail. A scope with optional parameters reads the count its caller recorded
//!   at the end of the tail (`super::ARGC_MARKER_PREFIX`); a call that could not record it
//!   gets the conservative reconstruction, which counts every declared parameter.
//! - `func_get_args()` reports the *current* values of the parameter variables, not the
//!   values originally passed (verified against PHP 8.4: reassigning a parameter, or
//!   writing through a by-reference parameter, changes what `func_get_args()` returns).
//!   Reading the parameter variables at the call point reproduces that exactly.
//! - The array is rebuilt at each use because PHP returns a fresh array every time.
//! - `func_get_arg()` raises `ValueError` — not `ArgumentCountError` — for both a negative
//!   position and a position at or past the argument count, with php-src's two distinct
//!   messages.

use crate::names::{Name, NameKind};
use crate::parser::ast::{BinOp, CastType, Expr, ExprKind};
use crate::span::Span;

use super::{IntrospectionCall, ARGC_MARKER_PREFIX, POSITION_TEMP};

/// php-src's message when `func_get_arg()` is given a negative position.
const NEGATIVE_POSITION_MESSAGE: &str =
    "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0";

/// php-src's message when `func_get_arg()` is given a position at or past the number of
/// arguments the current call actually passed.
const OUT_OF_RANGE_POSITION_MESSAGE: &str =
    "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function";

/// Builds the replacement expression for one introspection call.
///
/// `param_names` lists the scope's declared regular parameters in order; `args` is the
/// (already rewritten) call-site argument list, which is empty except for
/// `func_get_arg($position)`. `argc_marker` is set for a scope with optional parameters, whose
/// callers record the count they really passed at the end of the hidden tail.
pub(super) fn replacement(
    call: IntrospectionCall,
    param_names: &[String],
    variadic_name: &str,
    args: &[Expr],
    span: Span,
    argc_marker: bool,
) -> ExprKind {
    match call {
        IntrospectionCall::NumArgs => {
            count_expr(param_names, variadic_name, span, argc_marker).kind
        }
        IntrospectionCall::GetArgs => {
            list_expr(param_names, variadic_name, span, argc_marker).kind
        }
        IntrospectionCall::GetArg => {
            get_arg_expr(param_names, variadic_name, &args[0], span, argc_marker)
        }
    }
}

/// The argument count, read from the caller's marker when the scope has one.
fn count_expr(param_names: &[String], variadic_name: &str, span: Span, argc_marker: bool) -> Expr {
    if !argc_marker {
        return argc_expr(param_names, variadic_name, span);
    }
    ternary(
        marker_present(variadic_name, span),
        marker_count(variadic_name, span),
        argc_expr(param_names, variadic_name, span),
        span,
    )
}

/// The argument list, cut to what the caller passed when the scope has a marker.
///
/// A caller only records its count when it passed FEWER arguments than the scope declares
/// regular parameters -- with all of them passed, the conservative reconstruction is exact --
/// so a marked tail holds nothing else and the list is a prefix of the parameters:
///
/// ```text
/// <marker present>
///     ? (<count> === 0 ? [] : (<count> === 1 ? [$p0] : … [$p0, …, $pN-2]))
///     : [$p0, …, ...$tail]
/// ```
///
/// An omitted optional parameter still holds its default, which is not an argument PHP
/// reports. The prefixes are spelled out rather than sliced because `array_slice` of a mixed
/// array is a prelude-backed builtin this late pass cannot pull in.
fn list_expr(param_names: &[String], variadic_name: &str, span: Span, argc_marker: bool) -> Expr {
    if !argc_marker || param_names.is_empty() {
        return args_array_expr(param_names, variadic_name, span);
    }
    let prefix = |len: usize| {
        Expr::new(
            ExprKind::ArrayLiteral(
                param_names[..len]
                    .iter()
                    .map(|name| Expr::new(ExprKind::Variable(name.clone()), span))
                    .collect(),
            ),
            span,
        )
    };
    let last = param_names.len() - 1;
    let mut prefixes = prefix(last);
    for len in (0..last).rev() {
        prefixes = ternary(
            Expr::new(
                ExprKind::BinaryOp {
                    left: Box::new(marker_count(variadic_name, span)),
                    op: BinOp::StrictEq,
                    right: Box::new(int_literal(len as i64, span)),
                },
                span,
            ),
            prefix(len),
            prefixes,
            span,
        );
    }
    ternary(
        marker_present(variadic_name, span),
        prefixes,
        args_array_expr(param_names, variadic_name, span),
        span,
    )
}

/// Builds `$tail[0]`, where a marked tail keeps its marker.
fn tail_marker(variadic_name: &str, span: Span) -> Expr {
    Expr::new(
        ExprKind::ArrayAccess {
            array: Box::new(args_var(variadic_name, span)),
            index: Box::new(int_literal(0, span)),
        },
        span,
    )
}

/// Builds `count($tail) === 1 && is_string($tail[0]) && str_starts_with($tail[0], <prefix>)`.
fn marker_present(variadic_name: &str, span: Span) -> Expr {
    let single = Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(count_call(args_var(variadic_name, span), span)),
            op: BinOp::StrictEq,
            right: Box::new(int_literal(1, span)),
        },
        span,
    );
    let is_string = builtin_call("is_string", vec![tail_marker(variadic_name, span)], span);
    let has_prefix = builtin_call(
        "str_starts_with",
        vec![
            tail_marker(variadic_name, span),
            Expr::new(ExprKind::StringLiteral(ARGC_MARKER_PREFIX.to_string()), span),
        ],
        span,
    );
    and(and(single, is_string, span), has_prefix, span)
}

/// Builds `(int) substr($tail[0], <prefix length>)`, the count the caller recorded.
fn marker_count(variadic_name: &str, span: Span) -> Expr {
    Expr::new(
        ExprKind::Cast {
            target: CastType::Int,
            expr: Box::new(builtin_call(
                "substr",
                vec![
                    tail_marker(variadic_name, span),
                    int_literal(ARGC_MARKER_PREFIX.len() as i64, span),
                ],
                span,
            )),
        },
        span,
    )
}

/// Builds `<left> && <right>`.
fn and(left: Expr, right: Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(left),
            op: BinOp::And,
            right: Box::new(right),
        },
        span,
    )
}

/// Builds `<condition> ? <then> : <otherwise>`.
fn ternary(condition: Expr, then_expr: Expr, otherwise: Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::Ternary {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(otherwise),
        },
        span,
    )
}

/// Builds an integer literal.
fn int_literal(value: i64, span: Span) -> Expr {
    Expr::new(ExprKind::IntLiteral(value), span)
}

/// Builds a call to a global builtin, named fully qualified because this AST is synthesized
/// after name resolution.
///
/// The call node itself gets the dummy span, never the introspection call's: builtin result
/// types are looked up by span, and several synthesized calls on one span each overwrote the
/// others' entry -- `count()` inside `func_get_args()` came back typed as the `array` the
/// whole expression has, and its integer was released as a heap pointer.
fn builtin_call(name: &str, args: Vec<Expr>, _span: Span) -> Expr {
    Expr::new(
        ExprKind::FunctionCall {
            name: Name::from_parts(NameKind::FullyQualified, vec![name.to_string()]),
            args,
        },
        Span::dummy(),
    )
}

/// Builds `<declared count> + count($__elephc_func_args)`, or just the `count()` call when
/// the scope declares no regular parameters.
fn argc_expr(param_names: &[String], variadic_name: &str, span: Span) -> Expr {
    let surplus = count_call(args_var(variadic_name, span), span);
    if param_names.is_empty() {
        return surplus;
    }
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(Expr::new(
                ExprKind::IntLiteral(param_names.len() as i64),
                span,
            )),
            op: BinOp::Add,
            right: Box::new(surplus),
        },
        span,
    )
}

/// Builds `[$p0, …, $pN-1, ...$__elephc_func_args]`, the full argument list in call order.
///
/// The array literal is fresh at every use, matching PHP's copy semantics, and the spread
/// of the hidden variadic keeps the surplus arguments renumbered from `N`.
fn args_array_expr(param_names: &[String], variadic_name: &str, span: Span) -> Expr {
    let mut elements: Vec<Expr> = param_names
        .iter()
        .map(|name| Expr::new(ExprKind::Variable(name.clone()), span))
        .collect();
    elements.push(Expr::new(
        ExprKind::Spread(Box::new(args_var(variadic_name, span))),
        span,
    ));
    Expr::new(ExprKind::ArrayLiteral(elements), span)
}

/// Builds the range-checked indexed read behind `func_get_arg($position)`:
///
/// ```text
/// $position < 0
///     ? throw new \ValueError(<negative message>)
///     : ($position < <argc> ? <args>[$position] : throw new \ValueError(<range message>))
/// ```
///
/// The position expression is bound to a hidden local first unless it is already
/// side-effect free, so a call such as `func_get_arg($i++)` evaluates its operand once.
fn get_arg_expr(
    param_names: &[String],
    variadic_name: &str,
    position: &Expr,
    span: Span,
    argc_marker: bool,
) -> ExprKind {
    let (first_read, later_read) = position_reads(position, span);
    ExprKind::Ternary {
        condition: Box::new(less_than(
            first_read,
            Expr::new(ExprKind::IntLiteral(0), span),
            span,
        )),
        then_expr: Box::new(throw_value_error(NEGATIVE_POSITION_MESSAGE, span)),
        else_expr: Box::new(Expr::new(
            ExprKind::Ternary {
                condition: Box::new(less_than(
                    later_read.clone(),
                    count_expr(param_names, variadic_name, span, argc_marker),
                    span,
                )),
                then_expr: Box::new(Expr::new(
                    ExprKind::ArrayAccess {
                        array: Box::new(list_expr(param_names, variadic_name, span, argc_marker)),
                        index: Box::new(later_read),
                    },
                    span,
                )),
                else_expr: Box::new(throw_value_error(OUT_OF_RANGE_POSITION_MESSAGE, span)),
            },
            span,
        )),
    }
}

/// Returns the two expressions that read the requested position: the first one is
/// evaluated once at the start of the range check, the second is re-read afterwards.
///
/// A literal or a plain variable is safe to re-evaluate, so it is used directly and no
/// hidden local is introduced. Anything else is bound to `$__elephc_func_arg_pos` by the
/// first read (an assignment expression yields the assigned value in PHP) and re-read from
/// that local, which preserves single evaluation of the operand's side effects.
fn position_reads(position: &Expr, span: Span) -> (Expr, Expr) {
    if matches!(
        position.kind,
        ExprKind::IntLiteral(_) | ExprKind::Variable(_)
    ) {
        return (position.clone(), position.clone());
    }
    let temp = Expr::new(ExprKind::Variable(POSITION_TEMP.to_string()), span);
    let bind = Expr::new(
        ExprKind::Assignment {
            target: Box::new(temp.clone()),
            value: Box::new(position.clone()),
            result_target: None,
            prelude: Vec::new(),
            conditional_value_temp: None,
        },
        span,
    );
    (bind, temp)
}

/// Builds a read of the variadic array that backs argument introspection in this scope.
fn args_var(name: &str, span: Span) -> Expr {
    Expr::new(ExprKind::Variable(name.to_string()), span)
}

/// Builds `count(<value>)`.
fn count_call(value: Expr, span: Span) -> Expr {
    // This AST is synthesized after name resolution, so identify the global builtin explicitly
    // instead of leaving a fresh unqualified call unresolved.
    builtin_call("count", vec![value], span)
}

/// Builds `<left> < <right>`.
fn less_than(left: Expr, right: Expr, span: Span) -> Expr {
    Expr::new(
        ExprKind::BinaryOp {
            left: Box::new(left),
            op: BinOp::Lt,
            right: Box::new(right),
        },
        span,
    )
}

/// Builds `throw new \ValueError(<message>)` as an expression, PHP 8's throw-expression
/// form, so it can sit in a ternary branch.
fn throw_value_error(message: &str, span: Span) -> Expr {
    Expr::new(
        ExprKind::Throw(Box::new(Expr::new(
            ExprKind::NewObject {
                class_name: Name::from_parts(NameKind::FullyQualified, vec!["ValueError".to_string()]),
                args: vec![Expr::new(ExprKind::StringLiteral(message.to_string()), span)],
            },
            span,
        ))),
        span,
    )
}
