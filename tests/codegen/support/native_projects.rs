//! Purpose:
//! Builds hermetic managed-native project fixtures for CLI codegen tests.
//! Supplies exact cache metadata around the existing system-aligned PCRE2 test provider.
//!
//! Called from:
//! - CLI codegen helpers whose programs require the dynamic eval bridge.
//!
//! Key details:
//! - Fixtures use the production cache key and receipt schema without downloading sources.
//! - System PCRE2 archives are admitted only inside this test provider; production resolution
//!   remains fail-closed and never falls back to system libraries.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::json;
use sha2::{Digest, Sha256};

use super::*;

const PCRE2_VERSION: &str = "10.47";
const PCRE2_RECIPE: u32 = 1;
const PCRE2_SOURCE_SHA256: &str =
    "c08ae2388ef333e8403e670ad70c0a11f1eed021fd88308d7e02f596fcd9dc16";

/// Tool identity fields needed to reproduce the production native cache key.
struct TestNativeToolchain {
    target: String,
    abi: String,
    fingerprint: String,
    compiler_command: String,
    compiler_version: String,
    archiver_command: String,
    archiver_version: String,
    ranlib_command: String,
    ranlib_version: String,
}

/// Creates a PCRE2 project and verified cache artifact for one isolated CLI test.
pub(crate) fn prepare_managed_pcre2_cli_project(dir: &Path) -> PathBuf {
    fs::write(
        dir.join("elephc.toml"),
        include_bytes!("../../../examples/date-json-regex/elephc.toml"),
    )
    .expect("failed to write managed-PCRE2 test manifest");
    fs::write(
        dir.join("elephc.lock"),
        include_bytes!("../../../examples/date-json-regex/elephc.lock"),
    )
    .expect("failed to write managed-PCRE2 test lock");

    let toolchain = resolve_test_native_toolchain();
    let cache = dir.join("managed-native-cache");
    let artifact = cache
        .join("artifacts")
        .join("pcre2")
        .join(PCRE2_VERSION)
        .join(format!("r{PCRE2_RECIPE}"))
        .join(PCRE2_SOURCE_SHA256)
        .join(&toolchain.target)
        .join(&toolchain.abi)
        .join(&toolchain.fingerprint);
    let include = artifact.join("include");
    let lib = artifact.join("lib");
    fs::create_dir_all(&include).expect("failed to create managed-PCRE2 include fixture");
    fs::create_dir_all(&lib).expect("failed to create managed-PCRE2 library fixture");

    copy_nonempty(
        &test_pcre2_header_path("pcre2.h"),
        &include.join("pcre2.h"),
    );
    copy_nonempty(
        &test_pcre2_header_path("pcre2posix.h"),
        &include.join("pcre2posix.h"),
    );
    copy_nonempty(
        test_pcre2_shim_archive(),
        &lib.join("libelephc_pcre2_shim.a"),
    );
    copy_nonempty(
        &test_pcre2_static_archive_path("pcre2-posix"),
        &lib.join("libpcre2-posix.a"),
    );
    copy_nonempty(
        &test_pcre2_static_archive_path("pcre2-8"),
        &lib.join("libpcre2-8.a"),
    );

    let output_paths = [
        "include/pcre2.h",
        "include/pcre2posix.h",
        "lib/libelephc_pcre2_shim.a",
        "lib/libpcre2-posix.a",
        "lib/libpcre2-8.a",
    ];
    let outputs = output_paths
        .iter()
        .map(|relative| {
            let path = artifact.join(relative);
            let metadata = fs::metadata(&path).expect("failed to inspect managed-PCRE2 output");
            json!({
                "path": relative,
                "size": metadata.len(),
                "sha256": sha256_file(&path),
            })
        })
        .collect::<Vec<_>>();
    let receipt = json!({
        "schema": 1,
        "package": "pcre2",
        "version": PCRE2_VERSION,
        "recipe": PCRE2_RECIPE,
        "source_sha256": PCRE2_SOURCE_SHA256,
        "target": toolchain.target,
        "abi": toolchain.abi,
        "compiler": {
            "command": toolchain.compiler_command,
            "version": toolchain.compiler_version,
        },
        "archiver": {
            "command": toolchain.archiver_command,
            "version": toolchain.archiver_version,
        },
        "ranlib": {
            "command": toolchain.ranlib_command,
            "version": toolchain.ranlib_version,
        },
        "toolchain_fingerprint": toolchain.fingerprint,
        "outputs": outputs,
        "created_by": "elephc-codegen-test-provider",
    });
    let mut encoded =
        serde_json::to_vec_pretty(&receipt).expect("failed to encode managed-PCRE2 receipt");
    encoded.push(b'\n');
    fs::write(artifact.join("receipt.json"), encoded)
        .expect("failed to write managed-PCRE2 receipt");
    cache
}

/// Resolves the host tools and reproduces the production fingerprint payload.
fn resolve_test_native_toolchain() -> TestNativeToolchain {
    let target = Target::detect_host();
    let suffix = target
        .as_str()
        .replace('-', "_")
        .to_ascii_uppercase();
    let cc = selected_native_tool("CC", &suffix);
    let ar = selected_native_tool("AR", &suffix);
    let ranlib = selected_native_tool("RANLIB", &suffix);
    let tuple = normalized_success(
        Command::new(&cc).arg("-dumpmachine").output(),
        "query native test compiler tuple",
    );
    let abi = native_test_abi(target, &tuple);
    let cc_version = normalized_success(
        Command::new(&cc).arg("--version").output(),
        "query native test compiler version",
    );
    let ar_version = native_tool_version(&ar, "archiver");
    let ranlib_version = native_tool_version(&ranlib, "ranlib");
    let sdk = if target.platform == Platform::MacOS {
        normalized_success(
            Command::new("xcrun")
                .args(["--sdk", "macosx", "--show-sdk-version"])
                .output(),
            "query native test SDK",
        )
    } else {
        String::new()
    };
    let cc_name = cc.to_string_lossy().into_owned();
    let ar_name = ar.to_string_lossy().into_owned();
    let ranlib_name = ranlib.to_string_lossy().into_owned();
    let environment = fingerprinted_test_environment();
    let payload = format!(
        "target={}\nabi={}\ntuple={}\ncc={}\ncc-version={}\nar={}\nar-version={}\nranlib={}\nranlib-version={}\nsdk={}\nCFLAGS=-fPIC\n{}",
        target.as_str(),
        abi,
        tuple,
        cc_name,
        cc_version,
        ar_name,
        ar_version,
        ranlib_name,
        ranlib_version,
        sdk,
        environment
    );
    TestNativeToolchain {
        target: target.as_str().to_string(),
        abi,
        fingerprint: sha256_bytes(payload.as_bytes()),
        compiler_command: cc_name,
        compiler_version: cc_version,
        archiver_command: ar_name,
        archiver_version: ar_version,
        ranlib_command: ranlib_name,
        ranlib_version,
    }
}

/// Applies the production target-specific, generic, then conventional tool precedence.
fn selected_native_tool(tool: &str, suffix: &str) -> PathBuf {
    for name in [
        format!("ELEPHC_NATIVE_{tool}_{suffix}"),
        format!("ELEPHC_NATIVE_{tool}"),
    ] {
        if let Some(value) = std::env::var_os(&name) {
            assert!(!value.is_empty(), "{name} must not be empty");
            return PathBuf::from(value);
        }
    }
    PathBuf::from(tool.to_ascii_lowercase())
}

/// Normalizes the host tuple into the ABI directory used by native resolution.
fn native_test_abi(target: Target, tuple: &str) -> String {
    let arch = match target.arch {
        Arch::AArch64 => "aarch64",
        Arch::X86_64 => "x86_64",
    };
    match target.platform {
        Platform::MacOS => format!("{arch}-apple-darwin"),
        Platform::Linux => {
            let lower = tuple.to_ascii_lowercase();
            let environment = if lower.contains("musl") {
                "musl"
            } else if lower.contains("gnu") {
                "gnu"
            } else if cfg!(target_env = "musl") {
                "musl"
            } else {
                "gnu"
            };
            format!("{arch}-unknown-linux-{environment}")
        }
        Platform::Windows => panic!("managed native fixtures do not support Windows"),
    }
}

/// Returns a normalized tool version using the production flag and hash fallback order.
fn native_tool_version(command: &Path, label: &str) -> String {
    for flag in ["--version", "-V"] {
        if let Ok(output) = Command::new(command).arg(flag).output() {
            if output.status.success() {
                let normalized = normalized_output(output);
                if !normalized.is_empty() {
                    return normalized;
                }
            }
        }
    }
    let executable = resolve_test_executable(command)
        .unwrap_or_else(|| panic!("cannot resolve native test {label} '{}'", command.display()));
    format!("binary-sha256={}", sha256_file(&executable))
}

/// Resolves a bare tool through PATH or canonicalizes an explicit executable path.
fn resolve_test_executable(command: &Path) -> Option<PathBuf> {
    if command.components().count() > 1 {
        return fs::canonicalize(command).ok();
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|directory| {
        let candidate = directory.join(command);
        fs::metadata(&candidate)
            .is_ok_and(|metadata| metadata.is_file())
            .then(|| fs::canonicalize(candidate).ok())
            .flatten()
    })
}

/// Requires a successful tool probe and returns its normalized output.
fn normalized_success(
    output: std::io::Result<Output>,
    action: &str,
) -> String {
    let output = output.unwrap_or_else(|error| panic!("{action}: {error}"));
    assert!(
        output.status.success(),
        "{action} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    normalized_output(output)
}

/// Collapses a tool's stdout or stderr whitespace exactly like the production resolver.
fn normalized_output(output: Output) -> String {
    let bytes = if output.stdout.is_empty() {
        output.stderr
    } else {
        output.stdout
    };
    String::from_utf8_lossy(&bytes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Encodes the inherited allowlisted environment portion of the native fingerprint.
fn fingerprinted_test_environment() -> String {
    let mut output = String::new();
    for name in ["PATH", "TMPDIR", "TMP", "TEMP", "SYSTEMROOT"] {
        let value = std::env::var_os(name)
            .unwrap_or_else(OsString::new)
            .to_string_lossy()
            .into_owned();
        output.push_str(&format!("{name}={value}\n"));
    }
    output.push_str("LC_ALL=C\nLANG=C\n");
    output
}

/// Copies one fixture file and rejects missing or empty provider outputs.
fn copy_nonempty(source: &Path, destination: &Path) {
    let metadata = fs::metadata(source)
        .unwrap_or_else(|error| panic!("missing native test provider '{}': {error}", source.display()));
    assert!(
        metadata.is_file() && metadata.len() > 0,
        "native test provider '{}' must be a non-empty file",
        source.display()
    );
    fs::copy(source, destination).unwrap_or_else(|error| {
        panic!(
            "failed to copy native test provider '{}' to '{}': {error}",
            source.display(),
            destination.display()
        )
    });
}

/// Computes a lowercase SHA-256 digest for one fixture output.
fn sha256_file(path: &Path) -> String {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("failed to hash '{}': {error}", path.display()));
    sha256_bytes(&bytes)
}

/// Computes a lowercase SHA-256 digest for an in-memory fingerprint payload.
fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
