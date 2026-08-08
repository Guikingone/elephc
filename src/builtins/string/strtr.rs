//! Purpose:
//! Home of the PHP `strtr` builtin: its declaration and semantic metadata.
//!
//! Called from:
//! - Checker, EIR, optimizer, ownership, and callable consumers through
//!   `crate::builtins::registry`.
//!
//! Key details:
//! - php-src exposes one function with two shapes: `strtr($string, $from, $to)` translates
//!   bytes pairwise (truncated to the shorter of `$from`/`$to`), and `strtr($string, $pairs)`
//!   applies replacement pairs longest-match-first in a single left-to-right pass.
//! - `$from` is declared `Mixed` because it is `array|string` in php-src; the check hook
//!   enforces php-src's own `TypeError` wording at compile time, where elephc can already see
//!   the argument's type.
//! - The two-argument form needs string replacement VALUES: elephc reads them straight out of
//!   the runtime hash instead of converting each one, so an array of non-string values is
//!   rejected with an explicit diagnostic rather than silently mis-rendered.

use crate::builtins::spec::{BuiltinCheckCtx, DefaultSpec};
use crate::errors::CompileError;
use crate::parser::ast::ExprKind;
use crate::types::PhpType;

builtin! {
    name: "strtr",
    area: String,
    params: [
        string: Str,
        from: Mixed,
        to: Str = DefaultSpec::Null
    ],
    returns: Str,
    check: check,
    semantics: crate::builtins::semantics::runtime_fn_semantics(
        crate::ir::RuntimeFnId::Strtr,
    ),
    summary: "Translates bytes pairwise, or applies longest-match-first replacement pairs.",
    php_manual: "https://www.php.net/manual/en/function.strtr.php",
}

/// Validates the `strtr` call shape and returns its `string` result type.
///
/// Argument types are inferred by the common registry dispatch path before this hook fires,
/// and arity is pre-validated by the registry. The two-argument form requires an array
/// `$from` whose values are strings, and the three-argument form requires a string `$from`;
/// both mismatches carry php-src's own `TypeError` wording.
fn check(cx: &mut BuiltinCheckCtx) -> Result<PhpType, CompileError> {
    let from = cx.checker.infer_type(from_argument(cx.args), cx.env)?;
    if cx.args.len() >= 3 {
        if matches!(from, PhpType::Array(_) | PhpType::AssocArray { .. }) {
            return Err(CompileError::new(
                cx.span,
                "strtr(): Argument #2 ($from) must be of type string, array given",
            ));
        }
        return Ok(PhpType::Str);
    }
    let values = match &from {
        PhpType::Array(values) => values.as_ref().clone(),
        PhpType::AssocArray { value, .. } => value.as_ref().clone(),
        _ => {
            return Err(CompileError::new(
                cx.span,
                "strtr(): Argument #2 ($from) must be of type array, string given",
            ))
        }
    };
    if !matches!(values, PhpType::Str | PhpType::Never) {
        return Err(CompileError::new(
            cx.span,
            "strtr() replacement values must be strings in AOT mode",
        ));
    }
    Ok(PhpType::Str)
}

/// Returns the `$from` argument expression from a call's source-order argument list.
///
/// A `from:` named argument is matched by name first so `strtr($s, from: [...])` validates
/// like the positional spelling; otherwise the second positional argument is used. The
/// registry guarantees at least two arguments before this hook runs, so the caller always
/// gets an expression back.
fn from_argument(args: &[crate::parser::ast::Expr]) -> &crate::parser::ast::Expr {
    for arg in args {
        if let ExprKind::NamedArg { name, value } = &arg.kind {
            if name == "from" {
                return value;
            }
        }
    }
    args.iter()
        .filter(|arg| !matches!(arg.kind, ExprKind::NamedArg { .. }))
        .nth(1)
        .unwrap_or(&args[args.len() - 1])
}
