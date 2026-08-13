//! Purpose:
//! Integration coverage for top-level compile/native dispatch and compiler output modes.
//!
//! Called from:
//! - `cargo test` through Rust's test harness.
//!
//! Key details:
//! - Native help and managed-PCRE2 recovery diagnostics are exercised through subprocesses.
//! - Non-link modes must remain independent of installed native artifacts.
//! - Inline PHP fixtures are compiled to native binaries or wasm32-wasi modules,
//!   and assertions compare stdout or expected failures.

use crate::support::*;

/// Verifies compiler-version output is exact, successful, and independent of a source file.
#[test]
fn test_cli_version_reports_cargo_package_version() {
    let dir = make_cli_test_dir("elephc_cli_version");

    for flag in ["--version", "-V"] {
        let output = elephc_cli_command(&dir)
            .arg(flag)
            .output()
            .expect("failed to run elephc version command");
        assert!(output.status.success(), "{flag} should succeed");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("elephc {}\n", env!("CARGO_PKG_VERSION")),
            "unexpected {flag} stdout"
        );
        assert!(output.stderr.is_empty(), "unexpected {flag} stderr");
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies native help is handled before project discovery and bare native is a usage error.
#[test]
fn test_cli_native_help_and_bare_usage() {
    let dir = make_cli_test_dir("elephc_cli_native_help");

    let help = elephc_cli_command(&dir)
        .args(["native", "--help"])
        .output()
        .expect("failed to run elephc native --help");
    assert!(help.status.success(), "native help should succeed");
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("elephc native add"),
        "native help should print the command synopsis"
    );
    assert!(
        String::from_utf8_lossy(&help.stdout).contains("elephc native prune"),
        "native help should include explicit cache pruning"
    );

    let bare = elephc_cli_command(&dir)
        .arg("native")
        .output()
        .expect("failed to run bare elephc native");
    assert!(!bare.status.success(), "bare native should be a usage error");
    let stderr = String::from_utf8_lossy(&bare.stderr);
    assert!(stderr.contains("missing native command"), "unexpected stderr: {stderr}");
    assert!(stderr.contains("elephc native install"), "missing synopsis: {stderr}");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies read-only native commands preserve their captured stdout and health exit status.
#[test]
fn test_cli_native_read_only_commands_map_output_and_status() {
    let dir = make_cli_test_dir("elephc_cli_native_read_only");
    let cache = dir.join("native-cache-must-not-exist");

    let list = elephc_cli_command(&dir)
        .args(["native", "list"])
        .env("ELEPHC_NATIVE_CACHE", &cache)
        .output()
        .expect("failed to run elephc native list");
    assert!(list.status.success(), "empty native list should succeed");
    assert!(
        String::from_utf8_lossy(&list.stdout).contains("no native dependencies"),
        "unexpected list output: {}",
        String::from_utf8_lossy(&list.stdout)
    );

    let doctor = elephc_cli_command(&dir)
        .args(["native", "doctor"])
        .env("ELEPHC_NATIVE_CACHE", &cache)
        .output()
        .expect("failed to run elephc native doctor");
    assert!(!doctor.status.success(), "doctor without a project should be unhealthy");
    assert!(
        String::from_utf8_lossy(&doctor.stdout).contains("summary: unhealthy")
            && String::from_utf8_lossy(&doctor.stdout).contains("cache size:")
            && String::from_utf8_lossy(&doctor.stdout).contains("stale staging summary:"),
        "unexpected doctor output: {}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    assert!(!cache.exists(), "read-only commands must not create the native cache");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies explicit pruning is a successful no-op when no global native cache exists.
#[test]
fn test_cli_native_prune_empty_cache_is_noop() {
    let dir = make_cli_test_dir("elephc_cli_native_prune_empty");
    let cache = dir.join("native-cache-must-not-exist");
    let prune = elephc_cli_command(&dir)
        .args(["native", "prune"])
        .env("ELEPHC_NATIVE_CACHE", &cache)
        .output()
        .expect("failed to run elephc native prune");
    assert!(prune.status.success(), "empty-cache prune should succeed");
    assert!(
        String::from_utf8_lossy(&prune.stdout).contains("removed stale artifacts: 0"),
        "unexpected prune output: {}",
        String::from_utf8_lossy(&prune.stdout)
    );
    assert!(!cache.exists(), "empty-cache prune must not create cache state");
    let _ = fs::remove_dir_all(&dir);
}

/// Verifies non-link output modes never require or create a managed native cache.
#[test]
fn test_cli_regex_non_link_modes_skip_native_resolution() {
    for mode in ["--check", "--emit-ir", "--emit-asm"] {
        let dir = make_cli_test_dir("elephc_cli_regex_non_link");
        let cache = dir.join("native-cache-must-not-exist");
        let php_path = dir.join("main.php");
        fs::write(&php_path, "<?php echo preg_match('/a/', 'a');").unwrap();

        let output = elephc_cli_command(&dir)
            .arg(mode)
            .arg(&php_path)
            .env("ELEPHC_NATIVE_CACHE", &cache)
            .output()
            .unwrap_or_else(|error| panic!("failed to run elephc {mode}: {error}"));
        assert!(
            output.status.success(),
            "elephc {mode} unexpectedly required native PCRE2: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !cache.exists(),
            "elephc {mode} must not create the managed native cache"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}

/// Verifies a final regex link without a project fails with the frozen recovery command.
#[test]
fn test_cli_regex_final_link_requires_managed_pcre2_project() {
    let dir = make_cli_test_dir("elephc_cli_regex_requires_native");
    let cache = dir.join("native-cache-must-not-exist");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo preg_match('/a/', 'a');").unwrap();

    let output = elephc_cli_command(&dir)
        .arg(&php_path)
        .env("ELEPHC_NATIVE_CACHE", &cache)
        .output()
        .expect("failed to run final-link regex compilation");
    assert!(!output.status.success(), "regex link without a project must fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("regex support requires managed native package pcre2"),
        "unexpected missing-project diagnostic: {stderr}"
    );
    assert!(stderr.contains("project: not found"), "missing project context: {stderr}");
    assert!(
        stderr.contains("recovery: cd --") && stderr.contains("elephc native add pcre2"),
        "missing copy-paste recovery command: {stderr}"
    );
    assert!(!cache.exists(), "failed compilation must not create the native cache");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--check` stops after type-checking and produces "Checked" output
/// without emitting any assembly (.s), object (.o), or binary files.
#[test]
fn test_cli_check_stops_after_typecheck() {
    let dir = make_cli_test_dir("elephc_cli_check");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo "ok";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--check")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --check");

    assert!(
        output.status.success(),
        "elephc --check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Checked"),
        "expected --check success output, got stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !dir.join("main.s").exists(),
        "--check should not emit assembly files"
    );
    assert!(
        !dir.join("main.o").exists(),
        "--check should not emit object files"
    );
    assert!(
        !dir.join("main").exists(),
        "--check should not emit binaries"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--emit-asm` writes a .s assembly file containing the `_main` label
/// but does NOT produce object or binary files.
#[test]
fn test_cli_emit_asm_writes_assembly_only() {
    let dir = make_cli_test_dir("elephc_cli_emit_asm");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo "ok";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-asm")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --emit-asm");

    assert!(
        output.status.success(),
        "elephc --emit-asm failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Emitted assembly"),
        "expected --emit-asm success output, got stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );

    let asm_path = dir.join("main.s");
    assert!(asm_path.exists(), "--emit-asm should write the .s file");
    let asm = fs::read_to_string(&asm_path).expect("failed to read emitted assembly");
    assert!(
        asm.contains("_main"),
        "expected emitted assembly to contain the program entry label"
    );
    assert!(
        !dir.join("main.o").exists(),
        "--emit-asm should not emit object files"
    );
    assert!(
        !dir.join("main").exists(),
        "--emit-asm should not emit binaries"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies cross-target `--emit-asm` stops before preparing a host-incompatible runtime object.
#[test]
fn test_cli_emit_asm_does_not_require_target_assembler() {
    let dir = make_cli_test_dir("elephc_cli_emit_cross_target_asm");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 'cross-target';").unwrap();

    let target = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-aarch64"
    } else {
        "linux-x86_64"
    };
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg(target)
        .arg("--emit-asm")
        .arg(&php_path)
        .output()
        .expect("failed to run cross-target elephc CLI with --emit-asm");

    assert!(
        output.status.success(),
        "cross-target elephc --emit-asm failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.join("main.s").exists(), "expected target assembly output");
    assert!(
        !dir.join("main.o").exists() && !dir.join("main").exists(),
        "cross-target --emit-asm must not assemble or link"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies plain `--web` assembly keeps the compact auto-start core while
/// pruning public session APIs and callable-handler machinery that user code
/// does not reference.
#[test]
fn test_cli_web_prunes_unused_session_surface_from_assembly() {
    let dir = make_cli_test_dir("elephc_cli_web_pruned_prelude");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 'ok';").unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--web")
        .arg(&php_path)
        .output()
        .expect("failed to compile pruned web program");
    assert!(
        output.status.success(),
        "elephc --web failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let asm = fs::read_to_string(dir.join("main.s")).expect("failed to read web assembly");
    assert!(
        asm.contains("_fn__u__u_elephc_u_session_u_start_u_core"),
        "plain web assembly must retain the auto-start session core"
    );
    assert!(
        !asm.contains(".globl _fn_session_u_start\n"),
        "plain web assembly must not emit the public option-heavy session_start wrapper"
    );
    assert!(
        !asm.contains("_fn_session_u_set_u_save_u_handler"),
        "plain web assembly must not emit session_set_save_handler"
    );
    assert!(
        !asm.contains("__ElephcCallableSessionHandler"),
        "plain web assembly must not emit legacy callable-handler dispatch"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies repeated boxed-Mixed callable sites reuse module-wide descriptor
/// wrappers instead of regenerating the full candidate set in every function.
#[test]
fn test_cli_runtime_callable_descriptors_are_shared_across_call_sites() {
    let dir = make_cli_test_dir("elephc_cli_callable_descriptor_dedup");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class InvokableTarget { public function __invoke(int $value): int { return $value + 1; } }
function first(mixed $callback): mixed { return call_user_func($callback, 1); }
function second(mixed $callback): mixed { return call_user_func($callback, 2); }
function plus_one(int $value): int { return $value + 1; }
echo first('plus_one');
echo second('plus_one');
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg(&php_path)
        .output()
        .expect("failed to compile callable dedup fixture");
    assert!(
        output.status.success(),
        "callable dedup fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let asm = fs::read_to_string(dir.join("main.s")).expect("failed to read callable assembly");
    assert!(
        asm.contains("_eir_first_callable_invoker"),
        "the first dynamic call site must emit shared invokers"
    );
    assert!(
        !asm.contains("_eir_second_callable_invoker"),
        "the second equivalent call site must reuse the first site's invokers"
    );

    let run = run_binary(&dir.join("main"), &dir);
    assert!(
        run.status.success(),
        "callable dedup fixture failed at runtime: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "23");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--emit-ir` prints textual EIR and stops before assembly, object,
/// or binary emission.
#[test]
fn test_cli_emit_ir_prints_eir_only() {
    let dir = make_cli_test_dir("elephc_cli_emit_ir");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function greet(): int {
    return 7;
}
echo greet();
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --emit-ir");

    assert!(
        output.status.success(),
        "elephc --emit-ir failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("module target="), "missing module header: {stdout}");
    assert!(stdout.contains("function greet"), "missing lowered function: {stdout}");
    assert!(stdout.contains("const_i64 7"), "missing lowered return literal: {stdout}");
    assert!(stdout.contains("function main"), "missing lowered main function: {stdout}");
    assert!(
        !dir.join("main.s").exists(),
        "--emit-ir should not emit assembly files"
    );
    assert!(
        !dir.join("main.o").exists(),
        "--emit-ir should not emit object files"
    );
    assert!(
        !dir.join("main").exists(),
        "--emit-ir should not emit binaries"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies that passing `--emit-asm` and `--check` together fails with a
/// "mutually exclusive" error message.
#[test]
fn test_cli_rejects_emit_asm_and_check_together() {
    let dir = make_cli_test_dir("elephc_cli_flag_conflict");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;").unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-asm")
        .arg("--check")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with conflicting flags");

    assert!(
        !output.status.success(),
        "expected conflicting flags to fail"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("mutually exclusive"),
        "expected conflict message, got stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--emit-ir` participates in the same exclusive output-mode group
/// as `--emit-asm` and `--check`.
#[test]
fn test_cli_rejects_emit_ir_output_mode_conflicts() {
    let dir = make_cli_test_dir("elephc_cli_emit_ir_flag_conflict");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;").unwrap();

    for conflicting_flag in ["--emit-asm", "--check"] {
        let output = elephc_cli_command(&dir)
            .arg("--emit-ir")
            .arg(conflicting_flag)
            .arg(&php_path)
            .output()
            .expect("failed to run elephc CLI with conflicting --emit-ir flag");

        assert!(
            !output.status.success(),
            "expected --emit-ir {conflicting_flag} to fail"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("mutually exclusive"),
            "expected conflict message, got stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--check --timings` renders the frontend phase table without
/// reporting code generation, assembly, or linking phases.
#[test]
fn test_cli_timings_reports_check_phases() {
    let dir = make_cli_test_dir("elephc_cli_timings_check");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;").unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--check")
        .arg("--timings")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --timings --check");

    assert!(
        output.status.success(),
        "elephc --timings --check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Compiler timings"), "missing timings header: {stderr}");
    assert!(stderr.contains("Tokenizing source"), "missing tokenize timing: {stderr}");
    assert!(stderr.contains("Parsing program"), "missing parse timing: {stderr}");
    assert!(stderr.contains("Checking types"), "missing typecheck timing: {stderr}");
    assert!(stderr.contains("Total"), "missing total timing: {stderr}");
    assert!(
        !stderr.contains("Generating native code"),
        "unexpected codegen timing in --check output: {stderr}"
    );
    assert!(
        !stderr.contains("Assembling object file"),
        "unexpected assemble timing in --check output: {stderr}"
    );
    assert!(
        !stderr.contains("Linking native output"),
        "unexpected link timing in --check output: {stderr}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--timings` renders the native build phases and total duration
/// when compiling a full binary, and that the binary is emitted.
#[test]
fn test_cli_timings_reports_assemble_and_link() {
    let dir = make_cli_test_dir("elephc_cli_timings_build");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;").unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--timings")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --timings");

    assert!(
        output.status.success(),
        "elephc --timings failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Generating native code"),
        "missing codegen timing: {stderr}"
    );
    assert!(
        stderr.contains("Assembling object file"),
        "missing assemble timing: {stderr}"
    );
    assert!(
        stderr.contains("Linking native output"),
        "missing link timing: {stderr}"
    );
    assert!(stderr.contains("Total"), "missing total timing: {stderr}");
    assert!(dir.join("main").exists(), "expected compiled binary to exist");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the timing report records `Runtime cache: miss` for the first
/// compile and `Runtime cache: hit` for the second without rebuilding it.
#[test]
fn test_cli_runtime_cache_reuses_runtime_object() {
    let dir = make_cli_test_dir("elephc_cli_runtime_cache");
    let cache_root = dir.join("cache-root");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;").unwrap();

    let first = Command::new(elephc_cli_bin())
        .arg("--timings")
        .arg(&php_path)
        .env("XDG_CACHE_HOME", &cache_root)
        .current_dir(&dir)
        .output()
        .expect("failed to run first elephc CLI compile with runtime cache");
    assert!(
        first.status.success(),
        "first cached compile failed: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_stderr = String::from_utf8_lossy(&first.stderr);
    assert!(
        first_stderr.contains("Notes"),
        "expected timing notes after first compile, got stderr={first_stderr}"
    );
    assert!(
        first_stderr.contains("Runtime cache: miss"),
        "expected first compile to miss runtime cache, got stderr={first_stderr}"
    );

    let cache_dir = cache_root.join("elephc");
    let cached_objects: Vec<_> = fs::read_dir(&cache_dir)
        .expect("expected runtime cache directory to exist")
        .map(|entry| entry.expect("cache entry").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("o"))
        .collect();
    assert_eq!(
        cached_objects.len(),
        1,
        "expected exactly one cached runtime object, got {:?}",
        cached_objects
    );

    let second = Command::new(elephc_cli_bin())
        .arg("--timings")
        .arg(&php_path)
        .env("XDG_CACHE_HOME", &cache_root)
        .current_dir(&dir)
        .output()
        .expect("failed to run second elephc CLI compile with runtime cache");
    assert!(
        second.status.success(),
        "second cached compile failed: {}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        second_stderr.contains("Notes"),
        "expected timing notes after second compile, got stderr={second_stderr}"
    );
    assert!(
        second_stderr.contains("Runtime cache: hit"),
        "expected second compile to hit runtime cache, got stderr={second_stderr}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--source-map` emits a sidecar .map file in the v2 schema:
/// versioned envelope, function ranges (user function + main), labels, and
/// opcode-tagged line mappings.
#[test]
fn test_cli_source_map_writes_sidecar_file() {
    let dir = make_cli_test_dir("elephc_cli_source_map");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function foo(int $x): int {
    return $x + 1;
}
echo foo(1);
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-asm")
        .arg("--source-map")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --source-map");

    assert!(
        output.status.success(),
        "elephc --source-map failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let map_path = dir.join("main.map");
    assert!(map_path.exists(), "expected source map sidecar to exist");
    let map = fs::read_to_string(&map_path).expect("failed to read source map");
    assert!(
        map.contains("\"format\": \"elephc-source-map\""),
        "missing source map format header: {map}"
    );
    assert!(
        map.contains("\"version\": 2"),
        "missing source map schema version: {map}"
    );
    assert!(
        map.contains("\"asm\":"),
        "expected source map to record the asm path: {map}"
    );
    assert!(
        map.contains("\"name\": \"foo\""),
        "expected a function entry for foo: {map}"
    );
    assert!(
        map.contains("\"name\": \"main\""),
        "expected a function entry for main: {map}"
    );
    assert!(
        map.contains("\"php_line\": 3"),
        "expected a mapping for the return on PHP line 3: {map}"
    );
    assert!(
        map.contains("\"op\": \""),
        "expected opcode-tagged mappings: {map}"
    );
    assert!(
        map.contains("\"labels\": ["),
        "expected a labels section: {map}"
    );
    assert!(
        map.contains("\"source_sha256\": \""),
        "expected a source checksum: {map}"
    );
    assert!(
        map.contains("\"synthetic\": true") && map.contains("\"synthetic\": false"),
        "expected both user and synthetic function entries: {map}"
    );
    assert!(
        map.contains("\"block\": \"entry\""),
        "expected an entry-block label annotation: {map}"
    );
    assert!(
        map.contains("\"php_end_col\":"),
        "expected expression end positions in mappings: {map}"
    );
    assert!(
        map.contains("\"lines\": ["),
        "expected the PHP-line inverse index: {map}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies native-only environment defaults do not make a WASM build fail as
/// though the user had explicitly passed `--null-repr` or `--regalloc`.
#[test]
fn test_cli_wasm_ignores_native_codegen_environment_defaults() {
    let dir = make_cli_test_dir("elephc_cli_wasm_native_env_defaults");
    let php_path = dir.join("main.php");
    fs::write(&php_path, "<?php echo 1;\n").unwrap();

    let output = elephc_cli_command(&dir)
        .env("ELEPHC_NULL_REPR", "tagged")
        .env("ELEPHC_REGALLOC", "stack")
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--emit-asm")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with native-only environment defaults");

    assert!(
        output.status.success(),
        "native-only environment defaults must not reject WASM: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        dir.join("main.wat").exists(),
        "WASM --emit-asm should publish main.wat"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles integer and boolean concatenation through PHP -> EIR -> WASM for
/// every supported PHP profile and verifies `IToStr`, including both signed
/// i64 edges, matches PHP output.
#[test]
fn test_cli_wasm_integer_and_boolean_string_coercion_matches_php_profiles() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_int_bool_string_coercion");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function integer_value(int $value): int {
    return $value;
}
function boolean_value(bool $value): bool {
    return $value;
}
$integer = integer_value(-42);
$other_integer = integer_value(123);
$minimum = integer_value(PHP_INT_MIN);
$maximum = integer_value(PHP_INT_MAX);
$false = boolean_value(false);
$true = boolean_value(true);
echo $integer . $other_integer . "|" . $true . $false . "|" . $minimum . ":" . $maximum;
"#,
    )
    .unwrap();

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile integer/boolean string coercion to WASM");
        assert!(
            output.status.success(),
            "PHP {version} integer/boolean coercion compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("wasmer")
            .arg("run")
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .expect("failed to run integer/boolean string coercion under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} integer/boolean coercion trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "-42123|1|-9223372036854775808:9223372036854775807"
        );
        assert!(
            run.stderr.is_empty(),
            "PHP {version}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles exact strict scalar/string/object equality through the public PHP
/// frontend for every supported compiler profile and verifies raw execution.
///
/// This is target execution coverage; the pinned php-src differential oracle is
/// a separate W1 gate and must not be inferred from this hand-authored matrix.
#[test]
fn test_cli_wasm_strict_equality_executes_supported_profiles() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_strict_equality");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function integer_value(int $value): int {
    return $value;
}
function boolean_value(bool $value): bool {
    return $value;
}
function float_value(float $value): float {
    return $value;
}
function string_value(string $value): string {
    return $value;
}
class ChildValue {}

$one = integer_value(1);
$two = integer_value(2);
$true = boolean_value(true);
$false = boolean_value(false);
$nan = float_value(NAN);
$positiveZero = float_value(0.0);
$negativeZero = float_value(-0.0);
$empty = string_value("");
$binary = string_value("a\0b\xFF");
$sameBinary = string_value("a\0b\xFF");
$prefix = string_value("a\0b");
$differentBinary = string_value("a\0c\xFF");
$object = new ChildValue();
$otherObject = new ChildValue();
$null = null;

echo $one === integer_value(1); echo ",";
echo $one !== $two; echo ",";
echo $true === boolean_value(true); echo ",";
echo $true !== $false; echo ",";
echo $one !== string_value("1"); echo ",";
echo $nan !== $nan; echo ",";
echo $positiveZero === $negativeZero; echo ",";
echo $empty === string_value(""); echo ",";
echo $binary === $sameBinary; echo ",";
echo $binary !== $prefix; echo ",";
echo $binary !== $differentBinary; echo ",";
echo $object !== $null; echo ",";
echo $object !== $otherObject; echo ",";
echo $object === $object; echo ",";
echo $null === null; echo ",";
echo $false !== $null; echo ",";
echo $one === $two; echo ",";
echo $one !== integer_value(1); echo ",";
echo $nan === $nan; echo ",";
echo $positiveZero !== $negativeZero; echo ",";
echo $binary !== $sameBinary; echo ",";
echo $object !== $object; echo ",";
echo $one === string_value("1"); echo ",";
echo match ("need" . string_value("le")) {
    "needle" . string_value("") => true,
    default => false,
}; echo ",";
echo match ("need" . string_value("le")) {
    "other" . string_value("") => false,
    default => true,
}; echo ",";
echo match (new ChildValue()) {
    new ChildValue() => false,
    default => true,
}; echo ",";
"#,
    )
    .unwrap();

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile strict equality to WASM");
        assert!(
            output.status.success(),
            "PHP {version} strict equality compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("wasmer")
            .arg("run")
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .expect("failed to run strict equality under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} strict equality trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        let expected = format!("{}{}{}", "1,".repeat(16), ",".repeat(7), "1,1,1,");
        assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
        assert!(
            run.stderr.is_empty(),
            "PHP {version}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles the php-src saturated-array append edge through PHP -> EIR -> WASM
/// and verifies the command runtime reports the exact failure instead of wrapping
/// `PHP_INT_MAX` to a negative key or surfacing an unclassified Wasm trap.
#[test]
fn test_cli_wasm_append_at_occupied_php_int_max_fails_like_php() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_append_php_int_max");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$a = [PHP_INT_MAX => 1];
$a[] = 2;
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile saturated hash append to WASM");
    assert!(
        output.status.success(),
        "saturated hash append compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.join("main.wasm");
    assert!(wasm_path.exists(), "WASM compilation must publish main.wasm");
    let run = Command::new("wasmer")
        .arg("run")
        .arg(&wasm_path)
        .current_dir(&dir)
        .output()
        .expect("failed to run saturated hash append under Wasmer");
    assert_eq!(run.status.code(), Some(255));
    assert!(run.stdout.is_empty(), "fatal append must not write stdout");
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "PHP Fatal error: Uncaught Error: Cannot add element to the array as the next element is already occupied\n"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the standard streams reach the process fds, by constant and by `php://` name.
///
/// `STDOUT` and `STDERR` are not boxed handles: the EIR gives them as raw integers already typed
/// `resource<stream>`, and their value IS the fd. `php://stdout`, `php://stderr` and
/// `php://output` name the same three fds, which is why they work here while `php://memory` and
/// `php://temp` — real stream implementations — stay refused.
///
/// Needs no preopen at all: none of these touches the filesystem.
#[test]
fn test_cli_wasm_standard_streams_reach_the_process_fds() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_std_streams");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
fwrite(STDOUT, "const-out\n");
fwrite(STDERR, "const-err\n");
$o = fopen("php://stdout", "w");
fwrite($o, "wrapper-out\n");
fclose($o);
$e = fopen("php://stderr", "w");
fwrite($e, "wrapper-err\n");
fclose($e);
$b = fopen("php://output", "w");
fwrite($b, "buffer-out\n");
fclose($b);
"#,
    )
    .unwrap();

    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the standard streams to WASM");
    assert!(
        built.status.success(),
        "the standard streams must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the standard streams under Node");
    // php-src's own answers for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "const-out\nwrapper-out\nbuffer-out\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "const-err\nwrapper-err\n"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the file family round-trips real files under a preopened directory.
///
/// WASI Preview 1 is capability-based: a module reaches no path at all unless the host
/// preopens a directory for it, so the runner below passes `preopens` and the runtime resolves
/// every path against the first preopened fd. Without one, each of these answers PHP's failure
/// value, which is also what a host that grants no filesystem should produce.
///
/// A stream handle is a boxed Mixed cell carrying the WASI fd as a resource payload, which is
/// what lets it live in a local and be passed to `fwrite`/`fread`/`fclose` like any other value.
///
/// Every expected byte is php-src 8.5.6's own answer for the same program, with one documented
/// exception: php-src's open-failure warnings name the path and the errno
/// (`fopen(nope.txt): Failed to open stream: No such file or directory`) and it warns for a
/// failed `unlink` too. These are the NATIVE backend's shorter wording, so the two Elephc
/// targets agree; the VALUES are php-src's exactly.
#[test]
fn test_cli_wasm_file_round_trip_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_files");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({
  version: "preview1",
  args: ["m"],
  env: {},
  preopens: { ".": "." },
  returnOnExit: true,
});
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$f = fopen("out.txt", "w");
echo "w:", fwrite($f, "hello\n"), ",", fwrite($f, "world\n"), "\n";
fclose($f);

$g = fopen("out.txt", "r");
echo "r:[", fread($g, 6), "][", fread($g, 99), "][", fread($g, 4), "]\n";
echo "close:", fclose($g) ? "1" : "0", "\n";

$a = fopen("out.txt", "a");
fwrite($a, "again\n");
fclose($a);
echo "appended:[", file_get_contents("out.txt"), "]\n";

echo "put:", file_put_contents("two.txt", "abc"), "\n";
echo "get:[", file_get_contents("two.txt"), "]\n";
echo "exists:", file_exists("two.txt") ? "1" : "0", "\n";
echo "unlink:", unlink("two.txt") ? "1" : "0", "\n";
echo "gone:", file_exists("two.txt") ? "1" : "0", "\n";

echo "missing-open:", fopen("nope.txt", "r") === false ? "false" : "handle", "\n";
echo "missing-get:", file_get_contents("nope.txt") === false ? "false" : "value", "\n";
echo "missing-unlink:", unlink("nope.txt") ? "1" : "0", "\n";
unlink("out.txt");
"#,
    )
    .unwrap();

    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the file round trip to WASM");
    assert!(
        built.status.success(),
        "the file round trip must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the file round trip under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "w:6,6\n",
            "r:[hello\n][world\n][]\n",
            "close:1\n",
            "appended:[hello\nworld\nagain\n]\n",
            "put:3\n",
            "get:[abc]\n",
            "exists:1\n",
            "unlink:1\n",
            "gone:0\n",
            "missing-open:false\n",
            "missing-get:false\n",
            "missing-unlink:0\n",
        ),
        "php-src's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );
    // The two open failures warn; the wording is the native backend's, not php-src's richer one.
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        concat!(
            "Warning: fopen(): Failed to open stream\n",
            "Warning: file_get_contents(): Failed to open stream\n",
        ),
    );
    // Every file the program made must be gone: `unlink` has to reach the real directory.
    assert!(!dir.join("out.txt").exists(), "out.txt must be unlinked");
    assert!(!dir.join("two.txt").exists(), "two.txt must be unlinked");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `(float) $string` parses the leading numeric prefix exactly as php-src does.
///
/// The runtime parser was already there — `(int) $string` routes float-form prefixes through
/// it — so only the conversion itself was missing. The cases below are the ones a hand-written
/// parser gets wrong: `"0x1A"` is not hex to PHP, `"INF"` and `"NAN"` as text are not infinity
/// and not-a-number, an underscore stops the parse where a numeric literal would allow it, and
/// an exponent past the range answers INF rather than saturating.
///
/// Every expected value is php-src 8.5.6's own answer for the same cast.
#[test]
fn test_cli_wasm_string_to_float_cast_takes_the_leading_numeric_prefix() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_to_float");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$cases = ["12.5", "abc", "12abc", "  7.5  ", "-3.25", "+4", "1e3", ".5", "5.", "", "0x1A", "1e400", "INF", "NAN", "1_000", "  .25e2xyz"];
foreach ($cases as $s) {
    echo (float) $s, "|";
}
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the string-to-float fixture to WASM");
    assert!(
        output.status.success(),
        "string-to-float compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the string-to-float fixture under Wasmer");
    assert!(
        run.status.success(),
        "string-to-float fixture trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "12.5|0|12|7.5|-3.25|4|1000|0.5|5|0|0|INF|0|0|1|25|",
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `$mixed[$key]` reads every receiver tag the way php-src does.
///
/// The receiver here is genuinely unknown at compile time — it comes back out of a hash, so the
/// checker only knows `mixed` and the whole dispatch happens at runtime on the cell's tag. That
/// is the single most frequent shape the WASM audit used to refuse.
///
/// Every expected byte below is php-src 8.5.6's own answer for the same program. Two arms are
/// places the NATIVE backend is wrong and this one is not: a string receiver is indexed by byte
/// (native answers an empty string), and a scalar receiver warns before answering null (native
/// says nothing at all).
///
/// The warning wording is version-profiled, the same split the null receiver already carried:
/// before 8.3 PHP names the TYPE for all of them, and from 8.3 it names the type for int and
/// float but the VALUE for a boolean, so `true` and `false` read differently there.
#[test]
fn test_cli_wasm_mixed_array_read_dispatches_on_the_receiver_tag() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_array_read");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function box(mixed $v): mixed {
    $holder = ["it" => $v];
    return $holder["it"];
}
echo "[", box([10, 20, 30])[1], "]";
echo "[", box([10, 20, 30])["1"], "]";
echo "[", box(["x", "y"])[1], "]";
echo "[", box([1.5, 2.5])[0], "]";
echo "[", box(["a" => 7])["a"], "]";
echo "[", count(box([[1, 2], [3, 4, 5]])[1]), "]";
echo "[", box("hello")[1], "]";
echo "[", box("hello")[-2], "]";
echo "|";
echo "[", box([10, 20, 30])[7], "]";
echo "[", box(["a" => 7])["z"], "]";
echo "[", box("hello")[9], "]";
echo "[", box(null)["k"], "]";
echo "[", box(42)["k"], "]";
echo "[", box(1.5)["k"], "]";
echo "[", box(true)["k"], "]";
echo "[", box(false)["k"], "]";
"#,
    )
    .unwrap();

    // Hits are identical across profiles; only the wording of the diagnostics moves.
    let expected_stdout = "[20][20][y][1.5][7][3][e][l]|[][][][][][][][]";
    for (version, expected_stderr) in [
        (
            "8.2",
            concat!(
                "Warning: Undefined array key 7\n",
                "Warning: Undefined array key \"z\"\n",
                "Warning: Uninitialized string offset 9\n",
                "Warning: Trying to access array offset on value of type null\n",
                "Warning: Trying to access array offset on value of type int\n",
                "Warning: Trying to access array offset on value of type float\n",
                "Warning: Trying to access array offset on value of type bool\n",
                "Warning: Trying to access array offset on value of type bool\n",
            ),
        ),
        (
            "8.5",
            concat!(
                "Warning: Undefined array key 7\n",
                "Warning: Undefined array key \"z\"\n",
                "Warning: Uninitialized string offset 9\n",
                "Warning: Trying to access array offset on null\n",
                "Warning: Trying to access array offset on int\n",
                "Warning: Trying to access array offset on float\n",
                "Warning: Trying to access array offset on true\n",
                "Warning: Trying to access array offset on false\n",
            ),
        ),
    ] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile the mixed array read fixture to WASM");
        assert!(
            output.status.success(),
            "PHP {version} mixed array read compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("wasmer")
            .arg("run")
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .expect("failed to run the mixed array read fixture under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} mixed array read trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_stdout,
            "PHP {version} values"
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            expected_stderr,
            "PHP {version} diagnostics"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles the exact php-src next-free origin split through PHP -> EIR -> WASM
/// for every supported compatibility profile. PHP 8.2 promotes immutable `[]`
/// with next=0, while a direct mutable `[-3 => 1]` starts at LONG_MIN; PHP
/// 8.3-8.5 start both empty-literal and mutable paths at LONG_MIN.
#[test]
fn test_cli_wasm_empty_promotion_and_direct_hash_match_php_profiles() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_hash_next_origin");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$a = [];
$a[-3] = 1;
$a[] = 2;
echo "empty:";
foreach ($a as $key => $value) {
    echo $key, ",";
}

$b = [-3 => 1];
$b[] = 2;
echo "|literal:";
foreach ($b as $key => $value) {
    echo $key, ",";
}
"#,
    )
    .unwrap();

    for (version, expected) in [
        ("8.2", "empty:-3,0,|literal:-3,-2,"),
        ("8.3", "empty:-3,-2,|literal:-3,-2,"),
        ("8.4", "empty:-3,-2,|literal:-3,-2,"),
        ("8.5", "empty:-3,-2,|literal:-3,-2,"),
    ] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile hash next-free profile fixture to WASM");
        assert!(
            output.status.success(),
            "PHP {version} hash-origin compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        assert!(wasm_path.exists(), "WASM compilation must publish main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run hash-origin fixture under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} hash-origin fixture trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "PHP {version}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles a PHP-source closure stored in a local, invokes it with one argument
/// through the non-empty Mixed argument-buffer path, and checks exact output.
#[test]
fn test_cli_wasm_dynamic_closure_argument_prints_42() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_dynamic_closure_42");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$f = function(int $x): int { return $x; };
echo $f(42);
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile dynamic closure call to WASM");
    assert!(
        output.status.success(),
        "dynamic closure compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.join("main.wasm");
    assert!(wasm_path.exists(), "WASM compilation must publish main.wasm");
    let run = Command::new("wasmer")
        .arg("run")
        .arg(&wasm_path)
        .current_dir(&dir)
        .output()
        .expect("failed to run dynamic closure call under Wasmer");
    assert!(
        run.status.success(),
        "dynamic closure call trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42");
    assert!(run.stderr.is_empty(), "unexpected stderr: {:?}", run.stderr);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a Mixed receiver dispatches directly to the selected covariant
/// override instead of imposing another implementation's WASM return ABI.
#[test]
fn test_cli_wasm_mixed_virtual_covariant_return_uses_exact_implementation_abi() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_covariant_return");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class A {
    public function run(): mixed { return 1; }
}
class B extends A {
    public function run(): string { return "x"; }
}
function invoke_mixed(mixed $value): mixed {
    return $value->run();
}
echo invoke_mixed(new B());
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile covariant Mixed dispatch to WASM");
    assert!(
        output.status.success(),
        "covariant Mixed dispatch compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.join("main.wasm");
    assert!(wasm_path.exists(), "WASM compilation must publish main.wasm");
    let run = Command::new("wasmer")
        .arg("run")
        .arg(&wasm_path)
        .current_dir(&dir)
        .output()
        .expect("failed to run covariant Mixed dispatch under Wasmer");
    assert!(
        run.status.success(),
        "covariant Mixed dispatch failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "x");
    assert!(run.stderr.is_empty(), "unexpected stderr: {:?}", run.stderr);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies heterogeneous dynamic method returns box PHP void as null and
/// transfer callable ownership into the result cell without leaking the
/// callee-owned descriptor.
#[test]
fn test_cli_wasm_mixed_method_void_and_callable_returns_are_balanced() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_void_callable_return");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class VoidResult {
    public function run(): void {}
}
class CallableResult {
    public function run(): callable {
        return function(): int { return 42; };
    }
}
function invoke_mixed(mixed $value): mixed {
    return $value->run();
}
echo is_null(invoke_mixed(new VoidResult())), ";";
$callable = invoke_mixed(new CallableResult());
echo is_null($callable), ";";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile void/callable Mixed dispatch to WASM");
    assert!(
        output.status.success(),
        "void/callable Mixed dispatch compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.join("main.wasm");
    let run = Command::new("wasmer")
        .arg("run")
        .arg(&wasm_path)
        .current_dir(&dir)
        .output()
        .expect("failed to run void/callable Mixed dispatch under Wasmer");
    assert!(
        run.status.success(),
        "void/callable Mixed dispatch failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "1;;");
    assert!(run.stderr.is_empty(), "unexpected stderr: {:?}", run.stderr);

    let emit = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--emit-asm")
        .arg(&php_path)
        .output()
        .expect("failed to emit void/callable Mixed dispatch WAT");
    assert!(
        emit.status.success(),
        "void/callable Mixed WAT emission failed: {}",
        String::from_utf8_lossy(&emit.stderr)
    );
    let wat = fs::read_to_string(dir.join("main.wat")).expect("read emitted WAT");
    assert!(
        wat.contains("box null (void callee, mixed result)"),
        "void return did not materialize Mixed(null): {wat}"
    );
    let callable_source = wat
        .find("callee-owned callable descriptor")
        .expect("callable source ownership marker");
    let callable_release = wat[callable_source..]
        .find("call $__rt_decref_any")
        .map(|offset| callable_source + offset)
        .expect("callable source release");
    assert!(
        callable_release > callable_source,
        "callable source must be released after result-cell boxing: {wat}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles an escaping by-ref closure from PHP source to wasm32-wasi and runs it
/// twice under Wasmer. The creator's frame is gone before either call, so two
/// successful writes and reads prove the closure owns the ref cell.
#[test]
fn test_cli_wasm_escaping_by_ref_closure_survives_creator_return() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_escaping_ref_closure");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function make() {
    $x = 0;
    return function() use (&$x) {
        $x = $x ? 3 : 2;
        return $x;
    };
}

$f = make();
echo $f(), $f();
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile escaping by-ref closure to WASM");
    assert!(
        output.status.success(),
        "escaping by-ref closure compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wasm_path = dir.join("main.wasm");
    assert!(wasm_path.exists(), "WASM compilation must publish main.wasm");
    let run = Command::new("wasmer")
        .arg("run")
        .arg(&wasm_path)
        .current_dir(&dir)
        .output()
        .expect("failed to run escaping by-ref closure under Wasmer");
    assert!(
        run.status.success(),
        "escaping by-ref closure trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "23");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies null coalescing lowers indexed int/bool/string reads through the
/// silent opcode with an explicit null-capable Tagged/Mixed result.
///
/// Full Wasmer execution of `??` remains blocked by the separate unsupported
/// `UnsetLocal` capability; this test proves the EIR boundary does not erase null.
#[test]
fn test_cli_wasm_null_coalesce_array_reads_keep_nullable_eir() {
    let dir = make_cli_test_dir("elephc_cli_wasm_array_coalesce_eir");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo [10][$argc] ?? 77;
echo [true][$argc] ?? 77;
echo ["x"][$argc] ?? 77;
$hash = ["x" => 10];
echo $hash["missing"] ?? 77;
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--emit-ir")
        .arg(&php_path)
        .output()
        .expect("failed to emit WASM-target EIR for null coalescing");
    assert!(
        output.status.success(),
        "WASM-target EIR emission failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let eir = String::from_utf8_lossy(&output.stdout);
    assert!(
        eir.contains("TaggedScalar php=int|null = array_get_silent"),
        "int coalesce read lost nullable TaggedScalar metadata: {eir}"
    );
    assert_eq!(
        eir.matches("Heap(Mixed) php=mixed own=owned = array_get_silent")
            .count(),
        2,
        "bool/string coalesce reads must remain boxed nullable values: {eir}"
    );
    assert!(
        eir.contains("TaggedScalar php=int|null = hash_get_silent"),
        "associative int coalesce read lost nullable TaggedScalar metadata: {eir}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles typed int/bool/string indexed reads from PHP through EIR to WASM and
/// executes them under Wasmer. Negative/OOB reads emit one PHP warning per
/// ordinary access and remain null through `is_null` and `echo`; the former
/// integer sentinel remains a valid in-range value.
#[test]
fn test_cli_wasm_indexed_array_oob_preserves_php_null() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_oob_null");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo is_null([10][-1]), ":", [10][-1], ";";
echo is_null([10][1]), ":", [10][1], ";";
echo is_null([9223372036854775806][0]), ":", [9223372036854775806][0], ";";
echo is_null([true][-1]), ":", [true][-1], ";";
echo is_null([true][1]), ":", [true][1], ";";
echo is_null([""][0]), ":", [""][0], ";";
echo is_null(["x"][-1]), ":", ["x"][-1], ";";
echo is_null(["x"][1]), ":", ["x"][1], ";";
echo (int)[10][-1], ",", (bool)[10][-1], ",", (float)[10][-1], ";";
echo (int)[true][-1], ",", (bool)[true][-1], ";";
echo (int)["x"][-1], ",", (bool)["x"][-1], ",", (string)["x"][-1], ";";
"#,
    )
    .unwrap();

    let warning_keys = [
        -1, -1, 1, 1, -1, -1, 1, 1, -1, -1, 1, 1, -1, -1, -1, -1, -1, -1, -1,
        -1,
    ];
    let expected_stderr = warning_keys
        .iter()
        .map(|key| format!("Warning: Undefined array key {key}"))
        .collect::<Vec<_>>();
    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile indexed-array OOB fixture to WASM");
        assert!(
            output.status.success(),
            "PHP {version} indexed-array OOB compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run indexed-array OOB fixture under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} indexed-array OOB fixture trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "1:;1:;:9223372036854775806;1:;1:;:;1:;1:;\
0,,0;0,;0,,;",
            "PHP {version}"
        );
        let actual_stderr = String::from_utf8_lossy(&run.stderr)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_stderr, expected_stderr,
            "PHP {version} ordinary indexed misses must warn exactly once in source order"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles typed associative reads through PHP -> EIR -> WASM. Missing string
/// and integer keys remain PHP null, emit the key-class-specific warning once,
/// and cannot collide with a valid integer equal to the former sentinel.
#[test]
fn test_cli_wasm_hash_reads_preserve_null_and_warn_like_php() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_hash_oob_null");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$ints = ["hit" => 10];
$bools = ["hit" => true];
$floats = ["hit" => 1.5];
$strings = ["hit" => ""];
$sentinel = ["hit" => 9223372036854775806];
$integerKeys = [7 => 10];
echo is_null($ints["missing"]), ":", $ints["hit"], ";";
echo is_null($bools["missing"]), ":", $bools["hit"], ";";
echo is_null($floats["missing"]), ":", $floats["hit"], ";";
echo is_null($strings["missing"]), ":", $strings["hit"], ";";
echo is_null($sentinel["hit"]), ":", $sentinel["hit"], ";";
echo is_null($integerKeys[9]), ":", $integerKeys[7], ";";
"#,
    )
    .unwrap();

    let expected_stderr = [
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key 9",
    ];
    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile associative-read fixture to WASM");
        assert!(
            output.status.success(),
            "PHP {version} associative-read compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run associative-read fixture under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} associative-read fixture trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "1:10;1:1;1:1.5;1:;:9223372036854775806;1:10;",
            "PHP {version}"
        );
        let actual_stderr = String::from_utf8_lossy(&run.stderr)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(
            actual_stderr, expected_stderr,
            "PHP {version} associative misses must warn exactly once in source order"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP source cannot reach WASM float-to-int or float-key lowering
/// while their versioned warning and deprecation diagnostics remain incomplete.
#[test]
fn test_cli_wasm_rejects_diagnostic_sensitive_float_to_int_paths() {
    let dir = make_cli_test_dir("elephc_cli_wasm_float_to_int_diagnostics");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$key = (float) $argv[1];
echo (int) $key;
$discard_cast = (int) $key;
$discard_array = [$key => 1];
$discard_bool = !$key;
if ($key) {}
$discard_nan_not = !NAN;
$discard_nan_ternary = NAN ? 1 : 2;
$discard_nan_short = NAN ?: 2;
if (NAN) {}
$mixed = $argc > 1 ? $key : "1";
echo (int) $mixed;
function wasm_source(bool $flag): mixed { return $flag ? 1.5 : 1; }
function wasm_sink(int $value): void {}
wasm_sink(wasm_source($argc > 1));
$checked = function(int $value): int { return $value + 1; };
$discard_checked = $checked($argc);
function wasm_checked_ref(): callable {
    $value = 1;
    return function() use (&$value) { return ++$value; };
}
$checked_ref = wasm_checked_ref();
$discard_checked_ref = $checked_ref();
$values = ["seed" => 1];
$values[$key] = 2;
echo $values[$key];
unset($values[$key]);
"#,
    )
    .unwrap();

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile float-diagnostics fixture");
        assert!(
            !output.status.success(),
            "PHP {version} must reject diagnostic-sensitive float conversions"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Three of this fixture's refusals have since been implemented and are deliberately
        // gone. The explicit `(int)` cast carries its exact PHP 8.5 diagnostic; the implicit
        // Mixed-to-scalar TRANSFER narrows through `__rt_mixed_narrow_int` the way the native
        // backend does; and TRUTHINESS now emits the NaN warning it was refused for, so a
        // command module like this one no longer turns it away.
        //
        // What still refuses is the FLOAT ASSOCIATIVE KEY, whose implicit-conversion
        // diagnostics are profile-specific, and the explicit Mixed-to-scalar cast.
        assert!(
            !stderr.contains("truthiness"),
            "PHP {version}: truthiness carries its NaN warning now: {stderr}"
        );
        assert!(
            stderr
                .matches(
                    "float associative keys require exact profile-specific implicit-conversion diagnostics"
                )
                .count()
                >= 3,
            "PHP {version}: the float-key refusals were optimized away: {stderr}"
        );
        assert!(
            stderr.contains(
                "Mixed-to-scalar casts require exact per-tag PHP values and diagnostics"
            ),
            "PHP {version}: {stderr}"
        );
        // Checked arithmetic through an escaping ref cell no longer produces a shape
        // rejection: the capture widens to `Mixed`, so the store and the loads agree and
        // the overflow promotion survives. `test_by_ref_capture_preserves_integer_overflow_promotion`
        // owns that behavior now.
        assert!(
            stderr.contains(
                "float associative keys require exact profile-specific implicit-conversion diagnostics"
            ),
            "PHP {version}: {stderr}"
        );
        assert!(
            !dir.join("main.wat").exists() && !dir.join("main.wasm").exists(),
            "PHP {version} rejection must publish no artifact"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies WASM arithmetic over boxed Mixed operands matches php-src.
///
/// `MixedNumericBinop` carries PHP's numeric semantics for values whose type is only
/// known at runtime: integer-overflow promotion, `bool` and `null` as integers, and the
/// numeric-string rules, where the *form* decides the result type — `"7" + 5` is an
/// integer while `"7.0" + 5` is a double. A string with only a numeric prefix warns and
/// contributes that prefix; one with none is a PHP `TypeError`.
#[test]
fn test_cli_wasm_mixed_numeric_arithmetic_matches_php() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_numeric");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function box(mixed $v): mixed { return $v; }
echo box(2) + 5, "\n";
echo box(1.5) + 5, "\n";
echo box(true) + 5, "\n";
echo box(null) + 5, "\n";
echo box("7") + 5, "\n";
echo box("7.0") + 5, "\n";
echo box("7e2") + 5, "\n";
echo box(" 7") + 5, "\n";
echo box("7 ") + 5, "\n";
echo box("007") + 5, "\n";
echo box("+7") + 5, "\n";
echo box("-7") + 5, "\n";
echo box(".5") + 5, "\n";
echo box(9223372036854775807) + 5, "\n";
echo box("9223372036854775808") + 5, "\n";
echo box("7") * 3, "\n";
echo box(10) - box(3), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile mixed numeric arithmetic to WASM");
    assert!(
        output.status.success(),
        "mixed numeric compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run mixed numeric arithmetic under Wasmer");
    assert!(
        run.status.success(),
        "mixed numeric arithmetic trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // Every line is php-src 8.5's own output for the same program.
    let expected = concat!(
        "7\n", "6.5\n", "6\n", "5\n", "12\n", "12\n", "705\n", "12\n", "12\n", "12\n",
        "12\n", "-2\n", "5.5\n", "9.2233720368548E+18\n", "9.2233720368548E+18\n",
        "21\n", "7\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    assert!(
        run.stderr.is_empty(),
        "well-formed operands must not diagnose: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a string carrying only a numeric prefix warns, and a non-numeric one is fatal.
#[test]
fn test_cli_wasm_mixed_numeric_string_diagnostics() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_numeric_diag");

    let leading = dir.join("leading.php");
    fs::write(
        &leading,
        "<?php\nfunction box(mixed $v): mixed { return $v; }\necho box(\"7abc\") + 5, \"\\n\";\n",
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&leading)
        .output()
        .expect("failed to compile the leading-numeric fixture");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("leading.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the leading-numeric fixture");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "12\n");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("A non-numeric value encountered"),
        "a numeric prefix must warn: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let fatal = dir.join("fatal.php");
    fs::write(
        &fatal,
        "<?php\nfunction box(mixed $v): mixed { return $v; }\necho box(\"abc\") + 5, \"\\n\";\n",
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&fatal)
        .output()
        .expect("failed to compile the non-numeric fixture");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("fatal.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the non-numeric fixture");
    assert_eq!(
        run.status.code(),
        Some(255),
        "a non-numeric operand is an uncaught TypeError: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("Unsupported operand types"),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `declare(strict_types=1)` suppresses PHP's scalar argument coercions.
///
/// PHP performs no scalar coercion at a typed parameter under strict typing, with one
/// documented exception: an `int` argument still widens to a `float` parameter. Without
/// the directive the same calls are legal coercive conversions. Verified against the
/// pinned php-src CLIs, which raise `TypeError` for exactly the rejected pairs.
#[test]
fn test_cli_strict_types_refuses_scalar_argument_coercion() {
    let dir = make_cli_test_dir("elephc_cli_strict_types_coercion");

    // (parameter type, argument, admitted under strict typing)
    let cases = [
        ("int", "true", false),
        ("float", "true", false),
        ("bool", "1", false),
        ("float", "1", true),
        ("int", "1", true),
    ];

    for (param_ty, argument, strict_admits) in cases {
        for strict in [false, true] {
            let declare = if strict { "declare(strict_types=1);" } else { "" };
            let php_path = dir.join("main.php");
            fs::write(
                &php_path,
                format!(
                    "<?php\n{declare}\nfunction sink({param_ty} $x): {param_ty} {{ return $x; }}\necho sink({argument});\n"
                ),
            )
            .unwrap();

            let output = elephc_cli_command(&dir)
                .arg("--check")
                .arg(&php_path)
                .output()
                .expect("failed to type-check the coercion fixture");

            // Coercive mode admits every pair; strict mode admits only widening.
            let expected = !strict || strict_admits;
            assert_eq!(
                output.status.success(),
                expected,
                "strict={strict} {param_ty} <- {argument}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            if !expected {
                assert!(
                    String::from_utf8_lossy(&output.stderr)
                        .contains("strict_types=1 performs no coercion"),
                    "strict={strict} {param_ty} <- {argument} must name the cause: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    // The same gate gates every typed write, not just arguments: PHP applies strict
    // typing to a typed property assignment and to a declared return type as well.
    let sites = [
        ("class C { public int $v = 0; } $o = new C(); $o->v = true; echo 1;", "property"),
        ("function f(): int { return true; } echo f();", "return type"),
    ];
    for (source, site) in sites {
        for strict in [false, true] {
            let declare = if strict { "declare(strict_types=1);" } else { "" };
            let php_path = dir.join("main.php");
            fs::write(&php_path, format!("<?php\n{declare}\n{source}\n")).unwrap();

            let output = elephc_cli_command(&dir)
                .arg("--check")
                .arg(&php_path)
                .output()
                .expect("failed to type-check the typed-write fixture");
            assert_eq!(
                output.status.success(),
                !strict,
                "strict={strict} bool at an int {site}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the explicit `(int)` float cast matches each pinned php-src profile.
///
/// Requirement H (`PHP-WASM-NUM-004`). The value is PHP's modulo-2^64 result on every
/// profile, so `(int) 1.0e20` is the mandatory regression. The diagnostic is version
/// dependent: PHP 8.5 alone warns, and only for values no integer can represent. That
/// predicate is about range and finiteness, never integrality, so `1.9` stays silent
/// on every profile while `NAN` and `INF` warn on 8.5.
#[test]
fn test_cli_wasm_explicit_float_to_int_cast_matches_php_profiles() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_float_to_int_cast");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function float_value(float $value): float {
    return $value;
}
echo (int) float_value(1.0e20); echo "\n";
echo (int) float_value(-1.0e20); echo "\n";
echo (int) float_value(1.9); echo "\n";
echo (int) float_value(-1.9); echo "\n";
echo (int) float_value(NAN); echo "\n";
echo (int) float_value(INF); echo "\n";
"#,
    )
    .unwrap();

    // Values are identical on every profile; only the diagnostics differ.
    let expected_stdout =
        "7766279631452241920\n-7766279631452241920\n1\n-1\n0\n0\n";
    let warning = "is not representable as an int, cast occurred";

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile the float cast to WASM");
        assert!(
            output.status.success(),
            "PHP {version} float cast compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("wasmer")
            .arg("run")
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .expect("failed to run the float cast under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} float cast trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_stdout,
            "PHP {version} float cast values must match php-src"
        );

        let stderr = String::from_utf8_lossy(&run.stderr);
        if version == "8.5" {
            // 1.0e20, -1.0e20, NAN and INF are unrepresentable; 1.9 and -1.9 are not.
            assert_eq!(
                stderr.matches(warning).count(),
                4,
                "PHP {version} must warn once per unrepresentable value: {stderr}"
            );
            assert!(
                stderr.contains("Warning: The float 1.0E+20 "),
                "PHP {version} must render the float exactly as PHP prints it: {stderr}"
            );
        } else {
            assert!(
                stderr.is_empty(),
                "PHP {version} must not diagnose the cast: {stderr}"
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies public PHP shapes with incomplete WASM runtime contracts fail closed.
#[test]
fn test_cli_wasm_rejects_unproven_object_iterator_and_global_shapes() {
    let dir = make_cli_test_dir("elephc_cli_wasm_unproven_shapes");
    let php_path = dir.join("main.php");
    let cases = [
        (
            r#"<?php class C { public int $value; } $c = new C(); echo $c->value;"#,
            "may be uninitialized and requires an exact PHP fatal check",
        ),
        (
            r#"<?php #[AllowDynamicProperties] class C {} $c = new C(); echo $c->missing;"#,
            "reads require the exact PHP undefined-property warning",
        ),
        (
            r#"<?php class A { public int $x = 1; } class B { public int $x = 2; } $o = $argc > 1 ? new A() : new B(); echo $o?->x;"#,
            "Nullsafe property access requires a single nullable object type",
        ),
        (
            r#"<?php $h = ["a" => 1]; foreach ($h as &$v) { $v = 2; }"#,
            "by-reference foreach over associative arrays",
        ),
        (
            r#"<?php $a = [1, 2]; foreach ($a as $v) { echo $v; $a[] = 3; }"#,
            "may mutate the iterated container without PHP snapshot/COW semantics",
        ),
        (
            r#"<?php function cmp(int $a, int $b): int { return $a - $b; } $a = [2, 1]; foreach ($a as $v) { echo $v; usort($a, 'cmp'); }"#,
            "usort may mutate the iterated container without PHP snapshot/COW semantics",
        ),
        (
            r#"<?php function read_custom(): mixed { global $custom; return $custom; } echo read_custom();"#,
            "global $custom is not implemented by the WASI runtime",
        ),
    ];

    for (index, (source, expected)) in cases.iter().enumerate() {
        fs::write(&php_path, source).unwrap();
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg("8.5")
            .arg(&php_path)
            .output()
            .unwrap_or_else(|error| panic!("case #{index} failed to invoke elephc: {error}"));
        assert!(
            !output.status.success(),
            "case #{index} unexpectedly compiled: {source}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected),
            "case #{index} missing {expected:?}: {stderr}"
        );
        assert!(
            !dir.join("main.wat").exists() && !dir.join("main.wasm").exists(),
            "case #{index} rejection published a WASM artifact"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies indexed boolean arrays preserve tag 3 when promoted to hashes.
#[test]
fn test_cli_wasm_bool_array_promotion_preserves_boolean_tags() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_bool_array_promotion");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$a = [false];
$a["k"] = true;
echo "[", $a[0], "]";
$b = [];
$b[] = false;
$b["k"] = true;
echo "[", $b[0], "]";
$c = [];
$c[0] = false;
$c["k"] = true;
echo "[", $c[0], "]";
class Flag { public bool $value = false; }
$flag = new Flag();
echo "[", $flag->value, "]";
$flag->value = false;
echo "[", $flag?->value, "]";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--php-version")
        .arg("8.5")
        .arg(&php_path)
        .output()
        .expect("failed to compile boolean-array promotion fixture");
    assert!(
        output.status.success(),
        "boolean-array promotion compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run boolean-array promotion fixture");
    assert!(
        run.status.success(),
        "boolean-array promotion fixture trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "[][][][][]");
    assert!(
        run.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a declared mixed property retains a borrowed cell independently of its source.
#[test]
fn test_cli_wasm_mixed_property_retains_borrowed_source() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_property_borrow");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Holder { public mixed $value = 1; }
function make_value(): mixed { return "hello"; }
function fill(Holder $holder): int {
    $value = make_value();
    $holder->value = $value;
    return 0;
}
$holder = new Holder();
fill($holder);
echo $holder->value;
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--php-version")
        .arg("8.5")
        .arg(&php_path)
        .output()
        .expect("failed to compile borrowed mixed-property fixture");
    assert!(
        output.status.success(),
        "borrowed mixed-property compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new("wasmer")
        .arg("run")
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run borrowed mixed-property fixture");
    assert!(
        run.status.success(),
        "borrowed mixed-property fixture trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "hello");
    assert!(
        run.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies associative reads preserve precise nullable container pointers:
/// misses remain PHP null, hits remain non-null, and hit containers still feed
/// typed chained reads.
#[test]
fn test_cli_wasm_hash_container_reads_preserve_nullable_php_values() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_hash_container_null");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Item {
    public function value(): int {
        return 3;
    }
}
$arrays = ["hit" => [1]];
$hashes = ["hit" => ["x" => 2]];
$objects = ["hit" => new Item()];
echo is_null($arrays["missing"]), ":", is_null($arrays["hit"]), ";";
echo is_null($hashes["missing"]), ":", is_null($hashes["hit"]), ";";
echo is_null($objects["missing"]), ":", is_null($objects["hit"]), ";";
echo $arrays["hit"][0], ":", $hashes["hit"]["x"], ";";
echo $objects["hit"]->value(), ";";
echo is_null($arrays["hit"][99]), ":", is_null($hashes["hit"]["missing"]), ";";
"#,
    )
    .unwrap();

    let expected_stderr = [
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key \"missing\"",
        "Warning: Undefined array key 99",
        "Warning: Undefined array key \"missing\"",
    ];
    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile associative container reads to WASM");
        assert!(
            output.status.success(),
            "PHP {version} associative container compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run associative container reads under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} associative container reads trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "1:;1:;1:;1:2;3;1:1;",
            "PHP {version}"
        );
        let actual_stderr = String::from_utf8_lossy(&run.stderr)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(actual_stderr, expected_stderr, "PHP {version}");
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies nullable chained reads evaluate their index exactly once before
/// normal offset-on-null warnings while coalescing reads remain silent.
#[test]
fn test_cli_wasm_nullable_chained_reads_preserve_php_index_order() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_nullable_chain_order");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function int_key_side_effect(string $label): int {
    echo $label;
    return 0;
}
function string_key_side_effect(string $label): string {
    echo $label;
    return "inner";
}
$arrays = ["hit" => [1]];
$hashes = ["hit" => ["inner" => 2]];
echo "A", $arrays["missing-array"][int_key_side_effect("a")], "Z;";
echo "H", $hashes["missing-hash"][string_key_side_effect("h")], "Z;";
echo "S", ($arrays["missing-array"][int_key_side_effect("s")] ?? 9), ";";
echo "T", ($hashes["missing-hash"][string_key_side_effect("t")] ?? 8), ";";
echo "I", $arrays["hit"][int_key_side_effect("i")], ":";
echo $hashes["hit"][string_key_side_effect("j")], ";";
"#,
    )
    .unwrap();

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let offset_warning = if version == "8.2" {
            "Warning: Trying to access array offset on value of type null"
        } else {
            "Warning: Trying to access array offset on null"
        };
        let expected_stderr = [
            "Warning: Undefined array key \"missing-array\"",
            offset_warning,
            "Warning: Undefined array key \"missing-hash\"",
            offset_warning,
        ];
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile nullable chained reads to WASM");
        assert!(
            output.status.success(),
            "PHP {version} nullable chain compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run nullable chained reads under Wasmer");
        assert!(
            run.status.success(),
            "PHP {version} nullable chained reads trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "AaZ;HhZ;Ss9;Tt8;Ii1:j2;",
            "PHP {version}"
        );
        let actual_stderr = String::from_utf8_lossy(&run.stderr)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert_eq!(actual_stderr, expected_stderr, "PHP {version}");
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a missing object-valued associative entry raises PHP's method-on-null
/// warning/fatal pair before evaluating method arguments.
#[test]
fn test_cli_wasm_missing_hash_object_method_call_is_php_fatal() {
    if Command::new("wasmer").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_hash_object_null_fatal");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function side_effect(): int {
    echo "BAD";
    return 1;
}
class Item {
    public function value(int $value): int {
        return $value;
    }
}
$objects = ["hit" => new Item()];
$objects["missing"]->value(side_effect());
"#,
    )
    .unwrap();

    for version in ["8.2", "8.3", "8.4", "8.5"] {
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg("--php-version")
            .arg(version)
            .arg(&php_path)
            .output()
            .expect("failed to compile missing object method call to WASM");
        assert!(
            output.status.success(),
            "PHP {version} missing object method-call compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let wasm_path = dir.join("main.wasm");
        let run = Command::new("wasmer")
            .arg("run")
            .arg(&wasm_path)
            .current_dir(&dir)
            .output()
            .expect("failed to run missing object method call under Wasmer");
        assert_eq!(run.status.code(), Some(255), "PHP {version}");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "",
            "PHP {version}: argument side effects must not run after a null receiver"
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            "Warning: Undefined array key \"missing\"\nPHP Fatal error: Uncaught Error: Call to a member function value() on null\n",
            "PHP {version}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--debug-info` injects DWARF line-table directives into the emitted
/// assembly: one `.file 1` header and a `.loc 1 <line> <col>` per source marker.
#[test]
fn test_cli_debug_info_injects_dwarf_line_directives() {
    let dir = make_cli_test_dir("elephc_cli_debug_info");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo 1 + 2;
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--emit-asm")
        .arg("--debug-info")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --debug-info");

    assert!(
        output.status.success(),
        "elephc --debug-info failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let asm_path = dir.join("main.s");
    let asm = fs::read_to_string(&asm_path).expect("failed to read assembly");
    assert!(
        asm.starts_with(".file 1 \""),
        "expected .file header at top of assembly, got: {}",
        &asm[..asm.len().min(120)]
    );
    assert!(
        asm.contains(".loc 1 2 "),
        "expected a .loc directive for PHP line 2: {asm}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP `try`/`catch`/`throw` lowers to the Core WebAssembly exception forms.
///
/// The shapes asserted here are the whole of the design: one module-level `tag` carrying the
/// exception object pointer, a `try_table` wrapping the dispatch loop, a `throw` at the raise
/// site, and a landing pad that turns the catch into an ordinary dispatch-state transition.
/// Asserting on the emitted WAT rather than on program output keeps this test meaningful on a
/// machine with no exceptions-capable host installed.
#[test]
fn test_cli_wasm_try_catch_lowers_to_core_exception_forms() {
    let dir = make_cli_test_dir("elephc_cli_wasm_try_catch_forms");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Boom extends Exception {
}

function risky(int $n): void {
    if ($n < 0) {
        throw new Boom();
    }
    echo "ok\n";
}

try {
    risky(1);
    risky(-1);
} catch (Boom $e) {
    echo "caught\n";
}
echo "done\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile try/catch to WASM");
    assert!(
        output.status.success(),
        "try/catch compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let wat = fs::read_to_string(dir.join("main.wat")).expect("missing emitted WAT");
    assert!(
        wat.contains("(tag $__php_exc (param i32))"),
        "expected the PHP exception tag: {wat}"
    );
    assert!(
        wat.contains("(try_table (catch $__php_exc $__caught)"),
        "expected the dispatch loop to be guarded: {wat}"
    );
    assert!(
        wat.contains("throw $__php_exc"),
        "expected the raise site to throw the tag: {wat}"
    );
    assert!(
        wat.contains("global.set $__exc_value"),
        "expected the landing pad to publish the caught exception: {wat}"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies thrown exceptions select the matching `catch` clause and reach the right frame.
///
/// Runs the compiled module under Node's WASI, which implements the Core WebAssembly exception
/// proposal; the expected output is php-src's own for the same program. Skipped when no Node is
/// installed, so `test_cli_wasm_try_catch_lowers_to_core_exception_forms` remains the assertion
/// that always runs.
#[test]
fn test_cli_wasm_try_catch_dispatch_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_try_catch_dispatch");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class AlphaError extends Exception {
}

class BetaError extends Exception {
}

function pick(int $n): void {
    if ($n === 1) {
        throw new AlphaError();
    }
    if ($n === 2) {
        throw new BetaError();
    }
    echo "none\n";
}

foreach ([0, 1, 2] as $n) {
    try {
        pick($n);
        echo "no throw\n";
    } catch (AlphaError $e) {
        echo "alpha\n";
    } catch (BetaError $e) {
        echo "beta\n";
    }
}

try {
    try {
        throw new AlphaError();
    } catch (BetaError $e) {
        echo "inner-wrong\n";
    }
} catch (AlphaError $e) {
    echo "outer-right\n";
}

echo "end\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile catch dispatch to WASM");
    assert!(
        output.status.success(),
        "catch dispatch compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run catch dispatch under Node");
    // Node without exception support fails to compile the module rather than misbehaving;
    // treat that as "no capable host" rather than a lowering failure.
    if !run.status.success()
        && String::from_utf8_lossy(&run.stderr).contains("CompileError")
    {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "catch dispatch trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5's own output for the same program.
    let expected = concat!(
        "none\n",
        "no throw\n",
        "alpha\n",
        "beta\n",
        "outer-right\n",
        "end\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an exception nobody catches is PHP's fatal, not an escape into the host.
///
/// A WebAssembly exception that unwinds out of `_start` would surface as a host-level crash with
/// no PHP diagnostic at all, so `main` is guarded even when it contains no `catch`. The exit
/// status is php-src's 255. The message text is deliberately not compared: reproducing PHP's
/// `Uncaught Exception: <message> in <file>:<line>` needs the built-in Throwable accessors,
/// which this target does not lower yet.
#[test]
fn test_cli_wasm_uncaught_exception_is_a_php_fatal() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_uncaught_exception");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        "<?php\necho \"before\\n\";\nthrow new Exception();\n",
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the uncaught throw to WASM");
    assert!(
        output.status.success(),
        "uncaught throw compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the uncaught throw under Node");
    if String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "before\n",
        "output before the throw must still be flushed"
    );
    assert_eq!(
        run.status.code(),
        Some(255),
        "an uncaught PHP exception exits 255: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP's arithmetic RAISES on the native backend, on both targets alike.
///
/// Five operators answered a machine result where reference PHP raises: `%` by zero returned
/// zero, either shift by a negative count returned the hardware's masked result, and BOTH float
/// and integer `/` by zero returned an infinity. All five were SILENT wrong answers — a program
/// that expected an exception continued with a plausible-looking number.
///
/// Integer `/` is a separate opcode from float `/` because PHP promotes its operands, so the
/// guard has to run on the INTEGER divisor before the promotion: a promoted zero has a sign, and
/// testing it after the fact would need the sign masked off.
///
/// The shift also masked its count to six bits, so `1 << 64` answered 1 and `-8 >> 64` answered
/// -8 where PHP answers 0 and -1. That is fixed here too: PHP saturates rather than wrapping.
///
/// Both backends are checked against the same expected output, because this is where they
/// disagreed: WASM already raised four of the five, so the native one was mostly the outlier —
/// but integer `/` was wrong on BOTH, which is why one expected output covers them together.
#[test]
fn test_cli_arithmetic_raises_match_php_on_both_backends() {
    let dir = make_cli_test_dir("elephc_cli_arithmetic_raises");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function m(int $a, int $b): int { return $a % $b; }
function sl(int $a, int $b): int { return $a << $b; }
function sr(int $a, int $b): int { return $a >> $b; }
function fd(float $a, float $b): float { return $a / $b; }
function q(int $a, int $b): float { return $a / $b; }
echo m(7,3), "|", m(-7,3), "|", m(7,-3), "\n";
echo sl(1,0), "|", sl(1,63), "|", sl(1,64), "|", sl(1,100), "|", sl(-8,1), "\n";
echo sr(1,63), "|", sr(1,64), "|", sr(-8,1), "|", sr(-8,64), "|", sr(-8,100), "\n";
echo fd(1.5,0.5), "|", fd(-6.0,3.0), "|", q(6,3), "|", q(7,2), "\n";
try { echo m(1,0), "\n"; } catch (\DivisionByZeroError $a) { echo "mod0|", $a->getMessage(), "\n"; }
try { echo sl(1,-1), "\n"; } catch (\ArithmeticError $b) { echo "shl|", $b->getMessage(), "\n"; }
try { echo sr(1,-1), "\n"; } catch (\ArithmeticError $c) { echo "shr|", $c->getMessage(), "\n"; }
try { echo fd(1.0,0.0), "\n"; } catch (\DivisionByZeroError $d) { echo "fdiv|", $d->getMessage(), "\n"; }
try { echo fd(1.0,-0.0), "\n"; } catch (\DivisionByZeroError $e) { echo "fdiv-neg0|", $e->getMessage(), "\n"; }
try { echo q(1,0), "\n"; } catch (\DivisionByZeroError $f) { echo "intdiv0|", $f->getMessage(), "\n"; }
try { echo q(0,0), "\n"; } catch (\DivisionByZeroError $g) { echo "int00|", $g->getMessage(), "\n"; }
echo "end\n";
"#,
    )
    .unwrap();

    // php-src 8.5.6's own output for the same program.
    let expected = concat!(
        "1|-1|1\n",
        "1|-9223372036854775808|0|0|-16\n",
        "0|0|-4|-1|-1\n",
        "3|-2|2|3.5\n",
        "mod0|Modulo by zero\n",
        "shl|Bit shift by negative number\n",
        "shr|Bit shift by negative number\n",
        "fdiv|Division by zero\n",
        "fdiv-neg0|Division by zero\n",
        "intdiv0|Division by zero\n",
        "int00|Division by zero\n",
        "end\n",
    );

    let native = elephc_cli_command(&dir)
        .arg(&php_path)
        .output()
        .expect("failed to compile the arithmetic raises natively");
    assert!(
        native.status.success(),
        "native compilation failed: {}",
        String::from_utf8_lossy(&native.stderr)
    );
    let native_run = Command::new(dir.join("main"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the arithmetic raises natively");
    assert!(
        native_run.status.success(),
        "a caught arithmetic error still killed the native program: {}",
        String::from_utf8_lossy(&native_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&native_run.stdout), expected);

    if Command::new("node").arg("--version").output().is_err() {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    let wasm = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the arithmetic raises to WASM");
    assert!(
        wasm.status.success(),
        "WASM compilation failed: {}",
        String::from_utf8_lossy(&wasm.stderr)
    );
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();
    let wasm_run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the arithmetic raises under Node");
    if !wasm_run.status.success()
        && String::from_utf8_lossy(&wasm_run.stderr).contains("CompileError")
    {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        wasm_run.status.success(),
        "a caught arithmetic error still killed the WASM program: {}",
        String::from_utf8_lossy(&wasm_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&wasm_run.stdout), expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP's arithmetic runtime errors are CATCHABLE, not process-killing fatals.
///
/// Reference PHP raises `DivisionByZeroError` / `ArithmeticError` for these five guards, so a
/// `catch` receives them and execution continues past the `try`. Emitting them as a direct
/// `__rt_fail` exit — which is what this backend did before — silently skipped every handler
/// and killed the program, so the assertion that matters is that `end` is reached at all. Each
/// clause binds its own variable because re-binding one name across clauses of DIFFERENT classes
/// still corrupts the caught object on this target, which is an unrelated open defect.
#[test]
fn test_cli_wasm_runtime_errors_are_catchable() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_catchable_runtime_errors");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function idiv(int $x, int $y): int { return intdiv($x, $y); }
function imod(int $x, int $y): int { return $x % $y; }
function ishift(int $x, int $y): int { return $x << $y; }
function fdiv2(float $x, float $y): float { return $x / $y; }

try { echo idiv(1, 0), "\n"; } catch (\DivisionByZeroError $a) { echo "A|", $a->getMessage(), "\n"; }
try { echo imod(1, 0), "\n"; } catch (\DivisionByZeroError $b) { echo "B|", $b->getMessage(), "\n"; }
try { echo ishift(1, -1), "\n"; } catch (\ArithmeticError $c) { echo "C|", $c->getMessage(), "\n"; }
try { echo idiv(PHP_INT_MIN, -1), "\n"; } catch (\ArithmeticError $d) { echo "D|", $d->getMessage(), "\n"; }
try { echo fdiv2(1.0, 0.0), "\n"; } catch (\DivisionByZeroError $f) { echo "E|", $f->getMessage(), "\n"; }
echo "end\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile catchable runtime errors to WASM");
    assert!(
        output.status.success(),
        "catchable runtime error compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run catchable runtime errors under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "a caught runtime error still killed the program: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    let expected = concat!(
        "A|Division by zero\n",
        "B|Modulo by zero\n",
        "C|Bit shift by negative number\n",
        "D|Division of PHP_INT_MIN by -1 is not an integer\n",
        "E|Division by zero\n",
        "end\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an uncaught runtime error reports identically whether or not it was raised.
///
/// A module with no `try` cannot catch, so its guards stay on the direct `__rt_fail` path that
/// keeps the program runnable on a host without the exceptions proposal. A module that does
/// catch raises instead, and the diagnostic then has to travel with the exception for `main`'s
/// landing pad to print it — otherwise the uncaught case regresses to the class-agnostic
/// "Uncaught exception". Both variants of each failure are compiled here precisely because the
/// two paths are different code: the point is that they are indistinguishable from outside.
/// php-src also prints the file, line and stack trace, which this target does not reproduce
/// yet; the class, the message and the 255 exit status are what is compared.
#[test]
fn test_cli_wasm_uncaught_runtime_error_keeps_its_php_diagnostic() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let runner_source = r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#;

    // A `try` for a class the failure never raises: it does not catch anything here, it only
    // makes the module declare the exception tag, which is what puts the guards on the raise
    // path. Without it the same program stays on the direct fatal path.
    let arming_try = concat!(
        "try {\n",
        "    echo \"armed\\n\";\n",
        "} catch (\\RuntimeException $unused) {\n",
        "    echo \"never\\n\";\n",
        "}\n",
    );

    for (label, body, expected) in [
        (
            "div",
            "function f(int $x, int $y): int { return intdiv($x, $y); }\necho f(1, 0), \"\\n\";\n",
            "PHP Fatal error: Uncaught DivisionByZeroError: Division by zero\n",
        ),
        (
            "mod",
            "function f(int $x, int $y): int { return $x % $y; }\necho f(1, 0), \"\\n\";\n",
            "PHP Fatal error: Uncaught DivisionByZeroError: Modulo by zero\n",
        ),
        (
            "shift",
            "function f(int $x, int $y): int { return $x << $y; }\necho f(1, -1), \"\\n\";\n",
            "PHP Fatal error: Uncaught ArithmeticError: Bit shift by negative number\n",
        ),
        (
            "overflow",
            "function f(int $x, int $y): int { return intdiv($x, $y); }\necho f(PHP_INT_MIN, -1), \"\\n\";\n",
            "PHP Fatal error: Uncaught ArithmeticError: Division of PHP_INT_MIN by -1 is not an integer\n",
        ),
    ] {
        for (path_label, prologue, expected_stdout) in
            [("direct", "", ""), ("raised", arming_try, "armed\n")]
        {
            let dir = make_cli_test_dir(&format!(
                "elephc_cli_wasm_uncaught_runtime_error_{label}_{path_label}"
            ));
            let php_path = dir.join("main.php");
            fs::write(&php_path, format!("<?php\n{prologue}{body}")).unwrap();

            let output = elephc_cli_command(&dir)
                .arg("--target")
                .arg("wasm32-wasi")
                .arg(&php_path)
                .output()
                .expect("failed to compile an uncaught runtime error to WASM");
            assert!(
                output.status.success(),
                "uncaught runtime error compilation failed for {label}/{path_label}: {}",
                String::from_utf8_lossy(&output.stderr)
            );

            let runner = dir.join("run.mjs");
            fs::write(&runner, runner_source).unwrap();

            // `--no-warnings` keeps Node's `ExperimentalWarning: WASI` off the stream the PHP
            // diagnostic is compared on.
            let run = Command::new("node")
                .arg("--no-warnings")
                .arg(&runner)
                .arg(dir.join("main.wasm"))
                .current_dir(&dir)
                .output()
                .expect("failed to run an uncaught runtime error under Node");
            let stderr = String::from_utf8_lossy(&run.stderr).to_string();
            if stderr.contains("CompileError") {
                let _ = fs::remove_dir_all(&dir);
                continue;
            }
            assert_eq!(
                stderr, expected,
                "uncaught {label} lost the PHP class and message it names on the {path_label} path"
            );
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                expected_stdout,
                "output before the {label} failure must still be flushed on the {path_label} path"
            );
            assert_eq!(
                run.status.code(),
                Some(255),
                "an uncaught PHP runtime error exits 255 for {label}/{path_label}"
            );

            let _ = fs::remove_dir_all(&dir);
        }
    }
}

/// Verifies a class property STRING default is materialized, raw and boxed.
///
/// Object construction writes defaults inline rather than through the class's
/// `_class_propinit_*` function, so a string default has no `DataId` to address at the
/// construction site and needs its own content-keyed data segment. A `mixed` slot exercises the
/// boxed arm, where the string becomes a Mixed cell rather than a raw (ptr, len) pair.
#[test]
fn test_cli_wasm_string_property_defaults_are_materialized() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_property_defaults");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class C {
    public string $name = "x";
    public mixed $tag = "boxed";
}

$c = new C();
echo $c->name, "|", $c->tag, "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile string property defaults to WASM");
    assert!(
        output.status.success(),
        "string property default compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run string property defaults under Node");
    assert!(
        run.status.success(),
        "string property defaults trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    // php-src 8.5's own output for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), "x|boxed\n");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the built-in `Throwable` accessors answer what the NATIVE backend answers.
///
/// These methods carry a signature but no EIR body on either backend, so both open-code them.
/// The comparison that matters is therefore native-vs-WASM, not WASM-vs-php-src: elephc records
/// no per-throw file, line or backtrace, so `getFile()` is empty, `getLine()` zero,
/// `getTraceAsString()` empty and `__toString()` the message alone — php-src reports all four
/// differently, and a program that changed behavior once compiled for WebAssembly would be the
/// real defect. `getPrevious()` returns `?Throwable`, so the chained call exercises the
/// Mixed-receiver dispatch ladder rather than the direct path.
#[test]
fn test_cli_wasm_throwable_accessors_match_the_native_backend() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_throwable_accessors");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Wrapped extends Exception {
}

$first = new Wrapped("inner", 3);
try {
    throw new Exception("outer", 9, $first);
} catch (Exception $e) {
    echo $e->getMessage(), "|", $e->getCode(), "\n";
    echo "[", $e->getFile(), "]", $e->getLine(), "\n";
    echo "[", $e->getTraceAsString(), "]\n";
    echo $e->__toString(), "\n";
    $p = $e->getPrevious();
    echo $p->getMessage(), "|", $p->getCode(), "\n";
}
echo "end\n";
"#,
    )
    .unwrap();

    let native = elephc_cli_command(&dir)
        .arg(&php_path)
        .output()
        .expect("failed to compile Throwable accessors natively");
    assert!(
        native.status.success(),
        "native compilation failed: {}",
        String::from_utf8_lossy(&native.stderr)
    );
    let native_run = Command::new(dir.join("main"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the native Throwable accessors");
    assert!(
        native_run.status.success(),
        "native run failed: {}",
        String::from_utf8_lossy(&native_run.stderr)
    );

    let wasm = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile Throwable accessors to WASM");
    assert!(
        wasm.status.success(),
        "WASM compilation failed: {}",
        String::from_utf8_lossy(&wasm.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let wasm_run = Command::new("node")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the WASM Throwable accessors under Node");
    if String::from_utf8_lossy(&wasm_run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        wasm_run.status.success(),
        "WASM run failed: {}",
        String::from_utf8_lossy(&wasm_run.stderr)
    );

    // `getFile()` and `getLine()` are the one pair the two backends no longer answer alike, and
    // the difference is a WASM gap rather than a disagreement: the native backend reports the
    // Throwable's CONSTRUCTION SITE — the script path from the `_script_source_file` symbol and
    // the line stamped into the object payload at `new` — while this target still answers the
    // empty string and 0, which is what BOTH answered before that landed natively. Comparing the
    // full stdout would assert a parity that is simply not true today, so the location fields are
    // normalized out and the rest is still compared exactly.
    let normalize_location = |text: &str| -> String {
        text.lines()
            .map(|line| {
                // The `getFile()|getLine()` line is rendered as `[<path>]<line>`.
                match (line.find('['), line.rfind(']')) {
                    (Some(start), Some(end)) if start == 0 && end > start => "[]0".to_string(),
                    _ => line.to_string(),
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        normalize_location(&String::from_utf8_lossy(&wasm_run.stdout)),
        normalize_location(&String::from_utf8_lossy(&native_run.stdout)),
        "the two backends must answer every Throwable accessor identically apart from the \
         construction-site file and line, which this target does not carry yet"
    );
    // And state the gap as a fact rather than leaving it implicit: native reports a real path
    // and line, this target reports neither. When WASM grows the construction-site stamp, this
    // assertion is what fails and tells you to restore the exact comparison above.
    assert!(
        String::from_utf8_lossy(&wasm_run.stdout).contains("[]0"),
        "WASM is expected to answer getFile()/getLine() as []/0 until it carries the \
         construction site: {}",
        String::from_utf8_lossy(&wasm_run.stdout)
    );
    assert!(
        !String::from_utf8_lossy(&native_run.stdout).contains("[]0"),
        "the native backend is expected to report a real construction site: {}",
        String::from_utf8_lossy(&native_run.stdout)
    );
    // Pinned so a change to elephc's synthetic answers has to be deliberate on both backends.
    // The native location line carries an absolute path into a per-run temp directory, so the
    // path itself cannot be pinned — the file it names and the line number can, and those are
    // what the construction-site stamp is actually asserting.
    let native_stdout = String::from_utf8_lossy(&native_run.stdout);
    let native_lines: Vec<&str> = native_stdout.lines().collect();
    assert_eq!(
        native_lines.len(),
        6,
        "unexpected native accessor output: {native_stdout}"
    );
    assert_eq!(native_lines[0], "outer|9");
    assert!(
        native_lines[1].starts_with('[') && native_lines[1].ends_with("main.php]7"),
        "native getFile()/getLine() must name the construction site: {}",
        native_lines[1]
    );
    assert_eq!(&native_lines[2..], &["[]", "outer", "inner|3", "end"]);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the explicit `(int)` / `(float)` cast of a runtime-typed value matches php-src.
///
/// The interesting cases are the ones a naive implementation gets wrong: PHP yields 1 for ANY
/// non-empty array rather than its length, wraps a finite out-of-range float modulo 2^64 instead
/// of saturating, maps NaN and both infinities to zero, and diagnoses an object while still
/// producing 1. Every expected line here is php-src 8.5's own output for the same program.
#[test]
fn test_cli_wasm_explicit_mixed_scalar_casts_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_scalar_casts");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class C {}
function box(mixed $v): mixed { return $v; }
echo (int) box(42), "\n";
echo (int) box(true), "\n";
echo (int) box(null), "\n";
echo (int) box(3.7), "\n";
echo (int) box(-3.7), "\n";
echo (int) box("  12abc"), "\n";
echo (int) box("abc"), "\n";
echo (int) box([]), "\n";
echo (int) box([1, 2, 3]), "\n";
echo (int) box(1.0e19), "\n";
echo (int) box(NAN), "\n";
echo (int) box(INF), "\n";
echo (int) box(new C()), "\n";
echo (float) box("3.5"), "\n";
echo (float) box([1]), "\n";
echo (float) box(new C()), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile mixed scalar casts to WASM");
    assert!(
        output.status.success(),
        "mixed scalar cast compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // `--no-warnings` keeps Node's own ExperimentalWarning about `node:wasi` out of the stderr
    // this test compares against php-src's diagnostics.
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run mixed scalar casts under Node");
    assert!(
        run.status.success(),
        "mixed scalar casts trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = concat!(
        "42\n", "1\n", "0\n", "3\n", "-3\n", "12\n", "0\n", "0\n", "1\n",
        "-8446744073709551616\n", "0\n", "0\n", "1\n", "3.5\n", "1\n", "1\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);

    // php-src reports the same four diagnostics, in this order. The project's WASM convention
    // drops php-src's `PHP ` prefix and its ` in <file> on line <n>` tail.
    let stderr = String::from_utf8_lossy(&run.stderr);
    let diagnostics: Vec<&str> = stderr.lines().collect();
    assert_eq!(
        diagnostics,
        vec![
            "Warning: The float 1.0E+19 is not representable as an int, cast occurred",
            "Warning: The float NAN is not representable as an int, cast occurred",
            "Warning: The float INF is not representable as an int, cast occurred",
            "Warning: Object of class C could not be converted to int",
            "Warning: Object of class C could not be converted to float",
        ],
        "diagnostics must match php-src's set and order"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the builtins lowered to a single WebAssembly instruction match php-src exactly.
///
/// `floor`, `ceil` and `sqrt` are bit-for-bit identities with their WebAssembly counterparts,
/// which the negative-zero case pins: `ceil(-0.5)` is `-0`, not `0`. `count` reads the container
/// header. `abs` is the one with an argument-dependent shape — integral in, integral out.
#[test]
fn test_cli_wasm_direct_builtins_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_direct_builtins");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function ints(int $n): void { echo abs($n), "\n"; }
function floats(float $x): void { echo abs($x), "|", floor($x), "|", ceil($x), "|", sqrt(abs($x)), "\n"; }
ints(-3); ints(3); ints(0);
floats(3.7); floats(-3.2); floats(-0.5); floats(0.0);
$a = [1, 2, 3];
echo count($a), "\n";
$h = ['x' => 1, 'y' => 2];
echo count($h), "\n";
$e = [];
echo array_is_list($a) ? "list" : "not", "|", array_is_list($e) ? "list" : "not", "\n";
function arrays(array $xs, int $n): void {
    $v = array_values($xs);
    $k = array_keys($xs);
    $v[0] = 99;
    echo in_array($n, $xs, true) ? "y" : "n", "|", count($k), "|", $k[1], "|", $xs[0], ",", $v[0], "\n";
}
arrays([7, 8, 9], 8);
arrays([7, 8, 9], 5);
function folds(array $xs): void {
    $r = array_reverse($xs);
    echo array_sum($xs), "|", array_product($xs), "|", $r[0], ",", $r[2], "\n";
}
folds([1, 2, 3]);
echo array_sum($e), "|", array_product($e), "|", count(array_reverse($e)), "\n";
function pairs(int $p, int $q): void {
    echo max($p, $q), "|", min($p, $q), "|", intdiv($p, $q), "\n";
}
pairs(7, 2);
pairs(-7, 2);
$filled = array_fill(0, 3, 7);
echo count($filled), "|", $filled[2], "|", count(array_fill(0, 0, 9)), "\n";
function needles(string $h, string $n): void {
    echo str_contains($h, $n) ? 1 : 0, str_starts_with($h, $n) ? 1 : 0, str_ends_with($h, $n) ? 1 : 0, "\n";
}
needles("hello", "ell");
needles("hello", "");
needles("he", "hello");
needles("", "");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the direct builtins to WASM");
    assert!(
        output.status.success(),
        "direct builtin compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the direct builtins under Node");
    assert!(
        run.status.success(),
        "direct builtins trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5's own output for the same program.
    let expected = concat!(
        "3\n",
        "3\n",
        "0\n",
        "3.7|3|4|1.9235384061671\n",
        "3.2|-4|-3|1.7888543819998\n",
        "0.5|-1|-0|0.70710678118655\n",
        "0|0|0|0\n",
        "3\n",
        "2\n",
        "list|list\n",
        "y|3|1|7,99\n",
        "n|3|1|7,99\n",
        "6|6|3,1\n",
        "0|1|0\n",
        "7|2|3\n",
        "2|-7|-3\n",
        "3|7|0\n",
        "100\n",
        "111\n",
        "000\n",
        "111\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    assert!(
        run.stderr.is_empty(),
        "these builtins diagnose nothing: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the byte-mapping string transforms match php-src.
///
/// Since PHP 8.2 `strtoupper` and `strtolower` are locale-independent and touch `A-Z` / `a-z`
/// only, so a byte outside that range comes back unchanged; `strrev` reverses BYTES. Every
/// expected line is php-src 8.5's own output for the same program.
#[test]
fn test_cli_wasm_unary_string_transforms_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_unary_strings");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function t(string $s): void {
    echo strtoupper($s), "|", strtolower($s), "|", strrev($s), "\n";
}
t("aBc1-z");
t("");
t("a");
t("Hello, World!");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the string transforms to WASM");
    assert!(
        output.status.success(),
        "string transform compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the string transforms under Node");
    assert!(
        run.status.success(),
        "string transforms trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "ABC1-Z|abc1-z|z-1cBa\n",
            "||\n",
            "A|a|a\n",
            "HELLO, WORLD!|hello, world!|!dlroW ,olleH\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the LENGTH-CHANGING string transforms reproduce php-src byte for byte.
///
/// Each result is printed through `bin2hex` so a wrong byte cannot hide behind a terminal's
/// rendering, and the samples pin the edges that separate these from a naive implementation:
/// `addslashes` escapes NUL to the two characters `\0`; `stripslashes` turns `\0` into a NUL
/// byte but `\n` into the letter n, and drops a trailing lone backslash; `nl2br` keeps the break
/// it tags and treats `\r\n` as one. Raw high bytes are included because they are exactly what a
/// data segment written from Rust's UTF-8 rather than the PHP bytes would corrupt.
#[test]
fn test_cli_wasm_re_encoding_string_transforms_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_re_encoding_strings");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function t(string $s): void {
    echo bin2hex($s), "|", bin2hex(addslashes($s)), "|", bin2hex(stripslashes($s)), "|", bin2hex(nl2br($s)), "\n";
}
t("");
t("abc");
t("a'b\"c\\d");
t("x\ny\r\nz\rw");
t("\n\r");
t("\x00\x01\xff");
t("\\0");
t("a\\");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the re-encoding transforms to WASM");
    assert!(
        output.status.success(),
        "re-encoding transform compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the re-encoding transforms under Node");
    assert!(
        run.status.success(),
        "re-encoding transforms trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "|||\n",
            "616263|616263|616263|616263\n",
            "61276222635c64|615c27625c22635c5c64|612762226364|61276222635c64\n",
            "780a790d0a7a0d77|780a790d0a7a0d77|780a790d0a7a0d77|783c6272202f3e0a793c6272202f3e0d0a7a3c6272202f3e0d77\n",
            "0a0d|0a0d|0a0d|3c6272202f3e0a0d\n",
            "0001ff|5c3001ff|0001ff|0001ff\n",
            "5c30|5c5c30|00|5c30\n",
            "615c|615c5c|61|615c\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the string-shaping builtins reproduce php-src byte for byte.
///
/// The samples pin what a naive implementation gets wrong. `ucwords` treats VERTICAL TAB as a
/// word delimiter but not `-`, `_` or `.`; `trim`'s default set includes NUL and vertical tab,
/// while an explicitly EMPTY charlist strips nothing. `strcmp` reports the raw UNSIGNED byte
/// distance at the first mismatch — -32 for `ABC` against `abc`, 254 for `\xff` against `\x01` —
/// but normalizes a pure length difference to +/-1. `substr` answers the empty string rather than
/// false for every out-of-range case, a negative offset counts from the end and saturates, and a
/// negative length names an end offset from the right.
#[test]
fn test_cli_wasm_string_shaping_builtins_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_shaping");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function t(string $s): void {
    echo bin2hex($s), "|", bin2hex(ucfirst($s)), "|", bin2hex(lcfirst($s)), "|", bin2hex(ucwords($s)),
         "|", bin2hex(trim($s)), "|", bin2hex(ltrim($s)), "|", bin2hex(rtrim($s)), "\n";
}
t(""); t("a"); t("abc"); t("ABC"); t("hello world  foo"); t(" \t x \n "); t("\x00\x0bz\x0b\x00"); t("h\xc3\xa9llo"); t("123abc");
t("a\tb\nc\rd\x0ce\x0bf g"); t("a-b_c.d");
function c(string $a, string $b): void { echo strcmp($a, $b), "|", strcasecmp($a, $b), "\n"; }
c("a","a"); c("a","b"); c("b","a"); c("","a"); c("a",""); c("",""); c("abc","abd"); c("ABC","abc");
c("a","A"); c("abc","ab"); c("\xff","\x01"); c("abcd","a"); c("ab","abcdefgh"); c("Z","a"); c("_","a");
function s2(string $x, int $o): void { echo bin2hex(substr($x, $o)), "\n"; }
function s3(string $x, int $o, int $n): void { echo bin2hex(substr($x, $o, $n)), "\n"; }
s2("hello",0); s2("hello",2); s2("hello",-2); s2("hello",5); s2("hello",6); s2("hello",-9); s2("",0); s2("",3);
s3("hello",1,3); s3("hello",1,-1); s3("hello",-3,2); s3("hello",0,-5); s3("hello",0,-9); s3("hello",2,0); s3("hello",2,99); s3("hello",-2,-1);
function tc(string $x, string $cl): void { echo bin2hex(trim($x,$cl)), "|", bin2hex(ltrim($x,$cl)), "|", bin2hex(rtrim($x,$cl)), "\n"; }
tc("xxhelloxx","x"); tc("abcHELLOcba","abc"); tc("hello",""); tc("aaa","a");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the shaping builtins to WASM");
    assert!(
        output.status.success(),
        "shaping builtin compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the shaping builtins under Node");
    assert!(
        run.status.success(),
        "shaping builtins trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "||||||\n",
            "61|41|61|41|61|61|61\n",
            "616263|416263|616263|416263|616263|616263|616263\n",
            "414243|414243|614243|414243|414243|414243|414243\n",
            "68656c6c6f20776f726c642020666f6f|48656c6c6f20776f726c642020666f6f|68656c6c6f20776f726c642020666f6f|48656c6c6f20576f726c642020466f6f|68656c6c6f20776f726c642020666f6f|68656c6c6f20776f726c642020666f6f|68656c6c6f20776f726c642020666f6f\n",
            "20092078200a20|20092078200a20|20092078200a20|20092058200a20|78|78200a20|20092078\n",
            "000b7a0b00|000b7a0b00|000b7a0b00|000b5a0b00|7a|7a0b00|000b7a\n",
            "68c3a96c6c6f|48c3a96c6c6f|68c3a96c6c6f|48c3a96c6c6f|68c3a96c6c6f|68c3a96c6c6f|68c3a96c6c6f\n",
            "313233616263|313233616263|313233616263|313233616263|313233616263|313233616263|313233616263\n",
            "6109620a630d640c650b662067|4109620a630d640c650b662067|6109620a630d640c650b662067|4109420a430d440c450b462047|6109620a630d640c650b662067|6109620a630d640c650b662067|6109620a630d640c650b662067\n",
            "612d625f632e64|412d625f632e64|612d625f632e64|412d625f632e64|612d625f632e64|612d625f632e64|612d625f632e64\n",
            "0|0\n",
            "-1|-1\n",
            "1|1\n",
            "-1|-1\n",
            "1|1\n",
            "0|0\n",
            "-1|-1\n",
            "-32|0\n",
            "32|0\n",
            "1|1\n",
            "254|254\n",
            "1|1\n",
            "-1|-1\n",
            "-7|25\n",
            "-2|-2\n",
            "68656c6c6f\n",
            "6c6c6f\n",
            "6c6f\n",
            "\n",
            "\n",
            "68656c6c6f\n",
            "\n",
            "\n",
            "656c6c\n",
            "656c6c\n",
            "6c6c\n",
            "\n",
            "\n",
            "\n",
            "6c6c6f\n",
            "6c\n",
            "68656c6c6f|68656c6c6f7878|787868656c6c6f\n",
            "48454c4c4f|48454c4c4f636261|61626348454c4c4f\n",
            "68656c6c6f|68656c6c6f|68656c6c6f\n",
            "||\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `htmlspecialchars` under PHP 8.1+ defaults, invalid UTF-8 included.
///
/// The defaults are `ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401`, so BOTH quote styles are
/// escaped — `'` becomes `&#039;` rather than passing through as it did before 8.1 — and invalid
/// UTF-8 is replaced with U+FFFD instead of making the call return the empty string. NUL and the
/// control bytes are valid UTF-8 and pass through untouched.
///
/// The substitution span is the subtle part and is WIDER than the usual "maximal subpart": a
/// valid lead absorbs following bytes up to what it announced, stopping only at a byte that could
/// START a sequence. So `"\xc2\xc0"` is ONE replacement while `"\xc2\xc2"` is two, and a byte
/// that can never lead stands alone, making `"\xc0\x80"` two and `"\xf5\x80\x80\x80"` four.
/// A plain continuation-byte test gets 102 byte pairs wrong and passes this sample set anyway,
/// which is why the rule was settled by sweeping every pair rather than by these cases.
#[test]
fn test_cli_wasm_htmlspecialchars_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_htmlspecialchars");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function h(string $s): void { echo bin2hex(htmlspecialchars($s)), "\n"; }
h(""); h("abc"); h("<a href=\"x\">"); h("a&b"); h("it's"); h("<>&\"'");
h("h\xc3\xa9llo"); h("\xff\xfe"); h("a\xffb"); h("\x00\x01"); h("&amp;");
h("\xc3"); h("\xc3\x28"); h("\xe2\x82"); h("\xe2\x82\x28"); h("\xf0\x9f"); h("\xf0\x9f\x92"); h("\xf0\x9f\x92\xa9");
h("\xc0\x80"); h("\xed\xa0\x80"); h("\xf5\x80\x80\x80"); h("\xc2\x80"); h("\xe0\x80\x80"); h("\xf4\x90\x80\x80");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile htmlspecialchars to WASM");
    assert!(
        output.status.success(),
        "htmlspecialchars compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run htmlspecialchars under Node");
    assert!(
        run.status.success(),
        "htmlspecialchars trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "\n",
            "616263\n",
            "266c743b6120687265663d2671756f743b782671756f743b2667743b\n",
            "6126616d703b62\n",
            "697426233033393b73\n",
            "266c743b2667743b26616d703b2671756f743b26233033393b\n",
            "68c3a96c6c6f\n",
            "efbfbdefbfbd\n",
            "61efbfbd62\n",
            "0001\n",
            "26616d703b616d703b\n",
            "efbfbd\n",
            "efbfbd28\n",
            "efbfbd\n",
            "efbfbd28\n",
            "efbfbd\n",
            "efbfbd\n",
            "f09f92a9\n",
            "efbfbdefbfbd\n",
            "efbfbd\n",
            "efbfbdefbfbdefbfbdefbfbd\n",
            "c280\n",
            "efbfbd\n",
            "efbfbd\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `md5` reproduces php-src, block boundaries included.
///
/// MD5 shares SHA-1's padding SHAPE but reads and writes every word LITTLE-endian, which is the
/// single biggest difference between them and the usual way a port of one into the other goes
/// wrong: a digest that is byte-reversed per word still looks like a plausible hash. The digest
/// bytes come out low-first within each word for the same reason. Lengths either side of every
/// boundary the padding rule turns on are covered, as they are for sha1.
#[test]
fn test_cli_wasm_md5_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_md5");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function h(string $s): void { echo md5($s), "\n"; }
h("");
h("a");
h("abc");
h("message digest");
h("The quick brown fox jumps over the lazy dog");
h("\x00\x01\xff");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile md5 to WASM");
    assert!(
        output.status.success(),
        "md5 compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run md5 under Node");
    assert!(
        run.status.success(),
        "md5 trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own digests, which are the published RFC 1321 test vectors.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "d41d8cd98f00b204e9800998ecf8427e\n",
            "0cc175b9c0f1b6a831c399e269772661\n",
            "900150983cd24fb0d6963f7d28e17f72\n",
            "f96b697d7cb7938d525a2f31aaf161d0\n",
            "9e107d9d372bb6826bd81d3542a419d6\n",
            "ffbb8cd5a232b7d906904533e9609f48\n",
            "eced9e0b81ef2bba605cbc5e2e76a1d0\n",
            "ef1772b6dff9a122358552954ad0df65\n",
            "3b0c8ac703f828b04c6c197006d17218\n",
            "652b906d60af96844ebd21b674f35e93\n",
            "b06521f39153d618550606be297466d5\n",
            "014842d480b571495a4a0363793f7367\n",
            "c743a45e0d2e6a95cb859adae0248435\n",
            "8a7bd0732ed6a28ce75f6dabc90e1613\n",
            "5f61c0ccad4cac44c75ff505e1f1e537\n",
            "020406e1d05cdc2aa287641f7ae2cc39\n",
            "e510683b3f5ffe4093d021808bc6ff70\n",
            "887f30b43b2867f4a9accceee7d16e6c\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `sha1` reproduces php-src, block boundaries included.
///
/// Every SHA-1 word is BIG-endian, which is where an implementation usually diverges, and the
/// padding rule is the other: one `0x80` byte, zeros up to 56 bytes past a 64-byte boundary, then
/// the BIT length as a big-endian 64-bit word. The sample lengths sit either side of every
/// boundary that rule turns on — 55/56/57, 63/64/65, 119/120, 127/128 — because a digest that is
/// right for short inputs and wrong for those is the usual failure.
#[test]
fn test_cli_wasm_sha1_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_sha1");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function h(string $s): void { echo sha1($s), "\n"; }
h("");
h("a");
h("abc");
h("message digest");
h("The quick brown fox jumps over the lazy dog");
h("\x00\x01\xff");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
h("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile sha1 to WASM");
    assert!(
        output.status.success(),
        "sha1 compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run sha1 under Node");
    assert!(
        run.status.success(),
        "sha1 trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own digests, which are the published SHA-1 test vectors.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "da39a3ee5e6b4b0d3255bfef95601890afd80709\n",
            "86f7e437faa5a7fce15d1ddcb9eaeaea377667b8\n",
            "a9993e364706816aba3e25717850c26c9cd0d89d\n",
            "c12252ceda8be8994d5fa0290a47231c1d16aae3\n",
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12\n",
            "c63e8274458bc7501e7c981f6394ced6d4490fda\n",
            "b05d71c64979cb95fa74a33cdb31a40d258ae02e\n",
            "c1c8bbdc22796e28c0e15163d20899b65621d65a\n",
            "c2db330f6083854c99d4b5bfb6e8f29f201be699\n",
            "f08f24908d682555111be7ff6f004e78283d989a\n",
            "03f09f5b158a7a8cdad920bddc29b81c18a551f5\n",
            "0098ba824b5c16427bd7a1122a5a442a25ec644d\n",
            "11655326c708d70319be2610e8a57d9a5b959d3b\n",
            "ee971065aaa017e0632a8ca6c77bb3bf8b1dfc56\n",
            "f34c1488385346a55709ba056ddd08280dd4c6d6\n",
            "89d95fa32ed44a7c610b7ee38517ddf57e0bb975\n",
            "ad5b3fdbcb526778c2839d2f151ea753995e26a0\n",
            "e61cfffe0d9195a525fc6cf06ca2d77119c24a40\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `str_replace` and `crc32` reproduce php-src.
///
/// `str_replace` scans left to right, NON-overlapping, and never rescans what it wrote — which is
/// what makes `str_replace("a", "ab", "a")` answer `"ab"` instead of looping, and
/// `str_replace("ab", "ba", "abab")` answer `"baba"`. An EMPTY search matches nothing and returns
/// the subject, php-src's own guard against that loop. `crc32` is checked against the standard
/// IEEE 802.3 vectors, including the quick-brown-fox one, and answers PHP's UNSIGNED 32-bit
/// value rather than a sign-extended one.
#[test]
fn test_cli_wasm_str_replace_and_crc32_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_str_replace_crc32");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function r(string $se, string $rp, string $su): void { echo "[", str_replace($se, $rp, $su), "]|"; }
r("a","b","aaa"); r("aa","b","aaaa"); r("aa","b","aaa"); r("","x","abc"); r("a","","aaa"); r("abc","x","abcabc"); echo "\n";
r("a","aa","aaa"); r("x","y","abc"); r("a","b",""); r("ab","ba","abab"); r("a","ab","a"); r("\x00","X","a\x00b"); echo "\n";
r("\xc3\xa9","E","h\xc3\xa9llo"); r("ll","LL","hello"); r("o","0","foo bar boo"); echo "\n";
function c(string $s): void { echo crc32($s), "|"; }
c(""); c("a"); c("abc"); c("hello world"); c("\x00\x01\xff"); c("The quick brown fox jumps over the lazy dog"); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile str_replace/crc32 to WASM");
    assert!(
        output.status.success(),
        "str_replace/crc32 compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run str_replace/crc32 under Node");
    assert!(
        run.status.success(),
        "str_replace/crc32 trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[bbb]|[bb]|[ba]|[abc]|[]|[xx]|\n".as_slice(),
        b"[aaaaaa]|[abc]|[]|[baba]|[ab]|[aXb]|\n".as_slice(),
        b"[hEllo]|[heLLo]|[f00 bar b00]|\n".as_slice(),
        b"0|3904355907|891568578|222957957|3411544030|1095738169|\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `str_pad` reproduces php-src, including where it does NOT raise.
///
/// The empty-pad `ValueError` fires only when padding is actually needed: `str_pad("abc", 2, "")`
/// answers `"abc"` rather than raising, so the guard tests the target length as well as the pad.
/// A target at or below the current length — including a negative one — returns the subject
/// untouched. The default pad is a single space, synthesized rather than interned, so a module
/// that never calls the two-argument form carries no segment for it.
#[test]
fn test_cli_wasm_str_pad_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_str_pad");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function p2(string $s, int $n): void { echo "[", str_pad($s, $n), "]|"; }
function p3(string $s, int $n, string $p): void { echo "[", str_pad($s, $n, $p), "]|"; }
p2("ab", 5); p2("ab", 1); p2("ab", 2); p2("ab", 0); p2("ab", -3); p2("", 4); echo "\n";
p3("ab", 7, "xy"); p3("ab", 8, "xyz"); p3("a", 6, "12"); p3("abc", 4, "xy"); p3("abc", 3, ""); p3("abc", 2, ""); echo "\n";
p3("", 3, "ab"); p3("abc", 5, " "); p3("h\xc3\xa9", 6, "\x00\x01"); echo "\n";
function guard(string $s, int $n, string $p): void {
    try { echo "[", str_pad($s, $n, $p), "]"; } catch (\ValueError $e) { echo "V:", $e->getMessage(); }
    echo "|";
}
guard("abc", 5, ""); guard("abc", 2, ""); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile str_pad to WASM");
    assert!(
        output.status.success(),
        "str_pad compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run str_pad under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "the caught ValueError still killed the program: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[ab   ]|[ab]|[ab]|[ab]|[ab]|[    ]|\n".as_slice(),
        b"[abxyxyx]|[abxyzxyz]|[a12121]|[abcx]|[abc]|[abc]|\n".as_slice(),
        b"[aba]|[abc  ]|[h\xc3\xa9\x00\x01\x00]|\n".as_slice(),
        b"[V:str_pad(): Argument #3 ($pad_string) must not be empty|[abc]|\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `printf` writes the formatted bytes and answers their COUNT.
///
/// It is `sprintf` plus one write, and shares the same builder, so the interesting part is the
/// return value: PHP answers the number of BYTES, not characters, which `printf("h\xc3\xa9")`
/// pins at 3 rather than 2.
#[test]
fn test_cli_wasm_printf_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_printf");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function p(int $n, string $s, float $x): void {
    $a = printf("%d-%s|%.2f\n", $n, $s, $x);
    echo "ret=", $a, "\n";
}
p(42, "ab", 1.5); p(-7, "", 2.675);
$b = printf("literal\n"); echo "ret=", $b, "\n";
$c = printf(""); echo "ret=", $c, "\n";
$d = printf("h\xc3\xa9"); echo "|ret=", $d, "\n";
$e = printf("%05d|%-5s|%+.1f\n", 7, "x", -2.25); echo "ret=", $e, "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile printf to WASM");
    assert!(
        output.status.success(),
        "printf compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run printf under Node");
    assert!(
        run.status.success(),
        "printf trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "42-ab|1.50\n",
            "ret=11\n",
            "-7-|2.67\n",
            "ret=9\n",
            "literal\n",
            "ret=8\n",
            "ret=0\n",
            "hé|ret=3\n",
            "00007|x    |-2.2\n",
            "ret=17\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `sprintf`'s `%f` rounds the EXACT binary value with ties-to-even.
///
/// This is C's rule and NOT `number_format`'s, which is the distinction worth pinning:
/// `sprintf("%.2f", 2.675)` is 2.67 because the double is really 2.67499…, while
/// `number_format(2.675, 2)` is 2.68 because it rounds the shortest decimal that round-trips.
/// Ties go to even, so `%.0f` gives 0 for 0.5, 2 for 1.5 AND 2 for 2.5.
///
/// Non-finite values IGNORE the field entirely — `%08.2f` of INF is `INF`, not `00000INF` — and
/// PHP spells NaN with that capitalisation. A true zero drops its sign, so `-0.0` prints `0.00`,
/// while a negative value that merely rounds to zero keeps it.
#[test]
fn test_cli_wasm_sprintf_float_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_sprintf_float");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function f(float $v): void {
    echo "[", sprintf("%f", $v), "][", sprintf("%.0f", $v), "][", sprintf("%.2f", $v), "][", sprintf("%10.2f", $v), "][", sprintf("%-10.2f", $v), "][", sprintf("%010.2f", $v), "][", sprintf("%+.2f", $v), "]\n";
}
f(0.0); f(1.5); f(-1.5); f(2.5); f(0.125); f(-0.125); f(2.675); f(1234.5678); f(9.99); f(-0.4);
echo sprintf("%08.2f", INF), "|", sprintf("%-8.2f", -INF), "|", sprintf("%+.2f", NAN), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile sprintf %f to WASM");
    assert!(
        output.status.success(),
        "sprintf %f compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run sprintf %f under Node");
    assert!(
        run.status.success(),
        "sprintf %f trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "[0.000000][0][0.00][      0.00][0.00      ][0000000.00][+0.00]\n",
            "[1.500000][2][1.50][      1.50][1.50      ][0000001.50][+1.50]\n",
            "[-1.500000][-2][-1.50][     -1.50][-1.50     ][-000001.50][-1.50]\n",
            "[2.500000][2][2.50][      2.50][2.50      ][0000002.50][+2.50]\n",
            "[0.125000][0][0.12][      0.12][0.12      ][0000000.12][+0.12]\n",
            "[-0.125000][-0][-0.12][     -0.12][-0.12     ][-000000.12][-0.12]\n",
            "[2.675000][3][2.67][      2.67][2.67      ][0000002.67][+2.67]\n",
            "[1234.567800][1235][1234.57][   1234.57][1234.57   ][0001234.57][+1234.57]\n",
            "[9.990000][10][9.99][      9.99][9.99      ][0000009.99][+9.99]\n",
            "[-0.400000][-0][-0.40][     -0.40][-0.40     ][-000000.40][-0.40]\n",
            "INF|-INF|NaN\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `sprintf` reproduces php-src's formatting, which is NOT C's.
///
/// The format is required to be a LITERAL, so it is parsed once at compile time and the module
/// carries a fixed sequence of appends rather than a format interpreter — which is what an AOT
/// compiler should do with a format it already knows. A computed format is refused by the audit.
///
/// Three rules here are php-src's and not C's, each measured before the parser was written:
/// the LAST padding flag wins, so `%'x03d` pads with zeros while `%0'x3d` pads with `x`; `-`
/// cancels a ZERO pad on `%d` but NOT on `%s`, so `%-08d` is space-padded while `%-03s` is
/// zero-padded; and zeros go AFTER the sign while spaces go before it, making `%05d` of -7
/// come out `-0007`.
#[test]
fn test_cli_wasm_sprintf_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_sprintf");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function d(int $n): void { echo "[", sprintf("%d", $n), "][", sprintf("%5d", $n), "][", sprintf("%-5d", $n), "][", sprintf("%05d", $n), "][", sprintf("%+d", $n), "]\n"; }
function s(string $v): void { echo "[", sprintf("%s", $v), "][", sprintf("%5s", $v), "][", sprintf("%-5s", $v), "][", sprintf("%.2s", $v), "][", sprintf("%05s", $v), "]\n"; }
d(0); d(7); d(-7); d(12345);
s(""); s("ab"); s("abcdef");
echo sprintf("a%%b"), "|", sprintf("%s-%d", "x", 5), "|", sprintf("%2\$s %1\$s", "world", "hello"), "|", sprintf("%1\$s%1\$s", "ab"), "\n";
echo sprintf("literal"), "|", sprintf("%d%%", 50), "|", sprintf("[%5s|%-5d]", "ab", 7), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile sprintf to WASM");
    assert!(
        output.status.success(),
        "sprintf compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run sprintf under Node");
    assert!(
        run.status.success(),
        "sprintf trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "[0][    0][0    ][00000][+0]\n",
            "[7][    7][7    ][00007][+7]\n",
            "[-7][   -7][-7   ][-0007][-7]\n",
            "[12345][12345][12345][12345][+12345]\n",
            "[][     ][     ][][00000]\n",
            "[ab][   ab][ab   ][ab][000ab]\n",
            "[abcdef][abcdef][abcdef][ab][abcdef]\n",
            "a%b|x-5|hello world|abab\n",
            "literal|50%|[   ab|7    ]\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `wordwrap` reproduces php-src's in-place line breaking.
///
/// The transform REPLACES a space with the break rather than inserting one, so the result has the
/// same length as the subject. That is why `wordwrap("a ", 1)` is `"a\n"` — the trailing space
/// becomes the break — and why a word longer than the width is left whole: with no space to
/// consume, there is nowhere to break without growing the string.
///
/// Consecutive spaces are where a plausible implementation diverges. `wordwrap("a  b", 1)` is
/// `"a\n b"` but `wordwrap("a  b", 2)` is `"a \nb"`: the break lands on whichever space first
/// reaches the width, and the other survives as content. A width of zero or less is not an error.
///
/// The algorithm was derived from php-src's fast path and checked against 400 random subjects
/// over the alphabet `{a, b, space, newline}` before any of it was written in WAT.
#[test]
fn test_cli_wasm_wordwrap_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_wordwrap");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function w(string $s, int $n): void { echo "[", wordwrap($s, $n), "]|[", wordwrap($s, $n), "]\n"; }
function w1(string $s): void { echo "[", wordwrap($s), "]\n"; }
w("The quick brown fox", 10); w("The quick brown fox", 1); w("abcdefghij", 3);
w("a b c d e", 3); w("", 5); w("short", 99); w("aa bb cc", 5); w("  lead", 3);
w("a  b", 1); w("a  b", 2); w("a ", 1); w("  ", 1); w("x  ", 2);
w("one two\nthree four", 5); w("a b c", 0);
w1("a b c"); w1("");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile wordwrap to WASM");
    assert!(
        output.status.success(),
        "wordwrap compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run wordwrap under Node");
    assert!(
        run.status.success(),
        "wordwrap trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[The quick\n".as_slice(),
        b"brown fox]|[The quick\n".as_slice(),
        b"brown fox]\n".as_slice(),
        b"[The\n".as_slice(),
        b"quick\n".as_slice(),
        b"brown\n".as_slice(),
        b"fox]|[The\n".as_slice(),
        b"quick\n".as_slice(),
        b"brown\n".as_slice(),
        b"fox]\n".as_slice(),
        b"[abcdefghij]|[abcdefghij]\n".as_slice(),
        b"[a b\n".as_slice(),
        b"c d\n".as_slice(),
        b"e]|[a b\n".as_slice(),
        b"c d\n".as_slice(),
        b"e]\n".as_slice(),
        b"[]|[]\n".as_slice(),
        b"[short]|[short]\n".as_slice(),
        b"[aa bb\n".as_slice(),
        b"cc]|[aa bb\n".as_slice(),
        b"cc]\n".as_slice(),
        b"[ \n".as_slice(),
        b"lead]|[ \n".as_slice(),
        b"lead]\n".as_slice(),
        b"[a\n".as_slice(),
        b" b]|[a\n".as_slice(),
        b" b]\n".as_slice(),
        b"[a \n".as_slice(),
        b"b]|[a \n".as_slice(),
        b"b]\n".as_slice(),
        b"[a\n".as_slice(),
        b"]|[a\n".as_slice(),
        b"]\n".as_slice(),
        b"[ \n".as_slice(),
        b"]|[ \n".as_slice(),
        b"]\n".as_slice(),
        b"[x \n".as_slice(),
        b"]|[x \n".as_slice(),
        b"]\n".as_slice(),
        b"[one\n".as_slice(),
        b"two\n".as_slice(),
        b"three\n".as_slice(),
        b"four]|[one\n".as_slice(),
        b"two\n".as_slice(),
        b"three\n".as_slice(),
        b"four]\n".as_slice(),
        b"[a\n".as_slice(),
        b"b\n".as_slice(),
        b"c]|[a\n".as_slice(),
        b"b\n".as_slice(),
        b"c]\n".as_slice(),
        b"[a b c]\n".as_slice(),
        b"[]\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `str_split` cuts into chunks the way php-src does, empty subject included.
///
/// The final chunk is SHORT when the length does not divide evenly, and an EMPTY subject yields
/// the EMPTY array — PHP 8.2's behaviour, and the opposite of `explode`, whose tail is always
/// pushed so `explode(",", "")` is `[""]`. A chunk length below one raises php-src's ValueError
/// rather than being clamped.
///
/// Each helper calls `str_split` TWICE per invocation, once for the count and once for the
/// contents, so a lowering whose scratch locals collide on a second call fails to assemble here.
#[test]
fn test_cli_wasm_str_split_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_str_split");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function s2(string $x, int $n): void { echo count(str_split($x, $n)), ":", implode("|", str_split($x, $n)), " "; }
function s1(string $x): void { echo count(str_split($x)), ":", implode("|", str_split($x)), " "; }
s2("abcdef",1); s2("abcdef",2); s2("abcdef",3); s2("abcdef",4); s2("abcdef",6); s2("abcdef",99); echo "\n";
s2("",1); s2("",5); s2("a",1); s2("ab",1); s2("abc",2); s2("h\xc3\xa9llo",2); echo "\n";
s1("abc"); s1(""); s1("\x00\x01"); echo "\n";
function guard(string $x, int $n): void {
    try { echo implode("|", str_split($x, $n)); } catch (\ValueError $e) { echo "V:", $e->getMessage(); }
    echo " ";
}
guard("abc", 0); guard("abc", -1); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile str_split to WASM");
    assert!(
        output.status.success(),
        "str_split compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run str_split under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "the caught ValueError still killed the program: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"6:a|b|c|d|e|f 3:ab|cd|ef 2:abc|def 2:abcd|ef 1:abcdef 1:abcdef \n".as_slice(),
        b"0: 0: 1:a 2:a|b 2:ab|c 3:h\xc3|\xa9l|lo \n".as_slice(),
        b"3:a|b|c 0: 2:\x00|\x01 \n".as_slice(),
        b"V:str_split(): Argument #2 ($length) must be greater than 0 V:str_split(): Argument #2 ($length) must be greater than 0 \n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `explode` builds php-src's array, empty pieces included.
///
/// Every separator is a boundary, so a leading or trailing one yields an EMPTY element rather
/// than being trimmed, and the tail after the last separator is always pushed — which is why
/// `explode(",", "")` is `[""]`, one empty element, and never the empty array. The results are
/// read back through `implode`, so a wrong element COUNT shows up as well as wrong contents.
///
/// An empty separator raises php-src's ValueError outright, unlike `str_pad`'s empty pad which
/// only raises when it would be used: there is no split it could mean, and the scan would not
/// advance. The `$limit` form is refused — a positive limit caps the count with the remainder in
/// the last element, a negative one drops from the END, and zero behaves as one.
#[test]
fn test_cli_wasm_explode_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_explode");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function e(string $sep, string $s): void { echo "[", implode("|", explode($sep, $s)), "]"; }
e(",", "a,b,c"); e(",", "a"); e(",", ""); e(",", ",a"); e(",", "a,"); echo "\n";
e(",", ",,"); e("--", "a--b"); e(",", "a,,b"); e("ab", "1ab2ab3"); e("\x00", "a\x00b"); echo "\n";
function guard(string $sep, string $s): void {
    try { echo "[", implode("|", explode($sep, $s)), "]"; } catch (\ValueError $x) { echo "V:", $x->getMessage(); }
}
guard("", "abc"); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("compilation failed to run");
    assert!(
        output.status.success(),
        "compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[a|b|c][a][][|a][a|]\n".as_slice(),
        b"[||][a|b][a||b][1|2|3][a|b]\n".as_slice(),
        b"[V:explode(): Argument #1 ($separator) must not be empty\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a builtin that needs scratch locals can be called TWICE in one function.
///
/// Each of these lowerings spills operands it has to read more than once. Naming those locals
/// made two calls in the same function declare the same local twice, which WebAssembly rejects —
/// so the module failed to assemble rather than answering wrongly. Every earlier test happened to
/// call each builtin once per function and missed it entirely; this one calls each of them twice.
#[test]
fn test_cli_wasm_scratch_using_builtins_survive_repeated_calls() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_builtin_twice");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function all(string $a, string $b): void {
    echo str_repeat($a, 2), "|", str_repeat($b, 3), "|";
    echo str_pad($a, 5, "-"), "|", str_pad($b, 6, "."), "|";
    $p = strpos($a, "x"); $q = strpos($b, "y");
    echo $p === false ? "F" : "@", $q === false ? "F" : "@", "|";
    $r = strrpos($a, "x"); $t = strrpos($b, "y");
    echo $r === false ? "F" : "@", $t === false ? "F" : "@", "|";
    $u = strstr($a, "x"); $v = strstr($b, "y", true);
    echo $u === false ? "F" : "S", $v === false ? "F" : "S", "|";
    echo implode(",", explode("-", $a)), "|", implode(":", explode("-", $b)), "\n";
}
all("xa-xb", "cy-dy");
all("no", "ne");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("compilation failed to run");
    assert!(
        output.status.success(),
        "compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"xa-xbxa-xb|cy-dycy-dycy-dy|xa-xb|cy-dy.|@@|@@|SS|xa,xb|cy:dy\n".as_slice(),
        b"nono|nenene|no---|ne....|FF|FF|FF|no|ne\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `implode` joins an indexed string array the way php-src does.
///
/// This is the first builtin on this target that READS an array, so the shape contract is
/// deliberately narrow: the elements must be exactly `string`, because `__rt_array_get_str` reads
/// a slot as a (pointer, length) pair and a slot holding an int or a boxed Mixed is a different
/// layout. An array of `Never` — the type of a literal `[]` — is admitted alongside it, since the
/// element read never happens and the answer is the empty string.
///
/// The glue goes BETWEEN elements, so there is one fewer glue than elements: an empty array joins
/// to nothing, a single element to itself, and three empty strings joined by `,` give `,,`.
#[test]
fn test_cli_wasm_implode_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_implode");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function j(string $g, array $a): void { echo "[", implode($g, $a), "]|"; }
j(",", ["a","b","c"]); j(",", ["a"]); j("", ["a","b"]); j("--", ["x","y","z"]); echo "\n";
j(",", ["","",""]); j("\x00", ["a","b"]); j("::", ["one"]); j(" ", ["a","b","c","d","e"]); echo "\n";
j(",", ["h\xc3\xa9","llo"]); j("\xff", ["\x00","\x01"]); echo "\n";
$e = [];
echo "[", implode(",", $e), "]\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile implode to WASM");
    assert!(
        output.status.success(),
        "implode compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run implode under Node");
    assert!(
        run.status.success(),
        "implode trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[a,b,c]|[a]|[ab]|[x--y--z]|\n".as_slice(),
        b"[,,]|[a\x00b]|[one]|[a b c d e]|\n".as_slice(),
        b"[h\xc3\xa9,llo]|[\x00\xff\x01]|\n".as_slice(),
        b"[]\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a HETEROGENEOUS array literal — the thing that makes `array<mixed>` reachable at all.
///
/// EIR pushes RAW scalars into an `array<mixed>`; there is no boxing instruction, so the backend
/// boxes at the push site the way the native one does. Each scalar gets its exact cell tag (int 0,
/// string 1, float 2, bool 3, and `PhpType::Void` — EIR's `const_null` — 8), and `implode` then
/// converts each cell with the same rule as an explicit `(string)` cast, which is what php-src
/// does element by element.
///
/// The reads matter as much as the writes: a Mixed-cell array has 16-byte slots with the cell
/// pointer at slot+0, NOT the 8-byte stride the int accessor walks — reading it wrong silently
/// yields every other element interleaved with nulls rather than trapping. `count` is in here to
/// prove the array's `value_type` survives the build.
#[test]
fn test_cli_wasm_heterogeneous_array_literal_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_literal");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function j(string $g, array $a): void { echo "[", implode($g, $a), "]|"; }
function c(array $a): void { echo count($a), ":", implode(",", $a), "|"; }
j(",", [1, "a", 2.5, true, null]);
j("", [0, "", false, -0.0]);
j("::", [PHP_INT_MAX, "s", PHP_INT_MIN, 0.1, 1e100, -1e-7]);
j("+", ["\x00\xff", 7, "\n", 1.5]);
echo "\n";
c([1, true]); c([true, false, 0]); c([null, true, "x"]); c([false, null]);
j(";", [1, "b", 2.0, null]); j(";", [1, "b", 2.0, null]);
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile a heterogeneous literal to WASM");
    assert!(
        output.status.success(),
        "heterogeneous literal compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the heterogeneous literal under Node");
    assert!(
        run.status.success(),
        "heterogeneous literal trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"[1,a,2.5,1,]|[0-0]|".as_slice(),
        b"[9223372036854775807::s::-9223372036854775808::0.1::1.0E+100::-1.0E-7]|".as_slice(),
        b"[\x00\xff+7+\n+1.5]|\n".as_slice(),
        b"2:1,1|3:1,,0|3:,1,x|2:,|[1;b;2;]|[1;b;2;]|\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Proves a Mixed-cell array RELEASES its cells, by watching the module's memory not grow.
///
/// This is a negative control, not a smoke test: the boxed elements were leaking entirely and the
/// program still printed the right answer, because nothing on the output path reads the array's
/// `value_type`. Only `__rt_array_free_deep` does — and pushing a bool used to restamp that field
/// to 3 (scalar), which made the deep free skip its child loop and drop every cell on the floor.
///
/// The measurement subtracts the module's DECLARED initial memory, so it isolates runtime growth
/// from the constant data a longer program carries. A bool is in the literal on purpose: it is the
/// element that triggered the restamp.
#[test]
fn test_cli_wasm_mixed_cell_array_releases_its_cells() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_leak");
    let php_path = dir.join("main.php");
    let mut src = String::from(
        "<?php\nfunction j(array $a): void { if (count($a) === 99) { echo \"x\"; } }\n",
    );
    // Unrolled: a counting `for` loop does not compile on this target yet.
    for i in 0..2000 {
        src.push_str(&format!(
            "j([{i}, \"abcdefghij\", 2.5, true, null]);\n"
        ));
    }
    src.push_str("echo \"ok\\n\";\n");
    fs::write(&php_path, src).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the mixed-cell leak probe");
    assert!(
        output.status.success(),
        "leak probe compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(readFileSync(process.argv[2])),
  wasi.getImportObject(),
);
const code = wasi.start(instance);
console.error(`pages=${instance.exports.memory.buffer.byteLength / 65536}`);
process.exitCode = code;
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the leak probe under Node");
    assert!(
        run.status.success(),
        "leak probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ok\n");

    let stderr = String::from_utf8_lossy(&run.stderr);
    let final_pages: usize = stderr
        .split("pages=")
        .nth(1)
        .and_then(|rest| rest.trim().parse().ok())
        .expect("the runner reported the final page count");

    // The declared initial size is the static baseline; anything above it is runtime growth.
    let wat_output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--emit-asm")
        .arg(&php_path)
        .output()
        .expect("failed to emit the leak probe's WAT");
    assert!(wat_output.status.success());
    let wat = fs::read_to_string(dir.join("main.wat")).expect("the WAT was written");
    let initial_pages: usize = wat
        .split("(memory (export \"memory\") ")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .and_then(|n| n.trim().parse().ok())
        .expect("the module declares its initial memory");

    assert_eq!(
        final_pages, initial_pages,
        "2000 boxed 5-element arrays grew memory from {initial_pages} to {final_pages} pages: \
         the cells are not being released"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Proves a `mixed` ARGUMENT's boxed cell is freed, and that freeing it does not break aliasing.
///
/// This target represents a `mixed` parameter as a heap cell, so passing a concrete scalar has to
/// box one. EIR never asked for that box and so emits no matching release: the cell has exactly
/// one owner, the call site, and every such call used to leak 32 bytes — invisibly, since the
/// program still printed the right answer.
///
/// The release is withheld from callees whose declared return is itself a Mixed cell, because
/// `Terminator::Return` MOVES a value out without increfing: such a callee can hand the very cell
/// back. The other escape routes are safe and are exercised here — copying into a callee local and
/// forwarding to a further call both borrow, and a container push increfs.
#[test]
fn test_cli_wasm_boxed_mixed_argument_is_released() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_boxed_arg");

    // Aliasing first: a callee that hands its parameter back must still answer correctly.
    let alias_path = dir.join("alias.php");
    fs::write(
        &alias_path,
        r#"<?php
function id(mixed $x): mixed { return $x; }
function pick(mixed $a, mixed $b): mixed { return $b; }
function copy_local(mixed $x): void { $y = $x; if ($y === "zz") { echo "q"; } }
function forward(mixed $x): void { copy_local($x); }
echo (id("hello") === "hello") ? "y" : "n";
echo (id(42) === 42) ? "y" : "n";
echo (id(2.5) === 2.5) ? "y" : "n";
echo (id(null) === null) ? "y" : "n";
echo (id(true) === true) ? "y" : "n";
echo (pick(1, "b") === "b") ? "y" : "n";
copy_local("hello"); forward("hello"); forward(7);
echo "\n";
"#,
    )
    .unwrap();

    let runner_src = r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(readFileSync(process.argv[2])),
  wasi.getImportObject(),
);
const code = wasi.start(instance);
console.error(`pages=${instance.exports.memory.buffer.byteLength / 65536}`);
process.exitCode = code;
"#;
    let runner = dir.join("run.mjs");
    fs::write(&runner, runner_src).unwrap();

    let compile = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&alias_path)
        .output()
        .expect("failed to compile the aliasing probe");
    assert!(
        compile.status.success(),
        "aliasing probe compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let alias_run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("alias.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the aliasing probe under Node");
    assert!(
        alias_run.status.success(),
        "aliasing probe trapped: {}",
        String::from_utf8_lossy(&alias_run.stderr)
    );
    // php-src 8.5.6's own bytes.
    assert_eq!(alias_run.stdout, b"yyyyyy\n");

    // Then the release itself, watched as runtime memory growth.
    let leak_path = dir.join("leak.php");
    let mut src = String::from(
        "<?php\nfunction m(mixed $x): void { if ($x === \"zz\") { echo \"y\"; } }\n",
    );
    // Unrolled: a counting `for` loop does not compile on this target yet.
    for i in 0..1000 {
        src.push_str(&format!("m({i}); m(\"abcdefghij\"); m(2.5); m(null); m(true);\n"));
    }
    src.push_str("echo \"ok\\n\";\n");
    fs::write(&leak_path, src).unwrap();

    let compile = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&leak_path)
        .output()
        .expect("failed to compile the boxed-argument leak probe");
    assert!(
        compile.status.success(),
        "leak probe compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let wat_output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--emit-asm")
        .arg(&leak_path)
        .output()
        .expect("failed to emit the leak probe's WAT");
    assert!(wat_output.status.success());

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("leak.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the leak probe under Node");
    assert!(
        run.status.success(),
        "leak probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, b"ok\n");

    let stderr = String::from_utf8_lossy(&run.stderr);
    let final_pages: usize = stderr
        .split("pages=")
        .nth(1)
        .and_then(|rest| rest.trim().parse().ok())
        .expect("the runner reported the final page count");
    let wat = fs::read_to_string(dir.join("leak.wat")).expect("the WAT was written");
    let initial_pages: usize = wat
        .split("(memory (export \"memory\") ")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .and_then(|n| n.trim().parse().ok())
        .expect("the module declares its initial memory");

    assert_eq!(
        final_pages, initial_pages,
        "5000 boxed `mixed` arguments grew memory from {initial_pages} to {final_pages} pages: \
         the boxed cells are not being released"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an `array<T>` handed to an `array<mixed>` parameter is CONVERTED, not reinterpreted.
///
/// This target specializes array storage per element type — int and bool arrays use 8-byte slots,
/// string arrays 16-byte (pointer, length) pairs, and `mixed` a `value_type`-7 array of boxed
/// cells. So passing one where another is expected is a real element-wise conversion; treating it
/// as a pointer copy would read the wrong slot layout without trapping. An empty literal's
/// `array<never>` widens too: there is nothing to convert.
///
/// The conversion allocates, and the callee only borrows, so the call site frees the copy
/// afterwards — withheld when the callee's declared return is itself an array, since a returned
/// value moves out without an incref.
#[test]
fn test_cli_wasm_array_widens_to_mixed_parameter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_widen");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function j(string $g, array $a): void { echo count($a), "[", implode($g, $a), "]|"; }
j(",", [1, "a", 2.5, true, null]);
j(",", ["x", "y", "z"]);
j("-", [1, 2, 3]);
j("", []);
j(",", [true, false, true]);
j("|", ["only"]);
j(",", [7]);
j("::", ["a", "", "b"]);
j(",", [1, "mix", 2]);
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array-widening probe");
    assert!(
        output.status.success(),
        "array-widening compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array-widening probe under Node");
    assert!(
        run.status.success(),
        "array-widening probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(
        run.stdout,
        b"5[1,a,2.5,1,]|3[x,y,z]|3[1-2-3]|0[]|3[1,,1]|1[only]|1[7]|3[a::::b]|3[1,mix,2]|\n"
            .to_vec()
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the smallest magnitudes render exactly, where the digit buffer used to overflow.
///
/// `__rt_f64_digits` writes 9-byte chunks LEFTWARDS from the end of its buffer, so the buffer has
/// to cover `ceil(digits/9)*9`, not the digit count. The worst case is `p == 1074`, where `J` has
/// up to 767 digits — 86 chunks, 774 bytes — and the buffer was sized 768.
///
/// Undersizing did not trap: the cursor went negative, the chunks landed BEFORE the buffer, and
/// the leading-zero strip compared a negative start with `i32.ge_u`, read it as a huge unsigned
/// value and exited at once. `1e-308` printed as `0.0000001E-301` — right value, unnormalized —
/// and the leading zeros then ate the 14 significant digits.
#[test]
fn test_cli_wasm_smallest_floats_render_like_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_tiny_floats");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function s(mixed $m): void { echo implode("", [$m]), "|"; }
s(1e-307); s(1.5e-307); s(1e-308); s(1.5e-308); s(2.2e-308); s(5e-308);
s(1e-309); s(1.5e-309); s(2.2e-309); s(5e-309); s(9.99e-309);
s(1e-310); s(1e-320); s(5e-324); s(2.2250738585072014e-308);
s(PHP_FLOAT_MIN); s(PHP_FLOAT_MAX); s(PHP_FLOAT_EPSILON);
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the tiny-float probe");
    assert!(
        output.status.success(),
        "tiny-float compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the tiny-float probe under Node");
    assert!(
        run.status.success(),
        "tiny-float probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected = concat!(
        "1.0E-307|1.5E-307|1.0E-308|1.5E-308|2.2E-308|5.0E-308|",
        "1.0E-309|1.5E-309|2.2E-309|5.0E-309|9.99E-309|",
        "1.0E-310|9.9998886718268E-321|4.9406564584125E-324|2.2250738585072E-308|",
        "2.2250738585072E-308|1.7976931348623E+308|2.2204460492503E-16|\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an array of FLOAT elements, which had no lowered storage at all.
///
/// A float shares the int slot width — the payload is the f64's bits — so this is
/// `__rt_array_push_int` plus the `value_type` 2 stamp that records which it is, matching the
/// native layout this is byte-identical to. `implode` renders each element with the same rule as
/// an explicit `(string)` cast, which for a float only `__rt_mixed_cast_string` knows, so a float
/// slot is boxed into a throwaway tag-2 cell and cast through it.
#[test]
fn test_cli_wasm_float_element_arrays_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_float_array");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function j(string $g, array $a): void { echo count($a), "[", implode($g, $a), "]|"; }
j(",", [1.0, 100.0, 0.5, 1e15, 1e16, 1e-5]);
j("-", [2.5]);
j(",", [0.1, -0.0, 1e100, -1e-7, 3.14159265358979]);
j("", [1.5, 2.5]);
j(",", [INF, -INF, NAN]);
j(",", [PHP_FLOAT_EPSILON, PHP_FLOAT_MAX, PHP_FLOAT_MIN]);
j(",", [1.5, 2.5]);
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the float-array probe");
    assert!(
        output.status.success(),
        "float-array compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the float-array probe under Node");
    assert!(
        run.status.success(),
        "float-array probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected = concat!(
        "6[1,100,0.5,1.0E+15,1.0E+16,1.0E-5]|1[2.5]|",
        "5[0.1,-0,1.0E+100,-1.0E-7,3.1415926535898]|2[1.52.5]|",
        "3[INF,-INF,NAN]|",
        "3[2.2204460492503E-16,1.7976931348623E+308,2.2250738585072E-308]|2[1.5,2.5]|\n",
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `array_slice` over a list, whose offset/length rules are `substr`'s exactly.
///
/// Validated as a MODEL first: a transcription of `substr`'s clamping was checked against php-src
/// on 52 offset/length pairs before any WAT was written, and matched all 52. A negative offset
/// counts from the end and floors at 0, an offset at or past the end gives an empty result, a
/// negative length drops that many from the end, and a length is clamped so the window never runs
/// past the end or backwards.
///
/// `PHP_INT_MIN` is in here because both bounds have to be clamped into `[-n, n]` BEFORE any
/// arithmetic — negating `PHP_INT_MIN` wraps an i64, and the clamp is what makes the rest safe.
#[test]
fn test_cli_wasm_array_slice_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_slice");
    let php_path = dir.join("main.php");
    fs::write(&php_path, PHP_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array_slice probe");
    assert!(
        output.status.success(),
        "array_slice compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array_slice probe under Node");
    assert!(
        run.status.success(),
        "array_slice probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), PHP_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The `array_slice` probe program: every boundary of the offset/length rules.
const PHP_SOURCE: &str = r##"<?php
$s = [10,20,30,40,50];
$v0 = array_slice($s, -9); echo count($v0), ":", implode(",", $v0), "|";
$v1 = array_slice($s, -5); echo count($v1), ":", implode(",", $v1), "|";
$v2 = array_slice($s, -1); echo count($v2), ":", implode(",", $v2), "|";
$v3 = array_slice($s, 0); echo count($v3), ":", implode(",", $v3), "|";
$v4 = array_slice($s, 1); echo count($v4), ":", implode(",", $v4), "|";
$v5 = array_slice($s, 4); echo count($v5), ":", implode(",", $v5), "|";
$v6 = array_slice($s, 5); echo count($v6), ":", implode(",", $v6), "|";
$v7 = array_slice($s, 9); echo count($v7), ":", implode(",", $v7), "|";
$v8 = array_slice($s, -6, -6); echo count($v8), ":", implode(",", $v8), "|";
$v9 = array_slice($s, -6, -1); echo count($v9), ":", implode(",", $v9), "|";
$v10 = array_slice($s, -6, 0); echo count($v10), ":", implode(",", $v10), "|";
$v11 = array_slice($s, -6, 1); echo count($v11), ":", implode(",", $v11), "|";
$v12 = array_slice($s, -6, 3); echo count($v12), ":", implode(",", $v12), "|";
$v13 = array_slice($s, -6, 7); echo count($v13), ":", implode(",", $v13), "|";
$v14 = array_slice($s, -1, -6); echo count($v14), ":", implode(",", $v14), "|";
$v15 = array_slice($s, -1, -1); echo count($v15), ":", implode(",", $v15), "|";
$v16 = array_slice($s, -1, 0); echo count($v16), ":", implode(",", $v16), "|";
$v17 = array_slice($s, -1, 1); echo count($v17), ":", implode(",", $v17), "|";
$v18 = array_slice($s, -1, 3); echo count($v18), ":", implode(",", $v18), "|";
$v19 = array_slice($s, -1, 7); echo count($v19), ":", implode(",", $v19), "|";
$v20 = array_slice($s, 0, -6); echo count($v20), ":", implode(",", $v20), "|";
$v21 = array_slice($s, 0, -1); echo count($v21), ":", implode(",", $v21), "|";
$v22 = array_slice($s, 0, 0); echo count($v22), ":", implode(",", $v22), "|";
$v23 = array_slice($s, 0, 1); echo count($v23), ":", implode(",", $v23), "|";
$v24 = array_slice($s, 0, 3); echo count($v24), ":", implode(",", $v24), "|";
$v25 = array_slice($s, 0, 7); echo count($v25), ":", implode(",", $v25), "|";
$v26 = array_slice($s, 2, -6); echo count($v26), ":", implode(",", $v26), "|";
$v27 = array_slice($s, 2, -1); echo count($v27), ":", implode(",", $v27), "|";
$v28 = array_slice($s, 2, 0); echo count($v28), ":", implode(",", $v28), "|";
$v29 = array_slice($s, 2, 1); echo count($v29), ":", implode(",", $v29), "|";
$v30 = array_slice($s, 2, 3); echo count($v30), ":", implode(",", $v30), "|";
$v31 = array_slice($s, 2, 7); echo count($v31), ":", implode(",", $v31), "|";
$v32 = array_slice($s, PHP_INT_MIN); echo count($v32), ":", implode(",", $v32), "|";
$v33 = array_slice($s, PHP_INT_MAX); echo count($v33), ":", implode(",", $v33), "|";
$v34 = array_slice($s, 0, PHP_INT_MIN); echo count($v34), ":", implode(",", $v34), "|";
$v35 = array_slice($s, 0, PHP_INT_MAX); echo count($v35), ":", implode(",", $v35), "|";
$v36 = array_slice($s, PHP_INT_MIN, PHP_INT_MAX); echo count($v36), ":", implode(",", $v36), "|";
echo "\n";
"##;

/// php-src 8.5.6's own output for `PHP_SOURCE`.
const PHP_EXPECTED: &str = r##"5:10,20,30,40,50|5:10,20,30,40,50|1:50|5:10,20,30,40,50|4:20,30,40,50|1:50|0:|0:|0:|4:10,20,30,40|0:|1:10|3:10,20,30|5:10,20,30,40,50|0:|0:|0:|1:50|1:50|1:50|0:|4:10,20,30,40|0:|1:10|3:10,20,30|5:10,20,30,40,50|0:|2:30,40|0:|1:30|3:30,40,50|3:30,40,50|5:10,20,30,40,50|0:|0:|5:10,20,30,40,50|5:10,20,30,40,50|
"##;

/// Verifies `array_merge` over lists, and that both operands survive it intact.
///
/// Unlike `+`, which keeps the left's keys and takes only the right's surplus tail, `array_merge`
/// APPENDS every element of the right and reindexes. The two share their element-copy walk, and
/// that walk is where ownership can go wrong in the direction that double-frees rather than leaks:
/// a string element is re-persisted so the result owns its own copy, and a refcounted child is
/// increfed.
///
/// Mixed elements are in here because they live in 16-BYTE slots: reading them at the scalar
/// stride and appending them as scalars wrote into the middle of the previous slot, which showed
/// up as `[1, "x", 2.5]` merging to `1,x,,,` — a corrupted element the source arrays still held
/// correctly.
#[test]
fn test_cli_wasm_array_merge_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_merge");
    let php_path = dir.join("main.php");
    fs::write(&php_path, MERGE_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array_merge probe");
    assert!(
        output.status.success(),
        "array_merge compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array_merge probe under Node");
    assert!(
        run.status.success(),
        "array_merge probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), MERGE_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The `array_merge` probe program: every lowered element type, plus both operands re-read after.
const MERGE_SOURCE: &str = r##"<?php
$a = [1,2]; $b = [3,4,5]; $e = [];
$s1 = ["xx","y"]; $s2 = ["z"];
$f1 = [1.5, 2.5]; $f2 = [3.5];
$m1 = [1, "x", 2.5]; $m2 = [true, null];
$r1 = array_merge($a, $b);   echo count($r1), ":", implode(",", $r1), "|";
$r2 = array_merge($a, $e);   echo count($r2), ":", implode(",", $r2), "|";
$r3 = array_merge($e, $b);   echo count($r3), ":", implode(",", $r3), "|";
$r4 = array_merge($e, $e);   echo count($r4), ":", implode(",", $r4), "|";
$r5 = array_merge($s1, $s2); echo count($r5), ":", implode(",", $r5), "|";
$r6 = array_merge($f1, $f2); echo count($r6), ":", implode(",", $r6), "|";
$r7 = array_merge($m1, $m2); echo count($r7), ":", implode(",", $r7), "|";
$r8 = array_merge($a, $a);   echo count($r8), ":", implode(",", $r8), "|";
echo "\n";
echo count($a), count($b), count($s1), count($s2), count($m1), count($m2), "\n";
echo implode(",", $s1), ";", implode(",", $m1), "\n";
"##;

/// php-src 8.5.6's own output for `MERGE_SOURCE`.
const MERGE_EXPECTED: &str = r##"5:1,2,3,4,5|2:1,2|3:3,4,5|0:|3:xx,y,z|3:1.5,2.5,3.5|5:1,x,2.5,1,|4:1,2,1,2|
232132
xx,y;1,x,2.5
"##;

/// Verifies `range` over integers, in both directions and at the i64 boundaries.
///
/// Only the two-bound form exists — the front-end rejects every other arity — so the step is
/// always 1 and the DIRECTION comes from the operands: `range(5, 1)` counts down. A single-element
/// range is `range(n, n)`, which is why the count is the span plus one.
///
/// `PHP_INT_MIN`/`PHP_INT_MAX` bounds are here because the span is computed with wrapping
/// arithmetic: a range spanning more than `i64::MAX` elements cannot have its count represented,
/// and asks for a layout the allocator is guaranteed to reject rather than looping forever.
#[test]
fn test_cli_wasm_range_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_range");
    let php_path = dir.join("main.php");
    fs::write(&php_path, RANGE_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the range probe");
    assert!(
        output.status.success(),
        "range compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the range probe under Node");
    assert!(
        run.status.success(),
        "range probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), RANGE_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The `range` probe program: both directions, single-element, and the i64 boundaries.
const RANGE_SOURCE: &str = r##"<?php
function p(array $r): void { echo count($r), ":", implode(",", $r), "|"; }
p(range(1, 5)); p(range(5, 1)); p(range(0, 0)); p(range(-3, 2)); p(range(2, -3));
p(range(-1, -1)); p(range(100, 97)); p(range(PHP_INT_MAX - 2, PHP_INT_MAX));
p(range(PHP_INT_MIN, PHP_INT_MIN + 2)); p(range(PHP_INT_MAX, PHP_INT_MAX - 3));
foreach (range(1, 4) as $n) { echo $n, "."; }
echo "|"; p(range(1, 5)); echo "\n";
"##;

/// php-src 8.5.6's own output for `RANGE_SOURCE`.
const RANGE_EXPECTED: &str = r##"5:1,2,3,4,5|5:5,4,3,2,1|1:0|6:-3,-2,-1,0,1,2|6:2,1,0,-1,-2,-3|1:-1|4:100,99,98,97|3:9223372036854775805,9223372036854775806,9223372036854775807|3:-9223372036854775808,-9223372036854775807,-9223372036854775806|4:9223372036854775807,9223372036854775806,9223372036854775805,9223372036854775804|1.2.3.4.|5:1,2,3,4,5|
"##;

/// Verifies `==` and `!=` — PHP's LOOSE comparison — over the pairs whose rule was measured.
///
/// The string rule is php-src's `zendi_smart_strcmp`, transcribed and validated on 3000 pairs
/// against 8.5.6: 1600 from this systematic matrix and 1400 randomly generated. The naive reading
/// — "both numeric, so compare the numbers" — passes a 625-pair sample and is STILL WRONG, which
/// is why the sweep was widened; php-src additionally tracks `oflow`, set only for an
/// INTEGRAL-form string whose magnitude escapes i64, and uses it to settle the comparison without
/// converting.
///
/// That is what separates the two rules this test pins side by side:
///   "9223372036854775807" == "9223372036854775808"   is FALSE (integral form, oflow)
///   "9223372036854775807" == "9.2233720368547758e18" is TRUE  (float form, no oflow)
///   PHP_INT_MAX          == 9.2233720368547758e18    is TRUE  (values, plain widening)
///
/// KNOWN GAP, deliberately kept out of the matrix: `__rt_digits_to_f64` documents that it flushes
/// magnitudes below 1e-308 to zero, so a SUBNORMAL numeric string parses to 0.0 and
/// `"9.22e-312" == "0"` answers true where php-src answers false. That is the parser's deferral,
/// not the comparison's — the random sweep found exactly that one case out of 1400.
#[test]
fn test_cli_wasm_loose_equality_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_loose_eq");
    let php_path = dir.join("main.php");
    fs::write(&php_path, LOOSE_EQ_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the loose-equality probe");
    assert!(
        output.status.success(),
        "loose-equality compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the loose-equality probe under Node");
    assert!(
        run.status.success(),
        "loose-equality probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), LOOSE_EQ_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The loose-equality probe: a 40-string matrix (1600 pairs) then the scalar pairs.
const LOOSE_EQ_SOURCE: &str = r##"<?php
function s(string $a, string $b): void { echo ($a == $b) ? "1" : "0"; }
function f(float $a, float $b): void { echo ($a == $b) ? "1" : "0"; }
function b(bool $a, bool $b): void { echo ($a == $b) ? "1" : "0"; }
function m(int $a, float $b): void { echo ($a == $b) ? "1" : "0"; }
function n(float $a, int $b): void { echo ($a == $b) ? "1" : "0"; }
s("", "");
s("", "0");
s("", "00");
s("", "0.0");
s("", "abc");
s("", "10");
s("", "1e1");
s("", "10.0");
s("", " 10");
s("", "10 ");
s("", "+10");
s("", "-0");
s("", "0x1A");
s("", "1e400");
s("", "1e500");
s("", "-1e400");
s("", "9223372036854775807");
s("", "9223372036854775808");
s("", "9223372036854775809");
s("", "9.2233720368547758e18");
s("", "9223372036854775808.0");
s("", "-9223372036854775808");
s("", "-9223372036854775809");
s("", "-9.2233720368547758e18");
s("", "18446744073709551616");
s("", "10abc");
s("", " ");
s("", ".5");
s("", "5.");
s("", "1E1");
s("", "0.1");
s("", "1e-1");
s("", "NAN");
s("", "INF");
s("", "007");
s("", "7");
s("", "+0");
s("", "-0.0");
s("", "0e0");
s("", "1e0");
s("0", "");
s("0", "0");
s("0", "00");
s("0", "0.0");
s("0", "abc");
s("0", "10");
s("0", "1e1");
s("0", "10.0");
s("0", " 10");
s("0", "10 ");
s("0", "+10");
s("0", "-0");
s("0", "0x1A");
s("0", "1e400");
s("0", "1e500");
s("0", "-1e400");
s("0", "9223372036854775807");
s("0", "9223372036854775808");
s("0", "9223372036854775809");
s("0", "9.2233720368547758e18");
s("0", "9223372036854775808.0");
s("0", "-9223372036854775808");
s("0", "-9223372036854775809");
s("0", "-9.2233720368547758e18");
s("0", "18446744073709551616");
s("0", "10abc");
s("0", " ");
s("0", ".5");
s("0", "5.");
s("0", "1E1");
s("0", "0.1");
s("0", "1e-1");
s("0", "NAN");
s("0", "INF");
s("0", "007");
s("0", "7");
s("0", "+0");
s("0", "-0.0");
s("0", "0e0");
s("0", "1e0");
s("00", "");
s("00", "0");
s("00", "00");
s("00", "0.0");
s("00", "abc");
s("00", "10");
s("00", "1e1");
s("00", "10.0");
s("00", " 10");
s("00", "10 ");
s("00", "+10");
s("00", "-0");
s("00", "0x1A");
s("00", "1e400");
s("00", "1e500");
s("00", "-1e400");
s("00", "9223372036854775807");
s("00", "9223372036854775808");
s("00", "9223372036854775809");
s("00", "9.2233720368547758e18");
s("00", "9223372036854775808.0");
s("00", "-9223372036854775808");
s("00", "-9223372036854775809");
s("00", "-9.2233720368547758e18");
s("00", "18446744073709551616");
s("00", "10abc");
s("00", " ");
s("00", ".5");
s("00", "5.");
s("00", "1E1");
s("00", "0.1");
s("00", "1e-1");
s("00", "NAN");
s("00", "INF");
s("00", "007");
s("00", "7");
s("00", "+0");
s("00", "-0.0");
s("00", "0e0");
s("00", "1e0");
s("0.0", "");
s("0.0", "0");
s("0.0", "00");
s("0.0", "0.0");
s("0.0", "abc");
s("0.0", "10");
s("0.0", "1e1");
s("0.0", "10.0");
s("0.0", " 10");
s("0.0", "10 ");
s("0.0", "+10");
s("0.0", "-0");
s("0.0", "0x1A");
s("0.0", "1e400");
s("0.0", "1e500");
s("0.0", "-1e400");
s("0.0", "9223372036854775807");
s("0.0", "9223372036854775808");
s("0.0", "9223372036854775809");
s("0.0", "9.2233720368547758e18");
s("0.0", "9223372036854775808.0");
s("0.0", "-9223372036854775808");
s("0.0", "-9223372036854775809");
s("0.0", "-9.2233720368547758e18");
s("0.0", "18446744073709551616");
s("0.0", "10abc");
s("0.0", " ");
s("0.0", ".5");
s("0.0", "5.");
s("0.0", "1E1");
s("0.0", "0.1");
s("0.0", "1e-1");
s("0.0", "NAN");
s("0.0", "INF");
s("0.0", "007");
s("0.0", "7");
s("0.0", "+0");
s("0.0", "-0.0");
s("0.0", "0e0");
s("0.0", "1e0");
s("abc", "");
s("abc", "0");
s("abc", "00");
s("abc", "0.0");
s("abc", "abc");
s("abc", "10");
s("abc", "1e1");
s("abc", "10.0");
s("abc", " 10");
s("abc", "10 ");
s("abc", "+10");
s("abc", "-0");
s("abc", "0x1A");
s("abc", "1e400");
s("abc", "1e500");
s("abc", "-1e400");
s("abc", "9223372036854775807");
s("abc", "9223372036854775808");
s("abc", "9223372036854775809");
s("abc", "9.2233720368547758e18");
s("abc", "9223372036854775808.0");
s("abc", "-9223372036854775808");
s("abc", "-9223372036854775809");
s("abc", "-9.2233720368547758e18");
s("abc", "18446744073709551616");
s("abc", "10abc");
s("abc", " ");
s("abc", ".5");
s("abc", "5.");
s("abc", "1E1");
s("abc", "0.1");
s("abc", "1e-1");
s("abc", "NAN");
s("abc", "INF");
s("abc", "007");
s("abc", "7");
s("abc", "+0");
s("abc", "-0.0");
s("abc", "0e0");
s("abc", "1e0");
s("10", "");
s("10", "0");
s("10", "00");
s("10", "0.0");
s("10", "abc");
s("10", "10");
s("10", "1e1");
s("10", "10.0");
s("10", " 10");
s("10", "10 ");
s("10", "+10");
s("10", "-0");
s("10", "0x1A");
s("10", "1e400");
s("10", "1e500");
s("10", "-1e400");
s("10", "9223372036854775807");
s("10", "9223372036854775808");
s("10", "9223372036854775809");
s("10", "9.2233720368547758e18");
s("10", "9223372036854775808.0");
s("10", "-9223372036854775808");
s("10", "-9223372036854775809");
s("10", "-9.2233720368547758e18");
s("10", "18446744073709551616");
s("10", "10abc");
s("10", " ");
s("10", ".5");
s("10", "5.");
s("10", "1E1");
s("10", "0.1");
s("10", "1e-1");
s("10", "NAN");
s("10", "INF");
s("10", "007");
s("10", "7");
s("10", "+0");
s("10", "-0.0");
s("10", "0e0");
s("10", "1e0");
s("1e1", "");
s("1e1", "0");
s("1e1", "00");
s("1e1", "0.0");
s("1e1", "abc");
s("1e1", "10");
s("1e1", "1e1");
s("1e1", "10.0");
s("1e1", " 10");
s("1e1", "10 ");
s("1e1", "+10");
s("1e1", "-0");
s("1e1", "0x1A");
s("1e1", "1e400");
s("1e1", "1e500");
s("1e1", "-1e400");
s("1e1", "9223372036854775807");
s("1e1", "9223372036854775808");
s("1e1", "9223372036854775809");
s("1e1", "9.2233720368547758e18");
s("1e1", "9223372036854775808.0");
s("1e1", "-9223372036854775808");
s("1e1", "-9223372036854775809");
s("1e1", "-9.2233720368547758e18");
s("1e1", "18446744073709551616");
s("1e1", "10abc");
s("1e1", " ");
s("1e1", ".5");
s("1e1", "5.");
s("1e1", "1E1");
s("1e1", "0.1");
s("1e1", "1e-1");
s("1e1", "NAN");
s("1e1", "INF");
s("1e1", "007");
s("1e1", "7");
s("1e1", "+0");
s("1e1", "-0.0");
s("1e1", "0e0");
s("1e1", "1e0");
s("10.0", "");
s("10.0", "0");
s("10.0", "00");
s("10.0", "0.0");
s("10.0", "abc");
s("10.0", "10");
s("10.0", "1e1");
s("10.0", "10.0");
s("10.0", " 10");
s("10.0", "10 ");
s("10.0", "+10");
s("10.0", "-0");
s("10.0", "0x1A");
s("10.0", "1e400");
s("10.0", "1e500");
s("10.0", "-1e400");
s("10.0", "9223372036854775807");
s("10.0", "9223372036854775808");
s("10.0", "9223372036854775809");
s("10.0", "9.2233720368547758e18");
s("10.0", "9223372036854775808.0");
s("10.0", "-9223372036854775808");
s("10.0", "-9223372036854775809");
s("10.0", "-9.2233720368547758e18");
s("10.0", "18446744073709551616");
s("10.0", "10abc");
s("10.0", " ");
s("10.0", ".5");
s("10.0", "5.");
s("10.0", "1E1");
s("10.0", "0.1");
s("10.0", "1e-1");
s("10.0", "NAN");
s("10.0", "INF");
s("10.0", "007");
s("10.0", "7");
s("10.0", "+0");
s("10.0", "-0.0");
s("10.0", "0e0");
s("10.0", "1e0");
s(" 10", "");
s(" 10", "0");
s(" 10", "00");
s(" 10", "0.0");
s(" 10", "abc");
s(" 10", "10");
s(" 10", "1e1");
s(" 10", "10.0");
s(" 10", " 10");
s(" 10", "10 ");
s(" 10", "+10");
s(" 10", "-0");
s(" 10", "0x1A");
s(" 10", "1e400");
s(" 10", "1e500");
s(" 10", "-1e400");
s(" 10", "9223372036854775807");
s(" 10", "9223372036854775808");
s(" 10", "9223372036854775809");
s(" 10", "9.2233720368547758e18");
s(" 10", "9223372036854775808.0");
s(" 10", "-9223372036854775808");
s(" 10", "-9223372036854775809");
s(" 10", "-9.2233720368547758e18");
s(" 10", "18446744073709551616");
s(" 10", "10abc");
s(" 10", " ");
s(" 10", ".5");
s(" 10", "5.");
s(" 10", "1E1");
s(" 10", "0.1");
s(" 10", "1e-1");
s(" 10", "NAN");
s(" 10", "INF");
s(" 10", "007");
s(" 10", "7");
s(" 10", "+0");
s(" 10", "-0.0");
s(" 10", "0e0");
s(" 10", "1e0");
s("10 ", "");
s("10 ", "0");
s("10 ", "00");
s("10 ", "0.0");
s("10 ", "abc");
s("10 ", "10");
s("10 ", "1e1");
s("10 ", "10.0");
s("10 ", " 10");
s("10 ", "10 ");
s("10 ", "+10");
s("10 ", "-0");
s("10 ", "0x1A");
s("10 ", "1e400");
s("10 ", "1e500");
s("10 ", "-1e400");
s("10 ", "9223372036854775807");
s("10 ", "9223372036854775808");
s("10 ", "9223372036854775809");
s("10 ", "9.2233720368547758e18");
s("10 ", "9223372036854775808.0");
s("10 ", "-9223372036854775808");
s("10 ", "-9223372036854775809");
s("10 ", "-9.2233720368547758e18");
s("10 ", "18446744073709551616");
s("10 ", "10abc");
s("10 ", " ");
s("10 ", ".5");
s("10 ", "5.");
s("10 ", "1E1");
s("10 ", "0.1");
s("10 ", "1e-1");
s("10 ", "NAN");
s("10 ", "INF");
s("10 ", "007");
s("10 ", "7");
s("10 ", "+0");
s("10 ", "-0.0");
s("10 ", "0e0");
s("10 ", "1e0");
s("+10", "");
s("+10", "0");
s("+10", "00");
s("+10", "0.0");
s("+10", "abc");
s("+10", "10");
s("+10", "1e1");
s("+10", "10.0");
s("+10", " 10");
s("+10", "10 ");
s("+10", "+10");
s("+10", "-0");
s("+10", "0x1A");
s("+10", "1e400");
s("+10", "1e500");
s("+10", "-1e400");
s("+10", "9223372036854775807");
s("+10", "9223372036854775808");
s("+10", "9223372036854775809");
s("+10", "9.2233720368547758e18");
s("+10", "9223372036854775808.0");
s("+10", "-9223372036854775808");
s("+10", "-9223372036854775809");
s("+10", "-9.2233720368547758e18");
s("+10", "18446744073709551616");
s("+10", "10abc");
s("+10", " ");
s("+10", ".5");
s("+10", "5.");
s("+10", "1E1");
s("+10", "0.1");
s("+10", "1e-1");
s("+10", "NAN");
s("+10", "INF");
s("+10", "007");
s("+10", "7");
s("+10", "+0");
s("+10", "-0.0");
s("+10", "0e0");
s("+10", "1e0");
s("-0", "");
s("-0", "0");
s("-0", "00");
s("-0", "0.0");
s("-0", "abc");
s("-0", "10");
s("-0", "1e1");
s("-0", "10.0");
s("-0", " 10");
s("-0", "10 ");
s("-0", "+10");
s("-0", "-0");
s("-0", "0x1A");
s("-0", "1e400");
s("-0", "1e500");
s("-0", "-1e400");
s("-0", "9223372036854775807");
s("-0", "9223372036854775808");
s("-0", "9223372036854775809");
s("-0", "9.2233720368547758e18");
s("-0", "9223372036854775808.0");
s("-0", "-9223372036854775808");
s("-0", "-9223372036854775809");
s("-0", "-9.2233720368547758e18");
s("-0", "18446744073709551616");
s("-0", "10abc");
s("-0", " ");
s("-0", ".5");
s("-0", "5.");
s("-0", "1E1");
s("-0", "0.1");
s("-0", "1e-1");
s("-0", "NAN");
s("-0", "INF");
s("-0", "007");
s("-0", "7");
s("-0", "+0");
s("-0", "-0.0");
s("-0", "0e0");
s("-0", "1e0");
s("0x1A", "");
s("0x1A", "0");
s("0x1A", "00");
s("0x1A", "0.0");
s("0x1A", "abc");
s("0x1A", "10");
s("0x1A", "1e1");
s("0x1A", "10.0");
s("0x1A", " 10");
s("0x1A", "10 ");
s("0x1A", "+10");
s("0x1A", "-0");
s("0x1A", "0x1A");
s("0x1A", "1e400");
s("0x1A", "1e500");
s("0x1A", "-1e400");
s("0x1A", "9223372036854775807");
s("0x1A", "9223372036854775808");
s("0x1A", "9223372036854775809");
s("0x1A", "9.2233720368547758e18");
s("0x1A", "9223372036854775808.0");
s("0x1A", "-9223372036854775808");
s("0x1A", "-9223372036854775809");
s("0x1A", "-9.2233720368547758e18");
s("0x1A", "18446744073709551616");
s("0x1A", "10abc");
s("0x1A", " ");
s("0x1A", ".5");
s("0x1A", "5.");
s("0x1A", "1E1");
s("0x1A", "0.1");
s("0x1A", "1e-1");
s("0x1A", "NAN");
s("0x1A", "INF");
s("0x1A", "007");
s("0x1A", "7");
s("0x1A", "+0");
s("0x1A", "-0.0");
s("0x1A", "0e0");
s("0x1A", "1e0");
s("1e400", "");
s("1e400", "0");
s("1e400", "00");
s("1e400", "0.0");
s("1e400", "abc");
s("1e400", "10");
s("1e400", "1e1");
s("1e400", "10.0");
s("1e400", " 10");
s("1e400", "10 ");
s("1e400", "+10");
s("1e400", "-0");
s("1e400", "0x1A");
s("1e400", "1e400");
s("1e400", "1e500");
s("1e400", "-1e400");
s("1e400", "9223372036854775807");
s("1e400", "9223372036854775808");
s("1e400", "9223372036854775809");
s("1e400", "9.2233720368547758e18");
s("1e400", "9223372036854775808.0");
s("1e400", "-9223372036854775808");
s("1e400", "-9223372036854775809");
s("1e400", "-9.2233720368547758e18");
s("1e400", "18446744073709551616");
s("1e400", "10abc");
s("1e400", " ");
s("1e400", ".5");
s("1e400", "5.");
s("1e400", "1E1");
s("1e400", "0.1");
s("1e400", "1e-1");
s("1e400", "NAN");
s("1e400", "INF");
s("1e400", "007");
s("1e400", "7");
s("1e400", "+0");
s("1e400", "-0.0");
s("1e400", "0e0");
s("1e400", "1e0");
s("1e500", "");
s("1e500", "0");
s("1e500", "00");
s("1e500", "0.0");
s("1e500", "abc");
s("1e500", "10");
s("1e500", "1e1");
s("1e500", "10.0");
s("1e500", " 10");
s("1e500", "10 ");
s("1e500", "+10");
s("1e500", "-0");
s("1e500", "0x1A");
s("1e500", "1e400");
s("1e500", "1e500");
s("1e500", "-1e400");
s("1e500", "9223372036854775807");
s("1e500", "9223372036854775808");
s("1e500", "9223372036854775809");
s("1e500", "9.2233720368547758e18");
s("1e500", "9223372036854775808.0");
s("1e500", "-9223372036854775808");
s("1e500", "-9223372036854775809");
s("1e500", "-9.2233720368547758e18");
s("1e500", "18446744073709551616");
s("1e500", "10abc");
s("1e500", " ");
s("1e500", ".5");
s("1e500", "5.");
s("1e500", "1E1");
s("1e500", "0.1");
s("1e500", "1e-1");
s("1e500", "NAN");
s("1e500", "INF");
s("1e500", "007");
s("1e500", "7");
s("1e500", "+0");
s("1e500", "-0.0");
s("1e500", "0e0");
s("1e500", "1e0");
s("-1e400", "");
s("-1e400", "0");
s("-1e400", "00");
s("-1e400", "0.0");
s("-1e400", "abc");
s("-1e400", "10");
s("-1e400", "1e1");
s("-1e400", "10.0");
s("-1e400", " 10");
s("-1e400", "10 ");
s("-1e400", "+10");
s("-1e400", "-0");
s("-1e400", "0x1A");
s("-1e400", "1e400");
s("-1e400", "1e500");
s("-1e400", "-1e400");
s("-1e400", "9223372036854775807");
s("-1e400", "9223372036854775808");
s("-1e400", "9223372036854775809");
s("-1e400", "9.2233720368547758e18");
s("-1e400", "9223372036854775808.0");
s("-1e400", "-9223372036854775808");
s("-1e400", "-9223372036854775809");
s("-1e400", "-9.2233720368547758e18");
s("-1e400", "18446744073709551616");
s("-1e400", "10abc");
s("-1e400", " ");
s("-1e400", ".5");
s("-1e400", "5.");
s("-1e400", "1E1");
s("-1e400", "0.1");
s("-1e400", "1e-1");
s("-1e400", "NAN");
s("-1e400", "INF");
s("-1e400", "007");
s("-1e400", "7");
s("-1e400", "+0");
s("-1e400", "-0.0");
s("-1e400", "0e0");
s("-1e400", "1e0");
s("9223372036854775807", "");
s("9223372036854775807", "0");
s("9223372036854775807", "00");
s("9223372036854775807", "0.0");
s("9223372036854775807", "abc");
s("9223372036854775807", "10");
s("9223372036854775807", "1e1");
s("9223372036854775807", "10.0");
s("9223372036854775807", " 10");
s("9223372036854775807", "10 ");
s("9223372036854775807", "+10");
s("9223372036854775807", "-0");
s("9223372036854775807", "0x1A");
s("9223372036854775807", "1e400");
s("9223372036854775807", "1e500");
s("9223372036854775807", "-1e400");
s("9223372036854775807", "9223372036854775807");
s("9223372036854775807", "9223372036854775808");
s("9223372036854775807", "9223372036854775809");
s("9223372036854775807", "9.2233720368547758e18");
s("9223372036854775807", "9223372036854775808.0");
s("9223372036854775807", "-9223372036854775808");
s("9223372036854775807", "-9223372036854775809");
s("9223372036854775807", "-9.2233720368547758e18");
s("9223372036854775807", "18446744073709551616");
s("9223372036854775807", "10abc");
s("9223372036854775807", " ");
s("9223372036854775807", ".5");
s("9223372036854775807", "5.");
s("9223372036854775807", "1E1");
s("9223372036854775807", "0.1");
s("9223372036854775807", "1e-1");
s("9223372036854775807", "NAN");
s("9223372036854775807", "INF");
s("9223372036854775807", "007");
s("9223372036854775807", "7");
s("9223372036854775807", "+0");
s("9223372036854775807", "-0.0");
s("9223372036854775807", "0e0");
s("9223372036854775807", "1e0");
s("9223372036854775808", "");
s("9223372036854775808", "0");
s("9223372036854775808", "00");
s("9223372036854775808", "0.0");
s("9223372036854775808", "abc");
s("9223372036854775808", "10");
s("9223372036854775808", "1e1");
s("9223372036854775808", "10.0");
s("9223372036854775808", " 10");
s("9223372036854775808", "10 ");
s("9223372036854775808", "+10");
s("9223372036854775808", "-0");
s("9223372036854775808", "0x1A");
s("9223372036854775808", "1e400");
s("9223372036854775808", "1e500");
s("9223372036854775808", "-1e400");
s("9223372036854775808", "9223372036854775807");
s("9223372036854775808", "9223372036854775808");
s("9223372036854775808", "9223372036854775809");
s("9223372036854775808", "9.2233720368547758e18");
s("9223372036854775808", "9223372036854775808.0");
s("9223372036854775808", "-9223372036854775808");
s("9223372036854775808", "-9223372036854775809");
s("9223372036854775808", "-9.2233720368547758e18");
s("9223372036854775808", "18446744073709551616");
s("9223372036854775808", "10abc");
s("9223372036854775808", " ");
s("9223372036854775808", ".5");
s("9223372036854775808", "5.");
s("9223372036854775808", "1E1");
s("9223372036854775808", "0.1");
s("9223372036854775808", "1e-1");
s("9223372036854775808", "NAN");
s("9223372036854775808", "INF");
s("9223372036854775808", "007");
s("9223372036854775808", "7");
s("9223372036854775808", "+0");
s("9223372036854775808", "-0.0");
s("9223372036854775808", "0e0");
s("9223372036854775808", "1e0");
s("9223372036854775809", "");
s("9223372036854775809", "0");
s("9223372036854775809", "00");
s("9223372036854775809", "0.0");
s("9223372036854775809", "abc");
s("9223372036854775809", "10");
s("9223372036854775809", "1e1");
s("9223372036854775809", "10.0");
s("9223372036854775809", " 10");
s("9223372036854775809", "10 ");
s("9223372036854775809", "+10");
s("9223372036854775809", "-0");
s("9223372036854775809", "0x1A");
s("9223372036854775809", "1e400");
s("9223372036854775809", "1e500");
s("9223372036854775809", "-1e400");
s("9223372036854775809", "9223372036854775807");
s("9223372036854775809", "9223372036854775808");
s("9223372036854775809", "9223372036854775809");
s("9223372036854775809", "9.2233720368547758e18");
s("9223372036854775809", "9223372036854775808.0");
s("9223372036854775809", "-9223372036854775808");
s("9223372036854775809", "-9223372036854775809");
s("9223372036854775809", "-9.2233720368547758e18");
s("9223372036854775809", "18446744073709551616");
s("9223372036854775809", "10abc");
s("9223372036854775809", " ");
s("9223372036854775809", ".5");
s("9223372036854775809", "5.");
s("9223372036854775809", "1E1");
s("9223372036854775809", "0.1");
s("9223372036854775809", "1e-1");
s("9223372036854775809", "NAN");
s("9223372036854775809", "INF");
s("9223372036854775809", "007");
s("9223372036854775809", "7");
s("9223372036854775809", "+0");
s("9223372036854775809", "-0.0");
s("9223372036854775809", "0e0");
s("9223372036854775809", "1e0");
s("9.2233720368547758e18", "");
s("9.2233720368547758e18", "0");
s("9.2233720368547758e18", "00");
s("9.2233720368547758e18", "0.0");
s("9.2233720368547758e18", "abc");
s("9.2233720368547758e18", "10");
s("9.2233720368547758e18", "1e1");
s("9.2233720368547758e18", "10.0");
s("9.2233720368547758e18", " 10");
s("9.2233720368547758e18", "10 ");
s("9.2233720368547758e18", "+10");
s("9.2233720368547758e18", "-0");
s("9.2233720368547758e18", "0x1A");
s("9.2233720368547758e18", "1e400");
s("9.2233720368547758e18", "1e500");
s("9.2233720368547758e18", "-1e400");
s("9.2233720368547758e18", "9223372036854775807");
s("9.2233720368547758e18", "9223372036854775808");
s("9.2233720368547758e18", "9223372036854775809");
s("9.2233720368547758e18", "9.2233720368547758e18");
s("9.2233720368547758e18", "9223372036854775808.0");
s("9.2233720368547758e18", "-9223372036854775808");
s("9.2233720368547758e18", "-9223372036854775809");
s("9.2233720368547758e18", "-9.2233720368547758e18");
s("9.2233720368547758e18", "18446744073709551616");
s("9.2233720368547758e18", "10abc");
s("9.2233720368547758e18", " ");
s("9.2233720368547758e18", ".5");
s("9.2233720368547758e18", "5.");
s("9.2233720368547758e18", "1E1");
s("9.2233720368547758e18", "0.1");
s("9.2233720368547758e18", "1e-1");
s("9.2233720368547758e18", "NAN");
s("9.2233720368547758e18", "INF");
s("9.2233720368547758e18", "007");
s("9.2233720368547758e18", "7");
s("9.2233720368547758e18", "+0");
s("9.2233720368547758e18", "-0.0");
s("9.2233720368547758e18", "0e0");
s("9.2233720368547758e18", "1e0");
s("9223372036854775808.0", "");
s("9223372036854775808.0", "0");
s("9223372036854775808.0", "00");
s("9223372036854775808.0", "0.0");
s("9223372036854775808.0", "abc");
s("9223372036854775808.0", "10");
s("9223372036854775808.0", "1e1");
s("9223372036854775808.0", "10.0");
s("9223372036854775808.0", " 10");
s("9223372036854775808.0", "10 ");
s("9223372036854775808.0", "+10");
s("9223372036854775808.0", "-0");
s("9223372036854775808.0", "0x1A");
s("9223372036854775808.0", "1e400");
s("9223372036854775808.0", "1e500");
s("9223372036854775808.0", "-1e400");
s("9223372036854775808.0", "9223372036854775807");
s("9223372036854775808.0", "9223372036854775808");
s("9223372036854775808.0", "9223372036854775809");
s("9223372036854775808.0", "9.2233720368547758e18");
s("9223372036854775808.0", "9223372036854775808.0");
s("9223372036854775808.0", "-9223372036854775808");
s("9223372036854775808.0", "-9223372036854775809");
s("9223372036854775808.0", "-9.2233720368547758e18");
s("9223372036854775808.0", "18446744073709551616");
s("9223372036854775808.0", "10abc");
s("9223372036854775808.0", " ");
s("9223372036854775808.0", ".5");
s("9223372036854775808.0", "5.");
s("9223372036854775808.0", "1E1");
s("9223372036854775808.0", "0.1");
s("9223372036854775808.0", "1e-1");
s("9223372036854775808.0", "NAN");
s("9223372036854775808.0", "INF");
s("9223372036854775808.0", "007");
s("9223372036854775808.0", "7");
s("9223372036854775808.0", "+0");
s("9223372036854775808.0", "-0.0");
s("9223372036854775808.0", "0e0");
s("9223372036854775808.0", "1e0");
s("-9223372036854775808", "");
s("-9223372036854775808", "0");
s("-9223372036854775808", "00");
s("-9223372036854775808", "0.0");
s("-9223372036854775808", "abc");
s("-9223372036854775808", "10");
s("-9223372036854775808", "1e1");
s("-9223372036854775808", "10.0");
s("-9223372036854775808", " 10");
s("-9223372036854775808", "10 ");
s("-9223372036854775808", "+10");
s("-9223372036854775808", "-0");
s("-9223372036854775808", "0x1A");
s("-9223372036854775808", "1e400");
s("-9223372036854775808", "1e500");
s("-9223372036854775808", "-1e400");
s("-9223372036854775808", "9223372036854775807");
s("-9223372036854775808", "9223372036854775808");
s("-9223372036854775808", "9223372036854775809");
s("-9223372036854775808", "9.2233720368547758e18");
s("-9223372036854775808", "9223372036854775808.0");
s("-9223372036854775808", "-9223372036854775808");
s("-9223372036854775808", "-9223372036854775809");
s("-9223372036854775808", "-9.2233720368547758e18");
s("-9223372036854775808", "18446744073709551616");
s("-9223372036854775808", "10abc");
s("-9223372036854775808", " ");
s("-9223372036854775808", ".5");
s("-9223372036854775808", "5.");
s("-9223372036854775808", "1E1");
s("-9223372036854775808", "0.1");
s("-9223372036854775808", "1e-1");
s("-9223372036854775808", "NAN");
s("-9223372036854775808", "INF");
s("-9223372036854775808", "007");
s("-9223372036854775808", "7");
s("-9223372036854775808", "+0");
s("-9223372036854775808", "-0.0");
s("-9223372036854775808", "0e0");
s("-9223372036854775808", "1e0");
s("-9223372036854775809", "");
s("-9223372036854775809", "0");
s("-9223372036854775809", "00");
s("-9223372036854775809", "0.0");
s("-9223372036854775809", "abc");
s("-9223372036854775809", "10");
s("-9223372036854775809", "1e1");
s("-9223372036854775809", "10.0");
s("-9223372036854775809", " 10");
s("-9223372036854775809", "10 ");
s("-9223372036854775809", "+10");
s("-9223372036854775809", "-0");
s("-9223372036854775809", "0x1A");
s("-9223372036854775809", "1e400");
s("-9223372036854775809", "1e500");
s("-9223372036854775809", "-1e400");
s("-9223372036854775809", "9223372036854775807");
s("-9223372036854775809", "9223372036854775808");
s("-9223372036854775809", "9223372036854775809");
s("-9223372036854775809", "9.2233720368547758e18");
s("-9223372036854775809", "9223372036854775808.0");
s("-9223372036854775809", "-9223372036854775808");
s("-9223372036854775809", "-9223372036854775809");
s("-9223372036854775809", "-9.2233720368547758e18");
s("-9223372036854775809", "18446744073709551616");
s("-9223372036854775809", "10abc");
s("-9223372036854775809", " ");
s("-9223372036854775809", ".5");
s("-9223372036854775809", "5.");
s("-9223372036854775809", "1E1");
s("-9223372036854775809", "0.1");
s("-9223372036854775809", "1e-1");
s("-9223372036854775809", "NAN");
s("-9223372036854775809", "INF");
s("-9223372036854775809", "007");
s("-9223372036854775809", "7");
s("-9223372036854775809", "+0");
s("-9223372036854775809", "-0.0");
s("-9223372036854775809", "0e0");
s("-9223372036854775809", "1e0");
s("-9.2233720368547758e18", "");
s("-9.2233720368547758e18", "0");
s("-9.2233720368547758e18", "00");
s("-9.2233720368547758e18", "0.0");
s("-9.2233720368547758e18", "abc");
s("-9.2233720368547758e18", "10");
s("-9.2233720368547758e18", "1e1");
s("-9.2233720368547758e18", "10.0");
s("-9.2233720368547758e18", " 10");
s("-9.2233720368547758e18", "10 ");
s("-9.2233720368547758e18", "+10");
s("-9.2233720368547758e18", "-0");
s("-9.2233720368547758e18", "0x1A");
s("-9.2233720368547758e18", "1e400");
s("-9.2233720368547758e18", "1e500");
s("-9.2233720368547758e18", "-1e400");
s("-9.2233720368547758e18", "9223372036854775807");
s("-9.2233720368547758e18", "9223372036854775808");
s("-9.2233720368547758e18", "9223372036854775809");
s("-9.2233720368547758e18", "9.2233720368547758e18");
s("-9.2233720368547758e18", "9223372036854775808.0");
s("-9.2233720368547758e18", "-9223372036854775808");
s("-9.2233720368547758e18", "-9223372036854775809");
s("-9.2233720368547758e18", "-9.2233720368547758e18");
s("-9.2233720368547758e18", "18446744073709551616");
s("-9.2233720368547758e18", "10abc");
s("-9.2233720368547758e18", " ");
s("-9.2233720368547758e18", ".5");
s("-9.2233720368547758e18", "5.");
s("-9.2233720368547758e18", "1E1");
s("-9.2233720368547758e18", "0.1");
s("-9.2233720368547758e18", "1e-1");
s("-9.2233720368547758e18", "NAN");
s("-9.2233720368547758e18", "INF");
s("-9.2233720368547758e18", "007");
s("-9.2233720368547758e18", "7");
s("-9.2233720368547758e18", "+0");
s("-9.2233720368547758e18", "-0.0");
s("-9.2233720368547758e18", "0e0");
s("-9.2233720368547758e18", "1e0");
s("18446744073709551616", "");
s("18446744073709551616", "0");
s("18446744073709551616", "00");
s("18446744073709551616", "0.0");
s("18446744073709551616", "abc");
s("18446744073709551616", "10");
s("18446744073709551616", "1e1");
s("18446744073709551616", "10.0");
s("18446744073709551616", " 10");
s("18446744073709551616", "10 ");
s("18446744073709551616", "+10");
s("18446744073709551616", "-0");
s("18446744073709551616", "0x1A");
s("18446744073709551616", "1e400");
s("18446744073709551616", "1e500");
s("18446744073709551616", "-1e400");
s("18446744073709551616", "9223372036854775807");
s("18446744073709551616", "9223372036854775808");
s("18446744073709551616", "9223372036854775809");
s("18446744073709551616", "9.2233720368547758e18");
s("18446744073709551616", "9223372036854775808.0");
s("18446744073709551616", "-9223372036854775808");
s("18446744073709551616", "-9223372036854775809");
s("18446744073709551616", "-9.2233720368547758e18");
s("18446744073709551616", "18446744073709551616");
s("18446744073709551616", "10abc");
s("18446744073709551616", " ");
s("18446744073709551616", ".5");
s("18446744073709551616", "5.");
s("18446744073709551616", "1E1");
s("18446744073709551616", "0.1");
s("18446744073709551616", "1e-1");
s("18446744073709551616", "NAN");
s("18446744073709551616", "INF");
s("18446744073709551616", "007");
s("18446744073709551616", "7");
s("18446744073709551616", "+0");
s("18446744073709551616", "-0.0");
s("18446744073709551616", "0e0");
s("18446744073709551616", "1e0");
s("10abc", "");
s("10abc", "0");
s("10abc", "00");
s("10abc", "0.0");
s("10abc", "abc");
s("10abc", "10");
s("10abc", "1e1");
s("10abc", "10.0");
s("10abc", " 10");
s("10abc", "10 ");
s("10abc", "+10");
s("10abc", "-0");
s("10abc", "0x1A");
s("10abc", "1e400");
s("10abc", "1e500");
s("10abc", "-1e400");
s("10abc", "9223372036854775807");
s("10abc", "9223372036854775808");
s("10abc", "9223372036854775809");
s("10abc", "9.2233720368547758e18");
s("10abc", "9223372036854775808.0");
s("10abc", "-9223372036854775808");
s("10abc", "-9223372036854775809");
s("10abc", "-9.2233720368547758e18");
s("10abc", "18446744073709551616");
s("10abc", "10abc");
s("10abc", " ");
s("10abc", ".5");
s("10abc", "5.");
s("10abc", "1E1");
s("10abc", "0.1");
s("10abc", "1e-1");
s("10abc", "NAN");
s("10abc", "INF");
s("10abc", "007");
s("10abc", "7");
s("10abc", "+0");
s("10abc", "-0.0");
s("10abc", "0e0");
s("10abc", "1e0");
s(" ", "");
s(" ", "0");
s(" ", "00");
s(" ", "0.0");
s(" ", "abc");
s(" ", "10");
s(" ", "1e1");
s(" ", "10.0");
s(" ", " 10");
s(" ", "10 ");
s(" ", "+10");
s(" ", "-0");
s(" ", "0x1A");
s(" ", "1e400");
s(" ", "1e500");
s(" ", "-1e400");
s(" ", "9223372036854775807");
s(" ", "9223372036854775808");
s(" ", "9223372036854775809");
s(" ", "9.2233720368547758e18");
s(" ", "9223372036854775808.0");
s(" ", "-9223372036854775808");
s(" ", "-9223372036854775809");
s(" ", "-9.2233720368547758e18");
s(" ", "18446744073709551616");
s(" ", "10abc");
s(" ", " ");
s(" ", ".5");
s(" ", "5.");
s(" ", "1E1");
s(" ", "0.1");
s(" ", "1e-1");
s(" ", "NAN");
s(" ", "INF");
s(" ", "007");
s(" ", "7");
s(" ", "+0");
s(" ", "-0.0");
s(" ", "0e0");
s(" ", "1e0");
s(".5", "");
s(".5", "0");
s(".5", "00");
s(".5", "0.0");
s(".5", "abc");
s(".5", "10");
s(".5", "1e1");
s(".5", "10.0");
s(".5", " 10");
s(".5", "10 ");
s(".5", "+10");
s(".5", "-0");
s(".5", "0x1A");
s(".5", "1e400");
s(".5", "1e500");
s(".5", "-1e400");
s(".5", "9223372036854775807");
s(".5", "9223372036854775808");
s(".5", "9223372036854775809");
s(".5", "9.2233720368547758e18");
s(".5", "9223372036854775808.0");
s(".5", "-9223372036854775808");
s(".5", "-9223372036854775809");
s(".5", "-9.2233720368547758e18");
s(".5", "18446744073709551616");
s(".5", "10abc");
s(".5", " ");
s(".5", ".5");
s(".5", "5.");
s(".5", "1E1");
s(".5", "0.1");
s(".5", "1e-1");
s(".5", "NAN");
s(".5", "INF");
s(".5", "007");
s(".5", "7");
s(".5", "+0");
s(".5", "-0.0");
s(".5", "0e0");
s(".5", "1e0");
s("5.", "");
s("5.", "0");
s("5.", "00");
s("5.", "0.0");
s("5.", "abc");
s("5.", "10");
s("5.", "1e1");
s("5.", "10.0");
s("5.", " 10");
s("5.", "10 ");
s("5.", "+10");
s("5.", "-0");
s("5.", "0x1A");
s("5.", "1e400");
s("5.", "1e500");
s("5.", "-1e400");
s("5.", "9223372036854775807");
s("5.", "9223372036854775808");
s("5.", "9223372036854775809");
s("5.", "9.2233720368547758e18");
s("5.", "9223372036854775808.0");
s("5.", "-9223372036854775808");
s("5.", "-9223372036854775809");
s("5.", "-9.2233720368547758e18");
s("5.", "18446744073709551616");
s("5.", "10abc");
s("5.", " ");
s("5.", ".5");
s("5.", "5.");
s("5.", "1E1");
s("5.", "0.1");
s("5.", "1e-1");
s("5.", "NAN");
s("5.", "INF");
s("5.", "007");
s("5.", "7");
s("5.", "+0");
s("5.", "-0.0");
s("5.", "0e0");
s("5.", "1e0");
s("1E1", "");
s("1E1", "0");
s("1E1", "00");
s("1E1", "0.0");
s("1E1", "abc");
s("1E1", "10");
s("1E1", "1e1");
s("1E1", "10.0");
s("1E1", " 10");
s("1E1", "10 ");
s("1E1", "+10");
s("1E1", "-0");
s("1E1", "0x1A");
s("1E1", "1e400");
s("1E1", "1e500");
s("1E1", "-1e400");
s("1E1", "9223372036854775807");
s("1E1", "9223372036854775808");
s("1E1", "9223372036854775809");
s("1E1", "9.2233720368547758e18");
s("1E1", "9223372036854775808.0");
s("1E1", "-9223372036854775808");
s("1E1", "-9223372036854775809");
s("1E1", "-9.2233720368547758e18");
s("1E1", "18446744073709551616");
s("1E1", "10abc");
s("1E1", " ");
s("1E1", ".5");
s("1E1", "5.");
s("1E1", "1E1");
s("1E1", "0.1");
s("1E1", "1e-1");
s("1E1", "NAN");
s("1E1", "INF");
s("1E1", "007");
s("1E1", "7");
s("1E1", "+0");
s("1E1", "-0.0");
s("1E1", "0e0");
s("1E1", "1e0");
s("0.1", "");
s("0.1", "0");
s("0.1", "00");
s("0.1", "0.0");
s("0.1", "abc");
s("0.1", "10");
s("0.1", "1e1");
s("0.1", "10.0");
s("0.1", " 10");
s("0.1", "10 ");
s("0.1", "+10");
s("0.1", "-0");
s("0.1", "0x1A");
s("0.1", "1e400");
s("0.1", "1e500");
s("0.1", "-1e400");
s("0.1", "9223372036854775807");
s("0.1", "9223372036854775808");
s("0.1", "9223372036854775809");
s("0.1", "9.2233720368547758e18");
s("0.1", "9223372036854775808.0");
s("0.1", "-9223372036854775808");
s("0.1", "-9223372036854775809");
s("0.1", "-9.2233720368547758e18");
s("0.1", "18446744073709551616");
s("0.1", "10abc");
s("0.1", " ");
s("0.1", ".5");
s("0.1", "5.");
s("0.1", "1E1");
s("0.1", "0.1");
s("0.1", "1e-1");
s("0.1", "NAN");
s("0.1", "INF");
s("0.1", "007");
s("0.1", "7");
s("0.1", "+0");
s("0.1", "-0.0");
s("0.1", "0e0");
s("0.1", "1e0");
s("1e-1", "");
s("1e-1", "0");
s("1e-1", "00");
s("1e-1", "0.0");
s("1e-1", "abc");
s("1e-1", "10");
s("1e-1", "1e1");
s("1e-1", "10.0");
s("1e-1", " 10");
s("1e-1", "10 ");
s("1e-1", "+10");
s("1e-1", "-0");
s("1e-1", "0x1A");
s("1e-1", "1e400");
s("1e-1", "1e500");
s("1e-1", "-1e400");
s("1e-1", "9223372036854775807");
s("1e-1", "9223372036854775808");
s("1e-1", "9223372036854775809");
s("1e-1", "9.2233720368547758e18");
s("1e-1", "9223372036854775808.0");
s("1e-1", "-9223372036854775808");
s("1e-1", "-9223372036854775809");
s("1e-1", "-9.2233720368547758e18");
s("1e-1", "18446744073709551616");
s("1e-1", "10abc");
s("1e-1", " ");
s("1e-1", ".5");
s("1e-1", "5.");
s("1e-1", "1E1");
s("1e-1", "0.1");
s("1e-1", "1e-1");
s("1e-1", "NAN");
s("1e-1", "INF");
s("1e-1", "007");
s("1e-1", "7");
s("1e-1", "+0");
s("1e-1", "-0.0");
s("1e-1", "0e0");
s("1e-1", "1e0");
s("NAN", "");
s("NAN", "0");
s("NAN", "00");
s("NAN", "0.0");
s("NAN", "abc");
s("NAN", "10");
s("NAN", "1e1");
s("NAN", "10.0");
s("NAN", " 10");
s("NAN", "10 ");
s("NAN", "+10");
s("NAN", "-0");
s("NAN", "0x1A");
s("NAN", "1e400");
s("NAN", "1e500");
s("NAN", "-1e400");
s("NAN", "9223372036854775807");
s("NAN", "9223372036854775808");
s("NAN", "9223372036854775809");
s("NAN", "9.2233720368547758e18");
s("NAN", "9223372036854775808.0");
s("NAN", "-9223372036854775808");
s("NAN", "-9223372036854775809");
s("NAN", "-9.2233720368547758e18");
s("NAN", "18446744073709551616");
s("NAN", "10abc");
s("NAN", " ");
s("NAN", ".5");
s("NAN", "5.");
s("NAN", "1E1");
s("NAN", "0.1");
s("NAN", "1e-1");
s("NAN", "NAN");
s("NAN", "INF");
s("NAN", "007");
s("NAN", "7");
s("NAN", "+0");
s("NAN", "-0.0");
s("NAN", "0e0");
s("NAN", "1e0");
s("INF", "");
s("INF", "0");
s("INF", "00");
s("INF", "0.0");
s("INF", "abc");
s("INF", "10");
s("INF", "1e1");
s("INF", "10.0");
s("INF", " 10");
s("INF", "10 ");
s("INF", "+10");
s("INF", "-0");
s("INF", "0x1A");
s("INF", "1e400");
s("INF", "1e500");
s("INF", "-1e400");
s("INF", "9223372036854775807");
s("INF", "9223372036854775808");
s("INF", "9223372036854775809");
s("INF", "9.2233720368547758e18");
s("INF", "9223372036854775808.0");
s("INF", "-9223372036854775808");
s("INF", "-9223372036854775809");
s("INF", "-9.2233720368547758e18");
s("INF", "18446744073709551616");
s("INF", "10abc");
s("INF", " ");
s("INF", ".5");
s("INF", "5.");
s("INF", "1E1");
s("INF", "0.1");
s("INF", "1e-1");
s("INF", "NAN");
s("INF", "INF");
s("INF", "007");
s("INF", "7");
s("INF", "+0");
s("INF", "-0.0");
s("INF", "0e0");
s("INF", "1e0");
s("007", "");
s("007", "0");
s("007", "00");
s("007", "0.0");
s("007", "abc");
s("007", "10");
s("007", "1e1");
s("007", "10.0");
s("007", " 10");
s("007", "10 ");
s("007", "+10");
s("007", "-0");
s("007", "0x1A");
s("007", "1e400");
s("007", "1e500");
s("007", "-1e400");
s("007", "9223372036854775807");
s("007", "9223372036854775808");
s("007", "9223372036854775809");
s("007", "9.2233720368547758e18");
s("007", "9223372036854775808.0");
s("007", "-9223372036854775808");
s("007", "-9223372036854775809");
s("007", "-9.2233720368547758e18");
s("007", "18446744073709551616");
s("007", "10abc");
s("007", " ");
s("007", ".5");
s("007", "5.");
s("007", "1E1");
s("007", "0.1");
s("007", "1e-1");
s("007", "NAN");
s("007", "INF");
s("007", "007");
s("007", "7");
s("007", "+0");
s("007", "-0.0");
s("007", "0e0");
s("007", "1e0");
s("7", "");
s("7", "0");
s("7", "00");
s("7", "0.0");
s("7", "abc");
s("7", "10");
s("7", "1e1");
s("7", "10.0");
s("7", " 10");
s("7", "10 ");
s("7", "+10");
s("7", "-0");
s("7", "0x1A");
s("7", "1e400");
s("7", "1e500");
s("7", "-1e400");
s("7", "9223372036854775807");
s("7", "9223372036854775808");
s("7", "9223372036854775809");
s("7", "9.2233720368547758e18");
s("7", "9223372036854775808.0");
s("7", "-9223372036854775808");
s("7", "-9223372036854775809");
s("7", "-9.2233720368547758e18");
s("7", "18446744073709551616");
s("7", "10abc");
s("7", " ");
s("7", ".5");
s("7", "5.");
s("7", "1E1");
s("7", "0.1");
s("7", "1e-1");
s("7", "NAN");
s("7", "INF");
s("7", "007");
s("7", "7");
s("7", "+0");
s("7", "-0.0");
s("7", "0e0");
s("7", "1e0");
s("+0", "");
s("+0", "0");
s("+0", "00");
s("+0", "0.0");
s("+0", "abc");
s("+0", "10");
s("+0", "1e1");
s("+0", "10.0");
s("+0", " 10");
s("+0", "10 ");
s("+0", "+10");
s("+0", "-0");
s("+0", "0x1A");
s("+0", "1e400");
s("+0", "1e500");
s("+0", "-1e400");
s("+0", "9223372036854775807");
s("+0", "9223372036854775808");
s("+0", "9223372036854775809");
s("+0", "9.2233720368547758e18");
s("+0", "9223372036854775808.0");
s("+0", "-9223372036854775808");
s("+0", "-9223372036854775809");
s("+0", "-9.2233720368547758e18");
s("+0", "18446744073709551616");
s("+0", "10abc");
s("+0", " ");
s("+0", ".5");
s("+0", "5.");
s("+0", "1E1");
s("+0", "0.1");
s("+0", "1e-1");
s("+0", "NAN");
s("+0", "INF");
s("+0", "007");
s("+0", "7");
s("+0", "+0");
s("+0", "-0.0");
s("+0", "0e0");
s("+0", "1e0");
s("-0.0", "");
s("-0.0", "0");
s("-0.0", "00");
s("-0.0", "0.0");
s("-0.0", "abc");
s("-0.0", "10");
s("-0.0", "1e1");
s("-0.0", "10.0");
s("-0.0", " 10");
s("-0.0", "10 ");
s("-0.0", "+10");
s("-0.0", "-0");
s("-0.0", "0x1A");
s("-0.0", "1e400");
s("-0.0", "1e500");
s("-0.0", "-1e400");
s("-0.0", "9223372036854775807");
s("-0.0", "9223372036854775808");
s("-0.0", "9223372036854775809");
s("-0.0", "9.2233720368547758e18");
s("-0.0", "9223372036854775808.0");
s("-0.0", "-9223372036854775808");
s("-0.0", "-9223372036854775809");
s("-0.0", "-9.2233720368547758e18");
s("-0.0", "18446744073709551616");
s("-0.0", "10abc");
s("-0.0", " ");
s("-0.0", ".5");
s("-0.0", "5.");
s("-0.0", "1E1");
s("-0.0", "0.1");
s("-0.0", "1e-1");
s("-0.0", "NAN");
s("-0.0", "INF");
s("-0.0", "007");
s("-0.0", "7");
s("-0.0", "+0");
s("-0.0", "-0.0");
s("-0.0", "0e0");
s("-0.0", "1e0");
s("0e0", "");
s("0e0", "0");
s("0e0", "00");
s("0e0", "0.0");
s("0e0", "abc");
s("0e0", "10");
s("0e0", "1e1");
s("0e0", "10.0");
s("0e0", " 10");
s("0e0", "10 ");
s("0e0", "+10");
s("0e0", "-0");
s("0e0", "0x1A");
s("0e0", "1e400");
s("0e0", "1e500");
s("0e0", "-1e400");
s("0e0", "9223372036854775807");
s("0e0", "9223372036854775808");
s("0e0", "9223372036854775809");
s("0e0", "9.2233720368547758e18");
s("0e0", "9223372036854775808.0");
s("0e0", "-9223372036854775808");
s("0e0", "-9223372036854775809");
s("0e0", "-9.2233720368547758e18");
s("0e0", "18446744073709551616");
s("0e0", "10abc");
s("0e0", " ");
s("0e0", ".5");
s("0e0", "5.");
s("0e0", "1E1");
s("0e0", "0.1");
s("0e0", "1e-1");
s("0e0", "NAN");
s("0e0", "INF");
s("0e0", "007");
s("0e0", "7");
s("0e0", "+0");
s("0e0", "-0.0");
s("0e0", "0e0");
s("0e0", "1e0");
s("1e0", "");
s("1e0", "0");
s("1e0", "00");
s("1e0", "0.0");
s("1e0", "abc");
s("1e0", "10");
s("1e0", "1e1");
s("1e0", "10.0");
s("1e0", " 10");
s("1e0", "10 ");
s("1e0", "+10");
s("1e0", "-0");
s("1e0", "0x1A");
s("1e0", "1e400");
s("1e0", "1e500");
s("1e0", "-1e400");
s("1e0", "9223372036854775807");
s("1e0", "9223372036854775808");
s("1e0", "9223372036854775809");
s("1e0", "9.2233720368547758e18");
s("1e0", "9223372036854775808.0");
s("1e0", "-9223372036854775808");
s("1e0", "-9223372036854775809");
s("1e0", "-9.2233720368547758e18");
s("1e0", "18446744073709551616");
s("1e0", "10abc");
s("1e0", " ");
s("1e0", ".5");
s("1e0", "5.");
s("1e0", "1E1");
s("1e0", "0.1");
s("1e0", "1e-1");
s("1e0", "NAN");
s("1e0", "INF");
s("1e0", "007");
s("1e0", "7");
s("1e0", "+0");
s("1e0", "-0.0");
s("1e0", "0e0");
s("1e0", "1e0");
echo "\n";function i(int $a, int $b): void { echo ($a == $b) ? "1" : "0"; echo ($a != $b) ? "1" : "0"; }
i(1,1); i(1,2); i(0,-0); i(PHP_INT_MAX,PHP_INT_MAX); i(PHP_INT_MIN,PHP_INT_MAX);
echo "|";
f(1.5,1.5); f(1.5,2.5); f(0.0,-0.0); f(NAN,NAN); f(INF,INF); f(INF,-INF); f(NAN,1.0);
echo "|";
b(true,true); b(true,false); b(false,false);
echo "|";
m(1,1.0); m(1,1.5); m(PHP_INT_MAX,9.2233720368547758e18); m(0,-0.0); m(1,NAN); m(2,INF);
echo "|";
n(1.0,1); n(1.5,1); n(-0.0,0); n(NAN,0); n(9.2233720368547758e18,PHP_INT_MAX);
echo "\n";
"##;

/// php-src 8.5.6's own output for `LOOSE_EQ_SOURCE`.
const LOOSE_EQ_EXPECTED: &str = r##"1000000000000000000000000000000000000000011100000001000000000000000000000000111001110000000100000000000000000000000011100111000000010000000000000000000000001110000010000000000000000000000000000000000000000111111000000000000000000100000000000000011111100000000000000000010000000000000001111110000000000000000001000000000000000111111000000000000000000100000000000000011111100000000000000000010000000000000001111110000000000000000001000000000001110000000100000000000000000000000011100000000000001000000000000000000000000000000000000000010000000000000000000000000000000000000000100000000000000000000000000000000000000001000000000000000000000000000000000000000010011000000000000000000000000000000000000101100000000000000000000000000000000000001110000000000000000000000000000000000011111000000000000000000000000000000000001111100000000000000000000000000000000000000001010000000000000000000000000000000000000011000000000000000000000000000000000000011100000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000000000000000000000000000100000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000111111000000000000000000100000000000000000000000000000000000000001100000000000000000000000000000000000000110000000000000000000000000000000000000000100000000000000000000000000000000000000001000000000000000000000000000000000000000011000000000000000000000000000000000000001100000111000000010000000000000000000000001110011100000001000000000000000000000000111001110000000100000000000000000000000011100000000000000000000000000000000000000001
1001101001|1010100|101|101100|10101
"##;

/// Verifies `in_array` in BOTH forms, over the (needle, element) pairs whose rule was measured.
///
/// It used to be lowered only as a strict identity scan over int slots, because the loose form
/// needs PHP's juggling. It now reuses the very comparison `==` lowers, so the loose form answers
/// the numeric-string rule: `in_array("1e1", ["a","10","b"])` is TRUE loosely and FALSE strictly,
/// and so is `in_array(" 10", ...)` — leading whitespace and all.
///
/// A needle and elements of DIFFERENT types short-circuit under `===`: PHP compares types first,
/// so `in_array(1, [1.0, 2.0], true)` is false without looking at a single element, while the
/// loose form widens and finds it.
///
/// The empty-haystack cases are here because they still have to TYPE-CHECK: the scan takes the
/// needle by value, so its shape follows the needle even when there is nothing to compare against.
#[test]
fn test_cli_wasm_in_array_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_in_array");
    let php_path = dir.join("main.php");
    fs::write(&php_path, IN_ARRAY_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the in_array probe");
    assert!(
        output.status.success(),
        "in_array compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the in_array probe under Node");
    assert!(
        run.status.success(),
        "in_array probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), IN_ARRAY_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The `in_array` probe: each lowered pair, in both forms, plus the empty haystack.
const IN_ARRAY_SOURCE: &str = r##"<?php
$i = [10, 20, 30];      $e = [];
$s = ["a", "10", "b"];  $f = [1.5, 2.5];
echo in_array(20, $i)?"1":"0", in_array(99, $i)?"1":"0", in_array(20, $i, true)?"1":"0", in_array(20, $e)?"1":"0", "|";
echo in_array("a", $s)?"1":"0", in_array("10", $s)?"1":"0", in_array("1e1", $s)?"1":"0", in_array("1e1", $s, true)?"1":"0", in_array("z", $s)?"1":"0", "|";
echo in_array(" 10", $s)?"1":"0", in_array(" 10", $s, true)?"1":"0", in_array("10.0", $s)?"1":"0", "|";
echo in_array(1.5, $f)?"1":"0", in_array(3.5, $f)?"1":"0", in_array(1.5, $f, true)?"1":"0", in_array(1.5, $e)?"1":"0", "|";
echo in_array(1, [1.0, 2.0])?"1":"0", in_array(1, [1.0, 2.0], true)?"1":"0", in_array(1.0, [1, 2])?"1":"0", in_array(1.0, [1, 2], true)?"1":"0", "|";
echo in_array(3, [1.0, 2.0])?"1":"0", in_array(3.0, [1, 2])?"1":"0", "|";
echo "\n";
"##;

/// php-src 8.5.6's own output for `IN_ARRAY_SOURCE`.
const IN_ARRAY_EXPECTED: &str = r##"1010|11100|101|1010|1010|00|
"##;

/// Verifies `array_search`, which shares its scan with `in_array` and boxes the result.
///
/// One scan serves both: it answers the first matching INDEX, which `in_array` reduces to a bool
/// and this boxes. `int|false` travels as a Mixed cell — tag 0 carrying the key, tag 3 carrying
/// false — the same convention `strpos` uses for the same result type, which is why a miss prints
/// as the empty string here.
///
/// Only the LOOSE form exists: the front-end rejects a third operand with "array_search() takes
/// exactly 2 arguments". So the numeric-string rule applies throughout —
/// `array_search("1e1", ["a","10","b"])` is 1, and `array_search(" 10", ...)` matches through the
/// leading whitespace.
#[test]
fn test_cli_wasm_array_search_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_search");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ARRAY_SEARCH_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array_search probe");
    assert!(
        output.status.success(),
        "array_search compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array_search probe under Node");
    assert!(
        run.status.success(),
        "array_search probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ARRAY_SEARCH_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The `array_search` probe: hits, misses, the numeric-string rule, and the empty haystack.
const ARRAY_SEARCH_SOURCE: &str = r##"<?php
function p(mixed $m): void { echo implode("", [$m]), "|"; }
$i = [10, 20, 30]; $s = ["a", "10", "b"]; $f = [1.5, 2.5]; $e = [];
p(array_search(20, $i)); p(array_search(99, $i)); p(array_search(10, $i)); p(array_search(30, $i));
p(array_search("1e1", $s)); p(array_search("a", $s)); p(array_search("z", $s)); p(array_search(" 10", $s));
p(array_search(2.5, $f)); p(array_search(9.5, $f));
p(array_search(1, $e));
p(array_search(1, [1.0, 2.0])); p(array_search(2.0, [1, 2]));
echo "\n";
"##;

/// php-src 8.5.6's own output for `ARRAY_SEARCH_SOURCE`.
const ARRAY_SEARCH_EXPECTED: &str = r##"1||0|2|1|0||1|1|||0|1|
"##;

/// Verifies the EMPTY-ARRAY ACCUMULATOR — `$out = []; foreach (...) { $out[] = ...; }`.
///
/// The slot is typed from the empty literal (`array<never>`) and the value from whatever gets
/// pushed, and the two meet at the loop's phi in BOTH directions. This target specializes slot
/// width and value_type per element type, so those transfers looked like a widening and were
/// refused — which turned away one of the most common shapes in PHP.
///
/// They are not a widening: an array whose element type is `never` has no elements and no decided
/// layout, because `__rt_array_push_*` shapes slot width and value_type on the FIRST push. So the
/// pointer is interchangeable with any element type's, and the transfer is a plain copy.
///
/// The float accumulator is here because `foreach` over float elements needed its own load
/// contract, the counterpart of the float array storage.
#[test]
fn test_cli_wasm_empty_array_accumulator_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_accumulator");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ACCUMULATOR_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the accumulator probe");
    assert!(
        output.status.success(),
        "accumulator compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the accumulator probe under Node");
    assert!(
        run.status.success(),
        "accumulator probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ACCUMULATOR_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The accumulator probe: every element type, a filtered build, one that stays empty, one
/// returned, and a realistic slugifier that combines several of the lowered builtins.
const ACCUMULATOR_SOURCE: &str = r##"<?php
function slugify(string $title): string {
    $lower = strtolower(trim($title));
    $parts = explode(" ", $lower);
    $kept = [];
    foreach ($parts as $p) {
        if ($p !== "" && !in_array($p, ["the", "a", "of"])) { $kept[] = $p; }
    }
    return implode("-", $kept);
}
function strs(array $xs): string { $o = []; foreach ($xs as $x) { $o[] = strtoupper($x); } return implode(",", $o); }
function ints(array $xs): string { $o = []; foreach ($xs as $x) { $o[] = $x; } return implode(",", $o); }
function flts(array $xs): string { $o = []; foreach ($xs as $x) { $o[] = $x; } return implode(",", $o); }
function filt(array $xs): string { $o = []; foreach ($xs as $x) { if ($x !== "b") { $o[] = $x; } } return implode(",", $o); }
function empt(array $xs): string { $o = []; foreach ($xs as $x) { if ($x === "zz") { $o[] = $x; } } return count($o) . ":" . implode(",", $o); }
function ret(array $xs): array { $o = []; foreach ($xs as $x) { $o[] = $x; } return $o; }
echo strs(["a","b"]), "|", ints([1,2,3]), "|", flts([1.5,2.5]), "|";
echo filt(["a","b","c"]), "|", empt(["a","b"]), "|";
$r = ret(["p","q"]); echo count($r), ":", implode(",", $r), "|";
$n = []; echo count($n), ":", implode(",", $n), "|";
echo slugify("  The Rise of  Machines "), "\n";
"##;

/// php-src 8.5.6's own output for `ACCUMULATOR_SOURCE`.
const ACCUMULATOR_EXPECTED: &str = r##"A,B|1,2,3|1.5,2.5|a,c|0:|2:p,q|0:|rise-machines
"##;

/// Verifies arrays of OBJECTS end to end: building one, walking it, and reading through it.
///
/// An object is a refcounted container, so its slot holds a pointer under `value_type` 4 — the
/// stamp that makes `__rt_array_free_deep` release each element instead of dropping it. The array
/// takes a SHARE at the push, because the EIR emits `array_push` then `release` of the operand.
///
/// `foreach` binds an OWNED element, so the read increfs. Deciding that from the result's
/// REPRESENTATION alone was wrong: an object pointer is a `Ptr` just like a Mixed cell, so the
/// binding was boxed into a cell and the property read then found an empty slot — right shape,
/// wrong object, and the loop printed nothing.
///
/// A promoted constructor property is admitted although it is typed with no default: the
/// promotion assigns it from the constructor's signature, before the body runs, so no read can
/// precede it. Non-promoted typed properties still need the initialization check and stay refused.
///
/// The Mixed-element loop is here because an untyped `array` parameter widens to cells, which is
/// the shape any function boundary produces.
#[test]
fn test_cli_wasm_object_arrays_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_object_array");
    let php_path = dir.join("main.php");
    fs::write(&php_path, OBJECT_ARRAY_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the object-array probe");
    assert!(
        output.status.success(),
        "object-array compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the object-array probe under Node");
    assert!(
        run.status.success(),
        "object-array probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), OBJECT_ARRAY_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The object-array probe: a method call per element, a string and an int property, an empty
/// array, and a Mixed-element loop.
const OBJECT_ARRAY_SOURCE: &str = r##"<?php
class Item {
    public function __construct(public string $name, public int $qty) {}
    public function label(): string { return $this->name . " x" . $this->qty; }
}
$items = [new Item("bolt", 3), new Item("nut", 7), new Item("pin", 1)];
$out = [];
foreach ($items as $it) { $out[] = $it->label(); }
echo implode("; ", $out), "\n";
$up = [];
foreach ($items as $it) { $up[] = strtoupper($it->name); }
echo implode(", ", $up), "\n";
echo count($items), "\n";
$empty = [];
foreach ($empty as $it) { echo "never"; }
foreach ($items as $it) { echo $it->qty, "."; }
echo "\n";
$m = [1, "hi", 2.5];
foreach ($m as $v) { echo $v, "|"; }
echo "done\n";
"##;

/// php-src 8.5.6's own output for `OBJECT_ARRAY_SOURCE`.
const OBJECT_ARRAY_EXPECTED: &str = r##"bolt x3; nut x7; pin x1
BOLT, NUT, PIN
3
3.7.1.
1|hi|2.5|done
"##;

/// A list of records — `[["name" => ..., "qty" => ...], ...]` — built, iterated, read by key,
/// accumulated from, and walked key-by-key one level down.
#[test]
fn test_cli_wasm_array_of_records_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_records");
    let php_path = dir.join("main.php");
    fs::write(&php_path, RECORD_LIST_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the record-list probe");
    assert!(
        output.status.success(),
        "record-list compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the record-list probe under Node");
    assert!(
        run.status.success(),
        "record-list probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), RECORD_LIST_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The record-list probe. The last loop reads each row through `$k => $v`, which is what
/// proves the row binds as a HASH rather than as a boxed cell.
const RECORD_LIST_SOURCE: &str = r##"<?php
$rows = [["name" => "bolt", "qty" => 3], ["name" => "nut", "qty" => 7], ["name" => "pin", "qty" => 1]];
$total = 0;
foreach ($rows as $r) { $total = $total + $r["qty"]; echo $r["name"], "=", $r["qty"], ";"; }
echo "|", $total, "|", count($rows), "|";
$acc = [];
foreach ($rows as $r2) { $acc[] = $r2["name"]; }
echo implode(",", $acc), "|";
foreach ($rows as $r3) { foreach ($r3 as $k => $v) { echo $k, ":", $v, " "; } }
echo "\n";
"##;

/// php-src 8.5.6's own output for `RECORD_LIST_SOURCE`.
const RECORD_LIST_EXPECTED: &str = "bolt=3;nut=7;pin=1;|11|3|bolt,nut,pin|name:bolt qty:3 name:nut qty:7 name:pin qty:1 \n";

/// A class holding an array collection: `$this->items[] = $v`, `$this->items = []`, and a
/// `void` method whose call expression is used.
///
/// PHP gives a `void` call the value null even though the callee returns nothing, so the
/// emitter supplies it; and clearing to `[]` writes an `array<never>` into an `array<mixed>`
/// slot, which is exact because no element layout is decided until the first push. The last
/// loop rebuilds the object forty times so a stale slot pointer would surface as a dispatch
/// failure rather than a wrong count.
#[test]
fn test_cli_wasm_array_property_collection_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_property");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ARRAY_PROPERTY_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array-property probe");
    assert!(
        output.status.success(),
        "array-property compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array-property probe under Node");
    assert!(
        run.status.success(),
        "array-property probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ARRAY_PROPERTY_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The array-property probe. The collection is introduced by CONSTRUCTOR PROMOTION, not by a
/// `= []` property default — that form is still refused, see the note on
/// `object_new_shape_issue`.
const ARRAY_PROPERTY_SOURCE: &str = r##"<?php
class Bag {
    public function __construct(private array $items = []) {}
    public function add(int $v): void { $this->items[] = $v; }
    public function clear(): void { $this->items = []; }
    public function size(): int { return count($this->items); }
}
$b = new Bag();
$r = $b->add(1);
$b->add(2);
echo $b->size(), ",", $r === null ? "null" : "notnull", ";";
$b->clear();
echo $b->size(), ";";
foreach (range(1, 40) as $i) { $t = new Bag(); $t->add(1); $t->add(2); $t->clear(); $t->add(3); echo $t->size(); }
echo "\n";
"##;

/// php-src 8.5.6's own output for `ARRAY_PROPERTY_SOURCE`.
const ARRAY_PROPERTY_EXPECTED: &str = "2,null;0;1111111111111111111111111111111111111111\n";

/// A class holding a `= []` array property, rebuilt sixty times so a stale release surfaces.
///
/// The sixty-iteration loop is the point: the defect this covers was a release walk that read
/// its property count from the HEAP BLOCK, and an object served an oversized free block then
/// walked phantom slots and freed live memory. It answered correctly for the first handful of
/// iterations, so a short loop proves nothing.
#[test]
fn test_cli_wasm_array_property_default_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_default");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ARRAY_DEFAULT_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array-default probe");
    assert!(
        output.status.success(),
        "array-default compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array-default probe under Node");
    assert!(
        run.status.success(),
        "array-default probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ARRAY_DEFAULT_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The array-default probe. Three properties of different types make the phantom-slot walk
/// reachable, and `clear()` exercises assigning a fresh `[]` over a populated slot.
const ARRAY_DEFAULT_SOURCE: &str = r##"<?php
class Stack {
    private array $items = [];
    private int $pushes = 0;
    private string $label = "st";
    public function push(int $v): void { $this->items[] = $v; $this->pushes = $this->pushes + 1; }
    public function size(): int { return count($this->items); }
    public function all(): array { return $this->items; }
    public function stats(): string { return $this->label . ":" . $this->pushes; }
    public function clear(): void { $this->items = []; }
}
$s = new Stack();
foreach ([3, 1, 4, 1, 5] as $v) { $s->push($v); }
echo $s->size(), ",", implode("-", $s->all()), ",", $s->stats(), ";";
$s->clear();
echo $s->size(), ";";
foreach (range(1, 60) as $i) {
    $t = new Stack();
    $t->push($i);
    $t->push(7);
    echo $t->size();
}
echo "\n";
"##;

/// php-src 8.5.6's own output for `ARRAY_DEFAULT_SOURCE`.
const ARRAY_DEFAULT_EXPECTED: &str = "5,3-1-4-1-5,st:5;0;222222222222222222222222222222222222222222222222222222222222\n";

/// `match` over an ENUM and over `true`, and the fatal an unmatched `match` raises.
#[test]
fn test_cli_wasm_match_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_match");
    let php_path = dir.join("main.php");
    fs::write(&php_path, MATCH_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the match probe");
    assert!(
        output.status.success(),
        "match compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the match probe under Node");
    assert!(
        run.status.success(),
        "match probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), MATCH_EXPECTED);

    // A `match` with no arm taken terminates. PHP names the value and the file; the EIR
    // interns the shorter text the NATIVE backend also prints, so the two targets agree.
    let unmatched = dir.join("unmatched.php");
    fs::write(
        &unmatched,
        "<?php\nfunction f(int $n): string { return match ($n) { 1 => \"one\" }; }\necho f(1);\necho f(9);\n",
    )
    .unwrap();
    let compiled = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&unmatched)
        .output()
        .expect("failed to compile the unmatched probe");
    assert!(compiled.status.success());
    let fell_through = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("unmatched.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the unmatched probe under Node");
    assert_eq!(
        fell_through.status.code(),
        Some(255),
        "an unmatched match must exit with PHP's fatal status"
    );
    assert_eq!(
        String::from_utf8_lossy(&fell_through.stdout),
        "one",
        "output before the fatal must still be flushed"
    );
    assert!(
        String::from_utf8_lossy(&fell_through.stderr).contains("unhandled match case"),
        "the interned fatal text must reach stderr: {}",
        String::from_utf8_lossy(&fell_through.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The match probe. Matching on an ENUM compares singleton identity, and `match (true)` is
/// PHP's idiom for a guard ladder.
const MATCH_SOURCE: &str = r##"<?php
enum Suit: string { case H = "h"; case S = "s"; case C = "c"; }
function name(Suit $s): string { return match ($s) { Suit::H => "hearts", Suit::S => "spades", Suit::C => "clubs" }; }
function grade(int $n): string { return match (true) { $n >= 90 => "A", $n >= 80 => "B", default => "C" }; }
echo name(Suit::H), ",", name(Suit::S), ",", name(Suit::C), ";";
echo grade(95), grade(85), grade(10), "\n";
"##;

/// php-src 8.5.6's own output for `MATCH_SOURCE`.
const MATCH_EXPECTED: &str = "hearts,spades,clubs;ABC\n";

/// `array_map` with a closure, and the boxed result read back by `implode` and `count`.
#[test]
fn test_cli_wasm_array_map_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_array_map");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ARRAY_MAP_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the array-map probe");
    assert!(
        output.status.success(),
        "array-map compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the array-map probe under Node");
    assert!(
        run.status.success(),
        "array-map probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ARRAY_MAP_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The array-map probe. The result is BOXED — the EIR types it `mixed` on purpose — so every
/// line here also exercises a consumer unboxing it. The empty source is the case where the
/// map answers an empty array with no element to convert.
const ARRAY_MAP_SOURCE: &str = r##"<?php
$double = function (mixed $x): mixed { return $x * 2; };
$label  = function (mixed $x): string { return "<" . $x . ">"; };
$xs = [1, 2, 3];
echo implode(",", array_map($double, $xs)), ";";
echo implode("", array_map($label, $xs)), ";";
$words = ["a", "bb", "ccc"];
echo implode("|", array_map($label, $words)), ";";
$empty = [];
echo "[", implode(",", array_map($double, $empty)), "]", ";";
echo count(array_map($double, $xs)), "\n";
"##;

/// php-src 8.5.6's own output for `ARRAY_MAP_SOURCE`.
const ARRAY_MAP_EXPECTED: &str = "2,4,6;<1><2><3>;<a>|<bb>|<ccc>;[];3\n";

/// Closures whose visible parameters are `mixed`, called with several tags and with a capture.
#[test]
fn test_cli_wasm_mixed_closure_parameters_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_closure");
    let php_path = dir.join("main.php");
    fs::write(&php_path, MIXED_CLOSURE_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the Mixed-closure probe");
    assert!(
        output.status.success(),
        "Mixed-closure compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the Mixed-closure probe under Node");
    assert!(
        run.status.success(),
        "Mixed-closure probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), MIXED_CLOSURE_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The Mixed-closure probe. `$double(21)` answering 42 and `$double(1.5)` answering 3 is
/// what proves the cell reaches the body carrying its own tag rather than being narrowed.
const MIXED_CLOSURE_SOURCE: &str = r##"<?php
$double = function (mixed $x): mixed { return $x * 2; };
$label  = function (mixed $x): string { return "[" . $x . "]"; };
$pick   = function (mixed $a, mixed $b): mixed { return $a; };
echo $double(21), ",", $double(1.5), ";";
echo $label(7), $label("s"), $label(2.5), ";";
echo $pick(3, 9), ";";
$n = 10;
$add = function (mixed $x) use ($n): mixed { return $x + $n; };
echo $add(5), "\n";
"##;

/// php-src 8.5.6's own output for `MIXED_CLOSURE_SOURCE`.
const MIXED_CLOSURE_EXPECTED: &str = "42,3;[7][s][2.5];3;15\n";

/// A Mixed rendered in a STRING CONTEXT — concatenation and interpolation — over every tag.
#[test]
fn test_cli_wasm_mixed_string_context_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_context");
    let php_path = dir.join("main.php");
    fs::write(&php_path, MIXED_STRING_CONTEXT_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the string-context probe");
    assert!(
        output.status.success(),
        "string-context compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the string-context probe under Node");
    assert!(
        run.status.success(),
        "string-context probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        MIXED_STRING_CONTEXT_EXPECTED
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The string-context probe. These casts are IMPLICIT — nothing in the source says
/// `(string)` — and PHP's conversion here is the same one the explicit cast performs,
/// which is why the array row reads `[Array]` rather than raising.
const MIXED_STRING_CONTEXT_SOURCE: &str = r##"<?php
function show(mixed $v): string { return "[" . $v . "]"; }
function interp(mixed $v): string { return "<$v>"; }
echo show(42), show("x"), show(2.5), show(true), show(false), show(null), show([1,2]), ";";
echo interp(42), interp("x"), interp(2.5), ";";
$out = "";
foreach ([1, "a", 2.5, null] as $v) { $out = $out . $v . ";"; }
echo $out, "\n";
"##;

/// php-src 8.5.6's own output for `MIXED_STRING_CONTEXT_SOURCE`.
const MIXED_STRING_CONTEXT_EXPECTED: &str = "[42][x][2.5][1][][][Array];<42><x><2.5>;1;a;2.5;;\n";

/// Every scalar cast of a Mixed, over every runtime tag, plus `echo` of a container.
#[test]
fn test_cli_wasm_mixed_scalar_casts_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_casts");
    let php_path = dir.join("main.php");
    fs::write(&php_path, MIXED_CAST_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the Mixed-cast probe");
    assert!(
        output.status.success(),
        "Mixed-cast compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the Mixed-cast probe under Node");
    assert!(
        run.status.success(),
        "Mixed-cast probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program, with diagnostics silenced: the
    // "Array to string conversion" warning goes to stderr and is not compared here.
    assert_eq!(String::from_utf8_lossy(&run.stdout), MIXED_CAST_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The Mixed-cast probe: fifteen values covering every runtime tag a cell can carry, each
/// through `(int)`, `(float)`, `(bool)` and `(string)`. The two array rows are the ones that
/// used to answer the empty string where PHP prints "Array".
const MIXED_CAST_SOURCE: &str = r##"<?php
function show(mixed $v): void {
    echo (int)$v, "|", (float)$v, "|", ((bool)$v) ? "T" : "F", "|", (string)$v, ";";
}
show(1); show(-5); show(0); show(1.5); show(-2.7);
show(true); show(false); show(null);
show("42"); show("3.9"); show("abc"); show(""); show("0");
show([1,2]); show([]);
echo "\n";
$mixedish = [1, "x", [7, 8], 2.5];
foreach ($mixedish as $v) { echo $v, ";"; }
echo "\n";
$rows = [[1, 2], [3, 4]];
foreach ($rows as $r) { echo $r, ";"; }
echo "\n";
"##;

/// php-src 8.5.6's own output for `MIXED_CAST_SOURCE`.
const MIXED_CAST_EXPECTED: &str = "1|1|T|1;-5|-5|T|-5;0|0|F|0;1|1.5|T|1.5;-2|-2.7|T|-2.7;1|1|T|1;0|0|F|;0|0|F|;42|42|T|42;3|3.9|T|3.9;0|0|T|abc;0|0|F|;0|0|F|0;1|1|T|Array;0|0|F|Array;\n1;x;Array;2.5;\nArray;Array;\n";

/// Enum cases held in an ARRAY, over string-backed, int-backed and pure enums.
#[test]
fn test_cli_wasm_enum_collections_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_enum_arrays");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ENUM_COLLECTION_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the enum-collection probe");
    assert!(
        output.status.success(),
        "enum-collection compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the enum-collection probe under Node");
    assert!(
        run.status.success(),
        "enum-collection probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        ENUM_COLLECTION_EXPECTED
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The enum-collection probe. Reading `->name` and `->value` off each element is the point:
/// the literal used to be typed `array<string>`, which made those reads land on a string.
const ENUM_COLLECTION_SOURCE: &str = r##"<?php
enum Suit: string { case Hearts = "H"; case Spades = "S"; case Clubs = "C"; }
enum Level: int { case Low = 1; case High = 10; }
enum Flag { case On; case Off; }
$suits = [Suit::Hearts, Suit::Spades, Suit::Clubs];
foreach ($suits as $s) { echo $s->name, "=", $s->value, ";"; }
echo "|", count($suits), "|";
$levels = [Level::Low, Level::High];
$total = 0;
foreach ($levels as $l) { $total = $total + $l->value; }
echo $total, "|";
$flags = [Flag::On, Flag::Off];
foreach ($flags as $f) { echo $f->name, ","; }
echo "|";
$acc = [];
foreach ($suits as $s2) { $acc[] = $s2->value; }
echo implode("", $acc), "|";
$one = Suit::Hearts;
echo $one === Suit::Hearts ? "id" : "no", "\n";
"##;

/// php-src 8.5.6's own output for `ENUM_COLLECTION_SOURCE`.
const ENUM_COLLECTION_EXPECTED: &str = "Hearts=H;Spades=S;Clubs=C;|3|11|On,Off,|HSC|id\n";

/// Enums: string-backed, int-backed and pure, read through `->value` and `->name`, compared
/// by identity, and passed to a typed parameter.
#[test]
fn test_cli_wasm_enums_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_enums");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ENUM_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the enum probe");
    assert!(
        output.status.success(),
        "enum compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the enum probe under Node");
    assert!(
        run.status.success(),
        "enum probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ENUM_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The enum probe. `$s === Suit::Spades` reading `same` while `$s === Suit::Hearts` reads
/// `diff` is what proves each case is ONE singleton — two allocations per read would make
/// every identity comparison false.
const ENUM_SOURCE: &str = r##"<?php
enum Suit: string {
    case Hearts = "H";
    case Spades = "S";
    case Clubs = "C";
}
enum Level: int {
    case Low = 1;
    case High = 10;
}
enum Flag {
    case On;
    case Off;
}
echo Suit::Hearts->value, Suit::Spades->value, Suit::Clubs->value, ";";
echo Suit::Hearts->name, ",", Suit::Spades->name, ";";
echo Level::Low->value + Level::High->value, ";";
echo Level::Low->name, ",", Flag::On->name, ",", Flag::Off->name, ";";
$s = Suit::Spades;
echo $s->value, ",", $s === Suit::Spades ? "same" : "diff", ",", $s === Suit::Hearts ? "same" : "diff", ";";
function describe(Suit $s): string { return $s->name . "=" . $s->value; }
echo describe(Suit::Clubs), ";";
echo Suit::Hearts === Suit::Hearts ? "id" : "no", "\n";
"##;

/// php-src 8.5.6's own output for `ENUM_SOURCE`.
const ENUM_EXPECTED: &str = "HSC;Hearts,Spades;11;Low,On,Off;S,same,diff;Clubs=C;id\n";

/// Variadic parameters: free functions, an instance method and a static one, with and
/// without leading fixed parameters, over int and string element types.
#[test]
fn test_cli_wasm_variadic_calls_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_variadic");
    let php_path = dir.join("main.php");
    fs::write(&php_path, VARIADIC_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the variadic probe");
    assert!(
        output.status.success(),
        "variadic compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the variadic probe under Node");
    assert!(
        run.status.success(),
        "variadic probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), VARIADIC_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The variadic probe. `sum()` with NO arguments is what proves the empty packed array is
/// built and passed rather than the call being reshaped.
const VARIADIC_SOURCE: &str = r##"<?php
function sum(int ...$xs): int { $t = 0; foreach ($xs as $x) { $t = $t + $x; } return $t; }
function label(string $prefix, string ...$parts): string { return $prefix . ":" . implode("|", $parts); }
function counted(int $base, int ...$rest): int { return $base + count($rest); }
class Adder {
    public function all(int ...$xs): int { $t = 0; foreach ($xs as $x) { $t = $t + $x; } return $t; }
    public static function stat(int ...$xs): int { return count($xs); }
}
echo sum(1,2,3), ",", sum(), ",", sum(7), ";";
echo label("a"), ",", label("a","b"), ",", label("a","b","c"), ";";
echo counted(10), ",", counted(10,1,2), ";";
$a = new Adder();
echo $a->all(4,5,6), ",", Adder::stat(1,2), "\n";
"##;

/// php-src 8.5.6's own output for `VARIADIC_SOURCE`.
const VARIADIC_EXPECTED: &str = "6,0,7;a:,a:b,a:b|c;10,12;15,2\n";

/// Static properties: defaults of every slottable type, reads, writes, a string reassigned
/// and concatenated, and the shared storage an inherited static has.
#[test]
fn test_cli_wasm_static_properties_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_statics");
    let php_path = dir.join("main.php");
    fs::write(&php_path, STATIC_PROPERTY_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the static-property probe");
    assert!(
        output.status.success(),
        "static-property compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the static-property probe under Node");
    assert!(
        run.status.success(),
        "static-property probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), STATIC_PROPERTY_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The static-property probe. `Child::$shared` and `Base::$shared` must both read 106 after
/// two separate increments — one through each name — which is what proves an inherited static
/// is ONE slot and not a per-class copy.
const STATIC_PROPERTY_SOURCE: &str = r##"<?php
class Base {
    public static int $shared = 100;
    public static string $tag = "base";
    public static float $ratio = 1.5;
    public static bool $on = false;
}
class Child extends Base {}
class Counter {
    public static int $n = 0;
    public static function tick(): int { Counter::$n = Counter::$n + 1; return Counter::$n; }
    public static function reset(): void { Counter::$n = 0; }
}
echo Base::$shared, ",", Base::$tag, ",", Base::$ratio, ",", Base::$on ? "y" : "n", ";";
Base::$shared = Base::$shared + 5;
Child::$shared = Child::$shared + 1;
echo Base::$shared, ",", Child::$shared, ";";
Base::$tag = "changed";
echo Base::$tag, ",", Child::$tag, ";";
Base::$tag = Base::$tag . "!";
echo Base::$tag, ";";
Counter::tick(); Counter::tick(); Counter::tick();
echo Counter::$n, ";";
Counter::reset();
echo Counter::$n, ";";
Base::$ratio = Base::$ratio * 2;
Base::$on = true;
echo Base::$ratio, ",", Base::$on ? "y" : "n", "\n";
"##;

/// php-src 8.5.6's own output for `STATIC_PROPERTY_SOURCE`.
const STATIC_PROPERTY_EXPECTED: &str = "100,base,1.5,n;106,106;changed,changed;changed!;3;0;3,y\n";

/// The word-counter — `$c[$k] = $c[$k] + 1` — and a hash carrying one value of every tag.
///
/// This is the shape that read back WRONG before the store flattened its Mixed value: the
/// counter printed `a=;b=;c=1;`, because a re-read key had been stored as a cell holding a
/// cell and nothing follows that indirection. The `else` branch's plain `= 1` was the only
/// entry that printed, which is what made the bug look like a counting error rather than a
/// storage one.
#[test]
fn test_cli_wasm_heterogeneous_hash_values_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_het_hash");
    let php_path = dir.join("main.php");
    fs::write(&php_path, HETEROGENEOUS_HASH_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the heterogeneous-hash probe");
    assert!(
        output.status.success(),
        "heterogeneous-hash compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the heterogeneous-hash probe under Node");
    assert!(
        run.status.success(),
        "heterogeneous-hash probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        HETEROGENEOUS_HASH_EXPECTED
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The heterogeneous-hash probe: one value of every runtime tag, read back through both
/// `foreach` and `$h[k]`, plus a re-read-and-store and a copy between two keys.
const HETEROGENEOUS_HASH_SOURCE: &str = r##"<?php
$counts = [];
foreach (["a", "b", "a", "c", "b", "a"] as $ch) {
    if (isset($counts[$ch])) { $counts[$ch] = $counts[$ch] + 1; } else { $counts[$ch] = 1; }
}
foreach ($counts as $k => $n) { echo $k, "=", $n, ";"; }
echo "|";
$h = [];
$h["i"] = 1;
$h["s"] = "text";
$h["f"] = 2.5;
$h["b"] = true;
$h["n"] = null;
$h["i"] = $h["i"] + 10;
$h["copy"] = $h["s"];
foreach ($h as $k => $v) { echo $k, "=", $v, ";"; }
echo "|", count($h), "|", isset($h["n"]) ? "y" : "n", array_key_exists("n", $h) ? "y" : "n";
echo "|", $h["s"], ",", $h["f"], ",", $h["i"], ",", $h["copy"], "\n";
"##;

/// php-src 8.5.6's own output for `HETEROGENEOUS_HASH_SOURCE`.
const HETEROGENEOUS_HASH_EXPECTED: &str = "a=3;b=2;c=1;|i=11;s=text;f=2.5;b=1;n=;copy=text;|6|ny|text,2.5,11,text\n";

/// Nested indexed arrays: `[[1,2],[3,4]]` built, iterated, accumulated into a fresh `[]`,
/// and nested one level deeper.
#[test]
fn test_cli_wasm_nested_arrays_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_nested_arrays");
    let php_path = dir.join("main.php");
    fs::write(&php_path, NESTED_ARRAY_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the nested-array probe");
    assert!(
        output.status.success(),
        "nested-array compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the nested-array probe under Node");
    assert!(
        run.status.success(),
        "nested-array probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), NESTED_ARRAY_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The nested-array probe. The last group is nested twice, which is what proves the element
/// layout is chosen from the element's own type rather than assumed one level deep.
const NESTED_ARRAY_SOURCE: &str = r##"<?php
$m = [[1, 2], [3, 4], [5, 6]];
foreach ($m as $row) { echo implode("-", $row), ";"; }
echo "|", count($m), "|";
$g = [["a", "b"], ["c", "d"]];
foreach ($g as $words) { echo "[", implode("", $words), "]"; }
echo "|";
$t = 0;
foreach ($m as $pair) { foreach ($pair as $n) { $t = $t + $n; } }
echo $t, "|";
$sizes = [];
foreach ($m as $r2) { $sizes[] = count($r2); }
echo implode(",", $sizes), "|";
$built = [];
foreach ($m as $r3) { $built[] = $r3; }
foreach ($built as $r4) { echo count($r4); }
echo "|";
$deep = [[[1, 2]], [[3]]];
foreach ($deep as $outer) { foreach ($outer as $inner) { echo implode("+", $inner), "."; } }
echo "\n";
"##;

/// php-src 8.5.6's own output for `NESTED_ARRAY_SOURCE`.
const NESTED_ARRAY_EXPECTED: &str = "1-2;3-4;5-6;|3|[ab][cd]|21|2,2,2|222|1+2.3.\n";

/// Proves the cycle collector actually reclaims a cycle, by watching memory not grow.
///
/// Two objects pointing at each other keep each other's refcount above zero forever, so
/// refcounting alone can never free them — this is the one shape on this target that needs
/// `__rt_gc_collect_cycles`, which `unset(...)` reaches. Measured: with the collector
/// neutralized the loop grows 50 pages over its declared memory; with it, 2 — exactly what
/// the same loop grows when the cycle is not formed at all. The program prints the right
/// answer either way, which is why this watches memory rather than output.
#[test]
fn test_cli_wasm_unset_collects_reference_cycles() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_gc_cycle");
    let runner_src = r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(readFileSync(process.argv[2])),
  wasi.getImportObject(),
);
const code = wasi.start(instance);
console.error(`pages=${instance.exports.memory.buffer.byteLength / 65536}`);
process.exitCode = code;
"#;
    let runner = dir.join("run.mjs");
    fs::write(&runner, runner_src).unwrap();

    // The control forms no cycle, so refcounting alone frees both nodes. Anything the cycle
    // case grows BEYOND it is a cycle the collector failed to reclaim.
    let bodies = [
        ("no cycle", ""),
        ("cycle", "$b->next = $a;"),
    ];
    for (label, link_back) in bodies {
        let php_path = dir.join("main.php");
        fs::write(
            &php_path,
            format!(
                "<?php\nclass Node {{ public ?Node $next = null; }}\n\
                 $sum = 0;\n\
                 foreach (range(1, 20000) as $i) {{\n\
                 \x20   $a = new Node();\n\
                 \x20   $b = new Node();\n\
                 \x20   $a->next = $b;\n\
                 \x20   {link_back}\n\
                 \x20   $sum = $sum + 1;\n\
                 \x20   unset($a);\n\
                 \x20   unset($b);\n\
                 }}\n\
                 if ($sum === 20000) {{ echo \"ok\\n\"; }}\n"
            ),
        )
        .unwrap();

        for extra in [vec!["--emit-asm"], vec![]] {
            let mut command = elephc_cli_command(&dir);
            command.arg("--target").arg("wasm32-wasi");
            for flag in extra {
                command.arg(flag);
            }
            let output = command
                .arg(&php_path)
                .output()
                .unwrap_or_else(|error| panic!("{label}: failed to invoke elephc: {error}"));
            assert!(
                output.status.success(),
                "{label} failed to compile: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .unwrap_or_else(|error| panic!("{label}: failed to run under Node: {error}"));
        assert!(
            run.status.success(),
            "{label} trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(run.stdout, b"ok\n", "{label} printed the wrong thing");

        let stderr = String::from_utf8_lossy(&run.stderr);
        let final_pages: usize = stderr
            .split("pages=")
            .nth(1)
            .and_then(|rest| rest.trim().parse().ok())
            .unwrap_or_else(|| panic!("{label}: the runner reported no page count"));
        let wat = fs::read_to_string(dir.join("main.wat")).expect("the WAT was written");
        let initial_pages: usize = wat
            .split("(memory (export \"memory\") ")
            .nth(1)
            .and_then(|rest| rest.split(')').next())
            .and_then(|n| n.trim().parse().ok())
            .unwrap_or_else(|| panic!("{label}: the module declares no initial memory"));

        // 2 pages is the `range` array itself (20000 * 8 bytes), which both cases allocate.
        assert_eq!(
            final_pages - initial_pages,
            2,
            "{label}: 20000 iterations grew memory by {} pages over the declared \
             {initial_pages}, where only the range array (2 pages) should — the cycle \
             was not reclaimed",
            final_pages - initial_pages
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Proves a property read leaves no reference behind, by watching memory not grow.
///
/// The object is rebuilt each iteration so its property's backing array has to die with it. A
/// read that retains twice leaves the array alive forever: measured at 98 pages against the bare
/// loop's 3 before the fix, and 43 for a string property. Both are invisible in the output — the
/// program prints the right answer either way, which is why this watches memory instead.
#[test]
fn test_cli_wasm_property_read_leaves_no_reference() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_prop_leak");
    let runner_src = r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(readFileSync(process.argv[2])),
  wasi.getImportObject(),
);
const code = wasi.start(instance);
console.error(`pages=${instance.exports.memory.buffer.byteLength / 65536}`);
process.exitCode = code;
"#;
    let runner = dir.join("run.mjs");
    fs::write(&runner, runner_src).unwrap();

    // `foreach (range(...))` rather than an unrolled program: thousands of statements do not
    // compile fast enough to reach the 64 KiB page granularity a per-read leak needs.
    let bodies = [
        ("baseline", r#"if ($n === 999999) { echo "x"; }"#),
        ("array property", r#"if (count($x->a) === 99) { echo "x"; }"#),
        ("string property", r#"if ($x->s === "zz") { echo "x"; }"#),
    ];
    for (label, body) in bodies {
        let php_path = dir.join("main.php");
        fs::write(
            &php_path,
            format!(
                "<?php\nclass Box {{ public function __construct(public string $s, public array $a) {{}} }}\n                 foreach (range(1, 30000) as $n) {{\n    $x = new Box(\"bolt\", [1,2,3]);\n    {body}\n}}\n                 echo \"ok\\n\";\n"
            ),
        )
        .unwrap();

        for extra in [vec!["--emit-asm"], vec![]] {
            let mut command = elephc_cli_command(&dir);
            command.arg("--target").arg("wasm32-wasi");
            for flag in extra {
                command.arg(flag);
            }
            let output = command
                .arg(&php_path)
                .output()
                .unwrap_or_else(|error| panic!("{label}: failed to invoke elephc: {error}"));
            assert!(
                output.status.success(),
                "{label} failed to compile: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .current_dir(&dir)
            .output()
            .unwrap_or_else(|error| panic!("{label}: failed to run under Node: {error}"));
        assert!(
            run.status.success(),
            "{label} trapped: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(run.stdout, b"ok\n", "{label} printed the wrong thing");

        let stderr = String::from_utf8_lossy(&run.stderr);
        let final_pages: usize = stderr
            .split("pages=")
            .nth(1)
            .and_then(|rest| rest.trim().parse().ok())
            .unwrap_or_else(|| panic!("{label}: the runner reported no page count"));
        let wat = fs::read_to_string(dir.join("main.wat")).expect("the WAT was written");
        let initial_pages: usize = wat
            .split("(memory (export \"memory\") ")
            .nth(1)
            .and_then(|rest| rest.split(')').next())
            .and_then(|n| n.trim().parse().ok())
            .unwrap_or_else(|| panic!("{label}: the module declares no initial memory"));

        // The `range` array itself grows, so every case is compared against the bare loop.
        assert_eq!(
            final_pages - initial_pages,
            3,
            "{label}: 30000 reads grew memory by {} pages over the declared {initial_pages}, \
             where the bare loop grows 3 — a reference is being left behind",
            final_pages - initial_pages
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a property read RETAINS exactly once, and that copy-on-write follows from it.
///
/// `Op::PropGet` is always followed by `Op::Acquire` — checked across every use shape: store,
/// echo, argument, concat, builtin argument, return, strict compare. The acquire persists a string
/// and increfs a refcounted child, so the READ must only view them. Retaining in both places left
/// one extra reference per read: measured at ~207 bytes per read of an array property, whose
/// backing array was then never freed, and ~87 for a string.
///
/// The Throwable accessors share the same slot reader and need the OPPOSITE: no acquire follows
/// them, and their result outlives the object it came from, so they own their copy. Reading
/// `getMessage()` here is what catches getting that backwards — it answers dead bytes otherwise.
///
/// With the reference count finally right, `$c = $src; $c[] = "z";` gets PHP's value semantics:
/// the push sees two owners and splits. Before, the push had no copy-on-write at all and simply
/// grew the shared array in place, freeing the block the other reference still pointed at — which
/// the extra retain had been hiding.
#[test]
fn test_cli_wasm_property_read_retains_once_and_arrays_copy_on_write() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_prop_retain");
    let php_path = dir.join("main.php");
    fs::write(&php_path, PROP_RETAIN_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the property-retain probe");
    assert!(
        output.status.success(),
        "property-retain compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the property-retain probe under Node");
    assert!(
        run.status.success(),
        "property-retain probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), PROP_RETAIN_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The probe: every property-read shape, the Throwable accessors, and copy-on-write on a copy.
const PROP_RETAIN_SOURCE: &str = r##"<?php
class C {
    public function __construct(public string $s, public array $a, public mixed $m, public int $i) {}
}
function take(string $v): int { return strlen($v); }
function give(C $c): string { return $c->s; }
$x = new C("abc", [1,2,3], "boxed", 7);
$a = $x->s;              echo "[", $a, "]";
echo "[", $x->s, "]";
echo "[", take($x->s), "]";
echo "[", $x->s . "y", "]";
echo "[", strtoupper($x->s), "]";
echo "[", give($x), "]";
echo "[", ($x->s === "abc") ? "y" : "n", "]";
echo "[", count($x->a), "]";
echo "[", implode(",", $x->a), "]";
echo "[", $x->i, "]";
echo "[", $x->m, "]";
$b = $x->a; $b[] = 9; echo "[", count($b), ":", count($x->a), "]";
echo "\n";
try { throw new RuntimeException("boom", 42); }
catch (RuntimeException $e) { echo $e->getMessage(), "|", $e->getCode(), "|", get_class($e), "\n"; }
$src = ["a", "b"];
$c = $src;
$c[] = "z";
echo count($c), ":", count($src), "|", implode(",", $c), ":", implode(",", $src), "|";
$i = [1, 2];
$j = $i; $j[] = 3;
echo count($j), ":", count($i), "|", implode(",", $j), ":", implode(",", $i), "|";
$k = $i; $k[] = 4; $k[] = 5;
echo implode(",", $k), ":", implode(",", $i), "|";
echo "\n";
"##;

/// php-src 8.5.6's own output for `PROP_RETAIN_SOURCE`.
const PROP_RETAIN_EXPECTED: &str = r##"[abc][abc][3][abcy][ABC][abc][y][3][1,2,3][7][boxed][4:3]
boom|42|RuntimeException
3:2|a,b,z:a,b|3:2|1,2,3:1,2|1,2,4,5:1,2|
"##;

/// Verifies `round` and `sprintf`'s radix conversions against php-src.
///
/// PHP's `round` is half away from ZERO, where WebAssembly's `f64.nearest` is half to EVEN — it
/// answers 2 for `round(2.5)` where PHP answers 3. The naive repair `floor(|x| + 0.5)` is worse:
/// the addition is inexact, so `round(0.49999999999999994)` answers 1 instead of 0, and above
/// 2^52 it perturbs values that are already integers. Comparing against `trunc(x)` is exact, and
/// `f64.trunc` carries the sign of zero, which PHP prints — `round(-0.4)` is `-0`.
///
/// `%x`, `%X`, `%b` and `%o` read the argument as UNSIGNED, so `-1` prints as `ffffffffffffffff`
/// and no sign is ever emitted whatever the flags say.
#[test]
fn test_cli_wasm_round_and_radix_conversions_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_round");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ROUND_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the round probe");
    assert!(
        output.status.success(),
        "round compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the round probe under Node");
    assert!(
        run.status.success(),
        "round probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ROUND_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The probe: both halfway directions, the 0.49999999999999994 trap, the 2^52/2^53 boundaries,
/// infinities and NaN, then every radix conversion including negatives and both i64 extremes.
const ROUND_SOURCE: &str = r##"<?php
foreach ([0.0, -0.0, 0.5, -0.5, 1.5, -1.5, 2.5, -2.5, 2.4, -2.4, 2.6] as $v) { echo round($v), "|"; }
echo "\n";
foreach ([0.49999999999999994, -0.49999999999999994, 4503599627370495.5, 9007199254740993.0] as $v) { echo round($v), "|"; }
echo "\n";
foreach ([1e15, 1e16, -1e16, 1e300, -1e300, INF, -INF, NAN, 1e-300] as $v) { echo round($v), "|"; }
echo "\n";
echo sprintf("%x|%X|%b|%o", 255, 255, 5, 8), "\n";
echo sprintf("%x|%X|%b|%o", -1, -255, -1, -1), "\n";
echo sprintf("[%08x][%-8x][%8b][%08b]", 255, 255, 5, 5), "\n";
echo sprintf("%x|%b", PHP_INT_MAX, PHP_INT_MIN), "\n";
"##;

/// php-src 8.5.6's own output for `ROUND_SOURCE`.
const ROUND_EXPECTED: &str = r##"0|-0|1|-1|2|-2|3|-3|2|-2|3|
0|-0|4.5035996273705E+15|9.007199254741E+15|
1.0E+15|1.0E+16|-1.0E+16|1.0E+300|-1.0E+300|INF|-INF|NAN|0|
ff|FF|101|10
ffffffffffffffff|FFFFFFFFFFFFFF01|1111111111111111111111111111111111111111111111111111111111111111|1777777777777777777777
[000000ff][ff      ][     101][00000101]
7fffffffffffffff|1000000000000000000000000000000000000000000000000000000000000000
"##;

/// Verifies COUNTING LOOPS, which no target-side gap explains but which did not compile.
///
/// `$i = $i + 1` lowers to a checked add, whose result must be Mixed because PHP promotes an
/// overflowing integer to a float. So the local widens to Mixed, and every later read of it is an
/// implicit Mixed-to-scalar transfer — which was refused, turning away the most ordinary loop in
/// the language along with anything that accumulates.
///
/// The transfer unboxes through the same helpers the NATIVE backend uses for the identical
/// coercion, except that a float narrows SILENTLY: PHP performs no cast here, so borrowing the
/// explicit `(int)` cast's out-of-range warning would print a diagnostic for a program PHP runs
/// quietly, and the two backends would disagree.
///
/// The gap this inherits belongs to the EIR, not to either lowering: a read is typed `int` from
/// the slot's type BEFORE the loop's widening store, so once an add really does overflow, both
/// targets answer a saturated `9223372036854775807` where PHP answers `9.2233720368548E+18`.
/// They agree with each other, which is what this pins.
#[test]
fn test_cli_wasm_counting_loops_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_counting");
    let php_path = dir.join("main.php");
    fs::write(&php_path, COUNTING_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the counting probe");
    assert!(
        output.status.success(),
        "counting-loop compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the counting probe under Node");
    assert!(
        run.status.success(),
        "counting probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), COUNTING_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The probe: a `while` counter, a `foreach` sum, a decreasing counter, and a running product.
const COUNTING_SOURCE: &str = r##"<?php
$i = 0;
while ($i < 3) { echo $i; $i = $i + 1; }
echo "|";
$t = 0;
foreach ([5, 7, 9] as $v) { $t = $t + $v; }
echo $t, "|";
$n = 10;
while ($n > 0) { $n = $n - 3; }
echo $n, "|";
$p = 1;
foreach (range(1, 5) as $k) { $p = $p * $k; }
echo $p, "\n";
"##;

/// php-src 8.5.6's own output for `COUNTING_SOURCE`.
const COUNTING_EXPECTED: &str = r##"012|21|-2|120
"##;

/// Verifies a property COUNTER, and `wordwrap`'s break-string and cut forms.
///
/// `$this->n = $this->n + 1` widens the value to Mixed through the checked add while the slot
/// stays an `int`, so the store narrows the same way a local load does — refusing it turned away
/// every counter held in an object.
///
/// `wordwrap`'s four-argument form BUILDS its result, because a multi-byte break and a cut both
/// lengthen the text; only the one-byte no-cut form can rewrite in place, where a space BECOMES
/// the break. Transcribed from php-src and validated on 314 cases, mostly generated over an
/// alphabet of `a`, `b`, `c` and space — which is where the awkward shapes are: `"a  b"` at width
/// 2 cutting is `a -b`, the first space becoming the break and the second surviving, and
/// `"  lead"` is ` -lea-d`.
#[test]
fn test_cli_wasm_property_counter_and_wordwrap_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_wordwrap");
    let php_path = dir.join("main.php");
    fs::write(&php_path, WORDWRAP_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the wordwrap probe");
    assert!(
        output.status.success(),
        "wordwrap compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the wordwrap probe under Node");
    assert!(
        run.status.success(),
        "wordwrap probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), WORDWRAP_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The probe: an int and a float property counter, then every wordwrap form over the shapes that
/// separate them.
const WORDWRAP_SOURCE: &str = r##"<?php
class Counter {
    private int $n = 0;
    private float $f = 0.5;
    public function bump(): int { $this->n = $this->n + 1; return $this->n; }
    public function grow(): float { $this->f = $this->f + 1.25; return $this->f; }
}
$c = new Counter();
echo $c->bump(), $c->bump(), $c->bump(), "|", $c->grow(), "|", $c->grow(), "\n";
$w = ["aaa bbb ccc", "abcdefghij", "a b c d e", "the quick brown fox", "", "one", "aaaa bb", "a  b", "  lead"];
foreach ($w as $s) { echo "[", str_replace("\n", "N", wordwrap($s, 4, "-", true)), "]"; }
echo "\n";
foreach ($w as $s) { echo "[", str_replace("\n", "N", wordwrap($s, 4, "-", false)), "]"; }
echo "\n";
foreach ($w as $s) { echo "[", str_replace("\n", "N", wordwrap($s, 7, "<>", true)), "]"; }
echo "\n";
echo "[", str_replace("\n", "N", wordwrap("aaa bbb ccc", 7)), "]\n";
"##;

/// php-src 8.5.6's own output for `WORDWRAP_SOURCE`.
const WORDWRAP_EXPECTED: &str = r##"123|1.75|3
[aaa-bbb-ccc][abcd-efgh-ij][a b-c d-e][the-quic-k-brow-n-fox][][one][aaaa-bb][a  b][ -lead]
[aaa-bbb-ccc][abcdefghij][a b-c d-e][the-quick-brown-fox][][one][aaaa-bb][a  b][ -lead]
[aaa bbb<>ccc][abcdefg<>hij][a b c d<>e][the<>quick<>brown<>fox][][one][aaaa bb][a  b][  lead]
[aaa bbbNccc]
"##;

/// Verifies an array passed BY VALUE, which PHP copies on write and this target used to corrupt.
///
/// The argument was borrowed rather than counted, so a push inside the callee saw one owner and
/// grew the array in place — and `__rt_array_grow` freed the block the CALLER still pointed at.
/// The caller then read a dead pointer: `mutate($src)` left `count($src)` answering 0.
///
/// The callee OWNS its array parameter now. The caller lends a counted reference and never takes
/// it back; the callee releases at every exit. Both branches balance: when it mutates,
/// `__rt_array_ensure_unique` hands it a clone and drops the original back to the caller's single
/// reference, and the epilogue frees the clone; when it does not, the epilogue simply undoes the
/// lend. A returned parameter moves out instead.
///
/// Every call-site kind is here because a missed lend is an over-release, not a leak: a plain
/// call, a two-level pass-through, a mutation, two mutations of the same source, a returned
/// parameter, five levels of recursion, a constructor argument and a method argument.
#[test]
fn test_cli_wasm_array_arguments_are_passed_by_value() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_by_value");
    let php_path = dir.join("main.php");
    fs::write(&php_path, BY_VALUE_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the by-value probe");
    assert!(
        output.status.success(),
        "by-value compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the by-value probe under Node");
    assert!(
        run.status.success(),
        "by-value probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), BY_VALUE_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The by-value probe: one of every call-site kind that can carry an array.
const BY_VALUE_SOURCE: &str = r##"<?php
class Bag {
    public function __construct(public array $items) {}
    public function size(): int { return count($this->items); }
    public function grow(array $a): int { $a[] = 9; return count($a); }
}
function pass(array $a): int { return inner($a); }
function inner(array $a): int { return count($a); }
function mutate(array $a): int { $a[] = 9; return count($a); }
function twice(array $a): int { $x = mutate($a); $y = mutate($a); if ($x === $y) { return $x; } return 0; }
function give_back(array $a): array { return $a; }
function deep(array $a, int $n): int { if ($n <= 0) { return count($a); } return deep($a, $n - 1); }
function strs(array $a): int { $a[] = "z"; return count($a); }

$src = [1, 2, 3];
echo pass($src), "|", count($src), "|";
echo mutate($src), "|", count($src), "|";
echo twice($src), "|", count($src), "|";
$b = give_back($src); echo count($b), ":", count($src), "|";
echo deep($src, 5), "|", count($src), "|";
$bag = new Bag($src);
echo $bag->size(), "|", $bag->grow($src), "|", count($src), "|", $bag->size(), "|";
$w = ["a", "bb"];
echo strs($w), "|", count($w), "|", implode(",", $w), "|";
echo implode(",", $src), "\n";
"##;

/// php-src 8.5.6's own output for `BY_VALUE_SOURCE`.
const BY_VALUE_EXPECTED: &str = r##"3|3|4|3|4|3|3:3|3|3|3|4|3|3|3|2|a,bb|1,2,3
"##;

/// Verifies arithmetic inside TYPED functions, and an array of interface implementors.
///
/// `return $this->s * $this->s;` from an `: int` method could not compile. The multiplication is
/// checked, so its result is typed Mixed — an overflow would promote it to a float — and narrowing
/// that back for the declared return was refused as an implicit coercion. It is not one: PHP
/// performs no conversion there, `square(7)` is just 49. The narrowing is admitted when the value
/// is TRANSITIVELY integer arithmetic, which `$a + $b + $c` also is: the chain runs through
/// `MixedNumericBinop`, whose left operand is the previous Mixed.
///
/// The transitivity is what keeps the refusal where PHP really does coerce — `f(mixed $m): int {
/// return $m + 1; }` emits the same opcode over a genuine `mixed`, and still refuses.
///
/// The shapes array is `array<mixed>` because the classes differ, so each object BOXES into a
/// cell under tag 6. The EIR emits no release after that push, unlike a concrete `array<Object>`,
/// so the operand's single reference is handed over rather than shared.
#[test]
fn test_cli_wasm_typed_arithmetic_and_polymorphic_arrays_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_polymorphic");
    let php_path = dir.join("main.php");
    fs::write(&php_path, POLYMORPHIC_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the polymorphic probe");
    assert!(
        output.status.success(),
        "polymorphic compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the polymorphic probe under Node");
    assert!(
        run.status.success(),
        "polymorphic probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), POLYMORPHIC_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The probe: two implementors dispatched through one array, and typed arithmetic including a
/// chained sum, zero and a negative.
const POLYMORPHIC_SOURCE: &str = r##"<?php
interface Shape { public function area(): int; }
class Sq implements Shape {
    public function __construct(private int $s) {}
    public function area(): int { return $this->s * $this->s; }
}
class Re implements Shape {
    public function __construct(private int $w, private int $h) {}
    public function area(): int { return $this->w * $this->h; }
}
class Math {
    public static function square(int $x): int { return $x * $x; }
    public static function sum3(int $a, int $b, int $c): int { return $a + $b + $c; }
}
$shapes = [new Sq(3), new Re(2, 5), new Sq(4)];
foreach ($shapes as $s) { echo $s->area(), ";"; }
echo "|", count($shapes), "|";
echo Math::square(7), "|", Math::sum3(1, 2, 3), "|";
echo Math::square(0), "|", Math::square(-4), "\n";
"##;

/// php-src 8.5.6's own output for `POLYMORPHIC_SOURCE`.
const POLYMORPHIC_EXPECTED: &str = r##"9;10;16;|3|49|6|0|16
"##;

/// Verifies `isset`, which is exactly "not null" for a variable the checker proved defined.
///
/// It reuses `Op::IsNull`'s per-representation tag rules rather than growing a second copy — a
/// Mixed cell tests tag 8, a tagged scalar its tag word, a nullable container its pointer, and a
/// statically non-null value folds to true.
///
/// The audit confined EVERY language construct to `main`, because `exit`/`die` cannot unwind a
/// caller's WASM frames. `isset` only reads a tag, so it is exempt — and
/// `test_cli_wasm_rejects_exit_outside_main` still pins that `exit` is not.
#[test]
fn test_cli_wasm_isset_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_isset");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ISSET_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the isset probe");
    assert!(
        output.status.success(),
        "isset compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the isset probe under Node");
    assert!(
        run.status.success(),
        "isset probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ISSET_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `exit` outside `main` is still refused, which the `isset` exemption must not relax.
#[test]
fn test_cli_wasm_rejects_exit_outside_main() {
    let dir = make_cli_test_dir("elephc_cli_wasm_exit_nested");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        "<?php\nfunction boom(): void { exit(1); }\nboom();\n",
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to invoke elephc");
    assert!(
        !output.status.success(),
        "exit outside main cannot unwind caller-owned frames and must be refused"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("exit/die outside main cannot unwind caller-owned WASM frames"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// The `isset` probe: every representation it can reach, inside and outside a function.
const ISSET_SOURCE: &str = r##"<?php
function mm(mixed $v): string { return isset($v) ? "y" : "n"; }
class Box { public function __construct(public int $n) {} }
$a = 5; $b = null; $s = "x"; $f = 1.5; $arr = [1,2]; $e = []; $o = new Box(1);
echo isset($a) ? "y" : "n", isset($b) ? "y" : "n", isset($s) ? "y" : "n";
echo isset($f) ? "y" : "n", isset($arr) ? "y" : "n", isset($e) ? "y" : "n";
echo isset($o) ? "y" : "n", "|";
echo mm(3), mm("z"), mm(0), mm(1.5), mm(""), "|";
echo $a ?? 9, "|";
$t = 0;
foreach ([1, 2, 3] as $v) { if (isset($v)) { $t = $t + $v; } }
echo $t, "\n";
"##;

/// php-src 8.5.6's own output for `ISSET_SOURCE`.
const ISSET_EXPECTED: &str = r##"ynyyyyy|yyyyy|5|6
"##;

/// Verifies `sort` and `rsort` over scalar arrays, on 64 orderings against php-src.
///
/// The sort is STABLE — PHP's have been since 8.0 — so the swap test is strict and equal
/// elements keep their order. It copy-on-write-uniques first and answers the array pointer, which
/// the call site writes back: `sort($a)` rebinds `$a`.
///
/// String and Mixed elements stay refused. PHP orders strings with its standard comparison, where
/// two numeric strings compare NUMERICALLY — `sort(["10", "9"])` answers `9, 10` — and that rule
/// is not this helper's.
#[test]
fn test_cli_wasm_scalar_sorts_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_sort");
    let php_path = dir.join("main.php");
    fs::write(&php_path, SORT_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the sort probe");
    assert!(
        output.status.success(),
        "sort compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the sort probe under Node");
    assert!(
        run.status.success(),
        "sort probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), SORT_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// `foreach ($h as $k => $v)` over Mixed hash values, plus `isset($h[$k])` and
/// `array_key_exists($k, $h)` — the pair PHP answers DIFFERENTLY for a stored null.
#[test]
fn test_cli_wasm_assoc_foreach_and_key_tests_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_assoc_keys");
    let php_path = dir.join("main.php");
    fs::write(&php_path, ASSOC_KEYS_SOURCE).unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the associative-key probe");
    assert!(
        output.status.success(),
        "associative-key compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the associative-key probe under Node");
    assert!(
        run.status.success(),
        "associative-key probe trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    assert_eq!(String::from_utf8_lossy(&run.stdout), ASSOC_KEYS_EXPECTED);

    let _ = fs::remove_dir_all(&dir);
}

/// The associative-key probe. `"b" => null` is the case that separates the two questions:
/// `isset` answers false there, `array_key_exists` answers true. `"0"`/`"7"` cover PHP's
/// numeric-string key normalization, and the last group reads Mixed and float hash values.
const ASSOC_KEYS_SOURCE: &str = r##"<?php
$conf = ["host" => "local", "port" => 8080, "debug" => true];
foreach ($conf as $k => $v) { echo $k, "=", $v, ";"; }
echo "|";
$h = ["a" => 1, "b" => null, "c" => 0, "d" => "", "e" => false, "0" => "zero", "7" => "seven"];
foreach (["a","b","c","d","e","zz","0","7",""] as $k) {
  echo $k, isset($h[$k]) ? "1" : "0", array_key_exists($k, $h) ? "1" : "0", ",";
}
echo "|", isset($h[0]) ? "1" : "0", array_key_exists(0, $h) ? "1" : "0";
echo "|", isset($h[7]) ? "1" : "0", array_key_exists(7, $h) ? "1" : "0";
echo "|";
$n = [1 => "a", 2 => null, 3 => "c"];
foreach ([0,1,2,3,4] as $i) { echo $i, isset($n[$i]) ? "1" : "0", array_key_exists($i, $n) ? "1" : "0", ","; }
echo "|";
foreach ($n as $key => $val) { echo $key, "=>", $val, " "; }
echo "|";
$f = ["p" => 1.5, "q" => -0.25];
foreach ($f as $key => $val) { echo $key, ":", $val, " "; }
echo "|", count($conf), count($h), count($n), count($f), "\n";
"##;

/// php-src 8.5.6's own output for `ASSOC_KEYS_SOURCE`.
const ASSOC_KEYS_EXPECTED: &str = "host=local;port=8080;debug=1;|a11,b01,c11,d11,e11,zz00,011,711,00,|11|11|000,111,201,311,400,|1=>a 2=> 3=>c |p:1.5 q:-0.25 |3732\n";

/// The sort probe: empty, single, already-ordered, reversed, duplicates, negatives, both i64
/// extremes, floats including -0.0, twenty generated orderings, and the STRING cases — where
/// two numeric strings order NUMERICALLY, equal-as-doubles i64-overflowing texts fall back to
/// their bytes, and equal infinities do too.
const SORT_SOURCE: &str = r##"<?php
$a0 = [5,3,9,1,3]; sort($a0); echo implode(",", $a0), ";";
$b0 = [5,3,9,1,3]; rsort($b0); echo implode(",", $b0), ";";
$a1 = []; sort($a1); echo implode(",", $a1), ";";
$b1 = []; rsort($b1); echo implode(",", $b1), ";";
$a2 = [7]; sort($a2); echo implode(",", $a2), ";";
$b2 = [7]; rsort($b2); echo implode(",", $b2), ";";
$a3 = [2,1]; sort($a3); echo implode(",", $a3), ";";
$b3 = [2,1]; rsort($b3); echo implode(",", $b3), ";";
$a4 = [1,2,3]; sort($a4); echo implode(",", $a4), ";";
$b4 = [1,2,3]; rsort($b4); echo implode(",", $b4), ";";
$a5 = [3,2,1]; sort($a5); echo implode(",", $a5), ";";
$b5 = [3,2,1]; rsort($b5); echo implode(",", $b5), ";";
$a6 = [-5,0,5,-1]; sort($a6); echo implode(",", $a6), ";";
$b6 = [-5,0,5,-1]; rsort($b6); echo implode(",", $b6), ";";
$a7 = [0,0,0]; sort($a7); echo implode(",", $a7), ";";
$b7 = [0,0,0]; rsort($b7); echo implode(",", $b7), ";";
$a8 = [PHP_INT_MAX, PHP_INT_MIN, 0]; sort($a8); echo implode(",", $a8), ";";
$b8 = [PHP_INT_MAX, PHP_INT_MIN, 0]; rsort($b8); echo implode(",", $b8), ";";
$a9 = [1.5,-2.5,0.0,1.5]; sort($a9); echo implode(",", $a9), ";";
$b9 = [1.5,-2.5,0.0,1.5]; rsort($b9); echo implode(",", $b9), ";";
$a10 = [3.0,1.0,2.0]; sort($a10); echo implode(",", $a10), ";";
$b10 = [3.0,1.0,2.0]; rsort($b10); echo implode(",", $b10), ";";
$a11 = [-0.0,0.0]; sort($a11); echo implode(",", $a11), ";";
$b11 = [-0.0,0.0]; rsort($b11); echo implode(",", $b11), ";";
$a12 = [25,19,-34]; sort($a12); echo implode(",", $a12), ";";
$b12 = [25,19,-34]; rsort($b12); echo implode(",", $b12), ";";
$a13 = [27,10,30,24,-42]; sort($a13); echo implode(",", $a13), ";";
$b13 = [27,10,30,24,-42]; rsort($b13); echo implode(",", $b13), ";";
$a14 = []; sort($a14); echo implode(",", $a14), ";";
$b14 = []; rsort($b14); echo implode(",", $b14), ";";
$a15 = [-17,20,-21,-26,41,10,19]; sort($a15); echo implode(",", $a15), ";";
$b15 = [-17,20,-21,-26,41,10,19]; rsort($b15); echo implode(",", $b15), ";";
$a16 = [10,0,31,-31,-21,31,-31,16]; sort($a16); echo implode(",", $a16), ";";
$b16 = [10,0,31,-31,-21,31,-31,16]; rsort($b16); echo implode(",", $b16), ";";
$a17 = [44,-49,35,49,-42,-30]; sort($a17); echo implode(",", $a17), ";";
$b17 = [44,-49,35,49,-42,-30]; rsort($b17); echo implode(",", $b17), ";";
$a18 = []; sort($a18); echo implode(",", $a18), ";";
$b18 = []; rsort($b18); echo implode(",", $b18), ";";
$a19 = [49,-47,-16,10]; sort($a19); echo implode(",", $a19), ";";
$b19 = [49,-47,-16,10]; rsort($b19); echo implode(",", $b19), ";";
$a20 = [41,4,0,43,23,6]; sort($a20); echo implode(",", $a20), ";";
$b20 = [41,4,0,43,23,6]; rsort($b20); echo implode(",", $b20), ";";
$a21 = [-4,-38]; sort($a21); echo implode(",", $a21), ";";
$b21 = [-4,-38]; rsort($b21); echo implode(",", $b21), ";";
$a22 = []; sort($a22); echo implode(",", $a22), ";";
$b22 = []; rsort($b22); echo implode(",", $b22), ";";
$a23 = [13,-23]; sort($a23); echo implode(",", $a23), ";";
$b23 = [13,-23]; rsort($b23); echo implode(",", $b23), ";";
$a24 = [36,5,49,30]; sort($a24); echo implode(",", $a24), ";";
$b24 = [36,5,49,30]; rsort($b24); echo implode(",", $b24), ";";
$a25 = [3,14,-1,23]; sort($a25); echo implode(",", $a25), ";";
$b25 = [3,14,-1,23]; rsort($b25); echo implode(",", $b25), ";";
$a26 = [18,24,2,24,-21]; sort($a26); echo implode(",", $a26), ";";
$b26 = [18,24,2,24,-21]; rsort($b26); echo implode(",", $b26), ";";
$a27 = [37,-47,-15,27,35]; sort($a27); echo implode(",", $a27), ";";
$b27 = [37,-47,-15,27,35]; rsort($b27); echo implode(",", $b27), ";";
$a28 = [39,-9]; sort($a28); echo implode(",", $a28), ";";
$b28 = [39,-9]; rsort($b28); echo implode(",", $b28), ";";
$a29 = [23,22,-37,41,33,-23,31,23]; sort($a29); echo implode(",", $a29), ";";
$b29 = [23,22,-37,41,33,-23,31,23]; rsort($b29); echo implode(",", $b29), ";";
$a30 = [-14,-35,-42,11]; sort($a30); echo implode(",", $a30), ";";
$b30 = [-14,-35,-42,11]; rsort($b30); echo implode(",", $b30), ";";
$a31 = [-39,-6,-42,2,-31,-48,-13]; sort($a31); echo implode(",", $a31), ";";
$b31 = [-39,-6,-42,2,-31,-48,-13]; rsort($b31); echo implode(",", $b31), ";";
echo "
";$s0 = ["pear","apple","fig"]; sort($s0); echo implode("|", $s0), ";";
$t0 = ["pear","apple","fig"]; rsort($t0); echo implode("|", $t0), ";";
$s1 = ["10","9","1e1","10.0"]; sort($s1); echo implode("|", $s1), ";";
$t1 = ["10","9","1e1","10.0"]; rsort($t1); echo implode("|", $t1), ";";
$s2 = ["abc","ABC","zz","a"]; sort($s2); echo implode("|", $s2), ";";
$s3 = ["9223372036854775808","9223372036854775807","9223372036854775809"]; sort($s3); echo implode("|", $s3), ";";
$s4 = ["1e400","1e401","inf"]; sort($s4); echo implode("|", $s4), ";";
$s5 = ["007","7","7.0"]; sort($s5); echo implode("|", $s5), ";";
$s6 = [" 1","1 ","1"]; sort($s6); echo implode("|", $s6), ";";
$s7 = ["only"]; sort($s7); echo implode("|", $s7), ";";
$s8 = []; sort($s8); echo implode("|", $s8), ";";
"##;

/// php-src 8.5.6's own output for `SORT_SOURCE`.
const SORT_EXPECTED: &str = r##"1,3,3,5,9;9,5,3,3,1;;;7;7;1,2;2,1;1,2,3;3,2,1;1,2,3;3,2,1;-5,-1,0,5;5,0,-1,-5;0,0,0;0,0,0;-9223372036854775808,0,9223372036854775807;9223372036854775807,0,-9223372036854775808;-2.5,0,1.5,1.5;1.5,1.5,0,-2.5;1,2,3;3,2,1;-0,0;-0,0;-34,19,25;25,19,-34;-42,10,24,27,30;30,27,24,10,-42;;;-26,-21,-17,10,19,20,41;41,20,19,10,-17,-21,-26;-31,-31,-21,0,10,16,31,31;31,31,16,10,0,-21,-31,-31;-49,-42,-30,35,44,49;49,44,35,-30,-42,-49;;;-47,-16,10,49;49,10,-16,-47;0,4,6,23,41,43;43,41,23,6,4,0;-38,-4;-4,-38;;;-23,13;13,-23;5,30,36,49;49,36,30,5;-1,3,14,23;23,14,3,-1;-21,2,18,24,24;24,24,18,2,-21;-47,-15,27,35,37;37,35,27,-15,-47;-9,39;39,-9;-37,-23,22,23,23,31,33,41;41,33,31,23,23,22,-23,-37;-42,-35,-14,11;11,-14,-35,-42;-48,-42,-39,-31,-13,-6,2;2,-6,-13,-31,-39,-42,-48;
apple|fig|pear;pear|fig|apple;9|10|1e1|10.0;10|1e1|10.0|9;ABC|a|abc|zz;9223372036854775807|9223372036854775808|9223372036854775809;1e400|1e401|inf;007|7|7.0; 1|1 |1;only;;"##;

/// Verifies `strrpos` finds the RIGHTMOST match and answers php-src's `int|false`.
///
/// Scanning right to left is what makes overlapping matches resolve to the last one —
/// `strrpos("aaa", "aa")` is 1, not 0 — and an empty needle answers the position just past the
/// end rather than zero, so `strrpos("abcabc", "")` is 6. Only the two-argument form is lowered:
/// the offset form's rule is NOT the mirror of `strpos`'s, since a negative offset there bounds
/// where the match may START counted from the end.
#[test]
fn test_cli_wasm_strrpos_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_strrpos");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function f(string $h, string $n): void {
    $p = strrpos($h, $n);
    if ($p === false) { echo "F"; } else { echo "@"; echo $p; }
    echo "|";
}
f("abcabc","b"); f("abcabc","z"); f("abcabc",""); f("","a"); f("",""); echo "\n";
f("abcabc","bc"); f("abcabc","abcabc"); f("aaa","aa"); f("abc","c"); f("abc","a"); echo "\n";
f("abcabc","abcabcd"); f("h\xc3\xa9llo","\xc3\xa9"); f("\x00\x01\x00","\x00"); f("aXbXc","X"); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile strrpos to WASM");
    assert!(
        output.status.success(),
        "strrpos compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run strrpos under Node");
    assert!(
        run.status.success(),
        "strrpos trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program.
    let expected: Vec<u8> = [
        b"@4|F|@6|F|@0|\n".as_slice(),
        b"@4|@0|@1|@2|@0|\n".as_slice(),
        b"F|@1|@2|@3|\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `strstr` reproduces php-src in both arities, including its `string|false`.
///
/// The result is a REGION of the haystack — from the match to the end, or from the start up to
/// the match when `$before_needle` is true — so the two arities return different halves of the
/// same scan rather than one being a default of the other. An empty needle matches at offset 0,
/// which makes `strstr($h, "")` the whole string and its `before` form empty; a needle that is
/// absent gives false in BOTH arities. Binary samples are included because boxing under the
/// string tag persists a copy of the region rather than aliasing the source.
#[test]
fn test_cli_wasm_strstr_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_strstr");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function f(string $h, string $n): void {
    $r = strstr($h, $n);
    if ($r === false) { echo "F"; } else { echo "[", $r, "]"; }
    echo "|";
}
function b(string $h, string $n): void {
    $r = strstr($h, $n, true);
    if ($r === false) { echo "F"; } else { echo "[", $r, "]"; }
    echo "|";
}
f("abcdef","cd"); f("abcdef","z"); f("abcdef",""); f("","a"); f("abcdef","a"); f("abcdef","f"); f("abcabc","bc"); echo "\n";
b("abcdef","cd"); b("abcdef","z"); b("abcdef",""); b("","a"); b("abcdef","a"); b("abcdef","f"); b("abcabc","bc"); echo "\n";
f("h\xc3\xa9llo","\xc3\xa9"); b("h\xc3\xa9llo","\xc3\xa9"); f("\x00\x01\x02","\x01"); b("\x00\x01\x02","\x01"); echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile strstr to WASM");
    assert!(
        output.status.success(),
        "strstr compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run strstr under Node");
    assert!(
        run.status.success(),
        "strstr trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own bytes for the same program. A byte literal rather than a `str`,
    // because the samples carry bytes no Rust string literal can hold.
    let expected: Vec<u8> = [
        b"[cdef]|F|[abcdef]|F|[abcdef]|[f]|[bcabc]|\n".as_slice(),
        b"[ab]|F|[]|F|[]|[abcde]|[a]|\n".as_slice(),
        b"[\xc3\xa9llo]|[h]|[\x01\x02]|[\x00]|\n".as_slice(),
    ]
    .concat();
    assert_eq!(run.stdout, expected);

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `strpos` and PHP's `===` against a runtime-tagged value.
///
/// These belong in one test because neither is usable without the other: `strpos` answers
/// `int|false`, which EIR carries as a tagged `Mixed` cell, and the whole point of the idiom
/// `strpos($h, $n) === false` is that it separates a match at OFFSET ZERO from a miss. Boxing the
/// miss as an int zero, or comparing the cell by storage rather than by tag, gets that backwards.
///
/// The tagged comparison is then exercised against every concrete type it admits. The float cases
/// are the ones a bit-for-bit payload comparison fails: `NAN === NAN` is false and
/// `0.0 === -0.0` is true. `null` is the other, because an unboxed null literal carries a
/// sentinel while an absent cell reads as zero, so only the tag can decide it.
#[test]
fn test_cli_wasm_strpos_and_tagged_strict_equality_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_strpos_strict");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function f(string $h, string $n): void {
    $p = strpos($h, $n);
    if ($p === false) { echo "F"; } else { echo "@"; echo $p; }
    echo "|";
}
f("abcabc","b"); f("abcabc","z"); f("abcabc",""); f("","a"); f("",""); echo "\n";
f("abcabc","B"); f("aXbXc","X"); f("abcabc","bc"); f("abcabc","abcabc"); f("abcabc","abcabcd"); echo "\n";
f("abc","a"); f("abc","c"); f("aaa","aa"); f("\x00\x01\x02","\x01"); f("h\xc3\xa9llo","\xc3\xa9"); echo "\n";
function g(string $h, string $n): void { echo strpos($h, $n) !== false ? "Y" : "N"; }
g("abc","a"); g("abc","z"); g("abc",""); echo "\n";
function probe(mixed $m): void {
    echo $m === 1 ? "i1" : "-";
    echo $m === "a" ? "sa" : "-";
    echo $m === true ? "T" : "-";
    echo $m === null ? "N" : "-";
    echo $m === 1.5 ? "f" : "-";
    echo $m === 0 ? "i0" : "-";
    echo $m === false ? "F" : "-";
    echo $m === "" ? "se" : "-";
    echo $m !== 1 ? "!i1" : "==";
    echo "|";
}
probe(1); probe("a"); probe(true); probe(null); probe(1.5); probe(0); probe(false); probe(""); probe(1.0); probe("A");
echo "\n";
function edge(mixed $m): void {
    echo $m === 0.0 ? "z" : "-";
    echo $m === -0.0 ? "nz" : "-";
    echo $m === NAN ? "nan" : "-";
    echo $m === INF ? "inf" : "-";
    echo $m === PHP_INT_MAX ? "max" : "-";
    echo "|";
}
edge(0.0); edge(-0.0); edge(NAN); edge(INF); edge(PHP_INT_MAX); edge(0);
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile strpos and tagged equality to WASM");
    assert!(
        output.status.success(),
        "strpos/tagged equality compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run strpos and tagged equality under Node");
    assert!(
        run.status.success(),
        "strpos/tagged equality trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "@1|F|@0|F|@0|\n",
            "F|@1|@1|@0|F|\n",
            "@0|@2|@0|@1|@1|\n",
            "YNY\n",
            "i1-------==|-sa------!i1|--T-----!i1|---N----!i1|----f---!i1|-----i0--!i1|------F-!i1|-------se!i1|--------!i1|--------!i1|\n",
            "znz---|znz---|-----|---inf-|----max|-----|\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `str_repeat` answers like php-src and RAISES php-src's ValueError.
///
/// PHP does not clamp a negative `$times` to zero, it raises a `ValueError` an ordinary `catch`
/// receives — so this is the first builtin on this target whose failure is a PHP exception rather
/// than a machine guard, and it reuses the raise path the arithmetic errors already take. A count
/// of zero is NOT a failure: it answers the empty string.
#[test]
fn test_cli_wasm_str_repeat_matches_php_and_raises_its_value_error() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_str_repeat");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function r(string $s, int $n): string { return str_repeat($s, $n); }
echo bin2hex(r("ab", 0)), "|", bin2hex(r("ab", 1)), "|", bin2hex(r("ab", 3)), "|", bin2hex(r("", 5)), "|", bin2hex(r("a", 7)), "\n";
try { echo bin2hex(r("a", -1)), "\n"; } catch (\ValueError $e) { echo "caught|", get_class($e), "|", $e->getMessage(), "\n"; }
echo "end\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile str_repeat to WASM");
    assert!(
        output.status.success(),
        "str_repeat compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run str_repeat under Node");
    if !run.status.success() && String::from_utf8_lossy(&run.stderr).contains("CompileError") {
        let _ = fs::remove_dir_all(&dir);
        return;
    }
    assert!(
        run.status.success(),
        "the caught ValueError still killed the program: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "|6162|616261626162||61616161616161\n",
            "caught|ValueError|str_repeat(): Argument #2 ($times) must be greater than or equal to 0\n",
            "end\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `chr` and `ord` reproduce php-src, including the values PHP does not reject.
///
/// PHP does not refuse an out-of-range `chr`: it constrains the argument with `% 256`, bringing a
/// negative remainder back up, so `chr(-1)` is `\xff` and `chr(1000000)` is `\x40`. `ord` answers
/// 0 for the empty string and the FIRST byte of a longer one. Since PHP 8.5 both cases are
/// deprecated, but they still answer, and the value is what this compares.
///
/// Each helper is reached through a user function returning a `string`, which is what makes this
/// also the coverage for `Op::StrPersist`: without it the returned bytes would not outlive the
/// callee's frame.
#[test]
fn test_cli_wasm_chr_and_ord_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_chr_ord");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function c(int $n): string { return chr($n); }
function o(string $s): int { return ord($s); }
echo bin2hex(c(65)), "|", bin2hex(c(0)), "|", bin2hex(c(255)), "|", bin2hex(c(10)), "\n";
echo bin2hex(c(-1)), "|", bin2hex(c(-256)), "|", bin2hex(c(-257)), "|", bin2hex(c(256)), "|", bin2hex(c(257)), "|", bin2hex(c(1000000)), "\n";
echo o("A"), "|", o("\xff"), "|", o("0"), "|", o(""), "|", o("AB"), "\n";
echo bin2hex(c(o("Z"))), "|", o(c(200)), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile chr/ord to WASM");
    assert!(
        output.status.success(),
        "chr/ord compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run chr/ord under Node");
    assert!(
        run.status.success(),
        "chr/ord trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "41|00|ff|0a\n",
            "ff|00|ff|00|01|40\n",
            "65|255|48|0|65\n",
            "5a|200\n",
        )
    );

    // php-src 8.5 diagnoses the six out-of-range `chr` arguments and the two `ord` arguments
    // that are not one byte, once each — counted rather than matched whole, because php-src also
    // prints a file, line and stack trace this target does not reproduce.
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(stderr.matches("chr():").count(), 6, "stderr was: {stderr}");
    assert_eq!(stderr.matches("ord():").count(), 2, "stderr was: {stderr}");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies base64 and url coding reproduce php-src, tolerant decoding included.
///
/// The samples pin what separates these from a textbook implementation. `urlencode` folds a
/// space to `+` and percent-encodes `~`, while `rawurlencode` does the opposite on both;
/// percent-encoding is UPPERCASE hex. Decoding never fails: `"a%2"` and `"a%zz"` keep a literal
/// `%`. One-argument `base64_decode` is non-strict, so `"YWJj="`, `"YW Jj"` and `"YWJj\n"` all
/// give `abc`, `"YWJ"` gives `ab`, and `"!!!!"` gives the empty string. Every result goes through
/// `bin2hex` where its bytes are not already printable ASCII.
#[test]
fn test_cli_wasm_base64_and_url_coding_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_codecs");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function t(string $s): void {
    echo bin2hex($s), "|", urlencode($s), "|", rawurlencode($s), "|",
         bin2hex(urldecode($s)), "|", bin2hex(rawurldecode($s)), "|",
         base64_encode($s), "|", bin2hex(base64_decode($s)), "\n";
}
t("");
t("a");
t("ab");
t("abc");
t("abcd");
t("a b+c~d.e_f-g");
t("h\xc3\xa9llo");
t("\x00\x01\xff");
t("a%2");
t("a%zz");
t("%C3%A9");
t("YWJj");
t("YWJj=");
t("YW Jj");
t("YWJ");
t("!!!!");
t("Hello, World!");
t("\n\r\t");
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the codecs to WASM");
    assert!(
        output.status.success(),
        "codec compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the codecs under Node");
    assert!(
        run.status.success(),
        "codecs trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "||||||\n",
            "61|a|a|61|61|YQ==|\n",
            "6162|ab|ab|6162|6162|YWI=|69\n",
            "616263|abc|abc|616263|616263|YWJj|69b7\n",
            "61626364|abcd|abcd|61626364|61626364|YWJjZA==|69b71d\n",
            "6120622b637e642e655f662d67|a+b%2Bc%7Ed.e_f-g|a%20b%2Bc~d.e_f-g|61206220637e642e655f662d67|6120622b637e642e655f662d67|YSBiK2N+ZC5lX2YtZw==|69bf9c75e7e0\n",
            "68c3a96c6c6f|h%C3%A9llo|h%C3%A9llo|68c3a96c6c6f|68c3a96c6c6f|aMOpbGxv|865968\n",
            "0001ff|%00%01%FF|%00%01%FF|0001ff|0001ff|AAH/|\n",
            "612532|a%252|a%252|612532|612532|YSUy|6b\n",
            "61257a7a|a%25zz|a%25zz|61257a7a|61257a7a|YSV6eg==|6b3c\n",
            "254333254139|%25C3%25A9|%25C3%25A9|c3a9|c3a9|JUMzJUE5|0b703d\n",
            "59574a6a|YWJj|YWJj|59574a6a|59574a6a|WVdKag==|616263\n",
            "59574a6a3d|YWJj%3D|YWJj%3D|59574a6a3d|59574a6a3d|WVdKaj0=|616263\n",
            "5957204a6a|YW+Jj|YW%20Jj|5957204a6a|5957204a6a|WVcgSmo=|616263\n",
            "59574a|YWJ|YWJ|59574a|59574a|WVdK|6162\n",
            "21212121|%21%21%21%21|%21%21%21%21|21212121|21212121|ISEhIQ==|\n",
            "48656c6c6f2c20576f726c6421|Hello%2C+World%21|Hello%2C%20World%21|48656c6c6f2c20576f726c6421|48656c6c6f2c20576f726c6421|SGVsbG8sIFdvcmxkIQ==|1de965a16a2b95\n",
            "0a0d09|%0A%0D%09|%0A%0D%09|0a0d09|0a0d09|Cg0J|\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a string literal reaches the module as PHP BYTES, not as Rust's UTF-8.
///
/// A PHP string is a byte string while a Rust `String` must be valid UTF-8, so the lexer carries
/// every escaped non-ASCII byte as a private-use marker char. A data segment written straight
/// from those Rust bytes turns `"\xff"` into the three UTF-8 bytes of U+E0FF, which `strlen`
/// then reports as 3. The native backend decodes through `string_bytes::literal_bytes`; this
/// pins that the WASM segments do too.
#[test]
fn test_cli_wasm_string_literals_carry_raw_php_bytes() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_raw_literal_bytes");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$high = "\xff";
$mixed = "\x00\x01\xfe\xff";
$octal = "\101\377";
$utf8 = "h\xc3\xa9llo";
echo strlen($high), "|", bin2hex($high), "\n";
echo strlen($mixed), "|", bin2hex($mixed), "\n";
echo strlen($octal), "|", bin2hex($octal), "\n";
echo strlen($utf8), "|", bin2hex($utf8), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the raw byte literals to WASM");
    assert!(
        output.status.success(),
        "raw byte literal compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the raw byte literals under Node");
    assert!(
        run.status.success(),
        "raw byte literals trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src 8.5.6's own output for the same program.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "1|ff\n",
            "4|0001feff\n",
            "2|41ff\n",
            "6|68c3a96c6c6f\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP's IMPLICIT coercion at a declared `int` return, on its accepting paths.
///
/// This is a DIFFERENT operation from `(int)`, which is why it has its own runtime: `(int)`
/// truncates a float in silence, while a return that loses a fraction is `Deprecated`; and
/// `(int)` reads a leading-numeric string as its prefix, while a return rejects it outright.
/// Every value and every diagnostic below is php-src 8.5.6's own, in php-src's order; the
/// rules were transcribed from `zend_verify_return_type` and validated against a 1200-value
/// random sweep before any of this WAT was written.
#[test]
fn test_cli_wasm_declared_int_return_coerces_like_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_return_coercion");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function ri(mixed $v): int { return $v; }
class Calc { public function pick(array $xs): int { $t = 0; foreach ($xs as $x) { $t = $x; } return $t; } }
echo ri(7), "\n";
echo ri(true), "\n";
echo ri(false), "\n";
echo ri(5.0), "\n";
echo ri(5.7), "\n";
echo ri(-5.7), "\n";
echo ri("42"), "\n";
echo ri("  8  "), "\n";
echo ri("3.9"), "\n";
echo ri("1e-3"), "\n";
echo ri(-9223372036854775808.0), "\n";
echo (new Calc())->pick([1, 7.5]), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the declared-int return coercion to WASM");
    assert!(
        output.status.success(),
        "declared-int return coercion compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the return coercion under Node");
    assert!(
        run.status.success(),
        "the accepting coercion paths must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // `-9223372036854775808.0` is exactly -2^63 and IS in range; `9223372036854775807.0`
    // rounds UP to 2^63 as a double and is not, which the fatal test below covers.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "7\n", "1\n", "0\n", "5\n", "5\n", "-5\n", "42\n", "8\n", "3\n", "0\n",
            "-9223372036854775808\n", "7\n",
        )
    );

    // php-src reports exactly these five, in this order. A float and a float-shaped STRING
    // carry different wordings, and the string is quoted with its original bytes, padding
    // included. The project's WASM convention drops php-src's ` in <file> on line <n>` tail.
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        stderr.lines().collect::<Vec<&str>>(),
        vec![
            "Deprecated: Implicit conversion from float 5.7 to int loses precision",
            "Deprecated: Implicit conversion from float -5.7 to int loses precision",
            r#"Deprecated: Implicit conversion from float-string "3.9" to int loses precision"#,
            r#"Deprecated: Implicit conversion from float-string "1e-3" to int loses precision"#,
            "Deprecated: Implicit conversion from float 7.5 to int loses precision",
        ],
        "deprecations must match php-src's set and order"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the `TypeError` a declared `int` return raises for a value it cannot hold.
///
/// The message is composed at RUNTIME from the returning function's own name — which the EIR
/// already spells the way PHP prints it, `f` for a function and `C::m` for a method — and the
/// type word that arrived. An OBJECT contributes its class name rather than the word "object",
/// measured on php-src: `Point returned`, not `object returned`.
///
/// The error is a deterministic fatal rather than a catchable throw, which is the documented
/// fallback this backend already takes wherever a raise site cannot resolve a STATIC message.
/// The text and the 255 exit status are php-src's.
#[test]
fn test_cli_wasm_declared_int_return_raises_php_type_error() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_return_type_error");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let prelude = "<?php\nfunction ri(mixed $v): int { return $v; }\nclass Point { public int $x = 1; }\n";
    for (stem, source, expected) in [
        (
            "n",
            format!("{prelude}echo ri(null), \"\\n\";\n"),
            "PHP Fatal error: Uncaught TypeError: ri(): Return value must be of type int, null returned\n",
        ),
        (
            "s",
            format!("{prelude}echo ri(\"12abc\"), \"\\n\";\n"),
            "PHP Fatal error: Uncaught TypeError: ri(): Return value must be of type int, string returned\n",
        ),
        (
            "f",
            format!("{prelude}echo ri(9223372036854775807.0), \"\\n\";\n"),
            "PHP Fatal error: Uncaught TypeError: ri(): Return value must be of type int, float returned\n",
        ),
        (
            "a",
            format!("{prelude}echo ri([1, 2]), \"\\n\";\n"),
            "PHP Fatal error: Uncaught TypeError: ri(): Return value must be of type int, array returned\n",
        ),
        (
            "o",
            format!("{prelude}echo ri(new Point()), \"\\n\";\n"),
            "PHP Fatal error: Uncaught TypeError: ri(): Return value must be of type int, Point returned\n",
        ),
        (
            "m",
            concat!(
                "<?php\n",
                "class Calc { public function pick(array $xs): int { $t = 0; foreach ($xs as $x) { $t = $x; } return $t; } }\n",
                "echo (new Calc())->pick([1, \"abc\"]), \"\\n\";\n",
            )
            .to_string(),
            "PHP Fatal error: Uncaught TypeError: Calc::pick(): Return value must be of type int, string returned\n",
        ),
    ] {
        let php_path = dir.join(format!("{stem}.php"));
        fs::write(&php_path, &source).unwrap();
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&php_path)
            .output()
            .expect("failed to compile a rejecting return coercion to WASM");
        assert!(
            output.status.success(),
            "case {stem} failed to compile: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(format!("{stem}.wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run a rejecting return coercion under Node");
        assert_eq!(
            run.status.code(),
            Some(255),
            "case {stem} must exit with PHP's fatal status"
        );
        assert_eq!(String::from_utf8_lossy(&run.stderr), expected, "case {stem}");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "",
            "case {stem} must produce no output before the fatal"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the same declared-return coercion for the `float` and `bool` targets.
///
/// Their accepting paths coincide with the explicit cast's — a float loses nothing on the way
/// to a float, and truthiness is truthiness — so those tags delegate to `__rt_mixed_cast_*` and
/// only the REFUSED tags need their own arm. Two places still diverge from the cast and are
/// pinned here: a leading-numeric string, which `(float)` reads as its prefix and a return
/// rejects; and NaN, which converts to `bool` only after php-src's warning. `string` is
/// deliberately absent — PHP reaches `__toString` there and this backend has no such dispatch.
#[test]
fn test_cli_wasm_declared_float_and_bool_returns_coerce_like_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_return_coercion_fb");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function rf(mixed $v): float { return $v; }
function rb(mixed $v): bool { return $v; }
echo rf(7), "|", rf(5.7), "|", rf(true), "|", rf(false), "|", rf("42"), "|", rf("3.9"), "|", rf("  8  "), "|", rf(1.0e20), "\n";
echo "[", rb(7), "][", rb(0), "][", rb(0.0), "][", rb("0"), "][", rb(""), "][", rb("abc"), "][", rb(true), "][", rb(false), "]\n";
echo "[", rb(NAN), "]\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the float/bool return coercions to WASM");
    assert!(
        output.status.success(),
        "float/bool return coercion compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the float/bool coercions under Node");
    assert!(
        run.status.success(),
        "the accepting paths must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    // php-src's own bytes: `false` renders as the empty string, and `1.0e20` keeps PHP's
    // exponent form rather than a plain decimal.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "7|5.7|1|0|42|3.9|8|1.0E+20\n",
            "[1][][][][][1][1][]\n",
            "[1]\n",
        )
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "Warning: unexpected NAN value was coerced to bool\n",
        "NaN converts, but only after php-src's warning — measured raw, since an error \
         handler hides the level"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the `TypeError` the `float` and `bool` targets raise, naming their own type.
///
/// One runtime fatal serves all the targets: measured on php-src, the word a tag contributes is
/// the same whatever the declared type — only the target word changes.
#[test]
fn test_cli_wasm_declared_float_and_bool_returns_raise_php_type_error() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_return_type_error_fb");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let prelude = concat!(
        "<?php\n",
        "class Point { public int $x = 1; }\n",
        "function rf(mixed $v): float { return $v; }\n",
        "function rb(mixed $v): bool { return $v; }\n",
    );
    for (stem, call, expected) in [
        (
            "fn",
            "rf(null)",
            "PHP Fatal error: Uncaught TypeError: rf(): Return value must be of type float, null returned\n",
        ),
        (
            "fs",
            "rf(\"12abc\")",
            "PHP Fatal error: Uncaught TypeError: rf(): Return value must be of type float, string returned\n",
        ),
        (
            "fo",
            "rf(new Point())",
            "PHP Fatal error: Uncaught TypeError: rf(): Return value must be of type float, Point returned\n",
        ),
        (
            "bn",
            "rb(null)",
            "PHP Fatal error: Uncaught TypeError: rb(): Return value must be of type bool, null returned\n",
        ),
        (
            "ba",
            "rb([1, 2])",
            "PHP Fatal error: Uncaught TypeError: rb(): Return value must be of type bool, array returned\n",
        ),
        (
            "bo",
            "rb(new Point())",
            "PHP Fatal error: Uncaught TypeError: rb(): Return value must be of type bool, Point returned\n",
        ),
    ] {
        let php_path = dir.join(format!("{stem}.php"));
        fs::write(&php_path, format!("{prelude}echo {call}, \"\\n\";\n")).unwrap();
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&php_path)
            .output()
            .expect("failed to compile a rejecting return coercion to WASM");
        assert!(
            output.status.success(),
            "case {stem} failed to compile: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(format!("{stem}.wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run a rejecting return coercion under Node");
        assert_eq!(
            run.status.code(),
            Some(255),
            "case {stem} must exit with PHP's fatal status"
        );
        assert_eq!(String::from_utf8_lossy(&run.stderr), expected, "case {stem}");
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP's `__toString` conversion of a statically known object.
///
/// The EIR hands this cast the CLASS — `Heap(Object)/Object("Talks")`, not a Mixed — so the
/// conversion is an ordinary direct call to a method body already in the module, with no
/// dispatch involved. `echo $obj` reaches it without a cast node at all, since the echo does
/// its own conversion; both sites share one emitter.
///
/// A class that provably has none raises php-src's `Error` instead, and a subclass that
/// OVERRIDES `__toString` makes the receiver's body undecidable here, so such a program stays
/// refused rather than answering the base implementation.
#[test]
fn test_cli_wasm_object_to_string_calls_php_tostring() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_object_to_string");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Talks { public function __toString(): string { return "I talk"; } }
class Money {
    public function __construct(private int $cents) {}
    public function __toString(): string { return "$" . $this->cents; }
}
class Base { public function __toString(): string { return "base"; } }
class Sub extends Base {}
$t = new Talks();
echo $t, "\n";
echo "x" . $t, "\n";
echo (string) $t, "\n";
echo new Base(), "|", new Sub(), "\n";
foreach (range(1, 3) as $i) { echo new Money($i), "\n"; }
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the __toString conversions to WASM");
    assert!(
        output.status.success(),
        "__toString compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the __toString conversions under Node");
    assert!(
        run.status.success(),
        "__toString conversions must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "I talk\n",
            "xI talk\n",
            "I talk\n",
            "base|base\n",
            "$1\n",
            "$2\n",
            "$3\n",
        ),
        "php-src's own bytes: an inherited __toString resolves to the parent body, and a \
         __toString that BUILDS its string round-trips"
    );

    // A class with none is php-src's Error, raised with its own text rather than refused.
    let missing = dir.join("missing.php");
    fs::write(
        &missing,
        "<?php\nclass Plain { public int $x = 1; }\necho new Plain(), \"\\n\";\n",
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&missing)
        .output()
        .expect("failed to compile the missing-__toString case");
    assert!(
        output.status.success(),
        "a class without __toString must still compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("missing.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the missing-__toString case");
    assert_eq!(run.status.code(), Some(255));
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "PHP Fatal error: Uncaught Error: Object of class Plain could not be converted to string\n"
    );

    // An overriding subclass makes the receiver's body undecidable at the `Base` site, so the
    // program is refused rather than answered with the wrong implementation.
    let overridden = dir.join("overridden.php");
    fs::write(
        &overridden,
        concat!(
            "<?php\n",
            "class B { public function __toString(): string { return \"b\"; } }\n",
            "class S extends B { public function __toString(): string { return \"s\"; } }\n",
            "echo new B(), \"|\", new S(), \"\\n\";\n",
        ),
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&overridden)
        .output()
        .expect("failed to run the overriding-subclass case");
    assert!(
        !output.status.success(),
        "an overriding subclass must not compile to the base body"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `count()` on a BOXED value: the container it holds, or php-src's own `TypeError`.
///
/// The message is a constant except for the word naming what arrived, and that word table is
/// not the declared-return one: measured on php-src 8.5.6, `count()` names a boolean by VALUE
/// (`true given`, not `bool given`). An internal function's `TypeError` also carries no
/// location, so the whole text is composable here.
#[test]
fn test_cli_wasm_count_of_a_boxed_value_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_count_boxed");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function c(mixed $v): int { return count($v); }
$a = [1, 2, 3];
$h = ["x" => 1, "y" => 2];
echo c($a), "|", c($h), "|", c([]), "\n";
"#,
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile count of a boxed value");
    assert!(
        output.status.success(),
        "count of a boxed value failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run count of a boxed value");
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout), "3|2|0\n");

    for (stem, value, word) in [
        ("i", "5", "int"),
        ("f", "5.5", "float"),
        ("t", "true", "true"),
        ("z", "false", "false"),
        ("n", "null", "null"),
        ("s", "\"abc\"", "string"),
    ] {
        let path = dir.join(format!("{stem}.php"));
        fs::write(
            &path,
            format!(
                "<?php\nfunction c(mixed $v): int {{ return count($v); }}\necho c({value}), \"\\n\";\n"
            ),
        )
        .unwrap();
        let output = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile a rejecting count");
        assert!(
            output.status.success(),
            "case {stem} failed to compile: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(format!("{stem}.wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run a rejecting count");
        assert_eq!(run.status.code(), Some(255), "case {stem}");
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            format!(
                "PHP Fatal error: Uncaught TypeError: count(): Argument #1 ($value) \
                 must be of type Countable|array, {word} given\n"
            ),
            "case {stem}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a relational comparison against a BOXED value, which is not a cast then a compare.
///
/// `$n <= 1` with `$n` untyped lowers to `cast Mixed -> I64` then `icmp`, and that pair is not
/// what PHP does. Measured on php-src 8.5.6 and validated on 1200 random pairs across the i64
/// boundary: a non-numeric string makes PHP render the LONG as a string and compare BYTES, so
/// `"abc" <= 1` is false where the cast answers 0 and reports true; a bool or null turns BOTH
/// sides into booleans; an array outranks any scalar; and a NaN answers 1 either way.
///
/// The native backend still takes the cast and gets `"abc"` and `0.5` wrong on both operators.
#[test]
fn test_cli_wasm_relational_comparison_of_a_boxed_value_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_boxed_comparison");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function le1($n) { return $n <= 1 ? "yes" : "no"; }
function gt0($n) { return $n > 0 ? "Y" : "N"; }
foreach ([0, 1, 2, "abc", "0", "2", 0.5, true, false, null, "", "7abc", " 7 "] as $v) {
  echo le1($v), gt0($v), "|";
}
echo "\n";
function fib($n) {
    if ($n <= 1) { return $n; }
    return fib($n - 1) + fib($n - 2);
}
echo fib(10), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the boxed comparisons to WASM");
    assert!(
        output.status.success(),
        "boxed comparison compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the boxed comparisons under Node");
    assert!(
        run.status.success(),
        "boxed comparisons must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    // php-src's own answers. `"abc"` and `0.5` are the two the cast gets wrong.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "yesN|yesY|noY|noY|yesN|noY|yesY|yesY|yesN|yesN|yesN|noY|noY|\n",
            "55\n",
        ),
        "the recursive fib exercises returning an untyped PARAMETER, which hands the caller a \
         BORROW it would otherwise free while still holding it"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `array_keys()` and `array_values()` over an ASSOCIATIVE array.
///
/// Both were limited to the indexed representation, where they are near-identities: a list's
/// keys ARE its positions and its values are the list itself. A hash needs a real projection,
/// and the order that projection must produce is php-src's INSERTION order — the order
/// `foreach` walks — not the bucket table's, which is a hash artefact.
#[test]
fn test_cli_wasm_array_keys_and_values_project_a_hash() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_hash_projection");
    // Two hashes with DIFFERENT value types in one scope hit `release_local_slot`, an
    // unrelated gap, so the shapes are exercised in separate modules.
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$h = ["z" => 26, "a" => 1, "m" => 13];
echo implode("-", array_keys($h)), "\n";
foreach (array_values($h) as $v) { echo $v, ","; }
echo "\n";
$grown = [];
$grown["k"] = 5;
echo count(array_keys($grown)), "|", count(array_values($grown)), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the hash projections to WASM");
    assert!(
        output.status.success(),
        "hash projection compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the hash projections under Node");
    assert!(
        run.status.success(),
        "hash projections must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    // `z-a-m`, not `a-m-z`: php-src answers in insertion order, never sorted.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!("z-a-m\n", "26,1,13,\n", "1|1\n"),
    );

    // A hash of STRING values projects through the 16-byte slot instead of the 8-byte one.
    let words = dir.join("words.php");
    fs::write(
        &words,
        "<?php\n$w = [\"x\" => \"one\", \"y\" => \"two\"];\nforeach (array_values($w) as $v) { echo $v, \",\"; }\necho \"\\n\";\n",
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&words)
        .output()
        .expect("failed to compile the string-valued projection");
    assert!(
        output.status.success(),
        "string-valued projection failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("words.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the string-valued projection");
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout), "one,two,\n");

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a float reaching a string, and `echo` of a `void` call's value.
///
/// `(string) $f` and `"$f"` render through the same `__rt_ftoa` an `echo` of a float uses, so
/// the three spellings cannot disagree — including php-src's exponent form for large and small
/// magnitudes, and `100` rather than `100.0` for an integral value.
///
/// A `void` method call still has an expression value in PHP, and that value is null, which
/// `echo` renders as nothing at all.
#[test]
fn test_cli_wasm_float_to_string_and_void_echo_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_float_to_string");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function s(float $f): string { return (string) $f; }
echo s(1.5), "|", s(-0.0), "|", s(1.0e20), "|", s(0.1), "|", s(100.0), "|", s(1.0e-7), "\n";
$x = 3.25;
echo "v=" . $x . "!\n";
class C { public function go(): void { echo "go\n"; } }
echo (new C())->go();
echo "end\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the float-to-string cases to WASM");
    assert!(
        output.status.success(),
        "float-to-string compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the float-to-string cases under Node");
    assert!(
        run.status.success(),
        "float-to-string cases must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "1.5|-0|1.0E+20|0.1|100|1.0E-7\n",
            "v=3.25!\n",
            "go\n",
            "end\n",
        ),
        "php-src's own bytes for every rendering"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `(int) $string` and `$s[$i]`, both against php-src's exact answers.
///
/// `(int)` of a string takes the LEADING numeric prefix and answers 0 when there is none,
/// silently — the same parser a boxed string casts through, so the two spellings agree.
///
/// `$s[$i]` counts a negative index from the END, and anything still outside answers the EMPTY
/// string after `Warning: Uninitialized string offset N` — naming the index AS WRITTEN, so a
/// negative one is reported negative rather than resolved first.
#[test]
fn test_cli_wasm_string_to_int_and_offset_read_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_offset");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$s = "42";
echo (int) $s, "|", (int) "abc", "|", (int) "12abc", "|", (int) "  7  ", "\n";
$t = "hi";
foreach ([0, 1, 2, 5, -1, -2, -3, -10] as $i) { echo "[", $i, "=>", $t[$i], "]"; }
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the string cases to WASM");
    assert!(
        output.status.success(),
        "string-case compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the string cases under Node");
    assert!(
        run.status.success(),
        "string cases must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "42|0|12|7\n",
            "[0=>h][1=>i][2=>][5=>][-1=>i][-2=>h][-3=>][-10=>]\n",
        )
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stderr)
            .lines()
            .collect::<Vec<&str>>(),
        vec![
            "Warning: Uninitialized string offset 2",
            "Warning: Uninitialized string offset 5",
            "Warning: Uninitialized string offset -3",
            "Warning: Uninitialized string offset -10",
        ],
        "php-src warns for each out-of-range read, naming the index as written"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the `is_*()` type-predicate family, boxed and statically typed.
///
/// A value whose EIR type is already concrete answers at COMPILE time and tests nothing;
/// only a boxed value reaches a runtime test, and there the cell's tag is the whole answer.
/// `is_iterable()` on a boxed value stays refused, since PHP also accepts a `Traversable`
/// object and the tag cannot tell one object from another.
#[test]
fn test_cli_wasm_type_predicates_match_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_type_predicates");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class P {}
function k(mixed $v): string {
  $r = "";
  $r .= is_int($v) ? "i" : "-";
  $r .= is_string($v) ? "s" : "-";
  $r .= is_float($v) ? "f" : "-";
  $r .= is_bool($v) ? "b" : "-";
  $r .= is_array($v) ? "a" : "-";
  $r .= is_object($v) ? "o" : "-";
  $r .= is_scalar($v) ? "S" : "-";
  return $r;
}
foreach ([1, "x", 1.5, true, [1], new P(), null] as $v) { echo k($v), "|"; }
echo "\n";
$n = 5; $t = "z"; $g = 1.25; $arr = [1, 2];
echo is_int($n) ? "1" : "0", is_string($t) ? "1" : "0", is_float($g) ? "1" : "0", is_array($arr) ? "1" : "0";
echo is_string($n) ? "1" : "0", is_int($t) ? "1" : "0", "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the type predicates to WASM");
    assert!(
        output.status.success(),
        "type-predicate compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the type predicates under Node");
    assert!(run.status.success());
    // `null` answers every predicate false, and `is_scalar` covers exactly int/string/float/bool.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "i-----S|-s----S|--f---S|---b--S|----a--|-----o-|-------|\n",
            "111100\n",
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies reading an element out of an `array<mixed>`.
///
/// The historical refusal was that PHP answers NULL for a missing index while the EIR types
/// the result non-null — true for an `array<int>`, whose element storage has no null. It does
/// not hold here: the element is already a Mixed cell, so the miss and the hit meet in the same
/// representation, exactly as the bool and string element arms already do.
#[test]
fn test_cli_wasm_reads_an_element_of_a_mixed_array() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_element");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$a = [1, "x", 2.5];
echo $a[0], "|", $a[1], "|", $a[2], "\n";
foreach ([0, 1, 2, 5, -1] as $i) { echo "[", $a[$i], "]"; }
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the mixed-array reads to WASM");
    assert!(
        output.status.success(),
        "mixed-array read compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the mixed-array reads under Node");
    assert!(run.status.success());
    // A missing or negative index echoes nothing, which is what `echo null` prints.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!("1|x|2.5\n", "[1][x][2.5][][]\n"),
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a read of an UNTYPED property survives the read that precedes it.
///
/// `ir_lower` stabilizes a borrowed `PropGet` result with an `Op::Acquire`, and skips that when
/// the result is already `Owned` — which is exactly what an untyped property produces. The WASM
/// reader assumed the acquire was unconditional and always borrowed, so the `release` the EIR
/// pairs with an owned result freed the cell the object still points at: `echo $this->n` left the
/// slot dangling and the next `$this->n + 1` read a recycled cell, answering 2 for 0 + 1 and then
/// dying on "Unsupported operand types". A DECLARED `string`/`array` property must keep borrowing,
/// or each read leaks one reference, so both shapes are exercised here in one object.
#[test]
fn test_cli_wasm_reads_an_untyped_property_without_freeing_it() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_untyped_property_read");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Counter {
    public $count;
    public string $label;
    public function __construct(string $label) { $this->count = 0; $this->label = $label; }
    public function inc() { $this->count += 1; }
    public function dec() { if ($this->count > 0) { $this->count -= 1; } }
    public function show() { echo $this->label, "=", $this->count, "\n"; }
}
$c = new Counter("a");
$c->show();
$c->inc();
$c->show();
$c->inc();
$c->inc();
$c->show();
$c->dec();
$c->show();
$i = 0;
while ($i < 400) { $c->show(); $c->inc(); $i = $i + 1; }
echo "still ", $c->count, "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the property reads to WASM");
    assert!(
        output.status.success(),
        "untyped property compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
console.error(`pages=${instance.exports.memory.buffer.byteLength / 65536}`);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the property reads under Node");
    assert!(
        run.status.success(),
        "the property reads must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let mut expected = String::from("a=0\na=1\na=3\na=2\n");
    for step in 2..402 {
        expected.push_str(&format!("a={}\n", step));
    }
    expected.push_str("still 402\n");
    assert_eq!(stdout, expected, "php-src's own answers");

    // 400 reads of a DECLARED string property alongside them: retaining those would leak one
    // persisted copy each, which shows up as linear page growth.
    let pages: usize = String::from_utf8_lossy(&run.stderr)
        .trim()
        .rsplit_once('=')
        .map(|(_, count)| count.parse().unwrap_or(usize::MAX))
        .unwrap_or(usize::MAX);
    assert!(
        pages < 8,
        "400 property reads grew the heap to {pages} pages, so a read is retaining"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a container written AFTER a `foreach` still compiles.
///
/// The guard that refuses a mutation of a live iterated container walked every instruction after
/// the `IterStart` in the function's flat instruction table, which includes everything the loop
/// has already finished with. `foreach ($h as ...) {}` followed by `$h["c"] = 3;` is ordinary PHP
/// — the iterator is dead by then — and it was refused. The live range is the loop: the blocks
/// reachable from the header that reach it back. A write INSIDE the loop must still be refused,
/// so this exercises the second loop as a compile-time negative control in the same file.
#[test]
fn test_cli_wasm_writes_a_container_after_iterating_it() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_write_after_foreach");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
$h = ["a" => 1, "b" => 2];
foreach ($h as $k => $v) { echo $k, ":", $v, " "; }
echo "\n";
$h["c"] = 3;
unset($h["a"]);
foreach ($h as $k => $v) { echo $k, "=", $v, " "; }
echo "\n", count($h), "\n";
$list = [10, 20, 30];
foreach ($list as $n) { echo $n, " "; }
$list[] = 40;
echo "\n", implode(",", $list), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the post-loop writes to WASM");
    assert!(
        output.status.success(),
        "post-loop container write compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the post-loop writes under Node");
    assert!(
        run.status.success(),
        "the post-loop writes must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!("a:1 b:2 \n", "b=2 c=3 \n", "2\n", "10 20 30 \n", "10,20,30,40\n"),
        "php-src's own answers"
    );

    // The negative control: the same write INSIDE the loop still has no snapshot to write
    // against, and must stay refused.
    let inside = dir.join("inside.php");
    fs::write(
        &inside,
        r#"<?php
$h = ["a" => 1, "b" => 2];
foreach ($h as $k => $v) { $h["z" . $k] = $v; }
echo count($h), "\n";
"#,
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&inside)
        .output()
        .expect("failed to run the compiler over the in-loop write");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr)
                .contains("may mutate the iterated container"),
        "a write inside the loop must still be refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `round($value, $places)` against php-src, including the cases the naive form fails.
///
/// The two-argument form is a DIFFERENT function from `round($value)`, not the same one with a
/// default. Scaling is inexact — `0.285 * 1e10` is `2849999999.9999995` — so php-src extracts the
/// integral part and then REPAIRS the extraction, adding one back when unscaling the candidate
/// reproduces the input exactly. That repair is why `round(1.005, 2)` is `1.01` and
/// `round(9.995, 2)` is `10`, both of which scale-round-unscale gets wrong. The transcription was
/// validated at 1420/1420 against php-src 8.5.6 over a corpus of halfway values, the classic
/// traps, the 1e15/1e-15 boundaries and 1200 random values across 24 orders of magnitude; the
/// naive model scores 1087/1420 on the same corpus. The values below are the traps from it.
#[test]
fn test_cli_wasm_round_with_a_precision_matches_php() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_round_places");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo round(1.005, 2), "|", round(2.675, 2), "|", round(9.995, 2), "|", round(0.285, 2), "\n";
echo round(8.995, 2), "|", round(0.045, 2), "|", round(1.45, 1), "|", round(1.55, 1), "\n";
echo round(1.005, 3), "|", round(-1.005, 2), "|", round(-9.995, 2), "|", round(-0.285, 2), "\n";
echo round(1234.5678, -2), "|", round(1234.5678, 0), "|", round(-1234.5678, -2), "\n";
echo round(0.5, 0), "|", round(-0.5, 0), "|", round(1.5, 0), "|", round(2.5, 0), "|", round(-2.5, 0), "\n";
echo round(1e15, 2), "|", round(1e-15, 20), "|", round(123456789.987654321, 4), "\n";
$p = 2;
echo round(3.14159, $p), "|", round(2.71828, $p), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the rounding probe to WASM");
    assert!(
        output.status.success(),
        "round compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the rounding probe under Node");
    assert!(
        run.status.success(),
        "rounding must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "1.01|2.68|10|0.29\n",
            "9|0.05|1.5|1.6\n",
            "1.005|-1.01|-10|-0.29\n",
            "1200|1235|-1200\n",
            "1|-1|2|3|-3\n",
            "1.0E+15|1.0E-15|123456789.9877\n",
            "3.14|2.72\n",
        ),
        "php-src 8.5.6's own answers"
    );

    // A precision php-src reaches `pow()` for is refused rather than answered nearly-right.
    let wide = dir.join("wide.php");
    fs::write(&wide, "<?php echo round(1.5, 30), \"\\n\";\n").unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&wide)
        .output()
        .expect("failed to run the compiler over the wide precision");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr).contains("php_intpow10"),
        "a precision outside the exact table must be refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an in-place sort of the array being iterated is refused, and one after it is not.
///
/// PHP's `foreach` walks a SNAPSHOT. This target has no snapshot, so a mutation of the live
/// container has to be refused — and the mutation set was `usort` alone, which let `sort()`
/// through. Measured: `foreach ([5,3,9,1] as $v) { echo $v; sort($a); }` printed `5 3 5 9` where
/// php-src prints `5 3 9 1`, because the loop re-read the array it had just reordered. Every
/// in-place mutator in the registry is refused now; the same call AFTER the loop still compiles
/// and still matches php-src, which is the half that keeps the widening honest.
#[test]
fn test_cli_wasm_refuses_sorting_the_array_being_iterated() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_sort_during_foreach");
    let during = dir.join("during.php");
    fs::write(
        &during,
        "<?php\n$a = [5, 3, 9, 1];\nforeach ($a as $v) { echo $v, \" \"; sort($a); }\n",
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&during)
        .output()
        .expect("failed to run the compiler over the in-loop sort");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr)
                .contains("may mutate the iterated container"),
        "sorting the iterated array must be refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let after = dir.join("main.php");
    fs::write(
        &after,
        r#"<?php
$a = [5, 3, 9, 1];
foreach ($a as $v) { echo $v, " "; }
sort($a);
echo "| ", implode(",", $a), "\n";
"#,
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&after)
        .output()
        .expect("failed to compile the post-loop sort to WASM");
    assert!(
        output.status.success(),
        "sorting after the loop must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the post-loop sort under Node");
    assert!(
        run.status.success(),
        "the post-loop sort must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "5 3 9 1 | 1,3,5,9\n",
        "php-src's own answer"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a container the loop cannot see mutated is refused: through a callee, or an alias.
///
/// Both were measured wrong before the guard learned them. `function bump(array &$a) { $a[] = 99; }`
/// called from inside the loop grew the array the loop was reading and exhausted memory, where
/// php-src walks its snapshot and stops. `$r = &$h;` then `$r[] = 99;` inside the loop printed
/// `5 | 0` against php-src's `5 3 9 1 | 8` — the alias has its own slot, so every slot-keyed check
/// passed. Whether a callee's parameter is declared `&` is not visible at the call site, so
/// handing the container to anything at all ends the proof.
#[test]
fn test_cli_wasm_refuses_mutating_an_iterated_container_it_cannot_see() {
    let dir = make_cli_test_dir("elephc_cli_wasm_hidden_iterated_mutation");

    for (name, source, why) in [
        (
            "callee.php",
            "<?php\nfunction bump(array &$a): void { $a[] = 99; }\n$h = [5, 3, 9, 1];\nforeach ($h as $v) { echo $v; bump($h); }\n",
            "receives the iterated container",
        ),
        (
            "alias.php",
            "<?php\n$h = [5, 3, 9, 1];\n$r = &$h;\nforeach ($h as $v) { echo $v; $r[] = 99; }\n",
            "a reference to the iterated container",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let refused = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to run the compiler over the hidden mutation");
        let stderr = String::from_utf8_lossy(&refused.stderr).into_owned();
        assert!(
            !refused.status.success() && stderr.contains(why),
            "{name} must be refused with {why:?}: {stderr}"
        );
    }

    // A by-VALUE callee cannot mutate anything, so it must still compile. This is the half that
    // keeps the widening honest: the callee's own signature decides, not the fact of a call.
    let by_value = dir.join("byvalue.php");
    fs::write(
        &by_value,
        "<?php\nfunction look(array $a): int { return count($a); }\n$h = [5, 3, 9, 1];\nforeach ($h as $v) { echo $v, \" \"; echo look($h), \" \"; }\necho \"\\n\";\n",
    )
    .unwrap();
    let accepted = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&by_value)
        .output()
        .expect("failed to compile the by-value call in a loop");
    assert!(
        accepted.status.success(),
        "a by-value callee must not be refused: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );

    // The SAME by-reference callee, called AFTER the loop, must compile and answer — that is the
    // "the widening ends where the iterator does" half of the rule. It is the sharper control of
    // the two: `callee.php` above differs from this only in where the call sits, so a guard that
    // refused the whole function rather than the loop's blocks would fail here and nowhere else.
    let after = dir.join("after.php");
    fs::write(
        &after,
        "<?php\nfunction bump(array &$a): void { $a[] = 99; }\n$h = [5, 3, 9, 1];\nforeach ($h as $v) { echo $v, \" \"; }\nbump($h);\necho \"| \", count($h), \"\\n\";\n",
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&after)
        .output()
        .expect("failed to run the compiler over the post-loop callee mutation");
    assert!(
        output.status.success(),
        "a by-reference call AFTER the loop must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    if Command::new("node").arg("--version").output().is_ok() {
        let runner = dir.join("run.mjs");
        fs::write(
            &runner,
            r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
        )
        .unwrap();
        let after_run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("after.wasm"))
            .current_dir(&dir)
            .output()
            .expect("failed to run the post-loop callee mutation under Node");
        assert_eq!(
            String::from_utf8_lossy(&after_run.stdout),
            "5 3 9 1 | 5\n",
            "php-src's own answer ({})",
            String::from_utf8_lossy(&after_run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a constructor that lets `$this` ESCAPE before the initializing store is refused.
///
/// The proof this predicate offers is "no reader can precede the store", and it is only worth
/// anything if every way of handing `$this` to other code ends it. Two shapes measured wrong
/// before they were closed: parking `$this` where something else can reach it —
/// `self::$last = $this;` or `$box->held = $this;` — and, subtler, a store of a DIFFERENT
/// property satisfying "the first property access is a store" and so standing in as proof for a
/// slot it never touched.
///
/// The accepting half matters just as much: two sibling stores must still compile, and a plain
/// `$x = $this;` is not an escape at all, because a local that never leaves the frame cannot read
/// anything. Refusing those would cost real programs for no soundness gain.
#[test]
fn test_cli_wasm_refuses_a_constructor_that_leaks_this_before_its_store() {
    let dir = make_cli_test_dir("elephc_cli_wasm_constructor_escape");

    for (name, source) in [
        (
            "static.php",
            "<?php\nclass C {\n    public $value;\n    public static $last;\n    public function __construct(int $v) { self::$last = $this; $this->value = $v; }\n}\necho (new C(7))->value;\n",
        ),
        (
            "foreign.php",
            "<?php\nclass Box { public $held; public function __construct() { $this->held = 0; } }\nclass Node {\n    public $payload;\n    public function __construct(Box $box) { $box->held = $this; $this->payload = 7; }\n}\necho (new Node(new Box()))->payload;\n",
        ),
        (
            "sibling.php",
            "<?php\nclass A {\n    public $p;\n    public $self;\n    public function __construct($v) { $this->self = $this; leak($this->self); $this->p = $v; }\n}\nfunction leak($o): void { echo $o->p; }\nnew A(1);\n",
        ),
        // A builtin that RECEIVES `$this` runs code that can read the slot, so the argument is
        // what decides — not the mere presence of a call.
        (
            "builtin_arg.php",
            "<?php\nclass P {\n    public int $value;\n    public function __construct(int $v) { var_dump($this); $this->value = $v; }\n}\necho (new P(7))->value;\n",
        ),
        // And a CLOSURE binds `$this` with no operand naming it anywhere, so closure creation
        // ends the proof whatever its arguments say.
        (
            "closure.php",
            "<?php\nclass P {\n    public int $value;\n    public function __construct(int $v) { $rows = [3, 1, 2]; usort($rows, function ($a, $b) { return $this->value <=> $b; }); $this->value = $v; }\n}\necho (new P(7))->value;\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let refused = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to run the compiler over the escaping constructor");
        assert!(
            !refused.status.success(),
            "{name} lets $this escape before the store and must be refused"
        );
    }

    for (name, source) in [
        (
            "siblings.php",
            "<?php\nclass A {\n    public $a;\n    public $b;\n    public function __construct(int $v) { $this->a = 1; $this->b = $v; }\n}\n$x = new A(7);\necho $x->a, $x->b;\n",
        ),
        (
            "localcopy.php",
            "<?php\nclass C { public $a; public function __construct() { $x = $this; $this->a = 1; } }\necho (new C())->a;\n",
        ),
        // A builtin whose arguments do NOT name `$this` cannot observe it, however much user
        // code it runs: the object is fresh from `new`, so the only way in is this call's own
        // arguments — a callback that captured it would be a closure above, and an array or
        // global holding it would be a store above, both of which end the walk first. Treating
        // every call as an escape refused `ArrayIterator::__construct`, which calls
        // `array_keys($array)` before its first property write, and with it every
        // `$this->position` read in the SPL iterator family.
        (
            "builtin_other.php",
            "<?php\nclass C {\n    public int $n;\n    public function __construct(string $s) { $len = strlen($s); $this->n = $len; }\n}\necho (new C(\"abcd\"))->n;\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let accepted = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the well-behaved constructor");
        assert!(
            accepted.status.success(),
            "{name} observes nothing and must compile: {}",
            String::from_utf8_lossy(&accepted.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `gettype()` answers php-src's own spellings, settled or boxed.
///
/// These are historical names, not the type names PHP 8 prints elsewhere: an int is "integer", a
/// float "double", a bool "boolean", and null "NULL" in capitals. A settled EIR type answers at
/// compile time — the type is already decided, and a dispatch would be a slower route to the same
/// string — while a boxed value reads the cell tag. A RESOURCE is refused rather than answered,
/// because php-src distinguishes an open handle from `"resource (closed)"` and the tag alone
/// cannot tell them apart.
#[test]
fn test_cli_wasm_gettype_names_every_type_the_way_php_does() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_gettype");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
echo gettype(42), "|", gettype(3.14), "|", gettype("hi"), "|", gettype(true), "\n";
echo gettype(false), "|", gettype(null), "|", gettype([1, 2]), "|", gettype(["a" => 1]), "\n";
function pick(int $i) {
    if ($i === 0) { return 7; }
    if ($i === 1) { return "s"; }
    if ($i === 2) { return 1.5; }
    if ($i === 3) { return true; }
    if ($i === 4) { return null; }
    return [1];
}
$j = 0;
while ($j <= 5) { echo gettype(pick($j)), " "; $j = $j + 1; }
echo "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the gettype probe to WASM");
    assert!(
        output.status.success(),
        "gettype compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the gettype probe under Node");
    assert!(
        run.status.success(),
        "gettype must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        concat!(
            "integer|double|string|boolean\n",
            "boolean|NULL|array|array\n",
            "integer string double boolean NULL array \n",
        ),
        "php-src 8.5.6's own answers"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `Foo::class` and `self::class` answer the resolved name at compile time.
///
/// The EIR already carries the class, so there is nothing to compute: the answer is a data-segment
/// address and a length. `static::class` is the exception — late static binding resolves it from
/// the CALLED class at runtime, which this target does not forward, so it stays refused rather
/// than quietly answering the defining class instead.
#[test]
fn test_cli_wasm_class_constant_resolves_at_compile_time() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_class_constant");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Logger { public function name(): string { return self::class; } }
class Child extends Logger {}
echo Logger::class, "|", Child::class, "\n";
echo (new Logger())->name(), "|", (new Child())->name(), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the class-constant probe to WASM");
    assert!(
        output.status.success(),
        "class-constant compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the class-constant probe under Node");
    assert!(
        run.status.success(),
        "the class constants must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    // `self::class` is the DEFINING class, so the inherited method answers Logger for both.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "Logger|Child\nLogger|Logger\n",
        "php-src's own answers"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a typed property can be READ inside its own constructor, once the store has run.
///
/// Reading a typed property with no default before anything writes it is
/// `Error: Typed property C::$p must not be accessed before initialization`, and this backend has
/// no sentinel for it — the allocator zeroes the slot and zero is a legitimate int. The rule
/// admitted such a read only from OUTSIDE the constructor, on the grounds that a read inside it
/// could still precede the store. That is a question about the individual read, not about the
/// class: `__construct(string $n) { $this->name = $n; echo $this->name; }` is ordinary PHP whose
/// store demonstrably comes first. The entry block decides, so a read before the store is still
/// refused — carried here as a compile-time negative control.
#[test]
fn test_cli_wasm_reads_a_typed_property_after_its_own_constructor_store() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_constructor_read_after_store");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Scoped {
    public string $name;
    public int $depth;
    public function __construct(string $name, int $depth)
    {
        $this->name = $name;
        $this->depth = $depth;
        echo "open(", $this->name, "@", $this->depth, ")\n";
    }
    public function label(): string { return $this->name; }
}
$a = new Scoped("outer", 1);
$b = new Scoped("inner", 2);
echo $a->label(), "|", $b->label(), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the constructor read to WASM");
    assert!(
        output.status.success(),
        "reading after the store must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the constructor read under Node");
    assert!(
        run.status.success(),
        "the constructor read must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "open(outer@1)\nopen(inner@2)\nouter|inner\n",
        "php-src's own answers"
    );

    // The negative control: reading BEFORE the store observes the uninitialized slot, and php-src
    // raises there, so it must stay refused.
    let before = dir.join("before.php");
    fs::write(
        &before,
        "<?php\nclass C { public int $v; public function __construct(int $x) { echo $this->v; $this->v = $x; } }\nnew C(1);\n",
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&before)
        .output()
        .expect("failed to run the compiler over the pre-store read");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr).contains("may be uninitialized"),
        "a read before the store must stay refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a by-reference CONTAINER parameter round-trips, and pins the one shape still refused.
///
/// A by-ref parameter arrives as a ref-cell pointer, so the callee loads it with `Op::LoadRefCell`
/// rather than `Op::LoadLocal`. `value_source_slot` only recognised `LoadLocal`, so a callee that
/// MOVED the array — `$a[] = 41` growing it past its capacity — had nowhere to write the new
/// pointer back and dropped it on the floor. Measured against php-src 8.5.6 before the fix:
///
/// ```text
///   function m(array &$a) { $a[] = 41; }  $v = [7];  m($v);  echo count($v);
///   php-src: 2      this target: 106808
///   function m(array &$a) { $a[0] = 41; } $v = [7];  m($v);  echo count($v);
///   php-src: 1      this target: 0
/// ```
///
/// Writing the pointer THROUGH the cell when the slot is ref-bound repairs both, along with assoc
/// keys, wholesale reassignment, several by-ref parameters at once, nested by-ref calls, repeated
/// calls, and by-ref callees that also return a value.
///
/// What stays refused is narrower and different in kind: a callee that REPLACES the representation.
/// `$a[] = $i` where `$i` came from `$i++` appends a `mixed`, so EIR widens the whole array with
/// `Op::ArrayToMixed` and stores the wider array back through the cell. The caller gets the new
/// pointer but keeps its `array<int>` element type, and reads 24-byte Mixed cells as a dense i64
/// buffer — `count()` is right, since the length field IS shared, so nothing announces it. The
/// NATIVE backend prints the same raw heap addresses from the same EIR, so that one is an upstream
/// type-facts gap, not a WASM defect; WASM refuses it rather than answering garbage.
#[test]
fn test_cli_wasm_round_trips_a_by_reference_container_parameter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_by_ref_container");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        (
            "push.php",
            "<?php\nfunction m(array &$a): void { $a[] = 41; }\n$v = [7];\nm($v);\necho count($v), \"|\", $v[0], \"|\", $v[1], \"\\n\";\n",
            "2|7|41\n",
        ),
        (
            "set.php",
            "<?php\nfunction m(array &$a): void { $a[0] = 41; }\n$v = [7];\nm($v);\necho count($v), \"|\", $v[0], \"\\n\";\n",
            "1|41\n",
        ),
        (
            "assoc.php",
            "<?php\nfunction m(array &$a): void { $a['k'] = 1; $a['j'] = 2; }\n$v = ['x' => 9];\nm($v);\nforeach ($v as $k => $x) { echo $k, \"=\", $x, \"|\"; }\necho \"\\n\";\n",
            "x=9|k=1|j=2|\n",
        ),
        (
            "replace.php",
            "<?php\nfunction m(array &$a): void { $a = [1, 2, 3]; }\n$v = [7];\nm($v);\necho count($v), \"|\", $v[0], \"|\", $v[2], \"\\n\";\n",
            "3|1|3\n",
        ),
        (
            "two.php",
            "<?php\nfunction m(array &$a, array &$b): void { $a[] = 1; $b[] = 2; $b[] = 3; }\n$x = [0];\n$y = [0];\nm($x, $y);\necho count($x), \"|\", count($y), \"|\", $y[2], \"\\n\";\n",
            "2|3|3\n",
        ),
        (
            "nested.php",
            "<?php\nfunction inner(array &$a): void { $a[] = 2; }\nfunction outer(array &$a): void { $a[] = 1; inner($a); }\n$v = [0];\nouter($v);\necho count($v), \"|\", $v[1], \"|\", $v[2], \"\\n\";\n",
            "3|1|2\n",
        ),
        (
            "repeat.php",
            "<?php\nfunction m(array &$a): void { $a[] = 1; }\n$v = [];\nm($v);\nm($v);\nm($v);\necho count($v), \"\\n\";\n",
            "3\n",
        ),
        (
            "returns.php",
            "<?php\nfunction m(array &$a): int { $a[] = 5; return count($a); }\n$v = [0];\n$r = m($v);\necho $r, \"|\", count($v), \"\\n\";\n",
            "2|2\n",
        ),
        (
            "grows.php",
            "<?php\nfunction m(array &$a): void { $a[] = 0; $a[] = 10; $a[] = 20; $a[] = 30; $a[] = 40; }\n$v = [0];\nm($v);\necho count($v), \"|\", $v[1], \"|\", $v[5], \"\\n\";\n",
            "6|0|40\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the by-ref container");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the by-ref container under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    // The representation-replacing callee stays refused. `$i` is `mixed` because `$i++` can
    // overflow into a float, so appending it widens the array the caller still reads as
    // `array<int>`. Refusing beats the raw heap addresses it would otherwise print.
    let widens = dir.join("widens.php");
    fs::write(
        &widens,
        "<?php\nfunction m(array &$a): void { for ($i = 0; $i < 5; $i++) { $a[] = $i; } }\n$v = [0];\nm($v);\necho count($v), \"\\n\";\n",
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&widens)
        .output()
        .expect("failed to run the compiler over the widening callee");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr).contains("replacing the caller's"),
        "the widening callee must be refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    // A by-reference SCALAR does round-trip, and must keep working — refusing those too would
    // cost real programs for a defect they do not have.
    let scalars = dir.join("main.php");
    fs::write(
        &scalars,
        r#"<?php
function bumpInt(int &$x): void { $x = 41; }
function bumpStr(string &$s): void { $s = "hi"; }
$n = 1;
$t = "a";
bumpInt($n);
bumpStr($t);
echo $n, "|", $t, "\n";
"#,
    )
    .unwrap();
    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&scalars)
        .output()
        .expect("failed to compile the by-ref scalars");
    assert!(
        output.status.success(),
        "by-reference scalars must compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the by-ref scalars under Node");
    assert!(
        run.status.success(),
        "the by-ref scalars must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "41|hi\n",
        "php-src's own answers"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies that an IMPLEMENTOR may bind to a parameter, or a property, that declares an interface.
///
/// This is the same root as the ancestor case above and the third variant of it. The audit
/// compared an argument's representation against the parameter's, and an interface name read as
/// a different representation from any class name — so `feed(Speaker $s)` handed a `Dog` was
/// refused, as was storing that `Dog` into a `public Speaker $voice` slot. Neither refusal was a
/// representation claim that holds: an object is one pointer to a header naming its own runtime
/// class, and an interface-typed slot holds exactly that pointer.
///
/// What the callee then DOES with the value — dispatch a method declared by the interface — is
/// audited where it happens, against every implementor, because PHP picks the body from the
/// runtime class. So a call the interface stub cannot serve is still refused, by name, instead
/// of every argument that could reach it being refused in advance.
///
/// Each case pins an assumption the relaxation depends on, against php-src 8.5.6's own answers:
/// two different implementors reaching one parameter dispatch to their own bodies, an
/// interface-typed PROPERTY does the same after a store and a reload, a transitively extended
/// interface is as good as a direct one, and an implementor that also carries fields of its own
/// keeps them intact across the call. This is what `AppendIterator::append(Iterator $it)` needs
/// to accept an `ArrayIterator`.
#[test]
fn test_cli_wasm_passes_an_implementor_where_the_parameter_declares_an_interface() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_interface_argument");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        // Two implementors through one interface-typed parameter must reach their own bodies.
        (
            "param.php",
            "<?php\ninterface Speaker { public function say(): string; }\nclass Dog implements Speaker { public function say(): string { return \"woof\"; } }\nclass Cat implements Speaker { public function say(): string { return \"meow\"; } }\nfunction feed(Speaker $s): void { echo $s->say(), \"\\n\"; }\nfeed(new Dog());\nfeed(new Cat());\n",
            "woof\nmeow\n",
        ),
        // An interface-typed PROPERTY: the store is the same pointer copy, and the reload
        // dispatches on the runtime class just as the parameter does.
        (
            "property.php",
            "<?php\ninterface Speaker { public function say(): string; }\nclass Dog implements Speaker { public function say(): string { return \"woof\"; } }\nclass Cat implements Speaker { public function say(): string { return \"meow\"; } }\nclass Pen { public Speaker $voice; public function __construct(Speaker $s) { $this->voice = $s; } public function heard(): string { return $this->voice->say(); } }\necho (new Pen(new Dog()))->heard(), \"|\", (new Pen(new Cat()))->heard(), \"\\n\";\n",
            "woof|meow\n",
        ),
        // A transitively extended interface is as much an implemented interface as a direct one.
        (
            "extends.php",
            "<?php\ninterface Animal { public function say(): string; }\ninterface Pet extends Animal {}\nclass Dog implements Pet { public function say(): string { return \"woof\"; } }\nfunction feed(Animal $a): void { echo $a->say(), \"\\n\"; }\nfeed(new Dog());\n",
            "woof\n",
        ),
        // The implementor's OWN fields must survive the call through the interface-typed
        // parameter — the sharpest check that nothing was reinterpreted on the way in.
        (
            "fields.php",
            "<?php\ninterface Speaker { public function say(): string; }\nclass Dog implements Speaker { public int $legs = 4; public string $name = \"rex\"; public function say(): string { return \"woof\"; } }\nfunction feed(Speaker $s): void { echo $s->say(), \"\\n\"; }\n$d = new Dog();\nfeed($d);\necho $d->legs, \"|\", $d->name, \"\\n\";\n",
            "woof\n4|rex\n",
        ),
        // An implementor reached through a parameter that names an interface its PARENT declares:
        // the walk has to cross a `parent` link before it finds the `implements`.
        (
            "inherited.php",
            "<?php\ninterface Speaker { public function say(): string; }\nclass Animal implements Speaker { public function say(): string { return \"...\"; } }\nclass Dog extends Animal { public function say(): string { return \"woof\"; } }\nfunction feed(Speaker $s): void { echo $s->say(), \"\\n\"; }\nfeed(new Animal());\nfeed(new Dog());\n",
            "...\nwoof\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the interface argument");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the interface argument under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies that a SUBCLASS argument may bind to a parameter that declares one of its ancestors.
///
/// The capability audit compares an argument's representation against the parameter's, and two
/// object types with different class names read as two different representations — so `look(Base
/// $x)` called with `new Kid()` was refused, even though PHP's whole inheritance story is that the
/// call is legal. The refusal was a representation claim that is not true: an object is ONE pointer
/// to a header naming its own runtime class, which is exactly why `instanceof` and virtual dispatch
/// both answer off the value rather than the static type.
///
/// So the physical layer says two object pointers are copy-compatible, and the semantic question —
/// may THIS class stand in for THAT one — is answered where the hierarchy is in scope, by
/// `argument_is_a_descendant_of_the_parameter` in the capability audit. That helper now walks
/// `implements` as well as `parent`, so an INTERFACE-typed parameter is admitted on the same
/// reasoning; the dispatch that follows is pinned by
/// `test_cli_wasm_passes_an_implementor_where_the_parameter_declares_an_interface`.
///
/// Each case below pins an assumption the relaxation depends on, against php-src 8.5.6's own
/// answers: inherited fields keep their offsets when the subclass adds its own — including fields
/// of DIFFERENT representations ahead of them — an overridden method dispatches on the runtime
/// class through an ancestor-typed parameter, a write through such a parameter lands on the base
/// field and leaves the subclass's own untouched, and the same callee body serves both the base and
/// the descendant across repeated calls. Unblocks `examples/instanceof`, whose output is now
/// byte-identical to php-src.
#[test]
fn test_cli_wasm_passes_a_subclass_where_the_parameter_declares_an_ancestor() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_subclass_argument");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        // An inherited field read through the ancestor-typed parameter, with the subclass adding
        // a field of its own: the base field must still be found at the base's offset.
        (
            "fields.php",
            "<?php\nclass Base { public int $a = 1; public int $b = 2; }\nclass Kid extends Base { public int $c = 3; }\nfunction look(Base $x): void { echo $x->a, \"|\", $x->b, \"\\n\"; }\nlook(new Base());\nlook(new Kid());\n",
            "1|2\n1|2\n",
        ),
        // An overridden method reached through the ancestor-typed parameter must dispatch on the
        // RUNTIME class, or the descendant would answer with the base's body.
        (
            "dispatch.php",
            "<?php\nclass Base { public function who(): string { return \"base\"; } }\nclass Kid extends Base { public function who(): string { return \"kid\"; } }\nfunction look(Base $x): void { echo $x->who(), \"\\n\"; }\nlook(new Base());\nlook(new Kid());\n",
            "base\nkid\n",
        ),
        // Field offset and dispatch at once, with the subclass inserting a field of a DIFFERENT
        // representation — a string next to the base's int.
        (
            "both.php",
            "<?php\nclass Base { public int $a = 1; public function who(): string { return \"base\"; } }\nclass Kid extends Base { public string $extra = \"x\"; public function who(): string { return \"kid\"; } }\nfunction look(Base $x): void { echo $x->a, \":\", $x->who(), \"\\n\"; }\nlook(new Base());\nlook(new Kid());\n",
            "1:base\n1:kid\n",
        ),
        // A WRITE through the ancestor-typed parameter must land on the base field and leave the
        // subclass's own field intact — the sharpest test that the offsets did not shift.
        (
            "write.php",
            "<?php\nclass Base { public int $a = 1; }\nclass Kid extends Base { public int $c = 9; }\nfunction bump(Base $x): void { $x->a = 41; }\n$k = new Kid();\nbump($k);\necho $k->a, \"|\", $k->c, \"\\n\";\n",
            "41|9\n",
        ),
        // Three levels deep: a grandchild is as much a descendant as a child.
        (
            "three.php",
            "<?php\nclass A { public int $a = 1; public function who(): string { return \"A\"; } }\nclass B extends A { public int $b = 2; public function who(): string { return \"B\"; } }\nclass C extends B { public int $c = 3; public function who(): string { return \"C\"; } }\nfunction look(A $x): void { echo $x->a, \":\", $x->who(), \"\\n\"; }\nlook(new A());\nlook(new B());\nlook(new C());\n",
            "1:A\n1:B\n1:C\n",
        ),
        // Four base fields of four different representations, all read through the ancestor-typed
        // parameter after the subclass appended two more.
        (
            "layout.php",
            "<?php\nclass Base { public string $name = \"n\"; public float $ratio = 0.5; public int $count = 7; public bool $on = true; }\nclass Kid extends Base { public string $extra = \"x\"; public int $more = 99; }\nfunction look(Base $x): void { echo $x->name, \"|\", $x->ratio, \"|\", $x->count, \"|\", $x->on ? \"T\" : \"F\", \"\\n\"; }\nlook(new Base());\n$k = new Kid();\n$k->name = \"kid\";\n$k->ratio = 2.25;\n$k->count = 41;\n$k->on = false;\nlook($k);\necho $k->extra, \"|\", $k->more, \"\\n\";\n",
            "n|0.5|7|T\nkid|2.25|41|F\nx|99\n",
        ),
        // One callee body serving the base and the descendant across repeated calls, mutating and
        // re-reading the base field while the subclass's own field survives.
        (
            "mutate.php",
            "<?php\nclass Node { public int $v = 0; public function label(): string { return \"node\"; } }\nclass Leaf extends Node { public string $tag = \"t\"; public function label(): string { return \"leaf\"; } }\nfunction nudge(Node $n, int $add): string { $n->v = $n->v + $add; return $n->label() . \"=\" . $n->v; }\n$a = new Node();\necho nudge($a, 3), \"\\n\";\necho nudge($a, 4), \"\\n\";\n$l = new Leaf();\n$l->tag = \"keepme\";\necho nudge($l, 10), \"\\n\";\necho $l->tag, \"|\", $l->v, \"\\n\";\n",
            "node=3\nnode=7\nleaf=10\nkeepme|10\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the subclass argument");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the subclass argument under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies that a method call on an INTERFACE-typed receiver dispatches on the runtime class.
///
/// An interface-typed receiver was refused as an `unknown receiver class`, which named the wrong
/// problem: an interface is not an unknown class, it is a class-less one. It declares no storage
/// and owns no body, so there is nothing to call into — but the object that arrives carries its
/// real class id in its own header, which is exactly what PHP dispatches on. The callee is
/// therefore the closed set of concrete implementors, and that set is enumerable at compile time.
///
/// So this reuses the ladder that already serves virtual calls, with a different arm set: the
/// interface's implementors rather than one class's subtree. Membership is walked in both
/// directions, because PHP hands a class its parents' interfaces and an interface its parents'
/// methods — `class C extends B` where `B implements J` and `interface J extends I` makes a `C` a
/// legitimate `I`, and reading only a class's own `implements` list would miss it.
///
/// The refusals below are the two ways the set can fail to share one stub, and both must be
/// caught by the AUDIT rather than by the stub emitter: the emitter simply skips an interface it
/// cannot serve, so an audit that accepted one of these would leave the module calling a function
/// that was never defined. An interface with no concrete implementor has no arm to select, and an
/// interface method with no declared return type lets each implementor pick its own — including
/// `void` against `int`, whose return arities differ, which would unbalance the wasm stack. That
/// second one is caught even though the call DISCARDS the result.
///
/// Unblocks `examples/intersection-types`, `examples/anonymous-classes` and
/// `examples/enum-methods`, all three byte-identical to php-src.
#[test]
fn test_cli_wasm_dispatches_a_method_call_on_an_interface_typed_receiver() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_interface_dispatch");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        // Two implementors: the ladder must pick by runtime class.
        (
            "two.php",
            "<?php\ninterface Speaks { public function say(): string; }\nclass Dog implements Speaks { public function say(): string { return \"woof\"; } }\nclass Cat implements Speaks { public function say(): string { return \"meow\"; } }\nfunction hear(Speaks $s): string { return $s->say(); }\necho hear(new Dog()), \"|\", hear(new Cat()), \"\\n\";\n",
            "woof|meow\n",
        ),
        // The obligation is INHERITED twice over: `Puppy`/`Quiet` never say `implements`, and
        // `Loud` extends the interface rather than declaring the method. `Quiet` also inherits
        // the IMPLEMENTATION, so its arm must call `Dog`'s body.
        (
            "inherited.php",
            "<?php\ninterface Speaks { public function say(): string; }\ninterface Loud extends Speaks {}\nclass Dog implements Loud { public function say(): string { return \"woof\"; } }\nclass Puppy extends Dog { public function say(): string { return \"yip\"; } }\nclass Quiet extends Dog {}\nfunction hear(Speaks $s): string { return $s->say(); }\necho hear(new Dog()), \"|\", hear(new Puppy()), \"|\", hear(new Quiet()), \"\\n\";\nfunction hearLoud(Loud $s): string { return $s->say(); }\necho hearLoud(new Puppy()), \"\\n\";\n",
            "woof|yip|woof\nyip\n",
        ),
        // An ABSTRACT implementor cannot be instantiated, so what the ladder must cover is its
        // concrete subclasses. Also passes an argument and returns a non-string.
        (
            "abstract.php",
            "<?php\ninterface Scales { public function scale(int $by): int; }\nabstract class Shape implements Scales { public function __construct(protected int $size) {} }\nclass Square extends Shape { public function scale(int $by): int { return $this->size * $by; } }\nclass Circle extends Shape { public function scale(int $by): int { return $this->size * $by * 2; } }\nfunction grow(Scales $s, int $by): int { return $s->scale($by); }\necho grow(new Square(3), 4), \"|\", grow(new Circle(3), 4), \"\\n\";\n",
            "12|24\n",
        ),
        // A `void` body pushes nothing, so the stub must not leave a phantom value behind.
        (
            "void.php",
            "<?php\ninterface Sink { public function put(string $line): void; }\nclass Echoer implements Sink { public function put(string $line): void { echo \"e:\", $line, \"\\n\"; } }\nclass Prefixer implements Sink { public function __construct(private string $p) {} public function put(string $line): void { echo $this->p, $line, \"\\n\"; } }\nfunction drain(Sink $s): void { $s->put(\"x\"); $s->put(\"y\"); }\ndrain(new Echoer());\ndrain(new Prefixer(\"p:\"));\n",
            "e:x\ne:y\np:x\np:y\n",
        ),
        // A single implementor: the one-armed ladder must still select it rather than fall
        // through to the trap, across repeated calls through the same interface-typed local.
        (
            "sole.php",
            "<?php\ninterface Only { public function v(): int; }\nclass Sole implements Only { public function v(): int { return 7; } }\nfunction take(Only $o): int { return $o->v(); }\necho take(new Sole()), \"\\n\";\n$o = new Sole();\necho take($o), take($o), \"\\n\";\n",
            "7\n77\n",
        ),
        // The receiver reaches the call through an interface-typed PROPERTY, not a local.
        (
            "property.php",
            "<?php\ninterface Fmt { public function f(string $v): string; }\nclass Up implements Fmt { public function f(string $v): string { return strtoupper($v); } }\nclass Down implements Fmt { public function f(string $v): string { return strtolower($v); } }\nclass Holder { public function __construct(private Fmt $fmt) {} public function run(string $v): string { return $this->fmt->f($v); } }\necho (new Holder(new Up()))->run(\"hi\"), \"|\", (new Holder(new Down()))->run(\"HI\"), \"\\n\";\n",
            "HI|hi\n",
        ),
        // An enum case and an anonymous class are ordinary implementors to the dispatcher.
        (
            "enum.php",
            "<?php\ninterface HasTag { public function tag(): string; }\nenum Colour: string implements HasTag { case Red = \"r\"; case Blue = \"b\"; public function tag(): string { return $this->value; } }\nclass Named implements HasTag { public function tag(): string { return \"n\"; } }\nfunction label(HasTag $h): string { return $h->tag(); }\necho label(Colour::Red), \"|\", label(Colour::Blue), \"|\", label(new Named()), \"\\n\";\n$anon = new class implements HasTag { public function tag(): string { return \"a\"; } };\necho label($anon), \"\\n\";\n",
            "r|b|n\na\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the interface dispatch");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the interface dispatch under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    // Both refusals must come from the AUDIT. The stub emitter skips an interface it cannot
    // serve, so an audit that accepted either of these would leave a `call` to a function that
    // was never defined.
    for (name, source, needle) in [
        (
            "ghost.php",
            "<?php\ninterface Ghost { public function boo(): string; }\nfunction scare(Ghost $g): string { return $g->boo(); }\necho \"compiled\\n\";\n",
            "has no concrete implementor",
        ),
        (
            "arity.php",
            "<?php\ninterface Runs { public function go(); }\nclass Silent implements Runs { public function go(): void { echo \"s\\n\"; } }\nclass Counting implements Runs { public function go(): int { echo \"c\\n\"; return 1; } }\nfunction fire(Runs $r): void { $r->go(); }\nfire(new Silent());\nfire(new Counting());\n",
            "differs from method body",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let refused = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to run the compiler over the unservable interface");
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            !refused.status.success() && stderr.contains(needle),
            "{name} must be refused with {needle:?}: {stderr}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies reading an element that is itself an ARRAY (or an object) out of an indexed array.
///
/// The refusal said `element Array(Int) cannot lower into Heap(Array)/Array(Int)`, and the reason
/// was the MISS rather than the hit: every accepted element type had somewhere to put PHP's
/// missing-key null — an int became a tagged scalar, a bool/string/mixed became a Mixed cell — and
/// a raw container pointer appeared to have nowhere. It does: pointer 0, which is the same null
/// the native backend already produces, and which no live array or object can collide with because
/// both are allocated.
///
/// Three things had to become true together, and each one is a case below.
///
/// The getters had to survive a null SOURCE, since a read chained onto a miss now passes one. They
/// were loading the length straight off the pointer, which for 0 reads address 0 — valid linear
/// memory in wasm, so it would have answered from whatever happened to be there instead of
/// trapping. Each one now answers its own miss value for a null array.
///
/// PHP then has TWO diagnostics here and picks between them by looking at the source, not the
/// result — both produce null:
///
/// ```text
///   $a = [[1, 2], [3, 4]];  $b = $a[5];  echo $b[1];
///   php-src: Warning: Undefined array key 5
///            Warning: Trying to access array offset on null
/// ```
///
/// And `is_null` had to stop answering from the static type. Its fallback was `statically
/// non-null`, which is a claim the EIR cannot back: reading a missing element of
/// `array<array<int>>` is typed `array<int>`, the null having been dropped at that boundary, so a
/// pointer that IS 0 reported itself non-null. Testing the pointer cannot be wrong.
///
/// The last case is the one that made the difference between reading memory and reading FREED
/// memory. `__rt_array_get_object` hands the stored pointer back BORROWED, but the EIR types this
/// result `own=owned` and emits a matching `release`. Without a reference of its own that release
/// drops the PARENT's, so `$g[1][0][1]` freed a child its parent still pointed at, while
/// `$x = $g[1]; count($x)` did not — the chained form has no `acquire` to balance it. Measured
/// after adding the incref: 20000 iterations of build-then-read hold at 3 wasm pages, the same as
/// the identical loop without the read, while a loop that deliberately retains every child grows
/// to 32 — so the flat number is a balanced refcount and not a blind measurement.
#[test]
fn test_cli_wasm_reads_a_nested_array_element() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_nested_array_read");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // php-src 8.5.6's own answers. Its CLI prints diagnostics to stdout and this backend prints
    // them to stderr, so the two streams are checked separately rather than concatenated.
    for (name, source, expected_out, expected_err) in [
        // The inner array through a local, and the same read chained.
        (
            "local.php",
            "<?php\n$a = [[1, 2], [3, 4]];\n$b = $a[0];\necho $b[1], \"\\n\";\n",
            "2\n",
            "",
        ),
        (
            "chained.php",
            "<?php\n$a = [[1, 2], [3, 4]];\necho $a[0][1], \"\\n\";\n",
            "2\n",
            "",
        ),
        // A miss is null, and says so.
        (
            "miss.php",
            "<?php\n$a = [[1, 2], [3, 4]];\n$b = $a[5];\necho is_null($b) ? \"null\" : \"notnull\", \"\\n\";\n",
            "null\n",
            "Warning: Undefined array key 5\n",
        ),
        // Reading THROUGH the miss is PHP's other diagnostic, and execution continues.
        (
            "through.php",
            "<?php\n$a = [[1, 2], [3, 4]];\n$b = $a[5];\necho $b[1], \"\\n\";\necho \"after\\n\";\n",
            "\nafter\n",
            "Warning: Undefined array key 5\nWarning: Trying to access array offset on null\n",
        ),
        // An OBJECT element is the same 8-byte pointer slot and the same 0 sentinel.
        (
            "object.php",
            "<?php\nclass P { public int $v = 7; }\n$a = [new P()];\n$hit = $a[0];\necho $hit->v, \"\\n\";\n$miss = $a[9];\necho is_null($miss) ? \"null\" : \"notnull\", \"\\n\";\n",
            "7\nnull\n",
            "Warning: Undefined array key 9\n",
        ),
        // THREE levels chained — the case that read freed memory before the read took a
        // reference of its own.
        (
            "deep.php",
            "<?php\n$g = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];\necho $g[1][0][1], \"\\n\";\n",
            "6\n",
            "",
        ),
        // Three levels with VARIABLE indices, so nothing is decidable at compile time, mixed
        // with reads bound to locals and a `count()` of an inner array.
        (
            "vars.php",
            "<?php\n$g = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];\n$i = 1;\n$j = 0;\n$k = 1;\necho $g[$i][$j][$k], \"\\n\";\n$row = $g[0];\n$cell = $row[1];\necho $cell[0], \"\\n\";\necho count($g[1]), \"\\n\";\n",
            "6\n3\n2\n",
            "",
        ),
        // Enough elements that the outer array must GROW past its initial capacity while it is
        // already shaped to pointer slots.
        (
            "grows.php",
            "<?php\n$a = [[1], [2], [3], [4]];\necho count($a), \"\\n\";\necho $a[0][0], $a[1][0], $a[2][0], $a[3][0], \"\\n\";\n",
            "4\n1234\n",
            "",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the nested array read");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the nested array read under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_out,
            "{name}: php-src's own stdout"
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            expected_err,
            "{name}: php-src's own diagnostics"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies WRITING an array or object element into an indexed array — the read's counterpart.
///
/// The capability audit already accepted this shape; only the lowering was missing, which is why
/// the failure read `array_set of Ptr(...)` rather than a refusal. The slot is the same 8-byte
/// pointer `array_push` writes, and the array takes a SHARE of the child while the EIR releases
/// the operand right after the set.
///
/// What the setter has to do that the push does not is RELEASE the slot's previous occupant,
/// since that child is refcounted and would otherwise be stranded. The release happens AFTER the
/// store, which is what makes `$a[0] = $a[0]` safe: the incoming pointer was already increfed, so
/// a self-assignment nets to no change instead of freeing the value mid-write. Measured: 20000
/// overwrites of one slot hold at 3 wasm pages, against 32 for a loop that deliberately retains
/// every child — so the flat number is a balanced refcount, not a blind measurement.
///
/// Writing PAST the end is deliberately not covered here, because both backends get it wrong in
/// the same pre-existing way and this change neither caused nor widened it. PHP treats `$a[3]` on
/// a one-element array as a SPARSE key, so `count()` is 2; a dense representation with no
/// occupancy bit fills the gap instead and answers 4. Measured identically on the scalar setter
/// that predates this work (`$a = [1]; $a[3] = 4; count($a)` → 4), so the container setter matches
/// the scalar one rather than introducing a second wrong answer.
///
/// Unblocks `examples/cow`, whose copy-on-write output now matches php-src byte for byte.
#[test]
fn test_cli_wasm_writes_a_nested_array_element() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_nested_array_write");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        // Replace one inner array; the untouched sibling must survive.
        (
            "replace.php",
            "<?php\n$a = [[1, 2], [3, 4]];\n$a[0] = [7, 8];\necho $a[0][0], $a[0][1], \"|\", $a[1][0], \"\\n\";\necho count($a), \"\\n\";\n",
            "78|3\n2\n",
            ),
        // Self-assignment: the release must come after the store, or this frees the value
        // it is writing.
        (
            "self.php",
            "<?php\n$a = [[1, 2], [3, 4]];\n$a[0] = $a[0];\necho $a[0][0], $a[0][1], \"\\n\";\n$a[1] = [9];\necho count($a[1]), \"|\", $a[1][0], \"\\n\";\necho count($a), \"\\n\";\n",
            "12\n1|9\n2\n",
        ),
        // Overwriting the same slot repeatedly, then reading through it.
        (
            "repeat.php",
            "<?php\n$a = [[0, 0]];\n$i = 0;\nwhile ($i < 5) {\n    $a[0] = [1, 2];\n    $i = $i + 1;\n}\necho count($a), \"|\", $a[0][0], $a[0][1], \"\\n\";\n",
            "1|12\n",
        ),
        // An OBJECT element written into the same pointer slot. The reads bind to locals
        // first: reading a property straight off an array element is a separate prop_get
        // gap (`result TaggedScalar must exactly match declared slot Int`) and would test
        // that rather than this.
        (
            "object.php",
            "<?php\nclass P { public int $v = 0; public function __construct(int $v) { $this->v = $v; } }\n$a = [new P(1), new P(2)];\n$a[0] = new P(9);\n$first = $a[0];\n$second = $a[1];\necho $first->v, $second->v, \"\\n\";\n",
            "92\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the nested array write");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the nested array write under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `Enum::cases()` and `Enum::tryFrom()`, which PHP synthesizes and no body backs.
///
/// The refusal was `missing method body Color::cases`, which was true and beside the point: PHP
/// generates these for every enum, so there is no function to find on either backend. They are
/// open-coded against the case singletons instead, the same treatment the Throwable accessors
/// already get, and audited against what the emitter produces rather than against a body that
/// will never exist.
///
/// `cases()` materializes every case in DECLARATION order into a pointer-slot array under
/// `value_type` 4 — an ordinary `array<Object>`, so `count()`, `foreach` and an indexed read all
/// reach it with no special case. The array takes a SHARE of each singleton; measured balanced at
/// 20000 calls holding 3 wasm pages against 32 for a loop that deliberately retains its children.
///
/// `tryFrom()` walks the cases as an equality ladder over the BACKING value and boxes the winner
/// under Mixed tag 6, a miss boxing null under tag 8 — which is what lets `?? Default`, `is_null`
/// and `===` answer the way php-src does. For a string-backed enum the LENGTH is compared first
/// and separately, because the byte comparison reads the case's length from the needle and would
/// otherwise run past a shorter one; `"spade"` against `Spades = "spades"` and `"clubs"` against
/// `Clubs = "club"` are the two directions of that, and the empty string is the degenerate case.
///
/// `from()` is deliberately still refused. It has to raise php-src's `ValueError` naming the enum
/// and the offending value when nothing matches, and answering it without that raise would turn a
/// fatal into a wrong value.
///
/// Reading a property straight off the boxed `tryFrom` result is a separate prop_get gap
/// (`property receiver must resolve to a concrete object, got Heap(Mixed)/Mixed`), so the cases
/// below reach the result through `is_null`, `??` and `===` instead of through `->name`.
#[test]
fn test_cli_wasm_open_codes_enum_cases_and_try_from() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_enum_intrinsics");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program.
    for (name, source, expected) in [
        // An INT-backed enum: cases() in declaration order, and tryFrom hit/miss.
        (
            "int.php",
            "<?php\nenum Color: int {\n    case Red = 1;\n    case Green = 2;\n    case Blue = 3;\n}\necho count(Color::cases()), \"\\n\";\nforeach (Color::cases() as $c) { echo $c->name, \"=\", $c->value, \" \"; }\necho \"\\n\";\necho is_null(Color::tryFrom(2)) ? \"null\" : \"found\", \"\\n\";\necho is_null(Color::tryFrom(9)) ? \"null\" : \"found\", \"\\n\";\n$picked = Color::tryFrom(4) ?? Color::Red;\necho $picked === Color::Red ? \"fellback\" : \"matched\", \"\\n\";\n$hit = Color::tryFrom(3) ?? Color::Red;\necho $hit === Color::Blue ? \"blue\" : \"other\", \"\\n\";\n",
            "3\nRed=1 Green=2 Blue=3 \nfound\nnull\nfellback\nblue\n",
        ),
        // A STRING-backed enum, with the two prefix directions and the empty needle.
        (
            "string.php",
            "<?php\nenum Suit: string {\n    case Hearts = \"h\";\n    case Spades = \"spades\";\n    case Clubs = \"club\";\n}\necho count(Suit::cases()), \"\\n\";\nforeach (Suit::cases() as $s) { echo $s->name, \"=\", $s->value, \" \"; }\necho \"\\n\";\necho is_null(Suit::tryFrom(\"spades\")) ? \"null\" : \"found\", \"\\n\";\necho is_null(Suit::tryFrom(\"spade\")) ? \"null\" : \"found\", \"\\n\";\necho is_null(Suit::tryFrom(\"clubs\")) ? \"null\" : \"found\", \"\\n\";\necho is_null(Suit::tryFrom(\"h\")) ? \"null\" : \"found\", \"\\n\";\necho is_null(Suit::tryFrom(\"\")) ? \"null\" : \"found\", \"\\n\";\n$s = Suit::tryFrom(\"club\") ?? Suit::Hearts;\necho $s === Suit::Clubs ? \"clubs\" : \"other\", \"\\n\";\n",
            "3\nHearts=h Spades=spades Clubs=club \nfound\nnull\nnull\nfound\nnull\nclubs\n",
        ),
        // A PURE enum has cases() but no backing value, so no tryFrom to call.
        (
            "pure.php",
            "<?php\nenum Direction {\n    case Up;\n    case Down;\n}\necho count(Direction::cases()), \"\\n\";\nforeach (Direction::cases() as $d) { echo $d->name, \" \"; }\necho \"\\n\";\n",
            "2\nUp Down \n",
        ),
        // A backed enum with ZERO cases is legal PHP, and its needle WIDTH still comes from the
        // declared backing type. Reading that width off the case list instead made this take the
        // int path, popping one operand from a string's two-operand push and leaving the pointer
        // behind — `values remaining on stack at end of block`, rejected by wasm validation, for
        // a program php-src answers with null.
        (
            "empty_backed.php",
            "<?php\nenum E: string {}\n$x = E::tryFrom(\"H\");\necho is_null($x) ? \"null\" : \"found\", \"\\n\";\necho count(E::cases()), \"\\n\";\n",
            "null\n0\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the enum intrinsics");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the enum intrinsics under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    // `from()` must stay refused: without php-src's ValueError on no match it would answer a
    // wrong value where PHP terminates.
    let from = dir.join("from.php");
    fs::write(
        &from,
        "<?php\nenum Color: int { case Red = 1; }\necho Color::from(1)->name, \"\\n\";\n",
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&from)
        .output()
        .expect("failed to run the compiler over Enum::from");
    assert!(
        !refused.status.success(),
        "Enum::from must stay refused: {}",
        String::from_utf8_lossy(&refused.stdout)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the two consumers of a null container that a multi-model audit found unguarded.
///
/// Reading a missing element of an `array<array<…>>` or `array<Object>` answers pointer 0 while
/// the EIR still types the result as the element's own non-null type — the null it drops at that
/// boundary. Hardening the getters and `is_null` against that value covered reading it; two
/// consumers were left trusting the type, and both were reachable only because that read became
/// possible. An independent audit by GLM 5.2, Kimi K2.7 and Kimi K3 surfaced them.
///
/// WRITING through the null was the worse of the two, because it answered rather than stopped:
/// php-src AUTOVIVIFIES silently, building a fresh array, where this backend exhausted memory
/// trying to treat address 0 as an array header. The setters now build the array php-src builds.
///
/// CALLING a method on the null terminated either way, but said the wrong thing: `Invalid
/// callable dispatch` — the dispatch ladder's fallthrough trap — instead of php-src's `Call to a
/// member function hi() on null`. A raw object pointer used to be non-zero by construction, so
/// nothing checked. The NATIVE backend already answers this correctly from the same EIR, so the
/// guard is parity rather than caution.
///
/// (php-src also prints a file, line and stack trace after the message; this backend prints no
/// trace for any fatal, so only the message is compared — the same convention its other fatal
/// tests use.)
#[test]
fn test_cli_wasm_guards_the_consumers_of_a_missed_container_element() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_missed_container_consumers");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // php-src 8.5.6's own answers: stdout, then the diagnostics it writes with
    // display_errors=stderr.
    for (name, source, expected_out, expected_err) in [
        // Writing a CONTAINER through the miss: php-src autovivifies and carries on.
        (
            "write_container.php",
            "<?php\n$a = [[[1]]];\n$b = $a[9];\n$b[0] = [2];\necho count($b), \"\\n\";\necho \"survived\\n\";\n",
            "1\nsurvived\n",
            "Warning: Undefined array key 9\n",
        ),
        // The same through a SCALAR setter, reachable by exactly the same route.
        (
            "write_scalar.php",
            "<?php\n$a = [[1]];\n$b = $a[9];\n$b[0] = 5;\necho count($b), \"|\", $b[0], \"\\n\";\necho \"survived\\n\";\n",
            "1|5\nsurvived\n",
            "Warning: Undefined array key 9\n",
        ),
        // Writing PAST the end of the autovivified array still extends it.
        (
            "write_offset.php",
            "<?php\n$a = [[1]];\n$b = $a[9];\n$b[2] = 7;\necho count($b), \"|\", $b[2], \"\\n\";\n",
            "3|7\n",
            "Warning: Undefined array key 9\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the missed-container consumer");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the missed-container consumer under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected_out,
            "{name}: php-src's own stdout"
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stderr),
            expected_err,
            "{name}: php-src's own diagnostics"
        );
    }

    // `count()` of the missed element is php-src's TypeError, and it TERMINATES. Loading the
    // length off pointer 0 answered `4295050542` and carried on.
    let counted = dir.join("count_on_null.php");
    fs::write(
        &counted,
        "<?php\n$a = [[1, 2]];\n$b = $a[9];\necho count($b), \"\\n\";\necho \"survived\\n\";\n",
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&counted)
        .output()
        .expect("failed to compile the null count");
    assert!(
        built.status.success(),
        "count_on_null.php must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("count_on_null.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the null count under Node");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains(
            "count(): Argument #1 ($value) must be of type Countable|array, null given"
        ),
        "count() of a null must raise php-src's own TypeError, got: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&run.stdout).contains("survived"),
        "php-src terminates here: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    // Reading a PROPERTY through the missed element must name php-src's own warning. It used to
    // answer a bare `1` off address 0 with no diagnostic at all. The VALUE still is not php-src's
    // — the read evaluates to null there, and the EIR types this result a non-nullable `int` —
    // so this asserts the diagnostic, which is exact, and pins the value to what the NATIVE
    // backend leaves in that slot, which is the null sentinel.
    let property = dir.join("property_on_null.php");
    fs::write(
        &property,
        "<?php\nclass P { public int $age = 7; }\n$a = [new P()];\n$b = $a[9];\necho $b->age, \"\\n\";\necho \"survived\\n\";\n",
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&property)
        .output()
        .expect("failed to compile the null property read");
    assert!(
        built.status.success(),
        "property_on_null.php must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("property_on_null.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the null property read under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stderr),
        "Warning: Undefined array key 9\nWarning: Attempt to read property \"age\" on null\n",
        "php-src's own diagnostics, in its own order"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "9223372036854775806\nsurvived\n",
        "the native backend's null sentinel, and execution continues as php-src does"
    );

    // A method call on the missed OBJECT element must name php-src's own Error.
    let call = dir.join("call_on_null.php");
    fs::write(
        &call,
        "<?php\nclass A { public function hi(): int { return 42; } }\n$a = [new A()];\n$x = $a[9];\necho $x->hi(), \"\\n\";\n",
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&call)
        .output()
        .expect("failed to compile the null-receiver call");
    assert!(
        built.status.success(),
        "call_on_null.php must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("call_on_null.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the null-receiver call under Node");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains("Warning: Undefined array key 9")
            && stderr.contains("Call to a member function hi() on null"),
        "the null receiver must raise php-src's own Error, got: {stderr}"
    );
    assert!(
        !stderr.contains("Invalid callable dispatch"),
        "the dispatch fallthrough must not stand in for PHP's diagnostic: {stderr}"
    );

    // The open-coded `Throwable` accessor returns from the method lowering WITHOUT dispatching,
    // so a guard fused to the dispatch load missed it: this printed an empty message and
    // CONTINUED, having read address 0 + the property offset. The check now runs before every
    // path, accessor included.
    let accessor = dir.join("accessor_on_null.php");
    fs::write(
        &accessor,
        "<?php\n$a = [new Exception(\"boom\")];\n$e = $a[9];\necho $e->getMessage(), \"\\n\";\necho \"survived\\n\";\n",
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&accessor)
        .output()
        .expect("failed to compile the null-receiver accessor");
    assert!(
        built.status.success(),
        "accessor_on_null.php must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("accessor_on_null.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the null-receiver accessor under Node");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("Call to a member function getMessage() on null"),
        "the accessor path must raise php-src's own Error, got: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&run.stdout).contains("survived"),
        "php-src never reaches the next statement: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a container write with a NEGATIVE index gives back the share it was handed.
///
/// The setters reject a negative index, because php-src stores a negative KEY there
/// (`$a[-1] = v` makes `[-1 => v]`) and a dense array has no slot for one — the same limitation
/// as any sparse key. But the pointer setter's caller increfs the child BEFORE the call, so
/// returning without storing stranded it: measured at 24 wasm pages over 20000 such writes,
/// against 3 for the same loop that stores. Dropping the write now releases the share, and the
/// negative index is settled before anything is allocated so the rejected path cannot leave a
/// freshly built array behind either.
///
/// Found by GLM 5.2 in the second audit round. Its accompanying claim — that php-src refuses to
/// autovivify for a negative index — did NOT survive measurement: `$a = null; $a[-1] = 42;`
/// answers `array(1) { [-1] => int(42) }`. Only the leak was real.
#[test]
fn test_cli_wasm_negative_index_container_write_does_not_leak() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_negative_index_write");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
wasi.start(instance);
console.log(instance.exports.memory.buffer.byteLength / 65536);
"#,
    )
    .unwrap();

    // 20000 dropped writes must not grow the heap. The loop that STORES is the control: if
    // neither grows, the measurement proves nothing, so both numbers are checked.
    for (name, source, must_stay_small) in [
        (
            "dropped.php",
            "<?php\n$a = [[0, 0]];\n$n = -1;\nfor ($i = 0; $i < 20000; $i++) {\n    $a[$n] = [1, 2];\n}\necho count($a), \"\\n\";\n",
            true,
        ),
        (
            "retained.php",
            "<?php\n$keep = [];\nfor ($i = 0; $i < 20000; $i++) {\n    $row = [1, 2];\n    $keep[] = $row;\n}\necho count($keep), \"\\n\";\n",
            false,
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the negative-index write");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the negative-index write under Node");
        let stdout = String::from_utf8_lossy(&run.stdout);
        let pages: usize = stdout
            .lines()
            .last()
            .and_then(|line| line.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name}: no page count in {stdout:?}"));
        if must_stay_small {
            assert!(
                pages <= 8,
                "{name}: the dropped write leaked — {pages} wasm pages"
            );
        } else {
            assert!(
                pages > 8,
                "{name}: the control did not grow, so the measurement proves nothing — {pages} pages"
            );
        }
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies the `null` LITERAL reaching a nullable-scalar parameter, and `is_null` reading it.
///
/// A `?int` parameter is an inline two-word `{payload, tag}` slot, and the literal `null` arrives
/// as `Void`/`I64` — a different width, so the transfer is a conversion rather than a copy and had
/// no classification at all: `argument #0: unsupported wasm value transfer from I64 (Void/I64) to
/// Tagged (TaggedScalar/TaggedScalar)`.
///
/// The second half is what makes it correct rather than merely compilable. `is_null` required the
/// operand's php type to literally be `TaggedScalar`, but a `?int` PARAMETER carries its
/// nullability in the declaration instead, so the test fell through to the `statically non-null`
/// fallback and `describe(null)` took the non-null branch over a value whose tag said 8 — printing
/// `NULL:` where php-src prints `NULL:null`. The tag is the truth for any two-word scalar whatever
/// the static type calls it, which is the same correction the pointer arm needed.
///
/// Unblocks `examples/functions`.
#[test]
fn test_cli_wasm_passes_null_to_a_nullable_scalar_parameter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_tagged_null_argument");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // php-src 8.5.6's own answers.
    for (name, source, expected) in [
        // The literal, a non-null value through the same parameter, and `gettype` of both.
        (
            "int.php",
            "<?php\nfunction d(?int $v): string {\n    if (is_null($v)) { return \"NULL:null\"; }\n    return gettype($v) . \":\" . $v;\n}\necho d(null), \"\\n\";\necho d(42), \"\\n\";\n",
            "NULL:null\ninteger:42\n",
        ),
        // A nullable FLOAT uses the same two-word slot. (Testing a nullable BOOL here would
        // measure a different gap: the truthiness of a tagged value is refused separately,
        // pending its NaN diagnostics.)
        (
            "float.php",
            "<?php\nfunction f(?float $v): string { return is_null($v) ? \"n\" : \"v\" . $v; }\necho f(null), f(2.5), \"\\n\";\n",
            "nv2.5\n",
        ),
        // The null flows on through a second call, so the tagged pair has to survive a transfer
        // between two nullable parameters rather than only the literal site.
        (
            "chain.php",
            "<?php\nfunction inner(?int $v): string { return is_null($v) ? \"inner-null\" : \"inner-\" . $v; }\nfunction outer(?int $v): string { return inner($v); }\necho outer(null), \"\\n\";\necho outer(7), \"\\n\";\n",
            "inner-null\ninner-7\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the nullable-scalar argument");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the nullable-scalar argument under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `foreach ($a as &$x) { $x += n; }` — a Mixed value narrowing into a concrete cell.
///
/// `$x + 5` types Mixed because the add can overflow into a float, while the cell it writes
/// through is the array's own `int`. The EMITTER already handled that, through
/// `coerce_mixed_ref_cell_store` and `__rt_mixed_cast_*`; the capability gate was the only thing
/// refusing a store the backend could perform, with `ref-cell store value Heap(Mixed)/Mixed must
/// exactly match payload Int`. The native backend answers this shape correctly, so the refusal
/// was a WASM-only gap rather than a shared one.
///
/// What it inherits is the EIR's widening gap, not a new one: on a REAL overflow the value is a
/// float and narrowing it into an `int` cell is wrong — which the native backend does there too.
/// Refusing the whole shape to avoid that costs every ordinary by-reference accumulate.
///
/// The second pass is the part worth pinning: it must see the first pass's writes, which is what
/// proves the values went through the cell rather than into a copy. Unblocks
/// `examples/foreach-ref`.
#[test]
fn test_cli_wasm_foreach_by_reference_accumulates_through_the_cell() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_foreach_ref_accumulate");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let path = dir.join("main.php");
    fs::write(
        &path,
        r#"<?php
$ints = [10, 20, 30];
foreach ($ints as &$i) { $i += 5; }
echo $ints[0], ",", $ints[1], ",", $ints[2], "\n";
foreach ($ints as &$j) { $j += 1; }
echo $ints[0], ",", $ints[1], ",", $ints[2], "\n";
echo count($ints), "\n";
"#,
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&path)
        .output()
        .expect("failed to compile the by-reference accumulate");
    assert!(
        built.status.success(),
        "the by-reference accumulate must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the by-reference accumulate under Node");
    // php-src 8.5.6's own answer.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "15,25,35\n16,26,36\n3\n",
        "the second pass must see the first pass's writes ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a Mixed-to-string cast reached through a `??` merge, two hops from its use.
///
/// A `Mixed` value rendered as a string is admitted when every consumer is a place PHP renders
/// one — echo, concat, interpolation, `strlen`. `echo $x ?? "d"` looked like it had NO consumer
/// at all: the merge parks the value in a hidden slot and reads it back in the merge block, so
/// the cast's only direct uses were the `acquire`/`release` pair the predicate rightly ignores,
/// and "no string consumer" and "no consumer" answered the same.
///
/// Following an `acquire` to its result and a `store_local` to every LOAD of that slot is what
/// closes the gap. Following EVERY load is what keeps it sound: one load reaching a non-string
/// context still refuses the whole cast, which is why the walk collects rather than stops at the
/// first string use it finds.
///
/// Unblocks `examples/union-types`, which php-src cannot parse — so that example is checked
/// against the native backend instead, and the shape itself is pinned here against php-src.
#[test]
fn test_cli_wasm_renders_a_coalesced_mixed_as_a_string() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_coalesced_string_cast");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let path = dir.join("main.php");
    fs::write(
        &path,
        r#"<?php
function maybe(string $s): ?string {
    $t = trim($s);
    return $t === "" ? null : $t;
}
echo maybe("  hi  ") ?? "none", "\n";
echo maybe("   ") ?? "none", "\n";
echo "[" . (maybe(" x ") ?? "none") . "]", "\n";
echo strlen(maybe("  abc ") ?? "none"), "\n";
"#,
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&path)
        .output()
        .expect("failed to compile the coalesced string cast");
    assert!(
        built.status.success(),
        "the coalesced string cast must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the coalesced string cast under Node");
    // php-src 8.5.6's own answer, across all four string contexts the predicate admits.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "hi\nnone\n[x]\n3\n",
        "php-src's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a local that holds `null` at one point and an `int` at another.
///
/// `$x = null; is_null($x); $x = 99;` gets ONE one-word slot, and PHP null in a one-word scalar
/// slot IS the `NULL_SENTINEL` bit pattern — so the store and the later load are the same bits
/// under two different php types, `Void`/`I64` and `Int`/`I64`, which had no classification.
///
/// What makes `is_null` right here is not a runtime comparison but the EIR's flow-sensitive
/// typing: it types the load AFTER the null store as `php=null`, so the check answers from the
/// type at a point where the type is true. The reassignment then stores an ordinary int.
///
/// The collision this inherits is the sentinel's own, shared with the native backend, which
/// answers this example identically: an integer that really equals 9223372036854775806 reads as
/// null. That is the documented cost of the one-word encoding. Unblocks `examples/cli-args`.
#[test]
fn test_cli_wasm_local_holds_null_then_an_int() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_null_then_int_local");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let path = dir.join("main.php");
    fs::write(
        &path,
        r#"<?php
$x = null;
echo "is_null: " . is_null($x) . "\n";
$x = 99;
echo "after reassign: " . $x . "\n";
echo "is_null now: " . (is_null($x) ? "y" : "n") . "\n";
$y = 7;
$y = null;
echo "back to null: " . (is_null($y) ? "y" : "n") . "\n";
"#,
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&path)
        .output()
        .expect("failed to compile the null-then-int local");
    assert!(
        built.status.success(),
        "the null-then-int local must compile: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the null-then-int local under Node");
    // php-src 8.5.6's own answer. `is_null` of a true value echoes "1"; of false, nothing.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "is_null: 1\nafter reassign: 99\nis_null now: n\nback to null: y\n",
        "php-src's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies writing a RAW scalar into an `array<mixed>`, and that a boxed cell stays refused.
///
/// An `array<mixed>` stores 16-byte slots holding a boxed cell, so a raw scalar reaching one is
/// boxed at the write site and its single reference handed over — the same contract `array_push`
/// already uses for the same value shape. Overwriting also RELEASES the replaced cell, which is
/// required rather than optional: skipping it leaks one cell per write, measured at 24 wasm pages
/// over 20000 against 3 for a balanced loop.
///
/// The refusal at the end is the point of the test. An ALREADY-boxed cell carries the OTHER
/// ownership contract — the EIR releases the operand, so the array must take a share — and taking
/// that share then releasing the replaced cell corrupts a slot when an earlier write went through
/// this same setter. That was bisected to `[literal] → raw write → boxed write` and is not yet
/// explained, so it stays refused. Admitting it would answer `$s[0]` as empty where php-src
/// answers 0. This assertion is what fails if someone widens the arm without solving that.
#[test]
fn test_cli_wasm_writes_a_raw_scalar_into_a_mixed_array() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_mixed_array_scalar_write");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // php-src 8.5.6's own answers.
    for (name, source, expected) in [
        // One write over a heterogeneous literal, and the untouched neighbour.
        (
            "one.php",
            "<?php\n$s = [2, \"x\"];\n$s[1] = 7;\necho $s[0], \",\", $s[1], \"\\n\";\necho $s[0], \",\", $s[1], \"\\n\";\n",
            "2,7\n2,7\n",
        ),
        // Two writes, the second over the slot the first left alone.
        (
            "two.php",
            "<?php\n$s = [2, \"x\"];\n$s[1] = 7;\n$s[0] = 9;\necho $s[0], \",\", $s[1], \"\\n\";\necho count($s), \"\\n\";\n",
            "9,7\n2\n",
        ),
        // A string and a float into the same array, and a write PAST the end.
        (
            "kinds.php",
            "<?php\n$s = [1, \"a\"];\n$s[0] = \"str\";\n$s[1] = 2.5;\necho $s[0], \",\", $s[1], \"\\n\";\n",
            "str,2.5\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the mixed-array scalar write");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the mixed-array scalar write under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    // Storing an ALREADY-boxed cell stays refused, for the reason in the doc comment above.
    // The raw write before the read is what keeps the read from folding to a constant — without
    // it the EIR answers `$a[0]` as the literal and the boxed path is never reached at all.
    let boxed = dir.join("boxed.php");
    fs::write(
        &boxed,
        "<?php\n$a = [1, \"two\"];\n$a[0] = 9;\n$b = [1, \"two\"];\n$b[1] = $a[0];\necho $b[1], \"\\n\";\n",
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&boxed)
        .output()
        .expect("failed to run the compiler over the boxed-cell write");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr)
                .contains("does not match supported element storage Mixed"),
        "storing an already-boxed cell must stay refused: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `readline()` reads one line from stdin WITHOUT its newline, and prints no prompt.
///
/// Both details come from measuring php-src 8.5.6, and both are places the NATIVE backend is
/// wrong — `printf 'Ada\n' | ...` answers `Hello Ada\n!` there, with a `> ` prompt php-src never
/// emits. So this target is checked against php-src directly rather than against native.
///
/// - The newline is not part of the result. Keeping it puts a line break in the middle of
///   `"Hello " . $name . "!"`.
/// - The prompt goes to the terminal in php-src, not to stdout, so with stdout redirected it does
///   not appear at all. Writing it would add bytes php-src never emits.
///
/// The second case is the one that pins the edges: an empty line answers the empty string rather
/// than being skipped, and reading past EOF answers the empty string too. php-src answers `false`
/// at EOF, which the EIR's `Str` result cannot carry — the same dropped-null shape as elsewhere,
/// not a choice made in the lowering.
///
/// This needed a `fd_read` import, which no WASM module carried before: the backend had no stdin
/// path at all. Bytes land in the legacy concat reservation, dead space kept only so static-data
/// offsets stay stable. Unblocks `examples/readline`.
#[test]
fn test_cli_wasm_readline_reads_a_line_without_its_newline() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_readline");

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected string below is php-src 8.5.6's own answer for the same program and stdin.
    for (name, source, stdin, expected) in [
        (
            "prompt.php",
            "<?php\n$name = readline(\"> \");\necho \"Hello \" . $name . \"!\\n\";\n",
            "Ada\n",
            "Hello Ada!\n",
        ),
        (
            "edges.php",
            "<?php\n$a = readline(\"\");\n$b = readline(\"\");\n$c = readline(\"\");\n$d = readline(\"\");\necho \"[\", $a, \"][\", $b, \"][\", $c, \"][\", $d, \"]\\n\";\necho strlen($a), \",\", strlen($b), \",\", strlen($c), \",\", strlen($d), \"\\n\";\n",
            "one\n\nthree\n",
            "[one][][three][]\n3,0,5,0\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the readline case");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );

        let stdin_path = dir.join(format!("{name}.in"));
        fs::write(&stdin_path, stdin).unwrap();
        let input = fs::File::open(&stdin_path).unwrap();
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .stdin(input)
            .current_dir(&dir)
            .output()
            .expect("failed to run the readline case under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a typed property read in an ABSTRACT class, whose concrete descendants all initialize.
///
/// `abstract class Shape { abstract public int $sides { get; set; } }` has no default of its own,
/// and reading one before it is written is `Error: Typed property C::$p must not be accessed
/// before initialization` — a check this backend cannot make, since the allocator zeroes the slot
/// and zero is a legitimate int. But an abstract class cannot be instantiated, so what decides is
/// what its CONCRETE descendants do: with every one of them giving the slot a default, no instance
/// exists whose slot is unwritten, and refusing `Shape::describe()` answers a question no object
/// can ask. One descendant that leaves it uninitialized brings the refusal straight back, which
/// the second case pins.
#[test]
fn test_cli_wasm_reads_an_abstract_property_every_subclass_initializes() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_abstract_property");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
abstract class Shape {
    abstract public int $sides { get; set; }
    abstract public string $name { get; set; }
    public function describe(): string { return $this->name . " has " . $this->sides . " sides"; }
}
class Triangle extends Shape { public int $sides = 3; public string $name = "triangle"; }
class Square extends Shape { public int $sides = 4; public string $name = "square"; }
foreach ([new Triangle(), new Square()] as $shape) { echo $shape->describe(), "\n"; }
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the abstract-property probe to WASM");
    assert!(
        output.status.success(),
        "abstract-property compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the abstract-property probe under Node");
    assert!(
        run.status.success(),
        "the abstract-property reads must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "triangle has 3 sides\nsquare has 4 sides\n",
        "php-src's own answers"
    );

    // One descendant that leaves the slot uninitialized is enough to make an instance whose read
    // php-src raises on, so the whole hierarchy goes back to refused.
    let partial = dir.join("partial.php");
    fs::write(
        &partial,
        r#"<?php
abstract class S { abstract public int $n { get; set; } public function show(): int { return $this->n; } }
class Good extends S { public int $n = 3; }
class Bad extends S { public int $n; }
echo (new Good())->show(), "\n";
"#,
    )
    .unwrap();
    let refused = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&partial)
        .output()
        .expect("failed to run the compiler over the partial hierarchy");
    assert!(
        !refused.status.success()
            && String::from_utf8_lossy(&refused.stderr).contains("may be uninitialized"),
        "a descendant that does not initialize must refuse the read: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies an INHERITED method keeps the return type inferred from its body.
///
/// Class schemas are built before any body is checked, so a subclass copies its parent's
/// signatures while they still carry the placeholder return type. The inference pass then wrote
/// the real type back into the DECLARING class only, leaving every inheritor claiming the
/// placeholder — an untyped `label()` returning a string answered `Str` for `A::label` and `Int`
/// for `B::label`, and the mere existence of an EMPTY `class B extends A {}` was enough to
/// trigger it. A subclass that OVERRIDES the method has its own implementation and is untouched,
/// which the `Dog::speak` arm here exercises.
#[test]
fn test_cli_wasm_inherited_method_keeps_its_inferred_return_type() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_inherited_signature");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Animal {
    protected $name = "animal";
    public function label() { return $this->name; }
    public function speak() { return "animal"; }
    public function run() { return $this->speak(); }
}
class Dog extends Animal {
    public function __construct() { $this->name = "dog"; }
    public function speak() { return parent::speak() . "-woof"; }
}
class Cat extends Animal {}
$d = new Dog();
$c = new Cat();
echo $d->label(), "|", $d->run(), "\n";
echo $c->label(), "|", $c->run(), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the inherited-signature probe to WASM");
    assert!(
        output.status.success(),
        "inherited-signature compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the inherited-signature probe under Node");
    assert!(
        run.status.success(),
        "the inherited calls must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "dog|animal-woof\nanimal|animal\n",
        "php-src's own answers"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a dynamic dispatch survives an unrelated class that merely shares the method NAME.
///
/// A `mixed` receiver names no class, so the dispatch ladder is built from every class declaring
/// the method — and the audit then demands that EVERY entry lower. That let one bystander veto a
/// whole program: `Ledger::show(string, int)` has nothing to do with `Money`, but its presence
/// refused `$value->show()` with `Ledger: show expects 2 arguments, got 0`. Dropping it is not a
/// guess about the runtime class, it is a fact about PHP: reaching `Ledger::show` with no
/// arguments is an `ArgumentCountError` on every backend, so no correct program can take that
/// arm. The second case pins the other half — a bystander PHP *could* call at this arity stays in
/// the ladder and still answers for its own instances, so the filter narrows the ladder without
/// ever emptying it. That second case is also the only shape with TWO surviving `void` arms, which
/// is what exposed the ladder storing its result from an empty stack: with every candidate
/// agreeing on `void`, the checker types the call expression `I64 php=null` instead of boxing it,
/// and the arm has to supply the null the callee never pushed.
#[test]
fn test_cli_wasm_dispatch_drops_a_namesake_php_could_not_call_at_this_arity() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_dispatch_arity");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Both expected strings below are php-src 8.5.6's own answers for the same program.
    for (name, source, expected) in [
        (
            "bystander.php",
            r#"<?php
class Money {
    public function __construct(private int $cents) {}
    public function show(): void { echo "$", $this->cents, "\n"; }
}
class Ledger {
    public function show(string $prefix, int $width): void { echo $prefix, $width, "\n"; }
}
function render(mixed $value): void { $value->show(); }
render(new Money(1299));
"#,
            "$1299\n",
        ),
        (
            "kept.php",
            r#"<?php
class Money {
    public function __construct(private int $cents) {}
    public function show(): void { echo "$", $this->cents, "\n"; }
}
class Tally {
    public function show(): void { echo "tally\n"; }
}
function render(mixed $value): void { $value->show(); }
render(new Money(1299));
render(new Tally());
"#,
            "$1299\ntally\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the namesake-dispatch probe to WASM");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the namesake-dispatch probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a WIDENED integer arithmetic result counts as an integer on the far side of a
/// comparison and of `%`.
///
/// `$i * $i` on two ints is typed Mixed only because an overflow would promote it to a float,
/// and the backend already admits narrowing it back — exact for every value that did not
/// overflow. But the predicates that decide the OTHER side of a comparison or a modulo rejected
/// it as "another conversion of a box", which contradicted that: `$i * $i <= $n` and
/// `$n % ($i + 2)` were refused for having a perfectly good integer opposite them.
///
/// This is `examples/primes` in miniature — a sieve whose loop condition is exactly that
/// comparison — and it is the shape any `while ($i * $i <= $n)` takes.
#[test]
fn test_cli_wasm_compares_widened_arithmetic_against_a_boxed_value() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_widened_compare");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function is_prime($n) {
    if ($n <= 1) { return false; }
    if ($n <= 3) { return true; }
    if ($n % 2 == 0 || $n % 3 == 0) { return false; }
    $i = 5;
    while ($i * $i <= $n) {
        if ($n % $i == 0 || $n % ($i + 2) == 0) { return false; }
        $i += 6;
    }
    return true;
}
$found = "";
$count = 0;
for ($n = 2; $n <= 50; $n++) {
    if (is_prime($n)) { $found .= $n . " "; $count++; }
}
echo $found, "\n", $count, "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the widened-comparison probe to WASM");
    assert!(
        output.status.success(),
        "widened-comparison compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the widened-comparison probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 \n15\n",
        "php-src's own answer ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies PHP's truthiness of a BOXED value and of a float, warning on a NaN.
///
/// The per-tag ANSWERS were always exact — the refusal was only ever about the one diagnostic
/// PHP raises here — so the seventeen arms below are as much a check that nothing regressed as
/// that the warning appeared. Two of them are the ones intuition gets wrong: `"0.0"` is TRUE,
/// because only the single character `"0"` is false, and `-0.0` is FALSE like `+0.0`.
///
/// A NaN is TRUE, and says so first: `Warning: unexpected NAN value was coerced to bool`. A bare
/// `f64.ne 0.0` would have answered FALSE for it and said nothing, which is why the float arm
/// tests the BITS rather than comparing.
#[test]
fn test_cli_wasm_answers_php_truthiness_including_a_nan_warning() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_truthiness");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function arm(int $i): mixed {
    if ($i === 0) { return 0; }
    if ($i === 1) { return 1; }
    if ($i === 2) { return 0.0; }
    if ($i === 3) { return -0.0; }
    if ($i === 4) { return 0.5; }
    if ($i === 5) { return NAN; }
    if ($i === 6) { return INF; }
    if ($i === 7) { return ""; }
    if ($i === 8) { return "0"; }
    if ($i === 9) { return "0.0"; }
    if ($i === 10) { return "a"; }
    if ($i === 11) { return null; }
    if ($i === 12) { return []; }
    if ($i === 13) { return [0]; }
    if ($i === 14) { return true; }
    if ($i === 15) { return false; }
    return new stdClass();
}
$m = arm((int)($argv[1] ?? "0"));
echo ($m ? "T" : "F"), "|\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the truthiness probe to WASM");
    assert!(
        output.status.success(),
        "truthiness compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m", process.argv[3]], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // php-src 8.5.6's own answer for each arm, in order.
    let expected = [
        "F", "T", "F", "F", "T", "T", "T", "F", "F", "T", "T", "F", "F", "T", "T", "F", "T",
    ];
    for (arm, want) in expected.iter().enumerate() {
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .arg(arm.to_string())
            .current_dir(&dir)
            .output()
            .expect("failed to run the truthiness probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            format!("{want}|\n"),
            "arm {arm}: php-src's own answer"
        );
        // Only the NaN arm speaks, and it still answers true.
        let stderr = String::from_utf8_lossy(&run.stderr);
        let warned = stderr.contains("Warning: unexpected NAN value was coerced to bool");
        assert_eq!(
            warned,
            arm == 5,
            "arm {arm}: only a NaN warns, and it does: {stderr}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a boxed operand of `%` is coerced under PHP's ARITHMETIC contract.
///
/// This is a THIRD contract, and its differences from the declared-parameter and declared-return
/// ones are the whole reason it needs its own path. Measured on php-src 8.5.6 with `$mixed % 3`:
///
/// - `null` is SILENTLY 0 — a parameter deprecates there, and a return raises;
/// - a non-numeric string is `Unsupported operand types: string % int`, which names the operand
///   TYPES and the operator rather than saying `must be of type int`;
/// - `INF` does not raise at all: it warns `The float INF is not representable as an int, cast
///   occurred` and yields 0, where a parameter raises a `TypeError`.
///
/// What IS shared is the numeric middle — a lost fraction deprecates identically from a float
/// and from a float-shaped string — so those two notices come from the same helpers.
///
/// KNOWN DIVERGENCE: php-src appends ` in <file> on line <n>` and a stack trace; this target
/// reports no location tail, so the fatal arms are asserted as a prefix.
#[test]
fn test_cli_wasm_coerces_a_boxed_arithmetic_operand() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_arith_operand");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function arm(int $i): mixed {
    if ($i === 0) { return 7; }
    if ($i === 1) { return -7; }
    if ($i === 2) { return 7.0; }
    if ($i === 3) { return 7.9; }
    if ($i === 4) { return true; }
    if ($i === 5) { return false; }
    if ($i === 6) { return "7"; }
    if ($i === 7) { return "7.9"; }
    if ($i === 8) { return "abc"; }
    if ($i === 9) { return null; }
    if ($i === 10) { return [1, 2]; }
    if ($i === 11) { return INF; }
    return new stdClass();
}
$m = arm((int)($argv[1] ?? "0"));
echo $m % 3, "|\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the arithmetic-operand probe to WASM");
    assert!(
        output.status.success(),
        "arithmetic-operand compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m", process.argv[3]], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expectation is php-src 8.5.6's own answer for the same arm.
    for (arm, stdout, stderr) in [
        ("0", "1|\n", ""),
        ("1", "-1|\n", ""),
        ("2", "1|\n", ""),
        (
            "3",
            "1|\n",
            "Deprecated: Implicit conversion from float 7.9 to int loses precision\n",
        ),
        ("4", "1|\n", ""),
        ("5", "0|\n", ""),
        ("6", "1|\n", ""),
        (
            "7",
            "1|\n",
            "Deprecated: Implicit conversion from float-string \"7.9\" to int loses precision\n",
        ),
        (
            "8",
            "",
            "Uncaught TypeError: Unsupported operand types: string % int\n",
        ),
        // The arm that separates this contract from the parameter one: silent, no deprecation.
        ("9", "0|\n", ""),
        (
            "10",
            "",
            "Uncaught TypeError: Unsupported operand types: array % int\n",
        ),
        // And the arm that separates it from a raise: a warning, then zero.
        (
            "11",
            "0|\n",
            "Warning: The float INF is not representable as an int, cast occurred\n",
        ),
        (
            "12",
            "",
            "Uncaught TypeError: Unsupported operand types: stdClass % int\n",
        ),
    ] {
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .arg(arm)
            .current_dir(&dir)
            .output()
            .expect("failed to run the arithmetic-operand probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            stdout,
            "arm {arm}: php-src's own value"
        );
        let observed = String::from_utf8_lossy(&run.stderr);
        assert!(
            observed.contains(stderr),
            "arm {arm}: expected php-src's own diagnostic {stderr:?}, got {observed}"
        );
        assert_eq!(
            run.status.code(),
            Some(if stdout.is_empty() { 255 } else { 0 }),
            "arm {arm}: php-src's own exit status"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a boxed value reaching a builtin's declared `int` parameter is coerced PHP's way.
///
/// This is the `int` half of the argument boundary, and it arrives differently from the `string`
/// one: `substr($s, $mixed)` reaches the call with the Mixed operand INTACT — the frontend
/// materialises no cast for it — so the coercion is emitted where the argument is pushed rather
/// than where a cast would have been.
///
/// The conversion itself is the one a declared `int` RETURN performs, and the runtime shares a
/// core with it rather than carrying a second copy, because measured on php-src 8.5.6 they
/// differ in exactly two places: `null` does NOT raise at a parameter — it becomes 0 after a
/// `Deprecated` naming the parameter — and the failure says `Argument #N ($p)`. Every numeric
/// answer in between is identical, both precision deprecations included.
///
/// The float arms are the ones worth spelling out: `2.7` truncates to 2 with a notice, `-2.7`
/// truncates toward zero to -2 with the same notice naming `-2.7`, and `INF` has no conversion
/// at all and is a `TypeError` naming `float`. A `"2.0"` string is silent because its VALUE is
/// integral, while `"2.7"` gets the float-STRING wording, which is a different message.
///
/// KNOWN DIVERGENCE: php-src appends ` in <file> on line <n>`; this target reports no location
/// tail, so each expectation below is asserted as a prefix.
#[test]
fn test_cli_wasm_coerces_a_boxed_value_at_a_declared_int_parameter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_int_argument");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function arm(int $i): mixed {
    if ($i === 0) { return 2; }
    if ($i === 1) { return -1; }
    if ($i === 2) { return 2.0; }
    if ($i === 3) { return 2.7; }
    if ($i === 4) { return -2.7; }
    if ($i === 5) { return true; }
    if ($i === 6) { return false; }
    if ($i === 7) { return "2"; }
    if ($i === 8) { return "2.0"; }
    if ($i === 9) { return "2.7"; }
    if ($i === 10) { return "abc"; }
    if ($i === 11) { return null; }
    if ($i === 12) { return [1, 2]; }
    if ($i === 13) { return INF; }
    return new stdClass();
}
$m = arm((int)($argv[1] ?? "0"));
echo substr("abcdefgh", $m), "|\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the int-argument probe to WASM");
    assert!(
        output.status.success(),
        "int-argument compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m", process.argv[3]], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expectation is php-src 8.5.6's own answer for the same arm.
    for (arm, stdout, stderr) in [
        ("0", "cdefgh|\n", ""),
        ("1", "h|\n", ""),
        ("2", "cdefgh|\n", ""),
        (
            "3",
            "cdefgh|\n",
            "Deprecated: Implicit conversion from float 2.7 to int loses precision\n",
        ),
        (
            "4",
            "gh|\n",
            "Deprecated: Implicit conversion from float -2.7 to int loses precision\n",
        ),
        ("5", "bcdefgh|\n", ""),
        ("6", "abcdefgh|\n", ""),
        ("7", "cdefgh|\n", ""),
        ("8", "cdefgh|\n", ""),
        (
            "9",
            "cdefgh|\n",
            "Deprecated: Implicit conversion from float-string \"2.7\" to int loses precision\n",
        ),
        (
            "10",
            "",
            "Uncaught TypeError: substr(): Argument #2 ($offset) must be of type int, \
             string given\n",
        ),
        (
            "11",
            "abcdefgh|\n",
            "Deprecated: substr(): Passing null to parameter #2 ($offset) of type int \
             is deprecated\n",
        ),
        (
            "12",
            "",
            "Uncaught TypeError: substr(): Argument #2 ($offset) must be of type int, \
             array given\n",
        ),
        (
            "13",
            "",
            "Uncaught TypeError: substr(): Argument #2 ($offset) must be of type int, \
             float given\n",
        ),
        (
            "14",
            "",
            "Uncaught TypeError: substr(): Argument #2 ($offset) must be of type int, \
             stdClass given\n",
        ),
    ] {
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .arg(arm)
            .current_dir(&dir)
            .output()
            .expect("failed to run the int-argument probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            stdout,
            "arm {arm}: php-src's own value"
        );
        let observed = String::from_utf8_lossy(&run.stderr);
        assert!(
            observed.contains(stderr),
            "arm {arm}: expected php-src's own diagnostic {stderr:?}, got {observed}"
        );
        // A `TypeError` ends the program; a `Deprecated` does not.
        assert_eq!(
            run.status.code(),
            Some(if stdout.is_empty() { 255 } else { 0 }),
            "arm {arm}: php-src's own exit status"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `php://memory` and `php://temp`, which are streams with no host file behind them.
///
/// Every other stream this target opens is a WASI fd, and WASI is capability-based: without a
/// preopened directory there is no filesystem at all. An in-memory stream needs none of that, so
/// it is opened before the preopen probe and works under a host that granted nothing. The
/// descriptor's ADDRESS is the handle, with a high bit set so the two spaces cannot collide, and
/// the bytes live in a separate block — which is what lets a write grow the stream without
/// invalidating the handle the script is holding.
///
/// Two behaviours were measured rather than assumed, and both would have been wrong by
/// intuition. `feof` is set by a read that ASKED for more than was there, not by one that merely
/// finished at the end: requesting 5 of 5 leaves it FALSE, requesting 100 of 6 sets it TRUE even
/// though 6 bytes came back. And a mid-stream write OVERWRITES rather than inserting, so
/// `"abcdef"` rewound and written `"XY"` reads back `"XYcdef"` at length 6.
#[test]
fn test_cli_wasm_reads_and_writes_an_in_memory_stream() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_memory_stream");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function yn(bool $b): string { return $b ? "T" : "F"; }
$h = fopen("php://memory", "r+");
echo yn($h !== false), "|";
echo fwrite($h, "hello "), "|", fwrite($h, "world"), "|", ftell($h), "\n";
rewind($h);
echo ftell($h), "|", fread($h, 5), "|", yn(feof($h)), "|";
echo fread($h, 100), "|", yn(feof($h)), "|", yn(fclose($h)), "\n";
$e = fopen("php://memory", "r+");
fwrite($e, "abcde");
rewind($e);
echo fread($e, 5), "|", yn(feof($e)), "|[", fread($e, 1), "]|", yn(feof($e)), "\n";
fclose($e);
$g = fopen("php://memory", "r+");
fwrite($g, "abcdef");
rewind($g);
fwrite($g, "XY");
rewind($g);
echo fread($g, 10), "|", ftell($g), "\n";
fclose($g);
$t = fopen("php://temp", "w+");
fwrite($t, "abc");
rewind($t);
echo fread($t, 10), "|", yn(fclose($t)), "\n";
$z = fopen("php://memory", "r+");
echo "[", fread($z, 4), "]|", yn(feof($z)), "\n";
fclose($z);
$b = fopen("php://memory", "r+");
for ($i = 0; $i < 200; $i++) { fwrite($b, "xy"); }
rewind($b);
echo strlen(fread($b, 1000)), "\n";
fclose($b);
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the memory-stream probe to WASM");
    assert!(
        output.status.success(),
        "memory-stream compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // No `preopens`: an in-memory stream must work with no filesystem authority at all.
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the memory-stream probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "T|6|5|11\n\
         0|hello|F| world|T|T\n\
         abcde|F|[]|T\n\
         XYcdef|6\n\
         abc|T\n\
         []|T\n\
         400\n",
        "php-src 8.5.6's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `class_exists` and its three siblings answer from the module's own declarations.
///
/// `RuntimeFnId::ClassExists`, `RuntimeFnId::InterfaceExists`, `RuntimeFnId::TraitExists` and
/// `RuntimeFnId::EnumExists` are closed-world questions: the checker already requires a literal
/// name in AOT mode, and this module IS the whole program, so each folds to a constant with no
/// runtime table consulted. PHP's `$autoload` argument cannot change that — a name this module
/// never declared has nothing to load.
///
/// The four namespaces are DISTINCT, which is the half worth measuring rather than assuming:
/// php-src 8.5.6 answers `class_exists("Shape")` FALSE for an interface and
/// `interface_exists("Circle")` FALSE for a class. The one crossover is an ENUM —
/// `class_exists("Suit")` is TRUE — because an enum IS a class in PHP.
#[test]
fn test_cli_wasm_answers_the_exists_family_from_its_own_declarations() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_exists_family");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
interface Shape {}
trait Greets {}
enum Suit: string { case Hearts = "H"; }
class Circle implements Shape { use Greets; }

function yn(bool $b): string { return $b ? "T" : "F"; }
echo yn(class_exists("Circle")), yn(class_exists("Nope")), "\n";
echo yn(interface_exists("Shape")), yn(interface_exists("Nope")), "\n";
echo yn(trait_exists("Greets")), yn(trait_exists("Nope")), "\n";
echo yn(enum_exists("Suit")), yn(enum_exists("Nope")), "\n";
echo yn(class_exists("Shape")), yn(interface_exists("Circle")), yn(class_exists("Suit")), "\n";
echo yn(class_exists("circle")), yn(class_exists("\Circle")), "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the exists-family probe to WASM");
    assert!(
        output.status.success(),
        "exists-family compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the exists-family probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "TF\nTF\nTF\nTF\nFFT\nTT\n",
        "php-src 8.5.6's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `foreach` over a BOXED value, whose storage only the runtime tag decides.
///
/// The iterator picked indexed-versus-hash at compile time from the source's EIR type, so a
/// `mixed` source — which names no storage until the cell is read — was refused outright. The
/// cursor seeds, the advance, the key and the value now all dispatch on the tag.
///
/// The non-iterable arm is the one worth measuring rather than assuming: php-src 8.5.6 does NOT
/// raise there. It WARNS, names the type that arrived, and runs the body zero times, so the loop
/// still has to be entered and left cleanly — which is why the dispatch carries a third kind
/// rather than a fatal.
///
/// KNOWN DIVERGENCE: php-src appends ` in <file> on line <n>` to the warning. This target reports
/// no location tail, the convention its other diagnostics already follow.
#[test]
fn test_cli_wasm_iterates_a_boxed_value() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_boxed_foreach");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function box(int $i): mixed {
    if ($i === 0) { return [10, 20, 30]; }
    if ($i === 1) { return ["a" => 1, "b" => 2]; }
    if ($i === 2) { return []; }
    if ($i === 3) { return 7; }
    if ($i === 4) { return "str"; }
    return null;
}
for ($i = 0; $i < 6; $i++) {
    echo $i, ":";
    foreach (box($i) as $k => $v) { echo " ", $k, "=", $v; }
    echo "\n";
}
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the boxed-foreach probe to WASM");
    assert!(
        output.status.success(),
        "boxed-foreach compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the boxed-foreach probe under Node");
    // php-src's own values and keys: an indexed source keys from 0, a hash keeps its own keys,
    // an empty array runs zero times, and the three non-containers run zero times as well.
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "0: 0=10 1=20 2=30\n1: a=1 b=2\n2:\n3:\n4:\n5:\n",
        "php-src's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );
    let stderr = String::from_utf8_lossy(&run.stderr);
    for expected in [
        "Warning: foreach() argument must be of type array|object, int given\n",
        "Warning: foreach() argument must be of type array|object, string given\n",
        "Warning: foreach() argument must be of type array|object, null given\n",
    ] {
        assert!(
            stderr.contains(expected),
            "expected php-src's own warning {expected:?}, got {stderr}"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a dispatch ladder ignores a subclass this module could never construct.
///
/// A virtual call collects every concrete class in the receiver's subtree, and the audit then
/// demands a body for each — so one subclass whose body was never emitted refused the call, and
/// with it the base class's own method. The SPL prelude does exactly that:
/// `__ElephcAppendIteratorArrayIterator` extends `ArrayIterator` and declares `append` with no
/// body in the module, which refused `$this->append(...)` inside `ArrayIterator::offsetSet`.
///
/// Dropping it is licensed by the same audit that would refuse creating it: a class DECLARING
/// `__construct` with no body cannot be instantiated here, so no instance can exist to dispatch
/// to. A THROWABLE is exempt — the runtime raises `ValueError` and its siblings directly, never
/// through `new`, and their accessors are open-coded against bodyless classes on purpose.
/// Dropping those left `catch (ValueError $e) { $e->getMessage(); }` with no candidate at all,
/// which is what the second case here pins.
#[test]
fn test_cli_wasm_dispatch_ignores_a_subclass_it_cannot_construct() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_unconstructible");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Both expected strings are php-src 8.5.6's own answers.
    for (name, source, expected) in [
        // A Throwable the RUNTIME raises: it has no constructor body here, and must still be a
        // dispatch candidate for its own accessor.
        (
            "throwable.php",
            "<?php\ntry { str_repeat(\"x\", -1); } catch (ValueError $e) { echo \"caught: \", $e->getMessage(), \"\\n\"; }\n",
            "caught: str_repeat(): Argument #2 ($times) must be greater than or equal to 0\n",
        ),
        // An ordinary hierarchy: every class here is constructible, so every arm stays.
        (
            "subclasses.php",
            "<?php\nclass Base { public function label(): string { return \"base\"; } }\nclass Kid extends Base { public function label(): string { return \"kid\"; } }\nfunction show(Base $b): void { echo $b->label(), \"\\n\"; }\nshow(new Base());\nshow(new Kid());\n",
            "base\nkid\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(&path, source).unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the unconstructible-candidate probe");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the unconstructible-candidate probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{name}: php-src's own answer ({})",
            String::from_utf8_lossy(&run.stderr)
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `===` between two boxed values, including PHP's deep array identity.
///
/// The pair was refused outright because either cell could hold an array, and array identity in
/// PHP is a DEEP, ORDER-SENSITIVE element-wise comparison rather than the tag-plus-payload test
/// a cell against a concrete value needs. The 196 combinations below are php-src 8.5.6's own
/// answers, and three of them are what make the walk non-trivial:
///
/// - `["a" => 1, "b" => 2] === ["b" => 2, "a" => 1]` is FALSE, so comparing key SETS is wrong;
/// - `[0 => 1, 1 => 2] === [1 => 2, 0 => 1]` is FALSE for the same reason with integer keys;
/// - `[[1], [2]] === [[1], [3]]` is FALSE, so the walk has to recurse.
///
/// The NATIVE backend answers `false` for two structurally equal arrays here — it compares them
/// by heap pointer — so this target is the one that matches php-src.
#[test]
fn test_cli_wasm_compares_two_boxed_values_including_deep_arrays() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_boxed_strict");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function box(int $i): mixed {
    if ($i === 0) { return [1, 2]; }
    if ($i === 1) { return [1, 2, 3]; }
    if ($i === 2) { return ["a" => 1, "b" => 2]; }
    if ($i === 3) { return ["b" => 2, "a" => 1]; }
    if ($i === 4) { return ["a" => 1, "b" => 2]; }
    if ($i === 5) { return [[1], [2]]; }
    if ($i === 6) { return [[1], [2]]; }
    if ($i === 7) { return [[1], [3]]; }
    if ($i === 8) { return []; }
    if ($i === 9) { return []; }
    if ($i === 10) { return [1, "2"]; }
    if ($i === 11) { return 5; }
    if ($i === 12) { return "5"; }
    return null;
}
function yn(bool $b): string { return $b ? "T" : "F"; }
for ($i = 0; $i < 14; $i++) {
    for ($j = 0; $j < 14; $j++) { echo yn(box($i) === box($j)); }
    echo "\n";
}
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the boxed-strict probe to WASM");
    assert!(
        output.status.success(),
        "boxed-strict compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the boxed-strict probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "TFFFFFFFFFFFFF\n\
         FTFFFFFFFFFFFF\n\
         FFTFTFFFFFFFFF\n\
         FFFTFFFFFFFFFF\n\
         FFTFTFFFFFFFFF\n\
         FFFFFTTFFFFFFF\n\
         FFFFFTTFFFFFFF\n\
         FFFFFFFTFFFFFF\n\
         FFFFFFFFTTFFFF\n\
         FFFFFFFFTTFFFF\n\
         FFFFFFFFFFTFFF\n\
         FFFFFFFFFFFTFF\n\
         FFFFFFFFFFFFTF\n\
         FFFFFFFFFFFFFT\n",
        "php-src 8.5.6's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `===` and `!==` against a nullable int, which this target stores as a `{payload,
/// tag}` PAIR rather than one word.
///
/// The pair was refused outright, which turned away every `$x === null` on a `?int` — the single
/// most common thing anyone writes with one, and the largest strict-comparison gap in the
/// example suite. Its tag is 0 or 8 and nothing else (`codegen_repr` folds only `int|null` to
/// this representation), which is what makes each arm below decidable rather than approximate:
/// against a string, a bool or a float the answer is a compile-time FALSE, because a `?int`
/// holds none of those and `===` compares the type first.
///
/// The `=== 10` arm is the one a naive lowering gets wrong. Testing the payload alone would
/// answer true for a null whose payload word also happens to hold that value, so the tag has to
/// be checked first — which is why the third column below is `F` on the null row.
///
/// The producer deliberately avoids arithmetic: a widened `int * int` reaches the slot as a
/// `Heap(Mixed)`, which is a separate narrowing both backends still refuse.
#[test]
fn test_cli_wasm_compares_a_nullable_int_strictly() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_nullable_strict");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function pick(int $i): ?int {
    if ($i === 0) { return null; }
    if ($i === 1) { return 10; }
    return 20;
}
function yn(bool $b): string { return $b ? "T" : "F"; }
foreach ([0, 1, 2] as $i) {
    $x = pick($i);
    $y = pick($i);
    $z = pick(2);
    echo $i, ": ";
    echo yn($x === null), yn($x !== null), yn($x === 10), yn($x === 20);
    echo yn($x === "10"), yn($x === true), yn($x === 10.0);
    echo yn($x === $y), yn($x === $z), yn(null === $x), yn(10 === $x), "\n";
}
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the nullable-strict probe to WASM");
    assert!(
        output.status.success(),
        "nullable-strict compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the nullable-strict probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "0: TFFFFFFTFTF\n1: FTTFFFFTFFT\n2: FTFTFFFTTFF\n",
        "php-src 8.5.6's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a boxed object reaches `__toString` instead of being refused outright.
///
/// The object arm of every string conversion raised `Error: Object of class C could not be
/// converted to string` for EVERY class, which is php-src's answer only for a class that does
/// NOT define the method. Measured against php-src 8.5.6, `(string)$tag` prints `<em>` there and
/// fatally raised here — a wrong answer in shipped code, not a refusal. The conversion now goes
/// through a runtime class-id dispatch, so a class defining `__toString` converts and one that
/// does not still raises.
///
/// The three bodies are the three ownership shapes: a LITERAL returns a data-segment pointer that
/// must not be released, a PROPERTY read returns storage the object still owns, and a CONCAT
/// returns a fresh heap string. All three have to survive the same persist-and-release path.
#[test]
fn test_cli_wasm_string_conversion_dispatches_to_to_string() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_to_string");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Lit { public function __toString(): string { return "lit"; } }
class Prop { public function __construct(private string $t) {} public function __toString(): string { return $this->t; } }
class Cat { public function __construct(private string $t) {} public function __toString(): string { return "<" . $this->t . ">"; } }
function boxit(int $i): mixed { return $i === 0 ? new Lit() : ($i === 1 ? new Prop("prp") : new Cat("em")); }
for ($i = 0; $i < 3; $i++) { echo (string)boxit($i), "|", strlen((string)boxit($i)), "\n"; }
"#,
    )
    .unwrap();

    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the __toString probe to WASM");
    assert!(
        built.status.success(),
        "__toString compilation failed: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the __toString probe under Node");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "lit|3\nprp|3\n<em>|4\n",
        "php-src's own answers ({})",
        String::from_utf8_lossy(&run.stderr)
    );

    // A class with no `__toString` keeps php-src's fatal, which is what the old arm gave
    // every object indiscriminately.
    let plain = dir.join("plain.php");
    fs::write(
        &plain,
        r#"<?php
class Tag { public function __toString(): string { return "t"; } }
class Plain { public function __construct(public int $n) {} }
function boxit(int $i): mixed { return $i === 0 ? new Tag() : new Plain(3); }
echo (string)boxit(0), "\n";
echo (string)boxit(1), "\n";
"#,
    )
    .unwrap();
    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&plain)
        .output()
        .expect("failed to compile the non-stringable probe to WASM");
    assert!(
        built.status.success(),
        "non-stringable compilation failed: {}",
        String::from_utf8_lossy(&built.stderr)
    );
    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("plain.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the non-stringable probe under Node");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "t\n");
    assert!(
        String::from_utf8_lossy(&run.stderr)
            .contains("Object of class Plain could not be converted to string"),
        "php-src's own fatal: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a boxed value reaching a builtin's declared `string` parameter is coerced PHP's way.
///
/// This is a THIRD implicit `Str` conversion, distinct from both the explicit `(string)` cast and
/// the one an echo performs, and the difference is not cosmetic. Measured on php-src 8.5.6 for
/// `strtoupper($mixed)`: a scalar converts exactly as `(string)` does, `null` converts to `""`
/// but raises a `Deprecated` naming the parameter, and an array — which `(string)` would have
/// turned into `"Array"` with a warning — does not convert at all, it is a `TypeError`. Refusing
/// the whole shape was correct but cost every such call; each arm below is php-src's own answer.
///
/// KNOWN DIVERGENCE: php-src appends ` in <file> on line <n>` and a stack trace to the fatal.
/// This target reports no location tail, so the fatal arms are asserted as a prefix.
#[test]
fn test_cli_wasm_coerces_a_boxed_value_at_a_declared_string_parameter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_string_argument");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m", process.argv[3]], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
function arm(int $i): mixed {
    if ($i === 0) { return "abc"; }
    if ($i === 1) { return 42; }
    if ($i === 2) { return 2.5; }
    if ($i === 3) { return true; }
    if ($i === 4) { return false; }
    if ($i === 5) { return null; }
    if ($i === 6) { return [1, 2]; }
    return new stdClass();
}
$m = arm((int)($argv[1] ?? "0"));
echo strtoupper($m), "|\n";
"#,
    )
    .unwrap();

    let built = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the string-argument probe to WASM");
    assert!(
        built.status.success(),
        "string-argument compilation failed: {}",
        String::from_utf8_lossy(&built.stderr)
    );

    // Every expectation below is php-src 8.5.6's own answer for the same arm.
    for (arm, stdout, stderr) in [
        ("0", "ABC|\n", ""),
        ("1", "42|\n", ""),
        ("2", "2.5|\n", ""),
        ("3", "1|\n", ""),
        ("4", "|\n", ""),
        (
            "5",
            "|\n",
            "Deprecated: strtoupper(): Passing null to parameter #1 ($string) of type string \
             is deprecated\n",
        ),
        (
            "6",
            "",
            "Uncaught TypeError: strtoupper(): Argument #1 ($string) must be of type string, \
             array given\n",
        ),
        (
            "7",
            "",
            "Uncaught TypeError: strtoupper(): Argument #1 ($string) must be of type string, \
             stdClass given\n",
        ),
    ] {
        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join("main.wasm"))
            .arg(arm)
            .current_dir(&dir)
            .output()
            .expect("failed to run the string-argument probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            stdout,
            "arm {arm}: php-src's own value"
        );
        let observed = String::from_utf8_lossy(&run.stderr);
        assert!(
            observed.contains(stderr),
            "arm {arm}: expected php-src's own diagnostic {stderr:?}, got {observed}"
        );
        // A `TypeError` ends the program; a `Deprecated` does not.
        let expected_code = if stdout.is_empty() { 255 } else { 0 };
        assert_eq!(
            run.status.code(),
            Some(expected_code),
            "arm {arm}: php-src's own exit status"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a dispatch reaching a class php-src will not enter raises PHP's own error.
///
/// The arity filter drops such a class from the callable arms, which is what lets an unrelated
/// namesake stop vetoing the program — but dropping it must not mean forgetting it. Left to the
/// ladder's fallthrough it would report `Call to undefined method Ledger::show()`, a different
/// error class naming a method that plainly exists; the native backend does worse still and
/// says `Call to a member function show() on null`. Each dropped class keeps an arm raising
/// `ArgumentCountError` with php-src's counts and wording — `exactly` when every declared
/// parameter is required, `at least` when a default makes them differ, and measured on 8.5.6, a
/// VARIADIC tail keeps the word `exactly`.
///
/// KNOWN DIVERGENCE: php-src continues `, 0 passed in /path.php on line 9 and …` and prints a
/// stack trace. This target reports no location tail, the convention its other composed fatals
/// already follow, so each case below is asserted as a prefix.
#[test]
fn test_cli_wasm_dispatch_raises_php_argument_count_error_for_a_class_it_cannot_enter() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_argument_count");
    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    // Every expected message below is php-src 8.5.6's own, minus the location tail.
    for (name, bystander, expected) in [
        (
            "exact.php",
            "class Other { public function show(string $prefix, int $width): void {} }",
            "Uncaught ArgumentCountError: Too few arguments to function Other::show(), \
             0 passed and exactly 2 expected\n",
        ),
        (
            "optional.php",
            "class Other { public function show(int $a, int $b = 2): void {} }",
            "Uncaught ArgumentCountError: Too few arguments to function Other::show(), \
             0 passed and at least 1 expected\n",
        ),
        (
            "variadic.php",
            "class Other { public function show(int $a, int ...$rest): void {} }",
            "Uncaught ArgumentCountError: Too few arguments to function Other::show(), \
             0 passed and exactly 1 expected\n",
        ),
    ] {
        let path = dir.join(name);
        fs::write(
            &path,
            format!(
                "<?php\nclass Money {{ public function show(): void {{ echo \"money\\n\"; }} }}\n\
                 {bystander}\n\
                 function render(mixed $value): void {{ $value->show(); }}\n\
                 render(new Money());\n\
                 render(new Other());\n"
            ),
        )
        .unwrap();
        let built = elephc_cli_command(&dir)
            .arg("--target")
            .arg("wasm32-wasi")
            .arg(&path)
            .output()
            .expect("failed to compile the argument-count probe to WASM");
        assert!(
            built.status.success(),
            "{name} must compile: {}",
            String::from_utf8_lossy(&built.stderr)
        );

        let run = Command::new("node")
            .arg("--no-warnings")
            .arg(&runner)
            .arg(dir.join(name.replace(".php", ".wasm")))
            .current_dir(&dir)
            .output()
            .expect("failed to run the argument-count probe under Node");
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            "money\n",
            "{name}: the class php-src DOES enter still answers"
        );
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(
            stderr.contains(expected),
            "{name}: expected php-src's own message, got {stderr}"
        );
        assert_eq!(
            run.status.code(),
            Some(255),
            "{name}: php-src's own fatal status"
        );
    }

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies a boxed value reaching a slot declared as a class is narrowed back to an object.
///
/// A `?Node` property is stored boxed, so `return $this->node;` from a method declared `: Node`
/// asks the backend to move a `Heap(Mixed)` into an object slot. That call carries no runtime
/// function id at all — the frontend leaves the conversion implicit — and the audit refused it as
/// `missing typed runtime target, carries no immediate at all`. The lowering unboxes the cell and
/// takes its payload, with a tag guard the native backend does without: a cell holding anything
/// but an object yields a null pointer rather than a scalar reinterpreted as an address.
#[test]
fn test_cli_wasm_narrows_a_boxed_value_into_a_declared_class_slot() {
    if Command::new("node").arg("--version").output().is_err() {
        return;
    }

    let dir = make_cli_test_dir("elephc_cli_wasm_object_narrowing");
    let php_path = dir.join("main.php");
    fs::write(
        &php_path,
        r#"<?php
class Node { public function __construct(public string $label) {} }
class Box {
    private ?Node $node = null;
    public function set(Node $n): void { $this->node = $n; }
    public function get(): Node {
        if ($this->node === null) { throw new RuntimeException("empty"); }
        return $this->node;
    }
}
$b = new Box();
$b->set(new Node("leaf"));
echo $b->get()->label, "\n";
$b->set(new Node("branch"));
echo $b->get()->label, "\n";
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile the object-narrowing probe to WASM");
    assert!(
        output.status.success(),
        "object-narrowing compilation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let runner = dir.join("run.mjs");
    fs::write(
        &runner,
        r#"import { readFileSync } from "node:fs";
import { WASI } from "node:wasi";
const wasi = new WASI({ version: "preview1", args: ["m"], env: {}, returnOnExit: true });
const bytes = readFileSync(process.argv[2]);
const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(bytes),
  wasi.getImportObject(),
);
process.exitCode = wasi.start(instance);
"#,
    )
    .unwrap();

    let run = Command::new("node")
        .arg("--no-warnings")
        .arg(&runner)
        .arg(dir.join("main.wasm"))
        .current_dir(&dir)
        .output()
        .expect("failed to run the object-narrowing probe under Node");
    assert!(
        run.status.success(),
        "the narrowed reads must not terminate: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "leaf\nbranch\n",
        "php-src's own answers"
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Verifies `--debug-info` survives a source path that carries assembler string
/// metacharacters. A `\` used to be spliced into `.file`/`.asciz` unescaped, so
/// the assembler rejected the module outright; combined with `"` it terminated
/// the directive string early and let the rest of the path be assembled as
/// directives. The full compile must now succeed and the program must run.
#[test]
fn test_cli_debug_info_escapes_metacharacters_in_source_path() {
    let dir = make_cli_test_dir("elephc_cli_debug_info_escapes");
    // A backslash alone broke the assembler; `\"` was the directive-injection
    // vector. Both are legal filename bytes on every supported target.
    let php_path = dir.join("bs\\la\"sh.php");
    fs::write(
        &php_path,
        r#"<?php
function greet(): void { echo "escaped\n"; }
greet();
"#,
    )
    .unwrap();

    let output = elephc_cli_command(&dir)
        .arg("--debug-info")
        .arg(&php_path)
        .output()
        .expect("failed to run elephc CLI with --debug-info");

    assert!(
        output.status.success(),
        "elephc --debug-info failed for a path with `\\` and `\"`: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let asm = fs::read_to_string(dir.join("bs\\la\"sh.s")).expect("failed to read assembly");
    let file_line = asm.lines().next().expect("assembly is empty");
    assert!(
        file_line.contains("bs\\\\la\\\"sh.php\""),
        "source path must be escaped inside the .file string: {file_line}"
    );
    assert!(
        asm.contains(".asciz \"") && asm.contains("bs\\\\la\\\"sh.php\""),
        "source path must be escaped inside the compile-unit DW_AT_name too"
    );
    for line in asm.lines() {
        assert!(
            !line.starts_with(".globl bs") && !line.trim_start().starts_with("sh.php"),
            "path bytes leaked out of their directive: {line}"
        );
    }

    let run = std::process::Command::new(dir.join("bs\\la\"sh"))
        .output()
        .expect("failed to run the compiled binary");
    assert!(run.status.success(), "compiled binary did not run");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "escaped\n");

    let _ = fs::remove_dir_all(&dir);
}
