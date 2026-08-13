//! Purpose:
//! Renders mutable OPcache state and compile-time restrict_api gates.
//!
//! Called from:
//! - The OPcache prelude facade and sibling rendering modules.
//!
//! Key details:
//! - Warning text and entry-path prefix matching remain reference-verified.

#[allow(unused_imports)]
use super::*;

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
pub(super) const OPCACHE_STATE_HELPERS: &str = r#"function __elephc_opcache_restart_pending(bool $schedule): bool {
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
pub(super) fn render_opcache_state_helpers() -> String {
    OPCACHE_STATE_HELPERS.to_string()
}

/// The `opcache.restrict_api` directive name.
pub(super) const RESTRICT_API_DIRECTIVE: &str = "opcache.restrict_api";

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
pub(super) const RESTRICT_API_WARNING_TEXT: &str =
    "Zend OPcache API is restricted by \"restrict_api\" configuration directive";

/// Renders the restricted-path diagnostic as a PHP statement indented by `indent`.
///
/// Written straight to `STDERR` as `Warning: <text>` rather than through `trigger_error`:
/// on CLI there is no `trigger_error` at all, and under `--web` the prelude's
/// `trigger_error($msg, E_WARNING)` itself does `fwrite(STDERR, 'Warning: ' . $msg . "\n")`
/// (see `crate::web_prelude`), so this emits BYTE-IDENTICAL output in both SAPIs while
/// staying resolvable on a plain CLI binary. The message is single-quoted so its embedded
/// `"restrict_api"` double quotes need no escaping.
pub(super) fn render_restrict_api_warning_stmt(indent: &str) -> String {
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
pub(super) fn splice_restrict_api_warning(template: &str, restricted: bool, indent: &str) -> String {
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
pub(super) fn restrict_api_denies(
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
