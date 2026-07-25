//! Purpose:
//! Injects the OPcache standard-library functions written in elephc-PHP:
//! `opcache_get_configuration()` (returns the compile-time configuration array
//! `['directives' => [...], 'version' => [...], 'blacklist' => []]`),
//! `opcache_reset()` (returns the compile-time cache-enabled boolean), and
//! `opcache_get_status()` (returns `false` when the cache is disabled, or the runtime
//! status array when enabled). All are baked from the version-keyed matrix in
//! `crate::opcache`, rendered to PHP source and substituted into tiny per-function
//! templates.
//!
//! Called from:
//! - `crate::pipeline::compile()` via `inject_if_used`, after include/PDO/tz/list-id
//!   injection and before name resolution, so a user `opcache_get_configuration()` /
//!   `opcache_reset()` call resolves to the injected function through the normal
//!   pipeline (function declaration + literal) with no dedicated codegen or runtime
//!   helper.
//! - `crate::pipeline::compile()` again via `bake_manifest`, immediately AFTER
//!   `autoload::run`, which re-renders the three manifest-dependent functions against the
//!   COMPLETE script manifest. Injection and manifest baking are split because the
//!   autoloaded file set does not exist until after name resolution, while the
//!   declarations must exist BEFORE it — see `bake_manifest` for the full argument.
//!
//! Key details:
//! - Modeled on `list_id_prelude`/`tz_prelude`: the compiler bakes a
//!   `__OPCACHE_CONFIGURATION__` array literal from `opcache_directives(version_id)`
//!   and a `__OPCACHE_RESET_ENABLED__` boolean from `opcache_cache_enabled(...)`,
//!   substitutes them, and lets the ordinary literal lowering do the rest — no new
//!   `RuntimeFnId`. Neither function is a checker catalog builtin: registering one
//!   would trip the "Cannot redeclare built-in function" guard against this prelude
//!   declaration. Being a real declared function is exactly what makes
//!   `function_exists('opcache_reset')` report `true` (see
//!   `codegen::lower_inst::builtins::lower_function_exists`).
//! - Pay-for-use *per function*: each function is injected only when `detect` finds a
//!   call or a matching string literal (covering `function_exists`/callable forms),
//!   and never when the program already declares its own function of that name (so a
//!   user definition wins and there is no redeclaration conflict). A program that uses
//!   only `opcache_reset` therefore injects only `opcache_reset`.
//! - `opcache_get_configuration` is version-dependent: the compile target's
//!   `PhpVersion` selects the directive set and reported version string.
//! - `opcache_reset` is SAPI-dependent: its baked boolean is the compile-time
//!   cache-enabled state — `false` for a default CLI binary (`opcache.enable_cli`
//!   default), `true` for a `--web` binary (`opcache.enable` default),
//!   matching reference PHP where `php script.php` reports the cache disabled.
//! - `opcache_get_status()['opcache_statistics']['start_time']` is MEMOIZED in a function
//!   `static`, not re-read per call: reference PHP reports the moment the cache started, a fixed
//!   point identical on every call for the life of the process (VERIFIED on 8.5.6 with two calls
//!   two seconds apart). See `GET_STATUS_TEMPLATE`.
//! - `opcache_get_status()['jit']` reports the FULL reference `opcache.jit` mapping for
//!   `kind`/`opt_level`/`opt_flags` (parsed by `crate::opcache::directives`) under one
//!   explicit clamp — `enabled`/`on` false and both buffer figures 0, always — because an
//!   AOT binary is permanently in reference PHP's own "JIT configured but unavailable in
//!   this process" state. See `render_jit_status` for the clamp's reference evidence.
//! - `opcache.restrict_api` is resolved AT COMPILE TIME, not at runtime. Reference PHP's
//!   guard compares the directive against the ENTRY SCRIPT path
//!   (`SG(request_info).path_translated`), and elephc's entry script is a compile-time
//!   constant while `--ini` is a compile-time flag — so `restrict_api_denies` decides the
//!   outcome once and `inject_if_used` bakes either the normal body or the restricted body
//!   (warning + `false`). This is EXACT, not an approximation: there is no runtime input the
//!   decision could depend on. See `restrict_api_denies` for the verified matching rule and
//!   `RESTRICT_API_WARNING_TEXT` for the verbatim message.
//! - RUNTIME `ELEPHC_INI_*` OVERRIDES are the one part of the INI surface that is NOT frozen at
//!   compile time. `ENV_OVERRIDE_HELPERS` bakes a small PHP block that reads
//!   `ELEPHC_INI_opcache__<directive>` (and the dotted `ELEPHC_INI_opcache.<directive>` as a
//!   fallback) through the ordinary `getenv` builtin, normalizes it with the PHP mirror of
//!   `ini_scanner_value` + `parse_ini_override` (`__elephc_ini_scan` then the per-type
//!   normalizer, in that order — the two implementations of every rule must answer identically,
//!   which `tests/opcache_ini_tests.rs::rust_and_php_override_paths_agree` pins by driving the
//!   same value down both paths), and feeds BOTH the typed `opcache_get_configuration()['directives']`
//!   entry and the raw `ini_get()` string — so the two move together exactly as `-d` moves both
//!   in reference PHP. Precedence is baked default → `--ini` → env. This is an elephc EXTENSION
//!   (reference PHP has no per-directive environment override, VERIFIED on 8.5.6), and it is
//!   deliberately NARROWER than `--ini`: only directives elephc merely REPORTS are overridable at
//!   runtime, because every directive it DERIVES compiled-in behavior from would otherwise make
//!   the binary contradict itself. See `crate::opcache::directives::directive_runtime_overridable`
//!   for the excluded set and the argument, and `render_opcache_env_helpers` for the injection
//!   rules.
//! - `opcache.preload` is likewise resolved AT COMPILE TIME. Reference PHP preloads during
//!   startup, BEFORE the script runs, and a preload failure is a startup FATAL — so the AOT
//!   equivalent of "the preload file is not there" is a COMPILE ERROR, not a runtime one.
//!   `preload_verdict` makes that decision once (empty directive or disabled cache ⇒ nothing
//!   happens at all; set + enabled + unresolvable ⇒ compile error; set + enabled + resolvable
//!   ⇒ `preload_statistics` is emitted, with a compile warning when the file is outside the
//!   compile-time script manifest). See `preload_verdict` for the verified reference matrix and
//!   `render_preload_statistics_stmt` for the verified key shape.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::names::{canonical_name_for_decl, Name};
use crate::parser::ast::{Program, Stmt, StmtKind};
use crate::web_prelude::PhpVersion;

use crate::opcache::directives::{
    accel_hash_max_num_entries, directive_access, directive_env_type_code, directive_env_var_names,
    directive_ini_null_default, directive_runtime_overridable, effective_directive_ini_string,
    effective_jit_config, effective_opcache_directives, opcache_directives, opcache_version_string,
    DirectiveValue, OPCACHE_PRODUCT_NAME,
};
use crate::opcache::state::opcache_cache_enabled_with_overrides;

mod detect;

/// Returns whether a compile-time `--ini` override was supplied for directive `name`.
///
/// Only the PRESENCE matters here (it is what turns a NULL-defaulting directive into an assigned
/// one — see the `__elephc_opcache_ini_null` arms in [`render_opcache_ini_helpers`]), so this does
/// not duplicate `crate::opcache::directives`' value-resolving lookup.
fn latest_ini_override<'a>(overrides: &'a [(String, String)], name: &str) -> Option<&'a str> {
    overrides
        .iter()
        .rev()
        .find(|(key, _)| key.as_str() == name)
        .map(|(_, value)| value.as_str())
}

/// The `opcache_get_configuration` function name (lowercase) the detector matches.
const GET_CONFIGURATION_FN: &str = "opcache_get_configuration";

/// The `opcache_reset` function name (lowercase) the detector matches.
const RESET_FN: &str = "opcache_reset";

/// The `opcache_get_status` function name (lowercase) the detector matches.
const GET_STATUS_FN: &str = "opcache_get_status";

/// The `opcache_is_script_cached` function name (lowercase) the detector matches.
const IS_SCRIPT_CACHED_FN: &str = "opcache_is_script_cached";

/// The `opcache_invalidate` function name (lowercase) the detector matches.
const INVALIDATE_FN: &str = "opcache_invalidate";

/// The `opcache_compile_file` function name (lowercase) the detector matches.
const COMPILE_FILE_FN: &str = "opcache_compile_file";

/// The `opcache_is_script_cached_in_file_cache` function name (lowercase) the detector matches.
const IS_SCRIPT_CACHED_IN_FILE_CACHE_FN: &str = "opcache_is_script_cached_in_file_cache";

/// The `opcache_jit_blacklist` function name (lowercase) the detector matches.
const JIT_BLACKLIST_FN: &str = "opcache_jit_blacklist";

/// One entry in the compile-time OPcache *script manifest* — a PHP source file baked
/// into the AOT binary and therefore reported as "cached" by the OPcache status/query
/// functions (`opcache_get_status`, `opcache_is_script_cached`, `opcache_compile_file`).
///
/// Constructed in `crate::pipeline` by [`collect_manifest`] from THREE sources, each path
/// stat'd once at compile time:
/// 1. the canonicalized main entry file,
/// 2. every statically-resolved `include`/`require`/`include_once`/`require_once` target
///    (`resolver::resolve_collecting_includes`),
/// 3. every autoloaded file — Composer `autoload.files`, PSR-4 / SPL-rule class files, and
///    the includes those files themselves pull in (`autoload::run_collecting_included`).
///
/// Together that is exactly the set of PHP source files compiled into the binary, which is
/// what "cached script" means for an AOT build. The `eval` interpreter has no AOT binary and
/// thus no manifest at all; its OPcache file functions stay empty-cache (see
/// `crates/elephc-magician/.../opcache_file_functions.rs`).
///
/// NOT represented (and cannot be): a file reached only through a DYNAMIC include whose path
/// the resolver could not fold to a constant — such a file is not compiled into the binary
/// either, so omitting it is correct rather than a shortfall.
pub struct ScriptEntry {
    /// Canonical (realpath-normalized) absolute path of the cached script. MUST match the
    /// canonicalization `__FILE__` bakes (`crate::magic_constants::file_pass`, which uses
    /// `Path::canonicalize`) so `opcache_is_script_cached(__FILE__)` and the `scripts` map
    /// key line up with a userland `realpath()` result.
    pub path: String,
    /// The script's source-file mtime as Unix seconds (from `std::fs::metadata`).
    /// Reference PHP reports this as the `timestamp` field (with `validate_timestamps` on,
    /// the default).
    pub timestamp: i64,
    /// A synthetic-but-stable per-script memory figure (implementation-defined; here it is
    /// derived from the source file size). No two real PHP builds agree on this value; only
    /// its presence and rough magnitude matter for callers.
    pub memory_consumption: i64,
}

/// Builds the compile-time OPcache script manifest: every PHP source file compiled into this
/// binary. Called by `crate::pipeline` twice — once before `inject_if_used` (the placeholder
/// manifest, which cannot yet see the autoloaded set) and once after `autoload::run` with the
/// complete set, whose result [`bake_manifest`] bakes in. See [`bake_manifest`] for why the
/// split is necessary.
///
/// - `main_file` is `CliConfig.filename`; it is `canonicalize`d so the manifest path matches
///   the canonicalization `__FILE__` bakes (`crate::magic_constants::file_pass`).
/// - `included_files` are the statically-resolved include/require targets
///   (`resolver::resolve_collecting_includes`), already canonical.
/// - `autoloaded_files` are the files the autoload pass loaded — Composer `autoload.files`,
///   PSR-4 / SPL-rule class files, and their own include targets
///   (`autoload::run_collecting_included`), already canonical.
///
/// ORDER (deterministic and documented; reference PHP's own `scripts` order is its internal
/// hash order and therefore not reproducible, so any stable order is as faithful):
/// 1. the entry file,
/// 2. the statically-included files, sorted by canonical path,
/// 3. the autoloaded files, sorted by canonical path.
/// Both callers pass groups 2 and 3 pre-sorted by their producing pass. Duplicates are
/// dropped across ALL groups, first occurrence winning — so a file that is both `require`d
/// and autoloaded appears once, in the include group, and the entry file never repeats.
///
/// Each path is stat'd once: `timestamp` = mtime as Unix seconds, `memory_consumption` =
/// the source file size in bytes (implementation-defined; see `ScriptEntry`). An entry whose
/// `canonicalize`/`metadata`/`modified` fails is SKIPPED rather than fabricated — an honest
/// omission. The `eval` interpreter has no manifest at all (documented on `ScriptEntry`).
pub fn collect_manifest(
    main_file: &str,
    included_files: &[PathBuf],
    autoloaded_files: &[PathBuf],
) -> Vec<ScriptEntry> {
    let mut manifest: Vec<ScriptEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    push_manifest_entry(&mut manifest, &mut seen, Path::new(main_file));
    for file in included_files {
        push_manifest_entry(&mut manifest, &mut seen, file);
    }
    for file in autoloaded_files {
        push_manifest_entry(&mut manifest, &mut seen, file);
    }
    manifest
}

/// Stats `path` and appends a `ScriptEntry` (deduplicated by canonical string). Any I/O
/// failure — missing file, unreadable metadata, or a pre-epoch mtime — skips the entry
/// silently rather than baking a fabricated timestamp or size.
///
/// `path` is `canonicalize`d HERE rather than trusted: the producing passes already
/// canonicalize, but the entry file arrives as the raw CLI argument, and canonicalizing every
/// candidate is what guarantees the dedup key and the baked path both match `__FILE__`.
fn push_manifest_entry(manifest: &mut Vec<ScriptEntry>, seen: &mut HashSet<String>, path: &Path) {
    let Ok(path) = path.canonicalize() else {
        return;
    };
    let path_str = path.display().to_string();
    if !seen.insert(path_str.clone()) {
        return;
    }
    let path = path.as_path();
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(elapsed) = modified.duration_since(UNIX_EPOCH) else {
        return;
    };
    manifest.push(ScriptEntry {
        path: path_str,
        timestamp: elapsed.as_secs() as i64,
        // Implementation-defined per-script memory: the source file's byte size.
        memory_consumption: metadata.len() as i64,
    });
}

/// The `opcache_get_configuration` function template. `__OPCACHE_CONFIGURATION__` is
/// spliced with the baked array literal at injection time; `replace` is used rather
/// than `format!` so the PHP body needs no brace escaping.
const GET_CONFIGURATION_TEMPLATE: &str = r#"function opcache_get_configuration(): array {
    return __OPCACHE_CONFIGURATION__;
}
"#;

/// The `opcache_reset` function template. `__OPCACHE_RESET_ENABLED__` is spliced with
/// the compile-time cache-enabled boolean (`true`/`false`) at injection time.
///
/// A SUCCESSFUL RESET IS NOT IDEMPOTENT. php-src's `opcache_reset()` calls
/// `zend_accel_schedule_restart(ACCEL_RESTART_USER)`, which sets `ZCSG(restart_pending)` AND clears
/// `ZCSG(accelerator_enabled)` — the very flag the function's own guard tests — so a SECOND call in
/// the same request takes the `RETURN_FALSE` exit. VERIFIED on reference PHP 8.5.6:
///
/// ```text
/// R1=true  pending=true
/// R2=false pending=true  in_progress=false
/// ```
///
/// The restart itself is deferred to the next request, so NOTHING ELSE observable changes within
/// the request: `opcache_get_status()['opcache_enabled']` stays `true`,
/// `opcache_is_script_cached()` still reports `true`, `opcache_invalidate()` still returns `true`,
/// `num_cached_scripts` / `manual_restarts` are untouched, and the `scripts` map is intact
/// (all VERIFIED in the same run). That is because the guards those functions use read the
/// REQUEST-LOCAL `ZCG(accelerator_enabled)`, snapshotted at request activation, while
/// `opcache_reset` reads the SHARED `ZCSG(accelerator_enabled)`. So the whole in-process effect is
/// the one latch this template flips through [`OPCACHE_STATE_HELPERS`].
const RESET_TEMPLATE: &str = r#"function opcache_reset(): bool {
    if (__OPCACHE_RESET_ENABLED__ === false) {
        return false;
    }
    if (__elephc_opcache_restart_pending(false)) {
        return false;
    }
    $scheduled = __elephc_opcache_restart_pending(true);
    return $scheduled;
}
"#;

/// The `opcache_get_status` function template. All `__…__` placeholders are spliced
/// with baked scalar literals at injection time (`replace` is used rather than
/// `format!` so the PHP array bodies need no brace escaping). The return type is
/// intentionally left off the signature: reference PHP is `array|false`, and omitting
/// the hint lets the ordinary union return inference handle the two exit points without
/// leaning on union-return-type codegen.
///
/// Key-order note: reference PHP emits `preload_statistics` (when preloading) and then
/// `scripts` *between* `opcache_statistics` and `jit`. To reproduce that order exactly, the
/// prefix literal stops after `opcache_statistics`, `preload_statistics` is inserted next
/// (only when preloading), `scripts` after it (only when requested), and `jit` is appended
/// last — appending `scripts` after `jit` (the naive shape) would misorder the keys. When
/// `$include_scripts === false` the `scripts` key is *absent*, not empty; VERIFIED on
/// reference PHP 8.5.6, `preload_statistics` is NOT affected by `$include_scripts` and still
/// precedes `jit` in that case.
///
/// `start_time` IS MEMOIZED IN A `static`, NOT A BARE `time()` CALL. Reference PHP reports the
/// moment the SHARED CACHE started, which for a CLI run is effectively process start — a fixed
/// point, identical on every call for the life of the process. VERIFIED on reference PHP 8.5.6:
/// two `opcache_get_status()` calls with `sleep(2)` between them report the SAME `start_time`.
/// An earlier revision baked `'start_time' => time()` straight into the array literal, which
/// re-evaluated the clock per call and made the same two-call probe report values two apart.
/// The `static` is initialized on the first call that gets past the enabled gate, so the value
/// is stable from then on; a plain function-local or a re-read of `time()` would not be, and a
/// baked compile-time constant would report the BUILD time rather than a run time.
///
/// THE SAME `static` ALSO FEEDS THE `scripts` MAP'S CLOCKS. Reference PHP's per-script
/// `last_used_timestamp` is the REQUEST time (`dynamic_members.last_used`), identical for every
/// entry, and `revalidate` is that same request time plus `opcache.revalidate_freq` — see
/// [`render_scripts_map_literal`]. For a CLI process the request time and the cache start time are
/// the same instant, so one memoized clock serves both and they cannot disagree.
///
/// `interned_strings_usage` IS A WHOLE-KEY SLOT (`__INTERNED_STRINGS_USAGE__`), not four scalars,
/// because reference PHP OMITS THE KEY ENTIRELY when the interned-strings buffer was never stood
/// up. php-src guards it with `if (ZCSG(interned_strings).start && ZCSG(interned_strings).end)`,
/// and `opcache.interned_strings_buffer=0` leaves both NULL. VERIFIED on reference PHP 8.5.6:
/// with `-d opcache.interned_strings_buffer=0` the status array has EIGHT top-level keys
/// (`opcache_enabled, cache_full, restart_pending, restart_in_progress, memory_usage,
/// opcache_statistics, scripts, jit`) against the usual nine. See
/// [`render_interned_strings_usage`].
const GET_STATUS_TEMPLATE: &str = r#"function opcache_get_status($include_scripts = true) {
    if (__OPCACHE_STATUS_ENABLED__ === false) {
__RESTRICT_API_WARNING__
        return false;
    }
    static $__elephc_opcache_start_time = 0;
    if ($__elephc_opcache_start_time === 0) { $__elephc_opcache_start_time = time(); }
    $status = [
        'opcache_enabled' => true,
        'cache_full' => false,
        'restart_pending' => __elephc_opcache_restart_pending(false),
        'restart_in_progress' => false,
        'memory_usage' => [
            'used_memory' => __MEM_USED__,
            'free_memory' => __MEM_FREE__,
            'wasted_memory' => 0,
            'current_wasted_percentage' => 0.0,
        ],
__INTERNED_STRINGS_USAGE__
        'opcache_statistics' => [
            'num_cached_scripts' => __NUM_CACHED_SCRIPTS__,
            'num_cached_keys' => __NUM_CACHED_KEYS__,
            'max_cached_keys' => __MAX_CACHED_KEYS__,
            'hits' => 0,
            'start_time' => $__elephc_opcache_start_time,
            'last_restart_time' => 0,
            'oom_restarts' => 0,
            'hash_restarts' => 0,
            'manual_restarts' => 0,
            'misses' => 0,
            'blacklist_misses' => 0,
            'blacklist_miss_ratio' => 0.0,
            'opcache_hit_rate' => 0.0,
        ],
    ];
__PRELOAD_STATISTICS__
    if ($include_scripts) {
        $status['scripts'] = __SCRIPTS_MAP__;
    }
    $status['jit'] = [
        'enabled' => __JIT_ENABLED__,
        'on' => __JIT_ON__,
        'kind' => __JIT_KIND__,
        'opt_level' => __JIT_OPT_LEVEL__,
        'opt_flags' => __JIT_OPT_FLAGS__,
        'buffer_size' => __JIT_BUFFER_SIZE__,
        'buffer_free' => __JIT_BUFFER_FREE__,
    ];
    return $status;
}
"#;

/// The `opcache_is_script_cached` function template. `__OPCACHE_ENABLED__` is spliced with
/// the compile-time cache-enabled boolean and `__MANIFEST_PATHS__` with the baked PHP array
/// literal of canonical cached-script paths.
///
/// php-src gates `opcache_is_script_cached` on OPcache being enabled: when disabled it
/// returns `false` for every path (the disabled gate below). When enabled it reports whether
/// `$filename` resolves (via `realpath`) to a script in the cache. Because the argument may
/// be a relative path or `__FILE__` (already canonical), the body normalizes it with
/// `realpath()` before membership so both `opcache_is_script_cached('./main.php')` and
/// `opcache_is_script_cached(__FILE__)` hit the canonical manifest entry.
///
/// The manifest is every PHP source file compiled into this binary: the entry file, every
/// statically-resolved `include`/`require` target, and every autoloaded file (Composer
/// `autoload.files` + PSR-4 / SPL-rule class files + their own includes) — see `ScriptEntry`.
/// The parameter is kept named `$filename` to match reference PHP for named-argument callers.
/// A FORCE-INVALIDATED entry reports `false` here even though it is still in the manifest: that is
/// php-src's `filename_is_in_cache`, which requires `!persistent_script->corrupted`, and
/// `zend_accel_invalidate($f, true)` sets exactly that flag. See [`INVALIDATE_TEMPLATE`] and
/// [`OPCACHE_STATE_HELPERS`].
///
/// The EMPTY PATH is resolved through `getcwd()` rather than `realpath('')` — see
/// [`INVALIDATE_TEMPLATE`] for the reference evidence and the `realpath('')` divergence it works
/// around. Inlined rather than factored into a shared helper: routing the result through a
/// `string|false` wrapper function makes elephc's checker widen it to `Str`, and the
/// `$rp === false` test against `__rt_realpath`'s empty-string sentinel then always fails
/// (reproduced: a nonexistent path came back as `'OK:'` instead of `FALSE`).
const IS_SCRIPT_CACHED_TEMPLATE: &str = r#"function opcache_is_script_cached($filename): bool {
    if (__OPCACHE_ENABLED__ === false) {
        return false;
    }
    $path = '';
    if ($filename === '') {
        $cwd = getcwd();
        if ($cwd === false) {
            return false;
        }
        $path = (string) $cwd;
    } else {
        $rp = realpath($filename);
        if ($rp === false) {
            return false;
        }
        $path = (string) $rp;
    }
    if (__elephc_opcache_invalidate_state($path, 0)) {
        return false;
    }
    return in_array($path, __MANIFEST_PATHS__, true);
}
"#;

/// The `opcache_invalidate` function template. `__OPCACHE_ENABLED__` is spliced with the
/// compile-time cache-enabled boolean (`true`/`false`). This one is FULLY correct (not an
/// interim): reference PHP's `zend_accel_invalidate` returns `file_found` = the script is in
/// the cache OR `realpath($filename)` resolves. That disjunction reduces to its right-hand
/// side ALGEBRAICALLY, not because the cache is empty (it is not — `ScriptEntry`/
/// `__MANIFEST_PATHS__` is the compile-time manifest `opcache_is_script_cached` and
/// `opcache_compile_file` read): every manifest member is a CANONICALIZED path to a file that
/// was stat'd at compile time, so cache membership already implies `realpath` resolves and the
/// left operand can never be the deciding one. Hence `realpath($filename) !== false` on the
/// enabled path, and `false` when the cache is disabled (verified against reference PHP 8.5.6:
/// an existing but uncached file → `true` when enabled, `false` when disabled; a nonexistent
/// path → `false`; the manifest needs no lookup here).
///
/// `$force = true` DISCARDS THE ENTRY, and that IS observable. php-src's `zend_accel_invalidate`
/// calls `zend_accel_lock_discard_script`, which sets `persistent_script->corrupted = true` and
/// `persistent_script->timestamp = 0`. VERIFIED on reference PHP 8.5.6 in one request:
///
/// ```text
/// is_script_cached  = true
/// invalidate($f, true) = true
/// is_script_cached  = false          <- the discard
/// count(scripts)    = 1              <- the ENTRY STAYS
/// num_cached_scripts/num_cached_keys = 1/2   <- and so do the COUNTS
/// scripts[$f]['timestamp'] = 0       <- the only field that moves
/// opcache_compile_file($f) = true; is_script_cached = true   <- and re-caching undoes it
/// ```
///
/// So the model is a per-process DISCARD SET ([`OPCACHE_STATE_HELPERS`]) consulted by
/// `opcache_is_script_cached` and by the `scripts` map's `timestamp` field, added to by a forced
/// `opcache_invalidate` and removed from by `opcache_compile_file`. `num_cached_scripts`,
/// `num_cached_keys` and the `scripts` map's membership are deliberately NOT changed, because
/// reference PHP does not change them either — the discarded script keeps its shared-memory slot
/// until the next restart.
///
/// THE EMPTY PATH RETURNS `true`, and the reason is `realpath`, not OPcache. Reference PHP's
/// `zend_accel_invalidate` returns SUCCESS whenever the path RESOLVES, and PHP's `realpath('')`
/// resolves to the CURRENT WORKING DIRECTORY (`string(18) "/private/tmp/spike"` on this host) —
/// so `opcache_invalidate('')` is `true` for the same reason `opcache_invalidate('.')`,
/// `opcache_invalidate('/')` and `opcache_invalidate('/tmp')` are (all VERIFIED true; `' '` and a
/// NUL byte are VERIFIED false). Elephc's `realpath('')` returns `false` instead — a `__rt_realpath`
/// divergence with a much wider blast radius than this prelude, since libc `realpath("")` fails
/// with ENOENT while PHP's `expand_filepath` maps the empty path to the cwd. Rather than change
/// four targets' hand-written assembly from inside an OPcache batch, the three path-taking OPcache
/// functions each spell the empty case out as `getcwd()`, which is precisely what PHP resolves it
/// to. The residual `realpath('')` bug is reported separately and unfixed.
///
/// The `$force` argument is coerced to `bool` in the body (which also keeps the checker from
/// reporting the parameter unused). The parameters are kept named `$filename`/`$force` to match
/// reference PHP for named-argument callers.
const INVALIDATE_TEMPLATE: &str = r#"function opcache_invalidate($filename, $force = false): bool {
    if (__OPCACHE_ENABLED__ === false) {
        return false;
    }
    $force = (bool) $force;
    $path = '';
    if ($filename === '') {
        $cwd = getcwd();
        if ($cwd === false) {
            return false;
        }
        $path = (string) $cwd;
    } else {
        $rp = realpath($filename);
        if ($rp === false) {
            return false;
        }
        $path = (string) $rp;
    }
    if ($force && in_array($path, __MANIFEST_PATHS__, true)) {
        __elephc_opcache_invalidate_state($path, 1);
    }
    return true;
}
"#;

/// The `opcache_compile_file` function template. `__OPCACHE_ENABLED__` is spliced with the
/// compile-time cache-enabled boolean (`true`/`false`) and it takes exactly one argument.
///
/// Disabled path: reference PHP emits an engine-level `E_NOTICE` (level 8) with the exact
/// message below and returns `false`. Elephc has no `set_error_handler` dispatch and its
/// `trigger_error` is a web-only prelude that (a) cannot be resolved on a CLI binary and
/// (b) would need an `E_USER_*` level (real userland `trigger_error(E_NOTICE)` throws a
/// `ValueError`). So the notice is rendered directly to `STDERR` as `Notice: <message>` —
/// the same shape elephc's own `trigger_error` uses for notices, and matching reference
/// PHP's stderr notice line (minus the ` in <file> on line <n>` suffix, which elephc does
/// not synthesize). The message text is reproduced verbatim from php-src.
///
/// Enabled path: Elephc is an AOT compiler with no runtime PHP compiler, so a file not
/// already baked into the binary cannot be compiled at runtime. A file that IS in the
/// script manifest, however, was already compiled into the binary, so `opcache_compile_file`
/// reports success (`true`) for it — matching reference PHP, where compiling an
/// already-cached file is a hit. Unknown files return `false` (elephc cannot compile them at
/// runtime). `__MANIFEST_PATHS__` is the baked canonical-path array; `$filename` is
/// `realpath`-normalized first so a relative path or `__FILE__` both resolve against it.
/// This REMOVES the earlier empty-cache divergence for manifest files (they now correctly
/// return `true`), and the manifest now covers statically-included and autoloaded files too,
/// so `opcache_compile_file('<a required or PSR-4 file>')` returns `true` as well.
/// The parameter is kept named `$filename` to match reference PHP for named-argument callers.
/// RE-CACHING CLEARS A FORCED INVALIDATION — UNLESS A RESTART IS PENDING. Reference PHP
/// recompiles and re-inserts the script, so a previously discarded entry becomes cached again:
/// VERIFIED on 8.5.6 that `invalidate($f, true)` → `is_script_cached = false` →
/// `opcache_compile_file($f) = true` → `is_script_cached = true`. But the SAME sequence run after
/// an `opcache_reset()` ends `is_script_cached = false` — also VERIFIED — because
/// `persistent_compile_file` only STORES into the cache while `ZCSG(accelerator_enabled)` holds,
/// and `zend_accel_schedule_restart` has cleared it. `opcache_compile_file` still returns `true`
/// in both cases: its return reports whether the COMPILE succeeded, not whether the store did.
/// The restart-latch guard below is exactly that condition.
const COMPILE_FILE_TEMPLATE: &str = r#"function opcache_compile_file($filename): bool {
    if (__OPCACHE_ENABLED__ === false) {
        fwrite(STDERR, "Notice: Zend OPcache has not been properly started, can't compile file\n");
        return false;
    }
    $path = '';
    if ($filename === '') {
        $cwd = getcwd();
        if ($cwd === false) {
            return false;
        }
        $path = (string) $cwd;
    } else {
        $rp = realpath($filename);
        if ($rp === false) {
            return false;
        }
        $path = (string) $rp;
    }
    if (!in_array($path, __MANIFEST_PATHS__, true)) {
        return false;
    }
    if (!__elephc_opcache_restart_pending(false)) {
        __elephc_opcache_invalidate_state($path, 2);
    }
    return true;
}
"#;

/// The `opcache_is_script_cached_in_file_cache` function template.
///
/// ALWAYS `false`, and that is EXACT rather than a shortfall. php-src gates the function on
/// `if (!ZCG(accel_directives).file_cache) { RETURN_FALSE; }` BEFORE it looks anything up, and
/// `opcache.file_cache` has no default — it is registered with a C `NULL` (the one directive that
/// is, see `crate::opcache::directives::directive_ini_null_default`). So an unconfigured reference
/// PHP returns `false` for every path too. VERIFIED on reference PHP 8.5.6:
/// `opcache_is_script_cached_in_file_cache(__FILE__)` is `bool(false)` with
/// `-d opcache.enable=1 -d opcache.enable_cli=1` and no `opcache.file_cache`. Elephc has no
/// on-disk opcode cache to point `opcache.file_cache` at in the first place, so the `false` holds
/// unconditionally.
///
/// The signature is reference's: `opcache_is_script_cached_in_file_cache(string $filename): bool`,
/// one required parameter, confirmed through `ReflectionFunction` on 8.5.6. `$filename` is cast
/// (a no-op) so the checker does not report it unused.
const IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE: &str =
    r#"function opcache_is_script_cached_in_file_cache($filename): bool {
    $filename = (string) $filename;
    return false;
}
"#;

/// The `opcache_jit_blacklist` function template — a NO-OP returning `null`.
///
/// php-src's body is `zend_jit_blacklist_function()` guarded by `#ifdef HAVE_JIT`, i.e. it only
/// ever mutates the JIT's own blacklist and reports nothing. An elephc binary is ahead-of-time
/// compiled with no runtime JIT engine (the same fact `render_jit_status` clamps `enabled`/`on` on),
/// so there is no blacklist to add to and the no-op is the whole of the observable behavior.
///
/// The signature is reference's: `opcache_jit_blacklist(Closure $closure): void`, one required
/// parameter typed `Closure`, confirmed through `ReflectionFunction` on 8.5.6, where the call
/// evaluates to `NULL`. The parameter is left UNTYPED here so a caller passing a first-class
/// callable or a closure-typed variable is not rejected by elephc's checker on a shape reference
/// PHP would accept, and it is reassigned once so the checker does not report it unused.
///
/// NOT RESTRICTED by `opcache.restrict_api` — it is one of only TWO exported OPcache functions
/// php-src does not run `validate_api_restriction()` on (the other is `opcache_compile_file`).
/// VERIFIED on reference PHP 8.5.6 with `-d opcache.restrict_api=/nonexistent`: the other six
/// functions each emit the warning and return `false`, while `opcache_jit_blacklist` returns
/// `NULL` and `opcache_compile_file` returns `true`, both silently.
const JIT_BLACKLIST_TEMPLATE: &str = r#"function opcache_jit_blacklist($closure): void {
    $closure = $closure;
}
"#;

/// The IN-PROCESS OPCACHE STATE block, written in elephc-PHP and baked verbatim.
///
/// WHY IT EXISTS. Three OPcache observables are neither compile-time constants nor clocks: they are
/// MUTABLE PER-PROCESS STATE that one API call writes and another reads. An AOT binary can model
/// them exactly, because "the process" is the whole lifetime of the cache it is pretending to have.
///
/// 1. `__elephc_opcache_restart_pending` — the `ZCSG(restart_pending)` latch. `opcache_reset()`
///    sets it and thereafter returns `false`; `opcache_get_status()['restart_pending']` reads it.
///    See [`RESET_TEMPLATE`] for the verified reference transcript.
/// 2. `__elephc_opcache_invalidate_state` — the DISCARD SET (`persistent_script->corrupted`).
///    `opcache_invalidate($f, true)` adds ($op 1), `opcache_compile_file($f)` removes ($op 2), and
///    `opcache_is_script_cached` / the `scripts` map's `timestamp` read ($op 0). See
///    [`INVALIDATE_TEMPLATE`] for the verified reference transcript.
/// 3. `__elephc_opcache_script_timestamp` — the read side of (2) for the `scripts` map, kept a
///    FUNCTION so the map stays a single array literal (see below).
///
/// It also carries the `scripts` map's TIME FORMATTER, which is not state but is here because it
/// is the map's other non-literal field:
///
/// 4. `__elephc_opcache_system_timezone` + `__elephc_opcache_asctime` — the `last_used` string.
///    php-src builds it as `asctime(localtime(&…last_used))`, i.e. through LIBC, which means it
///    follows the SYSTEM timezone and NOT PHP's `date.timezone`. VERIFIED on reference PHP 8.5.6
///    for the same timestamp 1784994683: `TZ=UTC` → `Sat Jul 25 15:51:23 2026`,
///    `TZ=America/New_York` → `Sat Jul 25 11:51:23 2026`, and with TZ UNSET →
///    `Sat Jul 25 17:51:23 2026` (this host's `/etc/localtime` → `…/zoneinfo/Europe/Paris`) — while
///    `date('D M d H:i:s Y', …)` in the SAME process reports `15:51:23` in all three, because
///    `date.timezone` is UTC throughout. So `date()` alone can never reproduce this field.
///    `__elephc_opcache_system_timezone` resolves the zone the way libc does — `TZ` first, then the
///    `/etc/localtime` symlink's `…/zoneinfo/<Zone>` tail — and `__elephc_opcache_asctime` applies
///    it around the formatting and RESTORES the previous default, so a caller's own `date()` after
///    `opcache_get_status()` is unaffected (verified: the default reads `UTC` again on return). An
///    unresolvable zone leaves the default in place, matching libc's own fallback; a zone name
///    elephc's tz data does not know formats as UTC without a fatal.
///
///    `asctime`'s day-of-month is `%3d` — SPACE-padded — hence the explicit `$pad`, verified
///    against reference PHP's `Thu Jul  2 13:46:40 2026` (two spaces) for a single-digit day.
///
/// THREE ELEPHC CODEGEN CONSTRAINTS SHAPED THIS CODE, all reproduced before it was written:
/// - `static $set = [];` is rejected by the EIR backend (`init_static_local assigning PHP type
///   Array(Never) to static local`), so the discard set is seeded with one typed dummy entry. The
///   `''` key is harmless: no `realpath`/`getcwd` result is the empty string, and it reads `false`.
/// - `$set[$path] ?? false` on an ABSENT key returns `true` in elephc (reproduced: the first probe
///   of an untouched path answered `true` where PHP answers `false`), so the read is spelled as an
///   `isset` guard plus a bound local — which was verified to match PHP exactly over an
///   add/probe/re-add/remove sequence.
/// - `$map[$key]['timestamp'] = 0` is rejected outright (`Nested array assignment requires a Mixed
///   or ArrayAccess target`), which is why the `scripts` map cannot be patched after the fact and
///   the discard is folded into the literal through `__elephc_opcache_script_timestamp` instead.
///   (A string-set encoding was tried first and abandoned: `strpos` against a MUTATED static-local
///   haystack reports a match for a needle that is not there — reproduced, and reported separately.)
///
/// INJECTED EXACTLY ONCE, by [`inject_if_used`], whenever any OPcache function that reads or writes
/// this state is injected. `crate::web_prelude` does NOT own a copy (unlike the INI/env helpers):
/// the OPcache functions themselves are injected from here under `--web` too, so there is only ever
/// one emitter.
const OPCACHE_STATE_HELPERS: &str = r#"function __elephc_opcache_restart_pending(bool $schedule): bool {
    static $pending = false;
    if ($schedule) { $pending = true; }
    return $pending;
}
function __elephc_opcache_invalidate_state(string $path, int $op): bool {
    static $discarded = ['' => false];
    if ($op === 1) { $discarded[$path] = true; }
    if ($op === 2) { $discarded[$path] = false; }
    if (!isset($discarded[$path])) { return false; }
    $state = $discarded[$path];
    return $state;
}
function __elephc_opcache_script_timestamp(string $path, int $timestamp): int {
    if (__elephc_opcache_invalidate_state($path, 0)) { return 0; }
    return $timestamp;
}
function __elephc_opcache_system_timezone(): string {
    $tz = (string) getenv('TZ');
    if ($tz !== '') { return $tz; }
    $link = readlink('/etc/localtime');
    if ($link === false) { return ''; }
    $target = (string) $link;
    $at = strpos($target, '/zoneinfo/');
    if ($at === false) { return ''; }
    $zone = substr($target, $at + 10);
    return $zone;
}
function __elephc_opcache_asctime(int $timestamp): string {
    $zone = __elephc_opcache_system_timezone();
    $previous = '';
    if ($zone !== '') { $previous = date_default_timezone_get(); date_default_timezone_set($zone); }
    $day = (int) date('j', $timestamp);
    $pad = '';
    if ($day < 10) { $pad = ' '; }
    $formatted = date('D M ', $timestamp) . $pad . $day . date(' H:i:s Y', $timestamp);
    if ($zone !== '') { date_default_timezone_set($previous); }
    return $formatted;
}
"#;

/// Returns the in-process OPcache state block ([`OPCACHE_STATE_HELPERS`]).
fn render_opcache_state_helpers() -> String {
    OPCACHE_STATE_HELPERS.to_string()
}

/// The `opcache.restrict_api` directive name.
const RESTRICT_API_DIRECTIVE: &str = "opcache.restrict_api";

/// The VERBATIM `E_WARNING` text php-src's OPcache API guard emits when
/// `opcache.restrict_api` denies a call. In php-src this is
/// `zend_error(E_WARNING, ACCELERATOR_PRODUCT_NAME " API is restricted by \"restrict_api\"
/// configuration directive")` with `ACCELERATOR_PRODUCT_NAME` = `"Zend OPcache"`.
///
/// VERIFIED byte-for-byte against reference PHP 8.5.6 (Homebrew, `Zend OPcache` loaded):
/// `php -d opcache.enable=1 -d opcache.enable_cli=1 -d opcache.restrict_api=/nonexistent`
/// emits exactly `Warning: Zend OPcache API is restricted by "restrict_api" configuration
/// directive in <file> on line <n>`. Elephc reproduces the message but not the
/// ` in <file> on line <n>` suffix, which it does not synthesize — the same documented
/// shortfall the `opcache_compile_file` notice carries (see `COMPILE_FILE_TEMPLATE`).
const RESTRICT_API_WARNING_TEXT: &str =
    "Zend OPcache API is restricted by \"restrict_api\" configuration directive";

/// Renders the restricted-path diagnostic as a PHP statement indented by `indent`.
///
/// Written straight to `STDERR` as `Warning: <text>` rather than through `trigger_error`:
/// on CLI there is no `trigger_error` at all, and under `--web` the prelude's
/// `trigger_error($msg, E_WARNING)` itself does `fwrite(STDERR, 'Warning: ' . $msg . "\n")`
/// (see `crate::web_prelude`), so this emits BYTE-IDENTICAL output in both SAPIs while
/// staying resolvable on a plain CLI binary. The message is single-quoted so its embedded
/// `"restrict_api"` double quotes need no escaping.
fn render_restrict_api_warning_stmt(indent: &str) -> String {
    format!(
        "{indent}fwrite(STDERR, {} . \"\\n\");",
        render_php_single_quoted(&format!("Warning: {RESTRICT_API_WARNING_TEXT}")),
    )
}

/// Fills a template's `__RESTRICT_API_WARNING__` slot.
///
/// When NOT restricted the placeholder line is removed WHOLE (newline included), so the
/// rendered function is BYTE-IDENTICAL to the template as it read before `opcache.restrict_api`
/// existed — that is what keeps the default path a true no-op rather than a whitespace diff.
fn splice_restrict_api_warning(template: &str, restricted: bool, indent: &str) -> String {
    if restricted {
        template.replace(
            "__RESTRICT_API_WARNING__",
            &render_restrict_api_warning_stmt(indent),
        )
    } else {
        template.replace("__RESTRICT_API_WARNING__\n", "")
    }
}

/// Decides — AT COMPILE TIME — whether `opcache.restrict_api` denies this binary's calls into
/// the OPcache API, reproducing php-src's `validate_api_restriction()` exactly.
///
/// php-src:
/// ```c
/// if (ZCG(accel_directives).restrict_api && *ZCG(accel_directives).restrict_api) {
///     size_t len = strlen(ZCG(accel_directives).restrict_api);
///     if (!SG(request_info).path_translated ||
///         strlen(SG(request_info).path_translated) < len ||
///         memcmp(SG(request_info).path_translated, ZCG(accel_directives).restrict_api, len) != 0) {
///         zend_error(E_WARNING, ...); return 0;
///     }
/// }
/// ```
///
/// Every rule below is VERIFIED against reference PHP 8.5.6, not merely derived from the source:
/// - EMPTY prefix disables the restriction entirely (`restrict_api=` → allowed).
/// - The comparison target is the ENTRY SCRIPT, not the currently-executing file. PROVEN with an
///   entry in one directory that `require`s a script in another and calls the API from there:
///   `restrict_api=<entry's dir>` ALLOWED the call, `restrict_api=<includee's dir>` DENIED it.
///   This is precisely what makes the compile-time evaluation exact for elephc — the entry script
///   is fixed when the binary is built.
/// - It is a PLAIN BYTE PREFIX, NOT a path-component match: prefix `/private/tmp/ra/foo` ALLOWS
///   entry `/private/tmp/ra/foobar/x.php`. (`str::starts_with` on `&str` is a byte compare, so it
///   reproduces `memcmp` verbatim.)
/// - It is CASE-SENSITIVE even on a case-insensitive filesystem: prefix `…/Foobar` DENIES entry
///   `…/foobar/x.php` (memcmp, not a filesystem lookup).
/// - A prefix LONGER than the entry path denies (the `strlen(...) < len` arm).
/// - A prefix EQUAL to the whole entry path allows.
/// - The path compared is the RESOLVED one: invoking `php /tmp/ra/foobar/x.php` on macOS (where
///   `/tmp` symlinks to `/private/tmp`) DENIES prefix `/tmp/ra` and ALLOWS `/private/tmp/ra`.
///   That is why `entry_path` must be the canonicalized path — the same canonicalization
///   `__FILE__` and `ScriptEntry::path` use.
///
/// `entry_path` of `None` denies, mirroring php-src's `!SG(request_info).path_translated` arm
/// (no entry script to compare against ⇒ the restriction cannot be satisfied).
///
/// COMPILE-TIME vs RUNTIME: reference PHP evaluates this per request against the live script
/// path. An elephc AOT binary has exactly one entry script, fixed when it was compiled, and
/// `--ini` is a compile-time flag — so the predicate has no runtime-varying input and baking its
/// result loses nothing.
fn restrict_api_denies(
    entry_path: Option<&str>,
    version_id: u32,
    overrides: &[(String, String)],
) -> bool {
    let prefix = directive_str(version_id, RESTRICT_API_DIRECTIVE, overrides);
    if prefix.is_empty() {
        return false;
    }
    match entry_path {
        // php-src's `!SG(request_info).path_translated` arm: nothing to compare ⇒ deny.
        None => true,
        // `starts_with` on `&str` compares bytes, matching php-src's `memcmp`; a prefix longer
        // than the path can never match, covering the `strlen(path) < len` arm.
        Some(path) => !path.starts_with(&prefix),
    }
}

/// Canonicalizes the compile-time entry script path for the `opcache.restrict_api` comparison.
///
/// Uses `Path::canonicalize`, the SAME normalization `__FILE__` bakes
/// (`crate::magic_constants::file_pass`) and `collect_manifest` applies to `ScriptEntry::path`,
/// because reference PHP compares the RESOLVED script path (verified: on macOS a `/tmp/...`
/// invocation is compared as `/private/tmp/...`). Returns `None` when the path cannot be
/// resolved, which `restrict_api_denies` treats as php-src's null-`path_translated` deny arm.
///
/// This is deliberately NOT read back out of the manifest: `collect_manifest` skips any entry it
/// cannot stat, so a manifest's first element is only *usually* the entry file — and silently
/// comparing an `autoload.files` path instead would flip a security-shaped decision.
pub fn canonical_entry_path(main_file: &str) -> Option<String> {
    Path::new(main_file)
        .canonicalize()
        .ok()
        .map(|path| path.display().to_string())
}

/// The `opcache.preload` directive name.
const PRELOAD_DIRECTIVE: &str = "opcache.preload";

/// The user-declared PHP symbol names an elephc binary bakes, in source declaration order.
///
/// Collected by [`collect_preload_symbols`] from the resolved AST BEFORE any compiler prelude is
/// injected, so the lists carry USER functions/classes only — never `var_export`, the PDO surface,
/// or the OPcache functions this module itself injects. That mirrors reference PHP, where
/// `preload_statistics` reports the DELTA the preload pass added to the symbol tables and can
/// therefore never contain a built-in.
#[derive(Debug, Default, Clone)]
pub struct PreloadSymbols {
    /// Fully-qualified function names, original case, no leading `\` (the reference spelling —
    /// VERIFIED: a `namespace My\Space; function MixedCaseFn(){}` preload reports
    /// `My\Space\MixedCaseFn`).
    functions: Vec<String>,
    /// Fully-qualified class-like names, original case, no leading `\`. Reference PHP puts
    /// classes, INTERFACES, TRAITS and ENUMS all under the single `classes` key — VERIFIED on
    /// PHP 8.5.6 with one preload file declaring all four.
    classes: Vec<String>,
}

/// Collects the user-declared function and class-like names of a resolved program.
///
/// Recursion set is EXACTLY the one `detect::stmt_declares` uses (`NamespaceBlock`,
/// `IncludeOnceGuard`, `Synthetic`): those are the block forms that can host a hoisted top-level
/// declaration. Conditionally-declared functions (inside `if`/loops) are not collected, matching
/// that precedent — and matching reference PHP, where preloading a file whose declarations are
/// conditional does not add them either (the preload pass runs the file, but elephc's manifest
/// analogue is static).
///
/// Names are canonicalized with [`canonical_name_for_decl`] against the enclosing namespace, since
/// this runs BEFORE `name_resolver` and the raw `FunctionDecl`/`ClassDecl` names are local.
/// Duplicates are dropped case-insensitively (PHP symbol names are case-insensitive, and elephc's
/// `FunctionVariantGroup` extension can surface one PHP-visible name through several declarations).
pub fn collect_preload_symbols(program: &[Stmt]) -> PreloadSymbols {
    let mut symbols = PreloadSymbols::default();
    let mut seen_functions: HashSet<String> = HashSet::new();
    let mut seen_classes: HashSet<String> = HashSet::new();
    collect_symbols_in(program, None, &mut symbols, &mut seen_functions, &mut seen_classes);
    symbols
}

/// Walks one statement list under `namespace`, appending every declaration it hosts.
///
/// `NamespaceDecl` (the statement form, `namespace X;`) rebinds the namespace for the REST of the
/// current list; `NamespaceBlock` (the brace form) scopes it to its own body only.
fn collect_symbols_in(
    body: &[Stmt],
    namespace: Option<&str>,
    out: &mut PreloadSymbols,
    seen_functions: &mut HashSet<String>,
    seen_classes: &mut HashSet<String>,
) {
    let mut current: Option<String> = namespace.map(str::to_string);
    for stmt in body {
        match &stmt.kind {
            StmtKind::NamespaceDecl { name } => {
                current = name.as_ref().map(Name::as_canonical);
            }
            StmtKind::NamespaceBlock { name, body } => {
                collect_symbols_in(
                    body,
                    name.as_ref().map(Name::as_str),
                    out,
                    seen_functions,
                    seen_classes,
                );
            }
            StmtKind::IncludeOnceGuard { body, .. } | StmtKind::Synthetic(body) => {
                collect_symbols_in(body, current.as_deref(), out, seen_functions, seen_classes);
            }
            StmtKind::FunctionDecl { name, .. } => {
                let fqn = canonical_name_for_decl(current.as_deref(), name);
                if seen_functions.insert(fqn.to_ascii_lowercase()) {
                    out.functions.push(fqn);
                }
            }
            // Reference PHP reports all four class-like kinds under `classes` (VERIFIED).
            // `PackedClassDecl` is elephc's `#[Packed] class` extension — still a user-declared
            // class from PHP's point of view, so it is reported like any other.
            StmtKind::ClassDecl { name, .. }
            | StmtKind::EnumDecl { name, .. }
            | StmtKind::InterfaceDecl { name, .. }
            | StmtKind::TraitDecl { name, .. }
            | StmtKind::PackedClassDecl { name, .. } => {
                let fqn = canonical_name_for_decl(current.as_deref(), name);
                if seen_classes.insert(fqn.to_ascii_lowercase()) {
                    out.classes.push(fqn);
                }
            }
            _ => {}
        }
    }
}

/// The COMPILE-TIME verdict on `opcache.preload` for this binary.
///
/// THE REFERENCE MATRIX, all four rows VERIFIED against reference PHP 8.5.6 (Homebrew, `Zend
/// OPcache` loaded), not derived from php-src:
/// - `opcache.preload` EMPTY (the default) → no preloading, `opcache_get_status()` carries NO
///   `preload_statistics` key. (Also verified with an explicit `-d opcache.preload=`.)
/// - Set + cache ENABLED + the path RESOLVES → `preload_statistics` appears, between
///   `opcache_statistics` and `scripts`.
/// - Set + cache ENABLED + the path does NOT resolve → reference FATALS AT STARTUP, before a
///   single line of the script runs, and exits 1:
///   `PHP Warning:  PHP Startup: Failed to open stream: No such file or directory in Unknown on
///   line 0` then `PHP Fatal error:  Failed opening required '<path>' (include_path='…') in
///   Unknown on line 0`.
/// - Set + cache DISABLED (`opcache.enable_cli=0`, the CLI default) → the process runs FINE,
///   nothing is preloaded (`function_exists()` on a preload-file symbol is `false`) and
///   `opcache_get_status()` returns `false`. A MISSING path in this state is also harmless: no
///   validation happens at all, exit 0.
///
/// WHY A COMPILE ERROR IS THE HONEST AOT MAPPING of the startup fatal: reference resolves
/// `opcache.preload` once, at process startup, before user code. elephc's INI is fixed when the
/// binary is built (`--ini` is a compile-time flag), so "startup" for an elephc binary IS compile
/// time. Refusing to build is the only way to avoid shipping a binary that would report
/// statistics for a file that is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreloadVerdict {
    /// `opcache.preload` is empty, OR the OPcache cache is disabled for this target. Nothing is
    /// validated and no `preload_statistics` key is emitted — byte-identical to the behavior
    /// before this feature existed.
    NotPreloading,
    /// Set, cache enabled, but the path does not resolve to a readable file: a COMPILE ERROR.
    /// Carries the directive value VERBATIM (not canonicalized — there is nothing to canonicalize),
    /// which is also the spelling reference PHP puts in its fatal.
    Unresolvable {
        /// The raw `opcache.preload` value as written.
        requested: String,
    },
    /// Set, cache enabled, path resolves. `in_manifest` is whether the resolved path is one of the
    /// scripts elephc actually baked into this binary; `false` earns a compile WARNING but not an
    /// error, because a preload file that this program never includes, requires or autoloads is a
    /// legitimate configuration — reference PHP would preload it, elephc simply does not compile
    /// it in. The membership test must be made against the COMPLETE manifest, so `crate::pipeline`
    /// evaluates the warning only after `autoload::run` (see [`bake_manifest`]).
    Preloading {
        /// The canonicalized preload path (same normalization `__FILE__` and [`ScriptEntry`] use).
        resolved: String,
        /// Whether `resolved` is a member of the compile-time script manifest.
        in_manifest: bool,
    },
}

impl PreloadVerdict {
    /// The compile-error text for the unresolvable case, or `None` when the build may proceed.
    /// Names the directive and the unresolvable path, and states why a startup fatal became a
    /// compile error.
    pub fn compile_error(&self) -> Option<String> {
        match self {
            PreloadVerdict::Unresolvable { requested } => Some(format!(
                "opcache.preload: failed opening required '{requested}': no such readable file. \
                 Reference PHP resolves opcache.preload during startup and FATALS there when the \
                 file is missing; elephc resolves it at compile time, so the equivalent failure is \
                 this compile error. Fix the path, or drop `--ini opcache.preload=…`."
            )),
            _ => None,
        }
    }

    /// The compile-warning text for a resolvable preload file that is NOT in the compile-time
    /// script manifest, or `None` when there is nothing to warn about. Not an error: preloading a
    /// file this program never includes/requires/autoloads is a legitimate configuration that must
    /// not break a build.
    pub fn compile_warning(&self) -> Option<String> {
        match self {
            PreloadVerdict::Preloading {
                resolved,
                in_manifest: false,
            } => Some(format!(
                "opcache.preload: '{resolved}' is not in this binary's compile-time OPcache script \
                 manifest (the entry file, its statically-resolved includes, and its autoloaded \
                 files), so it is not compiled into the binary; \
                 opcache_get_status()['preload_statistics'] reports the manifest and the \
                 symbols this binary actually bakes instead."
            )),
            _ => None,
        }
    }
}

/// Decides — AT COMPILE TIME — what `opcache.preload` means for this binary. See
/// [`PreloadVerdict`] for the verified reference matrix each arm reproduces.
///
/// The cache-enabled test comes SECOND on purpose: with the cache off, reference PHP never looks at
/// the directive at all (verified: a missing preload path with `opcache.enable_cli=0` runs cleanly),
/// so elephc must not validate the path either.
///
/// The path is resolved with `Path::canonicalize` — the same normalization `__FILE__`,
/// [`ScriptEntry::path`] and [`canonical_entry_path`] use — so manifest membership is compared on
/// equal spellings (on macOS a `/tmp/...` preload resolves to `/private/tmp/...`, exactly as
/// reference PHP reports it in `preload_statistics.scripts`). A path that canonicalizes to a
/// DIRECTORY is treated as unresolvable: reference cannot `require` a directory either.
pub fn preload_verdict(
    php_version: PhpVersion,
    web: bool,
    overrides: &[(String, String)],
    manifest: &[ScriptEntry],
) -> PreloadVerdict {
    let version_id = php_version.version_id();
    let requested = directive_str(version_id, PRELOAD_DIRECTIVE, overrides);
    if requested.is_empty() {
        return PreloadVerdict::NotPreloading;
    }
    if !opcache_cache_enabled_with_overrides(version_id, web, overrides) {
        return PreloadVerdict::NotPreloading;
    }
    let Ok(canonical) = Path::new(&requested).canonicalize() else {
        return PreloadVerdict::Unresolvable { requested };
    };
    if !canonical.is_file() {
        return PreloadVerdict::Unresolvable { requested };
    }
    let resolved = canonical.display().to_string();
    let in_manifest = manifest.iter().any(|entry| entry.path == resolved);
    PreloadVerdict::Preloading {
        resolved,
        in_manifest,
    }
}

/// The baked `opcache_get_status()['preload_statistics']` block.
///
/// THE REFERENCE SHAPE, VERIFIED on PHP 8.5.6 (`php -d opcache.enable=1 -d opcache.enable_cli=1
/// -d opcache.preload=<file> -r 'var_export(opcache_get_status());'`) — keys in THIS order:
/// `memory_consumption` (int), `functions` (list<string>), `classes` (list<string>),
/// `scripts` (list<string>).
///
/// CRUCIALLY, `functions` and `classes` are OMITTED ENTIRELY when empty — they are not reported as
/// empty arrays. VERIFIED by preloading a file containing only `<?php`: the block came back as just
/// `['memory_consumption' => 568, 'scripts' => ['…/empty.php']]`. `memory_consumption` and
/// `scripts` are always present. This renderer reproduces that omission, so elephc never emits a
/// shape reference PHP cannot produce.
///
/// TOP-LEVEL SHAPE: preloading adds `preload_statistics` and NOTHING ELSE to the status array —
/// there is no `preload_cached_scripts` key (VERIFIED by diffing the top-level key list with and
/// without preloading: `opcache_enabled, cache_full, restart_pending, restart_in_progress,
/// memory_usage, interned_strings_usage, opcache_statistics, scripts, jit` gains exactly one
/// entry, `preload_statistics`, in eighth position). `preload_statistics` is also NOT suppressed
/// by `opcache_get_status(false)`; only `scripts` is.
///
/// DOCUMENTED DIVERGENCE (verified, deliberately not reproduced): reference PHP additionally
/// inserts a SYNTHETIC `$PRELOAD$` pseudo-entry into the top-level `scripts` map (with
/// `full_path` literally `$PRELOAD$` and `memory_consumption` equal to
/// `preload_statistics.memory_consumption`), which also bumps
/// `opcache_statistics.num_cached_scripts` by one. That entry stands for the shared-memory block
/// preloading itself allocates. An elephc binary allocates no such block — its scripts are native
/// code in the executable — so fabricating a `$PRELOAD$` script would be inventing a cache entry
/// that does not exist. `scripts` and `num_cached_scripts` therefore keep reporting exactly the
/// manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreloadStatistics {
    /// Total memory the preloaded scripts occupy. Derived as Σ of the manifest entries'
    /// `memory_consumption`, so it stays coherent with the `scripts` map and `used_memory` that the
    /// same manifest feeds (implementation-defined in reference PHP too — no two builds agree).
    memory_consumption: i64,
    /// Fully-qualified user function names this binary bakes.
    functions: Vec<String>,
    /// Fully-qualified user class/interface/trait/enum names this binary bakes.
    classes: Vec<String>,
    /// Canonical paths of the preloaded scripts — the compile-time script manifest.
    scripts: Vec<String>,
}

/// Builds the `preload_statistics` block from the compile-time manifest and symbol tables, or
/// `None` when [`preload_verdict`] says this binary does not preload.
///
/// WHY THE MANIFEST IS THE RIGHT SOURCE: this repo has already committed to "the AOT binary IS the
/// cache" — `opcache_compile_file` returns `true` for manifest members, and `scripts` /
/// `num_cached_scripts` already report the manifest. Reporting preload statistics from the same
/// source is the consistent choice, and it is the ONLY source that is true: an elephc binary really
/// does hold every one of those scripts and symbols resident for its whole life, which is precisely
/// what preloading buys in reference PHP.
///
/// `functions` / `classes` are REAL SYMBOLS, not an empty-list interim: they come from
/// [`collect_preload_symbols`] walking the resolved pre-prelude AST. They are, however, the symbols
/// of the WHOLE binary rather than of the preload file specifically — an elephc binary cannot
/// separate "preloaded" from "compiled in", because everything it bakes is permanently resident.
/// That is a documented divergence, and it is a superset relationship, never a fabrication: every
/// name reported is genuinely declared by this program.
pub fn preload_statistics(
    verdict: &PreloadVerdict,
    manifest: &[ScriptEntry],
    symbols: &PreloadSymbols,
) -> Option<PreloadStatistics> {
    if !matches!(verdict, PreloadVerdict::Preloading { .. }) {
        return None;
    }
    Some(PreloadStatistics {
        memory_consumption: manifest.iter().map(|entry| entry.memory_consumption).sum(),
        functions: symbols.functions.clone(),
        classes: symbols.classes.clone(),
        scripts: manifest.iter().map(|entry| entry.path.clone()).collect(),
    })
}

/// The restricted `opcache_get_configuration()` body.
///
/// The `: array` return hint of the normal template is DROPPED and the normal array exit is KEPT
/// as a dead branch behind the always-taken `false === false` gate — the same baked-constant gate
/// idiom `GET_STATUS_TEMPLATE` uses. Two reasons, both load-bearing:
/// - It restores reference PHP's `array|false` signature, which is what the restricted function
///   genuinely is, so the idiomatic `if (is_array($c)) { … }` guard still NARROWS and compiles.
///   A single `return false;` exit types as plain `bool`, and elephc's checker then rejects
///   `count($c)` inside that guard — an over-rejection of correct defensive PHP. (Observed: the
///   first cut of this template made a probe using `is_array()`/`count()` fail to compile with
///   `count() argument must be array or Countable object`.)
/// - Keeping the REAL configuration literal (not `[]`) as the dead arm means the narrowed arm has
///   the true array shape, so `$c['directives']` still type-checks after narrowing.
///
/// At runtime the gate always fires: warning + `false`, matching reference PHP exactly.
const RESTRICTED_GET_CONFIGURATION_TEMPLATE: &str = r#"function opcache_get_configuration() {
    if (false === false) {
__RESTRICT_API_WARNING__
        return false;
    }
    return __OPCACHE_CONFIGURATION__;
}
"#;

/// The restricted `opcache_reset()` body: warning + `false` (VERIFIED against reference PHP).
const RESTRICTED_RESET_TEMPLATE: &str = r#"function opcache_reset(): bool {
__RESTRICT_API_WARNING__
    return false;
}
"#;

/// The restricted `opcache_is_script_cached()` body: warning + `false` (VERIFIED).
///
/// No dead exit is needed here (unlike the two array-returning functions): reference PHP's
/// signature is plain `bool` and the restricted value is `false`, so the static type is already
/// identical to the unrestricted one. `$filename` is coerced so the checker does not report the
/// parameter unused — the same no-op device `INVALIDATE_TEMPLATE` uses for `$force`. Reference
/// PHP likewise parses parameters BEFORE the restriction guard runs, so consuming the argument
/// first is faithful ordering.
const RESTRICTED_IS_SCRIPT_CACHED_TEMPLATE: &str = r#"function opcache_is_script_cached($filename): bool {
    $filename = (string) $filename;
__RESTRICT_API_WARNING__
    return false;
}
"#;

/// The restricted `opcache_invalidate()` body: warning + `false` (VERIFIED).
const RESTRICTED_INVALIDATE_TEMPLATE: &str = r#"function opcache_invalidate($filename, $force = false): bool {
    $filename = (string) $filename;
    $force = (bool) $force;
__RESTRICT_API_WARNING__
    return false;
}
"#;

/// The restricted `opcache_is_script_cached_in_file_cache()` body: warning + `false`.
///
/// It IS guarded, unlike its two silent siblings: php-src runs `validate_api_restriction()` in
/// `ZEND_FUNCTION(opcache_is_script_cached_in_file_cache)` before either of its own gates.
/// VERIFIED on reference PHP 8.5.6 with `-d opcache.restrict_api=/nonexistent` — it emits
/// `Warning: Zend OPcache API is restricted by "restrict_api" configuration directive` and returns
/// `false`, exactly like `opcache_is_script_cached`. (The unrestricted return is `false` too, so
/// only the WARNING distinguishes the two paths — which is precisely why the restricted body has
/// to exist rather than being folded into the normal one.)
const RESTRICTED_IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE: &str =
    r#"function opcache_is_script_cached_in_file_cache($filename): bool {
    $filename = (string) $filename;
__RESTRICT_API_WARNING__
    return false;
}
"#;

/// Splices the STDERR warning statement into one of the single-exit `RESTRICTED_*` templates
/// (`opcache_reset`, `opcache_is_script_cached`, `opcache_invalidate`,
/// `opcache_is_script_cached_in_file_cache`), whose gate body sits at one indent level.
fn render_restricted_function(template: &str) -> String {
    splice_restrict_api_warning(template, true, "    ")
}

/// Synthetic baseline of OPcache shared memory reported in-use for a freshly started
/// cache (0 cached scripts). The absolute figure is implementation-defined; only the
/// invariant `free_memory = memory_consumption - used_memory - wasted_memory` (with
/// `wasted_memory = 0`) is guaranteed exact. 6 MiB is a modest plausible baseline.
const STATUS_USED_MEMORY: i64 = 6_291_456;

/// Synthetic baseline of the interned-strings buffer reported in-use, for the DEFAULT 8 MiB
/// buffer. The invariant `free_memory = buffer_size - used_memory` is guaranteed exact; the
/// absolute figure is implementation-defined (reference PHP 8.5.6 reports 2659216 of 8388608 on
/// this host, with 11679 strings, and no two builds agree). 1 MiB is a modest plausible baseline.
///
/// It is a CEILING, not a constant: see [`render_interned_used_memory`] for the small-buffer
/// scaling that keeps `used_memory < buffer_size` and `free_memory > 0`.
const STATUS_INTERNED_USED_MEMORY: i64 = 1_048_576;

/// Synthetic plausible count of interned strings for a freshly started cache. Absolute
/// figure is implementation-defined.
const STATUS_INTERNED_NUMBER_OF_STRINGS: i64 = 4_096;

/// `interned_strings_buffer` is reported in MiB by the directive table but as a byte
/// count in the status array; this is the MiB→byte factor.
const BYTES_PER_MIB: i64 = 1_048_576;

/// Renders the `interned_strings_usage.used_memory` figure for a buffer of `buffer_size` bytes.
///
/// TWO HARD INVARIANTS, both taken from reference PHP's own arithmetic
/// (`used = top - base`, `free = end - top`, `buffer = end - base`, with `base <= top < end`):
/// `0 < used_memory < buffer_size` and therefore `free_memory > 0`. They are what make the figures
/// COHERENT rather than merely plausible, and they are exactly what the old flat constant broke:
/// with `--ini opcache.interned_strings_buffer=1` the 1 MiB baseline equalled the whole 1 MiB
/// buffer and elephc reported `free_memory => 0`, which reference PHP never does. VERIFIED on
/// reference PHP 8.5.6 with `-d opcache.interned_strings_buffer=1`:
/// `buffer_size 1048576, used_memory 824200, free_memory 224376` — used strictly below buffer.
///
/// The rule is `min(baseline, buffer_size / 2)`: the 1 MiB baseline for every buffer of 2 MiB or
/// more (so the DEFAULT 8 MiB rendering is byte-identical to what it was before this function
/// existed), and half the buffer below that. A buffer of 0 never reaches here — the whole key is
/// omitted (see [`render_interned_strings_usage`]) — so the result is always at least 1 byte for
/// any buffer of 2 bytes or more; the sub-2-byte buffers `opcache.interned_strings_buffer` can
/// never express (its unit is the MiB) are the only inputs that would degenerate.
fn render_interned_used_memory(buffer_size: i64) -> i64 {
    STATUS_INTERNED_USED_MEMORY.min(buffer_size / 2)
}

/// Looks up an integer-valued `opcache.*` directive for `version_id`, with any `--ini`
/// overrides applied (`effective_opcache_directives`). The byte-verified table always carries
/// these keys as integers, and an integer directive's override only ever parses to an integer,
/// so a miss or type mismatch is a compiler bug and panics.
fn directive_int(version_id: u32, key: &str, overrides: &[(String, String)]) -> i64 {
    effective_opcache_directives(version_id, overrides)
        .into_iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| match value {
            DirectiveValue::Int(int) => int,
            _ => panic!("opcache directive `{key}` must be an integer"),
        })
        .unwrap_or_else(|| panic!("opcache directive `{key}` must exist"))
}

/// Looks up a string-valued `opcache.*` directive for `version_id`, with any `--ini` overrides
/// applied. The byte-verified table always carries `opcache.jit` as a string, and a string
/// directive's override only ever parses to a string, so a miss or type mismatch panics.
fn directive_str(version_id: u32, key: &str, overrides: &[(String, String)]) -> String {
    effective_opcache_directives(version_id, overrides)
        .into_iter()
        .find(|(name, _)| *name == key)
        .map(|(_, value)| match value {
            DirectiveValue::Str(string) => string.to_string(),
            _ => panic!("opcache directive `{key}` must be a string"),
        })
        .unwrap_or_else(|| panic!("opcache directive `{key}` must exist"))
}

/// One `jit` sub-array's baked scalar fields, derived from the `opcache.jit` directive for the
/// compile target (see [`render_jit_status`] for the always-unavailable clamp on the other four).
struct JitStatus {
    enabled: bool,
    on: bool,
    kind: i64,
    opt_level: i64,
    opt_flags: i64,
    buffer_size: i64,
    buffer_free: i64,
}

/// Derives the `jit` sub-array of `opcache_get_status()` from the target's `opcache.jit`
/// directive (plus any `--ini` override), using the FULL reference directive → status mapping
/// for `kind` / `opt_level` / `opt_flags` (`crate::opcache::directives::effective_jit_config`,
/// which also models what an invalid spelling does), and then applying ONE clamp:
///
/// > **`enabled = false`, `on = false`, `buffer_size = 0`, `buffer_free = 0` — ALWAYS**,
/// > whatever `opcache.jit` says.
///
/// WHY THIS IS THE FAITHFUL CHOICE, not a shortcut. Reference PHP emits exactly this shape
/// itself whenever the JIT is CONFIGURED BUT UNAVAILABLE IN THIS PROCESS: it keeps reporting the
/// configured `kind`/`opt_level`/`opt_flags` (that is what was asked for) while reporting
/// `enabled`/`on` false and both buffer figures 0 (nothing was actually stood up). An elephc
/// binary is ahead-of-time compiled native code with no runtime JIT engine and no JIT buffer, so
/// "configured but unavailable" is not an approximation of its situation — it IS its situation.
/// Reporting `enabled = true` would be the divergence, since no caller could then trust
/// `$s['jit']['enabled']` as a "will my code be JIT-compiled?" probe.
///
/// THE REFERENCE EVIDENCE (re-verified on this host, PHP 8.5.6 and 8.2.31 Homebrew, macOS arm64
/// — all three are byte-identical apart from the version-dependent `kind`/`opt_level`/`opt_flags`):
/// - 8.5.6, JIT unavailable because Xdebug overrides `zend_execute_ex`, with
///   `-d opcache.jit=tracing -d opcache.jit_buffer_size=64M` →
///   `enabled=false, on=false, kind=5, opt_level=4, opt_flags=6, buffer_size=0, buffer_free=0`
///   (plus a startup `Warning: JIT is incompatible with third party extensions…`).
/// - 8.5.6, no Xdebug, JIT unavailable because there is no buffer, with
///   `-d opcache.jit=tracing -d opcache.jit_buffer_size=0` → the IDENTICAL array, silently.
/// - 8.2.31 with its DEFAULT `opcache.jit=tracing` and Xdebug loaded → the identical array
///   again, which is exactly what an elephc 8.2/8.3 target now renders with no `--ini` at all.
///
/// For contrast, the same 8.5.6 with the JIT genuinely available reports
/// `enabled=true, on=true, kind=5, opt_level=4, opt_flags=6, buffer_size=67108848,
/// buffer_free=67105256` — the shape elephc must NOT claim.
///
/// PER-VERSION CONSEQUENCE: on an 8.4/8.5 target the default `opcache.jit = disable` renders the
/// all-zero/false array, byte-identical to what this function returned before the mapping
/// existed. On an 8.2/8.3 target the default is `tracing`, so the default array now carries
/// `kind = 5, opt_level = 4, opt_flags = 6` (with the clamp still forcing the other four) instead
/// of the previous all-zero tuning fields — a correction, pinned to the 8.2.31 observation above.
///
/// `opcache.jit_buffer_size` is deliberately NOT read here: under the clamp both buffer figures
/// are 0 regardless of the directive, and reference PHP agrees (the 64M run above still reports
/// 0). The directive remains reported verbatim by `opcache_get_configuration()`/`ini_get`.
fn render_jit_status(version_id: u32, overrides: &[(String, String)]) -> JitStatus {
    let config = effective_jit_config(version_id, overrides);
    JitStatus {
        // The clamp: no runtime JIT engine exists in an AOT binary.
        enabled: false,
        on: false,
        // The full reference mapping: what was CONFIGURED is reported verbatim.
        kind: config.kind,
        opt_level: config.opt_level,
        opt_flags: config.opt_flags,
        // The clamp: no JIT buffer is ever allocated.
        buffer_size: 0,
        buffer_free: 0,
    }
}

/// Renders a boolean as its PHP-literal text.
fn render_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

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
fn render_get_status_function(
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
    let scripts_map = render_scripts_map_literal(manifest, revalidate_freq);

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
fn render_directive_value(value: &DirectiveValue) -> String {
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
fn render_php_single_quoted(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{escaped}'")
}

/// Renders the manifest's canonical paths as a flat PHP array literal
/// (`['<path1>', '<path2>']`), single-quote-escaped. Spliced into `opcache_is_script_cached`
/// and `opcache_compile_file` as the `in_array(..., true)` haystack. An empty manifest
/// renders `[]` (valid PHP; membership is always `false`).
fn render_manifest_paths_literal(manifest: &[ScriptEntry]) -> String {
    let paths: Vec<String> = manifest.iter().map(|entry| entry.path.clone()).collect();
    render_php_string_list(&paths)
}

/// Renders a list of strings as a flat PHP array literal (`['a', 'b']`), single-quote-escaped.
/// An empty slice renders `[]`.
fn render_php_string_list(values: &[String]) -> String {
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
fn render_preload_statistics_stmt(stats: &PreloadStatistics) -> String {
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
fn splice_preload_statistics(template: &str, stats: Option<&PreloadStatistics>) -> String {
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
fn render_interned_strings_usage_stmt(buffer_size: i64) -> String {
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
fn splice_interned_strings_usage(template: &str, buffer_size: i64) -> String {
    if buffer_size <= 0 {
        return template.replace("__INTERNED_STRINGS_USAGE__\n", "");
    }
    template.replace(
        "__INTERNED_STRINGS_USAGE__",
        &render_interned_strings_usage_stmt(buffer_size),
    )
}

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
fn render_scripts_map_literal(manifest: &[ScriptEntry], revalidate_freq: i64) -> String {
    let mut literal = String::from("[");
    for entry in manifest {
        let path = render_php_single_quoted(&entry.path);
        literal.push_str(&format!(
            "{path} => ['full_path' => {path}, 'hits' => 0, \
             'memory_consumption' => {mem}, \
             'last_used' => __elephc_opcache_asctime($__elephc_opcache_start_time), \
             'last_used_timestamp' => $__elephc_opcache_start_time, \
             'timestamp' => __elephc_opcache_script_timestamp({path}, {ts}), \
             'revalidate' => $__elephc_opcache_start_time + {revalidate_freq}], ",
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
fn render_configuration_literal(php_version: PhpVersion, overrides: &[(String, String)]) -> String {
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
fn render_reset_body(php_version: PhpVersion, web: bool, overrides: &[(String, String)]) -> &'static str {
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
fn render_invalidate_function(
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
    INVALIDATE_TEMPLATE
        .replace("__OPCACHE_ENABLED__", enabled)
        .replace("__MANIFEST_PATHS__", &render_manifest_paths_literal(manifest))
}

/// Renders the `opcache_is_script_cached()` body baked with the compile-time cache-enabled
/// gate and the manifest paths. Disabled → always `false`; enabled → `realpath`-normalized
/// membership in the baked manifest (see `IS_SCRIPT_CACHED_TEMPLATE`).
fn render_is_script_cached_function(
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
fn render_compile_file_function(
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

/// The CLI `ini_get` wrapper. On a plain CLI binary there is no session bridge (that is a
/// `--web`-only surface), so `ini_get` models exactly the `opcache.*` directive strings and
/// returns `false` for every other key — including `session.*` — matching reference PHP,
/// where a default `php script.php` has OPcache's directives registered but reports `false`
/// for a directive of an unloaded/absent extension. Being a real declared function is what
/// makes `function_exists('ini_get')` report `true`.
const CLI_INI_GET_TEMPLATE: &str = r#"function ini_get(string $option): string|false {
    return __elephc_opcache_ini_string($option);
}
"#;

/// The CLI `ini_set` wrapper. Every `opcache.*` directive is compile-time-baked into the AOT
/// binary and cannot be mutated at runtime, and a CLI binary models nothing else settable, so
/// `ini_set` reports failure (`false`) for every key while `ini_get` keeps returning the baked
/// value. This is exact for the PHP_INI_SYSTEM majority; the 18 PHP_INI_ALL opcache directives
/// (which reference PHP would let you set) are a documented interim divergence. Both parameters
/// are consumed so the checker does not flag them unused.
const CLI_INI_SET_TEMPLATE: &str = r#"function ini_set(string $option, $value): string|false {
    $value = (string) $value;
    if (__elephc_opcache_ini_string($option) === $value) { return false; }
    return false;
}
"#;

/// The CLI `ini_get_all` wrapper — the extension-filter dispatch, byte-modeled on php-src.
///
/// Reference PHP matches `$extension` VERBATIM against the module registry, whose keys are
/// lowercase, and does NOT case-fold (unlike `extension_loaded`, which does). So
/// `ini_get_all('zend opcache')` yields the 54 opcache entries while `ini_get_all('Zend OPcache')`
/// — the spelling `get_loaded_extensions()` reports — is "not found": an `E_WARNING`
/// (`ini_get_all(): Extension "…" cannot be found`) and `false`. A module that IS known but
/// registers no INI directives yields an EMPTY ARRAY, not `false` (verified on reference PHP
/// 8.5.6: `spl`/`json`/`ctype`/`reflection` → `[]`), which is what `__elephc_ini_module_known`
/// distinguishes.
///
/// `'core'` maps to the UNFILTERED surface, reproducing php-src's rule that Core's
/// `module_number` is 0 and the per-module filter is skipped for it. DOCUMENTED DIVERGENCE:
/// reference PHP's unfiltered surface is every registered directive of every loaded module
/// (403 on the reference build); elephc models only the directive blocks it actually owns, so
/// the unfiltered count is 54 on CLI (opcache only) and 87 under `--web` (session + opcache).
/// The rule is reproduced, the population is elephc's.
///
/// The RETURN TYPE HINT IS DELIBERATELY OMITTED (reference PHP is `array|false`): the
/// `GET_STATUS_TEMPLATE` precedent above applies verbatim — omitting the hint lets ordinary
/// union return inference handle the exits instead of leaning on union-return-type codegen.
///
/// SHAPE CONSTRAINT (an elephc codegen limitation, not a PHP-semantics one): a function that
/// writes an array-literal value on one branch and a scalar on the other into the SAME array
/// slot inside one loop miscompiles (SIGSEGV / heap exhaustion, no diagnostic). The
/// `$details` split therefore happens HERE, by dispatching to one of two single-shape
/// helpers, never inside a shared loop. See `render_opcache_ini_helpers`.
///
/// `__elephc_ini_module_known` takes `?string` rather than `string` because `$extension !== null`
/// does not currently narrow a `?string` parameter to `Str` in the checker; a nullable parameter
/// accepts the un-narrowed union while comparing identically against the string literals.
const CLI_INI_GET_ALL_TEMPLATE: &str =
    r#"function ini_get_all(?string $extension = null, bool $details = true) {
    if ($extension !== null && $extension !== 'zend opcache' && $extension !== 'core') {
        if (__elephc_ini_module_known($extension)) { return []; }
        fwrite(STDERR, 'Warning: ini_get_all(): Extension "' . $extension . '" cannot be found' . "\n");
        return false;
    }
    if ($details) { return __elephc_opcache_ini_all_details(); }
    return __elephc_opcache_ini_all_plain();
}
"#;

/// Renders `__elephc_ini_module_known(?string $m): bool` — the KNOWN-MODULE predicate the
/// `ini_get_all` extension filter uses to tell "known module with no INI directives" (`[]`)
/// from "no such module" (`E_WARNING` + `false`).
///
/// The list is derived from [`CORE_LOADED_EXTENSIONS`] — the same compile-time set that backs
/// `extension_loaded()` / `get_loaded_extensions()` — LOWERCASED at render time, so the two
/// cannot drift and the comparison is verbatim against lowercase registry keys (reference PHP
/// does NOT case-fold this argument; do not share a comparison helper with `extension_loaded`,
/// which does). `web` adds `'session'`, the extra module a `--web` binary registers.
///
/// Bridge-linked extensions (`PDO`, `hash`, …) are deliberately NOT included: they are a
/// per-compilation link-set decision made in codegen, while this prelude is rendered before
/// codegen. A program compiled `--with-pdo` therefore reports `ini_get_all('pdo')` as
/// "cannot be found" rather than `[]` — a documented interim narrower than reference PHP.
pub(crate) fn render_ini_module_known(web: bool) -> String {
    let mut names: Vec<String> = crate::codegen::lower_inst::builtins::CORE_LOADED_EXTENSIONS
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();
    if web {
        names.push("session".to_string());
    }
    let conditions: Vec<String> = names
        .iter()
        .map(|name| format!("$m === {}", render_php_single_quoted(name)))
        .collect();
    format!(
        "function __elephc_ini_module_known(?string $m): bool {{\n\
         \x20   return {};\n\
         }}\n",
        conditions.join("\n        || "),
    )
}

/// The RUNTIME per-directive environment-override helper block, written in elephc-PHP and baked
/// verbatim (it carries no per-target data — the per-directive facts are the arguments its call
/// sites pass). Injected exactly once per binary alongside whichever surface needs it; see
/// [`render_opcache_env_helpers`] for the injection rules and the reference-PHP non-parity note.
///
/// The block is four layers:
///
/// 1. `__elephc_opcache_env($u, $d)` — the LOOKUP. `$u` is the `__` spelling
///    (`ELEPHC_INI_opcache__save_comments`), `$d` the dotted one
///    (`ELEPHC_INI_opcache.save_comments`); the dotted form is consulted ONLY when the `__` form
///    is empty. Both names are rendered in RUST (`directive_env_var_names`) and passed as
///    literals, so no runtime string surgery is on the path and the derivation is unit-testable.
/// 2. `__elephc_ini_scan($v)` — the SCANNER, the PHP mirror of
///    `crate::opcache::directives::ini_scanner_value`. It rewrites the boolean-alias barewords
///    (`on`/`true`/`yes` → `'1'`, `off`/`false`/`no`/`none`/`null` → `''`) for EVERY directive
///    type, and — exactly as in Rust — it runs BEFORE every normalizer and before the raw-string
///    surface reports anything, so `ELEPHC_INI_opcache__preferred_memory_model=on` reports `'1'`.
/// 3. The NORMALIZERS — the PHP mirror of `crate::opcache::directives::parse_ini_override`, one
///    `_val` converter per type plus, for the ONE type that can still refuse a value, an `_ok`
///    predicate. Only `_pct` needs the predicate: `__elephc_ini_bool_val` and
///    `__elephc_ini_quantity` mirror handlers that CANNOT FAIL (`zend_ini_parse_bool` and
///    `zend_ini_parse_quantity`), so there is nothing for an `_ok` half to answer. The split
///    exists because a single function cannot return both "did it parse" and the parsed value
///    without a union return.
/// 4. `__elephc_opcache_env_bool` / `_int` / `_float` / `_pct` / `_str` (the TYPED surface that
///    feeds `opcache_get_configuration()['directives']`) and `__elephc_opcache_env_raw` (the RAW
///    STRING surface that feeds `ini_get()` / `ini_get_all()`). Both consult the same lookup, the
///    same scanner and the same `_ok` predicate, which is what makes the two surfaces move
///    TOGETHER: a value the typed side stores is the SCANNED value the raw side reports, and the
///    one value `_pct` rejects leaves BOTH at the compile-time default.
///
/// TWO OVERFLOW NARROWINGS, both unreachable for a real directive value and both documented
/// rather than modelled: `__elephc_ini_quantity` accumulates in PHP integers (the Rust side
/// carries a `u128` so it can reproduce `strtoul`'s ULONG_MAX-on-overflow result), and it does
/// not carry the quantity DIAGNOSTICS — `ini_override_warnings` emits those at compile time,
/// where reference PHP emits them, and a compiled binary has no startup phase to warn in.
///
/// EMPTY MEANS UNSET. `getenv` reports a missing variable as an empty string in elephc's runtime
/// (`__rt_getenv` returns `(ptr 0, len 0)` on a NULL from libc), so an environment variable set
/// to the empty string is indistinguishable from an unset one and is treated as unset — the
/// compile-time value stays. `--ini opcache.error_log=` (compile time) still stores the empty
/// string; only the runtime path has this floor. It is documented rather than worked around
/// because no mechanism can distinguish the two through `getenv`.
///
/// WHITESPACE: the normalizers use PHP's `trim()`, whose default charlist is
/// `" \t\n\r\0\x0B"`, against a Rust side that uses `str::trim` (`" \t\n\r\x0B\x0C"` plus
/// Unicode). The two disagree only on a leading/trailing NUL or form feed, which no directive
/// value carries.
const ENV_OVERRIDE_HELPERS: &str = r#"function __elephc_opcache_env(string $u, string $d): string {
    $v = (string) getenv($u);
    if ($v !== '') { return $v; }
    return (string) getenv($d);
}
function __elephc_ini_scan(string $v): string {
    $l = strtolower(trim($v));
    if ($l === 'on' || $l === 'true' || $l === 'yes') { return '1'; }
    if ($l === 'off' || $l === 'false' || $l === 'no' || $l === 'none' || $l === 'null') { return ''; }
    return $v;
}
function __elephc_ini_bool_val(string $v): bool {
    $l = strtolower($v);
    if ($l === 'true' || $l === 'yes' || $l === 'on') { return true; }
    return __elephc_ini_atoi($v) !== 0;
}
function __elephc_ini_isspace(string $c): bool {
    $o = ord($c);
    return $o === 32 || ($o >= 9 && $o <= 13);
}
function __elephc_ini_digit(string $c, int $radix): int {
    $o = ord($c);
    $d = -1;
    if ($o >= 48 && $o <= 57) { $d = $o - 48; }
    if ($o >= 97 && $o <= 122) { $d = $o - 87; }
    if ($o >= 65 && $o <= 90) { $d = $o - 55; }
    if ($d < 0 || $d >= $radix) { return -1; }
    return $d;
}
function __elephc_ini_quantity(string $v): int {
    $n = strlen($v);
    if ($n === 0) { return 0; }
    $s = 0;
    while ($s < $n && __elephc_ini_isspace(substr($v, $s, 1))) { $s = $s + 1; }
    $e = $n;
    while ($e > $s && __elephc_ini_isspace(substr($v, $e - 1, 1))) { $e = $e - 1; }
    if ($s >= $e) { return 0; }
    $neg = substr($v, $s, 1) === '-';
    $i = $s;
    $c = substr($v, $i, 1);
    if ($c === '-' || $c === '+') { $i = $i + 1; }
    if ($i >= $e) { return 0; }
    $o = ord(substr($v, $i, 1));
    if ($o < 48 || $o > 57) { return 0; }
    $radix = 10;
    if ($o === 48) {
        $radix = 8;
        if ($i + 1 < $e) {
            $p = strtolower(substr($v, $i + 1, 1));
            if ($p === 'x') { $radix = 16; $i = $i + 2; }
            if ($p === 'b') { $radix = 2; $i = $i + 2; }
        }
    }
    if ($i >= $e) { return 0; }
    if (__elephc_ini_digit(substr($v, $i, 1), $radix) < 0) { return 0; }
    $acc = 0;
    while ($i < $e) {
        $d = __elephc_ini_digit(substr($v, $i, 1), $radix);
        if ($d < 0) { break; }
        $acc = $acc * $radix + $d;
        $i = $i + 1;
    }
    if ($neg) { $acc = -$acc; }
    if ($i >= $e) { return $acc; }
    $last = strtolower(substr($v, $e - 1, 1));
    if ($last === 'k') { return $acc * 1024; }
    if ($last === 'm') { return $acc * 1048576; }
    if ($last === 'g') { return $acc * 1073741824; }
    return $acc;
}
function __elephc_ini_atoi(string $v): int {
    $s = ltrim($v);
    $n = strlen($s);
    $i = 0;
    $neg = false;
    $c = substr($s, 0, 1);
    if ($c === '-') { $neg = true; $i = 1; }
    if ($c === '+') { $i = 1; }
    $acc = 0;
    $seen = 0;
    while ($i < $n) {
        $o = ord(substr($s, $i, 1));
        if ($o < 48 || $o > 57) { break; }
        if ($seen < 18) { $acc = $acc * 10 + ($o - 48); }
        $seen = $seen + 1;
        $i = $i + 1;
    }
    if ($neg) { return -$acc; }
    return $acc;
}
function __elephc_ini_pct_ok(string $v): bool {
    $p = __elephc_ini_atoi($v);
    return $p > 0 && $p <= 50;
}
function __elephc_ini_pct_val(string $v): float {
    return __elephc_ini_atoi($v) / 100.0;
}
function __elephc_opcache_env_bool(string $u, string $d, bool $def): bool {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return __elephc_ini_bool_val(__elephc_ini_scan($v));
}
function __elephc_opcache_env_int(string $u, string $d, int $def): int {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return __elephc_ini_quantity(__elephc_ini_scan($v));
}
function __elephc_opcache_env_float(string $u, string $d, float $def): float {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return (float) trim(__elephc_ini_scan($v));
}
function __elephc_opcache_env_trunc(string $u, string $d, int $def): int {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    return (int) (float) trim(__elephc_ini_scan($v));
}
function __elephc_opcache_env_pct(string $u, string $d, float $def): float {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    if (__elephc_ini_pct_ok($s)) { return __elephc_ini_pct_val($s); }
    return $def;
}
function __elephc_opcache_env_str(string $u, string $d, string $def): string {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    return $s;
}
function __elephc_opcache_env_raw(string $u, string $d, string $t, string $def): string {
    $v = __elephc_opcache_env($u, $d);
    if ($v === '') { return $def; }
    $s = __elephc_ini_scan($v);
    if ($t === 'p') { if (__elephc_ini_pct_ok($s)) { return $s; } return $def; }
    return $s;
}
"#;

/// Returns the RUNTIME environment-override helper block ([`ENV_OVERRIDE_HELPERS`]).
///
/// WHY THIS EXISTS AT ALL. Every `opcache.*` directive is compiled into the binary, so the
/// natural analogue of `php -d` is elephc's compile-time `--ini KEY=VALUE`. That leaves no way to
/// re-point a directive on an ALREADY-BUILT binary, which is exactly what a deployment needs
/// (`ELEPHC_INI_opcache__save_comments=0 ./app`). Reference PHP has no per-directive environment
/// override to copy — VERIFIED on 8.5.6 that `PHP_INI_opcache_jit`, `opcache_jit` and
/// `opcache.jit` in the environment all leave `ini_get('opcache.jit')` at the compiled default —
/// so this is a documented elephc EXTENSION, not a parity feature. Precedence:
/// baked default → `--ini` (compile time) → `ELEPHC_INI_*` (runtime, wins).
///
/// WHY IT IS PHP RATHER THAN RUST. A plain CLI binary links NO Rust staticlib — every elephc
/// runtime is an opt-in bridge selected in `crate::linker` — so a Rust-side override table would
/// force every binary to link one (killing pay-for-use) or need a hand-written `__rt_*` helper in
/// assembly for four targets. `getenv` is already a first-class codegen builtin with a CONCRETE
/// `Str` EIR result type, available identically on CLI and `--web`, so baking the lookup as PHP
/// costs nothing and works on every target the compiler already supports.
///
/// INJECTED EXACTLY ONCE. The block declares plain functions, so a second copy is a redeclaration
/// error. Ownership mirrors `render_opcache_ini_helpers`: under `--web` the web prelude bakes it
/// (see `crate::web_prelude`), and `inject_if_used` emits it only when NOT web. On CLI it is
/// emitted when `opcache_get_configuration` is injected (its directives array calls the typed
/// helpers — including through the RESTRICTED template's dead array exit, which still has to
/// resolve) or when the `opcache.*` INI dispatcher is injected (its raw-string arms call
/// `__elephc_opcache_env_raw`), and never twice.
pub(crate) fn render_opcache_env_helpers() -> String {
    ENV_OVERRIDE_HELPERS.to_string()
}

/// Renders the PHP expression that yields directive `name`'s effective TYPED value at runtime:
/// the compile-time literal for a directive outside the runtime-override scope
/// ([`directive_runtime_overridable`]), otherwise a call into the typed environment helper with
/// the two environment-variable spellings and the compile-time value as the default.
///
/// `value` is the EFFECTIVE compile-time value (defaults with `--ini` already applied), which is
/// what makes the precedence chain baked default → `--ini` → env fall out for free: the env
/// helper's `$def` argument IS the `--ini`-resolved value, so an unset or invalid environment
/// variable reproduces today's output exactly.
fn render_directive_value_expr(name: &str, value: &DirectiveValue) -> String {
    let literal = render_directive_value(value);
    if !directive_runtime_overridable(name) {
        return literal;
    }
    let (under, dotted) = directive_env_var_names(name);
    let helper = match directive_env_type_code(name, value) {
        'b' => "__elephc_opcache_env_bool",
        'i' => "__elephc_opcache_env_int",
        'p' => "__elephc_opcache_env_pct",
        'f' => "__elephc_opcache_env_float",
        // `opcache.jit_prof_threshold` in the 8.2 profile ONLY: a `zend_strtod` READ whose value is
        // REPORTED truncated to an int (php-src 8.2 uses `add_assoc_long` on a `double` field).
        // See `crate::opcache::directives::JIT_PROF_THRESHOLD`.
        't' => "__elephc_opcache_env_trunc",
        _ => "__elephc_opcache_env_str",
    };
    format!(
        "{helper}({}, {}, {literal})",
        render_php_single_quoted(&under),
        render_php_single_quoted(&dotted),
    )
}

/// Renders the shared `opcache.*` INI helper functions for the compile target, baked from the
/// version-keyed directive table so CLI and `--web` share one source of truth:
///
/// - `__elephc_opcache_ini_string(string $option): string|false` — the RAW INI STRING for an
///   `opcache.*` directive (what `ini_get` reports), or `false` for a non-opcache key.
/// - `__elephc_opcache_ini_access(string $option): int` — the `PHP_INI_*` access bitmask
///   (`7`/`4`) for an `opcache.*` directive, or `-1` for a non-opcache key.
/// - `__elephc_opcache_ini_keys(): array` — the `opcache.*` directive names SORTED ASCENDING.
/// - `__elephc_opcache_ini_all_details(): array` — the whole block as
///   `['global_value' => rawstr, 'local_value' => rawstr, 'access' => N]` entries.
/// - `__elephc_opcache_ini_all_plain(): array` — the whole block as flat raw strings.
///
/// The raw strings and access levels come from `directive_ini_string` / `directive_access`
/// (byte-verified against reference PHP 8.5.6), so this is a pure projection of the same table
/// that backs `opcache_get_configuration()`. Rendered as `if`-chains (matching the session INI
/// dispatcher's proven shape) so no `switch`/`match` lowering is on the path.
///
/// KEY ORDER: `ini_get_all` reports its keys SORTED ASCENDING (reference PHP 8.5.6), so the
/// rendered key list is a sorted COPY. `opcache_directives()` itself keeps REGISTRATION order,
/// which is what `opcache_get_configuration()['directives']` reports and is byte-correct there
/// — it must not be reordered. Only this projection sorts.
///
/// TWO ALL-HELPERS, NOT ONE `$details` LOOP: a function that writes an ARRAY-LITERAL value on
/// one branch and a SCALAR on the other into the SAME array slot inside one loop miscompiles in
/// elephc's codegen — SIGSEGV or heap exhaustion, with no diagnostic. (Reproduced: a single
/// dual-shape `ini_get_all` loop made `ini_get_all(null, false)` exit 139 with no output, and
/// crashed the `--web` worker into an empty HTTP reply.) The rule is ONE VALUE SHAPE PER
/// FUNCTION, so the `$details` branch is resolved by the CALLER picking a helper.
pub(crate) fn render_opcache_ini_helpers(
    php_version: PhpVersion,
    overrides: &[(String, String)],
) -> String {
    let version_id = php_version.version_id();
    let directives = opcache_directives(version_id);

    // __elephc_opcache_ini_string: raw INI string per opcache key; false for anything else. The
    // raw string is the user's `--ini` override verbatim when validly overridden, else the
    // default projection (`effective_directive_ini_string`). Access levels and the key list do
    // not vary with overrides, so only the string arm consults them.
    let mut string_arms = String::new();
    for (name, value) in &directives {
        let raw = effective_directive_ini_string(name, value, overrides);
        // RUNTIME env override (`ELEPHC_INI_*`) for the reporting-only directives: the arm returns
        // a call that yields the environment value VERBATIM when it parses for the directive's
        // type and the compile-time raw string otherwise. Excluded directives keep the plain
        // literal — see `directive_runtime_overridable` for why honoring them here would make the
        // binary contradict its own `opcache_get_status()`.
        let arm = if directive_runtime_overridable(name) {
            // The type code is read off the DEFAULT value: `parse_ini_override` preserves the
            // `DirectiveValue` variant, so a `--ini` override never changes a directive's type.
            let (under, dotted) = directive_env_var_names(name);
            format!(
                "__elephc_opcache_env_raw({}, {}, '{}', {})",
                render_php_single_quoted(&under),
                render_php_single_quoted(&dotted),
                directive_env_type_code(name, value),
                render_php_single_quoted(&raw),
            )
        } else {
            render_php_single_quoted(&raw)
        };
        string_arms.push_str(&format!(
            "    if ($option === {}) {{ return {arm}; }}\n",
            render_php_single_quoted(name),
        ));
    }

    // __elephc_opcache_ini_null: whether ini_get_all() reports this directive's global_value /
    // local_value as PHP `null` rather than a string. Reference PHP does that for exactly the
    // directives php-src registers with a C NULL default AND that were never assigned a value —
    // `opcache.file_cache` is the only one in the block (see `directive_ini_null_default`).
    //
    // THE `?string` RETURN HINT ON `__elephc_opcache_ini_detail_value` IS LOAD-BEARING. Without an
    // explicit union hint elephc infers the function's return as plain `Str` and COERCES the
    // `return null` to `''` — reproduced: the same body typed `?string` var_dumps `NULL` and typed
    // implicitly var_dumps `string(0) ""`, which is exactly the bug being fixed here.
    //
    // A COMPILE-TIME `--ini opcache.file_cache=<v>` ASSIGNS it, so the arm collapses to `false`
    // (reference reports `''`/`''` for `-d opcache.file_cache=` and `'/x'`/`'/x'` for
    // `-d opcache.file_cache=/x`; only the untouched run reports NULL/NULL). Otherwise the arm
    // consults the RUNTIME environment override with the same "empty means unset" rule the rest of
    // the `ELEPHC_INI_*` surface uses, so `ELEPHC_INI_opcache__file_cache=/x` flips it to a string
    // exactly as `-d` would.
    let null_arms: Vec<String> = directives
        .iter()
        .filter(|(name, _)| directive_ini_null_default(name))
        .map(|(name, _)| {
            let condition = if latest_ini_override(overrides, name).is_some() {
                "false".to_string()
            } else {
                let (under, dotted) = directive_env_var_names(name);
                format!(
                    "__elephc_opcache_env({}, {}) === ''",
                    render_php_single_quoted(&under),
                    render_php_single_quoted(&dotted),
                )
            };
            format!(
                "    if ($option === {}) {{ return {condition}; }}\n",
                render_php_single_quoted(name),
            )
        })
        .collect();
    let null_arms = null_arms.concat();

    // __elephc_opcache_ini_access: 7 for the PHP_INI_ALL directives, 4 for the rest, and -1
    // for a non-opcache key (detected by the string dispatcher returning false).
    let all_conditions: Vec<String> = directives
        .iter()
        .filter(|(name, _)| directive_access(name) == 7)
        .map(|(name, _)| format!("$option === {}", render_php_single_quoted(name)))
        .collect();
    let all_expr = all_conditions.join("\n        || ");

    // __elephc_opcache_ini_keys: the directive-name list for ini_get_all, SORTED ASCENDING to
    // match reference PHP's ini_get_all key order. This is a sorted COPY: the table backing
    // opcache_get_configuration() keeps registration order and is left untouched.
    let mut ini_keys: Vec<&str> = directives.iter().map(|(name, _)| *name).collect();
    ini_keys.sort_unstable();
    let mut keys_literal = String::from("[");
    for (index, name) in ini_keys.iter().enumerate() {
        if index > 0 {
            keys_literal.push_str(", ");
        }
        keys_literal.push_str(&render_php_single_quoted(name));
    }
    keys_literal.push(']');

    format!(
        "function __elephc_opcache_ini_string(string $option): string|false {{\n\
         {string_arms}    return false;\n\
         }}\n\
         function __elephc_opcache_ini_null(string $option): bool {{\n\
         {null_arms}    return false;\n\
         }}\n\
         function __elephc_opcache_ini_detail_value(string $option): ?string {{\n\
         \x20   if (__elephc_opcache_ini_null($option)) {{ return null; }}\n\
         \x20   $__elephc_raw = (string) __elephc_opcache_ini_string($option);\n\
         \x20   return $__elephc_raw;\n\
         }}\n\
         function __elephc_opcache_ini_access(string $option): int {{\n\
         \x20   if (__elephc_opcache_ini_string($option) === false) {{ return -1; }}\n\
         \x20   if ({all_expr}) {{ return 7; }}\n\
         \x20   return 4;\n\
         }}\n\
         function __elephc_opcache_ini_keys(): array {{\n\
         \x20   return {keys_literal};\n\
         }}\n\
         function __elephc_opcache_ini_all_details(): array {{\n\
         \x20   $__elephc_all = [];\n\
         \x20   foreach (__elephc_opcache_ini_keys() as $__elephc_k) {{\n\
         \x20       $__elephc_v = __elephc_opcache_ini_detail_value($__elephc_k);\n\
         \x20       $__elephc_all[$__elephc_k] = ['global_value' => $__elephc_v, 'local_value' => $__elephc_v, 'access' => __elephc_opcache_ini_access($__elephc_k)];\n\
         \x20   }}\n\
         \x20   return $__elephc_all;\n\
         }}\n\
         function __elephc_opcache_ini_all_plain(): array {{\n\
         \x20   $__elephc_all = [];\n\
         \x20   foreach (__elephc_opcache_ini_keys() as $__elephc_k) {{\n\
         \x20       $__elephc_all[$__elephc_k] = __elephc_opcache_ini_detail_value($__elephc_k);\n\
         \x20   }}\n\
         \x20   return $__elephc_all;\n\
         }}\n"
    )
}

/// Prepends the OPcache prelude functions (`opcache_get_configuration`, `opcache_reset`,
/// `opcache_get_status`) each when the program references it and does not declare its
/// own, so unrelated binaries pay nothing and a user definition is not clobbered. `web`
/// is the compiler SAPI flag that selects the baked enabled/disabled state for
/// `opcache_reset` and `opcache_get_status` (a disabled cache makes `opcache_get_status`
/// return `false`, matching reference `php script.php`). The prelude
/// is hoisted function declarations only, so prepending does not change top-level
/// execution order. The rendered source is static data, so a tokenize/parse failure is
/// a compiler bug and panics rather than degrading silently.
///
/// `manifest` is the compile-time OPcache script manifest (see `ScriptEntry`). It is baked into
/// `opcache_get_status` (the `scripts` map and cached-script counts), `opcache_is_script_cached`,
/// and `opcache_compile_file`. An empty manifest renders valid PHP (empty `scripts` map, `false`
/// membership).
///
/// At THIS point the manifest is necessarily a PLACEHOLDER: the autoloaded file set does not
/// exist until `autoload::run`, which runs after name resolution — but the declarations must
/// exist BEFORE name resolution or a namespaced caller would not resolve to them. The returned
/// [`ManifestBakeSites`] records which manifest-dependent functions were injected so
/// [`bake_manifest`] can re-render exactly those against the complete manifest. See
/// [`bake_manifest`] for the full argument and the soundness of the substitution.
///
/// `entry_path` is the canonicalized entry script (`canonical_entry_path`), used ONLY for the
/// `opcache.restrict_api` decision. When that directive denies (see `restrict_api_denies`), the
/// five RESTRICTED functions render as warning + `false` instead of their normal bodies.
/// `opcache_compile_file` is deliberately NOT among them: reference PHP does not guard it —
/// VERIFIED on PHP 8.5.6, where `restrict_api=/nonexistent` still returns `true` from
/// `opcache_compile_file()` with no warning, while all five others warn and return `false`.
/// With the default empty `restrict_api` every function renders byte-identically to before.
///
/// `preload` is the compile-time `opcache.preload` block ([`preload_statistics`]), or `None` when
/// this binary does not preload (the default, and every disabled-cache binary). It only ever adds
/// the `preload_statistics` key to `opcache_get_status`; `None` renders byte-identically to before
/// the directive was supported. The UNRESOLVABLE case never reaches here — `crate::pipeline`
/// turns [`PreloadVerdict::compile_error`] into a hard compile failure BEFORE injection, exactly
/// as reference PHP fatals at startup before running a line of the script, and independently of
/// whether the program calls any OPcache function at all.
pub fn inject_if_used(
    program: Program,
    php_version: PhpVersion,
    web: bool,
    entry_path: Option<&str>,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
    preload: Option<&PreloadStatistics>,
) -> (Program, ManifestBakeSites) {
    let mut bodies = String::new();
    let mut sites = ManifestBakeSites {
        restricted: restrict_api_denies(entry_path, php_version.version_id(), overrides),
        ..ManifestBakeSites::default()
    };

    // One compile-time decision shared by all five restricted functions.
    let restricted = sites.restricted;

    // Whether the runtime `ELEPHC_INI_*` helper block has to be emitted here. It is needed by
    // `opcache_get_configuration`'s directives array (including the RESTRICTED template's dead
    // array exit, which still has to name-resolve) and by the `opcache.*` INI dispatcher's
    // raw-string arms. Under `--web` the web prelude owns the block — emitting it here too would
    // be a redeclaration — so the flag is only ever consulted on the `!web` path below.
    let mut needs_env_helpers = detect::program_references(&program, GET_CONFIGURATION_FN)
        && !detect::program_declares(&program, GET_CONFIGURATION_FN);

    if detect::program_references(&program, GET_CONFIGURATION_FN)
        && !detect::program_declares(&program, GET_CONFIGURATION_FN)
    {
        let template = if restricted {
            splice_restrict_api_warning(RESTRICTED_GET_CONFIGURATION_TEMPLATE, true, "        ")
        } else {
            GET_CONFIGURATION_TEMPLATE.to_string()
        };
        bodies.push_str(&template.replace(
            "__OPCACHE_CONFIGURATION__",
            &render_configuration_literal(php_version, overrides),
        ));
    }

    if detect::program_references(&program, RESET_FN)
        && !detect::program_declares(&program, RESET_FN)
    {
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_RESET_TEMPLATE)
        } else {
            RESET_TEMPLATE
                .replace("__OPCACHE_RESET_ENABLED__", render_reset_body(php_version, web, overrides))
        });
    }

    if detect::program_references(&program, GET_STATUS_FN)
        && !detect::program_declares(&program, GET_STATUS_FN)
    {
        // Manifest-dependent even when restricted: the restricted gate keeps the array exit
        // (as a dead branch) so the `array|false` signature survives, and that exit still
        // carries the `scripts` map and the cached-script counts.
        sites.get_status = true;
        bodies.push_str(&render_get_status_function(
            php_version,
            web,
            manifest,
            overrides,
            restricted,
            preload,
        ));
    }

    if detect::program_references(&program, IS_SCRIPT_CACHED_FN)
        && !detect::program_declares(&program, IS_SCRIPT_CACHED_FN)
    {
        // The restricted body is a bare warning + `false` with no manifest in it, so only the
        // normal body is a bake site.
        sites.is_script_cached = !restricted;
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_IS_SCRIPT_CACHED_TEMPLATE)
        } else {
            render_is_script_cached_function(php_version, web, manifest, overrides)
        });
    }

    if detect::program_references(&program, INVALIDATE_FN)
        && !detect::program_declares(&program, INVALIDATE_FN)
    {
        // The restricted body warns and returns `false` with no manifest in it, so only the
        // normal body is a bake site.
        sites.invalidate = !restricted;
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_INVALIDATE_TEMPLATE)
        } else {
            render_invalidate_function(php_version, web, manifest, overrides)
        });
    }

    // NOT restricted in reference PHP (verified) — always the normal body.
    if detect::program_references(&program, COMPILE_FILE_FN)
        && !detect::program_declares(&program, COMPILE_FILE_FN)
    {
        sites.compile_file = true;
        bodies.push_str(&render_compile_file_function(php_version, web, manifest, overrides));
    }

    if detect::program_references(&program, IS_SCRIPT_CACHED_IN_FILE_CACHE_FN)
        && !detect::program_declares(&program, IS_SCRIPT_CACHED_IN_FILE_CACHE_FN)
    {
        // Carries no manifest either way (elephc has no file cache), so it is never a bake site.
        bodies.push_str(&if restricted {
            render_restricted_function(RESTRICTED_IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE)
        } else {
            IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE.to_string()
        });
    }

    // NOT restricted in reference PHP (verified) — always the normal body, and it needs no baked
    // compile-time data at all.
    if detect::program_references(&program, JIT_BLACKLIST_FN)
        && !detect::program_declares(&program, JIT_BLACKLIST_FN)
    {
        bodies.push_str(JIT_BLACKLIST_TEMPLATE);
    }

    // The in-process OPcache state block, emitted ONCE for whichever of the five state-touching
    // functions were injected. `opcache_get_status` needs it for `restart_pending` and for the
    // `scripts` map's `timestamp` / `last_used`; `opcache_reset` for the restart latch; the three
    // path functions for the discard set. Unlike the INI/env helpers this is NOT web-gated,
    // because `crate::web_prelude` never emits a copy — the OPcache functions themselves are
    // injected from here under `--web` too.
    let needs_state_helpers = [
        RESET_FN,
        GET_STATUS_FN,
        IS_SCRIPT_CACHED_FN,
        INVALIDATE_FN,
        COMPILE_FILE_FN,
    ]
    .iter()
    .any(|name| {
        detect::program_references(&program, name) && !detect::program_declares(&program, name)
    });
    if needs_state_helpers {
        bodies.push_str(&render_opcache_state_helpers());
    }

    // The `opcache.*` INI surface (`ini_get`/`ini_set`/`ini_get_all`) for CLI binaries.
    // Under `--web` the session-aware definitions in `web_prelude` own these three names
    // (and consult the shared opcache helpers themselves), so the CLI wrappers are injected
    // only when NOT web — otherwise a `--web` build would redeclare `ini_get`. Each wrapper
    // is pay-for-use with its own redeclaration guard; the shared helpers are injected once
    // whenever any of the three is.
    if !web {
        let ini_get_used = detect::program_references(&program, "ini_get")
            && !detect::program_declares(&program, "ini_get");
        let ini_set_used = detect::program_references(&program, "ini_set")
            && !detect::program_declares(&program, "ini_set");
        let ini_get_all_used = detect::program_references(&program, "ini_get_all")
            && !detect::program_declares(&program, "ini_get_all");
        if ini_get_used || ini_set_used || ini_get_all_used {
            needs_env_helpers = true;
            bodies.push_str(&render_opcache_ini_helpers(php_version, overrides));
            if ini_get_used {
                bodies.push_str(CLI_INI_GET_TEMPLATE);
            }
            if ini_set_used {
                bodies.push_str(CLI_INI_SET_TEMPLATE);
            }
            if ini_get_all_used {
                // The known-module predicate is only reachable from ini_get_all's extension
                // filter, so it is injected with it rather than with the shared helpers.
                bodies.push_str(&render_ini_module_known(false));
                bodies.push_str(CLI_INI_GET_ALL_TEMPLATE);
            }
        }
    }

    // The runtime `ELEPHC_INI_*` helper block, emitted ONCE and only on the CLI path (under
    // `--web` the web prelude bakes it — see `render_opcache_env_helpers`). It is appended last
    // because these are hoisted function declarations: order among them is irrelevant, and
    // appending keeps every earlier body byte-identical to what it rendered before.
    if !web && needs_env_helpers {
        bodies.push_str(&render_opcache_env_helpers());
    }

    if bodies.is_empty() {
        return (program, ManifestBakeSites::default());
    }

    let src = format!("<?php\n{bodies}");
    let tokens = crate::lexer::tokenize(&src).expect("opcache prelude must tokenize");
    let mut combined = crate::parser::parse(&tokens).expect("opcache prelude must parse");
    combined.extend(program);
    (combined, sites)
}

/// Which manifest-dependent OPcache functions [`inject_if_used`] actually injected, and under
/// which `opcache.restrict_api` verdict — everything [`bake_manifest`] needs to re-render them
/// against the complete script manifest.
///
/// Recording the sites (rather than having `bake_manifest` re-scan for the three names) is what
/// makes baking safe when the program declares its OWN `opcache_get_status()`: `inject_if_used`
/// skips injection in that case, the corresponding flag stays `false`, and the user's function
/// is never touched.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ManifestBakeSites {
    /// `opcache_get_status` was injected (its `scripts` map, cached-script counts, memory
    /// figures and `preload_statistics` block are all manifest-derived).
    get_status: bool,
    /// `opcache_is_script_cached` was injected with its NORMAL body (the restricted body
    /// carries no manifest).
    is_script_cached: bool,
    /// `opcache_compile_file` was injected. Reference PHP never restricts this one, so its body
    /// is always the manifest-carrying form.
    compile_file: bool,
    /// `opcache_invalidate` was injected with its NORMAL body. It reads the manifest to decide
    /// whether a FORCED call records a discard (see `INVALIDATE_TEMPLATE`); the restricted body
    /// carries no manifest.
    invalidate: bool,
    /// The compile-time `opcache.restrict_api` verdict ([`restrict_api_denies`]), replayed so
    /// the re-rendered `opcache_get_status` keeps the same gate it was injected with.
    restricted: bool,
}

impl ManifestBakeSites {
    /// Whether there is nothing to bake — no manifest-dependent function was injected.
    pub fn is_empty(&self) -> bool {
        !self.get_status && !self.is_script_cached && !self.compile_file && !self.invalidate
    }
}

/// Re-renders the manifest-dependent OPcache functions against the COMPLETE script manifest and
/// substitutes them into the program, replacing the placeholder declarations `inject_if_used`
/// left behind. A no-op when no such function was injected.
///
/// # Why injection and baking are split
///
/// The pipeline order is `resolver::resolve` → `opcache_prelude::inject_if_used` →
/// `name_resolver::resolve` → `autoload::run`. The manifest's third group — the autoloaded
/// files — is produced by `autoload::run`, which runs LAST, because PSR-4 resolution is a
/// fixpoint over CANONICAL class FQNs and those only exist after name resolution. So the
/// manifest is not knowable at the injection point.
///
/// Moving `inject_if_used` after `autoload::run` would fix that and break something worse: a
/// namespaced caller. `name_resolver` resolves an unqualified `opcache_get_status()` written
/// inside `namespace App;` by consulting the symbol table it collected from the program — if
/// the declaration is not there yet, the call does not resolve to the injected function. The
/// DECLARATION must therefore exist before name resolution; only its BODY needs the manifest.
///
/// # The mechanism, and why it is sound
///
/// `inject_if_used` renders each function with the manifest it can see at that point (entry +
/// includes + Composer `autoload.files`) — a valid, parseable, self-consistent placeholder.
/// This pass then re-renders the same functions from the same templates with the full manifest,
/// parses them, and swaps the whole top-level `FunctionDecl` statement in by name.
///
/// The freshly parsed declarations are run through `name_resolver::resolve` in isolation before
/// substitution. That is the same device `autoload::load_autoloaded_file` already uses to splice
/// a file parsed after the main name-resolution pass, and it is exact here for a stronger
/// reason: these bodies live in the GLOBAL namespace with no `use` imports, so every name in
/// them (`realpath`, `in_array`, `date`, `fwrite`, `STDERR`) resolves identically whether it is
/// resolved with the whole program's symbol table or with its own — there is no namespace or
/// import context that could differ. `substitutes_a_name_resolution_identical_body` pins that
/// equality on the real rendered bodies.
///
/// Baking runs BEFORE `optimize::fold_constants` and the type checker, so the substituted
/// literals go through every later pass exactly as the placeholder would have.
///
/// A recorded site that is not found is a COMPILER BUG (something moved or dropped a
/// declaration this module injected), and panics rather than silently shipping a binary whose
/// `scripts` map omits the autoloaded files — the same policy as the `expect`s above.
pub fn bake_manifest(
    program: Program,
    sites: &ManifestBakeSites,
    php_version: PhpVersion,
    web: bool,
    manifest: &[ScriptEntry],
    overrides: &[(String, String)],
    preload: Option<&PreloadStatistics>,
) -> Program {
    if sites.is_empty() {
        return program;
    }

    let mut baked: Vec<(&str, Stmt)> = Vec::new();
    if sites.get_status {
        baked.push((
            GET_STATUS_FN,
            parse_baked_function(&render_get_status_function(
                php_version,
                web,
                manifest,
                overrides,
                sites.restricted,
                preload,
            )),
        ));
    }
    if sites.is_script_cached {
        baked.push((
            IS_SCRIPT_CACHED_FN,
            parse_baked_function(&render_is_script_cached_function(
                php_version,
                web,
                manifest,
                overrides,
            )),
        ));
    }
    if sites.compile_file {
        baked.push((
            COMPILE_FILE_FN,
            parse_baked_function(&render_compile_file_function(
                php_version,
                web,
                manifest,
                overrides,
            )),
        ));
    }
    if sites.invalidate {
        baked.push((
            INVALIDATE_FN,
            parse_baked_function(&render_invalidate_function(
                php_version,
                web,
                manifest,
                overrides,
            )),
        ));
    }

    let mut program = program;
    for stmt in program.iter_mut() {
        let StmtKind::FunctionDecl { name, .. } = &stmt.kind else {
            continue;
        };
        // PHP function names are case-insensitive; the templates declare them lowercase.
        let name = name.to_ascii_lowercase();
        let Some(index) = baked.iter().position(|(fn_name, _)| *fn_name == name) else {
            continue;
        };
        *stmt = baked.swap_remove(index).1;
    }

    assert!(
        baked.is_empty(),
        "opcache prelude: injected function(s) {:?} vanished before manifest baking",
        baked.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
    );
    program
}

/// Parses one rendered OPcache function declaration and name-resolves it in isolation,
/// returning the single top-level `FunctionDecl` statement [`bake_manifest`] substitutes.
///
/// The rendered source is static compiler data, so a tokenize/parse/resolve failure is a
/// compiler bug and panics rather than degrading silently — matching `inject_if_used`.
fn parse_baked_function(body: &str) -> Stmt {
    let src = format!("<?php\n{body}");
    let tokens = crate::lexer::tokenize(&src).expect("opcache prelude must tokenize");
    let parsed = crate::parser::parse(&tokens).expect("opcache prelude must parse");
    let mut resolved =
        crate::name_resolver::resolve(parsed).expect("opcache prelude must name-resolve");
    assert_eq!(
        resolved.len(),
        1,
        "opcache prelude: a baked function must render exactly one top-level declaration",
    );
    resolved.remove(0)
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Verifies the rendered configuration literal parses and reflects the version
    //! deltas, and that injection is pay-for-use.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.

    use super::*;

    /// Parses source the way `inject_if_used` sees it.
    fn parse(source: &str) -> Program {
        let tokens = crate::lexer::tokenize(source).expect("test source must tokenize");
        crate::parser::parse(&tokens).expect("test source must parse")
    }

    /// The rendered 8.5 literal tokenizes/parses and carries the 8.5 markers.
    ///
    /// The EXCLUDED directives (`opcache.jit`, `opcache.memory_consumption`, …) are asserted as
    /// PLAIN LITERALS — that is the runtime-override scope rule showing up in the rendered text —
    /// while a reporting-only directive carries its env-override call with the compile-time value
    /// as the `$def` argument.
    #[test]
    fn renders_parsable_php85_literal() {
        let literal = render_configuration_literal(PhpVersion::Php85, &[]);
        assert!(literal.contains("'opcache.jit' => 'disable'"));
        assert!(literal.contains("'opcache.memory_consumption' => 134217728"));
        assert!(literal.contains(
            "'opcache.max_wasted_percentage' => __elephc_opcache_env_pct('ELEPHC_INI_opcache__max_wasted_percentage', 'ELEPHC_INI_opcache.max_wasted_percentage', 0.05)"
        ));
        assert!(literal.contains(
            "'opcache.file_cache_read_only' => __elephc_opcache_env_bool('ELEPHC_INI_opcache__file_cache_read_only', 'ELEPHC_INI_opcache.file_cache_read_only', false)"
        ));
        assert!(literal.contains("'version' => '8.5.0'"));
        assert!(literal.contains("'opcache_product_name' => 'Zend OPcache'"));
        // The literal must parse as a standalone expression statement.
        let _ = parse(&format!("<?php $c = {literal};"));
    }

    /// The 8.2 literal flips the JIT defaults and drops the 8.5-only directive.
    #[test]
    fn renders_php82_deltas() {
        let literal = render_configuration_literal(PhpVersion::Php82, &[]);
        assert!(literal.contains("'opcache.jit' => 'tracing'"));
        assert!(literal.contains("'opcache.jit_buffer_size' => 0"));
        // 8.2-only, and reporting-only ⇒ it carries the runtime env-override call.
        assert!(literal.contains(
            "'opcache.consistency_checks' => __elephc_opcache_env_int('ELEPHC_INI_opcache__consistency_checks', 'ELEPHC_INI_opcache.consistency_checks', 0)"
        ));
        assert!(!literal.contains("file_cache_read_only"));
        assert!(literal.contains("'version' => '8.2.0'"));
    }

    /// Injection is skipped for a program that never references either function.
    #[test]
    fn skips_injection_when_unused() {
        let program = parse("<?php echo 1;");
        let injected = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert_eq!(injected.len(), program.len());
    }

    /// Injection fires when `opcache_get_configuration` is called.
    #[test]
    fn injects_when_called() {
        let program = parse("<?php $c = opcache_get_configuration();");
        let injected = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert!(injected.len() > program.len());
    }

    /// `render_reset_body` follows the compile-time SAPI: CLI disabled, web enabled,
    /// for every maintained version.
    #[test]
    fn reset_body_follows_sapi() {
        for version in [
            PhpVersion::Php82,
            PhpVersion::Php83,
            PhpVersion::Php84,
            PhpVersion::Php85,
        ] {
            assert_eq!(render_reset_body(version, false, &[]), "false");
            assert_eq!(render_reset_body(version, true, &[]), "true");
        }
    }

    /// A default CLI binary that calls `opcache_reset()` injects a body returning
    /// `false`; the same program compiled `--web` returns `true`.
    #[test]
    fn injects_reset_with_sapi_gated_constant() {
        let program = parse("<?php var_dump(opcache_reset());");

        let cli = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert!(cli.len() > program.len());

        let web = inject_if_used(program.clone(), PhpVersion::Php85, true, None, &[], &[], None).0;
        assert!(web.len() > program.len());
    }

    /// The number of declarations [`OPCACHE_STATE_HELPERS`] contributes when any state-touching
    /// OPcache function is injected: the restart latch, the discard set, the discard-aware
    /// `timestamp` reader, the system-timezone resolver and the `asctime` formatter.
    const STATE_HELPER_DECLS: usize = 5;

    /// A program that references only `opcache_reset` does not inject
    /// `opcache_get_configuration`, and vice versa (pay-for-use per function).
    ///
    /// `opcache_reset` reads the in-process restart latch, so it also pulls in the shared state
    /// block — which is emitted ONCE however many of the five state-touching functions are used.
    #[test]
    fn injection_is_per_function() {
        let reset_only = parse("<?php opcache_reset();");
        let injected = inject_if_used(reset_only.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        // Exactly one OPcache function plus the one shared state block.
        assert_eq!(injected.len(), reset_only.len() + 1 + STATE_HELPER_DECLS);
    }

    /// The rendered 8.5 `opcache_get_status` body parses and carries the enabled-cache
    /// literals with the class-B invariants intact (memory/interned free = total - used,
    /// the derived `max_cached_keys`, and the default disabled-JIT sub-array).
    #[test]
    fn renders_parsable_php85_status_web() {
        let body = render_get_status_function(PhpVersion::Php85, true, &[], &[], false, None);
        // Web SAPI bakes the enabled gate as `true === false` (never returns false).
        assert!(body.contains("if (true === false)"));
        // memory_usage invariant: 134217728 - 6291456 = 127926272.
        assert!(body.contains("'used_memory' => 6291456"));
        assert!(body.contains("'free_memory' => 127926272"));
        assert!(body.contains("'wasted_memory' => 0"));
        // interned_strings_usage invariant: 8388608 - 1048576 = 7340032.
        assert!(body.contains("'buffer_size' => 8388608"));
        assert!(body.contains("'free_memory' => 7340032"));
        // Derived hash capacity for the default max_accelerated_files=10000.
        assert!(body.contains("'max_cached_keys' => 16229"));
        // start_time is a run-time clock reading, not a baked compile-time constant — and it is
        // MEMOIZED in a `static` so repeated calls report the SAME value, as reference PHP does.
        assert!(body.contains("static $__elephc_opcache_start_time = 0;"));
        assert!(body.contains(
            "if ($__elephc_opcache_start_time === 0) { $__elephc_opcache_start_time = time(); }"
        ));
        assert!(body.contains("'start_time' => $__elephc_opcache_start_time,"));
        assert!(
            !body.contains("'start_time' => time()"),
            "the per-call time() read is the bug this replaced"
        );
        // Rates are floats, so `0.0` (not `0`) must be emitted.
        assert!(body.contains("'opcache_hit_rate' => 0.0"));
        // Default JIT (disable) sub-array is entirely zero/false.
        assert!(body.contains("'enabled' => false"));
        assert!(body.contains("'buffer_size' => 0"));
        // scripts precedes jit: the `$status['scripts']` insert is before the jit block.
        let scripts_at = body.find("$status['scripts']").expect("scripts insert");
        let jit_at = body.find("$status['jit']").expect("jit insert");
        assert!(scripts_at < jit_at, "scripts must be inserted before jit");
        // The whole function tokenizes/parses.
        let _ = parse(&format!("<?php {body}"));
    }

    /// The CLI (non-web) 8.5 body bakes the disabled gate `false === false`, so the
    /// function returns `false` before building the array; it still parses.
    #[test]
    fn renders_php85_status_cli_disabled_gate() {
        let body = render_get_status_function(PhpVersion::Php85, false, &[], &[], false, None);
        assert!(body.contains("if (false === false)"));
        let _ = parse(&format!("<?php {body}"));
    }

    /// Extracts the rendered `$status['jit'] = [...]` block from a `opcache_get_status()` body,
    /// so a jit assertion cannot accidentally match a key of one of the earlier sub-arrays.
    fn jit_block(body: &str) -> String {
        let start = body.find("$status['jit']").expect("jit block must be rendered");
        let end = body[start..].find("];").expect("jit block must terminate") + start;
        body[start..end].to_string()
    }

    /// The BASELINE the whole feature must not disturb: on the default 8.5 target
    /// (`opcache.jit = disable`, no `--ini`), the jit sub-array is the all-zero/false array,
    /// byte-identical to what reference PHP 8.5.6 reports for its own default.
    #[test]
    fn renders_php85_default_jit_all_zero() {
        let body = render_get_status_function(PhpVersion::Php85, true, &[], &[], false, None);
        assert_eq!(
            jit_block(&body),
            "$status['jit'] = [\n        \
             'enabled' => false,\n        \
             'on' => false,\n        \
             'kind' => 0,\n        \
             'opt_level' => 0,\n        \
             'opt_flags' => 0,\n        \
             'buffer_size' => 0,\n        \
             'buffer_free' => 0,\n    "
        );
        let _ = parse(&format!("<?php {body}"));
    }

    /// Renders the jit block's seven values for readable assertions.
    fn jit_values(version: PhpVersion, overrides: &[(String, String)]) -> String {
        let body = render_get_status_function(version, true, &[], overrides, false, None);
        let block = jit_block(&body);
        let field = |key: &str| {
            let at = block
                .find(&format!("'{key}' => "))
                .unwrap_or_else(|| panic!("jit block must carry {key}"))
                + key.len()
                + 6;
            block[at..]
                .split(',')
                .next()
                .expect("field must terminate")
                .to_string()
        };
        format!(
            "{}/{}/{}/{}/{}/{}/{}",
            field("enabled"),
            field("on"),
            field("kind"),
            field("opt_level"),
            field("opt_flags"),
            field("buffer_size"),
            field("buffer_free"),
        )
    }

    /// The `--ini opcache.jit=<spelling>` overrides render the FULL reference
    /// kind/opt_level/opt_flags mapping while the clamp keeps `enabled`/`on` false and both
    /// buffer figures 0 — reference PHP's own "configured but unavailable" shape.
    #[test]
    fn renders_overridden_jit_modes_with_unavailable_clamp() {
        let ini = |raw: &str| vec![("opcache.jit".to_string(), raw.to_string())];
        // tracing (= 1254): kind 5, opt_level 4, opt_flags 6.
        assert_eq!(
            jit_values(PhpVersion::Php85, &ini("tracing")),
            "false/false/5/4/6/0/0"
        );
        assert_eq!(
            jit_values(PhpVersion::Php85, &ini("1254")),
            jit_values(PhpVersion::Php85, &ini("tracing"))
        );
        // function (= 1205): kind 0, opt_level 5, opt_flags 6.
        assert_eq!(
            jit_values(PhpVersion::Php85, &ini("function")),
            "false/false/0/5/6/0/0"
        );
        // A hand-written CRTO form decodes digit by digit.
        assert_eq!(
            jit_values(PhpVersion::Php85, &ini("1111")),
            "false/false/1/1/5/0/0"
        );
        // The switched-off spellings stay all-zero, indistinguishable from `disable` under the
        // clamp — which is exactly what reference PHP reports when the JIT is unavailable.
        assert_eq!(
            jit_values(PhpVersion::Php85, &ini("off")),
            "false/false/0/0/0/0/0"
        );
        // `opcache.jit_buffer_size` cannot lift the buffer clamp.
        let tracing_64m = vec![
            ("opcache.jit".to_string(), "tracing".to_string()),
            ("opcache.jit_buffer_size".to_string(), "64M".to_string()),
        ];
        assert_eq!(
            jit_values(PhpVersion::Php85, &tracing_64m),
            "false/false/5/4/6/0/0"
        );
    }

    /// The 8.2/8.3 targets default to `opcache.jit = tracing`, so their DEFAULT jit sub-array
    /// carries the tracing triple under the clamp. Pinned to reference PHP 8.2.31 with Xdebug
    /// loaded and its stock `opcache.jit` default, which reports exactly this array.
    /// An INVALID override on 8.2 is masked by re-applying that default (php-src's INI
    /// two-pass), where the same override on 8.5 would leave its partial residue.
    #[test]
    fn renders_php82_default_jit_tracing_under_clamp() {
        assert_eq!(jit_values(PhpVersion::Php82, &[]), "false/false/5/4/6/0/0");
        assert_eq!(jit_values(PhpVersion::Php83, &[]), "false/false/5/4/6/0/0");
        // 8.4 flipped the default to `disable`.
        assert_eq!(jit_values(PhpVersion::Php84, &[]), "false/false/0/0/0/0/0");

        let bad = vec![("opcache.jit".to_string(), "1355".to_string())];
        assert_eq!(
            jit_values(PhpVersion::Php82, &bad),
            "false/false/5/4/6/0/0",
            "the tracing default overwrites the rejected value's residue"
        );
        assert_eq!(
            jit_values(PhpVersion::Php85, &bad),
            "false/false/5/5/0/0/0",
            "the disable default leaves the rejected value's residue visible"
        );
        let body = render_get_status_function(PhpVersion::Php82, true, &[], &[], false, None);
        let _ = parse(&format!("<?php {body}"));
    }

    /// Injection fires for `opcache_get_status`, independently of the other two prelude
    /// functions (pay-for-use per function).
    #[test]
    fn injects_get_status_per_function() {
        let status_only = parse("<?php var_dump(opcache_get_status());");
        let injected = inject_if_used(status_only.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        // Exactly one OPcache function plus the one shared state block (`opcache_get_status`
        // reads the restart latch, the discard-aware `timestamp`, and the `asctime` formatter).
        assert_eq!(injected.len(), status_only.len() + 1 + STATE_HELPER_DECLS);
    }

    /// `opcache_is_script_cached` bakes the SAPI gate and the manifest: CLI disabled short-
    /// circuits to `false`; web enabled `realpath`-normalizes `$filename` and tests membership
    /// in the baked manifest. Both bodies parse; the empty manifest renders `[]`.
    #[test]
    fn is_script_cached_bakes_gate_and_manifest() {
        let cli = render_is_script_cached_function(PhpVersion::Php85, false, &[], &[]);
        assert!(cli.contains("if (false === false)"));
        assert!(cli.contains("in_array($path, [], true)"));
        let _ = parse(&format!("<?php {cli}"));

        let entries = [ScriptEntry {
            path: "/srv/app/main.php".to_string(),
            timestamp: 1_700_000_000,
            memory_consumption: 4_096,
        }];
        let web = render_is_script_cached_function(PhpVersion::Php85, true, &entries, &[]);
        // Web enabled → the gate never fires, realpath membership is reached.
        assert!(web.contains("if (true === false)"));
        assert!(web.contains("$rp = realpath($filename)"));
        assert!(web.contains("in_array($path, ['/srv/app/main.php'], true)"));
        let _ = parse(&format!("<?php {web}"));
    }

    /// `opcache_invalidate` bakes the SAPI gate AND the manifest: CLI disabled returns `false`,
    /// web enabled resolves the path (with the empty path going through `getcwd()`, which is what
    /// PHP's `realpath('')` resolves to) and records a discard for a FORCED call on a manifest
    /// member. Both bodies parse.
    #[test]
    fn invalidate_bakes_sapi_gate_and_manifest() {
        let entries = [ScriptEntry {
            path: "/srv/app/main.php".to_string(),
            timestamp: 1_700_000_000,
            memory_consumption: 4096,
        }];

        let cli = render_invalidate_function(PhpVersion::Php85, false, &entries, &[]);
        assert!(cli.contains("if (false === false)"));
        assert!(cli.contains("$rp = realpath($filename)"));
        let _ = parse(&format!("<?php {cli}"));

        let web = render_invalidate_function(PhpVersion::Php85, true, &entries, &[]);
        // Web enabled → the gate never fires, the path resolution is reached.
        assert!(web.contains("if (true === false)"));
        assert!(web.contains("$rp = realpath($filename)"));
        // The empty path resolves through `getcwd()`, not `realpath('')` (see INVALIDATE_TEMPLATE).
        assert!(web.contains("if ($filename === '') {"));
        assert!(web.contains("$cwd = getcwd();"));
        // A forced call on a manifest member records the discard.
        assert!(web.contains("if ($force && in_array($path, ['/srv/app/main.php'], true)) {"));
        assert!(web.contains("__elephc_opcache_invalidate_state($path, 1);"));
        let _ = parse(&format!("<?php {web}"));
    }

    /// `opcache_compile_file` bakes the SAPI gate and emits the exact php-src notice text
    /// to STDERR on the disabled path; the enabled path tests `realpath` membership in the
    /// baked manifest (a member → `true`). Both bodies parse.
    #[test]
    fn compile_file_bakes_sapi_gate_and_notice() {
        let cli = render_compile_file_function(PhpVersion::Php85, false, &[], &[]);
        assert!(cli.contains("if (false === false)"));
        assert!(cli.contains(
            "Notice: Zend OPcache has not been properly started, can't compile file"
        ));
        assert!(cli.contains("fwrite(STDERR,"));
        let _ = parse(&format!("<?php {cli}"));

        let entries = [ScriptEntry {
            path: "/srv/app/main.php".to_string(),
            timestamp: 1_700_000_000,
            memory_consumption: 4_096,
        }];
        let web = render_compile_file_function(PhpVersion::Php85, true, &entries, &[]);
        assert!(web.contains("if (true === false)"));
        assert!(web.contains("$rp = realpath($filename)"));
        assert!(web.contains("in_array($path, ['/srv/app/main.php'], true)"));
        let _ = parse(&format!("<?php {web}"));
    }

    /// The three file functions are injected pay-for-use, one per reference, and only when
    /// referenced. Each also pulls in the ONE shared in-process state block (they all consult the
    /// discard set — see `OPCACHE_STATE_HELPERS`).
    #[test]
    fn injects_file_functions_per_function() {
        for source in [
            "<?php var_dump(opcache_is_script_cached(__FILE__));",
            "<?php var_dump(opcache_invalidate(__FILE__));",
            "<?php var_dump(opcache_compile_file(__FILE__));",
        ] {
            let program = parse(source);
            let injected = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
            // Exactly one OPcache function per referenced name, plus the shared state block.
            assert_eq!(injected.len(), program.len() + 1 + STATE_HELPER_DECLS);
        }
    }

    /// A two-entry manifest sample used by the manifest-rendering tests.
    fn sample_manifest() -> Vec<ScriptEntry> {
        vec![
            ScriptEntry {
                path: "/srv/app/index.php".to_string(),
                timestamp: 1_700_000_000,
                memory_consumption: 12_345,
            },
            ScriptEntry {
                path: "/srv/app/vendor/autoload_files/helpers.php".to_string(),
                timestamp: 1_699_999_000,
                memory_consumption: 678,
            },
        ]
    }

    /// The flat manifest-paths literal is a parsable PHP array of the canonical paths, and
    /// an empty manifest renders `[]`.
    #[test]
    fn renders_manifest_paths_literal() {
        let literal = render_manifest_paths_literal(&sample_manifest());
        assert_eq!(
            literal,
            "['/srv/app/index.php', '/srv/app/vendor/autoload_files/helpers.php']"
        );
        let _ = parse(&format!("<?php $h = {literal};"));

        assert_eq!(render_manifest_paths_literal(&[]), "[]");
        let _ = parse(&format!("<?php $h = {};", render_manifest_paths_literal(&[])));
    }

    /// The `scripts` map is keyed by full_path and each entry carries the exact 7-key shape,
    /// reading its two REQUEST-clock fields off `opcache_get_status`'s memoized `static` and its
    /// `timestamp` off the file mtime through the discard-aware reader. Reference PHP 8.5.6
    /// (VERIFIED): `last_used_timestamp == time()`, `revalidate == last_used_timestamp +
    /// opcache.revalidate_freq`, `timestamp == filemtime()`, `last_used ==
    /// asctime(localtime(last_used))`.
    #[test]
    fn renders_scripts_map_literal() {
        // revalidate_freq = 2 (the 8.5 directive default).
        let map = render_scripts_map_literal(&sample_manifest(), 2);
        // Keyed by full_path.
        assert!(map.contains("'/srv/app/index.php' => ["));
        assert!(map.contains("'full_path' => '/srv/app/index.php'"));
        // All 7 keys present with integer/int-derived values.
        assert!(map.contains("'hits' => 0"));
        assert!(map.contains("'memory_consumption' => 12345"));
        // The two REQUEST-clock fields read `opcache_get_status`'s memoized `static`, not the
        // mtime — see `render_scripts_map_literal` for the verified reference transcript.
        assert!(map.contains("'last_used' => __elephc_opcache_asctime($__elephc_opcache_start_time)"));
        assert!(map.contains("'last_used_timestamp' => $__elephc_opcache_start_time"));
        // `timestamp` stays the FILE mtime, routed through the discard-aware reader so a
        // force-invalidated entry reports 0 (php-src `zend_accel_discard_script`).
        assert!(map.contains(
            "'timestamp' => __elephc_opcache_script_timestamp('/srv/app/index.php', 1700000000)"
        ));
        // revalidate = the request clock + opcache.revalidate_freq (NOT the mtime + freq).
        assert!(map.contains("'revalidate' => $__elephc_opcache_start_time + 2"));
        // The whole map parses as a PHP expression.
        let _ = parse(&format!("<?php $s = {map};"));

        // Empty manifest → empty map.
        assert_eq!(render_scripts_map_literal(&[], 2), "[]");
    }

    /// `opcache_get_status` bakes the manifest count into `num_cached_scripts` /
    /// `num_cached_keys`, splices the scripts map, and grows `used_memory` by the sum of the
    /// per-script memory (keeping `free = total - used - wasted`). The body still parses.
    #[test]
    fn get_status_bakes_manifest_counts_and_scripts() {
        let manifest = sample_manifest();
        let body = render_get_status_function(PhpVersion::Php85, true, &manifest, &[], false, None);
        // Two cached scripts / keys.
        assert!(body.contains("'num_cached_scripts' => 2"));
        assert!(body.contains("'num_cached_keys' => 2"));
        // The scripts map is spliced (not the empty literal).
        assert!(body.contains("'full_path' => '/srv/app/index.php'"));
        assert!(body.contains("'full_path' => '/srv/app/vendor/autoload_files/helpers.php'"));
        // used_memory = 6291456 baseline + (12345 + 678) = 6304479.
        assert!(body.contains("'used_memory' => 6304479"));
        // free_memory = 134217728 - 6304479 = 127913249.
        assert!(body.contains("'free_memory' => 127913249"));
        // scripts precedes jit.
        let scripts_at = body.find("$status['scripts']").expect("scripts insert");
        let jit_at = body.find("$status['jit']").expect("jit insert");
        assert!(scripts_at < jit_at);
        let _ = parse(&format!("<?php {body}"));
    }

    /// An empty manifest still renders a valid `opcache_get_status` body: zero counts, an
    /// empty scripts map, and the untouched baseline memory figures.
    #[test]
    fn get_status_empty_manifest_is_valid() {
        let body = render_get_status_function(PhpVersion::Php85, true, &[], &[], false, None);
        assert!(body.contains("'num_cached_scripts' => 0"));
        assert!(body.contains("'num_cached_keys' => 0"));
        assert!(body.contains("$status['scripts'] = [];"));
        // Baseline used_memory unchanged when no scripts contribute memory.
        assert!(body.contains("'used_memory' => 6291456"));
        let _ = parse(&format!("<?php {body}"));
    }

    /// The rendered opcache INI helpers parse and carry the raw-string projection: booleans
    /// as "1"/"0", the four non-derivable overrides, an empty string, and the access sets.
    ///
    /// The compile-time raw string is the same in every arm as before; what the runtime
    /// env-override adds is that a REPORTING-ONLY arm wraps it in an
    /// `__elephc_opcache_env_raw(<under>, <dotted>, <type code>, <compile-time raw>)` call while an
    /// EXCLUDED arm (`opcache.enable`, `opcache.memory_consumption`, `opcache.jit`,
    /// `opcache.jit_buffer_size`, `opcache.preload`, …) still returns the bare literal.
    #[test]
    fn renders_parsable_opcache_ini_helpers() {
        let helpers = render_opcache_ini_helpers(PhpVersion::Php85, &[]);
        // EXCLUDED directives keep the bare raw-string literal.
        assert!(helpers.contains("if ($option === 'opcache.enable') { return '1'; }"));
        assert!(helpers.contains("if ($option === 'opcache.memory_consumption') { return '128'; }"));
        assert!(helpers.contains("if ($option === 'opcache.jit_buffer_size') { return '64M'; }"));
        assert!(helpers.contains("if ($option === 'opcache.jit') { return 'disable'; }"));
        assert!(helpers.contains("if ($option === 'opcache.preload') { return ''; }"));
        // Reporting-only directives: raw strings (not the normalized configuration values) behind
        // the runtime env-override call, one type code each.
        assert!(helpers.contains(
            "if ($option === 'opcache.protect_memory') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__protect_memory', 'ELEPHC_INI_opcache.protect_memory', 'b', '0'); }"
        ));
        assert!(helpers.contains(
            "if ($option === 'opcache.max_wasted_percentage') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__max_wasted_percentage', 'ELEPHC_INI_opcache.max_wasted_percentage', 'p', '5'); }"
        ));
        assert!(helpers.contains(
            "if ($option === 'opcache.optimization_level') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__optimization_level', 'ELEPHC_INI_opcache.optimization_level', 'i', '0x7FFEBFFF'); }"
        ));
        assert!(helpers.contains(
            "if ($option === 'opcache.jit_prof_threshold') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__jit_prof_threshold', 'ELEPHC_INI_opcache.jit_prof_threshold', 'f', '0.005'); }"
        ));
        assert!(helpers.contains(
            "if ($option === 'opcache.error_log') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__error_log', 'ELEPHC_INI_opcache.error_log', 's', ''); }"
        ));
        // The helper functions are present and the whole block parses.
        assert!(helpers.contains("function __elephc_opcache_ini_string(string $option): string|false"));
        assert!(helpers.contains("function __elephc_opcache_ini_access(string $option): int"));
        assert!(helpers.contains("function __elephc_opcache_ini_keys(): array"));
        assert!(helpers.contains("function __elephc_opcache_ini_all_details(): array"));
        assert!(helpers.contains("function __elephc_opcache_ini_all_plain(): array"));
        let _ = parse(&format!("<?php {helpers}"));
    }

    /// Extracts the PHP string literals from the rendered `__elephc_opcache_ini_keys()` body.
    fn rendered_ini_keys(helpers: &str) -> Vec<String> {
        let body = helpers
            .split("function __elephc_opcache_ini_keys(): array {")
            .nth(1)
            .expect("keys helper must be rendered");
        let literal = body
            .split_once('[')
            .and_then(|(_, rest)| rest.split_once(']'))
            .expect("keys helper must render a list literal")
            .0;
        literal
            .split(", ")
            .map(|entry| entry.trim().trim_matches('\'').to_string())
            .collect()
    }

    /// `__elephc_opcache_ini_keys()` renders the directive names SORTED ASCENDING, matching
    /// reference PHP's `ini_get_all` key order, while `opcache_directives()` itself keeps
    /// REGISTRATION order (what `opcache_get_configuration()['directives']` reports). The two
    /// orders must differ — if they ever coincide this test still passes, so the registration
    /// list is asserted to be un-sorted on 8.5 to prove the sort is doing real work.
    #[test]
    fn ini_keys_are_sorted_but_directive_table_is_not() {
        let helpers = render_opcache_ini_helpers(PhpVersion::Php85, &[]);
        let keys = rendered_ini_keys(&helpers);

        let registration: Vec<String> = opcache_directives(80500)
            .iter()
            .map(|(name, _)| (*name).to_string())
            .collect();
        assert_eq!(
            keys.len(),
            registration.len(),
            "sorting must not add or drop directives"
        );

        let mut expected = registration.clone();
        expected.sort();
        assert_eq!(keys, expected, "rendered ini_get_all keys must be sorted");
        assert_eq!(keys[0], "opcache.blacklist_filename");
        assert_eq!(keys[keys.len() - 1], "opcache.validate_timestamps");

        // The registration order is genuinely different, so the sort is load-bearing and
        // opcache_directives() was left untouched.
        assert_ne!(
            registration, expected,
            "registration order must stay unsorted (opcache_get_configuration relies on it)"
        );
        assert_eq!(
            registration[0], "opcache.enable",
            "registration order still starts at opcache.enable"
        );
    }

    /// `render_ini_module_known` renders the known-module predicate from
    /// `CORE_LOADED_EXTENSIONS`, LOWERCASED so the comparison is verbatim against php-src's
    /// lowercase registry keys, and adds `'session'` only for the web SAPI. Every core
    /// extension must appear, and the canonical mixed-case spellings must NOT.
    #[test]
    fn module_known_list_is_lowercased_core_extensions() {
        let cli = render_ini_module_known(false);
        let web = render_ini_module_known(true);

        for name in crate::codegen::lower_inst::builtins::CORE_LOADED_EXTENSIONS {
            let lowered = name.to_ascii_lowercase();
            assert!(
                cli.contains(&format!("$m === '{lowered}'")),
                "CLI known-module list must contain {lowered}"
            );
            assert!(
                web.contains(&format!("$m === '{lowered}'")),
                "web known-module list must contain {lowered}"
            );
        }
        // Verbatim, not case-folded: the mixed-case spellings never appear as literals.
        assert!(!cli.contains("'Zend OPcache'"));
        assert!(!cli.contains("'Core'"));
        assert!(!cli.contains("'SPL'"));
        // 'session' is a --web-only module.
        assert!(!cli.contains("$m === 'session'"));
        assert!(web.contains("$m === 'session'"));
        // The parameter is nullable so `$extension !== null` need not narrow ?string to Str.
        assert!(cli.contains("function __elephc_ini_module_known(?string $m): bool"));
        let _ = parse(&format!("<?php {cli}"));
        let _ = parse(&format!("<?php {web}"));
    }

    /// The CLI `ini_get_all` wrapper is injected with its known-module predicate, drops the
    /// return type hint (reference PHP is `array|false`; the hint is omitted so ordinary union
    /// return inference handles the exits), and dispatches to the two single-shape helpers
    /// rather than branching on `$details` inside a loop.
    #[test]
    fn cli_ini_get_all_renders_filter_dispatch() {
        let program = parse("<?php var_dump(ini_get_all(null, false));");
        let injected = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        let rendered = format!("{injected:?}");
        assert!(injected.len() > program.len(), "ini_get_all must be injected");
        // The predicate is injected alongside the wrapper.
        assert!(rendered.contains("__elephc_ini_module_known"));
        assert!(rendered.contains("__elephc_opcache_ini_all_details"));
        assert!(rendered.contains("__elephc_opcache_ini_all_plain"));
    }

    /// The 8.2 helpers flip the version-dependent raw strings (jit tracing, buffer 0).
    #[test]
    fn opcache_ini_helpers_follow_version() {
        let helpers = render_opcache_ini_helpers(PhpVersion::Php82, &[]);
        assert!(helpers.contains("if ($option === 'opcache.jit') { return 'tracing'; }"));
        assert!(helpers.contains("if ($option === 'opcache.jit_buffer_size') { return '0'; }"));
        // 8.2-only directive is present in the dispatch, and is reporting-only ⇒ overridable.
        assert!(helpers.contains(
            "if ($option === 'opcache.consistency_checks') { return __elephc_opcache_env_raw('ELEPHC_INI_opcache__consistency_checks', 'ELEPHC_INI_opcache.consistency_checks', 'i', '0'); }"
        ));
        let _ = parse(&format!("<?php {helpers}"));
    }

    /// CLI (`web = false`) injects the opcache ini_get wrapper plus the shared helpers when
    /// `ini_get` is referenced; `--web` does not (web_prelude owns the ini surface there).
    #[test]
    fn cli_injects_ini_get_opcache_wrapper() {
        let program = parse("<?php echo ini_get('opcache.enable');");

        let cli = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert!(cli.len() > program.len());
        // web = true must not inject the CLI wrappers (would redeclare web_prelude's ini_get).
        let web = inject_if_used(program.clone(), PhpVersion::Php85, true, None, &[], &[], None).0;
        assert_eq!(web.len(), program.len());
    }

    /// Counts top-level declarations of `name` in a program (test helper for the
    /// inject-exactly-once rules below).
    fn declarations_of(program: &Program, name: &str) -> usize {
        program
            .iter()
            .filter(|stmt| {
                matches!(&stmt.kind, StmtKind::FunctionDecl { name: decl, .. } if decl == name)
            })
            .count()
    }

    /// The runtime env-override block is a self-contained, parsable set of PHP functions covering
    /// the lookup, the per-type normalizers, and both consumer surfaces.
    #[test]
    fn renders_parsable_env_override_helpers() {
        let helpers = render_opcache_env_helpers();
        // The lookup consults the `__` spelling first and the dotted one only as a fallback.
        assert!(helpers.contains("function __elephc_opcache_env(string $u, string $d): string"));
        assert!(helpers.contains("$v = (string) getenv($u);"));
        assert!(helpers.contains("return (string) getenv($d);"));
        // The typed surface (opcache_get_configuration) — one helper per type code.
        for typed in [
            "function __elephc_opcache_env_bool(string $u, string $d, bool $def): bool",
            "function __elephc_opcache_env_int(string $u, string $d, int $def): int",
            "function __elephc_opcache_env_float(string $u, string $d, float $def): float",
            "function __elephc_opcache_env_pct(string $u, string $d, float $def): float",
            "function __elephc_opcache_env_str(string $u, string $d, string $def): string",
        ] {
            assert!(helpers.contains(typed), "missing {typed}");
        }
        // The raw-string surface (ini_get / ini_get_all).
        assert!(helpers.contains(
            "function __elephc_opcache_env_raw(string $u, string $d, string $t, string $def): string"
        ));
        // The scanner and the normalizers mirroring `ini_scanner_value` / `parse_ini_override`.
        // `_bool` and `_int` carry NO `_ok` predicate: the reference handlers they mirror
        // (`zend_ini_parse_bool`, `zend_ini_parse_quantity`) cannot fail, so `_pct` is the only
        // type left with a rejection path.
        for normalizer in [
            "__elephc_ini_scan",
            "__elephc_ini_bool_val",
            "__elephc_ini_isspace",
            "__elephc_ini_digit",
            "__elephc_ini_quantity",
            "__elephc_ini_atoi",
            "__elephc_ini_pct_ok",
            "__elephc_ini_pct_val",
        ] {
            assert!(helpers.contains(normalizer), "missing {normalizer}");
        }
        let _ = parse(&format!("<?php {helpers}"));
    }

    /// `render_directive_value_expr` IS the scope rule in rendered form: an excluded directive
    /// stays a plain literal, a reporting-only one becomes an env-override call whose `$def` is
    /// that same literal.
    #[test]
    fn directive_value_expr_honors_the_override_scope() {
        // Excluded — the ten directives elephc derives compiled-in behavior from.
        for (name, value, expected) in [
            ("opcache.enable_cli", DirectiveValue::Bool(false), "false"),
            ("opcache.memory_consumption", DirectiveValue::Int(134_217_728), "134217728"),
            ("opcache.jit", DirectiveValue::Str("disable"), "'disable'"),
            ("opcache.preload", DirectiveValue::Str(""), "''"),
        ] {
            assert_eq!(render_directive_value_expr(name, &value), expected);
        }
        // Reporting-only — the literal becomes the call's default argument.
        assert_eq!(
            render_directive_value_expr("opcache.save_comments", &DirectiveValue::Bool(true)),
            "__elephc_opcache_env_bool('ELEPHC_INI_opcache__save_comments', \
             'ELEPHC_INI_opcache.save_comments', true)"
        );
        assert_eq!(
            render_directive_value_expr("opcache.lockfile_path", &DirectiveValue::Str("/tmp")),
            "__elephc_opcache_env_str('ELEPHC_INI_opcache__lockfile_path', \
             'ELEPHC_INI_opcache.lockfile_path', '/tmp')"
        );
    }

    /// The env-override block is injected EXACTLY ONCE on CLI — a second copy would be a
    /// redeclaration — whether it is pulled in by `opcache_get_configuration`, by the `opcache.*`
    /// INI dispatcher, or by both at the same time. Under `--web` it is never injected here: the
    /// web prelude owns it (see `render_opcache_env_helpers`).
    #[test]
    fn env_override_helpers_are_injected_exactly_once() {
        let configuration_only = parse("<?php $c = opcache_get_configuration();");
        let ini_only = parse("<?php echo ini_get('opcache.enable');");
        let both = parse("<?php $c = opcache_get_configuration(); echo ini_get('opcache.enable');");

        for program in [&configuration_only, &ini_only, &both] {
            let cli =
                inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
            assert_eq!(
                declarations_of(&cli, "__elephc_opcache_env"),
                1,
                "the env-override block must be injected exactly once on CLI"
            );
            assert_eq!(declarations_of(&cli, "__elephc_opcache_env_raw"), 1);
            // web = true never emits it here; the web prelude bakes it instead.
            let web =
                inject_if_used(program.clone(), PhpVersion::Php85, true, None, &[], &[], None).0;
            assert_eq!(declarations_of(&web, "__elephc_opcache_env"), 0);
        }

        // A program that uses neither surface pays nothing.
        let unrelated = parse("<?php echo 1;");
        let none =
            inject_if_used(unrelated.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert_eq!(declarations_of(&none, "__elephc_opcache_env"), 0);
    }

    /// The RESTRICTED `opcache_get_configuration` keeps its dead array exit, which still names the
    /// typed env helpers — so the block has to be injected for it too or the body would not
    /// name-resolve.
    #[test]
    fn restricted_configuration_still_injects_the_env_helpers() {
        let program = parse("<?php $c = opcache_get_configuration();");
        let overrides = vec![("opcache.restrict_api".to_string(), "/nowhere".to_string())];
        let injected = inject_if_used(
            program,
            PhpVersion::Php85,
            false,
            Some("/tmp/app.php"),
            &[],
            &overrides,
            None,
        )
        .0;
        assert_eq!(declarations_of(&injected, "__elephc_opcache_env"), 1);
        assert_eq!(declarations_of(&injected, "__elephc_opcache_env_bool"), 1);
    }

    /// A user-declared `ini_get` wins: the CLI wrapper is not injected (no redeclaration).
    #[test]
    fn cli_ini_get_respects_user_declaration() {
        let program = parse("<?php function ini_get($o): string|false { return 'x'; } echo ini_get('a');");
        let injected = inject_if_used(program.clone(), PhpVersion::Php85, false, None, &[], &[], None).0;
        assert_eq!(injected.len(), program.len());
    }

    /// Builds a `--ini opcache.restrict_api=<prefix>` override list.
    fn restrict_api_override(prefix: &str) -> Vec<(String, String)> {
        vec![(RESTRICT_API_DIRECTIVE.to_string(), prefix.to_string())]
    }

    /// `restrict_api_denies` reproduces php-src's `validate_api_restriction()` byte-compare.
    /// Every case below is PINNED FROM REFERENCE PHP 8.5.6 (see the doc comment on the function
    /// for the exact `php -d` invocations that produced them).
    #[test]
    fn restrict_api_prefix_rule_matches_reference() {
        let entry = "/private/tmp/ra/foobar/x.php";
        let deny = |prefix: &str| {
            restrict_api_denies(Some(entry), 80500, &restrict_api_override(prefix))
        };

        // Empty prefix disables the restriction entirely — today's default behavior.
        assert!(!deny(""), "empty restrict_api must allow");
        assert!(
            !restrict_api_denies(Some(entry), 80500, &[]),
            "no --ini override at all must allow"
        );

        // Exact directory prefix, and the whole path as the prefix, both allow.
        assert!(!deny("/private/tmp/ra/foobar"));
        assert!(!deny("/private/tmp/ra/foobar/x.php"));
        assert!(!deny("/"), "root prefix allows every absolute entry");

        // PLAIN BYTE PREFIX, not a path-component match: `/private/tmp/ra/foo` ALLOWS an entry
        // under `/private/tmp/ra/foobar/`. Verified on reference PHP, which uses memcmp.
        assert!(
            !deny("/private/tmp/ra/foo"),
            "a partial path component still matches (memcmp, not component-wise)"
        );

        // CASE-SENSITIVE even on a case-insensitive filesystem (memcmp, not a fs lookup).
        assert!(deny("/private/tmp/ra/Foobar"));

        // A prefix LONGER than the entry path can never match.
        assert!(deny("/private/tmp/ra/foobar/x.php/deeper"));

        // A wholly unrelated prefix denies.
        assert!(deny("/nonexistent"));

        // Reference compares the RESOLVED path, so an unresolved spelling of the same file
        // denies (macOS: /tmp is a symlink to /private/tmp).
        assert!(deny("/tmp/ra"));

        // No entry path at all mirrors php-src's null `path_translated` arm: deny — but ONLY
        // when a non-empty prefix is configured.
        assert!(restrict_api_denies(None, 80500, &restrict_api_override("/srv")));
        assert!(!restrict_api_denies(None, 80500, &restrict_api_override("")));
        assert!(!restrict_api_denies(None, 80500, &[]));
    }

    /// The restriction is entry-script-relative, and the directive is version-independent
    /// (`opcache.restrict_api` is registered by every maintained version).
    #[test]
    fn restrict_api_applies_to_every_version() {
        for version in [
            PhpVersion::Php82,
            PhpVersion::Php83,
            PhpVersion::Php84,
            PhpVersion::Php85,
        ] {
            let id = version.version_id();
            assert!(restrict_api_denies(
                Some("/srv/app/index.php"),
                id,
                &restrict_api_override("/other")
            ));
            assert!(!restrict_api_denies(
                Some("/srv/app/index.php"),
                id,
                &restrict_api_override("/srv/app")
            ));
        }
    }

    /// With the default (empty) `restrict_api` every rendered body is BYTE-IDENTICAL to the
    /// unrestricted rendering — the warning slot is removed whole, newline included. This is the
    /// regression guard for "the default path is untouched".
    #[test]
    fn default_restrict_api_renders_byte_identical_bodies() {
        let manifest = sample_manifest();
        // An unrelated --ini override must not disturb the slot removal either.
        let unrelated = vec![("opcache.enable_cli".to_string(), "1".to_string())];

        for overrides in [Vec::new(), unrelated, restrict_api_override("")] {
            assert!(!restrict_api_denies(
                Some("/srv/app/index.php"),
                80500,
                &overrides
            ));
            let status = render_get_status_function(PhpVersion::Php85, true, &manifest, &overrides, false, None);
            // No placeholder survives and no warning leaks into the default body.
            assert!(!status.contains("__RESTRICT_API_WARNING__"));
            assert!(!status.contains("restricted by"));
            // The gate line is followed IMMEDIATELY by `return false;` — no blank line, which is
            // what makes the removal byte-identical rather than merely whitespace-equivalent.
            assert!(
                status.contains("=== false) {\n        return false;\n    }"),
                "default status gate must render exactly as before: {status}"
            );
            let _ = parse(&format!("<?php {status}"));
        }
    }

    /// A denying `restrict_api` renders the five RESTRICTED bodies: each emits the verbatim
    /// reference warning to STDERR and returns `false`. `opcache_compile_file` is UNTOUCHED —
    /// reference PHP does not guard it (verified: it still returns `true` with no warning under
    /// `restrict_api=/nonexistent`).
    #[test]
    fn denying_restrict_api_renders_restricted_bodies() {
        let overrides = restrict_api_override("/nonexistent");
        let entry = Some("/srv/app/index.php");
        assert!(restrict_api_denies(entry, 80500, &overrides));

        let program = parse(
            "<?php opcache_get_status(); opcache_get_configuration(); opcache_reset(); \
             opcache_is_script_cached(__FILE__); opcache_invalidate(__FILE__); \
             opcache_compile_file(__FILE__);",
        );
        let injected = inject_if_used(program, PhpVersion::Php85, false, entry, &[], &overrides, None).0;
        let rendered = format!("{injected:?}");

        // The warning text appears once per restricted function, and never a sixth time.
        // Counted on a QUOTE-FREE slice of the message: the AST's `Debug` rendering escapes the
        // embedded `"restrict_api"` quotes, so the full const would never match here.
        let hits = rendered.matches("API is restricted by").count();
        assert_eq!(
            hits, 5,
            "exactly the five restricted functions carry the warning (compile_file must not)"
        );

        // The two array-returning functions keep their dead array exit, so `array|false`
        // narrowing still works for callers.
        let status = render_get_status_function(PhpVersion::Php85, true, &[], &overrides, true, None);
        assert!(
            status.contains("if (false === false)"),
            "restricted status forces the always-taken gate regardless of SAPI"
        );
        assert!(
            status.contains("return $status;"),
            "the array exit must survive so the signature stays array|false"
        );
        assert!(status.contains(RESTRICT_API_WARNING_TEXT));
        let _ = parse(&format!("<?php {status}"));

        let config =
            splice_restrict_api_warning(RESTRICTED_GET_CONFIGURATION_TEMPLATE, true, "        ")
                .replace(
                    "__OPCACHE_CONFIGURATION__",
                    &render_configuration_literal(PhpVersion::Php85, &overrides),
                );
        assert!(config.contains("function opcache_get_configuration() {"));
        assert!(config.contains("if (false === false)"));
        assert!(config.contains("'opcache_product_name' => 'Zend OPcache'"));
        let _ = parse(&format!("<?php {config}"));

        // The three bool-returning functions are single-exit: reference type is already `bool`.
        for template in [
            RESTRICTED_RESET_TEMPLATE,
            RESTRICTED_IS_SCRIPT_CACHED_TEMPLATE,
            RESTRICTED_INVALIDATE_TEMPLATE,
        ] {
            let body = render_restricted_function(template);
            assert!(body.contains("): bool {"));
            assert!(body.contains(RESTRICT_API_WARNING_TEXT));
            assert!(body.contains("return false;"));
            let _ = parse(&format!("<?php {body}"));
        }
    }

    /// The rendered warning statement is the verbatim reference text with PHP's `Warning: `
    /// prefix — byte-identical to what the `--web` prelude's `trigger_error(..., E_WARNING)`
    /// would write, so one form serves both SAPIs.
    #[test]
    fn restrict_api_warning_statement_is_verbatim() {
        let expected = format!(
            "    fwrite(STDERR, 'Warning: {RESTRICT_API_WARNING_TEXT}' . \"\\n\");"
        );
        assert_eq!(render_restrict_api_warning_stmt("    "), expected);
        // Pin the message itself, so a typo in the const cannot pass by matching itself.
        assert_eq!(
            RESTRICT_API_WARNING_TEXT,
            "Zend OPcache API is restricted by \"restrict_api\" configuration directive"
        );
        // Double quotes around restrict_api survive single-quoting unescaped.
        assert!(render_restrict_api_warning_stmt("").contains("\"restrict_api\""));
    }

    /// `canonical_entry_path` resolves symlinked spellings the way reference PHP's
    /// `path_translated` does, and yields `None` for a path that does not exist.
    #[test]
    fn canonical_entry_path_resolves_and_reports_missing() {
        use std::io::Write;

        let dir = std::env::temp_dir().join(format!(
            "elephc_opcache_entry_test_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let main = dir.join("entry.php");
        let mut file = std::fs::File::create(&main).expect("create temp entry");
        file.write_all(b"<?php echo 1;").expect("write temp entry");
        drop(file);

        let resolved = canonical_entry_path(main.to_str().unwrap()).expect("entry must resolve");
        assert_eq!(
            resolved,
            main.canonicalize().unwrap().display().to_string(),
            "the entry path must be canonicalized like __FILE__ and ScriptEntry::path"
        );
        // The canonical entry is always allowed by a prefix of its own directory.
        let parent = Path::new(&resolved).parent().unwrap().display().to_string();
        assert!(!restrict_api_denies(
            Some(&resolved),
            80500,
            &restrict_api_override(&parent)
        ));

        assert!(canonical_entry_path(dir.join("nope.php").to_str().unwrap()).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `collect_manifest` canonicalizes and stats the entry file, dedupes it against the
    /// autoloaded list, and skips paths that cannot be stat'd (never fabricated).
    #[test]
    fn collect_manifest_stats_and_dedupes() {
        use std::io::Write;

        let dir = std::env::temp_dir().join(format!(
            "elephc_opcache_manifest_test_{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let main = dir.join("main.php");
        let mut file = std::fs::File::create(&main).expect("create temp main");
        file.write_all(b"<?php echo 1;").expect("write temp main");
        drop(file);

        let canonical = main.canonicalize().expect("canonicalize temp main");

        // The entry file also appears in the autoloaded list, plus a nonexistent path that
        // must be skipped (not fabricated).
        let missing = dir.join("does_not_exist.php");
        let always = [canonical.clone(), missing];

        let manifest = collect_manifest(main.to_str().unwrap(), &[], &always);

        // Exactly one entry: the deduped entry file; the missing path is skipped.
        assert_eq!(manifest.len(), 1);
        assert_eq!(manifest[0].path, canonical.display().to_string());
        // Size of "<?php echo 1;" is 13 bytes.
        assert_eq!(manifest[0].memory_consumption, 13);
        // A plausible recent mtime (> 2020-01-01).
        assert!(manifest[0].timestamp > 1_577_836_800);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The manifest ORDER is entry file, then includes, then autoloaded — each group in the
    /// order its producing pass hands over (both sort) — with duplicates dropped across groups,
    /// first occurrence winning. Pinned here because the baked `scripts` map key order and the
    /// `preload_statistics.scripts` list both follow it, so it must not drift silently.
    #[test]
    fn collect_manifest_orders_entry_then_includes_then_autoloaded() {
        use std::io::Write;

        let dir = std::env::temp_dir().join(format!(
            "elephc_opcache_manifest_order_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let write = |name: &str| {
            let path = dir.join(name);
            let mut file = std::fs::File::create(&path).expect("create temp script");
            file.write_all(b"<?php\n").expect("write temp script");
            path.canonicalize().expect("canonicalize temp script")
        };
        // Deliberately named so alphabetical order differs from argument order.
        let main = write("z_main.php");
        let inc_a = write("a_inc.php");
        let inc_b = write("b_inc.php");
        let auto = write("c_auto.php");

        // `inc_b` is ALSO in the autoloaded group (a required file that is autoloadable too):
        // it must appear once, in the include group.
        let manifest = collect_manifest(
            main.to_str().unwrap(),
            &[inc_a.clone(), inc_b.clone()],
            &[auto.clone(), inc_b.clone()],
        );

        let paths: Vec<&str> = manifest.iter().map(|entry| entry.path.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                main.display().to_string().as_str(),
                inc_a.display().to_string().as_str(),
                inc_b.display().to_string().as_str(),
                auto.display().to_string().as_str(),
            ]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// THE SOUNDNESS PIN for the split injection/baking mechanism (see `bake_manifest`):
    /// name-resolving a re-rendered manifest function IN ISOLATION produces the byte-identical
    /// AST that name-resolving it as part of the whole program does. That equality is what makes
    /// substituting the baked declaration after `name_resolver::resolve` has already run a pure
    /// substitution rather than a semantic change — and it is checked on the REAL rendered
    /// bodies, including the `date(…)` call inside the `scripts` map and the `in_array` /
    /// `realpath` / `fwrite(STDERR, …)` references in the other two.
    #[test]
    fn substitutes_a_name_resolution_identical_body() {
        let manifest = sample_manifest();
        let bodies = [
            render_get_status_function(PhpVersion::Php85, true, &manifest, &[], false, None),
            render_is_script_cached_function(PhpVersion::Php85, true, &manifest, &[]),
            render_compile_file_function(PhpVersion::Php85, true, &manifest, &[]),
        ];
        for body in &bodies {
            // Isolated: exactly what `bake_manifest` substitutes.
            let isolated = parse_baked_function(body);
            // In-program: the same declaration name-resolved alongside a namespaced caller,
            // which is the situation the declaration must survive at the injection point.
            let program = parse(&format!(
                "<?php\n{body}\nnamespace App;\nfunction caller() {{ return opcache_get_status(); }}\n"
            ));
            let resolved = crate::name_resolver::resolve(program)
                .expect("in-program source must name-resolve");
            let in_program = resolved
                .iter()
                .find(|stmt| matches!(&stmt.kind, StmtKind::FunctionDecl { name, .. }
                    if name.to_ascii_lowercase().starts_with("opcache_")))
                .expect("the opcache declaration must survive name resolution at top level");
            assert_eq!(format!("{:?}", isolated), format!("{:?}", in_program));
        }
    }

    /// `bake_manifest` swaps ONLY the recorded sites, and swaps in the full manifest. A program
    /// that declares its own `opcache_get_status()` records no site, so nothing is touched.
    #[test]
    fn bake_manifest_replaces_only_recorded_sites() {
        let program = parse("<?php $s = opcache_get_status(); $c = opcache_is_script_cached(__FILE__);");
        let (injected, sites) =
            inject_if_used(program, PhpVersion::Php85, true, None, &[], &[], None);
        assert!(!sites.is_empty());

        let manifest = sample_manifest();
        let baked = bake_manifest(injected, &sites, PhpVersion::Php85, true, &manifest, &[], None);
        let rendered = format!("{:?}", baked);
        assert!(rendered.contains("/srv/app/index.php"));
        assert!(rendered.contains("/srv/app/vendor/autoload_files/helpers.php"));

        // A user-declared `opcache_get_status` is never a bake site.
        let own = parse("<?php function opcache_get_status($x = true) { return false; } $s = opcache_get_status();");
        let (_, own_sites) = inject_if_used(own, PhpVersion::Php85, true, None, &[], &[], None);
        assert!(!own_sites.get_status);
    }

    // ---------------------------------------------------------------------------------------
    // `opcache.preload` — the compile-time verdict and the rendered `preload_statistics` block.
    // Every expectation is pinned to reference PHP 8.5.6 (Homebrew, `Zend OPcache` loaded); the
    // probe commands are recorded on `PreloadVerdict` / `PreloadStatistics`.
    // ---------------------------------------------------------------------------------------

    /// Builds an `--ini` override list for `opcache.preload`.
    fn preload_override(path: &str) -> Vec<(String, String)> {
        vec![("opcache.preload".to_string(), path.to_string())]
    }

    /// Creates a temp dir holding a real file, and returns `(dir, canonical file path)`.
    fn temp_preload_file(tag: &str) -> (PathBuf, String) {
        use std::io::Write;

        let dir = std::env::temp_dir().join(format!(
            "elephc_opcache_preload_{tag}_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("pre.php");
        let mut handle = std::fs::File::create(&file).expect("create temp preload file");
        handle.write_all(b"<?php\n").expect("write temp preload file");
        drop(handle);
        let canonical = file.canonicalize().expect("canonicalize temp preload file");
        (dir, canonical.display().to_string())
    }

    /// THE DEFAULT: an empty `opcache.preload` never preloads, whatever the SAPI — so no
    /// `preload_statistics` key is ever emitted on a stock build. This is the row the whole
    /// feature must not disturb.
    #[test]
    fn empty_preload_directive_never_preloads() {
        for version in [
            PhpVersion::Php82,
            PhpVersion::Php83,
            PhpVersion::Php84,
            PhpVersion::Php85,
        ] {
            for web in [false, true] {
                assert_eq!(
                    preload_verdict(version, web, &[], &[]),
                    PreloadVerdict::NotPreloading,
                    "the default empty opcache.preload must not preload ({version:?}, web={web})"
                );
            }
        }
        // An explicit empty override is the same thing (verified against reference
        // `-d opcache.preload=`, which reports no `preload_statistics` key).
        assert_eq!(
            preload_verdict(PhpVersion::Php85, true, &preload_override(""), &[]),
            PreloadVerdict::NotPreloading
        );
    }

    /// CACHE DISABLED: a set `opcache.preload` is ignored ENTIRELY — including a path that does
    /// not exist, which must NOT become a compile error. Pinned to reference PHP, where
    /// `opcache.enable_cli=0` with a missing preload path runs cleanly and exits 0.
    #[test]
    fn disabled_cache_ignores_preload_entirely() {
        let (dir, file) = temp_preload_file("disabled");

        // CLI defaults to `opcache.enable_cli=0` → disabled.
        assert_eq!(
            preload_verdict(PhpVersion::Php85, false, &preload_override(&file), &[]),
            PreloadVerdict::NotPreloading
        );
        // A missing path is not even looked at when the cache is off.
        let missing = dir.join("nope.php").display().to_string();
        let verdict = preload_verdict(PhpVersion::Php85, false, &preload_override(&missing), &[]);
        assert_eq!(verdict, PreloadVerdict::NotPreloading);
        assert!(verdict.compile_error().is_none());
        assert!(verdict.compile_warning().is_none());

        // Explicitly disabling the web cache reaches the same row.
        let mut overrides = preload_override(&missing);
        overrides.push(("opcache.enable".to_string(), "0".to_string()));
        assert_eq!(
            preload_verdict(PhpVersion::Php85, true, &overrides, &[]),
            PreloadVerdict::NotPreloading
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// CACHE ENABLED + UNRESOLVABLE PATH: a compile ERROR naming the directive and the path — the
    /// AOT equivalent of reference PHP's startup fatal `Failed opening required '<path>'`.
    /// A DIRECTORY is unresolvable too (reference cannot `require` one either).
    #[test]
    fn enabled_cache_with_missing_preload_is_a_compile_error() {
        let (dir, _file) = temp_preload_file("missing");
        let missing = dir.join("nope.php").display().to_string();

        let verdict = preload_verdict(PhpVersion::Php85, true, &preload_override(&missing), &[]);
        assert_eq!(
            verdict,
            PreloadVerdict::Unresolvable {
                requested: missing.clone()
            }
        );
        let message = verdict.compile_error().expect("an unresolvable path must error");
        assert!(message.contains("opcache.preload"), "{message}");
        assert!(message.contains(&missing), "the message must name the path: {message}");
        assert!(
            message.contains("failed opening required"),
            "the message must echo reference's fatal wording: {message}"
        );
        // An error, not a warning.
        assert!(verdict.compile_warning().is_none());

        // A directory does not resolve to a preloadable file.
        let dir_verdict = preload_verdict(
            PhpVersion::Php85,
            true,
            &preload_override(&dir.display().to_string()),
            &[],
        );
        assert!(matches!(dir_verdict, PreloadVerdict::Unresolvable { .. }));

        // `--ini opcache.enable_cli=1` reaches the same row on a CLI target.
        let mut overrides = preload_override(&missing);
        overrides.push(("opcache.enable_cli".to_string(), "1".to_string()));
        assert!(matches!(
            preload_verdict(PhpVersion::Php85, false, &overrides, &[]),
            PreloadVerdict::Unresolvable { .. }
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// CACHE ENABLED + RESOLVABLE PATH: preloading, with `in_manifest` deciding the warning.
    /// A manifest member is silent; a resolvable file outside the manifest warns but still
    /// compiles (preloading a file this program never compiles in is legitimate).
    #[test]
    fn enabled_cache_with_resolvable_preload_preloads() {
        let (dir, file) = temp_preload_file("resolvable");

        // Outside the manifest → warning, but no error.
        let outside = preload_verdict(PhpVersion::Php85, true, &preload_override(&file), &[]);
        assert_eq!(
            outside,
            PreloadVerdict::Preloading {
                resolved: file.clone(),
                in_manifest: false
            }
        );
        assert!(outside.compile_error().is_none(), "a resolvable path must never error");
        let warning = outside
            .compile_warning()
            .expect("a preload file outside the manifest must warn");
        assert!(warning.contains("opcache.preload"), "{warning}");
        assert!(warning.contains(&file), "the warning must name the path: {warning}");

        // In the manifest → completely silent.
        let manifest = [ScriptEntry {
            path: file.clone(),
            timestamp: 1_700_000_000,
            memory_consumption: 6,
        }];
        let inside = preload_verdict(PhpVersion::Php85, true, &preload_override(&file), &manifest);
        assert_eq!(
            inside,
            PreloadVerdict::Preloading {
                resolved: file.clone(),
                in_manifest: true
            }
        );
        assert!(inside.compile_error().is_none());
        assert!(inside.compile_warning().is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `collect_preload_symbols` reports FULLY-QUALIFIED user names in ORIGINAL CASE with no
    /// leading `\` — the reference spelling (VERIFIED: `My\Space\MixedCaseFn`). Interfaces,
    /// traits and enums all land under `classes`, as reference PHP does. Both the statement and
    /// the brace form of `namespace` are handled, and duplicates are dropped case-insensitively.
    #[test]
    fn collect_preload_symbols_qualifies_and_dedupes() {
        let program = parse(
            "<?php\n\
             function GlobalFn() {}\n\
             class GlobalClass {}\n\
             namespace My\\Space;\n\
             function MixedCaseFn() {}\n\
             class MixedCaseClass {}\n\
             interface MyIface {}\n\
             trait MyTrait {}\n\
             enum MyEnum {}\n",
        );
        let symbols = collect_preload_symbols(&program);
        assert_eq!(
            symbols.functions,
            vec!["GlobalFn".to_string(), "My\\Space\\MixedCaseFn".to_string()]
        );
        assert_eq!(
            symbols.classes,
            vec![
                "GlobalClass".to_string(),
                "My\\Space\\MixedCaseClass".to_string(),
                "My\\Space\\MyIface".to_string(),
                "My\\Space\\MyTrait".to_string(),
                "My\\Space\\MyEnum".to_string(),
            ]
        );

        // The brace form scopes the namespace to its own body only.
        let braced = parse(
            "<?php\n\
             namespace A { function InA() {} }\n\
             namespace B { function InB() {} }\n",
        );
        let braced = collect_preload_symbols(&braced);
        assert_eq!(
            braced.functions,
            vec!["A\\InA".to_string(), "B\\InB".to_string()]
        );

        // A program that declares nothing reports nothing.
        assert!(collect_preload_symbols(&parse("<?php echo 1;"))
            .functions
            .is_empty());
    }

    /// `preload_statistics` is `None` for every non-preloading verdict, and derives its four
    /// fields from the manifest plus the collected symbols when preloading.
    #[test]
    fn preload_statistics_derives_from_manifest_and_symbols() {
        let symbols = collect_preload_symbols(&parse("<?php function f() {} class C {}"));
        let manifest = sample_manifest();

        assert!(preload_statistics(&PreloadVerdict::NotPreloading, &manifest, &symbols).is_none());
        assert!(preload_statistics(
            &PreloadVerdict::Unresolvable {
                requested: "/nope".to_string()
            },
            &manifest,
            &symbols
        )
        .is_none());

        let stats = preload_statistics(
            &PreloadVerdict::Preloading {
                resolved: "/srv/app/index.php".to_string(),
                in_manifest: true,
            },
            &manifest,
            &symbols,
        )
        .expect("a preloading verdict must produce statistics");
        // Σ of the sample manifest's per-script memory: 12345 + 678.
        assert_eq!(stats.memory_consumption, 13_023);
        assert_eq!(stats.functions, vec!["f".to_string()]);
        assert_eq!(stats.classes, vec!["C".to_string()]);
        assert_eq!(
            stats.scripts,
            vec![
                "/srv/app/index.php".to_string(),
                "/srv/app/vendor/autoload_files/helpers.php".to_string(),
            ]
        );
    }

    /// The RENDERED `preload_statistics` literal: the VERIFIED reference key ORDER
    /// (`memory_consumption`, `functions`, `classes`, `scripts`), inserted BETWEEN
    /// `opcache_statistics` and `scripts`, and BEFORE `jit`.
    #[test]
    fn renders_preload_statistics_in_reference_key_order() {
        let symbols = collect_preload_symbols(&parse(
            "<?php namespace App; function helper() {} class Widget {}",
        ));
        let manifest = sample_manifest();
        let stats = preload_statistics(
            &PreloadVerdict::Preloading {
                resolved: "/srv/app/index.php".to_string(),
                in_manifest: true,
            },
            &manifest,
            &symbols,
        )
        .expect("statistics");

        let body =
            render_get_status_function(PhpVersion::Php85, true, &manifest, &[], false, Some(&stats));

        assert!(body.contains("$status['preload_statistics'] = ["), "{body}");
        assert!(body.contains("'memory_consumption' => 13023,"), "{body}");
        assert!(body.contains("'functions' => ['App\\\\helper'],"), "{body}");
        assert!(body.contains("'classes' => ['App\\\\Widget'],"), "{body}");
        assert!(
            body.contains(
                "'scripts' => ['/srv/app/index.php', '/srv/app/vendor/autoload_files/helpers.php'],"
            ),
            "{body}"
        );

        // Key ORDER inside the block, and the block's position in the status array.
        let block_at = body.find("$status['preload_statistics']").expect("block");
        let mem_at = body[block_at..].find("'memory_consumption'").expect("mem");
        let fns_at = body[block_at..].find("'functions'").expect("functions");
        let cls_at = body[block_at..].find("'classes'").expect("classes");
        let scr_at = body[block_at..].find("'scripts'").expect("scripts");
        assert!(mem_at < fns_at && fns_at < cls_at && cls_at < scr_at, "{body}");

        let stats_map_at = body.find("'opcache_statistics' =>").expect("opcache_statistics");
        let scripts_at = body.find("$status['scripts']").expect("scripts insert");
        let jit_at = body.find("$status['jit']").expect("jit insert");
        assert!(
            stats_map_at < block_at && block_at < scripts_at && scripts_at < jit_at,
            "preload_statistics must sit between opcache_statistics and scripts: {body}"
        );

        // The whole function still tokenizes and parses.
        let _ = parse(&format!("<?php {body}"));
    }

    /// A program with NO user functions/classes renders the block WITHOUT the `functions` and
    /// `classes` keys — reference PHP omits them entirely when empty rather than reporting empty
    /// arrays (VERIFIED by preloading a file containing only `<?php`).
    #[test]
    fn renders_preload_statistics_omitting_empty_symbol_lists() {
        let symbols = collect_preload_symbols(&parse("<?php echo 1;"));
        let manifest = sample_manifest();
        let stats = preload_statistics(
            &PreloadVerdict::Preloading {
                resolved: "/srv/app/index.php".to_string(),
                in_manifest: true,
            },
            &manifest,
            &symbols,
        )
        .expect("statistics");

        let rendered = render_preload_statistics_stmt(&stats);
        assert!(rendered.contains("'memory_consumption' => 13023,"), "{rendered}");
        assert!(rendered.contains("'scripts' => ["), "{rendered}");
        assert!(
            !rendered.contains("'functions'"),
            "an empty functions list must be OMITTED, not reported as []: {rendered}"
        );
        assert!(
            !rendered.contains("'classes'"),
            "an empty classes list must be OMITTED, not reported as []: {rendered}"
        );

        // Only `classes` empty → `functions` present, `classes` absent.
        let fns_only = collect_preload_symbols(&parse("<?php function f() {}"));
        let stats = preload_statistics(
            &PreloadVerdict::Preloading {
                resolved: "/srv/app/index.php".to_string(),
                in_manifest: true,
            },
            &manifest,
            &fns_only,
        )
        .expect("statistics");
        let rendered = render_preload_statistics_stmt(&stats);
        assert!(rendered.contains("'functions' => ['f'],"), "{rendered}");
        assert!(!rendered.contains("'classes'"), "{rendered}");
    }

    /// THE BASELINE: with no preloading, `opcache_get_status` renders BYTE-IDENTICALLY to the
    /// template — the `__PRELOAD_STATISTICS__` slot is removed WHOLE (newline included), so the
    /// default build carries not even a whitespace diff. Mirrors
    /// `default_restrict_api_renders_byte_identical_bodies`.
    #[test]
    fn absent_preload_renders_byte_identical_status_body() {
        let manifest = sample_manifest();
        let body =
            render_get_status_function(PhpVersion::Php85, true, &manifest, &[], false, None);
        assert!(
            !body.contains("preload_statistics"),
            "no preload key may appear on the default path: {body}"
        );
        assert!(
            !body.contains("__PRELOAD_STATISTICS__"),
            "the placeholder must be removed, not left in: {body}"
        );
        // The line between the status literal and the `if ($include_scripts)` insert must close
        // up completely — no blank line left behind.
        assert!(body.contains("    ];\n    if ($include_scripts) {"), "{body}");
        let _ = parse(&format!("<?php {body}"));
    }
}
