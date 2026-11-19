//! Purpose:
//! Caches the assembled compiler catalog so an opaque-`eval` build assembles it once per shape.
//!
//! Called from:
//! - `crate::pipeline::backend` right before the catalog unit is assembled.
//!
//! Key details:
//! - The catalog unit is emitted without any dependency on the user's program: its literals are
//!   content-addressed and every function carries its own name, so the same target, PHP profile,
//!   and catalog contents always produce the same unit.
//! - The one thing that still differs between programs is the numbering of the assembler-local
//!   labels. They never leave the object, so the cache key is computed over a normalized text
//!   where those numeric suffixes are dropped, and the stored object keeps whichever numbering
//!   the compilation that filled the cache produced.
//! - The stored object is named after the key and its length, so a truncated or foreign file is
//!   rejected without a checksum sidecar, and a store is an atomic rename that tolerates
//!   concurrent compiler runs.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::codegen::platform::Target;

/// Whether the catalog object came from the cache or was assembled now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CatalogCacheStatus {
    /// The object was found and reused.
    Hit,
    /// The object was assembled during this build.
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

/// The catalog object to link, and how it was obtained.
pub(crate) struct PreparedCatalogObject {
    /// Path of the object file, stable for the rest of the build.
    pub(crate) path: PathBuf,
    /// Cache status of this preparation.
    pub(crate) status: CatalogCacheStatus,
}

/// Returns the catalog object for `catalog_asm`, assembling and storing it on a miss.
///
/// `local_object` is where the linker reads the object from. It is a hardlink of the cache entry
/// when the filesystem allows it, so the link never depends on a cache file another compiler may
/// prune or replace, and the caller removes it with the rest of the build's objects.
pub(crate) fn prepare_catalog_object(
    target: Target,
    catalog_asm: &str,
    local_object: &Path,
) -> Result<PreparedCatalogObject, String> {
    prepare_in(
        &crate::runtime_cache::runtime_cache_dir().join("catalog"),
        target,
        catalog_asm,
        local_object,
    )
}

/// Implements [`prepare_catalog_object`] against an explicit cache directory.
fn prepare_in(
    directory: &Path,
    target: Target,
    catalog_asm: &str,
    local_object: &Path,
) -> Result<PreparedCatalogObject, String> {
    fs::create_dir_all(directory).map_err(|err| {
        format!(
            "failed to create catalog cache '{}': {}",
            directory.display(),
            err
        )
    })?;
    let key = catalog_cache_key(target, catalog_asm);
    let object_path = directory.join(format!("catalog-{key:016x}.o"));
    let witness_path = directory.join(format!("catalog-{key:016x}.len"));
    if cached_object_is_intact(&object_path, &witness_path) {
        link_or_copy(&object_path, local_object)?;
        return Ok(PreparedCatalogObject {
            path: local_object.to_path_buf(),
            status: CatalogCacheStatus::Hit,
        });
    }

    let unique = format!(
        "{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let temp_asm = directory.join(format!("catalog-{key}.{unique}.s"));
    let temp_obj = directory.join(format!("catalog-{key}.{unique}.o"));
    fs::write(&temp_asm, catalog_asm).map_err(|err| {
        format!(
            "failed to write temporary catalog assembly '{}': {}",
            temp_asm.display(),
            err
        )
    })?;
    let mut assembler = crate::linker::assembler_command(target);
    assembler.arg("-o").arg(&temp_obj).arg(&temp_asm);
    let status = assembler.status().map_err(|err| {
        format!(
            "failed to run the catalog assembler for '{}': {}",
            temp_obj.display(),
            err
        )
    });
    let _ = fs::remove_file(&temp_asm);
    let status = status?;
    if !status.success() {
        let _ = fs::remove_file(&temp_obj);
        return Err(format!(
            "catalog assembler failed while building '{}'",
            object_path.display()
        ));
    }
    let witness = fs::metadata(&temp_obj)
        .map(|metadata| metadata.len())
        .map_err(|err| format!("failed to measure '{}': {}", temp_obj.display(), err))?;
    match fs::rename(&temp_obj, &object_path).and_then(|()| {
        fs::write(&witness_path, witness.to_string())
    }) {
        Ok(()) => {
            link_or_copy(&object_path, local_object)?;
            Ok(PreparedCatalogObject {
                path: local_object.to_path_buf(),
                status: CatalogCacheStatus::Miss,
            })
        }
        // Another compiler published the same entry first; both objects are equivalent.
        Err(_) if object_path.exists() => {
            let _ = fs::remove_file(&temp_obj);
            link_or_copy(&object_path, local_object)?;
            Ok(PreparedCatalogObject {
                path: local_object.to_path_buf(),
                status: CatalogCacheStatus::Hit,
            })
        }
        Err(err) => {
            let _ = fs::remove_file(&temp_obj);
            Err(format!(
                "failed to store the catalog cache '{}': {}",
                object_path.display(),
                err
            ))
        }
    }
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

/// Returns whether the cached object still matches the length recorded when it was stored.
///
/// A truncated or half-copied object would fail the link with a confusing message, so a size that
/// no longer matches its witness is treated as a miss and rebuilt.
fn cached_object_is_intact(object: &Path, witness: &Path) -> bool {
    let Ok(expected) = fs::read_to_string(witness) else {
        return false;
    };
    let Ok(expected) = expected.trim().parse::<u64>() else {
        return false;
    };
    fs::metadata(object)
        .map(|metadata| metadata.len() == expected)
        .unwrap_or(false)
}

/// Hashes the inputs that determine the catalog object: build identity, target, PHP profile, and
/// the unit text with the assembler-local label numbering normalized away.
fn catalog_cache_key(target: Target, catalog_asm: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in env!("ELEPHC_RUNTIME_BUILD_ID").as_bytes() {
        hash = fnv1a64_step(hash, *byte);
    }
    for byte in format!("{target:?}|{}", crate::codegen::compile_php_version().version_id())
        .as_bytes()
    {
        hash = fnv1a64_step(hash, *byte);
    }
    let mut token = String::new();
    for character in catalog_asm.chars() {
        if character.is_ascii_alphanumeric() || character == '_' || character == '.' || character == '$'
        {
            token.push(character);
            continue;
        }
        hash_local_label(&mut hash, &mut token);
        hash = fnv1a64_step(hash, character as u32 as u8);
    }
    hash_local_label(&mut hash, &mut token);
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

    /// A second preparation of the same unit reuses the object the first one assembled.
    #[test]
    fn stores_then_reuses_the_catalog_object() {
        let dir = scratch("reuse");
        let asm = ".text\n.globl _catalog_fn\n_catalog_fn:\n    ret\n";
        let first = prepare_in(&dir, target(), asm, &dir.join("first.o")).expect("first");
        assert_eq!(first.status, CatalogCacheStatus::Miss);
        let first_bytes = fs::read(&first.path).expect("first object");
        let second = prepare_in(&dir, target(), asm, &dir.join("second.o")).expect("second");
        assert_eq!(second.status, CatalogCacheStatus::Hit);
        assert_eq!(fs::read(&second.path).expect("second object"), first_bytes);
    }

    /// A different unit does not reuse the stored object.
    #[test]
    fn different_units_do_not_share_an_object() {
        let dir = scratch("distinct");
        let first = prepare_in(&dir, target(), ".text\n_catalog_fn:\n    ret\n", &dir.join("a.o"))
            .expect("first");
        assert_eq!(first.status, CatalogCacheStatus::Miss);
        let second = prepare_in(
            &dir,
            target(),
            ".text\n_catalog_fn:\n    nop\n    ret\n",
            &dir.join("b.o"),
        )
        .expect("second");
        assert_eq!(second.status, CatalogCacheStatus::Miss);
    }

    /// Two units that differ only in assembler-local label numbering share one key.
    #[test]
    fn key_ignores_local_label_numbering() {
        let first = ".text\nL_eir_fn_loop_7:\n    b L_eir_fn_loop_7\n";
        let second = ".text\nL_eir_fn_loop_9:\n    b L_eir_fn_loop_9\n";
        assert_eq!(
            catalog_cache_key(target(), first),
            catalog_cache_key(target(), second)
        );
    }

    /// A unit with different content gets a different key.
    #[test]
    fn key_follows_the_emitted_content() {
        let first = ".text\n_fn_a:\n    ret\n";
        let second = ".text\n_fn_a:\n    nop\n    ret\n";
        assert_ne!(
            catalog_cache_key(target(), first),
            catalog_cache_key(target(), second)
        );
    }

    /// A symbol whose name is not a numeric suffix keeps its full text in the key.
    #[test]
    fn key_keeps_non_numeric_suffixes() {
        let first = ".text\n_fn_alpha:\n    ret\n";
        let second = ".text\n_fn_beta:\n    ret\n";
        assert_ne!(
            catalog_cache_key(target(), first),
            catalog_cache_key(target(), second)
        );
    }
}
