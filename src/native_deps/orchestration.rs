//! Purpose:
//! Coordinates transactional native command state transitions through injected services.
//!
//! Called from:
//! - `crate::native_deps::run_native_command` and deterministic unit tests.
//!
//! Key details:
//! - Compilation never enters this module; only explicit native commands may mutate cache or project state.

use std::path::Path;

use super::cli::NativeCommand;
use super::download::Downloader;
use super::error::NativeError;
use super::recipe::RecipeRunner;
use super::toolchain::ToolchainProvider;

mod inspection;
mod mutations;
mod support;

/// Captured stable command output and process status chosen by top-level integration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRunOutput {
    pub stdout: String,
    pub exit_code: i32,
}

/// Executes a command through injected network, recipe, and toolchain services.
pub(crate) fn run_native_command_with(
    command: &NativeCommand,
    cwd: &Path,
    downloader: &dyn Downloader,
    recipes: &dyn RecipeRunner,
    toolchains: &dyn ToolchainProvider,
) -> Result<NativeRunOutput, NativeError> {
    match command {
        NativeCommand::Add { package, version, options } => mutations::add(package, version.as_deref(), options, cwd, downloader, recipes, toolchains),
        NativeCommand::Install { locked, options } => mutations::install(*locked, options, cwd, downloader, recipes, toolchains),
        NativeCommand::Update { package, version, options } => mutations::update(package.as_deref(), version.as_deref(), options, cwd, downloader, recipes, toolchains),
        NativeCommand::Remove { package, manifest_path } => mutations::remove(package, manifest_path.as_deref(), cwd),
        NativeCommand::List { options } => inspection::list(options, cwd, toolchains),
        NativeCommand::Doctor { options } => inspection::doctor(options, cwd, toolchains),
        NativeCommand::Prune { target } => inspection::prune(*target, cwd, toolchains),
    }
}

#[cfg(test)]
#[path = "orchestration_tests.rs"]
mod tests;
