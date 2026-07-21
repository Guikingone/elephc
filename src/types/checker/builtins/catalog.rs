//! Purpose:
//! Defines the canonical set of PHP builtin functions known to the type system.
//! Provides case-insensitive lookup used by name resolution, redeclaration checks, and PHP visibility checks.
//!
//! Called from:
//! - `crate::types::checker::builtins`
//! - `crate::name_resolver`
//!
//! Key details:
//! - `SUPPORTED_BUILTIN_FUNCTIONS` is the source of truth for PHP-visible builtin names.
//! - `INTERNAL_BUILTIN_FUNCTIONS` is now an empty placeholder; internal builtins are
//!   registered via `internal: true` in `src/builtins/` and recognized through the registry.
//! - `LANGUAGE_CONSTRUCT_FUNCTIONS` participates in call resolution but stays
//!   hidden from `function_exists()` and first-class callable surfaces.

const SUPPORTED_BUILTIN_FUNCTIONS: &[&str] = &[
    "__elephc_gmmktime_raw",
    "__elephc_mktime_raw",
    "__elephc_strtotime_raw",
    "abs",
    "acos",
    "addcslashes",
    "addslashes",
    "array_chunk",
    "array_column",
    "array_combine",
    "array_diff",
    "array_diff_key",
    "array_fill",
    "array_fill_keys",
    "array_filter",
    "array_flip",
    "array_intersect",
    "array_intersect_key",
    "array_is_list",
    "array_key_exists",
    "array_key_first",
    "array_key_last",
    "array_keys",
    "array_map",
    "array_merge",
    "array_pad",
    "array_pop",
    "array_product",
    "array_push",
    "array_rand",
    "array_reduce",
    "array_replace",
    "array_replace_recursive",
    "array_reverse",
    "array_search",
    "array_shift",
    "array_slice",
    "array_splice",
    "array_sum",
    "array_unique",
    "array_unshift",
    "array_values",
    "array_walk",
    "array_walk_recursive",
    "arsort",
    "asin",
    "asort",
    "assert",
    "atan",
    "atan2",
    "base64_decode",
    "base64_encode",
    "basename",
    "bin2hex",
    "bindec",
    "boolval",
    "buffer_free",
    "buffer_len",
    "buffer_new",
    "call_user_func",
    "call_user_func_array",
    "ceil",
    "chdir",
    "checkdate",
    "chgrp",
    "chmod",
    "chop",
    "chown",
    "chr",
    "clamp",
    "class_alias",
    "class_attribute_args",
    "class_attribute_names",
    "class_exists",
    "class_get_attributes",
    "class_implements",
    "class_parents",
    "class_uses",
    "clearstatcache",
    "cli_set_process_title",
    "closedir",
    "connection_aborted",
    "constant",
    "copy",
    "cos",
    "cosh",
    "count",
    "crc32",
    "ctype_alnum",
    "ctype_alpha",
    "ctype_digit",
    "ctype_space",
    "ctype_upper",
    "current",
    "date",
    "date_default_timezone_get",
    "date_default_timezone_set",
    "decbin",
    "dechex",
    "decoct",
    "define",
    "defined",
    "deg2rad",
    "die",
    "dirname",
    "disk_free_space",
    "disk_total_space",
    "empty",
    "end",
    "enum_exists",
    "error_get_last",
    "error_log",
    "error_reporting",
    "escapeshellarg",
    "exec",
    "exit",
    "exp",
    "explode",
    "extension_loaded",
    "fclose",
    "fdatasync",
    "fdiv",
    "feof",
    "fflush",
    "fgetc",
    "fgetcsv",
    "fgets",
    "file",
    "file_exists",
    "file_get_contents",
    "file_put_contents",
    "fileatime",
    "filectime",
    "filegroup",
    "fileinode",
    "filemtime",
    "fileowner",
    "fileperms",
    "filesize",
    "filetype",
    "filter_var",
    "floatval",
    "flock",
    "floor",
    "flush",
    "fmod",
    "fnmatch",
    "fopen",
    "fpassthru",
    "fprintf",
    "fputcsv",
    "fread",
    "fscanf",
    "fseek",
    "fsockopen",
    "fstat",
    "fsync",
    "ftell",
    "ftruncate",
    "func_get_arg",
    "func_get_args",
    "func_num_args",
    "function_exists",
    "fwrite",
    "gc_collect_cycles",
    "gc_disable",
    "gc_enable",
    "gc_enabled",
    "gc_mem_caches",
    "get_cfg_var",
    "get_class",
    "get_class_methods",
    "get_debug_type",
    "get_declared_classes",
    "get_declared_interfaces",
    "get_declared_traits",
    "get_defined_constants",
    "get_parent_class",
    "get_resource_id",
    "get_resource_type",
    "getcwd",
    "getdate",
    "getenv",
    "gethostbyaddr",
    "gethostbyname",
    "gethostname",
    "getmypid",
    "getprotobyname",
    "getprotobynumber",
    "getservbyname",
    "getservbyport",
    "gettype",
    "glob",
    "gmdate",
    "gmmktime",
    "grapheme_extract",
    "grapheme_str_split",
    "grapheme_stripos",
    "grapheme_strlen",
    "grapheme_strpos",
    "grapheme_strrev",
    "grapheme_strripos",
    "grapheme_strrpos",
    "grapheme_substr",
    "gzcompress",
    "gzdeflate",
    "gzinflate",
    "gzuncompress",
    "hash",
    "hash_algos",
    "hash_copy",
    "hash_equals",
    "hash_file",
    "hash_final",
    "hash_hmac",
    "hash_init",
    "hash_update",
    "header",
    "header_remove",
    "headers_sent",
    "hex2bin",
    "hexdec",
    "hrtime",
    "html_entity_decode",
    "htmlentities",
    "htmlspecialchars",
    "http_build_query",
    "http_response_code",
    "hypot",
    "iconv",
    "iconv_mime_decode",
    "iconv_strlen",
    "iconv_strpos",
    "iconv_strrpos",
    "iconv_substr",
    "ignore_user_abort",
    "implode",
    "in_array",
    "inet_ntop",
    "inet_pton",
    "ini_get",
    "ini_set",
    "intdiv",
    "interface_exists",
    "intval",
    "ip2long",
    "is_a",
    "is_array",
    "is_bool",
    "is_callable",
    "is_countable",
    "is_dir",
    "is_double",
    "is_executable",
    "is_file",
    "is_finite",
    "is_float",
    "is_infinite",
    "is_int",
    "is_integer",
    "is_iterable",
    "is_link",
    "is_long",
    "is_nan",
    "is_null",
    "is_numeric",
    "is_object",
    "is_readable",
    "is_real",
    "is_resource",
    "is_scalar",
    "is_string",
    "is_subclass_of",
    "is_writable",
    "is_writeable",
    "isset",
    "iterator_apply",
    "iterator_count",
    "iterator_to_array",
    "json_decode",
    "json_encode",
    "json_last_error",
    "json_last_error_msg",
    "json_validate",
    "key",
    "krsort",
    "ksort",
    "lcfirst",
    "lchgrp",
    "lchown",
    "levenshtein",
    "libxml_clear_errors",
    "libxml_get_errors",
    "libxml_use_internal_errors",
    "link",
    "linkinfo",
    "localtime",
    "log",
    "log10",
    "log2",
    "long2ip",
    "lstat",
    "ltrim",
    "max",
    "mb_convert_case",
    "mb_convert_encoding",
    "mb_detect_encoding",
    "mb_encode_numericentity",
    "mb_internal_encoding",
    "mb_ord",
    "mb_str_split",
    "mb_stripos",
    "mb_stristr",
    "mb_strlen",
    "mb_strpos",
    "mb_strripos",
    "mb_strrpos",
    "mb_strstr",
    "mb_strtolower",
    "mb_strtoupper",
    "mb_strwidth",
    "mb_substr",
    "md5",
    "method_exists",
    "microtime",
    "min",
    "mkdir",
    "mktime",
    "mt_rand",
    "natcasesort",
    "natsort",
    "nl2br",
    "normalizer_is_normalized",
    "normalizer_normalize",
    "number_format",
    "ob_end_clean",
    "ob_end_flush",
    "ob_get_clean",
    "ob_get_contents",
    "ob_get_level",
    "ob_get_status",
    "ob_start",
    "octdec",
    "opendir",
    "ord",
    "pack",
    "parse_str",
    "parse_url",
    "passthru",
    "pathinfo",
    "pclose",
    "pcntl_alarm",
    "pcntl_async_signals",
    "pcntl_signal",
    "pcntl_signal_get_handler",
    "pfsockopen",
    "php_uname",
    "phpversion",
    "pi",
    "popen",
    "posix_kill",
    "pow",
    "preg_grep",
    "preg_last_error",
    "preg_last_error_msg",
    "preg_match",
    "preg_match_all",
    "preg_quote",
    "preg_replace",
    "preg_replace_callback",
    "preg_split",
    "print_r",
    "printf",
    "proc_close",
    "proc_open",
    "property_exists",
    "ptr",
    "ptr_get",
    "ptr_is_null",
    "ptr_null",
    "ptr_offset",
    "ptr_read16",
    "ptr_read32",
    "ptr_read8",
    "ptr_read_string",
    "ptr_set",
    "ptr_sizeof",
    "ptr_write16",
    "ptr_write32",
    "ptr_write8",
    "ptr_write_string",
    "putenv",
    "rad2deg",
    "rand",
    "random_bytes",
    "random_int",
    "range",
    "rawurldecode",
    "rawurlencode",
    "readdir",
    "readfile",
    "readline",
    "readlink",
    "realpath",
    "realpath_cache_get",
    "realpath_cache_size",
    "rename",
    "reset",
    "restore_error_handler",
    "restore_exception_handler",
    "rewind",
    "rewinddir",
    "rmdir",
    "round",
    "rsort",
    "rtrim",
    "sapi_windows_cp_conv",
    "sapi_windows_cp_get",
    "sapi_windows_cp_set",
    "sapi_windows_vt100_support",
    "scandir",
    "serialize",
    "set_error_handler",
    "set_exception_handler",
    "set_time_limit",
    "setlocale",
    "setproctitle",
    "settype",
    "sha1",
    "shell_exec",
    "shuffle",
    "sin",
    "sinh",
    "sleep",
    "sort",
    "spl_autoload",
    "spl_autoload_call",
    "spl_autoload_extensions",
    "spl_autoload_functions",
    "spl_autoload_register",
    "spl_autoload_unregister",
    "spl_classes",
    "spl_object_hash",
    "spl_object_id",
    "sprintf",
    "sqrt",
    "sscanf",
    "stat",
    "str_contains",
    "str_ends_with",
    "str_getcsv",
    "str_ireplace",
    "str_pad",
    "str_repeat",
    "str_replace",
    "str_split",
    "str_starts_with",
    "strcasecmp",
    "strcmp",
    "strcspn",
    "stream_bucket_append",
    "stream_bucket_make_writeable",
    "stream_bucket_new",
    "stream_bucket_prepend",
    "stream_context_create",
    "stream_context_get_default",
    "stream_context_get_options",
    "stream_context_get_params",
    "stream_context_set_default",
    "stream_context_set_option",
    "stream_context_set_params",
    "stream_copy_to_stream",
    "stream_filter_append",
    "stream_filter_prepend",
    "stream_filter_register",
    "stream_filter_remove",
    "stream_get_contents",
    "stream_get_filters",
    "stream_get_line",
    "stream_get_meta_data",
    "stream_get_transports",
    "stream_get_wrappers",
    "stream_is_local",
    "stream_isatty",
    "stream_resolve_include_path",
    "stream_select",
    "stream_set_blocking",
    "stream_set_chunk_size",
    "stream_set_read_buffer",
    "stream_set_timeout",
    "stream_set_write_buffer",
    "stream_socket_accept",
    "stream_socket_client",
    "stream_socket_enable_crypto",
    "stream_socket_get_name",
    "stream_socket_pair",
    "stream_socket_recvfrom",
    "stream_socket_sendto",
    "stream_socket_server",
    "stream_socket_shutdown",
    "stream_supports_lock",
    "stream_wrapper_register",
    "stream_wrapper_restore",
    "stream_wrapper_unregister",
    "strip_tags",
    "stripcslashes",
    "stripos",
    "stripslashes",
    "strlen",
    "strnatcasecmp",
    "strnatcmp",
    "strncasecmp",
    "strncmp",
    "strpbrk",
    "strpos",
    "strrchr",
    "strrev",
    "strripos",
    "strrpos",
    "strspn",
    "strstr",
    "strtolower",
    "strtotime",
    "strtoupper",
    "strtr",
    "strval",
    "substr",
    "substr_compare",
    "substr_count",
    "substr_replace",
    "symlink",
    "sys_get_temp_dir",
    "system",
    "tan",
    "tanh",
    "tempnam",
    "time",
    "tmpfile",
    "touch",
    "trait_exists",
    "trigger_deprecation",
    "trigger_error",
    "trim",
    "uasort",
    "ucfirst",
    "ucwords",
    "uksort",
    "umask",
    "unlink",
    "unpack",
    "unserialize",
    "unset",
    "urldecode",
    "urlencode",
    "usleep",
    "usort",
    "var_dump",
    "version_compare",
    "vfprintf",
    "vprintf",
    "vsprintf",
    "wordwrap",
];

// All former entries migrated to `src/builtins/io/__elephc_phar_*.rs` with `internal: true`
// (io batch C2). Name recognition now flows through `registry::is_supported` inside
// `canonical_builtin_function_name`. The slice is kept as an empty placeholder so that
// `is_supported_builtin_function_exact` compiles unchanged.
const INTERNAL_BUILTIN_FUNCTIONS: &[&str] = &[];

const LANGUAGE_CONSTRUCT_FUNCTIONS: &[&str] = &["eval"];

/// Checks if the exact (lowercase) name is in any callable-resolution builtin list.
/// Does not perform case folding; use `is_supported_builtin_function` for case-insensitive lookup.
fn is_supported_builtin_function_exact(name: &str) -> bool {
    SUPPORTED_BUILTIN_FUNCTIONS.contains(&name)
        || INTERNAL_BUILTIN_FUNCTIONS.contains(&name)
        || LANGUAGE_CONSTRUCT_FUNCTIONS.contains(&name)
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

/// Returns the union of PHP-visible builtin names from the legacy static list
/// and the builtin registry, WITHOUT the strict-PHP filter.
///
/// This is the raw catalog snapshot for metadata consumers (parity gates, docs
/// exporters) that memoize the result and must be independent of the thread's
/// strict-mode state. Compilation surfaces use `supported_builtin_function_names`.
pub(crate) fn all_supported_builtin_function_names() -> Vec<&'static str> {
    let mut result: Vec<&'static str> = SUPPORTED_BUILTIN_FUNCTIONS.to_vec();
    for name in crate::builtins::registry::names() {
        let def = match crate::builtins::registry::lookup(name) {
            Some(d) => d,
            None => continue,
        };
        if def.spec.internal {
            continue;
        }
        // De-duplicate: skip names already present in the legacy list.
        let lower = name.to_ascii_lowercase();
        if !SUPPORTED_BUILTIN_FUNCTIONS.contains(&lower.as_str()) {
            result.push(def.name);
        }
    }
    result
}

/// Returns the union of PHP-visible supported builtin function names from the
/// legacy static list and the builtin registry.
///
/// Registry entries flagged as `internal` are excluded, mirroring the semantics
/// of `is_php_visible_builtin_function`. Names present in both sources appear
/// exactly once. With an empty registry this returns the legacy list unchanged,
/// so behavior is preserved while the registry is empty. Under `--strict-php`,
/// extension builtins are excluded entirely.
pub(crate) fn supported_builtin_function_names() -> Vec<&'static str> {
    all_supported_builtin_function_names()
        .into_iter()
        .filter(|name| !strict_php_hidden_builtin(&name.to_ascii_lowercase()))
        .collect()
}

/// Converts a function name to lowercase and returns it if it is a supported builtin.
///
/// Returns `None` if the name is not in either the legacy catalog or the builtin
/// registry, or if `--strict-php` hides it (extension builtins). Implements PHP's
/// case-insensitive builtin lookup. The legacy static list is consulted first;
/// the registry is the fallback.
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

/// Returns true only for PHP-visible builtin functions (non-internal builtins).
///
/// Checks both the legacy static list and the builtin registry. Registry entries
/// flagged as `internal` are excluded from the PHP-visible set, and `--strict-php`
/// additionally excludes extension builtins.
pub(crate) fn is_php_visible_builtin_function(name: &str) -> bool {
    let canonical = name.to_ascii_lowercase();
    if strict_php_hidden_builtin(&canonical) {
        return false;
    }
    SUPPORTED_BUILTIN_FUNCTIONS.contains(&canonical.as_str())
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

    /// No-op lowering hook for test probe; does nothing and succeeds.
    fn noop_lower(
        _c: &mut crate::codegen::context::FunctionContext,
        _i: &crate::ir::Instruction,
    ) -> Result<(), crate::codegen::CodegenIrError> {
        Ok(())
    }

    // Register a PHP-visible (non-internal) probe to exercise the catalog API.
    // This verifies that `supported_builtin_function_names` and the catalog
    // lookup functions include registry entries with `internal: false`.
    builtin! {
        name: "__catalog_probe_visible",
        area: Internal,
        params: [x: Int],
        returns: Bool,
        lower: noop_lower,
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
