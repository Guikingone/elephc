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
use crate::types::filter_constants::FILTER_INT_CONSTANTS;
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
        "func_num_args" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "func_num_args() takes no arguments"));
            }
            Ok(Some(PhpType::Int))
        }
        "func_get_args" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "func_get_args() takes no arguments"));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Mixed))))
        }
        "func_get_arg" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "func_get_arg() expects exactly 1 argument (position)",
                ));
            }
            checker.infer_type(&args[0], env)?;
            Ok(Some(PhpType::Mixed))
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
            //
            // A spread argument (`trigger_deprecation(...$args)`) unpacks at runtime into a
            // statically-unknown number of positional arguments, so the pre-expansion count is
            // not the real arity and the compile-time gate must be skipped — exactly as
            // `Checker::check_builtin` already does for registry-backed builtins. Symfony's
            // `ParameterBag::get()` calls `trigger_deprecation(...$this->deprecatedParameters[$name])`.
            let has_spread = args.iter().any(|arg| matches!(arg.kind, ExprKind::Spread(_)));
            if !has_spread && args.len() < 3 {
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
        // -- output buffering (K2): a capture-buffer stack sharing __rt_stdout_write's --
        // choke point (echo/print/scalar-to-string writes). elephc supports only the
        // plain, callback-free ob_start() form (see the signature's zero-parameter arity).
        "ob_start" => {
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "ob_start() takes no arguments in AOT mode (callback/chunk_size forms are unsupported)",
                ));
            }
            Ok(Some(PhpType::Bool))
        }
        "ob_get_contents" | "ob_end_flush" | "ob_get_clean" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, &format!("{}() takes no arguments", name)));
            }
            Ok(Some(if name == "ob_end_flush" {
                PhpType::Bool
            } else {
                checker.normalize_union_type(vec![PhpType::Str, PhpType::Bool])
            }))
        }
        "ob_end_clean" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "ob_end_clean() takes no arguments"));
            }
            Ok(Some(PhpType::Bool))
        }
        "ob_get_level" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "ob_get_level() takes no arguments"));
            }
            Ok(Some(PhpType::Int))
        }
        "ob_get_status" => {
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "ob_get_status() takes at most 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::AssocArray {
                key: Box::new(PhpType::Str),
                value: Box::new(PhpType::Mixed),
            }))
        }
        "headers_sent" => {
            if args.len() > 2 {
                return Err(CompileError::new(
                    span,
                    "headers_sent() takes at most 2 arguments",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "flush" => {
            if !args.is_empty() {
                return Err(CompileError::new(span, "flush() takes no arguments"));
            }
            Ok(Some(PhpType::Void))
        }
        "header_remove" => {
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "header_remove() takes at most 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Void))
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
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "phpversion() takes at most 1 argument",
                ));
            }
            // phpversion(): the runtime version string. phpversion($extension): the
            // extension's version or false. elephc has no loadable extensions, so any
            // extension query is the concrete PHP false (bool).
            if let Some(arg) = args.first() {
                checker.infer_type(arg, env)?;
                return Ok(Some(PhpType::Bool));
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
        "gc_collect_cycles" => {
            // gc_collect_cycles(): int — runs the real cycle collector. The freed-cycle
            // count is not tracked (honest AOT limitation), so 0 is returned.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "gc_collect_cycles() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Int))
        }
        "gc_enabled" => {
            // gc_enabled(): bool — returns the queryable garbage-collector enabled flag,
            // which defaults to true (PHP's default) and round-trips through gc_enable/gc_disable.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "gc_enabled() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Bool))
        }
        "gc_enable" => {
            // gc_enable(): void — sets the queryable garbage-collector enabled flag to 1.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "gc_enable() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Void))
        }
        "gc_disable" => {
            // gc_disable(): void — sets the queryable garbage-collector enabled flag to 0.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "gc_disable() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Void))
        }
        "gc_mem_caches" => {
            // gc_mem_caches(): int — always 0 (elephc has no request-scoped memory cache to free).
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "gc_mem_caches() takes no arguments",
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
        "error_get_last" => {
            // error_get_last(): ?array — null when no error has been recorded. elephc records
            // no runtime error into any state today (trigger_error() is registered but not
            // EIR-backend-implemented, so it fails loudly at compile time instead of silently
            // succeeding), so this is faithful for every program that compiles and runs: null,
            // always. A real global slot backs the value so future error-recording work can
            // populate it without a checker/signature change.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "error_get_last() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Union(vec![
                PhpType::Void,
                PhpType::Array(Box::new(PhpType::Mixed)),
            ])))
        }
        "libxml_use_internal_errors" => {
            // libxml_use_internal_errors(?bool $use_errors = null): bool — null/no-arg reads
            // the current flag without changing it; a passed bool sets it and the call
            // returns the *previous* value, matching PHP.
            if args.len() > 1 {
                return Err(CompileError::new(
                    span,
                    "libxml_use_internal_errors() takes at most 1 argument",
                ));
            }
            for arg in args {
                checker.infer_type(arg, env)?;
            }
            Ok(Some(PhpType::Bool))
        }
        "libxml_clear_errors" => {
            // libxml_clear_errors(): void — no-op. elephc has no libxml/DOM subsystem, so there
            // is never a recorded parse error to clear; this is observably identical to PHP
            // clearing an empty error buffer.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "libxml_clear_errors() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Void))
        }
        "libxml_get_errors" => {
            // libxml_get_errors(): array — always an empty array. elephc has no libxml/DOM
            // subsystem, so no libxml parse error can ever be recorded; PHP's own behavior
            // with no recorded errors is exactly an empty array, so this is byte-identical.
            if !args.is_empty() {
                return Err(CompileError::new(
                    span,
                    "libxml_get_errors() takes no arguments",
                ));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Mixed))))
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
            // array|int $options = 0): mixed — the filtered value, `false` on
            // failure, or `null` on failure when FILTER_NULL_ON_FAILURE is set.
            //
            // Core semantics (VALIDATE_INT/FLOAT/BOOL/IP + DEFAULT/UNSAFE_RAW
            // passthrough) are implemented with dedicated runtime parsers — see
            // `crate::ir_lower::expr::filter` and
            // `crate::codegen_ir::lower_inst::builtins::filter`. VALIDATE_IP
            // honors FILTER_FLAG_IPV4/FILTER_FLAG_IPV6 as family restrictions
            // (either flag alone restricts to that family; both or neither
            // accept either family) via libc `inet_pton`; FLAG_NO_PRIV_RANGE/
            // NO_RES_RANGE stay loud (see `filter_constants`'s module doc).
            // Everything else (VALIDATE_EMAIL/URL/MAC/DOMAIN/REGEXP, array-form
            // `$options`, FILTER_CALLBACK, REQUIRE_ARRAY/FORCE_ARRAY) is kept LOUD here rather
            // than silently mis-validated. FILTER_REQUIRE_SCALAR is accepted as a
            // verified no-op: without REQUIRE_ARRAY/FORCE_ARRAY an array input
            // already fails every supported filter by default (php-verified), so
            // REQUIRE_SCALAR never changes observable behavior in this scope —
            // this unblocks Symfony's `InputBag::filter()`, which always sets it.
            if !(1..=3).contains(&args.len()) {
                return Err(CompileError::new(
                    span,
                    "filter_var() takes 1 to 3 arguments",
                ));
            }
            let value_ty = checker.infer_type(&args[0], env)?;
            if !is_filter_var_value_type(&value_ty) {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "filter_var(): unsupported value type {:?} is not supported yet",
                        value_ty
                    ),
                ));
            }
            let filter_id = if args.len() >= 2 {
                checker.infer_type(&args[1], env)?;
                match filter_static_int_value(&args[1]) {
                    Some(v) => v,
                    None => {
                        // A dynamic (runtime) `$filter` is routed to the
                        // `__elephc_filter_var_dyn` prelude helper by
                        // `crate::ir_lower::expr::filter::lower_static_filter_var`, which dispatches
                        // the runtime id to the SAME per-filter literal lowering. Type-check the
                        // remaining argument and accept the call as `Mixed` (the filtered value,
                        // `false`, or `null`), exactly as the literal path is typed.
                        if let Some(options) = args.get(2) {
                            checker.infer_type(options, env)?;
                        }
                        return Ok(Some(PhpType::Mixed));
                    }
                }
            } else {
                516 // FILTER_DEFAULT (== FILTER_UNSAFE_RAW)
            };
            if !matches!(filter_id, 516 | 257 | 258 | 259 | 275) {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "filter_var(): filter {} is not supported yet",
                        filter_id
                    ),
                ));
            }
            // FILTER_VALIDATE_INT with a compile-time-constant integer range
            // constraint (`['options' => ['min_range' => C, 'max_range' => C]]`,
            // optional constant `flags`). ir_lower resolves the SAME options via
            // `static_filter_int_range_options`, so an accepted call always lowers.
            if filter_id == 257 && args.len() == 3 {
                if let Some(range) =
                    crate::types::filter_constants::static_filter_int_range_options(&args[2])
                {
                    // Only NULL_ON_FAILURE / REQUIRE_SCALAR (verified no-op) are
                    // honored, matching the scalar filters' allowed-flags set.
                    const RANGE_ALLOWED_FLAGS: i64 = 134_217_728 | 33_554_432;
                    if range.flags & !RANGE_ALLOWED_FLAGS != 0 {
                        return Err(CompileError::new(
                            span,
                            &format!(
                                "filter_var(): flag combination {} is not supported yet",
                                range.flags
                            ),
                        ));
                    }
                    return Ok(Some(PhpType::Mixed));
                }
            }
            let flags = if args.len() == 3 {
                // Resolve the effective flags int, accepting either a constant integer flags
                // expression or the `['flags' => <const>]`-only array form (semantically
                // identical in PHP). `static_filter_options_flags` returns `None` for an array
                // carrying an `options` entry (min_range/max_range/regexp — unimplemented) or any
                // non-constant shape; `crate::ir_lower::expr::filter` uses the SAME resolver, so an
                // accepted call always lowers.
                match crate::types::filter_constants::static_filter_options_flags(&args[2]) {
                    Some(v) => v,
                    None => match &args[2].kind {
                        ExprKind::ArrayLiteral(_) | ExprKind::ArrayLiteralAssoc(_) => {
                            return Err(CompileError::new(
                                span,
                                "filter_var(): array-form $options (['flags' => ..., 'options' => ...]) is not supported yet",
                            ));
                        }
                        _ => {
                            let options_ty = checker.infer_type(&args[2], env)?;
                            if matches!(
                                options_ty,
                                PhpType::Array(_) | PhpType::AssocArray { .. }
                            ) {
                                return Err(CompileError::new(
                                    span,
                                    "filter_var(): array-form $options (['flags' => ..., 'options' => ...]) is not supported yet",
                                ));
                            }
                            return Err(CompileError::new(
                                span,
                                "filter_var(): a dynamic (non-compile-time-constant) $options is not supported yet",
                            ));
                        }
                    },
                }
            } else {
                0
            };
            const ALLOWED_FLAGS: i64 = 134_217_728 /* NULL_ON_FAILURE */ | 33_554_432 /* REQUIRE_SCALAR, verified no-op */;
            // FILTER_FLAG_IPV4 (1048576) / FILTER_FLAG_IPV6 (2097152) only apply
            // to FILTER_VALIDATE_IP; every other filter keeps its existing
            // allowed-flags set. FILTER_FLAG_NO_PRIV_RANGE/NO_RES_RANGE are
            // deliberately absent from both sets (see `filter_constants`'s module
            // doc): PHP's private/reserved-range matrix has quirks (e.g.
            // link-local IPv6 is NOT "private" for this flag) that would need a
            // full php-verified v4+v6 implementation, so they stay loud.
            const IP_ALLOWED_FLAGS: i64 = ALLOWED_FLAGS | 1_048_576 /* FLAG_IPV4 */ | 2_097_152 /* FLAG_IPV6 */;
            let allowed_flags = if filter_id == 275 {
                IP_ALLOWED_FLAGS
            } else {
                ALLOWED_FLAGS
            };
            if flags & !allowed_flags != 0 {
                return Err(CompileError::new(
                    span,
                    &format!(
                        "filter_var(): flag combination {} is not supported yet",
                        flags
                    ),
                ));
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
        "get_class_methods" => {
            if args.len() != 1 {
                return Err(CompileError::new(
                    span,
                    "get_class_methods() takes exactly 1 argument",
                ));
            }
            let ty = checker.infer_type(&args[0], env)?;
            // A literal class name and a statically-typed object are compile-time baked; any
            // other value that can CARRY a class name at runtime (a computed string, or a
            // gradual `Mixed`/union) resolves through the `_class_methods_table` registry
            // instead — see the EIR lowering in
            // `crate::codegen::lower_inst::builtins::get_class_methods`. Both sides must accept
            // the same set: widening here without the runtime path would only relocate the
            // failure into codegen.
            //
            // Anything that can never name a class (int, float, bool, array, …) stays refused.
            // PHP raises a TypeError there, and the runtime path raises that same TypeError for
            // a `Mixed` that turns out to hold one — this is the statically-provable case, so
            // it is reported at compile time rather than deferred.
            fn can_name_a_class(ty: &PhpType) -> bool {
                match ty {
                    PhpType::Object(_) | PhpType::Str | PhpType::Mixed => true,
                    PhpType::Union(members) => members.iter().any(can_name_a_class),
                    _ => false,
                }
            }
            if !can_name_a_class(&ty) {
                return Err(CompileError::new(
                    span,
                    "get_class_methods() requires an object or a class-name string; a value of \
                     this type can never name a class",
                ));
            }
            Ok(Some(PhpType::Array(Box::new(PhpType::Str))))
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

/// Returns `true` if `ty` is a value type `filter_var()` core lowering supports.
///
/// Scalars (Int/Float/Str/Bool/Void=null) and `Mixed` are dispatched at runtime
/// by their boxed tag. `Array`/`AssocArray` are supported too: without
/// `FILTER_REQUIRE_ARRAY`/`FILTER_FORCE_ARRAY` (both kept LOUD via the flags
/// check), PHP's `filter_var()` always fails on array input regardless of the
/// filter, so a statically-known array input trivially lowers to a constant
/// failure result — real behavior, not a stub. `Union(_)` is supported too:
/// `PhpType::codegen_repr()` collapses every non-tagged-scalar union (e.g.
/// `string|bool`, common on real Symfony env/input variables) to `Mixed`, and
/// elephc represents a union value at runtime as the SAME boxed-Mixed cell as
/// a genuine `Mixed` value — so `filter_var()`'s Mixed-tag dispatch already
/// handles it soundly with no separate code path. Everything else (objects,
/// callables, resources, pointers, buffers, iterables) is unsupported.
fn is_filter_var_value_type(ty: &PhpType) -> bool {
    matches!(
        ty,
        PhpType::Int
            | PhpType::Float
            | PhpType::Str
            | PhpType::Bool
            | PhpType::False
            | PhpType::Void
            | PhpType::Mixed
            | PhpType::Array(_)
            | PhpType::AssocArray { .. }
            | PhpType::Union(_)
    )
}

/// Attempts to evaluate an expression as a static integer at compile time
/// against the `ext/filter` constant table (`FILTER_INT_CONSTANTS`).
/// Supports literals, known filter constants, negation, and bitwise ops (so a
/// combined `FILTER_NULL_ON_FAILURE | FILTER_REQUIRE_SCALAR` flags expression
/// resolves statically). Returns `Some(value)` if the expression is statically
/// computable, `None` otherwise (a genuinely dynamic `$filter`/`$options`).
fn filter_static_int_value(expr: &Expr) -> Option<i64> {
    match &expr.kind {
        ExprKind::IntLiteral(value) => Some(*value),
        ExprKind::ConstRef(name) => FILTER_INT_CONSTANTS
            .iter()
            .find_map(|(constant, value)| (*constant == name.as_str()).then_some(*value)),
        ExprKind::Negate(inner) => filter_static_int_value(inner).map(|value| -value),
        ExprKind::BinaryOp { left, op, right } => {
            let left = filter_static_int_value(left)?;
            let right = filter_static_int_value(right)?;
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
