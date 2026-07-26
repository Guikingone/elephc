//! Purpose:
//! Parses expression suffixes that involve calls, casts, static receivers, and first-class callables.
//! Recognizes scoped static call forms and PHP cast syntax around grouped expressions.
//!
//! Called from:
//! - `crate::parser::expr::pratt` and `crate::parser::expr::prefix`.
//!
//! Key details:
//! - Callable and cast disambiguation depends on PHP token spelling and case-insensitive type names.

use crate::errors::CompileError;
use crate::lexer::{SpannedToken, Token};
use crate::parser::ast::{CallableTarget, CastType, Expr, ExprKind, StaticReceiver};
use crate::span::Span;

use super::parse_args;

/// Parses the `::` suffix of a scoped static access (static method call, static property
/// access, class constant, or scoped constant access) after the initial receiver has been
/// consumed. Consumes the `::` token, then dispatches on the following token to produce the
/// appropriate `ExprKind` variant. Returns a static method call if a `(` follows, a static
/// property access if a variable follows, a class constant if `class` follows, or a scoped
/// constant access otherwise.
pub(super) fn parse_scoped_static_call(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
    receiver: StaticReceiver,
    receiver_name: &str,
) -> Result<Expr, CompileError> {
    if *pos >= tokens.len() || tokens[*pos].0 != Token::DoubleColon {
        return Err(CompileError::new(
            span,
            &format!("Expected '::' after '{}'", receiver_name),
        ));
    }
    *pos += 1;
    if matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::Dollar)) {
        // `self::${$expr}` / `self::$$var` — a static property named at runtime.
        let property = parse_dynamic_static_property_name(tokens, pos, span)?;
        return Ok(Expr::new(
            ExprKind::DynamicStaticPropertyAccess {
                receiver,
                property: Box::new(property),
            },
            span,
        ));
    }
    let method = match tokens.get(*pos).map(|(token, _)| token) {
        Some(Token::Variable(property)) => {
            let property = property.clone();
            let property_span = tokens[*pos].1.span;
            *pos += 1;
            // `self::$method(args)` / `static::$method(args)` / `parent::$method(args)` is a dynamic
            // static method call (the method name lives in `$method`). Desugar to
            // `call_user_func([receiver::class, $method], ...args)` so it reuses the runtime
            // dynamic-dispatch path, exactly like the named-class form `C::$method(args)`. The
            // `receiver::class` constant resolves `static::` to the runtime class, preserving late
            // static binding. Without a following `(` it stays a static property access.
            if matches!(tokens.get(*pos).map(|(token, _)| token), Some(Token::LParen)) {
                *pos += 1; // consume '('
                if parse_first_class_callable_parens(tokens, pos)? {
                    // `self::$method(...)` / `static::$method(...)` / `parent::$method(...)` — a
                    // first-class callable bound to a runtime-named static method. Reuse the same
                    // `[receiver::class, $method]` callable as the call form, wrapped in a Closure.
                    // Keep the method name's own span so no two synthesized nodes collapse onto one
                    // span key (see the helper's note).
                    let class_const =
                        Expr::new(ExprKind::ClassConstant { receiver: receiver.clone() }, span);
                    let method_expr = Expr::new(ExprKind::Variable(property), property_span);
                    return Ok(super::prefix_complex::build_dynamic_method_first_class_callable(
                        class_const,
                        method_expr,
                        span,
                    ));
                }
                let dynamic_args = crate::parser::expr::parse_args(tokens, pos, span)?;
                crate::parser::expr::pratt::reject_named_args_in_dynamic_call(&dynamic_args, span)?;
                let class_const = Expr::new(ExprKind::ClassConstant { receiver: receiver.clone() }, span);
                let method_expr = Expr::new(ExprKind::Variable(property), span);
                let mut call_args =
                    vec![Expr::new(ExprKind::ArrayLiteral(vec![class_const, method_expr]), span)];
                call_args.extend(dynamic_args);
                return Ok(Expr::new(
                    ExprKind::FunctionCall {
                        name: crate::names::Name::unqualified("call_user_func"),
                        args: call_args,
                    },
                    span,
                ));
            }
            return Ok(Expr::new(
                ExprKind::StaticPropertyAccess { receiver, property },
                span,
            ));
        }
        Some(Token::Class) => {
            *pos += 1;
            return Ok(Expr::new(ExprKind::ClassConstant { receiver }, span));
        }
        // PHP 8 allows semi-reserved keywords as static method / class-constant names
        // (e.g. `Foo::self()`, `Foo::print`); `class` and `$var` are handled above.
        Some(t)
            if crate::parser::keyword_name::bareword_name_from_token(t, &tokens[*pos].1)
                .is_some() =>
        {
            let name =
                crate::parser::keyword_name::bareword_name_from_token(t, &tokens[*pos].1).unwrap();
            *pos += 1;
            name
        }
        _ => {
            return Err(CompileError::new(
                span,
                &format!("Expected method or property name after '{}::'", receiver_name),
            ))
        }
    };
    // If a `(` follows, this is a static method call; otherwise it's a
    // user-declared class-constant access (`MyClass::FOO`).
    if *pos >= tokens.len() || tokens[*pos].0 != Token::LParen {
        return Ok(Expr::new(
            ExprKind::ScopedConstantAccess {
                receiver,
                name: method,
            },
            span,
        ));
    }
    *pos += 1;
    if parse_first_class_callable_parens(tokens, pos)? {
        Ok(Expr::new(
            ExprKind::FirstClassCallable(CallableTarget::StaticMethod { receiver, method }),
            span,
        ))
    } else {
        let args = parse_args(tokens, pos, span)?;
        let span = crate::parser::expr::span_through_prev_token(tokens, *pos, span);
        Ok(Expr::new(
            ExprKind::StaticMethodCall {
                receiver,
                method,
                args,
            },
            span,
        ))
    }
}

/// Parses the dynamic property-name expression of a dynamic static property access, starting at
/// a `Token::Dollar` that immediately follows a static receiver's `::`. Consumes the `$` marker,
/// then either `{ <expr> }` (for `self::${$expr}`) or a `Variable` (for `self::$$var`), returning
/// the parsed name expression. Returns an error if neither form follows the `$`.
pub(super) fn parse_dynamic_static_property_name(
    tokens: &[SpannedToken],
    pos: &mut usize,
    span: Span,
) -> Result<Expr, CompileError> {
    *pos += 1; // consume the bare `$` marker
    match tokens.get(*pos).map(|(token, _)| token.clone()) {
        Some(Token::LBrace) => {
            *pos += 1; // consume `{`
            let name_expr = crate::parser::expr::parse_expr(tokens, pos)?;
            if *pos >= tokens.len() || tokens[*pos].0 != Token::RBrace {
                return Err(CompileError::new(
                    span,
                    "Expected '}' after dynamic static property name",
                ));
            }
            *pos += 1; // consume `}`
            Ok(name_expr)
        }
        Some(Token::Variable(var)) => {
            *pos += 1; // consume the variable
            Ok(Expr::new(ExprKind::Variable(var), span))
        }
        _ => Err(CompileError::new(
            span,
            "Expected '{' or variable after '$' in dynamic static property access",
        )),
    }
}

/// Checks whether `...)` appears at the current position, indicating PHP's first-class callable
/// syntax `Name::method(...)`. Returns `true` and consumes both the ellipsis and `)` tokens if
/// found; returns `false` and advances nothing otherwise. Called after the initial `(` of a
/// method-call-like form has already been consumed by the caller.
pub(super) fn parse_first_class_callable_parens(
    tokens: &[SpannedToken],
    pos: &mut usize,
) -> Result<bool, CompileError> {
    if *pos + 1 < tokens.len()
        && tokens[*pos].0 == Token::Ellipsis
        && tokens[*pos + 1].0 == Token::RParen
    {
        *pos += 2;
        return Ok(true);
    }
    Ok(false)
}

/// Peeks ahead to detect a PHP cast expression `(type)` where the middle token is an
/// identifier matching a valid cast type name. Does not consume any tokens. Returns the
/// matching `CastType` variant or `None` if the three-token window does not form a cast.
pub(super) fn peek_cast(tokens: &[SpannedToken], pos: usize) -> Option<CastType> {
    if pos + 2 >= tokens.len() {
        return None;
    }
    if tokens[pos].0 != Token::LParen || tokens[pos + 2].0 != Token::RParen {
        return None;
    }
    match &tokens[pos + 1].0 {
        Token::Identifier(name) if matches_case_insensitive(name, &["int", "integer"]) => {
            Some(CastType::Int)
        }
        Token::Identifier(name) if matches_case_insensitive(name, &["float", "double", "real"]) => {
            Some(CastType::Float)
        }
        Token::Identifier(name) if name.eq_ignore_ascii_case("string") => Some(CastType::String),
        Token::Identifier(name) if matches_case_insensitive(name, &["bool", "boolean"]) => {
            Some(CastType::Bool)
        }
        Token::Identifier(name) if name.eq_ignore_ascii_case("array") => Some(CastType::Array),
        Token::Identifier(name) if name.eq_ignore_ascii_case("object") => Some(CastType::Object),
        _ => None,
    }
}

/// Returns `true` if `name` matches any of the `keywords` case-insensitively.
fn matches_case_insensitive(name: &str, keywords: &[&str]) -> bool {
    keywords
        .iter()
        .any(|keyword| name.eq_ignore_ascii_case(keyword))
}
