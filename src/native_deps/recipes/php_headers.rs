//! Purpose:
//! Stages the PHP headers a hosted extension compiles against, from a pinned
//! php-src tarball plus the three configure-generated headers Elephc owns.
//!
//! Called from:
//! - `super::super::recipe::CuratedRecipes` for the `php-headers` package.
//!
//! Key details:
//! - No `configure` run. Of the ~90 headers an extension pulls in, only three
//!   are absent from the tarball, and all three are supplied here — which is
//!   what makes this a copy rather than a build.
//! - `php_config.h` is 20 defines against php's generated 2234. Almost all of
//!   that describes php's own `.c` files; Elephc compiles extensions only.
//! - Headers are flattened into the installed layout (`include/Zend/…`) rather
//!   than left in the source tree layout, so extensions see the include paths
//!   they expect.

use std::fs;
use std::path::Path;

use super::super::error::{NativeError, NativeErrorKind};
use super::super::recipe::RecipeRequest;

/// The three headers `./configure` would generate, embedded so an installed
/// Elephc needs no repository access.
pub const PHP_CONFIG_H: &str = include_str!("php_config.h");
pub const ZEND_CONFIG_H: &str = include_str!("zend_config.h");
pub const BUILD_DEFS_H: &str = include_str!("build-defs.h");

/// Source subtrees whose headers are staged, in installed-layout order.
const HEADER_ROOTS: &[&str] = &["Zend", "main", "TSRM", "ext"];

fn build_error(message: impl Into<String>) -> NativeError {
    NativeError::new(NativeErrorKind::Build, message)
}

/// Copies every `.h` under `root` into `dest`, preserving relative structure.
fn stage_headers(root: &Path, dest: &Path) -> Result<u64, NativeError> {
    let mut staged = 0;
    if !root.is_dir() {
        return Ok(0);
    }
    let entries = fs::read_dir(root)
        .map_err(|error| NativeError::io("read php-src header directory", root, error))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| NativeError::io("enumerate php-src headers", root, error))?;
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() {
            staged += stage_headers(&path, &dest.join(&name))?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("h") {
            fs::create_dir_all(dest)
                .map_err(|error| NativeError::io("create staged header directory", dest, error))?;
            let target = dest.join(&name);
            fs::copy(&path, &target)
                .map_err(|error| NativeError::io("stage php header", &path, error))?;
            staged += 1;
        }
    }
    Ok(staged)
}

/// Stages headers for the catalog-declared prefix.
pub fn build(request: &RecipeRequest<'_>) -> Result<(), NativeError> {
    let include = request.staging_prefix.join("include");
    fs::create_dir_all(&include)
        .map_err(|error| NativeError::io("create staged include directory", &include, error))?;

    let mut staged = 0;
    for root in HEADER_ROOTS {
        staged += stage_headers(&request.source.join(root), &include.join(root))?;
    }
    if staged == 0 {
        return Err(build_error(
            "no headers found in the php-src tree: the archive layout is not what the recipe expects",
        ));
    }

    // The three configure would have generated. Written last so they overwrite
    // nothing and their absence upstream is obvious in the diff.
    write_owned(&include.join("main").join("php_config.h"), PHP_CONFIG_H)?;
    write_owned(&include.join("Zend").join("zend_config.h"), ZEND_CONFIG_H)?;
    write_owned(&include.join("main").join("build-defs.h"), BUILD_DEFS_H)?;

    Ok(())
}

fn write_owned(path: &Path, contents: &str) -> Result<(), NativeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| NativeError::io("create staged header directory", parent, error))?;
    }
    fs::write(path, contents)
        .map_err(|error| NativeError::io("write Elephc-owned php header", path, error))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole reason this is a copy and not a build: the configuration is
    /// small enough to own outright.
    #[test]
    fn the_owned_configuration_is_small() {
        let defines = PHP_CONFIG_H
            .lines()
            .filter(|l| l.trim_start().starts_with("#define"))
            .count();
        assert!(
            (10..=40).contains(&defines),
            "php_config.h should stay a small owned file, found {defines} defines"
        );
    }

    /// These are exactly the headers `./configure` generates and the tarball
    /// therefore lacks; if one went missing, extensions would fail to compile
    /// with an error pointing at php rather than at us.
    #[test]
    fn supplies_every_header_the_tarball_lacks() {
        assert!(PHP_CONFIG_H.contains("ZEND_API"), "visibility is defined here, not in zend_portability.h");
        assert!(PHP_CONFIG_H.contains("SIZEOF_SIZE_T"), "zval layout depends on this");
        assert!(PHP_CONFIG_H.contains("ZEND_MM_ALIGNMENT"), "zend_alloc.h refuses without it");
        assert!(ZEND_CONFIG_H.contains("php_config.h"), "zend_config.h is a redirect");
        assert!(BUILD_DEFS_H.contains("PHP_EXTENSION_DIR"), "extensions read install paths");
    }

    /// zend_string.h calls free() before any Zend header pulls stdlib in on Unix.
    #[test]
    fn guarantees_stdlib_is_available() {
        assert!(PHP_CONFIG_H.contains("#include <stdlib.h>"));
    }

    /// Hosted extensions are release builds; a debug-mode mismatch against a
    /// release shim would be an ABI difference, not a preference.
    #[test]
    fn pins_release_mode() {
        assert!(PHP_CONFIG_H.contains("#define ZEND_DEBUG 0"));
    }
}
