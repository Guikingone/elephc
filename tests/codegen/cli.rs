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
        // The explicit `(int)` cast now carries its exact PHP 8.5 diagnostic and is
        // admitted, so this fixture no longer produces a float-to-int issue. The
        // implicit coercion stays rejected but the checker refuses it earlier, so no
        // PHP source in this fixture can reach that capability message.
        assert!(
            stderr.contains(
                "float or Mixed truthiness requires exact profile-specific NAN diagnostics"
            ),
            "PHP {version}: {stderr}"
        );
        assert!(
            stderr
                .matches(
                    "float or Mixed truthiness requires exact profile-specific NAN diagnostics"
                )
                .count()
                >= 6,
            "PHP {version}: constant NAN truthiness was optimized away: {stderr}"
        );
        assert!(
            stderr.contains(
                "implicit Mixed-to-scalar transfer requires exact per-tag PHP diagnostics"
            ),
            "PHP {version}: {stderr}"
        );
        assert!(
            stderr.contains(
                "Mixed-to-scalar casts require exact per-tag PHP values and diagnostics"
            ),
            "PHP {version}: {stderr}"
        );
        assert!(
            stderr.contains("ref-cell store value"),
            "PHP {version}: checked arithmetic must not narrow an escaping ref cell: {stderr}"
        );
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
            r#"<?php class C { public string $name = "x"; } echo (new C())->name;"#,
            "non-scalar default unsupported by WASM object construction",
        ),
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
            r#"<?php function m(): mixed { return 1; } $a = [m()]; foreach ($a as $v) { echo $v; }"#,
            "indexed foreach element Mixed has no exact WASM load contract",
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
