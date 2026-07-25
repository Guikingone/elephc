//! Purpose:
//! Home of the PHP `class_alias` builtin: its single-source registry declaration and semantic target.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through `crate::builtins::registry`.
//!
//! Key details:
//! - Top-level literal `class_alias()` calls are stripped and synthesized into subclass
//!   declarations by `crate::autoload::alias` before the checker runs, so they never reach
//!   this hook. The hook only sees non-top-level calls (inside methods, closures, `if`
//!   blocks). It accepts those whose original/alias arguments are compile-time class-name
//!   literals (`"Foo"` or `Foo::class`) — the alias target is fixed regardless of statement
//!   position — and rejects genuinely dynamic names, which AOT cannot resolve ahead of time.
//! - Arguments are pre-inferred by the registry common path before the hook runs.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind, StaticReceiver};
use crate::types::PhpType;

builtin! {
    name: "class_alias",
    area: Callables,
    params: [class: Str, alias: Str, autoload: Bool = DefaultSpec::Bool(true)],
    returns: Bool,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::ClassAlias,
    ),
    summary: "Creates an alias for a class.",
    php_manual: "function.class-alias",
}

/// Accepts a non-top-level `class_alias()` call whose original and alias arguments are
/// compile-time class-name literals, and rejects calls built from dynamic names.
///
/// Top-level literal calls are already stripped and synthesized into subclass declarations
/// before checking (`crate::autoload::alias`); this hook only sees the ones nested inside
/// methods, closures, or conditionals. A literal (`"Foo"`) or `Foo::class` argument names a
/// fixed class regardless of statement position, so the alias is resolvable ahead of time and
/// the call is accepted. A variable or other runtime expression cannot be resolved by AOT, so
/// it stays rejected — this keeps genuinely dynamic aliasing loud.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    if is_literal_class_name(cx.args.first()) && is_literal_class_name(cx.args.get(1)) {
        return Ok(PhpType::Bool);
    }
    Err(CompileError::new(
        cx.span,
        "class_alias() is only supported as a top-level statement with literal class names",
    ))
}

/// Returns true when `arg` is a compile-time class-name literal: a string literal
/// (`"Foo\\Bar"`) or a `Foo::class` constant on a named receiver. Both resolve to a fixed
/// class name during AOT, so a `class_alias()` call built from them can be honored in any
/// statement position. `self::class`/`static::class`/`parent::class`, variables, and other
/// runtime expressions are not fixed literals and return false.
fn is_literal_class_name(arg: Option<&Expr>) -> bool {
    matches!(
        arg.map(|expr| &expr.kind),
        Some(ExprKind::StringLiteral(_))
            | Some(ExprKind::ClassConstant {
                receiver: StaticReceiver::Named(_),
            })
    )
}
