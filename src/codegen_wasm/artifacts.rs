//! Purpose:
//! Owns the WebAssembly artifact publication contract for the `wasm32-wasi`
//! backend: assemble generated WAT to bytes, type-validate those bytes with
//! `wasmparser`, and only then write any user-visible `.wat`/`.wasm`/npm
//! artifact. All directly written files use a staged write and rename so a
//! failure never exposes a partially written destination.
//!
//! Called from:
//! - `crate::pipeline::emit_wasm_artifacts()` after `crate::codegen_wasm::generate()`.
//!
//! Key details:
//! - Validation runs in memory before the filesystem is touched. `--emit-asm`
//!   performs the same assembly and binary validation even when only `.wat` is
//!   requested, then writes the validated text.
//! - `wasmparser` is a normal production dependency: the publish path depends on
//!   it, not just tests.
//! - The `.wat` text is always published (it is the readable analogue of a
//!   native `.s` file) once its bytes have been assembled and validated.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::codegen::Emit;

use super::npm;

static SEQ: AtomicU64 = AtomicU64::new(0);

/// Error raised while assembling, validating, or publishing a WebAssembly
/// artifact. Assembly and validation errors leave the filesystem untouched;
/// publication errors never expose a partially written destination.
#[derive(Debug)]
pub enum WasmPublishError {
    /// `wat::parse_str` could not assemble the generated text into a binary.
    /// Covers malformed WAT (unknown instructions, bad syntax) before validation.
    Assemble(wat::Error),
    /// `wasmparser::validate` rejected the assembled bytes. Covers type-invalid
    /// WebAssembly (stack underflow, result-type mismatch, illegal control flow).
    Validate(wasmparser::BinaryReaderError),
    /// An atomic file write failed while publishing a single artifact. The
    /// `path` is the final destination; any temp file has already been cleaned up.
    Write {
        /// The artifact path that could not be (atomically) written.
        path: PathBuf,
        /// The underlying filesystem error.
        source: io::Error,
    },
    /// The recoverable NPM package swap failed. Staging is cleaned and an
    /// existing package is restored whenever the filesystem permits it.
    Npm {
        /// The final package directory that could not be published.
        dir: PathBuf,
        /// The underlying filesystem error.
        source: io::Error,
    },
    /// `Emit::NpmPackage` was requested but no package directory was supplied.
    /// This is a pipeline wiring bug rather than a user-facing condition.
    MissingPackageDir,
}

impl std::fmt::Display for WasmPublishError {
    /// Formats the error for the compiler's stderr diagnostic, naming the
    /// offending artifact path or assembly/validation stage.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmPublishError::Assemble(err) => {
                write!(f, "WebAssembly assembly error: {err}")
            }
            WasmPublishError::Validate(err) => {
                write!(f, "WebAssembly validation error: {err}")
            }
            WasmPublishError::Write { path, source } => {
                write!(f, "Error writing '{}': {}", path.display(), source)
            }
            WasmPublishError::Npm { dir, source } => {
                write!(f, "Error publishing NPM package '{}': {}", dir.display(), source)
            }
            WasmPublishError::MissingPackageDir => {
                write!(f, "NPM output requested without a package directory")
            }
        }
    }
}

impl std::error::Error for WasmPublishError {}

/// Assembles WAT text to a WebAssembly binary in memory and fully type-validates
/// the resulting bytes with `wasmparser`, returning the validated bytes.
///
/// This is the single gate that rejects both malformed WAT (assembly failure)
/// and type-invalid WebAssembly (validation failure) before any artifact is
/// written. It touches no filesystem state.
pub fn assemble_and_validate(wat: &str) -> Result<Vec<u8>, WasmPublishError> {
    let bytes = wat::parse_str(wat).map_err(WasmPublishError::Assemble)?;
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::WASM3)
        .validate_all(&bytes)
        .map_err(WasmPublishError::Validate)?;
    Ok(bytes)
}

/// A complete file written beside its destination but not yet published.
struct StagedFile {
    destination: PathBuf,
    staging: PathBuf,
    backup: PathBuf,
    had_previous: bool,
    previous_backed_up: bool,
    published: bool,
    finalized: bool,
}

impl StagedFile {
    /// Moves the staged file into place while retaining the previous file for a
    /// possible rollback by the enclosing multi-artifact transaction.
    fn publish(&mut self) -> io::Result<()> {
        if self.had_previous {
            fs::rename(&self.destination, &self.backup)?;
            self.previous_backed_up = true;
        }

        if let Err(publish_error) = fs::rename(&self.staging, &self.destination) {
            if self.previous_backed_up {
                if let Err(restore_error) = fs::rename(&self.backup, &self.destination) {
                    return Err(io::Error::other(format!(
                        "publishing file failed ({publish_error}); restoring the previous file failed ({restore_error})"
                    )));
                }
                self.previous_backed_up = false;
            }
            return Err(publish_error);
        }
        self.published = true;
        Ok(())
    }

    /// Restores the file state that existed before publication and removes all
    /// staging and backup paths owned by this transaction.
    fn rollback(&mut self) -> io::Result<()> {
        let mut errors = Vec::new();

        if self.published {
            if let Err(error) = remove_file_if_exists(&self.destination) {
                errors.push(format!(
                    "removing newly published file '{}': {error}",
                    self.destination.display()
                ));
            } else {
                self.published = false;
            }
        }

        if self.previous_backed_up && !self.destination.exists() {
            if let Err(error) = fs::rename(&self.backup, &self.destination) {
                errors.push(format!(
                    "restoring previous file '{}': {error}",
                    self.destination.display()
                ));
            } else {
                self.previous_backed_up = false;
            }
        }

        if let Err(error) = remove_file_if_exists(&self.staging) {
            errors.push(format!(
                "removing file staging path '{}': {error}",
                self.staging.display()
            ));
        }
        if !self.previous_backed_up {
            if let Err(error) = remove_file_if_exists(&self.backup) {
                errors.push(format!(
                    "removing file backup path '{}': {error}",
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

    /// Marks the published destination as committed so later cleanup failures
    /// cannot roll it back independently of sibling artifacts.
    fn commit(&mut self) {
        self.previous_backed_up = false;
        self.finalized = true;
    }

    /// Deletes retained transaction debris after the enclosing transaction has
    /// committed every sibling artifact.
    fn cleanup(&mut self) -> io::Result<()> {
        remove_file_if_exists(&self.staging)?;
        remove_file_if_exists(&self.backup)?;
        Ok(())
    }
}

impl Drop for StagedFile {
    /// Best-effort safety net for early returns; explicit transaction paths
    /// still surface rollback errors to the caller.
    fn drop(&mut self) {
        if !self.finalized {
            let _ = self.rollback();
        }
    }
}

/// Writes a file to a unique sibling staging path after verifying that an
/// existing destination is a regular file rather than a directory or symlink.
fn stage_file(path: &Path, bytes: &[u8]) -> io::Result<StagedFile> {
    let had_previous = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => true,
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "artifact destination '{}' exists and is not a regular file",
                    path.display()
                ),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    let staging = unique_sibling_path(path, "stage");
    let backup = unique_sibling_path(path, "backup");
    if let Err(write_error) = fs::write(&staging, bytes) {
        return match remove_file_if_exists(&staging) {
            Ok(()) => Err(write_error),
            Err(cleanup_error) => Err(io::Error::other(format!(
                "staging file failed ({write_error}); cleaning staging failed ({cleanup_error})"
            ))),
        };
    }

    Ok(StagedFile {
        destination: path.to_path_buf(),
        staging,
        backup,
        had_previous,
        previous_backed_up: false,
        published: false,
        finalized: false,
    })
}

/// Publishes two staged files as one recoverable transaction.
fn publish_file_pair(
    mut first: StagedFile,
    mut second: StagedFile,
) -> Result<(), WasmPublishError> {
    if let Err(error) = first.publish() {
        let error = rollback_file_pair_error(error, &mut first, &mut second);
        return Err(write_error(&first.destination, error));
    }
    if let Err(error) = second.publish() {
        let error = rollback_file_pair_error(error, &mut first, &mut second);
        return Err(write_error(&second.destination, error));
    }

    first.commit();
    second.commit();
    let first_cleanup = first.cleanup();
    let second_cleanup = second.cleanup();
    if let Err(error) = combine_cleanup_errors(first_cleanup, second_cleanup) {
        return Err(write_error(&first.destination, error));
    }
    Ok(())
}

/// Publishes a staged WAT and NPM directory as one recoverable transaction.
fn publish_file_and_package(
    mut wat: StagedFile,
    mut package: npm::StagedPackage,
    package_dir: &Path,
) -> Result<(), WasmPublishError> {
    if let Err(error) = wat.publish() {
        let wat_rollback = wat.rollback();
        let package_rollback = package.rollback();
        let error = combine_rollback_errors(error, wat_rollback, package_rollback);
        return Err(write_error(&wat.destination, error));
    }
    if let Err(error) = package.publish() {
        let wat_rollback = wat.rollback();
        let package_rollback = package.rollback();
        let error = combine_rollback_errors(error, wat_rollback, package_rollback);
        return Err(WasmPublishError::Npm {
            dir: package_dir.to_path_buf(),
            source: error,
        });
    }

    wat.commit();
    package.commit();
    let wat_cleanup = wat.cleanup();
    let package_cleanup = package.cleanup();
    if let Err(error) = combine_cleanup_errors(wat_cleanup, package_cleanup) {
        return Err(WasmPublishError::Npm {
            dir: package_dir.to_path_buf(),
            source: error,
        });
    }
    Ok(())
}

/// Rolls both staged files back and appends any rollback failures to the
/// original publication error.
fn rollback_file_pair_error(
    error: io::Error,
    first: &mut StagedFile,
    second: &mut StagedFile,
) -> io::Error {
    combine_rollback_errors(error, first.rollback(), second.rollback())
}

/// Combines an original publication failure with up to two rollback failures.
fn combine_rollback_errors(
    error: io::Error,
    first_rollback: io::Result<()>,
    second_rollback: io::Result<()>,
) -> io::Error {
    let mut message = error.to_string();
    if let Err(rollback_error) = first_rollback {
        message.push_str(&format!("; first rollback failed ({rollback_error})"));
    }
    if let Err(rollback_error) = second_rollback {
        message.push_str(&format!("; second rollback failed ({rollback_error})"));
    }
    io::Error::new(error.kind(), message)
}

/// Combines cleanup failures after a multi-artifact transaction is committed.
fn combine_cleanup_errors(first: io::Result<()>, second: io::Result<()>) -> io::Result<()> {
    match (first, second) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Err(first_error), Err(second_error)) => Err(io::Error::other(format!(
            "first cleanup failed ({first_error}); second cleanup failed ({second_error})"
        ))),
    }
}

/// Wraps an I/O failure with the final artifact path expected by diagnostics.
fn write_error(path: &Path, source: io::Error) -> WasmPublishError {
    WasmPublishError::Write {
        path: path.to_path_buf(),
        source,
    }
}

/// Returns a process- and sequence-unique sibling path that does not currently
/// exist, without deleting a colliding stale or unrelated path.
fn unique_sibling_path(destination: &Path, role: &str) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");
    loop {
        let candidate = parent.join(format!(
            ".{name}.elephc-{role}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        if fs::symlink_metadata(&candidate)
            .is_err_and(|error| error.kind() == io::ErrorKind::NotFound)
        {
            return candidate;
        }
    }
}

/// Removes a staged, backup, or published file if present and rejects a
/// directory rather than recursively deleting an unexpected path.
fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!("'{}' is a directory", path.display()),
        )),
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Atomically writes `bytes` to `path` via a sibling temp file and rename.
///
/// The temp file lives in the same directory as `path` (so the rename is atomic
/// on the same filesystem) and is named with a process- and counter-unique
/// suffix to stay collision-free across parallel compilations. On any failure
/// the temp file is removed before returning, so a partial destination is never
/// exposed.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), WasmPublishError> {
    let mut file = stage_file(path, bytes).map_err(|source| write_error(path, source))?;
    if let Err(publish_error) = file.publish() {
        let source = match file.rollback() {
            Ok(()) => publish_error,
            Err(rollback_error) => io::Error::other(format!(
                "publishing file failed ({publish_error}); rollback failed ({rollback_error})"
            )),
        };
        return Err(write_error(path, source));
    }
    file.commit();
    file.cleanup().map_err(|source| write_error(path, source))
}

/// Validates the generated WAT in memory and then publishes the WebAssembly
/// artifacts.
///
/// The WAT is assembled to bytes and type-validated with `wasmparser` before any
/// file is written. Every requested output is then staged before publication.
/// Multi-output modes retain backups until both the readable `.wat` and the
/// `.wasm` or NPM package have committed, so a later failure restores the exact
/// pre-publication state.
///
/// `asm_path` is the readable `.wat` destination, `bin_path` is the `.wasm`
/// destination (unused for npm/`--emit-asm`), and `package_dir` is required for
/// `Emit::NpmPackage`. Returns `WasmPublishError` for any rejected module or
/// failed write. Assembly, validation, staging, and publication errors preserve
/// all previous artifacts and leave no transaction-owned staging or backup path.
pub fn publish_wasm_artifacts(
    wat: &str,
    emit: Emit,
    emit_asm: bool,
    source_stem: &str,
    asm_path: &Path,
    bin_path: &Path,
    package_dir: Option<&Path>,
) -> Result<(), WasmPublishError> {
    // Assemble and fully type-validate in memory before touching the filesystem.
    // This runs even for `--emit-asm`, so a `.wat` is only ever written once its
    // bytes have been validated.
    let wasm_bytes = assemble_and_validate(wat)?;

    if emit_asm {
        return atomic_write(asm_path, wat.as_bytes());
    }

    let staged_wat =
        stage_file(asm_path, wat.as_bytes()).map_err(|source| write_error(asm_path, source))?;

    if matches!(emit, Emit::NpmPackage) {
        let package_dir = package_dir.ok_or(WasmPublishError::MissingPackageDir)?;
        let staged_package = npm::stage_package(package_dir, source_stem, &wasm_bytes).map_err(
            |source| WasmPublishError::Npm {
                dir: package_dir.to_path_buf(),
                source,
            },
        )?;
        return publish_file_and_package(staged_wat, staged_package, package_dir);
    }

    let staged_wasm =
        stage_file(bin_path, &wasm_bytes).map_err(|source| write_error(bin_path, source))?;
    publish_file_pair(staged_wat, staged_wasm)
}

#[cfg(test)]
mod tests {
    //! Purpose:
    //! Production-path regression tests for the WASM artifact publish contract
    //! (requirement WASM-ART-001): invalid WAT is rejected and no artifact is
    //! published, across the `--emit-asm`, normal `.wasm`, and npm paths.
    //!
    //! Called from:
    //! - `cargo test` through Rust's test harness.
    //!
    //! Key details:
    //! - Each test builds paths inside a unique temporary directory so parallel
    //!   runs never collide, and asserts that a rejected module leaves no new or
    //!   partially overwritten artifact (including no leftover temp files).
    //! - `wasmparser` is exercised through the real production
    //!   `assemble_and_validate` / `publish_wasm_artifacts` entry points, not a
    //!   test-only shim.

    use super::{
        assemble_and_validate, publish_file_and_package, publish_file_pair,
        publish_wasm_artifacts, stage_file, WasmPublishError,
    };
    use crate::codegen_wasm::npm::stage_package;
    use crate::codegen::Emit;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    /// Returns a unique empty temporary directory for one test.
    fn unique_dir(tag: &str) -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("elephc_wasm_artifacts_{}_{}_{}", tag, std::process::id(), n));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// A minimal, fully valid WebAssembly text module.
    fn valid_wat() -> &'static str {
        "(module\n  (memory (export \"memory\") 1)\n  (func (export \"_start\"))\n)\n"
    }

    /// Syntactically valid WAT that assembles but is type-invalid: `i32.eqz`
    /// requires an `i32` operand but the stack is empty.
    fn type_invalid_wat() -> &'static str {
        "(module\n  (func (export \"f\") (result i32)\n    i32.eqz\n  )\n)\n"
    }

    /// Malformed WAT that `wat::parse_str` rejects at assembly time.
    fn malformed_wat() -> &'static str {
        "(module\n  (func (export \"f\")\n    not.a.real.instruction\n  )\n)\n"
    }

    /// A valid Component Model artifact that is outside the Core WebAssembly
    /// 3.0 module profile accepted by the wasm32-wasi backend.
    fn component_wat() -> &'static str {
        "(component)\n"
    }

    /// Lists every entry inside `dir` (recursive), ignoring nothing, for the
    /// "no leftover artifact" assertions.
    fn all_entries(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        collect_entries(dir, &mut out);
        out.sort();
        out
    }

    /// Recursive helper for `all_entries`.
    fn collect_entries(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            out.push(path.clone());
            if path.is_dir() {
                collect_entries(&path, out);
            }
        }
    }

    /// Lists transaction-owned staging and backup paths below a test directory.
    fn transaction_debris(dir: &Path) -> Vec<PathBuf> {
        all_entries(dir)
            .into_iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.contains(".elephc-stage-") || name.contains(".elephc-backup-")
                    })
            })
            .collect()
    }

    /// Verifies a type-invalid module is rejected by `assemble_and_validate`
    /// with a validation error, never an assembly error.
    #[test]
    fn type_invalid_wat_is_rejected_by_validation() {
        let err = assemble_and_validate(type_invalid_wat())
            .err()
            .expect("type-invalid WAT must be rejected");
        assert!(
            matches!(err, WasmPublishError::Validate(_)),
            "expected a validation error, got {err:?}"
        );
    }

    /// Verifies malformed WAT is rejected by assembly before validation runs.
    #[test]
    fn malformed_wat_is_rejected_by_assembly() {
        let err = assemble_and_validate(malformed_wat())
            .err()
            .expect("malformed WAT must be rejected");
        assert!(
            matches!(err, WasmPublishError::Assemble(_)),
            "expected an assembly error, got {err:?}"
        );
    }

    /// Verifies production validation is explicitly limited to Core 3.0 and
    /// rejects a syntactically valid Component Model artifact.
    #[test]
    fn component_model_is_rejected_by_core3_validation() {
        wat::parse_str(component_wat()).expect("the WAT parser must accept component syntax");
        let error = assemble_and_validate(component_wat())
            .expect_err("Component Model must be outside the Core 3.0 profile");
        assert!(
            matches!(error, WasmPublishError::Validate(_)),
            "expected a Core 3.0 validation error, got {error:?}"
        );
    }

    /// Verifies the normal `.wasm` path publishes no artifact when the WAT is
    /// type-invalid: neither `.wat` nor `.wasm` nor any temp file appears.
    #[test]
    fn invalid_wat_publishes_no_binary_artifact() {
        let dir = unique_dir("invalid-bin");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");

        let err = publish_wasm_artifacts(
            type_invalid_wat(),
            Emit::Executable,
            false,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .err()
        .expect("invalid WAT must be rejected");

        assert!(matches!(err, WasmPublishError::Validate(_)), "got {err:?}");
        assert!(!asm_path.exists(), "no .wat should be published on failure");
        assert!(!bin_path.exists(), "no .wasm should be published on failure");
        assert!(
            all_entries(&dir).is_empty(),
            "no temp or partial artifacts should remain: {:?}",
            all_entries(&dir)
        );
    }

    /// Verifies `--emit-asm` performs assembly and validation even though only
    /// `.wat` is requested, and publishes no `.wat` when the WAT is type-invalid.
    #[test]
    fn emit_asm_rejects_invalid_wat_and_writes_nothing() {
        let dir = unique_dir("invalid-asm");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");

        let err = publish_wasm_artifacts(
            type_invalid_wat(),
            Emit::Executable,
            true,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .err()
        .expect("invalid WAT must be rejected even for --emit-asm");

        assert!(matches!(err, WasmPublishError::Validate(_)), "got {err:?}");
        assert!(
            !asm_path.exists(),
            "no .wat should be published when validation fails"
        );
        assert!(
            all_entries(&dir).is_empty(),
            "no temp or partial artifacts should remain: {:?}",
            all_entries(&dir)
        );
    }

    /// Verifies the npm path publishes no package directory when the WAT is
    /// type-invalid, and leaves no staging directory behind.
    #[test]
    fn invalid_wat_publishes_no_npm_package() {
        let dir = unique_dir("invalid-npm");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");
        let package_dir = dir.join("out-npm");

        let err = publish_wasm_artifacts(
            type_invalid_wat(),
            Emit::NpmPackage,
            false,
            "out",
            &asm_path,
            &bin_path,
            Some(&package_dir),
        )
        .err()
        .expect("invalid WAT must be rejected for npm output");

        assert!(matches!(err, WasmPublishError::Validate(_)), "got {err:?}");
        assert!(
            !package_dir.exists(),
            "no npm package directory should be published on failure"
        );
        assert!(
            !asm_path.exists(),
            "no .wat should be published before the package is validated/published"
        );
        assert!(
            all_entries(&dir).is_empty(),
            "no staging or temp artifacts should remain: {:?}",
            all_entries(&dir)
        );
    }

    /// Verifies a valid module is published on the normal path: the `.wat` and
    /// `.wasm` both exist and the `.wasm` re-validates with `wasmparser`.
    #[test]
    fn valid_wat_publishes_binary_artifact() {
        let dir = unique_dir("valid-bin");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");

        publish_wasm_artifacts(
            valid_wat(),
            Emit::Executable,
            false,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .expect("valid WAT must publish");

        assert!(asm_path.is_file(), "the .wat should be published");
        let wasm = fs::read(&bin_path).expect("read published .wasm");
        wasmparser::validate(&wasm).expect("published .wasm must validate");
    }

    /// Verifies `--emit-asm` with a valid module writes the `.wat` only.
    #[test]
    fn emit_asm_publishes_wat_only() {
        let dir = unique_dir("valid-asm");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");

        publish_wasm_artifacts(
            valid_wat(),
            Emit::Executable,
            true,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .expect("valid WAT must publish");

        assert!(asm_path.is_file(), "the .wat should be published for --emit-asm");
        assert!(
            !bin_path.exists(),
            "no .wasm should be written when only --emit-asm was requested"
        );
        // The published .wat must still assemble and validate.
        let wat_text = fs::read_to_string(&asm_path).expect("read published .wat");
        assemble_and_validate(&wat_text).expect("published .wat must validate");
    }

    /// Verifies a valid module publishes a complete npm package.
    #[test]
    fn valid_wat_publishes_npm_package() {
        let dir = unique_dir("valid-npm");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");
        let package_dir = dir.join("out-npm");

        publish_wasm_artifacts(
            valid_wat(),
            Emit::NpmPackage,
            false,
            "out",
            &asm_path,
            &bin_path,
            Some(&package_dir),
        )
        .expect("valid WAT must publish npm package");

        assert!(asm_path.is_file(), "the .wat should be published alongside the package");
        assert!(package_dir.is_dir(), "the npm package directory should be published");
        let wasm = fs::read(package_dir.join("module.wasm")).expect("read package module.wasm");
        wasmparser::validate(&wasm).expect("published npm module.wasm must validate");
        assert!(package_dir.join("index.mjs").is_file());
        assert!(package_dir.join("package.json").is_file());
    }

    /// Verifies that a previously published `.wasm` is not clobbered or
    /// corrupted when a later compilation rejects invalid WAT: validation runs
    /// before any write, so the original artifact survives untouched.
    #[test]
    fn invalid_wat_does_not_overwrite_existing_artifact() {
        let dir = unique_dir("overwrite");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");

        // Publish a valid artifact first.
        publish_wasm_artifacts(
            valid_wat(),
            Emit::Executable,
            false,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .expect("first publish");
        let original_wasm = fs::read(&bin_path).expect("read original wasm");
        let original_wat = fs::read_to_string(&asm_path).expect("read original wat");

        // A subsequent invalid compilation must fail and leave the prior
        // artifacts byte-identical (no partial overwrite).
        let err = publish_wasm_artifacts(
            type_invalid_wat(),
            Emit::Executable,
            false,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .err()
        .expect("invalid WAT must be rejected");
        assert!(matches!(err, WasmPublishError::Validate(_)), "got {err:?}");

        assert_eq!(
            fs::read(&bin_path).expect("reread wasm"),
            original_wasm,
            "existing .wasm must not be modified by a rejected compilation"
        );
        assert_eq!(
            fs::read_to_string(&asm_path).expect("reread wat"),
            original_wat,
            "existing .wat must not be modified by a rejected compilation"
        );
    }

    /// Verifies a second file's publish failure rolls the already-published
    /// first file back and removes both outputs' staging and backup files.
    #[test]
    fn second_file_publish_failure_rolls_back_first_file() {
        let dir = unique_dir("second-publish-rollback");
        let first_path = dir.join("out.wat");
        let second_path = dir.join("out.wasm");
        fs::write(&first_path, b"previous-wat").expect("write previous wat");

        let first = stage_file(&first_path, b"replacement-wat").expect("stage wat");
        let second = stage_file(&second_path, b"replacement-wasm").expect("stage wasm");

        // Simulate a destination race after preflight/staging. Publishing the
        // first file succeeds; publishing the second file must fail on this
        // unexpected non-file path and trigger a full rollback.
        fs::create_dir(&second_path).expect("create racing destination");
        fs::write(second_path.join("sentinel"), b"unrelated").expect("write sentinel");

        let error = publish_file_pair(first, second).expect_err("second publish must fail");
        assert!(
            matches!(error, WasmPublishError::Write { ref path, .. } if path == &second_path),
            "unexpected error: {error:?}"
        );
        assert_eq!(
            fs::read(&first_path).expect("read restored wat"),
            b"previous-wat",
            "the first artifact must be restored byte-for-byte"
        );
        assert_eq!(
            fs::read(second_path.join("sentinel")).expect("read sentinel"),
            b"unrelated",
            "the unexpected second destination must remain untouched"
        );
        assert!(
            transaction_debris(&dir).is_empty(),
            "transaction debris remains: {:?}",
            transaction_debris(&dir)
        );
    }

    /// Verifies backup-cleanup failure after the commit point cannot roll back
    /// one published file while leaving its sibling committed.
    #[test]
    fn cleanup_failure_after_commit_keeps_both_published_files() {
        let dir = unique_dir("cleanup-after-commit");
        let first_path = dir.join("out.wat");
        let second_path = dir.join("out.wasm");
        fs::write(&first_path, b"previous-wat").expect("write previous wat");
        fs::write(&second_path, b"previous-wasm").expect("write previous wasm");

        let mut first = stage_file(&first_path, b"replacement-wat").expect("stage wat");
        let mut second = stage_file(&second_path, b"replacement-wasm").expect("stage wasm");
        first.publish().expect("publish wat");
        second.publish().expect("publish wasm");

        // Turn the first retained backup into a directory so file cleanup fails.
        // Both destinations are committed before cleanup begins; Drop must
        // therefore leave both replacements visible.
        fs::remove_file(&first.backup).expect("remove first backup file");
        fs::create_dir(&first.backup).expect("replace backup with directory");
        first.commit();
        second.commit();

        let first_cleanup = first.cleanup();
        let second_cleanup = second.cleanup();
        assert!(first_cleanup.is_err(), "directory backup must reject file cleanup");
        assert!(second_cleanup.is_ok(), "sibling cleanup should still run");
        drop(first);
        drop(second);

        assert_eq!(
            fs::read(&first_path).expect("read committed wat"),
            b"replacement-wat"
        );
        assert_eq!(
            fs::read(&second_path).expect("read committed wasm"),
            b"replacement-wasm"
        );

        fs::remove_dir_all(dir).expect("remove temporary directory");
    }

    /// Verifies preflight failure of the second binary destination leaves an
    /// existing WAT unchanged and publishes no transaction-owned path.
    #[test]
    fn invalid_binary_destination_preserves_existing_wat() {
        let dir = unique_dir("binary-preflight");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");
        fs::write(&asm_path, b"previous-wat").expect("write previous wat");
        fs::create_dir(&bin_path).expect("create invalid wasm destination");

        let error = publish_wasm_artifacts(
            valid_wat(),
            Emit::Executable,
            false,
            "out",
            &asm_path,
            &bin_path,
            None,
        )
        .expect_err("directory destination must fail");

        assert!(
            matches!(error, WasmPublishError::Write { ref path, .. } if path == &bin_path),
            "unexpected error: {error:?}"
        );
        assert_eq!(
            fs::read(&asm_path).expect("read existing wat"),
            b"previous-wat",
            "the first artifact must not change when the second preflight fails"
        );
        assert!(bin_path.is_dir(), "the invalid destination must remain untouched");
        assert!(
            transaction_debris(&dir).is_empty(),
            "transaction debris remains: {:?}",
            transaction_debris(&dir)
        );
    }

    /// Verifies an NPM destination file is rejected before WAT publication,
    /// preserving both prior paths and leaving no staging or backup debris.
    #[test]
    fn npm_file_destination_preserves_existing_artifacts() {
        let dir = unique_dir("npm-file-destination");
        let asm_path = dir.join("out.wat");
        let bin_path = dir.join("out.wasm");
        let package_dir = dir.join("out-npm");
        fs::write(&asm_path, b"previous-wat").expect("write previous wat");
        fs::write(&package_dir, b"user-owned").expect("write package destination file");

        let error = publish_wasm_artifacts(
            valid_wat(),
            Emit::NpmPackage,
            false,
            "out",
            &asm_path,
            &bin_path,
            Some(&package_dir),
        )
        .expect_err("file package destination must fail");

        assert!(
            matches!(error, WasmPublishError::Npm { ref dir, .. } if dir == &package_dir),
            "unexpected error: {error:?}"
        );
        assert_eq!(
            fs::read(&asm_path).expect("read existing wat"),
            b"previous-wat",
            "WAT must remain unchanged when package staging is rejected"
        );
        assert_eq!(
            fs::read(&package_dir).expect("read package destination"),
            b"user-owned",
            "the existing destination file must remain byte-identical"
        );
        assert!(
            transaction_debris(&dir).is_empty(),
            "transaction debris remains: {:?}",
            transaction_debris(&dir)
        );
    }

    /// Verifies a package destination race after staging rolls the published WAT
    /// back and preserves the unexpected path that caused the second commit to fail.
    #[test]
    fn npm_publish_failure_rolls_back_published_wat() {
        let dir = unique_dir("npm-publish-rollback");
        let asm_path = dir.join("out.wat");
        let package_dir = dir.join("out-npm");
        fs::write(&asm_path, b"previous-wat").expect("write previous wat");

        let staged_wat = stage_file(&asm_path, b"replacement-wat").expect("stage wat");
        let staged_package =
            stage_package(&package_dir, "out", b"\0asm\x01\0\0\0").expect("stage package");

        // A non-directory appears after package preflight. The WAT commit runs
        // first, then the package rename fails and must restore the prior WAT.
        fs::write(&package_dir, b"racing-user-file").expect("write racing destination");

        let error = publish_file_and_package(staged_wat, staged_package, &package_dir)
            .expect_err("package publish must fail");
        assert!(
            matches!(error, WasmPublishError::Npm { ref dir, .. } if dir == &package_dir),
            "unexpected error: {error:?}"
        );
        assert_eq!(
            fs::read(&asm_path).expect("read restored wat"),
            b"previous-wat",
            "the already-published WAT must be restored byte-for-byte"
        );
        assert_eq!(
            fs::read(&package_dir).expect("read racing destination"),
            b"racing-user-file",
            "the racing destination must remain untouched"
        );
        assert!(
            transaction_debris(&dir).is_empty(),
            "transaction debris remains: {:?}",
            transaction_debris(&dir)
        );
    }
}
