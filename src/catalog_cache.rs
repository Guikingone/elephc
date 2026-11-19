//! Purpose:
//! Caches and assembles the compiler catalog units so an opaque-`eval` build pays for them once.
//!
//! Called from:
//! - `crate::pipeline::backend` right before the catalog units are assembled.
//!
//! Key details:
//! - The catalog units carry no dependency on the user's program: their literals are
//!   content-addressed and every function keeps its own name, so the same target, PHP profile, and
//!   catalog contents always produce the same units.
//! - The one thing that still differs between programs is the numbering of the assembler-local
//!   labels. They never leave their object, so the cache key is computed over a normalized text
//!   where those numeric suffixes are dropped, and the stored objects keep whichever numbering the
//!   compilation that filled the cache produced.
//! - A miss assembles every unit concurrently. Each stored object is named after the key and its
//!   index, a length witness rejects truncated entries, and a store is an atomic rename that
//!   tolerates concurrent compiler runs.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::codegen::platform::Target;

/// Whether the catalog objects came from the cache or were assembled now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatalogCacheStatus {
    /// The objects were found and reused.
    Hit,
    /// The objects were assembled during this build.
    Miss,
}

impl CatalogCacheStatus {
    /// Returns the static name used by progress notes.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            CatalogCacheStatus::Hit => "hit",
            CatalogCacheStatus::Miss => "miss",
        }
    }
}

/// Returns the objects to link, publishing each at its local path, and how they were obtained.
///
/// `local_paths` are the build's own object paths, in unit order. They are hardlinks of the cache
/// entries when the filesystem allows it, so the link never depends on a cache file another
/// compiler may prune or replace, and the caller removes them with the rest of its objects.
pub(crate) fn prepare_catalog_objects(
    target: Target,
    units: &[String],
    local_paths: &[PathBuf],
) -> Result<(Vec<PathBuf>, CatalogCacheStatus), String> {
    prepare_in(
        &crate::runtime_cache::runtime_cache_dir().join("catalog"),
        target,
        units,
        local_paths,
    )
}

/// Implements [`prepare_catalog_objects`] against an explicit cache directory.
fn prepare_in(
    directory: &Path,
    target: Target,
    units: &[String],
    local_paths: &[PathBuf],
) -> Result<(Vec<PathBuf>, CatalogCacheStatus), String> {
    fs::create_dir_all(directory).map_err(|err| {
        format!(
            "failed to create catalog cache '{}': {}",
            directory.display(),
            err
        )
    })?;
    let key = catalog_cache_key(target, units);
    let witness_path = directory.join(format!("catalog-{key:016x}.len"));
    let cached: Vec<PathBuf> = (0..units.len())
        .map(|index| directory.join(format!("catalog-{key:016x}-{index}.o")))
        .collect();
    if cached_objects_are_intact(&cached, &witness_path) {
        for (source, destination) in cached.iter().zip(local_paths) {
            link_or_copy(source, destination)?;
        }
        return Ok((local_paths.to_vec(), CatalogCacheStatus::Hit));
    }

    let unique = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    // Every unit assembles concurrently: a catalog build pays for all of them once, and they are
    // independent objects.
    let assembled = std::thread::scope(|scope| {
        let handles: Vec<_> = units
            .iter()
            .enumerate()
            .map(|(index, unit)| {
                let temp_obj = directory.join(format!("catalog-{key:016x}-{index}.{unique}.o"));
                let temp_asm = directory.join(format!("catalog-{key:016x}-{index}.{unique}.s"));
                scope.spawn(move || {
                    fs::write(&temp_asm, unit)
                        .map_err(|err| format!("failed to write '{}': {}", temp_asm.display(), err))?;
                    let mut assembler = crate::linker::assembler_command(target);
                    assembler.arg("-o").arg(&temp_obj).arg(&temp_asm);
                    let status = assembler.status().map_err(|err| {
                        format!(
                            "failed to run the catalog assembler for '{}': {}",
                            temp_obj.display(),
                            err
                        )
                    })?;
                    let _ = fs::remove_file(&temp_asm);
                    if !status.success() {
                        let _ = fs::remove_file(&temp_obj);
                        return Err(format!(
                            "catalog assembler failed while building '{}'",
                            temp_obj.display()
                        ));
                    }
                    Ok(temp_obj)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap_or_else(|_| Err("catalog assembler panicked".to_string())))
            .collect::<Result<Vec<PathBuf>, String>>()
    })?;

    let mut lengths = Vec::with_capacity(assembled.len());
    let mut published = false;
    for (temp_obj, object_path) in assembled.iter().zip(&cached) {
        let length = fs::metadata(temp_obj)
            .map(|metadata| metadata.len())
            .map_err(|err| format!("failed to measure '{}': {}", temp_obj.display(), err))?;
        match fs::rename(temp_obj, object_path) {
            Ok(()) => {
                lengths.push(length);
                published = true;
            }
            // Another compiler published the same entry first; both are equivalent.
            Err(_) if object_path.exists() => {}
            Err(err) => {
                return Err(format!(
                    "failed to store the catalog cache '{}': {}",
                    object_path.display(),
                    err
                ))
            }
        }
    }
    let status = if published {
        fs::write(&witness_path, lengths_to_witness(&lengths)).map_err(|err| {
            format!(
                "failed to write the catalog witness '{}': {}",
                witness_path.display(),
                err
            )
        })?;
        CatalogCacheStatus::Miss
    } else {
        CatalogCacheStatus::Hit
    };
    for (source, destination) in cached.iter().zip(local_paths) {
        link_or_copy(source, destination)?;
    }
    Ok((local_paths.to_vec(), status))
}

/// Returns the text stored as the length witness for `lengths`.
fn lengths_to_witness(lengths: &[u64]) -> String {
    lengths
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Returns whether every cached object still matches the length recorded when it was stored.
///
/// A truncated or half-copied object would fail the link with a confusing message, so a size that
/// no longer matches its witness is treated as a miss and rebuilt.
fn cached_objects_are_intact(objects: &[PathBuf], witness: &Path) -> bool {
    let Ok(recorded) = fs::read_to_string(witness) else {
        return false;
    };
    let expected: Vec<u64> = recorded
        .trim()
        .split(',')
        .filter_map(|length| length.parse().ok())
        .collect();
    if expected.len() != objects.len() {
        return false;
    }
    objects.iter().zip(expected).all(|(object, expected)| {
        fs::metadata(object)
            .map(|metadata| metadata.len() == expected)
            .unwrap_or(false)
    })
}

/// Publishes `source` at `destination`, preferring a hardlink over a byte copy.
fn link_or_copy(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        let _ = fs::remove_file(destination);
    }
    match fs::hard_link(source, destination) {
        Ok(()) => Ok(()),
        Err(_) => fs::copy(source, destination)
            .map(|_| ())
            .map_err(|err| {
                format!(
                    "failed to publish the catalog object at '{}': {}",
                    destination.display(),
                    err
                )
            }),
    }
}

/// Hashes the inputs that determine the catalog objects: build identity, target, PHP profile, and
/// the unit texts with the assembler-local label numbering normalized away.
fn catalog_cache_key(target: Target, units: &[String]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in env!("ELEPHC_RUNTIME_BUILD_ID").as_bytes() {
        hash = fnv1a64_step(hash, *byte);
    }
    for byte in format!("{target:?}|{}", crate::codegen::compile_php_version().version_id())
        .as_bytes()
    {
        hash = fnv1a64_step(hash, *byte);
    }
    // The unit boundaries are part of the key: the same text split differently produces different
    // objects, and a hit must describe the split the pipeline is about to assemble.
    for unit in units {
        for byte in (unit.len() as u64).to_le_bytes() {
            hash = fnv1a64_step(hash, byte);
        }
        let mut token = String::new();
        for character in unit.chars() {
            if character.is_ascii_alphanumeric()
                || character == '_'
                || character == '.'
                || character == '$'
            {
                token.push(character);
                continue;
            }
            hash_local_label(&mut hash, &mut token);
            hash = fnv1a64_step(hash, character as u32 as u8);
        }
        hash_local_label(&mut hash, &mut token);
    }
    hash
}

/// Mixes one symbol token into `hash`, dropping the suffix the generator assigns per compilation
/// to assembler-local labels, and clears the token.
///
/// Only `L`-prefixed labels are normalized. A global symbol whose name ends in a number can encode
/// something the program decides (`_class_ctor_7_4` is class 7's constructor), so it must stay in
/// the key: two programs that number their classes differently then miss instead of reusing an
/// object that names the wrong helper.
fn hash_local_label(hash: &mut u64, token: &mut String) {
    if token.is_empty() {
        return;
    }
    let mut normalized = token.as_str();
    let is_local = normalized.starts_with('L') || normalized.starts_with(".L");
    if is_local {
        if let Some((stem, suffix)) = normalized.rsplit_once('_') {
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                normalized = stem;
            }
        }
    }
    for byte in normalized.as_bytes() {
        *hash = fnv1a64_step(*hash, *byte);
    }
    *hash = fnv1a64_step(*hash, 0xff);
    token.clear();
}

/// Folds one byte into an FNV-1a hash.
fn fnv1a64_step(hash: u64, byte: u8) -> u64 {
    (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::platform::{Arch, Platform};

    /// Returns the target the key tests use.
    fn target() -> Target {
        Target::new(Platform::Linux, Arch::X86_64)
    }

    /// Returns a fresh temporary directory for one cache test.
    fn scratch(test: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "elephc-catalog-{test}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    /// A second preparation of the same units reuses the objects the first one assembled.
    #[test]
    fn stores_then_reuses_the_catalog_objects() {
        let dir = scratch("reuse");
        let units = vec![
            ".text\n.globl _catalog_a\n_catalog_a:\n    ret\n".to_string(),
            ".text\n.globl _catalog_b\n_catalog_b:\n    ret\n".to_string(),
        ];
        let first_paths = vec![dir.join("first-0.o"), dir.join("first-1.o")];
        let (first, status) =
            prepare_in(&dir, target(), &units, &first_paths).expect("first");
        assert_eq!(status, CatalogCacheStatus::Miss);
        let bytes: Vec<Vec<u8>> = first
            .iter()
            .map(|path| fs::read(path).expect("first object"))
            .collect();
        let second_paths = vec![dir.join("second-0.o"), dir.join("second-1.o")];
        let (second, status) =
            prepare_in(&dir, target(), &units, &second_paths).expect("second");
        assert_eq!(status, CatalogCacheStatus::Hit);
        for (path, expected) in second.iter().zip(bytes) {
            assert_eq!(fs::read(path).expect("second object"), expected);
        }
    }

    /// A different unit does not reuse the stored objects.
    #[test]
    fn different_units_do_not_share_an_object() {
        let dir = scratch("distinct");
        let first = vec![".text\n_catalog_fn:\n    ret\n".to_string()];
        let (_, status) = prepare_in(&dir, target(), &first, &[dir.join("a.o")]).expect("first");
        assert_eq!(status, CatalogCacheStatus::Miss);
        let second = vec![".text\n_catalog_fn:\n    nop\n    ret\n".to_string()];
        let (_, status) = prepare_in(&dir, target(), &second, &[dir.join("b.o")]).expect("second");
        assert_eq!(status, CatalogCacheStatus::Miss);
    }

    /// Two units that differ only in assembler-local label numbering share one key.
    #[test]
    fn key_ignores_local_label_numbering() {
        let first = vec![".text\nL_eir_fn_loop_7:\n    b L_eir_fn_loop_7\n".to_string()];
        let second = vec![".text\nL_eir_fn_loop_9:\n    b L_eir_fn_loop_9\n".to_string()];
        assert_eq!(
            catalog_cache_key(target(), &first),
            catalog_cache_key(target(), &second)
        );
    }

    /// A unit with different content gets a different key.
    #[test]
    fn key_follows_the_emitted_content() {
        let first = vec![".text\n_fn_a:\n    ret\n".to_string()];
        let second = vec![".text\n_fn_a:\n    nop\n    ret\n".to_string()];
        assert_ne!(
            catalog_cache_key(target(), &first),
            catalog_cache_key(target(), &second)
        );
    }

    /// A symbol whose name is not a numeric suffix keeps its full text in the key.
    #[test]
    fn key_keeps_non_numeric_suffixes() {
        let first = vec![".text\n_fn_alpha:\n    ret\n".to_string()];
        let second = vec![".text\n_fn_beta:\n    ret\n".to_string()];
        assert_ne!(
            catalog_cache_key(target(), &first),
            catalog_cache_key(target(), &second)
        );
    }

    /// The same text split into different units must not share a key.
    #[test]
    fn key_covers_the_unit_boundaries() {
        let one = vec!["A\nB\n".to_string()];
        let two = vec!["A\n".to_string(), "B\n".to_string()];
        assert_ne!(
            catalog_cache_key(target(), &one),
            catalog_cache_key(target(), &two)
        );
    }
}
