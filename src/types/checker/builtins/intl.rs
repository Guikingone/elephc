//! Purpose:
//! Type-checks the intl grapheme (`grapheme_*`), intl normalizer (`normalizer_*`), and
//! iconv (`iconv*`) PHP builtin families at the recognition layer. Validates arity and
//! infers PHP-accurate return types so calls type-check; runtime lowering is deferred
//! (these builtins have no EIR/codegen path yet).
//!
//! Called from:
//! - `crate::types::checker::builtins::check_builtin()`
//!
//! Key details:
//! - Registered so namespaced call sites (e.g. `Symfony\Component\String\grapheme_strlen`)
//!   resolve through the existing builtin namespace fallback, and the conditional
//!   `function_exists('grapheme_...')` polyfill guard drops the user-land definitions.
//! - All functions here are pure string transforms with no persistent state; keep that in
//!   lockstep with `optimize::effects`. `grapheme_extract`'s trailing `&$next` is by-reference
//!   (see `signatures.rs`); the by-ref write is modeled through `ref_params`, not purity.

use crate::errors::CompileError;
use crate::parser::ast::Expr;
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

type BuiltinResult = Result<Option<PhpType>, CompileError>;

/// Type-checks a grapheme/normalizer/iconv builtin call, validating arity and inferring the
/// return type.
///
/// Dispatches on `name`, infers each argument (to propagate constraints), and returns the
/// PHP 8.2 return type. Returns `Ok(None)` for names outside these families so the caller
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
        "grapheme_strlen" => {
            // grapheme_strlen(string $string): int|false|null — grapheme cluster count.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "grapheme_strlen() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
                PhpType::Void,
            ])))
        }
        "grapheme_substr" => {
            // grapheme_substr(string $string, int $offset, ?int $length = null): string|false.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "grapheme_substr() takes 2 or 3 arguments",
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
        "grapheme_stripos" | "grapheme_strripos" | "grapheme_strpos" | "grapheme_strrpos" => {
            // grapheme_strpos/grapheme_strrpos (case-sensitive) and grapheme_stripos/
            // grapheme_strripos (case-insensitive): (string $haystack, string $needle,
            // int $offset = 0): int|false — grapheme-cluster search returning an index or false.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 or 3 arguments", name),
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
        "grapheme_str_split" => {
            // grapheme_str_split(string $string, int $length = 1): array|false.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "grapheme_str_split() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Array(Box::new(PhpType::Str)),
                PhpType::Bool,
            ])))
        }
        "grapheme_extract" => {
            // grapheme_extract(string $haystack, int $size, int $type = GRAPHEME_EXTR_COUNT,
            // int $offset = 0, int &$next = null): string|false. $next is by-reference.
            if args.len() < 2 || args.len() > 5 {
                return Err(CompileError::new(
                    span,
                    "grapheme_extract() takes 2 to 5 arguments",
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
        "normalizer_normalize" => {
            // normalizer_normalize(string $string, int $form = Normalizer::FORM_C): string|false.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "normalizer_normalize() takes 1 or 2 arguments",
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
        "normalizer_is_normalized" => {
            // normalizer_is_normalized(string $string, int $form = Normalizer::FORM_C): bool.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "normalizer_is_normalized() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "iconv" => {
            // iconv(string $from_encoding, string $to_encoding, string $string): string|false.
            if args.len() != 3 {
                return Err(CompileError::new(span, "iconv() takes exactly 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "iconv_strlen" => {
            // iconv_strlen(string $string, ?string $encoding = null): int|false.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "iconv_strlen() takes 1 or 2 arguments",
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
        "iconv_strpos" => {
            // iconv_strpos(string $haystack, string $needle, int $offset = 0,
            // ?string $encoding = null): int|false.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "iconv_strpos() takes 2 to 4 arguments",
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
        "iconv_strrpos" => {
            // iconv_strrpos(string $haystack, string $needle, ?string $encoding = null): int|false.
            // Note: iconv_strrpos has no $offset parameter (unlike iconv_strpos).
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "iconv_strrpos() takes 2 or 3 arguments",
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
        "iconv_substr" => {
            // iconv_substr(string $string, int $offset, ?int $length = null,
            // ?string $encoding = null): string|false.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "iconv_substr() takes 2 to 4 arguments",
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
        "iconv_mime_decode" => {
            // iconv_mime_decode(string $string, int $mode = 0, ?string $encoding = null): string|false.
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "iconv_mime_decode() takes 1 to 3 arguments",
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
        _ => Ok(None),
    }
}
