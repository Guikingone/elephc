//! Purpose:
//! Owns the PHP templates for public OPcache functions.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Each template remains an intact cohesive PHP source leaf.

#[allow(unused_imports)]
use super::*;

/// The `opcache_get_configuration` function template. `__OPCACHE_CONFIGURATION__` is
/// spliced with the baked array literal at injection time; `replace` is used rather
/// than `format!` so the PHP body needs no brace escaping.
pub(super) const GET_CONFIGURATION_TEMPLATE: &str = r#"function opcache_get_configuration(): array {
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
pub(super) const RESET_TEMPLATE: &str = r#"function opcache_reset(): bool {
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
pub(super) const GET_STATUS_TEMPLATE: &str = r#"function opcache_get_status($include_scripts = true) {
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
/// The manifest is every PHP/LFC source file compiled into this binary: the entry file, every
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
pub(super) const IS_SCRIPT_CACHED_TEMPLATE: &str = r#"function opcache_is_script_cached($filename): bool {
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
pub(super) const INVALIDATE_TEMPLATE: &str = r#"function opcache_invalidate($filename, $force = false): bool {
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

/// The `--strict-opcache` variant of [`INVALIDATE_TEMPLATE`].
///
/// D5 IS THE ONE DOCUMENTED DIVERGENCE of this OPcache model, and it is the only one that can
/// silently change what a program DOES rather than what it reports. Reference PHP's
/// `opcache_invalidate($f, true)` discards the cached script so the NEXT include re-reads and
/// re-compiles `$f` from disk. Code that elephc compiled into the binary cannot be re-read: it
/// is frozen at link time. The default body therefore reports success exactly as reference PHP
/// does, which is right for a program that merely inspects the cache — and wrong, silently, for
/// one that invalidates in order to pick up changed code (a dev-mode cache-buster, a plugin
/// reloader). Such a program keeps running the OLD code with no signal at all.
///
/// Under `--strict-opcache` that request throws instead. The throw is deliberately narrow:
/// ONLY a manifest member — code actually frozen into this binary — is impossible to honor.
/// A non-manifest path is a file this binary never compiled, so invalidating it is a no-op in
/// reference PHP too and stays a plain `true`. The `$force` distinction is likewise preserved:
/// without `$force` reference PHP does not discard anything either, so there is nothing elephc
/// fails to honor and nothing to throw about.
///
/// `RuntimeException` is thrown directly rather than through a prelude-declared subclass:
/// a prelude class extending a built-in `Exception` cannot call `parent::__construct` without
/// a link error, so subclassing here would buy a nicer type at the cost of a fragile ctor.
pub(super) const STRICT_INVALIDATE_TEMPLATE: &str = r#"function opcache_invalidate($filename, $force = false): bool {
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
        throw new RuntimeException('opcache_invalidate(): --strict-opcache: cannot invalidate "' . $path . '": it is compiled into this binary and cannot be reloaded from disk');
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
pub(super) const COMPILE_FILE_TEMPLATE: &str = r#"function opcache_compile_file($filename): bool {
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
pub(super) const IS_SCRIPT_CACHED_IN_FILE_CACHE_TEMPLATE: &str =
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
pub(super) const JIT_BLACKLIST_TEMPLATE: &str = r#"function opcache_jit_blacklist($closure): void {
    $closure = $closure;
}
"#;
