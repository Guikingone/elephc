//! Purpose:
//! The on-disk half of `opcache.file_cache`: stores a script's parsed segments under the
//! configured directory so a process that starts cold can skip the read, the `<?php` scan
//! and the parse. php-src's file cache does the same for its opcodes.
//!
//! Called from:
//! - `crate::script_cache::store` on a fill (write) and on a miss (read).
//! - `crate::script_cache::file_cache` for the `opcache_is_script_cached_in_file_cache()`
//!   answer.
//!
//! Key details:
//! - WHY IT PAYS, measured rather than assumed (`super::format_bench`): bincode decoding is
//!   3.5–5x faster than re-parsing at ~1.8x the source size. `serde_json`, the format this
//!   crate already had, decodes a small script SLOWER than parsing it — which is why the
//!   dependency exists.
//! - ENTRIES ARE NEVER TRUSTED BLIND. Three independent checks must all pass before one is
//!   used: the [`SYSTEM_ID`] directory (so a different build never reads another's IR), the
//!   header's magic and format version, and the source's own mtime, size and canonical path.
//!   Any mismatch — or any deserialization error — is treated as a miss, never as an error:
//!   a cache that cannot be read is a cache that is not there.
//! - The canonical path lives in the HEADER, not only in the file name. The name is a
//!   64-bit hash, so two paths can collide; verifying the path on read makes a collision a
//!   miss instead of a wrong script.
//! - Every filesystem failure is swallowed. A cache is an optimisation, so a full disk or a
//!   read-only mount must degrade to "no cache", never to a failed include.

use super::config::ScriptCacheConfig;
use super::segments::ScriptSegment;
use bincode::Options;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Bumped whenever the stored shape changes in a way an older file could not be read as.
///
/// THIS IS A CORRECTNESS SWITCH, not a label. bincode is not self-describing: a payload
/// written by a different eval-IR shape can decode into plausible-looking garbage rather
/// than failing, and the result would be executed. `super::format_guard` fails the build's
/// tests when the IR changes without this moving, so the bump is not left to memory.
pub(crate) const FORMAT_VERSION: u32 = 1;

/// Identifies the writer, so one build never reads another's entries.
///
/// php-src calls this the `system_id` and derives it from the PHP version, extension set
/// and build flags. The analogue here is the crate version plus the format version: the
/// eval IR travels inside this crate, so a released change to it moves the former and a
/// development change moves the latter.
fn system_id() -> String {
    format!("{}-{}", env!("CARGO_PKG_VERSION"), FORMAT_VERSION)
}

/// What one cache file holds. The header fields are all validated before the segments are
/// used, so this struct is the contract rather than a convenience wrapper.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CacheFile {
    /// Guards against reading a file that is not one of ours at all.
    magic: u64,
    /// Guards against an older or newer stored shape. See [`FORMAT_VERSION`].
    format_version: u32,
    /// The canonical source path, so a file-name hash collision is a miss, not a mix-up.
    path: String,
    /// The source's mtime when the entry was written.
    mtime: Option<i64>,
    /// The source's length when the entry was written.
    size: u64,
    /// The parsed script.
    segments: Vec<ScriptSegment>,
}

/// `elephc opcache file cache`, as a little-endian tag. Any other leading bytes are foreign.
const MAGIC: u64 = 0x454C_5048_435F_4F46;

/// Returns the directory this build's entries live in, or `None` when caching is off.
///
/// Answers `None` for an unconfigured `opcache.file_cache`, which is php-src's default and
/// the state in which the whole feature is inert.
fn cache_dir(config: &ScriptCacheConfig) -> Option<PathBuf> {
    let file_cache = super::file_cache::file_cache_config();
    if !config.enabled || file_cache.path.is_empty() {
        return None;
    }
    Some(Path::new(&file_cache.path).join(system_id()))
}

/// Returns the file one script's entry is stored in.
///
/// The name is a 64-bit hash of the canonical path rather than the path itself: a path can
/// be longer than a file name may be, and can contain separators. The full path is carried
/// in the header, so the hash only has to spread entries out, not identify them.
fn entry_path(dir: &Path, canonical: &Path) -> PathBuf {
    dir.join(format!(
        "{:016x}.bin",
        stable_hash(canonical.to_string_lossy().as_bytes())
    ))
}

/// A SPECIFIED 64-bit hash (FNV-1a), used wherever a value has to mean the same thing across
/// processes, toolchains and releases.
///
/// `DefaultHasher` does not qualify and the standard library says so: its algorithm is
/// explicitly unspecified and free to change between Rust versions. That is fine for a
/// `HashMap` living inside one process and wrong for a name written to disk — a toolchain
/// upgrade would silently rename every entry, orphaning the whole cache directory rather than
/// reusing it. It is wrong for the same reason in the format fingerprint, where it would move
/// a guard that is supposed to move only when the format does.
///
/// FNV-1a is chosen for being short enough to read and fixed forever, not for its quality:
/// this only has to spread entries out, and every entry is identity-checked after it is read.
pub(crate) fn stable_hash(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Reads a script's segments from the file cache, if a valid entry is there.
///
/// Returns `None` for every failure mode — absent, foreign, stale, unreadable or corrupt —
/// because each of them means the same thing to the caller: parse the file instead.
pub(crate) fn load(
    config: &ScriptCacheConfig,
    canonical: &Path,
    mtime: Option<i64>,
    size: u64,
) -> Option<Vec<ScriptSegment>> {
    let dir = cache_dir(config)?;
    let bytes = std::fs::read(entry_path(&dir, canonical)).ok()?;
    // The header is checked on the RAW BYTES, before anything is decoded. The same two fields
    // are re-checked below on the decoded struct, and that is not redundant: the decoded check
    // can only reject what the decoder already agreed to build, so it cannot protect the
    // decode itself. A file whose length prefixes are hostile is consumed before a single
    // identity field has been looked at.
    if !header_is_ours(&bytes) {
        return None;
    }
    // Bounded decode. `bincode::deserialize` is `DefaultOptions` + fixint + trailing bytes with
    // NO byte limit, so a corrupt length prefix asks the allocator for whatever it says. The
    // limit is the file's own size: nothing legitimately decodes to more than it was read from,
    // and it needs no tuning as the format grows. The encoding flags reproduce
    // `bincode::deserialize` exactly — `DefaultOptions` alone is VARINT, which would silently
    // stop reading entries written by `bincode::serialize` on the store side.
    let decoder = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(bytes.len() as u64);
    let cached: CacheFile = decoder.deserialize(&bytes).ok()?;
    if cached.magic != MAGIC || cached.format_version != FORMAT_VERSION {
        return None;
    }
    // The source must be EXACTLY what was stored. `opcache.validate_timestamps` governs how
    // often an in-memory entry is re-checked; it does not license running a stale file from
    // disk, so this comparison is unconditional.
    if cached.path != canonical.to_string_lossy() || cached.mtime != mtime || cached.size != size {
        return None;
    }
    Some(cached.segments)
}

/// Writes a script's segments to the file cache, doing nothing when it cannot.
///
/// Silent by design: `opcache.file_cache_read_only` forbids writing, a missing directory is
/// created if possible, and any I/O failure leaves the caller with a working — merely
/// uncached — include.
pub(crate) fn store(
    config: &ScriptCacheConfig,
    canonical: &Path,
    mtime: Option<i64>,
    size: u64,
    segments: &[ScriptSegment],
) {
    if super::file_cache::file_cache_config().read_only {
        return;
    }
    let Some(dir) = cache_dir(config) else {
        return;
    };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let payload = CacheFile {
        magic: MAGIC,
        format_version: FORMAT_VERSION,
        path: canonical.to_string_lossy().into_owned(),
        mtime,
        size,
        segments: segments.to_vec(),
    };
    let Ok(bytes) = bincode::serialize(&payload) else {
        return;
    };
    // Written through a temporary and renamed, so a reader never sees a half-written entry.
    // A crashed write leaves a stray temp file rather than a corrupt cache hit.
    //
    // The temporary is created with `create_new` (`O_CREAT|O_EXCL`), which both refuses to
    // follow a pre-existing symlink at that path and refuses to collide with another worker.
    // `std::fs::write` is `File::create` — `O_TRUNC`, no `O_EXCL`, no `O_NOFOLLOW` — so with a
    // name as predictable as the old `<entry>.tmp<pid>` it would truncate whatever a symlink
    // planted there pointed at. The collision half is not hypothetical even without an
    // attacker: PIDs are reused, and `--web` preforks several workers over one directory.
    let target = entry_path(&dir, canonical);
    let Some((mut file, temp)) = create_temp_entry(&target) else {
        return;
    };
    if file.write_all(&bytes).is_err() {
        drop(file);
        let _ = std::fs::remove_file(&temp);
        return;
    }
    drop(file);
    if std::fs::rename(&temp, &target).is_err() {
        let _ = std::fs::remove_file(&temp);
    }
}

/// Returns whether these bytes even claim to be one of our cache entries.
///
/// `bincode::serialize` writes `CacheFile`'s fields in declaration order with fixed-width
/// little-endian integers, so the first twelve bytes are `magic` then `format_version`.
/// `first_bytes_are_the_header` pins that layout against a real round trip, so this stays
/// honest if the struct is ever reordered.
fn header_is_ours(bytes: &[u8]) -> bool {
    const HEADER_LEN: usize = 12;
    if bytes.len() < HEADER_LEN {
        return false;
    }
    let Ok(magic) = <[u8; 8]>::try_from(&bytes[0..8]) else {
        return false;
    };
    let Ok(version) = <[u8; 4]>::try_from(&bytes[8..12]) else {
        return false;
    };
    u64::from_le_bytes(magic) == MAGIC && u32::from_le_bytes(version) == FORMAT_VERSION
}

/// Creates an exclusive, unguessable temporary next to `target`.
///
/// The nonce only spreads attempts out; `create_new` is what provides the safety, so a weak
/// source of uniqueness costs a retry rather than correctness. Gives up after a few attempts
/// so a directory that refuses every create fails the store instead of spinning.
fn create_temp_entry(target: &Path) -> Option<(std::fs::File, PathBuf)> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    for _ in 0..8 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let nonce = stable_hash(
            &[
                nanos.to_le_bytes(),
                u64::from(std::process::id()).to_le_bytes(),
                COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes(),
            ]
            .concat(),
        );
        let temp = target.with_extension(format!("tmp{nonce:016x}"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
        {
            Ok(file) => return Some((file, temp)),
            Err(_) => continue,
        }
    }
    None
}

/// Returns whether a usable entry for `canonical` is on disk right now.
///
/// This is what `opcache_is_script_cached_in_file_cache()` answers, so it applies the same
/// validation a read does: reporting an entry that a read would reject would be a lie.
pub(crate) fn contains(canonical: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(canonical) else {
        return false;
    };
    let canonical = std::fs::canonicalize(canonical).unwrap_or_else(|_| canonical.to_path_buf());
    let mtime = super::store::mtime_seconds(&metadata);
    load(
        &super::config::config(),
        &canonical,
        mtime,
        metadata.len(),
    )
    .is_some()
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Pins the three independent checks an entry must pass, because each of them exists to
    //! stop a WRONG script being executed rather than merely to avoid a miss.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Each test uses its own directory, so they stay independent under the parallel
    //!   harness even though the configuration they read is thread-local.

    use super::super::config::ScriptCacheConfig;
    use super::super::file_cache::{set_file_cache_config, FileCacheConfig};
    use super::super::segments::{segment_script, ParseMode};
    use super::*;

    /// Points the thread's file cache at a fresh directory and returns it.
    fn configure(name: &str, read_only: bool) -> (ScriptCacheConfig, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "elephc-file-store-{}-{}-{:?}",
            name,
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).expect("cache dir");
        set_file_cache_config(FileCacheConfig {
            path: dir.to_string_lossy().into_owned(),
            read_only,
        });
        let config = ScriptCacheConfig {
            enabled: true,
            ..ScriptCacheConfig::disabled()
        };
        (config, dir)
    }

    /// Verifies a stored script comes back with the same segment shape.
    #[test]
    fn a_stored_script_round_trips() {
        let (config, _dir) = configure("roundtrip", false);
        let source = b"<?php $a = 1; ?>tail<?php $b = 2;";
        let segments = segment_script(source, ParseMode::Fresh);
        let path = Path::new("/tmp/elephc-file-store-roundtrip.php");

        store(&config, path, Some(42), source.len() as u64, &segments);
        let loaded = load(&config, path, Some(42), source.len() as u64);

        assert_eq!(
            loaded.map(|s| s.len()),
            Some(segments.len()),
            "the entry must come back with every segment"
        );
    }

    /// Verifies a source whose mtime or size moved is REFUSED rather than served stale.
    ///
    /// This is the check that keeps a changed file from running as its old self, so it is
    /// unconditional — `opcache.validate_timestamps` governs re-`stat` frequency in memory,
    /// not whether a disk entry may be trusted.
    #[test]
    fn a_moved_source_is_refused() {
        let (config, _dir) = configure("stale", false);
        let source = b"<?php $a = 1;";
        let segments = segment_script(source, ParseMode::Fresh);
        let path = Path::new("/tmp/elephc-file-store-stale.php");
        store(&config, path, Some(42), source.len() as u64, &segments);

        assert!(
            load(&config, path, Some(43), source.len() as u64).is_none(),
            "mtime moved"
        );
        assert!(load(&config, path, Some(42), 999).is_none(), "size moved");
        assert!(
            load(&config, path, Some(42), source.len() as u64).is_some(),
            "unchanged hits"
        );
    }

    /// Verifies another path never reads this one's entry, even on a name-hash collision.
    ///
    /// The file name is a 64-bit hash, so a collision is possible; the canonical path in the
    /// header is what makes one a miss instead of the wrong script.
    #[test]
    fn another_path_never_reads_this_entry() {
        let (config, _dir) = configure("path", false);
        let source = b"<?php $a = 1;";
        let segments = segment_script(source, ParseMode::Fresh);
        let mine = Path::new("/tmp/elephc-file-store-mine.php");
        store(&config, mine, Some(42), source.len() as u64, &segments);

        let theirs = Path::new("/tmp/elephc-file-store-theirs.php");
        assert!(load(&config, theirs, Some(42), source.len() as u64).is_none());
    }

    /// Verifies `opcache.file_cache_read_only` writes nothing.
    #[test]
    fn read_only_stores_nothing() {
        let (config, dir) = configure("readonly", true);
        let source = b"<?php $a = 1;";
        let segments = segment_script(source, ParseMode::Fresh);
        let path = Path::new("/tmp/elephc-file-store-readonly.php");

        store(&config, path, Some(42), source.len() as u64, &segments);

        assert!(load(&config, path, Some(42), source.len() as u64).is_none());
        assert!(
            !dir.join(system_id()).exists(),
            "not even the directory is created"
        );
    }

/// Verifies the raw-byte header check reads the fields it thinks it reads.
    ///
    /// `header_is_ours` asserts a LAYOUT — magic at 0..8, version at 8..12, little-endian —
    /// that no compiler checks, so a reordered struct or a changed bincode configuration would
    /// turn it into a check of the wrong bytes without any build failure. Proving it against a
    /// real `store()` round trip is the only way that assumption stays honest.
    #[test]
    fn first_bytes_are_the_header() {
        let (config, _dir) = configure("header", false);
        let source = b"<?php $a = 1;";
        let segments = segment_script(source, ParseMode::Fresh);
        let path = Path::new("/tmp/elephc-file-store-header.php");
        store(&config, path, Some(7), source.len() as u64, &segments);

        let dir = cache_dir(&config).expect("cache dir configured");
        let bytes = std::fs::read(entry_path(&dir, path)).expect("entry written");
        assert!(
            header_is_ours(&bytes),
            "a freshly stored entry must pass its own header check"
        );
        assert_eq!(u64::from_le_bytes(bytes[0..8].try_into().unwrap()), MAGIC);
        assert_eq!(
            u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            FORMAT_VERSION
        );

        // And the check actually discriminates, rather than accepting everything.
        assert!(!header_is_ours(&[]), "an empty file is not an entry");
        assert!(!header_is_ours(&bytes[..11]), "a truncated header is not an entry");
        let mut foreign = bytes.clone();
        foreign[0] ^= 0xff;
        assert!(!header_is_ours(&foreign), "a foreign magic is not an entry");
    }

    /// Verifies a hostile entry is refused rather than acted on.
    ///
    /// The entry keeps a VALID header — that is the point, since a wrong one is already
    /// rejected a step earlier — and then claims a string far larger than the file.
    ///
    /// MEASURED LIMIT OF THIS TEST, stated because it would otherwise read as proof of
    /// something it does not show: it passes with `with_limit` removed. bincode 1.3 decoding
    /// from a slice bounds byte buffers by the remaining input, and serde caps `Vec`
    /// preallocation, so the "unbounded allocation" half of the review finding does not
    /// reproduce on this reader. The limit is kept as defence in depth — it costs one builder
    /// call and it stops being free only if this ever decodes from a streaming reader, where
    /// the bound disappears — but nothing here demonstrates it is load-bearing today.
    ///
    /// What the test DOES pin is the refusal: a malformed entry returns `None` and the caller
    /// parses the script, rather than half-decoding one into execution.
    #[test]
    fn a_hostile_length_prefix_is_refused_not_allocated() {
        let (config, _dir) = configure("hostile", false);
        let dir = cache_dir(&config).expect("cache dir configured");
        std::fs::create_dir_all(&dir).expect("entry dir");
        let path = Path::new("/tmp/elephc-file-store-hostile.php");

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC.to_le_bytes());
        bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        // A `String` length of 2^60: the decoder would reserve it before reading a byte of it.
        bytes.extend_from_slice(&(1u64 << 60).to_le_bytes());
        bytes.extend_from_slice(b"short");
        std::fs::write(entry_path(&dir, path), &bytes).expect("hostile entry written");

        assert!(
            load(&config, path, Some(1), 5).is_none(),
            "a hostile entry must be refused"
        );
    }

    /// Verifies the temporary is created exclusively, so a planted symlink is not followed.
    ///
    /// The old `<entry>.tmp<pid>` name was fully predictable and opened through
    /// `std::fs::write`, i.e. `O_TRUNC` with neither `O_EXCL` nor `O_NOFOLLOW`: a symlink left
    /// at that path pointed the next cache fill at any file the worker could write. This pins
    /// the property that makes that impossible — `create_new` on a path that already exists
    /// fails rather than following it — without depending on the nonce being unguessable.
    #[test]
    fn a_pre_existing_temp_path_is_never_followed() {
        let (_config, dir) = configure("symlink", false);
        let target = dir.join("victim-target.bin");
        std::fs::write(&target, b"ORIGINAL").expect("victim written");

        // Reproduce the collision deterministically: create_temp_entry hands back the name it
        // chose, so plant a file AT that name and ask for it again.
        let entry = dir.join("entry.bin");
        let (file, temp) = create_temp_entry(&entry).expect("first temp");
        drop(file);
        assert!(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .is_err(),
            "create_new must refuse a path that already exists"
        );

        // The same refusal is what protects a symlink: replace the temp with one and confirm
        // the open fails instead of truncating the victim through it.
        std::fs::remove_file(&temp).expect("temp removed");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &temp).expect("symlink planted");
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temp)
                    .is_err(),
                "create_new must refuse an existing symlink rather than follow it"
            );
            assert_eq!(
                std::fs::read(&target).expect("victim still readable"),
                b"ORIGINAL",
                "the symlink target was written through"
            );
        }
    }

        /// Verifies an unconfigured `opcache.file_cache` is completely inert.
    #[test]
    fn an_unconfigured_directory_stores_nothing() {
        set_file_cache_config(FileCacheConfig::new());
        let config = ScriptCacheConfig {
            enabled: true,
            ..ScriptCacheConfig::disabled()
        };
        let segments = segment_script(b"<?php $a = 1;", ParseMode::Fresh);
        let path = Path::new("/tmp/elephc-file-store-unset.php");

        store(&config, path, Some(42), 13, &segments);

        assert!(load(&config, path, Some(42), 13).is_none());
    }
}
