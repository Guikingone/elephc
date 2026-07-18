//! Purpose:
//! Type-checks PHP IO builtin debug helpers and signatures.
//! Validates arity, argument categories, resource handling, and return types before codegen sees calls.
//!
//! Called from:
//! - `crate::types::checker::builtins::io::check_builtin()`
//!
//! Key details:
//! - Return types and diagnostics must stay aligned with `crate::types::signatures` and builtin codegen emitters.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{PhpType, TypeEnv};

use super::common::BuiltinResult;
use super::super::super::Checker;

/// Type-checks `var_dump` and `print_r`.
///
/// `var_dump` takes exactly one argument of any type and returns `void`.
/// `print_r($value, $return = false)` php-verified return type is `string|true`
/// — NOT `void`: PHP's `print_r()` always returns something (`true` when
/// `$return` is falsy, the rendered string when `$return` is truthy). When
/// `$return` is a literal `true`/`false` the precise type is used; a
/// non-literal `$return` gets the conservative `string|bool` union.
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        "var_dump" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "var_dump() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Void))
        }
        "print_r" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "print_r() takes 1 or 2 arguments"));
            }
            checker.infer_type(&args[0], env)?;
            let return_ty = match args.get(1) {
                None => PhpType::Bool,
                Some(arg) => {
                    let ty = checker.infer_type(arg, env)?;
                    if ty != PhpType::Bool {
                        return Err(CompileError::new(
                            arg.span,
                            "print_r() return argument must be bool",
                        ));
                    }
                    match &arg.kind {
                        ExprKind::BoolLiteral(true) => PhpType::Str,
                        ExprKind::BoolLiteral(false) => PhpType::Bool,
                        _ => checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool]),
                    }
                }
            };
            Ok(Some(return_ty))
        }
        _ => Ok(None),
    }
}
