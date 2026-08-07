//! Purpose:
//! Implements read-only native dependency inspection and explicit cache pruning commands.
//!
//! Called from:
//! - `crate::native_deps::orchestration::run_native_command_with()`.
//!
//! Key details:
//! - Listing and doctor diagnostics include deterministic recovery commands.

use std::path::Path;

use crate::codegen_support::platform::Target;
use crate::native_deps::cache::CacheLayout;
use crate::native_deps::cli::NativeOptions;
use crate::native_deps::doctor::{self, PackageHealth};
use crate::native_deps::error::{recovery_from_project, NativeError};
use crate::native_deps::lockfile::NativeLock;
use crate::native_deps::manifest::ManifestDocument;
use crate::native_deps::prune as cache_prune;
use crate::native_deps::project::discover_for_native;
use crate::native_deps::toolchain::ToolchainProvider;

use super::support::{project_cache, selected_target, success};
use super::NativeRunOutput;

/// Lists deterministic manifest/lock/artifact state without mutating any path.
pub(super) fn list(options: &NativeOptions, cwd: &Path, toolchains: &dyn ToolchainProvider) -> Result<NativeRunOutput, NativeError> {
    let Some(project) = discover_for_native(cwd, options.manifest_path.as_deref(), false)? else {
        return Ok(success("no native dependencies (no elephc.toml discovered)\n".to_string()));
    };
    let cache = project_cache(
        cwd,
        &project,
        &format!(
            "elephc native install --locked --target {}",
            selected_target(options).as_str()
        ),
    )?;
    let rows = doctor::inspect(&project, selected_target(options), &cache, toolchains)?;
    if rows.is_empty() {
        return Ok(success("no native dependencies declared\n".to_string()));
    }
    let mut output = String::new();
    let mut healthy = true;
    let mut lock_repair = false;
    for (name, manifest_version, locked_version, abi, health) in rows {
        healthy &= health == PackageHealth::Installed;
        lock_repair |= health == PackageHealth::Stale;
        output.push_str(&format!("{name}\t{manifest_version}\t{}\t{}\t{abi}\t{}\n", locked_version.unwrap_or_else(|| "unlocked".to_string()), selected_target(options).as_str(), health.as_str()));
    }
    if !healthy {
        let command = if lock_repair {
            format!(
                "elephc native install --target {}",
                selected_target(options).as_str()
            )
        } else {
            format!(
                "elephc native install --locked --target {}",
                selected_target(options).as_str()
            )
        };
        output.push_str(&format!("project: {}\n", project.root.display()));
        output.push_str(&format!(
            "recovery: {}\n",
            recovery_from_project(&project.root, &command)
        ));
    }
    Ok(NativeRunOutput { stdout: output, exit_code: if healthy { 0 } else { 1 } })
}

/// Reports project, cache, toolchain, package, and stale-staging health without cleanup.
pub(super) fn doctor(options: &NativeOptions, cwd: &Path, toolchains: &dyn ToolchainProvider) -> Result<NativeRunOutput, NativeError> {
    let target = selected_target(options);
    let discovered = discover_for_native(cwd, options.manifest_path.as_deref(), false)?;
    let search_root = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let cache = CacheLayout::from_environment(cwd).map_err(|error| match &discovered {
        Some(project) => error
            .with_project(&project.root)
            .with_recovery(format!(
                "elephc native install --locked --target {}",
                target.as_str()
            )),
        None => error
            .with_missing_project(&search_root)
            .with_recovery("elephc native add pcre2"),
    })?;
    let stats = doctor::cache_stats(&cache);
    let Some(project) = discovered else {
        let selected_toolchain = toolchains.resolve(target);
        let cache_available = cache.root.is_dir();
        let stale = doctor::stale_staging_paths(&cache);
        let (tuple, abi) = selected_toolchain.as_ref().map(|toolchain| (toolchain.target_tuple.as_str(), toolchain.abi.as_str())).unwrap_or(("unresolved", "unresolved"));
        let mut output = format!(
            "project: missing (searched from {})\ncache: {} ({})\ncache size: {}\nstale staging summary: {} ({})\ntarget: {}\ntoolchain: {}\nabi: {}\n",
            search_root.display(),
            cache.root.display(),
            if cache_available { "available" } else { "missing" },
            doctor::approximate_size(stats.approximate_bytes),
            stats.stale_staging_count,
            doctor::approximate_size(stats.stale_staging_bytes),
            target.as_str(),
            tuple,
            abi,
        );
        for path in stale {
            output.push_str(&format!("stale staging: {path}\n"));
        }
        output.push_str(&format!(
            "recovery: {}\n",
            recovery_from_project(&search_root, "elephc native add pcre2")
        ));
        output.push_str("summary: unhealthy\n");
        return Ok(NativeRunOutput { stdout: output, exit_code: 1 });
    };
    let manifest = ManifestDocument::load(&project.manifest)?;
    let lock_consistent = NativeLock::load(&project.lock).and_then(|lock| lock.validate_current(&manifest)).is_ok();
    let selected_toolchain = toolchains.resolve(selected_target(options));
    let cache_available = cache.root.is_dir();
    let rows = doctor::inspect(&project, selected_target(options), &cache, toolchains)?;
    let stale = doctor::stale_staging_paths(&cache);
    let mut healthy = stale.is_empty() && lock_consistent && cache_available && selected_toolchain.is_ok();
    let mut artifact_repair = false;
    let (tuple, abi) = selected_toolchain.as_ref().map(|toolchain| (toolchain.target_tuple.as_str(), toolchain.abi.as_str())).unwrap_or(("unresolved", "unresolved"));
    let mut output = format!("project: {}\nmanifest: {}\nlock: {} ({})\ncache: {} ({})\ncache size: {}\nstale staging summary: {} ({})\ntarget: {}\ntoolchain: {}\nabi: {}\n", project.root.display(), project.manifest.display(), project.lock.display(), if lock_consistent { "current" } else { "missing-or-stale" }, cache.root.display(), if cache_available { "available" } else { "missing" }, doctor::approximate_size(stats.approximate_bytes), stats.stale_staging_count, doctor::approximate_size(stats.stale_staging_bytes), selected_target(options).as_str(), tuple, abi);
    for (name, manifest_version, locked_version, abi, health) in rows {
        healthy &= health == PackageHealth::Installed;
        artifact_repair |= matches!(
            health,
            PackageHealth::Missing | PackageHealth::Corrupt | PackageHealth::ToolchainError
        );
        output.push_str(&format!("package {name}: manifest={manifest_version} lock={} abi={abi} {}\n", locked_version.unwrap_or_else(|| "missing".to_string()), health.as_str()));
    }
    for path in stale {
        output.push_str(&format!("stale staging: {path}\n"));
    }
    if !healthy {
        let command = if !lock_consistent {
            format!(
                "elephc native install --target {}",
                selected_target(options).as_str()
            )
        } else if artifact_repair {
            format!(
                "elephc native install --locked --target {}",
                selected_target(options).as_str()
            )
        } else {
            "elephc native prune".to_string()
        };
        output.push_str(&format!(
            "recovery: {}\n",
            recovery_from_project(&project.root, &command)
        ));
    }
    output.push_str(if healthy { "summary: healthy\n" } else { "summary: unhealthy\n" });
    Ok(NativeRunOutput { stdout: output, exit_code: if healthy { 0 } else { 1 } })
}

/// Explicitly prunes selected-target stale fingerprints and abandoned publication siblings.
pub(super) fn prune(
    requested_target: Option<Target>,
    cwd: &Path,
    toolchains: &dyn ToolchainProvider,
) -> Result<NativeRunOutput, NativeError> {
    let cache = CacheLayout::from_environment(cwd)?;
    let target = requested_target.unwrap_or_else(Target::detect_host);
    if !cache.artifacts.exists() {
        return Ok(success(format!(
            "cache: {}\nremoved stale artifacts: 0\nremoved abandoned staging: 0\nreclaimed: ~0 B\n",
            cache.root.display()
        )));
    }
    let toolchain = toolchains.resolve(target).map_err(|error| {
        error.with_default_recovery(format!(
            "elephc native prune --target {}",
            target.as_str()
        ))
    })?;
    let report = cache_prune::prune_cache(&cache, target, &toolchain)?;
    Ok(success(format!(
        "cache: {}\nremoved stale artifacts: {}\nremoved abandoned staging: {}\nreclaimed: {}\n",
        cache.root.display(),
        report.removed_artifacts,
        report.removed_staging,
        doctor::approximate_size(report.reclaimed_bytes)
    )))
}

