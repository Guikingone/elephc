//! Purpose:
//! Builds the catalogued zlib release as one static position-independent C archive.
//!
//! Called from:
//! - `crate::native_deps::recipe::CuratedRecipes` for zlib recipe revision 1.
//!
//! Key details:
//! - Runs the upstream static-only configure path with the selected target tools and retains only
//!   `libz.a`, `zlib.h`, and the generated `zconf.h`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::codegen_support::platform::Target;

use super::super::error::{NativeError, NativeErrorKind};
use super::super::recipe::RecipeRequest;
use super::super::toolchain::run_checked;

/// Builds zlib into the catalog-declared staging prefix.
pub fn build(request: &RecipeRequest<'_>) -> Result<(), NativeError> {
    let build = request.staging_prefix.join("build");
    let include = request.staging_prefix.join("include");
    let library = request.staging_prefix.join("lib");
    fs::create_dir_all(&build)
        .map_err(|error| NativeError::io("create zlib build directory", &build, error))?;
    fs::create_dir_all(&include)
        .map_err(|error| NativeError::io("create zlib include directory", &include, error))?;
    fs::create_dir_all(&library)
        .map_err(|error| NativeError::io("create zlib library directory", &library, error))?;

    let configure = request.source.join("configure");
    require_regular(&configure)?;
    let mut command = request.toolchain.command(Path::new("/bin/sh"));
    command.current_dir(&build).arg(&configure).arg("--static");
    if request.target != Target::detect_host() {
        command.env("CHOST", &request.toolchain.target_tuple);
    }
    run_checked(&mut command, "configure trusted zlib recipe")?;

    let mut make = request.toolchain.command(Path::new("make"));
    make.current_dir(&build).arg("libz.a");
    run_checked(&mut make, "build trusted zlib static library")?;

    copy_regular(&build.join("libz.a"), &library.join("libz.a"))?;
    copy_regular(&request.source.join("zlib.h"), &include.join("zlib.h"))?;
    copy_regular(&build.join("zconf.h"), &include.join("zconf.h"))?;

    let archive = library.join("libz.a");
    let mut inspect = request.toolchain.command(&request.toolchain.ar);
    inspect.arg("t").arg(&archive);
    run_checked(&mut inspect, "validate trusted zlib static archive")?;
    fs::remove_dir_all(&build)
        .map_err(|error| NativeError::io("remove trusted zlib build tree", &build, error))?;
    Ok(())
}

/// Requires a non-empty, non-symlink regular file produced by the trusted recipe.
fn require_regular(path: &Path) -> Result<(), NativeError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| NativeError::io("inspect zlib recipe file", path, error))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
    {
        return Err(NativeError::new(
            NativeErrorKind::Build,
            "zlib recipe file is missing, empty, symlinked, or not regular",
        )
        .with_path(path));
    }
    Ok(())
}

/// Copies one verified regular recipe output to its retained staging path.
fn copy_regular(source: &Path, destination: &Path) -> Result<PathBuf, NativeError> {
    require_regular(source)?;
    fs::copy(source, destination)
        .map_err(|error| NativeError::io("copy retained zlib output", destination, error))?;
    require_regular(destination)?;
    Ok(destination.to_path_buf())
}
