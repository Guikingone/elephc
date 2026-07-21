//! Purpose:
//! Parses and desugars non-plain-variable foreach binding targets, such as
//! `foreach ($defs as $this->id => $d)` or `foreach ($rows as $out["k"])`.
//! Property, static-property, dynamic-property, and array-element key/value targets lower
//! onto a hidden per-loop variable plus a prepended plain assignment statement.
//!
//! Called from:
//! - `crate::parser::control::parse_foreach()`.
//!
//! Key details:
//! - The `Foreach` AST node is unchanged: only plain variable names reach `key_var`/`value_var`,
//!   so every downstream AST pass keeps its existing contract.
//! - Desugared stores reuse the exact statement shapes the assignment parser produces
//!   (`PropertyAssign`, `StaticPropertyAssign`, `ArrayAssign`, ...) via
//!   `assignment_target_store_stmt`, inheriting all COW/Mixed/ownership behavior.
//! - PHP assigns the value binding before the key binding each iteration, so callers must
//!   prepend the desugared stores in value-then-key order ahead of the user body.

use crate::errors::CompileError;
use crate::lexer::token::SpannedToken;
use crate::lexer::Token;
use crate::parser::ast::{Expr, ExprKind, Stmt};
use crate::parser::expr::parse_expr;
use crate::parser::stmt::assignment_target_store_stmt;
use crate::span::Span;

/// A parsed foreach key or value binding target: either a plain `$variable` (stored directly
/// on the `Foreach` node, byte-identical to the historical parser) or a writable lvalue
/// expression (`$this->prop`, `C::$prop`, `$obj->{$name}`, `$arr[$k]`) that desugars to a
/// hidden loop variable plus a prepended assignment statement.
pub(crate) enum ForeachBinding {
    /// A plain `$variable` binding, used directly as the `Foreach` node's key/value name.
    Plain(String),
    /// A non-plain writable lvalue target that requires the hidden-variable desugar.
    Lvalue(Expr),
}

/// Parses one foreach binding target after `as` or `=>`.
///
/// A plain `$variable` not followed by a member/index marker takes the historical fast path
/// and returns `ForeachBinding::Plain` after consuming exactly one token, keeping the AST
/// byte-identical for existing programs. A target that starts a writable-lvalue expression
/// (`$v->`, `$v?->`, `$v[`, `$this->`, `Name::`, `self::`/`static::`/`parent::`, or a leading
/// `\` fully-qualified name) is parsed as a full postfix expression and validated with
/// `is_foreach_lvalue_shape`. Anything else — call results, literals, constants, or lvalues
/// that parse to a non-writable shape — fails with `expected_variable_msg`, the same
/// diagnostic the parser produced before lvalue targets were supported.
pub(crate) fn parse_foreach_binding(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    expected_variable_msg: &str,
) -> Result<ForeachBinding, CompileError> {
    let next = tokens.get(*pos + 1).map(|(token, _)| token);
    let starts_lvalue_expr = match tokens.get(*pos).map(|(token, _)| token) {
        // `$v->p`, `$v?->p`, `$v[...]`, `$this->p`, `$this[...]` continue past the leading
        // token; a bare `$v` stays on the plain fast path below.
        Some(Token::Variable(_)) | Some(Token::This) => matches!(
            next,
            Some(Token::Arrow | Token::QuestionArrow | Token::LBracket)
        ),
        // `R::$p`, `self::$p`, `static::$p`, `parent::$p` static-property targets.
        Some(Token::Identifier(_) | Token::Self_ | Token::Static | Token::Parent) => {
            matches!(next, Some(Token::DoubleColon))
        }
        // `\Fully\Qualified::$p` — a fully-qualified static-property target.
        Some(Token::Backslash) => matches!(next, Some(Token::Identifier(_))),
        _ => false,
    };
    if !starts_lvalue_expr {
        return match tokens.get(*pos).map(|(token, _)| token) {
            Some(Token::Variable(name)) => {
                let name = name.clone();
                *pos += 1;
                Ok(ForeachBinding::Plain(name))
            }
            _ => Err(CompileError::new(span, expected_variable_msg)),
        };
    }
    let target = parse_expr(tokens, pos)?;
    if !is_foreach_lvalue_shape(&target) {
        return Err(CompileError::new(span, expected_variable_msg));
    }
    Ok(ForeachBinding::Lvalue(target))
}

/// Returns whether a parsed foreach binding expression is a writable-lvalue shape the desugar
/// supports: a property access, a static-property access, a dynamic property access, or an
/// array element whose base is a plain variable or itself such an lvalue. Nullsafe accesses
/// are rejected (PHP forbids `?->` in write context), as are calls, literals, and constants.
fn is_foreach_lvalue_shape(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::PropertyAccess { .. }
        | ExprKind::StaticPropertyAccess { .. }
        | ExprKind::DynamicPropertyAccess { .. } => true,
        ExprKind::ArrayAccess { array, .. } => {
            matches!(array.kind, ExprKind::Variable(_)) || is_foreach_lvalue_shape(array)
        }
        _ => false,
    }
}

/// Converts a parsed binding into the name stored on the `Foreach` node plus the optional
/// desugared store statement to prepend to the loop body.
///
/// A `Plain` binding is returned unchanged with no extra statement. An `Lvalue` binding yields
/// a hidden loop variable (`__elephc_fe_{role}_{line}_{col}`, unique per foreach by its
/// starting span; `role` is `"key"` or `"val"`) and a `<target> = $hidden;` statement built
/// through `assignment_target_store_stmt`, so the per-iteration store reuses the existing
/// property/static-property/array-element assignment machinery unchanged.
pub(crate) fn lower_foreach_binding(
    binding: ForeachBinding,
    role: &str,
    foreach_span: Span,
) -> Result<(String, Option<Stmt>), CompileError> {
    match binding {
        ForeachBinding::Plain(name) => Ok((name, None)),
        ForeachBinding::Lvalue(target) => {
            let hidden = format!(
                "__elephc_fe_{}_{}_{}",
                role, foreach_span.line, foreach_span.col
            );
            let target_span = target.span;
            let store = assignment_target_store_stmt(
                target,
                Expr::new(ExprKind::Variable(hidden.clone()), target_span),
                target_span,
            )?;
            Ok((hidden, Some(Stmt::new(store, target_span))))
        }
    }
}
