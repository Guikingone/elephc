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
//! - CATALOG MEMBERSHIP IS A `function_exists()` PROMISE THE BACKEND MAY NOT KEEP.
//!   `is_php_visible_builtin_function` is what `crate::optimize::function_existence` folds
//!   `function_exists()` on, and it accepts every `CAMPAIGN_LEGACY_BUILTIN_FUNCTIONS` entry
//!   regardless of whether any EIR lowering exists. A measured sweep (compiling one probe per
//!   name against the release compiler) confirmed AT LEAST 73 of the 125 legacy names abort with
//!   "unsupported EIR backend feature: builtin call <name>" — among them `stripos`, `strtr`,
//!   `parse_url`, `parse_str`, `preg_quote`, `strip_tags`, `substr_compare`, `strncasecmp`,
//!   `random_bytes`, `is_countable`, `pack`/`unpack`, `http_build_query`, `set_error_handler`,
//!   `reset`/`current`/`key`, and the whole `mb_*`/`grapheme_*`/`iconv_*` families. See
//!   `is_prelude_overridable_builtin` below for the one case where this is deliberate and
//!   compensated (`trigger_error`, implemented by the web prelude in PHP).
//!   Two consequences bind anyone extending this file:
//!   1. Adding a name here to clear an "Undefined function" diagnostic RELOCATES the failure
//!      into the backend rather than fixing it, and `--web` cannot see the relocation because
//!      that run aborts at the checker. A name belongs here only once its lowering exists.
//!   2. A `!function_exists('x')` polyfill/capability guard in user code reads `true` for all 73
//!      and therefore does NOT fire, so PHP-level fallbacks that would have worked stay dead.
//!   The `catalog_gap_tripwires` tests below pin the specific names a current campaign was
//!   tempted to add; they are guard rails, not a claim that the rest of the list is sound.

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
    "current",
    "debug_backtrace",
    "decbin",
    "dechex",
    "decoct",
    "end",
    "error_get_last",
    "error_log",
    "error_reporting",
    "escapeshellarg",
    "extract",
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
    "get_class_methods",
    "get_debug_type",
    "get_defined_constants",
    "getmypid",
    // The `grapheme_*` family is deliberately UNCLAIMED, for the same reason as the `mb_*` names
    // above: elephc has no implementation (correct grapheme clustering needs Unicode tables it
    // does not ship), and claiming the names made `function_exists('grapheme_strlen')` fold TRUE
    // so `symfony/polyfill-intl-grapheme`'s own guard DROPPED its userland definition — leaving
    // codegen to answer `unsupported EIR backend feature`. Unclaimed, the vendored polyfill
    // supplies real, upstream-maintained implementations.
    "header_remove",
    "headers_sent",
    "hexdec",
    "highlight_file",
    "http_build_query",
    "iconv",
    "iconv_mime_decode",
    "iconv_strlen",
    "iconv_strpos",
    "iconv_strrpos",
    "iconv_substr",
    "ignore_user_abort",
    "ini_get",
    "ini_set",
    "is_countable",
    "key",
    "next",
    "levenshtein",
    "libxml_clear_errors",
    "libxml_get_errors",
    "libxml_use_internal_errors",
    // Only `mb_convert_encoding` is claimed: it is the one member of the family elephc actually
    // IMPLEMENTS (`crate::mb_convert_encoding_prelude`). The rest were catalog-visible with no EIR
    // lowering at all, which is worse than being absent: `function_exists('mb_strripos')` folded to
    // TRUE, `symfony/polyfill-mbstring`'s own `if (!function_exists(...))` guard therefore dropped
    // its userland definition, and codegen then answered
    // `unsupported EIR backend feature: builtin call mb_strripos`. Leaving the names UNCLAIMED lets
    // the vendored polyfill supply real, upstream-maintained PHP implementations.
    "mb_convert_encoding",
    // `normalizer_*` is unclaimed for the same reason as `grapheme_*`:
    // `symfony/polyfill-intl-normalizer` ships the real implementation and its
    // `if (!function_exists(...))` guard must see the name as ABSENT to install it.
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
    "prev",
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
    // `mb_convert_encoding` keeps its catalog membership — `function_exists()` and the mbstring
    // polyfill guards must still see it as a real PHP function — while its BODY ships as an
    // elephc-PHP prelude (`crate::mb_convert_encoding_prelude`), because the `mb_*` family has no
    // EIR lowering at all and a call would otherwise die with
    // `unsupported EIR backend feature: builtin call mb_convert_encoding`.
    // The `crate::string_compat_prelude` names keep their catalog membership —
    // `function_exists('strncmp')` must still report a real PHP function — while their BODIES ship
    // as elephc-PHP preludes, because they had no EIR lowering at all and a call would otherwise
    // die with `unsupported EIR backend feature: builtin call <name>`.
    matches!(
        canonical,
        "trigger_error" | "mb_convert_encoding" | "get_defined_constants"
    ) || crate::string_compat_prelude::supplies(canonical)
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

    /// Guard rails for the four core-PHP names a `--web` Symfony scan repeatedly surfaces as
    /// "Undefined function" (`debug_backtrace`, `next`, `highlight_file`, `extract`).
    ///
    /// Catalog membership is only a `function_exists()` promise (see the module preamble): it
    /// makes the checker stop reporting the name while doing nothing about the missing EIR
    /// lowering, so adding one of these to clear the diagnostic simply moves the failure into the
    /// backend where the `--web` run — which aborts at the checker — cannot observe it. Each
    /// assertion below therefore fails LOUDLY the moment a name is registered, and the message
    /// names the concrete prerequisite that must land first.
    /// `crate::types::checker::builtins::late_bound`'s module doc carries the full evidence.
    mod catalog_gap_tripwires {
        use super::*;

        /// `debug_backtrace()` stays visible with Composer file attribution and the fiber-aware
        /// activation-record shadow stack.
        #[test]
        fn debug_backtrace_is_visible_with_real_frames() {
            assert!(
                is_supported_builtin_function("debug_backtrace"),
                "debug_backtrace() must stay visible with its declaration-file map and shadow stack"
            );
        }

        /// The complete PHP array-cursor family must remain catalog-visible together.
        #[test]
        fn array_cursor_family_is_catalog_visible() {
            for sibling in ["reset", "current", "key", "next", "prev", "end"] {
                assert!(
                    is_supported_builtin_function(sibling),
                    "{sibling}() must stay visible with the shared cursor runtime"
                );
            }
        }

        /// `highlight_file()` stays visible with the runtime source highlighter.
        #[test]
        fn highlight_file_is_visible_with_runtime_highlighter() {
            assert!(
                is_supported_builtin_function("highlight_file"),
                "highlight_file() must stay visible with its runtime source highlighter"
            );
        }

        /// `extract()` stays catalog-visible with the materialized dynamic-scope runtime.
        #[test]
        fn extract_is_visible_with_materialized_dynamic_scope() {
            assert!(
                is_supported_builtin_function("extract"),
                "extract() must stay visible with its materialized dynamic-scope runtime"
            );
        }

        /// `proc_open` is the one legacy catalog name whose CALL the checker itself rejects (no
        /// per-area `check_builtin` arm), while `function_exists('proc_open')` still folds true.
        /// It stays registered because `by_ref_outputs` uses that registration to initialize the
        /// `&$pipes` out-parameter its only call site reads; dropping it trades one diagnostic
        /// for an "Undefined variable: $pipes".
        #[test]
        fn proc_open_stays_registered_for_its_by_ref_out_parameter() {
            assert!(
                is_php_visible_builtin_function("proc_open"),
                "proc_open must stay catalog-visible: Console\\Terminal::readFromProcess() relies \
                 on its by-ref $pipes out-parameter knowledge; only a real implementation clears \
                 that call site without surfacing a new undefined-variable error"
            );
        }
    }
}
