//! Purpose:
//! Shares project discovery, cache selection, publication, and output helpers across native commands.
//!
//! Called from:
//! - Native dependency mutation and inspection command modules.
//!
//! Key details:
//! - Manifest and lock publication remains atomic and ordered.

use std::path::Path;

use crate::codegen_support::platform::Target;
use crate::native_deps::cache::CacheLayout;
use crate::native_deps::cli::NativeOptions;
use crate::native_deps::error::{NativeError, NativeErrorKind};
use crate::native_deps::lockfile::NativeLock;
use crate::native_deps::manifest::ManifestDocument;
use crate::native_deps::project::{discover_for_native, ProjectPaths};
use crate::native_deps::util::atomic_write;

use super::NativeRunOutput;

/// Returns the explicitly selected target or the supported host target.
pub(super) fn selected_target(options: &NativeOptions) -> Target {
    options.target.unwrap_or_else(Target::detect_host)
}

/// Resolves cache configuration while retaining the already discovered project in diagnostics.
pub(super) fn project_cache(
    cwd: &Path,
    project: &ProjectPaths,
    recovery: &str,
) -> Result<CacheLayout, NativeError> {
    CacheLayout::from_environment(cwd).map_err(|error| {
        error
            .with_project(&project.root)
            .with_default_recovery(recovery)
    })
}

/// Discovers a project and converts an absent manifest into a command-specific hard error.
pub(super) fn required_project(cwd: &Path, explicit: Option<&Path>, create: bool) -> Result<ProjectPaths, NativeError> {
    discover_for_native(cwd, explicit, create)?.ok_or_else(|| {
        let search_root = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
        NativeError::new(
            NativeErrorKind::Project,
            "no elephc.toml discovered; pass --manifest-path or initialize this directory",
        )
        .with_missing_project(search_root)
        .with_recovery("elephc native add pcre2")
    })
}

/// Atomically publishes manifest then deterministic lock after successful installation.
pub(super) fn publish_project(project: &ProjectPaths, manifest: &ManifestDocument, lock: &NativeLock) -> Result<(), NativeError> {
    atomic_write(&project.manifest, manifest.render().as_bytes())?;
    atomic_write(&project.lock, lock.render()?.as_bytes())
}

/// Constructs successful captured output.
pub(super) fn success(stdout: String) -> NativeRunOutput {
    NativeRunOutput { stdout, exit_code: 0 }
}
