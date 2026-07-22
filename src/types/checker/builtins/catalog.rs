//! Purpose:
//! Defines the canonical set of PHP builtin functions known to the type system.
//! Provides case-insensitive lookup used by name resolution, redeclaration checks, and PHP visibility checks.
//!
//! Called from:
//! - `crate::types::checker::builtins`
//! - `crate::name_resolver`
//!
//! Key details:
//! - `COMPILER_RESIDENT_BUILTIN_FUNCTIONS` lists only language constructs or
//!   dedicated syntax that cannot be represented by an ordinary registry call.
//! - `LANGUAGE_CONSTRUCT_FUNCTIONS` participates in call resolution but stays
//!   hidden from `function_exists()` and first-class callable surfaces.

const COMPILER_RESIDENT_BUILTIN_FUNCTIONS: &[&str] = &[
    // `buffer_new` is a catalog-name-only entry: `buffer_new<T>(len)` is parsed as
    // dedicated syntax (`ExprKind::BufferNew`), so the name never dispatches as a
    // builtin call; it is listed here for `function_exists`, case-insensitive
    // lookup, and redeclaration checks. `buffer_len`/`buffer_free` live in the
    // registry (`src/builtins/pointers/`). Like them, it is an elephc extension
    // hidden by `--strict-php`.
    "buffer_new",
    "die",
    "empty",
    "exit",
    "isset",
    "unset",
];

const LANGUAGE_CONSTRUCT_FUNCTIONS: &[&str] = &["eval"];

/// Campaign builtins still checked/lowered through the legacy per-category checker
/// modules and legacy emitters, not yet migrated into the single-source `builtin!`
/// registry. Recognized here so name resolution, `function_exists`, and metadata treat
/// them as real builtins.
const CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS: &[&str] = &[
    "addcslashes",
    "assert",
    "bindec",
    "cli_set_process_title",
    "connection_aborted",
    "constant",
    "ctype_upper",
    "current",
    "decbin",
    "dechex",
    "decoct",
    "error_get_last",
    "error_log",
    "error_reporting",
    "escapeshellarg",
    "extension_loaded",
    "filter_var",
    "flush",
    "func_get_arg",
    "func_get_args",
    "func_num_args",
    "gc_collect_cycles",
    "gc_disable",
    "gc_enable",
    "gc_enabled",
    "gc_mem_caches",
    "get_cfg_var",
    "get_debug_type",
    "get_defined_constants",
    "getmypid",
    "grapheme_extract",
    "grapheme_str_split",
    "grapheme_stripos",
    "grapheme_strlen",
    "grapheme_strpos",
    "grapheme_strripos",
    "grapheme_strrpos",
    "grapheme_substr",
    "header_remove",
    "headers_sent",
    "hexdec",
    "http_build_query",
    "iconv_mime_decode",
    "iconv_strlen",
    "iconv_strpos",
    "iconv_strrpos",
    "iconv_substr",
    "ignore_user_abort",
    "ini_get",
    "ini_set",
    "is_countable",
    "levenshtein",
    "libxml_clear_errors",
    "libxml_get_errors",
    "libxml_use_internal_errors",
    "mb_convert_case",
    "mb_convert_encoding",
    "mb_detect_encoding",
    "mb_encode_numericentity",
    "mb_internal_encoding",
    "mb_ord",
    "mb_str_split",
    "mb_stripos",
    "mb_stristr",
    "mb_strpos",
    "mb_strripos",
    "mb_strrpos",
    "mb_strstr",
    "mb_strtolower",
    "mb_strtoupper",
    "mb_strwidth",
    "mb_substr",
    "normalizer_is_normalized",
    "normalizer_normalize",
    "octdec",
    "pack",
    "parse_str",
    "parse_url",
    "pcntl_alarm",
    "pcntl_async_signals",
    "pcntl_signal",
    "pcntl_signal_get_handler",
    "posix_kill",
    "preg_grep",
    "preg_last_error",
    "preg_last_error_msg",
    "preg_quote",
    "proc_close",
    "proc_open",
    "random_bytes",
    "reset",
    "restore_error_handler",
    "restore_exception_handler",
    "sapi_windows_cp_conv",
    "sapi_windows_cp_get",
    "sapi_windows_cp_set",
    "sapi_windows_vt100_support",
    "set_error_handler",
    "set_exception_handler",
    "set_time_limit",
    "setlocale",
    "setproctitle",
    "str_getcsv",
    "strcspn",
    "strip_tags",
    "stripcslashes",
    "stripos",
    "strnatcasecmp",
    "strnatcmp",
    "strncasecmp",
    "strncmp",
    "strpbrk",
    "strrchr",
    "strripos",
    "strspn",
    "strtr",
    "substr_compare",
    "substr_count",
    "trigger_deprecation",
    "trigger_error",
    "unpack",
    "version_compare",
];


/// Checks if the exact (lowercase) name is in any callable-resolution builtin list.
/// Does not perform case folding; use `is_supported_builtin_function` for case-insensitive lookup.
fn is_supported_builtin_function_exact(name: &str) -> bool {
    COMPILER_RESIDENT_BUILTIN_FUNCTIONS.contains(&name)
        || LANGUAGE_CONSTRUCT_FUNCTIONS.contains(&name)
        || CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS.contains(&name)
}

/// Returns true when `--strict-php` hides the (lowercase) name from user programs.
///
/// Extension builtins have no PHP equivalent, so strict mode makes them behave
/// as if they did not exist: calls fall through to user-function resolution and
/// the standard undefined-function diagnostics, redeclaration checks accept user
/// functions with these names, and `function_exists()` reports `false`.
/// `internal: true` builtins are never hidden — injected compiler preludes call
/// them and they are already invisible to user programs. `buffer_new` is the one
/// catalog-name-only extension (its call form is dedicated syntax).
pub(crate) fn strict_php_hidden_builtin(canonical: &str) -> bool {
    if !crate::strict_php::is_enabled() {
        return false;
    }
    if canonical == "buffer_new" {
        return true;
    }
    crate::builtins::registry::lookup(canonical)
        .map(|def| def.spec.extension && !def.spec.internal)
        .unwrap_or(false)
}

/// Returns PHP-visible registry names plus compiler-resident call-like names,
/// without applying the strict-PHP filter.
///
/// This is the raw catalog snapshot for metadata consumers (parity gates, docs
/// exporters) that memoize the result and must be independent of the thread's
/// strict-mode state. Compilation surfaces use `supported_builtin_function_names`.
pub(crate) fn all_supported_builtin_function_names() -> Vec<&'static str> {
    let mut result: Vec<&'static str> = COMPILER_RESIDENT_BUILTIN_FUNCTIONS.to_vec();
    for name in crate::builtins::registry::names() {
        let def = match crate::builtins::registry::lookup(name) {
            Some(d) => d,
            None => continue,
        };
        if def.spec.internal {
            continue;
        }
        // De-duplicate any name also represented by dedicated compiler syntax.
        let lower = name.to_ascii_lowercase();
        if !COMPILER_RESIDENT_BUILTIN_FUNCTIONS.contains(&lower.as_str()) {
            result.push(def.name);
        }
    }
    result.extend_from_slice(CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS);
    result
}

/// Returns the union of PHP-visible registry and compiler-resident builtin names.
///
/// Registry entries flagged as `internal` are excluded, mirroring the semantics
/// of `is_php_visible_builtin_function`. Names present in both sources appear
/// exactly once. Under `--strict-php`, extension builtins are excluded entirely.
pub(crate) fn supported_builtin_function_names() -> Vec<&'static str> {
    all_supported_builtin_function_names()
        .into_iter()
        .filter(|name| !strict_php_hidden_builtin(&name.to_ascii_lowercase()))
        .collect()
}

/// Converts a function name to lowercase and returns it if it is a supported builtin.
///
/// Returns `None` if the name is neither registry-backed nor compiler-resident,
/// or if `--strict-php` hides it. Implements PHP's case-insensitive builtin lookup.
pub(crate) fn canonical_builtin_function_name(name: &str) -> Option<String> {
    let canonical = name.to_ascii_lowercase();
    if strict_php_hidden_builtin(&canonical) {
        return None;
    }
    if is_supported_builtin_function_exact(&canonical)
        || crate::builtins::registry::is_supported(&canonical)
    {
        Some(canonical)
    } else {
        None
    }
}

/// Returns whether a recognized builtin may be (re)declared by user or prelude code
/// without tripping the redeclare-builtin guard.
///
/// `trigger_error` is registered for recognition (`function_exists`, name resolution) but
/// has no EIR backend lowering — the real implementation is supplied by the web prelude's
/// web-SAPI stderr renderer (or by user code). Because the prelude declares
/// `function trigger_error(...)`, the redeclaration check must treat it as overridable, or
/// every `--web` compile fails with "Cannot redeclare built-in function: trigger_error".
pub(crate) fn is_prelude_overridable_builtin(canonical: &str) -> bool {
    matches!(canonical, "trigger_error")
}

/// Returns true only for PHP-visible builtin functions (non-internal builtins).
///
/// Checks both compiler-resident names and the builtin registry. Registry entries
/// flagged as `internal` are excluded from the PHP-visible set, and `--strict-php`
/// additionally excludes extension builtins.
pub(crate) fn is_php_visible_builtin_function(name: &str) -> bool {
    let canonical = name.to_ascii_lowercase();
    if strict_php_hidden_builtin(&canonical) {
        return false;
    }
    COMPILER_RESIDENT_BUILTIN_FUNCTIONS.contains(&canonical.as_str())
        || CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS.contains(&canonical.as_str())
        || crate::builtins::registry::lookup(&canonical)
            .map(|def| !def.spec.internal)
            .unwrap_or(false)
}

/// Returns `true` if the name is a supported builtin function (case-insensitive).
/// Delegates to `canonical_builtin_function_name` and checks for `Some`.
pub(crate) fn is_supported_builtin_function(name: &str) -> bool {
    canonical_builtin_function_name(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin;

    // Register a PHP-visible (non-internal) probe to exercise the catalog API.
    // This verifies that `supported_builtin_function_names` and the catalog
    // lookup functions include registry entries with `internal: false`.
    builtin! {
        name: "__catalog_probe_visible",
        area: Types,
        params: [x: Int],
        returns: Bool,
        semantics: crate::builtins::semantics::test_probe_semantics(),
        summary: "catalog probe for PHP-visibility test",
        internal: false,
    }

    /// Verifies that a `builtin!`-registered probe with `internal: false` is reported
    /// as supported by the catalog's `is_supported_builtin_function` and
    /// `canonical_builtin_function_name` surfaces.
    #[test]
    fn catalog_reports_registered_visible_probe_as_supported() {
        assert!(
            is_supported_builtin_function("__catalog_probe_visible"),
            "catalog must report a non-internal registered builtin as supported"
        );
        let canonical = canonical_builtin_function_name("__catalog_probe_visible");
        assert_eq!(
            canonical,
            Some("__catalog_probe_visible".to_string()),
            "catalog must canonicalize a non-internal registered builtin"
        );
    }

    /// Verifies that a non-internal registered probe appears in `supported_builtin_function_names`.
    #[test]
    fn supported_builtin_function_names_includes_registered_visible_probe() {
        let names = supported_builtin_function_names();
        assert!(
            names.contains(&"__catalog_probe_visible"),
            "supported_builtin_function_names must include non-internal registry entries"
        );
    }

    /// Verifies strict mode hides extension builtins from every catalog surface:
    /// canonical lookup, PHP-visibility, the supported-name set, and the
    /// `buffer_new` catalog-name-only entry. Strict state is thread-local (and
    /// guard-restored on panic), so this cannot affect parallel tests.
    #[test]
    fn strict_mode_hides_extension_builtins_from_catalog() {
        let _guard = crate::strict_php::scoped_enable();
        assert!(
            canonical_builtin_function_name("ptr_get").is_none(),
            "strict must hide ptr_get"
        );
        assert!(
            !is_php_visible_builtin_function("ptr_get"),
            "strict must hide ptr_get from PHP visibility"
        );
        assert!(
            canonical_builtin_function_name("buffer_new").is_none(),
            "strict must hide buffer_new"
        );
        assert!(
            !is_php_visible_builtin_function("buffer_new"),
            "strict must hide buffer_new from PHP visibility"
        );
        let names = supported_builtin_function_names();
        assert!(
            !names.contains(&"ptr_get") && !names.contains(&"buffer_new"),
            "strict must drop extension names from the supported set"
        );
    }

    /// Verifies strict mode keeps genuine PHP builtins and internal prelude
    /// aliases resolvable: hiding either would break normal programs or
    /// compiler-injected prelude code.
    #[test]
    fn strict_mode_keeps_php_builtins_and_internal_aliases() {
        let _guard = crate::strict_php::scoped_enable();
        assert_eq!(
            canonical_builtin_function_name("strlen"),
            Some("strlen".to_string())
        );
        assert_eq!(
            canonical_builtin_function_name("is_real"),
            Some("is_real".to_string()),
            "is_real is treated as PHP for strict purposes"
        );
        assert!(
            canonical_builtin_function_name("__elephc_ptr_read_string").is_some(),
            "internal prelude aliases must stay resolvable in strict mode"
        );
    }

    /// Verifies the unfiltered name set ignores strict mode entirely: metadata
    /// consumers (parity gates, docs exporters) memoize this snapshot and must
    /// never observe a strict-filtered view.
    #[test]
    fn unfiltered_name_set_ignores_strict_mode() {
        let _guard = crate::strict_php::scoped_enable();
        let names = all_supported_builtin_function_names();
        assert!(names.contains(&"ptr_get"));
        assert!(names.contains(&"buffer_new"));
        assert!(names.contains(&"strlen"));
    }

    /// Verifies extension builtins remain fully visible without strict mode, so
    /// the filter cannot regress the default compilation mode.
    #[test]
    fn non_strict_keeps_extension_builtins_visible() {
        assert!(canonical_builtin_function_name("ptr_get").is_some());
        assert!(is_php_visible_builtin_function("ptr_get"));
        assert!(canonical_builtin_function_name("buffer_new").is_some());
        assert!(supported_builtin_function_names().contains(&"buffer_new"));
    }
}
