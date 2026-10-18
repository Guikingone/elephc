//! Purpose:
//! Describes PHP weak scalar binding for runtime values passed to declared unions.
//!
//! Called from:
//! - Selected enum method runtime argument and variadic-element lowering.
//!
//! Key details:
//! - Exact union members win before numeric, string and bool coercion priority.
//! - Numeric strings choose int or float through the existing PHP arithmetic semantics.

use super::*;
use crate::parser::ast::{BinOp, InstanceOfTarget};
use crate::span::Span;

/// Builds a lazy binding expression over an already captured, validated runtime argument.
pub(crate) fn runtime_scalar_union_binding(expected: &PhpType, value: &Expr) -> Option<Expr> {
    let PhpType::Union(members) = expected else { return None; };
    if members.contains(&PhpType::Mixed) { return None; }
    let span = value.span;
    let exact = members.iter().map(|member| exact_member(member, value))
        .collect::<Option<Vec<_>>>()?.into_iter().reduce(|a, b| either(a, b, span))?;
    let mut conversions = Vec::new();
    let numeric = either(call("is_numeric", value), call("is_bool", value), span);
    if members.contains(&PhpType::Int) && members.contains(&PhpType::Float) {
        // Arithmetic selects the PHP numeric kind, including exponent/overflow spellings.
        // Cast the original value afterwards: adding zero alone would erase negative zero.
        let numeric_value = Expr::new(ExprKind::BinaryOp {
            left: Box::new(value.clone()), op: BinOp::Add,
            right: Box::new(Expr::new(ExprKind::IntLiteral(0), span)),
        }, span);
        let converted = select(call("is_int", &numeric_value),
            cast(CastType::Int, value), cast(CastType::Float, value), span);
        conversions.push((numeric, converted));
    } else if members.contains(&PhpType::Int) {
        conversions.push((numeric, cast(CastType::Int, value)));
    } else if members.contains(&PhpType::Float) {
        conversions.push((numeric, cast(CastType::Float, value)));
    }
    if members.contains(&PhpType::Str) {
        let stringable = Expr::new(ExprKind::InstanceOf {
            value: Box::new(value.clone()),
            target: InstanceOfTarget::Name(Name::from_parts(
                crate::names::NameKind::FullyQualified, vec!["Stringable".into()],
            )),
        }, span);
        conversions.push((either(call("is_scalar", value), stringable, span), cast(CastType::String, value)));
    }
    if members.contains(&PhpType::Bool) {
        conversions.push((call("is_scalar", value), cast(CastType::Bool, value)));
    }
    if conversions.is_empty() { return None; }
    let converted = conversions.into_iter().rev().fold(value.clone(), |fallback, (test, converted)| {
        select(test, converted, fallback, span)
    });
    Some(select(exact, value.clone(), converted, span))
}

/// Strict calls admit exact members, plus PHP's sole int-to-float widening exception.
pub(crate) fn runtime_strict_param_acceptance(expected: &PhpType, value: &Expr) -> Option<Expr> {
    match expected {
        PhpType::Float => Some(either(call("is_float", value), call("is_int", value), value.span)),
        PhpType::Union(members) => members.iter()
            .map(|member| runtime_strict_param_acceptance(member, value))
            .collect::<Option<Vec<_>>>()?.into_iter().reduce(|a, b| either(a, b, value.span)),
        _ => exact_member(expected, value),
    }
}

/// Tests identity membership rather than the broader coercive scalar acceptance predicate.
fn exact_member(ty: &PhpType, value: &Expr) -> Option<Expr> {
    Some(match ty {
        PhpType::Int => call("is_int", value),
        PhpType::Float => call("is_float", value),
        PhpType::Str => call("is_string", value),
        PhpType::Bool => call("is_bool", value),
        PhpType::False => Expr::new(ExprKind::BinaryOp {
            left: Box::new(value.clone()), op: BinOp::StrictEq,
            right: Box::new(Expr::new(ExprKind::BoolLiteral(false), value.span)),
        }, value.span),
        PhpType::Void => call("is_null", value),
        PhpType::Array(_) | PhpType::AssocArray { .. } => call("is_array", value),
        PhpType::Callable => call("is_callable", value),
        PhpType::Iterable => either(call("is_array", value), instance_of("Traversable", value), value.span),
        PhpType::Object(class) if class.is_empty() || class.eq_ignore_ascii_case("object") => call("is_object", value),
        PhpType::Object(class) => instance_of(class, value),
        _ => return None,
    })
}

/// Uses fully qualified builtin identities, independent of the source namespace.
fn call(name: &str, value: &Expr) -> Expr {
    Expr::new(ExprKind::FunctionCall {
        name: Name::from_parts(crate::names::NameKind::FullyQualified, vec![name.to_string()]),
        args: vec![value.clone()],
    }, value.span)
}

/// Tests an existing class-like contract without introducing a new lexical scope.
fn instance_of(class: &str, value: &Expr) -> Expr {
    Expr::new(ExprKind::InstanceOf {
        value: Box::new(value.clone()),
        target: InstanceOfTarget::Name(Name::from_parts(crate::names::NameKind::FullyQualified,
            class.trim_start_matches('\\').split('\\').map(str::to_string).collect())),
    }, value.span)
}

/// Combines predicates lazily so later runtime checks only run when needed.
fn either(left: Expr, right: Expr, span: Span) -> Expr {
    Expr::new(ExprKind::BinaryOp { left: Box::new(left), op: BinOp::Or, right: Box::new(right) }, span)
}

/// Delegates conversion to the compiler's existing target-independent cast semantics.
fn cast(target: CastType, value: &Expr) -> Expr {
    Expr::new(ExprKind::Cast { target, expr: Box::new(value.clone()) }, value.span)
}

/// Preserves only the selected conversion and its ownership effects.
fn select(condition: Expr, then_expr: Expr, else_expr: Expr, span: Span) -> Expr {
    Expr::new(ExprKind::Ternary {
        condition: Box::new(condition), then_expr: Box::new(then_expr), else_expr: Box::new(else_expr),
    }, span)
}
