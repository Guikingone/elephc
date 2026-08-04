//! Purpose:
//! Type-checks the multibyte-string (`mb_*`) PHP builtin family at the recognition layer.
//! Validates arity and infers PHP-accurate return types so calls type-check; runtime
//! lowering is deferred (these builtins have no EIR/codegen path yet).
//!
//! Called from:
//! - `crate::types::checker::builtins::check_builtin()`
//!
//! Key details:
//! - Registered so namespaced call sites (e.g. `Symfony\Component\String\mb_strlen`)
//!   resolve through the existing builtin namespace fallback, and the conditional
//!   `function_exists('mb_...')` polyfill guard drops the user-land definitions.
//! - All `mb_*` here are pure except `mb_internal_encoding`, which reads/writes the
//!   global mbstring encoding state; keep that in lockstep with `optimize::effects`.

use crate::errors::CompileError;
use crate::parser::ast::Expr;
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

type BuiltinResult = Result<Option<PhpType>, CompileError>;

/// Type-checks a multibyte-string builtin call, validating arity and inferring the return type.
///
/// Dispatches on `name`, infers each argument (to propagate constraints), and returns the
/// PHP 8.2 return type. Returns `Ok(None)` for names outside the `mb_*` family so the caller
/// falls through to other builtin handlers. These builtins are recognition-only: they have no
/// runtime lowering yet, so only arity and return type are modeled here.
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        "mb_substr" => {
            // mb_substr(string $string, int $start, ?int $length = null,
            // ?string $encoding = null): string.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(span, "mb_substr() takes 2 to 4 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "mb_strlen" => {
            // mb_strlen(string $string, ?string $encoding = null): int.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "mb_strlen() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "mb_stripos" | "mb_strripos" => {
            // mb_stripos/mb_strripos(string $haystack, string $needle, int $offset = 0,
            // ?string $encoding = null): int|false — case-insensitive multibyte search.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 to 4 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
            ])))
        }
        "mb_strpos" | "mb_strrpos" => {
            // mb_strpos/mb_strrpos(string $haystack, string $needle, int $offset = 0,
            // ?string $encoding = null): int|false — multibyte substring search.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 to 4 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
            ])))
        }
        "mb_strstr" | "mb_stristr" => {
            // mb_strstr/mb_stristr(string $haystack, string $needle,
            // bool $before_needle = false, ?string $encoding = null): string|false.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 to 4 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "mb_strtolower" | "mb_strtoupper" => {
            // mb_strtolower/mb_strtoupper(string $string, ?string $encoding = null): string.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 1 or 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "mb_convert_case" => {
            // mb_convert_case(string $string, int $mode, ?string $encoding = null): string.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "mb_convert_case() takes 2 or 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "mb_encode_numericentity" => {
            // mb_encode_numericentity(string $string, array $map, ?string $encoding = null,
            // bool $hex = false): string — recognized so the call type-checks and
            // namespace fallback resolves; codegen is deferred (loud unsupported error).
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "mb_encode_numericentity() takes 2 to 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        // `mb_convert_encoding` is deliberately NOT recognized here: it now ships as an
        // elephc-PHP prelude (`crate::mb_convert_encoding_prelude`) that actually CONVERTS and
        // follows PHP's result shape (an array subject returns an array). Recognizing it as a
        // builtin would shadow that declaration and put the call straight back on the
        // `unsupported EIR backend feature: builtin call mb_convert_encoding` path, which is
        // where `Console\Application::splitStringByWidth` used to die.
        "mb_detect_encoding" => {
            // mb_detect_encoding(string $string, array|string|null $encodings = null,
            // bool $strict = false): string|false.
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "mb_detect_encoding() takes 1 to 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "mb_internal_encoding" => {
            // mb_internal_encoding(?string $encoding = null): string|bool — returns the
            // current internal encoding (string) when read, or true/false when set. This is
            // the one stateful mb_* builtin (global mbstring encoding state).
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "mb_internal_encoding() takes 0 or 1 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "mb_ord" => {
            // mb_ord(string $string, ?string $encoding = null): int|false.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "mb_ord() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
            ])))
        }
        "mb_str_split" => {
            // mb_str_split(string $string, int $length = 1, ?string $encoding = null): array.
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "mb_str_split() takes 1 to 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "mb_strwidth" => {
            // mb_strwidth(string $string, ?string $encoding = null): int.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "mb_strwidth() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        _ => Ok(None),
    }
}
