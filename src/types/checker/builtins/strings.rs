//! Purpose:
//! Type-checks the strings PHP builtin family.
//! Validates arity, argument types, warning-producing cases, and inferred return types for direct calls.
//!
//! Called from:
//! - `crate::types::checker::builtins::check_builtin()`
//!
//! Key details:
//! - Signatures, callable aliases, optimizer effects, and codegen builtin dispatch must remain in lockstep.

use crate::errors::CompileError;
use crate::parser::ast::{Expr, ExprKind};
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

type BuiltinResult = Result<Option<PhpType>, CompileError>;

/// Type-checks a string builtin call, validating arity, argument types, and return type.
///
/// Dispatches on `name` to validate the call and infer the return `PhpType`.
/// Calls `checker.infer_type()` on each argument to propagate type constraints.
/// For cryptographic builtins (`hash`, `md5`, `sha1`), records a Linux library requirement.
///
/// Returns `Ok(Some(PhpType))` with the inferred return type, `Ok(None)` for unknown
/// builtins (caller will fall through to other handlers), or `Err(CompileError)` on
/// arity/type mismatch.
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        "strlen" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "strlen() takes exactly 1 argument"));
            }
            let ty = checker.infer_type(&args[0], env)?;
            // Accept Str, Mixed, and Union types — PHP's strlen() coerces its
            // argument to a string per the standard PHP type juggling rules
            // (numbers become their decimal representation, true → "1",
            // false/null → ""). Mixed inputs flow through __rt_mixed_strlen
            // at codegen time which reads the cell tag and returns the
            // length of the coerced representation.
            if !crate::builtins::semantics::strlen_accepts_source(&ty) {
                return Err(CompileError::new(span, "strlen() argument must be string"));
            }
            Ok(Some(PhpType::Int))
        }
        "intval" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "intval() takes 1 or 2 arguments"));
            }
            checker.infer_type(&args[0], env)?;
            if let Some(base_arg) = args.get(1) {
                let base_ty = checker.infer_type(base_arg, env)?;
                if !matches!(base_ty, PhpType::Int) {
                    return Err(CompileError::new(
                        base_arg.span,
                        "intval() base argument must be int",
                    ));
                }
            }
            Ok(Some(PhpType::Int))
        }
        "substr" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(span, "substr() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "strpos" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "strpos() takes 2 or 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Union(vec![PhpType::Int, PhpType::Bool])))
        }
        "strrpos" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "strrpos() takes 2 or 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Union(vec![PhpType::Int, PhpType::Bool])))
        }
        "strstr" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(span, "strstr() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "grapheme_strrev" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "grapheme_strrev() takes exactly 1 argument",
                ));
            }
            let ty = checker.infer_type(&args[0], env)?;
            if !matches!(ty, PhpType::Str | PhpType::Mixed | PhpType::Union(_)) {
                return Err(CompileError::new(
                    span,
                    "grapheme_strrev() argument must be string",
                ));
            }
            Ok(Some(PhpType::Union(vec![PhpType::Str, PhpType::Bool])))
        }
        "strtolower" | "strtoupper" | "ucfirst" | "lcfirst" | "ucwords" | "trim"
        | "ltrim" | "rtrim" | "chop" | "strrev" | "str_repeat" | "str_replace"
        | "str_ireplace" | "chr" | "addslashes" | "stripslashes" | "nl2br" | "bin2hex" => {
            let expected = match name {
                "str_repeat" => 2,
                "str_replace" | "str_ireplace" => 3,
                _ => 1,
            };
            if name == "chr" {
                if args.len() != 1 {
                    return Err(CompileError::new(span, "chr() takes exactly 1 argument"));
                }
            } else if matches!(name, "trim" | "ltrim" | "rtrim" | "chop" | "ucwords") {
                // ucwords(string $string, string $separators = " \t\r\n\f\v"): the
                // optional separators argument customizes the word-boundary set.
                if args.is_empty() || args.len() > 2 {
                    return Err(CompileError::new(
                        span,
                        &format!("{}() takes 1 or 2 arguments", name),
                    ));
                }
            } else if args.len() != expected {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "{}() takes exactly {} argument{}",
                        name,
                        expected,
                        if expected > 1 { "s" } else { "" }
                    ),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "long2ip" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "long2ip() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "ip2long" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "ip2long() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
            ])))
        }
        "inet_ntop" | "inet_pton" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "hex2bin" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "hex2bin() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "substr_replace" => {
            if args.len() != 3 && args.len() != 4 {
                return Err(CompileError::new(
                    span,
                    "substr_replace() takes 3 or 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "str_pad" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(span, "str_pad() takes 2 to 4 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "str_split" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "str_split() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "wordwrap" => {
            if args.is_empty() || args.len() > 4 {
                return Err(CompileError::new(span, "wordwrap() takes 1 to 4 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "octdec" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "octdec() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "dechex" | "decoct" | "decbin" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "substr_count" => {
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "substr_count() takes 2 to 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "ord" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "ord() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "strcmp" | "strcasecmp" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "strval" => {
            // strval(mixed $value): string — coerces its argument to a string with
            // PHP's `(string)` cast rules. Any scalar/Mixed argument is accepted.
            if args.len() != 1 {
                return Err(CompileError::new(span, "strval() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "strnatcmp" | "strnatcasecmp" => {
            // Natural-order string comparison; returns a signed int (<0, 0, >0).
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "strrchr" => {
            // strrchr(string $haystack, string $needle): string|false — returns the
            // suffix from the last occurrence of the first byte of $needle, or false.
            if args.len() != 2 {
                return Err(CompileError::new(span, "strrchr() takes exactly 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool])))
        }
        "addcslashes" => {
            // addcslashes(string $string, string $characters): string.
            if args.len() != 2 {
                return Err(CompileError::new(span, "addcslashes() takes exactly 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "stripcslashes" => {
            // stripcslashes(string $string): string.
            if args.len() != 1 {
                return Err(CompileError::new(span, "stripcslashes() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "str_getcsv" => {
            // str_getcsv(string, sep, enclosure, escape): array — recognized so the
            // call type-checks and namespace fallback resolves; codegen is deferred
            // (loud unsupported error) until CSV field parsing is implemented.
            if args.is_empty() || args.len() > 4 {
                return Err(CompileError::new(span, "str_getcsv() takes 1 to 4 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Mixed))))
        }
        "pack" => {
            // pack(string $format, mixed ...$values): string — recognized for
            // namespace fallback; codegen is deferred (loud unsupported error).
            if args.is_empty() {
                return Err(CompileError::new(span, "pack() requires at least 1 argument"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "parse_url" => {
            // parse_url(string $url, int $component = -1): array|string|int|false|null.
            // The heterogeneous result (assoc array without a component, or a single
            // scheme/host/port/... value with one) is modeled as `Mixed`.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "parse_url() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "strcspn" | "strspn" => {
            // PHP: strcspn/strspn(string $string, string $characters,
            // int $offset = 0, ?int $length = null): int.
            if args.len() < 2 || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 to 4 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "strpbrk" => {
            // PHP: strpbrk(string $string, string $characters): string|false.
            if args.len() != 2 {
                return Err(CompileError::new(span, "strpbrk() takes exactly 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool])))
        }
        "str_contains" | "str_starts_with" | "str_ends_with" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "explode" => {
            // PHP: `explode(string $separator, string $string, int $limit = PHP_INT_MAX):
            // array` — the trailing `$limit` is optional, so 2 or 3 arguments are valid.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(span, "explode() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "implode" => {
            if args.len() != 2 {
                return Err(CompileError::new(span, "implode() takes exactly 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "sprintf" => {
            if args.is_empty() {
                return Err(CompileError::new(span, "sprintf() requires at least 1 argument"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "printf" => {
            if args.is_empty() {
                return Err(CompileError::new(span, "printf() requires at least 1 argument"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "vsprintf" | "vprintf" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 2 arguments (format, values)", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // vsprintf returns the formatted string; vprintf prints it and
            // returns the number of bytes written.
            Ok(Some(if name == "vsprintf" {
                PhpType::Str
            } else {
                PhpType::Int
            }))
        }
        "hash" => {
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(span, "hash() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // hash() routes through the elephc-crypto staticlib (full algorithm
            // set, raw $binary output, catchable ValueError) on every target.
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Str))
        }
        "hash_hmac" => {
            if args.len() < 3 || args.len() > 4 {
                return Err(CompileError::new(span, "hash_hmac() takes 3 or 4 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // hash_hmac() routes through the elephc-crypto staticlib's
            // elephc_crypto_hmac entry point (raw $binary output, catchable
            // ValueError for unknown algo or non-crypto checksum) on every target.
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Str))
        }
        "hash_equals" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    "hash_equals() takes exactly 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // Pure timing-safe byte comparison — no crypto library required.
            Ok(Some(PhpType::Bool))
        }
        "hash_algos" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "hash_algos() takes no arguments"));
            }
            // Returns the supported-algorithm name list; pure, no crypto library.
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "hash_init" => {
            // HASH_HMAC streaming mode (flags/key) is not supported; use hash_hmac().
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "hash_init() flags/HASH_HMAC streaming mode is not supported; use hash_hmac() for HMAC",
                ));
            }
            checker.infer_type(&args[0], env)?;
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Mixed))
        }
        "hash_update" => {
            if args.len() != 2 {
                return Err(CompileError::new(span, "hash_update() takes exactly 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Bool))
        }
        "hash_final" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "hash_final() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Str))
        }
        "hash_copy" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "hash_copy() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Mixed))
        }
        "sscanf" => {
            if args.len() < 2 {
                return Err(CompileError::new(span, "sscanf() takes at least 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "md5" | "sha1" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 1 or 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // md5()/sha1() route through the elephc-crypto staticlib (same path as
            // hash()), so they honor the $binary flag and no longer depend on the
            // system crypto library.
            checker.require_builtin_library("elephc_crypto");
            Ok(Some(PhpType::Str))
        }
        "crc32" => {
            // crc32() is a pure table-free computation in __rt_crc32; unlike the
            // md5/sha1 digests it needs no system crypto library and returns int.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "crc32() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "htmlspecialchars" | "htmlentities" | "html_entity_decode" => {
            // htmlspecialchars/htmlentities accept 1–4 args (string, flags, encoding,
            // double_encode); html_entity_decode accepts 1–3 (no double_encode).
            let max_args = if name == "html_entity_decode" { 3 } else { 4 };
            if args.is_empty() || args.len() > max_args {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 1 to {} arguments", name, max_args),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "urlencode" | "urldecode" | "rawurlencode" | "rawurldecode" | "base64_encode" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        // PHP: base64_decode(string $string, bool $strict = false): string|false.
        // elephc decodes identically regardless of $strict for now, but must
        // accept the optional second argument so the call type-checks.
        "base64_decode" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "base64_decode() expects 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "gzcompress" | "gzuncompress" | "gzdeflate" | "gzinflate" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() expects 1 or 2 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            checker.require_builtin_library("z");
            // gzcompress/gzdeflate always produce a string; the decompressors
            // return false on a zlib error.
            if name == "gzcompress" || name == "gzdeflate" {
                Ok(Some(PhpType::Str))
            } else {
                Ok(Some(checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool])))
            }
        }
        "ctype_alpha" | "ctype_digit" | "ctype_alnum" | "ctype_space" | "ctype_upper" => {
            // ctype_*(mixed $text): bool — character-class predicate. ctype_upper is
            // recognition-only (no runtime lowering yet); the others are runtime-backed.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "strtr" => {
            // strtr(string $string, array|string $from, ?string $to = null): string.
            // The 2-arg map form and the 3-arg char-translation form both return string.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(span, "strtr() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "stripos" | "strripos" => {
            // Case-insensitive strpos/strrpos: returns int offset or false.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes 2 or 3 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![PhpType::Int, PhpType::Bool])))
        }
        "strncmp" | "strncasecmp" => {
            // Compare the first $length bytes; returns a signed int (<0, 0, >0).
            if args.len() != 3 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 3 arguments", name),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "substr_compare" => {
            // substr_compare(haystack, needle, offset, ?length = null,
            // case_insensitive = false): int.
            if args.len() < 3 || args.len() > 5 {
                return Err(CompileError::new(
                    span,
                    "substr_compare() takes 3 to 5 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "strip_tags" => {
            // strip_tags(string $string, array|string|null $allowed_tags = null): string.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "strip_tags() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "levenshtein" => {
            // levenshtein(string1, string2, insertion=1, replacement=1, deletion=1): int.
            if args.len() < 2 || args.len() > 5 {
                return Err(CompileError::new(
                    span,
                    "levenshtein() takes 2 to 5 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "parse_str" => {
            // PHP: parse_str(string $string, array &$result): void. `$result` is a
            // by-ref out-parameter (PHP auto-vivifies it), so — like preg_match's
            // `$matches` — it must be a variable and is not eagerly inferred.
            if args.len() != 2 {
                return Err(CompileError::new(span, "parse_str() takes exactly 2 arguments"));
            }
            checker.infer_type(&args[0], env)?;
            if !matches!(args[1].kind, ExprKind::Variable(_)) {
                return Err(CompileError::new(
                    args[1].span,
                    "parse_str() parameter $result must be passed a variable",
                ));
            }
            Ok(Some(PhpType::Void))
        }
        _ => Ok(None),
    }
}
