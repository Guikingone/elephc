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
$f = function(int $x): int { return $x + 1; };
echo $f(41);
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

/// Compiles an escaping by-ref closure from PHP source to wasm32-wasi and runs it
/// twice under Wasmer. The creator's frame is gone before either call, so `23`
/// proves the closure owns the ref cell instead of dereferencing freed storage.
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
    $x = 1;
    return function() use (&$x) {
        return ++$x;
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

    let _ = fs::remove_dir_all(&dir);
}

/// Compiles typed int/bool/string indexed reads from PHP through EIR to WASM and
/// executes them under Wasmer. Negative/OOB reads remain null through `is_null`
/// and `echo`; the former integer sentinel remains a valid in-range value.
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

    let output = elephc_cli_command(&dir)
        .arg("--target")
        .arg("wasm32-wasi")
        .arg(&php_path)
        .output()
        .expect("failed to compile indexed-array OOB fixture to WASM");
    assert!(
        output.status.success(),
        "indexed-array OOB compilation failed: {}",
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
        "indexed-array OOB fixture trapped: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "1:;1:;:9223372036854775806;1:;1:;:;1:;1:;\
0,,0;0,;0,,;"
    );

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
