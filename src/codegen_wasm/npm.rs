//! Purpose:
//! Writes the Node.js ESM package produced by `--emit npm` for a compiled
//! wasm32-wasi command module.
//!
//! Called from:
//! - `crate::pipeline::emit_wasm_artifacts()` after WAT has been encoded to WASM.
//!
//! Key details:
//! - The generated loader uses Node's built-in `node:wasi` preview1 runtime.
//! - The package keeps the WASM binary beside the loader and exposes a reusable
//!   asynchronous `run()` API as well as a directly executable `index.mjs`.

use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const WASM_FILENAME: &str = "module.wasm";
const MAX_NPM_PACKAGE_NAME_BYTES: usize = 214;
static STAGING_SEQ: AtomicU64 = AtomicU64::new(0);

/// Writes a complete Node.js ESM package containing `wasm_bytes`.
pub fn write_package(
    package_dir: &Path,
    source_stem: &str,
    wasm_bytes: &[u8],
) -> io::Result<()> {
    fs::create_dir_all(package_dir)?;
    let package_name = npm_package_name(source_stem);
    let package_json = serde_json::to_string_pretty(&json!({
        "name": &package_name,
        "version": "0.0.0",
        "description": format!("wasm32-wasi command compiled from {source_stem}.php by elephc"),
        "type": "module",
        "exports": "./index.mjs",
        "types": "./index.d.ts",
        "files": [
            "index.mjs",
            "index.d.ts",
            WASM_FILENAME,
            "README.md"
        ],
        "engines": {
            "node": ">=20.0.0"
        }
    }))
    .map_err(io::Error::other)?;

    fs::write(package_dir.join(WASM_FILENAME), wasm_bytes)?;
    fs::write(package_dir.join("package.json"), format!("{package_json}\n"))?;
    fs::write(package_dir.join("index.mjs"), loader_source(&package_name))?;
    fs::write(package_dir.join("index.d.ts"), type_declarations())?;
    fs::write(
        package_dir.join("README.md"),
        readme_source(&package_name, source_stem),
    )?;
    Ok(())
}

/// A complete package prepared beside its final destination but not yet visible.
///
/// The artifact publisher keeps the previous package in `backup` until every
/// sibling artifact has committed, allowing a later failure to roll the whole
/// publication back.
pub(super) struct StagedPackage {
    destination: PathBuf,
    staging: PathBuf,
    backup: PathBuf,
    had_previous: bool,
    previous_backed_up: bool,
    published: bool,
    finalized: bool,
}

impl StagedPackage {
    /// Moves the staged package into place while retaining any previous package
    /// in its private backup path for a possible transaction rollback.
    pub(super) fn publish(&mut self) -> io::Result<()> {
        if self.had_previous {
            fs::rename(&self.destination, &self.backup)?;
            self.previous_backed_up = true;
        }

        if let Err(publish_error) = fs::rename(&self.staging, &self.destination) {
            if self.previous_backed_up {
                if let Err(restore_error) = fs::rename(&self.backup, &self.destination) {
                    return Err(io::Error::other(format!(
                        "publishing package failed ({publish_error}); restoring the previous package failed ({restore_error})"
                    )));
                }
                self.previous_backed_up = false;
            }
            return Err(publish_error);
        }
        self.published = true;
        Ok(())
    }

    /// Restores the pre-publication state and removes every staging or backup
    /// path owned by this package transaction.
    pub(super) fn rollback(&mut self) -> io::Result<()> {
        let mut errors = Vec::new();

        if self.published {
            if let Err(error) = remove_path_if_exists(&self.destination) {
                errors.push(format!(
                    "removing newly published package '{}': {error}",
                    self.destination.display()
                ));
            } else {
                self.published = false;
            }
        }

        if self.previous_backed_up && !self.destination.exists() {
            if let Err(error) = fs::rename(&self.backup, &self.destination) {
                errors.push(format!(
                    "restoring previous package '{}': {error}",
                    self.destination.display()
                ));
            } else {
                self.previous_backed_up = false;
            }
        }

        if let Err(error) = remove_path_if_exists(&self.staging) {
            errors.push(format!(
                "removing package staging path '{}': {error}",
                self.staging.display()
            ));
        }
        if !self.previous_backed_up {
            if let Err(error) = remove_path_if_exists(&self.backup) {
                errors.push(format!(
                    "removing package backup path '{}': {error}",
                    self.backup.display()
                ));
            }
        }

        if errors.is_empty() {
            self.finalized = true;
            Ok(())
        } else {
            Err(io::Error::other(errors.join("; ")))
        }
    }

    /// Marks the published package as committed so cleanup failures cannot
    /// trigger an independent rollback after sibling artifacts have committed.
    pub(super) fn commit(&mut self) {
        self.previous_backed_up = false;
        self.finalized = true;
    }

    /// Removes retained staging and backup paths after the enclosing
    /// multi-artifact transaction has committed every destination.
    pub(super) fn cleanup(&mut self) -> io::Result<()> {
        remove_path_if_exists(&self.staging)?;
        remove_path_if_exists(&self.backup)?;
        Ok(())
    }
}

impl Drop for StagedPackage {
    /// Best-effort safety net for early returns; explicit callers still surface
    /// rollback failures through `rollback`.
    fn drop(&mut self) {
        if !self.finalized {
            let _ = self.rollback();
        }
    }
}

/// Builds a complete package in a unique sibling staging directory.
///
/// Existing destinations must be real directories. Files and symlinks are
/// rejected before staging so compiling cannot rename unrelated user data.
pub(super) fn stage_package(
    package_dir: &Path,
    source_stem: &str,
    wasm_bytes: &[u8],
) -> io::Result<StagedPackage> {
    let had_previous = match fs::symlink_metadata(package_dir) {
        Ok(metadata) if metadata.file_type().is_dir() => true,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "NPM package destination '{}' exists and is not a directory",
                    package_dir.display()
                ),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };

    let staging = unique_sibling_path(package_dir, "stage");
    let backup = unique_sibling_path(package_dir, "backup");
    if let Err(write_error) = write_package(&staging, source_stem, wasm_bytes) {
        return match remove_path_if_exists(&staging) {
            Ok(()) => Err(write_error),
            Err(cleanup_error) => Err(io::Error::other(format!(
                "building package failed ({write_error}); cleaning staging failed ({cleanup_error})"
            ))),
        };
    }

    Ok(StagedPackage {
        destination: package_dir.to_path_buf(),
        staging,
        backup,
        had_previous,
        previous_backed_up: false,
        published: false,
        finalized: false,
    })
}

/// Publishes the NPM package through sibling staging and backup directories so
/// a failed write restores the previous package and leaves no transaction debris.
#[cfg(test)]
pub fn write_package_atomic(
    package_dir: &Path,
    source_stem: &str,
    wasm_bytes: &[u8],
) -> io::Result<()> {
    let mut package = stage_package(package_dir, source_stem, wasm_bytes)?;
    if let Err(publish_error) = package.publish() {
        return match package.rollback() {
            Ok(()) => Err(publish_error),
            Err(rollback_error) => Err(io::Error::other(format!(
                "publishing package failed ({publish_error}); rollback failed ({rollback_error})"
            ))),
        };
    }
    package.commit();
    package.cleanup()
}

/// Returns a process- and sequence-unique sibling path that does not currently
/// exist, avoiding deletion of stale or unrelated paths on name collision.
fn unique_sibling_path(destination: &Path, role: &str) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("npm-package");
    loop {
        let candidate = parent.join(format!(
            ".{name}.elephc-{role}-{}-{}",
            std::process::id(),
            STAGING_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        if fs::symlink_metadata(&candidate)
            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        {
            return candidate;
        }
    }
}

/// Removes a file, symlink, or directory if it exists, preserving real errors
/// instead of silently leaving transaction debris.
fn remove_path_if_exists(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Converts a source stem into a valid, deterministic unscoped NPM package name.
fn npm_package_name(source_stem: &str) -> String {
    let mut name = String::with_capacity(source_stem.len());
    let mut previous_dash = false;
    for ch in source_stem.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            Some(ch.to_ascii_lowercase())
        } else if matches!(ch, '-' | '_' | '.') {
            Some(ch)
        } else {
            Some('-')
        };
        if let Some(ch) = normalized {
            if ch == '-' {
                if previous_dash {
                    continue;
                }
                previous_dash = true;
            } else {
                previous_dash = false;
            }
            name.push(ch);
        }
    }
    let name = name.trim_matches(['-', '.', '_']);
    let mut name = name
        .get(..name.len().min(MAX_NPM_PACKAGE_NAME_BYTES))
        .unwrap_or(name)
        .trim_end_matches(['-', '.', '_'])
        .to_string();
    if matches!(name.as_str(), "node_modules" | "favicon.ico") {
        name.insert_str(0, "elephc-");
    }
    if name.is_empty() {
        "elephc-wasm".to_string()
    } else {
        name
    }
}

/// Renders the reusable and directly executable Node.js WASI loader.
fn loader_source(package_name: &str) -> String {
    format!(
        r#"import {{ realpathSync }} from "node:fs";
import {{ readFile }} from "node:fs/promises";
import {{ fileURLToPath }} from "node:url";
import {{ WASI }} from "node:wasi";

const wasmUrl = new URL("./{WASM_FILENAME}", import.meta.url);

/**
 * Runs the compiled PHP command under Node's WASI preview1 runtime.
 *
 * @param {{ args?: string[], env?: Record<string, string | undefined>, preopens?: Record<string, string> }} options
 * @returns {{Promise<number>}} the WASI process exit code
 */
export async function run({{
  args = ["{package_name}"],
  env = process.env,
  preopens = {{}},
}} = {{}}) {{
  const wasi = new WASI({{
    version: "preview1",
    args,
    env: Object.fromEntries(
      Object.entries(env).filter(([, value]) => typeof value === "string"),
    ),
    preopens,
    returnOnExit: true,
  }});
  const module = await WebAssembly.compile(await readFile(wasmUrl));
  const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
  const exitCode = wasi.start(instance);
  return typeof exitCode === "number" ? exitCode : 0;
}}

const invokedPath = process.argv[1];
if (
  invokedPath &&
  realpathSync(invokedPath) === realpathSync(fileURLToPath(import.meta.url))
) {{
  process.exitCode = await run({{
    args: [invokedPath, ...process.argv.slice(2)],
  }});
}}
"#
    )
}

/// Returns TypeScript declarations for the generated loader API.
fn type_declarations() -> &'static str {
    r#"export interface RunOptions {
  args?: string[];
  env?: Readonly<Record<string, string | undefined>>;
  preopens?: Record<string, string>;
}

export declare function run(options?: RunOptions): Promise<number>;
"#
}

/// Renders concise usage documentation for the generated package.
fn readme_source(package_name: &str, source_stem: &str) -> String {
    format!(
        r#"# {package_name}

Node.js WASI package generated by elephc from `{source_stem}.php`.

Requires Node.js 20 or newer.

```js
import {{ run }} from "{package_name}";

const exitCode = await run({{
  args: ["{package_name}", "first-argument"],
  env: process.env,
  preopens: {{ "/work": process.cwd() }},
}});
```

Run the command directly:

```bash
node index.mjs first-argument
```
"#
    )
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Regression tests for the generated Node.js WASI package layout and metadata.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Tests use a unique temporary directory and do not require Node.js.

    use super::{npm_package_name, write_package, write_package_atomic};
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TMP_SEQ: AtomicU32 = AtomicU32::new(0);

    /// Returns a unique temporary package directory for a parallel test run.
    fn temp_package_dir() -> std::path::PathBuf {
        let sequence = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "elephc_npm_package_{}_{}",
            std::process::id(),
            sequence
        ))
    }

    /// Lists transaction staging and backup entries belonging to `package_dir`.
    fn package_debris(package_dir: &std::path::Path) -> Vec<String> {
        let parent = package_dir.parent().expect("package parent");
        let package_name = package_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("package name");
        fs::read_dir(parent)
            .expect("read package parent")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| {
                name.starts_with(&format!(".{package_name}.elephc-stage-"))
                    || name.starts_with(&format!(".{package_name}.elephc-backup-"))
            })
            .collect()
    }

    /// Verifies package names are lowercase, NPM-safe, and never empty.
    #[test]
    fn normalizes_npm_package_names() {
        assert_eq!(npm_package_name("Hello_WASM"), "hello_wasm");
        assert_eq!(npm_package_name("hello world!"), "hello-world");
        assert_eq!(npm_package_name("..."), "elephc-wasm");
        assert_eq!(npm_package_name("node_modules"), "elephc-node_modules");
        assert_eq!(npm_package_name(&"a".repeat(300)).len(), 214);
    }

    /// Verifies `--emit npm` writes the binary, ESM loader, types, metadata, and README.
    #[test]
    fn writes_complete_npm_package() {
        let package_dir = temp_package_dir();
        let wasm = b"\0asm\x01\0\0\0";
        write_package(&package_dir, "Hello App", wasm).expect("write npm package");

        assert_eq!(
            fs::read(package_dir.join("module.wasm")).expect("read module"),
            wasm
        );
        let metadata: serde_json::Value = serde_json::from_slice(
            &fs::read(package_dir.join("package.json")).expect("read package metadata"),
        )
        .expect("parse package metadata");
        assert_eq!(metadata["name"], "hello-app");
        assert_eq!(metadata["type"], "module");
        assert_eq!(metadata["exports"], "./index.mjs");

        let loader = fs::read_to_string(package_dir.join("index.mjs")).expect("read loader");
        assert!(loader.contains("new WASI"));
        assert!(loader.contains("version: \"preview1\""));
        assert!(loader.contains("wasi.getImportObject()"));
        assert!(loader.contains("typeof value === \"string\""));
        assert!(loader.contains("export async function run"));
        let declarations =
            fs::read_to_string(package_dir.join("index.d.ts")).expect("read declarations");
        assert!(declarations.contains("string | undefined"));
        assert!(package_dir.join("index.d.ts").is_file());
        assert!(package_dir.join("README.md").is_file());

        fs::remove_dir_all(package_dir).expect("remove temporary package");
    }

    /// Verifies replacing an existing package preserves a complete final tree
    /// and leaves no staging or backup directory behind.
    #[test]
    fn atomically_replaces_existing_package() {
        let package_dir = temp_package_dir();
        write_package(&package_dir, "Old App", b"old").expect("write old package");

        write_package_atomic(&package_dir, "New App", b"new").expect("replace package");

        assert_eq!(
            fs::read(package_dir.join("module.wasm")).expect("read replaced module"),
            b"new"
        );
        let debris = package_debris(&package_dir);
        assert!(debris.is_empty(), "unexpected package debris: {debris:?}");

        fs::remove_dir_all(package_dir).expect("remove temporary package");
    }

    /// Verifies an existing non-directory destination is rejected without
    /// renaming or deleting the user's file or leaving transaction debris.
    #[test]
    fn rejects_existing_file_destination_without_mutation() {
        let package_dir = temp_package_dir();
        fs::write(&package_dir, b"user-owned").expect("write destination file");

        let error = write_package_atomic(&package_dir, "New App", b"new")
            .expect_err("file destination must be rejected");

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(
            error.to_string().contains("is not a directory"),
            "unexpected error: {error}"
        );
        assert_eq!(
            fs::read(&package_dir).expect("read destination file"),
            b"user-owned",
            "the existing file must remain byte-identical"
        );
        let debris = package_debris(&package_dir);
        assert!(debris.is_empty(), "unexpected package debris: {debris:?}");

        fs::remove_file(package_dir).expect("remove destination file");
    }
}
