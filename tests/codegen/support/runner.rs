//! Purpose:
//! Native binary runner helpers for assembling runtimes, linking objects, and executing codegen fixtures.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Handles platform-specific linker flags, qemu ARM64 execution, and runtime object caching.
//! - Per-test assembly is fed to `as` over stdin so no intermediate `test.s`
//!   file is written, which shaves ~1/3 of the file-system events the macOS
//!   `syspolicyd` / on-access AV scans inspect during a full `cargo test`.

use std::io::{Read as _, Write as _};
use std::process::{Output, Stdio};
use std::time::{Duration, Instant};

use super::*;

/// Describes a Rust bridge staticlib needed by codegen integration fixtures.
struct TestBridgeStaticlib {
    /// Linker library name requested by the compiled program.
    lib_name: &'static str,
    /// Cargo package that produces `lib<lib_name>.a` for tests.
    package: &'static str,
}

/// Lists bridge staticlibs that codegen fixtures may link through `extra_link_libs`.
const TEST_BRIDGE_STATICLIBS: &[TestBridgeStaticlib] = &[
    TestBridgeStaticlib {
        lib_name: "elephc_tls",
        package: "elephc-tls",
    },
    TestBridgeStaticlib {
        lib_name: "elephc_pdo",
        package: "elephc-pdo",
    },
    TestBridgeStaticlib {
        lib_name: "elephc_crypto",
        package: "elephc-crypto",
    },
    TestBridgeStaticlib {
        lib_name: "elephc_phar",
        package: "elephc-phar",
    },
    TestBridgeStaticlib {
        lib_name: "elephc_tz",
        package: "elephc-tz",
    },
    TestBridgeStaticlib {
        lib_name: "elephc_image",
        package: "elephc-image",
    },
];

/// Default timeout for executing one compiled codegen fixture binary.
const DEFAULT_BINARY_TIMEOUT_SECS: u64 = 60;

/// Assemble `asm` to `obj_path` by piping the source through `as`'s stdin so
/// no intermediate `.s` file is created.
fn assemble_from_stdin(asm: &str, obj_path: &Path) {
    ensure_windows_runnable_or_skip();
    // Rewrite the shared x86_64 backend's raw Linux syscalls into windows shim
    // calls before assembling; a no-op on native targets, so their bytes are
    // unchanged (the borrowed `asm` is fed straight through).
    let windows_asm;
    let asm = if target().platform == Platform::Windows {
        windows_asm = finalize_asm_for_target(asm);
        windows_asm.as_str()
    } else {
        asm
    };
    let mut cmd = Command::new(assembler_cmd());
    if target().platform == Platform::MacOS {
        cmd.args(["-arch", target().darwin_arch_name()]);
    }
    cmd.arg("-o").arg(obj_path).arg("-");
    cmd.stdin(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn assembler");
    child
        .stdin
        .as_mut()
        .expect("assembler stdin missing")
        .write_all(asm.as_bytes())
        .expect("failed to feed assembler");
    let status = child.wait().expect("failed to wait for assembler");
    assert!(status.success(), "assembler failed");
}

/// Returns the cached base runtime object path, assembling the runtime on first call.
/// Creates a temp directory, generates an 8_388_608-byte heap runtime without optional
/// feature families, assembles it with `as`, and caches the `.o` path for legacy tests.
pub(crate) fn get_runtime_obj() -> &'static Path {
    RUNTIME_OBJ.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("elephc_test_runtime_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let runtime_asm = elephc::codegen::generate_runtime_with_features(
            8_388_608,
            target(),
            elephc::codegen::RuntimeFeatures::none(),
        );
        let obj_path = dir.join("runtime.o");
        assemble_from_stdin(&runtime_asm, &obj_path);
        obj_path
    })
}

/// Returns a cached runtime object assembled from the exact runtime assembly string.
///
/// The cache key is an FNV-1a hash of the full runtime assembly, so feature-gated
/// runtimes and custom heap sizes get distinct objects while repeated tests can
/// still share the assembled output.
pub(crate) fn runtime_obj_for_asm(runtime_asm: &str) -> std::path::PathBuf {
    ensure_windows_runnable_or_skip();
    // Key the cache on the untransformed runtime assembly: the windows rewrite is
    // deterministic, so identical raw assembly still shares one assembled object.
    let hash = runtime_asm_hash(runtime_asm);
    let cache = RUNTIME_OBJS_BY_ASM.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let mut cache = cache.lock().expect("runtime asm cache poisoned");
    if let Some(path) = cache.get(&hash) {
        return path.clone();
    }

    let dir = std::env::temp_dir().join(format!("elephc_test_runtime_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let asm_path = dir.join(format!("runtime_{hash:016x}.s"));
    let obj_path = dir.join(format!("runtime_{hash:016x}.o"));
    // Apply the windows syscall→shim rewrite before writing/assembling; a no-op on
    // native targets, so the runtime bytes are unchanged there.
    let windows_asm;
    let runtime_asm = if target().platform == Platform::Windows {
        windows_asm = finalize_asm_for_target(runtime_asm);
        windows_asm.as_str()
    } else {
        runtime_asm
    };
    fs::write(&asm_path, runtime_asm).unwrap();

    let mut cmd = Command::new(assembler_cmd());
    if target().platform == Platform::MacOS {
        cmd.args(["-arch", target().darwin_arch_name()]);
    }
    cmd.arg("-o").arg(&obj_path).arg(&asm_path);
    let status = cmd.status().expect("failed to assemble feature runtime");
    assert!(status.success(), "feature runtime assembler failed");
    cache.insert(hash, obj_path.clone());
    obj_path
}

/// Computes a stable FNV-1a hash for runtime assembly cache keys.
fn runtime_asm_hash(asm: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in asm.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Assembles a base runtime object with a custom heap size, writing the `.o` to `dir`.
/// Generates ARM64/x86_64 runtime assembly without optional feature families and assembles
/// it using `assembler_cmd()`. Used by tests that need non-default heap sizes.
pub(crate) fn assemble_custom_runtime(heap_size: usize, dir: &Path) -> std::path::PathBuf {
    let runtime_asm = elephc::codegen::generate_runtime_with_features(
        heap_size,
        target(),
        elephc::codegen::RuntimeFeatures::none(),
    );
    let obj_path = dir.join("runtime.o");
    assemble_from_stdin(&runtime_asm, &obj_path);
    obj_path
}

/// Returns the bridge staticlibs requested by a fixture's effective link libraries.
fn requested_bridge_staticlibs<'a>(actual_link_libs: &[&str]) -> Vec<&'a TestBridgeStaticlib> {
    TEST_BRIDGE_STATICLIBS
        .iter()
        .filter(|bridge| actual_link_libs.iter().any(|lib| *lib == bridge.lib_name))
        .collect()
}

/// Builds any requested bridge staticlibs missing from the target's debug
/// directory.
///
/// On the windows-x86_64 test target the bridge staticlibs must be cross-built
/// for `x86_64-pc-windows-gnu` (PE/COFF) so MinGW can link them into the `.exe`;
/// MinGW cannot link host ELF archives into a PE binary. To that end the
/// `cargo build -p <package>` command gains `--target x86_64-pc-windows-gnu` and
/// the `CC_x86_64_pc_windows_gnu`/`AR_x86_64_pc_windows_gnu`/`RANLIB_*` env vars
/// that cc-rs needs to compile bundled C (PDO's `libsqlite3-sys` amalgamation)
/// with the MinGW toolchain. On every other target the command is byte-identical
/// to the pre-Tier-2 path: no `--target`, no extra env, so macOS/Linux bridge
/// builds are unchanged.
fn ensure_bridge_staticlibs(actual_link_libs: &[&str], bridge_staticlib_dir: &str) {
    let _guard = BRIDGE_STATICLIB_BUILD_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("bridge staticlib build lock poisoned");
    let platform = target().platform;
    let cargo_target = bridge_staticlib_cargo_target(platform);
    for bridge in requested_bridge_staticlibs(actual_link_libs) {
        let archive_path =
            Path::new(bridge_staticlib_dir).join(format!("lib{}.a", bridge.lib_name));
        if !bridge_staticlib_needs_build(&archive_path, bridge.package) {
            continue;
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["build", "-p", bridge.package]);
        if let Some(triple) = cargo_target {
            cmd.args(["--target", triple]);
            for (key, value) in bridge_staticlib_cross_env(platform) {
                cmd.env(key, value);
            }
        }
        let status = cmd
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .unwrap_or_else(|err| {
                panic!(
                    "failed to run cargo build for bridge staticlib {}: {}",
                    bridge.package, err
                )
            });
        assert!(
            status.success(),
            "failed to build bridge staticlib {}",
            bridge.package
        );
        assert!(
            archive_path.exists(),
            "bridge staticlib {} was built but {} is still missing",
            bridge.package,
            archive_path.display()
        );
    }
}

/// Returns the `cargo build --target` triple the bridge staticlib build should
/// target, or `None` when building for the host. Only the windows-x86_64 test
/// target cross-compiles the bridges; every other target returns `None` so the
/// `cargo build` command stays byte-identical to the pre-Tier-2 path. Split out
/// as a pure helper so the gating can be unit-tested without mutating process
/// environment (which is racy under parallel test execution).
fn bridge_staticlib_cargo_target(platform: Platform) -> Option<&'static str> {
    match platform {
        Platform::Windows => Some("x86_64-pc-windows-gnu"),
        _ => None,
    }
}

/// Returns the `CC`/`AR`/`RANLIB` env vars cc-rs needs to compile bundled C
/// (PDO's `libsqlite3-sys` amalgamation) for the windows-x86_64 cross target
/// using the MinGW-w64 toolchain. Empty slice on non-Windows targets, so no env
/// is injected into the host `cargo build` command there.
fn bridge_staticlib_cross_env(platform: Platform) -> &'static [(&'static str, &'static str)] {
    static WINDOWS: [(&str, &str); 3] = [
        ("CC_x86_64_pc_windows_gnu", "x86_64-w64-mingw32-gcc"),
        ("AR_x86_64_pc_windows_gnu", "x86_64-w64-mingw32-ar"),
        ("RANLIB_x86_64_pc_windows_gnu", "x86_64-w64-mingw32-ranlib"),
    ];
    static OTHER: [(&str, &str); 0] = [];
    match platform {
        Platform::Windows => &WINDOWS,
        _ => &OTHER,
    }
}

/// Returns the subdirectory under the cargo target dir where bridge staticlibs
/// land. For the windows-x86_64 cross target, `cargo build --target
/// x86_64-pc-windows-gnu` emits archives under
/// `<target>/x86_64-pc-windows-gnu/debug`, so MinGW finds the PE/COFF archives
/// there; every other target uses `<target>/debug` (the host cargo output dir),
/// preserving the pre-Tier-2 layout on macOS/Linux. Pure helper so the dir
/// resolution can be unit-tested without env mutation.
fn bridge_staticlib_subdir(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "x86_64-pc-windows-gnu/debug",
        _ => "debug",
    }
}

/// Reports whether a bridge staticlib is missing or older than its package
/// sources. This keeps codegen tests from linking stale bridge archives after a
/// bridge crate changes inside the same worktree.
fn bridge_staticlib_needs_build(archive_path: &Path, package: &str) -> bool {
    let archive_mtime = match archive_path.metadata().and_then(|meta| meta.modified()) {
        Ok(mtime) => mtime,
        Err(_) => return true,
    };
    let package_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates")
        .join(package);

    source_path_newer_than(&package_dir.join("Cargo.toml"), archive_mtime)
        || source_path_newer_than(&package_dir.join("build.rs"), archive_mtime)
        || source_tree_newer_than(&package_dir.join("src"), archive_mtime)
}

/// Reports whether an existing source path was modified after `archive_mtime`.
/// Missing optional files such as `build.rs` do not force a rebuild.
fn source_path_newer_than(path: &Path, archive_mtime: std::time::SystemTime) -> bool {
    match path.metadata().and_then(|meta| meta.modified()) {
        Ok(source_mtime) => source_mtime > archive_mtime,
        Err(_) => false,
    }
}

/// Recursively scans a bridge package source directory for files newer than the
/// compiled staticlib. Directory-read failures are treated as stale so tests do
/// not silently link an archive whose source state could not be inspected.
fn source_tree_newer_than(dir: &Path, archive_mtime: std::time::SystemTime) -> bool {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return true,
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => return true,
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => return true,
        };
        if file_type.is_dir() {
            if source_tree_newer_than(&path, archive_mtime) {
                return true;
            }
        } else if file_type.is_file() && source_path_newer_than(&path, archive_mtime) {
            return true;
        }
    }
    false
}

/// Links a user object file and a runtime object into a final native binary.
/// On macOS uses `ld` with SDK/platform_version flags; on Linux uses `gcc` with
/// static linking when no extra libs are needed. Adds `-lm -lpthread` on Linux.
pub(crate) fn link_binary(
    obj_path: &Path,
    runtime_obj: &Path,
    bin_path: &Path,
    extra_link_libs: &[String],
    extra_link_paths: &[String],
    extra_frameworks: &[String],
) {
    let actual_link_libs = effective_link_libs(extra_link_libs);

    // The bridge staticlibs (elephc-tls, elephc-pdo, elephc-crypto, elephc-phar,
    // elephc-tz, elephc-image) all live in `<target>/debug` alongside the test
    // binaries; surface that directory on the linker search path automatically
    // whenever a compiled program links any bridge, so PDO/crypto/phar/tz/image
    // tests get the same robust, absolute `-L` as TLS instead of depending on a
    // cwd-relative lookup. The Docker scripts override CARGO_TARGET_DIR to point at
    // a shared volume, so honour that envvar before falling back to the in-tree
    // target/.
    let needs_bridge_staticlib = actual_link_libs.iter().any(|l| {
        *l == "elephc_tls"
            || *l == "elephc_pdo"
            || *l == "elephc_crypto"
            || *l == "elephc_phar"
            || *l == "elephc_tz"
            || *l == "elephc_image"
    });
    let bridge_staticlib_dir = match std::env::var("CARGO_TARGET_DIR") {
        Ok(dir) if !dir.is_empty() => {
            format!("{}/{}", dir, bridge_staticlib_subdir(target().platform))
        }
        _ => format!(
            "{}/target/{}",
            env!("CARGO_MANIFEST_DIR"),
            bridge_staticlib_subdir(target().platform)
        ),
    };
    if needs_bridge_staticlib {
        ensure_bridge_staticlibs(&actual_link_libs, &bridge_staticlib_dir);
    }

    match target().platform {
        Platform::MacOS => {
            let mut ld_cmd = Command::new("ld");
            ld_cmd.args(["-arch", target().darwin_arch_name(), "-e", "_main", "-o"]);
            ld_cmd.arg(bin_path);
            ld_cmd.arg(obj_path);
            ld_cmd.arg(runtime_obj);
            ld_cmd.args(["-lSystem", "-syslibroot"]);
            ld_cmd.arg(get_sdk_path());
            ld_cmd.args([
                "-platform_version",
                "macos",
                get_sdk_version(),
                get_sdk_version(),
            ]);
            if needs_bridge_staticlib {
                ld_cmd.arg(format!("-L{}", bridge_staticlib_dir));
            }
            for path in extra_link_paths {
                ld_cmd.arg(format!("-L{}", path));
            }
            for lib in &actual_link_libs {
                ld_cmd.arg(format!("-l{}", lib));
            }
            for framework in extra_frameworks {
                ld_cmd.args(["-framework", framework]);
            }
            // The PostgreSQL driver in the PDO bridge pulls in `whoami`, which
            // references CoreFoundation / SystemConfiguration on macOS.
            if actual_link_libs.iter().any(|lib| *lib == "elephc_pdo") {
                ld_cmd.args(["-framework", "CoreFoundation"]);
                ld_cmd.args(["-framework", "SystemConfiguration"]);
            }
            let ld_out = ld_cmd.output().expect("failed to run linker");
            assert!(
                ld_out.status.success(),
                "linker failed:\n{}",
                String::from_utf8_lossy(&ld_out.stderr)
            );
        }
        Platform::Linux => {
            let mut ld_cmd = Command::new(gcc_cmd());
            ld_cmd.arg("-o").arg(bin_path);
            ld_cmd.arg(obj_path);
            ld_cmd.arg(runtime_obj);
            if actual_link_libs.is_empty() {
                ld_cmd.arg("-static");
            }
            if !actual_link_libs.is_empty() {
                ld_cmd.arg("-Wl,--no-as-needed");
            }
            if needs_bridge_staticlib {
                ld_cmd.arg(format!("-L{}", bridge_staticlib_dir));
            }
            for path in extra_link_paths {
                ld_cmd.arg(format!("-L{}", path));
            }
            for lib in &actual_link_libs {
                ld_cmd.arg(format!("-l{}", lib));
            }
            if !actual_link_libs.is_empty() {
                ld_cmd.arg("-Wl,--as-needed");
            }
            // Math and POSIX regex libraries needed on Linux
            ld_cmd.args(["-lm", "-lpthread"]);
            // Rust bridge staticlibs pull in the dynamic loader for the libc
            // unwinder on Linux.
            if needs_bridge_staticlib {
                ld_cmd.arg("-ldl");
            }
            let ld_out = ld_cmd.output().expect("failed to run linker");
            assert!(
                ld_out.status.success(),
                "linker failed:\n{}",
                String::from_utf8_lossy(&ld_out.stderr)
            );
        }
        Platform::Windows => {
            // MinGW GCC (`x86_64-w64-mingw32-gcc`) links the user + runtime objects
            // into a PE32+ `.exe`, mirroring the production windows arm in
            // `src/linker.rs`. The `.exe` suffix matches what MinGW emits and what
            // the Wine runner then executes.
            let mut ld_cmd = Command::new(gcc_cmd());
            ld_cmd.arg("-o").arg(target_binary_path(bin_path));
            ld_cmd.arg(obj_path);
            ld_cmd.arg(runtime_obj);
            // Surface the CI MinGW sysroot (cross-built PCRE2/bzip2/zlib/iconv)
            // before any `-l` args so MinGW resolves those C symbols. Mirrors the
            // production arm in `src/linker.rs`; gated on the env var so local
            // non-CI runs are unaffected.
            for path in mingw_sysroot_link_paths() {
                ld_cmd.arg(format!("-L{}", path));
            }
            if needs_bridge_staticlib {
                ld_cmd.arg(format!("-L{}", bridge_staticlib_dir));
            }
            for path in extra_link_paths {
                ld_cmd.arg(format!("-L{}", path));
            }
            for lib in &actual_link_libs {
                ld_cmd.arg(format!("-l{}", lib));
            }
            // Windows system import libraries the runtime shims resolve against
            // (WriteFile/ReadFile/HeapAlloc/BCryptGenRandom/...); same set as the
            // production linker.
            ld_cmd.args([
                "-lkernel32",
                "-lmsvcrt",
                "-lwinmm",
                "-lws2_32",
                "-lbcrypt",
                "-lshlwapi",
            ]);
            let ld_out = ld_cmd.output().expect("failed to run linker");
            assert!(
                ld_out.status.success(),
                "linker failed:\n{}",
                String::from_utf8_lossy(&ld_out.stderr)
            );
        }
    }
}

/// Returns the `-L` search paths derived from the `ELEPHC_MINGW_SYSROOT` env
/// var, mirroring `src/linker.rs::mingw_sysroot_link_paths` for the test
/// harness. CI sets the env var to a cross-built MinGW sysroot (PCRE2, bzip2,
/// zlib, libiconv) so the MinGW linker resolves the C symbols those codegen
/// fixtures call. Unset on local non-CI runs, so no missing-directory warnings.
fn mingw_sysroot_link_paths() -> Vec<String> {
    let Some(dir) = std::env::var_os("ELEPHC_MINGW_SYSROOT") else {
        return Vec::new();
    };
    let base = std::path::PathBuf::from(dir);
    if !base.is_dir() {
        return Vec::new();
    }
    let mut paths = Vec::new();
    let lib = base.join("lib");
    if lib.is_dir() {
        paths.push(lib.to_string_lossy().into_owned());
    }
    let lib64 = base.join("lib64");
    if lib64.is_dir() {
        paths.push(lib64.to_string_lossy().into_owned());
    }
    paths
}

/// Returns the on-disk path of the compiled binary for the current target. For
/// windows-x86_64 this is `<bin>.exe` (MinGW emits a `.exe` and Wine runs it);
/// every other target uses the bare binary path unchanged.
fn target_binary_path(bin_path: &Path) -> std::path::PathBuf {
    if target().platform == Platform::Windows {
        bin_path.with_extension("exe")
    } else {
        bin_path.to_path_buf()
    }
}

/// Builds the base `Command` that executes a compiled codegen fixture for the
/// current target: a direct exec on the host, `qemu-aarch64-static` when running
/// ARM64 binaries on an x86_64 host, or Wine running the `.exe` for the
/// windows-x86_64 target. The caller sets the working directory and wires
/// args/stdin/stdout as needed. Centralizing run-dispatch here keeps every runner
/// path (plain, capture, stdin) on the same target-correct launcher, and leaves the
/// native/qemu commands byte-identical to the previous inline logic.
pub(crate) fn build_run_command(bin_path: &Path) -> Command {
    match target().platform {
        Platform::Windows => {
            let mut cmd = Command::new(wine_binary());
            cmd.arg(target_binary_path(bin_path));
            // Silence Wine's diagnostic chatter so it never pollutes captured stdout.
            cmd.env("WINEDEBUG", "-all");
            cmd
        }
        Platform::Linux if target().arch == Arch::AArch64 && cfg!(target_arch = "x86_64") => {
            let mut cmd = Command::new("qemu-aarch64-static");
            if let Some(sysroot) = qemu_sysroot() {
                cmd.args(["-L", sysroot]);
            }
            cmd.arg(bin_path);
            cmd
        }
        _ => Command::new(bin_path),
    }
}

/// Runs a compiled binary for the current target, capturing stdout/stderr under a
/// timeout. Uses qemu to emulate ARM64 on an x86_64 host and Wine to run the `.exe`
/// on the windows-x86_64 target; execs natively otherwise. Used for post-link
/// execution of already-assembled test binaries.
pub(crate) fn run_binary(bin_path: &Path, dir: &Path) -> Output {
    ensure_windows_runnable_or_skip();
    let mut cmd = build_run_command(bin_path);
    cmd.current_dir(dir);
    run_command_with_timeout(cmd)
}

/// Runs a child command with a timeout and captures stdout/stderr.
fn run_command_with_timeout(mut cmd: Command) -> Output {
    let label = format!("{:?}", cmd);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|err| panic!("failed to run compiled binary {label}: {err}"));
    let mut stdout_pipe = child.stdout.take().expect("compiled binary stdout missing");
    let mut stderr_pipe = child.stderr.take().expect("compiled binary stderr missing");
    let stdout_reader = std::thread::spawn(move || {
        let mut stdout = Vec::new();
        stdout_pipe
            .read_to_end(&mut stdout)
            .expect("failed to read compiled binary stdout");
        stdout
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut stderr = Vec::new();
        stderr_pipe
            .read_to_end(&mut stderr)
            .expect("failed to read compiled binary stderr");
        stderr
    });

    let timeout = codegen_binary_timeout();
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child
            .try_wait()
            .unwrap_or_else(|err| panic!("failed to wait for compiled binary {label}: {err}"))
        {
            let stdout = stdout_reader.join().expect("stdout reader panicked");
            let stderr = stderr_reader.join().expect("stderr reader panicked");
            return Output {
                status,
                stdout,
                stderr,
            };
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            let stdout = stdout_reader.join().expect("stdout reader panicked");
            let stderr = stderr_reader.join().expect("stderr reader panicked");
            panic!(
                "compiled binary timed out after {}s: {}\nstdout:\n{}\nstderr:\n{}",
                timeout.as_secs(),
                label,
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Returns the per-binary timeout for codegen fixtures.
fn codegen_binary_timeout() -> Duration {
    std::env::var("ELEPHC_TEST_BINARY_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(DEFAULT_BINARY_TIMEOUT_SECS))
}

/// Assembles user assembly, links it with a runtime object, runs the binary,
/// and returns stdout. Asserts the binary exits successfully. Used for happy-path codegen tests.
pub(crate) fn assemble_and_run(
    user_asm: &str,
    runtime_obj: &Path,
    dir: &Path,
    extra_link_libs: &[String],
    extra_link_paths: &[String],
    extra_frameworks: &[String],
) -> String {
    let obj_path = dir.join("test.o");
    let bin_path = dir.join("test");

    assemble_from_stdin(user_asm, &obj_path);

    link_binary(
        &obj_path,
        runtime_obj,
        &bin_path,
        extra_link_libs,
        extra_link_paths,
        extra_frameworks,
    );

    let output = run_binary(&bin_path, dir);
    assert!(
        output.status.success(),
        "binary exited with error: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap()
}

// Captures stdout and stderr from a compiled binary, along with its exit status.
// Used by tests that need to inspect both output streams without asserting success,
// or by error/regression tests that need to validate stderr without requiring exit failure.
pub(crate) struct ProgramOutput {
    // Raw stdout bytes decoded as UTF-8.
    pub(crate) stdout: String,
    // Raw stderr bytes decoded as UTF-8.
    pub(crate) stderr: String,
    // true if the process exited with a successful (zero) exit code.
    pub(crate) success: bool,
}

/// Assembles user assembly, links it with a runtime object, runs the binary,
/// and captures stdout, stderr, and exit status. Asserts the binary exits successfully.
pub(crate) fn assemble_and_run_capture(
    user_asm: &str,
    runtime_obj: &Path,
    dir: &Path,
    extra_link_libs: &[String],
    extra_link_paths: &[String],
    extra_frameworks: &[String],
) -> ProgramOutput {
    let obj_path = dir.join("test.o");
    let bin_path = dir.join("test");

    assemble_from_stdin(user_asm, &obj_path);

    link_binary(
        &obj_path,
        runtime_obj,
        &bin_path,
        extra_link_libs,
        extra_link_paths,
        extra_frameworks,
    );

    let output = run_binary(&bin_path, dir);

    ProgramOutput {
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
        success: output.status.success(),
    }
}

/// Assembles user assembly, links it with a runtime object, runs the binary,
/// and returns stderr. Asserts the binary exits with failure. Used for error/regression tests.
pub(crate) fn assemble_and_run_expect_failure(
    user_asm: &str,
    runtime_obj: &Path,
    dir: &Path,
    extra_link_libs: &[String],
    extra_link_paths: &[String],
    extra_frameworks: &[String],
) -> String {
    let obj_path = dir.join("test.o");
    let bin_path = dir.join("test");

    assemble_from_stdin(user_asm, &obj_path);

    link_binary(
        &obj_path,
        runtime_obj,
        &bin_path,
        extra_link_libs,
        extra_link_paths,
        extra_frameworks,
    );

    let output = run_binary(&bin_path, dir);
    assert!(!output.status.success(), "binary unexpectedly succeeded");

    String::from_utf8(output.stderr).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the windows-x86_64 target cross-compiles bridge staticlibs for
    /// `x86_64-pc-windows-gnu` so MinGW can link the PE/COFF archives, while
    /// macOS/Linux build for the host (no `--target`) and stay on the pre-Tier-2
    /// path.
    #[test]
    fn bridge_staticlib_cargo_target_gated_on_windows() {
        assert_eq!(
            bridge_staticlib_cargo_target(Platform::Windows),
            Some("x86_64-pc-windows-gnu")
        );
        assert_eq!(bridge_staticlib_cargo_target(Platform::MacOS), None);
        assert_eq!(bridge_staticlib_cargo_target(Platform::Linux), None);
    }

    /// Verifies the cc-rs `CC`/`AR`/`RANLIB` env vars are surfaced only for the
    /// windows-x86_64 cross target, so PDO's bundled `libsqlite3-sys` amalgamation
    /// is compiled by `x86_64-w64-mingw32-gcc` and not the host cc. Non-Windows
    /// targets get an empty slice so the host `cargo build` command is unchanged.
    #[test]
    fn bridge_staticlib_cross_env_only_on_windows() {
        let env = bridge_staticlib_cross_env(Platform::Windows);
        assert_eq!(env.len(), 3);
        assert!(env.iter().any(|(k, v)| *k == "CC_x86_64_pc_windows_gnu"
            && *v == "x86_64-w64-mingw32-gcc"));
        assert!(env.iter().any(|(k, v)| *k == "AR_x86_64_pc_windows_gnu"
            && *v == "x86_64-w64-mingw32-ar"));
        assert!(env.iter().any(|(k, v)| *k == "RANLIB_x86_64_pc_windows_gnu"
            && *v == "x86_64-w64-mingw32-ranlib"));
        assert!(bridge_staticlib_cross_env(Platform::MacOS).is_empty());
        assert!(bridge_staticlib_cross_env(Platform::Linux).is_empty());
    }

    /// Verifies bridge staticlibs are looked up under
    /// `x86_64-pc-windows-gnu/debug` for the windows-x86_64 cross target (where
    /// `cargo build --target x86_64-pc-windows-gnu` emits archives) and under
    /// `debug` for every other target, preserving the pre-Tier-2 host layout on
    /// macOS/Linux.
    #[test]
    fn bridge_staticlib_subdir_gated_on_windows() {
        assert_eq!(
            bridge_staticlib_subdir(Platform::Windows),
            "x86_64-pc-windows-gnu/debug"
        );
        assert_eq!(bridge_staticlib_subdir(Platform::MacOS), "debug");
        assert_eq!(bridge_staticlib_subdir(Platform::Linux), "debug");
    }
}
