//! Purpose:
//! Renders OPcache status fragments and PHP literals.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Preload and interned-string keys retain reference ordering and omission rules.

#[allow(unused_imports)]
use super::*;

/// Renders the `opcache_get_status()` body baked with the compile-time cache-enabled
/// gate and the target's directive-derived scalar literals. `web` selects the SAPI-gated
/// enabled constant (CLI disabled → the whole function returns `false`; web enabled →
/// returns the status array).
///
/// `restricted` is the compile-time `opcache.restrict_api` verdict (`restrict_api_denies`). It
/// FORCES the gate constant to `false` — so the function always returns `false`, as reference PHP
/// does when the API is restricted — and fills the gate's warning slot. The array exit is kept
/// either way, which is what preserves the reference `array|false` signature so a caller's
/// `is_array()` guard still narrows (see `RESTRICTED_GET_CONFIGURATION_TEMPLATE` for why that
/// matters). When `restricted` is false the warning slot is removed whole and the rendering is
/// byte-identical to the pre-`restrict_api` template.
///
/// `preload` is the compile-time `opcache.preload` block ([`preload_statistics`]). `Some` inserts
/// the `preload_statistics` key between `opcache_statistics` and `scripts` (the VERIFIED reference
/// position); `None` — the default, and every binary whose cache is disabled — removes the slot
/// whole, leaving the rendering byte-identical to the pre-`preload` template.
pub(super) fn render_get_status_function(
    php_version: PhpVersion,
    web: bool,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
    restricted: bool,
    preload: Option<&PreloadStatistics>,
) -> String {
    let version_id = php_version.version_id();

    // One manifest entry ≈ one cached script ≈ one cache key (reference OPcache keys a
    // script by full path plus optional aliases; the MVP has one key per script).
    let num_cached_scripts = manifest.len() as i64;
    let num_cached_keys = num_cached_scripts;

    // Sum the per-script memory so `used_memory` covers the reported scripts (coherence).
    let scripts_memory_total: i64 = manifest.iter().map(|entry| entry.memory_consumption).sum();

    let memory_total = directive_int(version_id, "opcache.memory_consumption", overrides);
    // baseline + Σ per-script memory, so used_memory >= the scripts map's total.
    let memory_used = STATUS_USED_MEMORY + scripts_memory_total;
    // INVARIANT (class-B): free = total - used - wasted, with wasted = 0.
    let memory_free = memory_total - memory_used;

    let revalidate_freq = directive_int(version_id, "opcache.revalidate_freq", overrides);
    let scripts_map = render_scripts_map_literal(manifest, revalidate_freq, version_id);

    // Reference reports `interned_strings_buffer` (MiB) as a byte count here.
    let interned_buffer_size =
        directive_int(version_id, "opcache.interned_strings_buffer", overrides) * BYTES_PER_MIB;

    // `max_cached_keys` is OPcache's prime-rounded hash capacity derived from
    // `max_accelerated_files` — the exact php-src table, byte-verified boundary by boundary. See
    // `crate::opcache::directives::accel_hash_max_num_entries`. An earlier revision baked 16229
    // for the default and reported the RAW requested count for any `--ini` override, so
    // `--ini opcache.max_accelerated_files=1000` reported 1000 where reference reports 1979.
    let max_cached_keys = accel_hash_max_num_entries(directive_int(
        version_id,
        "opcache.max_accelerated_files",
        overrides,
    ));

    let jit = render_jit_status(version_id, overrides);

    // A restricted API always returns false, so the gate constant is forced regardless of SAPI.
    let enabled = if restricted {
        "false"
    } else {
        render_bool(opcache_cache_enabled_with_overrides(version_id, web, overrides))
    };

    splice_interned_strings_usage(
        &splice_preload_statistics(
            &splice_restrict_api_warning(GET_STATUS_TEMPLATE, restricted, "        "),
            preload,
        ),
        interned_buffer_size,
    )
        .replace("__OPCACHE_STATUS_ENABLED__", enabled)
        .replace("__NUM_CACHED_SCRIPTS__", &num_cached_scripts.to_string())
        .replace("__NUM_CACHED_KEYS__", &num_cached_keys.to_string())
        .replace("__SCRIPTS_MAP__", &scripts_map)
        .replace("__MEM_USED__", &memory_used.to_string())
        .replace("__MEM_FREE__", &memory_free.to_string())
        .replace("__MAX_CACHED_KEYS__", &max_cached_keys.to_string())
        .replace("__JIT_ENABLED__", render_bool(jit.enabled))
        .replace("__JIT_ON__", render_bool(jit.on))
        .replace("__JIT_KIND__", &jit.kind.to_string())
        .replace("__JIT_OPT_LEVEL__", &jit.opt_level.to_string())
        .replace("__JIT_OPT_FLAGS__", &jit.opt_flags.to_string())
        .replace("__JIT_BUFFER_SIZE__", &jit.buffer_size.to_string())
        .replace("__JIT_BUFFER_FREE__", &jit.buffer_free.to_string())
}

/// Renders one directive value as its PHP-literal text.
pub(super) fn render_directive_value(value: &DirectiveValue) -> String {
    match value {
        DirectiveValue::Bool(true) => "true".to_string(),
        DirectiveValue::Bool(false) => "false".to_string(),
        DirectiveValue::Int(int) => int.to_string(),
        // Rust's default `f64` formatting is the shortest round-tripping decimal,
        // which parses back to the same PHP float (e.g. `0.05`, `0.005`).
        DirectiveValue::Float(float) => {
            let rendered = float.to_string();
            if rendered.contains('.') || rendered.contains('e') || rendered.contains('E') {
                rendered
            } else {
                // Keep it a float literal even for whole values (none today, but safe).
                format!("{rendered}.0")
            }
        }
        DirectiveValue::Str(string) => render_php_single_quoted(string),
    }
}

/// Renders a string as a PHP single-quoted literal, escaping `\` and `'`.
pub(super) fn render_php_single_quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

/// Renders the manifest's canonical paths as a flat PHP array literal
/// (`['<path1>', '<path2>']`), single-quote-escaped. Spliced into `opcache_is_script_cached`
/// and `opcache_compile_file` as the `in_array(..., true)` haystack. An empty manifest
/// renders `[]` (valid PHP; membership is always `false`).
pub(super) fn render_manifest_paths_literal(manifest: &[ScriptEntry]) -> String {
    let paths: Vec<String> = manifest.iter().map(|entry| entry.path.clone()).collect();
    render_php_string_list(&paths)
}

/// Renders a list of strings as a flat PHP array literal (`['a', 'b']`), single-quote-escaped.
/// An empty slice renders `[]`.
pub(super) fn render_php_string_list(values: &[String]) -> String {
    let mut literal = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            literal.push_str(", ");
        }
        literal.push_str(&render_php_single_quoted(value));
    }
    literal.push(']');
    literal
}

/// Renders the `$status['preload_statistics'] = [...];` statement in the VERIFIED reference key
/// order, omitting `functions` / `classes` when empty exactly as reference PHP does (see
/// [`PreloadStatistics`] for the pinned observation).
pub(super) fn render_preload_statistics_stmt(stats: &PreloadStatistics) -> String {
    let mut stmt = String::from("    $status['preload_statistics'] = [\n");
    stmt.push_str(&format!(
        "        'memory_consumption' => {},\n",
        stats.memory_consumption
    ));
    if !stats.functions.is_empty() {
        stmt.push_str(&format!(
            "        'functions' => {},\n",
            render_php_string_list(&stats.functions)
        ));
    }
    if !stats.classes.is_empty() {
        stmt.push_str(&format!(
            "        'classes' => {},\n",
            render_php_string_list(&stats.classes)
        ));
    }
    stmt.push_str(&format!(
        "        'scripts' => {},\n",
        render_php_string_list(&stats.scripts)
    ));
    stmt.push_str("    ];");
    stmt
}

/// Fills `GET_STATUS_TEMPLATE`'s `__PRELOAD_STATISTICS__` slot.
///
/// When NOT preloading the placeholder line is removed WHOLE (newline included), so the rendered
/// `opcache_get_status` is BYTE-IDENTICAL to the template as it read before `opcache.preload`
/// existed — the same device `splice_restrict_api_warning` uses to keep the default path a true
/// no-op rather than a whitespace diff.
pub(super) fn splice_preload_statistics(template: &str, stats: Option<&PreloadStatistics>) -> String {
    match stats {
        Some(stats) => {
            template.replace("__PRELOAD_STATISTICS__", &render_preload_statistics_stmt(stats))
        }
        None => template.replace("__PRELOAD_STATISTICS__\n", ""),
    }
}

/// Renders the `interned_strings_usage` sub-array as the four `'key' => value,` lines that sit
/// inside `opcache_get_status()`'s array literal, at the literal's own indent.
///
/// Reference PHP derives all four from pointer arithmetic over the buffer
/// (`buffer_size = end - base`, `used_memory = top - base`, `free_memory = end - top`,
/// `number_of_strings = nNumOfElements`), so `used + free == buffer` EXACTLY and `free > 0`
/// whenever the key is reported at all. Both hold here by construction — see
/// [`render_interned_used_memory`].
///
/// `number_of_strings` stays the synthetic constant: reference PHP reports the count of PHP's OWN
/// interned strings (11679 on this host, IDENTICAL for a 1 MiB and an 8 MiB buffer — VERIFIED), so
/// it is not a function of the buffer size and there is nothing to scale it against.
pub(super) fn render_interned_strings_usage_stmt(buffer_size: i64) -> String {
    let used = render_interned_used_memory(buffer_size);
    let free = buffer_size - used;
    format!(
        "        'interned_strings_usage' => [\n\
         \x20           'buffer_size' => {buffer_size},\n\
         \x20           'used_memory' => {used},\n\
         \x20           'free_memory' => {free},\n\
         \x20           'number_of_strings' => {},\n\
         \x20       ],",
        STATUS_INTERNED_NUMBER_OF_STRINGS,
    )
}

/// Fills `GET_STATUS_TEMPLATE`'s `__INTERNED_STRINGS_USAGE__` slot, OMITTING the key entirely when
/// the interned-strings buffer is zero-sized.
///
/// php-src emits the sub-array only under `if (ZCSG(interned_strings).start &&
/// ZCSG(interned_strings).end)`, and `opcache.interned_strings_buffer=0` allocates neither — so the
/// key is ABSENT, not empty and not zeroed. VERIFIED on reference PHP 8.5.6: with
/// `-d opcache.interned_strings_buffer=0` the top-level key list is
/// `opcache_enabled, cache_full, restart_pending, restart_in_progress, memory_usage,
/// opcache_statistics, scripts, jit` — eight keys, with `interned_strings_usage` gone; with the
/// default `8` the same list carries it as the sixth key, nine in total.
///
/// The placeholder line is removed WHOLE (newline included) in the omitting case, the same device
/// [`splice_preload_statistics`] uses, so nothing else in the rendered function shifts.
pub(super) fn splice_interned_strings_usage(template: &str, buffer_size: i64) -> String {
    if buffer_size <= 0 {
        return template.replace("__INTERNED_STRINGS_USAGE__\n", "");
    }
    template.replace(
        "__INTERNED_STRINGS_USAGE__",
        &render_interned_strings_usage_stmt(buffer_size),
    )
}
