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
use crate::parser::ast::Expr;
use crate::types::{PhpType, TypeEnv};

use super::common::BuiltinResult;
use super::super::super::Checker;

/// Type-checks `var_dump` and `print_r`.
///
/// `var_dump` is variadic (`mixed $value, mixed ...$values`) and accepts one or more
/// arguments, dumping each in source order. `print_r` accepts a single value and
/// returns `void` (the optional `$return` mode is not yet supported).
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        "var_dump" => {
            if args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "var_dump() takes at least 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Void))
        }
        "print_r" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "print_r() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Void))
        }
        _ => Ok(None),
    }
}
