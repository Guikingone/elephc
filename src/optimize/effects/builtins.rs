//! Purpose:
//! Models optimizer side effects for PHP builtin calls.
//! Feeds purity, callable alias, builtin, and call-effect decisions into pruning and dead-code elimination.
//!
//! Called from:
//! - `crate::optimize::effects`
//!
//! Key details:
//! - Effect summaries must account for globals, heap/runtime state, output, throws, and by-reference mutation.

/// Returns `true` if the named PHP builtin function is both pure (no side effects)
/// and non-throwing (does not emit warnings or fatal errors that could alter control flow).
///
/// Pure non-throwing builtins are safe to eliminate via dead-code elimination, fold as
/// constant expressions when all arguments are constant, and reorder freely relative to
/// other statements. The list intentionally excludes builtins that read/write shared runtime
/// state (`json_*`), produce observable side effects (output, filesystem, globals, heap
/// mutation), or can throw/fatal (pointer helpers, etc.).
///
/// # Arguments
/// * `name` - Lowercase ASCII builtin function name (case-insensitive PHP builtins use
///   lowercase in the catalog; callers must normalize before calling).
///
/// # Returns
/// `true` if the builtin is pure and non-throwing; `false` otherwise.
///
/// # Notes
/// `json_encode`/`json_decode`/`json_validate`/`json_last_error`/`json_last_error_msg`
/// are excluded because they read/write the shared `_json_last_error` runtime symbol.
/// Pointer memory helpers (`ptr_read16`, `ptr_write16`, `ptr_read_string`, `ptr_write_string`)
/// are excluded because raw memory access can null-dereference or fatal.
pub(super) fn is_pure_non_throwing_builtin(name: &str) -> bool {
    matches!(
        name,
        "strlen"
            | "intval"
            | "floatval"
            | "boolval"
            | "gettype"
            | "get_debug_type"
            | "is_array"
            | "is_object"
            | "is_scalar"
            | "is_bool"
            | "is_float"
            | "is_int"
            | "is_null"
            | "is_numeric"
            | "is_string"
            | "is_resource"
            | "get_resource_type"
            | "get_resource_id"
            | "abs"
            | "min"
            | "max"
            | "floor"
            | "ceil"
            | "round"
            | "sqrt"
            | "pow"
            | "fmod"
            | "fdiv"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "atan2"
            | "deg2rad"
            | "rad2deg"
            | "sinh"
            | "cosh"
            | "tanh"
            | "log"
            | "log2"
            | "log10"
            | "exp"
            | "hypot"
            | "pi"
            | "number_format"
            | "substr"
            | "strpos"
            | "strrpos"
            | "strstr"
            | "str_replace"
            | "str_ireplace"
            | "substr_replace"
            | "strtolower"
            | "strtoupper"
            | "ucfirst"
            | "lcfirst"
            | "ucwords"
            | "trim"
            | "ltrim"
            | "rtrim"
            | "chop"
            | "str_repeat"
            | "strrev"
            | "grapheme_strrev"
            | "str_pad"
            | "explode"
            | "str_split"
            | "strcmp"
            | "strcasecmp"
            | "strncmp"
            | "strncasecmp"
            | "stripos"
            | "strripos"
            | "strtr"
            | "strip_tags"
            | "levenshtein"
            // multibyte string builtins are pure reads (mb_internal_encoding is
            // omitted: it reads/writes global mbstring encoding state).
            | "mb_substr"
            | "mb_strlen"
            | "mb_stripos"
            | "mb_strripos"
            | "mb_strstr"
            | "mb_stristr"
            | "mb_strtolower"
            | "mb_strtoupper"
            | "mb_convert_case"
            | "mb_convert_encoding"
            | "mb_detect_encoding"
            | "mb_ord"
            | "mb_str_split"
            | "mb_strwidth"
            | "mb_strpos"
            | "mb_strrpos"
            // intl grapheme/normalizer and iconv builtins are pure string transforms with no
            // persistent state. grapheme_extract's trailing `&$next` write is modeled through
            // the signature's ref_params, not purity (same pattern as str_replace's `&$count`).
            | "grapheme_strlen"
            | "grapheme_substr"
            | "grapheme_stripos"
            | "grapheme_strripos"
            | "grapheme_strpos"
            | "grapheme_strrpos"
            | "grapheme_str_split"
            | "grapheme_extract"
            | "normalizer_normalize"
            | "normalizer_is_normalized"
            | "iconv"
            | "iconv_strlen"
            | "iconv_strpos"
            | "iconv_strrpos"
            | "iconv_substr"
            | "iconv_mime_decode"
            | "strcspn"
            | "strspn"
            | "strpbrk"
            | "hexdec"
            | "bindec"
            | "dechex"
            | "decoct"
            | "decbin"
            | "preg_last_error_msg"
            | "preg_last_error"
            | "str_contains"
            | "str_starts_with"
            | "str_ends_with"
            | "hash_equals"
            | "hash_algos"
            | "octdec"
            | "substr_count"
            | "ord"
            | "chr"
            | "nl2br"
            | "wordwrap"
            | "strval"
            | "strnatcmp"
            | "strnatcasecmp"
            | "strrchr"
            | "parse_url"
            | "addcslashes"
            | "stripcslashes"
            | "str_getcsv"
            | "pack"
            | "mb_encode_numericentity"
            | "addslashes"
            | "stripslashes"
            | "htmlspecialchars"
            | "htmlentities"
            | "html_entity_decode"
            | "urlencode"
            | "urldecode"
            | "rawurlencode"
            | "rawurldecode"
            | "md5"
            | "sha1"
            | "crc32"
            | "base64_encode"
            | "base64_decode"
            | "bin2hex"
            | "hex2bin"
            | "long2ip"
            | "ip2long"
            | "inet_ntop"
            | "inet_pton"
            | "ctype_alpha"
            | "ctype_digit"
            | "ctype_alnum"
            | "ctype_space"
            | "ctype_upper"
            | "array_key_exists"
            | "array_key_first"
            | "array_key_last"
            | "array_replace_recursive"
            | "array_replace"
            | "array_is_list"
            | "is_countable"
            | "array_search"
            | "array_keys"
            | "array_values"
            | "array_merge"
            | "array_combine"
            | "array_flip"
            | "array_reverse"
            | "array_unique"
            | "array_column"
            | "array_sum"
            | "array_product"
            | "array_chunk"
            | "array_pad"
            | "array_fill"
            | "array_fill_keys"
            | "array_diff"
            | "array_intersect"
            | "array_diff_key"
            | "array_intersect_key"
            | "range"
            // -- misc recognition-only builtins that are pure reads/transforms --
            // method_exists reads the class table; preg_quote/preg_grep/version_compare/unpack/
            // http_build_query/escapeshellarg are pure string/array transforms; filter_var is a
            // pure input-validation transform on its arguments. The impure siblings (trigger_error,
            // set_error_handler, restore_error_handler, restore_exception_handler,
            // set_exception_handler, random_bytes, assert, sapi_windows_cp_conv, posix_kill)
            // are intentionally absent: they emit, mutate global handler state, are
            // nondeterministic, or can throw.
            | "method_exists"
            | "preg_quote"
            | "preg_grep"
            | "version_compare"
            | "unpack"
            | "http_build_query"
            | "escapeshellarg"
            | "filter_var"
    )
    // Note: json_encode / json_decode / json_validate / json_last_error /
    // json_last_error_msg are intentionally NOT listed here — they read
    // and write the shared `_json_last_error` runtime symbol, so the
    // optimizer must treat them as side-effecting to avoid DCE-ing an
    // encode/decode call right before a json_last_error() observation.
    // Pointer memory helpers such as ptr_read16(), ptr_write16(),
    // ptr_read_string(), and ptr_write_string() are also intentionally absent:
    // raw memory reads/writes and null/length fatals must remain observable.
    // hash() is intentionally NOT listed: an unknown algorithm name throws a
    // catchable \ValueError, so DCE must not eliminate an unused hash() call.
    // hash_hmac() is likewise NOT listed: an unknown algorithm or a non-crypto
    // checksum throws a catchable \ValueError, so it must stay side-effecting.
}
