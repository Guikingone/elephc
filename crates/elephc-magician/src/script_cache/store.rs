//! Purpose:
//! The process-wide runtime script cache: one entry per canonical path holding the
//! segmented, parsed form of a dynamically included PHP file, plus the statistics
//! the OPcache API reports about it.
//!
//! Called from:
//! - `crate::interpreter::include_exec` for every runtime include/require.
//! - `crate::script_cache` re-exports, which the OPcache bridge symbols read.
//!
//! Key details:
//! - The cache is a no-op unless `ScriptCacheConfig::enabled`, which mirrors
//!   `opcache_cache_enabled`. A default CLI binary therefore behaves exactly as it
//!   did before this module existed.
//! - Freshness follows php-src, not "always re-read": on fill an entry records
//!   `revalidate_at = now + opcache.revalidate_freq` and is only re-`stat`ed once
//!   that instant has passed, so a changed file may serve stale for up to
//!   `revalidate_freq` seconds. `opcache.validate_timestamps = 0` never re-stats.
//! - The cache NEVER evicts. php-src refuses new entries once the budget or the
//!   entry ceiling is reached and latches `cache_full`; reproducing that is both
//!   simpler and more faithful than an LRU.
//! - A forced `opcache_invalidate()` marks an entry discarded rather than removing
//!   it, matching php-src keeping the shared-memory slot until the next restart —
//!   and matching what the compile-time manifest emulation already does.

use super::config::{config, ScriptCacheConfig};
use super::segments::{segment_script, ParseMode, ScriptSegment};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// One cached script: its replayable segments plus the freshness and accounting state.
#[derive(Debug)]
struct Entry {
    segments: Arc<[ScriptSegment]>,
    mtime: Option<i64>,
    size: u64,
    footprint: usize,
    hits: u64,
    last_used: i64,
    revalidate_at: i64,
    discarded: bool,
}

/// A read-only view of one cached script, for the `opcache_get_status()` surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedScriptInfo {
    pub full_path: String,
    pub hits: u64,
    pub memory_consumption: usize,
    pub last_used_timestamp: i64,
    pub timestamp: i64,
    /// The entry's own `revalidate_at`, not a figure derived from `last_used`.
    pub revalidate_at: i64,
}

/// The counters `opcache_get_status()` reports for the dynamic tier.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScriptCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub num_cached_scripts: usize,
    pub used_memory: usize,
    pub cache_full: bool,
    pub oom_restarts: u64,
    pub manual_restarts: u64,
    pub last_restart_time: i64,
    pub restart_pending: bool,
    /// `opcache.blacklist_filename` refusals — one per script run but not stored.
    pub blacklist_misses: u64,
}

/// The process-wide cache state.
#[derive(Debug, Default)]
struct ScriptCache {
    entries: HashMap<PathBuf, Entry>,
    hits: u64,
    misses: u64,
    used_memory: usize,
    cache_full: bool,
    oom_restarts: u64,
    manual_restarts: u64,
    blacklist_misses: u64,
    /// Bumped on every mutation, so a reader can tell whether a snapshot it took is still
    /// current. See `generation`.
    generation: u64,
    last_restart_time: i64,
    restart_pending: bool,
}

static SCRIPT_CACHE: OnceLock<Mutex<ScriptCache>> = OnceLock::new();

/// Returns the process-wide script cache singleton.
fn script_cache() -> &'static Mutex<ScriptCache> {
    SCRIPT_CACHE.get_or_init(|| Mutex::new(ScriptCache::default()))
}

/// Locks the cache, recovering the inner state if a previous panic poisoned it.
fn lock_script_cache() -> MutexGuard<'static, ScriptCache> {
    script_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Returns the current wall clock in whole seconds since the Unix epoch.
fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Returns a file's mtime in whole seconds since the Unix epoch, if it has one.
pub(super) fn mtime_seconds(metadata: &std::fs::Metadata) -> Option<i64> {
    metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|elapsed| elapsed.as_secs() as i64)
}

/// Loads a script's replayable segments, serving the cache when it is warm.
///
/// Returns the same `io::Error` `std::fs::read` would have produced for a file that
/// cannot be opened, so the caller's missing-include diagnostics are unchanged. When
/// the cache is disabled this reads and segments the file exactly as the uncached
/// path always did, storing nothing.
pub(crate) fn load_script(path: &Path) -> io::Result<Arc<[ScriptSegment]>> {
    let config = config();
    if !config.enabled {
        // No entry will hold this result, so the byte-keyed parse memo is what keeps a
        // repeated include off the parser — exactly as the inline loop did before.
        let bytes = std::fs::read(path)?;
        return Ok(Arc::from(segment_script(&bytes, ParseMode::Memoized)));
    }
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Some(segments) = serve_warm_entry(&key, &config)? {
        return Ok(segments);
    }
    fill_entry(&key, path, &config)
}

/// Returns a warm entry's segments when one is present and still considered fresh.
///
/// Answers `None` when the entry is absent, discarded, or has failed revalidation —
/// each of which must fall through to a fill.
fn serve_warm_entry(
    key: &Path,
    config: &ScriptCacheConfig,
) -> io::Result<Option<Arc<[ScriptSegment]>>> {
    let now = now_seconds();
    let mut cache = lock_script_cache();
    let Some(entry) = cache.entries.get(key) else {
        return Ok(None);
    };
    if entry.discarded {
        return Ok(None);
    }
    if config.validate_timestamps && now >= entry.revalidate_at {
        let Some(metadata) = std::fs::metadata(key).ok() else {
            // The file is gone. Fall through to the fill, which reproduces the
            // caller's missing-include diagnostics rather than serving stale code.
            return Ok(None);
        };
        if mtime_seconds(&metadata) != entry.mtime || metadata.len() != entry.size {
            return Ok(None);
        }
        let freq = config.revalidate_freq as i64;
        let entry = cache.entries.get_mut(key).expect("entry was just observed");
        entry.revalidate_at = now.saturating_add(freq);
    }
    let entry = cache.entries.get_mut(key).expect("entry was just observed");
    entry.hits += 1;
    entry.last_used = now;
    let segments = Arc::clone(&entry.segments);
    cache.hits += 1;
    // A WARM HIT IS A MUTATION TOO. It moves this entry's `hits` and `last_used`, both of
    // which `opcache_get_status()['scripts']` reports, so a snapshot taken before it is now
    // stale. Missing this bump made the status surface report a cached script's hit count as
    // whatever it was at the FIRST status call of the process, while the aggregate `hits`
    // beside it — read from `stats()` rather than the snapshot — kept counting.
    cache.generation = cache.generation.wrapping_add(1);
    Ok(Some(segments))
}

/// Reads, segments and admits a script, returning its segments either way.
///
/// A file the budget or the entry ceiling refuses is still returned to the caller:
/// refusing to CACHE is not refusing to RUN, which is php-src's behaviour once the
/// cache is full.
fn fill_entry(
    key: &Path,
    path: &Path,
    config: &ScriptCacheConfig,
) -> io::Result<Arc<[ScriptSegment]>> {
    // ONE HANDLE FOR BOTH the bytes and the metadata that will be stored beside them.
    //
    // Reading the file and then stat'ing the PATH is two lookups of a name that can change
    // in between: a rewrite landing in that window records the NEW mtime and size over the
    // OLD bytes, and the same pair is handed to `file_store::store`, so the mismatched entry
    // survives the process and every later validation agrees with it. php-src takes the
    // timestamp from the handle it compiles, for exactly this reason.
    //
    // `opcache.file_update_protection` narrows the window rather than closing it — a file
    // whose mtime is too young is not stored — so the default of 2 seconds hides this, and
    // the documented way to turn that guard off reopens it.
    let mut file = std::fs::File::open(path)?;
    let metadata = file.metadata().ok();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut bytes)?;
    let mtime = metadata.as_ref().and_then(mtime_seconds);
    let file_size = metadata.as_ref().map_or(bytes.len() as u64, |meta| meta.len());
    // `opcache.blacklist_filename` is decided FIRST, and the ordering is the contract, not
    // a preference. php-src hands a blacklisted file straight back to the original compiler
    // before any cache accounting, so such a script: runs normally, is stored NOWHERE — not
    // in the memory cache and not in `opcache.file_cache` either — and counts as a
    // `blacklist_misses` INSTEAD of a `misses`. VERIFIED against reference PHP 8.5.10, where
    // including a blacklisted file left `misses` untouched and moved only `blacklist_misses`,
    // and where including it twice counted TWO refusals: the counter is of refusals, not of
    // distinct files.
    if super::blacklist::blocks(key) {
        let segments: Arc<[ScriptSegment]> = Arc::from(segment_script(&bytes, ParseMode::Fresh));
        lock_script_cache().blacklist_misses += 1;
        return Ok(segments);
    }
    let size = file_size;
    // The SIZE refusal comes before any accounting, exactly like the blacklist one above and
    // for the same reason: php-src counts an oversized file as a `blacklist_misses` and NOT
    // as a miss. VERIFIED on reference PHP 8.5.10 — `-d opcache.max_file_size=50` over two
    // oversized scripts reports `misses=0 blacklist_misses=2`. The counter's name is
    // php-src's; what it means is "compiled but deliberately not stored", which a size
    // refusal is. Placing it after `misses += 1` made elephc report BOTH.
    if !config.admits_size(size) {
        let segments: Arc<[ScriptSegment]> = Arc::from(segment_script(&bytes, ParseMode::Fresh));
        lock_script_cache().blacklist_misses += 1;
        return Ok(segments);
    }
    // The FILE CACHE is consulted before the parser. It holds this script already parsed,
    // and only hands it back when the source's mtime, size and canonical path still match
    // what was stored — so a hit is the same segments a parse would produce, for roughly a
    // quarter of the cost (see `file_store`). A miss, a stale entry or any I/O failure all
    // fall through to the parse below.
    let from_disk = super::file_store::load(config, key, mtime, file_size);
    let parsed_here = from_disk.is_none();
    let segments: Arc<[ScriptSegment]> = match from_disk {
        Some(cached) => Arc::from(cached),
        None => Arc::from(segment_script(&bytes, ParseMode::Fresh)),
    };
    let now = now_seconds();
    let mut cache = lock_script_cache();
    // A SECOND-LEVEL HIT IS A HIT. php-src's `persistent_compile_file` only reaches
    // `ZCSG(misses)++` when the file cache did NOT produce a script; a load from it falls
    // into the same branch as a shared-memory hit and bumps `hits`. VERIFIED on reference PHP
    // 8.5.10: two runs against one `opcache.file_cache` directory report `hits=0 misses=2`
    // cold and `hits=2 misses=0` warm. Counting it as a miss reported the exact opposite of
    // reference in the recycled-worker case the file cache exists for.
    if parsed_here {
        cache.misses += 1;
    } else {
        cache.hits += 1;
    }
    // A SCRIPT THAT DID NOT PARSE IS NOT CACHED, in memory or on disk. php-src stores
    // nothing on a compile error; the next request compiles the file again and sees whatever
    // is there now. Caching the failure instead makes a FIXED file keep raising the old error
    // — for up to `opcache.revalidate_freq` seconds (2 by default), and with
    // `opcache.validate_timestamps=0` until the process restarts. That turns an ordinary
    // edit-and-reload into a stale syntax error the developer cannot clear, and the on-disk
    // `opcache.file_cache` copy survives the process entirely.
    //
    // The miss has already been counted above, which is also what php-src does: the compile
    // was attempted and did not come from the cache.
    if segments
        .iter()
        .any(|segment| matches!(segment, ScriptSegment::ParseError(_)))
    {
        return Ok(segments);
    }
    // A file younger than `opcache.file_update_protection` is RUN but not STORED, so a file
    // caught part-written never becomes a cached entry that outlives the write. Unlike the
    // size refusal this one DOES count a miss — VERIFIED: with the guard raised, including a
    // fresh file moves `misses` 1 -> 2 and leaves `blacklist_misses` at 0, while
    // `num_cached_scripts` stays put.
    if !config.admits_age(mtime, now) {
        return Ok(segments);
    }
    // A PENDING RESTART CLOSES ADMISSION. `opcache_reset()` schedules rather than flushes,
    // and php-src's accelerator refuses to admit anything new to shared memory from the
    // moment the flag is set — the entries already there keep answering until the restart
    // lands, but nothing joins them. Without this the script was cached DURING the window
    // the reset opened, and survived the flush that followed.
    //
    // MEASURED, `opcache_reset()` then `opcache_compile_file($p)`: reference reports the
    // file uncached, elephc reported it cached.
    //
    // The script still RUNS — the segments are returned exactly as the two refusals above
    // return them. The disk write is skipped with the admission, because storing an entry
    // the cache would not accept only moves the problem into the next process.
    if cache.restart_pending {
        return Ok(segments);
    }
    if parsed_here {
        // Only a script this process actually parsed is written back, and only once the
        // refusals above have passed. Writing before them persisted files those very rules
        // exist to keep out — an age refusal would still have left a part-written file in
        // the on-disk cache, which is precisely what `file_update_protection` is for.
        // Re-writing one that came FROM the cache would be pure I/O for a byte-identical file.
        super::file_store::store(config, key, mtime, file_size, &segments);
    }
    let footprint: usize = segments
        .iter()
        .map(ScriptSegment::memory_footprint)
        .sum::<usize>();
    let replacing = cache.entries.get(key).map_or(0, |entry| entry.footprint);
    if !cache.admits_entry(key, footprint, replacing, config) {
        return Ok(segments);
    }
    cache.used_memory = cache.used_memory - replacing + footprint;
    cache.generation = cache.generation.wrapping_add(1);
    cache.entries.insert(
        key.to_path_buf(),
        Entry {
            segments: Arc::clone(&segments),
            mtime,
            size,
            footprint,
            hits: 0,
            last_used: now,
            revalidate_at: now.saturating_add(config.revalidate_freq as i64),
            discarded: false,
        },
    );
    Ok(segments)
}

impl ScriptCache {
    /// Returns whether one more entry fits, latching `cache_full` when it does not.
    ///
    /// php-src stops storing new scripts once either the byte budget or the entry
    /// ceiling is reached and never evicts, so this refuses rather than making room.
    /// Replacing an existing entry is always allowed: it frees its own footprint.
    fn admits_entry(
        &mut self,
        key: &Path,
        footprint: usize,
        replacing: usize,
        config: &ScriptCacheConfig,
    ) -> bool {
        let replacement = self.entries.contains_key(key);
        if !replacement && self.entries.len() >= config.max_accelerated_files {
            self.cache_full = true;
            return false;
        }
        if self.used_memory - replacing + footprint > config.memory_consumption {
            self.cache_full = true;
            return false;
        }
        true
    }
}

/// Schedules the restart `opcache_reset()` asks for, and returns what it should report.
///
/// `true` on the FIRST call, `false` on every call after it, because php-src's
/// `zend_accel_schedule_restart()` sets `ZCSG(restart_pending)` AND clears the shared
/// `accelerator_enabled` flag that `opcache_reset()`'s own guard tests — so a second call
/// in the same request takes the `false` exit.
///
/// THERE IS NO DIVERGENCE HERE ANY MORE. This used to flush immediately, because the request
/// boundary a deferred flush needs lives in `elephc-web`, which does not depend on the
/// interpreter crate. `__elephc_eval_opcache_apply_restart` closed that gap: generated code
/// emits it at the top of the `--web` handler, so the flush now lands where php-src's does.
///
/// The byte budget and the `cache_full` latch are released with the entries: a restart is
/// exactly what clears them in php-src too.
pub fn schedule_restart() -> bool {
    let mut cache = lock_script_cache();
    if cache.restart_pending {
        return false;
    }
    // SCHEDULES, and does nothing else. php-src's `zend_accel_schedule_restart` sets the
    // flag and defers the restart itself to the next request, so within THIS one the cache
    // keeps answering and every figure stays put. VERIFIED on reference PHP 8.5.10: right
    // after `opcache_reset()`, `opcache_is_script_cached()` is still true,
    // `num_cached_scripts` is unchanged, `manual_restarts` is still 0 and
    // `last_restart_time` is still 0. [`apply_pending_restart`] is what moves all four.
    cache.restart_pending = true;
    true
}

/// Performs a scheduled restart, if one is pending. Returns whether anything was flushed.
///
/// This is the deferred half of `opcache_reset()`, and it runs at a REQUEST BOUNDARY —
/// generated code calls it at the top of each `--web` request. A CLI program is one
/// request, so it never runs there, which is exactly right: reference PHP would restart at
/// the next request, and a CLI process has none.
///
/// Clearing the latch is what lets a LATER request schedule its own restart again, matching
/// php-src, where the second `opcache_reset()` of one request fails but the next request's
/// succeeds.
pub fn apply_pending_restart() -> bool {
    let mut cache = lock_script_cache();
    if !cache.restart_pending {
        return false;
    }
    cache.generation = cache.generation.wrapping_add(1);
    // php-src's restart runs `zend_reset_cache_vars()`, which ZEROES the lookup counters
    // along with flushing the entries — the restart starts a fresh accounting period, not
    // just a fresh cache. VERIFIED on reference PHP 8.5.10 through `php -S`, which runs many
    // requests in one process so the deferred restart is actually performed: a request
    // reporting `hits=4 misses=2` before the reset is followed, after the restart, by
    // `hits=0` plus only the misses that request itself incurred. Carrying them over left
    // every later request — and both ratios — inflated for the life of the worker.
    cache.hits = 0;
    cache.misses = 0;
    cache.blacklist_misses = 0;
    cache.entries.clear();
    cache.used_memory = 0;
    cache.cache_full = false;
    cache.manual_restarts += 1;
    cache.last_restart_time = now_seconds();
    cache.restart_pending = false;
    true
}

/// Marks a cached script discarded, as a forced `opcache_invalidate()` does.
///
/// Returns whether an entry was present to discard. The entry keeps its slot and its
/// accounted footprint, matching php-src holding the shared-memory block until the
/// next restart, so `num_cached_scripts` does not move.
pub fn discard(path: &Path) -> bool {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut cache = lock_script_cache();
    // The mutable borrow of `entries` has to end before `generation` can be touched, so the
    // flag is read back rather than bumped inside the match arm.
    let discarded = match cache.entries.get_mut(&key) {
        Some(entry) => {
            entry.discarded = true;
            true
        }
        None => false,
    };
    if discarded {
        cache.generation = cache.generation.wrapping_add(1);
    }
    discarded
}

/// `opcache_invalidate()`: evicts `path` when php-src's predicate says to.
///
/// php-src's `accel_invalidate` is
/// `force || !validate_timestamps || do_validate_timestamps(...) == FAILURE`, and only the
/// FIRST of those three was implemented. The other two are not corner cases:
///
/// - `!validate_timestamps` is the DEPLOYMENT configuration. With
///   `opcache.validate_timestamps=0` nothing is ever re-stated, so an explicit
///   `opcache_invalidate()` is the ONLY way to retire a script — and it did nothing at all
///   unless the caller also passed `force`. MEASURED: reference reports the script
///   uncached after a plain `opcache_invalidate($p)`; elephc reported it still cached.
/// - the timestamp check is the ordinary `validate_timestamps=1` case, where reference
///   evicts a script whose source has moved on and keeps one that has not. MEASURED, both
///   directions: an untouched file survives a non-forced invalidate in reference too.
///
/// THE STALENESS TEST IS THE TIMESTAMP ALONE, not the mtime-or-size pair the warm-hit path
/// in `serve_warm_entry` uses. Sharing the warm path's definition was tried and is wrong:
/// `do_validate_timestamps` passes `NULL` for the size output and compares only the
/// timestamp, so a rewrite that changes the LENGTH while preserving the mtime is fresh to
/// reference and was stale here. MEASURED: reference reports the script still cached after
/// such a rewrite and a non-forced `opcache_invalidate()`; elephc discarded it.
///
/// Two definitions of "changed" in one crate is a real cost, and it buys correctness rather
/// than tidiness: the warm path is deciding whether to SERVE an entry, where a size change
/// with an unchanged mtime is worth refusing, and this is reproducing a php-src predicate
/// that a program can observe directly.
///
/// RETURNS THE EVICTION, NOT THE PHP ANSWER. `opcache_invalidate()` reports whether the
/// PATH RESOLVES, which is a question about the filesystem and not about the cache; that
/// stays with the builtin, where the rest of the PHP-visible behaviour lives. This reports
/// whether an entry was actually dropped, which is what a caller here can use and what a
/// test can assert on.
pub fn invalidate(path: &Path, force: bool) -> bool {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    // THE ON-DISK ENTRY GOES FIRST, AND UNCONDITIONALLY. php-src calls
    // `zend_file_cache_invalidate` outside the `force || !validate_timestamps || stale`
    // test, so an invalidate that deliberately KEEPS the in-memory entry still drops the
    // disk copy. MEASURED: reference reports `mem_after=1 disk_after=0` for an unchanged
    // file under `validate_timestamps=1`; elephc reported `disk_after=1`, because this
    // returned before reaching the removal.
    //
    // That ordering is the point rather than an accident. The in-memory entry dies with the
    // process; the disk entry outlives it, so leaving one behind is how an invalidated
    // script comes back in the next process — the failure the call exists to prevent.
    let removed = super::file_store::invalidate(&config(), &key);
    if !force && config().validate_timestamps && !entry_is_stale(&key) {
        return removed;
    }
    // A SECOND INVALIDATE RETIRES NOTHING. `discard` answers whether an entry is PRESENT,
    // which stays true once the latch is set, so chaining it here reported success for a
    // call that did no work — reference answers `false` for the second one. The liveness is
    // read before the discard rather than inferred from it, because the latch is idempotent
    // by design and cannot distinguish the two on its own.
    let was_live = is_cached(&key);
    discard(&key);
    was_live || removed
}

/// Returns whether `key`'s entry no longer matches the file on disk — php-src's
/// `do_validate_timestamps(...) == FAILURE`.
///
/// THE TIMESTAMP ONLY. php-src hands that function a `NULL` size output and compares the
/// mtime, so a rewrite that changes the length while preserving the timestamp leaves the
/// entry valid. Comparing the size here too made such a file stale and retired an entry
/// reference keeps.
///
/// A path with NO entry is not stale; there is nothing to be stale about, and reporting it
/// so would make the predicate above take a branch that has no work to do. A file that can
/// no longer be stated IS stale: it was cached and is now unreadable, which is the strongest
/// possible reason not to keep serving it.
fn entry_is_stale(key: &Path) -> bool {
    let cache = lock_script_cache();
    let Some(entry) = cache.entries.get(key) else {
        return false;
    };
    let entry_mtime = entry.mtime;
    drop(cache);
    match std::fs::metadata(key) {
        Ok(metadata) => mtime_seconds(&metadata) != entry_mtime,
        Err(_) => true,
    }
}

/// Returns whether a path has a live (present, non-discarded) cache entry.
pub fn is_cached(path: &Path) -> bool {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    lock_script_cache()
        .entries
        .get(&key)
        .is_some_and(|entry| !entry.discarded)
}

/// Reads and caches a script without executing it, as `opcache_compile_file()` does.
///
/// Returns whether the file could be read AND PARSED. A file that parses but the budget
/// refuses still reports success: php-src reports the COMPILE, not the store.
///
/// THE PARSE RESULT HAS TO BE INSPECTED, not inferred from `fill_entry` succeeding.
/// `fill_entry` returns `Ok` for a file that did not parse — deliberately, because the
/// segments it hands back carry the error so a later `include` can RAISE it at the right
/// moment. Reading `is_ok()` as "compiled" therefore reported success for a file with a
/// syntax error, which is the one answer `opcache_compile_file()` must never give: the
/// caller is told the file is ready and nothing is cached.
///
/// DIVERGENCE, stated rather than hidden: reference PHP 8.5 THROWS a `ParseError` here,
/// where this answers `false`. Throwing needs the message and line carried across the
/// bridge, which this signature cannot do; `false` is the honest half of the answer and no
/// longer the wrong one.
///
/// A discarded entry is re-admitted by the fill itself, which inserts a fresh entry with
/// the latch clear. Removing it first would be worse than redundant: the removal does not
/// return the entry's footprint to the budget, so the refill would count the same bytes
/// twice and the entry would be weighed against `max_accelerated_files` as a NEW one.
pub fn compile_file(path: &Path) -> bool {
    let config = config();
    if !config.enabled {
        return false;
    }
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    // A SECOND CALL ON A CACHED FILE IS A HIT, not another compile. php-src's
    // `opcache_compile_file()` goes through the same cache lookup as an include, so calling
    // it twice moves `hits`, not `misses`. Going straight to `fill_entry` re-read and
    // re-parsed the file every time, counted a miss for each, and rebuilt the entry with
    // `hits: 0` and a reset `last_used` — so repeated calls moved every figure the wrong way
    // and dragged `opcache_hit_rate` down with them. MEASURED on reference PHP 8.5.10: three
    // calls give `hits=2 misses=0` beyond the first, against `hits=0 misses=2` here.
    match serve_warm_entry(&key, &config) {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => return false,
    }
    match fill_entry(&key, path, &config) {
        Ok(segments) => !segments
            .iter()
            .any(|segment| matches!(segment, ScriptSegment::ParseError(_))),
        Err(_) => false,
    }
}

/// Returns a counter that changes whenever the cache's entries change.
///
/// The per-script readers snapshot the cache to answer one index at a time; comparing this
/// tells them whether a snapshot they already hold is still good, which is what keeps
/// `opcache_get_status()` from rebuilding it once per field.
pub fn generation() -> u64 {
    lock_script_cache().generation
}

/// Returns the aggregate counters for the `opcache_get_status()` surface.
pub fn stats() -> ScriptCacheStats {
    let cache = lock_script_cache();
    ScriptCacheStats {
        hits: cache.hits,
        misses: cache.misses,
        num_cached_scripts: cache.entries.len(),
        used_memory: cache.used_memory,
        cache_full: cache.cache_full,
        oom_restarts: cache.oom_restarts,
        manual_restarts: cache.manual_restarts,
        last_restart_time: cache.last_restart_time,
        restart_pending: cache.restart_pending,
        blacklist_misses: cache.blacklist_misses,
    }
}

/// Returns one entry per cached script, sorted by path for a deterministic surface.
///
/// A discarded entry reports `timestamp = 0`, which is the single field php-src moves
/// on a discard and the one the compile-time manifest emulation already moves.
pub fn cached_scripts() -> Vec<CachedScriptInfo> {
    let cache = lock_script_cache();
    let mut scripts: Vec<CachedScriptInfo> = cache
        .entries
        .iter()
        .map(|(path, entry)| CachedScriptInfo {
            full_path: path.to_string_lossy().into_owned(),
            hits: entry.hits,
            memory_consumption: entry.footprint,
            last_used_timestamp: entry.last_used,
            timestamp: if entry.discarded { 0 } else { entry.mtime.unwrap_or(0) },
            revalidate_at: entry.revalidate_at,
        })
        .collect();
    scripts.sort_by(|left, right| left.full_path.cmp(&right.full_path));
    scripts
}

/// Serializes tests against the process-wide cache singleton.
#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Takes the cache test lock and resets the cache and its counters to an empty state.
///
/// The cache is a process-wide singleton while its configuration is thread-local, so any
/// test that asserts a counter must both serialize against other tests and start from a
/// known state. Shared with the interpreter's OPcache tests, which drive the same cache
/// through the eval surface.
#[cfg(test)]
pub(crate) fn lock_for_test() -> MutexGuard<'static, ()> {
    let guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut cache = lock_script_cache();
    *cache = ScriptCache::default();
    guard
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
