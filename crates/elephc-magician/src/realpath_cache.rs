//! Purpose:
//! Memoizes `realpath()` for the paths interpreted `include`/`require` resolve, so a long-lived
//! worker pays the walk once per path instead of once per include.
//!
//! Called from:
//! - `crate::interpreter::include_exec::eval_include_key()`.
//!
//! Key details:
//! - This is php-src's `realpath_cache`, for the same reason and with the same defaults:
//!   `realpath_cache_ttl` is 120 seconds and `realpath_cache_size` is 4096K. Caching here is
//!   therefore MATCHING PHP, not diverging from it — a PHP-FPM pool has exactly this window of
//!   staleness after a symlink-swap deploy.
//! - The canonical path is what `include_once` dedupes on and what `get_included_files()` reports,
//!   so it cannot be replaced by the lexical path: two spellings that reach one file must produce
//!   one key, and on macOS even a lexically normal absolute path can traverse a symlinked
//!   component (`/tmp` and `/var` are symlinks to `/private/...`).
//! - The TTL is what bounds the one real hazard. A symlink-swap deploy changes what a path
//!   resolves to; until the entry expires, an `include_once` keyed on the OLD canonical path can
//!   let the new file in a second time, which for a class file is a `Cannot declare class` fatal.
//!   php-src accepts the same window; a shorter one here would only move the boundary.
//! - A canonicalization FAILURE is not cached. It means the file is missing, the include is about
//!   to fail its read anyway, and caching it would keep a file created a moment later invisible.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

/// Maximum number of distinct paths retained, mirroring `realpath_cache_size`'s intent.
const REALPATH_CACHE_CAPACITY: usize = 4096;

/// How long one canonical answer is trusted, matching php's `realpath_cache_ttl` default.
const REALPATH_CACHE_TTL: Duration = Duration::from_secs(120);

static REALPATH_CACHE: OnceLock<Mutex<RealpathCache>> = OnceLock::new();

/// Returns the canonical form of `path`, reusing a recent answer for the same path.
///
/// Falls back to `path` itself when it cannot be canonicalized, which is what the uncached call
/// did: a missing file still needs a stable include key for the read failure that follows.
pub(crate) fn canonicalize_cached(path: &Path) -> PathBuf {
    if let Some(cached) = lock_realpath_cache().lookup(path) {
        return cached;
    }
    let Ok(canonical) = std::fs::canonicalize(path) else {
        return path.to_path_buf();
    };
    lock_realpath_cache().insert(path.to_path_buf(), canonical.clone());
    canonical
}

/// Returns the process-wide realpath cache singleton.
fn realpath_cache() -> &'static Mutex<RealpathCache> {
    REALPATH_CACHE.get_or_init(|| {
        Mutex::new(RealpathCache::new(
            REALPATH_CACHE_CAPACITY,
            REALPATH_CACHE_TTL,
        ))
    })
}

/// Locks the realpath cache and recovers the inner cache if a previous panic poisoned it.
fn lock_realpath_cache() -> MutexGuard<'static, RealpathCache> {
    realpath_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Bounded FIFO cache of canonical paths, each trusted for a fixed time.
struct RealpathCache {
    capacity: usize,
    ttl: Duration,
    entries: HashMap<PathBuf, (PathBuf, Instant)>,
    order: VecDeque<PathBuf>,
}

impl RealpathCache {
    /// Creates an empty cache with the requested maximum entry count and entry lifetime.
    fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            capacity,
            ttl,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Returns the stored canonical path while it is still within its lifetime.
    fn lookup(&self, path: &Path) -> Option<PathBuf> {
        let (canonical, stored_at) = self.entries.get(path)?;
        (stored_at.elapsed() < self.ttl).then(|| canonical.clone())
    }

    /// Stores one canonical path, evicting the oldest distinct path when full.
    ///
    /// Re-storing a path that is already present refreshes its answer and its clock without
    /// touching the eviction order: an expired entry is overwritten in place, and letting it also
    /// move to the back would let a single hot path starve the rest of the cache.
    fn insert(&mut self, path: PathBuf, canonical: PathBuf) {
        if self.capacity == 0 {
            return;
        }
        if self.entries.contains_key(&path) {
            self.entries.insert(path, (canonical, Instant::now()));
            return;
        }
        while self.entries.len() >= self.capacity {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.entries.remove(&oldest);
        }
        self.order.push_back(path.clone());
        self.entries.insert(path, (canonical, Instant::now()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies a stored answer is served again without touching the filesystem.
    #[test]
    fn serves_a_stored_answer_within_its_lifetime() {
        let mut cache = RealpathCache::new(4, Duration::from_secs(120));
        cache.insert(PathBuf::from("/a/link"), PathBuf::from("/a/real"));
        assert_eq!(
            cache.lookup(Path::new("/a/link")),
            Some(PathBuf::from("/a/real"))
        );
    }

    /// Verifies an expired answer is withheld, so the next include re-walks the path.
    #[test]
    fn withholds_an_answer_past_its_lifetime() {
        let mut cache = RealpathCache::new(4, Duration::ZERO);
        cache.insert(PathBuf::from("/a/link"), PathBuf::from("/a/real"));
        assert_eq!(cache.lookup(Path::new("/a/link")), None);
    }

    /// Verifies the oldest distinct path leaves once the cache is full.
    #[test]
    fn evicts_the_oldest_path_when_full() {
        let mut cache = RealpathCache::new(2, Duration::from_secs(120));
        cache.insert(PathBuf::from("/one"), PathBuf::from("/one"));
        cache.insert(PathBuf::from("/two"), PathBuf::from("/two"));
        cache.insert(PathBuf::from("/three"), PathBuf::from("/three"));
        assert_eq!(cache.lookup(Path::new("/one")), None);
        assert_eq!(
            cache.lookup(Path::new("/three")),
            Some(PathBuf::from("/three"))
        );
    }

    /// Verifies a repeat store refreshes the answer rather than adding a second entry.
    #[test]
    fn refreshes_a_path_already_stored() {
        let mut cache = RealpathCache::new(2, Duration::from_secs(120));
        cache.insert(PathBuf::from("/link"), PathBuf::from("/old"));
        cache.insert(PathBuf::from("/link"), PathBuf::from("/new"));
        assert_eq!(cache.order.len(), 1);
        assert_eq!(
            cache.lookup(Path::new("/link")),
            Some(PathBuf::from("/new"))
        );
    }
}
