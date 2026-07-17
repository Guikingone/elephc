//! Purpose:
//! Type-checks the system PHP builtin family.
//! Validates arity, argument types, warning-producing cases, and inferred return types for direct calls.
//!
//! Called from:
//! - `crate::types::checker::builtins::check_builtin()`
//!
//! Key details:
//! - Signatures, callable aliases, optimizer effects, and codegen builtin dispatch must remain in lockstep.

use crate::errors::CompileError;
use crate::names::php_symbol_key;
use crate::parser::ast::{BinOp, Expr, ExprKind};
use crate::types::json_constants::JSON_INT_CONSTANTS;
use crate::types::{PhpType, TypeEnv};

use super::super::Checker;

type BuiltinResult = Result<Option<PhpType>, CompileError>;

/// Type-checks a system builtin call by name, validating arity, argument types,
/// and return type. Returns `Ok(Some(PhpType))` for handled builtins, `Ok(None)`
/// for unknown system builtins, or an error for misuse.
pub(super) fn check_builtin(
    checker: &mut Checker,
    name: &str,
    args: &[Expr],
    span: crate::span::Span,
    env: &TypeEnv,
) -> BuiltinResult {
    match name {
        "time" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "time() takes no arguments"));
            }
            Ok(Some(PhpType::Int))
        }
        "setlocale" => {
            // `setlocale(int $category, string|array $locales, string ...$rest): string|false`.
            // elephc has no real locale machinery; this is a minimal sound stub that
            // accepts the arguments, changes nothing, and returns the requested locale
            // string. The static type is `string|false` to match PHP. At least the
            // category and one locale are required.
            if args.len() < 2 {
                return Err(CompileError::new(
                    span,
                    "setlocale() takes at least 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool])))
        }
        "serialize" => {
            // `serialize(mixed $value): string`. elephc recognizes the call so symfony/yaml
            // and similar code type-check, but the runtime lowering is a deferred fatal stub
            // (the full PHP serialization format is not yet implemented). In symfony/yaml the
            // only call site is gated behind the opt-in `Yaml::DUMP_OBJECT` dump flag, so a
            // normal parse never reaches it. The argument is accepted as `mixed`.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "serialize() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "unserialize" => {
            // `unserialize(string $data, array $options = []): mixed`. Recognized for the same
            // reason as serialize(): the only symfony/yaml call site is the opt-in `!php/object`
            // parse tag, which a normal parse never reaches. The runtime lowering is a deferred
            // fatal stub. Accepts the serialized string plus the optional options array.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "unserialize() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "trigger_deprecation" => {
            // `trigger_deprecation(string $package, string $version, string $message,
            // mixed ...$args): void` is Symfony's symfony/deprecation-contracts global.
            // In PHP it formats the message and raises an `E_USER_DEPRECATED` notice.
            // Deprecation notices are advisory, so elephc accepts the call and treats it
            // as a sound no-op returning void. package/version/message are required.
            if args.len() < 3 {
                return Err(CompileError::new(
                    span,
                    "trigger_deprecation() takes at least 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Void))
        }
        "microtime" => {
            if args.len() > 1 {
                return Err(CompileError::new(span, "microtime() takes 0 or 1 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // PHP: `microtime()` / `microtime(false)` returns the "0.NNNNNNNN SSSSSSSSSS"
            // string; `microtime(true)` returns float seconds. A literal flag selects the
            // concrete form for the type checker (and the arg-aware EIR result type), while a
            // non-literal flag yields `string|float` (boxed `Mixed`), matching the runtime
            // `__rt_microtime_mixed` branch. Keep this in lockstep with `call_return_type_for_args`
            // and `call_return_type` in `src/ir_lower/expr/mod.rs`.
            Ok(Some(match args.first() {
                Some(arg) => match &arg.kind {
                    ExprKind::BoolLiteral(true) => PhpType::Float,
                    ExprKind::BoolLiteral(false) => PhpType::Str,
                    _ => checker.normalize_union_type(vec![PhpType::Str, PhpType::Float]),
                },
                None => PhpType::Str,
            }))
        }
        "sleep" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "sleep() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "usleep" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "usleep() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Void))
        }
        "http_response_code" => {
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "http_response_code() takes 0 or 1 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "header" => {
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(span, "header() takes 1 to 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Void))
        }
        "getenv" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "getenv() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "putenv" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "putenv() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "date_default_timezone_set" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "date_default_timezone_set() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "date_default_timezone_get" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "date_default_timezone_get() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Str))
        }
        "php_uname" => {
            if args.len() > 1 {
                return Err(CompileError::new(span, "php_uname() takes 0 or 1 arguments"));
            }
            if let Some(arg) = args.first() {
                let ty = checker.infer_type(arg, env)?;
                if ty != PhpType::Str {
                    return Err(CompileError::new(span, "php_uname() argument must be string"));
                }
            }
            Ok(Some(PhpType::Str))
        }
        "phpversion" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "phpversion() takes no arguments"));
            }
            Ok(Some(PhpType::Str))
        }
        "extension_loaded" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "extension_loaded() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "class_attribute_names" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "class_attribute_names() takes exactly 1 argument",
                ));
            }
            // Resolve at compile time: only string-literal class names are
            // supported in this iteration. Dynamic class names would require
            // a runtime name→class_id lookup table that elephc does not yet
            // expose.
            let arg_ty = checker.infer_type(&args[0], env)?;
            if !matches!(arg_ty, PhpType::Str) {
                return Err(CompileError::new(
                    span,
                    "class_attribute_names() argument must be a string class name",
                ));
            }
            let ExprKind::StringLiteral(class_name) = &args[0].kind else {
                return Err(CompileError::new(
                    span,
                    "class_attribute_names() requires a string literal class name (dynamic lookup is not yet supported)",
                ));
            };
            if resolve_class_name(checker, class_name).is_none() {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "class_attribute_names(): undefined class '{}'",
                        class_name
                    ),
                ));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
        }
        "class_attribute_args" => {
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args() takes exactly 2 arguments",
                ));
            }
            let class_arg_ty = checker.infer_type(&args[0], env)?;
            if !matches!(class_arg_ty, PhpType::Str) {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args() first argument must be a string class name",
                ));
            }
            let attr_arg_ty = checker.infer_type(&args[1], env)?;
            if !matches!(attr_arg_ty, PhpType::Str) {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args() second argument must be a string attribute name",
                ));
            }
            let ExprKind::StringLiteral(class_name) = &args[0].kind else {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args() requires a string literal class name (dynamic lookup is not yet supported)",
                ));
            };
            if !matches!(args[1].kind, ExprKind::StringLiteral(_)) {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args() requires a string literal attribute name (dynamic lookup is not yet supported)",
                ));
            }
            if resolve_class_name(checker, class_name).is_none() {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "class_attribute_args(): undefined class '{}'",
                        class_name
                    ),
                ));
            }
            let ExprKind::StringLiteral(attr_name) = &args[1].kind else {
                unreachable!("attribute argument literal checked above");
            };
            if class_attribute_args_unsupported(checker, class_name, attr_name) {
                return Err(CompileError::new(
                    span,
                    "class_attribute_args(): requested attribute uses argument metadata that is not supported yet",
                ));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Mixed))))
        }
        "class_get_attributes" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "class_get_attributes() takes exactly 1 argument",
                ));
            }
            let arg_ty = checker.infer_type(&args[0], env)?;
            if !matches!(arg_ty, PhpType::Str) {
                return Err(CompileError::new(
                    span,
                    "class_get_attributes() argument must be a string class name",
                ));
            }
            let ExprKind::StringLiteral(class_name) = &args[0].kind else {
                return Err(CompileError::new(
                    span,
                    "class_get_attributes() requires a string literal class name (dynamic lookup is not yet supported)",
                ));
            };
            if resolve_class_name(checker, class_name).is_none() {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "class_get_attributes(): undefined class '{}'",
                        class_name
                    ),
                ));
            }
            if class_get_attributes_unsupported(checker, class_name) {
                return Err(CompileError::new(
                    span,
                    "class_get_attributes(): class has attribute argument metadata that is not supported yet",
                ));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Object(
                "ReflectionAttribute".to_string(),
            )))))
        }
        "exec" | "shell_exec" | "system" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "passthru" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "passthru() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Void))
        }
        "define" => {
            if args.len() != 2 {
                return Err(CompileError::new(span, "define() takes exactly 2 arguments"));
            }
            let name_str = match &args[0].kind {
                ExprKind::StringLiteral(s) => s.clone(),
                _ => {
                    return Err(CompileError::new(
                        span,
                        "define() first argument must be a string literal",
                    ));
                }
            };
            let ty = checker.infer_type(&args[1], env)?;
            checker.constants.entry(name_str).or_insert(ty);
            Ok(Some(PhpType::Bool))
        }
        "defined" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "defined() takes exactly 1 argument"));
            }
            // A string-literal name still folds to a compile-time boolean later; a
            // non-literal name is accepted here and lowered to the `__rt_defined`
            // closed-world constant-registry lookup.
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "constant" => {
            if args.len() != 1 {
                return Err(CompileError::new(span, "constant() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            // A literal global-constant name keeps its precise checker type (the
            // value is folded during lowering); every other name — non-literal or
            // a class/enum `::` form — resolves at runtime through `__rt_constant`
            // and is typed as `Mixed`.
            if let ExprKind::StringLiteral(name) = &args[0].kind {
                let canonical = name.trim_start_matches('\\');
                if !canonical.contains("::") {
                    if let Some(ty) = checker.constants.get(canonical) {
                        return Ok(Some(ty.clone()));
                    }
                }
            }
            Ok(Some(PhpType::Mixed))
        }
        "date" | "gmdate" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    &format!("{name}() takes 1 or 2 arguments"),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "mktime" | "gmmktime" | "__elephc_mktime_raw" | "__elephc_gmmktime_raw" => {
            if args.len() != 6 {
                return Err(CompileError::new(
                    span,
                    &format!("{name}() takes exactly 6 arguments"),
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "checkdate" => {
            if args.len() != 3 {
                return Err(CompileError::new(span, "checkdate() takes exactly 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "hrtime" => {
            if args.len() > 1 {
                return Err(CompileError::new(span, "hrtime() takes at most 1 argument"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "localtime" => {
            if args.len() > 2 {
                return Err(CompileError::new(span, "localtime() takes at most 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "getdate" => {
            if args.len() > 1 {
                return Err(CompileError::new(span, "getdate() takes at most 1 argument"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // getdate() always returns a (heterogeneous int/string) associative array. The emitter
            // boxes it into a Mixed cell, so the inferred type is Mixed, like stat()/fstat().
            Ok(Some(PhpType::Mixed))
        }
        "strtotime" => {
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "strtotime() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            // PHP returns int|false: the timestamp, or false when the string cannot be parsed.
            Ok(Some(PhpType::Union(vec![PhpType::Int, PhpType::Bool])))
        }
        "__elephc_strtotime_raw" => {
            // Internal alias used by the synthetic DateTime constructor and modify():
            // identical parsing, but a raw integer result (failure maps to -1) so object
            // timestamp storage stays a plain int slot.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(span, "strtotime() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "json_encode" => {
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "json_encode() takes 1 to 3 arguments",
                ));
            }
            checker.infer_type(&args[0], env)?;
            for extra in &args[1..] {
                let ty = checker.infer_type(extra, env)?;
                if ty != PhpType::Int {
                    return Err(CompileError::new(
                        extra.span,
                        "json_encode() flags and depth must be integers",
                    ));
                }
            }
            Ok(Some(PhpType::Str))
        }
        "json_decode" => {
            if args.is_empty() || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "json_decode() takes 1 to 4 arguments",
                ));
            }
            let json_ty = checker.infer_type(&args[0], env)?;
            if !is_json_string_arg_type(&json_ty) {
                return Err(CompileError::new(
                    args[0].span,
                    "json_decode() json argument must be string-compatible",
                ));
            }
            if let Some(assoc) = args.get(1) {
                let assoc_ty = checker.infer_type(assoc, env)?;
                if !is_json_associative_arg_type(&assoc_ty) {
                    return Err(CompileError::new(
                        assoc.span,
                        "json_decode() associative argument must be bool-compatible or null",
                    ));
                }
            }
            for extra in args.iter().skip(2) {
                let ty = checker.infer_type(extra, env)?;
                if ty != PhpType::Int {
                    return Err(CompileError::new(
                        extra.span,
                        "json_decode() depth and flags must be integers",
                    ));
                }
            }
            // Returns a structural Mixed: scalars (null/bool/int/float/string)
            // box natively; arrays and objects currently fall back to a
            // Mixed(string) wrapping the trimmed JSON slice (full structural
            // decode of containers is on the roadmap).
            Ok(Some(PhpType::Mixed))
        }
        "json_validate" => {
            if args.is_empty() || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "json_validate() takes 1 to 3 arguments",
                ));
            }
            let json_ty = checker.infer_type(&args[0], env)?;
            if !is_json_string_arg_type(&json_ty) {
                return Err(CompileError::new(
                    args[0].span,
                    "json_validate() json argument must be string-compatible",
                ));
            }
            for extra in &args[1..] {
                let ty = checker.infer_type(extra, env)?;
                if ty != PhpType::Int {
                    return Err(CompileError::new(
                        extra.span,
                        "json_validate() depth and flags must be integers",
                    ));
                }
            }
            if let Some(flags) = args.get(2) {
                if let Some(value) = json_static_int_value(flags) {
                    const JSON_INVALID_UTF8_IGNORE: i64 = 1_048_576;
                    if value & !JSON_INVALID_UTF8_IGNORE != 0 {
                        return Err(CompileError::new(
                            flags.span,
                            "json_validate() flags must be 0 or JSON_INVALID_UTF8_IGNORE",
                        ));
                    }
                }
            }
            Ok(Some(PhpType::Bool))
        }
        "json_last_error" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "json_last_error() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Int))
        }
        "json_last_error_msg" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "json_last_error_msg() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Str))
        }
        "preg_last_error_msg" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "preg_last_error_msg() takes exactly 0 arguments",
                ));
            }
            Ok(Some(PhpType::Str))
        }
        "preg_last_error" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "preg_last_error() takes exactly 0 arguments",
                ));
            }
            Ok(Some(PhpType::Int))
        }
        "preg_match" => {
            // PHP: preg_match(string $pattern, string $subject, array &$matches = [],
            // int $flags = 0, int $offset = 0): int|false. `$matches` is a by-ref
            // out-parameter, so it must be a variable and is not eagerly inferred.
            if !(2..=5).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_match() takes 2 to 5 arguments",
                ));
            }
            checker.infer_type(&args[0], env)?;
            checker.infer_type(&args[1], env)?;
            if args.len() >= 3 && !matches!(args[2].kind, ExprKind::Variable(_)) {
                return Err(CompileError::new(
                    args[2].span,
                    "preg_match() parameter $matches must be passed a variable",
                ));
            }
            // `$flags` and `$offset` are read-only inputs.
            for arg in args.iter().skip(3) {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "preg_match_all" => {
            // PHP: preg_match_all(string $pattern, string $subject, array &$matches = null,
            //   int $flags = 0, int $offset = 0): int|false. `$matches` is a by-ref
            //   out-parameter, so it must be a variable and is not eagerly inferred
            //   (its value is produced by the call). `$flags` and `$offset` are read-only.
            if !(2..=5).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_match_all() takes 2 to 5 arguments",
                ));
            }
            checker.infer_type(&args[0], env)?;
            checker.infer_type(&args[1], env)?;
            if args.len() >= 3 && !matches!(args[2].kind, ExprKind::Variable(_)) {
                return Err(CompileError::new(
                    args[2].span,
                    "preg_match_all() parameter $matches must be passed a variable",
                ));
            }
            for arg in args.iter().skip(3) {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "preg_replace" => {
            // PHP: preg_replace(string|array $pattern, string|array $replacement,
            // string|array $subject, int $limit = -1, int &$count = null). `$count`
            // is a by-ref out-parameter: it must be a variable and is not eagerly
            // inferred (its value is produced by the call).
            if !(3..=5).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_replace() takes 3 to 5 arguments",
                ));
            }
            // pattern, replacement, subject, and the read-only $limit are inputs.
            for arg in args.iter().take(4) {
                checker.infer_type(arg, env)?;
            }
            if args.len() == 5 && !matches!(args[4].kind, ExprKind::Variable(_)) {
                return Err(CompileError::new(
                    args[4].span,
                    "preg_replace() parameter $count must be passed a variable",
                ));
            }
            Ok(Some(PhpType::Str))
        }
        "preg_split" => {
            if !(2..=4).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_split() takes between 2 and 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            let elem_ty = if args.len() >= 4 {
                PhpType::Mixed
            } else {
                PhpType::Str
            };
            Ok(Some(PhpType::Array(Box::new(elem_ty))))
        }
        // -- env/runtime state builtins (real AOT runtime; slice 1A) --
        "error_reporting" => {
            // error_reporting(?int $error_level = null): int — no-arg/null reads the
            // current level; a passed int sets it and returns the previous value.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "error_reporting() takes at most 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "ignore_user_abort" => {
            // ignore_user_abort(?bool $enable = null): int — no-arg/null reads the
            // current flag; a passed bool sets it (0/1) and returns the previous value.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "ignore_user_abort() takes at most 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "set_time_limit" => {
            // set_time_limit(int $seconds): bool — always true (a native binary has no timeout).
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "set_time_limit() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "connection_aborted" => {
            // connection_aborted(): int — always 0 (a compiled program's connection is never aborted).
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "connection_aborted() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Int))
        }
        "error_log" => {
            // error_log(string $message, int $message_type = 0, ?string $destination = null,
            // ?string $additional_headers = null): bool — writes $message to stderr, returns true.
            if args.is_empty() || args.len() > 4 {
                return Err(CompileError::new(
                    span,
                    "error_log() takes between 1 and 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        // -- process / system control builtins (recognition-only; runtime deferred) --
        // These are runtime-dead for the console app: registered so calls type-check and stop
        // being "Undefined function", but they have no EIR/codegen lowering yet.
        "pcntl_signal" => {
            // pcntl_signal(int $signal, callable|int $handler,
            // bool $restart_syscalls = true): bool.
            if args.len() < 2 || args.len() > 3 {
                return Err(CompileError::new(
                    span,
                    "pcntl_signal() takes 2 or 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "pcntl_alarm" => {
            // pcntl_alarm(int $seconds): int — seconds left on the previous alarm.
            if args.len() != 1 {
                return Err(CompileError::new(span, "pcntl_alarm() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "pcntl_async_signals" => {
            // pcntl_async_signals(?bool $enable = null): bool — returns the previous state.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "pcntl_async_signals() takes 0 or 1 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "pcntl_signal_get_handler" => {
            // pcntl_signal_get_handler(int $signal): int|string — an int constant
            // (SIG_DFL/SIG_IGN) or the installed callable's name.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "pcntl_signal_get_handler() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Str,
            ])))
        }
        "proc_close" => {
            // proc_close($process): int — waits on the process and returns its exit status.
            if args.len() != 1 {
                return Err(CompileError::new(span, "proc_close() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Int))
        }
        "getmypid" => {
            // getmypid(): int|false — the current process id.
            if !args.is_empty() {
                return Err(CompileError::new(span, "getmypid() takes no arguments"));
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Int,
                PhpType::Bool,
            ])))
        }
        "cli_set_process_title" | "setproctitle" => {
            // cli_set_process_title(string $title): bool; setproctitle is the ext/proctitle
            // alias with the same shape.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    &format!("{}() takes exactly 1 argument", name),
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "sapi_windows_cp_get" => {
            // sapi_windows_cp_get(string $kind = ''): int — Windows-only; recognized everywhere.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "sapi_windows_cp_get() takes 0 or 1 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Int))
        }
        "sapi_windows_cp_set" => {
            // sapi_windows_cp_set(int $code_page): bool.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "sapi_windows_cp_set() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Bool))
        }
        "sapi_windows_vt100_support" => {
            // sapi_windows_vt100_support($stream, ?bool $enable = null): bool.
            if args.is_empty() || args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "sapi_windows_vt100_support() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "ini_get" => {
            // ini_get(string $option): string|false — the directive value, or false if unset.
            if args.len() != 1 {
                return Err(CompileError::new(span, "ini_get() takes exactly 1 argument"));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "ini_set" => {
            // ini_set(string $option, mixed $value): string|false — set a runtime ini
            // directive and return its PREVIOUS value as a string, or false if unset.
            // The value argument accepts string|int|float|bool|null (coerced to string).
            if args.len() != 2 {
                return Err(CompileError::new(span, "ini_set() takes exactly 2 arguments"));
            }
            checker.infer_type(&args[0], env)?;
            checker.infer_type(&args[1], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "get_cfg_var" => {
            // get_cfg_var(string $option): string|false — the compiled master value for a
            // known core directive, or false for unknown/unset directives. Not affected by
            // ini_set (reads immutable master defaults).
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "get_cfg_var() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Bool,
            ])))
        }
        "get_defined_constants" => {
            // get_defined_constants(bool $categorize = false): array.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "get_defined_constants() takes 0 or 1 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Mixed))))
        }
        // -- misc / error-handling / process builtins (recognition-only; runtime deferred) --
        // These type-check so calls stop being "Undefined function"; no EIR/codegen lowering yet.
        "method_exists" => {
            // method_exists(object|string $object_or_class, string $method): bool.
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    "method_exists() takes exactly 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "trigger_error" => {
            // trigger_error(string $message, int $error_level = E_USER_NOTICE): bool.
            if !(1..=2).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "trigger_error() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "set_error_handler" => {
            // set_error_handler(?callable $callback, int $error_levels = E_ALL): ?callable.
            // The previous handler is returned; modeled as Mixed (callable-or-null).
            if !(1..=2).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "set_error_handler() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "restore_error_handler" => {
            // restore_error_handler(): true.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "restore_error_handler() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Bool))
        }
        "restore_exception_handler" => {
            // restore_exception_handler(): true — pops the previously installed
            // exception handler and always returns true.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "restore_exception_handler() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Bool))
        }
        "filter_var" => {
            // filter_var(mixed $value, int $filter = FILTER_DEFAULT,
            // array|int $options = 0): mixed — the filtered value, or false on
            // failure. Recognition-only; no EIR/runtime lowering yet.
            if !(1..=3).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "filter_var() takes 1 to 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Mixed))
        }
        "set_exception_handler" => {
            // set_exception_handler(?callable $callback): ?callable — the previous handler.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "set_exception_handler() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Mixed))
        }
        "preg_quote" => {
            // preg_quote(string $str, ?string $delimiter = null): string.
            if !(1..=2).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_quote() takes 1 or 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "preg_grep" => {
            // preg_grep(string $pattern, array $array, int $flags = 0): array|false.
            if !(2..=3).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "preg_grep() takes 2 or 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Bool,
            ])))
        }
        "version_compare" => {
            // version_compare(string $version1, string $version2,
            // ?string $operator = null): int|bool — int (-1/0/1) or bool when an operator is given.
            if !(2..=3).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "version_compare() takes 2 or 3 arguments",
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
        "unpack" => {
            // unpack(string $format, string $string, int $offset = 0): array|false.
            if !(2..=3).contains(&args.len()) {
                return Err(CompileError::new(span, "unpack() takes 2 or 3 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Array(Box::new(PhpType::Mixed)),
                PhpType::Bool,
            ])))
        }
        "random_bytes" => {
            // random_bytes(int $length): string.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "random_bytes() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "http_build_query" => {
            // http_build_query(array|object $data, string $numeric_prefix = '',
            // ?string $arg_separator = null, int $encoding_type = PHP_QUERY_RFC1738): string.
            if !(1..=4).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "http_build_query() takes 1 to 4 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Str))
        }
        "escapeshellarg" => {
            // escapeshellarg(string $arg): string.
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "escapeshellarg() takes exactly 1 argument",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Str))
        }
        "assert" => {
            // assert(mixed $assertion, Throwable|string|null $description = null): bool.
            if !(1..=2).contains(&args.len()) {
                return Err(CompileError::new(span, "assert() takes 1 or 2 arguments"));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "sapi_windows_cp_conv" => {
            // sapi_windows_cp_conv(int|string $in_codepage, int|string $out_codepage,
            // string $subject): ?string (Windows-only; recognized on every target).
            if args.len() != 3 {
                return Err(CompileError::new(
                    span,
                    "sapi_windows_cp_conv() takes exactly 3 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(checker.normalize_union_type(vec![
                PhpType::Str,
                PhpType::Void,
            ])))
        }
        "posix_kill" => {
            // posix_kill(int $process_id, int $signal): bool.
            if args.len() != 2 {
                return Err(CompileError::new(
                    span,
                    "posix_kill() takes exactly 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        _ => Ok(None),
    }
}

/// Resolves a class name to its canonical key in the checker's class table.
/// Returns `Some(canonical_name)` if the class exists, `None` otherwise.
/// The lookup is case-insensitive per PHP rules.
fn resolve_class_name<'a>(checker: &'a Checker, class_name: &str) -> Option<&'a str> {
    let class_key = php_symbol_key(class_name.trim_start_matches('\\'));
    checker
        .classes
        .keys()
        .find(|existing| php_symbol_key(existing) == class_key)
        .map(String::as_str)
}

/// Returns `true` if `ty` is a valid type for the JSON string argument in
/// `json_decode` / `json_validate` / `json_encode` (scalar types and `Mixed`).
fn is_json_string_arg_type(ty: &PhpType) -> bool {
    match ty {
        PhpType::Str
        | PhpType::Int
        | PhpType::Float
        | PhpType::Bool
        | PhpType::Void
        | PhpType::Mixed => true,
        PhpType::Union(types) => types.iter().all(is_json_string_arg_type),
        _ => false,
    }
}

/// Returns `true` if `ty` is a valid type for the associative argument in
/// `json_decode` (bool-compatible types plus `Mixed`).
fn is_json_associative_arg_type(ty: &PhpType) -> bool {
    match ty {
        PhpType::Bool
        | PhpType::Int
        | PhpType::Float
        | PhpType::Str
        | PhpType::Void
        | PhpType::Mixed => true,
        PhpType::Union(types) => types.iter().all(is_json_associative_arg_type),
        _ => false,
    }
}

/// Attempts to evaluate an expression as a static integer at compile time.
/// Supports literals, known constants, negation, and bitwise ops.
/// Returns `Some(value)` if the expression is statically computable, `None` otherwise.
fn json_static_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::ConstRef(name) => JSON_INT_CONSTANTS
            .iter()
            .find_map(|(constant, value)| (*constant == name.as_str()).then_some(*value)),
        ExprKind::Negate(inner) => json_static_int_value(inner).map(|value| -value),
        ExprKind::BinaryOp { left, op, right } => {
            let left = json_static_int_value(left)?;
            let right = json_static_int_value(right)?;
            match op {
                BinOp::BitAnd => Some(left & right),
                BinOp::BitOr => Some(left | right),
                BinOp::BitXor => Some(left ^ right),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Returns `true` if the named attribute on the class uses argument metadata
/// that the compiler does not yet support (i.e., `attribute_args` slot is `None`).
fn class_attribute_args_unsupported(checker: &Checker, class_name: &str, attr_name: &str) -> bool {
    let Some(resolved_class) = resolve_class_name(checker, class_name) else {
        return false;
    };
    let Some(class_info) = checker.classes.get(resolved_class) else {
        return false;
    };
    let attr_key = php_symbol_key(attr_name.trim_start_matches('\\'));
    class_info
        .attribute_names
        .iter()
        .enumerate()
        .find(|(_, name)| php_symbol_key(name.trim_start_matches('\\')) == attr_key)
        .is_some_and(|(idx, _)| match class_info.attribute_args.get(idx) {
            // The flat `class_attribute_args()` helper returns a positional
            // array of materialized scalars, so it cannot faithfully echo keyed
            // arguments (named arguments or associative arrays, at any depth) or
            // deferred symbolic references (global/class constants, enum cases).
            // Reject them and direct users to
            // `ReflectionClass::getAttributes()->getArguments()` instead.
            Some(Some(entries)) => attr_entries_unsupported_by_flat_helper(entries),
            _ => true,
        })
}

/// Returns true when the flat `class_attribute_args()` helper cannot faithfully
/// echo the captured entries: keyed arguments (named arguments or
/// associative-array keys, at any depth) would lose their keys, and deferred
/// symbolic references (global/class constants, enum cases) are not materialized
/// on this echo path. Both are supported through
/// `ReflectionClass::getAttributes()->getArguments()` instead.
fn attr_entries_unsupported_by_flat_helper(entries: &[crate::types::AttrArgEntry]) -> bool {
    entries.iter().any(|entry| {
        entry.key.is_some()
            || matches!(
                &entry.value,
                crate::types::AttrArgValue::ConstRef(_)
                    | crate::types::AttrArgValue::ScopedConst(..)
            )
            || matches!(
                &entry.value,
                crate::types::AttrArgValue::Array(inner)
                    if attr_entries_unsupported_by_flat_helper(inner)
            )
    })
}

/// Returns `true` if the class has any attribute whose argument metadata is not
/// fully supported (slot count mismatch or any `None` slot in `attribute_args`).
fn class_get_attributes_unsupported(checker: &Checker, class_name: &str) -> bool {
    let Some(resolved_class) = resolve_class_name(checker, class_name) else {
        return false;
    };
    checker.classes.get(resolved_class).is_some_and(|class_info| {
        class_info.attribute_names.len() != class_info.attribute_args.len()
            || class_info.attribute_args.iter().any(Option::is_none)
    })
}
