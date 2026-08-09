//! Purpose:
//! Builds and caches the reusable runtime object that is linked beside generated user code.
//! Keys cache entries by compiler version, target, heap size, and runtime feature shape.
//!
//! Called from:
//! - `crate::pipeline::compile()` before user assembly is linked into the final binary.
//!
//! Key details:
//! - Temporary assembly/object files are renamed into place to tolerate concurrent compiler runs.
//! - Prepared objects use short-lived hardlink leases so pruning cannot invalidate a concurrent link.
//! - Best-effort pruning retains at most eight published object/sidecar pairs outside live leases
//!   and removes only compiler-shaped temporary files whose owner process is known to be dead.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use fs2::FileExt;

use crate::codegen;
use crate::codegen::platform::{Platform, Target};
use crate::codegen::RuntimeFeatures;

/// Maximum number of complete runtime object entries retained in the shared cache.
const MAX_RUNTIME_CACHE_OBJECTS: usize = 8;
/// Monotonic process-local discriminator for lease names created in the same clock tick.
static RUNTIME_CACHE_LEASE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Runtime cache hit/miss status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeCacheStatus {
    Hit,
    Miss,
}

impl RuntimeCacheStatus {
    /// Returns a static string slice describing the cache status.
    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeCacheStatus::Hit => "hit",
            RuntimeCacheStatus::Miss => "miss",
        }
    }
}

/// Prepared runtime object with cache status.
#[derive(Debug)]
pub struct PreparedRuntimeObject {
    /// Path to the leased linker-visible snapshot of the cached runtime object.
    pub path: PathBuf,
    /// Whether the object was found in the cache (hit) or built now (miss).
    pub status: RuntimeCacheStatus,
    /// Keeps the linker-visible hardlink snapshot alive until this prepared object is dropped.
    _lease: RuntimeObjectLease,
}

/// One linker-visible snapshot of a cached runtime object and its integrity sidecar.
///
/// The snapshot paths are hardlinks, so they consume no duplicate object storage. A
/// shared advisory lock marks the lease as live across processes; pruning only cleans
/// unlocked markers left behind by completed or crashed compiler processes.
#[derive(Debug)]
struct RuntimeObjectLease {
    object_path: PathBuf,
    integrity_path: PathBuf,
    lock_path: PathBuf,
    lock_file: File,
}

impl Drop for RuntimeObjectLease {
    /// Removes this process's linker snapshot and releases its cross-process liveness marker.
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.object_path);
        let _ = fs::remove_file(&self.integrity_path);
        let _ = FileExt::unlock(&self.lock_file);
        let _ = fs::remove_file(&self.lock_path);
    }
}

/// Builds (or retrieves from cache) the runtime object file for the given heap size, target, and features.
/// On cache miss, generates runtime assembly, assembles it to an object file, and caches the result.
/// The cache key includes compiler version, target, heap size, the PIC mode, and the typed runtime
/// feature shape. A sidecar checksum validates cache bytes before a hit is accepted.
/// `pic` selects position-independent emission for `--emit cdylib` artifacts so the runtime object can be
/// linked into a shared library without text-segment relocations. The returned path remains valid until
/// the `PreparedRuntimeObject` is dropped, even if another compiler prunes the canonical cache entry.
pub fn prepare_runtime_object(
    heap_size: usize,
    target: Target,
    features: RuntimeFeatures,
    pic: bool,
) -> Result<PreparedRuntimeObject, String> {
    let cache_dir = runtime_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|err| format!("failed to create runtime cache '{}': {}", cache_dir.display(), err))?;
    harden_runtime_cache_dir(&cache_dir)?;

    // Worktrees commonly share a package version and cache directory while their
    // runtime emitters differ. Cargo supplies a source-derived emitter identity
    // at build time, letting a warm cache hit avoid regenerating the full runtime
    // assembly while still rejecting an object from a different source revision.
    let cache_key = runtime_cache_key_with_build_identity(
        heap_size,
        target,
        features,
        pic,
        env!("ELEPHC_RUNTIME_BUILD_ID").as_bytes(),
    );
    let cache_path = cache_dir.join(runtime_cache_file_name(heap_size, target, cache_key));
    let integrity_path = cache_dir.join(format!(
        "{}.integrity",
        cache_path.file_name().and_then(|name| name.to_str()).unwrap_or("runtime.o")
    ));
    if cache_path.exists() && runtime_object_is_intact(&cache_path, &integrity_path) {
        if let Some(prepared) = lease_runtime_object(
            &cache_path,
            &integrity_path,
            RuntimeCacheStatus::Hit,
        )? {
            prune_runtime_cache_objects(&cache_dir, &cache_path);
            return Ok(prepared);
        }
    }

    let runtime_asm =
        codegen::generate_runtime_with_features_pic(heap_size, target, features, pic);

    let unique = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let stem = cache_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("runtime");
    let temp_asm_path = cache_dir.join(format!("{stem}.{unique}.s"));
    let temp_obj_path = cache_dir.join(format!("{stem}.{unique}.o"));
    fs::write(&temp_asm_path, runtime_asm).map_err(|err| {
        format!(
            "failed to write temporary runtime assembly '{}': {}",
            temp_asm_path.display(),
            err
        )
    })?;

    let mut assembler = Command::new(target.assembler_cmd());
    if target.platform == Platform::MacOS {
        assembler.args(["-arch", target.darwin_arch_name()]);
    }
    assembler.arg("-o").arg(&temp_obj_path).arg(&temp_asm_path);
    let assembler_status = assembler.status().map_err(|err| {
        format!(
            "failed to run runtime assembler '{}' for '{}': {}",
            target.assembler_cmd(),
            temp_obj_path.display(),
            err
        )
    })?;
    let _ = fs::remove_file(&temp_asm_path);
    if !assembler_status.success() {
        let _ = fs::remove_file(&temp_obj_path);
        return Err(format!(
            "runtime assembler failed while building '{}'",
            cache_path.display()
        ));
    }

    let status = match fs::rename(&temp_obj_path, &cache_path) {
        Ok(()) => {
            write_runtime_object_integrity(&cache_path, &integrity_path)?;
            RuntimeCacheStatus::Miss
        }
        Err(_err) if cache_path.exists() && runtime_object_is_intact(&cache_path, &integrity_path) => {
            let _ = fs::remove_file(&temp_obj_path);
            RuntimeCacheStatus::Hit
        }
        Err(err) => {
            let _ = fs::remove_file(&temp_obj_path);
            return Err(format!(
                "failed to store runtime cache '{}': {}",
                cache_path.display(),
                err
            ));
        }
    };
    let Some(prepared) = lease_runtime_object(&cache_path, &integrity_path, status)? else {
        return prepare_runtime_object(heap_size, target, features, pic);
    };
    prune_runtime_cache_objects(&cache_dir, &cache_path);
    Ok(prepared)
}

/// Returns the platform-specific cache directory path for runtime objects.
fn runtime_cache_dir() -> PathBuf {
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(path).join("elephc")
    } else if let Some(home) = env::var_os("HOME") {
        PathBuf::from(home).join(".cache").join("elephc")
    } else {
        env::temp_dir().join("elephc-cache")
    }
}

/// Builds the cache file name for a runtime object.
fn runtime_cache_file_name(heap_size: usize, target: Target, runtime_hash: u64) -> String {
    format!(
        "runtime-v{}-{}-rt{:016x}-heap{}.o",
        env!("CARGO_PKG_VERSION"),
        target.as_str(),
        runtime_hash,
        heap_size
    )
}

/// Creates a process-scoped hardlink snapshot for one intact cache entry.
///
/// The returned path, rather than the canonical cache name, is passed to the
/// linker. Canonical pruning or replacement therefore cannot invalidate a
/// prepared object that another compiler still owns. The marker lock closes the
/// crash-cleanup race: a live holder keeps a shared lock, while pruning removes
/// only markers on which it can acquire an exclusive lock without waiting.
fn lease_runtime_object(
    cache_path: &Path,
    integrity_path: &Path,
    status: RuntimeCacheStatus,
) -> Result<Option<PreparedRuntimeObject>, String> {
    for _ in 0..4 {
        let sequence = RUNTIME_CACHE_LEASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let file_name = cache_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("runtime.o");
        let lease_name = format!(
            "{file_name}.lease-{}-{timestamp}-{sequence}",
            std::process::id()
        );
        let cache_dir = cache_path.parent().unwrap_or_else(|| Path::new("."));
        let object_path = cache_dir.join(&lease_name);
        let lease_integrity_path = cache_dir.join(format!("{lease_name}.integrity"));
        let lock_path = cache_dir.join(format!("{lease_name}.lock"));
        let lock_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(format!(
                    "failed to create runtime cache lease '{}': {}",
                    lock_path.display(),
                    err
                ));
            }
        };
        FileExt::lock_shared(&lock_file).map_err(|err| {
            let _ = fs::remove_file(&lock_path);
            format!(
                "failed to lock runtime cache lease '{}': {}",
                lock_path.display(),
                err
            )
        })?;
        let lease = RuntimeObjectLease {
            object_path: object_path.clone(),
            integrity_path: lease_integrity_path,
            lock_path,
            lock_file,
        };

        // A stale-lease collector can win between marker creation and locking.
        // In that case the locked descriptor refers to an unlinked inode, so retry
        // under a fresh unique marker rather than publishing an invisible lease.
        if !lease.lock_path.exists() {
            continue;
        }
        if let Err(err) = fs::hard_link(cache_path, &lease.object_path) {
            if err.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(format!(
                "failed to lease runtime cache object '{}': {}",
                cache_path.display(),
                err
            ));
        }
        if let Err(err) = fs::hard_link(integrity_path, &lease.integrity_path) {
            if err.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(format!(
                "failed to lease runtime cache integrity '{}': {}",
                integrity_path.display(),
                err
            ));
        }
        if !runtime_object_is_intact(&lease.object_path, &lease.integrity_path) {
            return Ok(None);
        }
        return Ok(Some(PreparedRuntimeObject {
            path: object_path,
            status,
            _lease: lease,
        }));
    }
    Err(format!(
        "failed to allocate a unique runtime cache lease for '{}'",
        cache_path.display()
    ))
}

/// Removes the oldest superseded runtime object/sidecar pairs beyond the cache bound.
///
/// Housekeeping is best-effort so an unreadable stale entry cannot fail compilation.
/// The active object is always excluded, temporary compiler outputs do not match the
/// canonical filename shape, and unrelated cache-directory files are left untouched.
fn prune_runtime_cache_objects(cache_dir: &Path, active_object: &Path) {
    cleanup_stale_runtime_cache_leases(cache_dir);
    cleanup_stale_runtime_cache_temporaries(cache_dir);
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    let mut objects: Vec<(PathBuf, SystemTime)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !runtime_cache_object_name_is_canonical(&path) {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            Some((path, metadata.modified().unwrap_or(UNIX_EPOCH)))
        })
        .collect();
    if objects.len() <= MAX_RUNTIME_CACHE_OBJECTS {
        return;
    }
    objects.sort_by_key(|(_, modified)| *modified);
    let remove_count = objects.len() - MAX_RUNTIME_CACHE_OBJECTS;
    for (object, _) in objects
        .into_iter()
        .filter(|(object, _)| object.as_path() != active_object)
        .take(remove_count)
    {
        if fs::remove_file(&object).is_err() {
            continue;
        }
        let Some(file_name) = object.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let _ = fs::remove_file(cache_dir.join(format!("{file_name}.integrity")));
    }
}

/// Removes compiler-owned runtime-cache temporaries after their owner process has exited.
fn cleanup_stale_runtime_cache_temporaries(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Some(owner_pid) = runtime_cache_temporary_owner_pid(&path) else {
            continue;
        };
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() || runtime_cache_process_is_alive(owner_pid) {
            continue;
        }
        let _ = fs::remove_file(path);
    }
}

/// Parses the owner PID from one of this module's assembler or integrity temp names.
fn runtime_cache_temporary_owner_pid(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    if let Some(without_tmp) = name.strip_suffix(".tmp") {
        let (object_name, owner) = without_tmp.rsplit_once(".integrity.")?;
        let canonical_name = if object_name.ends_with(".o") {
            object_name.to_string()
        } else {
            format!("{object_name}.o")
        };
        if !runtime_cache_object_name_is_canonical(Path::new(&canonical_name)) {
            return None;
        }
        return owner.parse::<u32>().ok();
    }

    let (temporary_stem, extension) = name.rsplit_once('.')?;
    if extension != "s" && extension != "o" {
        return None;
    }
    let (object_stem, owner_identity) = temporary_stem.rsplit_once('.')?;
    let (owner, nonce) = owner_identity.split_once('_')?;
    if nonce.is_empty() || !nonce.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let canonical_name = format!("{object_stem}.o");
    if !runtime_cache_object_name_is_canonical(Path::new(&canonical_name)) {
        return None;
    }
    owner.parse::<u32>().ok()
}

/// Returns whether a positive Unix PID still identifies a running process.
#[cfg(unix)]
fn runtime_cache_process_is_alive(pid: u32) -> bool {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    if pid <= 0 {
        return false;
    }
    if unsafe { libc::kill(pid, 0) } == 0 {
        return true;
    }
    io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

/// Conservatively preserves temporaries where process liveness cannot be queried safely.
#[cfg(not(unix))]
fn runtime_cache_process_is_alive(_pid: u32) -> bool {
    true
}

/// Removes abandoned hardlink snapshots whose owner no longer holds their marker lock.
fn cleanup_stale_runtime_cache_leases(cache_dir: &Path) {
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };
    for lock_path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let Some(holder_pid) = runtime_cache_lease_holder_pid(&lock_path) else {
            continue;
        };
        // Some flock implementations coalesce locks by process rather than open
        // file description. Never let this process's cleanup probe upgrade its
        // own shared lease and remove the snapshot it is about to return.
        if holder_pid == std::process::id() {
            continue;
        }
        let Ok(lock_file) = OpenOptions::new().read(true).write(true).open(&lock_path) else {
            continue;
        };
        match FileExt::try_lock_exclusive(&lock_file) {
            Ok(()) => {}
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => continue,
            Err(_) => continue,
        }
        let Some(lock_name) = lock_path.file_name().and_then(|name| name.to_str()) else {
            let _ = FileExt::unlock(&lock_file);
            continue;
        };
        let Some(lease_name) = lock_name.strip_suffix(".lock") else {
            let _ = FileExt::unlock(&lock_file);
            continue;
        };
        let lease_object = cache_dir.join(lease_name);
        let _ = fs::remove_file(&lease_object);
        let _ = fs::remove_file(cache_dir.join(format!("{lease_name}.integrity")));
        let _ = FileExt::unlock(&lock_file);
        let _ = fs::remove_file(&lock_path);
    }
}

/// Parses the holder PID from one of this module's canonical lease lock markers.
fn runtime_cache_lease_holder_pid(path: &Path) -> Option<u32> {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return None;
    };
    let Some(lease_name) = name.strip_suffix(".lock") else {
        return None;
    };
    let Some((object_name, lease_identity)) = lease_name.rsplit_once(".lease-") else {
        return None;
    };
    let mut fields = lease_identity.split('-');
    let holder_pid = fields.next()?.parse::<u32>().ok()?;
    let timestamp = fields.next()?;
    let sequence = fields.next()?;
    if fields.next().is_some()
        || timestamp.is_empty()
        || !timestamp.bytes().all(|byte| byte.is_ascii_digit())
        || sequence.is_empty()
        || !sequence.bytes().all(|byte| byte.is_ascii_digit())
        || !runtime_cache_object_name_is_canonical(Path::new(object_name))
    {
        return None;
    }
    Some(holder_pid)
}

/// Returns whether a path names a published runtime-cache object rather than a temp file.
fn runtime_cache_object_name_is_canonical(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let Some((identity, heap_suffix)) = name.rsplit_once("-heap") else {
        return false;
    };
    let Some(heap_size) = heap_suffix.strip_suffix(".o") else {
        return false;
    };
    let Some((prefix, runtime_hash)) = identity.rsplit_once("-rt") else {
        return false;
    };
    prefix.starts_with("runtime-v")
        && runtime_hash.len() == 16
        && runtime_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !heap_size.is_empty()
        && heap_size.bytes().all(|byte| byte.is_ascii_digit())
}

/// Combines runtime emission inputs with the compile-time runtime-emitter identity.
///
/// Keeping this separate makes cache-key invariants testable without assembling
/// an object. Production passes Cargo's source-derived build identity, so any
/// runtime emitter change invalidates the entry even if package versions match.
fn runtime_cache_key_with_build_identity(
    heap_size: usize,
    target: Target,
    features: RuntimeFeatures,
    pic: bool,
    build_identity: &[u8],
) -> u64 {
    let feature_bits = (features.regex as u8)
        | ((features.mb_strlen as u8) << 1)
        | ((features.phar_archive as u8) << 2)
        | ((features.descriptor_invoker as u8) << 3)
        | ((features.eval_bridge as u8) << 4)
        | ((features.eval_scope as u8) << 5)
        | ((features.web as u8) << 6)
        | ((pic as u8) << 7);
    let mut identity = format!("{}:{heap_size}:{feature_bits}:", target.as_str()).into_bytes();
    identity.extend_from_slice(build_identity);
    runtime_bytes_hash(&identity)
}

/// Ensures the shared cache is not writable or readable by another local user.
#[cfg(unix)]
/// Restricts a cache directory to the invoking user before publishing objects.
fn harden_runtime_cache_dir(cache_dir: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let metadata = fs::metadata(cache_dir).map_err(|err| {
        format!("failed to stat runtime cache '{}': {}", cache_dir.display(), err)
    })?;
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(format!("runtime cache '{}' is not owned by this user", cache_dir.display()));
    }
    if metadata.mode() & 0o077 != 0 {
        fs::set_permissions(cache_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            format!("failed to restrict runtime cache '{}': {}", cache_dir.display(), err)
        })?;
    }
    Ok(())
}

/// Platforms without Unix ownership modes rely on per-user cache roots.
#[cfg(not(unix))]
/// Keeps the cache setup portable where Unix ownership modes are unavailable.
fn harden_runtime_cache_dir(_cache_dir: &std::path::Path) -> Result<(), String> {
    Ok(())
}

/// Returns whether a cache object matches its atomically stored integrity sidecar.
fn runtime_object_is_intact(cache_path: &std::path::Path, integrity_path: &std::path::Path) -> bool {
    let Ok(bytes) = fs::read(cache_path) else {
        return false;
    };
    let Ok(expected) = fs::read_to_string(integrity_path) else {
        return false;
    };
    expected.trim() == format!("{:016x}", runtime_bytes_hash(&bytes))
}

/// Writes the checksum sidecar only after the object has been published.
fn write_runtime_object_integrity(
    cache_path: &std::path::Path,
    integrity_path: &std::path::Path,
) -> Result<(), String> {
    let bytes = fs::read(cache_path).map_err(|err| {
        format!("failed to read runtime cache '{}' for integrity: {}", cache_path.display(), err)
    })?;
    let temporary = integrity_path.with_extension(format!(
        "integrity.{}.tmp",
        std::process::id()
    ));
    fs::write(&temporary, format!("{:016x}\n", runtime_bytes_hash(&bytes))).map_err(|err| {
        format!("failed to write runtime cache integrity '{}': {}", temporary.display(), err)
    })?;
    fs::rename(&temporary, integrity_path).map_err(|err| {
        let _ = fs::remove_file(&temporary);
        format!("failed to publish runtime cache integrity '{}': {}", integrity_path.display(), err)
    })
}

/// Computes a 64-bit FNV-1a hash of arbitrary cache bytes.
fn runtime_bytes_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the cache key changes with runtime-emitter build identity even
    /// when compiler version, target, heap, features, and PIC mode are identical.
    #[test]
    fn runtime_cache_key_covers_runtime_emitter_build_identity() {
        let target = Target::detect_host();
        let features = RuntimeFeatures::none();
        let first = runtime_cache_key_with_build_identity(
            8 * 1024 * 1024,
            target,
            features,
            false,
            b"emitter-build-a",
        );
        let second = runtime_cache_key_with_build_identity(
            8 * 1024 * 1024,
            target,
            features,
            false,
            b"emitter-build-b",
        );
        assert_ne!(first, second, "different runtime emitters must never share a cache entry");
    }

    /// Verifies Cargo reruns the identity builder when a runtime source file is
    /// added or removed, not only when an already-enumerated file is edited.
    #[test]
    fn runtime_build_identity_tracks_source_tree_membership() {
        let build_script = include_str!("../build.rs");
        assert!(
            build_script.contains("cargo:rerun-if-changed=src"),
            "runtime build identity must be recomputed when src tree membership changes"
        );
    }

    /// Verifies the runtime cache directory is private to its owner so another
    /// local user cannot replace both the object and its integrity metadata.
    #[cfg(unix)]
    #[test]
    fn runtime_cache_directory_is_owner_only() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        const TEST_NAME: &str = "runtime_cache_directory_is_owner_only";
        if std::env::var("ELEPHC_RUNTIME_CACHE_MODE_PROBE").as_deref() != Ok(TEST_NAME) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([TEST_NAME, "--nocapture", "--test-threads=1"])
                .env("ELEPHC_RUNTIME_CACHE_MODE_PROBE", TEST_NAME)
                .output()
                .expect("spawn isolated runtime-cache mode probe");
            assert!(
                output.status.success(),
                "isolated runtime-cache mode probe failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephc-runtime-cache-mode-{}-{unique}",
            std::process::id()
        ));
        let cache = root.join("elephc");
        fs::create_dir_all(&cache).expect("create permissive cache fixture");
        fs::set_permissions(&cache, fs::Permissions::from_mode(0o777))
            .expect("make fixture world-writable");
        std::env::set_var("XDG_CACHE_HOME", &root);

        prepare_runtime_object(
            8 * 1024 * 1024,
            Target::detect_host(),
            RuntimeFeatures::none(),
            false,
        )
        .expect("prepare runtime in hardened cache");
        let metadata = fs::metadata(&cache).expect("stat runtime cache directory");
        assert_eq!(
            metadata.mode() & 0o077,
            0,
            "runtime cache must reject group/other access"
        );
        assert_eq!(
            metadata.uid(),
            fs::metadata(&root)
                .expect("stat trusted cache root")
                .uid(),
            "runtime cache must be owned by the compiler user"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Verifies a cache hit whose object bytes were replaced is detected and
    /// rebuilt to the deterministic object originally produced by the assembler.
    #[test]
    fn tampered_runtime_object_cache_entry_is_rebuilt() {
        const TEST_NAME: &str = "tampered_runtime_object_cache_entry_is_rebuilt";
        if std::env::var("ELEPHC_RUNTIME_CACHE_PROBE").as_deref() != Ok(TEST_NAME) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([TEST_NAME, "--nocapture", "--test-threads=1"])
                .env("ELEPHC_RUNTIME_CACHE_PROBE", TEST_NAME)
                .output()
                .expect("spawn isolated runtime-cache probe");
            assert!(
                output.status.success(),
                "isolated runtime-cache probe failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephc-runtime-cache-security-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create isolated cache root");
        std::env::set_var("XDG_CACHE_HOME", &root);

        let first = prepare_runtime_object(
            8 * 1024 * 1024,
            Target::detect_host(),
            RuntimeFeatures::none(),
            false,
        )
        .expect("build initial runtime object");
        let trusted = fs::read(&first.path).expect("read initial runtime object");
        fs::write(&first.path, b"attacker-controlled-object").expect("replace cache entry");

        let second = prepare_runtime_object(
            8 * 1024 * 1024,
            Target::detect_host(),
            RuntimeFeatures::none(),
            false,
        )
        .expect("repair tampered runtime object");
        let repaired = fs::read(&second.path).expect("read repaired runtime object");

        assert_eq!(repaired, trusted, "tampered cache bytes must never be reused");
        let _ = fs::remove_dir_all(&root);
    }

    /// Guards the warm-cache architecture: lookup and integrity metadata must
    /// be resolved before the expensive full runtime generator is invoked.
    #[test]
    fn warm_runtime_cache_lookup_precedes_runtime_assembly_generation() {
        let source = include_str!("runtime_cache.rs");
        let lookup = source
            .find("cache_path.exists()")
            .expect("runtime cache lookup remains explicit");
        let generation = source
            .find("generate_runtime_with_features_pic")
            .expect("runtime assembly generator remains explicit");

        assert!(
            lookup < generation,
            "a warm cache hit must not regenerate the complete runtime assembly"
        );
    }

    /// Verifies cache housekeeping bounds superseded runtime objects while
    /// preserving the active entry, its integrity sidecar, and unrelated files.
    #[test]
    fn runtime_cache_pruning_bounds_superseded_build_identities() {
        const TEST_NAME: &str = "runtime_cache_pruning_bounds_superseded_build_identities";
        if std::env::var("ELEPHC_RUNTIME_CACHE_PRUNE_PROBE").as_deref() != Ok(TEST_NAME) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([TEST_NAME, "--nocapture", "--test-threads=1"])
                .env("ELEPHC_RUNTIME_CACHE_PRUNE_PROBE", TEST_NAME)
                .output()
                .expect("spawn isolated runtime-cache pruning probe");
            assert!(
                output.status.success(),
                "isolated runtime-cache pruning probe failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephc-runtime-cache-prune-{}-{unique}",
            std::process::id()
        ));
        let cache = root.join("elephc");
        fs::create_dir_all(&cache).expect("create cache-pruning fixture");
        for identity in 0..12 {
            let object = cache.join(format!(
                "runtime-v0-test-rt{identity:016x}-heap1.o"
            ));
            fs::write(&object, format!("object-{identity}"))
                .expect("write cache object fixture");
            fs::write(
                cache.join(format!("{}.integrity", object.file_name().unwrap().to_string_lossy())),
                format!("integrity-{identity}"),
            )
            .expect("write cache integrity fixture");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let unrelated = cache.join("README.keep");
        fs::write(&unrelated, "unrelated").expect("write unrelated cache fixture");
        std::env::set_var("XDG_CACHE_HOME", &root);

        let prepared = prepare_runtime_object(
            8 * 1024 * 1024,
            Target::detect_host(),
            RuntimeFeatures::none(),
            false,
        )
        .expect("prepare active runtime while pruning old identities");

        let remaining: Vec<_> = fs::read_dir(&cache)
            .expect("read pruned cache fixture")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "o"))
            .collect();
        assert!(remaining.len() <= 8, "cache retained {remaining:?}");
        assert!(prepared.path.exists(), "the active runtime object was pruned");
        assert!(
            cache.join(format!(
                "{}.integrity",
                prepared.path.file_name().unwrap().to_string_lossy()
            ))
            .exists(),
            "the active runtime integrity sidecar was pruned"
        );
        assert!(unrelated.exists(), "cache pruning removed an unrelated file");
        let _ = fs::remove_dir_all(&root);
    }

    /// Verifies cache pruning removes assembler and integrity temporaries left
    /// behind by a dead compiler process without touching unrelated files.
    #[test]
    fn runtime_cache_pruning_removes_crash_abandoned_temporaries() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephc-runtime-cache-crash-litter-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create crash-litter fixture");

        let canonical = root.join("runtime-v0-test-rt0000000000000001-heap1.o");
        fs::write(&canonical, b"active").expect("write active cache object");
        let abandoned_asm = root.join(
            "runtime-v0-test-rt0000000000000002-heap1.4294967295_1.s",
        );
        let abandoned_object = root.join(
            "runtime-v0-test-rt0000000000000002-heap1.4294967295_1.o",
        );
        let abandoned_integrity = root.join(
            "runtime-v0-test-rt0000000000000002-heap1.integrity.4294967295.tmp",
        );
        fs::write(&abandoned_asm, b"assembly").expect("write abandoned assembly");
        fs::write(&abandoned_object, b"object").expect("write abandoned object");
        fs::write(&abandoned_integrity, b"checksum").expect("write abandoned integrity temp");
        let unrelated = root.join("runtime-not-owned.tmp");
        fs::write(&unrelated, b"keep").expect("write unrelated fixture");

        prune_runtime_cache_objects(&root, &canonical);

        assert!(!abandoned_asm.exists(), "abandoned assembly was retained");
        assert!(!abandoned_object.exists(), "abandoned object was retained");
        assert!(
            !abandoned_integrity.exists(),
            "abandoned integrity temporary was retained"
        );
        assert!(canonical.exists(), "active cache object was removed");
        assert!(unrelated.exists(), "unrelated cache file was removed");
        let _ = fs::remove_dir_all(&root);
    }

    /// Verifies pruning in one compiler process cannot delete a prepared object
    /// that another live compiler process has not consumed at link time yet.
    #[test]
    fn runtime_cache_pruning_preserves_cross_process_prepared_object_lease() {
        const TEST_NAME: &str =
            "runtime_cache_pruning_preserves_cross_process_prepared_object_lease";
        if std::env::var("ELEPHC_RUNTIME_CACHE_LEASE_PROBE").as_deref() != Ok(TEST_NAME) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([TEST_NAME, "--nocapture", "--test-threads=1"])
                .env("ELEPHC_RUNTIME_CACHE_LEASE_PROBE", TEST_NAME)
                .output()
                .expect("spawn isolated runtime-cache lease probe");
            assert!(
                output.status.success(),
                "isolated runtime-cache lease probe failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        if std::env::var("ELEPHC_RUNTIME_CACHE_LEASE_ROLE").as_deref() == Ok("holder") {
            let ready = PathBuf::from(
                std::env::var_os("ELEPHC_RUNTIME_CACHE_LEASE_READY")
                    .expect("lease holder ready path"),
            );
            let release = PathBuf::from(
                std::env::var_os("ELEPHC_RUNTIME_CACHE_LEASE_RELEASE")
                    .expect("lease holder release path"),
            );
            let prepared = prepare_runtime_object(
                7 * 1024 * 1024,
                Target::detect_host(),
                RuntimeFeatures::none(),
                false,
            )
            .expect("prepare runtime object held across another compiler's pruning");
            fs::write(&ready, prepared.path.to_string_lossy().as_bytes())
                .expect("publish held runtime object path");
            for _ in 0..400 {
                if release.exists() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            assert!(release.exists(), "lease holder timed out waiting for release");
            assert!(
                prepared.path.exists(),
                "another compiler pruned a runtime object before its holder linked it"
            );
            return;
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "elephc-runtime-cache-lease-{}-{unique}",
            std::process::id()
        ));
        let cache = root.join("elephc");
        let ready = root.join("holder.ready");
        let release = root.join("holder.release");
        fs::create_dir_all(&cache).expect("create cache-lease fixture");
        std::env::set_var("XDG_CACHE_HOME", &root);

        let mut holder = std::process::Command::new(std::env::current_exe().unwrap())
            .args([TEST_NAME, "--nocapture", "--test-threads=1"])
            .env("ELEPHC_RUNTIME_CACHE_LEASE_PROBE", TEST_NAME)
            .env("ELEPHC_RUNTIME_CACHE_LEASE_ROLE", "holder")
            .env("ELEPHC_RUNTIME_CACHE_LEASE_READY", &ready)
            .env("ELEPHC_RUNTIME_CACHE_LEASE_RELEASE", &release)
            .env("XDG_CACHE_HOME", &root)
            .spawn()
            .expect("spawn runtime-cache lease holder");
        for _ in 0..400 {
            if ready.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        assert!(ready.exists(), "runtime-cache lease holder did not become ready");
        let held_path = PathBuf::from(
            fs::read_to_string(&ready).expect("read held runtime object path"),
        );

        for identity in 0..12 {
            let object = cache.join(format!(
                "runtime-v0-lease-fixture-rt{identity:016x}-heap1.o"
            ));
            fs::write(&object, format!("object-{identity}"))
                .expect("write competing cache object fixture");
            fs::write(
                cache.join(format!("{}.integrity", object.file_name().unwrap().to_string_lossy())),
                format!("integrity-{identity}"),
            )
            .expect("write competing cache integrity fixture");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let competing = prepare_runtime_object(
            9 * 1024 * 1024,
            Target::detect_host(),
            RuntimeFeatures::none(),
            false,
        )
        .expect("prepare competing runtime while pruning old identities");
        let held_survived = held_path.exists();
        fs::write(&release, b"release").expect("release runtime-cache lease holder");
        let holder_status = holder.wait().expect("wait for runtime-cache lease holder");

        assert!(holder_status.success(), "runtime-cache lease holder failed");
        assert!(
            held_survived,
            "pruning removed another live compiler's prepared runtime object"
        );
        assert!(competing.path.exists(), "competing active runtime object was pruned");
        let _ = fs::remove_dir_all(&root);
    }
}
