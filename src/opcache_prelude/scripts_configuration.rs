//! Purpose:
//! Renders script maps, configuration, reset, and file-function bodies.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Manifest clocks, invalidation state, and SAPI gates remain coherent.

#[allow(unused_imports)]
use super::*;

/// First `version_id` whose `opcache_get_status()` script entries carry a `revalidate` key.
///
/// php-src added it in 8.3; a `--php-version 8.2` build must not report it. Verified against the
/// official `php:8.2-cli` and `php:8.3-cli` images.
pub(super) const SCRIPTS_REVALIDATE_MIN_VERSION_ID: u32 = 80300;

/// Renders the `opcache_get_status()['scripts']` map: a PHP array keyed by each script's
/// canonical `full_path`, each value the reference-shaped 7-key entry.
///
/// THREE OF THE SEVEN FIELDS ARE CLOCKS, and reference PHP reads them off TWO different clocks —
/// which is the whole point of this docblock, because an earlier revision read all three off the
/// file mtime and got two of them wrong. php-src's `accelerator_get_scripts` emits:
///
/// ```c
/// add_assoc_long(…, "timestamp",           (zend_long) script->timestamp);            // FILE mtime
/// add_assoc_long(…, "last_used_timestamp", script->dynamic_members.last_used);        // REQUEST time
/// add_assoc_long(…, "revalidate",          (zend_long) script->dynamic_members.revalidate);
/// ta = localtime(&script->dynamic_members.last_used);
/// str = asctime(ta);                                                                  // LOCAL time
/// ```
///
/// VERIFIED on reference PHP 8.5.6 (`-d opcache.enable=1 -d opcache.enable_cli=1
/// -d opcache.file_update_protection=0`, printing `time()` and `filemtime(__FILE__)` alongside):
///
/// | field                 | reference                                  | source                    |
/// |-----------------------|--------------------------------------------|---------------------------|
/// | `timestamp`           | 1784992711 = `filemtime()`                 | the file mtime — UNCHANGED|
/// | `last_used_timestamp` | 1784992714 = `time()` (the REQUEST clock)  | identical for EVERY entry |
/// | `revalidate`          | 1784992716 = `last_used_timestamp + 2`     | `+ opcache.revalidate_freq` |
/// | `last_used`           | `Sat Jul 25 15:18:34 2026` under `TZ=UTC`  | `asctime(localtime(…))`   |
///
/// and with `-d opcache.revalidate_freq=60`, `revalidate` = `last_used_timestamp + 60` (1784992775
/// against 1784992715) — confirming the base is the REQUEST clock, not the mtime. The old
/// `mtime + revalidate_freq` formula produced a `revalidate` several seconds IN THE PAST, which no
/// reference run ever reports.
///
/// `last_used` is `asctime`, NOT `date('D M d H:i:s Y')`. `asctime`'s day-of-month field is
/// `%3d` — SPACE-padded, not zero-padded — so a single-digit day renders `Thu Jul  2 13:46:40 2026`
/// with TWO spaces where `date('d')` would render `Jul 02`. VERIFIED against reference PHP with a
/// July-2 timestamp. It is also LOCAL time: the same run reports `Sat Jul 25 15:18:34 2026` under
/// `TZ=UTC` and `Sat Jul 25 11:18:35 2026` under `TZ=America/New_York`. Both facts are why the
/// field is a call to [`OPCACHE_STATE_HELPERS`]' `__elephc_opcache_asctime` rather than a `date()`
/// format string.
///
/// `$__elephc_opcache_start_time` IS THE REQUEST CLOCK. It is `opcache_get_status`'s memoized
/// `static` (see `GET_STATUS_TEMPLATE`), so every entry reports the same instant — exactly as
/// reference PHP does, where one request stamps every entry it touches with one `last_used`. For a
/// CLI process the cache-start instant and the request instant coincide, so one clock is faithful
/// for both; the residual is that a script which sleeps before its FIRST `opcache_get_status()`
/// call reports that later moment rather than process start, because a compiled binary has no
/// request-start hook to read.
///
/// `timestamp` is a call to `__elephc_opcache_script_timestamp` rather than the bare mtime so a
/// FORCE-INVALIDATED entry reports `0`, which is what php-src's `zend_accel_discard_script` writes
/// (see [`INVALIDATE_TEMPLATE`]). With nothing invalidated the call returns the mtime unchanged.
///
/// An empty manifest renders `[]`.
pub(super) fn render_scripts_map_literal(
    manifest: &[ScriptEntry],
    revalidate_freq: i64,
    version_id: u32,
) -> String {
    // `revalidate` DOES NOT EXIST in PHP 8.2's script entries; php-src added it in 8.3.
    // Captured from the official `php:8.X-cli` images on Linux:
    //
    //   8.2  full_path hits last_used last_used_timestamp memory_consumption timestamp
    //   8.3  full_path hits last_used last_used_timestamp memory_consumption revalidate timestamp
    //
    // Emitting it unconditionally made a `--php-version 8.2` build report a key the runtime it
    // targets never has. macOS could not catch this: its shared-memory model hides the `scripts`
    // map entirely, so every local probe saw an empty entry set.
    let revalidate_entry = if version_id >= SCRIPTS_REVALIDATE_MIN_VERSION_ID {
        format!(", 'revalidate' => $__elephc_opcache_start_time + {revalidate_freq}")
    } else {
        String::new()
    };
    let mut literal = String::from("[");
    for entry in manifest {
        let path = render_php_single_quoted(&entry.path);
        literal.push_str(&format!(
            "{path} => ['full_path' => {path}, 'hits' => 0, \
             'memory_consumption' => {mem}, \
             'last_used' => __elephc_opcache_asctime($__elephc_opcache_start_time), \
             'last_used_timestamp' => $__elephc_opcache_start_time, \
             'timestamp' => __elephc_opcache_script_timestamp({path}, {ts})\
             {revalidate_entry}], ",
            mem = entry.memory_consumption,
            ts = entry.timestamp,
        ));
    }
    literal.push(']');
    literal
}

/// Builds the full `opcache_get_configuration()` return array as a PHP array literal
/// for the given compile target.
///
/// Every entry outside the runtime-override scope is a plain literal, exactly as before. Each
/// reporting-only entry is instead a CALL to the typed environment helper carrying its
/// compile-time value as the default (see [`render_directive_value_expr`]) — which is what makes
/// `opcache_get_configuration()['directives']` and `ini_get()` move TOGETHER under an
/// `ELEPHC_INI_*` override, the way `-d` moves both surfaces in reference PHP. With no
/// environment variable set every call returns its `$def`, so the reported array is unchanged.
pub(super) fn render_configuration_literal(php_version: PhpVersion, overrides: &[(String, String)]) -> String {
    let version_id = php_version.version_id();

    let mut directives = String::from("[");
    for (name, value) in effective_opcache_directives(version_id, overrides) {
        directives.push_str(&render_php_single_quoted(name));
        directives.push_str(" => ");
        directives.push_str(&render_directive_value_expr(name, &value));
        directives.push_str(", ");
    }
    directives.push(']');

    format!(
        "['directives' => {directives}, 'version' => ['version' => {version}, \
         'opcache_product_name' => {product}], 'blacklist' => []]",
        version = render_php_single_quoted(opcache_version_string(version_id)),
        product = render_php_single_quoted(OPCACHE_PRODUCT_NAME),
    )
}

/// Renders the `opcache_reset()` body baked with the compile-time cache-enabled state.
/// `web` is the compiler SAPI flag (true for `--web`, whose alias is `--with-web`): a web/FPM
/// SAPI follows
/// `opcache.enable` (enabled), CLI follows `opcache.enable_cli` (disabled), read from
/// the shared directive table so the value stays correct if a default ever flips.
pub(super) fn render_reset_body(php_version: PhpVersion, web: bool, overrides: &[(String, String)]) -> &'static str {
    if opcache_cache_enabled_with_overrides(php_version.version_id(), web, overrides) {
        "true"
    } else {
        "false"
    }
}

/// Renders the `opcache_invalidate()` body baked with the compile-time cache-enabled gate and the
/// manifest paths. The disabled gate short-circuits to `false`; the enabled path returns whether
/// the argument resolves, and a FORCED call on a manifest member also records the discard (see
/// `INVALIDATE_TEMPLATE`).
///
/// It became MANIFEST-DEPENDENT when `$force` grew an observable effect, so it is now a
/// [`bake_manifest`] site like the other three.
pub(super) fn render_invalidate_function(
    php_version: PhpVersion,
    web: bool,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
    strict: bool,
) -> String {
    let enabled = render_bool(opcache_cache_enabled_with_overrides(
        php_version.version_id(),
        web,
        overrides,
    ));
    let template = if strict {
        STRICT_INVALIDATE_TEMPLATE
    } else {
        INVALIDATE_TEMPLATE
    };
    template
        .replace("__OPCACHE_ENABLED__", enabled)
        .replace("__MANIFEST_PATHS__", &render_manifest_paths_literal(manifest))
}

/// Renders the `opcache_is_script_cached()` body baked with the compile-time cache-enabled
/// gate and the manifest paths. Disabled → always `false`; enabled → `realpath`-normalized
/// membership in the baked manifest (see `IS_SCRIPT_CACHED_TEMPLATE`).
pub(super) fn render_is_script_cached_function(
    php_version: PhpVersion,
    web: bool,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
) -> String {
    let enabled = render_bool(opcache_cache_enabled_with_overrides(
        php_version.version_id(),
        web,
        overrides,
    ));
    IS_SCRIPT_CACHED_TEMPLATE
        .replace("__OPCACHE_ENABLED__", enabled)
        .replace("__MANIFEST_PATHS__", &render_manifest_paths_literal(manifest))
}

/// Renders the `opcache_compile_file()` body baked with the compile-time cache-enabled
/// gate and the manifest paths. The disabled gate emits the `Notice:` to STDERR then returns
/// `false`; the enabled path returns `true` for a manifest member (already compiled into the
/// binary) and `false` otherwise (see `COMPILE_FILE_TEMPLATE`).
pub(super) fn render_compile_file_function(
    php_version: PhpVersion,
    web: bool,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
) -> String {
    let enabled = render_bool(opcache_cache_enabled_with_overrides(
        php_version.version_id(),
        web,
        overrides,
    ));
    COMPILE_FILE_TEMPLATE
        .replace("__OPCACHE_ENABLED__", enabled)
        .replace("__MANIFEST_PATHS__", &render_manifest_paths_literal(manifest))
}
