//! Purpose:
//! Defines function signature metadata for user functions, builtins, closures, and callable aliases.
//! Stores parameter names, defaults, variadics, by-reference behavior, and return contracts used by call planning.
//!
//! Called from:
//! - `crate::types::checker::functions`
//! - `crate::types::call_args`
//!
//! Key details:
//! - Builtin signatures must match PHP so named arguments, first-class callables, and mutation semantics stay coherent.

use crate::parser::ast::{AttributeGroup, Expr, ExprKind, TypeExpr};
use crate::span::Span;

use super::PhpType;

#[derive(Debug, Clone, PartialEq)]
/// Metadata for a callable's parameter and return type contract.
///
/// Used by call planning, named-argument resolution, first-class callables,
/// and type inference. Builtin signatures must match PHP for coherence with
/// named arguments, callable aliases, and mutation semantics.
pub struct FunctionSig {
    pub params: Vec<(String, PhpType)>,
    pub param_type_exprs: Vec<Option<TypeExpr>>,
    pub param_attributes: Vec<Vec<AttributeGroup>>,
    pub defaults: Vec<Option<Expr>>,
    pub return_type: PhpType,
    pub declared_return: bool,
    /// `true` when declared with `function &f()` / `fn &()` — the function returns a
    /// reference (alias) to the returned lvalue rather than a copy.
    pub by_ref_return: bool,
    pub ref_params: Vec<bool>,
    pub declared_params: Vec<bool>,
    pub variadic: Option<String>,
    /// `Some(message)` if the declaration carried PHP 8.4 `#[\Deprecated]`.
    /// `Some("")` indicates the attribute was present without an explicit
    /// reason. `None` means the function/method is not deprecated.
    pub deprecation: Option<String>,
}

/// Upgrades a variadic signature for use as a first-class callable.
///
/// If the variadic parameter is not already typed as `Array`, upgrades it to
/// `Array<Mixed>`. Non-variadic signatures are returned unchanged.
///
/// Called from:
/// - first-class callable lowering in codegen
pub(crate) fn callable_wrapper_sig(sig: &FunctionSig) -> FunctionSig {
    let Some(variadic_name) = sig.variadic.as_ref() else {
        return sig.clone();
    };

    let mut wrapper_sig = sig.clone();
    if let Some((name, ty)) = wrapper_sig.params.last_mut() {
        if name == variadic_name {
            if !matches!(ty, PhpType::Array(_)) {
                *ty = PhpType::Array(Box::new(PhpType::Mixed));
            }
            return wrapper_sig;
        }
    }

    let variadic_index = wrapper_sig.params.len();
    let variadic_type_expr = if wrapper_sig.param_type_exprs.len() > variadic_index {
        wrapper_sig.param_type_exprs.remove(variadic_index)
    } else {
        None
    };
    let variadic_attributes = if wrapper_sig.param_attributes.len() > variadic_index {
        wrapper_sig.param_attributes.remove(variadic_index)
    } else {
        Vec::new()
    };
    let variadic_ref = if wrapper_sig.ref_params.len() > variadic_index {
        wrapper_sig.ref_params.remove(variadic_index)
    } else {
        false
    };
    let variadic_declared = if wrapper_sig.declared_params.len() > variadic_index {
        wrapper_sig.declared_params.remove(variadic_index)
    } else {
        false
    };

    wrapper_sig.params.push((
        variadic_name.clone(),
        PhpType::Array(Box::new(PhpType::Mixed)),
    ));
    // Matches `variadic()`'s and `ensure_variadic_for_func_args`'s convention: the
    // variadic slot's own `defaults` entry is `Some(<empty array literal>)`, not `None`.
    // Call-argument-count validation (`call_validation::check_known_callable_call_with_options`)
    // counts `None` defaults as "required parameters" — a bare `None` here falsely demanded
    // an argument for the variadic tail itself, rejecting zero-arg calls to variadic methods
    // (this path is exercised unconditionally by `build_method_sig` for every class method).
    wrapper_sig
        .defaults
        .push(Some(Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy())));
    wrapper_sig.ref_params.push(variadic_ref);
    wrapper_sig.declared_params.push(variadic_declared);
    wrapper_sig.param_type_exprs.push(variadic_type_expr);
    wrapper_sig.param_attributes.push(variadic_attributes);
    wrapper_sig
}

/// Looks up a builtin function's canonical call signature.
///
/// Consults the builtin registry first; falls back to the legacy static match
/// table when the name is not registered. Returns `Some(FunctionSig)` for known
/// PHP builtins (e.g., `strlen`, `array_push`) and `None` for untracked or
/// user-defined functions. The returned signature reflects PHP's actual parameter
/// ordering, defaults, variadics, and by-ref params.
///
/// Called from:
/// - type checker builtin validation
/// - first-class callable builtin sig construction
/// - optimizer effect modeling for builtins
pub(crate) fn builtin_call_sig(name: &str) -> Option<FunctionSig> {
    crate::builtins::registry::function_sig(name)
        .or_else(|| legacy_builtin_call_sig(name))
}

/// Legacy static lookup table for builtin call signatures.
///
/// Returns `Some(FunctionSig)` for builtins whose signatures are hand-coded
/// in this match table. `builtin_call_sig` delegates here when the builtin
/// registry does not have an entry for `name`.
///
/// Entries for builtins that have been migrated into `src/builtins/` are kept
/// here so that the parity gate in `src/builtins/parity_tests.rs` can compare
/// the registry-derived signature against a stable golden. Remove an entry only
/// after the parity gate has verified the registry matches and the golden is no
/// longer needed.
pub(crate) fn legacy_builtin_call_sig(name: &str) -> Option<FunctionSig> {
    match name {
        "eval" => Some(fixed(&["code"])),

        "time" | "json_last_error" | "json_last_error_msg" | "pi"
        | "ptr_null" | "getcwd" | "sys_get_temp_dir" | "tmpfile" | "hash_algos"
        | "date_default_timezone_get" => Some(fixed(&[])),
        // phpversion(?string $extension = null): string|false. With no argument PHP
        // returns the runtime version string; with an extension name it returns that
        // extension's version or false. elephc has no loadable extensions, so the
        // one-argument form is always the concrete false.
        "phpversion" => Some(optional(&["extension"], 0, vec![null_lit()])),

        "strlen" | "strtolower" | "strtoupper" | "ucfirst" | "lcfirst" | "strrev"
        | "grapheme_strrev" | "addslashes" | "stripslashes" | "nl2br" | "bin2hex"
        | "hex2bin" | "urlencode" | "urldecode" | "rawurlencode" | "rawurldecode"
        | "base64_encode" => Some(fixed(&["string"])),
        // htmlspecialchars/htmlentities(string $string, int $flags = ENT_QUOTES|ENT_SUBSTITUTE,
        // ?string $encoding = null, bool $double_encode = true): string. The default
        // flags value 11 = ENT_QUOTES|ENT_SUBSTITUTE; ENT_SUBSTITUTE is not registered
        // as a named constant here, so the literal 11 is used directly.
        "htmlspecialchars" | "htmlentities" => Some(optional(
            &["string", "flags", "encoding", "double_encode"],
            1,
            vec![int_lit(11), null_lit(), bool_lit(true)],
        )),
        // html_entity_decode(string $string, int $flags = ENT_QUOTES|ENT_SUBSTITUTE,
        // ?string $encoding = null): string.
        "html_entity_decode" => Some(optional(
            &["string", "flags", "encoding"],
            1,
            vec![int_lit(11), null_lit()],
        )),
        "base64_decode" => Some(optional(&["string", "strict"], 1, vec![bool_lit(false)])),
        // serialize(mixed $value): string and unserialize(string $data, array $options = []): mixed.
        // The signatures must match PHP so named arguments and the optional `$options` default
        // resolve correctly. The runtime lowering is a deferred fatal stub (see the EIR backend),
        // but the call surface is fully recognized so call sites type-check and resolve.
        "serialize" => Some(fixed(&["value"])),
        "unserialize" => Some(optional(
            &["data", "options"],
            1,
            vec![Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy())],
        )),
        "gzcompress" => Some(optional(&["data", "level"], 1, vec![int_lit(-1)])),
        "gzdeflate" => Some(optional(&["data", "level"], 1, vec![int_lit(-1)])),
        "gzinflate" => Some(optional(&["data", "max_length"], 1, vec![int_lit(0)])),
        "gzuncompress" => Some(optional(&["data", "max_length"], 1, vec![int_lit(0)])),
        "octdec" => Some(fixed(&["octal_string"])),
        "bindec" => Some(fixed(&["binary_string"])),
        "dechex" => Some(fixed(&["value"])),
        "decoct" => Some(fixed(&["value"])),
        "decbin" => Some(fixed(&["value"])),
        "preg_last_error_msg" => Some(fixed(&[])),
        "preg_last_error" => Some(fixed(&[])),
        // Migrated to src/builtins/string/ord.rs — kept as the parity-gate golden.
        "ord" => Some(fixed(&["character"])),
        "chr" => Some(fixed(&["codepoint"])),
        "substr_count" => Some(optional(
            &["haystack", "needle", "offset", "length"],
            2,
            vec![int_lit(0), null_lit()],
        )),

        "ctype_alpha" | "ctype_digit" | "ctype_alnum" | "ctype_space" | "ctype_upper" => {
            Some(fixed(&["text"]))
        }

        "floatval" | "boolval" | "strval" | "gettype" | "get_debug_type" | "is_bool"
        | "is_null" | "is_float" | "is_double" | "is_real" | "is_int" | "is_integer"
        | "is_long" | "is_iterable" | "is_string" | "is_numeric"
        | "is_array" | "is_object" | "is_scalar"
        | "empty" => {
            Some(fixed(&["value"]))
        }
        "intval" => Some(optional(&["value", "base"], 1, vec![int_lit(10)])),
        "var_dump" => Some(variadic(&["value"], "values")),
        "print_r" => Some(optional(
            &["value", "return"],
            1,
            vec![bool_lit(false)],
        )),
        "isset" => Some(variadic(&["var"], "vars")),
        "unset" => Some(variadic(&["var"], "vars")),
        "settype" => {
            let mut sig = fixed(&["var", "type"]);
            sig.ref_params[0] = true;
            Some(sig)
        }
        "function_exists" => Some(fixed(&["function"])),
        "func_num_args" => Some(FunctionSig {
            params: Vec::new(),
            defaults: Vec::new(),
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: Vec::new(),
            declared_params: Vec::new(),
            variadic: None,
            deprecation: None,
            param_attributes: Vec::new(),
            param_type_exprs: Vec::new(),
        }),
        "func_get_args" => Some(FunctionSig {
            params: Vec::new(),
            defaults: Vec::new(),
            return_type: PhpType::Array(Box::new(PhpType::Mixed)),
            declared_return: true,
            by_ref_return: false,
            ref_params: Vec::new(),
            declared_params: Vec::new(),
            variadic: None,
            deprecation: None,
            param_attributes: Vec::new(),
            param_type_exprs: Vec::new(),
        }),
        "func_get_arg" => Some(FunctionSig {
            params: vec![("position".to_string(), PhpType::Int)],
            defaults: vec![None],
            return_type: PhpType::Mixed,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false],
            declared_params: vec![true],
            variadic: None,
            deprecation: None,
            param_attributes: Vec::new(),
            param_type_exprs: Vec::new(),
        }),
        "extension_loaded" => Some(fixed(&["extension"])),
        "is_callable" => Some(fixed(&["value"])),
        "defined" => Some(fixed(&["constant_name"])),
        "constant" => Some(fixed(&["name"])),
        "class_alias" => Some(optional(
            &["class", "alias", "autoload"],
            2,
            vec![bool_lit(true)],
        )),
        "class_exists" => Some(optional(&["class", "autoload"], 1, vec![bool_lit(true)])),
        "interface_exists" => Some(optional(
            &["interface", "autoload"],
            1,
            vec![bool_lit(true)],
        )),
        "trait_exists" => Some(optional(&["trait", "autoload"], 1, vec![bool_lit(true)])),
        "enum_exists" => Some(optional(&["enum", "autoload"], 1, vec![bool_lit(true)])),
        "class_implements" | "class_parents" | "class_uses" => Some(optional(
            &["object_or_class", "autoload"],
            1,
            vec![bool_lit(true)],
        )),
        "method_exists" => Some(with_return(
            fixed(&["object_or_class", "method"]),
            PhpType::Bool,
        )),
        "property_exists" => Some(with_return(
            fixed(&["object_or_class", "property"]),
            PhpType::Bool,
        )),
        "iterator_to_array" => Some(optional(
            &["iterator", "preserve_keys"],
            1,
            vec![bool_lit(true)],
        )),
        "iterator_count" => Some(fixed(&["iterator"])),
        "iterator_apply" => Some(optional(
            &["iterator", "callback", "args"],
            2,
            vec![null_lit()],
        )),
        "get_class" => Some(optional(&["object"], 0, vec![null_lit()])),
        "get_parent_class" => Some(optional(&["object_or_class"], 0, vec![null_lit()])),
        "get_declared_classes" | "get_declared_interfaces" | "get_declared_traits"
        | "get_defined_functions" => Some(fixed(&[])),
        "is_a" => Some(optional(
            &["object_or_class", "class", "allow_string"],
            2,
            vec![bool_lit(false)],
        )),
        "is_subclass_of" => Some(optional(
            &["object_or_class", "class", "allow_string"],
            2,
            vec![bool_lit(true)],
        )),
        "is_resource" => Some(fixed(&["value"])),
        // is_countable(mixed $value): bool — true for arrays and Countable objects.
        "is_countable" => Some(fixed(&["value"])),
        "get_resource_type" | "get_resource_id" => Some(fixed(&["resource"])),
        "class_attribute_names" | "class_get_attributes" => Some(fixed(&["class_name"])),
        "class_attribute_args" => Some(fixed(&["class_name", "attribute_name"])),

        "is_nan" | "is_finite" | "is_infinite" | "abs" | "floor" | "ceil" | "sqrt"
        | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sinh" | "cosh"
        | "tanh" | "log2" | "log10" | "exp" | "deg2rad" | "rad2deg" => {
            Some(fixed(&["num"]))
        }

        "count" => Some(optional(&["value", "mode"], 1, vec![int_lit(0)])),
        "microtime" => Some(optional(&["as_float"], 0, vec![bool_lit(false)])),
        "php_uname" => Some(optional(&["mode"], 0, vec![string_lit("a")])),
        "readline" => Some(optional(&["prompt"], 0, vec![null_lit()])),
        "umask" => Some(optional(&["mask"], 0, vec![null_lit()])),
        "exit" | "die" => Some(optional(&["status"], 0, vec![int_lit(0)])),

        "trim" | "ltrim" | "rtrim" | "chop" => Some(optional(
            &["string", "characters"],
            1,
            vec![string_lit(" \n\r\t\u{0b}\u{0c}\0")],
        )),
        "ucwords" => Some(optional(
            &["string", "separators"],
            1,
            vec![string_lit(" \t\r\n\u{0c}\u{0b}")],
        )),
        // Migrated to src/builtins/string/substr.rs — kept as the parity-gate golden.
        "substr" => Some(optional(&["string", "offset", "length"], 2, vec![null_lit()])),
        "strpos" => Some(optional(
            &["haystack", "needle", "offset"],
            2,
            vec![int_lit(0)],
        )),
        "strrpos" => Some(optional(&["haystack", "needle", "offset"], 2, vec![int_lit(0)])),
        "strstr" => Some(optional(
            &["haystack", "needle", "before_needle"],
            2,
            vec![bool_lit(false)],
        )),
        "str_repeat" => Some(fixed(&["string", "times"])),
        "strcmp" | "strcasecmp" => Some(fixed(&["string1", "string2"])),
        // strval(mixed $value): string — coerces its single argument to a string.
        // strnatcmp/strnatcasecmp(string $string1, string $string2): int — the
        // natural-order comparison siblings of strcmp/strcasecmp.
        "strnatcmp" | "strnatcasecmp" => Some(fixed(&["string1", "string2"])),
        // strrchr(string $haystack, string $needle): string|false — returns the
        // suffix of $haystack from the last occurrence of $needle's first byte.
        "strrchr" => Some(fixed(&["haystack", "needle"])),
        // addcslashes(string $string, string $characters): string — C-style escaping
        // of every byte listed in $characters (which may use `a..z` ranges).
        "addcslashes" => Some(fixed(&["string", "characters"])),
        // stripcslashes(string $string): string — the inverse of addcslashes.
        "stripcslashes" => Some(fixed(&["string"])),
        // str_getcsv(string $string, string $separator = ",", string $enclosure = "\"",
        // string $escape = "\\"): array — recognized for namespace fallback; runtime
        // lowering is deferred (loud unsupported error) until CSV parsing lands.
        "str_getcsv" => Some(optional(
            &["string", "separator", "enclosure", "escape"],
            1,
            vec![string_lit(","), string_lit("\""), string_lit("\\")],
        )),
        // pack(string $format, mixed ...$values): string — recognized for namespace
        // fallback; runtime lowering is deferred (loud unsupported error).
        "pack" => Some(variadic(&["format"], "values")),
        "strcspn" | "strspn" => Some(optional(
            &["string", "characters", "offset", "length"],
            2,
            vec![int_lit(0), null_lit()],
        )),
        "strpbrk" => Some(fixed(&["string", "characters"])),
        // stripos/strripos: case-insensitive strpos/strrpos, returning int|false
        // (PHP: (string $haystack, string $needle, int $offset = 0)).
        "stripos" | "strripos" => Some(optional(
            &["haystack", "needle", "offset"],
            2,
            vec![int_lit(0)],
        )),
        // strncmp/strncasecmp compare the first $length bytes of two strings.
        "strncmp" | "strncasecmp" => Some(fixed(&["string1", "string2", "length"])),
        // substr_compare(string $haystack, string $needle, int $offset,
        // ?int $length = null, bool $case_insensitive = false): int.
        "substr_compare" => Some(optional(
            &["haystack", "needle", "offset", "length", "case_insensitive"],
            3,
            vec![null_lit(), bool_lit(false)],
        )),
        // strtr(string $string, array|string $from, ?string $to = null): string —
        // covers both the 2-arg map form and the 3-arg char-translation form.
        "strtr" => Some(optional(&["string", "from", "to"], 2, vec![null_lit()])),
        // strip_tags(string $string, array|string|null $allowed_tags = null): string.
        "strip_tags" => Some(optional(&["string", "allowed_tags"], 1, vec![null_lit()])),
        // levenshtein(string $string1, string $string2, int $insertion_cost = 1,
        // int $replacement_cost = 1, int $deletion_cost = 1): int.
        "levenshtein" => Some(optional(
            &["string1", "string2", "insertion_cost", "replacement_cost", "deletion_cost"],
            2,
            vec![int_lit(1), int_lit(1), int_lit(1)],
        )),
        // parse_str(string $string, array &$result): void — the parsed key/value
        // pairs are written back through the by-reference second parameter.
        "parse_str" => {
            let mut sig = fixed(&["string", "result"]);
            sig.ref_params[1] = true;
            Some(sig)
        }
        // parse_url(string $url, int $component = -1): array|string|int|false|null.
        "parse_url" => Some(optional(&["url", "component"], 1, vec![int_lit(-1)])),
        // -- multibyte string (mbstring) builtins (recognition-only; runtime deferred) --
        // mb_encode_numericentity(string $string, array $map, ?string $encoding = null,
        // bool $hex = false): string — recognized for namespace fallback; runtime
        // lowering is deferred (loud unsupported error).
        "mb_encode_numericentity" => Some(optional(
            &["string", "map", "encoding", "hex"],
            2,
            vec![null_lit(), bool_lit(false)],
        )),
        // mb_substr(string $string, int $start, ?int $length = null,
        // ?string $encoding = null): string.
        "mb_substr" => Some(optional(
            &["string", "start", "length", "encoding"],
            2,
            vec![null_lit(), null_lit()],
        )),
        // mb_strlen(string $string, ?string $encoding = null): int.
        "mb_strlen" => Some(optional(&["string", "encoding"], 1, vec![null_lit()])),
        // mb_stripos/mb_strripos(string $haystack, string $needle, int $offset = 0,
        // ?string $encoding = null): int|false.
        "mb_stripos" | "mb_strripos" => Some(optional(
            &["haystack", "needle", "offset", "encoding"],
            2,
            vec![int_lit(0), null_lit()],
        )),
        // mb_strpos/mb_strrpos(string $haystack, string $needle, int $offset = 0,
        // ?string $encoding = null): int|false.
        "mb_strpos" | "mb_strrpos" => Some(optional(
            &["haystack", "needle", "offset", "encoding"],
            2,
            vec![int_lit(0), null_lit()],
        )),
        // mb_strstr/mb_stristr(string $haystack, string $needle,
        // bool $before_needle = false, ?string $encoding = null): string|false.
        "mb_strstr" | "mb_stristr" => Some(optional(
            &["haystack", "needle", "before_needle", "encoding"],
            2,
            vec![bool_lit(false), null_lit()],
        )),
        // mb_strtolower/mb_strtoupper(string $string, ?string $encoding = null): string.
        "mb_strtolower" | "mb_strtoupper" => {
            Some(optional(&["string", "encoding"], 1, vec![null_lit()]))
        }
        // mb_convert_case(string $string, int $mode, ?string $encoding = null): string.
        "mb_convert_case" => Some(optional(
            &["string", "mode", "encoding"],
            2,
            vec![null_lit()],
        )),
        // `mb_convert_encoding` is intentionally absent: it ships as an elephc-PHP prelude
        // (`crate::mb_convert_encoding_prelude`), so a named-argument/default entry here would make
        // the checker treat the prelude's own declaration as redeclaring a builtin.
        // mb_detect_encoding(string $string, array|string|null $encodings = null,
        // bool $strict = false): string|false.
        "mb_detect_encoding" => Some(optional(
            &["string", "encodings", "strict"],
            1,
            vec![null_lit(), bool_lit(false)],
        )),
        // mb_internal_encoding(?string $encoding = null): string|bool.
        "mb_internal_encoding" => {
            Some(optional(&["encoding"], 0, vec![null_lit()]))
        }
        // mb_ord(string $string, ?string $encoding = null): int|false.
        "mb_ord" => Some(optional(&["string", "encoding"], 1, vec![null_lit()])),
        // mb_str_split(string $string, int $length = 1, ?string $encoding = null): array.
        "mb_str_split" => Some(optional(
            &["string", "length", "encoding"],
            1,
            vec![int_lit(1), null_lit()],
        )),
        // mb_strwidth(string $string, ?string $encoding = null): int.
        "mb_strwidth" => Some(optional(&["string", "encoding"], 1, vec![null_lit()])),
        // -- grapheme (intl) builtins (recognition-only; runtime deferred) --
        // grapheme_strlen(string $string): int|false|null.
        "grapheme_strlen" => Some(fixed(&["string"])),
        // grapheme_substr(string $string, int $offset, ?int $length = null): string|false.
        "grapheme_substr" => Some(optional(&["string", "offset", "length"], 2, vec![null_lit()])),
        // grapheme_stripos/grapheme_strripos(string $haystack, string $needle,
        // int $offset = 0): int|false.
        "grapheme_stripos" | "grapheme_strripos" => Some(optional(
            &["haystack", "needle", "offset"],
            2,
            vec![int_lit(0)],
        )),
        // grapheme_strpos/grapheme_strrpos(string $haystack, string $needle,
        // int $offset = 0): int|false — the case-sensitive grapheme search pair.
        "grapheme_strpos" | "grapheme_strrpos" => Some(optional(
            &["haystack", "needle", "offset"],
            2,
            vec![int_lit(0)],
        )),
        // grapheme_str_split(string $string, int $length = 1): array|false.
        "grapheme_str_split" => Some(optional(&["string", "length"], 1, vec![int_lit(1)])),
        // grapheme_extract(string $haystack, int $size, int $type = GRAPHEME_EXTR_COUNT (0),
        // int $offset = 0, int &$next = null): string|false. The trailing $next is by-reference —
        // grapheme_extract writes the next cursor position back into it.
        "grapheme_extract" => {
            let mut sig = optional(
                &["haystack", "size", "type", "offset", "next"],
                2,
                vec![int_lit(0), int_lit(0), null_lit()],
            );
            sig.ref_params[4] = true;
            Some(sig)
        }
        // -- normalizer (intl) builtins (recognition-only; runtime deferred) --
        // normalizer_normalize(string $string, int $form = 16 (Normalizer::FORM_C)): string|false.
        "normalizer_normalize" => Some(optional(&["string", "form"], 1, vec![int_lit(16)])),
        // normalizer_is_normalized(string $string, int $form = 16 (Normalizer::FORM_C)): bool.
        "normalizer_is_normalized" => Some(optional(&["string", "form"], 1, vec![int_lit(16)])),
        // -- iconv builtins (recognition-only; runtime deferred) --
        // iconv(string $from_encoding, string $to_encoding, string $string): string|false.
        "iconv" => Some(fixed(&["from_encoding", "to_encoding", "string"])),
        // iconv_strlen(string $string, ?string $encoding = null): int|false.
        "iconv_strlen" => Some(optional(&["string", "encoding"], 1, vec![null_lit()])),
        // iconv_strpos(string $haystack, string $needle, int $offset = 0,
        // ?string $encoding = null): int|false.
        "iconv_strpos" => Some(optional(
            &["haystack", "needle", "offset", "encoding"],
            2,
            vec![int_lit(0), null_lit()],
        )),
        // iconv_strrpos(string $haystack, string $needle, ?string $encoding = null): int|false.
        // Note: iconv_strrpos has no $offset parameter (unlike iconv_strpos).
        "iconv_strrpos" => Some(optional(&["haystack", "needle", "encoding"], 2, vec![null_lit()])),
        // iconv_substr(string $string, int $offset, ?int $length = null,
        // ?string $encoding = null): string|false.
        "iconv_substr" => Some(optional(
            &["string", "offset", "length", "encoding"],
            2,
            vec![null_lit(), null_lit()],
        )),
        // iconv_mime_decode(string $string, int $mode = 0, ?string $encoding = null): string|false.
        "iconv_mime_decode" => Some(optional(
            &["string", "mode", "encoding"],
            1,
            vec![int_lit(0), null_lit()],
        )),
        "hexdec" => Some(fixed(&["hex_string"])),
        "str_contains" | "str_starts_with" | "str_ends_with" => {
            Some(fixed(&["haystack", "needle"]))
        }
        "hash_equals" => Some(fixed(&["known_string", "user_string"])),
        "str_replace" | "str_ireplace" => Some(optional(
            &["search", "replace", "subject", "count"],
            3,
            vec![null_lit()],
        )),
        "explode" => Some(optional(
            &["separator", "string", "limit"],
            2,
            vec![int_lit(i64::MAX)],
        )),
        "implode" => Some(optional(&["separator", "array"], 1, vec![null_lit()])),
        "substr_replace" => Some(optional(
            &["string", "replace", "offset", "length"],
            3,
            vec![null_lit()],
        )),
        "str_pad" => Some(optional(
            &["string", "length", "pad_string", "pad_type"],
            2,
            vec![string_lit(" "), int_lit(1)],
        )),
        "str_split" => Some(optional(&["string", "length"], 1, vec![int_lit(1)])),
        "wordwrap" => Some(optional(
            &["string", "width", "break", "cut_long_words"],
            1,
            vec![int_lit(75), string_lit("\n"), bool_lit(false)],
        )),
        "sprintf" | "printf" => Some(variadic(&["format"], "values")),
        "fprintf" => Some(variadic(&["stream", "format"], "values")),
        "vsprintf" | "vprintf" => Some(fixed(&["format", "values"])),
        "vfprintf" => Some(fixed(&["stream", "format", "values"])),
        "sscanf" => Some(variadic(&["string", "format"], "vars")),
        "fscanf" => Some(variadic(&["stream", "format"], "vars")),
        "hash" => Some(optional(&["algo", "data", "binary"], 2, vec![bool_lit(false)])),
        "hash_hmac" => Some(optional(
            &["algo", "data", "key", "binary"],
            3,
            vec![bool_lit(false)],
        )),
        "hash_file" => Some(optional(&["algo", "filename", "binary"], 2, vec![bool_lit(false)])),
        "hash_init" => Some(optional(&["algo", "flags", "key"], 1, vec![int_lit(0), string_lit("")])),
        "hash_update" => Some(fixed(&["context", "data"])),
        "hash_final" => Some(optional(&["context", "binary"], 1, vec![bool_lit(false)])),
        "hash_copy" => Some(fixed(&["context"])),
        "md5" | "sha1" => Some(optional(&["string", "binary"], 1, vec![bool_lit(false)])),
        "crc32" => Some(fixed(&["string"])),
        "number_format" => Some(optional(
            &["num", "decimals", "decimal_separator", "thousands_separator"],
            1,
            vec![int_lit(0), string_lit("."), string_lit(",")],
        )),

        "array_pop" | "array_shift" => Some(first_param_ref(fixed(&["array"]))),
        "array_keys" | "array_values" | "array_flip" | "array_sum" | "array_product"
        | "array_rand" | "array_is_list" | "array_key_first" | "array_key_last" => {
            Some(fixed(&["array"]))
        }
        // array_reverse(array $array, bool $preserve_keys = false): array.
        "array_reverse" => Some(optional(&["array", "preserve_keys"], 1, vec![bool_lit(false)])),
        // array_unique(array $array, int $flags = SORT_STRING): array. SORT_STRING = 2.
        "array_unique" => Some(optional(&["array", "flags"], 1, vec![int_lit(2)])),
        // sort family: shuffle/natsort/natcasesort take exactly 1 arg in PHP;
        // sort/rsort/asort/arsort/ksort/krsort accept an optional $flags (SORT_REGULAR=0).
        "shuffle" | "natsort" | "natcasesort" => Some(first_param_ref(fixed(&["array"]))),
        "sort" | "rsort" | "asort" | "arsort" | "ksort" | "krsort" => {
            Some(first_param_ref(optional(&["array", "flags"], 1, vec![int_lit(0)])))
        }
        "in_array" => Some(optional(&["needle", "haystack", "strict"], 2, vec![bool_lit(false)])),
        "array_key_exists" => Some(fixed(&["key", "array"])),
        // array_key_first(array $array): string|int|null — reads the first key
        // without moving the internal pointer.
        // array_key_last(array $array): string|int|null — reads the last key
        // without moving the internal pointer.
        // `end(&$array)` takes the array by reference (PHP advances its internal
        // pointer); elephc only reads the last element, but the by-ref marker keeps
        // the original storage from being copied into a value temporary.
        "end" => Some(first_param_ref(fixed(&["array"]))),
        // reset(object|array &$array): mixed — moves the internal pointer to the
        // first element. By-ref like `end`, so the original storage is passed.
        "reset" => Some(first_param_ref(fixed(&["array"]))),
        // current(object|array $array)/key(object|array $array): read the value/key
        // at the array's internal pointer. Not by-ref in PHP 8.
        "current" | "key" => Some(fixed(&["array"])),
        // `setlocale(int $category, string|array $locales, string ...$rest)`. The
        // category and first locale are required; further locale fallbacks are
        // variadic.
        "setlocale" => Some(variadic(&["category", "locales"], "rest")),
        // Symfony's symfony/deprecation-contracts global:
        // `trigger_deprecation(string $package, string $version, string $message,
        // mixed ...$args): void`. package/version/message are required; the
        // printf-style arguments are variadic.
        "trigger_deprecation" => Some(variadic(&["package", "version", "message"], "args")),
        "array_search" => {
            Some(optional(&["needle", "haystack", "strict"], 2, vec![bool_lit(false)]))
        }
        "array_push" | "array_unshift" => Some(first_param_ref(variadic(&["array"], "values"))),
        "array_merge" | "array_merge_recursive" => Some(variadic(&[], "arrays")),
        // array_replace(array $array, array ...$replacements): array — later arrays
        // overwrite earlier entries by key (int keys are preserved, not renumbered).
        "array_replace" => Some(variadic(&["array"], "replacements")),
        // array_replace_recursive(array $array, array ...$replacements): array.
        "array_replace_recursive" => Some(variadic(&["array"], "replacements")),
        "array_diff" | "array_intersect" | "array_diff_key" | "array_intersect_key"
        | "array_diff_assoc" | "array_intersect_assoc" => {
            Some(variadic(&["array"], "arrays"))
        }
        "array_multisort" => {
            let mut sig = fixed(&["array1", "array2"]);
            sig.ref_params = vec![true, true];
            Some(sig)
        }
        "array_combine" => Some(fixed(&["keys", "values"])),
        "array_fill_keys" => Some(fixed(&["keys", "value"])),
        "array_pad" => Some(fixed(&["array", "length", "value"])),
        "array_fill" => Some(fixed(&["start_index", "count", "value"])),
        "array_slice" => Some(optional(
            &["array", "offset", "length", "preserve_keys"],
            2,
            vec![null_lit(), bool_lit(false)],
        )),
        "array_splice" => Some(first_param_ref(optional(
            &["array", "offset", "length", "replacement"],
            2,
            vec![
                null_lit(),
                Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy()),
            ],
        ))),
        "array_chunk" => Some(fixed(&["array", "length"])),
        "array_column" => Some(fixed(&["array", "column_key"])),
        "range" => Some(fixed(&["start", "end"])),
        "array_map" => Some(variadic(&["callback", "array"], "arrays")),
        "array_filter" => Some(optional(
            &["array", "callback", "mode"],
            1,
            vec![null_lit(), int_lit(0)],
        )),
        "array_reduce" => Some(optional(
            &["array", "callback", "initial"],
            2,
            vec![null_lit()],
        )),
        "array_walk" | "array_walk_recursive" | "usort" | "uksort" | "uasort" => {
            Some(first_param_ref(fixed(&["array", "callback"])))
        }
        // array_walk_recursive(object|array &$array, callable $callback,
        // mixed $arg = null): true — by-ref array like array_walk, plus recursion.
        "array_find" | "array_any" | "array_all" => Some(fixed(&["array", "callback"])),
        "array_udiff" | "array_uintersect" => {
            Some(fixed(&["array1", "array2", "callback"]))
        }
        "call_user_func" => Some(variadic(&["callback"], "args")),
        "call_user_func_array" => Some(fixed(&["callback", "args"])),

        "log" => Some(optional(
            &["num", "base"],
            1,
            vec![Expr::new(ExprKind::FloatLiteral(std::f64::consts::E), Span::dummy())],
        )),
        "atan2" => Some(fixed(&["y", "x"])),
        "hypot" => Some(fixed(&["x", "y"])),
        "pow" => Some(fixed(&["num", "exponent"])),
        "intdiv" | "fmod" | "fdiv" => Some(fixed(&["num1", "num2"])),
        "clamp" => Some(fixed(&["value", "min", "max"])),
        "min" | "max" => Some(variadic(&["value"], "values")),
        "rand" | "mt_rand" | "random_int" => Some(fixed(&["min", "max"])),
        "round" => Some(optional(&["num", "precision"], 1, vec![int_lit(0)])),

        "sleep" => Some(fixed(&["seconds"])),
        "usleep" => Some(fixed(&["microseconds"])),
        "http_response_code" => Some(optional(&["response_code"], 0, vec![int_lit(0)])),
        "header" => Some(optional(
            &["header", "replace", "response_code"],
            1,
            vec![bool_lit(true), int_lit(0)],
        )),
        "getenv" => Some(fixed(&["name"])),
        "putenv" => Some(fixed(&["assignment"])),
        // -- output buffering + header/env small builtins (K2) --
        // ob_start(): bool — deliberately ZERO parameters (not PHP's
        // `ob_start(?callable $callback = null, int $chunk_size = 0, int $flags = ...)`):
        // elephc supports only the plain, callback-free form, so passing a
        // callback/chunk_size is a PHP-faithful arity error, never accept-and-ignore.
        "ob_start" => Some(fixed(&[])),
        "ob_get_contents" | "ob_get_clean" | "ob_end_clean" | "ob_end_flush" | "ob_get_level" => {
            Some(fixed(&[]))
        }
        // ob_get_status(bool $full_status = false): array.
        "ob_get_status" => Some(optional(&["full_status"], 0, vec![bool_lit(false)])),
        // headers_sent(?string &$file = null, ?int &$line = null): bool.
        "headers_sent" => {
            let mut sig = optional(&["file", "line"], 0, vec![null_lit(), null_lit()]);
            sig.ref_params[0] = true;
            sig.ref_params[1] = true;
            Some(sig)
        }
        // flush(): void.
        "flush" => Some(fixed(&[])),
        // header_remove(?string $name = null): void.
        "header_remove" => Some(optional(&["name"], 0, vec![null_lit()])),
        // get_class_methods(object|string $object_or_class): array.
        "get_class_methods" => Some(fixed(&["object_or_class"])),
        // -- env/runtime state builtins (real AOT runtime; slice 1A) --
        // error_reporting(?int $error_level = null): int — null (or no arg) reads the
        // current level; a passed int sets it and returns the previous value.
        "error_reporting" => Some(optional(&["error_level"], 0, vec![null_lit()])),
        // ignore_user_abort(?bool $enable = null): int — null reads the current flag; a
        // passed bool sets it (0/1) and returns the previous value.
        "ignore_user_abort" => Some(optional(&["enable"], 0, vec![null_lit()])),
        // set_time_limit(int $seconds): bool — always true (a native binary has no timeout).
        "set_time_limit" => Some(fixed(&["seconds"])),
        // connection_aborted(): int — always 0 (a compiled program's connection is never aborted).
        "connection_aborted" => Some(fixed(&[])),
        // gc_* cycle-collector builtins (all 0-arg). gc_collect_cycles(): int runs the real
        // collector; gc_enabled(): bool queries the enabled flag; gc_enable()/gc_disable(): void
        // set it; gc_mem_caches(): int is always 0 (elephc has no request-scoped memory cache).
        "gc_collect_cycles" | "gc_disable" | "gc_enable" | "gc_enabled" | "gc_mem_caches" => {
            Some(fixed(&[]))
        }
        // error_log(string $message, int $message_type = 0, ?string $destination = null,
        // ?string $additional_headers = null): bool — writes $message to stderr, returns true.
        "error_log" => Some(optional(
            &["message", "message_type", "destination", "additional_headers"],
            1,
            vec![int_lit(0), null_lit(), null_lit()],
        )),
        // error_get_last(): ?array — null when no error has been recorded. elephc records no
        // runtime errors today (trigger_error() is not backend-implemented), so this always
        // returns null, which is PHP-identical for every program that triggers no error.
        "error_get_last" => Some(fixed(&[])),
        // libxml_use_internal_errors(?bool $use_errors = null): bool — null (or no arg) reads
        // the current flag; a passed bool sets it and returns the previous value.
        "libxml_use_internal_errors" => Some(optional(&["use_errors"], 0, vec![null_lit()])),
        // libxml_clear_errors(): void — no-op (elephc never records a libxml parse error).
        "libxml_clear_errors" => Some(fixed(&[])),
        // libxml_get_errors(): array — always empty (elephc has no libxml/DOM subsystem, so no
        // libxml parse error can ever be recorded).
        "libxml_get_errors" => Some(fixed(&[])),
        // -- process / system control builtins (recognition-only; runtime deferred) --
        // pcntl_signal(int $signal, callable|int $handler,
        // bool $restart_syscalls = true): bool.
        "pcntl_signal" => Some(optional(
            &["signal", "handler", "restart_syscalls"],
            2,
            vec![bool_lit(true)],
        )),
        // pcntl_alarm(int $seconds): int.
        "pcntl_alarm" => Some(fixed(&["seconds"])),
        // pcntl_async_signals(?bool $enable = null): bool.
        "pcntl_async_signals" => Some(optional(&["enable"], 0, vec![null_lit()])),
        // pcntl_signal_get_handler(int $signal): int|string.
        "pcntl_signal_get_handler" => Some(fixed(&["signal"])),
        // proc_close($process): int — $process is a resource returned by proc_open().
        "proc_close" => Some(fixed(&["process"])),
        // getmypid(): int|false.
        "getmypid" => Some(fixed(&[])),
        // cli_set_process_title(string $title): bool.
        "cli_set_process_title" => Some(fixed(&["title"])),
        // setproctitle(string $title): bool — the ext/proctitle alias of the above.
        "setproctitle" => Some(fixed(&["title"])),
        // sapi_windows_cp_get(string $kind = ''): int (Windows-only; recognized everywhere).
        "sapi_windows_cp_get" => Some(optional(&["kind"], 0, vec![string_lit("")])),
        // sapi_windows_cp_set(int $code_page): bool.
        "sapi_windows_cp_set" => Some(fixed(&["code_page"])),
        // sapi_windows_vt100_support($stream, ?bool $enable = null): bool.
        "sapi_windows_vt100_support" => {
            Some(optional(&["stream", "enable"], 1, vec![null_lit()]))
        }
        // ini_get(string $option): string|false.
        "ini_get" => Some(fixed(&["option"])),
        // ini_set(string $option, string|int|float|bool|null $value): string|false.
        "ini_set" => Some(fixed(&["option", "value"])),
        // get_cfg_var(string $option): array|string|false (compiled master value).
        "get_cfg_var" => Some(fixed(&["option"])),
        // get_defined_constants(bool $categorize = false): array.
        "get_defined_constants" => Some(optional(&["categorize"], 0, vec![bool_lit(false)])),
        // -- misc / error-handling / process builtins (recognition-only; runtime deferred) --
        // method_exists(object|string $object_or_class, string $method): bool.
        // trigger_error(string $message, int $error_level = E_USER_NOTICE (1024)): bool.
        "trigger_error" => Some(optional(&["message", "error_level"], 1, vec![int_lit(1024)])),
        // set_error_handler(?callable $callback, int $error_levels = E_ALL (30719)): ?callable.
        "set_error_handler" => {
            Some(optional(&["callback", "error_levels"], 1, vec![int_lit(30719)]))
        }
        // restore_error_handler(): true.
        "restore_error_handler" => Some(fixed(&[])),
        // restore_exception_handler(): true — pops the previously installed exception handler.
        "restore_exception_handler" => Some(fixed(&[])),
        // filter_var(mixed $value, int $filter = FILTER_DEFAULT (516),
        // array|int $options = 0): mixed — the filtered value, or false on failure.
        // 516 php-verified with `php -n -r 'var_dump(FILTER_DEFAULT);'` (PHP 8.5.6
        // local); the previous 518 here was the same bug fixed in filter_constants.rs.
        "filter_var" => Some(optional(
            &["value", "filter", "options"],
            1,
            vec![int_lit(516), int_lit(0)],
        )),
        // set_exception_handler(?callable $callback): ?callable.
        "set_exception_handler" => Some(fixed(&["callback"])),
        // preg_quote(string $str, ?string $delimiter = null): string.
        "preg_quote" => Some(optional(&["str", "delimiter"], 1, vec![null_lit()])),
        // preg_grep(string $pattern, array $array, int $flags = 0): array|false.
        "preg_grep" => Some(optional(&["pattern", "array", "flags"], 2, vec![int_lit(0)])),
        // version_compare(string $version1, string $version2,
        // ?string $operator = null): int|bool.
        "version_compare" => Some(optional(
            &["version1", "version2", "operator"],
            2,
            vec![null_lit()],
        )),
        // unpack(string $format, string $string, int $offset = 0): array|false.
        "unpack" => Some(optional(&["format", "string", "offset"], 2, vec![int_lit(0)])),
        // random_bytes(int $length): string.
        "random_bytes" => Some(fixed(&["length"])),
        // http_build_query(array|object $data, string $numeric_prefix = '',
        // ?string $arg_separator = null, int $encoding_type = PHP_QUERY_RFC1738 (1)): string.
        "http_build_query" => Some(optional(
            &["data", "numeric_prefix", "arg_separator", "encoding_type"],
            1,
            vec![string_lit(""), null_lit(), int_lit(1)],
        )),
        // escapeshellarg(string $arg): string.
        "escapeshellarg" => Some(fixed(&["arg"])),
        // assert(mixed $assertion, Throwable|string|null $description = null): bool.
        "assert" => Some(optional(&["assertion", "description"], 1, vec![null_lit()])),
        // sapi_windows_cp_conv(int|string $in_codepage, int|string $out_codepage,
        // string $subject): ?string (Windows-only; recognized everywhere).
        "sapi_windows_cp_conv" => Some(fixed(&["in_codepage", "out_codepage", "subject"])),
        // posix_kill(int $process_id, int $signal): bool.
        "posix_kill" => Some(fixed(&["process_id", "signal"])),
        "exec" | "shell_exec" | "system" | "passthru" => Some(fixed(&["command"])),
        "define" => Some(fixed(&["constant_name", "value"])),
        "date" | "gmdate" => Some(optional(&["format", "timestamp"], 1, vec![null_lit()])),
        "date_default_timezone_set" => Some(fixed(&["timezoneId"])),
        "checkdate" => Some(fixed(&["month", "day", "year"])),
        "getdate" => Some(optional(&["timestamp"], 0, vec![null_lit()])),
        "localtime" => {
            Some(optional(&["timestamp", "associative"], 0, vec![int_lit(-1), bool_lit(false)]))
        }
        "hrtime" => Some(optional(&["as_number"], 0, vec![bool_lit(false)])),
        "mktime" | "gmmktime" | "__elephc_mktime_raw" | "__elephc_gmmktime_raw" => {
            Some(fixed(&["hour", "minute", "second", "month", "day", "year"]))
        }
        "strtotime" | "__elephc_strtotime_raw" => Some(optional(&["datetime", "baseTimestamp"], 1, vec![null_lit()])),
        "json_encode" => Some(optional(
            &["value", "flags", "depth"],
            1,
            vec![int_lit(0), int_lit(512)],
        )),
        "json_decode" => Some(optional(
            &["json", "associative", "depth", "flags"],
            1,
            vec![null_lit(), int_lit(512), int_lit(0)],
        )),
        "json_validate" => Some(optional(
            &["json", "depth", "flags"],
            1,
            vec![int_lit(512), int_lit(0)],
        )),
        "preg_match" => {
            let mut sig = optional(
                &["pattern", "subject", "matches", "flags", "offset"],
                2,
                vec![
                    Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy()),
                    int_lit(0),
                    int_lit(0),
                ],
            );
            sig.ref_params[2] = true;
            Some(sig)
        }
        "preg_match_all" => {
            // PHP: preg_match_all(string $pattern, string $subject, array &$matches = null,
            //   int $flags = 0, int $offset = 0): int|false. `$matches` is a by-ref
            //   out-parameter that PHP auto-vivifies, so it must be skipped during eager
            //   argument inference and defined in the caller scope after the call.
            let mut sig = optional(
                &["pattern", "subject", "matches", "flags", "offset"],
                2,
                vec![
                    Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy()),
                    int_lit(0),
                    int_lit(0),
                ],
            );
            sig.ref_params[2] = true;
            Some(sig)
        }
        "proc_open" => {
            // PHP: proc_open($cmd, $descriptor_spec, array &$pipes, $cwd = null,
            //   $env = null, $options = []): resource|false. `$pipes` is a by-ref
            //   out-parameter auto-vivified by PHP, so the caller's variable becomes
            //   defined after the call even when previously unset.
            let mut sig = optional(
                &["command", "descriptorspec", "pipes", "cwd", "env", "options"],
                3,
                vec![
                    null_lit(),
                    null_lit(),
                    Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy()),
                ],
            );
            sig.ref_params[2] = true;
            Some(sig)
        }
        "preg_replace_callback" => {
            let mut sig = optional(
                &["pattern", "callback", "subject", "limit", "count", "flags"],
                3,
                vec![int_lit(-1), null_lit(), int_lit(0)],
            );
            // $count is the by-ref out-param mirroring preg_replace's by-ref $count.
            sig.ref_params[4] = true;
            Some(sig)
        }
        "preg_replace" => {
            let mut sig = optional(
                &["pattern", "replacement", "subject", "limit", "count"],
                3,
                vec![int_lit(-1), null_lit()],
            );
            sig.ref_params[4] = true;
            Some(sig)
        }
        "preg_split" => Some(optional(
            &["pattern", "subject", "limit", "flags"],
            2,
            vec![int_lit(-1), int_lit(0)],
        )),

        "file_get_contents" | "file" | "file_exists" | "is_file" | "is_dir"
        | "is_readable" | "is_writable" | "is_writeable" | "is_executable"
        | "is_link" | "filesize" | "filemtime" | "fileatime" | "filectime"
        | "fileperms" | "fileowner" | "filegroup" | "fileinode" | "filetype"
        | "stat" | "lstat" => Some(fixed(&["filename"])),
        "disk_free_space" | "disk_total_space" => Some(fixed(&["directory"])),
        "file_put_contents" => Some(optional(
            &["filename", "data", "flags", "context"],
            2,
            vec![int_lit(0), null_lit()],
        )),
        "__elephc_phar_list_entries" => Some(fixed(&["filename"])),
        "__elephc_phar_set_compression" => Some(fixed(&["filename", "compression"])),
        "__elephc_phar_get_metadata" => Some(fixed(&["filename"])),
        "__elephc_phar_get_stub" => Some(fixed(&["filename"])),
        "__elephc_phar_set_metadata" => Some(fixed(&["filename", "metadata"])),
        "__elephc_phar_set_stub" => Some(fixed(&["filename", "stub"])),
        "__elephc_phar_set_zip_password" => Some(fixed(&["password"])),
        "__elephc_phar_get_file_metadata" => Some(fixed(&["url"])),
        "__elephc_phar_set_file_metadata" => Some(fixed(&["url", "metadata"])),
        "__elephc_phar_gzip_archive" => Some(fixed(&["src"])),
        "__elephc_phar_bzip2_archive" => Some(fixed(&["src"])),
        "__elephc_phar_decompress_archive" => Some(fixed(&["src"])),
        "__elephc_phar_sign_openssl" => Some(fixed(&["path", "key"])),
        "__elephc_phar_sign_hash" => Some(fixed(&["path", "algo"])),
        "__elephc_phar_get_signature_hash" => Some(fixed(&["path"])),
        "__elephc_phar_get_signature_type" => Some(fixed(&["path"])),
        "copy" | "rename" => Some(fixed(&["from", "to"])),
        "unlink" => Some(fixed(&["filename"])),
        "rmdir" | "chdir" => Some(fixed(&["directory"])),
        "mkdir" => Some(optional(
            &["directory", "permissions", "recursive", "context"],
            1,
            vec![int_lit(0o777), bool_lit(false), null_lit()],
        )),
        "scandir" => Some(optional(
            &["directory", "sorting_order", "context"],
            1,
            vec![int_lit(0), null_lit()],
        )),
        "glob" => Some(optional(&["pattern", "flags"], 1, vec![int_lit(0)])),
        "tempnam" => Some(fixed(&["directory", "prefix"])),
        "chmod" => Some(fixed(&["filename", "permissions"])),
        "chown" => Some(fixed(&["filename", "user"])),
        "chgrp" => Some(fixed(&["filename", "group"])),
        "lchown" => Some(fixed(&["filename", "user"])),
        "lchgrp" => Some(fixed(&["filename", "group"])),
        "touch" => Some(optional(
            &["filename", "mtime", "atime"],
            1,
            vec![null_lit(), null_lit()],
        )),
        "basename" => Some(optional(&["path", "suffix"], 1, vec![string_lit("")])),
        "dirname" => Some(optional(&["path", "levels"], 1, vec![int_lit(1)])),
        "fnmatch" => Some(optional(&["pattern", "filename", "flags"], 2, vec![int_lit(0)])),
        "realpath" => Some(fixed(&["path"])),
        "realpath_cache_get" | "realpath_cache_size" => Some(fixed(&[])),
        "pathinfo" => Some(optional(&["path", "flags"], 1, vec![int_lit(15)])),
        "fopen" => Some(optional(
            &["filename", "mode", "use_include_path", "context"],
            2,
            vec![bool_lit(false), null_lit()],
        )),
        "fclose" | "fgets" | "fgetc" | "fpassthru" | "feof" | "ftell" | "rewind"
        | "fstat" | "fsync" | "fflush" | "fdatasync" => Some(fixed(&["stream"])),
        "flock" => {
            let mut sig = optional(&["stream", "operation", "would_block"], 2, vec![null_lit()]);
            sig.ref_params[2] = true;
            Some(sig)
        }
        "readfile" => Some(fixed(&["filename"])),
        "symlink" | "link" => Some(fixed(&["target", "link"])),
        "readlink" | "linkinfo" => Some(fixed(&["path"])),
        "fread" => Some(fixed(&["stream", "length"])),
        "fwrite" => Some(fixed(&["stream", "data"])),
        "fseek" => Some(optional(&["stream", "offset", "whence"], 2, vec![int_lit(0)])),
        "fgetcsv" => Some(optional(
            &["stream", "length", "separator"],
            1,
            vec![null_lit(), string_lit(",")],
        )),
        "fputcsv" => Some(optional(
            &["stream", "fields", "separator", "enclosure"],
            2,
            vec![string_lit(","), string_lit("\"")],
        )),
        "ftruncate" => Some(fixed(&["stream", "size"])),
        "clearstatcache" => Some(optional(
            &["clear_realpath_cache", "filename"],
            0,
            vec![bool_lit(false), string_lit("")],
        )),
        "stream_isatty" | "stream_is_local" | "stream_supports_lock" => {
            Some(fixed(&["stream"]))
        }
        "stream_get_contents" => Some(optional(
            &["stream", "length", "offset"],
            1,
            vec![null_lit(), int_lit(-1)],
        )),
        "stream_get_meta_data" => Some(fixed(&["stream"])),
        "stream_copy_to_stream" => Some(optional(
            &["from", "to", "length", "offset"],
            2,
            vec![null_lit(), int_lit(-1)],
        )),
        "stream_socket_server" => Some(fixed(&["address"])),
        "stream_socket_client" => {
            let mut sig = optional(
                &[
                    "address",
                    "error_code",
                    "error_message",
                    "timeout",
                    "flags",
                    "context",
                ],
                1,
                vec![null_lit(), null_lit(), null_lit(), int_lit(4), null_lit()],
            );
            sig.ref_params[1] = true;
            sig.ref_params[2] = true;
            Some(sig)
        }
        "stream_socket_accept" => {
            let mut sig = optional(
                &["socket", "timeout", "peer_name"],
                1,
                vec![null_lit(), null_lit()],
            );
            sig.ref_params[2] = true;
            Some(sig)
        }
        "fsockopen" | "pfsockopen" => {
            let mut sig = optional(
                &["hostname", "port", "error_code", "error_message", "timeout"],
                2,
                vec![null_lit(), null_lit(), null_lit()],
            );
            sig.ref_params[2] = true;
            sig.ref_params[3] = true;
            Some(sig)
        }
        "stream_wrapper_register" => Some(optional(
            &["protocol", "class", "flags"],
            2,
            vec![int_lit(0)],
        )),
        "stream_wrapper_unregister" => Some(fixed(&["protocol"])),
        "stream_wrapper_restore" => Some(fixed(&["protocol"])),
        "stream_socket_enable_crypto" => Some(optional(
            &["stream", "enable", "crypto_method", "session_stream"],
            2,
            vec![null_lit(), null_lit()],
        )),
        "stream_context_create" => Some(optional(
            &["options", "params"],
            0,
            vec![null_lit(), null_lit()],
        )),
        "stream_context_get_default" => {
            Some(optional(&["options"], 0, vec![null_lit()]))
        }
        "stream_context_set_default" => Some(fixed(&["options"])),
        "stream_context_set_option" => Some(optional(
            &["context", "wrapper_or_options", "option_name", "value"],
            2,
            vec![null_lit(), null_lit()],
        )),
        "stream_context_set_params" => Some(fixed(&["context", "params"])),
        "stream_context_get_options" => Some(fixed(&["context"])),
        "stream_context_get_params" => Some(fixed(&["context"])),
        "stream_resolve_include_path" => Some(fixed(&["filename"])),
        "stream_filter_register" => Some(fixed(&["filter_name", "class"])),
        "stream_bucket_make_writeable" => Some(fixed(&["brigade"])),
        "stream_bucket_new" => Some(fixed(&["stream", "buffer"])),
        "stream_bucket_append" => Some(fixed(&["brigade", "bucket"])),
        "stream_bucket_prepend" => Some(fixed(&["brigade", "bucket"])),
        "stream_set_chunk_size" => Some(fixed(&["stream", "size"])),
        "stream_set_read_buffer" => Some(fixed(&["stream", "size"])),
        "stream_set_write_buffer" => Some(fixed(&["stream", "size"])),
        "stream_get_line" => Some(optional(
            &["stream", "length", "ending"],
            2,
            vec![string_lit("")],
        )),
        "stream_select" => {
            let mut sig = optional(
                &["read", "write", "except", "seconds", "microseconds"],
                4,
                vec![int_lit(0)],
            );
            sig.ref_params[0] = true;
            sig.ref_params[1] = true;
            sig.ref_params[2] = true;
            Some(sig)
        }
        "stream_set_blocking" => Some(fixed(&["stream", "enable"])),
        "stream_set_timeout" => Some(optional(
            &["stream", "seconds", "microseconds"],
            2,
            vec![int_lit(0)],
        )),
        "stream_socket_sendto" => Some(optional(
            &["socket", "data", "flags", "address"],
            2,
            vec![int_lit(0), string_lit("")],
        )),
        "stream_socket_recvfrom" => {
            let mut sig = optional(
                &["socket", "length", "flags", "address"],
                2,
                vec![int_lit(0), string_lit("")],
            );
            sig.ref_params[3] = true;
            Some(sig)
        }
        "stream_socket_get_name" => Some(fixed(&["socket", "remote"])),
        "stream_socket_pair" => Some(fixed(&["domain", "type", "protocol"])),
        "popen" => Some(fixed(&["command", "mode"])),
        "pclose" => Some(fixed(&["handle"])),
        "opendir" => Some(fixed(&["directory"])),
        "readdir" | "closedir" | "rewinddir" => Some(fixed(&["dir_handle"])),
        "stream_socket_shutdown" => Some(fixed(&["stream", "mode"])),
        "gethostname" => Some(fixed(&[])),
        "gethostbyname" => Some(fixed(&["hostname"])),
        "gethostbyaddr" => Some(fixed(&["ip"])),
        "getprotobyname" => Some(fixed(&["protocol"])),
        "getprotobynumber" => Some(fixed(&["protocol"])),
        "getservbyname" => Some(fixed(&["service", "protocol"])),
        "getservbyport" => Some(fixed(&["port", "protocol"])),
        "long2ip" => Some(fixed(&["ip"])),
        "ip2long" => Some(fixed(&["ip"])),
        "inet_ntop" | "inet_pton" => Some(fixed(&["ip"])),
        "stream_get_transports" | "stream_get_wrappers" | "stream_get_filters" => {
            Some(fixed(&[]))
        }
        "stream_filter_append" | "stream_filter_prepend" => Some(optional(
            &["stream", "filtername", "read_write", "params"],
            2,
            vec![int_lit(3), null_lit()],
        )),
        "stream_filter_remove" => Some(fixed(&["stream_filter"])),

        "spl_autoload_register" => Some(optional(
            &["callback", "throw", "prepend"],
            0,
            vec![null_lit(), bool_lit(true), bool_lit(false)],
        )),
        "spl_autoload_unregister" => Some(fixed(&["callback"])),
        "spl_autoload_functions" | "spl_classes" => Some(fixed(&[])),
        "spl_autoload_extensions" => {
            Some(optional(&["file_extensions"], 0, vec![null_lit()]))
        }
        "spl_autoload_call" => Some(fixed(&["class"])),
        "spl_autoload" => Some(optional(&["class", "file_extensions"], 1, vec![null_lit()])),
        "spl_object_id" | "spl_object_hash" => Some(fixed(&["object"])),

        "ptr" => Some(fixed(&["value"])),
        "ptr_is_null" | "ptr_get" | "ptr_read8" | "ptr_read16" | "ptr_read32" => {
            Some(fixed(&["pointer"]))
        }
        "ptr_read_string" => Some(fixed(&["pointer", "length"])),
        "ptr_offset" => Some(fixed(&["pointer", "offset"])),
        "ptr_set" | "ptr_write8" | "ptr_write16" | "ptr_write32" => {
            Some(fixed(&["pointer", "value"]))
        }
        "ptr_write_string" => Some(fixed(&["pointer", "string"])),
        "ptr_sizeof" => Some(fixed(&["type"])),
        "zval_pack" => Some(fixed(&["value"])),
        "zval_unpack" | "zval_type" | "zval_free" => Some(fixed(&["zval"])),
        "buffer_new" => Some(fixed(&["length"])),
        "buffer_len" | "buffer_free" => Some(fixed(&["buffer"])),
        _ => None,
    }
}

/// Returns the signature used when a builtin is accessed as a first-class callable.
///
/// Consults the builtin registry first (via `registry::first_class_callable_sig`);
/// falls back to the legacy static table when the name is not registered. Some builtins
/// (e.g., `strlen`, `count`, `buffer_len`) have explicit first-class signatures with
/// precise parameter and return types. Unknown builtins fall through to
/// `general_first_class_callable_builtin_sig`.
///
/// Called from:
/// - first-class callable lowering for builtin references
pub(crate) fn first_class_callable_builtin_sig(name: &str) -> Option<FunctionSig> {
    crate::builtins::registry::first_class_callable_sig(name)
        .or_else(|| legacy_first_class_callable_builtin_sig(name))
}

/// Legacy static lookup table for first-class-callable builtin signatures.
///
/// Returns the precise typed `FunctionSig` used when a builtin is referenced
/// as a first-class callable. `first_class_callable_builtin_sig` delegates
/// here when the registry does not have an entry for `name`.
fn legacy_first_class_callable_builtin_sig(name: &str) -> Option<FunctionSig> {
    match name {
        "strlen" => Some(FunctionSig {
            params: vec![("string".to_string(), PhpType::Str)],
            param_type_exprs: vec![None],
            param_attributes: vec![Vec::new()],
            defaults: vec![None],
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false],
            declared_params: vec![true],
            variadic: None,
            deprecation: None,
        }),
        "count" => Some(FunctionSig {
            params: vec![(
                "value".to_string(),
                PhpType::AssocArray {
                    key: Box::new(PhpType::Mixed),
                    value: Box::new(PhpType::Mixed),
                },
            )],
            param_type_exprs: vec![None],
            param_attributes: vec![Vec::new()],
            defaults: vec![None],
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false],
            declared_params: vec![true],
            variadic: None,
            deprecation: None,
        }),
        "buffer_len" => Some(FunctionSig {
            params: vec![("buffer".to_string(), PhpType::Buffer(Box::new(PhpType::Int)))],
            param_type_exprs: vec![None],
            param_attributes: vec![Vec::new()],
            defaults: vec![None],
            return_type: PhpType::Int,
            declared_return: true,
            by_ref_return: false,
            ref_params: vec![false],
            declared_params: vec![true],
            variadic: None,
            deprecation: None,
        }),
        _ => general_first_class_callable_builtin_sig(name),
    }
}

/// Fallback first-class callable signature builder for builtins without an explicit override.
/// Constructs typed signatures for builtins where the canonical call signature
/// (from `builtin_call_sig`) is not appropriate for first-class use.
fn general_first_class_callable_builtin_sig(name: &str) -> Option<FunctionSig> {
    match name {
        "time" | "json_last_error" => Some(typed_first_class_builtin_sig(
            name,
            &[],
            PhpType::Int,
        )),
        "phpversion" | "getcwd" | "sys_get_temp_dir" | "json_last_error_msg"
        | "date_default_timezone_get" => {
            Some(typed_first_class_builtin_sig(name, &[], PhpType::Str))
        }
        "date_default_timezone_set" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Bool,
        )),
        "pi" => Some(typed_first_class_builtin_sig(name, &[], PhpType::Float)),
        "intval" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Int,
        )),
        "floatval" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Float,
        )),
        "strval" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Str,
        )),
        // NOTE: is_array/is_object/is_scalar are intentionally NOT first-class callable.
        // No runtime callable wrapper is emitted for these three predicates, so listing
        // them here would emit an undefined `_fn_is_*` invoker reference in any program
        // using dynamic string callbacks.
        // Direct calls are fully supported; first-class/string-callback use is not (yet).
        "boolval" | "is_bool" | "is_null" | "is_float" | "is_double" | "is_real"
        | "is_int" | "is_integer" | "is_long" | "is_iterable" | "is_string"
        | "is_numeric" | "is_nan" | "is_finite" | "is_infinite" | "is_countable"
        | "ctype_alpha" | "ctype_digit" | "ctype_alnum" | "ctype_space" | "ctype_upper" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Bool))
        }
        "defined" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Bool,
        )),
        "method_exists" | "property_exists" => {
            return_typed_first_class_builtin_sig(name, PhpType::Bool)
        }
        "gettype" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Str,
        )),
        // NOTE: serialize/unserialize are intentionally NOT first-class callable. They lower to a
        // deferred runtime fatal stub with no `_fn_serialize`/`_fn_unserialize` callable wrapper, so
        // listing them here would emit an undefined invoker reference in any program that materializes
        // builtin callable wrappers (same constraint as is_array/is_object/is_scalar above). Direct
        // calls are fully recognized; first-class/string-callback use is not.
        "printf" => return_typed_first_class_builtin_sig(name, PhpType::Int),
        "grapheme_strrev" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::False]),
        )),
        "strtolower" | "strtoupper" | "ucfirst" | "lcfirst" | "strrev"
        | "addslashes" | "stripslashes" | "nl2br" | "bin2hex" | "hex2bin"
        | "htmlspecialchars" | "htmlentities" | "html_entity_decode" | "urlencode"
        | "urldecode" | "rawurlencode" | "rawurldecode" | "base64_encode"
        | "base64_decode" | "trim" | "ltrim" | "rtrim" | "chop" | "ucwords"
        | "str_repeat" | "strstr" | "str_replace" | "str_ireplace" | "explode"
        | "implode" | "substr_replace" | "str_pad" | "str_split" | "wordwrap"
        | "sprintf" | "hash" | "hash_hmac" | "md5" | "sha1" | "crc32" | "number_format"
        | "chr" | "strtr" | "strip_tags" => {
            Some(typed_first_class_builtin_sig(
                name,
                &[PhpType::Str],
                PhpType::Str,
            ))
        }
        "strpos" | "strrpos" | "strcmp" | "strcasecmp" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Int,
        )),
        // stripos/strripos are the case-insensitive strpos/strrpos: int index or false.
        "stripos" | "strripos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // strncmp/strncasecmp/substr_compare/levenshtein return a signed int result.
        "strncmp" | "strncasecmp" | "substr_compare" | "levenshtein" => {
            Some(typed_first_class_builtin_sig(
                name,
                &[PhpType::Str, PhpType::Str],
                PhpType::Int,
            ))
        }
        // -- multibyte string (mbstring) first-class callables (recognition-only) --
        // mb_substr/mb_strtolower/mb_strtoupper/mb_convert_case/mb_convert_encoding
        // each take a string and return a string.
        "mb_substr" | "mb_strtolower" | "mb_strtoupper" | "mb_convert_case" => Some(
            typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Str,
        )),
        // mb_strlen/mb_strwidth measure a string and return an int.
        "mb_strlen" | "mb_strwidth" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Int,
        )),
        // mb_stripos/mb_strripos are case-insensitive multibyte search: int index or false.
        "mb_stripos" | "mb_strripos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // mb_strstr/mb_stristr return the matched substring or false.
        "mb_strstr" | "mb_stristr" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // mb_detect_encoding returns the detected encoding name or false.
        "mb_detect_encoding" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // mb_internal_encoding returns the current encoding (string) or true/false when set.
        "mb_internal_encoding" => Some(typed_first_class_builtin_sig(
            name,
            &[],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // mb_ord returns the code point (int) or false.
        "mb_ord" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // mb_str_split splits a string into an array of multibyte characters.
        "mb_str_split" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Array(Box::new(PhpType::Str)),
        )),
        // mb_strpos/mb_strrpos are multibyte search: int index or false.
        "mb_strpos" | "mb_strrpos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // -- grapheme (intl) first-class callables (recognition-only) --
        // grapheme_strlen counts graphemes: int, or false/null on error.
        "grapheme_strlen" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool, PhpType::Void]),
        )),
        // grapheme_substr/grapheme_extract return the matched substring or false.
        "grapheme_substr" | "grapheme_extract" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // grapheme_stripos/grapheme_strripos are grapheme search: int index or false.
        "grapheme_stripos" | "grapheme_strripos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // grapheme_strpos/grapheme_strrpos are case-sensitive grapheme search: int index or false.
        "grapheme_strpos" | "grapheme_strrpos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // grapheme_str_split splits a string into an array of graphemes, or false on error.
        "grapheme_str_split" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Array(Box::new(PhpType::Str)), PhpType::Bool]),
        )),
        // -- normalizer (intl) first-class callables (recognition-only) --
        // normalizer_normalize returns the normalized string or false.
        "normalizer_normalize" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // normalizer_is_normalized returns whether the string is already normalized.
        "normalizer_is_normalized" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Bool,
        )),
        // -- iconv first-class callables (recognition-only) --
        // iconv(from_encoding, to_encoding, string) returns the converted string or false.
        "iconv" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // iconv_substr/iconv_mime_decode return the converted string or false.
        "iconv_substr" | "iconv_mime_decode" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // iconv_strlen measures a string: int length or false.
        "iconv_strlen" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // iconv_strpos/iconv_strrpos are byte-offset search: int index or false.
        "iconv_strpos" | "iconv_strrpos" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // -- process / system control first-class callables (recognition-only) --
        // pcntl_signal(signal, handler, restart_syscalls?) installs a handler and returns bool.
        "pcntl_signal" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Int, PhpType::Mixed],
            PhpType::Bool,
        )),
        // pcntl_alarm returns the number of seconds remaining on any previous alarm.
        "pcntl_alarm" => Some(typed_first_class_builtin_sig(name, &[PhpType::Int], PhpType::Int)),
        // pcntl_async_signals toggles async signal dispatch and returns the prior state.
        "pcntl_async_signals" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Bool,
        )),
        // pcntl_signal_get_handler returns the installed handler: an int constant or a callable name.
        "pcntl_signal_get_handler" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Int],
            PhpType::Union(vec![PhpType::Int, PhpType::Str]),
        )),
        // proc_close waits on the process and returns its termination status int.
        "proc_close" => Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Int)),
        // getmypid returns the current process id, or false if it cannot be determined.
        "getmypid" => {
            return_typed_first_class_builtin_sig(name, PhpType::Union(vec![PhpType::Int, PhpType::Bool]))
        }
        // cli_set_process_title/setproctitle set the process title and return bool.
        "cli_set_process_title" | "setproctitle" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Str], PhpType::Bool))
        }
        // sapi_windows_cp_get returns the active Windows code page as an int.
        "sapi_windows_cp_get" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Str], PhpType::Int))
        }
        // sapi_windows_cp_set sets the Windows code page and returns bool.
        "sapi_windows_cp_set" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Int], PhpType::Bool))
        }
        // sapi_windows_vt100_support toggles/queries VT100 support on a stream and returns bool.
        "sapi_windows_vt100_support" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Bool))
        }
        // ini_get reads a php.ini directive: the string value, or false if unset.
        "ini_get" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Union(vec![PhpType::Str, PhpType::Bool]),
        )),
        // get_defined_constants returns the map of defined constants as an array.
        "get_defined_constants" => {
            return_typed_first_class_builtin_sig(name, PhpType::Array(Box::new(PhpType::Mixed)))
        }
        // -- misc / error-handling / process first-class callables (recognition-only) --
        // method_exists checks whether a class or object exposes a method and returns bool.
        // trigger_error emits a user-level error/notice and returns bool.
        "trigger_error" => Some(typed_first_class_builtin_sig(name, &[PhpType::Str], PhpType::Bool)),
        // set_error_handler installs a handler and returns the previous one (?callable).
        "set_error_handler" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Mixed))
        }
        // restore_error_handler pops the current handler and returns true.
        "restore_error_handler" => return_typed_first_class_builtin_sig(name, PhpType::Bool),
        // restore_exception_handler pops the current exception handler and returns true.
        "restore_exception_handler" => return_typed_first_class_builtin_sig(name, PhpType::Bool),
        // set_exception_handler installs a handler and returns the previous one (?callable).
        "set_exception_handler" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Mixed))
        }
        // filter_var(mixed $value, int $filter = FILTER_DEFAULT,
        // array|int $options = 0): mixed — the filtered value or false on failure.
        "filter_var" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Int, PhpType::Mixed],
            PhpType::Mixed,
        )),
        // preg_quote escapes regex metacharacters and returns the quoted string.
        "preg_quote" => Some(typed_first_class_builtin_sig(name, &[PhpType::Str], PhpType::Str)),
        // preg_grep returns the matching array entries, or false on error.
        "preg_grep" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Union(vec![PhpType::Array(Box::new(PhpType::Mixed)), PhpType::Bool]),
        )),
        // version_compare returns -1/0/1, or a bool when an operator is supplied.
        "version_compare" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Int, PhpType::Bool]),
        )),
        // unpack unpacks a binary string into an array, or false on error.
        "unpack" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Union(vec![PhpType::Array(Box::new(PhpType::Mixed)), PhpType::Bool]),
        )),
        // random_bytes returns a cryptographically secure random byte string.
        "random_bytes" => Some(typed_first_class_builtin_sig(name, &[PhpType::Int], PhpType::Str)),
        // http_build_query serializes an array/object into a URL query string.
        "http_build_query" => {
            Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Str))
        }
        // escapeshellarg returns the shell-escaped form of its argument.
        "escapeshellarg" => Some(typed_first_class_builtin_sig(name, &[PhpType::Str], PhpType::Str)),
        // assert evaluates the assertion and returns bool (may throw its description).
        "assert" => Some(typed_first_class_builtin_sig(name, &[PhpType::Mixed], PhpType::Bool)),
        // sapi_windows_cp_conv converts a string between code pages: a string or null.
        "sapi_windows_cp_conv" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Mixed],
            PhpType::Union(vec![PhpType::Str, PhpType::Void]),
        )),
        // posix_kill sends a signal to a process and returns bool.
        "posix_kill" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Int, PhpType::Int],
            PhpType::Bool,
        )),
        // strcspn/strspn/strpbrk/hexdec are intentionally absent here: they are
        // lowered only by the active EIR backend, so the frozen legacy direct
        // backend that emits dynamic first-class-callable wrapper bodies cannot
        // resolve them (it would emit an unresolved `bl _fn_<name>`). They remain
        // fully supported as direct calls; first-class-callable syntax is not
        // offered for them.
        "str_contains" | "str_starts_with" | "str_ends_with" | "hash_equals" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Str],
            PhpType::Bool,
        )),
        "hash_algos" => Some(typed_first_class_builtin_sig(
            name,
            &[],
            PhpType::Array(Box::new(PhpType::Str)),
        )),
        "array_keys" | "array_values" | "array_reverse" | "array_unique" | "array_rand" => {
            Some(typed_first_class_builtin_sig(
                name,
                &[PhpType::Array(Box::new(PhpType::Mixed))],
                PhpType::Array(Box::new(PhpType::Mixed)),
            ))
        }
        "array_chunk" | "array_pad" | "array_fill" | "array_slice" | "array_diff"
        | "array_intersect" | "array_replace_recursive" | "range" => {
            return_typed_first_class_builtin_sig(
                name,
                PhpType::Array(Box::new(PhpType::Mixed)),
            )
        }
        // array_key_first returns the first key (string|int) or null for an empty array.
        "array_key_first" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Union(vec![PhpType::Str, PhpType::Int, PhpType::Void]),
        )),
        // array_key_last returns the last key (string|int) or null for an empty array.
        "array_key_last" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Union(vec![PhpType::Str, PhpType::Int, PhpType::Void]),
        )),
        "array_flip" | "array_combine" | "array_fill_keys" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            },
        )),
        "array_sum" | "array_product" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Int))],
            PhpType::Int,
        )),
        "array_key_exists" | "in_array" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Bool,
        )),
        "array_search" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Array(Box::new(PhpType::Mixed)), PhpType::Bool],
            PhpType::Mixed,
        )),
        "array_pop" | "array_shift" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Mixed,
        )),
        "iterator_to_array" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Iterable, PhpType::Bool],
            PhpType::Array(Box::new(PhpType::Mixed)),
        )),
        "iterator_count" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Iterable],
            PhpType::Int,
        )),
        "iterator_apply" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Object("Traversable".to_string())],
            PhpType::Int,
        )),
        "array_push" | "array_unshift" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed)), PhpType::Mixed],
            PhpType::Void,
        )),
        "sort" | "rsort" | "shuffle" | "natsort" | "natcasesort" | "asort"
        | "arsort" | "ksort" | "krsort" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Array(Box::new(PhpType::Mixed))],
            PhpType::Void,
        )),
        "is_file" | "is_dir" | "is_readable" | "is_writable" | "is_writeable"
        | "is_executable" | "is_link" | "file_exists" | "fnmatch" | "chmod" | "chown"
        | "chgrp" | "touch" | "ftruncate" | "fflush" | "fsync" | "fdatasync"
        | "symlink" | "link" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str],
            PhpType::Bool,
        )),
        "file_get_contents" | "hash_file" | "file" | "filesize" | "filemtime" | "fileatime"
        | "filectime" | "fileperms" | "fileowner" | "filegroup" | "fileinode"
        | "filetype" | "stat" | "lstat" | "basename" | "dirname" | "realpath"
        | "pathinfo" | "readlink" | "linkinfo" | "tempnam" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Mixed;
            sig.declared_return = true;
            Some(sig)
        }
        "abs" | "min" | "max" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Mixed,
        )),
        "clamp" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Mixed, PhpType::Mixed],
            PhpType::Mixed,
        )),
        "floor" | "ceil" | "sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
        | "sinh" | "cosh" | "tanh" | "log2" | "log10" | "exp" | "deg2rad"
        | "rad2deg" | "microtime" | "log" | "atan2" | "hypot" | "pow" | "fmod"
        | "fdiv" | "round" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Float,
        )),
        "intdiv" | "rand" | "mt_rand" | "random_int" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Int, PhpType::Int],
            PhpType::Int,
        )),
        "date" | "gmdate" | "php_uname" | "readline" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Str;
            sig.declared_return = true;
            Some(sig)
        }
        "preg_match" | "preg_match_all" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Int;
            sig.declared_return = true;
            Some(sig)
        }
        // Date/time helpers that return scalars or arrays. Like `date`/`gmdate`,
        // they reuse the catalog parameter contract and override only the return
        // type, so first-class-callable references (`checkdate(...)`, etc.) match PHP.
        "checkdate" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Bool;
            sig.declared_return = true;
            Some(sig)
        }
        "mktime" | "gmmktime" | "__elephc_mktime_raw" | "__elephc_gmmktime_raw" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Int;
            sig.declared_return = true;
            Some(sig)
        }
        "strtotime" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Union(vec![PhpType::Int, PhpType::False]);
            sig.declared_return = true;
            Some(sig)
        }
        "getdate" | "localtime" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::AssocArray {
                key: Box::new(PhpType::Mixed),
                value: Box::new(PhpType::Mixed),
            };
            sig.declared_return = true;
            Some(sig)
        }
        "hrtime" => {
            let mut sig = builtin_call_sig(name)?;
            sig.return_type = PhpType::Mixed;
            sig.declared_return = true;
            Some(sig)
        }
        "json_encode" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed, PhpType::Int, PhpType::Int],
            PhpType::Union(vec![PhpType::Str, PhpType::False]),
        )),
        "json_decode" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Bool, PhpType::Int, PhpType::Int],
            PhpType::Mixed,
        )),
        "json_validate" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Int, PhpType::Int],
            PhpType::Bool,
        )),
        "serialize" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Str,
        )),
        "unserialize" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Mixed],
            PhpType::Mixed,
        )),
        "preg_replace_callback" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Str, PhpType::Callable, PhpType::Str],
            PhpType::Str,
        )),
        "ptr_read16" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None)],
            PhpType::Int,
        )),
        "ptr_write16" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None), PhpType::Int],
            PhpType::Void,
        )),
        "ptr_read_string" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None), PhpType::Int],
            PhpType::Str,
        )),
        "ptr_write_string" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None), PhpType::Str],
            PhpType::Int,
        )),
        "zval_pack" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Mixed],
            PhpType::Pointer(None),
        )),
        "zval_unpack" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None)],
            PhpType::Mixed,
        )),
        "zval_type" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None)],
            PhpType::Int,
        )),
        "zval_free" => Some(typed_first_class_builtin_sig(
            name,
            &[PhpType::Pointer(None)],
            PhpType::Void,
        )),
        _ => None,
    }
}

/// Builds a first-class callable signature by overriding parameter and return types
/// on top of an existing builtin catalog entry.
///
/// Panics if `name` is not in the builtin catalog — all first-class callable builtins
/// must have a prior `builtin_call_sig` entry.
fn typed_first_class_builtin_sig(
    name: &str,
    param_types: &[PhpType],
    return_type: PhpType,
) -> FunctionSig {
    let mut sig = builtin_call_sig(name).expect("first-class builtin must have a catalog signature");
    for (idx, param_ty) in param_types.iter().enumerate() {
        if let Some((_, ty)) = sig.params.get_mut(idx) {
            *ty = param_ty.clone();
        }
    }
    sig.return_type = return_type;
    sig.declared_return = true;
    sig
}

/// Builds a first-class callable signature by overriding only the return type
/// on top of an existing builtin catalog entry. Returns `None` if the builtin
/// has no catalog entry.
fn return_typed_first_class_builtin_sig(name: &str, return_type: PhpType) -> Option<FunctionSig> {
    let mut sig = builtin_call_sig(name)?;
    sig.return_type = return_type;
    sig.declared_return = true;
    Some(sig)
}

/// Constructs a signature with all parameters required (no defaults).
fn fixed(params: &[&str]) -> FunctionSig {
    make_sig(params, vec![None; params.len()], None)
}

/// Constructs a signature with some trailing parameters optional.
///
/// `required` indicates how many leading params are mandatory; the rest receive
/// defaults from `optional_defaults` (mapped positionally). Defaults are padded
/// with `None` if fewer are provided than total params.
fn optional(params: &[&str], required: usize, optional_defaults: Vec<Expr>) -> FunctionSig {
    let mut defaults = vec![None; required];
    defaults.extend(optional_defaults.into_iter().map(Some));
    while defaults.len() < params.len() {
        defaults.push(None);
    }
    make_sig(params, defaults, None)
}

/// Constructs a variadic signature — trailing param collects excess arguments as an array.
///
/// `regular_params` lists the fixed parameters; `variadic_name` names the trailing
/// variadic parameter. The variadic param starts as an empty `array` default.
fn variadic(regular_params: &[&str], variadic_name: &str) -> FunctionSig {
    let mut params = regular_params.to_vec();
    params.push(variadic_name);
    let mut defaults = vec![None; regular_params.len()];
    defaults.push(Some(Expr::new(ExprKind::ArrayLiteral(Vec::new()), Span::dummy())));
    make_sig(&params, defaults, Some(variadic_name))
}

/// Sets the declared return type for a signature while preserving its parameter contract.
fn with_return(mut sig: FunctionSig, return_type: PhpType) -> FunctionSig {
    sig.return_type = return_type;
    sig.declared_return = true;
    sig
}

/// Marks the first parameter of a signature as by-reference.
fn first_param_ref(mut sig: FunctionSig) -> FunctionSig {
    if let Some(first_ref) = sig.ref_params.first_mut() {
        *first_ref = true;
    }
    sig
}

/// Low-level `FunctionSig` constructor from raw parts.
///
/// Assembles params as `Mixed` types, sets all other fields from arguments,
/// and defaults `deprecation` to `None`.
fn make_sig(params: &[&str], defaults: Vec<Option<Expr>>, variadic: Option<&str>) -> FunctionSig {
    FunctionSig {
        params: params
            .iter()
            .map(|name| ((*name).to_string(), PhpType::Mixed))
            .collect(),
        param_type_exprs: vec![None; params.len()],
        param_attributes: vec![Vec::new(); params.len()],
        defaults,
        return_type: PhpType::Mixed,
        declared_return: false,
        by_ref_return: false,
        ref_params: vec![false; params.len()],
        declared_params: vec![false; params.len()],
        variadic: variadic.map(str::to_string),
        deprecation: None,
    }
}

/// Constructs an `i64` literal expression for use in default parameter values.
fn int_lit(value: i64) -> Expr {
    Expr::new(ExprKind::IntLiteral(value), Span::dummy())
}

/// Constructs a string literal expression for use in default parameter values.
fn string_lit(value: &str) -> Expr {
    Expr::new(ExprKind::StringLiteral(value.to_string()), Span::dummy())
}

/// Constructs a boolean literal expression for use in default parameter values.
fn bool_lit(value: bool) -> Expr {
    Expr::new(ExprKind::BoolLiteral(value), Span::dummy())
}

/// Constructs a null literal expression for use in default parameter values.
fn null_lit() -> Expr {
    Expr::new(ExprKind::Null, Span::dummy())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Computes the callable signature metadata for variadic.
    fn variadic_sig(params: Vec<(String, PhpType)>) -> FunctionSig {
        FunctionSig {
            defaults: vec![None; params.len()],
            param_type_exprs: vec![None; params.len()],
            param_attributes: vec![Vec::new(); params.len()],
            return_type: PhpType::Mixed,
            declared_return: false,
            by_ref_return: false,
            ref_params: vec![false; params.len()],
            declared_params: vec![false; params.len()],
            params,
            variadic: Some("values".to_string()),
            deprecation: None,
        }
    }

    /// Builds the parameter metadata for callable wrapper sig retypes existing non array variadic.
    #[test]
    fn callable_wrapper_sig_retypes_existing_non_array_variadic_param() {
        let sig = variadic_sig(vec![
            ("format".to_string(), PhpType::Str),
            ("values".to_string(), PhpType::Mixed),
        ]);

        let wrapper_sig = callable_wrapper_sig(&sig);

        assert_eq!(wrapper_sig.params.len(), 2);
        assert_eq!(
            wrapper_sig.params[1],
            (
                "values".to_string(),
                PhpType::Array(Box::new(PhpType::Mixed)),
            )
        );
        assert_eq!(wrapper_sig.defaults.len(), 2);
        assert_eq!(wrapper_sig.ref_params.len(), 2);
        assert_eq!(wrapper_sig.declared_params.len(), 2);
    }

    /// Builds the parameter metadata for callable wrapper sig appends missing variadic.
    #[test]
    fn callable_wrapper_sig_appends_missing_variadic_param() {
        let sig = variadic_sig(vec![("format".to_string(), PhpType::Str)]);

        let wrapper_sig = callable_wrapper_sig(&sig);

        assert_eq!(wrapper_sig.params.len(), 2);
        assert_eq!(
            wrapper_sig.params[1],
            (
                "values".to_string(),
                PhpType::Array(Box::new(PhpType::Mixed)),
            )
        );
        assert_eq!(wrapper_sig.defaults.len(), 2);
        assert_eq!(wrapper_sig.ref_params.len(), 2);
        assert_eq!(wrapper_sig.declared_params.len(), 2);
        // Regression: the synthesized variadic slot's default must be `Some(<empty array>)`,
        // matching `variadic()`/`ensure_variadic_for_func_args`'s convention — NOT `None`.
        // A bare `None` makes call-argument-count validation treat the variadic tail as a
        // required parameter, falsely rejecting zero-arg calls (this exact regression made
        // every zero/under-count call to a variadic class method error "expects at least 1
        // arguments, got 0" even though PHP accepts it).
        assert!(matches!(
            &wrapper_sig.defaults[1],
            Some(expr) if matches!(expr.kind, ExprKind::ArrayLiteral(ref items) if items.is_empty())
        ));
    }
}
