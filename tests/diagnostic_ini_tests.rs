//! Purpose:
//! Native CLI regressions for startup warning display and logging destinations.
//!
//! Called from:
//! - `cargo test --test diagnostic_ini_tests`.
//!
//! Key details:
//! - Separate process streams stay raw; display/log suppression must not suppress user handlers.
//! - Fresh executables share the runtime cache to catch accidental configuration baking.

use std::{fs, path::PathBuf, process::{Command, Output}};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Compile and execute an isolated source with startup INI values and retain raw streams.
fn run(source: &str, ini: &[&str]) -> Output {
    let dir = std::env::temp_dir().join(format!("elephc_diag_ini_{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.php");
    fs::write(&file, source).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_elephc").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().unwrap(); path.pop(); path.pop();
        path.join("elephc").to_string_lossy().into_owned()
    });
    let mut cmd = Command::new(binary);
    cmd.arg("--quiet").arg(&file);
    for value in ini { cmd.arg("--ini").arg(value); }
    let compile = cmd.output().unwrap();
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    let out = Command::new(PathBuf::from(&dir).join("main")).output().unwrap();
    fs::remove_dir_all(dir).unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out
}

const SOURCE: &str = "<?php trigger_error('user-warning', E_USER_WARNING); var_dump(fopen(__DIR__ . '/absent', 'r'));";

/// log_errors=0 leaves one stdout display and removes both user and ordinary stderr copies.
#[test]
fn warnings_follow_stdout_without_a_log_copy() {
    let out = run(SOURCE, &["display_errors=1", "log_errors=0"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(text.matches("user-warning").count(), 1, "{text}");
    assert!(text.contains("Warning:"), "{text}");
    assert!(text.contains("bool(false)"), "{text}");
    assert!(out.stderr.is_empty(), "{}", String::from_utf8_lossy(&out.stderr));
}

/// display_errors=stderr and its numeric alias route displays outside output buffers.
#[test]
fn stderr_display_bypasses_stdout_buffers() {
    for mode in ["stderr", "STDERR", "2", "258"] {
        let out = run("<?php ob_start(); trigger_error('user-warning', E_USER_WARNING); echo 'buffered'; $b=ob_get_clean(); echo $b;", &[&format!("display_errors={mode}"), "log_errors=0"]);
        assert_eq!(out.stdout, b"buffered");
        let text = String::from_utf8_lossy(&out.stderr);
        assert_eq!(text.matches("user-warning").count(), 1, "{text}");
        assert!(text.contains("on line 1"), "{text}");
    }
}

/// Disabling both default destinations still invokes registered PHP error handlers.
#[test]
fn hidden_displays_do_not_hide_user_handlers() {
    let out = run("<?php set_error_handler(function ($n, $s) { echo 'handled'; return false; }); trigger_error('user-warning', E_USER_WARNING);", &["display_errors=0", "log_errors=0"]);
    assert_eq!(out.stdout, b"handled");
    assert!(out.stderr.is_empty());
}

/// Startup overrides are last-wins and each executable seeds its own cached-runtime destinations.
#[test]
fn startup_destinations_are_per_program_and_last_wins() {
    let off = run(SOURCE, &["display_errors=0", "log_errors=0"]);
    assert_eq!(off.stdout, b"bool(false)\n");
    assert!(off.stderr.is_empty());
    let on = run(SOURCE, &["display_errors=0", "display_errors=stdout", "log_errors=1", "log_errors=0"]);
    assert_eq!(String::from_utf8_lossy(&on.stdout).matches("user-warning").count(), 1);
    assert!(on.stderr.is_empty());
    let log = run("<?php trigger_error('user-warning', E_USER_WARNING);", &["display_errors=0", "log_errors=1"]);
    assert!(log.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&log.stderr).matches("user-warning").count(), 1);
}
