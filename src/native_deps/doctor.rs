//! Purpose:
//! Produces deterministic read-only native project and artifact health reports.
//!
//! Called from:
//! - `native list` and `native doctor` orchestration.
//!
//! Key details:
//! - Inspection never creates cache directories, locks, staging paths, or project files.

use std::path::Path;

use crate::codegen_support::platform::Target;

use super::cache::{ArtifactKey, CacheLayout};
use super::catalog;
use super::error::NativeError;
use super::lockfile::NativeLock;
use super::manifest::ManifestDocument;
use super::project::ProjectPaths;
use super::receipt::{ArtifactReceipt, ReceiptIdentity};
use super::toolchain::ToolchainProvider;

/// Approximate read-only cache usage and abandoned publication state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub approximate_bytes: u64,
    pub stale_staging_count: usize,
    pub stale_staging_bytes: u64,
}

/// Deterministic package health state displayed by list and doctor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageHealth {
    Installed,
    Missing,
    Corrupt,
    Stale,
    ToolchainError,
}

impl PackageHealth {
    /// Returns the frozen lowercase CLI label for this state.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Missing => "missing",
            Self::Corrupt => "corrupt",
            Self::Stale => "stale",
            Self::ToolchainError => "toolchain-error",
        }
    }
}

/// Inspects all declared packages without mutating project or cache state.
pub fn inspect(
    project: &ProjectPaths,
    target: Target,
    cache: &CacheLayout,
    toolchains: &dyn ToolchainProvider,
) -> Result<Vec<(String, String, Option<String>, String, PackageHealth)>, NativeError> {
    let manifest = ManifestDocument::load(&project.manifest)?;
    let lock = NativeLock::load(&project.lock).ok();
    let lock_current = lock.as_ref().is_some_and(|lock| lock.validate_current(&manifest).is_ok());
    let toolchain = toolchains.resolve(target);
    let mut rows = Vec::new();
    for (name, version_name) in manifest.dependencies() {
        let locked_version = lock.as_ref().and_then(|lock| lock.package(name)).map(|package| package.version.clone());
        let (abi, health) = match &toolchain {
            Err(_) => ("unresolved".to_string(), PackageHealth::ToolchainError),
            Ok(toolchain) if !lock_current => (toolchain.abi.clone(), PackageHealth::Stale),
            Ok(toolchain) => {
                let version = catalog::version(name, Some(version_name))?;
                let key = ArtifactKey { package: name, version: version.version, recipe: version.recipe_revision, source_sha256: version.source.sha256, target: target.as_str(), abi: &toolchain.abi, toolchain_fingerprint: &toolchain.fingerprint };
                let root = cache.artifact_path(&key)?;
                if !root.exists() {
                    (toolchain.abi.clone(), PackageHealth::Missing)
                } else {
                    let retained = version.retained_headers.iter().chain(version.ordered_link_outputs.iter()).copied().collect::<Vec<_>>();
                    let identity = ReceiptIdentity { package: name, version: version.version, recipe: version.recipe_revision, source_sha256: version.source.sha256, target: target.as_str(), abi: &toolchain.abi, toolchain_fingerprint: &toolchain.fingerprint, required_outputs: &retained };
                    let valid = ArtifactReceipt::load(&root).and_then(|receipt| receipt.verify(&root, &identity)).is_ok();
                    (toolchain.abi.clone(), if valid { PackageHealth::Installed } else { PackageHealth::Corrupt })
                }
            }
        };
        rows.push((name.clone(), version_name.clone(), locked_version, abi, health));
    }
    Ok(rows)
}

/// Reports stale staging siblings without deleting them.
pub fn stale_staging_paths(cache: &CacheLayout) -> Vec<String> {
    let mut paths = Vec::new();
    collect_staging(&cache.artifacts, &mut paths);
    paths.sort();
    paths
}

/// Summarizes cache file bytes and staging/quarantine subtrees without following symlinks.
pub fn cache_stats(cache: &CacheLayout) -> CacheStats {
    let mut stats = CacheStats::default();
    collect_cache_stats(&cache.root, false, &mut stats);
    stats
}

/// Formats an approximate byte count with a stable binary unit.
pub fn approximate_size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    if bytes >= GIB {
        format!("~{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("~{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("~{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("~{bytes} B")
    }
}

/// Recursively collects staging/quarantine diagnostics without following symlinks.
fn collect_staging(root: &Path, output: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(root) else { return; };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".stage.") || name.contains(".quarantine.") {
            output.push(path.display().to_string());
        }
        if entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) {
            collect_staging(&path, output);
        }
    }
}

/// Recursively accumulates regular-file sizes and top-level stale subtree totals.
fn collect_cache_stats(path: &Path, inside_stale: bool, stats: &mut CacheStats) {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    let starts_stale = !inside_stale
        && (name.contains(".stage.") || name.contains(".quarantine."));
    if starts_stale {
        stats.stale_staging_count += 1;
    }
    let stale = inside_stale || starts_stale;
    if metadata.file_type().is_file() {
        stats.approximate_bytes = stats.approximate_bytes.saturating_add(metadata.len());
        if stale {
            stats.stale_staging_bytes =
                stats.stale_staging_bytes.saturating_add(metadata.len());
        }
        return;
    }
    if !metadata.file_type().is_dir() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_cache_stats(&entry.path(), stale, stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Verifies doctor size accounting includes stale subtree bytes without following symlinks.
    #[test]
    fn cache_stats_report_size_and_stale_staging_summary() {
        let root = std::env::temp_dir().join(format!(
            "elephc-doctor-stats-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache =
            CacheLayout::from_values(&root, Some(root.join("cache").as_os_str()), None, None)
                .unwrap();
        fs::create_dir_all(cache.sources.clone()).unwrap();
        fs::write(cache.sources.join("source"), vec![0_u8; 1024]).unwrap();
        let stale = cache.artifacts.join(".fixture.stage.abandoned");
        fs::create_dir_all(&stale).unwrap();
        fs::write(stale.join("partial"), vec![0_u8; 512]).unwrap();

        let stats = cache_stats(&cache);
        assert_eq!(stats.approximate_bytes, 1536);
        assert_eq!(stats.stale_staging_count, 1);
        assert_eq!(stats.stale_staging_bytes, 512);
        assert_eq!(approximate_size(stats.approximate_bytes), "~1.5 KiB");

        fs::remove_dir_all(root).unwrap();
    }
}
